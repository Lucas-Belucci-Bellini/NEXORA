# ADR-0014 — Content blocks enter through the API a mod uses

- **Status:** ACCEPTED
- **Date:** 2026-09-24
- **Follows:** [ADR-0013](ADR-0013-a-recipe-is-data-and-a-material-names-it.md)
  (the sixteen first-generation stones it made possible needed blocks)
- **Touches:** mod API, public IDs — both on the index's list

## Context

`World::create` registered a fixed block set — `air`, `stone`, `dirt`, `grass`
— and froze the registry. There was no other way for a block to exist. The
sixteen 16×16 stones of issue #5 had materials, recipes and a catalog, and no
block that used them: `integration_status: no block assigned`.

`Block System.md` §50: *"Conteúdo oficial deve utilizar exatamente a mesma
API"* as mods — not `Vanilla → private API, Mods → limited API`. §51:
*"idealmente um bloco pode ser descrito por dados."*

## Options

1. **Add the sixteen to `World::create`.** One line each. It is exactly the
   private path §50 forbids, and every future family (#6, #7, #15–#26) would
   grow the engine's source instead of the content directory.
2. **A block content document, registered through a public constructor.**

## Decision

Option 2.

- `World::create_with(descriptor, calendar, content)` registers content blocks
  **after** the four built-ins, in the order given, then freezes. `create` is
  `create_with(.., &[])`. The built-ins stay because terrain generation and
  air's runtime id 0 depend on them; they are the engine's, not content.
- A content block that collides with a built-in or another content block is
  refused by the registry — it cannot replace `stone`.
- `persist::load_with(container, content)` reopens a save with its content.
  A save naming a block the content does not supply is refused **by name**,
  exactly as a missing mod would be; `load` is `load_with(.., &[])`. Saves
  already store identifiers, never runtime ids (`Registry System.md` §12), so
  no save format change is needed.
- `nexora_simulation::content::BlockContent` reads a strict JSON document
  (`schema`, `materials`, `blocks[] { id, solid, surface }`), registers the
  listed material definitions and resolves each block's surface. A block whose
  surface the document does not list is refused at load. The field is
  `surface`, not `material`: "material" is the physics material
  (`NEXORA NAMING AND TERMINOLOGY.md`).

## Consequences

- The first generation's stones are blocks, meshed with their own surfaces,
  saved, reloaded and recovered from the journal — checked by
  `engine/simulation/tests/first_generation_blocks.rs` and by the headless
  slice's `--content` run in CI.
- Adding a family is a content change: definitions, recipes, a line in
  `blocks.json`. No engine code.
- A world's runtime ids now depend on its content list. That is safe for saves
  (identifiers), and it is the same property a mod list already had.

## Compatibility

Existing saves load unchanged through `load`: they name only built-ins. The
headless slice without `--content` produces the same numbers and the same save
bytes as before this ADR.
