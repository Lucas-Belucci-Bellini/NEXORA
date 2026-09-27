//! Device events from the window system, as the engine's input signals
//! (ADR-0031, DEBT-0043).
//!
//! `nexora_runtime::input` never reads a device (ADR-0018): the host hands it
//! [`Signal`]s. This is the host's half. It turns what `winit` reports into
//! signals, collects them between frames, and gives the client one
//! [`InputFrame`] per frame.
//!
//! # The numbering
//!
//! A [`ButtonCode`] is whatever the host says it is, so the host has to say
//! it once and keep it: bindings are saved with it. Keys are numbered by their
//! **USB HID usage** (Keyboard/Keypad page, 0x07): `A` is 4, `W` is 26,
//! `Space` is 44, `Left Control` is 224. That numbering belongs to no
//! operating system. It is also the table `winit`'s key codes come from
//! (the W3C UI Events `code` values). So a binding written on Windows means
//! the same key on macOS and Linux, and it names a **position** on the
//! keyboard, not a letter: `W` on QWERTY is `Z` on AZERTY, which is what
//! movement keys want.
//!
//! Mouse buttons are numbered by the HID Button page: primary 1, secondary 2,
//! middle 3, back 4, forward 5. Other buttons count on from 16.
//!
//! # What the host decides for the input system
//!
//! - **Key repeat is dropped.** The operating system repeats a held key; the
//!   input system already knows the key is held, and a repeat would read as a
//!   second press.
//! - **Losing focus releases everything.** A window that loses focus stops
//!   receiving key releases, so a key held when the player alt-tabs would stay
//!   pressed forever. The host sends `Detached` then `Attached` for the
//!   keyboard and the mouse, which the input system answers by releasing
//!   everything they held (its declared rule for a device that goes away).
//! - **The keyboard and the mouse attach when the window opens.** The window
//!   system reports one keyboard and one pointer, whatever is plugged in, so
//!   they are `keyboard/0` and `mouse/0`.
//! - **Keys with no HID usage here are not signals.** They are counted, not
//!   invented a number.

use nexora_runtime::input::{ButtonCode, DeviceId, DeviceKind, InputFrame, Signal};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

/// The window system's keyboard.
pub const KEYBOARD: DeviceId = DeviceId::new(DeviceKind::Keyboard, 0);

/// The window system's pointer.
pub const MOUSE: DeviceId = DeviceId::new(DeviceKind::Mouse, 0);

/// Where mouse buttons beyond the fifth start counting.
pub const MOUSE_OTHER_BASE: u16 = 16;

