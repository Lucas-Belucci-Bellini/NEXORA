# NEXORA — ARCHITECTURE FREEZE CHECKLIST

## Goal
Provide an explicit gate before large-scale implementation and final technology
lock.

## How to read this

The original checklist tracked one thing: whether a contract was *defined*. With
Phase 0 implemented, ticking everything as "defined" would make the list
decorative — nearly every contract has a specification document. It now tracks
two independent questions:

- **Defined** — a normative document specifies the contract.
- **Built** — code implements it, and CI builds, tests and runs that code.

`Defined` without `Built` is the intended state for most rows: `preparing is not
implementing` (ADR-0005). A row that is `Built` without being `Defined` is a
defect — it means code is inventing architecture.

Legend: `[x]` yes · `[ ]` no · `[~]` partial, with the gap named.

## Foundation

| Contract | Defined | Built | Where |
| --- | :---: | :---: | --- |
| Core lifecycle | [x] | [x] | `runtime::lifecycle` |
| Module lifecycle | [x] | [x] | `runtime::module` |
| Job / threading model | [x] | [x] | `runtime::jobs` |
| Resource ownership | [x] | [~] | the asset cache is built — byte budget, priority, last use, pinning, eviction, counters, `Arc` so eviction never takes a value from its holder (`engine/resource`, ADR-0021); **per-owner memory budgets** are built — one ledger, eight classes, `TARGET/WARNING/CRITICAL/EMERGENCY`, high-water marks, worst pressure, refusals, suspected leaks (`foundation::memory`, ADR-0024) — and the world, retained chunks and texture cache record into it, gated in CI; the RHI's null backend owns the first `Gpu` pool (ADR-0025); the `Frame`, `Network`, `Script` and `Editor` classes have no owner yet because none of those subsystems exists |
| Time model | [x] | [x] | `foundation::time` |
| Spatial model | [x] | [x] | `foundation::spatial` |
| Registry / ID rules | [x] | [x] | `runtime::registry`, `foundation::ident` |
| Event / Command / Query semantics | [x] | [~] | events, **commands** (`engine/command`, ADR-0010) and **queries** (`engine/query`, ADR-0023: read-only by type, versioned, deny-by-default through the same `SourcePolicy`, budget re-counted after the handler, answers in identifiers) are built; commands stop at CMD-4 (DEBT-0021) and queries read the live world, not snapshots |

## Runtime

