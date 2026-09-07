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

Superseded by Appendix C; see the table there.

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

> **Correction (Appendix C).** This appendix originally claimed that every
> stage measurable without hardware or a second language was now measured.
> That was wrong, and its own "not measured" table contradicted it two
> paragraphs later: **streaming** needed neither a GPU nor a second language —
> it needed a streaming manager, which is ordinary headless code. Appendix C
> builds and measures it. The claim below is the corrected one.

Physics was the last *unbuilt subsystem the slice already depended on*. What
remains is a GPU for the render stages, **streaming** (built in Appendix C),
and — the one that actually blocks the decision — the same workload built a
second time in another language.

**The gate still cannot close.** Rule 5 forbids deciding from one stack, and
there is still only one.

---

# Appendix C — streaming increment (2026-09-07, later run)

Streaming landed after the appendix above, adding the benchmark plan's
**`streaming`** stage — which Appendix B wrongly implied was blocked on
hardware. It was not. It was unbuilt.

## Which run these belong to

Same regime as Appendices A and B, not the original table:

| unchanged code | first run | Appendix A | Appendix B | this run |
| --- | ---: | ---: | ---: | ---: |
| `worldgen.chunk_32` | 1.29 ms | 2.47 ms | 2.48 ms | 2.61 ms |
| `entity.step_1000` | — | 2.95 µs | 2.66 µs | 2.46 µs |

Compare within a run. Every ratio below is from a single execution.

## Streaming measurements

| measurement | median | p95 | rel. σ |
| --- | ---: | ---: | ---: |
| `streaming.chunk_retained_cycle` | **338.9 ns** | 474.7 ns | 15.4% |
| `streaming.idle_tick_r3` | **4.70 µs** | 4.91 µs | 2.1% |
| `streaming.walk_one_chunk` | 9.72 µs | 9.92 µs | 1.1% |
| `streaming.fast_travel_r3` | 68.38 µs | 70.01 µs | 1.1% |
| `streaming.idle_tick_r12` | **167.47 µs** | 172.97 µs | 1.4% |
| `streaming.chunk_generate_cycle` | **2.72 ms** | 3.16 ms | 7.9% |
| `streaming.retained_bytes_per_chunk` | 20.0 KiB | — | — |
| `streaming.columns_of_interest_r12` | 625 | — | — |

## Findings

### 15. Retention beats regeneration by about eight thousand times

The design question the backend had to answer: when a chunk is evicted, keep it
or regenerate it later?

| coming back to an evicted column | median |
| --- | ---: |
| regenerate it from the seed | **2.72 ms** |
| restore it from retention | **338.9 ns** |
| ratio | **≈ 8 000×** |

So retention wins overwhelmingly — and the engine still does not retain most
chunks, on purpose. Generation is deterministic and position-seeded, so an
**unedited** column is regenerable and is dropped; only an **edited** one is
kept, at **20.0 KiB** each.

The consequence is the one worth remembering: **retained memory grows with how
much the world has been changed, not with how far it has been explored.** A
player who walks a thousand columns and edits ten holds ten columns of memory.
Retaining everything explored would have inverted that.

### 16. The idle tick is not free, and it grows faster than the area

A tick with *nothing to do* still enumerates every column in interest range:

| interest radius | columns | median | per column |
| ---: | ---: | ---: | ---: |
| 3 | 49 | 4.70 µs | 96 ns |
| 12 | 625 | **167.47 µs** | **268 ns** |

12.8× the columns costs **35.6×** the time. The growth is superlinear because
every candidate goes through a `BTreeSet` insert and a `BTreeMap` lookup whose
own costs rise with the size of the set — so the area term is multiplied by a
logarithmic one.

At radius 12 that is 167 µs **on every tick, whether or not anything moved**:
about 1% of a 60 Hz frame, spent deciding that nothing changed. Extrapolating
the per-column cost, radius 24 is roughly 1 ms. Recorded as **`DEBT-0017`**,
with the shape of the fix — the candidate set only changes when an observer
crosses a column boundary, so it can be cached and updated incrementally — and
the radius that triggers it.

