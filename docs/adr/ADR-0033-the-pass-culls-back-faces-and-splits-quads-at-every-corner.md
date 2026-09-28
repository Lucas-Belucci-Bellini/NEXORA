# ADR-0033 — The pass culls back faces, and splits its quads at every corner

- **Status:** ACCEPTED
- **Date:** 2026-09-28
- **Closes:** DEBT-0047
- **Amends:** ADR-0028, whose `PipelineDesc` gains a cull mode; ADR-0030,
  whose "six vertices a quad" becomes "whole triangles, at least two a quad";
  ADR-0032, whose first-frame rule tightens
- **Touches:** `engine/rhi` (`Cull`), `engine/rhi-wgpu` (the mapping and a
  pixel proof), `engine/render` (`Corners`, winding, `ChunkPass`),
  `engine/client`, the benchmark's frame stages, CI

## Context

The client's first frame (ADR-0032) had 0 to 5 wrong pixels in about 79,500
judged, on every one of twenty seeds. Dissected, they were:

- **T-junctions.** Greedy meshing puts a corner of one rectangle in the
  middle of a neighbour's edge. The rasteriser rounds the long edge and the
  two short ones separately, and a ray slips through the gap. On one pixel
  the gap was 0.0034 px from a shared edge.
- **Silhouette ties.** After the rasteriser snaps vertices to its sub-pixel
  grid, a pixel centre can sit exactly on a silhouette edge. The front face
  and the back face behind it share that edge but wind opposite ways, so the
  top-left rule can give the pixel to the back face.

The pass drew back faces, because the RHI contract had no way to discard
them. So both cases showed a face seen from behind. DEBT-0047 recorded this,
and ADR-0032's rule tolerated it (back faces or snapped edges, together at
most 1 in 5,000).

## Options

**Culling alone.** It removes the silhouette ties, and it halves the fragments
the pass shades. Measured with culling on and no splitting, **8 of 20 seeds
fail**: through a T-junction gap the ray now reaches a *front* face farther
away, which the reference cannot explain. Culling hides the symptom of a
gap; it does not close the gap.

**Splitting alone.** It closes the T-junction gaps (42 wrong pixels over 20
seeds fall to 28), but the silhouette ties remain.

**Expanding each quad by an epsilon.** This trades gaps for overlap, depends
on distance, and would make the frame disagree with the reference in new
places.

**Both culling and splitting.** Chosen.

## Decision

- **The contract gains `Cull { None, Back }` in `PipelineDesc`.**
  - The front of a triangle is where its vertices run counter-clockwise in
    normalized device coordinates (x right, y up), which is the default on
    all three APIs as `wgpu` exposes them.
  - `rhi-wgpu` maps `Back` to `cull_mode: Back` with `FrontFace::Ccw`.
  - Every existing pipeline says `None`, and behaves exactly as before.
- **`ChunkPass` culls back faces.** `chunk_vertices` winds every triangle
  counter-clockwise seen from the side the face points to. The quad order
  turns from the first of the axis's other two axes to the second, which is
  counter-clockwise about their cross product (`Y×Z=+X`, `X×Z=−Y`, `X×Y=+Z`),
  so the order is reversed where that product points against the facing.
- **Quads are split at every corner that lies on their edges.**
  - `Corners::of(meshes)` indexes every quad corner of the meshes drawn
    together, in world blocks, by the axis-aligned lines it lies on. It
    includes the other chunks' meshes, so the seams between regions are split
    too.
  - A quad with no corner on its edges stays two triangles. A quad with
    corners on its edges becomes a fan through all of them: from a corner
    whose two sides have none (`n − 2` triangles), otherwise from its centre
    (`n` triangles; the centre is a half-integer, exact in `f32`).
  - `ChunkPass::upload` takes the `Corners`.
- **The client's first frame now tolerates no back face at all.** With
  culling, a back face on screen means culling or winding broke. Only
  snapped edges remain tolerated: a colour the ray cast itself finds within
  1/64 px of the pixel's centre, at most 1 in 5,000 judged pixels. The
  benchmark's stages keep requiring every judged pixel to match.

## Verified

- **Client, 20 seeds, lavapipe, 384×256:**

  | pass | wrong pixels (sum over 20 seeds) | failing seeds |
  | --- | ---: | ---: |
  | before this ADR | 42 (back faces and snapped edges) | 0, under ADR-0032's tolerance |
  | splitting only | 28 | 0, under ADR-0032's tolerance |
  | culling only | — | **8**: gaps showed farther front faces |
  | **both** | **10**, all snapped edges, **0 back faces** | 0 |

  13 of the 20 seeds show no wrong pixel at all.
- **`nexora-rhi-probe`:** `cull 16 of 16 texels shaded counter-clockwise, 0
  clockwise`, read back from the device. It is also a native test.
- **Unit tests:**
  - every facing of every axis winds counter-clockwise from outside;
  - a corner in the middle of an edge splits it and keeps the area;
  - a corner across a region seam splits the edge, in world coordinates;
  - a face split on all four sides fans from its centre.
- **Mutation-checked:**

  | mutation | caught by |
  | --- | --- |
  | winding reversed | the unit test, and the client frame (22,165 back faces) |
  | no splitting | three unit tests; the client fails 8 of 20 seeds |
  | `ChunkPass` not culling | the client accuses back faces on 4 of 6 seeds |
  | the backend ignoring `Cull` | the probe (16 of 16 back texels drawn) |

- **Cost, lavapipe, benchmark smoke:**

  | row | before | after |
  | --- | ---: | ---: |
  | `frame.chunk_16_vertices` | 4,842 | 7,377 (+52%) |
  | `frame.draw_chunk_16` | 2.01 ms | **1.71 ms** |

  Culling halves the fragments, which outweighs the extra vertices on this
  rasteriser. The client's nine columns go from about 319,000 vertices to
  487,000, also +52%. On a discrete GPU the vertex side may weigh more. That
  is for a local report to measure, and index buffers (ADR-0028) are the
  lever if it does.

## Consequences

- DEBT-0047 closes. What remains, the snapped edges, is the reference being
  exact where the rasteriser is not. It is bounded, and it is not a gap.
- `FrameCheck::backfacing` stays in the reference. It is now an error class.

## Not built

Index buffers, which would give back most of the extra vertices; splitting
against the corners of chunks streamed in later (the client's scene is fixed);
and culling in any pipeline but the chunk pass.

## Migration

None. Nothing persisted refers to vertices or pipelines.

## Compatibility

A new, required field in `PipelineDesc`. Every pipeline in the workspace sets
it. `ChunkPass::upload` gains a parameter.
