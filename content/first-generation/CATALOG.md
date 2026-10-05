# First visual generation — catalog

The **catalog mestre** of issue #27, for what exists so far: the sixteen stones
of issue #5, ten terrain materials — dry earth, moist earth, wet mud, clay,
gravel, snow and ash from issue #17, pale quartz sand from issue #7, grass and
moss from issue #16 — and five construction materials from issue #15: brick,
concrete, worked stone, floor tile and plank floor. Every row is a first-generation asset under the rule the operator
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
| `nexora:material/stone/mountain_stone` | stone | Mountain Stone | mountain_stone | 16×16 | `nexora/stone/mountain_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/mountain_stone` | #5 prompt 15 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture · drawn | 334 B | `0xc9ba73cb4d6677c1` |
| `nexora:material/stone/ancient_stone` | stone | Ancient Stone | ancient_stone | 16×16 | `nexora/stone/ancient_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/ancient_stone` | #5 prompt 16 | 2026-09-24 | v3 | draft | block · meshed · saved · recovered · texture | 231 B | `0x60fc4c441a97f7d8` |
| `nexora:material/soil/dry_earth` | soil | Dry Earth | dry_earth | 16×16 | `nexora/soil/dry_earth/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/soil/dry_earth` | #17 prompt 01 | 2026-10-03 | v3 | draft | block · meshed · saved · recovered · texture · drawn | 361 B | `0x471d49ded5686c0b` |
| `nexora:material/soil/clay` | soil | Clay | clay | 16×16 | `nexora/soil/clay/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/soil/clay` | #17 prompt 05 | 2026-10-03 | v3 | draft | block · meshed · saved · recovered · texture | 241 B | `0xc4e824f0107c8487` |
| `nexora:material/sand/gravel` | sand | Gravel | gravel | 16×16 | `nexora/sand/gravel/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/sand/gravel` | #17 prompt 06 | 2026-10-03 | v3 | draft | block · meshed · saved · recovered · texture | 364 B | `0x961b635a407186d8` |
| `nexora:material/sand/quartz_sand` | sand | Quartz Sand | quartz_sand | 16×16 | `nexora/sand/quartz_sand/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/sand/quartz_sand` | #7 prompt 01 | 2026-10-03 | v3 | draft | block · meshed · saved · recovered · texture | 397 B | `0xe5852d5b9984e9dc` |
| `nexora:material/vegetation/grass` | vegetation | Grass | grass | 16×16 | `nexora/vegetation/grass/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/vegetation/grass` | #16 prompt 06 | 2026-10-03 | v3 | draft | block · meshed · saved · recovered · texture · drawn | 337 B | `0x8bd25abb9625597e` |
| `nexora:material/brick/nexora_brick` | brick | NEXORA Brick | nexora_brick | 16×16 | `nexora/brick/nexora_brick/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/brick/nexora_brick` | #15 prompt 01 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 274 B | `0x736e720c8d8a6ebb` |
| `nexora:material/concrete/concrete` | concrete | Concrete | concrete | 16×16 | `nexora/concrete/concrete/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/concrete/concrete` | #15 prompt 02 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 310 B | `0xe07a7fb010ce0140` |
| `nexora:material/stone/worked_stone` | stone | Worked Stone | worked_stone | 16×16 | `nexora/stone/worked_stone/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/stone/worked_stone` | #15 prompt 04 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 332 B | `0xa2bb7e8ea959068c` |
| `nexora:material/ceramic/floor_tile` | ceramic | Floor Tile | floor_tile | 16×16 | `nexora/ceramic/floor_tile/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/ceramic/floor_tile` | #15 prompt 05 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 220 B | `0x4a91ba496e279034` |
| `nexora:material/wood/plank_floor` | wood | Plank Floor | plank_floor | 16×16 | `nexora/wood/plank_floor/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/wood/plank_floor` | #15 prompt 13 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 260 B | `0x330385091b17ce4b` |
| `nexora:material/soil/moist_earth` | soil | Moist Earth | moist_earth | 16×16 | `nexora/soil/moist_earth/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/soil/moist_earth` | #17 prompt 02 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 385 B | `0xb0fac740a0f9fca1` |
| `nexora:material/soil/wet_mud` | soil | Wet Mud | wet_mud | 16×16 | `nexora/soil/wet_mud/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/soil/wet_mud` | #17 prompt 04 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 305 B | `0xcc614d552083ff29` |
| `nexora:material/mineral/snow` | mineral | Snow | snow | 16×16 | `nexora/mineral/snow/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/mineral/snow` | #17 prompt 08 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 280 B | `0x5df026b1c0a47d7f` |
| `nexora:material/mineral/ash` | mineral | Ash | ash | 16×16 | `nexora/mineral/ash/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/mineral/ash` | #17 prompt 12 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 395 B | `0x7b7781c3ce15d186` |
| `nexora:material/vegetation/moss` | vegetation | Moss | moss | 16×16 | `nexora/vegetation/moss/albedo.png` | PNG RGBA8 | opaque | yes | `nexora:recipe/vegetation/moss` | #16 prompt 07 | 2026-10-05 | v3 | draft | block · meshed · saved · recovered · texture | 424 B | `0x4efaa4d070e59442` |

