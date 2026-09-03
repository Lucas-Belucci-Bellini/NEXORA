# NEXORA — RESOURCE / ASSET SYSTEM

> Resources are addressable data or runtime assets. The Resource System resolves, validates, loads, caches and unloads them without making gameplay systems own asset lifetime.

## Scope
```text
ResourceID
→ Manifest
→ Resolver
→ Loader
→ Compiler / Decoder
→ Cache
→ Runtime Handle
```

Supported families include textures, meshes, materials, shaders, audio, fonts, animations, UI resources, data files, world-generation definitions and mod resources.

## Identity
Use stable namespaced IDs such as `nexora:textures/block/stone`. Source paths are implementation details.

## Resource lifecycle
`DISCOVERED → VALIDATED → RESOLVING → LOADING → READY → EVICTABLE → UNLOADED`.

## Ownership
The Resource System owns resource lifetime. Renderer, Audio, UI and gameplay systems hold typed handles rather than manipulating raw files directly.

## Dependencies
Resources can depend on other resources. Dependency cycles are rejected. Missing optional resources can use declared fallbacks; missing required resources fail the owning package or definition cleanly.

## Resource packs
Support layered packs with explicit precedence, namespace isolation and provenance metadata. Official and mod assets use the same runtime contract.

## Compilation
```text
source asset
→ validate
→ import
→ optimize
→ compile
→ package
→ runtime resource
```

Compilation must be reproducible where possible and record tool/version metadata.

## Cache
Separate memory cache, persistent disk cache and GPU/runtime cache. Every cache entry is derived and disposable.

## Hot reload
Safe for data and development assets. Runtime code or incompatible GPU resource changes may require restart.

## Security
Validate archive paths, file types, sizes, decompression ratios, dependency references and namespace ownership. Untrusted resources never gain filesystem or process access.

## API sketch
```ts
interface IResourceManager {
  resolve<T>(id: ResourceID, type: ResourceType): ResourceHandle<T>;
  load<T>(handle: ResourceHandle<T>): Promise<Resource<T>>;
  unload(id: ResourceID): void;
  invalidate(id: ResourceID): void;
  inspect(id: ResourceID): ResourceInfo;
}
```

## Integration
- Registry owns content identity.
- Mod Runtime supplies package resources.
- Renderer consumes graphics resources.
- Audio consumes sound resources.
- Animation consumes clips/rigs.
- UI consumes layouts/styles/fonts.
- Persistence records references, not derived resource data.

## Debug
`nexora resource list`, `inspect`, `deps`, `cache`, `reload`, `missing`, `provenance`.

## Tests
Resource resolution, dependency order, cache eviction, hot reload, missing asset fallback, malformed package rejection and reproducible compilation.

## Invariants
- Resource IDs are stable.
- Runtime code never assumes a source filesystem path.
- Cache contents are always rebuildable.
- Package resources cannot escape their declared namespace.
