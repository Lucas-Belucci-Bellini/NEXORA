# NEXORA — ART DIRECTION AND PROCEDURAL VARIATION

## Principle
NEXORA should pursue a recognizable original visual identity while using procedural variation to increase richness without multiplying storage requirements unnecessarily.

## Asset model
```text
ORIGINAL BASE ASSET
+
MATERIAL PARAMETERS
+
PROCEDURAL VARIATION
+
WORLD CONTEXT
+
INSTANCE STATE
=
FINAL RUNTIME APPEARANCE
```

## Example: stone
A stone family may define:
```text
shape
albedo range
roughness range
normal/detail patterns
fracture patterns
weathering
moisture response
ageing
biome/geology modifiers
```

Individual world instances can vary through deterministic seeds and context without requiring an independent source texture for every block.

## Visual identity layers
```text
BASE STYLE
→ MATERIAL LANGUAGE
→ BIOME LANGUAGE
→ ARCHITECTURAL LANGUAGE
→ CIVILIZATION LANGUAGE
→ TECHNOLOGY ERA
→ INDIVIDUAL VARIATION
```

## Determinism
Procedural appearance must use reproducible seeds when the result affects save state, networking or authoritative simulation.

## Asset quality
Procedural generation must not become an excuse for low-quality repeated patterns. Base assets should be authored to support meaningful variation.

## Pipeline
```text
AUTHOR
→ IMPORT
→ VALIDATE
→ OPTIMIZE
→ REGISTER
→ COOK
→ PACKAGE
→ STREAM
→ RENDER
```

## Performance
Visual variation must respect texture, material, mesh, shader and streaming budgets. Variants should be batched where possible.

## Provenance
Every original and generated asset follows the provenance requirements in the Asset Provenance and License Registry.
