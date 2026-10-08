# NEXORA — Production prompt 73: Space & Orbital Props

## Mission
Create original NEXORA assets for Space & Orbital Props, isolated to this prompt folder.

## Scope and asset families
Equipamentos de exploração espacial e instalações orbitais de ficção científica originais.
1. módulos de carga espacial
2. painéis de manutenção orbital
3. caixas de ferramentas pressurizadas
4. marcadores de doca espacial
5. antenas de comunicação
6. luzes de navegação
7. recipientes de amostras extraterrestres fictícias
8. suportes de traje espacial

## Before generation
Read this prompt and relevant art-direction/pipeline documentation. Inspect existing assets and manifests, enumerate missing assets, variants, IDs, counts, and dimensions, and avoid duplicates. Explicit source issue requirements take precedence.

## Image-generation protocol
- Create original NEXORA artwork; never copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Determine the correct output type: texture, tileable material, icon, sprite, or isolated prop.
- Follow exact pipeline dimensions; required 16×16 textures must be natively 16×16 pixels.
- Maintain coherent voxel/pixel-art style, readable silhouettes, controlled pixel clusters, clear contrast, and consistent scale.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, unrelated scenery, watermarks, logos, and unrequested text. Use transparency only when appropriate.
- Tileable textures must pass a repeated 3×3 seam check. Isolate props and use consistent framing.
- Inspect at native size and enlarged nearest-neighbor scale. Validate dimensions, format, alpha, readability, silhouette, duplicates, filenames, and requirements; regenerate failures.
- Contact sheets are previews only. Record only verifiable metadata and never claim generation or validation without inspecting the actual file.

## Output contract
Save final images only in `assets/generated/prompts/prompt-73-space-props/images/`. Use stable lowercase filenames. Maintain `manifest.json` with asset ID, filename, source prompt, tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets instead of overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, validations, gaps, and blockers. Distinguish verified facts from recommendations.