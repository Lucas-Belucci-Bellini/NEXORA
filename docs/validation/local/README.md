# Local validation — evidence from a real machine

The repository is developed in a container with **no GPU and no display**, and
the architecture freeze cannot close without both: the benchmark's GPU stages
(DEBT-0008) and an RHI that has met a real renderer
(`NEXORA ARCHITECTURE FREEZE CHECKLIST.md`, *Phase status*). This directory is
where a run on a real machine becomes evidence the repository can read.

## On the machine

```sh
python3 scripts/local-validation.py run          # everything; minutes
python3 scripts/local-validation.py run --quick  # no test suite, smoke benchmark
git add docs/validation/local/ && git commit -m "NEXORA: local validation on <machine class>"
```

On Windows the interpreter is usually `py` or `python`, not `python3`:
`py scripts\local-validation.py run`. Needs Rust through `rustup` (the pinned
toolchain installs itself on first build), Git, and Python 3.8 or newer — and,
**on Windows, the Visual Studio C++ Build Tools**, because Rust links there
with Microsoft's `link.exe`:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

`run` starts with a **preflight** that compiles a one-line program and stops in
seconds, naming what to install, if the machine cannot build. It can also run
alone: `py scripts\local-validation.py preflight`. The
`platforms` job in CI runs `run --quick` on Windows and macOS runners, so this
path is exercised on both before anyone is asked to use it.

It writes two files here:

| file | for |
| --- | --- |
| `NEXORA-LOCAL-VALIDATION.json` | sessions and tools: the report as data |
| `NEXORA-LOCAL-VALIDATION.md` | people: the same report as tables |

`run` refuses to write here from a container or a CI runner. Their reports are
not local evidence, and a committed one would claim hardware that was not there.
`--out <dir>` writes elsewhere for testing the plumbing.

## What it runs

| check | what it proves on this machine |
| --- | --- |
| `build_release` | the workspace builds with this toolchain on this OS |
| `tests` | `cargo test --workspace --all-targets` |
| `headless_slice` | the vertical slice, save / reload / journal / region store |
| `headless_slice_content` | the same with the first-generation stones |
| `forge_first_generation` | the forge builds the 16×16 set from its plan |
| `headless_slice_textures` | the runtime resolves, verifies and decodes that set |
| `rhi_native` | the native RHI backend on **this machine's GPU**: adapter, the eleven conformance cases, a 16×16 upload and a draw, both read back (ADR-0026), and a draw with a vertex layout, a sampled texture, a uniform and a depth test, read back texel for texel (ADR-0028) |
| `window` | a **real window** on this machine's display: its surface, the conformance cases with presentation on, 60 frames of a 16×16 target shown in it, the first read back from the surface where the platform allows (ADR-0027) |
| `benchmark_cpu` | the full CPU benchmark, judged against every published budget. The first report that kept it closed DEBT-0013 (baseline Appendix I) |

## What it cannot validate yet, and says so

A renderer, real input devices, client mode and the benchmark's GPU stages are
recorded as `NOT_IMPLEMENTED`, never as passed and never as "not tested". The
engine has no code for any of them yet, and a real GPU cannot validate code
that does not exist. When one of them is built, it gets a check here, and only
then can a report move it. The RHI, the GPU context, shaders and texture upload
were on this list until ADR-0026 built them; they are now the `rhi_native`
check. The window, the swapchain and presentation were on it until ADR-0027;
they are now the `window` check.

A machine with no GPU at all says so with `NEXORA_GPU=none`: `rhi_native` and
`window` are then recorded as `SKIPPED` with that reason, never as passed. A
machine with no display says so with `NEXORA_DISPLAY=none`, and `window` is
skipped the same way.

## Reading a report: `check`

```sh
python3 scripts/local-validation.py check
```

A report is evidence about **one commit on one machine**. `check` compares its
commit with `HEAD` and classifies every item:

| freshness | meaning | items that passed read as |
| --- | --- | --- |
| `CURRENT` | the report ran on `HEAD` | `VERIFIED_ON_LOCAL_HARDWARE` |
| `CURRENT_NO_RELEVANT_CHANGE` | later commits touched none of `engine/`, `tools/`, `content/`, `benchmarks/`, `Cargo.*` | `VERIFIED_ON_LOCAL_HARDWARE` |
| `STALE_LOCAL_EVIDENCE` | relevant code changed since; the paths are listed | `STALE_LOCAL_EVIDENCE` |
| `UNCOMMITTED_TREE` | the run had tracked changes not in any commit | `STALE_LOCAL_EVIDENCE` |
| `UNKNOWN_COMMIT` | the report's commit is not in this clone | `STALE_LOCAL_EVIDENCE` |

A failed check reads `FAILED_ON_LOCAL_HARDWARE` while the report is current. A
check that did not run, and every hardware-gated item, reads
`NOT_TESTED_LOCALLY`. Stale evidence is history, not verification: re-run on the
machine rather than carrying an old `PASS` forward.

A report can **reduce** an environment blocker. It cannot remove a requirement
that has not been built, which is why the hardware-gated items above can only
become checks when the code for them exists.

## What is never recorded

No hostname, user name, serial number, MAC address or absolute path. Command
output is kept as short tails with the repository root, the home directory and
the run's temporary directory replaced by placeholders. The machine is
described by class: OS, architecture, CPU model, core count, memory, GPU model
and driver, toolchain versions, and whether it is a VM.

## When it does not run

| what you see | cause | fix |
| --- | --- | --- |
| ``linker `link.exe` not found`` | Windows without the C++ Build Tools | the `winget` line above, or "Desktop development with C++" in the Visual Studio Installer; then a **new** terminal. VS Code does not include it |
| ``linker `cc` not found`` | macOS or Linux without a C toolchain | `xcode-select --install` / `sudo apt install build-essential` |
| `` `cargo` is not on PATH `` | Rust not installed, or the terminal predates it | https://rustup.rs, then a new terminal |
| `rhi_native` FAIL: `no GPU adapter` | no Vulkan, Direct3D 12 or Metal driver answered | update the GPU driver; on Linux without a GPU, `sudo apt install mesa-vulkan-drivers`; a machine with truly no GPU runs with `NEXORA_GPU=none` |
| `python3` not found (Windows) | the interpreter is `py` or `python` there | `py scripts\local-validation.py run` |
