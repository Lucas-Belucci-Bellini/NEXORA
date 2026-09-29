# ADR-0029 — The camera is reverse-Z, over a floating integer origin

- **Status:** ACCEPTED
- **Date:** 2026-09-26
- **Completes (partly):** `CAMERA SYSTEM.md` (view and projection state,
  perspective and orthographic, large-world precision); `RENDERER and
  GRAPHICS.md` RENDER-5 (camera, frustum, camera-relative coordinates),
  RENDER-6 (floating origin) and RENDER-7 (coordinate systems);
  `SPATIAL AND COORDINATE SYSTEM.md` (render space, rebasing)
- **Amends:** ADR-0028 (two depth comparisons; the depth convention moves out
  of the RHI)
- **Touches:** a new crate `engine/camera`; `engine/rhi` (`Compare`);
  `engine/rhi-wgpu` (the comparison, the `bound` proof); `engine/benchmark`
  (the camera stage); CI

## Context

The benchmark plan's slice runs `RHI → camera → 16³ chunk`. The RHI draws and
presents on real hardware now (local reports 4 and 5), and since ADR-0028 a
draw can read a uniform, so a view-projection matrix has somewhere to go.
Nothing produced one. The freeze gate's remaining blocker lists the camera
among the benchmark stages with no code (DEBT-0008).

Four things have to be settled before any shader reads a camera, because
each is baked into every shader and every depth buffer:

1. **Precision.** The world runs to 2^40 blocks in `f64` and `i64`
   (`SPATIAL AND COORDINATE SYSTEM.md`). A GPU works in `f32`, whose step at
   2^40 is 131,072 blocks. `f32` world coordinates are not an option.
2. **Depth.** The RHI has one depth format, `Depth32Float` (ADR-0028).
3. **Handedness and clip space.** WGSL, Direct3D and Metal share
   `x, y ∈ [-1, 1]` and depth `∈ [0, 1]`; the view convention is ours to pick.
4. **Where it lives.** `CAMERA SYSTEM.md`: the camera derives presentation
   from authoritative state and owns none of it.

## Options

**Precision.** (a) Camera-relative `f32`: subtract the camera position every
frame, which moves every static mesh every frame. (b) A floating origin that
is itself `f64`: rebasing is exact, but a block's offset from the origin is a
difference of two `f64` values. (c) **A floating origin that is an integer
block.** A block's offset is an integer difference, exact in `f32` below
2^24; a position's offset is an `f64` subtraction, exact by Sterbenz's lemma
when the two are close, rounded to `f32` once.

**Depth.** (a) Forward Z, near at 0: a float's precision is densest at 0, and
a perspective projection puts almost all of `[0, 1]` next to the near plane,
so both crowd the same end, and a voxel world z-fights at a few hundred
blocks. (b) **Reverse-Z, near at 1, far at 0**: the float's dense end meets
the projection's sparse one. This is the standard remedy for a `Depth32Float`
target, and it needs a `Greater` comparison the RHI did not have.

## Decision

- **A new crate, `engine/camera` (`nexora-camera`), over the foundation
  only.** It turns a camera into a `CameraState` (view, projection,
  view-projection, frustum, eye), and knows nothing about the RHI or
  `wgpu`. To the RHI a view-projection is 64 bytes in a uniform buffer.
- **Render space is the world minus an integer `RenderOrigin`.** Block
  offsets are exact up to `EXACT_OFFSET` (2^24) and refused beyond it.
  Positions are subtracted in `f64` and rounded once. The origin follows the
  camera when it drifts more than `REBASE_DISTANCE` (4,096 blocks, where an
  `f32` step is under half a millimetre), and `Camera::sample` refuses an
  origin the camera has outrun. Rebasing moves the origin and never a world
  coordinate. Meshes stay in their section's local space, so a rebase
  changes per-draw offsets, never vertex buffers.
- **Conventions.** World and render space: `+Y` up. View space:
  right-handed, the camera looks down `-Z`. Clip space: `x, y ∈ [-1, 1]`,
  `+Y` up, **depth reverse-Z**. Matrices: column-major `f32`, the layout of
  WGSL's `mat4x4<f32>`, **built and multiplied in `f64` and rounded once**.
  Yaw turns about `+Y` (zero looks down `-Z`), pitch is clamped to ±89.9°.
- **The frustum is extracted from the view-projection**, in `f64`, so it
  cannot disagree with the projection. Culling is conservative: it may keep a
  box it did not need to, never drop one with a visible point.
- **The RHI gains `Compare::Greater` and `Compare::GreaterEqual`**, and
  stops saying which end of `[0, 1]` is far. That is the projection's
  convention. The engine's is: clear depth to `0`, test `Greater`.
  `nexora_rhi_wgpu::proof::bound`, the depth proof the probes and the local
  report run, uses the engine's convention from now on.
- **The benchmark measures the camera stage** at 2^40 blocks: resolving a
  camera, and testing a radius-12 observer's 625 columns against its
  frustum. It checks that the target lands mid-screen, and that culling keeps
  some columns and drops others, before it times anything.

## Verified

- `cargo test -p nexora-camera`: near maps to depth 1 and far to 0; depth
  falls with distance; `+X` is right and `+Y` up on screen; yaw, pitch,
  `look_at` and clamping; block offsets exact and positions within 2^-11 of a
  block, at the world origin and at ±2^40; the same scene at the centre and
  the edge of the world produces **bit-identical** matrices; the frustum
  agrees with the projection on 20,000 points; culling never drops a column
  with a visible point, checked by brute force over 625 columns.
- `nexora-rhi-probe` on lavapipe: `bound 256 of 256 texels sampled, 256 kept
  by depth, 256 tinted by a uniform`, now with reverse-Z.
- Mutation-checked. Mapping `Greater` to `Less` in the backend fails the
  probe (0 of 256). Swapping the frustum's near plane for a forward-Z one
  fails two camera tests. A forward-Z projection coefficient fails four.

## Consequences

- The benchmark's `camera` row leaves *not measured*. Frame time is what
  remains of DEBT-0008's GPU stages, and it needs a pass that draws the
  meshed chunk through this camera.
- Everything a shader will read about space is now fixed in one place. A
  renderer that wants another convention needs an ADR, not a local change.

## Not built

Target tracking, first- and third-person modes, interpolation between
simulation ticks, collision-aware placement, cinematic cameras and jitter
(`CAMERA SYSTEM.md`, RENDER-5). They need a player, a frame clock and a
renderer, none of which exists yet.

## Migration

None. Nothing persisted refers to a camera or a depth convention.

## Compatibility

`Compare` gains two variants. It is not `#[non_exhaustive]`, so an
exhaustive `match` outside this repository would break; the one in this
repository is updated. `ClearValue::Depth`'s documentation no longer says
`1.0` is far.
