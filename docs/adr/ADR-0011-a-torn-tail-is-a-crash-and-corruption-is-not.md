# ADR-0011 — A torn tail is a crash; corruption is not, and they get different answers

- **Status:** ACCEPTED
- **Date:** 2026-09-07
- **Closes:** `DEBT-0001` (save had a snapshot and no incremental journal)
- **Follows:** [ADR-0008](ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md)
  (eviction is a data-loss mechanism; so is a checkpoint interval)

## Context

`NEXORA SAVE FORMAT AND COMPATIBILITY.md` defines recovery in three lines:

```text
State Snapshot + Incremental Journal → Recovery
```

Phase 0 built the snapshot — atomic, versioned, checksummed, verified before
rename, quarantinable. `DEBT-0001` recorded the half that was missing and what
it costs: *"mudanças entre checkpoints são perdidas se o processo cair."*

`NEXORA DEVELOPMENT ROADMAP.md` names **recovery** in Phase 3's exit criteria,
and the same save document lists *"crash durante save"*, *"corrupção parcial"*
and *"recuperação de journal"* among its **testes obrigatórios**.

## Problem

A journal is only worth having if a crash **while writing to it** is survivable
— that is the ordinary case, not the exotic one. Which produces the question
this ADR exists to answer.

**A damaged journal has two very different causes, and they look identical at
first glance.** Both leave a record that will not parse:

- the process died mid-append, leaving a partial record at the end;
- storage returned different bytes than were written.

Treating them the same is how a recovery system fails in one of two ways. Treat
everything as corruption and every ordinary crash throws away a journal that was
almost entirely intact. Treat everything as a torn tail and silent bit-rot gets
recovered *as if it were data*, which `NEXORA FAILURE AND RECOVERY
ARCHITECTURE.md` forbids outright: **"never hide a data-integrity failure."**

**And a journal replayed onto the wrong snapshot corrupts a world silently**,
which is worse than losing the journal entirely.

## Options

For framing:

1. **One checksum over the whole journal.** Any damage invalidates everything,
   so every crash costs every record.
2. **Per-record length and checksum.** Damage is localised to one record.

For a record that fails:

3. **Fail the whole replay.** Loses intact records for one bad byte.
4. **Skip the record and resynchronise.** Salvages the most, and requires
   *guessing* where the next frame starts.
5. **Stop at the first bad record, keep the prefix, report what stopped it.**

## Decision

**Option 2 and option 5**, with the cause reported as a distinguishable kind.

### The rule that separates a crash from corruption

> **A torn write truncates. It cannot produce a complete record whose contents
> are wrong.**

So the reader can tell them apart from the shape of the failure alone:

| what the reader finds | what it means | what it does |
| --- | --- | --- |
| fewer bytes than the record claims | died mid-append | recover the prefix; **expected** |
| a length beyond the cap | the frame header itself is unwritten or garbage | recover the prefix; **expected** |
| full payload, wrong checksum | storage returned other bytes | recover the prefix; **loud** |

`Damage::TornTail` carries `is_expected_after_a_crash() == true`;
`Damage::Corruption` carries `false` and names the offset. A caller can restart
quietly after the first and must not after the second.

### Replay stops at damage and never resynchronises

Option 4 salvages more records and is rejected anyway. Skipping ahead means
guessing where the next frame begins, and a wrong guess feeds garbage — or
attacker-chosen bytes — into a world *as if they were edits*. The same document
is explicit: **"prefer failing a bounded operation over corrupting global
state."** Losing the tail of a journal is bounded. A world containing invented
edits is not.

### A journal names the snapshot it continues from

The header carries the world id, the snapshot's length and its CRC-32. Replay
against any other snapshot is refused before a single record is read, and so is
reopening for append. This is the trap that would otherwise be invisible: the
records would apply *cleanly* and produce a world that never existed. There is a
test that checkpoints twice and asserts the stale journal is refused against the
new snapshot.

### The length field is bounded, because it is attacker-chosen

`MAX_RECORD_BYTES` is 16 MiB. A length field is the first thing a corrupt or
hostile file gets to choose, and an unbounded one is an allocation someone else
picks — the startup brief §29 lists save files among the inputs that must be
treated as untrusted. A length past the cap is treated as an unwritten frame
header rather than as an instruction.

### Persistence frames records; the world decides what they mean

`nexora-persistence` moves opaque, checksummed byte records, exactly as it moves
opaque named sections. `nexora-world::recovery` defines `EditRecord` and
replays it. The same split the crate already had, kept for the same reason.

**Edits are recorded by identifier, not by runtime id.** A `BlockStateId` is
assigned when a registry is built and a journal outlives the process that wrote
it; recording `17` and replaying it where `17` means something else places the
wrong block *silently*. Records carry `nexora:block/stone`, and a block whose
identifier is no longer registered is reported as Missing Content rather than
guessed.

### Durability is the caller's decision, and it is explicit

`append` frames and writes; `sync` makes it durable; `append_durable` does both.
A caller that has not synced must not report the operation as committed, and
`unsynced()` says how many records are at risk. The framing is what makes the
fast path defensible: losing the unsynced tail is recoverable by construction.

## Consequences

- **`DEBT-0001` closes.** Changes between checkpoints survive a crash.
- **The mandatory tests exist**: `engine/world/tests/crash_recovery.rs` truncates
  a journal mid-record to simulate the crash, flips a bit inside a complete
  record to simulate bit-rot, replays the same journal twice and compares the
  resulting saves byte-for-byte, quarantines a wrecked journal and shows the
  snapshot still loads, and closes the loop — recover, re-checkpoint, journal
  onward, and refuse the stale journal.
- **Recovery is only as complete as the resident set.** `set_block` requires a
  resident chunk, so an edit replayed into a chunk this session has not loaded
  is reported as `NotWritable` rather than applied. That is honest, and it is
  not sufficient: `DEBT-0024` records it with the trigger.
- **Nothing writes to a journal yet in the running engine.** The mechanism, its
  format and its recovery path are built and tested; wiring it into the world's
  own write path is a separate change, because doing both at once would have
  meant debugging a format and a call site together. `DEBT-0025`.
- **Two journals now exist with similar names.** `Chunk`'s in-memory ring is a
  bounded diagnostic aid that drops its oldest entry under pressure
  (`DEBT-0004`); the save journal is durable and complete. The module
  documentation says so in both places, because trusting the first for recovery
  would be a quiet disaster.

## Migration

None. A world with no journal file recovers exactly as before — from its
snapshot. The journal is additive, and its absence is not an error.

## Compatibility

The journal is a **new file alongside the save**, not a change to the container,
so no existing save is affected and no save-format version moves. The journal
carries its own `FORMAT_VERSION`, and a version this build does not know is
refused rather than guessed at.
