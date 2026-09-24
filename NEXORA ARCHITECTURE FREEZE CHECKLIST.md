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
| Resource ownership | [x] | [~] | the asset cache is built — byte budget, priority, last use, pinning, eviction, counters, `Arc` so eviction never takes a value from its holder (`engine/resource`, ADR-0015); **per-owner memory budgets** are built — one ledger, eight classes, `TARGET/WARNING/CRITICAL/EMERGENCY`, high-water marks, worst pressure, refusals, suspected leaks (`foundation::memory`, ADR-0018) — and the world, retained chunks and texture cache record into it, gated in CI; the `Frame`, `Gpu`, `Network`, `Script` and `Editor` classes have no owner yet because none of those subsystems exists |
| Time model | [x] | [x] | `foundation::time` |
| Spatial model | [x] | [x] | `foundation::spatial` |
| Registry / ID rules | [x] | [x] | `runtime::registry`, `foundation::ident` |
| Event / Command / Query semantics | [x] | [~] | events, **commands** (`engine/command`, ADR-0010) and **queries** (`engine/query`, ADR-0017: read-only by type, versioned, deny-by-default through the same `SourcePolicy`, budget re-counted after the handler, answers in identifiers) are built; commands stop at CMD-4 (DEBT-0021) and queries read the live world, not snapshots |

## Runtime

| Contract | Defined | Built | Where |
| --- | :---: | :---: | --- |
| RHI boundary | [x] | [ ] | no display or GPU to verify against (ADR-0005) |
| Input boundary | [x] | [ ] | meaningless without a window |
| Audio boundary | [x] | [ ] | — |
| Asset lifecycle | [x] | [~] | `ResourceID → Manifest → Resolver → Loader → Cache → Handle` built, with integrity checked before any loader runs and declared fallbacks (`engine/resource`, ADR-0015); the runtime decodes its own textures with the one PNG decoder, bounded by each file's declared size (`engine/image`, ADR-0016); resource packs do not layer, and deflate's dynamic-Huffman blocks are not read |
| Streaming lifecycle | [x] | [ ] | chunks are loaded explicitly for now |
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
| Performance budgets | [x] | [~] | the `MEMORY` dimension is published and enforced for the slice's pools, from measured per-column figures (ADR-0018); time budgets are not, because a shared runner's clock is noise (DEBT-0013) |
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
| Validation invariants | [x] | [~] | foundation and world invariants are enforced and tested; cross-system validation is not |
| Mod / script trust model | [x] | [ ] | configuration namespace isolation is built; no script sandbox |

## Content / tools

| Contract | Defined | Built | Note |
| --- | :---: | :---: | --- |
| Content pipeline | [x] | [~] | for surface materials: authored definition → recipe → generate → validate → PNG → batch by manifest → INDEX (`tools/texture-forge`, ADR-0013, ADR-0015); no other asset type has a pipeline |
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
| 0 — Architecture Freeze | **open, blocked by environment** | the Final gate below is not met: the benchmark cannot run its GPU stages or an engine-scale comparison here (DEBT-0008), and the RHI boundary has never met a renderer. Both are recorded with decisions (ADR-0001, ADR-0005, ADR-0009); neither can honestly be reclassified as a post-freeze extension, because the RHI row is exactly the contract that has not been tested |
| 1 — Engine Bootstrap | **largely built, not exited** | core, modules, jobs, time, spatial, registry, events, commands, diagnostics, configuration and now resources are built and run in CI. Exit asks the runtime to start *"em modo client/headless"*: headless does; client needs a window (ADR-0005) |
| 2 — Voxel Vertical Slice | partly built ahead of order | chunk, storage, meshing, coordinates, streaming exist; camera, input, render and player do not |

Work continues on what can be verified headless — the roadmap's own rule is
that a phase advances on its technical criteria, and the criteria that remain
need hardware this environment does not have. Nothing here claims otherwise.

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
   kernels; two things it asked for are still missing, and neither is fixable by
   writing more code here. **The GPU stages** (RHI, window, camera, mesh) cannot
   run in a headless container. **An engine-scale comparison** is explicitly out
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
2. **The RHI and presentation boundary is unbuilt.** It is specified, but no
   implementation has ever run, so nothing has tested whether the boundary
   survives contact with a real renderer.
