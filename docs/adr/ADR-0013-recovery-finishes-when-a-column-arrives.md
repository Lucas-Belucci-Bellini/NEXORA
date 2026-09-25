# ADR-0013 — Recovery is not a moment; it finishes when a column arrives

- **Status:** ACCEPTED
- **Date:** 2026-09-14
- **Closes:** `DEBT-0024` (replay reached only the resident set)
- **Follows:** [ADR-0008](ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md)
  (streaming decides residency; a backend provides it),
  [ADR-0011](ADR-0011-a-torn-tail-is-a-crash-and-corruption-is-not.md)
  (snapshot + journal → recovery)

## Context

ADR-0011 built recovery as one call: load the snapshot, replay the journal into
it, report what did not apply. That is correct as far as the world reaches, and
`World::set_block` reaches exactly as far as the resident set.

ADR-0008 made the resident set a moving thing. Streaming decides which columns
exist right now, and the plan is for interest to drive that rather than a fixed
radius. The two decisions meet badly: the set of columns resident at recovery is
not the set that was resident when the journal was written, and the difference
grows the moment loading stops being a fixed radius.

## Problem

An edit whose column is not resident at replay was reported as `NotWritable` and
left there. `DEBT-0024` measured the shape of it: *"um mundo que salvou com 25
colunas residentes e recarrega com 9 perde, no replay, as edições das 16 que
faltam."*

Nothing was corrupted and nothing lied. The record stayed in the journal, the
report listed it, and `is_complete()` said false. What was missing is that
**nobody ever went back for it.** The column arrived a tick later and arrived
empty.

## Options

1. **Load every column the journal mentions, then replay.** Correct, and it
   throws away what ADR-0008 exists for: a world too large to be resident. A
   journal that touched a thousand columns would force a thousand columns in.
2. **Replay lazily from the journal file each time a column loads.** No index to
   hold, but every activation re-reads and re-scans a file that only gets
   longer, and the journal is not indexed by column.
3. **Index the unapplied records by column at replay, and drain the index when
   the column becomes resident.** One pass over the journal, memory bounded by
   what could not be applied, and the drain happens where a column already
   enters.

## Decision

**Option 3. Recovery produces an index of what it could not place, and
residency finishes the job.**

`recovery::apply` files every record that failed *only* on residency against the
column it needs, and returns it in `RecoveryReport::deferred`.
`WorldResidency::recovering(&mut pending)` attaches that index to the streaming
backend; each column that becomes resident gets its records applied inside
`activate`, before the caller can read the column.

**Order survives because of what a position is.** `apply` insists on journal
order — an earlier edit written over a later one produces a world that never
existed. Deferring appears to break that and does not: two records that can
overwrite each other are at the same position, and the same position is in the
same column. Order is preserved *within* each column; between columns there is
nothing to preserve. A test writes twice to one position through the deferred
path and requires the second write to win.

**Only residency-blocked records are filed.** A block identifier this session
does not know will not become known by waiting; it is reported and not
enqueued, because enqueuing it means retrying it against every column that ever
loads, forever.

**A record still refused with its column resident is an error, not a discard.**
It was not waiting on residency. Swallowing it is how a world starts quietly
disagreeing with its own journal.

## Consequences

- **A partial load no longer costs the edits outside it.** The reload can bring
  in nine columns and still receive the sixteen columns' worth of edits, as each
  one arrives.
- **The report did not get less honest.** A deferred edit is still in `skipped`,
  and `is_complete()` is still false. The index *adds* the ability to finish;
  it does not replace the warning with silence.
- **A column nobody brings in keeps its edits unapplied**, and says how many:
  `PendingEdits::len`. That is the honest end state, and it is countable rather
  than silent.
- **The index is opt-in.** `WorldResidency::new` behaves exactly as before; a
  backend that does not attach one neither recovers nor complains. Every other
  `ResidencyBackend` implementation is unaffected.
- **`nexora_world::recovery` now re-exports `Replay` and `Damage`** (as
  `JournalReplay`/`JournalDamage`). Both appear in `apply`'s public signature,
  and without the re-export every caller had to depend on `nexora-persistence`
  for a type it only sees through this API.
- **The headless slice is unchanged**, deliberately: every chunk it edits is
  resident in its checkpoint, so it still requires zero skipped records and its
  save is byte-identical. The deferred path is proven by tests and by the
  streaming backend, not by loosening the slice's assertion.

## Migration

None. `recovery::apply` keeps its signature and its meaning; `RecoveryReport`
gains a field. Callers that ignore `deferred` behave exactly as they did.

## Compatibility

No save or journal format change. The index is built at replay from records that
are already in the file, and never written anywhere.
