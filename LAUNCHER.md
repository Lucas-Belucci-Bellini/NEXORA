# NEXORA — LAUNCHER

> The launcher manages installation profiles and startup selection; it does not contain game simulation logic.

## Responsibilities
```text
Account / Profile
Installations
Version Selection
Modpack Selection
Update Check
Repair
Logs
Launch Arguments
```

## Flow
`Select Profile → Validate Installation → Resolve Version/Mods → Verify Artifacts → Launch Client/Server`.

## Profiles
A profile can pin game version, content fingerprint, mods, resource packs, configuration and launch mode.

## Verification
Before launch, validate manifests, checksums, signatures where available, required dependencies and compatible content versions.

## Repair
Detect missing/corrupt official artifacts and restore them without touching unrelated saves.

## Privacy
Authentication credentials/tokens are handled by platform-approved mechanisms and never stored in game saves or plain logs.

## API sketch
```ts
interface ILauncher {
  profiles(): LauncherProfile[];
  validate(profile: LauncherProfile): ValidationReport;
  repair(profile: LauncherProfile): RepairResult;
  launch(profile: LauncherProfile): LaunchResult;
}
```

## Tests
Broken artifact repair, incompatible modpack, offline startup, profile switching and safe launch failure.

## Invariants
- Launcher cannot alter authoritative world state.
- A failed update does not destroy a working installation.
