# NEXORA — MOD DISTRIBUTION SYSTEM

> Mod Distribution discovers, verifies, resolves and delivers community packages. Runtime remains responsible for loading and executing them.

## Model
```text
Repository
→ Metadata
→ Dependency Resolution
→ Download
→ Verify
→ Install
→ Mod Runtime
```

## Package metadata
ID, version, game/API compatibility, dependencies, permissions, license, provenance, supported sides and content fingerprint.

## Resolution
Choose compatible versions using explicit constraints. Reject dependency cycles/conflicts and report why resolution failed.

## Verification
Verify checksum/signature where supported before installation. Package contents are scanned by the Content Pipeline/Mod Runtime validators.

## Installation
Install into isolated mod directories. Never overwrite official engine artifacts.

## Updates
Keep previous compatible versions for rollback when storage permits. Save compatibility is checked separately.

## Removal
Disable/remove only after dependency analysis. Missing content is represented using normal missing-content migration rules.

## API sketch
```ts
interface IModDistribution {
  search(query: ModQuery): ModListing[];
  resolve(request: ModpackRequest): Resolution;
  install(pkg: ModPackage): InstallResult;
  update(id: ModID): UpdateResult;
  remove(id: ModID): Result;
}
```

## Security
Do not execute unverified native modules. Sandboxable content/script packages use Runtime permissions. Native mods require explicit trust.

## Invariants
- Distribution never grants runtime capabilities.
- Installed packages remain fingerprintable.
- Failed installs leave the previous valid state usable.
