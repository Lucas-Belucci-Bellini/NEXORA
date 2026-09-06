# ADR-0005 — Phase 0 scope boundary

- **Status:** ACCEPTED
- **Date:** 2026-09-06

## Context

`NEXORA DEVELOPMENT ROADMAP.md` Phase 1 lists the engine bootstrap;
`NEXORA DEFINITION OF DONE.md` says a task is done only when it builds, is
tested, is documented and has no known blocking regression, and explicitly
rejects "works on my machine". The startup brief adds: never declare something
complete without verifying it, and never leave important architecture running
permanently on mocks.

The Phase 0 work was carried out in a headless Linux container with no display
and no GPU.

## Problem

The brief's §37 vertical slice includes a window and a renderer. Those cannot be
run, and therefore cannot be verified, in the environment where this work was
done. Writing an unrunnable renderer would produce exactly the "fake system"
the brief warns against, and claiming it works would violate the Definition of
Done.

## Decision

Phase 0 implements the part of the slice that can be **built, tested and run**,
and states plainly what it does not implement.

### Implemented and verified

| Area | Crate |
| --- | --- |
| Error taxonomy with owner and recovery path | `foundation::error` |
| Versioned contracts and compatibility gate | `foundation::version` |
| Namespaced identifiers | `foundation::ident` |
| Stable hashing and integrity checksums | `foundation::hashing` |
| Coordinate spaces with Euclidean division | `foundation::spatial` |
| Authoritative world clock and calendar | `foundation::time` |
| Deterministic, stream-separated RNG | `foundation::rng` |
| Structured diagnostics, counters, correlation | `foundation::diagnostics` |
| Layered configuration with authority | `foundation::config` |
| Runtime lifecycle state machine | `runtime::lifecycle` |
| Engine module graph, ordering, rollback | `runtime::module` |
| Registries with freeze and fingerprints | `runtime::registry` |
| Event bus with causation and loop protection | `runtime::events` |
| Bounded, prioritized job system | `runtime::jobs` |
| Versioned, checksummed, atomic save container | `persistence` |
| Palette-compressed voxel sections | `world::voxel` |
| Chunk lifecycle, dirty tracking, change journal | `world::chunk` |
| World lifecycle and deterministic generation | `world::world` |
| Identifier-remapped world persistence | `world::persist` |
| Headless vertical slice, verified end to end | `headless` |

### Deliberately not implemented

Each of these has a specification document; none has code, and none is stubbed
with something that pretends to work.

| Not implemented | Reason |
| --- | --- |
| RHI, window, renderer, camera | No display or GPU available to verify against |
| Input system | Meaningless without a window |
| Audio | Same |
| ECS / data-oriented runtime | `ECS AND DATA ORIENTED RUNTIME.md` separates entity identity from storage; neither is needed to prove the world slice, and both deserve the benchmark first |
| Entity system | Depends on the ECS decision above |
| Physics | Depends on entities |
| Networking, server | Phase 6 |
| Mod runtime, scripting | Phase 7 |
| Streaming manager, simulation LOD | Contracts exist; the slice loads chunks explicitly |
| Save journal, region files, compression | Snapshot half is implemented; see ADR-0004 |
| Cancelable / request events | `Event Bus.md` §44; facts only for now |
| Registry override and patch policies | `Registry System.md` §28–§31 |

### The rule this encodes

**"Preparing is not implementing."** A contract may be written before its
implementation. What may not happen is an implementation that reports success
without having run.

## Consequences

- The Phase 0 claim is narrow and true: everything listed as implemented is
  built, tested and executed by CI.
- The next contributor knows exactly where the edge is.
- The renderer slice remains outstanding and must be completed on hardware that
  can run it before Phase 2 can close.

## Migration

None.

## Compatibility

None of the unimplemented areas has a persisted or wire surface yet, so adding
them later is additive.
