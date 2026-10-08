# NEXORA — Prompt Library Migrated from GitHub Issues

This directory preserves the historical image-generation prompts mirrored from GitHub Issues. For day-to-day asset generation, use the **production prompt folders** linked below. Each has its own `PROMPT.md` and a dedicated `images/` directory.

## Mandatory rules for AI agents

1. Read the production `PROMPT.md` and the linked historical source prompt completely before generating.
2. Check existing repository assets and manifests first. Avoid duplicates and preserve already approved work.
3. Derive a checklist from the source. Preserve exact asset names, IDs, counts, variants, dimensions, and constraints.
4. Generate original NEXORA artwork. External references may inform abstract qualities only; do not trace or closely imitate recognizable third-party textures, characters, icons, logos, symbols, palettes, or pixel layouts.
5. Put each generated image **only in the `images/` folder of its originating prompt**. Never mix image outputs between prompt folders.
6. For textures, use 16×16 pixels when the source requires it. If another size or file type is explicitly requested, follow the source.
7. Keep individual output files; contact sheets are optional previews, never a replacement for individual assets.
8. Keep images crisp and readable. Validate format, dimensions, transparency, tiling where applicable, and native-size appearance.
9. Preserve originals; version revisions instead of silently overwriting approved assets.
10. When the first actual image is added to a prompt folder, create/update `images/manifest.json` with asset ID, filename, source issue and prompt, generation tool/model (if known), date, dimensions, format, version, status, originality review, validation results, and SHA-256 when available.
11. Inspect actual files after generation. Do not claim that an image exists, was validated, or was integrated unless that has been verified.
12. If the environment cannot create/write image binaries, clearly report the limitation and missing outputs. Never use fake placeholder images.

## Production prompts — one isolated folder per issue

| Issue | Topic | Production prompt and output folder |
|---:|---|---|
| [#4](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/4) | Texture pipeline / phase 1 | [issue-04-texture-pipeline-phase-1](../../../assets/generated/prompts/issue-04-texture-pipeline-phase-1/) |
| [#5](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/5) | Stones | [issue-05-stones](../../../assets/generated/prompts/issue-05-stones/) |
| [#6](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/6) | Ores | [issue-06-ores](../../../assets/generated/prompts/issue-06-ores/) |
| [#7](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/7) | Sand | [issue-07-sand](../../../assets/generated/prompts/issue-07-sand/) |
| [#8](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/8) | Glass | [issue-08-glass](../../../assets/generated/prompts/issue-08-glass/) |
| [#9](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/9) | Wool | [issue-09-wool](../../../assets/generated/prompts/issue-09-wool/) |
| [#10](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/10) | Banners | [issue-10-banners](../../../assets/generated/prompts/issue-10-banners/) |
| [#11](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/11) | Mobs | [issue-11-mobs](../../../assets/generated/prompts/issue-11-mobs/) |
| [#12](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/12) | Liquids | [issue-12-liquids](../../../assets/generated/prompts/issue-12-liquids/) |
| [#13](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/13) | Items | [issue-13-items](../../../assets/generated/prompts/issue-13-items/) |
| [#14](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/14) | Master checklist | [issue-14-texture-master-checklist](../../../assets/generated/prompts/issue-14-texture-master-checklist/) |
| [#15](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/15) | Construction blocks | [issue-15-construction-blocks](../../../assets/generated/prompts/issue-15-construction-blocks/) |
| [#16](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/16) | Vegetation and nature | [issue-16-vegetation-nature](../../../assets/generated/prompts/issue-16-vegetation-nature/) |
| [#17](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/17) | Terrain and biomes | [issue-17-terrain-biomes](../../../assets/generated/prompts/issue-17-terrain-biomes/) |
| [#18](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/18) | Technology and industry | [issue-18-technology-industry](../../../assets/generated/prompts/issue-18-technology-industry/) |
| [#19](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/19) | Equipment | [issue-19-equipment](../../../assets/generated/prompts/issue-19-equipment/) |
| [#20](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/20) | Vehicles and transport | [issue-20-vehicles-transport](../../../assets/generated/prompts/issue-20-vehicles-transport/) |
| [#21](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/21) | Architecture and structures | [issue-21-architecture-structures](../../../assets/generated/prompts/issue-21-architecture-structures/) |
| [#22](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/22) | History / Crônicas da Baluarte | [issue-22-history-baluarte](../../../assets/generated/prompts/issue-22-history-baluarte/) |
| [#23](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/23) | Magic and dimensions | [issue-23-magic-dimensions](../../../assets/generated/prompts/issue-23-magic-dimensions/) |
| [#24](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/24) | VFX | [issue-24-vfx](../../../assets/generated/prompts/issue-24-vfx/) |
| [#25](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/25) | UI, icons and signage | [issue-25-ui-icons-signage](../../../assets/generated/prompts/issue-25-ui-icons-signage/) |
| [#26](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/26) | Decals and surfaces | [issue-26-decals-surfaces](../../../assets/generated/prompts/issue-26-decals-surfaces/) |
| [#27](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/27) | Texture pipeline validation | [issue-27-texture-pipeline-validation](../../../assets/generated/prompts/issue-27-texture-pipeline-validation/) |
| [#28](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/28) | Special Baluarte armor | [issue-28-baluarte-special-armor](../../../assets/generated/prompts/issue-28-baluarte-special-armor/) |

## Standard folder layout

```text
assets/generated/prompts/issue-NN-topic/
├── PROMPT.md
└── images/
    ├── README.md
    ├── manifest.json       # created with the first real image
    └── asset-name.png      # actual image file, never a placeholder
```

Git does not track empty directories, so each `images/` directory contains a README until actual images are added. The README files are not images and do not imply that image generation has occurred.

## Historical prompt mirrors

The original issue text is retained in [`issues/`](issues/) for traceability. Policy/source documents from issues #44–#46 are in [`policies/`](policies/). Image attachments discovered in issue comments and existing repository images are catalogued in the [issue-image inventory](../images-from-issues/README.md); external attachment URLs are not local image files until their binary data is actually committed.
