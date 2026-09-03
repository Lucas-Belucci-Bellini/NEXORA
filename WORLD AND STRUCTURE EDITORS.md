# NEXORA — WORLD / STRUCTURE EDITORS

> Editors are authoring tools over the same public world and structure contracts used at runtime.

## World Editor
Provides inspection and sandbox authoring for chunks, terrain, biomes, climate, fluids, entities, events and simulation state.

## Structure Editor
Creates templates with palettes, anchors, ports, markers, variants and validation. Placement remains owned by Structure/Build systems.

## Modes
`READ_ONLY`, `SANDBOX`, `AUTHORING`, `LOCAL_TEST`. Production worlds are protected by explicit permissions and backups.

## Architecture
```text
Editor UI
→ Tool API
→ Commands / Queries
→ Domain Systems
→ Event Bus
```

## Data
Editor projects are versioned artifacts separate from live saves. Changes can be previewed and exported to runtime-compatible content packages.

## Preview
Support terrain preview, structure rotation/variants, lighting approximation, fluid checks, navigation checks and event simulation without mutating production state.

## Collaboration
Future multi-user authoring can use versioned project documents and conflict-aware changes; live authoritative worlds remain server-controlled.

## API sketch
```ts
interface IWorldEditor {
  inspect(area: WorldArea): WorldSnapshot;
  apply(command: EditorCommand): Result;
  preview(operation: PreviewRequest): PreviewResult;
  export(project: EditorProject): ContentArtifact;
}
```

## Tests
Editor serialization, preview isolation, command authorization, structure round-trip and deterministic export.

## Invariants
- Editor preview cannot mutate production state.
- Exported structures pass normal content validation.
- World edits remain attributable and reversible in authoring mode.
