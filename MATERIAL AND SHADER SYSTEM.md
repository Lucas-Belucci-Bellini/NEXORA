# NEXORA — MATERIAL / SHADER SYSTEM

> Materials and shaders define visual appearance and GPU programs without embedding rendering details into gameplay systems.

## Layers
```text
Material Definition
→ Material Instance
→ Shader Graph / Shader Program
→ Render Pipeline
→ RHI
→ GPU
```

## Material definition
Supports surface properties, textures, parameters, blend mode, culling, lighting model and feature flags.

## Shader architecture
- platform-independent material graph where practical;
- compiled shader variants;
- reflection/resource binding metadata;
- permutation control to avoid combinatorial explosion;
- hot reload in development;
- validated immutable release artifacts.

## Data driven
Blocks, entities, particles, terrain, UI and structures reference materials by registry/resource IDs.

## Streaming
Textures, shaders and material instances use Resource/Asset and Streaming systems with priority and eviction policies.

## API
```ts
interface IMaterialSystem {
  register(definition: MaterialDefinition): MaterialID;
  instantiate(id: MaterialID, overrides: MaterialOverrides): MaterialInstance;
  compile(material: MaterialInstance, target: RenderTarget): ShaderHandle;
}
```

## Security
Content Pipeline validates shader source/assets and release packaging rules. Untrusted content cannot silently execute arbitrary host processes through the asset pipeline.

## Tests
Shader variant determinism, resource lifetime, material hot reload, invalid graph rejection, backend compatibility and memory budgets.

## Invariants
- Gameplay never depends on shader implementation details.
- RHI remains the backend boundary.
- Material definitions are data-driven and versionable.
