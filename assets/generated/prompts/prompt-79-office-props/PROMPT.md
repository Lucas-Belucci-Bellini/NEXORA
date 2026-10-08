# NEXORA — Production prompt 79: Office & Administration Props

## Mission
Create original, production-ready NEXORA assets for Office & Administration Props. Keep outputs isolated to this prompt folder.

## Required asset families
1. mesas de escritório
2. cadeiras de trabalho
3. arquivos e gaveteiros
4. computadores e monitores
5. telefones de mesa
6. organizadores de documentos
7. impressoras estilizadas
8. divisórias modulares

## Before generation
Read this prompt and the relevant art-direction/pipeline documentation. Inspect repository assets and manifests, enumerate missing assets, variants, IDs, counts, and dimensions, and avoid duplicates. Explicit source issue requirements take precedence.

## Image-generation protocol
- Create original NEXORA artwork; never copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Identify the output type: texture, tileable material, icon, sprite, or isolated prop.
- Follow exact pipeline dimensions; required 16×16 textures must be natively 16×16 pixels.
- Maintain voxel/pixel-art consistency, readable silhouettes, controlled pixel clusters, clear contrast, and consistent scale.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, unrelated scenery, watermarks, logos, and unrequested text. Use transparency only when appropriate.
- Tileable textures must pass a repeated 3×3 seam check. Isolate props and keep framing consistent.
- Inspect at native size and enlarged nearest-neighbor scale. Validate dimensions, format, alpha, readability, silhouette, duplicate risk, filenames, and source requirements; regenerate failures.
- Contact sheets are previews only. Record only verifiable metadata; never claim an image exists or was validated without inspecting the actual file.

## Output contract
Save final images only in `assets/generated/prompts/prompt-79-office-props/images/`. Use stable lowercase filenames. Maintain `manifest.json` with asset ID, filename, source prompt, tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets rather than overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, validation performed, missing requirements, and blockers. Distinguish verified facts from recommendations.