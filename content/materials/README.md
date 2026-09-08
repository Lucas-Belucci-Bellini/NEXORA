# Authored material definitions

The **SOURCE** stage of the pipeline in
`NEXORA CONTENT PIPELINE SPECIFICATION.md`: what a person writes, before
anything has been generated from it.

```text
content/materials/*.json   authored here, checked in, small
        │
        ▼  nexora-texture-forge generate
assets/materials/          generated, not checked in
```

A definition here has `"generation": null` and a `class` of `original`,
because nothing has run yet — it is a recipe, not pixels. The copy the forge
writes under `assets/materials/` carries the full trace instead:
generator, version, pipeline, preset, seed and every parameter.

Generated output is deliberately **not** in the repository. It is derived, it
is reproducible from these files by construction, and the brief's §23 asks
that large files not be committed to demonstrate something. Regenerate with:

```sh
cargo run -p nexora-texture-forge -- generate content/materials/stone_rough.json
```
