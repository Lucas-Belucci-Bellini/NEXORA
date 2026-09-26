# ADR-0025 — The RHI is a contract a null backend keeps before a GPU does

- **Status:** ACCEPTED
- **Date:** 2026-09-25
- **Completes (partly):** `RENDER HARDWARE INTERFACE.md` — *Responsibilities*
  (capabilities, buffers, textures, pipelines, submission, fences,
  presentation), *Headless* (the null backend), *Tests* (resource lifetime,
  synchronization, device loss, backend parity, headless startup)
- **Amends:** ADR-0005 — the RHI row moves from "deliberately not implemented"
  to "contract built, no native backend"
- **Touches:** a new crate `engine/rhi`, foundation (`Domain::Render`),
  `nexora-headless`, CI, `scripts/local-validation.py`

## Context

The freeze checklist's Final gate names two blockers. The second is *"the RHI
and presentation boundary is unbuilt … nothing has tested whether the boundary
survives contact with a real renderer."* ADR-0005 deferred the RHI for one
reason: *"No display or GPU available to verify against."*

As of 2026-09-25 that reason is gone, in part. The first local validation
report (`docs/validation/local/`) comes from a real Windows 10 machine with an
AMD Radeon RX 6650 XT, a GPU with Vulkan and Direct3D 12 drivers. The report
lists every GPU item as `NOT_IMPLEMENTED`, correctly: **the hardware exists and
the code does not.** A GPU cannot validate code that has not been written.

The specification is 54 lines. It lists what the RHI is responsible for, asks
for a null backend for server builds (*"without initializing a GPU"*), and
names the tests: resource lifetime, synchronization, device loss handling,
backend parity, shader validation and headless startup. It sketches the API in
TypeScript and decides none of the rules a backend could disagree on.

ADR-0002 kept the engine crates dependency-free, and said so narrowly:
*"Rendering, windowing, networking and tooling will take dependencies; each
should be justified where it is added."*

## Problem

The obvious next step is to pick a native backend and draw something. Doing it
first has a cost that is easy to miss: **whatever the first backend does
becomes the contract.** If a buffer write at an unaligned offset happens to
work on it, the renderer will come to depend on that, and the second backend
will find out. The blocker says "the boundary has never met a renderer". A
boundary defined *by* its only backend can never fail to survive contact with
it, so it cannot be tested that way either.

The native backend is also a decision this environment cannot verify: it has
no GPU, and the operator's machine is reached through a report, not a shell.
Its dependency, where `unsafe` lives, and which shader language it reads all
deserve a decision made against a contract that already exists.

## Options

1. **Take a native backend now (for example `wgpu`) and write the renderer
   against it.** Fastest to pixels. The library's rules become NEXORA's, and
   the first thing to test the boundary is the thing that defined it.
2. **Raw Vulkan or Direct3D 12 behind an `unsafe` crate, now.** The same
   problem, plus the largest amount of code this environment could never run.
3. **Contract first:** the RHI as a Rust trait with every shared rule decided
   and enforced, a **null backend** that keeps all of them with no GPU, and a
   **conformance suite** that any backend must pass. The native backend comes
   next, under its own ADR, and has to pass the suite on real hardware.
4. **Keep waiting for hardware.** The hardware exists, so this is no longer
   the honest reason.

## Decision

Option 3.

### The crate

`engine/rhi` (`nexora-rhi`) depends on `nexora-foundation` alone, as
`NEXORA DEPENDENCY MATRIX.md` puts it (*RHI → Platform, Foundation*). It knows
buffers, textures and pipelines. It does not know what a block, a material or a
texture asset is; the caller translates.

### The rules, decided once for every backend

| Rule | Why here and not in a backend |
| --- | --- |
| A handle is `(index, generation, epoch)`: destroy bumps the generation, device loss bumps the epoch, and all three make a handle stale | Use-after-destroy and use-after-device-loss are refused the same way everywhere, not as a driver crash on one backend and silent corruption on another |
| A command list is **all or nothing**: one invalid command refuses the whole list | A half-applied list is a state no test can reason about |
| Buffer sizes, write offsets and write lengths are multiples of **4** | The strictest rule Vulkan, Direct3D 12 and Metal share: a write valid on one backend is valid on all |
| Every resource declares its `Usage`; a write needs `COPY_DST`, a draw's buffer `VERTEX`, its target `RENDER_TARGET` | Refused at submission, identically, instead of at the driver |
| A texture upload carries **exactly** every texel | A short or long upload is a caller bug, not something to pad or truncate |
| Formats are few: `R8`, `RG8`, `RGBA8` linear and sRGB, `Depth32Float`. **No three-channel format** | None of the three APIs guarantees one, so RGB is widened once, by the caller, and not by each backend differently. Every backend must offer RGBA8 in both colour spaces |
| Fences complete **in submission order** | That is the whole synchronization promise, and the suite checks it |
| Destroying a resource that in-flight work uses **defers its memory** until that work's fence completes; the handle dies at once | The rule most likely to be wrong in a real backend, and the one whose failure is a GPU reading freed memory |
| A lost device answers every call with `DisableSubsystem`, which is **not fatal**; `recreate` starts a new device life | Losing a GPU is not a reason to stop the simulation |
| `present` succeeds **exactly** when the capabilities say the backend presents | A backend that pretends to present is the fake system ADR-0005 warned against |
| Shader code is **opaque bytes** for now, and `Capabilities::validates_shaders` says whether a backend checks them | The shader language comes with the first native backend; deciding it here would be choosing that backend by the back door |

