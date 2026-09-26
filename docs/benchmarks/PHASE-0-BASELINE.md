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

### 10b. The fix bought a third of it, and the diagnosis needed correcting

Finding 10 named the cause: "walks a `BTreeMap` of chunk columns and then a
palette-indexed section, once per cell per axis per body, with no caching". Two
things about that turned out to be wrong, and the second is the useful one.

**There are two ordered maps on the path, not one.** `World::get_block` descends
`World::chunks` for the column, and then `Chunk::get` descends `Chunk::sections`
for the section. Finding 10 counted the first and read the second as part of
"the palette-indexed section". They are separate descents and both are per-cell.
`WorldVoxels` now keeps the last resolved *section* — one entry, not a map —
which removes both from every repeat question, and a repeat is what almost every
question is: a body is roughly 0.6 × 1.8 × 0.6 against sections of 32³.

**And the descents were about a third of the lookup, not the bulk of it.**

| | before | after | |
| --- | ---: | ---: | ---: |
| `physics.raycast_40m` | 1.90 µs | **1.14 µs** | −40.0% |
| `physics.depenetration_check` | 123.6 ns | **78.5 ns** | −36.5% |
| `physics.character_step` | 743.2 ns | **545.4 ns** | −26.6% |
| `physics.box_sweep` | 289.1 ns | **219.0 ns** | −24.3% |
| `physics.thousand_bodies_step` | 209.8 µs | **174.3 µs** | −16.9% |
| — the lookup half of it | 102.7 µs | **67.2 µs** | **−34.6%** |
| `physics.thousand_bodies_step_flat` | 107.1 µs | 107.1 µs | — |

The last row is the control and it is the reason the rest can be attributed at
all: the flat fixture does not go through `WorldVoxels`, and it did not move. A
machine that had simply got faster would have moved it too.

The ordering across the other rows is the second check. `raycast_40m` is a walk
of forty metres of cells and almost nothing else, and it gains the most.
`thousand_bodies_step` is half solver, and it gains the least. Nothing else
would produce that gradient in that order.

Two changes, measured separately: keeping the section is most of it (the lookup
falls to **77.1 µs** on its own), and `ChunkShape::split_of` — which returns the
section address and the section-local offset from one pass, because
`section_of` takes the quotient and `local_of` the remainder of the same three
divisions — takes it the rest of the way to 67.2 µs.

Medians over five to seven runs. `physics.thousand_bodies_step` carries the
highest variance in the suite (up to 30% relative σ on a loaded run), which is
why the argument rests on `raycast_40m`, `depenetration_check` and
`box_sweep`, all of which sit under 3%.

**`DEBT-0011` stays open at a smaller number.** The lookup was 49% of a physics
step and is now **38.6%** — still the larger half of what is left to win, and
still not the solver. What remains is the palette read and the address
arithmetic, neither of which a cache addresses; `voxel.get_paletted` at 6.9 ns
and `spatial.index_of` at 3.7 ns are what the next attempt would have to move.

`DEBT-0012` moved without being worked on. Its trigger read "junto com
DEBT-0011", and `physics.depenetration_check` fell 36.5% because the
depenetration scan reads the world through the same view. The debt is about the
scan happening at all, so it stays open — but it now costs a third less while
it waits.

### 10c. The depenetration scan stops running, and the pair that measured 10b stops working

`DEBT-0012` said the check for having started inside terrain costs ~42% of a
sweep and runs for every body every step, down a path that almost never fires.
It now runs only when it can tell the reader something new: when the body has
moved into different cells, or the source says it changed.

**The figure that does not depend on this machine.** A settled body makes two
world reads per step; one of them is that check. Over ten steps, against a
source that names a revision and one that does not:

| ten settled steps | cells asked |
| --- | ---: |
| source names a revision | **10** |
| source will not say | **20** |

Exactly half, asserted as an equality in `a_resting_body_stops_being_asked_
whether_it_started_inside_terrain`. Counting is the only honest way to state
this: a timing test proves it on one machine and nothing on another.

**And the timing, measured in interleaved pairs on one sitting:**

| | before | after | |
| --- | ---: | ---: | ---: |
| `physics.thousand_bodies_step` | 137.3 µs | **84.6 µs** | −38.4% |
| `physics.character_step` | 386.7 ns | **343.6 ns** | −11.1% |
| `physics.thousand_bodies_step_flat` | 82.8 µs | 82.2 µs | −0.7% |
| `physics.depenetration_check` | 56.1 ns | 55.2 ns | −1.6% |

Two controls this time, and both had to hold still. The flat fixture names no
revision, so it never skips — it did not move. `physics.depenetration_check`
calls `depenetrate` directly and never sees the skip at all — it did not move
either, which says the primitive is not faster; the calls to it stopped
happening.

**These are not on 10b's scale.** The same code that measured 174.3 µs there
measures 137.3 µs as the "before" here, on a machine in a different state hours
later. That is why the before column was re-measured by stashing rather than
quoted: a 21% drift would have been reported as a 21% improvement.

**A control caught a regression in the first attempt.** The cell span was
computed before checking whether the source names a revision, so a source that
had opted out paid for machinery it could not use — `thousand_bodies_step_flat`
went *up* 17%. The span is now computed only once both halves of the proof
exist.

**The cost of the fix is that 10b's measurement no longer works.** The
terrain/flat pair isolated the voxel lookup because the two rows differed in
exactly one thing: where terrain came from. They now differ in two — the
terrain row skips the depenetration scan and the flat row does not, because
`FlatGround` names no revision. So "38.6% of a physics step is the lookup"
cannot be re-derived from this pair, and any number taken from it now would be
measuring both changes at once. Restoring it needs a flat fixture that names a
constant revision, which is deliberately **not** done here: `FlatGround` is
constructed inline and two of them with different floors would report the same
constant, which is the one way a revision can lie. Recorded as **DEBT-0037**.

### 10d. The ruler is rebuilt out of two parts, and one of them never drifts (2026-09-14)

**Measured on a different, noisier box than 10a-10c.** Every number in this
section was taken here, and none of it is comparable to the tables above: the
same unchanged code that measured `thousand_bodies_step` at 84.6 µs in 10c
measures ~140 µs here. That is the machine, not a regression, which is exactly
why nothing below is quoted across.

**DEBT-0037 asked for a flat fixture that names a revision. That was the easy
half, and on its own it does not work.** `StillFloor` now gives the flat row a
revision, so the pair differs in one thing again, and the A/B says so:

| flat fixture | `thousand_bodies_step_flat` |
| --- | ---: |
| `FlatGround`, no revision | 145.34 µs |
| `StillFloor`, revision | 142.79 µs |

1.8%, against a run-to-run spread of 10–23% on this box. **The subtraction is
not a measurement here, it is noise.** And it was always going to be: after
DEBT-0012 a settled body makes one world read per step instead of two, so what
the pair has to resolve is now a fifth of a step hiding inside two numbers of
140 µs each.

**So the pair was replaced by a product rather than a difference.** Two
measurements, each resolvable on its own:

| | value | spread |
| --- | ---: | ---: |
| `physics.world_reads_per_step` | **1 000** | — |
| `physics.voxel_lookup` | **25.9 ns** | 5.8% |

`1 000 × 25.9 ns = 25.9 µs` of a `141.16 µs` step: **18%**. The first row is a
count — one cell question per settled body per step, the same integer on every
machine, and independent confirmation that DEBT-0012's skip is live. The second
is a 26 ns microbenchmark rather than a 140 µs one, which is the whole point:
it can be re-measured cheaply and it is the only half that needs a quiet box.

**How quiet is not a detail.** A second run of the same binary read
`physics.voxel_lookup` at **38.2 ns with a 26% spread**, putting the share at
27% instead of 18%. The count did not move by one. Reported as a range, 18–27%,
because choosing the run that reads better is how a measurement becomes an
advertisement.

**And 38.6% is not what this contradicts.** That figure was taken while the
depenetration scan still ran, when a settled body made two reads a step instead
of one. Halving the reads was supposed to roughly halve the share, and it did.

### 10e. The paletted read was one division, not two, and it was worth 13%

**`voxel.get_paletted`: 13.1 ns → 11.4 ns.** What is left of DEBT-0011 after
10b is the palette read and the address arithmetic, and the palette read had a
divisor no compiler could see: cells pack `64 / bits` to a word, and
`64 / 12` is five. Both the word and the offset inside it came from dividing by
that.