| Contract | Defined | Built | Where |
| --- | :---: | :---: | --- |
| RHI boundary | [x] | [~] | the **contract** is built (`engine/rhi`, ADR-0025): generational handles that a device loss also kills, all-or-nothing submission, four-byte copy alignment, declared usages, ordered fences, destruction deferred until the last use's fence, device loss as `DisableSubsystem` with `recreate`, `present` exactly when the capabilities say. A **null backend** keeps every rule with no GPU, and an eleven-case **conformance suite** runs against it on every slice, which also uploads the first generation's sixteen albedos through it. The **first native backend is built** (`engine/rhi-wgpu`, ADR-0026): `wgpu` over Vulkan / Direct3D 12 / Metal, WGSL validated by naga, rules shared with the null backend through `rhi::kit`. It passes the same eleven cases, uploads a texture and draws a triangle, and reads both back. Since ADR-0028 the contract also declares **vertex layouts, binding slots (uniform, texture, sampler) and a depth test**, and a draw that samples a texture through a uniform-tinted pipeline with depth is read back texel for texel on the driver. Since ADR-0033 it also declares **back-face culling** (`Cull`), read back on the driver by `nexora-rhi-probe`, and the chunk pass uses it. It is **verified in CI on all three of its APIs**: Vulkan (Mesa's lavapipe, Linux), Direct3D 12 (WARP, Windows) and Metal (Apple's paravirtual device, macOS). All three are software or virtual. It **presents** (ADR-0027): a `winit` window host (`engine/window`) owns the event loop, the backend opens a surface on the window and `present` draws the target onto it; CI reads the first frame back from the surface on Xvfb and matches all 65,536 texels. **Verified on the operator's hardware** (local reports 4 and 5, 2026-09-26, code identical to `HEAD`): an AMD Radeon RX 6650 XT over Vulkan passes the eleven cases with and without presentation, the upload, the draw and ADR-0028's bound draw read back, and a Win32 window shows the target in 65,536 of 65,536 texels read from its surface; local reports 6 to 9 (2026-09-29, the code of ADR-0033) add ADR-0033's culling read back on the same GPU (16 of 16 texels counter-clockwise, 0 clockwise). Not verified on hardware: Direct3D 12, Metal, any other GPU, and a device loss raised by a driver |
| Input boundary | [x] | [~] | the window host translates keyboard and mouse buttons into `runtime::input` signals as **USB HID usages** (keyboard page 0x07, mouse buttons 1–5 and 16+), collected per frame and handed to the client before its frame; key repeat is dropped, focus loss releases everything held, and a tap inside one frame is no longer lost (ADR-0031). **Verified in CI**: a real `W` pressed through the X server's test extension on Xvfb reaches the engine as usage 26 and the bound action's press and release. **Verified on the operator's hardware** (local reports 7 and 9, 2026-09-29): a W pressed by the operator on a Win32 window reached the engine as usage 26 and the action's press and release. Reports 6 and 8 timed out with no key reaching the engine; the probe now asks the system for focus when its window opens and says whether the window ever had it, so a timeout tells a missed click from a lost key. Gamepads, touch, pointer motion, the wheel and text entry are not built (DEBT-0043) |
| Audio boundary | [x] | [ ] | — |
| Asset lifecycle | [x] | [~] | `ResourceID → Manifest → Resolver → Loader → Cache → Handle` built, with integrity checked before any loader runs and declared fallbacks (`engine/resource`, ADR-0021); the runtime decodes its own textures with the one PNG decoder, bounded by each file's declared size (`engine/image`, ADR-0022); it inflates with the foundation's `inflate_bounded`, all three deflate block types, one inflater in the workspace; resource packs do not layer |
| Streaming lifecycle | [x] | [~] | `request · cancel · set_interest · tick` of `STREAMING SYSTEM.md` are built (ADR-0008), and of its test list fast travel, save-before-evict and low-memory pressure are covered (`engine/streaming::system` tests); the slice's **initial** load is still explicit, through the job system rather than through streaming (DEBT-0018, DEBT-0024), and dimension transfer and reconnect have no subsystem to test against. *Corrected 2026-09-25: the row said `[ ]` and predated ADR-0008.* |
| Headless mode | [x] | [x] | `nexora-headless` |
| Client mode | [x] | [~] | `nexora-client` (ADR-0032): the lifecycle in `RuntimeMode::Client` through `PresentationRunning`, the renderer module, a generated world drawn in a window, `runtime::frame` against a real clock, WASD through the input system; verified in CI (Xvfb, and Win32/AppKit through the quick local report) and **on the operator's machine** (local reports 6 to 9: all 17 phases, every first frame 79,390 of 79,391 judged pixels matching with 1 snapped edge and 0 back faces, frame wall median 9.87–9.90 ms, paced by a ~100 Hz display; baseline Appendix O); no streaming, textures, player or physics in the frame |

## Simulation

