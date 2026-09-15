# ADR-0014 — A region file is authoritative for its region; the resident set is not a delete list

- **Status:** ACCEPTED
- **Date:** 2026-09-15
- **Shrinks:** `DEBT-0002` (the whole world in one save section)
- **Follows:** [ADR-0004](ADR-0004-save-container-format-v1.md) (the save
  container), [ADR-0008](ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md)
  (streaming decides residency),
  [ADR-0011](ADR-0011-a-torn-tail-is-a-crash-and-corruption-is-not.md)
  (snapshot + journal → recovery)

## Context

`world::persist::save` writes every resident chunk into one section,
`nexora:save/chunks`, of one container. That was the right shape while the save
format was still being established: one file, inspectable end to end, and one
checksum boundary to reason about.

`CHUNK & VOXEL ENGINE.md` §27 (CHUNK-25) asks for the other shape — chunks
grouped into regions — and `DEBT-0002` records why: *"Salvar reescreve o mundo
inteiro, mesmo com um bloco sujo. O rastreamento de sujeira por seção já existe
e ainda não é aproveitado."* `Chunk::dirty_sections` has been tracked since the
chunk lifecycle was built, and nothing has ever read it at save time.

## Problem

A save costs the whole world however little of it moved: encode every column,
deflate every byte, write the file, read it back and decode it to verify
(ADR-0004's guarantee, quantified as `DEBT-0006`). Measured on the benchmark
world, that is **12.8–13.2 ms** and **40.8 KiB** for a world in which one block
changed.

Splitting the single section into per-region sections *inside* the same
container does not fix it. The container encodes as a unit: every section is
deflated on every `encode`, and the file is written and verified whole. The cost
follows the file, so the regions have to be files.

## Options

1. **Per-region sections in one container.** Small change, no new file layout,
   and it buys nothing today: the whole container still deflates and still gets
   written and verified on every save. It would be a structure with the
   benefit deferred to a later change — preparing, dressed as implementing.
2. **A new region file format.** Full control of the layout, and it throws away
   framing, checksums, versioning, atomic write with read-back verification and
   quarantine, all of which exist and are tested.
3. **A directory of ordinary containers, one per region.** A region file *is* a
   `SaveContainer`; the store is the arrangement. A clean region is not encoded,
   not deflated, not written and not read back, because its file is never
   opened.

## Decision

**Option 3.** `nexora_world::region::RegionStore` is a directory:

```text
<root>/world.nxsv               world header, and how columns were grouped
<root>/region/r.0.0.nxsv        one region: its palette and its columns
<root>/region/r.-1.0.nxsv
```

Three decisions make it safe rather than merely smaller.

### A region file is authoritative for its region; the resident set is not a delete list

Writing never removes a region file for a region the world does not currently
hold. This is the one place the store's semantics differ from the single
container, and it is the point of the change: the container is a snapshot of
what is resident, while a store is the world, most of which is not resident
(ADR-0008). A save that deleted what it had not loaded would turn every eviction
into data loss the moment streaming outgrows a fixed radius.

The consequence is that the two are not interchangeable. `persist::save` still
means "everything resident, in one file"; `RegionStore::write_all` means
"everything resident, filed by region, leaving the rest of the world alone".

### Each region file carries its own identifier palette

`Registry System.md` §12 requires a save to store namespaced identifiers rather
than runtime ids. A store could hold one palette beside the header and have
every region index it — until a region written under an older palette is skipped
while the palette grows, and its indices quietly mean different blocks.

Writing the palette per region removes the coupling instead of guarding it: a
region file resolves its own identifiers and is readable with nothing else
present. That is also exactly what a streaming `activate` needs, and what
`DEBT-0020` will need to read an evicted column back from disk. It costs the
palette once per region, which deflates well and is measured below.

### The header is written last

A save is several files, so a crash can land between them. Written in this
order, the surviving state is regions that are ahead of the clock in the header,
and ADR-0011's replay carries the clock forward over blocks that already hold
their final value — `SetBlock` names an absolute state, so re-applying one is a
no-op. Written the other way round, the clock would be ahead of the regions and
the journal records that would repair them are the ones already considered
applied. One ordering is recoverable and the other is not.

A store opened with a region extent different from the one recorded in its
header is refused rather than written into, because grouping columns a second
way files them under addresses that already hold different columns, and
`write_dirty` would skip straight past the mismatch.

## Consequences

Measured on the benchmark world (3×3 columns, two columns per region, so four
regions; at the engine default of 32 a world this size is one region and there
would be nothing to skip):

| | one container | region store |
| --- | ---: | ---: |
| save after one block changed | 12.8–13.2 ms | **2.9 ms** |
| bytes written for that save | 40.8 KiB | **4.7 KiB** |
| save of the whole world | 12.8–13.2 ms | 14.9 ms |
| bytes for the whole world | 40.8 KiB | 41.7 KiB |

- **A one-block save costs about 4.5× less time and 8.7× fewer bytes.** The
  count behind it does not depend on the machine: `save.regions_written_per_edit`
  is **1** of 4, on any box and in any build.
- **A full write costs 13–17% more**, and the size grows 2.2%. Five files
  instead of one means five sets of framing, five `fsync`s and five read-back
  verifications, and five deflate streams that cannot share a dictionary; the
  palette is also written once per region. A full write is the case this
  arrangement is *not* for.
- **`persist::save` and `persist::load` are unchanged**, and so is the container
  format. A store is built out of them, not instead of them: region files reuse
  the same section names and the same encoders, so the two layouts cannot drift
  into two formats.
- **A crash between the regions and the header is recoverable**, and a test
  constructs exactly that state: the blocks that reached their region files are
  there, and the clock is the older one.
- **`World::mark_saved` exists and `World::chunk_mut` cannot stand in for it.**
  Clearing a dirty set is not a change to any cell's answer, so it deliberately
  does not move `World::revision`; bumping it would invalidate every body's
  clear-span proof (ADR-0007, `DEBT-0038`) on every save.

## Migration

None is required, and none is automatic. No existing save is a store, no code
path writes one unless it asks for a `RegionStore`, and `SliceConfig::save_path`
still names a single container. Converting a container to a store is a `load`
followed by a `write_all`; the reverse is a `read` followed by a `save`.

## Compatibility

No change to the save container format, its version, or any section already
written. A store adds one section, `nexora:save/region_shape`, and adds it only
to the store's own header file — which region a column belongs to is a property
of this layout, not of the world.
