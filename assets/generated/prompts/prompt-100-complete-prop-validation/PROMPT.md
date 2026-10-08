# NEXORA — Production prompt 100: Complete Prop Asset Validation

## Mission
Audit and validate the existing NEXORA asset collection; do not invent missing images or mark absent files as complete.

## Required asset families
1. auditoria de nomes e IDs
2. checagem de dimensões e formatos
3. detecção de arquivos duplicados
4. verificação de transparência
5. teste de repetição 3×3 para texturas
6. revisão de silhueta e legibilidade
7. verificação de manifesto e hashes
8. relatório de assets ausentes e inválidos

## Pre-generation / audit requirements
Read this prompt and relevant project art-direction/pipeline documentation. Inspect repository assets and manifests. Enumerate existing and missing assets, variants, IDs, counts, and dimensions; avoid duplicates. Explicit source issue requirements take precedence. For validation-only tasks, do not generate replacement assets until failures are documented and the project owner’s replacement policy is followed.

## Image-generation and validation protocol
- Create original NEXORA artwork; never copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Identify each output as texture, tileable material, icon, sprite, or isolated prop.
- Follow exact pipeline dimensions; required 16×16 textures must be natively 16×16 pixels.
- Maintain coherent voxel/pixel-art styling, readable silhouettes, controlled pixel clusters, clear contrast, and consistent scale/material language.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, unrelated scenery, watermarks, logos, and unrequested text. Use transparency only when appropriate.
- Tileable textures must pass a repeated 3×3 seam check. Isolate props/icons and keep framing consistent.
- Inspect actual files at native size and enlarged nearest-neighbor scale. Validate dimensions, format, alpha, readability, silhouette, duplicate risk, filenames, and source requirements. Regenerate or flag failures; never silently accept them.
- Contact sheets are previews only and never replace individual production files.
- Record only verifiable metadata; use `unknown` when unavailable. Never claim an image was generated, saved, inspected, or validated unless the actual file exists and the check occurred.

## Output contract
Save the audit report and any generated previews only inside `assets/generated/prompts/prompt-100-complete-prop-validation/images/`; do not modify approved assets during audit. Use stable lowercase filenames. Maintain `manifest.json` in that directory with asset ID, filename, source prompt, generation tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets instead of overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts or audited/pass/fail counts, exact paths, validations performed, missing requirements, and blockers. Distinguish verified facts from recommendations.