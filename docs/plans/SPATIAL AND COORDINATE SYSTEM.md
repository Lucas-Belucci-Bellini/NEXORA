# NEXORA — SPATIAL / COORDINATE SYSTEM

> NEXORA's world scale requires explicit coordinate spaces, precision rules and spatial transforms before implementation language is selected.

## Spaces
```text
Universe
→ Galaxy
→ Star System
→ Planet / Dimension
→ Region
→ Chunk
→ Local Space
→ Render Space
```

## Responsibilities
- canonical integer block coordinates;
- high precision world transforms;
- local-frame transforms for physics/rendering;
- chunk/region addressing;
- spatial bounds and ownership;
- coordinate conversion;
- floating-origin or equivalent rebasing;
- stable serialization of positions.

## Rule
Gameplay state never depends on camera-relative coordinates. World identity remains stable while render/physics local frames may shift.

## Data model
Use integer/block coordinates for voxel identity and an explicit high-precision representation for large-scale positions. Do not assume one numeric type is sufficient for every subsystem.

## API
```ts
interface ISpatialSystem {
  toChunk(position: BlockPosition): ChunkCoord;
  toRegion(chunk: ChunkCoord): RegionCoord;
  worldToLocal(position: WorldPosition, frame: LocalFrame): LocalPosition;
  localToWorld(position: LocalPosition, frame: LocalFrame): WorldPosition;
  rebase(frame: LocalFrame, origin: WorldPosition): void;
}
```

## Integration
Physics uses local frames; Renderer uses render-space transforms; WorldGen/Storage use canonical coordinates; Networking serializes authoritative world positions or deterministic references.

## Tests
Extreme coordinates, negative coordinates, chunk boundaries, origin rebasing, save/reload and cross-dimension coordinates.

## Invariants
- Block identity is exact and stable.
- Rebasing never changes logical world coordinates.
- Coordinate conversion is deterministic and reversible within defined precision guarantees.
