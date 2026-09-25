//! The window host (ADR-0027): one window, the event loop that owns it, and
//! the native RHI presenting to it.
//!
//! Whoever owns the window owns the event loop, and the event loop owns the
//! thread it runs on: on macOS that must be the main thread. So the host does
//! not hand out a window to draw into. It runs the loop and calls a
//! [`Client`] back: once when the window and its device are ready, then once
//! per frame. The frame clock (DEBT-0041) and real input devices (DEBT-0043)
//! belong here as well, and neither exists yet.
//!
//! `winit` is the only crate here that knows about windows; the RHI receives
//! the window as a surface target and never names `winit` (ADR-0027).
//!
//! # No display is a failure, not a skip
//!
//! As with the GPU (ADR-0026): if no window can be opened, [`run`] fails with
//! [`Recovery::DisableSubsystem`], and callers that are allowed to run without
//! a display say so with `NEXORA_DISPLAY=none`.

pub mod probe;

use std::sync::Arc;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_rhi_wgpu::WgpuRhi;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// The window to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSpec {
    /// Its title.
    pub title: String,
    /// Its inner width in physical pixels, as requested; the window system
    /// may give another.
    pub width: u32,
    /// Its inner height in physical pixels, as requested.
    pub height: u32,
}

/// What a client wants after a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    /// Draw another frame.
    Continue,
    /// Close the window and return from [`run`].
    Exit,
}

/// What runs inside the window. The host calls it on the event loop's thread.
pub trait Client {
    /// The window is open and `rhi` presents to it.
    ///
    /// # Errors
    ///
    /// Any error ends the run, and [`run`] returns it.
    fn opened(&mut self, rhi: &mut WgpuRhi, window: &WindowFacts) -> Result<()>;

    /// Draw and present one frame.
    ///
    /// # Errors
    ///
    /// Any error ends the run, and [`run`] returns it.
    fn frame(&mut self, rhi: &mut WgpuRhi) -> Result<Flow>;
}

/// The window as opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowFacts {
    /// Inner width in physical pixels.
    pub width: u32,
    /// Inner height in physical pixels.
    pub height: u32,
    /// The window system: `x11`, `win32` or `appkit`.
    pub platform: &'static str,
}

/// How a run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// The window as it was opened.
    pub window: WindowFacts,
    /// Frames the client completed.
    pub frames: u64,
    /// Whether the window was closed from outside (the user, the window
    /// system) rather than by the client.
    pub closed: bool,
}

/// The window system this build talks to.
#[must_use]
pub const fn platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "macos") {
        "appkit"
    } else {
        "x11"
    }
}

/// Open the window, run `client` in it until it exits or the window closes,
/// and return how it went. Must be called from the main thread.
///
/// # Errors
///
/// No display could be opened, the window or its device could not be created,
/// or the client failed.
pub fn run(spec: &WindowSpec, client: &mut dyn Client) -> Result<Session> {
    let event_loop = EventLoop::new().map_err(|error| {
        Error::new(
            Domain::Platform,
            "window",
            "no display: the event loop did not start",
        )
        .with_recovery(Recovery::DisableSubsystem)
        .with_context("platform", platform())
        .with_context("cause", error.to_string())
    })?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut host = Host {
        spec,
        client,
        window: None,
        rhi: None,
        facts: None,
        frames: 0,
        closed: false,
        failure: None,
    };
    event_loop.run_app(&mut host).map_err(|error| {
        Error::new(Domain::Platform, "window", "the event loop failed")
            .with_recovery(Recovery::DisableSubsystem)
            .with_context("cause", error.to_string())
    })?;
    if let Some(error) = host.failure {
        return Err(error);
    }
    let window = host.facts.ok_or_else(|| {
        Error::new(
            Domain::Platform,
            "window",
            "the event loop ended before a window opened",
        )
        .with_recovery(Recovery::DisableSubsystem)
    })?;
    Ok(Session {
        window,
        frames: host.frames,
        closed: host.closed,
    })
}

struct Host<'a> {
    spec: &'a WindowSpec,
    client: &'a mut dyn Client,
    // Field order is drop order: the device and its surface go before the
    // window they draw into.
    rhi: Option<WgpuRhi>,
    window: Option<Arc<Window>>,
    facts: Option<WindowFacts>,
    frames: u64,
    closed: bool,
    failure: Option<Error>,
}

impl Host<'_> {
    fn open(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let attributes = Window::default_attributes()
            .with_title(self.spec.title.clone())
            .with_inner_size(PhysicalSize::new(self.spec.width, self.spec.height));
        let window = Arc::new(event_loop.create_window(attributes).map_err(|error| {
            Error::new(
                Domain::Platform,
                "window",
                "the window system refused a window",
            )
            .with_recovery(Recovery::DisableSubsystem)
            .with_context("cause", error.to_string())
        })?);
        let size = window.inner_size();
        let facts = WindowFacts {
            width: size.width,
            height: size.height,
            platform: platform(),
        };
        let mut rhi = WgpuRhi::with_surface(Arc::clone(&window), size.width, size.height)?;
        self.client.opened(&mut rhi, &facts)?;
        window.request_redraw();
        self.facts = Some(facts);
        self.rhi = Some(rhi);
        self.window = Some(window);
        Ok(())
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Error) {
        self.failure = Some(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for Host<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() || self.failure.is_some() {
            return;
        }
        if let Err(error) = self.open(event_loop) {
            self.fail(event_loop, error);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(rhi) = self.rhi.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                self.closed = true;
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Err(error) = rhi.resize(size.width, size.height) {
                    self.fail(event_loop, error);
                }
            }
            WindowEvent::RedrawRequested => match self.client.frame(rhi) {
                Ok(Flow::Continue) => {
                    self.frames += 1;
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                Ok(Flow::Exit) => {
                    self.frames += 1;
                    event_loop.exit();
                }
                Err(error) => self.fail(event_loop, error),
            },
            _ => {}
        }
    }
}
