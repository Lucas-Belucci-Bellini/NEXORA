//! What the keys mean in the client, and what one fixed step of that
//! meaning does to the camera.
//!
//! The bindings are data in the input system's own terms: an action, a
//! context and a keyboard button numbered by its USB HID usage (ADR-0031).
//! Nothing here reads a device or a clock. [`Controls::sample`] folds one
//! frame of signals into an [`Intent`], and [`step`] applies an intent for
//! one fixed step of the frame loop, so the same signals and the same step
//! count move the camera to the same place on any machine.

use std::f64::consts::FRAC_PI_2;
use std::time::Duration;

use nexora_camera::Camera;
use nexora_foundation::error::Result;
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::WorldPosition;
use nexora_runtime::input::Binding;
use nexora_runtime::input::{
    ActionDefinition, ActionKind, ButtonCode, DeviceKind, InputFrame, InputSystem, Source,
};

/// How fast the camera moves, in blocks per second of simulated time.
pub const MOVE_SPEED: f64 = 8.0;

/// How fast the camera turns, in radians per second of simulated time.
pub const TURN_SPEED: f64 = FRAC_PI_2;

/// The context every client binding lives in.
pub const CONTEXT: &str = "context/client";

/// The client's actions and the key each is bound to by default, as a HID
/// Keyboard/Keypad usage (page 0x07).
pub const DEFAULT_KEYS: [(&str, u16); 11] = [
    ("action/move_forward", 0x1A), // W
    ("action/move_back", 0x16),    // S
    ("action/move_left", 0x04),    // A
    ("action/move_right", 0x07),   // D
    ("action/move_up", 0x2C),      // Space
    ("action/move_down", 0xE1),    // Left Shift
    ("action/turn_left", 0x50),    // Left arrow
    ("action/turn_right", 0x4F),   // Right arrow
    ("action/look_up", 0x52),      // Up arrow
    ("action/look_down", 0x51),    // Down arrow
    ("action/exit", 0x29),         // Escape
];

/// What the player asked for this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Intent {
    /// `+1` forward, `-1` back, `0` both or neither.
    pub forward: f64,
    /// `+1` right, `-1` left.
    pub right: f64,
    /// `+1` up, `-1` down.
    pub up: f64,
    /// `+1` turn left (counter-clockwise seen from above), `-1` right.
    pub turn: f64,
    /// `+1` look up, `-1` down.
    pub look: f64,
    /// The exit action went down this frame.
    pub exit: bool,
    /// How many of the client's actions are held.
    pub held: u32,
}

impl Intent {
    /// Whether this intent moves or turns the camera.
    #[must_use]
    pub fn moves(&self) -> bool {
        [self.forward, self.right, self.up, self.turn, self.look]
            .iter()
            .any(|axis| *axis != 0.0)
    }
}

/// The input system, loaded with the client's actions and bindings.
#[derive(Debug)]
pub struct Controls {
    system: InputSystem,
    actions: Vec<Identifier>,
}

impl Controls {
    /// Register the client's actions in one context, bound to
    /// [`DEFAULT_KEYS`].
    ///
    /// # Errors
    ///
    /// An identifier or a binding was refused by the input system.
    pub fn new() -> Result<Self> {
        let context = Identifier::nexora(CONTEXT)?;
        let mut system = InputSystem::new();
        system.set_context(context.clone(), 0);
        let mut actions = Vec::with_capacity(DEFAULT_KEYS.len());
        for (path, usage) in DEFAULT_KEYS {
            let action = Identifier::nexora(path)?;
            system.register_action(ActionDefinition::new(action.clone(), ActionKind::Button))?;
            system.bind(Binding::new(
                context.clone(),
                action.clone(),
                Source::button(DeviceKind::Keyboard, ButtonCode(usage)),
            ))?;
            actions.push(action);
        }
        Ok(Self { system, actions })
    }

    /// Fold one frame of signals into what the player asked for.
    pub fn sample(&mut self, frame: &InputFrame) -> Intent {
        let snapshot = self.system.sample(frame);
        let held = |index: usize| snapshot.is_pressed(&self.actions[index]);
        let axis = |plus: usize, minus: usize| {
            f64::from(u8::from(held(plus))) - f64::from(u8::from(held(minus)))
        };
        Intent {
            forward: axis(0, 1),
            right: axis(3, 2),
            up: axis(4, 5),
            turn: axis(6, 7),
            look: axis(8, 9),
            exit: snapshot.just_pressed(&self.actions[10]),
            held: (0..self.actions.len()).filter(|index| held(*index)).count() as u32,
        }
    }
}

