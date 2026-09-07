# NEXORA — Phase 0 benchmark baseline

**Date:** 2026-09-06 · **Commit:** Phase 0 foundation · **Runner:** `cargo run --release -p nexora-benchmark`

## What this is, and what it is not

`NEXORA TECHNOLOGY BENCHMARK PLAN.md` exists to choose the implementation stack
**by evidence**. [ADR-0001](../adr/ADR-0001-rust-phase-0-reference-implementation.md)
kept that gate open and named the Rust implementation the *reference* the gate
would measure. This document is that reference measurement.

**It does not close the gate.** Two reasons, both from the plan itself:

1. **Coverage.** The plan's vertical slice has thirteen stages. Phase 0
   implements four (chunk, jobs, save/load, headless). The other nine are listed
   as *not measured*, with reasons, rather than omitted — a benchmark table with
   silent gaps reads as one that covered everything.
2. **Rule 5: "Não escolher pelo benchmark de um único microcaso."** These are
   absolute numbers for one stack. A gate needs a comparison. What did not exist
   before today was anything to compare *against*; that is what this provides.

`DEBT-0008` stays open.

## Method

Recorded per rule 4 of the plan.

- Warm-up runs precede every measurement, so first-touch page faults and a cold
  branch predictor are not reported as steady state.
- Each sample times a batch of iterations and divides, keeping the clock read
  out of measurements that cost nanoseconds.
- **Median and p95** are reported, not the mean alone: a mean hides the tail a
  budget actually has to survive. Relative σ is shown so a noisy run is visible
  rather than quietly averaged away.
- Every timed value passes through `std::hint::black_box`, so the optimizer
  cannot delete work whose result is unused.
- Numbers below are from one run; two further runs agreed within the reported σ.

Harness: [`engine/benchmark`](../../engine/benchmark).

## Environment

| | |
| --- | --- |
| logical CPUs | 4 |
| build profile | `release` (thin LTO, 1 codegen unit) |
| architecture | `x86_64` |
| peak resident memory, whole run | **7.0 MiB** |
| benchmark binary size | 803.1 KiB |

A container, not dedicated hardware. Ratios between measurements are meaningful;
absolute values should be re-measured on the target machine before being written
into a performance budget.

## Measurements

| measurement | median | p95 | rel. σ |
| --- | ---: | ---: | ---: |
| `startup.world_create` | 683 ns | 1.06 µs | 19.0% |
| `spatial.section_of_division` | **1.7 ns** | 2.0 ns | 6.9% |
| `spatial.section_of_shift_runtime` | 6.7 ns | 7.1 ns | 2.7% |
| `spatial.section_of_shift_const` | 6.8 ns | 7.4 ns | 4.4% |
| `spatial.index_of` | 2.7 ns | 3.1 ns | 6.0% |
| `voxel.get_uniform` | 2.6 ns | 3.6 ns | 14.9% |
| `voxel.get_paletted` | 5.2 ns | 6.1 ns | 6.2% |
| `voxel.set_existing_state` | 10.4 ns | 11.9 ns | 8.0% |
| `voxel.first_write_to_uniform` | 54.4 ns | 55.8 ns | 6.9% |
| `voxel.compact_section` | 155.29 µs | 160.59 µs | 1.6% |
| `worldgen.surface_height` | 29.5 ns | 36.5 ns | 9.7% |
| `worldgen.chunk_16` | 202.39 µs | 218.00 µs | 4.0% |
| `worldgen.chunk_32` | 1.29 ms | 1.33 ms | 1.4% |
| `jobs.submit_only` | 8.84 µs | 9.70 µs | 4.3% |
| `jobs.submit_wait_roundtrip` | 18.55 µs | 26.63 µs | 28.2% |
| `jobs.batch_1000_barrier` | 8.85 ms | 10.00 ms | 6.6% |
| `save.encode` | 103 MiB/s | 1.51 ms | 1.9% |
| `save.decode` | 104 MiB/s | 1.44 ms | 0.4% |
| `save.read_disk` | 96 MiB/s | 1.58 ms | 1.1% |
| `save.write_atomic_disk` | 37 MiB/s | 4.35 ms | 6.7% |
| `save.world_load` | 2.28 ms | 2.35 ms | 2.1% |
| `save.size_9_chunks` | 152.1 KiB | — | — |
| `world.voxel_storage_9_chunks` | 144.3 KiB | — | — |
| `world.non_air_blocks_9_chunks` | 18 293 973 | — | — |

