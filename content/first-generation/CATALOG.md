# First visual generation — catalog

The **catalog mestre** of issue #27, for what exists so far: the sixteen stones
of issue #5. Every row is a first-generation asset under the rule the operator
set for 2026 — **16×16, albedo only** — and that rule is not a promise here: it
is the `policy` of [`manifest.json`](manifest.json), and the forge refuses to
generate anything from that manifest that breaks it.

```sh
cargo run -p nexora-texture-forge -- batch content/first-generation/manifest.json
```

## How these were made

**Procedurally, not from the prompts.** Issue #4 asks for *"prompt individual
ou regra de geração reproduzível"* per asset; these are the second kind. Each
stone is:

```text
content/first-generation/stone/<variant>.json   the material: id, 16×16, no optional maps,
                                                and a provenance note naming its #5 prompt
content/recipes/nexora/stone/<variant>.json     the recipe: palette, lattice, cracks,
                                                layers, grains (ADR-0013)
```

The #5 prompt was read as art direction — *"pale mineral base with interlocking
grains"*, *"cool dark layers with thin planar divisions"* — and turned into
numbers. No image generator was run, no external image was read, and no pixel
of any other work is in the output: the class the forge records is
`procedural_derivative` / `nexora_original`, with the generator, its version,
the recipe, the recipe's fingerprint and the seed all in each material's trace.

`prompt_reference` below is therefore *what the recipe answers*, not *what was
sent to a model*. If a stone is later drawn from its prompt by an image model,
that is a **different asset** with class `editor_generated`, and it needs the
import path the audit (§4.6) has not built yet.

## Reproducibility

The last column is the FNV-1a 64 of the albedo PNG's bytes. It is not
decoration: `tools/texture-forge/tests/first_generation.rs` regenerates every
stone from this directory and **fails if one byte differs**. A change to a
recipe, to the renderer or to the PNG encoder therefore shows up as a failing
test naming the asset, and this table is updated deliberately rather than
drifting.

The same test also holds the policy (exactly 16×16, albedo only) and holds the
family to issue #5's *"cada pedra visualmente distinguível"* in the one way a
test can: no two stones may share most of their palette. That is precisely the
failure `DEBT-0036` described — sixteen seeds of one stone share one palette.

## Assets

