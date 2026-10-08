# NEXORA — Production prompt 112: Fungi & Mushroom Assets

## Mission
Create original, game-ready NEXORA assets for **Fungi & Mushroom Assets**, isolated to this prompt's folder and compatible with the established project art direction.

## Required asset families
1. cogumelos pequenos e grandes
2. fungos luminosos originais
3. colônias de fungos
4. esporos como efeito visual
5. troncos colonizados
6. fungos de cavernas
7. fungos de clima úmido
8. variantes seguras e decorativas

## Before generation
Read this prompt and the relevant art-direction, animation, texture, and asset-pipeline documentation. Inspect current assets and manifests. Enumerate missing asset IDs, variants, counts, dimensions, and file formats. Avoid duplicates and preserve existing approved versions.

## Generation requirements
- Create original designs. Do not copy, trace, extract, or closely reconstruct third-party games, mods, brands, or artwork.
- Determine whether each output is a texture, tileable material, sprite, icon, animation frame, particle sheet, or isolated asset.
- Follow repository-specific dimensions exactly; when 16×16 is required, produce a native 16×16 image rather than downscaling a larger image.
- Use coherent voxel/pixel-art language, crisp silhouettes, deliberate pixel clusters, consistent lighting, restrained palettes, and strong readability at actual game scale.
- For creatures, maintain consistent proportions across variants and make silhouettes distinguishable. For particles/effects, avoid visual noise and preserve gameplay readability.
- Avoid unwanted anti-aliasing, blur, photorealism, watermarks, logos, and unrequested text.
- Tileable materials must pass a 3×3 seam test. Inspect transparent edges and backgrounds for sprites.
- Inspect actual files at native size and enlarged nearest-neighbor scale. Validate dimensions, format, alpha, naming, readability, duplicates, and pipeline compatibility. Flag or regenerate failures.
- Contact sheets are previews only and do not replace individual assets.
- Record only verified metadata; use `unknown` when unavailable. Never claim generation, saving, or validation without checking the actual file.

## Output contract
Save outputs only in `assets/generated/prompts/prompt-112-flora-fungi/images/`. Use stable lowercase filenames and IDs. Maintain `manifest.json` with asset ID, filename, source prompt, generation tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved files instead of overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, paths, validation checks, missing requirements, and blockers. Clearly distinguish verified facts from recommendations.