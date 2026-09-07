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
| Resource ownership | [x] | [ ] | `NEXORA MEMORY AND RESOURCE OWNERSHIP.md` |
| Time model | [x] | [x] | `foundation::time` |
| Spatial model | [x] | [x] | `foundation::spatial` |
| Registry / ID rules | [x] | [x] | `runtime::registry`, `foundation::ident` |
| Event / Command / Query semantics | [x] | [~] | events built; **commands and queries are not** (DEBT-0007) |

## Runtime

| Contract | Defined | Built | Where |
| --- | :---: | :---: | --- |
| RHI boundary | [x] | [ ] | no display or GPU to verify against (ADR-0005) |
| Input boundary | [x] | [ ] | meaningless without a window |
| Audio boundary | [x] | [ ] | — |
| Asset lifecycle | [x] | [ ] | `RESOURCE AND ASSET SYSTEM.md` |
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
| Performance budgets | [x] | [~] | storage, counters and now physics are measurable; no budgets published or enforced (DEBT-0013) |
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
| Content pipeline | [x] | [ ] | |
| Asset provenance | [x] | [ ] | no assets ship yet |
| Original-content policy | [x] | [x] | no third-party code or assets; algorithms implemented from published specifications (ADR-0002) |
| Editor / runtime relationship | [x] | [ ] | |
| Mod API versioning | [x] | [~] | version types exist; no mod API |

## Engineering

| Contract | Defined | Built | Where |
| --- | :---: | :---: | --- |
| Save compatibility | [x] | [x] | `foundation::version`, ADR-0004 |
| Crash / recovery strategy | [x] | [~] | detect, quarantine and atomic write are built; journal recovery is not (DEBT-0001) |
| Observability | [x] | [x] | `foundation::diagnostics` |
| Testing strategy | [x] | [x] | 187 tests; unit, integration, property, determinism, corruption |
| CI / build / release strategy | [x] | [~] | format, lint, test, build and smoke run in CI; packaging and release do not |
| Technology benchmark | [x] | [~] | harness built; every slice stage is now either measured or blocked on a GPU or a second language, streaming included ([baseline](docs/benchmarks/PHASE-0-BASELINE.md)); **the gate is still open** — no second stack has been measured (DEBT-0008) |

## Final gate

The architecture can be frozen only when unresolved items are either completed
or explicitly classified as post-freeze extensions with no impact on frozen
contracts.

**Not met.** Two blockers stand out:

1. **The technology benchmark has one stack, and needs two** (DEBT-0008).
   Every stage of the plan's vertical slice that can be measured without a GPU
   or a second language is now measured — see
   [`docs/benchmarks/PHASE-0-BASELINE.md`](docs/benchmarks/PHASE-0-BASELINE.md),
   Appendix B closes the last of them. But no *second* stack has been measured,
   and rule 5 of the plan forbids deciding from one. ADR-0001 keeps the language
   gate open on purpose; freezing the architecture before that comparison would
   settle by default the question the gate exists to ask. **More Rust does not
   move this blocker** — and Appendix C is the correction to an earlier version
   of that sentence, which claimed the Rust side was finished one subsystem
   before it was.
2. **The RHI and presentation boundary is unbuilt.** It is specified, but no
   implementation has ever run, so nothing has tested whether the boundary
   survives contact with a real renderer.
