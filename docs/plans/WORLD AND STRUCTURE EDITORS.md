# NEXORA — WORLD / STRUCTURE EDITORS

> Editors are authoring tools over the same public world and structure contracts used at runtime.

## World Editor
Inspect chunks, terrain, biomes, climate, fluids, entities, events and simulation state. Support sandbox authoring and previews.

## Structure Editor
Create templates with palettes, anchors, ports, markers, variants and validation. Placement still belongs to Structure/Build systems.

## Modes
`READ_ONLY`, `SANDBOX`, `AUTHORING`, `LOCAL_TEST`. Production worlds require explicit authority and protection.

## Flow
`Editor UI → Tool API → Queries/Commands → Domain Systems → Events`.

## Export
Editor projects are versioned artifacts and can be exported through the Content Pipeline into runtime-compatible packages.

## API
```ts
interface IWorldEditor {
  inspect(area: WorldArea): WorldSnapshot;
  apply(command: EditorCommand): Result;
  preview(operation: PreviewRequest): PreviewResult;
  export(project: EditorProject): ContentArtifact;
}
```

## Tests
Serialization, preview isolation, authorization, structure round-trip and deterministic export.
