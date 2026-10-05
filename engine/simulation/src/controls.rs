//! What the keys and buttons mean to a player: the default bindings, and the
//! intent one frame of signals folds into (ADR-0031, ADR-0035, ADR-0036).
//!
//! The bindings are data in the input system's own terms: an action, a
//! context and a keyboard button numbered by its USB HID usage. Nothing here
//! reads a device or a clock. [`Controls::sample`] folds one frame of signals
//! into an [`Intent`], and [`Intent::walk`] keeps only the part a player
//! consumes — so the player is handed a [`Walk`] and never sees a key.
//!
//! This lived in the client while the keys flew a free camera. It moved here,
//! beside the player, so the slice and the benchmark drive the player through
//! the same table and the same sampling as the window: a scripted frame of
//! HID usages and a real keyboard reach the player by one path. It needs only
//! the runtime's input system and foundation's identifiers, which the
//! simulation already depends on; a server simply never builds one.

use nexora_foundation::error::Result;
use nexora_foundation::ident::Identifier;
use nexora_runtime::input::{
    ActionDefinition, ActionKind, Binding, ButtonCode, DeviceKind, InputFrame, InputSystem, Source,
};

use crate::player::Walk;

/// The context every client binding lives in.
pub const CONTEXT: &str = "context/client";

/// The actions and the key each is bound to by default, as a HID
/// Keyboard/Keypad usage (page 0x07).
///
/// Space is jump. Left Shift, which flew the free camera down, is unbound:
/// there is no crouch to give it, and a binding that does nothing is a
/// feature that does nothing.
pub const DEFAULT_KEYS: [(&str, u16); 10] = [
    ("action/move_forward", 0x1A), // W
    ("action/move_back", 0x16),    // S
    ("action/move_left", 0x04),    // A
    ("action/move_right", 0x07),   // D
    ("action/jump", 0x2C),         // Space
    ("action/turn_left", 0x50),    // Left arrow
    ("action/turn_right", 0x4F),   // Right arrow
    ("action/look_up", 0x52),      // Up arrow
    ("action/look_down", 0x51),    // Down arrow
    ("action/exit", 0x29),         // Escape
];

/// The actions bound to the mouse, and the button each is bound to by
/// default, as a HID Button-page usage (page 0x09): 1 is the primary button,
/// 2 the secondary (ADR-0031).
///
/// A separate table from [`DEFAULT_KEYS`] because the numbers are on another
/// page: usage 1 is the left button here and a reserved code on the keyboard.
pub const DEFAULT_BUTTONS: [(&str, u16); 2] = [
    ("action/break_block", 1), // primary (left)
    ("action/place_block", 2), // secondary (right)
];

/// What the player asked for this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Intent {
    /// `+1` forward, `-1` back, `0` both or neither.
    pub forward: f64,
    /// `+1` right, `-1` left.
    pub right: f64,
    /// `+1` turn left (counter-clockwise seen from above), `-1` right.
    pub turn: f64,
    /// `+1` look up, `-1` down.
    pub look: f64,
    /// Jump is held.
    pub jump: bool,
    /// The exit action went down this frame.
    pub exit: bool,
    /// The break action went down this frame: one press is one request.
    pub break_block: bool,
    /// The place action went down this frame.
    pub place_block: bool,
    /// How many of the actions are held.
    pub held: u32,
}

impl Intent {
    /// Whether this intent moves, turns or lifts the player.
    #[must_use]
    pub fn moves(&self) -> bool {
        self.jump
            || [self.forward, self.right, self.turn, self.look]
                .iter()
                .any(|axis| *axis != 0.0)
    }

    /// The part of this intent a player consumes.
    ///
    /// `exit` and `held` belong to whoever runs the frames, not to the
    /// player, and do not cross. Neither do `break_block` and `place_block`:
    /// they are requests to the authority over the world, not movement, and
    /// travel as commands (ADR-0036).
    #[must_use]
    pub const fn walk(&self) -> Walk {
        Walk {
            forward: self.forward,
            right: self.right,
            turn: self.turn,
            look: self.look,
            jump: self.jump,
        }
    }
}

/// The input system, loaded with the actions and bindings.
#[derive(Debug)]
pub struct Controls {
    system: InputSystem,
    actions: Vec<Identifier>,
}