### Not measured

| stage the plan asks for | why there is no number |
| --- | --- |
| window, input, RHI, camera | no windowing or graphics backend; ADR-0005 |
| mesh generation | no renderer to consume a mesh |
| 1,000 entities, physics | entity system not implemented; ADR-0005 |
| streaming | chunks load explicitly; no streaming manager |
| mod boundary | Phase 7 |
| frame time | no render loop exists to time |
| FFI overhead | single-language build; nothing crosses a boundary |
| incremental build | belongs to the build system, not this process |
| debugging / tooling effort | qualitative; the plan scores it separately |

## Findings

### 1. `DEBT-0005` is answered, and the answer is "do nothing"

The debt register recorded that `ChunkShape` is a runtime value — required by
`CHUNK & VOXEL ENGINE.md` §3 — so `section_of` performs a real integer division,
and asked whether that costs anything. It deliberately said **"Desconhecido —
não medido"** rather than optimizing on a hunch.

Measured, the division is **1.7 ns**. The shift-based shortcut that the debt
proposed is **6.7 ns**, and even with a compile-time width it is **6.8 ns** —
about **4× slower** than the thing it was supposed to improve.

The likely reason: `section_of` is a `const fn`, so where the optimizer can see
the extents it folds the division into an optimal inlined sequence, while the
hand-written shift does not receive the same treatment. That explanation is a
hypothesis; the timing is not.

> **`DEBT-0005` → WONT FIX.** The proposed optimization would make the engine
> slower. **Re-open trigger:** profiling shows `section_of` hot at a call site
> where the shape is genuinely opaque to the optimizer (behind a trait object,
> or read from a save at runtime), since this measurement cannot speak for that
> case.

This is the Baluarte lesson applied to NEXORA: *measure before you blame*. Had
this been "optimized" on intuition, the result would have been a regression
defended by a plausible story.

### 2. The job system has a throughput ceiling worth knowing about

Enqueueing a job costs **8.84 µs**, and 1,000 trivial jobs through a 4-worker
pool take **8.85 ms** — about **113,000 jobs/second**, essentially all of it
scheduling overhead rather than work.

Against chunk generation at 1.29 ms, that overhead is **0.7%** — irrelevant. Per
entity it is fatal: 10,000 entities as individual jobs would cost 88 ms of pure
overhead before any entity was simulated.

The likely cause is a futex wake per `notify_one` on submit, plus workers
contending on one mutex. Recorded as **`DEBT-0009`**, targeted before Phase 5,
where per-entity work arrives.

The architecture is not wrong — `NEXORA THREADING AND CONCURRENCY MODEL.md`
already says heavy work should be *divided into independent jobs*, and chunk
granularity is exactly that. The finding is that the current implementation sets
the minimum useful job size at roughly a millisecond.

### 3. Storage compression is doing what it was designed to do

**18 293 973 non-air blocks in 144.3 KiB** of voxel storage — about **124 blocks
per byte**. Uniform sections mean solid rock genuinely costs nothing, and peak
resident memory for the whole benchmark was **7.0 MiB**.

The save on disk is 152.1 KiB, close to the in-memory figure, which confirms the
container framing and checksums add negligible overhead at this size. Chunk
compression (`DEBT-0003`) has nothing to compress yet at this density; it will
matter once player-built structures break sections into large palettes.

### 4. The atomic-write guarantee costs about 2.6× on writes

Writes run at **37 MiB/s** against reads at **96 MiB/s**. The gap is the
guarantee: temp file, `fsync`, decode the bytes back to verify, then rename. That
is `DEBT-0006`, now quantified — 4.35 ms for a 152 KiB save.

**Keep it.** Millisecond-scale cost buys "never replace a valid save with
partially written data" as a demonstrable property rather than an intention.
Revisit only when a save is large enough for the read-back to dominate, and then
only with an ADR.

### 5. Smaller sections generate more efficiently per block

`worldgen.chunk_16` covers a 16×16 footprint against 32×32 for `chunk_32` — a
quarter of the columns — so the two are not the same work. Normalised, 16³ costs
roughly **0.40 ns/block** against **0.63 ns/block** for 32³.

