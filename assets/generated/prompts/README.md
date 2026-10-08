# NEXORA — Image Generation Workspace

This is the operational workspace for image-generation prompts. **Every source prompt has its own isolated folder.** The AI must read that folder's `PROMPT.md` and save its image outputs only in that same folder's `images/` directory.

## Start here

1. Read [the prompt-library rules and full catalog](../../../docs/assets/prompts/README.md).
2. Open the production prompt for the relevant issue below.
3. Read its linked historical issue prompt completely.
4. Check existing assets and manifests before generating.
5. Save only the outputs for that prompt under its own `images/` folder.

## Folder index

| Source issue | Prompt folder |
|---:|---|
| #4 | [issue-04-texture-pipeline-phase-1](issue-04-texture-pipeline-phase-1/) |
| #5 | [issue-05-stones](issue-05-stones/) |
| #6 | [issue-06-ores](issue-06-ores/) |
| #7 | [issue-07-sand](issue-07-sand/) |
| #8 | [issue-08-glass](issue-08-glass/) |
| #9 | [issue-09-wool](issue-09-wool/) |
| #10 | [issue-10-banners](issue-10-banners/) |
| #11 | [issue-11-mobs](issue-11-mobs/) |
| #12 | [issue-12-liquids](issue-12-liquids/) |
| #13 | [issue-13-items](issue-13-items/) |
| #14 | [issue-14-texture-master-checklist](issue-14-texture-master-checklist/) |
| #15 | [issue-15-construction-blocks](issue-15-construction-blocks/) |
| #16 | [issue-16-vegetation-nature](issue-16-vegetation-nature/) |
| #17 | [issue-17-terrain-biomes](issue-17-terrain-biomes/) |
| #18 | [issue-18-technology-industry](issue-18-technology-industry/) |
| #19 | [issue-19-equipment](issue-19-equipment/) |
| #20 | [issue-20-vehicles-transport](issue-20-vehicles-transport/) |
| #21 | [issue-21-architecture-structures](issue-21-architecture-structures/) |
| #22 | [issue-22-history-baluarte](issue-22-history-baluarte/) |
| #23 | [issue-23-magic-dimensions](issue-23-magic-dimensions/) |
| #24 | [issue-24-vfx](issue-24-vfx/) |
| #25 | [issue-25-ui-icons-signage](issue-25-ui-icons-signage/) |
| #26 | [issue-26-decals-surfaces](issue-26-decals-surfaces/) |
| #27 | [issue-27-texture-pipeline-validation](issue-27-texture-pipeline-validation/) |
| #28 | [issue-28-baluarte-special-armor](issue-28-baluarte-special-armor/) |

| Prompt number | Prompt folder |
|---:|---|
| 29 | [prompt-29-workstations](prompt-29-workstations/) |
| 30 | [prompt-30-storage](prompt-30-storage/) |
| 31 | [prompt-31-furniture](prompt-31-furniture/) |
| 32 | [prompt-32-industrial-machines](prompt-32-industrial-machines/) |
| 33 | [prompt-33-energy-technology](prompt-33-energy-technology/) |
| 34 | [prompt-34-exploration](prompt-34-exploration/) |
| 35 | [prompt-35-urban-infrastructure](prompt-35-urban-infrastructure/) |
| 36 | [prompt-36-construction](prompt-36-construction/) |
| 37 | [prompt-37-household](prompt-37-household/) |
| 38 | [prompt-38-laboratory](prompt-38-laboratory/) |
| 39 | [prompt-39-baluarte-military](prompt-39-baluarte-military/) |
| 40 | [prompt-40-commerce](prompt-40-commerce/) |
| 41 | [prompt-41-tools-workshop](prompt-41-tools-workshop/) |
| 42 | [prompt-42-alchemy-fantasy-science](prompt-42-alchemy-fantasy-science/) |
| 43 | [prompt-43-library-knowledge](prompt-43-library-knowledge/) |
| 44 | [prompt-44-food-kitchen](prompt-44-food-kitchen/) |
| 45 | [prompt-45-farming](prompt-45-farming/) |
| 46 | [prompt-46-fishing](prompt-46-fishing/) |
| 47 | [prompt-47-mining-equipment](prompt-47-mining-equipment/) |
| 48 | [prompt-48-medical](prompt-48-medical/) |
| 49 | [prompt-49-communication](prompt-49-communication/) |
| 50 | [prompt-50-lighting](prompt-50-lighting/) |
| 51 | [prompt-51-signs-wayfinding](prompt-51-signs-wayfinding/) |
| 52 | [prompt-52-security](prompt-52-security/) |
| 53 | [prompt-53-transport-stations](prompt-53-transport-stations/) |

## Non-negotiable output rules

- Never put generated images in a shared catch-all directory.
- Never put output from one prompt in another prompt's folder.
- Do not move existing repository images without checking references and updating them safely.
- Do not overwrite approved assets without versioning and a reason.
- Keep each image as a separate file. Contact sheets are optional previews only.
- Record provenance and validation in `images/manifest.json` when actual images are present.
- An `images/README.md` is only a directory marker, not evidence that an image was generated.
- If binary output cannot be written, report the limitation honestly; do not create fake images or claim completion.

## Expected layout

```text
assets/generated/prompts/
├── README.md
└── issue-NN-topic/
    ├── PROMPT.md
    └── images/
        ├── README.md
        ├── manifest.json
        └── <actual-generated-file>.png
```
