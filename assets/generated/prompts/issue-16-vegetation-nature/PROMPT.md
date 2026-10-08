# NEXORA — Production prompt #16: Vegetation and nature

> **Source of truth:** [ISSUE-16-vegetation-nature-16x16.md](../../../../docs/assets/prompts/issues/ISSUE-16-vegetation-nature-16x16.md) · [Original GitHub Issue #16](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/16)

## Mission
Read the linked source prompt completely. Execute its requested work and preserve its counts, IDs, constraints, and acceptance criteria. This file adds production controls; it does not replace the source.

## Mandatory workflow
1. Inspect existing assets, catalogs, this folder, and current art-direction/provenance policies before generating.
2. Make a complete checklist from the source prompt. Preserve all requested asset IDs, names, counts, variants, and constraints. A contact sheet does not replace individual image files.
3. Create original NEXORA designs. External works may inform only abstract qualities such as readability, detail density, and material variety; never trace, extract, reconstruct, or closely imitate a specific texture, character, icon, symbol, logo, palette, pixel layout, or recognizable design.
4. Use the project's first-generation target of **16×16 pixels per texture** unless the source prompt explicitly specifies another size or a non-texture format. Export each required asset individually.
5. Keep pixel art crisp and readable at native size. Avoid unintended blur, antialiasing, perspective, scene/background, text, watermark, logos, or accidental transparency. Use transparency only when the asset type needs it.
6. Validate tiling and edge continuity for tileable materials; validate silhouette, padding, transparency, and native-size readability for other assets.
7. Save all images produced by this prompt only in `assets/generated/prompts/issue-16-vegetation-nature/images/`.
8. Preserve originals and never overwrite approved assets silently. Track revisions as new versions and document the reason.
9. When the first image is created, add/update `images/manifest.json` with asset ID, filename, source issue, source prompt path, tool/model (if known), date, dimensions, actual format, version, status, originality review, validation results, and SHA-256 (if available).
10. Inspect actual saved files, verify dimensions/formats and that they open, confirm the checklist is complete, and avoid duplicates.
11. If image generation or binary file writing is unavailable, state the limitation and list missing outputs. Never claim files exist if they do not.

## Acceptance gate
- [ ] Source prompt read and complete checklist created.
- [ ] Existing assets checked and duplicates avoided.
- [ ] Only assets explicitly requested by the source were generated.
- [ ] Originality and current art direction reviewed.
- [ ] Images (if any) saved only in this prompt's `images/` folder.
- [ ] Technical dimensions/format/transparency/tiling validated as applicable.
- [ ] Provenance recorded for every generated image.
- [ ] Actual saved files inspected; no unsupported success claims.

**Output directory for images:** `assets/generated/prompts/issue-16-vegetation-nature/images/`