| Contract | Defined | Built | Where |
| --- | :---: | :---: | --- |
| ECS / data model | [x] | [~] | entity identity, lifecycle, components, queries and persistence built as a dense component store (`engine/entity`, ADR-0006); the archetype/query-planner layer is deferred with no measured need |
| Physics ownership | [x] | [~] | fixed timestep, rigid bodies, materials, gravity, swept voxel collision, character control and ray queries built (`engine/physics`, ADR-0007); body-versus-body collision, rotation and non-cube shapes are not (DEBT-0014, DEBT-0015, DEBT-0016) |
| Physics ↔ world boundary | [x] | [x] | terrain reaches the solver through `VoxelSource`; `engine/simulation` is the only crate that sees both sides, and Cargo enforces it (ADR-0007) |
| AI decision pipeline | [x] | [ ] | `NEXORA AI DECISION ARCHITECTURE.md` |
| LOD transitions | [x] | [~] | the `FULL → REGIONAL → ABSTRACT → UNRESIDENT` ladder, hysteresis and eviction are built and tested (`engine/streaming`, ADR-0008); the two middle tiers hold no distinct data until the regional simulation exists (DEBT-0019) |
| Streaming residency | [x] | [x] | interest, priority, budgets with backpressure, and eviction that persists before it drops — including the case where the write fails and the chunk is *not* dropped (ADR-0008) |
| Performance budgets | [x] | [~] | the `MEMORY` dimension is published and enforced for the slice's pools, from measured per-column figures (ADR-0024); the first **time** budget is published too: physics, one substep of 1,000 awake bodies, 250 µs / 500 µs / 1 ms / 2 ms, measured on two machines and judged in every benchmark run (`nexora_physics::budget`, DEBT-0013 closed; baseline Appendix I). It is not gated in CI, because a shared runner's clock is noise. I/O and job scheduling differ between the two machines in shape, not just in scale, and have no time budget yet |
| Deterministic requirements | [x] | [~] | RNG, generation and saves are deterministic and tested; replay is not built |

## World

| Contract | Defined | Built | Where |
| --- | :---: | :---: | --- |
| Chunk lifecycle | [x] | [x] | `world::chunk` |
| World-state lifecycle | [x] | [x] | `world::world` |
| Generation seed reproducibility | [x] | [x] | `foundation::rng`, `world::world` |
| Persistence boundary | [x] | [x] | `persistence`, `world::persist` |
| World event lifecycle | [x] | [ ] | `World Events.md` |

## Living world

| Contract | Defined | Built |
| --- | :---: | :---: |
| Civilization ownership | [x] | [ ] |
| Economy ownership | [x] | [ ] |
| Knowledge / information boundaries | [x] | [ ] |
| History truth model | [x] | [ ] |
| Lore derivation | [x] | [ ] |
| Archive / evidence model | [x] | [ ] |
| Player-independence rules | [x] | [ ] |

## Network / security

| Contract | Defined | Built | Note |
| --- | :---: | :---: | --- |
| Authority model | [x] | [ ] | |
| Replication boundaries | [x] | [ ] | |
| Threat model | [x] | [~] | save files are treated as untrusted input today: bounded lengths, checked reads, quarantine |
| Validation invariants | [x] | [~] | foundation and world invariants are enforced and tested; the first `CROSS-SYSTEM` check is built — a resource index against the content that needs it (`Manifest::gaps`/`provides`): the forge refuses to write an incomplete index and the slice refuses one before any load, every gap named at once; saves already resolve block identifiers against the registry and quarantine missing content (`runtime::registry`); commands ↔ entities have no check, because commands do not reach entities yet |
| Mod / script trust model | [x] | [ ] | configuration namespace isolation is built; no script sandbox |

## Content / tools

| Contract | Defined | Built | Note |
| --- | :---: | :---: | --- |
| Content pipeline | [x] | [~] | for surface materials: authored definition → recipe → generate → validate → PNG → batch by manifest → INDEX (`tools/texture-forge`, ADR-0019, ADR-0021); no other asset type has a pipeline |
| Asset provenance | [x] | [~] | every generated material carries its class, tool, generator, recipe fingerprint and seed; the first generation is catalogued with a byte-level hash per asset and held to it by a test (`content/first-generation/CATALOG.md`); nothing has been reviewed for release, so nothing may ship |
| Original-content policy | [x] | [x] | no third-party code or assets; algorithms implemented from published specifications (ADR-0002) |
| Editor / runtime relationship | [x] | [ ] | |
| Mod API versioning | [x] | [~] | version types exist; no mod API |

