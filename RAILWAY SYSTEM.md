# NEXORA — RAILWAY SYSTEM

> Railway is a persistent transport network. Vehicles operate trains; Railway owns tracks, routing, signaling, stations, schedules and network topology.

## Architecture
```text
Track Network
→ Signals / Switches
→ Route Planner
→ Timetable
→ Train Service
→ Cargo / Passenger Flow
```

## Core objects
`RailNetwork`, `TrackSegment`, `Switch`, `Signal`, `Station`, `RailRoute`, `TrainService`, `Timetable`, `RailCargoOrder`.

## Track model
Segments are chunk-aware, can cross regions and support speed limits, electrification, grades, gauge/type and maintenance condition.

## Signaling
Provide block, route and interlocking abstractions. Exact signaling rules remain data-driven and mod-extensible.

## Routing
Railway pathfinding uses Navigation with train-specific traversal profiles. Reservations prevent conflicting trains from occupying incompatible segments.

## Stations
Stations can provide loading, unloading, passenger transfer, maintenance, refueling and scheduling services.

## Trains
Advanced Vehicles represents locomotive and car physical entities. Railway groups them into logical train formations and services.

## Scheduling
Support departures, priorities, waypoints, expected arrival, delays and emergency rerouting.

## Logistics
Integrate Industry/Economy for bulk cargo; passenger services can integrate Civilization and Quest systems.

## Failure
Broken tracks, blocked routes, signal failures and maintenance events reduce network capacity and may generate World Events.

## API sketch
```ts
interface IRailwaySystem {
  createNetwork(definition: RailNetworkDefinition): RailNetworkID;
  findRoute(request: RailRouteRequest): RailRoute;
  reserve(route: RailRoute, trainId: VehicleID): ReservationResult;
  schedule(service: TrainServiceDefinition): Result;
}
```

## Persistence
Persist infrastructure topology, schedules, reservations needed across restart and maintenance state. Derived path caches are rebuildable.

## Multiplayer
Server owns reservations, signaling state and authoritative train service state.

## Tests
Cross-chunk railway, switch routing, two trains conflict, broken segment reroute, cargo delivery, timetable delay and save/reload.

## Invariants
- Vehicles do not implement railway graph rules.
- Railway does not implement vehicle physics.
- A route reservation is atomic and cannot be duplicated.
