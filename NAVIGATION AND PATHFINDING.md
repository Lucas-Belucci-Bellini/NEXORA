# NEXORA — NAVIGATION / PATHFINDING

> Navigation describes traversability and routes; Pathfinding computes paths; AI chooses intent; movement/Physics execute it.

## Domains
Terrain/voxel, structures, roads, rails, water, air, space and dimensions.

## Core
`NavigationWorld`, `NavigationRegion`, `NavigationNode`, `NavigationEdge`, `TraversalProfile`, `PathRequest`, `PathResult`.

## Hierarchy
`Local → Region → World route graph` to keep searches bounded.

## Dynamic updates
Build/destruction, vehicles, flooding and other changes invalidate only affected regions where possible.

## Costs
Distance, slope, danger, congestion, terrain, ownership, tolls, energy/fuel and permissions.

## API
```ts
interface INavigationSystem {
  requestPath(request: PathRequest): PathHandle;
  findRoute(request: RouteRequest): Route;
  isTraversable(profile: TraversalProfile, point: WorldPosition): boolean;
  invalidate(region: NavigationRegionID): void;
}
```

## Integration
Job/Task, Streaming, AI, Vehicles, Railway, Structure, World, Server and Security. Clients may predict but server validates movement.

## Tests
Dynamic obstacles, cross-chunk paths, railway routes, water/vehicle profiles and stale-route recovery.
