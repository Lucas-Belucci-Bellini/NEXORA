# ADR-0026 — The first native backend is wgpu, and CI runs it

- **Status:** ACCEPTED
- **Date:** 2026-09-25
- **Decides:** the questions ADR-0025 left for this ADR — the dependency, where
  `unsafe` lives, the shader language, and how a native backend is verified.
  The window host is the fourth question, and it is still open.
- **Amends:** ADR-0002 (the engine takes its first external dependency, in one
  crate)
- **Touches:** a new crate `engine/rhi-wgpu`; `engine/rhi` (a shared `kit`,
  two more shared rules); CI; `scripts/local-validation.py`

## Context

ADR-0025 built the RHI's contract, a null backend and a nine-case conformance
suite, and deferred the native backend to this record. The freeze gate's
second blocker, DEBT-0046, is *"no byte has ever reached a GPU"*.

Until now that blocker was described as a hardware problem, and that was
wrong. Four facts, all measured on 2026-09-25:

1. The operator's machine has a GPU (RX 6650 XT: Vulkan and Direct3D 12).
2. The development container has no GPU, but it does have Vulkan's loader, and
   Mesa's **lavapipe** installs in it: a conformant Vulkan 1.3 driver that runs
   on the CPU.
3. `wgpu` 30, built with only its Vulkan, Direct3D 12 and Metal back ends,
   finds that driver headless, with no window and no display:
   `llvmpipe (LLVM 20.1.2, 256 bits)`, backend `vulkan`, type `cpu`.
4. GitHub's Ubuntu runners can install the same driver with one `apt` line,
   and the CI check job now does.

So device, buffers, textures, shaders, pipelines, submission, fences, a draw
and a readback can all be built **and verified** without a GPU in the room.
They were `NOT_IMPLEMENTED`, not `BLOCKED_BY_ENVIRONMENT`. Only the window, the
surface and presentation need a display.

