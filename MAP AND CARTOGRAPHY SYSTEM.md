# NEXORA — MAP / CARTOGRAPHY SYSTEM

> Cartography turns discovered spatial information into persistent maps and layers. It is distinct from Navigation, which calculates movement routes.

## Layers
```text
Terrain
Biomes
Water
Structures
Roads / Rails
Resources
Settlements
Political Borders
Climate
Exploration
Space
Custom Mod Layers
```

## Core concepts
`Map`, `MapLayer`, `MapTile`, `CartographicRecord`, `Survey`, `Marker`, `DiscoveryState`, `MapProjection`.

## Discovery states
`UNKNOWN → OBSERVED → SKETCHED → MAPPED → VERIFIED`; data can become stale and require resurvey.

## Sources
Player exploration, NPC surveyors, vehicles, sensors, satellites, research and civilization cartographic institutions.

## Accuracy
Maps can preserve source, timestamp, resolution, uncertainty and confidence. Different civilizations may possess different maps of the same territory.

## Political maps
Political layers derive from Territory/Social/Civilization state and must not become authoritative world ownership by themselves.

## Navigation integration
Navigation can query suitable map layers for route planning; maps do not execute movement.

## Space
Support star charts, planet maps, orbital maps and discovered object catalogs using Space coordinate systems.

## Persistence
Persist authored and discovered map records, markers and surveys. Heavy rendered tiles are cache data and can be rebuilt.

## API sketch
```ts
interface ICartographySystem {
  createMap(definition: MapDefinition): MapID;
  addLayer(mapId: MapID, layer: MapLayerDefinition): void;
  recordSurvey(survey: Survey): Result;
  query(mapId: MapID, area: MapArea): MapSnapshot;
}
```

## Security
Respect map ownership and faction permissions. Server authority determines authoritative geographic state in multiplayer.

## Debug
`nexora map inspect`, `layers`, `survey`, `coverage`, `diff`.

## Tests
Partial exploration, outdated map, resurvey, political border change, space mapping and LOD rendering.

## Invariants
- Map data is a representation, not world truth.
- Navigation and Cartography remain separate.
- Map uncertainty is explicit.
