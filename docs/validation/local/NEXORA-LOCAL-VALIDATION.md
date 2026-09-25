# NEXORA — local validation report

Written by `scripts/local-validation.py run`. Evidence about **one commit on one
machine**; `python3 scripts/local-validation.py check` says whether it still
describes HEAD. See `docs/validation/local/README.md`.

| | |
| --- | --- |
| commit | `79910836fd3994f71e4809f44ef6ff373050b395` |
| branch | `main` |
| generated | 2026-09-25T02:10:56+00:00 |
| OS | Windows 10 (AMD64) |
| CPU | AMD Ryzen 5 5500 — 12 logical |
| memory | 17043542016 bytes |
| GPU | AMD Radeon RX 6650 XT |
| display | not probed |
| toolchain | rustc 1.94.1 (e408947bf 2026-03-25) · cargo 1.94.1 (29ea6fb6a 2026-03-24) |

## Checks that ran

| check | status | seconds | summary |
| --- | --- | ---: | --- |
| `build_release` | **PASS** | 0.2 | exit 0 |
| `tests` | **PASS** | 59.5 | 1138 passed, 0 failed |
| `headless_slice` | **PASS** | 0.6 | queries 76 answered, 2 refused; memory 2 pools, worst nominal, 0 suspected leaks; probes verified 76; result OK |
| `headless_slice_content` | **PASS** | 0.6 | queries 92 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); memory 2 pools, worst nominal, 0 suspected leaks; probes verified 92; result OK |
| `forge_first_generation` | **PASS** | 0.1 | build nexora-first-generation: 16 materials: 16 written, 0 unchanged, 0 replaced, 0 restored, 0 refused, 0 failed; index <scratch>\fg\resources.json (32 resources) |
| `headless_slice_textures` | **PASS** | 0.3 | queries 44 answered, 2 refused; content blocks 16 (16 surfaces in the mesh); content textures 16 (resolved, verified and decoded); memory 3 pools, worst nominal, 0 suspected leaks; probes verified 44; result OK |
| `benchmark_cpu` | **PASS** | 5.0 | see the report's benchmark section |

## What this machine could not validate, and why

Not "not tested": the engine has no code for these yet, so no machine can.

| item | status | reason |
| --- | --- | --- |
| `window` | NOT_IMPLEMENTED | no window host exists in the engine |
| `rhi` | NOT_IMPLEMENTED | the RHI boundary is specified, not built (freeze checklist, Runtime) |
| `gpu_context` | NOT_IMPLEMENTED | needs the RHI |
| `swapchain` | NOT_IMPLEMENTED | needs the RHI and a window |
| `presentation` | NOT_IMPLEMENTED | needs a swapchain |
| `shaders` | NOT_IMPLEMENTED | no shader pipeline exists |
| `texture_upload` | NOT_IMPLEMENTED | textures decode on the CPU (ADR-0022); nothing uploads them |
| `rendering` | NOT_IMPLEMENTED | no renderer; meshes are built and checked as data (ADR-0012) |
| `input_devices` | NOT_IMPLEMENTED | no real device has produced a signal (DEBT-0043) |
| `client_mode` | NOT_IMPLEMENTED | the runtime starts headless only; client mode is Phase 1's exit |
| `benchmark_gpu_stages` | NOT_IMPLEMENTED | DEBT-0008: the plan's GPU stages have no implementation |

## CPU benchmark (tail)

```text

| stage the plan asks for | why there is no number |
| --- | --- |
| window | no windowing layer; ADR-0005 |
| input | meaningless without a window |
| RHI | boundary specified, no backend implemented |
| camera | depends on the RHI |
| mod boundary | mod runtime not implemented; Phase 7 |
| frame time | no render loop exists to time |
| incremental build | measured by the build system, not by this process |
| debugging / tooling effort | qualitative; the plan scores it separately from timing |
| FFI overhead | built without the `cpp` feature; no boundary is linked in |
```

## Verdict

Every executed check passed.
