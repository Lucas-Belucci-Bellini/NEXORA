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
use std::time::{Duration, Instant};

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
    /// Nothing was shown: the window is not taking frames yet (on macOS, a
    /// window that is not visible yet cannot be drawn to). Not counted as a
    /// frame; the host asks again, up to [`SHOW_TIMEOUT`] before any frame.
    Wait,
    /// Close the window and return from [`run`].
    Exit,
}

/// How long the host sleeps before asking again after [`Flow::Wait`].
const WAIT_STEP: Duration = Duration::from_millis(10);

/// How long the host waits, after opening the window, for the client's
/// first frame before it gives up and says the window never became visible.
pub const SHOW_TIMEOUT: Duration = Duration::from_secs(20);

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
    /// Redraws on which the window was not taking frames yet ([`Flow::Wait`]).
    pub waited: u64,
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
        opened_at: None,
        resume_at: None,
        frames: 0,
        waited: 0,
        closed: false,
        exiting: false,
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
        waited: host.waited,
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
    opened_at: Option<Instant>,
    /// After [`Flow::Wait`]: when to ask for the next frame.
    resume_at: Option<Instant>,
    frames: u64,
    waited: u64,
    closed: bool,
    /// The run is over: `exit` has been asked for. The platform may still
    /// deliver events already queued (macOS delivers a pending redraw), and
    /// none of them may reach the client, which has finished.
    exiting: bool,
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
        self.opened_at = Some(Instant::now());
        self.facts = Some(facts);
        self.rhi = Some(rhi);
        self.window = Some(window);
        Ok(())
    }

    fn exit(&mut self, event_loop: &ActiveEventLoop) {
        self.exiting = true;
        event_loop.exit();
    }

    /// End the run with `error`. The first failure is the one reported.
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Error) {
        if self.failure.is_none() {
            self.failure = Some(error);
        }
        self.exit(event_loop);
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
        if self.exiting {
            return;
        }
        let Some(rhi) = self.rhi.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                self.closed = true;
                self.exit(event_loop);
            }
            WindowEvent::Resized(size) => {
                if let Err(error) = rhi.resize(size.width, size.height) {
                    self.fail(event_loop, error);
                }
            }
            WindowEvent::RedrawRequested => match self.client.frame(rhi) {
                Ok(Flow::Continue) => {
                    self.frames += 1;
                    event_loop.set_control_flow(ControlFlow::Poll);
                }
                Ok(Flow::Wait) => {
                    // Nothing to draw into: ask again shortly, not in a spin.
                    self.waited += 1;
                    let resume = Instant::now() + WAIT_STEP;
                    self.resume_at = Some(resume);
                    event_loop.set_control_flow(ControlFlow::WaitUntil(resume));
                }
                Ok(Flow::Exit) => {
                    self.frames += 1;
                    self.exit(event_loop);
                }
                Err(error) => self.fail(event_loop, error),
            },
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.exiting {
            return;
        }
        let Some(window) = &self.window else {
            return;
        };
        if let Some(opened_at) = self.opened_at {
            if self.frames == 0 && opened_at.elapsed() > SHOW_TIMEOUT {
                let error = Error::new(
                    Domain::Platform,
                    "window",
                    "the window never became visible: no frame could be shown",
                )
                .with_recovery(Recovery::Retry)
                .with_context("platform", platform())
                .with_context("waited_redraws", self.waited.to_string())
                .with_context("timeout_s", SHOW_TIMEOUT.as_secs().to_string());
                self.fail(event_loop, error);
                return;
            }
        }
        if self.resume_at.is_some_and(|resume| Instant::now() < resume) {
            return;
        }
        self.resume_at = None;
        // Every pass of the loop asks for the next frame. Asking here rather
        // than after a frame keeps asking while the window is not visible,
        // which is when a platform may skip a redraw.
        window.request_redraw();
    }
}
