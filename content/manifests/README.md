# Material set manifests

A manifest declares a whole set of materials in one file, so that a biome, a
structure kit or a block family is written once rather than once per material.
It is the same SOURCE stage as [`../materials/`](../materials/) — authored,
checked in, small — and it produces exactly the same output.

```sh
cargo run -p nexora-texture-forge -- batch content/manifests/temperate_forest.json
```

## What a manifest adds over eight separate definitions

| | separate definitions | one manifest |
| --- | --- | --- |
| shared settings | repeated in each file | `defaults`, overridden per entry |
| grouping | by convention | `prefix`, which becomes a path segment |
| provenance | per file | one author and one `source_tool` for the set |
| a bad entry | stops that file | reported, and the rest still build |

## Grouping is namespacing

`"prefix": "temperate_forest"` puts every entry at
`nexora:material/temperate_forest/<name>`. Nothing had to learn a new concept
for that: identifiers already nest, the layout already turns nesting into
directories, and the registry already sorts by identifier. It also means the
same short name can appear in two sets — `forest/soil` and `desert/soil` are
two materials, not a collision.

## Inheritance

An entry takes `defaults` and overrides only what it names, down to a single
number inside `pbr`. A field nobody names falls back to the same value a
hand-written definition falls back to, so a manifest and a definition describe
the same material when they say the same things — there is a test that asserts
exactly that.

## One bad entry does not stop the rest

`batch` carries on past a failure, reports every one of them, and exits
non-zero. Stopping at the first problem would mean fixing one material,
re-running, waiting, and finding the next — which is the loop the batch exists
to replace.

## Limits

`schema` is refused if it is newer than the build reading it, a manifest may
declare at most 4096 materials, unknown fields are refused rather than ignored,
and two entries claiming one identifier are refused before anything is written.
