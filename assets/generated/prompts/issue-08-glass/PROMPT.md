# NEXORA — Production prompt #8: Glass

> **Source of truth:** [ISSUE-08-glass-16x16.md](../../../../docs/assets/prompts/issues/ISSUE-08-glass-16x16.md) · [Original GitHub Issue #8](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/8)

## Mission

Execute the complete source prompt linked above. Read it from beginning to end before generating anything. This file strengthens its execution requirements; it does not remove or silently override the source prompt's requested assets, counts, names, IDs, or constraints.

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

`assets/generated/prompts/issue-08-glass/images/`

Do not place source prompt documents, reports, or unrelated assets in the `images/` directory.
