# NEXORA — Production prompt 56: Snow & Ice Props

## Mission
Create original, production-ready NEXORA assets for **Snow & Ice Props**. Keep all outputs isolated to this prompt folder.

## Scope
Objetos de ambiente frio, regiões nevadas, geleiras e instalações polares.

## Required asset families
1. blocos e formações de gelo
2. estalactites e estalagmites de gelo
3. marcadores de rota na neve
4. abrigos polares
5. caixas de suprimentos térmicos
6. equipamentos de exploração fria
7. luzes e balizas de emergência
8. estruturas parcialmente cobertas de neve

## Before generation
Read this prompt and relevant project art-direction/pipeline documentation. Inspect existing assets and manifests, identify missing items, variants, IDs, counts, and dimensions, and avoid duplicates. Explicit source requirements take precedence.

## Image-generation protocol
- Create original NEXORA artwork; do not copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Determine whether each output is a texture, tileable material, icon, sprite, or isolated prop.
- Follow exact pipeline dimensions. Required 16×16 textures must be natively 16×16 pixels.
- Maintain coherent voxel/pixel-art style, readable silhouettes, intentional pixel clusters, clear contrast, and consistent scale.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, unrelated scenery, watermarks, logos, and unrequested text. Use transparency only when appropriate.
- Tileable textures must pass a repeated 3×3 seam check. Isolate props and keep framing consistent.
- Inspect at native size and enlarged nearest-neighbor scale. Validate dimensions, format, transparency, readability, silhouette, duplicates, filenames, and requirements; regenerate failures.
- Contact sheets are previews only and never replace individual production files.
- Record only verifiable metadata; use `unknown` when data is unavailable. Never claim an image was generated, saved, inspected, or validated unless the actual file exists and the check occurred.

## Output contract
Save final images only in `assets/generated/prompts/prompt-56-snow-ice/images/`. Use stable lowercase filenames. Maintain `manifest.json` in that folder with asset ID, filename, source prompt, tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets rather than overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, validation performed, missing requirements, and blockers. Distinguish verified facts from recommendations.