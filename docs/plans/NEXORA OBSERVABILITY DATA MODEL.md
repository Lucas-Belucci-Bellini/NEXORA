# NEXORA — OBSERVABILITY DATA MODEL

## Purpose
Make simulation and engine behavior inspectable without depending on ad-hoc debug prints.

## Telemetry classes
```text
LOG
METRIC
TRACE
EVENT TRACE
PROFILE SAMPLE
SIMULATION SNAPSHOT
AUDIT RECORD
```

## Common fields
```text
timestamp
world_id
system_id
entity_id/resource_id when applicable
thread/job
build/version
severity
correlation_id
```

## Simulation trace
A trace may capture:
```text
input
intent
command
state transition
event
consequence
history record
knowledge propagation
```

## Performance telemetry
Systems should expose measurable counters such as:
```text
active entities
jobs queued/completed
chunk loads
streaming latency
memory usage
cache hit/miss
network bytes
save duration
simulation step duration
```

## Privacy
Telemetry must avoid collecting unnecessary user data. Local development diagnostics and production telemetry have separate retention/configuration policies.

## Debug reproducibility
Critical bug reports should preserve enough metadata to reproduce the issue:
```text
build
world seed
world version
mods
configuration
simulation mode
replay/snapshot reference
```
