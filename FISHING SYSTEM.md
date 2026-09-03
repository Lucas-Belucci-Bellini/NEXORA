# NEXORA — FISHING SYSTEM

> Fishing is the gameplay and resource-management layer over aquatic ecology; Water and Ecology remain responsible for the environment and populations.

## Model
```text
Aquatic Habitat → Species Population → Fish State → Fishing Activity → Catch → Processing / Trade
```

## Definitions
`FishSpecies`, `Habitat`, `FishingMethod`, `FishingTool`, `FishingSpot`, `CatchResult`, `FishPopulation`.

## Conditions
Water temperature, depth, season, current, weather, time, habitat, population pressure and equipment influence catch probability.

## Population
Use aggregate fish populations with local instances near players. Catching reduces local abundance and populations recover through ecology/reproduction rules.

## Methods
Rod, net, trap, boat-based and mod-defined methods. Abstractly model method capability rather than hardcode individual items.

## Sustainability
Overfishing can reduce future yield. Protected areas, quotas and aquaculture are possible civilization systems.

## API sketch
```ts
interface IFishingSystem {
  inspectSpot(position: WorldPosition): FishingSpot;
  fish(request: FishingRequest): FishingResult;
  simulatePopulation(habitat: HabitatID, time: WorldTime): PopulationState;
}
```

## Integration
Water supplies medium state; Ecology simulates populations; Tools provide capability; Inventory/Item stores catches; Cooking/Crafting can process products; Economy trades them; Civilization can regulate fishing.

## Tests
Seasonal migration, depleted population, successful catch, failed catch, boat fishing, save/load, multiplayer authority and LOD aggregation.

## Invariants
- Catching is server-authoritative in multiplayer.
- Fish population is not infinite by default.
- Fishing does not implement fluid physics.
