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
| RHI boundary | [x] | [~] | the **contract** is built (`engine/rhi`, ADR-0025): generational handles that a device loss also kills, all-or-nothing submission, four-byte copy alignment, declared usages, ordered fences, destruction deferred until the last use's fence, device loss as `DisableSubsystem` with `recreate`, `present` exactly when the capabilities say. A **null backend** keeps every rule with no GPU, and an eleven-case **conformance suite** runs against it on every slice, which also uploads the first generation's sixteen albedos through it. The **first native backend is built** (`engine/rhi-wgpu`, ADR-0026): `wgpu` over Vulkan / Direct3D 12 / Metal, WGSL validated by naga, rules shared with the null backend through `rhi::kit`. It passes the same eleven cases, uploads a texture and draws a triangle, and reads both back. Since ADR-0028 the contract also declares **vertex layouts, binding slots (uniform, texture, sampler) and a depth test**, and a draw that samples a texture through a uniform-tinted pipeline with depth is read back texel for texel on the driver. It is **verified in CI on all three of its APIs**: Vulkan (Mesa's lavapipe, Linux), Direct3D 12 (WARP, Windows) and Metal (Apple's paravirtual device, macOS). All three are software or virtual, and it is **not yet verified on the operator's GPU** (the `rhi_native` check waits for a report). It **presents** (ADR-0027): a `winit` window host (`engine/window`) owns the event loop, the backend opens a surface on the window and `present` draws the target onto it; CI reads the first frame back from the surface on Xvfb and matches all 65,536 texels. Not yet verified on the operator's desktop (the `window` check waits for the same report) |
| Input boundary | [x] | [ ] | a window and an OS event loop exist now (ADR-0027); no device event reaches `runtime::input` yet (DEBT-0043) |
| Audio boundary | [x] | [ ] | — |
| Asset lifecycle | [x] | [~] | `ResourceID → Manifest → Resolver → Loader → Cache → Handle` built, with integrity checked before any loader runs and declared fallbacks (`engine/resource`, ADR-0021); the runtime decodes its own textures with the one PNG decoder, bounded by each file's declared size (`engine/image`, ADR-0022); it inflates with the foundation's `inflate_bounded`, all three deflate block types, one inflater in the workspace; resource packs do not layer |
| Streaming lifecycle | [x] | [~] | `request · cancel · set_interest · tick` of `STREAMING SYSTEM.md` are built (ADR-0008), and of its test list fast travel, save-before-evict and low-memory pressure are covered (`engine/streaming::system` tests); the slice's **initial** load is still explicit, through the job system rather than through streaming (DEBT-0018, DEBT-0024), and dimension transfer and reconnect have no subsystem to test against. *Corrected 2026-09-25: the row said `[ ]` and predated ADR-0008.* |
| Headless mode | [x] | [x] | `nexora-headless` |

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
| Technology benchmark | [x] | [~] | harness built; a second stack (C++20 kernels, two compilers) is now measured and conformance-gated, FFI overhead included ([Appendix D](docs/benchmarks/PHASE-0-BASELINE.md)); **the gate is still open** — the GPU stages cannot run here and no engine-scale comparison exists (DEBT-0008, ADR-0009) |

## Phase status (2026-09-24)

Two numbering schemes meet here, and they must not be confused.
`NEXORA DEVELOPMENT ROADMAP.md` calls **Phase 0 the Architecture Freeze**, with
one exit: *"nenhum blocker arquitetural crítico sem decisão registrada"*.
ADR-0005 and the README use "Phase 0" for the **first implementation
increment**, whose scope that ADR bounds. Only the roadmap's phases gate
anything.

| Roadmap phase | State | Evidence |
| --- | --- | --- |
| 0 — Architecture Freeze | **open: blocked by unbuilt code, not by the environment** | the Final gate below is not met: the benchmark measures the RHI stage (on software Vulkan so far) but has no frame time, no camera and no engine-scale comparison (DEBT-0008), and the RHI has not run on the operator's hardware. Its contract, a null backend and a native `wgpu` backend are built and pass the same suite, the native one on a real (software) Vulkan driver in CI, and it presents to a real window (ADR-0025, ADR-0026, ADR-0027). *Reclassified 2026-09-25: this row said "blocked by environment", but lavapipe gives every headless GPU path a conformant driver, so what remains is code (window, presentation, the benchmark's GPU stages) plus one local report on real hardware.* Both are recorded with decisions (ADR-0001, ADR-0005, ADR-0009); neither can honestly be reclassified as a post-freeze extension, because the RHI row is exactly the contract that has not been tested |
| 1 — Engine Bootstrap | **largely built, not exited** | core, modules, jobs, time, spatial, registry, events, commands, diagnostics, configuration and now resources are built and run in CI. Exit asks the runtime to start *"em modo client/headless"*: headless does; client needs a window (ADR-0005), and now has one (ADR-0027), but nothing runs the frame loop, a renderer or input inside it yet |
| 2 — Voxel Vertical Slice | partly built ahead of order | chunk, storage, meshing, coordinates, streaming exist; camera, input, render and player do not |

Work continues on what can be verified headless — the roadmap's own rule is
that a phase advances on its technical criteria, and the criteria that remain
need hardware this environment does not have. Nothing here claims otherwise.

**Local evidence (2026-09-25): three reports, one machine** —
[`docs/validation/local/NEXORA-LOCAL-VALIDATION.md`](docs/validation/local/NEXORA-LOCAL-VALIDATION.md).
The current one ran on commit `0b7bcec`, a real Windows 10 machine (AMD Ryzen
5 5500, 12 threads, 16 GiB, AMD Radeon RX 6650 XT). Release build, **1138
tests**, the slice with and without the first generation, the forge's build of
it, the textures and the whole CPU benchmark: all `VERIFIED_ON_LOCAL_HARDWARE`
for that commit. The two earlier reports (`700eed6`, `7991083`) said the same
but lost the benchmark's numbers, which the script now keeps. The third report's
numbers closed DEBT-0013 (baseline Appendix I). Every GPU item stays
`NOT_IMPLEMENTED`: the reports prove the machine has a GPU, not that the engine
can use one, and they move neither blocker. The report predates `engine/rhi`,
so it reads as `STALE_LOCAL_EVIDENCE` against `HEAD`, and the RHI's null backend
has not yet run on that machine. The previous note, kept for the record:

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

**Not met.** Two blockers stand out:

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
   Finding 28). Frame time and camera still have no code (there is no
   renderer), and no GPU number exists until the operator's report runs it.
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
2. **The RHI has met a driver and a window, but not the operator's
   hardware.** Its contract, a null backend and a native `wgpu` backend are
   built (ADR-0025, ADR-0026), and a `winit` window host presents through it
   (ADR-0027). The native one passes the conformance suite, an
   upload-and-draw readback and, with presentation on, a frame read back from
   the window's surface, on Mesa's software Vulkan and Xvfb in CI. So the
   boundary has survived contact with a conformant driver and a real window
   system. One thing remains before this blocker closes: **a report from the
   operator's GPU and desktop** (`rhi_native` and `window` in
   `local-validation.py`). See DEBT-0046.
