# NEXORA — SCENE / LEVEL RUNTIME

> Scene/level runtime manages loaded authored or procedural world contexts. It is a runtime orchestration layer, not a second simulation engine.

## Model
```text
World / Dimension
→ Region / Cell
→ Scene Context
→ Prefabs / Structures / Entities
→ Simulation + Rendering
```

## Responsibilities
- scene/context lifecycle;
- cell activation and deactivation;
- editor/runtime scene parity;
- references between authored objects;
- streaming-aware activation;
- spawn sets and encounter areas;
- test scenes and benchmark scenes.

## Relationship to world
Voxel world, dimensions and persistence remain authoritative. Scene runtime only manages which authored context is active and how it binds to world/runtime services.

## Lifecycle
`UNLOADED → LOADING → ACTIVE → DEACTIVATING → UNLOADED` with `FAILED` quarantine state.

## API
```ts
interface ISceneRuntime {
  load(id: SceneID, context: SceneLoadContext): SceneHandle;
  unload(handle: SceneHandle): Result;
  activate(handle: SceneHandle): Result;
  snapshot(handle: SceneHandle): SceneSnapshot;
}
```

## Integration
Streaming decides residency; Prefab/Structure provide authored composition; Entity/World systems execute behavior; Renderer/Audio/UI consume presentation state.

## Tests
Cross-scene references, streaming activation, restart/reload, missing resources, headless scene loading and deterministic scene instantiation.

## Invariants
- Scene lifecycle cannot bypass world authority.
- Unloading a scene must release owned runtime resources.
- Scene data remains versionable and mod-compatible.
