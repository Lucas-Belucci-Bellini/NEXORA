# ADR-0018 — Input is intent the host hands in, and a context consumes a source

- **Status:** ACCEPTED
- **Date:** 2026-09-21
- **Implements:** `CORE.md` §24 (ENGINE-8 — Input) and `INPUT SYSTEM.md`
- **Follows:** [ADR-0017](ADR-0017-a-frame-is-time-the-host-hands-in.md), which
  established the seam this reuses
- **Relates to:** [ADR-0010](ADR-0010-commands-are-intent-and-carry-their-own-authority.md)
  — that ADR gave intent a destination; this one gives it a source
- **Opens:** `DEBT-0042` (what resolution costs per frame), `DEBT-0043` (no real
  device has ever produced a signal)

## Context

`INPUT SYSTEM.md` opens with a sentence that decides most of the design:
*"Input traduz sinais de hardware e entrada local em intenções sem executar
gameplay."* It then draws a chain — hardware, raw input, device abstraction,
actions, mapping contexts, modifiers and chords, player intent, command — and
asks for six tests: binding conflict, context priority, device disconnect,
remap persistence, deterministic input replay, and server/client input
validation. `CORE.md` §24 adds the constraint in one line: *"o jogo não depende
diretamente de teclas."*

Three things about this repository make the obvious implementation wrong.

The first is that an input system is conventionally the place where the engine
*talks to the operating system*. It opens devices, registers callbacks, polls.
That is precisely what `nexora_foundation::time` forbids in its opening
paragraph — a system that samples the host cannot be replayed — and what
ADR-0017 already decided for the frame loop.

The second is that `FrameStage::Input` already existed, declared by ENGINE-0 and
reporting `has_system() == false`. The stage was named and empty. Whatever was
built had to fill that stage rather than sit beside it.

The third is that the command system already exists and already decides
authority (ADR-0010). An input system that executed anything would be
duplicating a boundary the repository has already drawn once.

## Decision

**Input is a function from signals to intent, and the host provides the
signals.** `InputSystem::sample` takes an `InputFrame` and returns an
`InputSnapshot`. Nothing in `engine/runtime/src/input.rs` opens a device,
registers a callback, or asks the operating system anything — the same seam
ADR-0017 established for time, for the same reason and with the same
consequence: feed the same signals and you get the same intent, on any machine.
That is not a nicety. *Deterministic input replay* is one of the six tests the
spec names, and it is a property of this shape rather than a feature added to
it.

**A binding names a device kind; a signal names a device instance.** A player
binds "gamepad button 0" once, not once per controller they might own. The
snapshot is the same whether the stick that moved was gamepad 0 or gamepad 3,
and plugging in a second pad is not a remap.

**A context consumes a source.** When two active contexts bind the same control,
only the higher-priority one fires, and the lower one does not see it. This is
the whole reason priorities exist: a menu at priority 100 must swallow the key
so the player underneath does not also jump. Within reach, the longest chord
wins, because otherwise `Ctrl+S` could never beat `S` and chords would be
decorative. Ties fall to the two identifiers — not because a tie is acceptable
configuration, but because a tie must have *one* answer: two peers who built the
same bindings in a different order have to agree about what the player just did.

**The engine does not learn what keys are called.** `ButtonCode` is an opaque
`u16` the host assigns. A scancode table in the engine is a table the engine
then has to be right about on four platforms, forever, and `CORE.md` §24 asks
for the opposite.

## Consequences

The frame has a fourth staffed stage, and the slice's walk is now driven through
it: the route used to be a `Vec<i64>` of stops written down in `stream_a_walk`
and is now what the input system says the player asked for — two keys bound to
one movement axis, one inverted, tapped once per leg. The derived route is
checked against the closed form it replaced, so the stage proves the chain from
scancode to interest source rather than merely exercising it. Every number the
slice reports is unchanged.

Something must hand the signals in, and today only the scripted player does.
That is `DEBT-0043`, and it is the honest price of the same decision ADR-0017
made: no keyboard, mouse or touch surface has ever produced a `Signal`, no remap
file has ever been written to disk, and `validate_remote` has never seen a
snapshot that crossed a network. The logic is exercised; the edge is not.

Resolution is a linear scan of the bindings, once per frame, whether or not
anything was pressed — measured at 322–356 ns for 41 bindings, of which about
70% is looking up each binding's context priority (`DEBT-0042`, baseline finding
26). It is not fixed here, deliberately: 322 ns is 0.00064% of the frame's
TARGET budget, and `DEBT-0010` is the precedent for what happens when an index
is added to a scan the numbers did not condemn.

## What was deliberately not decided

No key names, no pointer deltas, no touch gestures. An action axis is a
normalised intention in `[-1, 1]`; a mouse-look delta in pixels is a different
quantity and will want its own kind rather than a reinterpretation of this one.

No suppression of a chord's own modifiers: binding `Ctrl` and `Ctrl+S` in one
context fires both. Fixing that needs a lookahead the module does not have, and
inventing one before a real keymap has complained about it would be guessing at
a requirement.

No session state in `validate_remote`. It checks what can be checked without the
peer: that the action exists, that the value is finite and inside the range its
kind allows, and that the edges agree with the level. It does **not** check that
the peer's contexts bind that action, or that the frame number is one the server
has not already accepted — both need per-peer state that belongs to a session
layer which does not exist. Stated in the method's own documentation rather than
left for a reader to assume, because a security check that is trusted for more
than it does is worse than no check.
