# ADR-0035 — The player is a body the simulation steers, and the camera only follows its eye

- **Status:** ACCEPTED
- **Date:** 2026-10-04
- **Amends:** [ADR-0032](ADR-0032-the-client-is-the-runtime-in-a-window.md) (the
  free camera is gone; "a player body ... physics in the frame" is built) and
  [ADR-0029](ADR-0029-the-camera-is-reverse-z-over-a-floating-integer-origin.md)
  (target tracking and first-person mode are built; interpolation is not)
- **Informs:** `DEBT-0008` (§17's *player movement* stage), the roadmap's
  Phase 2 *player*
- **Opens:** `DEBT-0049`, `DEBT-0050`

## Context

`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17 names *player movement*
as a stage of the language gate's minimum benchmark, and the roadmap's Phase 2
lists *player*. Until now the client moved a free camera: nothing walked,
nothing fell, nothing collided. The pieces existed separately — the
`CharacterController` (ADR-0007), the input system (ADR-0018, ADR-0031), the
camera (ADR-0029), `WorldVoxels` in `nexora-simulation`, the only crate that
sees both world and physics.

The specifications fix the shape: `PLAYER SYSTEM.md` — *"a física resolve
colisões; Player decide intenção de movimento"*, the player is *"um
participante do mundo, não o centro técnico"*; `CAMERA SYSTEM.md` — the camera
does not own gameplay transforms; `INPUT SYSTEM.md` — input produces intent and
never changes the world.

## Decision

1. **The player lives in `nexora_simulation::player`.** It owns one character
   body, steered by the existing `CharacterController` — no physics change —
   and its facing (yaw, pitch). It is driven by a `Walk`: action-level intent
   with no position or velocity, clamped, a non-finite axis read as nothing.
   One `Player::tick` is exactly one world tick: one controller apply, then one
   tick of physics substeps against the terrain, which it only reads. No input
   means stand still, never "repeat the last move".
2. **The key table moves down to `nexora_simulation::controls`**, so the client
   hands in device signals and the player never sees a scancode. Space becomes
   jump; Left Shift is unbound (there is no flight).
3. **A spawn search (`nexora_simulation::spawn`) finds a walkable run** of
   columns — floor below, headroom above, read through the collision provider
   so edits count, unloaded terrain solid — in a fixed spiral and four fixed
   facings, so the same world gives the same spawn.
4. **The client derives its camera from the player's eye every frame and
   never writes it back.** The physics module joins the client's module graph,
   and the client refuses to start unless its fixed step is one world tick.
5. **Movement is intent, not a per-tick command.** A `nexora:move` command per
   tick would be the server's shape (Phase 6) bolted onto a client with no
   session; the intent is the unit the future command and replay will carry.
6. **Verified as the stages before it were:** the headless slice walks a
   scripted player (stand still to the bit, walk, jump, turn, stop at a wall)
   deterministically; the benchmark measures `player.walk_route` and checks the
   route before timing it; CI asserts the client's player walked when W was
   held.

## Evidence (2026-10-04, the operator's machine)

- Client, RX 6650 XT over Vulkan on Win32, 120 frames: the first frame, now
  drawn from the player's eye, read back from the surface — **97,505 of 98,304
  pixels judged by the ray cast, 97,505 matching, 0 snapped edges, 0 back
  faces**; the player spawned grounded and, with no key held, walked 0.00.
- Headless slice: `player walked 2.52 in 12 ticks, rose 1.23, stopped 0.30 from
  a wall, 86 ticks in all`, identical across thread counts and seeds tested.
- Benchmark: the route walks the distance twenty ticks of walking cover on
  terrain and is bit-identical when replayed, before it is timed.

## Consequences

- *Player movement* is measured; of the stages the gate's documents name, the
  cross-language tool call and the mod boundary remain (`DEBT-0008`, ADR-0034).
- The camera shows whole 20 Hz ticks, with no interpolation, and a step-up
  lifts the eye a block in one tick (`DEBT-0049`).
- The player is a body in a physics world of its own, not an entity, and it is
  not saved (`DEBT-0050`). The public surface — `spawn`, `tick`, `eye`,
  `state` — is what stays when it joins the entity store.

## Migration

None for saves. The client's free camera and its `move_up`/`move_down` keys
are gone.

## Compatibility

No save format change. `nexora-client`'s report gains a `player` line.
