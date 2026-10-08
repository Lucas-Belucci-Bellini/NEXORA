# NEXORA — Production prompt 55: Desert Environment Props

## Mission
Create original, production-ready NEXORA assets for **Desert Environment Props**. Keep all outputs isolated to this prompt folder.

## Scope
Objetos e pequenos elementos ambientais de desertos, cânions e regiões áridas.

## Required asset families
1. rochas erodidas
2. ossos fossilizados estilizados
3. cactos e plantas resistentes
4. tendas e abrigos de viagem
5. marcadores de trilha
6. vasilhas e recipientes de água
7. ruínas parcialmente soterradas
8. equipamentos de proteção contra areia

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
Save final images only in `assets/generated/prompts/prompt-55-desert-environment/images/`. Use stable lowercase filenames. Maintain `manifest.json` in that folder with asset ID, filename, source prompt, tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets rather than overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, validation performed, missing requirements, and blockers. Distinguish verified facts from recommendations.