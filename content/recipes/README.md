# Recipes

What a procedural material looks like, as numbers — palette, noise lattice,
cracks, boards, courses, grains. A material names one with its `"recipe"`
field ([ADR-0019](../../docs/adr/ADR-0019-a-recipe-is-data-and-a-material-names-it.md)):

```text
nexora:recipe/stone/basalt   ->   content/recipes/nexora/stone/basalt.json
```

The path is derived from the identifier and never the other way round; a file
whose `"id"` is not the one its path implies is refused. A recipe must draw the
same `category` as the material that names it.

Every field is required and range-checked; see `tools/texture-forge/src/recipe_book.rs`
for the bounds. Ramp stops are `#rrggbb`, darkest first. `courses.rows` must be
even, or the offset bond does not tile.

**Editing a recipe is a change of appearance.** The forge records each
recipe's fingerprint in the materials it draws, so after an edit it refuses to
call them `unchanged` and asks for `--force` — and the first-generation test
fails until `content/first-generation/CATALOG.md` is updated to the new bytes.

The directory the forge reads is `--recipes` (default `content/recipes`).
