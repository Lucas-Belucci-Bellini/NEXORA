# NEXORA — INSTALLER / UPDATER

> Installation and updates are transactional artifact operations that preserve usable versions and never delete world saves implicitly.

## Flow
`Release Manifest → Download → Verify → Stage → Validate → Switch → Cleanup`.

## Rules
Artifacts have version, platform, architecture and checksum/signature metadata. Updates use a staging directory and atomic version switch. Previous versions remain available for rollback when configured.

## Save compatibility
The updater checks compatibility but Persistence/Versioning owns save migration.

## Repair
Reconcile installation against a trusted manifest and restore only missing/corrupt files.

## Offline
Verified cached artifacts may be reused without network access.

## API
```ts
interface IUpdater {
  check(manifest: ReleaseManifest): UpdatePlan;
  stage(plan: UpdatePlan): UpdateTransaction;
  commit(tx: UpdateTransaction): Result;
  rollback(tx: UpdateTransaction): Result;
}
```

## Tests
Interrupted update, checksum failure, rollback, disk full, incompatible save and repair.
