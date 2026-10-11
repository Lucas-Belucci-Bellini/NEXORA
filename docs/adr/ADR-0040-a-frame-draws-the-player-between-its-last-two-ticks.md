# ADR-0040 — A frame draws the player between its last two ticks, and never ahead of them

- **Status:** ACCEPTED
- **Date:** 2026-10-11
- **Resolves:** `DEBT-0049`
- **Amends:** [ADR-0017](ADR-0017-a-frame-is-time-the-host-hands-in.md)
  (the frame says how far real time has run into the next step) and
  [ADR-0036](ADR-0036-a-player-edits-through-commands-and-a-save-keeps-the-player.md)
  (the block aimed at is the block under the centre of a frame drawn at the
  eye; between ticks a frame trails the eye by less than a tick)
- **Builds on:** [ADR-0029](ADR-0029-the-camera-is-reverse-z-over-a-floating-integer-origin.md)
  (which left interpolation between ticks unbuilt until there was a target
  to follow), [ADR-0035](ADR-0035-the-player-is-a-body-the-simulation-steers.md)
  (the camera only follows the player's eye),
  [ADR-0030](ADR-0030-the-first-render-pass-is-checked-against-a-ray-cast.md)
  (a frame is judged against a ray cast through the camera that drew it)

## Context

The player ticks at 20 Hz (ADR-0035) and the client draws as often as the
display presents — about 100 Hz on the operator's machine (Appendix O). Every
frame drew the camera at the player's eye after its last tick, so four frames
in five repeated the one before, and the view moved in 50 ms steps; a step-up
lifted the eye a whole block between two frames (`DEBT-0049`). ADR-0029 left
interpolation between ticks unbuilt because nothing followed a target; since
ADR-0035 the camera follows the eye, so the trigger it named had fired.

ADR-0017 kept the interpolation factor out of the frame on purpose: *an alpha
nobody reads is a number nobody checks*. A reader now exists.

## Problem

Draw the player between ticks without:

- touching the authoritative state (the simulation must not read anything a
  frame computes, or a replay would depend on the display);
- drawing anything the simulation has not reached (extrapolation shows a
  player walking into a wall it then stops at);
- giving up the frame check: every judged frame is held against a ray cast
  through the camera that drew it (ADR-0030).

## Options

1. **Extrapolate from the last tick** by velocity. Zero latency, but it draws
   states that never happen: through a wall, over a ledge it then stops at.
   Rejected: the client would show what the simulation did not do.
2. **Interpolate between the last two ticks** by how far real time has run
   into the next step. Never ahead of the simulation; at most one tick behind
   it.
3. **Run the player at the frame rate.** Changes the simulation's tick, which
   every other system and the replay unit (ADR-0035) depend on. Rejected.
4. **Smooth the camera independently** (a spring towards the eye). Its lag
   depends on the frame rate and its state is one more thing a judged frame
   would have to reproduce. Rejected for now.

## Decision

Option 2.

1. **The frame says how far into the next step it is.**
   `nexora_runtime::frame::StepPlan` carries the schedule's `step`, and
   `StepPlan::alpha()` is `carried / step`, in `[0, 1)` — `carried` is below
   one step after every `advance`, discarded backlog included. The frame
   loop computes it and does nothing with it; what to draw is the
   presentation's.
2. **The client keeps two eyes.** `EyeTrack` holds the player's eye before
   its last tick and after it; every tick the frame runs moves the second to
   the first and records the new one. A frame draws `alpha` of the way from
   the first to the second: position and pitch in a straight line, yaw the
   shorter way round and wrapped into `(-π, π]`. What did not change is kept
   to the bit, so a player at rest is drawn exactly at its eye.
3. **Never ahead, at most one tick behind.** At `alpha = 0` a frame shows the
   eye before the last tick; it never reaches the eye after it. The first
   frame after a player sets off therefore still shows where it stood — the
   one-tick latency this option accepts.
4. **Authority is untouched.** The simulation never reads the track; edits
   are aimed from the player's own eye (ADR-0036's ray). When the player
   stands still the frame's centre is the aimed block exactly, as before;
   while it moves or turns, the centre trails the aim by less than one tick
   of that motion. ADR-0036's sentence *"the block aimed at is the block under
   the centre of the frame"* now reads: of a frame drawn at the eye.
5. **The run's integrity check follows.** At the end of a client run the
   track's second eye must be the player's eye, and the camera must be the
   one the track shows at the last frame's `alpha`; either parting is an
   error, as the camera parting from the eye was.
6. **Judged frames need nothing new.** A frame is captured with the camera
   that drew it and judged against a ray cast through that camera, so an
   interpolated frame is judged exactly as a whole-tick one was.

## Consequences

- Frames between ticks move: in a test at 100 Hz drawing a 20 Hz player,
  none of 96 walking frames repeats a view; with the old whole-tick camera,
  76 of 96 did. Mutation-checked both ways: drawing the eye after the tick
  repeats them, drawing ahead of it fails the never-ahead check.
- A step-up's one-block rise is spread over a tick's frames instead of one.
  Easing the eye over several ticks is **not built**; nothing asks for it yet.
- A player that sets off is drawn one tick late (50 ms). That is the cost of
  never drawing what the simulation did not do.
- `StepPlan` gained a field. It is built only by `FrameSchedule::advance`.

## Migration

None: nothing persisted refers to a frame, an alpha or a camera.

## Compatibility

`nexora-client`'s report is unchanged. `camera moved` can be smaller than
`player walked` by less than one tick of walking when a run ends while the
player moves; CI's play ends standing still, where the two are equal.

## Verification

- `nexora-runtime`: `alpha_is_how_far_into_the_next_step_real_time_has_run`
  (0.4 at 20 ms into a 50 ms step, 0 after a step lands, below 1 after a
  capped hitch).
- `nexora-client`: `between_keeps_what_did_not_change_and_starts_at_the_eye_before`,
  `a_turn_across_pi_takes_the_short_way_and_stays_in_range`, and
  `frames_between_ticks_move_and_never_lead_the_simulation` — the same
  `EyeTrack` and `FrameSchedule` the client's frame uses, 10 ms frames, a
  second of W and a second without: no repeated walking frame, every frame
  between the eye before the last tick and after it, and at rest the camera
  equal to the eye's.
- CI's client runs (Xvfb, lavapipe) still judge the first, edited and moved
  frames against the ray cast, now through interpolated cameras.
