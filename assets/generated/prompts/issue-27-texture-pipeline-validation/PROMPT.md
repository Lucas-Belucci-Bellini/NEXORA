# NEXORA — Production prompt #27: Texture pipeline validation

> **Source of truth:** [ISSUE-27-texture-pipeline-validation.md](../../../../docs/assets/prompts/issues/ISSUE-27-texture-pipeline-validation.md) · [Original GitHub Issue #27](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/27)

## Mission
Read and execute the linked source prompt completely. Preserve its required asset list, names, IDs, counts, variants, formats, and constraints. This wrapper adds production and verification requirements without silently changing the source.

## Special handling
This is a technical validation/pipeline prompt. Do not invent a new batch of images unless the source explicitly requests them. Keep reports, scripts, and manifests outside `images/`; put only real visual outputs or preview images explicitly required by the source in this folder.

## Acceptance gate
- [ ] Source prompt read and exact checklist recorded.
- [ ] Existing assets and applicable policies checked.
- [ ] Only source-requested outputs created.
- [ ] Originality and visual consistency reviewed.
- [ ] Images, if required, saved only in this prompt's `images/` folder.
- [ ] Dimensions, format, transparency, tiling, and readability validated as applicable.
- [ ] Provenance recorded.
- [ ] Actual files inspected; no unsupported claims.

**Output directory:** `assets/generated/prompts/issue-27-texture-pipeline-validation/images/`
