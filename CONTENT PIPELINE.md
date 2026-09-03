# NEXORA — CONTENT PIPELINE

> Turns authored data/assets into validated, reproducible runtime content.

```text
Author → Import → Normalize → Validate → Compile → Optimize → Package → Fingerprint → Registry/Resource → Runtime
```

## Inputs
Blocks, items, entities, recipes, biomes, structures, audio, textures, models, animations, UI, localization, quests, technologies, events and civilization data.

## Validation
Schemas, namespaces, references, dependency versions, asset limits, security, license/provenance metadata and duplicate IDs.

## Integration
Official content and mods use the same pipeline. Resource System owns runtime loading; Mod Runtime owns execution; Registry owns identity.

## Tests
Deterministic build, incremental rebuild, dependency failure, malformed asset, fingerprint stability.

## Invariants
Runtime consumes validated packages; failed builds never publish partial packages.
