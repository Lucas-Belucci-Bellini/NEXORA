# ADR-0036 — The player's hands are commands the authority checks against its body

- **Status:** ACCEPTED
- **Date:** 2026-10-05
- **Amends:** [ADR-0035](ADR-0035-the-player-is-a-body-the-simulation-steers.md)
  (the player gains `stance`; "nothing edits the client's world" no longer
  holds) and [ADR-0032](ADR-0032-the-client-is-the-runtime-in-a-window.md)
  (the client meshes and uploads again what an edit changed, and can start
  from a save)
- **Informs:** the roadmap's Phase 2 (*interaction*), the vertical slice
  `WORLD → PLAYER → INTERACTION → STATE CHANGE → SAVE → RELOAD`
- **Opens:** `DEBT-0053`; finds and closes `DEBT-0052`

## Context

The pieces of an interaction existed apart and nothing joined them. The block
commands, their handlers and a server-side reach rule (`nexora_simulation::
commands`, `MAX_REACH_BLOCKS` = 6) were exercised by the slice from a fixed
position no player stood at. The physics crate had the voxel ray cast
(`PHYSICS.md` §23: *what block is the player looking at*). The player
(ADR-0035) had an eye and a facing, and the window already delivered mouse
buttons as HID usages (ADR-0031). The client's world could not change after
the first frame: nothing meshed again, and the first frame's check read the
world as it was at the end of the run.

The specifications fix the shape. `PLAYER SYSTEM.md` PLAYER-13: *Player →
Interaction Ray → Target → Interaction API*; PLAYER-14: an interaction carries
the player, the target, the position and the face; PLAYER-16 narrows it to
blocks. `Build & Destruction Engine.md` BUILD-1: a placement is never a bare
`world.setBlock` and must check, among others, *colisão*. `Command System.md`
§72: a client's command is untrusted; §23: reach is target validation.

## Decision

1. **A new module, `nexora_simulation::interaction`, is the chain.**
   `look_direction` is the camera's forward formula; `target` casts it from the
   eye with `nexora_physics::query::raycast` and returns the block met and the
   face it was met by; `Interaction::act` turns `Break` or `Place` into
   `nexora:break_block` or `nexora:place_block` for the block, or for the cell
   in front of the face, and dispatches it through the full validation
   pipeline. Nothing writes the world except the existing handlers.
2. **The ray is shorter than the reach, by half a block's diagonal.** The
   authority measures reach to the target's centre and a ray meets a face; at
   `INTERACTION_REACH = 6 − √3/2` every block the ray offers is one the
   authority accepts. Pinned by a fan of 3,072 rays over generated terrain.
3. **The authority checks the request against a `Stance` read from the body,
   never from the request.** `Player::stance` is the eye and the body's box.
   `StanceValidator` refuses a target beyond reach of the eye
   (`TargetOutOfRange`), a placement whose cell shares volume with the body
   (`InvalidState`, BUILD-1's collision check), and any request when no stance
   is known. Touching is not sharing volume, so a block can go under the feet
   in mid-jump. The player never ends a tick inside a block, because the
   authority never puts one there.
4. **The mouse is bound as the keyboard is.** `DEFAULT_BUTTONS` binds the
   primary button to `action/break_block` and the secondary to
   `action/place_block`, on the HID Button page; one press is one request
   (`just_pressed`). They do not cross into `Walk`: they are requests to the
   authority, not movement.
5. **In the client, a click waits for a tick.** It is latched, like the jump,
   until a frame buys a world tick, and acts at the first of them, from the
   stance before the body moves. The columns an accepted edit changes are
   meshed again and uploaded with every column they touch, edge or corner, in
   the same submission as the frame that shows them — new corners on a shared
   edge must split the neighbour's quads too (DEBT-0047). The first frame is
   judged against the world as it was drawn (`AsFirstShown`): every edit after
   the capture records what the cell held.
6. **The camera keeps an in-range yaw to the bit.** Wrapping a negative yaw
   that is already in `(-π, π]` moved it by one unit in the last place, so the
   camera and the ray could disagree in the last bit. The client pins the two
   equal at every facing.
7. **The client can start from a save (`--load`)**, so what the hands changed
   is there the next time; `--seed` and `--load` together are refused.
8. **Verified where only generated terrain can show it.** The headless slice
   gives its player hands by scripted clicks, through the client's own button
   table, looking straight down — the one aim no terrain puts out of reach:
   it breaks the block underfoot and falls into the hole, is refused a block
   in its own cell, then jumps and places one under its feet and comes down
   standing on it. The cells it changed are probes the save, the reload, the
   region store and the journal's recovery must keep. A first script aimed
   60° down and failed on seed 1, where the ground ahead is out of reach; the
   straight-down script passed on 80 seed and radius combinations.

## Evidence (2026-10-05, the operator's machine)

- Headless slice, radius 2: `hands broke 1, placed 1, refused 1 in its own
  body, stood 1.00 higher on its own block`; `journal 80 edits, 77 probes
  recovered`; saves byte-identical with one worker and eight.
- Client, RX 6650 XT over Vulkan: 120 frames, the first frame **97,505 of
  97,505 judged pixels matching**; saved; loaded again with `--load` and
  judged the same; a world edited by the headless player's hands loaded and
  judged **97,552 of 97,552**.
- Tests: the ray never offers a block the authority refuses; breaking removes
  the block under the centre of the view; a placement lands in front of the
  face; a block is refused inside the body and accepted under the feet in
  mid-jump; the same actions on the same seed make the same edits; the
  validator's reach, body and missing-stance refusals; touching is not
  sharing volume. Mutation-checked: dropping the body rule, lengthening the
  ray to the full reach, counting touching as overlap and re-wrapping the
  camera's yaw each fail a test.

## Consequences

- The vertical slice's chain runs end to end in one process: a generated
  world, a player, an interaction, a state change, a save, a reload, and the
  world continues — in the headless slice by script and in the client by hand.
- The authority checks reach and occupancy, not line of sight: a client that
  sends a target behind a wall within reach is obeyed (`DEBT-0053`).
- An 80-seed sweep of the slice found seed 28 failing in the walk stage, which
  this decision did not touch: a body that landed from a fall rested one unit
  in the last place above where a body placed on the same floor rests, and a
  later jump landed on the plane exactly (`DEBT-0052`; it failed the same on
  `main`). Closed with this work: where a body stops flush against a plane,
  physics sets its centre to the plane plus or minus the half extent, as a
  placed body's is, so a resting body is one fixed point whatever brought it
  there.
- There is no crosshair and no outline on the targeted block: the hands act on
  the block under the centre of the view. Placement is always stone; there is
  no inventory. An edit outside the client's drawn band is saved and not
  drawn.

## Migration

None for saves.

## Compatibility

No save format change. `ClientConfig` gains `load`; `ClientReport` gains
`hands` and `loaded`; `Intent` gains `break_block` and `place_block`;
`SliceReport` gains the `hands_*` fields. The client's report gains a `hands`
line, and a `world load` line when it started from a save.
