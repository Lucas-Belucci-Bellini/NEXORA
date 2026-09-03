# NEXORA — NAVIGATION / PATHFINDING SYSTEM

> Navigation answers where an actor can and should move. Pathfinding computes routes; movement and Physics execute them.

## Separation
```text
Navigation → traversability / routes
Pathfinding → path solution
AI → intention
Control → movement intent
Physics → physical result
```

## Navigation domains
- voxel/terrain;
- structures and cities;
- roads and railways;
- water;
- air;
- space;
- dimensions.

## Core concepts
`NavigationWorld`, `NavigationRegion`, `NavigationNode`, `NavigationEdge`, `TraversalProfile`, `Route`, `PathRequest`, `PathResult`.

## Traversal profiles
Human, animal, vehicle, train, boat, aircraft, spacecraft and custom mod profiles. Each profile defines clearance, movement modes, slope limits, medium requirements and constraints.

## Hierarchical navigation
```text
Local voxels
→ local graph
→ region graph
→ world route graph
```
Use hierarchy rather than searching an entire planet for every request.

## Dynamic obstacles
Building, destruction, vehicles, floods and other world changes invalidate only affected navigation regions where possible.

## Pathfinding
Support A* or equivalent bounded graph search, hierarchical search and cached routes. The implementation is replaceable behind an API.

## Route planning
Separate a path from a strategic route. A civilization may choose a railway corridor while an NPC chooses a local walking path.

## Cost model
Costs may include distance, slope, danger, congestion, terrain, ownership, tolls, fuel/energy and permissions. AI decides which route is desirable.

## Navigation state
Routes are versioned against navigation data. Stale paths are detected and recomputed.

## API sketch
```ts
interface INavigationSystem {
  requestPath(request: PathRequest): PathHandle;
  findRoute(request: RouteRequest): Route;
  isTraversable(profile: TraversalProfile, point: WorldPosition): boolean;
  invalidate(region: NavigationRegionID): void;
}
```

## Streaming
Navigation data is streamed per region and can have abstract route graphs for distant areas.

## Networking
The server validates movement intent and uses authoritative navigation/physics. Clients may predict but cannot authoritatively teleport.

## Debug
`nexora nav inspect`, `path`, `route`, `graph`, `invalidate`, `profile`.

## Tests
Door/build obstruction, cross-chunk movement, dynamic collapse, road routing, train routing, water navigation, vehicle route and planetary/space route abstraction.

## Invariants
- Pathfinding never changes world state.
- AI does not own graph topology.
- Movement controllers remain responsible for execution.
- Navigation remains replaceable and mod-extensible.
