# NEXORA — Prompt Library Migrated from GitHub Issues

This folder is the repository-backed source of truth for image-generation prompts previously kept in GitHub Issues. Each imported file preserves the issue body, including prompts, counts, constraints, and checklists, and links back to its source issue.

## Rules for agents

1. Read this index and the relevant family prompt before generating assets.
2. Check the existing asset catalog and image folders first; do not generate duplicates.
3. Keep the first-generation texture target at 16×16 where the source issue requires it.
4. Do not copy or closely imitate third-party textures, models, icons, logos, or recognizable visual designs.
5. Save generated files under the appropriate asset-family folder and record prompt, tool/model, date, version, path, and status in provenance metadata.
6. Do not claim an image has been generated or integrated unless the file exists and the integration was checked.
7. The issue text is preserved for traceability. If it conflicts with newer normative architecture/art-direction documentation, follow the current project policy and document the conflict rather than silently changing the historical prompt.

## Imported issue catalog

| Issue | Family / purpose | Local prompt file |
|---:|---|---|
| [#4](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/4) | Texture pipeline, phase 1 | [ISSUE-04](issues/ISSUE-04-texture-pipeline-phase-1.md) |
| [#5](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/5) | Stones | [ISSUE-05](issues/ISSUE-05-stones-16x16.md) |
| [#6](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/6) | Ores | [ISSUE-06](issues/ISSUE-06-ores-16x16.md) |
| [#7](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/7) | Sand | [ISSUE-07](issues/ISSUE-07-sand-16x16.md) |
| [#8](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/8) | Glass | [ISSUE-08](issues/ISSUE-08-glass-16x16.md) |
| [#9](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/9) | Wool | [ISSUE-09](issues/ISSUE-09-wool-16x16.md) |
| [#10](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/10) | Banners | [ISSUE-10](issues/ISSUE-10-banners-16x16.md) |
| [#11](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/11) | Mobs | [ISSUE-11](issues/ISSUE-11-mobs-16x16.md) |
| [#12](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/12) | Liquids | [ISSUE-12](issues/ISSUE-12-liquids-16x16.md) |
| [#13](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/13) | Items | [ISSUE-13](issues/ISSUE-13-items-16x16.md) |
| [#14](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/14) | Master checklist | [ISSUE-14](issues/ISSUE-14-texture-master-checklist.md) |
| [#15](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/15) | Construction blocks | [ISSUE-15](issues/ISSUE-15-blocks-construction-16x16.md) |
| [#16](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/16) | Vegetation and nature | [ISSUE-16](issues/ISSUE-16-vegetation-nature-16x16.md) |
| [#17](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/17) | Terrain and biomes | [ISSUE-17](issues/ISSUE-17-terrain-biomes-16x16.md) |
| [#18](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/18) | Technology and industry | [ISSUE-18](issues/ISSUE-18-technology-industry-16x16.md) |
| [#19](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/19) | Additional equipment | [ISSUE-19](issues/ISSUE-19-equipment-16x16.md) |
| [#20](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/20) | Vehicles and transport | [ISSUE-20](issues/ISSUE-20-vehicles-transport-16x16.md) |
| [#21](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/21) | Architecture and structures | [ISSUE-21](issues/ISSUE-21-architecture-structures-16x16.md) |
| [#22](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/22) | History and Crônicas da Baluarte | [ISSUE-22](issues/ISSUE-22-history-baluarte-16x16.md) |
| [#23](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/23) | Magic, dimensions, special materials | [ISSUE-23](issues/ISSUE-23-magic-dimensions-16x16.md) |
| [#24](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/24) | VFX | [ISSUE-24](issues/ISSUE-24-vfx-16x16.md) |
| [#25](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/25) | UI, icons, signage | [ISSUE-25](issues/ISSUE-25-ui-icons-signage-16x16.md) |
| [#26](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/26) | Decals and surfaces | [ISSUE-26](issues/ISSUE-26-decals-surfaces-16x16.md) |
| [#27](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/27) | Pipeline, validation, provenance | [ISSUE-27](issues/ISSUE-27-texture-pipeline-validation.md) |
| [#28](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/28) | Special character armor | [ISSUE-28](issues/ISSUE-28-baluarte-special-armor-16x16.md) |

## Image files

Issue prompt text and binary image assets are tracked separately. See [the issue-image inventory](../images-from-issues/README.md) for image attachments found in the comments and existing repository images. Do not treat an external issue attachment as a committed local image until the binary exists under the repository path.
