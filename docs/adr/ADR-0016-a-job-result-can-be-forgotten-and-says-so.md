# ADR-0016 — A job result can be forgotten, and the pool says so rather than guessing

- **Status:** ACCEPTED
- **Date:** 2026-09-19
- **Closes:** `DEBT-0040` (the job system keeps every job's result forever)
- **Follows:** `DEBT-0004`, whose lesson this reuses: a bound is only acceptable
  when what it discards is impossible to miss

## Context

`worker_loop` records every finished job's outcome so a later
`JobSystem::wait` can find it. Nothing ever removed one. `wait` reads with
`get` and clones, deliberately, because waiting twice on the same handle has to
keep working — and the pool has no way to know that nobody will ask again.

The result was one `(JobHandle, JobOutcome)` per job ever submitted, for the
life of the process, inside the scheduler's mutex. At the ~113,000 jobs/s the
system sustains, that is tens of megabytes per hour of session.

It was found while measuring `DEBT-0009` rather than by reading for defects, and
the same measurement established what it is **not**: retention does not slow the
producer down. `submit()` reads 12.15, 10.99 and 12.28 µs with 0, 100,000 and
500,000 results held — flat, inside the spread. This is a memory defect and only
a memory defect.

## Problem

Bounding the map is easy. Bounding it *honestly* is the decision, because every
obvious option gives something up:

- **`wait` consumes the result.** Bounded by construction, and it breaks the
  property that made `get`-and-clone the original choice: the second `wait`
  stops finding anything. Callers that check an outcome twice — or two callers
  holding the same handle — silently diverge.
- **Retain only what someone may still ask for.** Correct and unimplementable:
  `JobHandle` is a `Copy` `u64`, so the pool cannot observe whether one is still
  held. Making it an owned, droppable receipt would work and rewrites every call
  site for a memory bound.
- **Cap it and discard the oldest.** Bounded, cheap, no API break — and on its
  own it trades a leak for a *silent wrong answer*: a discarded outcome makes
  `wait` behave exactly as it does for a job that has not finished. That is
  `DEBT-0004`'s defect in a new place, and the reason this entry said "probably
  an ADR" instead of just fixing it.

## Decision

**Cap it and discard the oldest — and make the discard an answer rather than an
absence.**

`JobOutcome` gains a variant:

```rust
/// The outcome is no longer known.
Forgotten,
```

`wait` now has three ends instead of two. It returns the outcome if the result
is retained; it keeps blocking while the pool still holds the job; and it
returns `Forgotten` when neither is true.

The signal that separates the last two is `State::tokens`, which already existed
and is already bounded: a token is inserted on submit and removed on completion,
so it holds exactly the queued and running jobs. No new bookkeeping was added to
answer the question — the map that answers it was already there.

`MAX_RETAINED_RESULTS` is 65,536, chosen against the largest wave the engine
submits rather than for roundness: chunk generation across an interest radius of
12 is 625 columns, so the cap is about a hundred times the biggest batch any
caller collects from today.

### Three consequences that are the point, not side effects

**A forgotten job is never reported as a successful one.** `Forgotten` is its
own variant precisely so that the tempting shortcut — treat a missing result as
"it must have worked" — is not available. A job that failed and was then
discarded would otherwise read as a success, which is the one answer a scheduler
must never invent. `JobOutcome::is_success` returns false for it and
`is_known` exists to ask the question directly.

**`wait` always terminates now, and it did not before.** Waiting on a handle
this pool never issued used to block forever: no result would arrive, and the
loop had no other way out while the pool was healthy. The `tokens` check that
distinguishes "still running" from "gone" answers that case too. This is a bug
fix that fell out of the design rather than one that was looked for.

**Discards are counted, not merely possible.** `JobMetrics::forgotten_results`
is a running total. A count rather than a flag because the useful question is
not whether the pool drops answers but how fast: a server that forgets a handful
over a day is working as designed, and one forgetting thousands a minute has a
caller submitting work it never collects.

### A second leak, deleted rather than bounded

`State` also carried `cancelled: HashSet<JobHandle>`. It was inserted into by
`cancel` and **read by nothing** — the actual cancellation mechanism is the
`CancellationToken`'s atomic flag, which `cancel` sets on the line below. So it
leaked one entry per cancelled job while having no effect on anything.

That one is not capped, it is removed. A set nobody reads is not state.

## Consequences

- **Memory is bounded**, at roughly two megabytes for a full retained set of
  completed or cancelled outcomes. A `Failed` carries its `Error` and is larger;
  failures are not the common case, and the cap still bounds the count.
- **`JobOutcome` gained a variant**, which is a breaking change for an
  exhaustive `match`. The workspace is the only consumer and has one match on
  it, which already had a catch-all that treats anything but `Completed` as a
  failure — the correct reading of `Forgotten`. The counter match inside
  `worker_loop` names the variant explicitly instead of using a catch-all, so
  the next variant added stops the build there rather than going uncounted.
- **`cancel` on a forgotten handle reports unknown**, the same as a handle never
  issued. Cancelling something that finished long ago was never going to do
  anything.
- **Nothing in the engine can observe the cap today.** The slice submits 25 jobs
  and collects all 25; the benchmark's largest wave is 1,000. Both are three
  orders of magnitude under 65,536, so this change is invisible to every current
  caller — which is why the tests drive two full caps through a pool rather than
  relying on any existing path to exercise it.

## Migration

None. No stored data and no wire format describes a job result, the cap applies
from process start, and every existing caller stays three orders of magnitude
below it.

## Compatibility

`submit`, `submit_all`, `cancel`, `barrier` and `metrics` are unchanged in
signature and in meaning. `wait` keeps its signature and gains a third answer in
a case that previously blocked forever, so no call that used to return can start
returning something different — only calls that used to hang now return.
