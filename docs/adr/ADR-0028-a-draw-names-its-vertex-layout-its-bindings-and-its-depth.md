# ADR-0028 — A draw names its vertex layout, its bindings and its depth

- **Status:** ACCEPTED
- **Date:** 2026-09-26
- **Completes (partly):** `RENDER HARDWARE INTERFACE.md` — *Responsibilities*
  (samplers, shader resource binding); `RENDERER and GRAPHICS.md` RENDER-1
  (`GPUSampler`) and RENDER-5 (a camera needs a view-projection the shader can
  read)
- **Amends:** ADR-0025 (two conformance cases, eleven in all); ADR-0026 (the
  interim vertex rule is gone)
- **Touches:** `engine/rhi` (descriptors, commands, shared rules,
  conformance), `engine/rhi-wgpu`, every caller that builds a pipeline or a
  draw

## Context

The RHI contract of ADR-0025 could draw one thing: vertices whose only
attribute is a position, into a colour target, with no data other than the
vertices. ADR-0026 said so and recorded an interim rule in DEBT-0046. The
native backend read the position as up to four `f32`, **chosen by the
stride**, because the contract had no way to say more.

That was enough to prove a driver draws. It is not enough for the next stage
of the plan. `RENDER HARDWARE INTERFACE.md` lists *samplers* and *shader
resource binding* among the RHI's responsibilities, and neither existed. The
benchmark's remaining GPU stages (DEBT-0008) are *camera* and *frame time*. A
camera is a view-projection matrix the vertex shader reads, and nothing could
read one. A voxel mesh needs a texture coordinate beside its position, and a
vertex could not hold one. Drawing a world without a depth test draws it in
submission order.

## Problem

Three things are missing, and each is a rule a backend could disagree on:

1. **What a vertex holds.** Guessing from the stride works for exactly one
   attribute.
2. **What a draw reads besides vertices.** Uniforms and textures, and how a
   texture is filtered.
3. **Which fragment is nearest.** A depth test, a depth texture, and a way to
   reset it every frame.

## Options

1. **Keep the contract, and let the renderer bake everything into vertices.**
   No camera without re-uploading every vertex each frame. Rejected: it is
   the workaround the stride rule already was.
2. **Expose the backend's own binding model** (`wgpu` bind groups,
   descriptor sets). Rejected: the contract would take its shape from its
   first backend, which ADR-0025 exists to prevent.
3. **Declare it in the pipeline, supply it in the draw.** A pipeline lists
   its vertex attributes, its binding slots by kind, and its depth test. A
   draw supplies one resource per slot and a depth texture when there is a
   depth test. A `Clear` command resets a colour or depth target. Every rule
   is checked in `rhi::kit`, identically for every backend, and held to the
   conformance suite.

## Decision

Option 3.

| Addition | Rule, checked for every backend |
| --- | --- |
| `VertexAttribute { location, format, offset }`, `VertexFormat` (`float32`–`float32x4`, `unorm8x4`, `uint32`) | one to eight attributes; unique locations below eight; each offset four-byte aligned and the attribute inside the stride |
| `BindingKind::{Uniform, Texture, Sampler(Filter)}`, slot `n` = `@group(0) @binding(n)` | at most eight slots; a draw supplies exactly one `Binding` per slot, of the slot's kind |
| uniform bindings | `UNIFORM` usage; size a multiple of 16 bytes (`UNIFORM_ALIGNMENT`, the `vec4` alignment WGSL, HLSL and MSL share) |
| texture bindings | a `SAMPLED` colour texture, **never the draw's own target** (a feedback loop no backend defines) |
| samplers | owned by the pipeline, which declares the filter (`Nearest` keeps 16×16 art exact); a draw names the slot with `Binding::Sampler` |
| `DepthState { compare, write }`, `Draw::depth` | a draw has a depth texture **exactly** when its pipeline tests depth: a `Depth32Float` render target the size of the colour target |
| `Command::Clear { texture, value }` | a render target; a colour for a colour texture, a depth for a depth texture; every value finite and in `[0, 1]` |

The null backend keeps all of it without a GPU. `WgpuRhi` builds an explicit
bind group layout and pipeline layout from the slots, a sampler per sampler
slot, the vertex layout from the attributes, and a `Depth32Float` depth
stencil state. A draw becomes a bind group plus a render pass with the depth
attachment; a clear is a render pass with a clear load and no draws.

A shader that reads its bindings differently from the layout is caught by
`wgpu`'s validation when the pipeline is created, and refused with `Reject`
(*the pipeline did not link*). The layout is the contract's, and the shader
must match it.

`Command` and `CommandList` lose `Eq`, because a clear value is a float.
Nothing compared them for equality except by `PartialEq`.

## Verified

- `cargo test -p nexora-rhi`: the descriptor rules each refuse what they
  should (missing attributes, overlap with the stride, misalignment,
  duplicate or out-of-range locations, too many slots, a depth test with no
  depth format). Two new conformance cases, `binding-rules` and
  `clear-rules`, pass on the null backend and on `wgpu`. The suite is now
  eleven cases.
- On a driver (lavapipe, in the container; the same test runs in CI's
  Windows and macOS jobs, on WARP and Metal): a 16×16 image of four
  quadrants, sampled through a nearest sampler, lands on a 16×16 target
  **texel for texel**. A farther draw is rejected by the depth test, and the
  target is unchanged. A nearer draw tinted green by the uniform replaces the
  image with exactly its green channel. Each step is read back.
- Mutation-checked. A depth compare forced to `Always` fails *a farther draw
  passed the depth test*. Removing the self-sampling rule fails the
  conformance case `binding-rules` on the null backend.

## Consequences

- The interim vertex rule of DEBT-0046 is gone: every pipeline states its
  layout, and the probes, the proof and the benchmark state a position of
  four `f32`.
- A camera is now expressible: a uniform slot holding a view-projection
  matrix. Nothing builds one yet; the camera system and a renderer are the
  next consumers.
- Not decided here: index buffers and indexed draws (a quad is six vertices
  until a renderer measures that it matters), instancing, blending, more
  than one bind group, and a second depth format. Each waits for a caller
  that needs it.

## Migration

None. Nothing persisted refers to the RHI.

## Compatibility

Source-breaking for anyone building a `PipelineDesc` or a `Command::Draw`:
both have new fields. Every caller in this repository is updated. The
conformance suite grows from nine cases to eleven, and the probes and the
headless slice print `11/11`.
