# NEXORA — RESOURCE / ASSET SYSTEM

> Resources are addressable data or runtime assets. The Resource System resolves, validates, loads, caches and unloads them without making gameplay systems own asset lifetime.

## Pipeline
`ResourceID → Manifest → Resolver → Loader → Cache → Runtime Handle`.

Supports textures, meshes, materials, shaders, audio, fonts, animations, UI resources and data-driven content.

## Rules
Stable namespaced IDs, explicit dependencies, reproducible compilation, layered resource packs, bounded caches and safe hot reload for development data.

## Security
Validate package paths, file types, sizes, decompression ratios and dependencies. Untrusted resources have no filesystem/process privileges.

## API
```ts
interface IResourceManager {
  resolve<T>(id: ResourceID, type: ResourceType): ResourceHandle<T>;
  load<T>(handle: ResourceHandle<T>): Promise<Resource<T>>;
  unload(id: ResourceID): void;
  invalidate(id: ResourceID): void;
}
```

## Invariants
- Runtime uses resource IDs, not source paths.
- Cache data is disposable.
- Missing optional resources use declared fallbacks.
