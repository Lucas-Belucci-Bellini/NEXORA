# NEXORA — ECS / DATA-ORIENTED RUNTIME

> NEXORA needs a data-oriented execution layer for large populations, mobs, vehicles, particles, simulation aggregates and background workloads. It must complement, not replace, the public Entity API.

## Model
```text
Entity ID
→ Component / Fragment Data
→ Archetype / Storage Layout
→ System Query
→ Batch Processing
→ State Change
```

## Goals
- cache-friendly storage;
- stable IDs and handles;
- batch iteration;
- predictable memory ownership;
- archetype or equivalent composition where beneficial;
- separate high-frequency simulation data from cold/persistent data;
- graceful transition between FULL, REGIONAL and ABSTRACT simulation.

## Entity separation
Public Entity System defines identity/lifecycle/capabilities. The data-oriented runtime decides how hot simulation data is physically laid out.

## Queries
Systems declare read/write access so the scheduler can detect conflicts and parallelize safe batches.

## LOD
A logical entity may move between full component state and aggregate representation. The conversion must preserve authoritative state.

## API sketch
```ts
interface IDataRuntime {
  createEntity(archetype: ArchetypeID): EntityHandle;
  query(query: DataQuery): QueryHandle;
  execute(system: DataSystem): void;
  migrate(entity: EntityHandle, target: ArchetypeID): Result;
}
```

## Integration
Entity, AI, Physics, Vehicles, Animation, World Events and Civilization can consume the runtime. Persistence serializes logical state, not storage layout internals.

## Tests
Archetype migration, query correctness, read/write conflict detection, deterministic batching, LOD conversion and large-scale entity stress.

## Invariants
- Storage layout is an implementation detail.
- Systems cannot depend on memory addresses or worker execution order.
- Authoritative results remain deterministic.