/// The HID usage of a key, or `None` for a key this table does not number.
#[must_use]
pub const fn key_usage(code: KeyCode) -> Option<ButtonCode> {
    use KeyCode as K;
    let usage: u16 = match code {
        K::KeyA => 0x04,
        K::KeyB => 0x05,
        K::KeyC => 0x06,
        K::KeyD => 0x07,
        K::KeyE => 0x08,
        K::KeyF => 0x09,
        K::KeyG => 0x0A,
        K::KeyH => 0x0B,
        K::KeyI => 0x0C,
        K::KeyJ => 0x0D,
        K::KeyK => 0x0E,
        K::KeyL => 0x0F,
        K::KeyM => 0x10,
        K::KeyN => 0x11,
        K::KeyO => 0x12,
        K::KeyP => 0x13,
        K::KeyQ => 0x14,
        K::KeyR => 0x15,
        K::KeyS => 0x16,
        K::KeyT => 0x17,
        K::KeyU => 0x18,
        K::KeyV => 0x19,
        K::KeyW => 0x1A,
        K::KeyX => 0x1B,
        K::KeyY => 0x1C,
        K::KeyZ => 0x1D,
        K::Digit1 => 0x1E,
        K::Digit2 => 0x1F,
        K::Digit3 => 0x20,
        K::Digit4 => 0x21,
        K::Digit5 => 0x22,
        K::Digit6 => 0x23,
        K::Digit7 => 0x24,
        K::Digit8 => 0x25,
        K::Digit9 => 0x26,
        K::Digit0 => 0x27,
        K::Enter => 0x28,
        K::Escape => 0x29,
        K::Backspace => 0x2A,
        K::Tab => 0x2B,
        K::Space => 0x2C,
        K::Minus => 0x2D,
        K::Equal => 0x2E,
        K::BracketLeft => 0x2F,
        K::BracketRight => 0x30,
        K::Backslash => 0x31,
        K::Semicolon => 0x33,
        K::Quote => 0x34,
        K::Backquote => 0x35,
        K::Comma => 0x36,
        K::Period => 0x37,
        K::Slash => 0x38,
        K::CapsLock => 0x39,
        K::F1 => 0x3A,
        K::F2 => 0x3B,
        K::F3 => 0x3C,
        K::F4 => 0x3D,
        K::F5 => 0x3E,
        K::F6 => 0x3F,
        K::F7 => 0x40,
        K::F8 => 0x41,
        K::F9 => 0x42,
        K::F10 => 0x43,
        K::F11 => 0x44,
        K::F12 => 0x45,
        K::PrintScreen => 0x46,
        K::ScrollLock => 0x47,
        K::Pause => 0x48,
        K::Insert => 0x49,
        K::Home => 0x4A,
        K::PageUp => 0x4B,
        K::Delete => 0x4C,
        K::End => 0x4D,
        K::PageDown => 0x4E,
        K::ArrowRight => 0x4F,
        K::ArrowLeft => 0x50,
        K::ArrowDown => 0x51,
        K::ArrowUp => 0x52,
        K::NumLock => 0x53,
        K::NumpadDivide => 0x54,
        K::NumpadMultiply => 0x55,
        K::NumpadSubtract => 0x56,
        K::NumpadAdd => 0x57,
        K::NumpadEnter => 0x58,
        K::Numpad1 => 0x59,
        K::Numpad2 => 0x5A,
        K::Numpad3 => 0x5B,
        K::Numpad4 => 0x5C,
        K::Numpad5 => 0x5D,
        K::Numpad6 => 0x5E,
        K::Numpad7 => 0x5F,
        K::Numpad8 => 0x60,
        K::Numpad9 => 0x61,
        K::Numpad0 => 0x62,
        K::NumpadDecimal => 0x63,
        K::IntlBackslash => 0x64,
        K::ContextMenu => 0x65,
        K::ControlLeft => 0xE0,
        K::ShiftLeft => 0xE1,
        K::AltLeft => 0xE2,
        K::SuperLeft => 0xE3,
        K::ControlRight => 0xE4,
        K::ShiftRight => 0xE5,
        K::AltRight => 0xE6,
        K::SuperRight => 0xE7,
        _ => return None,
    };
    Some(ButtonCode(usage))
}

/// The HID Button-page number of a mouse button.
#[must_use]
pub const fn mouse_usage(button: MouseButton) -> ButtonCode {
    ButtonCode(match button {
        MouseButton::Left => 1,
        MouseButton::Right => 2,
        MouseButton::Middle => 3,
        MouseButton::Back => 4,
        MouseButton::Forward => 5,
        MouseButton::Other(n) => MOUSE_OTHER_BASE.saturating_add(n),
    })
}

/// Signals collected between two frames.
#[derive(Debug, Default)]
pub struct InputCollector {
    frame: InputFrame,
    delivered: u64,
    unnumbered: u64,
    repeats: u64,
}

/// What the host has done with device events so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputCounts {
    /// Signals handed to the client.
    pub delivered: u64,
    /// Key events for keys this table does not number, dropped.
    pub unnumbered: u64,
    /// Key repeats the operating system sent, dropped.
    pub repeats: u64,
}

impl InputCollector {
    /// The window opened: the keyboard and the pointer are there.
    pub fn attach(&mut self) {
        self.frame.push(Signal::Attached(KEYBOARD));
        self.frame.push(Signal::Attached(MOUSE));
    }

