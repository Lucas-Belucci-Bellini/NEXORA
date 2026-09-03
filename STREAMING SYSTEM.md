# NEXORA — STREAMING SYSTEM

> Streaming controls which world, entity, resource and simulation data is resident. It does not own generation or gameplay rules.

## Pipeline
`Interest → Priority → Request → Load/Generate → Validate → Activate → Simulate → Deactivate → Persist → Evict`.

## Domains
Chunks, regions, entities, structures, resources, GPU assets, dimensions and simulation contexts.

## Interest
Player, camera, vehicles, AI, server visibility, prefetch and world-event relevance.

## Budgets
Separate memory, IO, generation, activation and network budgets. Use backpressure and hysteresis to prevent thrashing.

## LOD
`FULL → REGIONAL → ABSTRACT → UNRESIDENT`. Logical identity and persistent state survive eviction.

## API
```ts
interface IStreamingSystem {
  request(target: StreamTarget, reason: StreamReason): StreamHandle;
  cancel(handle: StreamHandle): void;
  setInterest(source: InterestSource): void;
  tick(budget: StreamingBudget): StreamingReport;
}
```

## Tests
Fast travel, cross-region streaming, dimension transfer, save-before-evict, low-memory pressure and multiplayer reconnect.
