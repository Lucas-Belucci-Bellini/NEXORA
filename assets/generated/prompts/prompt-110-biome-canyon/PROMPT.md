# NEXORA — Production prompt 110: Canyon & Badlands Biome Assets

## Mission
Create original, game-ready NEXORA assets for **Canyon & Badlands Biome Assets**, keeping every output isolated to this prompt's folder and consistent with the project's art direction.

## Required asset families
1. paredes estratificadas
2. rochas erodidas
3. arcos naturais
4. colunas de arenito
5. cascalho e solo árido
6. pedras em camadas de cor
7. vegetação esparsa
8. materiais de desfiladeiro

## Before generation
Read this prompt and relevant art-direction, texture, and asset-pipeline documentation. Inspect existing assets and manifests; list exact missing IDs, variants, counts, and required dimensions. Reuse valid existing assets when appropriate and avoid duplicates.

## Generation requirements
- Produce original artwork; do not copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Determine whether each item is a tileable texture, block material, sprite, icon, foliage card, or isolated prop.
- Follow the repository's exact dimension and format requirements. If a texture is specified as 16×16, create it natively at 16×16.
- Preserve a coherent voxel/pixel-art style, readable silhouettes, controlled pixel clusters, consistent lighting direction, and a clear material palette.
- Biome variants must be visually distinct without making neighboring assets stylistically inconsistent.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, watermarks, logos, unrequested text, and unrelated scenery.
- Tileable textures must pass a 3×3 repetition seam check. Check foliage and small objects against both light and dark backgrounds where useful.
- Inspect actual output at native size and enlarged nearest-neighbor scale. Validate dimensions, format, alpha, silhouette, readability, filenames, duplicates, and project compatibility. Regenerate or report failures.
- Contact sheets are previews only; keep each production asset as its own file.
- Record only verified metadata. Use `unknown` when information is unavailable and never claim an asset was generated or validated unless its actual file was checked.

## Output contract
Save outputs only in `assets/generated/prompts/prompt-110-biome-canyon/images/`. Use stable lowercase filenames and IDs. Maintain `manifest.json` with asset ID, filename, source prompt, generation tool/model/date if known, dimensions, format, version, status, originality review, and validation results. Do not overwrite approved versions; version replacements.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, checks performed, missing items, and blockers. Separate verified facts from recommendations.