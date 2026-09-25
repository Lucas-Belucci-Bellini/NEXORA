# First visual generation — catalog

The **catalog mestre** of issue #27, for what exists so far: the sixteen stones
of issue #5. Every row is a first-generation asset under the rule the operator
set for 2026 — **16×16, albedo only** — and that rule is not a promise here: it
is the `policy` of [`plan.json`](plan.json), and the forge refuses to
generate anything from that build plan that breaks it.

```sh
cargo run -p nexora-texture-forge -- build content/first-generation/plan.json
```

## How these were made

**Procedurally, not from the prompts.** Issue #4 asks for *"prompt individual
ou regra de geração reproduzível"* per asset; these are the second kind. Each
stone is:

```text
content/first-generation/stone/<variant>.json   the material: id, 16×16, no optional maps,
                                                and a provenance note naming its #5 prompt
content/recipes/nexora/stone/<variant>.json     the recipe: palette, lattice, cracks,
                                                layers, grains (ADR-0019)
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

It was updated once so far, on **2026-09-25**, for exactly that reason: the
engine's compressor learned dynamic-Huffman blocks and lazy matching
(`nexora_foundation::deflate`), and every albedo came out smaller — 5,390
bytes across the sixteen before, 4,889 after. **The pixels did not change**:
both encodings were decoded by Python's `zlib`, which is not this project's
code, and compared texel for texel. No recipe, seed or version moved, which is
why the `version` column did not.

The same test also holds the policy (exactly 16×16, albedo only) and holds the
family to issue #5's *"cada pedra visualmente distinguível"* in the one way a
test can: no two stones may share most of their palette. That is precisely the
failure `DEBT-0044` described — sixteen seeds of one stone share one palette.

## Assets

| asset_id | family | name | variant | resolution | file_path | format | alpha | seamless | recipe | prompt_reference | generation_date | version | status | integration_status | size | albedo FNV-1a 64 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---: | --- |
| `nexora:material/stone/granite_light` | stone | Light Granite | granite_light | 16×16 | `nexora/stone/granite_light/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/granite_light` | #5 prompt 01 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 344 B | `0xa2817db906db5171` |
| `nexora:material/stone/granite_dark` | stone | Dark Granite | granite_dark | 16×16 | `nexora/stone/granite_dark/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/granite_dark` | #5 prompt 02 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 372 B | `0x289c1061357df0bd` |
| `nexora:material/stone/limestone` | stone | Limestone | limestone | 16×16 | `nexora/stone/limestone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/limestone` | #5 prompt 03 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 221 B | `0x7b6078751755eca9` |
| `nexora:material/stone/sandstone_rocky` | stone | Rocky Sandstone | sandstone_rocky | 16×16 | `nexora/stone/sandstone_rocky/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/sandstone_rocky` | #5 prompt 04 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 300 B | `0x76cfd3511ce00dff` |
| `nexora:material/stone/slate` | stone | Slate | slate | 16×16 | `nexora/stone/slate/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/slate` | #5 prompt 05 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 202 B | `0xe54f708a44da2cf7` |
| `nexora:material/stone/basalt` | stone | Basalt | basalt | 16×16 | `nexora/stone/basalt/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/basalt` | #5 prompt 06 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 293 B | `0x37336d8e614f13ba` |
| `nexora:material/stone/volcanic_rock` | stone | Volcanic Rock | volcanic_rock | 16×16 | `nexora/stone/volcanic_rock/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/volcanic_rock` | #5 prompt 07 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 423 B | `0x9a64d910e01adcf5` |
| `nexora:material/stone/marble_light` | stone | Light Marble | marble_light | 16×16 | `nexora/stone/marble_light/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/marble_light` | #5 prompt 08 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 219 B | `0x63fa18f8bbe4bfe9` |
| `nexora:material/stone/marble_dark` | stone | Dark Marble | marble_dark | 16×16 | `nexora/stone/marble_dark/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/marble_dark` | #5 prompt 09 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 314 B | `0xbdcc3791cceb59ae` |
| `nexora:material/stone/quartzite` | stone | Quartzite | quartzite | 16×16 | `nexora/stone/quartzite/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/quartzite` | #5 prompt 10 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 372 B | `0xcbc75dd68ee40ab3` |
| `nexora:material/stone/ferruginous_stone` | stone | Ferruginous Stone | ferruginous_stone | 16×16 | `nexora/stone/ferruginous_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/ferruginous_stone` | #5 prompt 11 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 374 B | `0x34ca545eeec1150c` |
| `nexora:material/stone/limestone_aged` | stone | Aged Limestone | limestone_aged | 16×16 | `nexora/stone/limestone_aged/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/limestone_aged` | #5 prompt 12 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 271 B | `0xf98756ae7125f08a` |
| `nexora:material/stone/mossy_stone` | stone | Mossy Stone | mossy_stone | 16×16 | `nexora/stone/mossy_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/mossy_stone` | #5 prompt 13 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 404 B | `0x0c1f29468123ad63` |
| `nexora:material/stone/brittle_stone` | stone | Brittle Stone | brittle_stone | 16×16 | `nexora/stone/brittle_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/brittle_stone` | #5 prompt 14 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 215 B | `0x816f2570a18e0a0f` |
| `nexora:material/stone/mountain_stone` | stone | Mountain Stone | mountain_stone | 16×16 | `nexora/stone/mountain_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/mountain_stone` | #5 prompt 15 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 334 B | `0xc9ba73cb4d6677c1` |
| `nexora:material/stone/ancient_stone` | stone | Ancient Stone | ancient_stone | 16×16 | `nexora/stone/ancient_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/ancient_stone` | #5 prompt 16 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 231 B | `0x60fc4c441a97f7d8` |

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
| texture | its albedo is resolved by identifier through the forge's `resources.json`, hash-checked and decoded to 16×16 by the runtime (ADR-0021, ADR-0022) |

`engine/simulation/tests/first_generation_blocks.rs` checks the first three on
every `cargo test`; the headless slice checks the next one when run with
`--content content/first-generation/blocks.json`, and the texture stage when
also given `--resources <forge output>`. CI runs both.

What is not checked, because it does not exist yet: that the texture is
**drawn**. There is no renderer (ADR-0005), so "meshed" is as far into the game
as a surface can go today.

## What this catalog does not yet cover
- **The other families.** Ores (#6), sands (#7) and the families of #15–#26 are
  not catalogued. Each is a directory of definitions plus a directory of
  recipes, added to the build plan.
- **Thin veins.** At 16×16 the renderer's crack layer draws broad veins, not
  hairlines — the two marbles show it. A dedicated vein layer is a renderer
  change, not a recipe change.
