# ADR-0007 — Physics collides against a provider, not against the world

- **Status:** ACCEPTED
- **Date:** 2026-09-07
- **Amends:** [ADR-0005](ADR-0005-phase-0-scope-boundary.md) (physics moves from
  "deliberately not implemented" to implemented)
- **Extends:** [ADR-0003](ADR-0003-workspace-layout-enforces-dependency-matrix.md)
  (adds two crates to the graph that enforces the dependency matrix)

## Context

`PHYSICS.md` closes with the rule the whole design turns on:

> A Física deve conhecer **massa, posição, velocidade, colisão e forças**. Ela
> **não** deve conhecer *"isso é uma espada"*, *"isso é um aldeão"*, *"isso é
> minério"*.

§11 (PHY-10) asks for terrain to arrive through a `VoxelCollisionProvider`
rather than through a direct reach into the world runtime.
`NEXORA DEPENDENCY MATRIX.md` places Simulation above World and rule 1 requires
a dependency to point at *"uma abstração estável, não para estado privado"*.

Meanwhile `NEXORA TECHNOLOGY BENCHMARK PLAN.md` lists `physics` as a stage of
the vertical slice the language-selection gate measures — the last such stage
that needs neither a GPU nor a second language in the build.

## Problem

Physics needs to know whether a cell is solid. The world knows. Connecting them
the obvious way — `nexora-physics` depends on `nexora-world` — makes the solver
unusable without a generated world, drags block registries and chunk lifecycle
into every physics test, and puts world concepts inside the one crate that the
document says must not have them.

Connecting them the other way — the world calls physics — inverts the documented
layering and would let a storage crate depend on a simulation one.

## Options

1. **`nexora-physics` depends on `nexora-world`.** Direct, and every physics test
   then needs a world, a seed, a registry and a generated chunk.
2. **`nexora-world` depends on `nexora-physics`.** Inverts the matrix; the
   storage layer would import a simulation layer.
3. **A trait in physics, implemented above both.** Physics defines
   `VoxelSource`; a crate that is allowed to see both sides implements it.
4. **Generic parameter threaded through the world.** Same coupling as option 1,
   spread across more signatures.

## Decision

**Option 3**, with the implementation living in a new `nexora-simulation` crate.

```text
nexora-foundation ──► nexora-physics ─┐
                                      ├──► nexora-simulation ──► headless, benchmark
nexora-foundation ──► nexora-world ───┘
```

`nexora-physics` declares `trait VoxelSource { fn shape_at(&self, BlockPos) -> VoxelShape }`
and depends on `nexora-foundation` **only**. `nexora-simulation` is the single
crate allowed to see both the world and physics, and holds `WorldVoxels`, the
adapter between them.

Cargo enforces this: `nexora-physics` cannot reach the world because the
dependency is not declared, and adding it would be a visible change to a
manifest rather than an import nobody notices in review.

### The provider contract carries two policies

Both are world decisions, so both live in the adapter and not in the solver:

| question | answer | why |
| --- | --- | --- |
| below the world's floor? | **solid** | a body must not fall out of the bottom of the world |
| chunk not resident? | **solid** | refusing movement is the safe failure; "empty" lets a body walk into terrain that has not arrived and be ejected when it does |

### What was built, and what was not

Implements `PHYSICS.md` items PHY-0 to PHY-14 and PHY-22: fixed timestep, rigid
bodies, AABB colliders, materials with two-surface contact combination,
configurable gravity, forces and impulses, swept voxel collision, character
control with step-up and ground detection, sleeping, and ray queries.

Deliberately **not** built, each recorded in the debt register with a trigger:
body-versus-body collision, rotation and angular velocity, non-cube shapes,
buoyancy, vehicles, structural analysis, destruction. `PHYSICS.md`'s own build
order (§47) puts them later, and the startup brief §52 forbids reaching for them
before the ones underneath are measured.

### Axis-at-a-time resolution, in a fixed order

Collision resolves one axis at a time against the box as the previous axes left
it, in the order **Y, X, Z**. Sweeping the whole motion vector at once and
stopping at the first contact would freeze a body against a wall it is sliding
along; separating the axes is what produces sliding, and it is exact, because
every contact plane between an axis-aligned box and a cube is axis-aligned.

Vertical first, so that landing and walking happen in the same step. A different
order is defensible; a *varying* order is not, because axis order changes the
answer in a corner and `NEXORA REPLAY AND DETERMINISM.md` requires the same
inputs to produce the same state.

### Two mechanisms guard the floating-point boundary

Found by a test rather than by reasoning: `1.9 - 0.9` is `0.9999999999999999`, so
a 1.8 m body standing on a block is — as a bit pattern — a fraction of a unit in
the last place *inside* it. The sweep read that as "already past the surface",
looked only at the cells below, and let the body fall through the floor it was
standing on.

1. **Plane snapping.** Which cells a box occupies is decided from coordinates
   snapped to the grid within a tolerance that scales with magnitude, because
   `NEXORA SPATIAL AND COORDINATE SYSTEM.md` allows positions out to 2^40 where
   a `f64` step already exceeds any fixed epsilon. How far a body may *move* is
   still measured from the real coordinate, so snapping corrects a position
   rather than teleporting it.
2. **Depenetration.** A body can be inside terrain without the solver ever
   having moved it there: a block placed where it stands, a chunk generating
   around it, a save restoring a position the world no longer agrees with. The
   sweep alone cannot recover — it measures motion into *new* cells, and a body
   already inside one has no new cell to enter — so recovery is an explicit
   pass that pushes the body out the shortest way, bounded, and reports being
   stuck rather than flinging it somewhere arbitrary.

## Consequences

- The same solver runs against the real world, a flat test fixture and a
  synthetic benchmark terrain, and none of the three knows about the others.
  Every physics unit test is a few lines with no world setup.
- The benchmark can measure the solver **with and without** the world lookup.
  It does, and the answer matters: **49% of a 1,000-body step is the voxel
  lookup**, not the solver (Appendix B, finding 10). A single combined number
  would have sent the optimisation work to the wrong half.
- One extra crate, `nexora-simulation`, holding one adapter today. That is the
  cost of making the composition point a library instead of a binary — the
  headless slice and the benchmark both use it rather than each writing their
  own bridge.
- A new `Domain::Physics` in the foundation error taxonomy, so physics failures
  are attributable in diagnostics rather than borrowing `Domain::World`.
- Slabs, ramps and stairs arrive as variants of `VoxelShape`, which is
  `#[non_exhaustive]` for exactly that reason. Until they exist, the only
  climbable geometry is a whole cube, so the character preset's step height is
  `1.0` rather than the conventional `0.6`: a sub-block step height against
  cube-only terrain is a setting that can never climb anything.

## Migration

None. Nothing collided before, and physics writes nothing to the world — the
headless slice's byte-identical save comparison across thread counts still
passes with the physics stage in place, and a test now asserts precisely that.

## Compatibility

No save format change: bodies are not persisted in this phase. An entity's
transform and velocity already persist through `nexora:save/entities`, and the
mapping from an entity to a body is the caller's, so adding body persistence
later is additive rather than a format break.