The twelve possible widths are known at compile time, so each one's layout is
now a table entry: the quotient is a multiply and a shift against
`ceil(2^32 / per_word)`, and the remainder falls out of the quotient. The
identity is exact for every index a section can hold — `MAX_SECTION_EXTENT`
cubed is 2^24, and the error bound `(N + d - 1) · e < 2^32` clears it by two
orders of magnitude — and a test walks the last index of every word near the
top of the range, which is where an approximate reciprocal breaks first.

| | before | after | |
| --- | ---: | ---: | ---: |
| `voxel.get_paletted` | 13.1 ns | **11.4 ns** | −13% |
| `physics.voxel_lookup` | 28.3 ns | **25.9 ns** | −8.5% |
| `voxel.get_uniform` | 5.2 ns | 5.3 ns | — |
| `spatial.index_of` | 6.3 ns | 6.3 ns | — |

**The last two rows are the controls and they held still.** Uniform storage
never calls the packed read, and `index_of` was not touched; a machine that had
simply got faster would have moved both. `voxel.get_paletted` read 11.4 ns on
two separate runs, against a 3–4% spread.

**13% is less than a division costs, and that is the finding.** The guess
behind this change was that two integer divisions were most of a 13 ns read.
They are worth 1.7 ns. The likely reason is that there were never two: x86-64's
`div` yields the quotient and the remainder from one instruction, so the
compiler had already fused the pair, and an out-of-order core overlaps most of
what is left with the surrounding work. This is reasoning, not measurement —
what was measured is 1.7 ns.

**DEBT-0011 stays open at the smaller number.** The address arithmetic —
`split_of` at the top of every lookup, three divisions by runtime section
extents — is now the largest single piece left, and it has the same shape: a
divisor fixed for a world's lifetime that the compiler cannot see. Unlike the
palette width it is not drawn from twelve values, so a table does not close it.

### 10f. A proof needs to say who made it, and "controls held still" is not enough (2026-09-14)

**The skip from 10c believed a number with no owner.** A body remembers the
cells it was proven clear of solids in, keyed on the source's revision. A
revision is a `u64`: `World` counts its edits from zero, any other source that
opts in starts somewhere too, and two of them agree on the number while
describing different terrain.

**The obvious identity does not work, and that was checked rather than
assumed.** A source built on the stack for one step and a *different* source
built the same way for the next occupy the same address —
`two_sources_at_one_address_do_not_share_a_proof` guarantees it by reusing one
variable, and both grounds report revision 1. Address plus revision would have
matched, and the body would have been left standing inside a block.

What replaced it is a **session**: `PhysicsWorld::against(&source)` returns a
`Stepper` that borrows the source for as long as the proof can be consulted, so
every substep it runs is answered by the same live object. The borrow checker
says so; nothing is compared. Opening a session mints a number never used
before, and the proof records it.

| ten settled steps | cells asked |
| --- | ---: |
| inside one session | **10** |
| a session per step | **20** |

`step_once` and `advance` open a session for the single call, so the safe
default costs nothing to know: they are correct and they never skip. Holding the
optimisation is what now takes holding the session, which the headless slice and
the benchmark both do. The slice's save is **byte-identical** to the one from
before the change.

**And the test can fail.** With `proof.session == session` removed it reports
"the check was skipped on the strength of a proof the old source made".

### 10g. The palette read moved 30% in a commit that did not touch it

**This corrects 10e's precision, not its direction.** 10e reported
`voxel.get_paletted` at 13.1 ns → 11.4 ns, −13%, with `voxel.get_uniform` and
`spatial.index_of` held still as controls. One commit later, having changed
`engine/physics` and `engine/benchmark` and **not one line of `engine/world`**
(checked with `git diff --name-only`), the same source reads **7.8 ns and 8.2 ns
across two runs** — and the same two controls are still still, at 6.0 and 5.1 ns.

The release profile is `lto = "thin"` with `codegen-units = 1`. Changing any
crate in the workspace relinks the whole binary and moves `Section::get`; at
under 10 ns, alignment and inlining decisions are worth tens of percent.

**So the method has a hole in it.** "Move one thing and check the controls did
not move" is necessary and not sufficient when the change itself relinks the
binary — the controls can hold still while an untouched kernel moves 30%. What
10e's measurement supports is the direction and the order of magnitude: the
division came out, and the read got somewhere between ~13% and ~40% faster on
this box. Not a single figure. Recorded as **DEBT-0039**, along with the fix:
build the same source twice with a neutral change in between and publish the
spread between builds, not only within one.

### 10h. The save stops writing packed words raw (2026-09-14)

**744,954 → 116,904 bytes for the slice's world: 6.4×.** *(Corrected on
2026-09-15. The figure first published here paired 744,954, which is the slice
at its default seed, with 117,908, which is the determinism smoke's slice at
seed 987654321. Both numbers were right and the pair was not. At the default
seed the before and after are 744,954 and 116,904; at seed 987654321 the after
is 117,908. The ratio survives either way, which is why it went unnoticed.)*
`DEBT-0003` had the
container writing every section exactly as handed to it, which for the chunk
section means the palette-packed words with no further coding.

The codec was already written and already measured — `DEBT-0034` took it to
1.0233× of zlib -9 — and it was sitting in `tools/texture-forge`, where only the
PNG encoder could reach it. Promoting it to `nexora_foundation::deflate` added
**no dependency edge at all**: persistence and the texture forge both already
depended on foundation. All twelve generated PNGs are byte-identical across the
move, checked by hash rather than by argument.

| | bytes |
| --- | ---: |
| slice save, format 1 (raw sections) | 744,954 |
| slice save, format 2 (coded sections) | **116,904** |

**Compression is kept only when it wins.** High-entropy bytes deflate to
slightly more than they started with; writing that would make the format worse
at exactly the inputs it is already worst at. `Coding::Stored` is not a failure
path — there is no failure path — it is the answer when compressing did not pay.

**Format 1 stays readable.** `MIN_SUPPORTED_SAVE_FORMAT` was deliberately not
raised: one branch in the decoder, against throwing away every world written
before the change. Verified against a real format-1 file written by the previous
build, not only against a hand-built frame.

**A failing test found something the change had missed.** The first damage test
flipped "a byte in the middle" and hit the coding byte rather than the deflate
stream — caught, but by the coding range check rather than by the checksum. That
exposed the new frame fields sitting *outside* the section checksum, whose own
comment already explains why the name is inside it: a flipped bit in an
unchecksummed field does not look like damage, it looks like a different and
wrong instruction. A coding byte that flips turns a deflate stream into a raw
payload. Coding and length are inside `frame_crc` now, with a test for each.

### 10i. A region store pays for the regions that moved (2026-09-15)

