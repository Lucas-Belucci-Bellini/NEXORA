# ADR-0003 — The workspace layout enforces the dependency matrix

- **Status:** ACCEPTED
- **Date:** 2026-09-06

## Context

`NEXORA DEPENDENCY MATRIX.md` defines a one-way dependency direction:

```text
Platform -> Foundation -> Runtime -> Data/World -> Simulation -> Gameplay
         -> Society -> Presentation / Tools
```

and states that cycles require refactoring or a boundary interface.
`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §14 asks that the repository
make language boundaries explicit, while noting that exact paths can evolve.

## Problem

A layering rule that lives only in a document is checked only by reviewers, and
only when they remember. Layering violations are cheap to introduce and
expensive to unwind once other code depends on them.

## Options

1. **Document the rule and review for it.** Status quo; erodes over time.
2. **One crate with internal modules and a lint.** Rust module visibility does
   not express "world may use runtime, runtime may not use world".
3. **One crate per layer, so Cargo's own cycle rejection enforces the matrix.**
4. **A custom build-time dependency checker.** More machinery for a guarantee
   Cargo already provides.

## Decision

Option 3. Each layer is a crate, and the crate graph *is* the matrix:

```text
engine/foundation    -> (nothing)
engine/persistence   -> foundation
engine/runtime       -> foundation
engine/world         -> foundation, runtime, persistence
engine/headless      -> all of the above
```

Cargo refuses a dependency cycle, so an upward dependency does not compile. The
matrix stops being a convention and becomes a build error.

`engine/persistence` sits beside `engine/world` rather than above it. It moves
named, checksummed blobs; it knows nothing about chunks. That is what lets the
world depend on it without persistence depending on the world.

Directories follow §14 of the technology decision document (`engine/…`), leaving
`editor/`, `tools/` and `scripts/` free for the TypeScript and Python layers
that document assigns to them.

## Consequences

- Layering violations fail the build rather than a review.
- Each layer has its own test suite, lint configuration and documentation.
- Slightly more ceremony: a new shared type has to be placed in a layer
  deliberately. That friction is the point.
- Compile times are longer than a single crate would be, and parallelism across
  crates partly offsets it.

## Migration

None. New layers (simulation, gameplay, society, presentation) are added as
crates depending only downward.

## Compatibility

No runtime surface. Crate names are public API for anything that later links
against them.
