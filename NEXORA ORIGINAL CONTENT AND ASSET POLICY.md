# NEXORA — ORIGINAL CONTENT AND ASSET POLICY

## Principle
NEXORA is intended to ship with an original content stack. Code, assets, lore, data and presentation created for NEXORA must have known provenance and an explicit distribution status.

## Default rule
```text
UNKNOWN ORIGIN
→ BLOCK

CLEAR ORIGINAL / CLEARED LICENSE
→ ELIGIBLE
```

## Covered material
- engine/runtime code;
- shaders and materials;
- textures and procedural materials;
- meshes, models and animations;
- VFX and particles;
- audio, music and voice;
- UI and fonts;
- world-generation data;
- game data and configuration;
- documentation and examples;
- lore, characters, factions and narrative content.

## Originality boundary
NEXORA must not knowingly distribute copied source code, extracted assets, leaked data, proprietary files or content whose license does not permit the intended use.

Architectural inspiration may be studied. Implementation, assets and distinctive protected expression must be independently created or properly licensed.

## Asset status
Every distributable asset should have one of these statuses:

```text
NEXORA_ORIGINAL
NEXORA_DERIVED_FROM_OWN_SOURCE
THIRD_PARTY_LICENSED
EDITOR_ONLY
EXPERIMENTAL
BLOCKED
```

## Release gate
An asset cannot enter a release artifact without:
- source/provenance record;
- creator or source identification;
- license status;
- modification history;
- redistribution permission where applicable;
- target package status.

## Procedural content
Procedural generation is encouraged to increase variation without requiring a unique source file for every world instance.

```text
ORIGINAL BASE ASSETS
+
PROCEDURAL PARAMETERS
+
WORLD SEED
+
SIMULATION STATE
=
RUNTIME VARIATION
```

## Review rule
Automated generation does not bypass provenance review. Generated material must be checked for source contamination, tool/license constraints and unintended third-party resemblance before release.

## Goal
Prefer a small library of high-quality original primitives plus controlled procedural variation over massive duplication of near-identical files.