Plausibly because a shorter section spans less of the vertical range, so a larger
fraction of sections fall entirely below the soil layer and collapse to a
zero-cost uniform. This is an observation, not a recommendation: section size
also drives meshing, streaming and network granularity, none of which are
implemented. **Do not change the default on the strength of this number.**

## What the gate still needs

| requirement (plan §17-§18) | status |
| --- | --- |
| Same workload on a second candidate stack | **missing** — nothing to compare against yet |
| RHI, window, camera measured | missing — needs hardware and a backend |
| 1,000 entities, physics | missing — needs the entity system |
| Streaming, mesh generation | missing |
| FFI / IPC boundary cost | missing — needs a second language in the build |
| Build and iteration time | not captured by this harness |
| Serialization, save/load | **done** |
| Job overhead | **done** |
| Memory, binary size, startup | **done** |
| Deterministic simulation | **done** — byte-identical saves across worker counts, in CI |

Roughly a third of the gate. The next measurable increments are the entity
system and the RHI, in that order: entities need no hardware, and the renderer
cannot be measured in a headless container at all.

## Reproducing

```bash
cargo run --release -p nexora-benchmark              # plain text
cargo run --release -p nexora-benchmark -- --markdown # this table
cargo run --release -p nexora-benchmark -- --smoke    # minimal pass, used by CI
```

CI runs `--smoke` so the harness cannot rot unnoticed. It does not gate on
timings: a shared runner's numbers would produce false failures, and a
performance gate that cries wolf gets disabled.

---

# Appendix A — entity increment (2026-09-06, later run)

The entity system landed after the measurements above, adding the benchmark
plan's **"1,000 entities"** stage. These numbers come from a **separate, later
run**, recorded separately rather than merged into the table above.

## Why separate, and a warning about reading across the two

Measurements that the entity work does not touch moved uniformly between the two
runs:

| unchanged code | first run | this run |
| --- | ---: | ---: |
| `worldgen.chunk_32` | 1.29 ms | 2.47 ms |
| `voxel.first_write_to_uniform` | 54.4 ns | 89.6 ns |
| `jobs.submit_only` | 8.84 µs | 13.02 µs |

Entities do not touch world generation, voxel storage or the job queue. A
uniform ~2× shift across unrelated measurements is the **shared container being
busier**, not a regression. Merging the two runs into one table would have
manufactured a regression that did not happen.

**Compare within a run, never across these two.** The ratios below are all taken
from a single execution.

## Entity measurements

| measurement | median | p95 | rel. σ |
| --- | ---: | ---: | ---: |
| `entity.resolve_handle` | **2.6 ns** | 2.7 ns | 0.0% |
| `entity.spawn_despawn_cycle` | 151.7 ns | 152.5 ns | 0.5% |
| `entity.spawn` | 199.1 ns | 568.2 ns | 59.3% |
| `entity.step_1000` | **2.95 µs** | 3.49 µs | 7.0% |
| `entity.query_type_1000` | 18.73 µs | 19.47 µs | 1.7% |
| `entity.query_radius_1000` | 18.73 µs | 18.82 µs | 0.9% |
| `entity.query_tag_1000` | 22.06 µs | 22.34 µs | 0.6% |
| `entity.save_1000` | 780 MiB/s | 193.90 µs | 4.3% |
| `entity.load_1000` | 186 MiB/s | 822.67 µs | 4.1% |
| `entity.save_size_1000` | 141.6 KiB | — | — |

## Findings

### 6. The dense column layout works, and the safety property is free

Advancing 1,000 entities costs **2.95 µs** — about **3 ns per entity** for a
linear walk over parallel `Vec` columns. Validating a handle costs **2.6 ns**,
so the generational check that makes a destroyed entity unaddressable
(ADR-0006) is not a cost worth discussing.

`entity.spawn` has a 59% relative σ against `entity.spawn_despawn_cycle`'s 0.5%.
That is the difference between growing the columns and reusing a freed slot:
the spawn benchmark keeps allocating, so it periodically pays for a `Vec`
reallocation, while the cycle benchmark reaches steady state. Both numbers are
real; they answer different questions.

### 7. `DEBT-0009` now has a number, and it is worse than expected

Measured in the same run:

| | |
| --- | ---: |
| simulate 1,000 entities (`entity.step_1000`) | **2.95 µs** |
| submit 1,000 jobs and barrier (`jobs.batch_1000_barrier`) | **14.48 ms** |
| ratio | **≈ 4,900×** |

