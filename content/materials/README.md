# Authored material definitions

The **SOURCE** stage of the pipeline in
`NEXORA CONTENT PIPELINE SPECIFICATION.md`: what a person writes, before
anything has been generated from it.

```text
content/materials/*.json        authored here, checked in, small
content/materials/manifest.json the list a batch generates, and its seed
        │
        ▼  nexora-texture-forge batch
assets/materials/               generated, not checked in
```

A definition here has `"generation": null` and a `class` of `original`,
because nothing has run yet — it is a recipe, not pixels. The copy the forge
writes under `assets/materials/` carries the full trace instead:
generator, version, pipeline, preset, seed and every parameter.

Generated output is deliberately **not** in the repository. It is derived, it
is reproducible from these files by construction, and the brief's §23 asks
that large files not be committed to demonstrate something. Regenerate everything the manifest lists with:

```sh
cargo run -p nexora-texture-forge -- batch content/materials/manifest.json
```

## The manifest

`manifest.json` is what makes a batch reproducible from the repository alone:
it names the definitions, in order, and owns the seed — `batch` refuses
`--seed`, because a run that depends on a command-line flag depends on
something that is not checked in. An entry may carry its own `"seed"`; `null`
means the manifest's.

A manifest is checked **whole before anything is written**: unreadable files,
duplicate identifiers, paths that leave this directory, and policy violations
are all reported together, and a manifest with any of them generates nothing.

`"policy"` is where a generation's rules live. The first visual generation
(issues #4, #14, #27) is 16×16 with the albedo map alone, written as:

```json
"policy": { "resolution": { "width": 16, "height": 16 }, "maps": [] }
```

The two definitions here predate that rule — 64×64 with derived PBR maps, the
forge's own test surfaces — so this manifest's policy is `null`. The
first-generation catalog has its own manifest, with the policy above, in
[`../first-generation/`](../first-generation/CATALOG.md).