`version` is the material's revision as the forge writes it: authored at v1,
generated at v2, through the pipeline at v3. `status` is the release status
from the provenance record; nothing here has been reviewed for release, so
all thirty-one are `draft` and `may ship` is false.

## The terrain set (2026-10-03)

The five terrain materials were made exactly as the stones were: a material
under `soil/`, `sand/` or `vegetation/` naming a recipe under
`content/recipes/nexora/`, the prompt read as art direction and turned into a
ramp, a mottle and a speckle — no new forge code, no image model. Two took a
second recipe after being looked at enlarged and tiled 3×3: the first gravel
read as dark noise with pale flecks rather than separate stones, and the
first grass tiled into regular vertical stripes. Both were redrawn with a
lighter bed and shorter clusters before anything was catalogued.

**The surface of the generated world, since ADR-0037.** The world generator
places its own `nexora:block/stone`, `dirt` and `grass`; block content
schema 2 lets `blocks.json` restyle them (`"surfaces"`), and it draws them in
the mountain stone, the dry earth and the grass. Those three are the rows
marked `drawn` below. The other materials reach the game as content blocks
(`nexora:block/soil/dry_earth`, …), which the slice registers, places,
meshes, saves, recovers and textures.

## The second terrain set (2026-10-05)

Five more surfaces for the ground, from issue #17 — moist earth (prompt 02),
wet mud (04), snow (08), ash (12) — and moss from issue #16 (prompt 07): the
materials a biome other than the first one needs underfoot. Snow and ash are
`mineral`, the forge's category list having no frozen or burned ground. Made
exactly as the first terrain set was — a material and a recipe, the prompt
read as art direction, a ramp, a mottle and a speckle, no new forge code and
no image model — and looked at enlarged and tiled 3×3 under their real
identifiers before their hashes were taken
([`docs/texture-forge/first-generation-terrain-set-2.png`](../../docs/texture-forge/first-generation-terrain-set-2.png),
in this table's order). The moist earth is the dry earth's structure a step
darker with almost no cracks; the mud is greyer and flatter still, with fewer
flecks, so the two stay apart at 16×16 and in the palette test.

## The construction set (2026-10-05)

Five materials from issue #15 (*Blocos e Construção*), chosen because the
player can now build ([ADR-0036](../../docs/adr/ADR-0036-a-player-edits-through-commands-and-a-save-keeps-the-player.md))
and the first generation held no construction material: brick (prompt 01),
concrete (02), worked stone (04), floor tile (05) and plank floor (13). Made
as the stones were — a material under `brick/`, `concrete/`, `stone/`,
`ceramic/` or `wood/` naming a recipe under `content/recipes/nexora/`, the
prompt read as art direction — and with no new forge code: the brick, the
worked stone and the floor tile use the recipe's `courses`, the floor its
`strips`.

Each was looked at enlarged and tiled 3×3 before its hash was taken
([`docs/texture-forge/first-generation-construction-set.png`](../../docs/texture-forge/first-generation-construction-set.png),
in this table's order), and four needed more than one recipe:

- **Mortar at 16×16 is two texels.** The course layer measures each texel
  centre's distance to the nearest joint, and a joint has a texel on each side,
  so a joint is two texels wide whatever `mortar` says. Four courses of 4 texels
  left 2-texel bricks; two courses with one unit each (14×6 bricks in a running
  bond) read as brick.
- **The joint is the top of the ramp.** Mortar is drawn pale and lands on the
  last stops; with six even stops, bright bricks became mortar and the mortar
  stayed orange. Seven stops with only the last one pale, less jitter and more
  contrast keep the two apart — for the brick and for the worked stone.
- **The noise follows the material's identifier**, so a recipe tried under a
  test name is not the texture the plan draws; the worked stone and the floor
  tile lost their joints and their grid under their real names and were
  redrawn there (less mottle, more contrast).
- **No roof tile.** Prompt 07 was tried with the course layer four ways —
  inverted ramp, boards, small and large courses — and read as noise or as
  planks every time; overlapping tiles need a layer the forge does not have.
  The plank floor of prompt 13 took its place.

What none of them is yet: on screen. Each is a tile in the client's atlas
(ADR-0037), but the block the player builds is the generator's own stone
(ADR-0036), and no generated world places a construction block; choosing what
to build is the inventory's (Phase 5).

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
| drawn | the client shows it in a window, from its tile in the atlas, and the frame read back from the surface holds texel for texel against the ray cast through the same atlas (ADR-0037) |

`engine/simulation/tests/first_generation_blocks.rs` checks the first three on
every `cargo test`; the headless slice checks the next one when run with
`--content content/first-generation/blocks.json`, and the texture stage when
also given `--resources <forge output>`. CI runs both, and runs the client
with the same two arguments for the drawn stage.

Every material is a tile in the client's atlas, but `drawn` is only claimed
where a judged frame shows it: the three the generated terrain is made of. A
block no world places is never on a frame a ray cast checks.

## What this catalog does not yet cover
- **The other families.** Ores (#6), sands (#7) and the families of #15–#26 are
  not catalogued. Each is a directory of definitions plus a directory of
  recipes, added to the build plan.
- **Thin veins.** At 16×16 the renderer's crack layer draws broad veins, not
  hairlines — the two marbles show it. A dedicated vein layer is a renderer
  change, not a recipe change.
