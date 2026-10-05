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
| `rhi_native` | the native RHI backend on **this machine's GPU**: adapter, the eleven conformance cases, a 16×16 upload and a draw, both read back (ADR-0026), a draw with a vertex layout, a sampled texture, a uniform and a depth test, read back texel for texel (ADR-0028), and back faces culled while front faces are kept (ADR-0033) |
| `window` | a **real window** on this machine's display: its surface, the conformance cases with presentation on, 60 frames of a 16×16 target shown in it, the first read back from the surface where the platform allows (ADR-0027) |
| `client_mode` | the **runtime in client mode**: all 17 lifecycle phases, the generated world drawn in a window for 120 frames against a real clock, the first frame read back from the surface and held against the ray cast (ADR-0032), and each frame's work reported apart from its wait on presentation (ADR-0017 amendment). A window opens for a few seconds; no key needed |
| `client_resume` | the **world file** (ADR-0036): the client opens one twice for 60 frames — the first run creates the world and writes it with the player through the atomic save (the rename Windows can refuse for a moment, DEBT-0048), and the second must resume both, `the player where it was saved`. Two windows open for a few seconds; no key needed |
| `input_devices` | a **real key** through a real window: the probe opens a window titled *NEXORA input probe: press W*, and you click it and press W. The key must reach the engine as HID usage 26 and come out of `runtime::input` as the bound action's press and release (ADR-0031). It needs a person, so it runs only from a terminal and never with `--quick`; `NEXORA_INPUT=none` skips it |
| `benchmark_cpu` | the full CPU benchmark, judged against every published budget, plus the RHI, camera, frame-time and window stages on this machine's adapter and display (a window opens for a second or two; `NEXORA_DISPLAY=none` records the window stage as not measured). The first report that kept it closed DEBT-0013 (baseline Appendix I); reports 4 and 5 gave the RHI stage its first GPU numbers (Appendix K), and reports 6 to 9 the camera, frame and window stages theirs (Appendix O) |

## What it cannot validate yet, and says so

Rendering (the world, textured and streamed) is recorded as
`NOT_IMPLEMENTED`, never as passed and never as "not tested". The
engine has no code for it yet, and a real GPU cannot validate code
that does not exist. When one of them is built, it gets a check here, and only
then can a report move it. The RHI, the GPU context, shaders and texture upload
were on this list until ADR-0026 built them; they are now the `rhi_native`
check. The window, the swapchain and presentation were on it until ADR-0027;
they are now the `window` check. Real input devices were on it until ADR-0031;
they are now the `input_devices` check. Client mode was on it until ADR-0032; it
is now the `client_mode` check. Reports 4 and 5 (2026-09-26, an RX 6650 XT
over Vulkan on Windows 10) passed `rhi_native` and `window`, which closed
DEBT-0046 and the freeze gate's second blocker. Reports 6 to 9 (2026-09-29,
the same machine) passed `client_mode` all four times and `input_devices` in
reports 7 and 9, which put Phase 1's exit criterion on local hardware.

A failed check in one report and a pass in the next, on the same code, is
still a failure to explain, not noise to average away. Reports 6 to 9 had two:
a test that lost a Windows rename to a process holding the file (the save
container now retries that one refusal, a bounded number of times), and two
input timeouts that could not say whether the window ever had keyboard focus
(the probe now asks for focus and reports it).

A machine with no GPU at all says so with `NEXORA_GPU=none`: `rhi_native`,
`window` and `client_mode` are then recorded as `SKIPPED` with that reason,
never as passed. A machine with no display says so with `NEXORA_DISPLAY=none`,
and `window` and `client_mode` are skipped the same way. `NEXORA_INPUT=none` says no one will press a key, and
`input_devices` is skipped; so is it when the script is not run from a
terminal.

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
