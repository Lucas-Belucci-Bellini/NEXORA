# NEXORA — AGRICULTURE / FARMING SYSTEM

> Agriculture is a living production system connecting soil, plants, water, climate, labor, technology and economy.

## Scope
```text
Soil → Seed → Plant → Growth → Harvest → Storage → Processing → Economy
```

## Core entities
`CropDefinition`, `CropInstance`, `SoilPatch`, `Farm`, `Field`, `SeedLot`, `LivestockUnit`, `AgriculturalTask`.

## Soil
Track fertility, moisture, composition, temperature, erosion, contamination and recovery. Soil is regional state, not only a block property.

## Crop lifecycle
`SEEDED → GERMINATING → GROWING → MATURE → HARVESTABLE → HARVESTED`; support dormancy, disease and death.

## Inputs
Climate, water, light, soil, nutrients, genetics, season and farming methods.

## Farming methods
Manual, irrigated, mechanized, greenhouse, hydroponic, industrial and mod-defined methods.

## Livestock
Represent species, population, health, reproduction, feed, housing and production outputs without replacing Animal/Ecology simulation.

## Agriculture production
Use batches and aggregates at regional LOD. Individual plants are materialized only where gameplay or simulation requires them.

## Events
Droughts, crop failures, pests, disease and harvest booms can become World Events or economic/ecological signals.

## API sketch
```ts
interface IAgricultureSystem {
  createFarm(definition: FarmDefinition): FarmID;
  plant(request: PlantingRequest): Result;
  simulateField(field: FieldID, time: WorldTime): FieldState;
  harvest(request: HarvestRequest): Result;
}
```

## Integration
Climate/Water provide conditions; Vegetation owns ecological plant behavior; Inventory/Item holds products; Industry processes outputs; Economy values goods; Civilization plans food security.

## Tests
Seed→growth→harvest, irrigation failure, seasonal change, disease, soil depletion/recovery, regional LOD and persistence.

## Invariants
- Agriculture does not replace Vegetation ecology.
- Harvest is transactional.
- Food production remains deterministic under fixed world state.
