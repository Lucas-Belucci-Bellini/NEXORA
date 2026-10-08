# NEXORA — Production prompt #9: Wool

> **Source of truth:** [ISSUE-09-wool-16x16.md](../../../../docs/assets/prompts/issues/ISSUE-09-wool-16x16.md) · [Original GitHub Issue #9](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/9)

## Mission
Read the linked source prompt completely and execute its requested assets, counts, IDs, variants, and constraints. This file adds production controls; it does not replace the source requirements.

## Mandatory workflow
1. Inspect existing assets, catalogs, this folder, and the current art-direction/provenance policies before creating anything.
2. Build a checklist from the source prompt and preserve every requested asset ID, name, quantity, and variant. Do not replace individual files with only a contact sheet.
3. Create original NEXORA designs. Third-party works may inform only abstract qualities (e.g. readability or variety); never trace, extract, reconstruct, or closely imitate specific assets, characters, symbols, logos, palettes, pixel layouts, or recognizable designs.
4. Use the project's first-generation target of **16×16 pixels per texture** wherever the source prompt does not explicitly require another format. Export every asset individually even when also making a preview sheet.
5. Keep pixel art crisp and readable at native size. Avoid blur, unintended antialiasing, perspective, scene backgrounds, text, watermark, or logos. Use transparency only where appropriate to the asset type.
6. Validate tiling and edge continuity for tileable textures. For mobs/items/UI-like assets, validate silhouette, padding, transparency, and readability at native size as appropriate.
7. Save all images produced by this prompt only in `assets/generated/prompts/issue-09-wool/images/`. Use stable lowercase filenames and source IDs when available.
8. Never overwrite approved/original assets silently. Keep originals; version revisions and document why.
9. When the first image is generated, create/update `images/manifest.json` with asset ID, filename, source issue, source prompt path, tool/model (if known), date, dimensions, actual format, version, status (draft/approved/rejected), originality review, validation results, and SHA-256 (if available).
10. Inspect the actual saved files; verify that they open, dimensions and formats are correct, requested file counts are satisfied, and duplicates are avoided.
11. If image generation or binary file writing is unavailable, report that limitation and list missing outputs. Never claim a nonexistent file was generated.

## Acceptance gate
- [ ] Source prompt read and complete asset checklist made.
- [ ] Existing files checked and duplicates avoided.
- [ ] Each required asset saved as its own image.
- [ ] Originality and art direction reviewed.
- [ ] Images saved only in this prompt's `images/` folder.
- [ ] Dimensions, format, transparency, tiling, and native-size readability validated as applicable.
- [ ] Provenance manifest updated.
- [ ] Real files inspected; no unsupported success claims.

**Output directory:** `assets/generated/prompts/issue-09-wool/images/`
