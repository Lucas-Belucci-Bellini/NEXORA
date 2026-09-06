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
