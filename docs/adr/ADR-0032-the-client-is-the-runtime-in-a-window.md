# ADR-0032 — The client is the runtime in a window, not a second engine

- **Status:** ACCEPTED
- **Date:** 2026-09-27
- **Completes (partly):** `NEXORA DEVELOPMENT ROADMAP.md` Phase 1's exit
  (*"runtime mínimo inicia em modo client/headless"*); DEBT-0041 (a frame
  loop against a real clock)
- **Follows:** ADR-0005 (the Phase 0 scope), ADR-0017 (a frame is time the
  host hands in), ADR-0027 (the window host), ADR-0029 (the camera),
  ADR-0030 (the first render pass), ADR-0031 (input)
- **Opens:** DEBT-0047 (sub-pixel gaps: T-junctions and no back-face culling)
- **Touches:** a new crate `engine/client` (`nexora-client`);
  `engine/render::reference` (mismatches classified); `engine/runtime::frame`
  (which stages have a system); CI; `scripts/local-validation.py`

## Context

Phase 1's exit asks the minimal runtime to start *"em modo client/headless"*.
Headless has run since the first slice. Every piece a client needs existed
separately:

- the lifecycle, with `RuntimeMode::Client` and a `PresentationRunning` phase
  that only presentation modes must enter;
- a window host that owns the event loop (ADR-0027);
- a render pass checked against a ray cast (ADR-0030);
- keys handed in as HID usages (ADR-0031);
- a frame loop that takes time as an argument (ADR-0017).

Nothing composed them, so `client_mode` stayed `NOT_IMPLEMENTED` in every
local report. The checklist said so too: *"nothing runs the frame loop, a
renderer or input inside it yet"*.

## Options

**Where the client lives.** (a) A mode flag on `nexora-headless`: the slice
is a verification script, and a window inside it would make every CI run of
the slice need a display. (b) The benchmark's window stage grown into a
client: it is measurement, and the benchmark must stay able to run without
one. (c) **A crate of its own, `engine/client`**, that goes through the same
lifecycle, module manager and world as headless. The mode is chosen by which
binary starts, and it is the lifecycle, not a flag, that tells them apart.

**What the first frame proves.** The benchmark's rule, every judged pixel
right, held on a single 16³ region seen from close by. On the client's scene
(nine chunk columns seen from 40 to 100 blocks away), **every one of twenty
seeds** had 0 to 5 wrong pixels in about 79,500 judged. We dissected them by
crossing the centre ray with every quad of the mesh:

- **(244,208):** the ray passes **0.0034 px** from the edge shared by two
  coplanar −X quads, a 4×1 and a 1×1. That is a T-junction, and the +Z face
  behind shows through the gap.
- **The other two:** the ray passes within ~0.003 px of a silhouette edge that
  the rasteriser snapped across.

The frame is right, and the mesh draws sub-pixel gaps. The options were to
(a) keep the strict rule and not ship a client, (b) tolerate a percentage of
wrong pixels, or **(c) classify each wrong pixel by what the reference itself
can check, and tolerate only those classes, in a small bound.**

## Decision

- **`nexora-client`** walks `Lifecycle` in `RuntimeMode::Client`:
  1. foundation;
  2. the module graph, where the client-only `nexora:module/renderer` now
     initializes (headless skips it);
  3. runtime init and world attach;
  4. `SimulationRunning`;
  5. the world is generated (drawn columns plus a ring) and meshed, one region
     per column;
  6. the window opens (ADR-0027) and the lifecycle enters
     `PresentationRunning`;
  7. frames;
  8. `ShutdownRequested`, `SimulationStop` and `WorldFlush` (the world is saved
     through `persist::save`, and written with `--save`), then modules stop in
     reverse order, then `ProcessExit`.

  A client whose window never opens fails the lifecycle. It cannot skip
  presentation, because `Phase::required_for` does not allow it.
