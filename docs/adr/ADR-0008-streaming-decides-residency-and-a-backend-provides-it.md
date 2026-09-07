# ADR-0008 — Streaming decides residency; a backend provides it

- **Status:** ACCEPTED
- **Date:** 2026-09-07
- **Amends:** [ADR-0005](ADR-0005-phase-0-scope-boundary.md) (streaming moves
  from "deliberately not implemented" to implemented)
- **Follows:** [ADR-0007](ADR-0007-physics-collides-against-a-provider-not-the-world.md)
  (same boundary shape, same reason)

## Context

`STREAMING SYSTEM.md` opens by saying what streaming is *not*:

> Streaming controls which world, entity, resource and simulation data is
> resident. **It does not own generation or gameplay rules.**

It then requires interest sources, priority, per-domain budgets with
backpressure and **hysteresis to prevent thrashing**, and the ladder
`FULL → REGIONAL → ABSTRACT → UNRESIDENT`.
`WORLD CONTINUITY AND PLAYER INDEPENDENCE.md` §18 adds the invariant that
outranks all of it: **logical identity and persistent state survive eviction.**

`NEXORA TECHNOLOGY BENCHMARK PLAN.md` lists `streaming` as a stage of the
vertical slice the language gate measures.

## Problem

Two problems, and the second is the dangerous one.

**Layering.** Streaming needs to load, generate and drop chunks. Depending on
`nexora-world` directly would make the manager able to generate — the one thing
the document says it must not own — and would make every streaming test need a
seeded world.

**Eviction is a data-loss mechanism wearing a performance costume.** Its whole
job is to delete resident state. Every decision about *what* may be deleted is
a decision about what the player is allowed to lose.

## Options

For the boundary:

1. **`nexora-streaming` depends on `nexora-world`.** Direct, and it hands the
   manager the ability the document forbids it.
2. **A trait in streaming, implemented above both.** The manager decides; a
   backend acts.

For eviction:

3. **Retain every evicted chunk.** Never loses anything; memory grows with
   explored area, which is unbounded.
4. **Regenerate every evicted chunk.** Bounded memory; loses every edit.
5. **Retain only what cannot be regenerated.**

## Decision

**Option 2 and option 5.**

`nexora-streaming` declares `trait ResidencyBackend` and depends on
`nexora-foundation` **only**. `nexora-simulation` — already the crate allowed to
see both the world and physics — gains `WorldResidency`, the chunk backend.
Cargo enforces it: the manager cannot generate a chunk because the dependency
is not there.

### Only edited chunks are retained

Generation is deterministic and position-seeded — `nexora-world` proves that the
same seed produces the same column regardless of generation order. So:

| chunk | on eviction | on return |
| --- | --- | --- |
| unedited | dropped | regenerated from the seed |
| edited | moved to `RetainedChunks` | restored |

Measured (Appendix C, finding 15): restoring costs **338.9 ns**, regenerating
**2.72 ms** — retention is ~8 000× cheaper on the return path, at **20.0 KiB**
per held column. Retaining everything would have been faster still and would
have made memory a function of how far the world had been walked. This makes it
a function of **how much the world has been changed**, which is the quantity
that should bound it.

**Once edited, always retained.** A restored chunk looks clean again, so
dirtiness alone is not enough: `RetainedChunks` remembers every column that has
*ever* been edited. Without that, the second eviction drops the chunk and the
third activation regenerates terrain without the edit. There is a test that runs
four cycles.

### Eviction never loses an edit, even when saving fails

* The manager persists a dirty target **before** dropping it.
* If that write fails it **does not evict**. Holding memory is recoverable;
  losing an edit is not. A test supplies a backend whose `persist` always fails
  and asserts the chunk is still resident.
* `ResidencyBackend::is_dirty` defaults to **`true`**. A backend that has not
  thought about dirtiness gets the safe answer rather than the fast one.

### The trap this created, and the test that guards it

An evicted chunk is not in the world, and `nexora_world::persist::save` writes
*the world*. Saving with chunks retained therefore produces a save missing every
edit in them — silently, because nothing is broken from the world's side.

`RetainedChunks::flush_into` must be called before any save. There is a test
that saves *without* it and asserts the edit is gone, so the trap is documented
by something that fails if the situation ever changes.

### Hysteresis is a band, not a threshold

A target is promoted at the plain radius and demoted only past
`radius + hysteresis`. Without the band, an observer standing on a boundary
loads and evicts the same column on alternate ticks forever, paying a
generation or a disk read each time.

Demotion lands on the tier the plain radii give rather than stepping down one
tier per tick: `STREAMING SYSTEM.md` lists fast travel as a test case, and
stepping would hold a region resident for three more ticks after the observer
has gone somewhere else.

### Two kinds of budget pressure, reported separately

* **Deferred** — the per-tick budget ran out. The work is still queued, and
  happens next tick, nearest-first.
* **Shed** — memory or count pressure evicted something interest actively
  wanted. Non-zero `shed` means the budget is smaller than what the observer is
  asking for.

Collapsing them into one counter would hide the difference between "streaming is
pacing itself" and "streaming cannot give you what you asked for".

## Consequences

- Every streaming test runs against `MemoryBackend` in a few lines, with no
  world, no seed and no generation. The world-backed tests then check only what
  is specific to chunks.
- The benchmark measures the **decision** and the **residency** separately, and
  the split is where the useful numbers came from: generation dominates the
  decision by **578×** (finding 18), so a streaming budget is sizing generation
  time, not manager overhead — and the slice's own budget of 8 activations per
  tick would cost 21.8 ms if it ever spent all of it. `DEBT-0018` moves
  activation onto the job system.
- The idle tick has a floor that grows faster than the interest area:
  **167 µs at radius 12, on every tick, whether or not anything moved**
  (finding 16). `DEBT-0017` records the incremental candidate set that fixes it,
  with the radius that triggers the work.
- `Regional` and `Abstract` are real states of the manager but hold no distinct
  data: no backend can tell them apart until the regional simulation exists
  (Phase 4). They are tracked, keep identity, and are not resident.
  `DEBT-0019`.
- Retained chunks live in memory, not in region files on disk. `RegionCoord` and
  `RegionShape` already exist in the foundation for exactly that; `DEBT-0020`.

## Migration

None. Nothing streamed before. The headless slice's existing probe verification
now also proves that streaming did not lose an edit, because the slice walks an
observer away from the edited region and back before saving — and the
determinism check across thread counts still produces byte-identical saves.

## Compatibility

No save format change: streaming state is not persisted. Which chunks are
resident is a property of a session, not of a world, and reconstructing it from
interest on load is both cheaper and more correct than restoring a resident set
that was captured around an observer who is no longer there.