**A save after one block changed: 12.8–13.2 ms → 2.9 ms, and 40.8 KiB →
4.7 KiB.** `DEBT-0002` had every save rewriting the whole world however little
of it moved — encode every column, deflate every byte, write the file, then read
it back and decode it to verify (ADR-0004's guarantee, priced as `DEBT-0006`).

`nexora_world::region::RegionStore` writes the same world as a directory, one
`SaveContainer` per region. Decisions in
[ADR-0014](../adr/ADR-0014-a-region-file-is-authoritative-for-its-region.md).

| benchmark world, 3×3 columns | one container | region store |
| --- | ---: | ---: |
| save after one block changed | 12.76 / 13.16 ms | **2.91 / 2.87 ms** |
| bytes written for that save | 40.8 KiB | **4.7 KiB** |
| save of the whole world | 12.76 / 13.16 ms | 14.95 / 14.91 ms |
| bytes for the whole world | 40.8 KiB | 41.7 KiB |

Two runs of each, and **`save.write_atomic_disk` is the control**: 13.19 ms
before the change, 12.76 and 13.16 ms after. It did not move, which is what
makes the rest attributable — nothing here touches the container path.

**The figure that does not depend on the machine is a count.**
`save.regions_written_per_edit` is **1** of 4, on any box and in any build. It
is also the independent confirmation that per-section dirty tracking, which the
chunk lifecycle has recorded since it was built and no save had ever read, is
now read.

**Two columns per region, not the default 32.** At the engine default a 3×3
world is one region and the measurement could not tell skipping from writing.
The extent is a parameter of the store, and the mechanism is what is under test.

**The price, measured rather than omitted: a full write costs 13–17% more**, and
the total grows 2.2%. Five files instead of one is five sets of framing, five
`fsync`s, five read-back verifications, and five deflate streams that cannot
share a dictionary; the palette is also written once per region, deliberately,
so that a region file can be read with nothing else present. A full write is the
case this arrangement is not for.

**Splitting the section inside the same container would have bought nothing.**
The container encodes as a unit: every section deflates on every `encode`, and
the whole file is written and verified. The cost follows the file, so the
regions had to be files — and the alternative would have been a structure whose
benefit arrives in some later change.

**What this does not do yet.** Nothing in the engine saves through it. The slice
writes the single container and runs the store beside it as verification, which
is where the nine-region line in its report comes from. `DEBT-0002` is reduced,
not closed, and closing it waits on `DEBT-0020` — the evicted column going to
its region file instead of to a `BTreeMap` in memory.

### 10j. Retention can cost no memory, and the price is the file (2026-09-15)

**20.0 KiB → 0 B per evicted edited column, at 184.7 ns → 3.08 ms a round
trip.** `DEBT-0020` measured retention at 20.0 KiB per column, growing with how
much the world was edited and dying with the process. `RetainedChunks` can now
be backed by a region store ([ADR-0014](../adr/ADR-0014-a-region-file-is-authoritative-for-its-region.md)).

| | median |
| --- | ---: |
| `streaming.retained_bytes_per_chunk` (**control**, unchanged) | 20.0 KiB |
| `streaming.retained_bytes_after_flush` | **0 B** |
| `streaming.chunk_retained_cycle` (held in memory) | 184.7 ns |
| `streaming.chunk_flushed_cycle` (through a region file) | **3.08 ms** |
| `streaming.region_writes_per_eight_columns` | **4** |

Every pre-existing streaming row held still across the change —
`idle_tick_r3` 3.64 µs, `idle_tick_r12` 121.29 µs, `walk_one_chunk` 7.39 µs,
`chunk_generate_cycle` 1.30 ms, `retained_bytes_per_chunk` 20.0 KiB — which is
what makes the two new rows attributable.

**16,700× is the honest number, and it is not the one that decides this.** The
last row is: the cost is the **file**, not the column. Eight columns falling in
four regions are four writes, not eight — a count, the same on every machine.
`chunk_flushed_cycle` is the pathological shape, one column flushed and
immediately read back; a per-tick flush amortises over everything that tick
evicted.

**Eviction does not write; a flush does.** The backend's `persist` still hands
the chunk to memory, because eviction runs inside the streaming budget and a
file write does not belong there. `flush_to_store` runs at the end of the tick.
The bound on memory becomes *what was evicted since the last flush*.

Measured in the slice, which is the only place the real streaming loop runs:
peak retention **25 → 15** columns, 25 columns spilled to 12 region files and
25 read back, and the **save byte-identical at 116,904 bytes** either way. The
determinism smoke is still identical at one and eight threads, region stores
included.

**The default does not change, and that is the finding.** `RetainedChunks::new`
is untouched. At Phase 0 scale 25 edited columns are 500 KiB and 3 ms a column
is not worth paying; `DEBT-0020`'s own trigger is ~1,000 retained columns or the
first dedicated server, and neither has happened. The mechanism is built and
measured so that the switch is a decision rather than a project.

### 10k. Measuring a debt about silence found one about cost (2026-09-16)

**`DEBT-0004` is about a change feed going quietly incomplete. Measuring it
first turned up that enforcing the cap cost 49.95 µs per voxel write.**

The feed kept the newest 4,096 changes and dropped the oldest with
`Vec::remove(0)` — which moves the other 4,095 entries, about 190 KB, on every
write past the cap. Paid by exactly the column being edited hardest, because
that is the one that reaches the cap.

| | before | after | |
| --- | ---: | ---: | ---: |
| `chunk.change_feed_at_cap` | 49.95 µs | **42.0 ns** | −99.9% |
| `chunk.change_feed_append` | 40.7 ns | 40.9 ns | — |
| `voxel.get_paletted` | 6.6 ns | 6.5 ns | — |
| `voxel.get_uniform` | 3.2 ns | 3.2 ns | — |
| `voxel.set_existing_state` | 13.5 ns | 14.1 ns | — |
| `voxel.compact_section` | 164.34 µs | 168.84 µs | — |

**1,190× is not the finding. The finding is that the two top rows are now the
same number.** A `VecDeque` discards from the front in constant time, so a write
that also discards costs what a write costs — 42.0 against 40.9 ns. The cap
stopped being something the hot path pays for, rather than becoming cheaper.

The four rows below are the controls, and they held. The second row is the
sharper one: the **append** path did not change, which is what makes the whole
difference attributable to the discard and nothing else.

**This one is proved by the benchmark and not by a test, and that is not a gap
in the testing.** Swapping the container changes no behaviour: the correctness
tests — the cap holds at 4,096, the retained window is the most recent one, the
drop count is right — pass identically before and after. A test that could tell
the two apart would be a test of `VecDeque`.

**The silence the entry is actually about closed separately, and by the type.**
`Chunk::take_journal` returned the changes and *reset* the drop count, so a
consumer that never read `dropped_journal_entries` destroyed the only record
that entries had gone missing. It now returns a `#[must_use] ChangeFeed` holding
the changes and the gap together. Compiling that change pointed at the two call
sites that were discarding the return in silence; both meant "this chunk was
just built, there is nothing to consume" and now say so with `clear_journal`.

`World::change_feed_gaps` sums the gaps across resident chunks and the slice
fails if any is non-zero. At 76 edits across 25 columns against a 4,096 cap that
cannot trip today; what changed is that the signal is read at all.

**A gap here is not a lost block.** `World::set_block` writes the save journal
(ADR-0011) before it touches the chunk, so durability never depended on this
feed. A gap costs history and replication.

**And the feed still has no consumer.** `take_journal` is called nowhere in the
engine. The entry's own remediation — drain per tick into the History System —
waits on Phase 11, and building a consumer now would be working ahead of the
evidence. What this change buys is that the cap is free and the gap is
un-ignorable by the time one exists.

### 10l. An index made a query 1,253× faster and the tick 4× slower (2026-09-16)

**`DEBT-0010` said entity queries were a linear scan, and its remediation said
to measure the scan again before keeping anything. Measured first, at three
population sizes, the scan behaved almost exactly as the entry extrapolated —
and one row did not.**

| query, scan only | 1,000 | 10,000 | 100,000 | per entity |
| --- | ---: | ---: | ---: | ---: |
| `within_radius(16)` | 10.9 µs | 102.7 µs | 979.7 µs | 9.8–10.9 ns |
| `in_chunk` | 17.3 µs | 168.0 µs | 1,655.4 µs | 16.6–17.3 ns |
| `by_type` | 15.7 µs | 182.7 µs | **4,069.0 µs** | 15.7 → **40.7** ns |

The entry's "~19 ns per entity examined" holds for the first two rows across two
orders of magnitude. **The third row does not**, and neither does ADR-0006's
matching note that a type query "will not hold at 100,000": it costs 2.6× more
per entity at 100,000 than at 1,000, because it matches everything and the cost
is dominated by building a 100,000-element result rather than by examining
anything. That row is not a scan that wants an index. It is a query whose answer
is the population, and no index makes an answer smaller.

**What the radius row says instead is the whole case for indexing**: 100,000
entities examined to return **six**.

#### After: a loose grid, 16-block cells, maintained by the store

| | scan | index | |
| --- | ---: | ---: | ---: |
| `within_radius(16)`, 1,000 spread | 10.9 µs | **430 ns** | 25× |
| `within_radius(16)`, 10,000 spread | 102.7 µs | **654 ns** | 157× |
| `within_radius(16)`, 100,000 spread | 979.7 µs | **782 ns** | **1,253×** |
| `in_chunk`, 100,000 spread | 1,655.4 µs | **625 ns** | **2,647×** |
| `by_type`, 100,000 | 4,069.0 µs | 4,196.5 µs | — (still a scan) |

The last row is the control, and it is the one that had to not move: the
non-spatial queries were left alone deliberately, and they were.

**The number that is not from this machine is `entity.query_radius_candidates_100k`
= 41.** Forty-one entities examined, of 100,000, to answer a 16-block radius
query. It is a count, so it is the same in any build on any box, and it is what
the three orders of magnitude above are made of. The time went from growing with
the population to being flat in it — 430, 654, 782 ns as the population went
×100.

#### What it cost, which is not nothing

| | before | after | |
| --- | ---: | ---: | ---: |
| `entity.step_1000` | 2.07–2.11 µs | **7.2–8.7 µs** | ~4× |
| `entity.step_1000_crossing_cells` | — | 170.9 µs | new row |
| `entity.spawn` | 150–160 ns | 168–222 ns | noisy either way |
| `entity.spawn_despawn_cycle` | 119–175 ns | 176–186 ns | |
| `entity.query_radius_1000` (dense fixture) | 15.1–15.5 µs | 13.5–14.9 µs | ~1.1× |

Three readings before, five after; the spread is between runs, not within them
(each run's own relative stddev is 2–6%).

**Half of the step regression was `f64::floor`.** Placing a position in a cell is
`floor(x) as i64 >> 4` twice, and the x86-64 baseline has no `roundsd` — that is
SSE4.1, which the default target does not assume — so `floor` is a call into
libm. Measured over 1,000 placements: **4.94 ns** each via `floor`, **1.54 ns**
via a truncating cast and a comparison that corrects the one case truncation
gets wrong. `step_1000` went 10.57 → 7.27 µs on that change alone. The remaining
~6.5 ns per entity per tick is the placement, the comparison against the stored
cell, and the second data stream it reads.

Two smaller things that are worth writing down because they were guesses that
measurement settled. Hoisting the repeated read of the cell column changed
`step_1000` by 0.04 µs — the compiler was already doing it; the change was kept
for legibility and is not claimed as a saving. And the naive `truncated - 1`
fixup wraps `i64::MIN` to `i64::MAX`, filing an entity at the opposite end of
the world; `saturating_sub` costs nothing measurable and a test covers it.

#### The awkward part, stated plainly

**At the benchmark plan's own 1,000-entity stage, this index is a net loss of
about 6.5 µs per tick.** The plan's fixture packs 1,000 entities into
64 × 16 × 37 blocks — about twelve cells — so a 16-block radius query there asks
about most of the population and there is nothing to exclude. `query_radius_1000`
improved 1.1×, inside the run-to-run spread, while `step_1000` got 6.5 µs worse.

That fixture was deliberately not changed. Rewriting it would have made every
1,000-entity row in this document incomparable with every run before today, for
the sake of making one change look better. The new rows ask the same questions
of a population spread the way a world spreads one, and both sets are published.

6.5 µs is 0.013% of a 50 ms tick, and it is bounded: the worst case, where every
one of 1,000 entities changes cell on every tick, is 170.9 µs, and no entity
moving at a plausible speed does that. The decision to keep the index always on
rather than behind a switch is ADR-0015, and it turns on staleness rather than
on these numbers: an index the store maintains only sometimes is an index a
query cannot trust.

#### One thing the guard does that is not about speed

`Query` declines the index when the rectangle covers more cells than the store
holds entities. That reads like a performance heuristic and is not one. A finite
radius of `1e300` spans about 1.3 × 10³⁶ cells, and walking them does not
finish. Removing the guard does not make that query slow; it makes it hang —
which is also why the test for it asserts the *decision* rather than the answer.
A test that hangs reports nothing.

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

---

# Appendix F — meshing (2026-09-08, later run)

The stage this document listed as unmeasurable, and the reason that was wrong.

## The correction first

Earlier appendices carried `mesh generation` on the **not measured** list with
the reason *"no renderer to consume a mesh"*. That put it beside the window and
the RHI, as though it were blocked on hardware. It was not.

**Displaying** a mesh needs a renderer. **Building** one does not, and neither
does checking it. RENDER-11's pipeline is `block changed → chunk dirty →
neighbour check → remesh → GPU update`, and only the last arrow needs hardware.
A mesh is a data structure with properties a test can check: a solid region must
emit only its shell, merging must preserve surface area exactly, a border face
must be culled by the neighbouring chunk, and the same voxels must always
produce the same geometry.

Mesh generation is now measured. ADR-0012 has the design.

## Finding 22 — the mesher is not the slow part; the world lookup is

The same 16³ region across the ground/air boundary, meshed two ways, differing
in exactly one thing — where the voxels come from:

| measurement | median | rel. σ |
| --- | ---: | ---: |
| `mesh.region_16` (read from the world) | **3.30 ms** | 3.5% |
| `mesh.region_16_from_snapshot` (pre-read dense array) | **295.45 µs** | 1.7% |
| `mesh.cull_only_16` (culling, no merging) | **3.20 ms** | 2.5% |
| `mesh.region_32` (32³, the engine's section size) | **26.12 ms** | 3.0% |

**11.2×.** So **91% of meshing is the voxel lookup** — not culling, not merging.
`mesh.cull_only_16` confirms it from the other direction: strip merging out
entirely and you save about 3%.

This is [finding 10](#) again — 49% of a physics step was the voxel lookup, 38.6%
of one after finding 10b — and here it is far more extreme. Two independent measurements now point at the same
place, which is worth more than either alone.

**The consequence is architectural, not merely an optimisation note.** A region
snapshotted into a dense array can be meshed on a worker thread *without
touching the world at all*. So the fix for meshing speed and the mechanism that
makes RENDER-12's async meshing **safe** are the same change. `DEBT-0029` and
`DEBT-0027` are one piece of work wearing two labels.

### 22b. What the snapshot actually costs, once it is real (2026-09-13)

The measurement above is a **diagnostic**: it excludes the cost of *building*
the snapshot, which is itself a pass of world reads. It answers "how much of
meshing is lookup", not "is the snapshot worth it". Those are different
questions and the second one decides the work, so `DEBT-0029` added the
measurement that answers it:

| measurement | median |
| --- | ---: |
| `mesh.region_16` (read from the world) | **3.32 ms** |
| `mesh.region_16_with_snapshot` (snapshot **built**, then meshed) | **1.22 ms** |
| `mesh.region_16_from_snapshot` (snapshot already held) | **296 µs** |

Two different numbers, and quoting one for the other would be wrong:

* **First mesh of a region: 2.7×.** Building the snapshot costs 0.92 ms — 76%
  of the with-snapshot time — because it is the pass that pays the world reads.
* **Re-mesh while the snapshot is held: 11.2×.** That is the headline figure
  from finding 22, and it applies only when the voxels have not changed.

So the snapshot pays for itself immediately, and pays far more when a region is
meshed again — a moved camera, a changed LOD tier, a second pass — without the
voxels having changed underneath. It is worth being precise about which is
which: **11.2× is the re-mesh figure, not the speedup a single mesh gets.**

`mesh.region_32` at **26.12 ms** is the other half of why: that is more than a
60 Hz frame, for one section, on the tick thread.

## What culling and merging actually buy

Measured on generated terrain at the ground/air boundary, 16³:

| | faces |
| --- | ---: |
| every cell, all six sides (a naive mesher) | **24,576** |
| after face culling | **2,471** |
| after greedy merging | **807 rectangles** |
| vertices a renderer would upload | **3,228** |

Recorded as quantities, not as a ratio. How much merging wins depends entirely
on how smooth the ground is — a checkerboard merges nothing at all — and
quoting one terrain's ratio as a property of the mesher would be exactly the
kind of unearned claim the earlier appendices were corrected for.

## What the slice's remaining gaps are

| stage | state |
| --- | --- |
| **Mesh generation** | **done** — this appendix |
| 16³ voxel chunk, entities, physics, streaming, jobs, save/load | done |
| Same kernels on a second stack, FFI cost | done — Appendix D |
| Durability and crash recovery | done — Appendix E |
| RHI, window, camera, input, frame time | missing — needs a GPU |
| Same *engine* on a second stack | missing — out of scope, ADR-0009 |
| Build and iteration time | not captured by this harness |

Of the plan's vertical-slice stages, what is left is the render path and the
things that only exist inside one. **The list of "blocked on hardware" items is
one shorter than it was, and it was one too long.**

---

# Appendix G — the job system (2026-09-19)

## Finding 23 — a submission is 2% enqueue and 98% waking a worker

**`DEBT-0009` measured 8.84 µs to submit one trivial job and blamed *"a futex
wake per `notify_one`, plus contention of every worker on one mutex"*. That was
a hypothesis. Measured, it is right about the wake and the mutex is the
amplifier, not the cause.**

The submission was measured in three states that differ only in what the pool is
doing, so the wake is isolated from the enqueue:

| `submit()` with… | median |
| --- | ---: |
| every worker **parked** on the queue — a wake-up per submission | **12.20 µs** |
| every worker **busy** inside a job — nobody to wake | **288 ns** |
| workers **draining** as fast as the producer fills (what `jobs.submit_only` measures) | **12.18 µs** |

**42× between the first two rows, and the only difference is whether there was a
thread to wake.** The third row is the benchmark's own shape and matches the
first, which is what says the published 8.84 µs was never mostly enqueue.

The pieces confirm it from the other side. Summed standalone, everything a
submission actually *does* comes to about 142 ns:

| piece | median |
| --- | ---: |
| `Box::new(closure)` | 0.6 ns |
| `Arc::new` (the cancellation token) | 20.4 ns |
| `HashMap::insert` (the token table) | 79.4 ns |
| `BinaryHeap::push` | 28.5 ns |
| uncontended mutex lock + unlock | 12.7 ns |
| `Condvar::notify_one`, nobody parked | 125.2 ns |
| `Condvar::notify_one`, one thread parked | **645.6 ns** |

A bare wake of a parked thread is 646 ns; in the pool it costs ~11.9 µs. The
difference is the handoff: the woken worker immediately reaches for the same
mutex the producer needs for its next submission, so the producer ends up
serialized behind a worker's whole wake → lock → pop → run → lock → notify →
park cycle. That is the mutex's role — it turns one wake into a round trip.

### The fix the measurement points at

If the wake is the cost, the fix is to wake once per *wave* instead of once per
job. `JobSystem::submit_all` takes one lock, enqueues the batch, releases it, and
wakes as many workers as there is work for, capped at the pool.

| | one at a time | `submit_all` | |
| --- | ---: | ---: | ---: |
| `jobs.batch_1000_barrier*` — 1,000 jobs, submitted and run | 11.40 ms | **1.88 ms** | **6.1×** |
| producer's cost per job (`submit_only` vs `submit_all_1000`) | 10.95 µs | **103 ns** | **106×** |
| **`jobs.wakeups_per_1000_*`** | **1 000** | **4** | a count |

**The last row is the one that is not from this machine.** A wave of 1,000 jobs
issues 1,000 condvar wake-ups submitted one at a time and `min(1000, workers)`
— four here — through `submit_all`. That integer is the same in any build on any
box, and it is what the two rows above it are made of.

**The two rows being compared come from the same run of the same binary**, which
is deliberate: `jobs.batch_1000_barrier` reads 9.62, 10.11 and 11.40 ms across
three runs on this box, so a before/after quoted across runs could have reported
anything from 5× to 6×. Within one run the pool, the machine and the build are
held still and only the submission path differs.

**The controls are the unchanged paths, and they held.** `jobs.submit_only`
reads 10.92 and 10.87 µs before the change and 10.95 µs after;
`jobs.submit_wait_roundtrip` 36.26 and 34.44 µs before, 36.31 µs after. `submit`
was refactored to share its enqueue with the batch, so those two rows not moving
is what says the refactor cost nothing.

103 ns per job also lands where the decomposition said it should: the pieces sum
to ~142 ns and the busy-worker case measured 288 ns, and a batch is cheaper than
either because it takes the lock once for the whole wave rather than once per
job.

### What this does not change

**The register's architectural note still stands, and this does not soften it.**
*"Um job por entidade é anti-padrão, agora com número."* A batched submission is
103 ns against ~3 ns to simulate one entity inline — still ~35×. What the batch
fixes is submitting a *wave of real work*: the slice's 25 chunk-generation jobs
now go over as one batch, which is the pattern the API exists for. It does not
make a job a cheap unit of work, and nothing here argues for finer jobs.

`jobs.submit_only` is also deliberately left in place measuring the old path.
It is the honest number for a producer that submits one job and has nothing to
batch it with, and that case did not get faster.

### A suspicion the measurement killed

Reading the code first suggested the `results` map — which is inserted into on
every completion and **never removed by anything** — would be slowing the
producer down as it grew. It does not:

| finished jobs still held in the map | `submit()` |
| ---: | ---: |
| 0 | 12.15 µs |
| 100,000 | 10.99 µs |
| 500,000 | 12.28 µs |

Flat, inside the spread. The leak is real and is now recorded as `DEBT-0040`,
but it is a memory defect and not a throughput one, and saying otherwise would
have been a plausible story with no number behind it.

---

# Appendix H — the noise floor (2026-09-20)

## Finding 24 — the smallest numbers here are the steadiest, and the entry that said otherwise was guessing

**`DEBT-0039` recorded that `voxel.get_paletted` read 11.4 ns at one commit and
7.8–8.2 ns at the next, which touched no line of `engine/world`, and concluded:
*"no number below ~20 ns published here should be read as precise better than a
range."* The remediation asked for *n* builds of the same source, published with
the spread between them. That is `scripts/build-spread.sh`, and the answer it
gives is not the one the entry expected.**

The script builds the workspace *n* times from **identical source**, separating
each build with a neutral comment appended to `engine/benchmark/src/lib.rs` — a
crate holding none of the measured kernels but linked into the same binary,
which is the exact shape of the change that caused the entry. Whatever a row
moves across those builds, it moved for no reason at all.

### What moves, over three builds of one source

| the six worst rows in the suite | lowest | highest | spread |
| --- | ---: | ---: | ---: |
| `journal.append_durable` | 108.74 µs | 148.16 µs | **36.3%** |
| `physics.thousand_bodies_step_flat` | 86.03 µs | 111.77 µs | **29.9%** |
| `save.region_write_one_dirty` | 2.27 ms | 2.81 ms | **23.8%** |
| `jobs.batch_1000_barrier_submit_all` | 1.99 ms | 2.46 ms | **23.6%** |
| `journal.append_batched_sync` | 190.49 µs | 231.12 µs | **21.3%** |
| `jobs.submit_wait_roundtrip` | 18.04 µs | 21.35 µs | **18.3%** |

| the small arithmetic kernels | lowest | highest | spread |
| --- | ---: | ---: | ---: |
| `ffi.scalar_inlined` | 2.10 ns | 2.10 ns | **0.0%** |
| `ffi.scalar_opaque_rust` | 2.10 ns | 2.10 ns | **0.0%** |
| `physics.voxel_lookup` | 13.00 ns | 13.20 ns | **1.5%** |
| `physics.timestep_accumulate` | 5.20 ns | 5.30 ns | **1.9%** |
| `voxel.get_uniform` | 2.90 ns | 3.00 ns | **3.4%** |
| **`voxel.get_paletted`** — the row the entry is named for | 5.30 ns | 5.60 ns | **5.7%** |

**Magnitude does not predict instability. What the row touches does.** Every one
of the six worst rows goes through `fsync`, the disk, or thread scheduling. The
steadiest rows in the whole suite are the two-nanosecond arithmetic kernels,
which did not move by a single reported digit.

Across three separate invocations of the script:

| | rows under 20 ns, worst | any row, worst |
| --- | ---: | ---: |
| 2 builds | 5.0% | 17.2% (`jobs.submit_wait_roundtrip`) |
| 3 builds | 10.2% | 33.1% (`jobs.submit_all_1000`) |
| 3 builds, again | 13.0% | 36.3% (`journal.append_durable`) |

**The floor grows with the number of builds, which is the honest shape of it.**
A spread is a lower bound on the range, and sampling more finds more. Two builds
are enough to know a floor exists and not enough to size it; the default is
three and `--builds` takes more.

### One sub-20 ns row that did move, and why it is less than it looks

`entity.resolve_handle` spread **13.0%** — 2.30 ns to 2.60 ns. The harness
reports to 0.1 ns, so at 2.3 ns a single digit of display resolution is already
4.3%, and the whole "13%" is three of those ticks. That is a quantisation
artefact as much as a measurement, and it is the reason the rule below is about
comparing against a row's *measured* spread rather than against a percentage
someone picked.

### What this does not explain, and will not from here

**The 11.4 → 7.8 ns observation that opened `DEBT-0039` is not reproduced by
this, and cannot be.** Identical source is a different experiment from two
different commits: the commit in question did change real code, and thin LTO can
inline differently because of that, not merely lay code out differently. This
measures only the second effect.

The box has also moved. `voxel.get_paletted` reads **5.3–5.6 ns** here against
the 11.4 and 7.8–8.2 ns recorded then, so the original pair cannot be re-run for
comparison from this machine at all. The entry's observation stands as recorded;
what it *inferred* from it — that smallness is the hazard — is what the
measurement contradicts.

### The rule that replaces "below ~20 ns"

> Before publishing a gain, run `scripts/build-spread.sh` and compare the claim
> against **that row's** spread. A change smaller than the row's own
> build-to-build floor is not a finding, whatever the row's magnitude.

It is deliberately not in CI: three release builds of the workspace is minutes
of compute for a number that only matters when someone is about to publish a
comparison. It is also unrelated to `.github/workflows/release.yml`, despite the
entry's wording about "the release script" — the script meant is the one that
produces published numbers, which is this.

The script restores the file it edits on exit, including on failure, and refuses
to start if that file already has uncommitted changes — a tool for measuring
build noise has no business becoming a source of it.

---

## Finding 25 — a frame's accounting costs about 70 ns, and it is not the division

`CORE.md` §16 has specified a game loop since Phase 0 and nothing implemented
one, so every number in this document has been compared against a frame by hand.
`engine/runtime/src/frame.rs` ([ADR-0017](../adr/ADR-0017-a-frame-is-time-the-host-hands-in.md))
is that loop. The obvious objection to an accounting layer is that it changes
what it measures, so the first thing measured about it is itself.

| row | reading | what it covers |
| --- | --- | --- |
| `frame.schedule_advance` | **39.0 ns** | the accumulator alone: one delta in, a step plan out |
| `frame.accounting_one_stage` | **65–78 ns** | a whole frame: open, charge one stage, close, classify |
| `frame.accounting_seven_stages` | **74–77 ns** | the same frame with all seven stages charged |
| `frame.stages` | **7** | the sequence `CORE.md` §16 declares |
| `frame.stages_with_a_system` | **3** | of those, the ones anything in this repository can run in |

(`frame.stages_with_a_system` reads **4** since ENGINE-8 staffed the `Input`
stage — finding 26. The **3** above is what it read when this finding was taken.)

Two readings are given for the two timed frame rows because they were measured
twice, back to back, on the same binary: 78.0 and 65.0 ns, 77.0 and 74.0 ns. The
spread is the reading, not one of the numbers.

**Charging seven stages costs the same as charging one.** The two rows overlap
inside their own spread, so per-stage attribution is effectively free and the
whole fixed cost is in `begin` and `finish` — most of it in the accumulator,
which is 39 ns of the ~70.

### A guess about that 39 ns, and the measurement that killed it

`FrameSchedule::advance` divides `Duration::as_nanos()` by the step, which is a
128-bit division — a software routine on this target, and the obvious suspect
for a figure that large. It is wrong. Replacing it with a 64-bit division when
both operands fit (which they do for any step shorter than 584 years) moved the
row from **42.0 ns to 40.0 ns** against a **37% spread**: nothing.

The cost is the `Duration` arithmetic around it. One `advance` does a
`saturating_add`, a `checked_mul`, and one or two `saturating_sub`s, each on a
`(u64 seconds, u32 nanos)` pair with normalisation and overflow checks. Five
checked operations on a two-field type is where the time goes, and the 64-bit
fast path was reverted rather than kept: complexity bought nothing.

This is the fourth entry in this document where a prediction about *where* the
cost lived was wrong and the control said so — after `DEBT-0034`, `DEBT-0010`'s
`by_type` extrapolation, and `DEBT-0039`'s own diagnosis.

### Is it cheap enough to run every frame?

Against the 50 ms a 20 Hz frame is allowed, 70 ns is **0.00014%**. Against the
cheapest *real* frame this engine can currently run — `streaming.idle_tick_r3`
at **3.03 µs and 2.98 µs** in the same two runs — it is about **2.3%**.

The second figure is the honest one to quote, and it is small but not nothing:
an engine whose frames were all idle streaming ticks would spend a fortieth of
its time saying where the time went. That is an argument for the accounting
staying as thin as it is, not for removing it, and it is the reason the seven
stages are a fixed array indexed by `FrameStage::index` rather than a map.

### What is not measured here

No frame in this repository has ever been driven by a real clock. The slice's
walk hands the loop a scripted delta of exactly one step, on purpose — the slice
is a determinism proof, not a measurement — so `StepPlan::discarded` is always
zero, the budget class is always `Target`, and `FrameReport::unattributed` has
never been observed against real work outside these benchmark rows. That gap is
`DEBT-0041`, and it is deliberately queued behind `DEBT-0018` and `DEBT-0027`:
running frames against a clock while generation and meshing still sit on the
tick thread would measure those two, not the loop.

---

## Finding 26 — the input system costs a third of a microsecond a frame, and 70% of that is looking up the context

`CORE.md` §24 and `INPUT SYSTEM.md` specified ENGINE-8 from the start and
nothing implemented it, so `FrameStage::Input` existed and reported
`has_system() == false`. `engine/runtime/src/input.rs`
([ADR-0018](../adr/ADR-0018-input-is-intent-the-host-hands-in.md)) fills it.
What it costs matters more than most rows here for one reason: **resolution runs
every frame whether or not anything was pressed.** A player standing still pays
the same as a player in a fight.

| row | reading | what it covers |
| --- | --- | --- |
| `input.sample_idle` | **322–356 ns** | a frame with no signals at all, resolved against the whole keymap |
| `input.sample_one_key` | **551–872 ns** | a frame carrying one key transition |
| `input.sample_stick` | **723 ns – 1.02 µs** | a frame carrying an analogue reading through dead zone, curve and sensitivity |
| `input.validate_remote` | **87–129 ns** | checking a two-action snapshot from outside the trust boundary |
| `input.bindings` | **41** | bindings the sampled keymap holds, across two contexts |

The binding count is published beside the timings deliberately. Every timed row
above is a per-frame cost *at that keymap size*; quoting one without the other
says nothing, because the scan is linear in bindings.

### These are the least reproducible rows in the suite

Four separate runs of the same binary read `input.sample_idle` at **341, 485,
354 and 928 ns**, while `streaming.idle_tick_r3` in those same four runs held a
3.5% band (2.99–3.10 µs). A row that swings 2.7× beside a row that does not move
is the box, not the code — and it means **no point figure should be quoted for
these rows from this machine**, which is why the table above gives ranges.

This does not match the shape `DEBT-0039` found. There the unstable rows were
the ones touching fsync, the disk or thread scheduling, and the small arithmetic
kernels were the steadiest things in the suite. These rows touch none of that.
What they do is chase pointers — `BTreeMap` nodes and the heap-allocated strings
inside `Identifier` — so the reading depends on cache state in a way a
register-bound kernel does not. That is an explanation, not a measurement: what
is established is the instability, not its cause.

### The one comparison that is clean

Because a single reading proves nothing here, the diagnostic was run three times
a side, on the same machine, minutes apart, with `frame.schedule_advance` as the
control that must not move — and did not:

| `input.sample_idle`, 41 bindings | 1 | 2 | 3 | control |
| --- | ---: | ---: | ---: | ---: |
| as written | 322 ns | 342 ns | 356 ns | 39 / 42 / 40 ns |
| with the context lookup removed | 102 ns | 102 ns | 99 ns | 41 / 41 / 40 ns |

The ranges are disjoint by 3×. **About 70% of an idle frame's input cost is
`BTreeMap<Identifier, i32>::get`, called once per binding to find out which
context that binding belongs to** — roughly 5.4 ns each, which is what comparing
two heap strings per tree level costs. The remainder is the scan itself.

Note that the diagnostic changes what the other rows *mean* — with every
priority forced to zero, a pressed key resolves to a different winner — so only
the idle row, in which no binding fires either way, is being compared like for
like. The other two rows moved in that build and those movements are not
evidence of anything.

### Not fixed, on purpose

The remedy is known and written down: index bindings by context so the lookup
happens once per active context (1 to 3) instead of once per binding (dozens).
It is `DEBT-0042` and not a commit, because 322 ns is **0.00064%** of the 50 ms a
20 Hz frame is allowed, and about **11%** of an idle streaming tick. `DEBT-0010`
is the precedent that decides this: there the scan cost 979.7 µs and earned its
index; here it costs a third of a microsecond and does not. The trigger is
written into the entry — a keymap past ~150 bindings, or input showing up with a
non-trivial share of a real frame.

### What is not measured here

The second dimension of the scan. `button_held` walks the set of held buttons
*inside* the per-binding loop, so the real cost is O(bindings × held), and every
measurement above has at most one key down. A chord-heavy keymap with four
modifiers held is a case this document has no number for.

And, as with the frame loop: no real device has ever produced a signal. The only
caller outside the tests is the slice's scripted player, tapping two keys on a
keyboard that does not exist. `DeviceKind::Mouse` and `DeviceKind::Touch` have no
caller at all, no remap file has been written to disk, and `validate_remote` has
never examined a snapshot that crossed a network. That is `DEBT-0043`, and it
waits on the same host `DEBT-0041` waits on — whoever owns the clock owns the
devices.

# Appendix I — a second machine (2026-09-25)

Every number above came from one shared Linux container. `DEBT-0013` held the
physics budget back until something else had been measured, because a
threshold taken from a single shared runner would claim precision nobody had
earned. The local validation bridge (`docs/validation/local/`) delivered that
second machine: its third report keeps the whole benchmark.

| | machine A | machine B |
| --- | --- | --- |
| what | a real desktop | the shared container everything above ran in |
| OS | Windows 10, AMD64 | Linux, x86_64 |
| CPU | AMD Ryzen 5 5500, 12 logical | 4 logical, shared |
| commit | `0b7bcec` (local report 3) | `8b9b824`, two runs; `engine/physics`, `engine/benchmark`, `engine/world` and `engine/simulation` are unchanged since `0b7bcec` |
| toolchain | rustc 1.94.1 | rustc 1.94.1 |

## Finding 27 — the machines differ by a factor in compute, and by the opposite factor on disk

### Physics: one factor, one order

| measurement | A median | B median (two runs) | B / A |
| --- | ---: | ---: | ---: |
| `physics.character_step` | 370.4 ns | 502.8–504.8 ns | 1.36 |
| `physics.thousand_bodies_step` | 84.62 µs | 113.89–116.36 µs | 1.36 |
| `physics.thousand_bodies_step_flat` | 80.62 µs | 106.91–110.04 µs | 1.35 |
| `physics.voxel_lookup` | 12.3 ns | 16.8–17.0 ns | 1.37 |
| `physics.axis_sweep` | 29.1 ns | 36.9–38.3 ns | 1.29 |
| `physics.box_sweep` | 144.6 ns | 215.2–215.9 ns | 1.49 |
| `physics.depenetration_check` | 48.6 ns | 75.8–77.9 ns | 1.58 |
| `physics.raycast_40m` | 839.8 ns | 1.09 µs | 1.30 |
| `physics.thousand_sleeping_step` | 850.0 ns | 1.08 µs | 1.27 |

Every physics row is 1.27–1.58× slower on B, and the rows keep their order on
both machines. The ratios the earlier findings rest on survive the move: the
flat fixture is still ~95% of the terrain case (95.3% on A, 94–95% on B), so
Finding 10's diagnosis still holds after 10b–10e. Sleeping still saves about
100× (99.6× on A, 105–108× on B).

### Everything else: not one factor

Across all 69 timed rows the median B/A ratio is **1.29**, but the spread is
wide, and it points both ways:

| row | A | B | B / A |
| --- | ---: | ---: | ---: |
| `journal.append_unsynced` | 2.81 µs | 0.81 µs | **0.29** |
| `save.region_write_one_dirty` | 9.03 ms | 2.70 ms | **0.30** |
| `journal.append_durable` | 544.95 µs | 181.21 µs | **0.33** |
| `save.region_write_all` | 41.05 ms | 14.95 ms | **0.36** |
| `worldgen.chunk_32` | 1.11 ms | 1.31–1.35 ms | 1.20 |
| `mesh.region_32` | 12.13 ms | 15.07–15.43 ms | 1.26 |
| `save.crc32_64kib` | 263.67 µs | 354.12–355.16 µs | 1.35 |
| `jobs.batch_1000_barrier_submit_all` | 790.20 µs | ~1.8 ms | 2.30 |
| `jobs.submit_wait_roundtrip` | 7.82 µs | 29.67–35.26 µs | **4.15** |

**Disk is about three times slower on A**, and a fsync'd journal record costs
545 µs there against 181 µs here. The container's storage is not a desktop's
NTFS volume, and a durability budget taken from it would be three times too
generous. **The job system is two to four times slower on B**: 4 shared
logical CPUs against 12 dedicated ones, and a round trip that wakes a thread
is the measurement most exposed to that. Compute kernels sit in between, at
1.2–1.6×.

One row moved between B's two runs for no reason: `entity.spawn` read 1.19 µs
and then 177.8 ns. It is the noisiest row on both machines (rel. σ 39.5% on A),
and Appendix H already says what a single run of it is worth.

### What this licenses, and what it does not

- **Physics can publish.** Two machines, one factor, one order: the threshold
  can be set from the structure of the numbers instead of from one run. The
  budget is `nexora_physics::budget::CROWD_SUBSTEP`, derived in that module's
  documentation. TARGET is 250 µs: machine B's worst p95 (168.01 µs) with half
  again to spare. That margin also covers Appendix H's 29.9% build-to-build
  spread for the flat step. EMERGENCY is 2 ms, from frame arithmetic, not
  measurement: eight catch-up substeps of 2 ms fill a 60 Hz frame. On both
  machines the median and the p95 land in `target`. The benchmark now prints
  that verdict in every run, under **Published budgets**.
- **I/O cannot publish from these two.** The factor between them is 3× in the
  other direction, and B's disk is the unusual one. A durability or save
  budget needs a third machine, or a decision to budget against the slowest
  one measured.
- **The job system cannot publish from B.** A 4-CPU shared runner prices
  thread wake-ups at up to four times what a desktop does. `DEBT-0009`'s
  numbers are B's and should be read as a ceiling, not an estimate.
- **Nothing here is a GPU number.** DEBT-0008 is untouched.

# Appendix J — the RHI stage, on a software driver (2026-09-26)

The plan's slice lists `RHI` between `input` and `camera`. Until ADR-0026 there
was no backend to measure, and until this appendix the benchmark did not
drive the one that exists. `nexora_benchmark::gpu::rhi` now runs it, headless,
on whatever adapter the machine has, and prints that adapter at the top of the
report. It checks its answers before it times anything: the upload is read
back byte for byte, the draw must shade every texel, and the target is read
again after the timed draws. A backend that is fast because it skipped the
work fails there, and replacing the timed draw with a marker was tried to
show it: the run stops with `a timed draw did not shade its target`.

| | |
| --- | --- |
| machine | the shared container (machine B of Appendix I), 4 logical CPUs |
| adapter | `llvmpipe (LLVM 20.1.2, 256 bits)`: Vulkan, **device type `cpu`** |
| budget | `Budget::coarse(1)`, 7 samples |

| measurement | median | p95 | what it is |
| --- | ---: | ---: | --- |
| `rhi.device_open` | 21.25 ms | 24.81 ms | adapter and device, no surface |
| `rhi.fence_roundtrip` | 67.98 µs | 72.60 µs | an empty list and its fence |
| `rhi.texture_create_destroy` | 1.58 µs | 2.96 µs | a 16×16 RGBA8 texture, nothing in flight |
| `rhi.upload_texture_16` | ~98 µs (10 MiB/s) | 99.19 µs | 1 KiB, one write, one fence |
| `rhi.upload_first_generation` | ~256 µs (61 MiB/s) | 271.29 µs | 16 KiB, sixteen writes, one fence |
| `rhi.upload_mesh_16` | ~108 µs (464 MiB/s) | 112.58 µs | 50.4 KiB: the 16³ region's 3,228 vertices |
| `rhi.draw_16` | 279.47 µs | 305.98 µs | one triangle into a 16×16 target |

## Finding 28 — on a software driver, waiting costs more than the work, so the slice's one-fence upload is the right shape

**The fence dominates everything small.** An empty submission and its wait
cost 68 µs, and one 1 KiB texture costs 98 µs. Two thirds of that upload is
the round trip, not the copy. The vertex buffer for the meshed 16³ region is
fifty times larger and costs only 10 µs more.

**So batching pays, and the slice already batches.** Sixteen textures in one
submission cost 256 µs. Sixteen separate submissions would cost about
16 × 98 ≈ 1.6 ms, six times as much. ADR-0025's slice uploads the first
generation that way (sixteen writes, one fence), and the measurement agrees
with the choice. It is the same shape Finding 21 found for the job system,
where one wake-up per wave beat one per job.

### What these numbers are not

- **Not a GPU's.** lavapipe rasterizes on the CPU, and its "device" shares
  the four cores that run everything else. The draw's 279 µs is LLVM-compiled
  shader code running on those cores. A discrete GPU draws a 16×16 triangle
  in far less, and waits on its fence across PCIe, which lavapipe does not.
  The ratios above (fence against copy, batched against unbatched) are what
  carry over, and even they must be measured again on hardware before a
  budget is written from them.
- **Not a frame.** No window, no presentation, no camera: the benchmark
  stays runnable without a display. Frame time needs a renderer, which does
  not exist.
- **Not a budget.** One machine, one software driver. Appendix I's rule
  applies: a threshold waits for a second machine, and this stage has only
  one. The operator's RX 6650 XT runs this stage in `benchmark_cpu` of the
  next local report, with the adapter named in its environment table.

# Appendix K — the RHI stage, on a real GPU (2026-09-26)

Appendix J ended by saying the operator's RX 6650 XT would run this stage in
the next local report. Two reports did, sixteen minutes apart, on the same
code: report 4 on `786f77f` (the merge of ADR-0028) and report 5 on
`8331c15`, which only adds report 4's files. `local-validation.py check`
classifies the second as `CURRENT_NO_RELEVANT_CHANGE` against `HEAD`. The
benchmark checked its answers on this device before timing, as it does
everywhere: the upload read back identical and the draw shaded every texel.

| | |
| --- | --- |
| machine | machine A of Appendix I: Windows 10, AMD Ryzen 5 5500, 12 logical CPUs |
| adapter | `AMD Radeon RX 6650 XT`: Vulkan, **device type `discretegpu`** |
| budget | `Budget::coarse(1)`, 7 samples |

Throughput rows print a rate and the p95 of the time, as in every report;
the rate is rounded to a whole MiB/s, so it is coarse at 1 KiB.

| measurement | report 4 median | report 5 median | p95 (4 / 5) | lavapipe (J) |
| --- | ---: | ---: | ---: | ---: |
| `rhi.device_open` | 275.64 ms | 248.75 ms | 325.36 / 268.07 ms | 21.25 ms |
| `rhi.fence_roundtrip` | 111.79 µs | 162.34 µs | 180.35 / 289.84 µs | 67.98 µs |
| `rhi.texture_create_destroy` | 2.36 µs | 4.28 µs | 2.63 / 4.86 µs | 1.58 µs |
| `rhi.upload_texture_16` | 6 MiB/s | 4 MiB/s | 212.13 / 322.24 µs | 10 MiB/s |
| `rhi.upload_first_generation` | 58 MiB/s | 52 MiB/s | 328.50 / 353.93 µs | 61 MiB/s |
| `rhi.upload_mesh_16` | 342 MiB/s | 281 MiB/s | 281.07 / 301.75 µs | 464 MiB/s |
| `rhi.draw_16` | 192.04 µs | 205.31 µs | 212.93 / 260.45 µs | 279.47 µs |

## Finding 29 — on a real GPU the wait is larger, not smaller, so a renderer submits once per frame

Finding 28 said the ratios would carry over and the absolute numbers would
not. Both halves held, in a direction worth writing down.

**The fence costs more on the discrete GPU than on the software one.** An
empty submission and its wait took 112 and 162 µs here, against 68 µs on
lavapipe. lavapipe's "device" is a thread in the same process; this one is a
separate processor across PCIe, behind a kernel driver, and every wait
crosses both. Opening the device is twelve times slower for the same reason
(249–276 ms against 21 ms): a real driver initialises hardware.

**The draw is almost all wait.** One triangle into a 16×16 target costs
192–205 µs, barely more than an empty fence. The GPU's work is nothing; what
is measured is submit, execute, signal and wake. On lavapipe the draw cost
four times the fence because the CPU did the rasterising. Here it costs about
1.3–1.7 times the fence.

**So batching pays more here than in Appendix J.** One 1 KiB upload is
roughly 160–240 µs (1 KiB at 6 and 4 MiB/s); sixteen of them in one
submission are roughly 270–300 µs (16 KiB at 58 and 52 MiB/s). Sixteen
separate submissions would cost about ten to thirteen times the batched one,
against six on lavapipe. The slice's one-fence upload was the right shape on
the software driver and is more right on the hardware.

The consequence is for code not yet written: **a renderer submits one list
per frame and waits on at most one fence per frame.** A renderer that waits
per draw, or per chunk upload, would spend its frame waiting on PCIe round
trips of a tenth of a millisecond each, whatever the GPU's speed.

### What these numbers are not

- **Not a budget.** The same machine, the same code, sixteen minutes apart,
  moved the fence round trip by 45% and the draw by 7%. Appendix I's rule
  still applies, and a spread like that is a reason by itself: a threshold
  drawn from either run would misjudge the other.
- **Not a frame.** Still no camera and no frame time; nothing draws the
  world.
- **Not Direct3D 12 or Metal on hardware.** `wgpu` picked Vulkan on this
  machine. D3D12 runs in CI only on WARP, and Metal only on Apple's
  paravirtual device.

# Appendix L — the camera stage (2026-09-26)

The slice's `camera` stage had no code until ADR-0029. `suites::camera` now
resolves a camera standing **2^40 blocks out**, at the far corner of the
world, and tests the 625 columns a radius-12 observer streams (full height,
-64 to 320) against its frustum. Before timing, it checks that the point the
camera looks at lands mid-screen and that culling keeps some columns and
drops others.

| | |
| --- | --- |
| machine | the shared container (machine B of Appendix I), 4 logical CPUs |
| budget | `Budget::standard(1)` |
| camera | 70° vertical field of view, 1600×900, near 0.1, far 512, looking down and ahead |

| measurement | run 1 median | run 2 median | p95 (1 / 2) |
| --- | ---: | ---: | ---: |
| `camera.sample` | 112.0 ns | 145.0 ns | 152.0 / 147.0 ns |
| `camera.cull_columns_r12` | 3.42 µs | 4.35 µs | 5.52 / 6.87 µs |
| `camera.visible_columns_r12` | 261 | 261 | — |

## Finding 30 — the camera costs nothing a frame, and the edge of the world costs the same as its centre

**Resolving the camera costs about a tenth of a microsecond.** That covers
the view, the reverse-Z projection, their product in `f64`, the rounding to
`f32` and the six frustum planes. Culling a radius-12 observer's 625 columns
costs 3–7 µs, about 5–11 ns a column. Together that is **under 0.05% of a
60 Hz frame**. There is nothing in this stage worth optimising, and no reason
to cull more coarsely than per column.

**The edge of the world is not special.** The camera stands 2^40 blocks out,
where an `f32` step is 131,072 blocks, and the numbers show no cost for it.
The work happens relative to an integer origin, so it is the same arithmetic
at the centre and at the edge. The camera's tests check this as bits, not as
time: the same scene at the centre and at the edge produces identical
matrices.

**A 70° view keeps 261 of 625 columns, 42%.** The frustum is a wedge, so
more than half the columns a streaming radius loads are behind or beside the
camera. This is the first number that says what a renderer would *not* have
to draw. It is conservative: a column near a frustum corner is kept.

### What these numbers are not

- **Not a frame.** Frame time needs a pass that draws the meshed chunk
  through this camera, and that pass does not exist yet (DEBT-0008).
- **Not a budget.** The container's run-to-run spread (run 1's `sample` has a
  233.6% relative σ from one outlier) is larger than anything a threshold
  here would protect, and at under 0.05% of a frame there is nothing to
  protect.

# Appendix M — frame time, on a software driver (2026-09-26)

The last GPU stage the gate asked for with no code behind it. ADR-0030 built
the first render pass, and `gpu::frame_time` times one frame of it. The frame
shows the meshed 16³ region the `mesh` suite meshes, from the same generated
world, drawn at 256×256 through a camera looking down at it. One frame is
clear, camera, one draw, one submission and one fence.

Before timing, the frame is checked against a ray cast on the CPU
(`nexora_render::reference`). **51,376 of 65,536 pixels are judged, and all
of them match.** A `Less` depth test in place of `Greater` drops that to
39,391, and the run stops without timing. After timing, the target is read
again and must still hold the checked frame.

| | |
| --- | --- |
| machine | the shared container (machine B of Appendix I), 4 logical CPUs |
| adapter | `llvmpipe (LLVM 20.1.2, 256 bits)`: Vulkan, device type `cpu` |
| budget | `Budget::coarse(1)`, ten frames a sample |

| measurement | run 1 | run 2 |
| --- | ---: | ---: |
| `frame.draw_chunk_16` median | 1.80 ms | 1.52 ms |
| `frame.draw_chunk_16` p95 | 2.90 ms | 1.58 ms |
| `frame.chunk_16_vertices` | 4,842 | 4,842 |
| `frame.pixels_judged` | 51,376 | 51,376 |
| `rhi.fence_roundtrip` median, same run | 50.24 µs | 60.58 µs |

## Finding 31 — on a software driver a frame is rasterisation; on the operator's GPU it will be the wait

**On lavapipe the frame costs 1.5–1.8 ms, about thirty fence round trips.**
llvmpipe rasterises 65,536 pixels and shades 4,842 vertices on the same four
cores that run everything else, so the frame is almost all rasterisation.
That is about a tenth of a 60 Hz frame, for one chunk, on a CPU.

**What carries over is the shape, not the number.** Appendix K measured the
operator's RX 6650 XT at 112–162 µs a fence and 192–205 µs for a 16×16
draw. A discrete GPU draws 4,842 vertices into 256×256 pixels in a small
fraction of that. So there a frame of this pass should cost about one fence,
and a frame of many chunks should cost about one fence plus their draws.
That holds only because `ChunkPass::record` puts the whole frame in one
submission. Measuring it is the next local report's job: `benchmark_cpu`
runs this stage on the machine's own adapter.

### What these numbers are not

- **Not a GPU's.** As in Appendix J: lavapipe is the CPU.
- **Not a world.** One chunk, flat-coloured, no textures and no streaming.
  A frame of the world draws hundreds of chunks, and this pass has not yet
  been asked to.
- **Not a window.** Drawn into a texture. Presenting to a window, and
  waiting for its vertical blank, is the window probe's to measure (ADR-0027).
- **Not a budget.** A software driver, and 18% between two runs.