    /// A key changed state.
    pub fn key(&mut self, key: PhysicalKey, state: ElementState, repeat: bool) {
        if repeat {
            self.repeats += 1;
            return;
        }
        let PhysicalKey::Code(code) = key else {
            self.unnumbered += 1;
            return;
        };
        let Some(usage) = key_usage(code) else {
            self.unnumbered += 1;
            return;
        };
        self.frame
            .push(Signal::button(KEYBOARD, usage, state.is_pressed()));
    }

    /// A mouse button changed state.
    pub fn mouse(&mut self, button: MouseButton, state: ElementState) {
        self.frame.push(Signal::button(
            MOUSE,
            mouse_usage(button),
            state.is_pressed(),
        ));
    }

    /// The window lost focus: release everything the keyboard and the mouse
    /// held, and keep them attached.
    pub fn focus_lost(&mut self) {
        for device in [KEYBOARD, MOUSE] {
            self.frame.push(Signal::Detached(device));
            self.frame.push(Signal::Attached(device));
        }
    }

    /// The signals since the last frame, leaving the collector empty.
    pub fn take(&mut self) -> InputFrame {
        let frame = std::mem::take(&mut self.frame);
        self.delivered += frame.len() as u64;
        frame
    }

    /// What has happened so far.
    #[must_use]
    pub const fn counts(&self) -> InputCounts {
        InputCounts {
            delivered: self.delivered,
            unnumbered: self.unnumbered,
            repeats: self.repeats,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::ident::Identifier;
    use nexora_runtime::input::{ActionDefinition, ActionKind, Binding, InputSystem, Source};

    #[test]
    fn letters_and_digits_follow_the_hid_table() {
        assert_eq!(key_usage(KeyCode::KeyA), Some(ButtonCode(4)));
        assert_eq!(key_usage(KeyCode::KeyW), Some(ButtonCode(26)));
        assert_eq!(key_usage(KeyCode::KeyZ), Some(ButtonCode(29)));
        assert_eq!(key_usage(KeyCode::Digit1), Some(ButtonCode(30)));
        assert_eq!(key_usage(KeyCode::Digit0), Some(ButtonCode(39)));
        assert_eq!(key_usage(KeyCode::Space), Some(ButtonCode(44)));
        assert_eq!(key_usage(KeyCode::ShiftLeft), Some(ButtonCode(0xE1)));
        assert_eq!(key_usage(KeyCode::Fn), None);
    }

    #[test]
    fn no_two_keys_share_a_number() {
        use KeyCode as K;
        let keys = [
            K::KeyA,
            K::KeyB,
            K::KeyC,
            K::KeyD,
            K::KeyE,
            K::KeyF,
            K::KeyG,
            K::KeyH,
            K::KeyI,
            K::KeyJ,
            K::KeyK,
            K::KeyL,
            K::KeyM,
            K::KeyN,
            K::KeyO,
            K::KeyP,
            K::KeyQ,
            K::KeyR,
            K::KeyS,
            K::KeyT,
            K::KeyU,
            K::KeyV,
            K::KeyW,
            K::KeyX,
            K::KeyY,
            K::KeyZ,
            K::Digit0,
            K::Digit1,
            K::Digit2,
            K::Digit3,
            K::Digit4,
            K::Digit5,
            K::Digit6,
            K::Digit7,
            K::Digit8,
            K::Digit9,
            K::Enter,
            K::Escape,
            K::Backspace,
            K::Tab,
            K::Space,
            K::Minus,
            K::Equal,
            K::BracketLeft,
            K::BracketRight,
            K::Backslash,
            K::Semicolon,
            K::Quote,
            K::Backquote,
            K::Comma,
            K::Period,
            K::Slash,
            K::CapsLock,
            K::F1,
            K::F2,
            K::F3,
            K::F4,
            K::F5,
            K::F6,
            K::F7,
            K::F8,
            K::F9,
            K::F10,
            K::F11,
            K::F12,
            K::PrintScreen,
            K::ScrollLock,
            K::Pause,
            K::Insert,
            K::Home,
            K::PageUp,
            K::Delete,
            K::End,
            K::PageDown,
            K::ArrowRight,
            K::ArrowLeft,
            K::ArrowDown,
            K::ArrowUp,
            K::NumLock,
            K::NumpadDivide,
            K::NumpadMultiply,
            K::NumpadSubtract,
            K::NumpadAdd,
            K::NumpadEnter,
            K::Numpad0,
            K::Numpad1,
            K::Numpad2,
            K::Numpad3,
            K::Numpad4,
            K::Numpad5,
            K::Numpad6,
            K::Numpad7,
            K::Numpad8,
            K::Numpad9,
            K::NumpadDecimal,
            K::IntlBackslash,
            K::ContextMenu,
            K::ControlLeft,
            K::ShiftLeft,
            K::AltLeft,
            K::SuperLeft,
            K::ControlRight,
            K::ShiftRight,
            K::AltRight,
            K::SuperRight,
        ];
        let mut codes: Vec<u16> = keys
            .iter()
            .map(|key| key_usage(*key).expect("numbered").0)
            .collect();
        let total = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), total);
    }

