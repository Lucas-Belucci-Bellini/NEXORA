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
cargo run --release -p nexora-benchmark      # the Rust reference
scripts/compare-stacks.sh                    # Rust vs C++, every compiler found
```

The second script needs a C++20 compiler; nothing else in the repository does.
It checks conformance before it times anything, and **fails without producing a
single number** if the stacks disagree on any digest.

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
engine/command       intent: definitions, validation, dispatch, quotas
engine/benchmark     the measurement harness for the language gate
benchmarks/cpp       a C++20 reference of the hot kernels -- not an engine
benchmarks/ffi-probe the one crate allowed to say `unsafe`, and why (ADR-0009)
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

The open gate is still the **technology benchmark** (`DEBT-0008`), but it moved.
The second stack now exists: `benchmarks/cpp/` mirrors the engine's hot kernels
in C++20, and `scripts/compare-stacks.sh` checks twelve conformance digests
across Rust, g++ and clang++ before timing anything — because two
implementations that disagree about *what* they compute cannot be compared on
how fast they compute it.

The result is in [Appendix D](docs/benchmarks/PHASE-0-BASELINE.md), and the
useful half of it is a warning about how easy this is to get wrong:

> CRC-32 over 64 KiB runs in **668 µs under g++**, **353 µs under clang++**, and
> **364 µs in Rust** — same algorithm, same flags. Timed against g++ alone, Rust
> looks 1.8× faster than "C++"; the difference was GCC versus LLVM. **The
> optimizer backend costs more than the language.**

Across nine shared kernels Rust lands inside the band the two C++ builds span in
five, beats both in two, and trails the nearer one in two, by 13% and 11%. FFI
overhead — a plan metric that was unmeasurable with one language in the build —
is **~1.2 ns per crossing** at the C ABI floor, the same as any un-inlined call,
and invisible at 4 KiB per crossing.

**What still blocks the gate is no longer "no second stack".** It is the GPU
stages, which cannot run in a headless container, and an engine-scale
comparison, which a kernel reference is explicitly not (ADR-0009). Nine pieces
of arithmetic say nothing about allocation, cache behaviour at scale, or
threading — and `NEXORA LANGUAGE AND FFI BOUNDARY.md` reserves the language lock
for the completed benchmark.

Measurement also opened `DEBT-0009` through `DEBT-0020`, each with a trigger
point rather than a guess. Among them: the job system costs ~4,900× the work
when used per entity, entity queries stop being free above ~10,000 entities, the
voxel lookup is half of a physics step, and streaming generates chunks on the
tick thread — where spending its own activation budget would cost 21.8 ms, more
than a frame.

## Commands

`NEXORA ARCHITECTURE RULES.md` §4 separates **command** (intent), **event**
(fact) and **query** (read), and forbids using an event as a disguised command.
The engine had events and no commands, which is precisely the pressure that
produces that mistake — `DEBT-0007`. It now has both
([ADR-0010](docs/adr/ADR-0010-commands-are-intent-and-carry-their-own-authority.md)).

`engine/command` holds the framework and depends on `engine/foundation` and
`engine/runtime` only. That is not tidiness: `Command System.md` §134 lists what
the command system must never contain — block rules, physics, worldgen,
inventory, crafting, economy — and none of those crates are reachable from it,
so the list is a build error rather than a comment. Block handlers live in
`engine/simulation`, mirroring §28's own split: **the handler adapts, the
specialized system decides.**

Three properties it was built to have, each with a test that fails if it stops
being true:

- **Deny by default.** A definition that forgot to say who may send it permits
  *nobody*, and is refused at registration rather than at the first attempt.
  Authority defaults to server-required; the rate limit defaults to finite.
- **No trusted bypass.** §72 — *"nunca assumir internal = always valid"*. A
  server-issued command runs every validation layer a player's does.
- **Priority orders, and only orders.** §32 — a `Critical` command goes first
  and cannot skip a check, because the queue has no path to the pipeline.

The vertical slice runs the reachable part of §115's first slice: intent →
validation → one authority → a changed world. Three of its five commands are
**refused on purpose** (nothing there, out of reach, wrong actor kind) — a
pipeline only ever shown accepting things has not been shown to refuse anything.
The stage places a block and breaks it again, so the save stays byte-identical
across worker counts and the determinism check keeps measuring determinism.

## Crash recovery

`NEXORA SAVE FORMAT AND COMPATIBILITY.md` defines recovery as
`Snapshot + Journal → Recovery`. The snapshot half was built in Phase 0; the
journal half is [ADR-0011](docs/adr/ADR-0011-a-torn-tail-is-a-crash-and-corruption-is-not.md),
and it turns on one observation:

> **A torn write truncates. It cannot produce a complete record whose contents
> are wrong.**

So a damaged journal has a knowable cause. Fewer bytes than the record claims
means the process died mid-append — expected, recover the prefix, carry on. A
*complete* record failing its checksum means storage returned different bytes
than were written — that is never filed as a routine crash, because
`NEXORA FAILURE AND RECOVERY ARCHITECTURE.md` says **"never hide a
data-integrity failure."**

Replay stops at the first damaged record either way and never resynchronises
past it. Salvaging more would mean guessing where the next frame starts, and a
wrong guess feeds garbage into a world *as if it were an edit*. Losing a
journal's tail is bounded; a world with invented edits is not.

A journal names the snapshot it continues from and refuses any other — the
records would otherwise apply cleanly and produce a world that never existed.
Edits are recorded by identifier rather than runtime id, so a block whose
content is gone is reported as Missing Content instead of becoming whatever
happens to hold that id today.

`engine/world/tests/crash_recovery.rs` covers the save document's mandatory
list: truncate a journal mid-record, flip a bit inside a complete one, replay
the same journal twice and compare the saves byte-for-byte, quarantine a wrecked
journal and show the snapshot still loads.

**What this does not yet do:** the running engine does not write to the journal
(`DEBT-0025`), and replay reaches only chunks that are resident (`DEBT-0024`).
Both are recorded with triggers rather than implied to be finished.

## Originality

NEXORA is an original project. It contains no code, assets, textures, models,
sounds or shaders taken from any other game. Conceptual influence is not
implementation: see `NEXORA ORIGINAL CONTENT AND ASSET POLICY.md`. The
algorithms implemented in `engine/foundation` (FNV-1a, CRC-32, SplitMix64) are
public specifications, implemented here from those specifications and pinned by
their published reference vectors.
