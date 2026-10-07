# NEXORA — PREFAB / ASSEMBLY SYSTEM

> Prefabs are reusable authored assemblies of entities, blocks, components and references. They bridge authoring tools and runtime without becoming a special-case gameplay system.

## Model
```text
Prefab Definition
→ Validation
→ Instance
→ Resolve Components / References
→ Runtime Entity / Structure Assembly
```

## Uses
- NPC and mob assemblies;
- vehicles;
- machines;
- buildings;
- modular structures;
- starter scenes/world templates;
- editor prototypes;
- mod-provided assemblies.

## Separation
Structure System handles multi-block/world structures. Entity System handles runtime entities. Prefab stores reusable composition and initialization data.

## Variants
Support inheritance/composition, overrides, anchors, sockets, optional components and transform variants without requiring code generation.

## API
```ts
interface IPrefabSystem {
  register(definition: PrefabDefinition): void;
  instantiate(request: PrefabInstanceRequest): PrefabInstance;
  validate(definition: PrefabDefinition): ValidationReport;
  apply(instance: PrefabInstance): Result;
}
```

## Persistence
Save the logical prefab reference plus instance overrides. Do not duplicate immutable definition data into every save unless required for migration.

## Networking
Server-authoritative instantiation. Network stable instance identity and state deltas, not editor-only source graphs.

## Mods
Mods can register prefabs through the Content Pipeline and Registry using namespace ownership.

## Tests
Nested assemblies, missing references, variant overrides, migration, deterministic instantiation and chunk-crossing structures.

## Invariants
- Prefab is authored composition, not hidden gameplay logic.
- Runtime state remains authoritative and serializable.
