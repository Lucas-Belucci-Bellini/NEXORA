# ADR-0002 — Zero external dependencies in the engine crates

- **Status:** ACCEPTED
- **Date:** 2026-09-06

## Context

The Phase 0 crates need error handling, hashing, checksums, a pseudo-random
generator, binary serialization and a worker pool. Mature third-party crates
exist for all of these and are reachable from the build environment.

Three project documents constrain the choice:

- `NEXORA SAVE FORMAT AND COMPATIBILITY.md` requires saves to remain readable
  and migratable "durante anos de evolução do engine".
- `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md` requires that a seed
  plus a generator version reproduces a world exactly.
- `NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md` and the original-content
  policy require provenance to be recorded for what ships.

## Problem

A dependency that changes its representation changes NEXORA's on-disk format or
its world generation without any NEXORA version being bumped. Serialization
crates do not generally promise byte-stability across major versions, and the
standard library's `DefaultHasher` explicitly does not promise stability at all.
Either would silently break the two reproducibility guarantees above.

## Options

1. **Use `serde` plus a binary codec, and a random-number crate.** Fastest to
   write; delegates the stability of the save format and of world generation to
   third parties.
2. **Use dependencies but pin exact versions.** Reduces drift, does not remove
   it; a pin still has to be raised eventually, and the format moves with it.
3. **Implement the primitives in the engine, with no external dependencies.**
4. **No dependencies anywhere in the project, forever.** Overreaches: rendering,
   networking and tooling will need libraries, and reimplementing a GPU
   abstraction has no reproducibility argument behind it.

## Decision

Option 3, scoped to the engine crates that exist today:
`nexora-foundation`, `nexora-runtime`, `nexora-persistence`, `nexora-world` and
`nexora-headless` declare **no external dependencies**.

The primitives implemented in `nexora-foundation` are all published algorithms
implemented from their specifications, not copied source:

| Primitive | Algorithm | Why it must be ours |
| --- | --- | --- |
| `hashing::Fnv1a64` | FNV-1a, 64-bit | Registry fingerprints must match between peers, forever |
| `hashing::crc32` | CRC-32/ISO-HDLC | Save integrity; the value is written to disk |
| `rng::Rng` | SplitMix64 | World generation must reproduce exactly, on every platform |
| `codec` | NEXORA's own framing | The save layout is a NEXORA contract, not a library's |

Each is pinned by tests against published reference vectors, and the generator
is pinned by golden output values.

## Consequences

- The save format and world generation cannot drift because of an upgrade
  elsewhere. Reproducibility is a property of this repository alone.
- Builds are fast and fully reproducible; CI has no supply-chain surface.
- More code to own, and primitives that are correct rather than maximally
  optimized. `next_below` uses rejection sampling for unbiased draws, which
  costs an occasional retry.
- This is **not** a project-wide rule. Rendering, windowing, networking and
  tooling will take dependencies; each should be justified where it is added.

## Migration

None. Should a dependency later replace one of these primitives, the algorithm
must produce identical output (the reference-vector tests are the acceptance
criterion) or the change is a breaking format change requiring a version bump
and a migration.

## Compatibility

Locks the on-disk and on-wire meaning of every checksum and generated world to
algorithms specified in this repository.