    #[test]
    fn mouse_buttons_follow_the_hid_button_page() {
        assert_eq!(mouse_usage(MouseButton::Left), ButtonCode(1));
        assert_eq!(mouse_usage(MouseButton::Right), ButtonCode(2));
        assert_eq!(mouse_usage(MouseButton::Middle), ButtonCode(3));
        assert_eq!(mouse_usage(MouseButton::Other(2)), ButtonCode(18));
        assert_eq!(
            mouse_usage(MouseButton::Other(u16::MAX)),
            ButtonCode(u16::MAX)
        );
    }

    fn system() -> (InputSystem, Identifier) {
        let forward = Identifier::nexora("action/forward").unwrap();
        let context = Identifier::nexora("context/test").unwrap();
        let mut input = InputSystem::new();
        input
            .register_action(ActionDefinition::new(forward.clone(), ActionKind::Button))
            .unwrap();
        input.set_context(context.clone(), 0);
        input
            .bind(Binding::new(
                context,
                forward.clone(),
                Source::button(DeviceKind::Keyboard, key_usage(KeyCode::KeyW).unwrap()),
            ))
            .unwrap();
        (input, forward)
    }

    #[test]
    fn a_key_reaches_an_action_and_a_repeat_is_not_a_second_press() {
        let (mut input, forward) = system();
        let mut collector = InputCollector::default();
        collector.attach();
        collector.key(
            PhysicalKey::Code(KeyCode::KeyW),
            ElementState::Pressed,
            false,
        );
        assert!(input.sample(&collector.take()).just_pressed(&forward));
        collector.key(
            PhysicalKey::Code(KeyCode::KeyW),
            ElementState::Pressed,
            true,
        );
        let held = input.sample(&collector.take());
        assert!(held.is_pressed(&forward) && !held.just_pressed(&forward));
        collector.key(
            PhysicalKey::Code(KeyCode::KeyW),
            ElementState::Released,
            false,
        );
        assert!(input.sample(&collector.take()).just_released(&forward));
        assert_eq!(collector.counts().repeats, 1);
    }

    #[test]
    fn losing_focus_releases_a_held_key() {
        let (mut input, forward) = system();
        let mut collector = InputCollector::default();
        collector.attach();
        collector.key(
            PhysicalKey::Code(KeyCode::KeyW),
            ElementState::Pressed,
            false,
        );
        assert!(input.sample(&collector.take()).is_pressed(&forward));
        collector.focus_lost();
        let after = input.sample(&collector.take());
        assert!(
            !after.is_pressed(&forward),
            "a key held across focus loss stayed down"
        );
        // Still attached: the next press works.
        collector.key(
            PhysicalKey::Code(KeyCode::KeyW),
            ElementState::Pressed,
            false,
        );
        assert!(input.sample(&collector.take()).just_pressed(&forward));
    }

    #[test]
    fn a_key_without_a_number_is_counted_not_invented() {
        let mut collector = InputCollector::default();
        collector.key(PhysicalKey::Code(KeyCode::Fn), ElementState::Pressed, false);
        assert!(collector.take().is_empty());
        assert_eq!(collector.counts().unnumbered, 1);
    }
}