## Engineering

| Contract | Defined | Built | Where |
| --- | :---: | :---: | --- |
| Save compatibility | [x] | [x] | `foundation::version`, ADR-0004 |
| Crash / recovery strategy | [x] | [~] | detect, quarantine, atomic write and journal recovery are built and tested against the mandatory crash/corruption cases (ADR-0011); **the engine journals its own writes** and the slice rebuilds itself from checkpoint + journal, on a policy measured rather than guessed (Appendix E: an fsync is 303× an append); replay still reaches only resident chunks (DEBT-0024) |
| Observability | [x] | [x] | `foundation::diagnostics` |
| Testing strategy | [x] | [x] | 850+ tests; unit, integration, property, determinism, corruption, content-catalog, plus 12 cross-stack conformance digests |
| CI / build / release strategy | [x] | [~] | format, lint, test, build and smoke run in CI; packaging and release do not |
| Technology benchmark | [x] | [~] | harness built; a second stack (C++20 kernels, two compilers) is now measured and conformance-gated, FFI overhead included ([Appendix D](docs/benchmarks/PHASE-0-BASELINE.md)); the RHI stage is measured, in CI on lavapipe and on the operator's RX 6650 XT (Appendices J and K), and so are the camera stage (ADR-0029, Appendix L), frame time, a frame of the first render pass checked against a CPU ray cast before it is timed (ADR-0030, Appendix M), and the window stage, the same frame presented in a real window and read back from its surface (ADR-0030 amendment, Appendix N); **the gate is still open** — no engine-scale comparison exists (DEBT-0008, ADR-0009) |

## Phase status (2026-09-26)

Two numbering schemes meet here, and they must not be confused.
`NEXORA DEVELOPMENT ROADMAP.md` calls **Phase 0 the Architecture Freeze**, with
one exit: *"nenhum blocker arquitetural crítico sem decisão registrada"*.
ADR-0005 and the README use "Phase 0" for the **first implementation
increment**, whose scope that ADR bounds. Only the roadmap's phases gate
anything.

