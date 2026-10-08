# ADR-0039 — Terrain uses versioned fixed-point value noise with a bounded slope

- **Status:** ACCEPTED
- **Date:** 2026-10-08
- **Resolves:** `DEBT-0054`
- **Builds on:** [ADR-0035](ADR-0035-the-player-is-a-body-the-simulation-steers.md), [ADR-0038](ADR-0038-the-drawn-square-follows-the-player.md), and `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md`

## Context

Generator version 1 chooses each column's surface independently in the range
52..76. A pair of adjacent columns can differ by 24 blocks, so the player cannot
walk through the generated world to exercise the live streaming path. The
Phase 2 exit requires a small navigable world. The save header already records
`generator_version`, and the C++ reference compares `world.surface_height`
against Rust, so changing the terrain algorithm requires a versioned,
cross-stack decision.

## Decision

1. New worlds use generator version 2. Version 1 keeps its exact existing
   positional-RNG algorithm so an older save can still regenerate a missing
   chunk without changing its terrain. `World::surface_height` dispatches by
   the descriptor's generator version. `World::create_with` rejects unknown
   versions rather than silently generating them with the current algorithm.
2. Version 2 uses a 32-block lattice. At each lattice coordinate `(gx, gz)`,
   derive an independent value with the existing `terrain` positional stream
   at `(gx, 0, gz)`, then map `next_below(25)` to an integer height in 52..76.
3. For a world column `(x, z)`, use Euclidean division and remainder by 32 to
   obtain its lattice cell and non-negative fractions `fx` and `fz`. Let
   `S = 32`, and sample the four corner heights `h00`, `h10`, `h01`, `h11`.
   Compute the bilinear numerator:

   ```text
   N = h00*(S-fx)*(S-fz) + h10*fx*(S-fz)
     + h01*(S-fx)*fz + h11*fx*fz
   height = floor((N + S*S/2) / (S*S))
   ```

   The arithmetic is integer-only in both Rust and C++; no floating-point
   rounding or platform math library participates in world generation.
4. Every lattice value is in 52..76. Across one block in either axis, the
   unrounded bilinear surface changes by at most `24/32`; round-to-nearest
   therefore bounds the integer surface difference between adjacent columns
   to one block. The generated surface remains in the existing `surface_range`.
5. The C++ kernel mirrors the version-2 lattice coordinates, RNG stream,
   Euclidean coordinate handling, weighted sum, and rounding exactly. The
   `world.surface_height` conformance digest is updated for version 2.
6. The save container schema does not change. Existing version-1 saves retain
   their descriptor version and continue using the version-1 height function;
   new saves record version 2.

## Compatibility

The save format and existing chunk data remain unchanged. A version-1 world
keeps its exact height function when missing chunks are generated after load.
A version-2 world is reproducible across Rust and C++ by the conformance
digest. A build that does not support a descriptor's generator version refuses
that world instead of silently substituting terrain.

## Verification

- Pin the version-1 surface digest to the existing value before changing the
  default generator version.
- Assert version-2 heights stay in `surface_range` and adjacent columns differ
  by at most one block, including negative coordinates and lattice boundaries.
- Compare the version-2 `world.surface_height` digest across Rust and C++.
- Exercise `find_walkable_run` and the player solver on version-2 generated
  terrain, including a run that crosses chunk boundaries.

## Consequences

- New worlds become traversable by the existing one-block step rule, allowing
  a live player to exercise the streaming path.
- Version 1 remains as a compatibility path and is not retroactively restyled.
- Terrain has broad, deterministic hills at a 32-block lattice scale; erosion,
  biomes, caves, and other world-generation stages remain out of scope.
- The surface-height digest and measurements derived from generated terrain
  change and must be recorded against the new generator version.