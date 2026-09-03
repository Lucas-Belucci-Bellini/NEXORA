# NEXORA — SENSOR SYSTEM

> Sensors turn world signals into measurements and observations; they do not become an omniscient world-state API.

## Types
Visual, acoustic, thermal, chemical, pressure, magnetic, electrical, biological, resource, astronomical and mod-defined sensors.

## Pipeline
`World State → Sensor → Measurement → Observation → Knowledge/Research`.

## Factors
Range, line of sight, medium, resolution, noise, weather, signal strength and technology.

## API
```ts
interface ISensorSystem {
  observe(request: SensorRequest): Observation[];
  measure(sensorId: SensorID, target: SensorTarget): Measurement;
  inspect(sensorId: SensorID): SensorState;
}
```

## Integration
AI perception, Research/Knowledge, Communication, Space, Environment and Vehicles.

## Security
Scripts receive only authorized sensor capabilities. Untrusted code cannot read arbitrary world memory.

## Tests
Occlusion, range, noise, weather, underwater sensing, astronomy and permissions.
