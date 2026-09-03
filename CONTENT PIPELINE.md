# NEXORA — CONTENT PIPELINE

> The Content Pipeline turns authored source data and assets into validated, reproducible runtime content. It never hardcodes domain behavior into Core.

## Pipeline
```text
Authoring
→ Import
→ Normalize
→ Validate
→ Compile
→ Optimize
→ Package
→ Fingerprint
→ Registry / Resource System
→ Runtime
```

## Inputs
Blocks, items, entities, recipes, biomes, structures, sounds, textures, models, animations, UI, localization, quests, technologies, events, civilizations and mod metadata.

## Validation
Schema, namespace ownership, references, dependency versions, asset dimensions/types, duplicate IDs, unsupported fields, security limits and license/provenance metadata.

## Determinism
Compilation should be reproducible for the same source, toolchain and configuration. Generated IDs must be stable.

## Build artifacts
Packages contain manifest, content registry data, resource fingerprints, build metadata and optional debug symbols/source maps.

## Incremental builds
Track source hashes and dependency graphs so unchanged resources are not recompiled.

## Environment
Development builds can keep source mapping and hot reload. Release builds favor compact, validated artifacts.

## Mod integration
Official content and community content enter the same validation and registry pipeline. Trust level changes allowed capabilities, not basic content structure.

## API sketch
```ts
interface IContentPipeline {
  validate(input: ContentSource): ValidationReport;
  build(input: ContentSource, options: BuildOptions): BuildArtifact;
  package(artifact: BuildArtifact): ContentPackage;
  fingerprint(pkg: ContentPackage): ContentFingerprint;
}
```

## Security
Reject path traversal, executable payloads where not explicitly allowed, oversized assets, malformed archives and invalid references. Preserve provenance information for third-party content.

## Debug
`nexora content validate`, `build`, `package`, `deps`, `fingerprint`, `diff`.

## Tests
Schema validation, dependency ordering, deterministic builds, incremental rebuilds, missing dependency failure, invalid asset rejection and package fingerprint stability.

## Invariants
- Runtime consumes validated content.
- Build output is traceable to inputs.
- A failed content build cannot silently publish a partial package.
