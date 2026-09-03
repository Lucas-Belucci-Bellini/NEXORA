# NEXORA — HUNTING / WILDLIFE INTERACTION

> Hunting is a gameplay/resource layer over the existing Mob Ecology system. It must respect species populations, habitats and regeneration.

## Model
```text
Wildlife Population → Tracking → Encounter → Harvest → Resource Processing
```

## Concepts
`WildlifeSpecies`, `HabitatProfile`, `Track`, `ScentTrail`, `HuntingMethod`, `HarvestResult`, `WildlifeManagementPolicy`.

## Tracking
Animals can leave abstract or detailed signs such as footprints, sounds, disturbed vegetation and time-based traces. Detection depends on perception, tools, weather and knowledge.

## Population
Hunting pressure changes local population. Ecology governs reproduction, migration, competition and recovery.

## Methods
Stalking, tracking, trapping and other safe gameplay abstractions. Specialized tools provide capabilities through Tool/Weapon API.

## Resource use
Harvested resources can enter crafting, food, clothing, medicine or trade without hardcoding downstream recipes here.

## Conservation
Civilizations may create quotas, protected areas, hunting licenses and wildlife policies.

## API sketch
```ts
interface IHuntingSystem {
  queryTracks(request: TrackQuery): Track[];
  start(request: HuntingRequest): HuntSession;
  harvest(request: HarvestRequest): Result;
}
```

## Integration
Ecology/Mobs own wildlife simulation; Interaction selects targets; Tools execute capabilities; Inventory/Item handles results; Economy values products; Civilization regulates pressure.

## Tests
Tracking under weather changes, population depletion/recovery, multiplayer validation, harvest transaction and regional abstraction.

## Invariants
- Wildlife is not an infinite loot source.
- Harvesting is transactional.
- Ecology remains authoritative for population state.
