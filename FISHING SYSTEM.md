# NEXORA — FISHING SYSTEM

> Fishing is gameplay over aquatic ecology; Water and Ecology own the environment and populations.

## Model
`Habitat → Species Population → Fishing Activity → Catch → Processing/Trade`.

## Concepts
`FishSpecies`, `Habitat`, `FishingMethod`, `FishingSpot`, `CatchResult`, `FishPopulation`.

## Conditions
Temperature, depth, season, current, weather, time, habitat, population pressure and equipment affect catch.

## Sustainability
Overfishing reduces local abundance; ecology restores populations. Civilization may regulate quotas, protected areas and aquaculture.

## API
```ts
interface IFishingSystem {
  inspectSpot(position: WorldPosition): FishingSpot;
  fish(request: FishingRequest): FishingResult;
}
```

## Tests
Migration, depletion/recovery, successful/failed catch, boat fishing, persistence and multiplayer authority.
