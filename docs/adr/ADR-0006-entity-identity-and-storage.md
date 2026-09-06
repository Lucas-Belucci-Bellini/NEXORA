# ADR-0006 — Entity identity is public API; storage layout is not

- **Status:** ACCEPTED
- **Date:** 2026-09-06
- **Amends:** [ADR-0005](ADR-0005-phase-0-scope-boundary.md) (entity system moves
  from "deliberately not implemented" to implemented)

## Context

`Entity System.md` §1 defines the entity layer as *who exists in the world,
where they are, their lifecycle and their components* — and explicitly not
combat, AI, physics, inventory or rendering.

`ECS AND DATA ORIENTED RUNTIME.md` covers different ground: a data-oriented
execution layer for large populations. It states that the runtime *"must
complement, not replace, the public Entity API"*, and lists as an invariant that
*"storage layout is an implementation detail"*.

The startup brief §9 puts the same warning more bluntly: do not confuse entity
identity with ECS runtime storage, and *"não transformar absolutamente tudo em
ECS apenas por moda."*

## Problem

Two failure modes sit on either side of this.

Build a full archetype ECS now, and the storage model becomes the public API by
accident: callers hold component references, systems assume archetype layout,
and the "implementation detail" invariant is dead before anything measured
whether archetypes were needed.

Build naive per-entity objects instead, and the layer cannot batch, which is the
thing the population sizes in this project require.

## Options

1. **Full archetype ECS with a query planner and system scheduler.** Large,
   unmeasured, and it leaks layout into the API surface.
2. **`Vec<Box<dyn Entity>>` or one struct per entity.** Simple, and gives up
   cache-friendly iteration permanently.
3. **Public handle API over dense component columns.** Batch-friendly storage,
   with nothing about the layout observable from outside.
4. **Defer entities entirely until the RHI exists.** Blocks the one remaining
   benchmark stage that needs no hardware.

## Decision

Option 3.

**Public surface:** an opaque [`EntityId`] handle plus accessor methods. No
caller ever sees a slot, a column, or a component reference.

**Storage:** dense parallel `Vec` columns indexed by slot — generation, liveness,
persistent id, type, transform, velocity, bounds, lifecycle, policy, tags. A
tick walks them linearly.

**Not built:** archetypes, a query planner, a system scheduler, component
registration at runtime. Those are what distinguish an ECS from a component
store, and none of them has evidence behind it yet.

### Two identities, deliberately

| | scope | written to disk |
| --- | --- | --- |
| `EntityId` (slot + generation) | one session | **never** |
| `PersistentEntityId` (u64) | forever | yes |

This is the same split the block registry already makes between a runtime id and
a namespaced identifier, adopted for the same reason: a slot index is meaningful
only inside the process that assigned it. Writing one to disk means a reload can
hand entity A's history to entity B.

### Generations are the safety property

`NEXORA DATA VALIDATION AND INVARIANTS.md` requires that *a destroyed object
cannot remain addressable as active state*. Destroying an entity advances its
slot's generation, so every handle issued earlier stops resolving. A test spawns
into a just-freed slot and asserts the old handle refuses to resolve to the new
occupant.

When a generation counter would overflow, the slot is **retired** rather than
reused. Wrapping would let a very old handle alias a new entity — rare, silent,
and unfixable after the fact.

## Consequences

- Storage can become archetype-based later without changing a single call site.
- Batch iteration is already cheap: measured at **~3 ns per entity** to advance
  1,000 entities.
- Handle validation costs **2.6 ns**, so the safety property is effectively free.
- Queries are linear scans. At 1,000 entities a type query costs ~19 µs; that
  will not hold at 100,000. The spatial index of `Entity System.md` §34 is
  deferred with a measured trigger point rather than a guess — `DEBT-0010`.
- Components are fixed at compile time. Mod-defined components will need either
  a registration mechanism or a side table; deferred until the mod runtime
  exists to need it.

## Migration

None; nothing existed before. Saves store logical state only, so a future
archetype rewrite reads existing saves unchanged — which is precisely what the
"serialize logical state, not storage layout internals" rule buys.

## Compatibility

The entity save section is `nexora:save/entities`, versioned with the container
(ADR-0004). A world saved with no entity section loads as an empty population
rather than an error, so worlds written before this ADR remain valid.
