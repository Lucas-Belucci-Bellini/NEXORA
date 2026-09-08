# ADR-0012 — A mesh is a data structure, not a picture

- **Status:** ACCEPTED
- **Date:** 2026-09-08
- **Corrects:** the benchmark's claim that mesh generation could not be measured
  because there was *"no renderer to consume a mesh"*
- **Follows:** [ADR-0007](ADR-0007-physics-collides-against-a-provider-not-the-world.md),
  [ADR-0008](ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md),
  [ADR-0010](ADR-0010-commands-are-intent-and-carry-their-own-authority.md)
  (fourth use of the same boundary shape)

## Context

`RENDERER and GRAPHICS.md` **RENDER-9** asks for face culling and greedy
meshing; **RENDER-10** defines the per-chunk mesh and its states; **RENDER-11**
gives the rebuild pipeline `block changed → chunk dirty → neighbour check →
remesh → GPU update`; **RENDER-12** wants meshing off the main thread.
`CHUNK & VOXEL ENGINE.md` **CHUNK-27** and **CHUNK-28** require neighbour access
across chunk borders, naming meshing as the first reason.

Meshing is a Phase 2 item in `NEXORA DEVELOPMENT ROADMAP.md`, and the benchmark
plan lists it as a stage of the vertical slice.

## Problem

**The blocking claim was mine, and it was wrong.** The benchmark listed mesh
generation as unmeasurable with the reason *"no renderer to consume a mesh"*,
which put it in the same bucket as the window and the RHI — blocked on hardware
this container does not have.

That conflates two different things. **Displaying** a mesh needs a renderer.
**Building** one does not, and neither does checking that it is right:

- a solid region must emit only its outer shell, never its interior;
- merging must change how many rectangles describe a surface, never how much
  surface there is;
- a face on a chunk border must be culled by the neighbouring chunk;
- the same voxels must always produce the same geometry.

Every one of those is a property of a data structure. Of RENDER-11's five
arrows, only `→ GPU update` needs hardware.

Two real problems remain underneath the false one. **Layering**: a mesher that
could reach a `World` would be a presentation-layer component mutating, or at
least depending on, authoritative state. **Scope**: RENDER-10 asks for four
render layers, and `BlockDefinition` carries no render layer at all.

## Options

1. **Leave it blocked until a renderer exists.** Defers a Phase 2 item behind a
   Phase 5 one, on a reason that does not hold.
2. **Mesh inside `nexora-world`.** Convenient, and it puts geometry in the
   storage crate.
3. **A crate depending on foundation only, reading voxels through a trait.**
4. **Build all of RENDER-9 and RENDER-10**, including the four render layers.

## Decision

**Option 3**, scoped to what the block data actually supports.

`nexora-mesh` declares `trait VoxelView` and depends on `nexora-foundation`
alone. `nexora-simulation` holds `WorldSurfaces`, the adapter. This is the
fourth system built this way — physics, streaming, commands, now meshing — and
it is now the default answer for a boundary the documents state in prose.

### The sweep walks planes, not cells, and borders fall out for free

For each axis the mesher walks the `n + 1` planes *between* cells rather than
the `n` cells. At each plane the cell below and the cell above decide what
happens, and the first and last plane of a region reach one cell outside it —
where the view answers, per CHUNK-28.

So a seam between two solid chunks produces nothing, with no "am I at the edge"
branch anywhere. There is a test that meshes an 8×8×16 slab as two 8-cubes and
asserts the shared plane emits zero faces, and a stronger one asserting the two
halves cover exactly the same area as meshing the slab whole.

### An absent neighbour occludes

The adapter's most consequential line. If a non-resident chunk read as *empty*,
every face along that border would survive culling and then vanish the moment
the chunk loaded — a visible flash at every streaming boundary, and a mesh built
against data that was not there. So absent cells occlude: faces appear when data
does, never before. Physics makes the same call for the same reason and lands on
`solid`; ADR-0007 records it as a world decision living in the adapter, and this
is the same decision for the same reason.

### Only one opacity class, because that is all the data supports

RENDER-10 wants opaque, cutout, transparent and water meshes. `BlockDefinition`
carries `solid` and nothing else — there is no way to know that glass is
see-through. Building four layers would mean inventing block data, which the
startup brief §3 forbids.

So `VoxelView::occludes` defaults to "anything present hides what is behind it"
and is a **trait method a view can override**. A block system that grows render
layers overrides one function; nothing else changes. `DEBT-0026`.

### `Axis` moved to the foundation

The mesher needs an axis; physics already had one. A third copy would have meant
three places to disagree about what "the other two axes" means, so `Axis` is now
`nexora_foundation::spatial::Axis` and physics re-exports it. The *policy* that
was tangled with it — the order collision resolves axes in, vertical first —
stays in physics as `math::RESOLUTION_ORDER`, because that is a physics decision
and not a property of axes.

## Consequences

- **Mesh generation moves from the unmeasured list to the measured one**, and
  the reason that put it there is corrected rather than quietly deleted.
- **The measurement found that the mesher is not the slow part.** The same
  workload against the world and against a pre-read dense array:

  | | median |
  | --- | ---: |
  | `mesh.region_16` | **3.30 ms** |
  | `mesh.region_16_from_snapshot` | **295.45 µs** |

  **11.2× — so 91% of meshing is the world lookup**, not culling and not
  merging. `mesh.cull_only_16` at 3.20 ms says the same thing from the other
  side: merging is about 3% of the total. This is finding 10 again (49% of a
  physics step is the voxel lookup), much more extreme.

  The consequence is architectural, not just an optimisation note: **the fix for
  meshing speed and the mechanism for RENDER-12's async meshing are the same
  thing.** A region snapshotted into a dense array can be meshed on a worker
  thread without touching the world at all. `DEBT-0029` records it with the
  number.
- **Culling and merging earn their place, measured on real terrain**: 24,576
  cube faces → **2,471** visible after culling → **807** rectangles after
  merging → 3,228 vertices. Recorded as quantities rather than a ratio, because
  the ratio depends entirely on how smooth the ground is and quoting one as a
  property of the mesher would be wrong.
- **A region buried in solid rock meshes to nothing, and there is a test saying
  so.** Not a degenerate case — the expected answer, and the reason culling is
  worth having. My first version of that test asserted the opposite and failed;
  the test was wrong, not the mesher.
- **`ChunkMesh` reports vertices but does not produce them.** The vertex format
  belongs to a render backend, and this crate has no business choosing one.
- **Async meshing (RENDER-12) and LOD meshes are not built**: `DEBT-0027`,
  `DEBT-0028`.

## Migration

None. Nothing meshed before. The `Axis` move is source-compatible for every
existing caller — physics re-exports the type, and the two `RESOLUTION_ORDER`
call sites were updated.

## Compatibility

No save format change and no public API break: meshes are derived data, rebuilt
from voxels, and are never persisted. `NEXORA SAVE FORMAT AND COMPATIBILITY.md`
requires exactly that — a save must not depend on caches or derived data.
