# ADR-0004 — Save container format v1

- **Status:** ACCEPTED
- **Date:** 2026-09-06

## Context

`NEXORA SAVE FORMAT AND COMPATIBILITY.md` requires versioning, atomic writes,
corruption detection, quarantine over overwrite, and explicit migration.
`Registry System.md` §11–§12 establishes that runtime ids are session-local and
that persistence stores namespaced identifiers.
`NEXORA SECURITY THREAT MODEL.md` classifies save files as untrusted input.

## Problem

A save format decided casually becomes permanent the first time a player creates
a world. Every property above has to be present in version 1, because retrofitting
integrity or identifier indirection means abandoning existing worlds.

## Decision

Format version 1 is:

```text
"NXSV"                      magic
save_format_version         u32
engine major / minor / patch u16 x3
world_schema_version        u32
registry_version            u32
content_version             u32
section_count               u32
header_crc32                u32   over every byte above
  repeated section_count times:
    name                    length-prefixed UTF-8 Identifier
    section_crc32           u32   over name bytes ++ payload
    payload                 length-prefixed bytes
"NXEN"                      trailer
file_crc32                  u32   over every byte above
```

Five properties are load-bearing:

1. **Identifiers, never runtime ids.** Chunk data indexes a palette of
   `namespace:path` strings. Loading resolves each against the current registry
   and remaps. A save naming absent content fails with that name, rather than
   loading a world whose stone has become air.

2. **The section checksum covers the name.** A checksum over the payload alone
   leaves the name unprotected; a single flipped bit then renames a section into
   another still-valid identifier and the file decodes as though nothing
   happened. This was found by an exhaustive single-bit corruption test during
   implementation, and the test is retained as a regression guard.

3. **A whole-file checksum, verified before any field is interpreted.** Nothing
   damaged is ever acted upon — not a length, not a version, not a section
   count.

4. **Atomic write.** Write to `<path>.tmp`, `sync_all`, read back, decode, and
   only then rename over the original. A save that does not verify leaves the
   previous one untouched.

5. **Quarantine, not overwrite.** A file that fails verification is renamed
   aside with an incrementing suffix. Earlier quarantines are never replaced.

Every declared length is bounded, so a corrupt or hostile file cannot turn a
four-byte edit into a large allocation.

## Options considered

- **A serialization crate's derived format.** Rejected: see ADR-0002; the layout
  would belong to the crate.
- **Payload-only checksums.** Rejected: demonstrably misses name corruption.
- **Write in place.** Rejected: contradicts the atomicity requirement outright.
- **Runtime ids on disk.** Rejected: breaks the moment content set changes.

## Consequences

- Worlds written today remain readable, or explicitly refused, forever.
- A save from a newer engine is rejected rather than partially read.
- Cost: three checksum passes per file and a read-back on every write. At Phase 0
  sizes this is negligible; if it stops being negligible, the read-back is the
  first candidate to make configurable, and that will need a new ADR.
- Not yet implemented: the incremental **journal** half of "snapshot + journal",
  region files, and compression. Tracked in the technical debt register.

## Migration

`MIN_SUPPORTED_SAVE_FORMAT` is 1. When the format changes, either the reader
handles both layouts or a migration is written; `classify_save_format` already
distinguishes `ReadCurrent`, `ReadLegacy`, `Migrate` and `Breaking`.

## Compatibility

Format version 1 is the compatibility baseline for every future NEXORA build, in
any implementation language.
