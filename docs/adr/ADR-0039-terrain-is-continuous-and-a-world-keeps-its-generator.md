# ADR-0039 — Terrain is continuous, and a world keeps the generator it was made with

- **Status:** ACCEPTED
- **Date:** 2026-10-06
- **Amends:** the terrain of the Phase 0 world (`engine/world`,
  `World::surface_height`), and its C++ reference in `benchmarks/cpp`
- **Builds on:** `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md`
  (rule 1: a change of algorithm changes the generator version; rule 3: a
  seed is immutable once the world exists),
  [ADR-0004](ADR-0004-save-container-format-v1.md) (worlds written today
  remain readable), [ADR-0038](ADR-0038-the-drawn-square-follows-the-player.md)
  (the band a client draws comes from `World::surface_range`)
- **Resolves:** `DEBT-0054`

## Context

The Phase 2 exit is *"mundo pequeno navegável e carregamento/unloading
estável"*. Since ADR-0038 the drawn square streams with the player, and
loading and unloading are proven by test, but the world was not navigable:
the Phase 0 generator drew each column's height on its own, 64 ± 12, so
neighbouring columns differed by up to 24 blocks. A jump rises 1.23 and the
character steps up 1.0; a player could not reach the next column on foot.
The client found its start by searching for five walkable columns in a row
precisely because walkable ground was the exception. `DEBT-0054` named the
remedy — continuous noise, integer arithmetic so it stays identical across
platforms and between Rust and C++, bounded slope — and that it needs its
own decision.

## Decision

1. **Generator version 2 is continuous value noise, in integers.** Three
   octaves of heights are drawn on square lattices 64, 16 and 8 blocks
   apart, within ± 8, ± 3 and ± 1 blocks, in 1/256 of a block, from the
   terrain stream at `y = octave + 1`. Each is interpolated bilinearly
   between its lattice points in exact integers and floored once; the sum is
   floored to blocks around 64. No floating point is involved anywhere.
2. **Neighbouring columns differ by at most one block — by construction,
   checked at compile time.** Bilinear interpolation moves at most
   `2a/s` per block for an octave of amplitude `a` on spacing `s`:
   0.25 + 0.375 + 0.25 = 0.875, plus less than 1/256 of rounding per octave,
   is under a block, and two values under a block apart floor to integers at
   most one apart. A `const` assertion in `engine/world/src/terrain.rs` holds
   the octave table to it. The amplitudes sum to 12, so the surface stays in
   version 1's range (52 to 76) and `World::surface_range` — every client's
   band — is unchanged.
3. **A world keeps the generator it was made with.** `GENERATOR_VERSION`
   becomes 2 for new worlds; version 1 stays in the build, and
   `surface_height` and `generate_chunk` dispatch on the version recorded in
   the world's descriptor (already saved in its header). A world saved under
   version 1 goes on growing version 1's columns wherever it is extended, so
   it never meets a seam where the algorithm changed under it. A world made
   by a version this build does not have is refused (`Recovery::Reject`)
   rather than grown with a seam.
4. **The C++ reference follows.** `benchmarks/cpp` reproduces version 2, and
   the cross-stack conformance — `world.surface_height` and the generated
   chunk's digests — must agree with Rust before any timing means anything.
5. **A chunk draws each lattice point once.** `generate_chunk` computes its
   heightmap over the whole chunk with each octave's lattice points drawn
   once (38 for a 32 × 32 chunk, against 12,288 drawn column by column); a
   test holds the area path to the single-column path, including far out
   and across zero.

## Evidence (2026-10-06, the operator's machine)

- **Navigable, proven by walking.** A player holding W for 200 ticks walks
  more than 35 blocks straight on, in each of the four directions, on four
  seeds (seed 28 included), climbing and descending as it goes, never pushed
  out of terrain. With the generator set back to version 1 the same test
  walks 0.20 blocks and fails.
- **The bound.** Over 300 × 300 columns on five seeds, no two neighbours
  differ by more than one block; over 512 × 512 the relief spans at least 8
  blocks inside 52..76 (it is not flat).
- **Compatibility.** A version 1 world saved, loaded and extended by a chunk
  it had not generated keeps version 1's columns, grass on top and air above.
- **Cross-stack.** Rust and C++ (g++ 16.1) agree on all 12 conformance
  digests, `world.surface_height` = `0xbae8424cbede1cc1`.
- **Headless slice.** 80 of 80 seed and radius runs pass; saves and region
  stores are byte-identical across 1 and 8 workers. The walk stage's player
  now walks 443 ticks before it reaches the edge of the loaded area (86 on
  the pillars), and the default slice's world save fell from 115,218 to
  13,812 bytes: continuous ground compresses.
- **Client, RX 6650 XT.** On the new terrain the textured first frame holds
  97,143 of 97,143 judged pixels, no snapped edge or texel; a second run on
  the same `--world` file resumes the world and the player where they were
  saved and draws the same frame.
- **Tests.** The workspace suite passes. One test changed its expectation,
  and the change is a correction: the transparency test in
  `engine/simulation/src/surfaces.rs` expected two faces "either side of the
  shared plane" between stone and glass, and got them only because a version
  1 pillar beside its row put terrain dirt — glass in that test's table —
  against the glass cell. The shared plane exposes exactly one face, the
  stone's behind the glass; the row now stands clear of every neighbouring
  column.

## Consequences

- The Phase 2 exit's *navegável* is met: a player crosses columns on foot in
  any direction, and the client's streaming (ADR-0038) is exercised by
  walking, not only by test.
- Every number that depends on the shape of the terrain moved — the
  slice's walk length and save size, the client's judged pixel counts, the
  benchmark's generation timings and the C++ digests. Historical baselines
  in `docs/benchmarks` were measured on version 1 and stay as they were;
  measurements from here on are on version 2.
- `find_walkable_run` still finds the client's start; on version 2 the first
  column it tries is almost always walkable.
- Version 1 is kept for as long as saves made with it are supported. Caves,
  overhangs, biomes and steeper relief are later generator versions, each a
  new number under the same rule.

## Migration

None needed. A save records its generator version; version 1 saves load and
grow as version 1. No save format change.

## Compatibility

`GENERATOR_VERSION` is 2, and the new `GENERATOR_VERSIONS` lists the versions
this build can generate. `World::create_with` refuses a descriptor whose
version is not among them. `World::surface_height` and
`World::generate_chunk` keep their signatures; their output for a new world
changes.