| Roadmap phase | State | Evidence |
| --- | --- | --- |
| 0 — Architecture Freeze | **open: blocked by unbuilt code, not by the environment** | the Final gate below is not met, and **one blocker remains**: the benchmark measures the RHI stage, the camera stage (ADR-0029), frame time and a window (ADR-0030), in CI on software Vulkan and, since local reports 6 to 9, all of them on the operator's RX 6650 XT (baseline Appendix O), so every GPU stage of the plan has a number on real hardware. What is left of DEBT-0008 is not only the engine-scale comparison: the benchmark documents also name three stages nothing builds — *player movement* and *one cross-language tool call* (`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17) and the *mod boundary* (`NEXORA TECHNOLOGY BENCHMARK PLAN.md`; the benchmark itself lists it as "not measured: Phase 7"). Player movement is the roadmap's Phase 2 player and can be built now. **The mod boundary cannot, in roadmap order**: it is Phase 7's, and §18's lock also asks for a stable mod boundary and a stable editor/runtime boundary — Phases 7 and 8. Read as "the complete benchmark and the full lock before the freeze", the gate waits for phases that come after the freeze. That is a decision to register, not code to write; it is **proposed** in [ADR-0034](docs/adr/ADR-0034-the-freeze-gates-the-cores-language-not-every-boundarys.md) (status PROPOSED, the project owner's to accept or reject), and player movement and the tool-call stage are needed under either reading. **Since 2026-10-04 player movement is built and measured** ([ADR-0035](docs/adr/ADR-0035-the-player-is-a-body-the-simulation-steers.md), `player.walk_route`), **and so is the cross-language tool call**: a Python tool (`tools/catalog-digest`, standard library only) called over a versioned stdin/stdout contract, its answers checked against Rust's before timing (`tool.cold_call`, `tool.warm_roundtrip`, baseline Appendix P). Of the three, only the mod boundary remains. With it remain the engine-scale comparison and the ADR-0034 decision. The second blocker **closed on 2026-09-26**: the contract, a null backend and a native `wgpu` backend pass the same suite, and the native one draws and presents on the operator's GPU and desktop (ADR-0025 to ADR-0028, local reports 4 and 5). *Reclassified 2026-09-25: this row said "blocked by environment", but lavapipe gives every headless GPU path a conformant driver, so what remains is code (window, presentation, the benchmark's GPU stages) plus one local report on real hardware.* Both are recorded with decisions (ADR-0001, ADR-0005, ADR-0009); neither can honestly be reclassified as a post-freeze extension, because the RHI row is exactly the contract that has not been tested |
| 1 — Engine Bootstrap | **exit criterion met in CI and on local hardware; not exited** | core, modules, jobs, time, spatial, registry, events, commands, diagnostics, configuration and resources are built and run in CI. The exit asks the runtime to start *"em modo client/headless"*: headless does, and since ADR-0032 **client mode does**: `nexora-client` walks all 17 lifecycle phases in `RuntimeMode::Client` (the renderer module initializes, presentation is entered), draws a generated world in a window through the first render pass, runs `runtime::frame` against a real clock and walks a first-person player by real keys (ADR-0035). CI runs it on Xvfb (a real W through XTEST walks the player forward, and CI asserts it) and, through `local-validation.py --quick`, on Win32 and AppKit. **The operator's machine ran it in local reports 6 to 9** (2026-09-29, RX 6650 XT, Win32): `client_mode` passed in all four and is `VERIFIED_ON_LOCAL_HARDWARE`. It is not exited only because the roadmap's phases are ordered, and Phase 0 is still open |
| 2 — Voxel Vertical Slice | partly built ahead of order | chunk, storage, meshing, coordinates, streaming, the camera's core (view, projection, frustum, floating origin; ADR-0029), a render pass that draws meshed chunks (ADR-0030), keyboard and mouse input (ADR-0031) and a free camera over nine chunk columns in the client (ADR-0032), drawn without gaps (back faces culled, quads split at every corner; ADR-0033, DEBT-0047 closed) exist; a first-person player with gravity, collision and a jump (ADR-0035) exist; since 2026-10-05 so does **interaction** (ADR-0036): the player's hands break and place blocks through the block commands, the authority checks each request against the player's body (reach from the eye, no block inside the body), and the client meshes again what changed and can start from a save — so world → player → interaction → change → save → reload runs end to end, by script in the headless slice and by hand in the client. Camera interpolation (DEBT-0049), a persisted player (DEBT-0050), a crosshair, line of sight for the authority (DEBT-0053), textured rendering and streaming into the pass do not |

Work continues on what can be verified headless — the roadmap's own rule is
that a phase advances on its technical criteria, and the criteria that remain
need hardware this environment does not have. Nothing here claims otherwise.

