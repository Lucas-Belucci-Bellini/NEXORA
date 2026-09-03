# NEXORA — TOOLING ARCHITECTURE

> Tooling is a first-class part of NEXORA: editors, inspectors, profilers, validators and CLI utilities operate through public engine contracts instead of private hacks.

## Tool families
```text
CLI
Editors
Inspectors
Debuggers
Profilers
World tools
Content tools
Mod tools
Build tools
Test tools
```

## CLI
Common entry point:
```text
nexora <domain> <command>
```
Domains include `world`, `chunk`, `entity`, `item`, `registry`, `event`, `save`, `mod`, `jobs`, `perf`, `nav`, `industry`, `civilization` and `resource`.

## Editor architecture
Tools use service interfaces and command/query contracts. They must not mutate engine memory directly.

## World Editor
Future capabilities: inspect chunks, edit terrain, visualize biomes/climate/light, test structures, spawn entities and preview events with explicit non-production worlds.

## Structure Editor
Template creation, anchors, ports, palettes, variants, validation and preview. Final placement uses Structure/Build systems.

## Content Editors
Data-driven editing for items, blocks, recipes, machines, quests, events, technologies and civilizations.

## Debug overlays
Renderer/UI may expose diagnostic overlays, but diagnostic state remains owned by tooling services.

## Remote tooling
Dedicated servers may expose authenticated administration interfaces. Access is capability-based and auditable.

## Safety
Tools can operate in `READ_ONLY`, `SANDBOX`, `LOCAL_DEV` and `ADMIN_PRODUCTION` modes. Destructive actions require explicit authority.

## API sketch
```ts
interface IToolService {
  inspect(request: InspectRequest): InspectResult;
  execute(command: ToolCommand): ToolResult;
  registerPanel(panel: ToolPanelDefinition): void;
}
```

## Mod SDK integration
Tooling supplies project templates, validators, schema discovery, logs and local test worlds for mod authors.

## Tests
Golden CLI output, editor serialization, command authorization, read-only guarantees and sandbox isolation.

## Invariants
- Tooling does not bypass Command/Server authority.
- Production destructive operations are auditable.
- Editor files are portable and versioned.