### 17. Deciding costs more than moving

| | median | against an idle tick |
| --- | ---: | ---: |
| idle, radius 3 | 4.70 µs | 1× |
| observer moves one column | 9.72 µs | 2.1× |
| fast travel, whole set replaced | 68.38 µs | 14.5× |

Moving one column roughly doubles a tick; replacing the entire resident set
costs fifteen idle ticks. Neither is the problem. **The problem is that the idle
tick has a floor at all**, which is what finding 16 says and what `DEBT-0017`
addresses. Optimising the load path would be optimising the cheap half.

### 18. Generation dominates residency by 578×, so the budget is the parameter

| | median |
| --- | ---: |
| one chunk activated and dropped | **2.72 ms** |
| one idle streaming tick, radius 3 | 4.70 µs |
| ratio | **578×** |

A single chunk generation costs more than five hundred idle ticks. That settles
what a streaming budget is actually sizing: **generation time, not manager
overhead.**

And it produces a number the slice's own configuration fails: at its budget of
8 activations per tick, a tick that spends its whole budget costs
`8 × 2.72 ms ≈ 21.8 ms` — longer than a 60 Hz frame, on the tick thread.

`NEXORA THREADING AND CONCURRENCY MODEL.md` asks for heavy work to be divided
into independent jobs, and the slice's *earlier* stage already generates chunks
across the worker pool. Streaming's activation path does not: it calls
`load_or_generate` synchronously. Recorded as **`DEBT-0018`** — the fix is to
submit generation as jobs and let activation complete on a later tick, which is
what the deferral machinery in the report already exists to describe.

## Gate progress

| requirement (plan §17-§18) | status |
| --- | --- |
| **Streaming** | **done** — this appendix |
| Physics | done — Appendix B |
| 1,000 entities | done — Appendix A |
| Serialization, save/load | done |
| Job overhead | done |
| Memory, binary size, startup | done |
| Deterministic simulation | done |
| Same workload on a second candidate stack | **missing** — the blocker |
| RHI, window, camera, mesh generation | missing — needs a GPU |
| FFI / IPC boundary cost | missing — needs a second language |
| Build and iteration time | not captured by this harness |

Every stage of the plan's vertical slice is now either **measured** or blocked
on something this repository cannot provide by writing more Rust: a GPU, or a
second language. That is a narrower and more defensible statement than the one
Appendix B made, and this time the "not measured" table agrees with it.

**The gate still cannot close.** Rule 5 forbids deciding from a single stack.
The next thing that moves it is not another subsystem — it is the same workload,
built again, in something that is not Rust.

---

# Appendix D — a second stack (2026-09-07, later run)

*"The next thing that moves it is not another subsystem — it is the same
workload, built again, in something that is not Rust."* — Appendix C, above.

This is that. It moves the gate; it does not close it, and the difference
matters enough to state before the numbers rather than after.

## What was built

`benchmarks/cpp/` — a C++20 reference implementation of the arithmetic the
engine runs in its inner loops: SplitMix64 and its derived streams, FNV-1a 64,
CRC-32/ISO-HDLC, Euclidean block→section mapping, palette-compressed section
access, the heightmap and chunk generator, and one axis of the swept-box
collision. Roughly 700 lines. **It is not an engine** — no world, no
persistence, no streaming, no allocator strategy — and Appendix D's conclusions
never reach past the kernels it contains. ADR-0009 has the reasoning.

## Conformance first: 12 digests, all identical

`scripts/compare-stacks.sh` refuses to time anything until every build agrees on
every digest. Two implementations that disagree about what they compute cannot
be compared on how fast they compute it.

