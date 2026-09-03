# NEXORA — TOOLING ARCHITECTURE

> Tooling provides CLI, editors, inspectors, profilers and validation tools over public NEXORA contracts.

## Families
`CLI`, `Editors`, `Inspectors`, `Debuggers`, `Profilers`, `World Tools`, `Content Tools`, `Mod Tools`, `Build/Test Tools`.

## Rule
Tools use Query/Command/Tool APIs and never mutate engine memory directly.

## Modes
`READ_ONLY`, `SANDBOX`, `LOCAL_DEV`, `ADMIN_PRODUCTION`.

## Examples
`nexora world inspect`, `chunk`, `entity`, `registry`, `save`, `mod`, `jobs`, `perf`, `nav`, `industry`, `civilization`.

## API
```ts
interface IToolService {
  inspect(request: InspectRequest): InspectResult;
  execute(command: ToolCommand): ToolResult;
  registerPanel(panel: ToolPanelDefinition): void;
}
```

## Tests
Read-only guarantees, authorization, sandbox isolation and editor serialization.
