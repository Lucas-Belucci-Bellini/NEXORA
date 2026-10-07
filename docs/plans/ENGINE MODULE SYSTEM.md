# NEXORA — ENGINE MODULE SYSTEM

> Engine modules are the internal units from which Core, renderer, physics, audio, networking and other runtime services are assembled. Mod Runtime is for external content/extensions; this system is for trusted engine modules.

## Model
```text
Application
→ Module Manager
→ Module Graph
→ Load / Init
→ Runtime
→ Stop / Shutdown
```

## Module properties
`ModuleID`, version, dependencies, optional dependencies, capabilities, startup phase, runtime side and shutdown policy.

## Phases
`DISCOVERED → VALIDATED → RESOLVED → LOADED → INITIALIZED → RUNNING → STOPPING → UNLOADED`.

## Rules
- no circular module dependencies;
- explicit ownership of services/resources;
- no hidden initialization order;
- failure isolation where possible;
- deterministic startup/shutdown sequencing;
- modules expose interfaces instead of reaching into private state.

## API
```ts
interface IEngineModule {
  id(): ModuleID;
  dependencies(): ModuleDependency[];
  initialize(context: ModuleContext): Result;
  shutdown(): void;
}
```

## Integration
Core provides module lifecycle; Registry resolves definitions; Job System handles background work; Diagnostics records module health; Mod Runtime is intentionally outside this trusted engine-module boundary.

## Tests
Dependency ordering, missing module, circular dependency, startup failure, shutdown ordering and headless module selection.

## Invariants
- Module loading cannot silently bypass dependency resolution.
- A disabled client-only module must not prevent dedicated-server startup.
- External mods cannot gain trusted engine-module privileges automatically.
