# NEXORA — Architecture Decision Records

Each record follows the field list required by `NEXORA ADR INDEX.md`: context,
problem, options, decision, consequences, migration and compatibility.

Statuses are `PROPOSED`, `ACCEPTED`, `REJECTED`, `SUPERSEDED`, `DEPRECATED`.

An implementation must not contradict an accepted ADR. Changing course means
updating the record or superseding it with a new one — not quietly diverging.

| ADR | Title | Status |
| --- | --- | --- |
| [0001](ADR-0001-rust-phase-0-reference-implementation.md) | Rust as the Phase 0 reference implementation | ACCEPTED |
| [0002](ADR-0002-zero-dependency-foundation.md) | Zero external dependencies in the engine crates | ACCEPTED |
| [0003](ADR-0003-workspace-layout-enforces-dependency-matrix.md) | The workspace layout enforces the dependency matrix | ACCEPTED |
| [0004](ADR-0004-save-container-format-v1.md) | Save container format v1 | ACCEPTED |
| [0005](ADR-0005-phase-0-scope-boundary.md) | Phase 0 scope boundary | ACCEPTED (amended by 0006, 0007, 0008) |
| [0006](ADR-0006-entity-identity-and-storage.md) | Entity identity is public API; storage layout is not | ACCEPTED (amended by 0015) |
| [0007](ADR-0007-physics-collides-against-a-provider-not-the-world.md) | Physics collides against a provider, not against the world | ACCEPTED |
| [0008](ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md) | Streaming decides residency; a backend provides it | ACCEPTED |
| [0009](ADR-0009-a-second-stack-measures-kernels-not-an-engine.md) | A second stack measures kernels, not an engine — and one crate may say `unsafe` | ACCEPTED |
| [0010](ADR-0010-commands-are-intent-and-carry-their-own-authority.md) | Commands are intent, and the crate graph enforces the boundary | ACCEPTED |
| [0011](ADR-0011-a-torn-tail-is-a-crash-and-corruption-is-not.md) | A torn tail is a crash; corruption is not, and each has its own answer | ACCEPTED |
| [0012](ADR-0012-a-mesh-is-a-data-structure-not-a-picture.md) | A mesh is a data structure, not a picture | ACCEPTED |
| [0013](ADR-0013-recovery-finishes-when-a-column-arrives.md) | Recovery is not a moment; it finishes when a column arrives | ACCEPTED |
| [0014](ADR-0014-a-region-file-is-authoritative-for-its-region.md) | A region file is authoritative for its region; the resident set is not a delete list | ACCEPTED |
| [0015](ADR-0015-a-spatial-index-is-a-loose-grid-and-a-query-may-decline-it.md) | The spatial index is a loose grid the store maintains, and a query may decline it | ACCEPTED |
| [0016](ADR-0016-a-job-result-can-be-forgotten-and-says-so.md) | A job result can be forgotten, and the pool says so rather than guessing | ACCEPTED |
| [0017](ADR-0017-a-frame-is-time-the-host-hands-in.md) | A frame is time the host hands in, and the stages account for it | ACCEPTED |
| [0018](ADR-0018-input-is-intent-the-host-hands-in.md) | Input is intent the host hands in, and a context consumes a source | ACCEPTED |
