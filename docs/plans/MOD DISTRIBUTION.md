# NEXORA — MOD DISTRIBUTION SYSTEM

> Discovers, verifies, resolves and delivers mod packages. Runtime remains responsible for loading/executing them.

## Flow
`Repository → Metadata → Dependency Resolution → Download → Verify → Install → Mod Runtime`.

## Metadata
ID, version, compatibility, dependencies, permissions, license/provenance, supported sides and content fingerprint.

## Rules
Resolve compatible versions explicitly; reject cycles/conflicts; verify package integrity; install in isolated mod directories; never overwrite official engine files.

## Security
Native mods require explicit trust. Unverified packages cannot execute.

## API
```ts
interface IModDistribution {
  search(query: ModQuery): ModListing[];
  resolve(request: ModpackRequest): Resolution;
  install(pkg: ModPackage): InstallResult;
  update(id: ModID): UpdateResult;
  remove(id: ModID): Result;
}
```

## Tests
Dependency resolution, verification failure, rollback, missing dependency and safe removal.
