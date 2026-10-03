# ADR-0034 — The freeze gates the core's language, not every boundary's

- **Status:** PROPOSED — it changes what the Architecture Freeze requires, and
  that is the project owner's decision, not an engineering one. Until it is
  accepted the gate reads as it always has, and Phase 0 stays open.
- **Date:** 2026-09-29
- **Informs:** `DEBT-0008`; [ADR-0001](ADR-0001-rust-phase-0-reference-implementation.md)
  (this is **not** the lock that ADR promised — it locks nothing);
  [ADR-0009](ADR-0009-a-second-stack-measures-kernels-not-an-engine.md)
- **Would touch, once accepted:** the Final gate of
  `NEXORA ARCHITECTURE FREEZE CHECKLIST.md` and the scope of `DEBT-0008`

## Context

The Architecture Freeze has one blocker left, `DEBT-0008`: the language gate's
benchmark. Every GPU stage of it now has a number, in CI and on the operator's
RX 6650 XT and display (baseline Appendices K and O). Reading the documents
that define the gate side by side, on 2026-09-29, turned up what the blocker
is made of, and one part of it cannot be built in the order the roadmap
builds things.

What the documents say:

- `NEXORA DEVELOPMENT ROADMAP.md`, Phase 0: its list names the **benchmark
  plan**, not the benchmark's result, and its one exit is *"nenhum blocker
  arquitetural crítico sem decisão registrada"*.
- `NEXORA ARCHITECTURE FREEZE CHECKLIST.md`, Final gate: unresolved items are
  completed *"or explicitly classified as post-freeze extensions with no
  impact on frozen contracts"*.
- `NEXORA TECHNOLOGY BENCHMARK PLAN.md`: a slice that ends in **mod
  boundary**, and candidate combinations compared *"apenas quando houver
  justificativa"*.
- `ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17: a minimum benchmark
  that adds **player movement** and **one cross-language tool call**; §18: a
  lock that needs, among twelve criteria, *"Mod boundary is stable"* and
  *"Editor/runtime boundary is stable"*.
- The roadmap puts modding in **Phase 7** and the editor and toolchain in
  **Phase 8**.

Read as "the complete benchmark and the full §18 lock before the freeze", the
gate waits for Phases 7 and 8, which come after the freeze it gates. No code
written in Phases 0 to 2 can produce a stable mod boundary.

## Problem

The gate cannot be passed in roadmap order as written, and the two ways out
that need no decision are both wrong:

- **Keep Phase 0 open until Phase 8.** Then the freeze freezes nothing: every
  contract gets built "ahead of order" first and is frozen by accident, which
  is the outcome the freeze exists to prevent.
- **Lock the stack now.** That settles by default the question the gate exists
  to ask (ADR-0001, ADR-0009), from evidence that does not yet include two of
  §17's stages.

## What the freeze protects

The Final gate names it: frozen contracts. ADR-0001's migration section says
what those are: the contracts live in the documents, and a port *"reimplements
the same contracts and must reproduce the same save format (ADR-0004) and the
same generation output for a given seed"*.

The language map (`NEXORA LANGUAGE AND FFI BOUNDARY.md`) assigns languages **by
responsibility** — §3 of the technology decision: *split by responsibility,
not by fashion* — and gives each responsibility four ways to meet another:
same-process FFI, IPC, a file/resource contract, a network protocol.

That gives the question a shape. A language choice can wait past the freeze
exactly when the responsibility it belongs to meets the frozen core through a
**language-neutral** boundary — IPC, a file or resource contract, a network
protocol — because changing that language later changes no frozen contract.
A choice that the core crosses through same-process FFI cannot wait.

## Options

1. **Keep the gate as written.** Phase 0 stays open until Phases 7 and 8.
2. **Lock the whole stack now** from the evidence at hand.
3. **Split the gate by responsibility.** The freeze requires the *core*
   runtime's language to be settled by the benchmark's core stages, all of
   which are buildable before the freeze. The languages of responsibilities
   behind a language-neutral boundary — mod scripting, editor and tooling,
   research — are locked in the phases that build those boundaries, as
   post-freeze extensions.
4. **Drop the benchmark requirement.** Contradicts every document above.

## Decision (proposed)

**Option 3.**

### What stays a freeze blocker — `DEBT-0008`, narrowed

The language of the **core runtime**: core, simulation, world, persistence,
server — the Rust column of the technology decision's §4 and §19. It is
settled on the reference implementation by the §17 slice's core stages:

| stage | state on 2026-09-29 |
| --- | --- |
| window, RHI, camera, 16³ chunk, mesh generation, 1,000 entities, physics, jobs, streaming, save/load, headless server | measured (baseline Appendices A–O) |
| **player movement** | to build — it is also the roadmap's Phase 2 player |
| **one cross-language tool call** | to build: one tool, written in a language other than the core's, called across the boundary layer the language map gives tools (IPC or a file/resource contract), and timed |
| **the Benchmark Plan's decision outputs** | to write: performance score, engineering complexity, tooling score, maintenance risk, platform score and modding score, as a scorecard from the measured evidence. The modding score scores the mod boundary **contract** — defined, as the roadmap's Phase 0 list asks — not a built mod runtime |

And the **engine-scale comparison**, which ADR-0009 put outside the kernel
reference: the Benchmark Plan compares candidate combinations *"apenas quando
houver justificativa"*, and the kernel reference is what tests for that
justification. On nine shared kernels Rust fell inside the band two C++
compilers span (Appendix D). A second implementation of the core is built
**only if** the scorecard shows a core stage where the reference is
measurably held back by its language rather than by its design. Absent that,
the comparison is recorded as *not justified*, with its evidence: a decision,
not a gap.

### What becomes a post-freeze extension

- The **mod and scripting runtime's language** and the Benchmark Plan's **mod
  boundary** stage move to **Phase 7**, locked by that phase's ADR once the
  boundary exists.
- The **editor and tooling language** — TypeScript per the technology
  decision's §6, or Rust — moves to **Phase 8**.
- §18's *"Mod boundary is stable"* and *"Editor/runtime boundary is stable"*
  are evaluated in those phases, for those locks.

**The condition that makes each deferral legal, and what revokes it:** the
responsibility meets the core only through a language-neutral boundary. If a
later phase finds that a deferred responsibility must cross into the core
through same-process FFI on a per-tick path — which
`NEXORA LANGUAGE AND FFI BOUNDARY.md` already forbids — the deferral is void,
and the question returns to Phase 0 by ADR.

### What this does not do

It locks nothing. The core lock is still the future ADR that ADR-0001
promised, written when the narrowed `DEBT-0008` closes. Nor does it declare
the freeze met: with this accepted, the freeze has a finite list — player
movement, one cross-language tool call, the scorecard, and the core-lock ADR
— and it stays open until that list is done.

## Consequences

- **Accepted:** the Final gate reads the language gate as narrowed here;
  `DEBT-0008`'s scope is narrowed to the table above; the Phase 7 and Phase 8
  locks become tracked items of those phases. The freeze becomes something
  that can close before the phases it is meant to precede.
- **Rejected:** nothing changes. Phase 0 stays open until the full gate can
  close, which is Phase 8, and the checklist says so.
- **Either way:** player movement and the cross-language tool call are in §17
  under both readings, so building them does not wait for this decision.

## Migration

None. No code, save format or public API changes.

## Compatibility

None. This record changes how a gate is read, and only once accepted.