- **One frame is the four stages `CORE.md` §16 orders, charged to
  `FrameRun`:**
  - **input:** one `sample` of the host's signals, as an `Intent`;
  - **simulation:** the steps the measured time bought (50 ms, at most 4).
    Each step advances the world clock one tick and moves the camera by the
    intent;
  - **render prep:** the origin follows the camera, and the pass records;
  - **render:** one submission, then presentation.

  The host measures the time between frames and hands it in; the loop still
  reads no clock. Camera movement happens per step, so the same signals and
  the same step count give the same camera on any machine.
- **Bindings are data in the input system's terms.** They are eleven
  actions in one context, bound to HID usages: WASD, Space and Left Shift to
  move, the arrows to turn and look, Escape to exit. A unit test checks the
  table against the host's translation.
- **The first shown frame is read back from the surface and judged after the
  window closes.** Judging then leaves the world unborrowed while frames
  advance its clock. The judge classifies every wrong pixel, in
  `nexora_render::reference::FrameCheck`:
  - **`backfacing`:** the pixel shows a face that points away from its ray. No
    correct frame shows one, since every emitted face separates solid from
    air. This is what a gap shows while the pass does not cull.
  - **`snapped`:** the pixel shows a colour the ray cast itself finds within
    **1/64 px** of the pixel's centre. Vulkan guarantees at least 1/16 px of
    sub-pixel precision.
  - Anything else is simply wrong.

  The client's frame holds when at least half the pixels are judged, every
  wrong pixel is one of the two classes, and together they are at most **1 in
  5,000** judged. The benchmark stages keep the strict rule.
- **The frame budget stays `FrameBudget::doubling_from(50 ms)`**, and the
  report says so. A published frame budget needs measurements from real
  machines, which this client now produces.

## Verified

- `cargo test -p nexora-client` covers:
  - the key table against the host's HID translation;
  - W as forward, opposite keys cancelling, and unbound keys asking for
    nothing;
  - one step forward and right at yaw 0 (−Z and +X), a quarter turn left,
    pitch not slowing walking, and up;
  - nine columns drawn from 25 generated, tiling the bounds;
  - a client that cannot skip presentation;
  - the renderer module initializing in client mode and not in headless;
  - the frame rule.

  The `harness = false` test runs 12 real frames and requires:
  - all 17 phases;
  - the renderer module;
  - no movement without a key;
  - a first frame that holds.
- **End to end on Xvfb and lavapipe** (`scripts/client-xtest.sh`, in CI):
  - all 17 lifecycle phases;
  - the first frame: 79,391 of 98,304 pixels judged, 79,390 matching, and 1
    snapped edge;
  - W held for one second through XTEST: **8.00 blocks forward** (20 ticks at
    0.4);
  - Escape ends the loop.
- **Frame loop (120 frames):**
  - wall time: median 41 ms, p95 50 ms;
  - 0.19 ms unattributed in total;
  - 114 frames `target` and 6 `warning`, against the arithmetic budget.
- **Mutation-checked:** each of these fails the first frame.

  | mutation | matching, of 79,820 judged |
  | --- | ---: |
  | `Less` depth test | 57,560 |
  | no BGRA→RGBA conversion | 60,454 |
  | positive faces on their own cell's plane | 76,148 |

  None of them is explained by the two classes: none has more than 1,695
  back faces or 7 snapped edges.

## Consequences

- `client_mode` leaves the items no machine can validate: it is a check in
  `local-validation.py`, which opens a window for a few seconds and needs no
  key. CI runs it on Windows and macOS through `--quick`.
- `FrameStage::has_system` is true for every stage but audio.
- DEBT-0041 moves to partial, and DEBT-0047 is opened with the evidence and
  its remediation: back-face culling in the RHI contract, then T-junction-free
  quads.
- Phase 1's exit criterion is met in CI, on software GPUs. It is **not verified
  on the operator's machine** until a local report runs `client_mode`.

## Not built

Streaming into the pass, textures, pointer look, a player body, collision,
physics or world edits inside the frame, audio, resizing the render target
with the window (the target keeps its opening size and presentation scales
it), and a measured frame budget.

## Migration

None. The client writes a save only with `--save`, in the existing format.

## Compatibility

Additive: a new crate and binary, and two new fields in `FrameCheck` (`backfacing`
and `snapped`). Every existing check still requires `matching == judged`.
`frame.stages_with_a_system` reads 6.