**Local evidence (2026-09-29): nine reports, one machine** —
[`docs/validation/local/NEXORA-LOCAL-VALIDATION.md`](docs/validation/local/NEXORA-LOCAL-VALIDATION.md).
Reports 6 to 9 ran fourteen minutes apart on the same engine code, the merge
of ADR-0029 to ADR-0033 (`6ad11f2`); the current one, report 9, ran on
`8578d0f`. Same machine as below. **Report 9 passed every check it ran**:
1244 tests, the slices, the forge, `rhi_native` (with ADR-0033's cull case),
`window`, **`client_mode`** (all 17 phases, first frame 79,390 of 79,391
judged pixels matching, 1 snapped edge, 0 back faces) and **`input_devices`**
(the operator's W reached the engine as usage 26), and the benchmark, whose
camera, frame and window stages now carry numbers from this GPU (baseline
Appendix O, Finding 33). `client_mode` passed in all four reports and
`input_devices` in reports 7 and 9. Two failures, kept here because they are
evidence too:

- **Report 6, `tests`:** `the_same_seed_produces_the_same_world_twice` failed
  once in four runs of the same code with `Acesso negado. (os error 5)` on
  the rename that commits a region file. Windows refuses to replace a file
  another process holds open, and the most likely holder, since the same test
  passed the other three times, is a scanner reading the file just written. A
  rename refused that way is now tried again, up to 8 attempts with doubling
  pauses from 5 ms (635 ms at most); any other refusal fails at once, and the
  error records the attempts (`engine/persistence::container`).
- **Reports 6 and 8, `input_devices`:** timed out after 60 s with no key
  reaching the engine. The probe could not tell a key that never came from a
  window that never had keyboard focus. The window now asks for focus when it
  opens, counts every time it gains focus, and a timeout says which it was.

Both fixes change `engine/`, so after them report 9 reads
`STALE_LOCAL_EVIDENCE` until the next report runs on the new code.

**Before (2026-09-26): five reports.** Report 5 ran on commit `8331c15`;
report 4 ran sixteen minutes earlier on `786f77f`, the merge of ADR-0028. The
code is the same in both, and `local-validation.py check` then said
`CURRENT_NO_RELEVANT_CHANGE`. The machine is a real Windows 10 desktop (AMD
Ryzen 5 5500, 12 threads, 16 GiB, AMD Radeon RX 6650 XT). Every executed check
is `VERIFIED_ON_LOCAL_HARDWARE` in both: release build, **1178 tests**, the
slice with and without the first generation, the forge's build of it, the
textures, **`rhi_native`** (adapter `RX 6650 XT (vulkan, discretegpu)`, 11/11
cases, upload, draw and bound draw read back), **`window`** (Win32, surface
`Bgra8UnormSrgb` FIFO, 0 redraws waited, 65,536 of 65,536 texels read back,
61 frames) and the whole benchmark, whose RHI stage now carries GPU numbers
(baseline Appendix K, Finding 29). In that report `rendering`,
`input_devices`, `client_mode` and the remaining `benchmark_gpu_stages` stay
`NOT_IMPLEMENTED`: at its commit no code drew the world, read a device, ran a
client or timed a frame. Since then the camera (ADR-0029) and the first render
pass with frame time (ADR-0030) exist, so the next report on `main` runs them
in `benchmark_cpu`, and until then they are `NOT_TESTED_LOCALLY`. The input
boundary (ADR-0031) left `input_devices` for an interactive check that asks the
operator to press W; it is `NOT_TESTED_LOCALLY` until a report runs it. Client
mode (ADR-0032) left `client_mode` for a check that opens the client for 120
frames; it is `NOT_TESTED_LOCALLY` for the same reason.

**Before (2026-09-25): three reports.** Report 3 ran on `0b7bcec` and closed
DEBT-0013 (Appendix I). It predated `engine/rhi`, so it said nothing about the
GPU. The previous note, kept for the record:

**Before the first report:** The bridge exists —
`scripts/local-validation.py` and [`docs/validation/local/`](docs/validation/local/README.md):
a run on a real machine records its commit, its hardware class and every check
it could execute, and `check` classifies that evidence against `HEAD`
(`CURRENT`, `STALE_LOCAL_EVIDENCE`, …). It does not move either blocker by
itself. What a machine can give today is the CPU benchmark from a second
machine (DEBT-0013's trigger) and the slice on a real OS; the GPU stages, the
RHI, the window and client mode stay `NOT_IMPLEMENTED` in every report until
the code for them exists, because no hardware can validate code that is not
there.

## Final gate

The architecture can be frozen only when unresolved items are either completed
or explicitly classified as post-freeze extensions with no impact on frozen
contracts.

**Not met.** One blocker remains; the second closed on 2026-09-26:

1. **The technology benchmark now has two stacks, and still cannot close**
   (DEBT-0008). The second stack exists: `benchmarks/cpp/` mirrors the engine's
   hot kernels in C++20, twelve conformance digests match bit-for-bit across
   Rust, g++ and clang++, and `scripts/compare-stacks.sh` refuses to time
   anything until they do — see
   [`docs/benchmarks/PHASE-0-BASELINE.md`](docs/benchmarks/PHASE-0-BASELINE.md),
   Appendix D. Rule 5's "do not decide from one stack" is satisfied for the
   kernels; two things it asked for are still missing. **The GPU stages** were
   listed here as impossible
   in a headless container. That stopped being true with ADR-0026 and
   ADR-0027: the **RHI stage is measured** now, on lavapipe in CI and on any
   machine's own adapter, with its answers checked before timing (Appendix J,
   Finding 28). Since local reports 4 and 5 it is also measured **on the operator's RX
   6650 XT** (Appendix K, Finding 29: on real hardware the fence is the cost,
   so a renderer submits once per frame). The camera stage is measured too
   (ADR-0029, Appendix L), and so is **frame time**: the first render pass
   draws the meshed 16³ region through the camera, and the frame must match a
   CPU ray cast before it is timed (ADR-0030, Appendix M), and so is the
   **window** stage: the same frame presented in a real window, the first one
   read back from the surface and checked the same way (ADR-0030 amendment,
   Appendix N). **Every GPU stage of the plan now has a number.**
   **An engine-scale comparison** is explicitly out
   of scope for a kernel reference (ADR-0009) — nine pieces of arithmetic say
   nothing about allocation, cache behaviour at scale, or threading.

   What Appendix D did settle is worth stating precisely, because the temptation
   is to over-read it: on these kernels Rust is not measurably a handicap, and
   **the optimizer backend costs more than the language** — the widest gap
   between the two C++ builds (1.89×) is larger than every Rust-versus-C++ gap
   in the table. Freezing on that alone would still settle by default the
   question the gate exists to ask, and
   `NEXORA LANGUAGE AND FFI BOUNDARY.md` reserves the language lock for the
   completed benchmark.

   **What "completed" still lacks (read 2026-10-03):** three stages the
   benchmark documents name and nothing builds — *player movement* and *one
   cross-language tool call* (§17) and the *mod boundary* (the Benchmark
   Plan). The first two are buildable now. The mod boundary is Phase 7's, and
   §18's lock also asks for stable mod and editor/runtime boundaries (Phases 7
   and 8), so the gate, read as "the complete benchmark and the full lock
   before the freeze", waits for phases after the freeze.
   [ADR-0034](docs/adr/ADR-0034-the-freeze-gates-the-cores-language-not-every-boundarys.md)
   proposes how to read it instead; until the project owner accepts it, the
   gate reads as written. Since 2026-10-04 *player movement* is built and
   measured (ADR-0035). So is the *cross-language tool call*: Python over IPC
   costs ~40 µs of boundary per round trip to a running tool and 40–228 ms
   for a tool run once, against ~1.2 ns for an FFI crossing (Appendix P,
   Finding 34). What remains is the mod boundary, the engine-scale comparison
   and the ADR-0034 decision. If ADR-0034 is accepted, its own list also
   requires the scorecard and the core-lock ADR.
2. ~~**The RHI has met a driver and a window, but not the operator's
   hardware.**~~ **Closed 2026-09-26.** The contract, a null backend and a
   native `wgpu` backend are built (ADR-0025, ADR-0026, ADR-0028), and a
   `winit` window host presents through it (ADR-0027). They pass the
   conformance suite, the upload, draw and bound-draw readbacks and, with
   presentation on, a frame read back from the window's surface: in CI on
   Vulkan (lavapipe), Direct3D 12 (WARP) and Metal (paravirtual), and on the
   operator's **AMD Radeon RX 6650 XT over Vulkan on a Win32 desktop**, in two
   local reports on code identical to `HEAD`. DEBT-0046 is closed. What this
   does not claim: Direct3D 12 or Metal on hardware, any other GPU, or a
   device loss raised by a driver. None of those is a contract question the
   freeze depends on; each is a renderer test for later.
