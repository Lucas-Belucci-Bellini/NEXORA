# ADR-0024 — Memory is accounted by its owner, in one ledger

- **Status:** ACCEPTED
- **Date:** 2026-09-24
- **Completes (partly):** `NEXORA MEMORY AND RESOURCE OWNERSHIP.md` — *budgets
  por subsistema* and *Debug* (counters, high-water marks, suspected leaks,
  budget pressure); `NEXORA PERFORMANCE BUDGETS.md` — the `MEMORY` dimension
  with `TARGET / WARNING / CRITICAL / EMERGENCY`
- **Touches:** foundation (a new module), `engine/resource`, `nexora-headless`, CI

## Context

The freeze checklist listed *Resource ownership* as partial: the asset cache
had a byte budget (ADR-0021), but *"per-subsystem budgets for the other memory
classes are not"* built. Three owners already measured their own memory and
nobody else could see it:

- the world (`World::storage_bytes`), whose chunks streaming makes resident;
- the chunks held outside the world after an edit (`RetainedChunks`, DEBT-0020);
- the resource cache (`CacheStats::bytes`, `high_water`).

Each number lived in its owner, with no thresholds, no record of the worst
moment, and no way to ask "is anything holding memory it should have given
back?". `NEXORA PERFORMANCE BUDGETS.md`: *"a subsystem must not consume
another subsystem's budget invisibly."* Invisible is the default when each
count lives in a different struct.

## Decision

`nexora_foundation::memory` (no dependencies, like the rest of the foundation):

- **`MemoryClass`** — the eight classes of the ownership document, by name.
- **`MemoryBudget`** — the four thresholds, validated
  (`target <= warning <= critical <= emergency`, non-zero ceiling).
  `MemoryBudget::capacity(n)` is the budget of something whose job is to be
  full: all four at `n`, so a full cache is nominal and an over-full one is an
  emergency.
- **`Pressure`** — the highest threshold usage is *above*. `Emergency` means the
  ceiling was broken; it is always a defect, never a state to operate in.
- **`MemoryPool`** — an **account, not an allocator**. The owner measures what
  it holds and `record`s the total; the pool keeps current bytes, high-water
  mark, worst pressure, record count and refusals, in atomics, so it is shared
  as `Arc` and recorded from any thread. Recording totals rather than deltas
  means an account cannot drift.
- **`MemoryLedger`** — every pool by unique name. A second owner for a name is
  refused: two owners overwriting one total is exactly the invisible sharing
  the ledger exists to stop. `suspected_leaks()` names the pools that must be
  empty at rest (class `Frame`, or declared `drains_at_rest`) and are not.

### One owner per byte

Chunk memory is owned by the **world**, although **streaming** decides which
chunks are resident. Streaming therefore enforces the world pool's ceiling
(`StreamingBudget::max_bytes = world pool's EMERGENCY`) and opens no pool of
its own. Two pools counting the same bytes would make every total a lie.

### Integration

- `ResourceCache::attach(pool)` — the cache records after every insert, eviction
  and removal, counts every refusal in the pool, and records zero when dropped
  (values still held as `Arc` elsewhere are their holders' memory now). A pool
  whose ceiling is below the cache's capacity is refused: the cache would be
  over the ledger's budget while doing exactly what it was told.
- `nexora-headless` opens `world.chunks`, `world.retained` (drains at rest: it
  must be empty before the save) and, with `--resources`, `resource.textures`
  (drains at rest: the stage drops its cache). It records at every stage, on
  every streaming tick, and at the end, at rest, refuses the run on a suspected
  leak, a broken ceiling, or a pool opened and never recorded into. The report
  prints each pool; CI requires `worst nominal, 0 suspected leaks`.

### Thresholds, and why memory may be gated when time may not

DEBT-0013 keeps physics timings ungated because a shared runner's clock is
noise. A pool's bytes are not: they are a property of the data structures, and
come out identical on any machine. Measured, a generated and edited column
costs **28.7–32.1 KiB** at every radius from 0 to 6 and every seed tried
(1, 42, 999999, the default). The slice's per-column budget:

| threshold | per column | why |
| --- | ---: | --- |
| TARGET | 40 KiB | the worst measured column with a quarter to spare |
| WARNING | 48 KiB | |
| CRITICAL | 64 KiB | twice the worst measured column |
| EMERGENCY | 128 KiB | one 32×32×32 section stored without a palette: past this, the storage model failed, not the budget |

`world.retained` uses the same figures, because the slice edits every column
and so every one of them may be held outside the world. Since region-file
retention (DEBT-0020, merged from `main` on 2026-09-25) a held column spills
at the end of the tick that evicted it, so the pool is recorded twice per
tick -- after eviction, and after the spill. Measured at radius 2: a peak of
435,024 B (fifteen columns inside one tick) against a ceiling of 3,276,800,
and zero at the end of every tick. Before the spill existed the peak was the
whole edited area, 722,304 B.

These are the **slice's** budgets, for the slice's scale. Engine-wide budgets
for a shipped game are still DEBT-0013's, at Phase 4.

## Verified

- `nexora-foundation`: 10 tests (threshold order, pressure boundaries, the
  capacity budget, high-water and worst pressure surviving a fall, admission
  without overflow, name uniqueness, leak detection by class and declaration,
  per-class totals, recording from eight threads).
- `nexora-resource`: an attached cache's every change reaches the pool, its
  refusal is counted, a too-small pool is refused, and dropping it records zero.
- `nexora-headless`: the slice's pools stay nominal and drain; `verify` refuses a
  broken ceiling, a leak and an unused pool, each shown failing on its own.
- The save is **byte-identical** to the previous build at radius 2, radius 3 and
  with the first-generation content and textures: streaming's new finite
  `max_bytes` does not bind at the slice's scale.

## Alternatives

- **A counting global allocator.** Exact, and blind: it knows how many bytes,
  not whose. It also puts an atomic on every allocation in every hot path to
  learn totals the owners already compute.
- **Deltas (`grow` / `shrink`).** One missed `shrink` and the account drifts
  forever. A recorded total corrects itself on the next record.
- **Pools that refuse allocations.** That is an allocator. `admits(extra)` is
  offered as advice; the owner holds the memory and decides.

## Consequences

- Every new large owner opens a pool; the checklist row stays partial until the
  classes without an owner yet (`Frame`, `Gpu`, `Network`, `Script`, `Editor`)
  have one — none of those subsystems exists.
- The entity store and the mesher do not record yet: the slice's physics and
  mesh stages build and drop their data inside one function, so there is no
  resident memory to account. They open pools when they hold memory across
  stages.
- Diagnostics tooling can read `MemoryLedger::report()` without knowing any
  owner's types.
