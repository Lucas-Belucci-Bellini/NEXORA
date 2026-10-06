//! What the keys mean to a player: the default bindings, and the intent one
//! frame of signals folds into (ADR-0031, ADR-0035).
//!
//! The bindings are data in the input system's own terms: an action, a
//! context and a keyboard button numbered by its USB HID usage. Nothing here
//! reads a device or a clock. [`Controls::sample`] folds one frame of signals
//! into an [`Intent`], and [`Intent::walk`] keeps only the part a player
//! consumes — so the player is handed a [`Walk`] and never sees a key.
//!
//! The mouse's primary and secondary buttons mine and build (ADR-0036): the
//! same table, on the HID Button page, and the same fold — a press is one
//! edge, so one click asks for one edit however long the button is held.
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

/// The actions on the mouse and the button each is bound to by default, as a
/// HID Button page usage (primary 1, secondary 2; ADR-0031).
///
/// Mine and build are PLAYER-22's *Mining Intent* and PLAYER-21's *Build
/// Intent*: the player asks, and the command pipeline decides (ADR-0036).
pub const DEFAULT_BUTTONS: [(&str, u16); 2] = [
    ("action/mine", 1),  // primary
    ("action/build", 2), // secondary
];

/// What the player asked to do to the block it looks at this frame.
///
/// Edges, not held states: each is true on the one frame its button went
/// down. Nothing here names a block or a position — the authority casts the
/// ray from its own copy of the player (ADR-0036).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BlockIntent {
    /// Break the block looked at.
    pub mine: bool,
    /// Place a block against the face looked at.
    pub build: bool,
}

impl BlockIntent {
    /// Whether anything was asked for.
    #[must_use]
    pub const fn any(&self) -> bool {
        self.mine || self.build
    }

    /// Both intents: what was asked in either.
    #[must_use]
    pub const fn or(self, other: Self) -> Self {
        Self {
            mine: self.mine || other.mine,
            build: self.build || other.build,
        }
    }
}

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
    /// The mine button went down this frame.
    pub mine: bool,
    /// The build button went down this frame.
    pub build: bool,
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

    /// The part of this intent that asks for a block edit.
    #[must_use]
    pub const fn block(&self) -> BlockIntent {
        BlockIntent {
            mine: self.mine,
            build: self.build,
        }
    }

    /// The part of this intent a player consumes.
    ///
    /// `exit` and `held` belong to whoever runs the frames, not to the
    /// player, and do not cross; nor do `mine` and `build`, which go to the
    /// authority for edits ([`Intent::block`]).
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
        let bound = DEFAULT_KEYS
            .iter()
            .map(|(path, usage)| (DeviceKind::Keyboard, *path, *usage))
            .chain(
                DEFAULT_BUTTONS
                    .iter()
                    .map(|(path, usage)| (DeviceKind::Mouse, *path, *usage)),
            );
        for (device, path, usage) in bound {
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
            mine: snapshot.just_pressed(&self.actions[10]),
            build: snapshot.just_pressed(&self.actions[11]),
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
            mine: true,
            build: true,
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
    }

    const MOUSE: DeviceId = DeviceId::new(DeviceKind::Mouse, 0);

    fn click(button: u16, pressed: bool) -> InputFrame {
        InputFrame::new().with(Signal::button(MOUSE, ButtonCode(button), pressed))
    }

    /// A click is one edit however long it is held: the edge, not the state.
    #[test]
    fn the_primary_button_mines_once_per_press() {
        let mut controls = Controls::new().unwrap();
        let down = controls.sample(
            &InputFrame::new()
                .with(Signal::Attached(MOUSE))
                .with(Signal::button(MOUSE, ButtonCode(1), true)),
        );
        assert_eq!(
            down.block(),
            BlockIntent {
                mine: true,
                build: false
            }
        );
        assert_eq!(down.held, 1);
        assert!(!down.moves(), "mining is not movement");
        assert_eq!(down.walk(), Walk::default(), "and does not reach the walk");
        let still_held = controls.sample(&InputFrame::new());
        assert!(!still_held.block().any(), "held is not pressed again");
        assert_eq!(still_held.held, 1);
        controls.sample(&click(1, false));
        assert!(controls.sample(&click(1, true)).mine, "a second press");
    }

    #[test]
    fn the_secondary_button_builds_and_the_middle_one_is_unbound() {
        let mut controls = Controls::new().unwrap();
        controls.sample(&InputFrame::new().with(Signal::Attached(MOUSE)));
        assert_eq!(
            controls.sample(&click(2, true)).block(),
            BlockIntent {
                mine: false,
                build: true
            }
        );
        let mut controls = Controls::new().unwrap();
        controls.sample(&InputFrame::new().with(Signal::Attached(MOUSE)));
        assert_eq!(controls.sample(&click(3, true)), Intent::default());
    }

    /// A keyboard key with the primary button's number is not the primary
    /// button: the device kind is part of the binding.
    #[test]
    fn a_key_numbered_like_a_button_does_not_mine() {
        let mut controls = Controls::new().unwrap();
        // Keyboard usage 1 is ErrorRollOver, never a real key, but the
        // input system must still tell the pages apart.
        assert!(!controls.sample(&press(1)).block().any());
    }

    #[test]
    fn block_intents_combine() {
        let mine = BlockIntent {
            mine: true,
            build: false,
        };
        let build = BlockIntent {
            mine: false,
            build: true,
        };
        assert_eq!(
            mine.or(build),
            BlockIntent {
                mine: true,
                build: true
            }
        );
        assert!(!BlockIntent::default().any());
        assert!(mine.any());
    }

    #[test]
    fn every_action_has_its_own_key() {
        let mut usages: Vec<u16> = DEFAULT_KEYS.iter().map(|(_, usage)| *usage).collect();
        usages.sort_unstable();
        usages.dedup();
        assert_eq!(usages.len(), DEFAULT_KEYS.len());
    }
}
