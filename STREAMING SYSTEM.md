# NEXORA — STREAMING SYSTEM

> Streaming controls what world, entity, resource and simulation data is resident at each moment. It is a residency system, not a world-generation system.

## Pipeline
```text
Interest
→ Priority
→ Request
→ Load / Generate
→ Validate
→ Activate
→ Observe / Simulate
→ Deactivate
→ Persist if dirty
→ Evict
```

## Domains
- chunks and regions;
- entities and structures;
- resources and GPU assets;
- simulation contexts;
- dimensions and world areas.

## Interest sources
Player, camera, vehicles, NPCs, server visibility, prefetch, AI, world events and administrative tools.

## Priority
`CRITICAL`, `HIGH`, `NORMAL`, `BACKGROUND`. Consider distance, velocity, predicted route, visibility, gameplay relevance, dirty state and dependency chains.

## Prefetch
Fast vehicles and camera movement should trigger predictive prefetch. Never let prefetch bypass global memory/IO budgets.

## Residency states
`UNREQUESTED`, `REQUESTED`, `LOADING`, `GENERATING`, `READY`, `ACTIVE`, `DORMANT`, `EVICTING`, `UNLOADED`, `FAILED`.

## Hysteresis
Use separate load/unload thresholds to prevent thrashing when the player moves near a boundary.

## LOD
Streaming and Simulation LOD cooperate:
```text
FULL → REGIONAL → ABSTRACT → UNRESIDENT
```
A distant region may remain represented by compact state even when detailed data is evicted.

## Dirty state
Before eviction, route authoritative dirty data through Persistence. Derived render/cache data can be discarded.

## Dependencies
A chunk may require neighboring chunks, registry snapshots, dimension state or resource handles. Dependencies are explicit and bounded.

## API sketch
```ts
interface IStreamingSystem {
  request(target: StreamTarget, reason: StreamReason): StreamHandle;
  cancel(handle: StreamHandle): void;
  setInterest(source: InterestSource): void;
  tick(budget: StreamingBudget): StreamingReport;
  inspect(target: StreamTarget): StreamState;
}
```

## Performance
Use Job System for asynchronous work. Enforce memory, bandwidth, generation and activation budgets separately.

## Multiplayer
The server streams authoritative state; clients receive only interest-relevant data. Do not equate network visibility with local residency.

## Fault handling
Failed loads become retryable/quarantined states. Corrupt persistent data is sent to Persistence recovery instead of being silently regenerated.

## Debug
`nexora stream inspect`, `queues`, `budget`, `prefetch`, `residency`, `thrash`.

## Tests
Boundary movement, high-speed travel, dimension transition, save-before-evict, corrupted chunk, low-memory pressure and reconnect streaming.

## Invariants
- Streaming never changes authoritative game rules.
- Unloaded does not mean forgotten when persistent state exists.
- Memory budgets are hard limits with observable degradation.