Scheduling one job per entity would cost roughly **four thousand nine hundred
times** the simulation it schedules. The earlier estimate — "0.7% overhead at
chunk granularity, fatal per entity" — understated it.

This is not an argument against the job system; it is a measurement of its
correct granularity. `NEXORA THREADING AND CONCURRENCY MODEL.md` asks for heavy
work to be divided into independent jobs, and a chunk at ~2.5 ms is exactly
that. **One job per entity is an anti-pattern with a number attached**, which is
worth more than the same advice as an opinion.

### 8. Queries have a measured expiry date

A linear scan over 1,000 entities costs **18.7 µs** by type, **22.1 µs** by tag —
roughly **19 ns per entity examined**. `Entity System.md` §34 specifies a spatial
index; ADR-0006 defers it. Extrapolating that per-entity cost:

| population | one full scan |
| ---: | ---: |
| 1 000 | ~19 µs |
| 10 000 | ~190 µs |
| 100 000 | ~1.9 ms |

At 1,000 entities an index would be premature. At 100,000, a single query eats a
large fraction of a tick, and AI that queries per entity is finished long before
that. Recorded as **`DEBT-0010`** with that trigger point, so the index gets
built on evidence instead of on the day someone guesses it is time.

Tag queries cost ~18% more than type queries — the binary search through a
`TagSet` against a single comparison. Small enough that the `Vec`-backed tag set
(chosen over an interned bitset precisely to avoid an unmeasured optimization)
is holding up.

### 9. Entities are cheap to persist and cost more to load

**780 MiB/s** out, **186 MiB/s** back — loading is ~4× slower because it
re-resolves every persistent id and rebuilds the lookup map, which is the price
of not writing slot indices to disk. At **~145 bytes per entity**, 100,000
entities would be a ~14 MiB save section: large but not alarming, and compressible.

## Gate progress

Superseded by Appendix B; see the table there.



---

# Appendix B — physics increment (2026-09-07)

Physics landed after the appendix above, adding the benchmark plan's **`physics`**
stage. That was the last stage of the plan's vertical slice measurable without a
GPU or a second language in the build.

## Which run these belong to

This run sits in the **same regime as Appendix A**, not the original table:

| unchanged code | first run | Appendix A | this run |
| --- | ---: | ---: | ---: |
| `worldgen.chunk_32` | 1.29 ms | 2.47 ms | 2.48 ms |
| `entity.step_1000` | — | 2.95 µs | 2.66 µs |
| `voxel.set_existing_state` | — | — | 26.7 ns |

So the physics numbers below may be compared with Appendix A's, and **must not**
be compared with the original table's. Every ratio quoted below is taken from a
single execution, and every figure was reproduced within 4% on a second run.

## Physics measurements

| measurement | median | p95 | rel. σ |
| --- | ---: | ---: | ---: |
| `physics.timestep_accumulate` | **9.1 ns** | 10.3 ns | 4.7% |
| `physics.depenetration_check` | 166.2 ns | 170.7 ns | 2.1% |
| `physics.box_sweep` | **394.1 ns** | 399.0 ns | 0.7% |
| `physics.character_step` | **1.00 µs** | 1.03 µs | 1.1% |
| `physics.thousand_sleeping_step` | **1.61 µs** | 1.77 µs | 3.8% |
| `physics.raycast_40m` | 2.39 µs | 2.40 µs | 0.4% |
| `physics.thousand_bodies_step_flat` | 149.27 µs | 165.95 µs | 5.9% |
| `physics.thousand_bodies_step` | **293.82 µs** | 305.47 µs | 3.2% |
| `physics.sleeping_bodies_of_1000` | 1 000 | — | — |

## Findings

### 10. Half the cost of a physics step is the world lookup, not the solver

The same thousand bodies, the same substep, differing only in where the terrain
comes from:

| | median | per body |
| --- | ---: | ---: |
| against generated terrain | **293.82 µs** | 294 ns |
| against a flat fixture | **149.27 µs** | 149 ns |
| difference — the voxel lookup | **144.55 µs** | 145 ns |