| digest | value |
| --- | --- |
| `rng.splitmix64` | `0xd7f5375fa46d2dac` |
| `rng.next_below` | `0xeb0ab73b99f30cfe` |
| `rng.stream_seeds` | `0x0e6cfbe5674a4709` |
| `rng.positional` | `0xd5b76f71dbb067f1` |
| `hash.fnv1a` | `0x85cda2f59434f6cc` |
| `hash.crc32` | `0x1c880e1b88d0cae7` |
| `spatial.section_of` | `0xa5e5cdaa740e5ec8` |
| `world.surface_height` | `0x5bb0ba519f4d32a6` |
| `world.chunk_cells` | `0xc09113a274ebab66` |
| `world.chunk_non_air` | `2,032,268 blocks` |
| `world.chunk_sections` | `63 sections` |
| `physics.sweep` | `0x4f58c702db5dd49f` |

`world.chunk_cells` hashes every cell of a generated chunk, and `physics.sweep`
includes the `1.9 - 0.9 == 0.9999999999999999` case ADR-0007 exists because of.
Three builds — Rust, g++ 13.3.0, clang++ 18.1.3 — produce all twelve bit-for-bit.

## Finding 19 — the compiler backend explains more than the language does

This is the result, and it was nearly missed. The first run used g++ alone:

| build | CRC-32 over 64 KiB | what a single-compiler run would have concluded |
| --- | ---: | --- |
| Rust (LLVM) | **363.97 µs** | — |
| C++ / g++ (GCC) | **668.83 µs** | *"Rust is 1.8× faster than C++"* |
| C++ / clang++ (LLVM) | **353.11 µs** | *"C++ is 3% faster than Rust"* |

Same source, same `-O2`, same branchless reflected-CRC loop, same 64 KiB from
the same seeded stream. The 1.8× was **GCC versus LLVM**, and Rust and clang++
— which share a backend — land 3% apart. Timing against one compiler would have
published a language finding that does not exist.

**It reproduced on different hardware.** CI runs the same script on a GitHub
4-vCPU runner, and the pattern held: **321.36 µs** Rust, **589.57 µs** g++,
**322.28 µs** clang++ — g++ 1.83× slower again, Rust and clang++ 0.3% apart.
Two machines, two orders of magnitude apart in the absolute numbers, same
conclusion.

The same run also shows the effect is not *directional*: on that runner
`voxel.set_existing_state` was **6.5 ns** under g++ and **14.9 ns** under
clang++ — the backends swap places. Whichever compiler happened to be chosen
would have decided which language "won" that row.

So the C++ side is now built by every compiler on the machine, and the table has
three columns:

| kernel | Rust | g++ | clang++ | C++-internal spread | where Rust lands |
| --- | ---: | ---: | ---: | ---: | --- |
| `spatial.section_of_division` | 2.4 ns | 3.0 ns | 1.8 ns | 1.67× | inside |
| `voxel.get_paletted` | 6.9 ns | 8.6 ns | 6.4 ns | 1.34× | inside |
| `voxel.get_uniform` | 3.8 ns | 5.6 ns | 5.0 ns | 1.12× | **1.32× faster than both** |
| `voxel.set_existing_state` | 14.1 ns | 9.4 ns | 12.5 ns | 1.33× | **1.13× slower than both** |
| `worldgen.surface_height` | 36.7 ns | 73.5 ns | 40.1 ns | 1.83× | 1.09× faster than both |
| `worldgen.chunk_16` | 230.92 µs | 286.26 µs | 214.60 µs | 1.33× | inside |
| `worldgen.chunk_32` | 1.51 ms | 1.59 ms | 1.49 ms | 1.07× | inside |
| `physics.axis_sweep` | 40.1 ns | 33.6 ns | 36.0 ns | 1.07× | **1.11× slower than both** |
| `save.crc32_64kib` | 363.97 µs | 668.83 µs | 353.11 µs | 1.89× | inside |

Nine shared kernels. Rust falls **inside the band the two C++ builds span** in
five of them, beats both in two, and trails the nearer C++ build in two — by
13% and 11%. The widest C++-internal spread (1.89×) is larger than every
Rust-versus-C++ gap in the table.

