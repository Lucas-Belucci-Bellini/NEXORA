# NEXORA

A voxel sandbox built around one idea:

> **The world is generated. The world then lives.**

The simulation does not exist to serve a player who is standing in it. A world
keeps a history, and the player is one participant in it.

## Where the project is

The architecture is specified across ~145 documents in this repository. **Phase 0
— the engine foundation — is now implemented, tested and runnable**, and the
headless vertical slice proves the chain end to end:

```text
boot lifecycle -> resolve modules -> create world from a seed
   -> generate chunks across a worker pool -> mutate voxels -> advance the clock
   -> save -> shut down -> reopen -> verify the state survived
```

There is deliberately **no renderer and no window yet**. Those are specified but
unbuilt, because they cannot be verified in the environment Phase 0 was built in,
and `NEXORA DEFINITION OF DONE.md` does not accept unverified work.
[ADR-0005](docs/adr/ADR-0005-phase-0-scope-boundary.md) lists exactly what is and
is not implemented.

The implementation language is **not locked**. Rust is the reference
implementation for the benchmark gate defined in
`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17 — see
[ADR-0001](docs/adr/ADR-0001-rust-phase-0-reference-implementation.md).

## Running it

Requires the toolchain pinned in `rust-toolchain.toml`; `rustup` installs it
automatically.

```bash
cargo test --workspace          # 187 tests
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p nexora-headless    # the vertical slice, verified end to end
```

The runner reports what it did and exits non-zero if the world does not come
back intact:

```text
NEXORA headless vertical slice
------------------------------
world id           0x368cfbaaa04ab32c
chunks generated   25
non-air blocks     50818052
voxel storage      722304 bytes
save size          744954 bytes
probes verified    76
result             OK
```

Fifty million blocks held in 722 KB is the palette and uniform-section storage
doing its job: solid rock costs nothing to hold.

```bash
cargo run -p nexora-headless -- --help      # seed, radius, threads, save path
```

## Measuring it

The language selection gate needs evidence, so the reference implementation is
measured rather than asserted:

```bash
cargo run --release -p nexora-benchmark
```

Results and analysis:
[`docs/benchmarks/PHASE-0-BASELINE.md`](docs/benchmarks/PHASE-0-BASELINE.md).
Highlights — 18.3 million blocks held in 144 KiB, 7.0 MiB peak resident memory,
a measurement that closed `DEBT-0005` by showing the "optimization" it proposed
would have been **4× slower** than the code it was meant to improve, and the
number behind `DEBT-0009`: scheduling one job per entity costs **~4,900× the
simulation it schedules**.

## Layout

The crate graph *is* the dependency matrix — Cargo rejects cycles, so a layering
violation fails the build rather than a review
([ADR-0003](docs/adr/ADR-0003-workspace-layout-enforces-dependency-matrix.md)).

```text
engine/foundation    errors, versions, identifiers, space, time, determinism,
                     diagnostics, configuration          (no dependencies at all)
engine/persistence   versioned, checksummed, atomic save container
engine/runtime       lifecycle, engine modules, registries, event bus, jobs
engine/world         voxel storage, chunks, world lifecycle, generation
engine/entity        identity, lifecycle, components, queries, persistence
engine/benchmark     the measurement harness for the language gate
engine/headless      the Phase 0 vertical slice
docs/adr/            architecture decision records
```

## Reading order

| Start here | For |
| --- | --- |
| [`NEXORA MASTER ARCHITECTURE.md`](NEXORA%20MASTER%20ARCHITECTURE.md) | the whole shape |
| [`docs/adr/`](docs/adr/) | decisions taken, and why |
| [`NEXORA ARCHITECTURE FREEZE CHECKLIST.md`](NEXORA%20ARCHITECTURE%20FREEZE%20CHECKLIST.md) | what is defined vs. what is built |
| [`NEXORA DEVELOPMENT ROADMAP.md`](NEXORA%20DEVELOPMENT%20ROADMAP.md) | phase order |
| [`NEXORA TECHNICAL DEBT REGISTER.md`](NEXORA%20TECHNICAL%20DEBT%20REGISTER.md) | shortcuts taken, with owners |
| [`NEXORA DOCUMENTATION INDEX.md`](NEXORA%20DOCUMENTATION%20INDEX.md) | everything else |

Every source file names the document whose contract it implements. If code and
document disagree, the document wins until an ADR says otherwise
(`NEXORA DEVELOPER WORKFLOW AND CHANGE PROCESS.md`).

## What comes next

The open gate is still the **technology benchmark** (`DEBT-0008`), now about a
half complete: the reference stack is measured through the chunk, job,
save/load and 1,000-entity stages, but rule 5 of
`NEXORA TECHNOLOGY BENCHMARK PLAN.md` forbids deciding from a single stack, and
no second one has been measured. The next measurable increments are the **entity
system** (needs no hardware) and then the **RHI** (cannot be measured headless at
all).

Measurement also opened `DEBT-0009` and `DEBT-0010`: the job system costs
~4,900× the work when used per entity (it is a chunk-granularity tool), and
entity queries are linear scans that stop being free somewhere above 10,000
entities. Both have trigger points rather than guesses.

## Originality

NEXORA is an original project. It contains no code, assets, textures, models,
sounds or shaders taken from any other game. Conceptual influence is not
implementation: see `NEXORA ORIGINAL CONTENT AND ASSET POLICY.md`. The
algorithms implemented in `engine/foundation` (FNV-1a, CRC-32, SplitMix64) are
public specifications, implemented here from those specifications and pinned by
their published reference vectors.
