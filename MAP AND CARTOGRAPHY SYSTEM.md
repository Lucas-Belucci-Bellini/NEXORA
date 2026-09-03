# NEXORA — MAP / CARTOGRAPHY SYSTEM

> Cartography turns discovered spatial information into persistent maps; Navigation calculates movement routes.

## Layers
Terrain, biomes, water, structures, roads/rails, resources, settlements, borders, climate, exploration, space and mod layers.

## States
`UNKNOWN → OBSERVED → SKETCHED → MAPPED → VERIFIED`; survey data can become stale.

## Sources
Player exploration, NPC surveys, vehicles, sensors, satellites, research and civilization institutions.

## Accuracy
Store source, timestamp, resolution, uncertainty and confidence. Different actors can hold different maps of the same area.

## API
```ts
interface ICartographySystem {
  createMap(definition: MapDefinition): MapID;
  addLayer(mapId: MapID, layer: MapLayerDefinition): void;
  recordSurvey(survey: Survey): Result;
  query(mapId: MapID, area: MapArea): MapSnapshot;
}
```

## Integration
Navigation, Sensors, Space, Civilization, Communication, World Events and UI. Rendered map tiles are derived/cache data.

## Tests
Partial exploration, resurvey, border changes, space mapping and stale-map detection.