**The honest summary: on these kernels, Rust is not measurably a handicap, and
the choice of optimizer backend costs more than the choice of language.** That
is a narrow claim about nine pieces of arithmetic. It is not a claim that a C++
engine would perform the same, and nothing here measures allocation, cache
behaviour at engine scale, threading, or build times.

## Finding 20 — a language boundary costs about what any un-inlined call costs

`NEXORA TECHNOLOGY BENCHMARK PLAN.md` lists FFI overhead as a metric, and it was
unmeasurable while the build had one language in it. Three rungs, all timed in
one binary under one methodology:

| rung | scalar (one `u64`) | bulk (4 KiB) |
| --- | ---: | ---: |
| inlined Rust | 2.4 ns | 5.01 µs |
| `#[inline(never)]` Rust — same language | 3.6 ns | 4.90 µs |
| `extern "C"` into C++ | 3.7 ns | 5.00 µs |
| the crossing alone, near-empty callee | **1.2 ns** | — |

Two things fall out, and they cross-check each other:

1. **Losing inlining costs ~1.2 ns** (3.6 − 2.4), and the near-empty crossing
   costs **1.2 ns** independently. The two agree.
2. **Crossing into C++ costs no more than an un-inlined Rust call** — 3.7 vs
   3.6 ns, inside the noise. At the C ABI, the boundary *is* a call.
3. **At 4 KiB per crossing all three rungs are identical.** One crossing
   amortised over a page of work is invisible.

This sharpens `NEXORA LANGUAGE AND FFI BOUNDARY.md`'s rule — *"hot simulation
loops must not cross FFI repeatedly"* — rather than contradicting it. The rule
is right, but not because a crossing is expensive: at the floor it is about
1.2 ns. It is right because **the floor is not what a real boundary costs.**
Nothing here marshals a struct, converts a string, copies a buffer to satisfy an
ownership rule, catches a panic before it unwinds into foreign frames, or
validates a pointer arriving from the other side. Those are what the rule is
protecting against, and this measurement deliberately excludes all of them. Read
1.2 ns as a **lower bound**, never as an estimate.

## Two methodology defects of mine, found in this increment

Both were caught by disbelieving a number, not by a test, which is the part of
this most likely to be wrong again.

1. **The C++ harness took `const std::function<void()>&`; the Rust one is
   generic over `F: FnMut()`.** So every C++ iteration paid a type-erased
   indirect call that Rust did not — several nanoseconds on kernels costing
   three, on one side of the comparison only. The header comment two lines above
   it read *"a comparison where the two sides measure differently is not a
   comparison"*, which is exactly what it was. Fixed by templating on the
   callable.

2. **LLVM collapsed four mixes into one `imul`.** `ffi.scalar_inlined` reported
   0.5 ns, below the latency of the multiply in its own dependency chain. The
   disassembly showed the loop counter advancing by 4 per single `imul`. With a
   barrier the same kernel costs 2.4 ns — 4× more, exactly the collapse factor.
   The same class of defect then turned up in `physics.axis_sweep`, which was
   being hoisted out of its loop entirely and reporting 2.4 ns for a 40 ns
   kernel.

The second correction has a tail worth recording: the first fix over-corrected,
putting the whole 48-byte `Aabb` through `black_box` and forcing a memory
round-trip C++ was not paying. That read as *"Rust is 2× slower at sweeping"*
(66.8 ns vs 31.8 ns) and would have been published as a language finding. The
barrier is now on the one scalar the result depends on, applied identically on
both sides, and the kernels land within 19% of each other.

## What the gate still needs

| metric | state |
| --- | --- |
| Same kernels on a second stack | **done** — this appendix |
| FFI / IPC boundary cost | **done** — finding 20, at the C ABI floor |
| Same *engine* on a second stack | missing — out of scope, ADR-0009 |
| RHI, window, camera, mesh generation | missing — needs a GPU |
| Build and iteration time | not captured by this harness |
| Debugging and tooling effort | qualitative; the plan scores it separately |

