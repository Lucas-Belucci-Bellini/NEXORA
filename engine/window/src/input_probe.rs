//! Evidence that a real key reaches the engine as intent (ADR-0031,
//! DEBT-0043).
//!
//! The probe opens a window titled [`TITLE`], binds `W` to one action, and
//! waits until that action has been pressed and released: the key has come
//! from a keyboard, through the operating system and the window system, into
//! `winit`, through the host's translation into a signal, and through
//! `nexora_runtime::input` into an action. Nothing on the way is simulated
//! by the engine. In CI the key comes from the X server's test extension
//! (`xdotool`), which reaches the window the way a keyboard does. On the
//! operator's machine it comes from a person pressing `W`.

use std::time::{Duration, Instant};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_rhi::{Command, CommandList, Rhi, TextureDesc, TextureFormat, TextureHandle, Usage};
use nexora_rhi_wgpu::WgpuRhi;
use nexora_runtime::input::{
    ActionDefinition, ActionKind, Binding, InputFrame, InputSystem, Source,
};
use winit::keyboard::KeyCode;

use crate::input::{key_usage, InputCounts, KEYBOARD};
use crate::{run, Client, Flow, WindowFacts, WindowSpec};

/// The window's title, which tells a person what to do and lets a test tool
/// find the window.
pub const TITLE: &str = "NEXORA input probe: press W";

/// The key the probe waits for.
pub const KEY: KeyCode = KeyCode::KeyW;

/// What the probe observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputReport {
    /// The key's HID usage, as it reached the engine.
    pub usage: u16,
    /// The action it was bound to.
    pub action: String,
    /// Frames shown before the key went down.
    pub pressed_after: u64,
    /// Frames the key was held.
    pub held_for: u64,
    /// What the host did with device events.
    pub counts: InputCounts,
}

/// Open the window and wait up to `timeout` for `W` to be pressed and
/// released.
///
/// # Errors
///
/// No window could be opened, or the key did not arrive in time
/// ([`Recovery::Retry`]: press it next time).
pub fn run_input_probe(timeout: Duration) -> Result<InputReport> {
    let action = Identifier::nexora("action/probe")?;
    let context = Identifier::nexora("context/probe")?;
    let usage = key_usage(KEY).ok_or_else(|| wrong("the probe's key has no HID usage"))?;
    let mut system = InputSystem::new();
    system.register_action(ActionDefinition::new(action.clone(), ActionKind::Button))?;
    system.set_context(context.clone(), 0);
    system.bind(Binding::new(
        context,
        action.clone(),
        Source::button(KEYBOARD.kind(), usage),
    ))?;
    let mut probe = InputProbe {
        system,
        action: action.clone(),
        timeout,
        started: Instant::now(),
        target: None,
        frames: 0,
        pressed_at: None,
        released_at: None,
        pending: InputFrame::new(),
    };
    let spec = WindowSpec {
        title: TITLE.into(),
        width: 256,
        height: 128,
    };
    let session = run(&spec, &mut probe)?;
    match (probe.pressed_at, probe.released_at) {
        (Some(pressed), Some(released)) => Ok(InputReport {
            usage: usage.0,
            action: action.to_string(),
            pressed_after: pressed,
            held_for: released - pressed,
            counts: session.input,
        }),
        (pressed, _) => Err(Error::new(
            Domain::Input,
            "input-probe",
            "no key press and release reached the engine in time",
        )
        .with_recovery(Recovery::Retry)
        .with_context("pressed", pressed.is_some().to_string())
        .with_context("timeout_s", timeout.as_secs().to_string())
        .with_context("signals", session.input.delivered.to_string())),
    }
}

struct InputProbe {
    system: InputSystem,
    action: Identifier,
    timeout: Duration,
    started: Instant,
    target: Option<TextureHandle>,
    frames: u64,
    pressed_at: Option<u64>,
    released_at: Option<u64>,
    pending: InputFrame,
}

impl Client for InputProbe {
    fn opened(&mut self, rhi: &mut WgpuRhi, window: &WindowFacts) -> Result<()> {
        // Something to show while waiting: a flat target the window's size.
        let target = rhi.create_texture(&TextureDesc {
            label: "input probe".into(),
            width: window.width,
            height: window.height,
            format: TextureFormat::Rgba8Unorm,
            usage: Usage::RENDER_TARGET,
        })?;
        let mut clear = CommandList::new("input probe clear");
        clear.push(Command::Clear {
            texture: target,
            value: nexora_rhi::ClearValue::Color([0.1, 0.1, 0.3, 1.0]),
        });
        let fence = rhi.submit(clear)?;
        rhi.wait(fence)?;
        self.target = Some(target);
        self.started = Instant::now();
        Ok(())
    }

    fn input(&mut self, frame: &InputFrame) {
        for signal in frame.signals() {
            self.pending.push(*signal);
        }
    }

    fn frame(&mut self, rhi: &mut WgpuRhi) -> Result<Flow> {
        let Some(target) = self.target else {
            return Ok(Flow::Exit);
        };
        match rhi.present(target) {
            Ok(()) => {}
            Err(error) if error.recovery() == Recovery::Retry => return Ok(Flow::Wait),
            Err(error) => return Err(error),
        }
        // Sample once per shown frame, as a client does.
        let snapshot = self.system.sample(&std::mem::take(&mut self.pending));
        if self.pressed_at.is_none() && snapshot.just_pressed(&self.action) {
            self.pressed_at = Some(self.frames);
        }
        if self.pressed_at.is_some() && snapshot.just_released(&self.action) {
            self.released_at = Some(self.frames);
        }
        self.frames += 1;
        let done = self.released_at.is_some() || self.started.elapsed() > self.timeout;
        if !done {
            return Ok(Flow::Continue);
        }
        self.target = None;
        rhi.destroy_texture(target)?;
        Ok(Flow::Exit)
    }
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Input, "input-probe", message).with_recovery(Recovery::Manual)
}
