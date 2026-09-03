# NEXORA — RESOURCE PACK SYSTEM

> Resource Packs provide validated asset/data overrides without bypassing Resource, Registry, Security or Mod Runtime rules.

## Contents
Textures, models, materials, sounds, music, fonts, UI assets, particles, animation assets and explicitly overrideable data.

## Layering
`Base Game → Official Pack → User Pack → Mod Pack → Development Override` with deterministic precedence and conflict diagnostics.

## Compatibility
Packs declare game/content/API compatibility, namespaces and dependencies.

## Security
Validate archive paths, sizes, formats and manifests; no arbitrary code execution by default.

## Integration
Resource System resolves/loads assets; Content Pipeline validates data; Launcher manages profile selection.

## Tests
Priority resolution, conflicting overrides, missing assets, disable/enable and fingerprint stability.
