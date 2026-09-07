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
| [0006](ADR-0006-entity-identity-and-storage.md) | Entity identity is public API; storage layout is not | ACCEPTED |
| [0007](ADR-0007-physics-collides-against-a-provider-not-the-world.md) | Physics collides against a provider, not against the world | ACCEPTED |
| [0008](ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md) | Streaming decides residency; a backend provides it | ACCEPTED |
