# ADR-0031 — A window hands input in as HID usages, once per frame

- **Status:** ACCEPTED
- **Date:** 2026-09-27
- **Completes (partly):** DEBT-0043 (keyboard and mouse buttons);
  `NEXORA ARCHITECTURE FREEZE CHECKLIST.md`, *Input boundary*
- **Follows:** ADR-0018 (input is intent the host hands in), ADR-0027 (the
  window host owns the event loop)
- **Touches:** `engine/window` (a new `input` module, `Client::input`, the
  `--input` mode of `nexora-window-probe`); `engine/runtime::input` (taps);
  CI; `scripts/local-validation.py`

## Context

ADR-0018 built the input system with the signal as an argument: it never
opens a device, so it replays. That left DEBT-0043, *no real device has ever
produced a signal*, whose trigger fired with ADR-0027: `nexora-window` runs
the operating system's event loop, and `winit` already delivered keyboard and
mouse events to it, which the host dropped.

`runtime::input` deliberately has no key names. `ButtonCode` is an opaque
`u16` *the host assigns*. So the first host had to answer:

1. **Which numbers.** A code is written into binding files. It has to mean
   the same key on every platform and survive a `winit` upgrade.
2. **When signals reach the input system.** The window receives events
   between frames; `sample` must be called once per frame (ADR-0018).
3. **What the operating system does that the input system must not see:**
   key repeat, and a window losing focus while a key is held.

## Options

**Codes.** (a) `winit`'s `KeyCode` discriminant: an implementation detail of
a dependency, reordered between versions. (b) Platform scancodes: four tables,
and a binding file that is only valid on the machine that wrote it.
(c) **USB HID usages** (*HID Usage Tables*, Keyboard/Keypad page 0x07, Button
page 0x09): a published standard, the one that names physical positions
(`KeyW` is usage 0x1A wherever the keycap says Z), and the one `winit`'s
physical key codes are already modelled on.

**Delivery.** (a) Call `sample` from the host on each event: many samples per
frame, and a client that samples too would count edges twice. (b) **Collect
the frame's signals in the host and hand them to the client once, before its
frame.** The client samples; the host never interprets.

## Decision

- **Keyboard codes are HID Keyboard/Keypad usages** (`nexora_window::input::
  key_usage`): letters 0x04–0x1D, digits 0x1E–0x27, Enter to Slash 0x28–0x38,
  Caps Lock, F1–F12 0x3A–0x45, the navigation block, arrows 0x4F–0x52, the
  keypad 0x53–0x63, the ISO backslash 0x64, the context-menu key 0x65 and the
  eight modifiers 0xE0–0xE7. A key with no usage in that table is **counted,
  not guessed** (`InputCounts::unnumbered`).
- **Mouse buttons are Button-page usages**: left 1, right 2, middle 3, back 4,
  forward 5; any other button `n` is 16 + `n` (saturating), so it can never
  collide with the five named ones.
- **One keyboard and one mouse**, `keyboard/0` and `mouse/0`, attached when
  the window opens. `winit` does not tell keyboards apart; neither does this.
- **The host collects** (`InputCollector`) and calls `Client::input(&frame)`
  once per redraw, **before** `Client::frame`. `input` has an empty default,
  so every existing client compiles unchanged.
- **Key repeat is dropped** (and counted): the operating system's repeat is a
  text-entry feature, and a held key is already held.
- **Focus loss releases everything**: the host sends `Detached` then
  `Attached` for both devices. A window that loses focus never hears the
  release, and ADR-0018 already defines detach as *release all*.
- **A tap is not lost** (`runtime::input`). A button pressed and released
  between two samples used to leave only its end-of-frame level, *up*, so a
  key tapped during a long frame never reached intent. It is now held for
  that frame and released at the next sample. The snapshots it produces are
  ones `validate_remote` already accepts.

## Verified

- `cargo test -p nexora-window --lib`: the HID table against the published
  values, no two keys sharing a usage, the mouse numbering, a key reaching an
  action with a repeat that is not a second press, focus loss releasing a
  held key, and a key without a usage counted.
- `cargo test -p nexora-runtime --lib input`: the tap, pressed then released;
  a tap followed by a press in the same frame stays held; a held button
  released is released at once. Removing the deferral fails it.
- **A real key, end to end.** `nexora-window-probe --input` opens a window
  titled *NEXORA input probe: press W*, binds W to one action, and waits for
  its press and release through `InputSystem::sample`. In CI,
  `scripts/input-probe-xtest.sh` runs it on Xvfb and presses W through the X
  server's test extension (XTEST), which the X server delivers to the window
  as it would a keyboard's key: `keyboard/0 button 26 (W) reached
  nexora:action/probe`. Nothing on the way is simulated by the engine.
  Mutation-checked: without the host's key translation the probe times out
  (6 signals, all attach and focus, no key).
- On the operator's machine, `local-validation.py run` now has an
  `input_devices` check that asks a person to press W. **It has not run yet**,
  so the boundary is verified on Xvfb only.

One observation from the first runs, recorded because it will recur: a focus
change can arrive between a press and its release, and then correctly
releases the key. The CI script presses W again every second until the probe
exits, so that race cannot fail the job, and each press is still a real key.

## Consequences

- DEBT-0043 moves to *partial*: keyboard and mouse buttons are built and
  verified in CI.
- `local-validation.py` drops `input_devices` from the items no machine can
  validate; it is a check now, skipped with `--quick`, when not run from a
  terminal, or with `NEXORA_INPUT=none`.
- The benchmark's *input* stage stays *not measured*. What reaches the engine
  is now real, but the plan's *input* stage is latency, from key to frame, and
  that needs a timestamp on the device event that `winit` does not give.

## Not built

Gamepads, touch, pointer motion and the wheel (ADR-0018 already says a pixel
delta is a different quantity from an axis in `[-1, 1]`), text entry and IME,
a remap written to disk, a snapshot that crossed a network, and more than one
keyboard or mouse.

## Migration

None. No binding file has been written to disk yet, so no stored code changes
meaning.

## Compatibility

Additive: `Client::input` has a default, `Session` gains a field, and
`nexora-window-probe` keeps its default mode.
