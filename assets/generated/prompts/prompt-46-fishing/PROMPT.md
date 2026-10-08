# NEXORA — Production prompt 46: Fishing & Aquatic Work Props

## Mission
Create original, production-ready NEXORA art assets for **Fishing & Aquatic Work Props**. This prompt is isolated: assets created for it must never be mixed with another prompt's output.

## Scope
Equipamentos de pesca e trabalho aquático, com silhuetas claras e peças adequadas ao inventário.

## Required asset families
1. varas e molinetes
2. redes, boias e armadilhas de pesca
3. caixas de iscas e equipamento
4. baldes e cestos de pescado
5. arpões estilizados não realistas
6. âncoras e cordas
7. barris e caixas de convés
8. equipamento de mergulho e coleta aquática

Treat this list as the starting scope, not permission to invent unrequested systems. First inspect the existing project, current asset catalog, source issues, naming rules, and art direction. Identify missing variants and avoid duplicating assets that already exist. If a source issue defines exact counts, IDs, dimensions, or priorities, those requirements take precedence.

## Image-generation protocol
1. **Inspect before generating:** read this entire prompt and relevant project art-direction/pipeline documentation; inspect existing images and manifests; enumerate every intended asset, variant, stable asset ID, view, and required count.
2. **Originality:** create original NEXORA designs. Never copy, trace, extract, or closely reconstruct assets from Minecraft, mods, other games, brands, or third-party art. Use broad genre conventions only.
3. **Correct output type:** determine whether each asset is a 2D texture, tileable material, icon, sprite, or isolated prop render. Do not substitute a concept sheet for required individual production files.
4. **Exact dimensions:** follow the project's source pipeline. If a texture is specified as 16×16, the final file must be exactly 16×16 pixels. Do not resize a larger image and pretend it is native pixel art.
5. **Visual quality:** use clear silhouettes, controlled pixel clusters, readable value contrast, intentional palette choices, and consistent scale/material language. Keep voxel/pixel-art style coherent with existing NEXORA assets.
6. **Avoid defects:** no accidental blur, unwanted anti-aliasing, photorealistic rendering, unrelated scenery, watermark, logo, or text unless the source explicitly requires it. Use transparency only when required.
7. **Tileability:** for tileable materials, test edges and a repeated 3×3 tiling preview. Reject visible seams, lighting discontinuities, and stray edge pixels.
8. **Props and icons:** isolate each asset on a clean neutral or transparent background as required. Keep framing consistent; do not let shadows, decorations, or adjacent objects merge with the asset.
9. **Review and regenerate:** inspect candidates at native size and enlarged nearest-neighbor scale. Validate dimensions, format, alpha, readability, silhouette, duplicate risk, naming, and source requirements. Regenerate failed assets instead of silently accepting them.
10. **Traceability:** record the stable asset ID, exact filename, source prompt, generation tool/model when known, generation date, dimensions, format, version, originality review, and validation status. Never invent metadata.
11. **No false completion:** do not claim that an image was generated, saved, inspected, or validated unless the actual file exists and the relevant check was performed.

## Output contract
Save final production images only under `assets/generated/prompts/prompt-46-fishing/images/`. Use lowercase, stable, descriptive filenames with no spaces. Keep previews/contact sheets clearly separated from production assets and never use a contact sheet as a replacement for individual files.

Create or update `manifest.json` inside that same `images/` directory. Each record should include at least: `asset_id`, `filename`, `source_prompt`, `generation_tool` (or `unknown`), `generation_date` (or `unknown`), `width`, `height`, `format`, `version`, `status`, `originality_review`, and `validation_results`. Include SHA-256 when it can be calculated reliably.

## Completion report
At the end, report: requested count, generated count, accepted count, rejected/regenerated count, exact output paths, validation performed, missing requirements, and any blockers. Separate verified facts from recommendations.