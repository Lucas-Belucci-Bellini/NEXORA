# NEXORA — VERSIONING / MIGRATION

> Explicit version domains let NEXORA evolve without silently changing or corrupting state.

## Domains
Engine, API, content, registry, save format, network protocol, mod API, script API and resource format.

## Migration
`Detect → Validate Source → Plan → Dry Run → Transform → Validate Target → Commit`.

## Rules
Renamed IDs use aliases/migration maps. Unknown fields are not silently discarded. Failed migrations preserve the original source and quarantine partial output.

## Backups
Save migrations require a restorable checkpoint/backup before commit.

## API
```ts
interface IMigrationRegistry {
  register(migration: MigrationDefinition): void;
  plan(source: Version, target: Version): MigrationPlan;
  validate(plan: MigrationPlan): ValidationReport;
}
```

## Tests
Every supported version pair, corrupted input, missing mod data, renamed ID and interrupted migration.
