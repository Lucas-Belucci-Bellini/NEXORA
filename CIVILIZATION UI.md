# NEXORA — CIVILIZATION UI

> Civilization UI visualizes macro social state and submits read/decision intents through the same public contracts used by simulations.

## Views
```text
Population
Government
Factions
Economy
Industry
Technology
Research
Diplomacy
Infrastructure
Territory
Projects
History
```

## Model
UI consumes `CivilizationSnapshot` and domain ViewModels. It must not read arbitrary internal simulation memory.

## Actions
Examples: propose project, inspect budget, review treaty, fund research, view trade routes. Actions become Commands and are validated server-side.

## LOD
A detailed local civilization UI can show settlements and projects; distant societies may expose aggregated statistics.

## History
Use World Event/Civilization History records to present timelines and causality graphs.

## Accessibility
Supports localization, keyboard/gamepad navigation, scalable layouts and semantic labels.

## API sketch
```ts
interface ICivilizationUI {
  open(civilizationId: CivilizationID): void;
  snapshot(id: CivilizationID): CivilizationViewModel;
  dispatch(action: CivilizationUIAction): void;
}
```

## Tests
Read-only guarantees, stale snapshot, failed command, permission restrictions and localization.

## Invariants
- UI is never authoritative for civilization state.
- Actions produce explicit commands.
