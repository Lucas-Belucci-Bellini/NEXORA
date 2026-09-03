# NEXORA — LAUNCHER

> Manages installation profiles and startup selection; it does not contain simulation logic.

## Responsibilities
Profiles, installations, versions, modpacks, repair, verification, logs and launch modes.

## Flow
`Profile → Validate Installation → Resolve Content → Verify Artifacts → Launch`.

## Profiles
Pin game version, content fingerprint, mods, resource packs and configuration.

## Security
Authentication uses platform-approved mechanisms. Downloaded artifacts must be verified before use.

## API
```ts
interface ILauncher {
  profiles(): LauncherProfile[];
  validate(profile: LauncherProfile): ValidationReport;
  repair(profile: LauncherProfile): RepairResult;
  launch(profile: LauncherProfile): LaunchResult;
}
```

## Tests
Broken artifact, incompatible modpack, offline start, profile switching and repair.