impl Controls {
    /// Register the actions in one context, bound to [`DEFAULT_KEYS`] and
    /// [`DEFAULT_BUTTONS`].
    ///
    /// # Errors
    ///
    /// An identifier or a binding was refused by the input system.
    pub fn new() -> Result<Self> {
        let context = Identifier::nexora(CONTEXT)?;
        let mut system = InputSystem::new();
        system.set_context(context.clone(), 0);
        let mut actions = Vec::with_capacity(DEFAULT_KEYS.len() + DEFAULT_BUTTONS.len());
        let keys = DEFAULT_KEYS.map(|(path, usage)| (path, DeviceKind::Keyboard, usage));
        let buttons = DEFAULT_BUTTONS.map(|(path, usage)| (path, DeviceKind::Mouse, usage));
        for (path, device, usage) in keys.into_iter().chain(buttons) {
            let action = Identifier::nexora(path)?;
            system.register_action(ActionDefinition::new(action.clone(), ActionKind::Button))?;
            system.bind(Binding::new(
                context.clone(),
                action.clone(),
                Source::button(device, ButtonCode(usage)),
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
            turn: axis(5, 6),
            look: axis(7, 8),
            jump: held(4),
            exit: snapshot.just_pressed(&self.actions[9]),
            break_block: snapshot.just_pressed(&self.actions[10]),
            place_block: snapshot.just_pressed(&self.actions[11]),
            held: (0..self.actions.len()).filter(|index| held(*index)).count() as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_runtime::input::{DeviceId, Signal};

    const KEYBOARD: DeviceId = DeviceId::new(DeviceKind::Keyboard, 0);

    fn press(usage: u16) -> InputFrame {
        InputFrame::new()
            .with(Signal::Attached(KEYBOARD))
            .with(Signal::button(KEYBOARD, ButtonCode(usage), true))
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

    #[test]
    fn space_is_jump_and_left_shift_is_unbound() {
        let mut controls = Controls::new().unwrap();
        let intent = controls.sample(&press(0x2C));
        assert!(intent.jump);
        assert_eq!(intent.held, 1);
        assert!(intent.moves());
        let mut controls = Controls::new().unwrap();
        assert_eq!(controls.sample(&press(0xE1)), Intent::default());
    }

    #[test]
    fn intent_walk_carries_only_movement() {
        let intent = Intent {
            forward: 1.0,
            right: -1.0,
            turn: 1.0,
            look: -1.0,
            jump: true,
            exit: true,
            break_block: true,
            place_block: true,
            held: 6,
        };
        assert_eq!(
            intent.walk(),
            Walk {
                forward: 1.0,
                right: -1.0,
                turn: 1.0,
                look: -1.0,
                jump: true,
            }
        );
        let quiet = Intent {
            exit: true,
            held: 1,
            ..Intent::default()
        };
        assert_eq!(quiet.walk(), Walk::default(), "exit and held do not cross");
        let clicks = Intent {
            break_block: true,
            place_block: true,
            ..Intent::default()
        };
        assert_eq!(clicks.walk(), Walk::default(), "nor do the hands");
        assert!(!clicks.moves());
    }

    const MOUSE: DeviceId = DeviceId::new(DeviceKind::Mouse, 0);

    fn click(usage: u16) -> InputFrame {
        InputFrame::new()
            .with(Signal::Attached(MOUSE))
            .with(Signal::button(MOUSE, ButtonCode(usage), true))
    }

    #[test]
    fn the_primary_button_breaks_and_the_secondary_places_once_per_press() {
        let mut controls = Controls::new().unwrap();
        let intent = controls.sample(&click(1));
        assert!(intent.break_block && !intent.place_block);
        assert_eq!(intent.held, 1);
        assert!(!intent.moves(), "a click is not movement");
        // Still held on the next frame: the press is not repeated.
        let held = controls.sample(&InputFrame::new());
        assert!(!held.break_block, "one press, one request");
        assert_eq!(held.held, 1);

        let mut controls = Controls::new().unwrap();
        let intent = controls.sample(&click(2));
        assert!(intent.place_block && !intent.break_block);
    }

    #[test]
    fn a_keyboard_usage_is_not_a_mouse_button() {
        // Usage 1 on the keyboard page is a reserved code, not the primary
        // button: the device is part of the binding.
        let mut controls = Controls::new().unwrap();
        assert_eq!(controls.sample(&press(1)), Intent::default());
    }

    #[test]
    fn every_action_has_its_own_key() {
        for table in [&DEFAULT_KEYS[..], &DEFAULT_BUTTONS[..]] {
            let mut usages: Vec<u16> = table.iter().map(|(_, usage)| *usage).collect();
            usages.sort_unstable();
            usages.dedup();
            assert_eq!(usages.len(), table.len());
        }
    }
}
