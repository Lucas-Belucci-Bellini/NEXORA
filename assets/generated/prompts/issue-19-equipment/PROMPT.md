# NEXORA — Production prompt #19: Additional equipment

> **Source of truth:** [ISSUE-19-equipment-16x16.md](../../../../docs/assets/prompts/issues/ISSUE-19-equipment-16x16.md) · [Original GitHub Issue #19](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/19)

## Mission
Read the source prompt in full and execute its requested assets, counts, IDs, names, variants, and constraints. This production wrapper adds workflow and validation gates without silently replacing the source.

## Mandatory workflow
1. Inspect existing repository assets, catalogs, this family folder, and current art-direction/provenance policies before generation.
2. Create a checklist from the source prompt and preserve every requested ID, name, quantity, variant, and constraint. Do not substitute a contact sheet for individual deliverables.
3. Design original NEXORA assets. Use external references only to abstract qualities; never trace, extract, reconstruct, or closely imitate specific third-party textures, characters, costumes, models, icons, logos, symbols, palettes, or recognizable designs.
4. Use **16×16 pixels per texture** for the first generation unless the source explicitly requires a different format/resolution. For assets that are not textures, follow the source's native format requirements and do not force them into 16×16.
5. Keep pixel art crisp, legible, and internally consistent. Avoid unintended blur, antialiasing, perspective, scene/background, text, watermark, or unauthorized logos. Use transparency only when appropriate.
6. Validate tiling for tileable surfaces; for non-tileable assets, validate silhouette, padding, transparency, scale, and readability at intended display size.
7. Save every image produced by this prompt only under `assets/generated/prompts/issue-19-equipment/images/`. Use stable lowercase filenames and asset IDs when available.
8. Preserve originals and never overwrite approved files without traceable versioning and a reason.
9. When the first real image is added, create/update `images/manifest.json` with asset ID, filename, source issue, source prompt path, tool/model if known, generation date, dimensions, actual format, version, status, originality review, validation results, and SHA-256 if available.
10. Inspect actual files and verify they open, formats/dimensions are correct, expected counts are met, and duplicates were avoided.
11. If generation or binary writes are unavailable, clearly list missing outputs; do not claim uncreated files exist.

## Acceptance gate
- [ ] Source prompt read; full checklist captured.
- [ ] Existing assets checked and duplicates avoided.
- [ ] All required assets generated individually.
- [ ] Originality and NEXORA art direction reviewed.
- [ ] Images saved only to this prompt's own `images/` directory.
- [ ] Relevant format, resolution, transparency, tiling, and readability checks passed.
- [ ] Provenance manifest updated.
- [ ] Actual output files inspected and reported honestly.

**Output directory:** `assets/generated/prompts/issue-19-equipment/images/`
