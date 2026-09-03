# NEXORA — CIVILIZATION UI

> UI visualizes macro social state and submits explicit intents; Civilization remains authoritative.

## Views
Population, Government, Factions, Economy, Industry, Technology, Research, Diplomacy, Infrastructure, Territory, Projects and History.

## Flow
`Snapshot → ViewModel → User Intent → Command → Civilization/Specialized System → Event → UI update`.

## Rules
No direct mutation of civilization state. Distant societies use aggregated snapshots. History is sourced from Civilization/World Event records.

## API
```ts
interface ICivilizationUI {
  open(id: CivilizationID): void;
  snapshot(id: CivilizationID): CivilizationViewModel;
  dispatch(action: CivilizationUIAction): void;
}
```

## Integration
UI, Localization, Accessibility, Command, Social, Economy, Industry, Research and World Events.

## Tests
Stale snapshot, failed command, permission restrictions and localization.
