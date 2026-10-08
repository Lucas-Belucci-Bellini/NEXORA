# NEXORA — Production prompt 63: Doors & Gates

## Mission
Create original, production-ready NEXORA assets for **Doors & Gates**. Keep all outputs isolated to this prompt folder.

## Scope
Portas, portões, escotilhas e mecanismos de passagem para ambientes e estruturas.

## Required asset families
1. portas de madeira
2. portas metálicas
3. portas reforçadas
4. portões de cerca
5. escotilhas industriais
6. portas automáticas futuristas
7. dobradiças e maçanetas
8. mecanismos visuais de abertura

## Before generation
Read this prompt and relevant project art-direction/pipeline documentation. Inspect existing assets and manifests; enumerate missing assets, variants, stable IDs, counts, and required dimensions. Avoid duplicates. Explicit source issue requirements take precedence.

## Image-generation protocol
- Create original NEXORA artwork. Never copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Determine whether each output is a texture, tileable material, icon, sprite, or isolated prop.
- Follow exact pipeline dimensions. If a texture is specified as 16×16, the final texture must be natively 16×16 pixels.
- Maintain coherent voxel/pixel-art style, clear silhouettes, controlled pixel clusters, readable contrast, and consistent scale/material language.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, unrelated scenery, watermarks, logos, and unrequested text. Use transparency only when appropriate.
- Tileable textures must pass a repeated 3×3 seam check. Isolate props/icons and keep framing consistent.
- Inspect at native size and enlarged nearest-neighbor scale. Validate dimensions, format, alpha, readability, silhouette, duplicate risk, naming, and source requirements; regenerate failures.
- Contact sheets are previews only and never replace individual production files.
- Record only verifiable metadata; use `unknown` where unavailable. Never claim generation, saving, inspection, or validation unless the actual file exists and the check occurred.

## Output contract
Save final images only in `assets/generated/prompts/prompt-63-doors-gates/images/`. Use stable lowercase filenames. Maintain `manifest.json` in that directory with asset ID, filename, source prompt, generation tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets rather than overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, validations performed, missing requirements, and blockers. Separate verified facts from recommendations.