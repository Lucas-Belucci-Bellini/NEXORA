# NEXORA — Production prompt 90: Baluarte Archives & Records Props

## Mission
Create original, production-ready NEXORA art assets for **Baluarte Archives & Records Props**. Keep all outputs isolated to this prompt's folder.

## Required asset families
1. estantes de arquivos
2. caixas documentais
3. terminais de consulta fictícios
4. armários de registros
5. mesas de catalogação
6. cofres de documentos
7. marcadores de setor originais
8. módulos de armazenamento de dados

## Pre-generation requirements
Read this prompt and relevant project art-direction/pipeline documentation. Inspect existing repository assets and manifests; enumerate missing assets, variants, stable IDs, counts, and dimensions. Avoid duplicates. Explicit source issue requirements take precedence.

## Image-generation protocol
- Create original NEXORA artwork; never copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Identify the output type: texture, tileable material, icon, sprite, or isolated prop.
- Follow exact pipeline dimensions; required 16×16 textures must be natively 16×16 pixels.
- Maintain coherent voxel/pixel-art styling, readable silhouettes, controlled pixel clusters, clear contrast, and consistent scale/material language.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, unrelated scenery, watermarks, logos, and unrequested text. Use transparency only when appropriate.
- Tileable textures must pass a repeated 3×3 seam check. Isolate props/icons and keep framing consistent.
- Inspect at native size and enlarged nearest-neighbor scale. Validate dimensions, format, alpha, readability, silhouette, duplicate risk, filenames, and source requirements; regenerate failures.
- Contact sheets are previews only and never replace individual production files.
- Record only verifiable metadata; use `unknown` when unavailable. Never claim an image was generated, saved, inspected, or validated unless the actual file exists and the check occurred.

## Output contract
Save final images only in `assets/generated/prompts/prompt-90-baluarte-archives/images/`. Use stable lowercase filenames. Maintain `manifest.json` in that directory with asset ID, filename, source prompt, generation tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets instead of overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, validation performed, missing requirements, and blockers. Distinguish verified facts from recommendations.