| asset_id | family | name | variant | resolution | file_path | format | alpha | seamless | recipe | prompt_reference | generation_date | version | status | integration_status | size | albedo FNV-1a 64 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---: | --- |
| `nexora:material/stone/granite_light` | stone | Light Granite | granite_light | 16×16 | `nexora/stone/granite_light/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/granite_light` | #5 prompt 01 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 388 B | `0xb8d0e791df4d634a` |
| `nexora:material/stone/granite_dark` | stone | Dark Granite | granite_dark | 16×16 | `nexora/stone/granite_dark/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/granite_dark` | #5 prompt 02 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 404 B | `0xf37044360912a88e` |
| `nexora:material/stone/limestone` | stone | Limestone | limestone | 16×16 | `nexora/stone/limestone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/limestone` | #5 prompt 03 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 242 B | `0xca335b73d4f8c075` |
| `nexora:material/stone/sandstone_rocky` | stone | Rocky Sandstone | sandstone_rocky | 16×16 | `nexora/stone/sandstone_rocky/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/sandstone_rocky` | #5 prompt 04 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 332 B | `0x943f5f2dea3f8572` |
| `nexora:material/stone/slate` | stone | Slate | slate | 16×16 | `nexora/stone/slate/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/slate` | #5 prompt 05 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 211 B | `0x96c75b10d82dc94a` |
| `nexora:material/stone/basalt` | stone | Basalt | basalt | 16×16 | `nexora/stone/basalt/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/basalt` | #5 prompt 06 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 337 B | `0x1f27929ae300938c` |
| `nexora:material/stone/volcanic_rock` | stone | Volcanic Rock | volcanic_rock | 16×16 | `nexora/stone/volcanic_rock/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/volcanic_rock` | #5 prompt 07 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 452 B | `0x56b4f535ea0b52b6` |
| `nexora:material/stone/marble_light` | stone | Light Marble | marble_light | 16×16 | `nexora/stone/marble_light/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/marble_light` | #5 prompt 08 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 237 B | `0x9c4d2c71cf11b1a4` |
| `nexora:material/stone/marble_dark` | stone | Dark Marble | marble_dark | 16×16 | `nexora/stone/marble_dark/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/marble_dark` | #5 prompt 09 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 343 B | `0xdc470c3ec91fcaa5` |
| `nexora:material/stone/quartzite` | stone | Quartzite | quartzite | 16×16 | `nexora/stone/quartzite/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/quartzite` | #5 prompt 10 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 412 B | `0xc20ec487143faa77` |
| `nexora:material/stone/ferruginous_stone` | stone | Ferruginous Stone | ferruginous_stone | 16×16 | `nexora/stone/ferruginous_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/ferruginous_stone` | #5 prompt 11 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 421 B | `0x1cb232837af8d718` |
| `nexora:material/stone/limestone_aged` | stone | Aged Limestone | limestone_aged | 16×16 | `nexora/stone/limestone_aged/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/limestone_aged` | #5 prompt 12 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 299 B | `0xd477de7d08f8f00c` |
| `nexora:material/stone/mossy_stone` | stone | Mossy Stone | mossy_stone | 16×16 | `nexora/stone/mossy_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/mossy_stone` | #5 prompt 13 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 432 B | `0x8c9b2c152471ecd6` |
| `nexora:material/stone/brittle_stone` | stone | Brittle Stone | brittle_stone | 16×16 | `nexora/stone/brittle_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/brittle_stone` | #5 prompt 14 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 238 B | `0x8cab974033376253` |
| `nexora:material/stone/mountain_stone` | stone | Mountain Stone | mountain_stone | 16×16 | `nexora/stone/mountain_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/mountain_stone` | #5 prompt 15 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 385 B | `0xeff513882c5a37a0` |
| `nexora:material/stone/ancient_stone` | stone | Ancient Stone | ancient_stone | 16×16 | `nexora/stone/ancient_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/ancient_stone` | #5 prompt 16 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered | 257 B | `0xa732f8b50ff23c15` |

`version` is the material's revision as the forge writes it: authored at v1,
generated at v2, through the pipeline at v3. `status` is the release status
from the provenance record; nothing here has been reviewed for release, so
all sixteen are `draft` and `may ship` is false.

## In the game

[`blocks.json`](blocks.json) adds each stone as a block —
`nexora:material/stone/basalt` is drawn on `nexora:block/stone/basalt` — through
the same content path a mod would use (`Block System.md` §50–51), not through
the engine's built-in set. `integration_status` records what has been
**checked**, not what is intended:

| stage | what proves it |
| --- | --- |
| block | the block registers in a world created with the content |
| meshed | its surface is its own material, not `UNMAPPED_SURFACE`, and it appears in a mesh of the placed row |
| saved | it survives a save and a reload with the content present; without it, the save is refused by name |
| recovered | it survives journal replay onto the pre-edit checkpoint |

`engine/simulation/tests/first_generation_blocks.rs` checks the first three on
every `cargo test`; the headless slice checks all four when run with
`--content content/first-generation/blocks.json`, and CI runs it that way.

What is not checked, because it does not exist yet: that the texture is
**drawn**. There is no renderer (ADR-0005), so "meshed" is as far into the game
as a surface can go today.

## What this catalog does not yet cover
- **The other families.** Ores (#6), sands (#7) and the families of #15–#26 are
  not catalogued. Each is a directory of definitions plus a directory of
  recipes, added to the manifest.
- **Thin veins.** At 16×16 the renderer's crack layer draws broad veins, not
  hairlines — the two marbles show it. A dedicated vein layer is a renderer
  change, not a recipe change.
