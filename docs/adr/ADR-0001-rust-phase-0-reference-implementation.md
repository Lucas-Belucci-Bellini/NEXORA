# ADR-0001 — Rust as the Phase 0 reference implementation

- **Status:** ACCEPTED
- **Date:** 2026-09-06
- **Supersedes:** none

## Context

`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §1 states the final language
selection is **NOT YET LOCKED**, and §17 defines a language selection gate: the
same vertical slice must be built in the strongest candidate combinations and
measured. `NEXORA ARCHITECTURE GAP AUDIT.md` reaches the same conclusion — the
architecture is ready for a benchmark, not for a language decision taken from
feature count.

At the same time the repository contained 145 markdown documents and zero lines
of code. Several exit criteria in that gate — "job system works", "streaming
works", "persistence works", "benchmark results are acceptable" — cannot be
evaluated without an implementation to measure.

## Problem

The language gate cannot be passed without code, and code cannot be written
without choosing a language. Deciding "no code until the benchmark" freezes the
project; declaring the stack final without measurement violates the gate.

## Options

1. **Wait for the benchmark before writing any engine code.** Deadlocks: the
   benchmark *is* engine code.
2. **Declare Rust final now.** Contradicts §1 and §18 of the technology
   decision document and discards the gate.
3. **Write the Phase 0 foundation in the documented leading candidate,
   explicitly as the reference implementation the gate will measure.**
4. **Write the foundation in a throwaway scripting language first.** Produces a
   slice whose measurements say nothing about any candidate stack.

## Decision

Option 3. The Phase 0 foundation is implemented in **Rust**, which
`ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §4 and §19 already name as the
preferred candidate for the core runtime.

**This does not lock the stack.** This implementation is the reference slice
against which the §17 benchmark runs. The gate in §18 stays open, and its
criteria still have to be met before any final lock is recorded — in a future
ADR, not this one.

Two properties are maintained specifically so that the slice remains usable as a
benchmark subject rather than becoming a fait accompli:

- The workload is the one §17 names (world → chunk → voxel → mutation →
  streaming-shaped generation → job system → save/load → headless), so a
  competing stack has a defined target to reimplement.
- The slice is measurable: it reports chunk counts, voxel storage, save size and
  timings rather than only printing "OK".

## Consequences

- Phase 0 progresses with real, tested, runnable code.
- The benchmark has a concrete reference to compare against, which is what §17
  actually asks for.
- If a competing stack wins the benchmark, this implementation is discarded or
  ported. That cost is accepted, and is smaller than the cost of deadlock.
- Nothing in this ADR authorizes writing the editor, tooling or research layers
  in Rust; §6 and §7 assign those to TypeScript and Python.

## Migration

None. There is no prior implementation.

If the benchmark selects a different runtime stack, the migration path is the
architecture documents themselves: contracts live in markdown, and the Rust
crates implement them rather than defining them. A port reimplements the same
contracts and must reproduce the same save format (ADR-0004) and the same
generation output for a given seed.

## Compatibility

No compatibility surface exists yet. The save format is versioned from its first
byte (ADR-0004), so a future runtime in another language can read worlds written
by this one.
