# NEXORA — Production prompt 51: Signs & Wayfinding

## Mission
Create original, production-ready NEXORA art for **Signs & Wayfinding**. Keep all outputs isolated to this prompt's folder.

## Scope
Placas, indicadores e elementos de orientação visual legíveis no estilo voxel/pixel art.

## Required asset families
1. placas direcionais
2. marcadores de rota
3. placas de perigo e aviso
4. identificadores de salas
5. totens informativos
6. placas de transporte
7. marcos de trilha
8. sinalização de instalações

Before generating, read this prompt and relevant project documentation, inspect existing assets/manifests, and identify exact missing assets, variants, IDs, dimensions, and counts. Source issue requirements override assumptions. Do not duplicate assets already available.

## Image-generation protocol
- Create original NEXORA designs; never copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Identify the correct output type for each asset: texture, tileable material, icon, sprite, or isolated prop.
- Follow the source pipeline's exact dimensions. A required 16×16 texture must be natively 16×16, not resized from a larger render.
- Maintain coherent voxel/pixel-art styling, readable silhouettes, intentional pixel clusters, controlled palette, and clear material separation.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, unrelated scenery, watermark, logo, and unrequested text. Use transparency only when appropriate.
- For tileable textures, validate seams in a repeated 3×3 preview. For props/icons, use consistent framing and isolate each production asset.
- Review at native size and enlarged nearest-neighbor scale. Validate dimensions, format, transparency, readability, silhouette, duplication, filenames, and requirements. Regenerate failed candidates.
- Contact sheets and previews never replace required individual files.
- Record only verifiable metadata: asset ID, filename, source prompt, tool/model and date when known, dimensions, format, version, originality review, and validation status. Use `unknown` rather than inventing data.
- Never claim generation, saving, inspection, or validation unless the real file exists and the action occurred.

## Output contract
Save final images only in `assets/generated/prompts/prompt-51-signs-wayfinding/images/`. Use stable lowercase filenames. Keep previews separate and do not overwrite approved assets without versioning. Maintain `manifest.json` in this images directory with `asset_id`, `filename`, `source_prompt`, `generation_tool`, `generation_date`, `width`, `height`, `format`, `version`, `status`, `originality_review`, and `validation_results`. Include SHA-256 only when reliably calculated.

## Completion report
Report requested, generated, accepted, rejected/regenerated counts, exact paths, validations performed, missing requirements, and blockers. Distinguish verified facts from recommendations.