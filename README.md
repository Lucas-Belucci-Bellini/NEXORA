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
   -> generate chunks across a worker pool -> mutate voxels
   -> drop a character onto the terrain and simulate it until it settles
   -> walk an observer away and back, streaming chunks out and in
   -> advance the clock -> save -> shut down -> reopen -> verify the state survived
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
cargo test --workspace          # 469 tests
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
physics bodies     9 (9 settled)
physics substeps   600 (270 contacts)
character drop     1010 cm
streaming ticks    29 (95 generated, 120 evicted, 25 restored)
chunks retained    25 (peak, edits that cannot be regenerated)
probes verified    76
result             OK
```

Fifty million blocks held in 722 KB is the palette and uniform-section storage
doing its job: solid rock costs nothing to hold. The observer walks away from
the edited region and back before the save, so those 76 verified probes are also
proof that streaming evicted 120 columns without losing an edit.

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
would have been **4× slower** than the code it was meant to improve, the number
behind `DEBT-0009` (scheduling one job per entity costs **~4,900× the simulation
it schedules**), and the one behind `DEBT-0011`: **49% of a physics step is the
voxel lookup, not the solver** — measured by running the same 1,000 bodies
against generated terrain and against a flat fixture, because a single combined
number would have sent the optimisation work to the wrong half.

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
engine/physics       fixed timestep, bodies, swept voxel collision, characters,
                     ray queries         (depends on foundation and nothing else)
engine/streaming     interest, priority, budgets, LOD tiers, eviction
                     (also depends on foundation and nothing else)
engine/simulation    the one crate allowed to see the world, physics and
                     streaming at the same time
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

The open gate is still the **technology benchmark** (`DEBT-0008`). Every stage
of the plan's vertical slice is now either measured — chunks, jobs, save/load,
1,000 entities, physics, streaming — or blocked on something no amount of Rust
provides: a GPU for the render stages, or a second language for the FFI
boundary. Rule 5 of `NEXORA TECHNOLOGY BENCHMARK PLAN.md` forbids deciding from
a single stack, and no second one has been measured.

**The next thing that moves the gate is the same workload, built again, in
something that is not Rust.**

Measurement also opened `DEBT-0009` through `DEBT-0020`, each with a trigger
point rather than a guess. Among them: the job system costs ~4,900× the work
when used per entity, entity queries stop being free above ~10,000 entities, the
voxel lookup is half of a physics step, and streaming generates chunks on the
tick thread — where spending its own activation budget would cost 21.8 ms, more
than a frame.

## Originality

NEXORA is an original project. It contains no code, assets, textures, models,
sounds or shaders taken from any other game. Conceptual influence is not
implementation: see `NEXORA ORIGINAL CONTENT AND ASSET POLICY.md`. The
algorithms implemented in `engine/foundation` (FNV-1a, CRC-32, SplitMix64) are
public specifications, implemented here from those specifications and pinned by
their published reference vectors.
