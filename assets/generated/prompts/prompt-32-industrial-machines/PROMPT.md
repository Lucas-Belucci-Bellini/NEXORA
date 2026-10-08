# NEXORA — Production prompt: Industrial Machines

## Mission
Generate original NEXORA assets for **Industrial Machines**. This prompt is isolated: all production images belong only to this folder.

## Scope
Geradores, motores, compressores, prensas, trituradores, máquinas de mineração, processamento e fundição, esteiras, tanques e bombas.

## Image-generation protocol
- Read this prompt and the project art-direction/pipeline rules before generating.
- Generate original NEXORA designs. Do not copy, trace, extract, or closely reproduce third-party assets.
- Generate individual production files; contact sheets are references only and never replace the individual files.
- Preserve the source pipeline's exact dimensions. For 16×16 textures, the final texture itself must be exactly 16×16.
- Keep voxel/pixel-art readability, intentional silhouettes, controlled pixel clusters, clear material separation, and consistent family proportions.
- No accidental blur, unwanted anti-aliasing, photorealistic rendering, watermark, logo, unrelated scenery, or text unless explicitly requested.
- Use true transparency only when the asset requires it.
- Tileable textures must pass a repeated 3×3 seam check. Props must remain isolated from unrelated backgrounds.
- Inspect generated candidates and regenerate failures instead of accepting ambiguous results.

## Output contract
Save production images only in `assets/generated/prompts/prompt-32-industrial-machines/images/`. Use stable lowercase filenames and asset IDs. Keep previews separate from final assets.

Create/update `manifest.json` with asset ID, filename, dimensions, format, generation metadata when available, version, and validation status.

Never claim an asset exists until the actual file has been saved and inspected.
