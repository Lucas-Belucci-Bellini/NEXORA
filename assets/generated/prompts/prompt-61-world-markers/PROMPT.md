# NEXORA — Production prompt 61: World Signals & Markers

## Mission
Create original, production-ready NEXORA assets for **World Signals & Markers**. Keep all outputs isolated to this prompt folder.

## Scope
Marcadores visuais de mundo, pontos de interesse, fronteiras e orientação.

## Required asset families
1. marcos de trilha
2. totens de ponto de interesse
3. balizas de fronteira
4. marcadores de recursos
5. postes com bandeirolas
6. marcadores de perigo ambiental
7. placas de descoberta
8. monólitos de navegação

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
Save final images only in `assets/generated/prompts/prompt-61-world-markers/images/`. Use stable lowercase filenames. Maintain `manifest.json` in that directory with asset ID, filename, source prompt, generation tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets rather than overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, validations performed, missing requirements, and blockers. Separate verified facts from recommendations.