`DEBT-0008` is no longer blocked on *"no second stack exists"*. It stays open:
the plan's gate also wants the GPU stages, and an engine-scale comparison that
this deliberately is not.

**What can now be said that could not be said before:** the kernels the engine
spends its inner loops in are not slower in Rust than in C++ on this machine,
and the FFI boundary the polyglot option depends on costs about 1.2 ns per
crossing at its floor. **What still cannot be said:** which language the engine
should be written in. `NEXORA LANGUAGE AND FFI BOUNDARY.md` reserves that for
the completed benchmark, and it is not complete.

---

# Appendix E — what durability costs (2026-09-07, later run)

`DEBT-0025` was written with a condition attached: *"o custo de `sync` por edição
precisa ser medido antes de escolher a política."* This is that measurement, and
the policy it justifies.

## Finding 21 — an fsync is 303× an append, so durability has to be batched

| measurement | median | rel. σ | per edit |
| --- | ---: | ---: | ---: |
| `journal.append_unsynced` | **672.1 ns** | 4.6% | 672 ns |
| `journal.append_durable` | **203.42 µs** | 7.4% | **203 µs** |
| `journal.append_batched_sync` (64 records, one flush) | **293.11 µs** | 19.3% | **4.6 µs** |
| `journal.replay_10k_records` | **3.00 ms** | 0.9% | 300 ns |
| `journal.record_bytes` | **55 B** | — | — |

Framing and checksumming an edit is **672 ns** — cheap enough to do
unconditionally on every write. Making that edit durable costs **203 µs**, which
is **303× the append** and is almost entirely fixed cost: 64 records share one
flush for 293 µs total, so the second record through the sixty-fourth cost
**4.6 µs each** instead of 203.

That ratio decides the design. The headless slice makes **78 writes**. At
per-edit durability that is `78 × 203 µs ≈ 15.8 ms` — most of a 60 Hz frame,
spent on fsync. One flush per commit boundary costs **203 µs, about 1.2% of that
frame**, whatever the boundary holds.

**The policy: append on every write, flush at a commit boundary the caller
chooses.** `World::set_block` journals unconditionally; `World::sync_journal`
makes it durable; `World::unsynced_edits` reports exactly what a crash would
cost right now. The unsynced tail is the price, and ADR-0011's framing is what
makes paying it safe — whatever reached storage is still recoverable, because
each record checksums itself.

Replay is not a constraint on the policy: **300 ns per record** means a journal
of 10,000 edits costs 3 ms to read back, against **2.72 ms to generate a single
chunk** (finding 15). A journal would have to be enormous before load time
noticed it.

## The fsync number is the least portable number in this document

203 µs is what `sync_data` costs on **this container's storage**, which is
virtualised and shared. On a consumer NVMe drive it is typically faster; on
spinning rust or a network filesystem it can be one to two orders of magnitude
slower. The *ratio* to an in-memory append is what the policy rests on, and that
ratio only widens on slower storage — which strengthens the conclusion rather
than weakening it. But anyone re-deriving a checkpoint interval for real
hardware should re-measure rather than reuse 203 µs.

## What the slice now proves

The slice checkpoints the generated world **before** its first edit, opens a
journal bound to that snapshot, and then never mentions journalling again: the
78 writes that follow — 76 from the edit stage and 2 from the command stage —
are recorded by `World::set_block` itself. At the save boundary it flushes once.

Then it rebuilds the world from **checkpoint + journal** and re-checks every
probe: `journal 78 edits, 76 probes recovered`. Both numbers matter. 78 is every
write in the run, including the command handlers' — which never learned the
journal exists, because the journal lives on the world rather than wrapped
around it. 76 is every probe holding in a world that was reconstructed rather
than loaded.