**49% of a physics step is spent asking the world what is there.** That is the
single most useful number in this appendix, because it says where optimisation
would pay and where it would not: a faster solver would address the smaller
half. The lookup walks a `BTreeMap` of chunk columns and then a palette-indexed
section, once per cell per axis per body, with no caching between queries that
are almost always in the same chunk.

Recorded as **`DEBT-0011`**, with the concrete shape a fix would take (hold the
resident chunk across a body's sweeps) and the trigger for building it.

The pair of measurements is the point. A single "physics costs 294 ns per body"
number would have been attributed to the solver by default, and the work would
have gone to the wrong half.

### 11. Sleeping is worth about 180×, and it actually engages

| 1,000 bodies, one substep | median |
| --- | ---: |
| awake, against generated terrain | 293.82 µs |
| asleep | **1.61 µs** |
| ratio | **≈ 182×** |

`physics.sleeping_bodies_of_1000` is **1000 of 1000**: after ten seconds on a
floor, every body has settled. `PHYSICS.md` §35 calls sleeping "fundamental for
performance" without a number; this is the number, and it confirms that the
settle criterion is reachable rather than theoretically correct and practically
never met.

The residual 1.61 µs is the loop over slots that skips them — 1.6 ns per
sleeping body, which is the cost of asking rather than the cost of doing.

### 12. The depenetration check is a 42% tax on the common case

`physics.depenetration_check` measures the test **when the body is not
penetrating**, which is the case essentially always. At **166.2 ns** against a
**394.1 ns** box sweep, every body pays roughly **42% of a sweep** each step for
a recovery path that almost never fires.

That is not an argument for deleting it: without the pass, a body that ends up
inside terrain — a block placed where it stands, a chunk generating around it —
sinks forever, and the defect that motivated it was found by a test, not by
theory. But it is a scan of the body's whole cell volume on every step, and the
cheap version of the same question is available: the sweep already visits those
cells. Recorded as **`DEBT-0012`**.

### 13. Collision, not the entity walk, is what a tick actually costs

Two measurements from the same run, both over 1,000 objects:

| | median | per object |
| --- | ---: | ---: |
| `entity.step_1000` — advance transforms | 2.66 µs | 2.7 ns |
| `physics.thousand_bodies_step` — collide against terrain | 293.82 µs | 294 ns |
| ratio | | **≈ 110×** |

Moving a thousand entities is nearly free; deciding what they may move *through*
is a hundred times more. Any budget written from the entity number alone would
be wrong by two orders of magnitude.

At 60 physics steps per second, a thousand permanently-awake bodies would cost
**~17.6 ms of CPU per wall-clock second**, or about 1.8% of one core — which is
affordable, and which is affordable *because* finding 11 says almost nothing
stays awake. `NEXORA PERFORMANCE BUDGETS.md` asks every major system to publish
`TARGET`/`WARNING`/`CRITICAL` figures and physics has not published any; these
are the first measurements a budget could honestly be set from, and setting one
from a single machine's single run would be unearned precision. Recorded as
**`DEBT-0013`**.

### 14. A ray costs six box sweeps, and that is the wrong instinct to trust

`physics.raycast_40m` walks 40 metres of generated terrain in **2.39 µs**,
against **394.1 ns** for a full three-axis box sweep. A ray looks like the
cheaper primitive and is six times the cost here, because it visits ~40 cells
against the sweep's handful. Anything that raycasts per entity per frame — AI
line-of-sight is the obvious candidate — needs the budget worked out from this
number rather than from the intuition that a ray is light.

## Gate progress

| requirement (plan §17-§18) | status |
| --- | --- |
| **Physics** | **done** — this appendix |
| 1,000 entities | done — Appendix A |
| Serialization, save/load | done |
| Job overhead | done |
| Memory, binary size, startup | done |
| Deterministic simulation | done |
| Same workload on a second candidate stack | **missing** — the blocker |
| RHI, window, camera, mesh generation | missing — needs hardware |
| Streaming | missing |
| FFI / IPC boundary cost | missing |
| Build and iteration time | not captured by this harness |

**Every stage of the plan's vertical slice that can be measured without hardware
or a second language is now measured.** What remains is not more Rust: it is a
GPU for the render stages, a streaming manager, and — the one that actually
blocks the decision — the same workload built a second time in another language.

**The gate still cannot close.** Rule 5 forbids deciding from one stack, and
there is still only one. Nothing in this appendix changes that; it only removes
the last excuse that the Rust side was not measured enough.
