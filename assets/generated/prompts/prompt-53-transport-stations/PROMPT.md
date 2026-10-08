# NEXORA — Production prompt 53: Transport Stations & Terminals

## Mission
Create original NEXORA assets for stations, terminals, and passenger or cargo boarding points. All outputs must remain isolated to this prompt's folder.

## Scope
Station benches, ticket booths, ticket machines, abstract timetable panels without legible text, modular platforms and shelters, boarding barriers, luggage carts, and stop markers.

## Before generation
Read this prompt and relevant project art-direction/pipeline documentation. Inspect existing repository assets and manifests. List missing assets, variants, stable IDs, counts, and required dimensions. Source issue requirements take precedence and existing assets must not be duplicated.

## Image-generation protocol
- Create original NEXORA designs; never copy, trace, extract, or closely reconstruct third-party game assets, mods, brands, or artwork.
- Determine the correct asset type: texture, tileable material, icon, sprite, or isolated prop.
- Follow exact pipeline dimensions. A required 16×16 texture must be natively 16×16 pixels.
- Maintain consistent voxel/pixel-art language, readable silhouettes, controlled pixel clusters, clear material separation, and coherent scale.
- Avoid accidental blur, unwanted anti-aliasing, photorealism, unrelated scenery, watermarks, logos, and unrequested text. Use transparency only when appropriate.
- Tileable materials must pass a repeated 3×3 seam check. Isolate props and icons with consistent framing.
- Inspect at native size and enlarged nearest-neighbor scale. Validate dimensions, format, transparency, readability, silhouette, duplicate risk, naming, and source requirements. Regenerate failures.
- Contact sheets are previews only and never replace individual production files.
- Record only verifiable metadata. Use `unknown` if tool/model/date is not available. Never claim an image was generated, saved, inspected, or validated unless the actual file exists and the check occurred.

## Output contract
Save final images only in `assets/generated/prompts/prompt-53-transport-stations/images/`. Use stable lowercase filenames. Maintain `manifest.json` in the same directory with asset ID, filename, source prompt, tool/model/date when known, dimensions, format, version, status, originality review, and validation results. Version approved assets rather than overwriting them.

## Completion report
Report requested/generated/accepted/rejected counts, exact paths, validation performed, missing requirements, and blockers. Distinguish verified facts from recommendations.