/// Apply `intent` to `camera` for one fixed step.
///
/// Movement is in the horizontal plane the camera faces, whatever its pitch,
/// plus straight up or down, so looking down does not slow walking forward.
///
/// # Errors
///
/// The camera would leave the finite world; it does not move.
pub fn step(camera: &mut Camera, intent: Intent, step: Duration) -> Result<()> {
    let seconds = step.as_secs_f64();
    if intent.turn != 0.0 || intent.look != 0.0 {
        camera.set_orientation(
            camera.yaw() + intent.turn * TURN_SPEED * seconds,
            camera.pitch() + intent.look * TURN_SPEED * seconds,
        )?;
    }
    let (sin_yaw, cos_yaw) = camera.yaw().sin_cos();
    // Forward in the horizontal plane, and right of it (forward x up).
    let forward = [-sin_yaw, -cos_yaw];
    let right = [cos_yaw, -sin_yaw];
    let distance = MOVE_SPEED * seconds;
    let p = camera.position();
    camera.set_position(WorldPosition::new(
        p.x + (intent.forward * forward[0] + intent.right * right[0]) * distance,
        p.y + intent.up * distance,
        p.z + (intent.forward * forward[1] + intent.right * right[1]) * distance,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_camera::Projection;
    use nexora_runtime::input::{DeviceId, Signal};

    const KEYBOARD: DeviceId = DeviceId::new(DeviceKind::Keyboard, 0);
    const STEP: Duration = Duration::from_millis(50);

    fn camera() -> Camera {
        Camera::new(
            WorldPosition::new(0.0, 0.0, 0.0),
            Projection::perspective(1.0, 0.1, 100.0).unwrap(),
        )
        .unwrap()
    }

    fn press(usage: u16) -> InputFrame {
        InputFrame::new()
            .with(Signal::Attached(KEYBOARD))
            .with(Signal::button(KEYBOARD, ButtonCode(usage), true))
    }

    /// The table is HID usages, the numbers the window host hands in: checked
    /// against the host's own translation, so the two cannot drift.
    #[test]
    fn the_default_keys_are_the_hosts_hid_usages() {
        use nexora_window::input::key_usage;
        use winit::keyboard::KeyCode as K;
        let keys = [
            K::KeyW,
            K::KeyS,
            K::KeyA,
            K::KeyD,
            K::Space,
            K::ShiftLeft,
            K::ArrowLeft,
            K::ArrowRight,
            K::ArrowUp,
            K::ArrowDown,
            K::Escape,
        ];
        for ((path, usage), key) in DEFAULT_KEYS.iter().zip(keys) {
            assert_eq!(key_usage(key), Some(ButtonCode(*usage)), "{path}");
        }
    }

    #[test]
    fn w_is_forward_and_escape_is_exit() {
        let mut controls = Controls::new().unwrap();
        let intent = controls.sample(&press(0x1A));
        assert_eq!(intent.forward, 1.0);
        assert_eq!(intent.held, 1);
        assert!(intent.moves());
        assert!(!intent.exit);
        let intent = controls.sample(&InputFrame::new().with(Signal::button(
            KEYBOARD,
            ButtonCode(0x29),
            true,
        )));
        assert!(intent.exit, "Escape goes down this frame");
        assert_eq!(intent.forward, 1.0, "and W is still held");
    }

    #[test]
    fn opposite_keys_cancel() {
        let mut controls = Controls::new().unwrap();
        let both = press(0x04).with(Signal::button(KEYBOARD, ButtonCode(0x07), true));
        let intent = controls.sample(&both);
        assert_eq!(intent.right, 0.0);
        assert_eq!(intent.held, 2);
        assert!(!intent.moves());
    }

    #[test]
    fn an_unbound_key_asks_for_nothing() {
        let mut controls = Controls::new().unwrap();
        assert_eq!(controls.sample(&press(0x3A)), Intent::default()); // F1
    }

    /// Yaw zero looks down -Z (ADR-0029): forward is -Z, right is +X.
    #[test]
    fn one_step_forward_and_right_at_yaw_zero() {
        let mut camera = camera();
        let intent = Intent {
            forward: 1.0,
            right: 1.0,
            ..Intent::default()
        };
        step(&mut camera, intent, STEP).unwrap();
        let p = camera.position();
        let d = MOVE_SPEED * 0.05;
        assert!(
            (p.x - d).abs() < 1e-12 && (p.z + d).abs() < 1e-12 && p.y == 0.0,
            "{p:?}"
        );
    }

    /// A quarter turn left looks down -X, and forward follows it.
    #[test]
    fn turning_left_turns_forward_with_it() {
        let mut camera = camera();
        let turn = Intent {
            turn: 1.0,
            ..Intent::default()
        };
        for _ in 0..20 {
            step(&mut camera, turn, STEP).unwrap();
        }
        assert!((camera.yaw() - FRAC_PI_2).abs() < 1e-9, "{}", camera.yaw());
        let forward = Intent {
            forward: 1.0,
            ..Intent::default()
        };
        step(&mut camera, forward, STEP).unwrap();
        let p = camera.position();
        assert!(
            (p.x + MOVE_SPEED * 0.05).abs() < 1e-9 && p.z.abs() < 1e-9,
            "{p:?}"
        );
    }

    /// Looking down does not slow walking: movement is horizontal.
    #[test]
    fn pitch_does_not_change_the_walking_speed() {
        let mut camera = camera();
        camera.set_orientation(0.0, -1.2).unwrap();
        let forward = Intent {
            forward: 1.0,
            ..Intent::default()
        };
        step(&mut camera, forward, STEP).unwrap();
        let p = camera.position();
        assert!(
            (p.z + MOVE_SPEED * 0.05).abs() < 1e-12 && p.y == 0.0,
            "{p:?}"
        );
    }

    #[test]
    fn up_is_up() {
        let mut camera = camera();
        let up = Intent {
            up: 1.0,
            ..Intent::default()
        };
        step(&mut camera, up, STEP).unwrap();
        assert!((camera.position().y - MOVE_SPEED * 0.05).abs() < 1e-12);
    }
}
