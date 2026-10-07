# NEXORA — HUNTING / WILDLIFE

> Hunting is the resource/gameplay layer over Mob Ecology, which remains authoritative for animal populations and habitats.

## Model
`Wildlife Population → Tracking → Encounter → Harvest → Resource Processing`.

## Concepts
`WildlifeSpecies`, `HabitatProfile`, `Track`, `HuntingMethod`, `HarvestResult`, `WildlifeManagementPolicy`.

## Rules
Tracking uses perception, terrain, weather and knowledge. Harvest pressure changes local population; ecology controls reproduction and migration.

## Integration
Interaction selects targets; Tools provide capabilities; Item/Inventory stores products; Economy values them; Civilization can regulate them.

## API
```ts
interface IHuntingSystem {
  queryTracks(request: TrackQuery): Track[];
  start(request: HuntingRequest): HuntSession;
  harvest(request: HarvestRequest): Result;
}
```

## Tests
Tracking, depletion/recovery, harvest transactions, multiplayer validation and regional abstraction.
