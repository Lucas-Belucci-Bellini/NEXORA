# NEXORA — CONFIGURATION / SETTINGS SYSTEM

> Configuration is the canonical way to define startup, runtime and user/server settings without scattering constants through Core or gameplay systems.

## Layers
```text
Defaults
→ Project Configuration
→ World Configuration
→ Server Configuration
→ User Profile
→ Runtime Overrides
```

## Types
- engine settings;
- rendering/audio/input settings;
- world generation settings;
- simulation budgets;
- server policies;
- mod configuration schemas;
- accessibility preferences.

## Rules
Configuration is data. Systems consume typed snapshots/interfaces. Runtime mutation is explicit and validated.

## API
```ts
interface ISettingsSystem {
  register(schema: SettingsSchema): void;
  get<T>(key: SettingKey<T>): T;
  set<T>(key: SettingKey<T>, value: T, scope: SettingScope): Result;
  snapshot(scope: SettingsScope): SettingsSnapshot;
  validate(snapshot: SettingsSnapshot): ValidationReport;
}
```

## Persistence
User and server settings are versioned separately from world save state. Migrations are handled by Versioning/Migration.

## Security
Mods receive only declared configuration namespaces and cannot silently change protected server/security settings.

## Tests
Default merge order, invalid values, profile migration, hot reload safety and mod namespace isolation.

## Invariants
- No hidden global mutable configuration.
- Protected settings require explicit authority.
- Configuration changes cannot silently corrupt persistent world state.
