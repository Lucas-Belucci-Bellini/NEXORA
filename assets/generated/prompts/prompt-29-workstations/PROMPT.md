# NEXORA — Production prompt: Workstations & Workbenches

## Mission
Generate original NEXORA workstation and workbench assets for the voxel sandbox world. This prompt is isolated: every production image belongs only to this folder.

## Asset families
- Basic workbench / crafting table
- Tool bench
- Weapon bench
- Armor bench
- Engineering bench
- Research table
- Alchemy table
- Enchanting table
- Cartography table
- Sewing table
- Repair bench
- Anvil
- Stone/metal cutter
- Industrial press
- Foundry
- Furnace / smelter
- Electronics bench

## Image-generation protocol
- Generate original NEXORA artwork; do not copy or reconstruct third-party assets.
- First establish a coherent family language: voxel-readable silhouette, consistent proportions, material separation, restrained detail, and a shared palette logic.
- Generate individual production assets, not only a contact sheet. A reference sheet may accompany them but never replaces the individual files.
- Preserve the project's required 16×16 texture target whenever the source pipeline specifies it. Do not upscale/downscale a final asset merely to fit a preview.
- For block textures, produce clean tileable faces and validate seams with a repeated 3×3 test. For 3D props, keep the production render isolated from scenery and UI.
- No accidental text, watermark, logo, photorealism, blur, unwanted anti-aliasing, or unrelated background.
- If transparency is required, use intentional alpha and validate the silhouette.
- Generate, inspect, reject failures, and regenerate rather than accepting ambiguous outputs.

## Output contract
Save production images only under `assets/generated/prompts/prompt-29-workstations/images/` using stable lowercase filenames. Create/update `manifest.json` with asset ID, filename, dimensions, format, generation metadata when available, version, and validation status. Never place unrelated assets here.