`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §5 already draws the stack
this ADR fills in:

```text
RHI → Native graphics abstraction → Vulkan / DirectX / Metal → GPU
```

and it states the rule: *"If Rust provides a clean and sufficiently mature
path for a subsystem, avoid adding another language."*

## Options

1. **`wgpu`**: Rust, safe API, one code path over Vulkan, Direct3D 12 and
   Metal. It validates WGSL with naga. The workspace lockfile grows from 19
   packages to 122, counting every platform's bindings, and it adds its own
   validation layer, which costs some CPU per call.
2. **`ash`**: raw Vulkan. It is the most control and the most code. Every
   call is `unsafe`, so it needs an `unsafe`-permitted crate like
   `benchmarks/ffi-probe` (ADR-0009), plus manual synchronization, memory
   allocation and SPIR-V tooling. It is Vulkan only (macOS through
   MoltenVK).
3. **`windows` / Direct3D 12**: the operator's platform only, `unsafe`, and CI
   on Linux could never run it.
4. **Two backends at once, to compare.** Rejected. The conformance suite
   already exists to compare backends, and a second one belongs after the first
   has met a renderer, not before.

## Decision

Option 1, **`wgpu` pinned to `=30.0.1`**, in one crate: `engine/rhi-wgpu`
(`nexora-rhi-wgpu`). Default features are off; only `vulkan`, `dx12`,
`metal`, `wgsl` and `std` are on. No OpenGL, no WebGPU, no GLSL or SPIR-V
front ends.

- **`unsafe` stays forbidden.** The crate uses `wgpu`'s safe API only and
  inherits the workspace's `unsafe_code = "forbid"`. `ffi-probe` remains the
  only crate allowed to say `unsafe`.
- **The shader language is WGSL**, compiled and validated by naga through
  `wgpu`. The backend reports `validates_shaders: true` and refuses a pipeline
  whose shader does not compile, with `Reject`.
- **The dependency lives in one crate.** `nexora-rhi` stays dependency-free, as
  does every crate ADR-0002 named. The headless slice keeps the null backend:
  `RENDER HARDWARE INTERFACE.md` wants server builds to start *"without
  initializing a GPU"*, and they do.
- **The rules are shared, not copied.** `nexora_rhi::kit` holds the resource
  tables (generation and epoch), the command rules and the memory accounting.
  The null backend and `WgpuRhi` both run on it, so a rule is written once.
  Two rules only a real driver would have caught moved into the shared checks
  and into the conformance suite: a usage that does not fit the resource kind,
  and a depth format given as a colour target.
- **Order is faithful.** Writes are encoded as copies from staging buffers,
  not queued with `Queue::write_*`. Queued writes land before the whole
  submission, so a list that drew and then overwrote its vertex buffer would
  have drawn nothing. A test pins this. Mutated to queued writes, the test
  fails.

### How it is verified

| where | what runs | what it proves |
| --- | --- | --- |
| every developer machine and CI (Linux, lavapipe) | `cargo test -p nexora-rhi-wgpu`: conformance, WGSL rejection, order, device loss and recreation, memory accounting, upload and draw read back | the backend keeps the contract **on a conformant Vulkan driver** |
| CI smoke | `nexora-rhi-probe`, grepped: `conformance 9/9 cases on wgpu`, `16 of 16 texels shaded` | the release binary does the same |
| the operator's machine | `local-validation.py` → new check `rhi_native` (the same probe) | the same, on **real hardware**, recorded against a commit |

No adapter is a failure, not a skip. A machine without a GPU declares it with
`NEXORA_GPU=none`. The tests then say that they did not run, and the
validation script records `rhi_native` as `SKIPPED` with that reason.

## What is not decided here

- **The window host.** A surface needs a window. Who owns the window also owns
  the event loop, and that is the host DEBT-0041 (a real clock) and DEBT-0043
  (real input devices) are waiting for. It is the next decision, and it is
  the first one that genuinely needs a display: presentation cannot be
  verified on lavapipe without one.
- **The vertex format.** The contract has none yet; the renderer will bring
  one. Until then this backend reads a vertex as its position, up to four
  `f32` chosen by the stride, and ignores the rest. DEBT-0046 records this
  interim rule.
- **A second native backend.** Not before the first has met a renderer.

## Verified

- `nexora-rhi-wgpu`: 3 unit and 6 integration tests pass on lavapipe. The
  probe prints `conformance 9/9 cases on wgpu`, `1024 bytes to a 16x16
  texture, read back identical` and `16 of 16 texels shaded by the GPU`.
- Mutation-checked. Queued writes fail the order test, a different fragment
  colour fails the draw proof, and a readback that keeps the row padding
  fails the upload proof. The kit's generation, epoch and deferral checks,
  which moved out of the null backend, still fail their tests when removed.
- In CI, on the first push, each of the three APIs passed on a different
  runner, through the same probe:
  **Vulkan** on Linux (`llvmpipe`, lavapipe), **Direct3D 12** on Windows
  (`Microsoft Basic Render Driver`, WARP) and **Metal** on macOS (`Apple
  Paravirtual device`). All three are software or virtual devices, not
  hardware.
- `local-validation.py run --quick` records `rhi_native` as PASS in the
  container. That is not local evidence, and the script refuses to write it
  to `docs/validation/local/`.

## Consequences

- The freeze checklist's RHI row reads **native backend built, verified on
  software Vulkan; not yet on the operator's GPU; no surface**. The second
  blocker narrows to *presentation and the first real-GPU report*.
- The local report's GPU items change meaning. `rhi`, `gpu_context`,
  `shaders` and `texture_upload` are no longer `NOT_IMPLEMENTED`. They are
  one check, `rhi_native`, that can pass or fail on a machine. `window`,
  `swapchain`, `presentation`, `rendering` (a renderer), `input_devices`,
  `client_mode` and `benchmark_gpu_stages` stay gated, because their code
  does not exist yet.
- Build time and the lockfile grow by `wgpu`'s tree. CI caches it.
- DEBT-0008's GPU stages now have a device to run on. The benchmark can grow
  them without waiting for a window, except for frame presentation.

## Migration

None. Nothing persisted refers to the RHI.

## Compatibility

`kit` and the now-public `desc::check_*` functions are additions. The two new
shared rules refuse descriptors that no backend could have honoured.
