# NEXORA — INSTALLER / UPDATER

> Installation and updates are transactional artifact operations. They must preserve a usable installation and never modify world saves implicitly.

## Pipeline
```text
Release Manifest
→ Download
→ Verify
→ Stage
→ Validate
→ Switch
→ Cleanup
```

## Artifacts
Game binaries, assets, dedicated server, tools and official content packages are identified by version, platform, architecture and checksum.

## Atomic update
Install into a staging directory, verify all required files, then switch the active version pointer. Retain rollback data until the new version is known-good.

## Save compatibility
The updater must check save-format compatibility before launch. Migration is owned by Versioning/Persistence, not the installer.

## Repair
Reconcile installation against a trusted manifest and replace only missing/corrupt files.

## Offline support
Previously downloaded verified artifacts can be reused without network access.

## Security
Verify signatures/checksums according to release policy. Reject tampered or unexpected artifacts. Avoid executing downloaded files before verification.

## API sketch
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

## Invariants
- Updates are atomic from the user's perspective.
- Existing saves are never deleted by an update.
- An unverified artifact cannot become active.
