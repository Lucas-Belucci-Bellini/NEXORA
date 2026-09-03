# NEXORA — RAILWAY SYSTEM

> Railway owns tracks, stations, signaling, routing and schedules; Vehicles owns train physics.

## Core
`RailNetwork`, `TrackSegment`, `Switch`, `Signal`, `Station`, `RailRoute`, `TrainService`, `Timetable`, `RailCargoOrder`.

## Routing
Uses Navigation with train-specific traversal profiles. Route reservations prevent incompatible train occupancy.

## Stations
Loading/unloading, passenger transfer, maintenance and refueling services.

## Logistics
Industry/Economy can create cargo orders; Civilization can create passenger/infrastructure policies.

## Failure
Broken track, blocked route and signal/maintenance failures reduce capacity and can emit World Events.

## API
```ts
interface IRailwaySystem {
  createNetwork(definition: RailNetworkDefinition): RailNetworkID;
  findRoute(request: RailRouteRequest): RailRoute;
  reserve(route: RailRoute, trainId: VehicleID): ReservationResult;
  schedule(service: TrainServiceDefinition): Result;
}
```

## Tests
Cross-chunk track, switch routing, conflicts, rerouting, cargo, timetable delay and save/reload.
