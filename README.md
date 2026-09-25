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

There *is* an **RHI**, as a contract
([ADR-0025](docs/adr/ADR-0025-the-rhi-is-a-contract-a-null-backend-keeps-before-a-gpu-does.md)):
handles that die with their resource or their device, all-or-nothing
submission, ordered fences, destruction deferred until the GPU is done, device
loss and recreation. There is also a **null backend** that keeps all of those
rules without a GPU, and a **conformance suite** every backend must pass. The
slice runs that suite on every start and uploads the first generation's
textures through it. The first **native** backend is `wgpu`
([ADR-0026](docs/adr/ADR-0026-the-first-native-backend-is-wgpu-and-ci-runs-it.md)).
It passes the same suite, uploads a texture and draws a triangle, and reads
both back from the device. CI runs it on Mesa's software Vulkan. There is
still no window, so nothing is presented.

There *is* a frame, as of ENGINE-0
([ADR-0017](docs/adr/ADR-0017-a-frame-is-time-the-host-hands-in.md)): a fixed
timestep, a cap on catching up that says how many steps it threw away, and
per-stage attribution whose point is the time **no** stage claimed. A frame does
not need a renderer; it needs somewhere for the engine's own costs to be
compared against a budget, which until now existed only in prose. Nothing in the
repository yet drives it against a real clock — `DEBT-0041`.

There is also **input**, as of ENGINE-8
([ADR-0018](docs/adr/ADR-0018-input-is-intent-the-host-hands-in.md)): devices,
actions, mapping contexts that consume a control rather than share it, chords,
dead zones and curves, a remap that survives being written out, and a validator
for intent that arrived from outside the trust boundary. It produces intent and
nothing else — the command system already owns what intent is allowed to do. The
slice's walk is driven through it: the route used to be a list of stops and is
now what the input system says the player asked for. As with the frame, nothing
here has met a real device — `DEBT-0043`.

The implementation language is **not locked**. Rust is the reference
implementation for the benchmark gate defined in
`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17 — see
[ADR-0001](docs/adr/ADR-0001-rust-phase-0-reference-implementation.md).

## Running it

Requires the toolchain pinned in `rust-toolchain.toml`; `rustup` installs it
automatically.

```bash
cargo test --workspace          # 1138 tests
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
save size          116904 bytes
region store       9 regions, 119482 bytes; one edit rewrote 1 for 18718 bytes
physics bodies     9 (9 settled)
physics substeps   600 (270 contacts)
character drop     1010 cm
streaming ticks    29 (95 generated, 120 evicted, 25 restored)
chunks retained    15 (peak, edits that cannot be regenerated)
retention spill    25 columns to 12 region files, 25 read back
probes verified    76
result             OK
```

Fifty million blocks held in 722 KB is the palette and uniform-section storage
doing its job: solid rock costs nothing to hold. The observer walks away from
the edited region and back before the save, so those 76 verified probes are also
proof that streaming evicted 120 columns without losing an edit.

The region-store line is the same world written the other way round, one file
per region ([ADR-0014](docs/adr/ADR-0014-a-region-file-is-authoritative-for-its-region.md)):
changing one block afterwards rewrote one of the nine files instead of all of
them. The spill line is the other use of the same store: an evicted column that
cannot be regenerated goes to its region file at the end of the tick that
evicted it, so the walk's peak retention is one tick's evictions rather than
every edit ever made — and the save comes out byte-identical either way.

```bash
cargo run -p nexora-headless -- --help      # seed, radius, threads, save path
cargo run -p nexora-headless -- --content content/first-generation/blocks.json
```

The second run adds the first visual generation — the sixteen 16×16 stones of
[`content/first-generation/`](content/first-generation/CATALOG.md) — as blocks,
through the same content path a mod would use. Each one must show its own
surface in a mesh and survive the save, the reload and the journal replay, or
the run fails: `content blocks 16 (16 surfaces in the mesh)`, `probes verified 92`.

### Prebuilt binaries

[`web/`](web) is a download page that reads the repository's releases at load
time, and [`docs/RELEASING.md`](docs/RELEASING.md) describes the pipeline
behind it: a version tag builds on Linux, Windows and macOS, **runs the
binaries it is about to package on each platform**, and publishes the archives
with a `SHA256SUMS` covering them. No release has been cut yet, so building
from source is currently the only way to get it — which is the three commands
above and no dependencies.

### On your own machine: the local validation report

The development container has no GPU and no display, so evidence from a real
machine enters the repository as a report
([`docs/validation/local/`](docs/validation/local/README.md)). With Rust
(`rustup`), Git and Python 3 installed — and on Windows the Visual Studio C++
Build Tools, which Rust needs to link (the script's preflight says so, and how
to install them, before it builds anything):

```bash
python3 scripts/local-validation.py run        # Windows: py scripts\local-validation.py run
python3 scripts/local-validation.py check      # is the report still about HEAD?
```

It builds, tests, runs the slice with and without the first-generation
content, builds the 16×16 set and decodes it, runs the CPU benchmark, and
writes `docs/validation/local/NEXORA-LOCAL-VALIDATION.{json,md}` — without any
hostname, user name, serial or absolute path. Commit those two files.

## Measuring it

The language selection gate needs evidence, so the reference implementation is
measured rather than asserted:

```bash
cargo run --release -p nexora-benchmark      # the Rust reference
scripts/compare-stacks.sh                    # Rust vs C++, every compiler found
scripts/build-spread.sh                      # how much a row moves for no reason
```

The second script needs a C++20 compiler; nothing else in the repository does.
It checks conformance before it times anything, and **fails without producing a
single number** if the stacks disagree on any digest.

The third builds the same source three times and reports how far each row
drifted between builds. That is the floor a claimed improvement has to clear,
and it is not where it was assumed to be: the two-nanosecond arithmetic kernels
are the steadiest rows in the suite, and the noisiest are `fsync`, disk and
thread-scheduling rows three orders of magnitude larger (finding 24,
`DEBT-0039`).

Results and analysis:
[`docs/benchmarks/PHASE-0-BASELINE.md`](docs/benchmarks/PHASE-0-BASELINE.md).
Highlights — 18.3 million blocks held in 144 KiB, 7.0 MiB peak resident memory,
a measurement that closed `DEBT-0005` by showing the "optimization" it proposed
would have been **4× slower** than the code it was meant to improve, the number
behind `DEBT-0009` (scheduling one job per entity costs **~4,900× the simulation
it schedules**), and the one behind `DEBT-0011`: **49% of a physics step is the
voxel lookup, not the solver** — measured by running the same 1,000 bodies
against generated terrain and against a flat fixture, because a single combined
number would have sent the optimisation work to the wrong half. That one has
since been worked twice, and each pass corrected its own diagnosis. Keeping the
last resolved section took the lookup from **49% to 38.6%** of a step (a
40-metre raycast, almost nothing but lookup, got **40% faster**) and found two
ordered-map descents on that path rather than one, together a third of the cost
rather than the bulk of it. Removing the runtime division from the palette read
took `voxel.get_paletted` **13.1 ns → 11.4 ns**, and found that two integer
divisions were worth 1.7 ns rather than most of the read.

The ruler itself had to be rebuilt in between. The terrain-against-flat pair
stopped resolving anything once the depenetration scan was gone, so the lookup's
share is now a **product of a count and a microbenchmark** — 1,000 cell
questions per step, which is the same integer on every machine, times the cost
of one — rather than the difference between two 140 µs numbers.

## Layout

The crate graph *is* the dependency matrix — Cargo rejects cycles, so a layering
violation fails the build rather than a review
([ADR-0003](docs/adr/ADR-0003-workspace-layout-enforces-dependency-matrix.md)).

```text
engine/foundation    errors, versions, identifiers, space, time, determinism,
                     diagnostics, configuration, memory budgets (ADR-0024)
                                                         (no dependencies at all)
