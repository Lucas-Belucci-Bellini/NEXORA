# ADR-0015 — The spatial index is a loose grid the store maintains, and a query may decline it

- **Status:** ACCEPTED
- **Date:** 2026-09-16
- **Closes:** `DEBT-0010` (entity queries are a linear scan)
- **Amends:** [ADR-0006](ADR-0006-entity-identity-and-storage.md), which
  deferred the index and recorded an extrapolation this replaces

## Context

`Entity System.md` §34 (ENTITY-33) specifies a spatial index. ADR-0006
deliberately did not build one: *"the spatial index of `Entity System.md` §34 is
deferred with a measured trigger point rather than a guess"*. `DEBT-0010`
carried the deferral, and its remediation set the order explicitly — *"medir de
novo contra a varredura antes de manter"*, measure the scan again before keeping
anything.

That order was followed. The scan was measured at 1,000, 10,000 and 100,000
entities before a line of index existed, and the measurement changed what got
built (finding 10l in `docs/benchmarks/PHASE-0-BASELINE.md`).

## Problem

A 16-block radius query over 100,000 entities examined **all 100,000** to return
**six**. The cost of a spatial query followed the population; the answer followed
the volume asked about; and nothing connected the two.

The measurement also said something the entry did not predict. `Query::by_type`
costs 15.7 ns per entity at 1,000 and **40.7 ns** at 100,000 — it got worse per
entity, because it matches the whole population and the cost is in building the
result. ADR-0006's note that a type query "will not hold at 100,000" was right
about the direction and understated the size. It is also the one query an index
cannot help: no arrangement makes an answer of everything smaller.

So the problem is narrower than "queries are linear scans". It is that the
**spatial** queries are.

## Options

1. **Index by chunk column.** The natural key, and `Query::in_chunk` asks for
   exactly one. It requires the store to hold a
   [`ChunkShape`](../../engine/foundation/src/spatial.rs), which is a runtime
   value it has never needed and would have to be threaded through every
   constructor — and `CHUNK & VOXEL ENGINE.md` §3 forbids assuming a fixed one,
   so a store built for one shape could not answer for another.
2. **A three-dimensional grid.** Finer, and it subdivides the axis entities are
   least spread along: horizontally a world runs for kilometres, vertically for
   a few hundred blocks with most life near one surface. Three times the
   bookkeeping to narrow the axis that needs it least.
3. **A loose two-dimensional grid with its own fixed cell.** Answers a
   *world-space rectangle*, so `in_chunk` converts a column to a rectangle at
   the call site and any chunk shape works against the same index. A
   power-of-two cell makes placing a position two shifts.
4. **An index the caller maintains.** Rejected without measuring: the store is
   the only thing that sees every transform change, so any other owner is an
   index that can be stale, and a stale spatial index returns wrong answers
   silently.

## Decision

**Option 3**, as `nexora_entity::index::SpatialIndex`: a `BTreeMap` from a
16-block `(x, z)` cell to the slots in it, with the cell recorded as one more
dense column beside the transform.

Three decisions make it safe rather than merely faster.

### The store maintains it, always, and not on request

`EntityStore` files a slot on `spawn`, removes it on `despawn`, and moves it on
`set_transform` and on `step`. Those are every place a transform can change, and
they are all inside one type.

Making it optional was considered and rejected. It would add a mode in which
`Query` has to know whether the index can be trusted, and the failure of
guessing wrong is a query that quietly returns the wrong entities. The cost of
always maintaining it is **6.5 ns per entity per tick**, measured; at the
benchmark plan's 1,000-entity stage that is 6.5 µs of a 50 ms tick. The worst
case, every entity changing cell every tick, is 170.9 µs per 1,000. Both are
bounded, and neither is worth a correctness surface.

### A query declines the index when the rectangle is larger than the population

`EntityStore::prefers_index` falls back to the scan when the rectangle covers
more cells than the store holds entities, because past that the empty-cell
lookups alone cost more than looking at every entity.

**This is not a performance heuristic.** A finite radius of `1e300` spans about
1.3 × 10³⁶ cells; walking them does not finish. Removing the guard does not make
that query slow, it makes it hang. The bound is deliberately conservative — a
`BTreeMap` lookup costs more than a step through a dense array, so the true
crossover is below it — and one place decides for all three spatial queries, so
they cannot drift into three answers.

### Only the spatial queries use it, and the others are not an oversight

`within_radius`, `in_chunk` and `overlapping` go through the grid.
`matching`, `count`, `by_type` and `by_tag` do not, and will not: they ask a
question with no position in it, so every entity is a candidate by construction.

## Consequences

Measured on the benchmark world, before and after, on one machine:

| | scan | index | |
| --- | ---: | ---: | ---: |
| `within_radius(16)` over 100,000 spread entities | 979.7 µs | **782 ns** | 1,253× |
| `in_chunk` over the same | 1,655.4 µs | **625 ns** | 2,647× |
| `entity.query_radius_candidates_100k` | 100,000 | **41** | a count, not a time |
| `entity.step_1000` | 2.07–2.11 µs | 7.2–8.7 µs | the price |
| `by_type` over 100,000 | 4,069 µs | 4,196 µs | unchanged, deliberately |

- **The shape changed, not just the constant.** A radius query costs 430, 654
  and 782 ns at 1,000, 10,000 and 100,000 entities: flat in the population where
  it used to be proportional to it.
- **At 1,000 entities in the plan's own fixture the index is a net loss** of
  about 6.5 µs per tick. That fixture packs the population into twelve cells, so
  a radius query there asks about most of it. The fixture was left alone —
  changing it would make every 1,000-entity row in the baseline incomparable
  with every run before it — and new rows measure a spread population beside it.
- **Results stay in slot order.** `in_chunk` and `overlapping` sort by slot
  before returning, and `within_radius` breaks distance ties by slot, because a
  walk over cells does not produce the order a scan did and
  `NEXORA REPLAY AND DETERMINISM.md` requires the same inputs to give the same
  sequence. Without it, a despawn elsewhere in the world could reorder an
  unrelated query's answer.
- **A box query is widened by the largest half-extent the store has ever held**,
  because an entity filed by its centre can reach into a rectangle it is not in.
  That maximum never falls: lowering it on despawn would mean scanning
  everything that is left, on a despawn, to save candidate cells on a query. Too
  wide costs work; too narrow loses an answer.
- **`CellCoord::of` does not call `f64::floor`.** It is on `step`'s inner loop,
  where the baseline is about 2 ns per entity, and the x86-64 baseline has no
  `roundsd` — so `floor` is a libm call, 4.94 ns against 1.54 ns for a
  truncating cast with a correction. A test pins the two spellings together over
  20,000 random values and every special case, because they must agree exactly
  with `WorldPosition::to_block_pos`, which still spells it with `floor`.
- **The index is an implementation detail and stays one.** `SpatialIndex` is
  public so it can be documented and tested, but `EntityStore` exposes only
  `occupied_cells` as a diagnostic, and no caller addresses a cell. ADR-0006's
  rule that storage layout is not public API is unchanged.

## Migration

None. No stored data describes the grid: the index is rebuilt by the spawns that
`entity::persist::load` performs, so a save written before this change loads into
a fully indexed store with no conversion and no version bump.

## Compatibility

No change to the save format, to `EntityId`, or to the result of any query — the
returned sets and their order are the same ones the scan produced, which is what
the differential tests in `query.rs` assert against a brute-force reference.
