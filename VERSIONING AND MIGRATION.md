# NEXORA — VERSIONING / MIGRATION SYSTEM

> Versioning makes schemas, registries, saves, network protocols, mods and content evolvable without silently corrupting state.

## Version domains
```text
Engine Version
API Version
Content Version
Registry Version
Save Format
Network Protocol
Mod API
Script API
Resource Format
```

Never infer compatibility from a display version alone; record explicit format/API identifiers and compatibility ranges.

## Migration model
```text
old state
→ detect version
→ validate source
→ plan migration
→ dry run
→ transform
→ validate target
→ commit
```

## Save migrations
Persistence owns physical migration execution. Migrations are ordered, deterministic and idempotent where possible.

## Registry migrations
Handle renamed/deprecated IDs through aliases and explicit migration maps. Never silently reinterpret an ID as a different object.

## Network protocol
Handshake negotiates supported protocol versions and content/registry fingerprints. Incompatible peers are rejected cleanly.

## Mod API
Deprecations carry removal targets and replacement APIs. Compatibility shims live outside Core where practical.

## Migration registry
```ts
interface IMigrationRegistry {
  register(migration: MigrationDefinition): void;
  plan(source: Version, target: Version): MigrationPlan;
  validate(plan: MigrationPlan): ValidationReport;
}
```

## Dry run
Every destructive or format-changing migration should support dry-run reporting:
- changed records;
- deprecated IDs;
- missing dependencies;
- irrecoverable fields;
- estimated output.

## Backups
Before save-format migration, create or validate a restorable backup/checkpoint.

## Failure handling
A failed migration must preserve the original source. Partial target output is quarantined.

## Provenance
Record source version, migration chain, tool version and result fingerprint.

## Tests
Migration every supported version pair, downgrade rejection, corrupted input, missing mod content, renamed IDs and interrupted migration.

## Invariants
- Unknown data is not silently discarded.
- Migrations are explicit and reviewable.
- Original data remains recoverable until target validation succeeds.
