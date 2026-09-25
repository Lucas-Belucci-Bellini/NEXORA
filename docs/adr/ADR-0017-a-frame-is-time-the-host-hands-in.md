# ADR-0017 — A frame is time the host hands in, and the stages account for it

- **Status:** ACCEPTED
- **Date:** 2026-09-20
- **Implements:** `CORE.md` §16 (ENGINE-0 — Game Loop) and the *Runtime
  separation* rule of `NEXORA PERFORMANCE BUDGETS.md`
- **Unblocks:** `DEBT-0018` and `DEBT-0027`, whose trigger was literally
  "existir um loop de quadro"

## Context

Until now nothing in this engine was a *frame*. Streaming ticked when a test
called `tick`, physics stepped when the slice asked, meshing ran on whichever
thread wanted a mesh. That is not a gap in the sense of a missing feature — the
systems work — but it left three things with nowhere to live:

1. **A fixed timestep.** `nexora_foundation::time` declares 20 ticks per second
   and a world clock that advances by a delta. Nothing converted irregular real
   time into those equal ticks.
2. **A bound on catching up.** Nothing anywhere decided what happens when a
   frame arrives late.
3. **Somewhere for the measurements to point.** The baseline says a chunk costs
   2.72 ms to generate, a 32³ region 26.12 ms to mesh, an idle streaming tick
   ~3.0 µs. Every one of those was compared to a frame *in prose*, in the debt
   register, by a human holding 16.67 ms in their head.

`NEXORA PERFORMANCE BUDGETS.md` is explicit that the third one is a defect:
*"a subsystem must not consume another subsystem's budget invisibly"*, and
*"budgets are measurable in tests and benchmarks"*. Neither sentence had any
mechanism behind it.

## Problem

A game loop's textbook job is to reconcile real time with simulated time, and
the textbook shape is a `while` loop that reads a clock. Both halves of that are
refused here:

- **Reading a clock.** `nexora_foundation::time` opens with the rule that shapes
  it: *a system that samples the host clock cannot be replayed*. A loop that
  calls `Instant::now` puts wall-clock time inside the engine, and every test of
  it becomes a test of how fast the machine ran it.
- **Owning the process.** A `while running { … }` that owns the thread cannot be
  driven by a test, cannot be stepped, and cannot be handed a recorded sequence
  of frames by a replay.

There is a third problem, subtler and the reason this is an ADR rather than a
commit: **attribution that only sums its own inputs proves nothing.** If the
frame reports "simulation 2 ms, world 1 ms, total 3 ms", the total is a
tautology — it equals what was put in. A stage that forgot to report, or work
that happened between stages, is invisible in exactly the way the budgets
document forbids.

## Decision

**The frame loop is a value the host drives, time is an argument, and the frame
reports what no stage claimed.**

Three parts, in `engine/runtime/src/frame.rs`:

1. **`FrameSchedule`** — the accumulator. `advance(elapsed)` returns a
   `StepPlan`: how many fixed steps to run, how much time carries to the next
   frame, and **how many steps were discarded**. Past the cap, the backlog is
   thrown away rather than carried, because carrying it makes the next frame owe
   those steps *plus* whatever it earns — the spiral. `StepPlan::discarded` is
   how anyone finds out; this follows `DEBT-0004` and ADR-0016, where the same
   rule was settled for a change feed and for a job result: **a bound is only
   acceptable when what it discards is impossible to miss.**

2. **`FrameBudget`** — the four thresholds `NEXORA PERFORMANCE BUDGETS.md` asks
   every major system to publish: `TARGET`, `WARNING`, `CRITICAL`, `EMERGENCY`.
   `classify` puts a measured duration in one of them. The convenience
   constructor is named `doubling_from`, so that the one thing it invents — the
   2×/4×/8× ratios — is stated in its own name. The target is the period itself,
   which is arithmetic rather than a measurement.

3. **`FrameRun::record` and `FrameReport::unattributed`** — the attribution. A
   stage may be charged once and only after the stages before it; both
   violations are errors, because both are ways one subsystem's cost lands on
   another's line. `finish(wall)` takes the frame's own measured length, and
   `unattributed()` is the gap between that and the sum of the stages. That
   number is the one that cannot be faked by the accounting: it is work nobody
   owned. `overattributed()` is its mirror — stages claiming more than the frame
   lasted, which is a bug in the host rather than in the engine, and is surfaced
   rather than clamped away.

The stage sequence is `CORE.md` §16's, unchanged: Input → Simulation → World →
Physics → Render Preparation → Render → Audio. **Four of the seven have no
system in this repository**, and `FrameStage::has_system` says so rather than
letting a reader infer from an empty line that nothing ran this frame. Naming
them costs nothing and fixes the order; implementing them would be building
ahead of a consumer, which `DEBT-0021` forbids for the same reason.

## Consequences

**The loop reads no clock, so it is deterministic.** Feed the same deltas and
you get the same frames — asserted directly in `the_same_deltas_produce_the_same_frames`.
That is what lets the headless slice drive its streaming walk through the loop
and still be the deterministic proof it has always been: the delta it hands in
is scripted, one step per frame, and `SliceReport` gains counts rather than
durations.

**The settle bound gained a unit.** The slice's walk used to stop after 256
iterations. At 20 Hz the same bound is 12.8 s of simulated time — the same
number, in a unit a reader can argue with.

**The accounting is cheap, and the cost is not where it looks.** Measured on
this container: the whole per-frame accounting is 65–78 ns, of which 39 ns is
`FrameSchedule::advance` alone; charging all seven stages instead of one is
inside the spread, so attribution is effectively free and the fixed cost is the
accumulator. A guess that the 128-bit division in `advance` was that cost was
**wrong** — replacing it with a 64-bit one moved 42.0 ns to 40.0 ns against a
37% spread. It is the `Duration` arithmetic itself. Against the ~3.0 µs idle
streaming tick, the cheapest real frame this engine can run, the accounting is
about 2.3%; against the 50 ms it is allowed, 0.00016%.

**What this does not do.** It does not run anything on its own, it does not
interpolate between steps (nothing draws, and an alpha nobody reads is a number
nobody checks), and it does not make `DEBT-0018` or `DEBT-0027` go away. It
satisfies their trigger, which is a different and smaller claim: their entries
now say so, and the work they describe — moving generation and meshing off the
tick thread — is still ahead.

**No stage-to-budget-line mapping.** `NEXORA PERFORMANCE BUDGETS.md` also lists
six *runtime separation* lines (RENDER, SIMULATION, STREAMING, IO, NETWORK,
BACKGROUND WORK). Mapping seven stages onto six lines is a decision the document
does not make, and inventing the correspondence here would put a choice nobody
agreed to behind an API. Stages are the unit of attribution for now.
