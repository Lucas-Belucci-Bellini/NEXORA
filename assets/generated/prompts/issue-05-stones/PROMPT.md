# NEXORA — Production prompt #5: Stones

> **Source of truth:** [ISSUE-05-stones-16x16.md](../../../../docs/assets/prompts/issues/ISSUE-05-stones-16x16.md) · [Original GitHub Issue #5](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/5)

## Mission

Execute the complete source prompt linked above. Read it from beginning to end before generating anything. This file strengthens its execution requirements; it does not remove or silently override the source prompt's requested assets, counts, names, IDs, or constraints.

## Image-generation protocol

When an image-generation model is used, treat generation as a controlled production pipeline rather than a single free-form prompt.

### Before generation
- Read the source prompt and enumerate every requested asset, variant, count, identifier, material, and constraint.
- Inspect existing NEXORA assets and manifests to prevent duplicates and accidental visual drift.
- Decide whether the output is a tileable texture, isolated item/prop, block, UI asset, VFX element, or reference sheet before generating it.

### Generation requirements
- Generate **original NEXORA artwork**. References may guide abstract properties such as readability, material separation, voxel-scale detail, or technical clarity, but must not reproduce a specific third-party asset or distinctive design.
- Follow the source prompt's exact dimensions. When it calls for 16×16 textures, the final texture itself must be exactly 16×16; do not submit a resized preview as the final asset.
- Generate clean, game-ready images with intentional silhouettes, controlled pixel clusters, readable values/materials, and no accidental blur, anti-aliasing, photorealistic rendering, scene background, watermark, logo, text, or UI chrome unless explicitly requested.
- For transparent assets, use true transparency only where required and keep the silhouette/padding intentional.
- For tileable textures, make opposite edges compatible and test a repeated 3×3 tiling. Do not rely on a contact sheet to prove tiling.
- For props/objects, generate an isolated asset or orthographic/reference presentation appropriate to the source prompt; do not bake unrelated scenery into the production asset.
- If the generator returns multiple candidates, inspect all candidates and select only those that satisfy the checklist. Regenerate failures instead of silently accepting them.

### Post-generation validation
- Open the actual generated files and inspect them at native resolution and enlarged nearest-neighbor scale when pixel art is involved.
- Validate dimensions, file format, alpha/transparency, tiling where applicable, filename/asset ID, visual readability, and duplicate risk.
- Keep previews/contact sheets separate from production assets. Every required production asset must exist as an individual file.
- Record model/tool, generation date, prompt/source reference, validation status, and revision/version in the manifest when available.
- Never claim an image was generated, saved, or validated unless the actual file exists in this repository workspace.

## Mandatory workflow

1. Inspect the repository's current asset catalogs, this family folder, existing texture sheets, and relevant art-direction/provenance policies before generating.
2. Make an explicit checklist of required assets from the source prompt. Preserve its exact requested count and IDs; do not silently omit items or substitute a contact sheet for individual files.
3. Generate **original NEXORA designs**. Use external works only for abstract qualities such as readability, material variety, or detail density. Never trace, extract, reconstruct, or closely imitate specific third-party textures, characters, icons, logos, palettes, pixel layouts, or distinctive designs.
4. Unless the source prompt explicitly specifies a different format, use the project's first-generation target of **16×16 pixels per texture**. A preview sheet may be larger, but each final texture must also be exported as its own correctly sized file.
5. Keep textures pixel-crisp: no unintended blur, antialiasing, perspective, scene/background, text, watermark, logo, or accidental transparent pixels. Use transparency only when the asset type requires it.
6. For tileable materials, validate edges and a repeated 3×3 preview. For non-tileable assets, validate silhouette, transparent padding, readability at native size, and intended in-game use.
7. Save every generated image **inside this prompt's own `images/` directory**. Never save an image from this prompt into another prompt's folder. Use stable lowercase filenames and the source asset IDs where available.
8. Keep untouched originals. Revisions get versioned filenames or a clearly tracked subfolder; never overwrite an approved asset without recording why.
9. Record each output in `images/manifest.json` (create it when the first image is added). Include asset ID, filename, source issue, source prompt path, generation tool/model if known, date, dimensions, detected format, version, status (draft/approved/rejected), originality review, validation results, and SHA-256 when available.
10. Inspect the actual saved files after generation. Confirm that file count matches the checklist, dimensions and formats are correct, images open successfully, and no duplicates were introduced.
11. If the environment cannot generate or write binary files, do not pretend the images exist. Report exactly what is missing and provide a reproducible list of intended output paths.

## Acceptance gate

- [ ] Source prompt read and all requested assets enumerated.
- [ ] Existing assets checked; duplicates avoided.
- [ ] Each required asset generated as an individual file, not only embedded in a contact sheet.
- [ ] Originality and project art direction reviewed.
- [ ] Files saved only under this prompt's `images/` folder.
- [ ] Resolution/format/transparency/tiling validated as applicable.
- [ ] Manifest and provenance completed.
- [ ] Actual saved files inspected; no unverified success claims.

## Output location

`assets/generated/prompts/issue-05-stones/images/`

Do not place source prompt documents, reports, or unrelated assets in the `images/` directory.
