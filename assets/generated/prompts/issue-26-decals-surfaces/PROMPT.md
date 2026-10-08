# NEXORA — Production prompt #26: Decals and surfaces

> **Source of truth:** [ISSUE-26-decals-surfaces-16x16.md](../../../../docs/assets/prompts/issues/ISSUE-26-decals-surfaces-16x16.md) · [Original GitHub Issue #26](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/26)

## Mission
Read and execute the linked source prompt completely. Preserve its required asset list, names, IDs, counts, variants, formats, and constraints. This wrapper adds production and verification requirements without silently changing the source.

## Mandatory workflow
1. Inspect existing assets, catalogs, this family folder, and current art-direction/provenance policies before generating.
2. Convert the source prompt into a complete checklist. Preserve every requested ID, count, name, and variant. A contact sheet is supplementary and never replaces individual deliverables.
3. Create original NEXORA assets. External works may inform only abstract qualities; never trace, extract, reconstruct, or closely imitate third-party textures, character designs, armor, icons, symbols, logos, palettes, pixel layouts, or recognizable visual signatures.
4. Use **16×16 pixels per texture** unless the source explicitly requires another resolution or a different kind of asset. Do not force non-texture images into 16×16.
5. Keep pixel art crisp and readable. Avoid unintended blur, antialiasing, perspective, scene/background, text, watermark, logos, or accidental transparency. Use transparency only when the asset type requires it.
6. Validate tiling and edge continuity for tileable textures; validate silhouette, padding, transparency, intended display size, and readability for other assets.
7. Save all images produced by this prompt only in `assets/generated/prompts/issue-26-decals-surfaces/images/`.
8. Preserve originals and version revisions; never overwrite approved assets silently.
9. When the first real image is saved, create/update `images/manifest.json` with asset ID, filename, source issue, source prompt path, tool/model if known, date, dimensions, actual format, version, status, originality review, validation results, and SHA-256 if available.
10. Inspect actual files, verify they open and meet technical requirements, compare actual counts against the checklist, and avoid duplicates.
11. If generation or binary writing is unavailable, state that plainly and list missing outputs. Never claim nonexistent files were created.

## Acceptance gate
- [ ] Source prompt read and exact checklist recorded.
- [ ] Existing assets and applicable policies checked.
- [ ] Only source-requested outputs created.
- [ ] Originality and visual consistency reviewed.
- [ ] Images, if required, saved only in this prompt's `images/` folder.
- [ ] Dimensions, format, transparency, tiling, and readability validated as applicable.
- [ ] Provenance recorded.
- [ ] Actual files inspected; no unsupported claims.

**Output directory:** `assets/generated/prompts/issue-26-decals-surfaces/images/`
