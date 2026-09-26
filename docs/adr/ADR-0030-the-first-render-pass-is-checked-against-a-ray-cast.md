# ADR-0030 — The first render pass draws over the RHI contract, and a ray cast checks every frame it is timed on

- **Status:** ACCEPTED
- **Date:** 2026-09-26
- **Completes (partly):** `RENDERER and GRAPHICS.md` RENDER-3 (a frame),
  RENDER-8 (voxel renderer: chunk → mesh → GPU buffer → render) and RENDER-10
  (the per-chunk mesh on the device); `NEXORA TECHNOLOGY BENCHMARK PLAN.md`
  (the *frame time* metric); DEBT-0008
- **Follows:** ADR-0012 (a mesh is data), ADR-0025 (the RHI is a contract),
  ADR-0029 (the camera)
- **Touches:** a new crate `engine/render`; `engine/benchmark` (frame time);
  CI; `scripts/local-validation.py`

## Context

After ADR-0029 the freeze gate's remaining blocker listed *frame time* as
the one GPU stage with no code: *"nothing draws a frame through that camera
yet"*. Every piece a frame needs existed: a meshed 16³ region (ADR-0012), a
contract that draws with vertex layouts, uniforms and depth (ADR-0028), a
camera with reverse-Z (ADR-0029), and a native backend verified on the
operator's GPU (DEBT-0046, closed).

Three questions had no answer:

1. **Where the renderer lives, and what it may see.**
2. **What "frame time" measures**, when the benchmark must run without a
   display.
3. **How to know a frame is right**, so that a fast wrong frame cannot
   produce the best row in the table. The benchmark has refused to time a
   wrong answer since `compare-stacks.sh`.

## Options

**Checking a frame.** (a) A golden image: brittle, and it proves only that
nothing changed. (b) A second rasteriser on the CPU: it would share the
triangle setup and rounding rules it is meant to check. (c) **A ray per
pixel, walked through the voxel grid** (Amanatides and Woo) to the first
face of the drawn region. It shares no code with the mesher, the matrices,
the rasteriser or the depth test, so a mistake in any of them shows.

**Frame time.** (a) Only inside a window, which ties the number to a display
and a present mode (FIFO waits for the monitor). (b) **Headless: record,
submit and wait on one frame drawn into a texture.** Presentation is the
window probe's to measure, and waiting on a vertical blank is not rendering
cost.

## Decision

- **A new crate, `engine/render` (`nexora-render`)**, over foundation, the
  **RHI contract**, the mesh and the camera. It never names `wgpu`; the
  backend appears only in its tests. So the pass runs on the null backend
  without a GPU and on `wgpu` with one.
- **`ChunkPass`**: one pipeline (WGSL; position `Float32x3` and colour
  `Unorm8x4`, 16 bytes; a camera uniform and a chunk-offset uniform;
  reverse-Z, `Compare::Greater`). A chunk's vertices are **relative to its
  region** and uploaded once. Where the region sits relative to the render
  origin is a 16-byte uniform written each frame, so rebasing rebuilds no
  vertex buffer. **`record` appends a whole frame to one command list**:
  clear, camera, a draw per chunk the frustum keeps. The caller submits once
  and waits on one fence (Finding 29).
- **Faces are coloured by the direction they face**, six colours made of 0
  and 255. That is not art; it is what makes a frame checkable texel by
  texel. Textures come after the pass exists.
- **`nexora_render::reference`** traces the ray through a pixel and returns
  the face it meets first, the clear colour, or *not judged*. A pixel is
  judged when the ray through its centre and four rays 0.35 px around it
  agree. Rays entering the region through a face the mesher culled, because
  a neighbouring chunk occludes it, are not judged either: what shows there
  depends on a chunk this frame does not draw.
- **Frame time is `frame.draw_chunk_16`**: the meshed 16³ region the `mesh`
  suite meshes, from the same generated world, drawn at 256×256 through a
  camera, recorded, submitted and waited on. Before timing, every judged
  pixel must match the reference and at least half the pixels must be
  judged. After timing, the target must still hold the checked frame.

## Verified

- `cargo test -p nexora-render`: vertex placement against the mesher's
  plane rule; the null backend records and submits a frame, and culls a
  chunk behind the camera; on lavapipe **14,549 of 16,384 pixels judged,
  all matching**; and the same scene at the world's centre and **2^40 blocks
  out draws byte-identical frames**.
- The benchmark on lavapipe: **51,376 of 65,536 pixels judged, all
  matching**, then timed (Appendix M).
- Mutation-checked. A `Less` depth test: 10,966 of 14,549 match, and the
  benchmark refuses to time. A positive face placed on its cell's own plane:
  13,410 of 14,549. Depth cleared to 1: 10,966 of 14,549.

## Consequences

- The benchmark's *frame time* row leaves *not measured*. It is recorded as
  unmeasured only when the machine has no adapter, with the same reason as
  the RHI stage.
- `local-validation.py` drops `benchmark_gpu_stages` from the items no
  machine can validate. RHI, camera and frame time run inside
  `benchmark_cpu` on the machine's own adapter. `rendering` stays listed,
  as *partly built*, because nothing draws the world into a window yet.
- Of DEBT-0008's GPU stages, only a **window** stage in the benchmark
  remains. The engine-scale comparison is the other part of the gate, and
  it is not a GPU question.

## Not built

Textures (the first-generation albedos), cutout and transparent layers,
sorting, lighting, more than one chunk per test scene, index buffers,
drawing into a window, and anything that streams chunks into the pass.

## Migration

None. Nothing persisted refers to the renderer.

## Compatibility

Additive: a new crate, new benchmark rows, one less gated item in the local
report.
