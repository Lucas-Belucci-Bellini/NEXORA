# ADR-0019 — A recipe is data, and a material names it

- **Status:** ACCEPTED
- **Date:** 2026-09-24
- **Closes:** `DEBT-0044` (*"Uma definição não escolhe paleta: dezesseis pedras
  são uma pedra com dezesseis sementes"*)
- **Changes:** the material document schema, 2 → 3 (serialization and public
  identifiers, both on the index's list of what needs an ADR)
- **Amended at merge (2026-09-25):** written as 1 → 2 while `main`, in
  parallel, took schema 2 for the texture-forge backend seam (a generation
  record names its `backend`). `main`'s number stands; `recipe` is schema 3,
  and every step below reads with that number

## Context

The procedural generator drew every material with
`Recipe::for_category(definition.category())`. The recipe — palette, noise
lattice, cracks, boards, courses, grains — was chosen by category and by nothing
else. `recipe.rs` already called those defaults *"starting points a material
document overrides"*, but no document could override anything: the material
format had no field to do it with.

The first visual generation (issues #4, #14, #27) catalogues families in 16×16:
sixteen stones (#5), forty-eight ores (#6), twenty-four sands (#7), and the
families of #15–#26. Issue #5 requires each stone to be *"visualmente
distinguível"*.

## Problem

With category-only recipes, sixteen stone definitions render as the same grey
mottled stone, differing only in where the grains fall. The batch (FASE 7) can
already impose the 16×16 / albedo-only policy; it had nothing distinct to
generate under it.

## Options

1. **Presets in code** — add `Recipe::preset(id)` with a table of numbers, like
   `for_category`. Cheap, but every new family member is a code change and a
   generator version bump, and the art direction for hundreds of assets would
   live in a Rust `match`.
2. **Inline recipe in the material document** — the material carries its own
   palette and lattice. Self-contained, but the runtime crate (`engine/asset`)
   would then model art direction, which the audit (§4.4) and `material.rs`
   both place in the tool; and a family that shares one look would repeat it in
   every file.
3. **Recipes as documents, named by identifier** — a recipe is its own strict
   JSON file; the material names it by `Identifier`, exactly the way
   `Block System.md` §21 has blocks name textures.

## Decision

Option 3.

- `SurfaceMaterial` gains `recipe: Option<Identifier>`. `engine/asset` holds the
  **reference** only; what a recipe contains is the tool's business.
- The material document goes to **schema 3**, where `"recipe"` is required and
  is an identifier or `null`. Schema 1 and 2 documents still read, as
  `recipe = null`; an older document that carries the field is refused,
  because no build wrote one. An older build refuses schema 3 by the existing
  "written by a newer build" rule.
- `appearance_hash` includes the recipe **only when present**, so every
  material without one keeps its hash, its derived seed and its pixels.
- A recipe lives at a path derived from its identifier —
  `nexora:recipe/stone/basalt` → `<recipes>/nexora/stone/basalt.json` — and a
  file whose contents claim another identifier is refused.
- A recipe must draw the same **category** as the material that names it.
- The generator records the recipe as the trace's `preset`, and records a
  **fingerprint** of the recipe's canonical text as the trace parameter
  `recipe_fingerprint`. The forge compares it before calling a material
  `unchanged`.
- A batch resolves every entry's recipe **before writing anything**.

## Consequences

- A new family member is a file of numbers. No code, no generator version bump.
- Editing a recipe file is noticed even though the material that names it did
  not change: the fingerprint moves, the forge refuses without `--force` and
  replaces with it. Without the fingerprint the forge would have called stale
  pixels `unchanged` indefinitely — the identifier is the same, so the
  material's own hash cannot see the edit.
- Category defaults remain code and remain versioned by
  `PROCEDURAL_VERSION`, as before.
- The recipe directory is a CLI option (`--recipes`, default `content/recipes`),
  like the output root. The manifest does not name it; the fingerprint in each
  trace is what ties an output to the recipe that drew it.

## Migration

None required. Every checked-in definition is schema 1 and reads unchanged;
every generated `material.json` is rewritten as schema 3 the next time it is
generated, and the forge reports it `unchanged` until then because neither the
appearance hash nor the fingerprint (absent on both sides) moved.

## Compatibility

A schema 3 document cannot be read by a build older than this ADR, and says so
by name rather than failing on an unknown field.