engine/persistence   versioned, checksummed, atomic save container
engine/runtime       lifecycle, engine modules, registries, event bus, jobs,
                     the frame loop and its budget, input and its bindings
engine/world         voxel storage, chunks, world lifecycle, generation
engine/entity        identity, lifecycle, components, queries, persistence
engine/physics       fixed timestep, bodies, swept voxel collision, characters,
                     ray queries         (depends on foundation and nothing else)
engine/streaming     interest, priority, budgets, LOD tiers, eviction
                     (also depends on foundation and nothing else)
engine/simulation    the one crate allowed to see the world, physics and
                     streaming at the same time
engine/command       intent: definitions, validation, dispatch, quotas
engine/query         reads as a contract: versioned, permitted, bounded (ADR-0023)
engine/asset         surface materials, texture maps, provenance, validation,
                     the generator and pipeline contracts
engine/resource      resource manifest, integrity-checked loading, bounded
                     cache and typed handles (ADR-0021)
engine/mesh          greedy meshing into a data structure (ADR-0012)
engine/rhi           the render hardware interface: the contract, a null
                     backend and the conformance suite (ADR-0025)
engine/rhi-wgpu      the first native backend: wgpu over Vulkan, Direct3D 12
                     and Metal -- the engine's one external dependency
                     (ADR-0026); `nexora-rhi-probe` proves it on a machine
engine/image         the one PNG decoder and the texture loader (ADR-0022)
tools/texture-forge  the material generator -- a content tool, not an engine
                     crate, so it lives outside engine/: recipes, generation,
                     batch manifests, build plans, INDEX
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
| [`TEXTURE FORGE.md`](TEXTURE%20FORGE.md) | how materials and textures are made |
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
when used per entity, spatial entity queries scanned the whole population
(100,000 examined to return six — since fixed, `DEBT-0010` and ADR-0015), the
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

**The engine journals its own writes**, and the policy was measured before it
was chosen ([Appendix E](docs/benchmarks/PHASE-0-BASELINE.md)):

| | cost | per edit |
| --- | ---: | ---: |
| frame and checksum one edit | 672 ns | 672 ns |
| make one edit durable (fsync) | 203 µs | **203 µs** |
| 64 edits sharing one fsync | 293 µs | **4.6 µs** |

An fsync is **303× an append**, and almost all of it is fixed cost. The slice
makes 78 writes: per-edit durability would cost **15.8 ms**, most of a 60 Hz
frame, against **203 µs** for one flush at a commit boundary. So `set_block`
records every write, `sync_journal` commits, and `unsynced_edits` says exactly
what a crash would cost right now.

The journal lives *on* the world rather than wrapped around it, because a
durability mechanism a caller can forget to use will be forgotten. The proof is
that the command stage became journalled without its handlers knowing the
journal exists — the slice reports `78 edits` for 76 block edits plus the
command stage's 2 writes, then rebuilds itself from checkpoint + journal and
re-checks all 76 probes.

**What this still does not do:** replay reaches only chunks that are resident
(`DEBT-0024`), recorded with a trigger rather than implied to be finished.

## Originality

NEXORA is an original project. It contains no code, assets, textures, models,
sounds or shaders taken from any other game. Conceptual influence is not
implementation: see `NEXORA ORIGINAL CONTENT AND ASSET POLICY.md`. The
algorithms implemented in `engine/foundation` (FNV-1a, CRC-32, SplitMix64) are
public specifications, implemented here from those specifications and pinned by
their published reference vectors.
