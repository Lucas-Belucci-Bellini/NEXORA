# NEXORA — RESOURCE PACK SYSTEM

> Resource Packs are validated presentation/content overlays. They provide assets and compatible data without bypassing Registry, Resource, Security or Mod Runtime rules.

## Contents
```text
textures
models
materials
sounds
music
fonts
UI assets
particles
animation assets
optional data overrides
```

## Layering
`Base Game → Official Pack → User Pack → Mod Pack → Development Override`, with explicit precedence and conflict diagnostics.

## Compatibility
Each pack declares game/content/API compatibility, namespaces and dependencies. Incompatible packs are rejected or disabled with actionable diagnostics.

## Asset loading
The Resource System owns resolution, loading, caching and lifetime. Packs provide sources only.

## Data overrides
Only definitions marked as overrideable can be replaced. Authoritative schemas remain validated by the Content Pipeline.

## API sketch
```ts
interface IResourcePackSystem {
  register(pack: ResourcePackManifest): void;
  resolve(request: ResourceRequest): ResourceSource;
  validate(pack: ResourcePack): ValidationReport;
  enable(id: PackID): Result;
  disable(id: PackID): Result;
}
```

## Security
Validate archive paths, sizes, formats and manifests. Never permit path traversal or hidden executable injection.

## Persistence
Active pack configuration is local/profile state. World saves record relevant content fingerprints for compatibility.

## Tests
Priority ordering, missing assets, conflicting overrides, pack disable/enable and fingerprint stability.

## Invariants
- Packs cannot execute arbitrary code by default.
- Runtime resources remain addressable by stable IDs.
