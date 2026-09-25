# Authored material definitions

The **SOURCE** stage of the pipeline in
`NEXORA CONTENT PIPELINE SPECIFICATION.md`: what a person writes, before
anything has been generated from it.

```text
content/materials/*.json        authored here, checked in, small
content/materials/plan.json     the build plan: the list to generate, and its seed
        │
        ▼  nexora-texture-forge build
assets/materials/               generated, not checked in
```

A definition here has `"generation": null` and a `class` of `original`,
because nothing has run yet — it is a recipe, not pixels. The copy the forge
writes under `assets/materials/` carries the full trace instead:
generator, version, pipeline, preset, seed and every parameter.

Generated output is deliberately **not** in the repository. It is derived, it
is reproducible from these files by construction, and the brief's §23 asks
that large files not be committed to demonstrate something. Regenerate everything the build plan lists with:

```sh
cargo run -p nexora-texture-forge -- build content/materials/plan.json
```

## The build plan

Not to be confused with a batch manifest (`content/manifests/`), which
*declares* materials inline from defaults. A build plan *points at*
definitions that already exist here, and rebuilds exactly those.

`plan.json` is what makes a build reproducible from the repository alone:
it names the definitions, in order, and owns the seed — `build` refuses
`--seed`, because a run that depends on a command-line flag depends on
something that is not checked in. An entry may carry its own `"seed"`; `null`
means the plan's.

A plan is checked **whole before anything is written**: unreadable files,
duplicate identifiers, paths that leave this directory, and policy violations
are all reported together, and a plan with any of them generates nothing.

`"policy"` is where a generation's rules live. The first visual generation
(issues #4, #14, #27) is 16×16 with the albedo map alone, written as:

```json
"policy": { "resolution": { "width": 16, "height": 16 }, "maps": [] }
```

The two definitions here predate that rule — 64×64 with derived PBR maps, the
forge's own test surfaces — so this plan's policy is `null`. The
first-generation catalog has its own plan, with the policy above, in
[`../first-generation/`](../first-generation/CATALOG.md).