### The null backend

`NullRhi` keeps every rule above and draws nothing. Its GPU finishes **one
submission per `poll`**, so work is observably in flight. A backend that
finished everything at submission would make deferred destruction untestable.
It accounts device memory in a `MemoryClass::Gpu` pool (ADR-0024), refusing a
create that its owner's budget does not admit, and `lose_device` injects a
device loss for tests and fault drills. Its capabilities say `presents: false`
and `validates_shaders: false`, and it behaves accordingly.

### The conformance suite

`nexora_rhi::conformance::run(&mut dyn Rhi, &TestShaders)` runs nine cases
(capabilities, descriptor rules, stale handles, write rules, fence order,
deferred destruction, draw rules, present, at rest) through the trait alone. It
leaves the backend holding exactly what it held before. **Backend parity** is
thereby a check rather than a promise: the first native backend passes it on
real hardware, or it is not a backend. Device loss is not in the suite,
because no portable call makes a real GPU lose its device.

### In the slice

`nexora-headless` opens the null backend at startup with no GPU initialized,
which is the specification's *headless startup* test, and runs the suite
against it on every run. With the first generation's content, it also uploads
all sixteen decoded albedos through the RHI: each widened to RGBA8 in the
colour space its role names (albedo is sRGB), created with
`SAMPLED | COPY_DST`, uploaded in **one** submission and waited on with **one**
fence. Then they are destroyed, and the GPU pool must drain like every other.

## What the next ADR must settle

*Settled by ADR-0026 (2026-09-25): `wgpu` in one crate, safe code only, WGSL,
and conformance run in CI on software Vulkan and on the operator's machine
through `local-validation.py`. The window host was settled by ADR-0027:
`winit`, in its own crate, which owns the event loop.*

The first native backend. This ADR does not choose it, but it fixes what that
choice has to answer:

1. **The dependency**, justified as ADR-0002 asks: a portable layer (`wgpu`),
   a raw binding (`ash` for Vulkan, `windows` for Direct3D 12), or both behind
   the trait. The operator's machine supports Vulkan and Direct3D 12, so either
   can be verified there.
2. **Where `unsafe` lives.** The workspace forbids it; ADR-0009 already made one
   exception, `benchmarks/ffi-probe`, isolated in its own crate. A raw binding
   needs the same shape.
3. **The shader language** (WGSL, SPIR-V or HLSL), and with it whether
   `validates_shaders` becomes true.
4. **The window host**, which is also the host DEBT-0041 (a frame clock) and
   DEBT-0043 (real input devices) wait for. Whoever owns the window owns the
   event loop.
5. **Verification.** CI has no GPU. The backend is verified by running the
   conformance suite on a real machine, as a new check in
   `scripts/local-validation.py`, and only then can a report move the `rhi`,
   `gpu_context` or `texture_upload` items off `NOT_IMPLEMENTED`.

## Verified

- `cargo test -p nexora-rhi`: 20 tests. They cover every descriptor rule,
  stale handles under slot reuse and across a device loss (where only the epoch
  tells an old handle from a new one), all-or-nothing submission, ordered
  fences, deferred destruction, memory refused by the device limit and by the
  owner's budget, draw validation, and presentation.
- The suite catches the bugs it exists for. A backend that frees memory under
  the GPU fails `deferred-destruction`; one whose `present` claims success
  without a surface fails `present`.
- Mutation-checked. Removing the generation bump, the epoch check or the
  deferral each fails at least one test. The epoch check first survived, and
  the device-loss test was tightened until it did not.
- `nexora-headless`: every run reports `rhi null backend, conformance 9/9
  cases, no GPU initialized` and a third memory pool, `rhi.null` (`gpu`,
  drains at rest, 4096 bytes at peak). With textures it reports `rhi uploads
  16 textures as RGBA8, 16384 bytes, one fence`, and the pool peaks at
  16384 bytes and ends at 0. CI greps both lines.

## Consequences

- The freeze checklist's RHI row moves from `[ ]` to `[~]`. The contract is
  built and held to a suite, but the second blocker is **not** closed: the
  boundary has met a null backend it was written with, not a GPU. Nothing
  here is evidence that a real driver agrees with these rules.
- The `Gpu` memory class has its first owner.
- The local validation report says why `rhi` and `texture_upload` are still
  `NOT_IMPLEMENTED` on hardware: the contract exists and no native backend
  does.
- `Domain::Render` joins the error taxonomy.

## Not claimed

No pixel has been computed, no surface presented, and no shader compiled. The
null backend's `validates_shaders` is false and it says so. The API sketch's
render graph is not built either: the specification puts it above the RHI, in
the renderer, and there is no renderer.

## Migration

None. Nothing persisted or on the wire refers to the RHI.

## Compatibility

`nexora-rhi` is new. `Domain` is `#[non_exhaustive]`, so the new variant is
additive.
