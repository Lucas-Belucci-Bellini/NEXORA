# NEXORA — AGRICULTURE / FARMING SYSTEM

> Agriculture is a living production system connecting soil, seeds, crops, water, climate, labor, technology and economy.

## Scope
`Soil → Seed → Growth → Harvest → Storage → Processing → Economy`.

## Core
`CropDefinition`, `CropInstance`, `SoilPatch`, `Farm`, `Field`, `SeedLot`, `LivestockUnit`, `AgriculturalTask`.

## Rules
Climate/Water provide conditions; Vegetation owns ecological plant behavior; Agriculture owns food-production gameplay; Industry processes outputs; Economy values goods; Civilization plans food security.

## Lifecycle
`SEEDED → GERMINATING → GROWING → MATURE → HARVESTABLE → HARVESTED`, with dormancy, disease and failure states.

## LOD
Near fields may use individual crops; distant agriculture uses field/region aggregates.

## API
```ts
interface IAgricultureSystem {
  createFarm(definition: FarmDefinition): FarmID;
  plant(request: PlantingRequest): Result;
  simulateField(field: FieldID, time: WorldTime): FieldState;
  harvest(request: HarvestRequest): Result;
}
```

## Tests
Growth/harvest, irrigation failure, seasonal changes, disease, soil recovery, LOD and persistence.
