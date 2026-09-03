# NEXORA — SENSOR SYSTEM

> Sensors turn physical or simulated signals into observations. They do not create knowledge by themselves.

## Pipeline
```text
World State
→ Sensor
→ Measurement
→ Observation
→ Knowledge / Research
```

## Sensor types
Visual, acoustic, thermal, chemical, pressure, magnetic, electrical, biological, resource scanner, astronomical and mod-defined sensing.

## Core objects
`SensorDefinition`, `SensorInstance`, `SensorProfile`, `Measurement`, `Observation`, `SensorRange`, `DetectionModel`.

## Detection factors
Range, line of sight, medium, resolution, noise, weather, camouflage, signal strength, sensor technology and operator skill.

## Sensor uncertainty
Measurements may contain confidence, noise and resolution. The sensor must not silently reveal perfect world state unless its capability explicitly allows it.

## Research integration
Measurements can become Evidence/Observation records in Research and Knowledge systems.

## AI integration
Perception systems consume observations rather than reading arbitrary world memory.

## Space integration
Sensors cover radar, optical astronomy, spectrometry, navigation and deep-space detection through the Space environment.

## API sketch
```ts
interface ISensorSystem {
  observe(request: SensorRequest): Observation[];
  measure(sensorId: SensorID, target: SensorTarget): Measurement;
  inspect(sensorId: SensorID): SensorState;
}
```

## Performance
Use broad-phase spatial queries, batched observations and LOD. Distant sensors may produce statistical observations.

## Security
Untrusted scripts can only use permitted sensor capabilities; no sensor may become a hidden omniscient world-state API.

## Tests
Occlusion, range, noise, weather, underwater sensing, astronomy, dynamic targets and permission enforcement.

## Invariants
- Sensors observe; they do not decide truth.
- Measurement uncertainty is explicit.
- Observation data follows ownership/privacy rules where applicable.
