# NEXORA — PERFORMANCE BUDGETS

## Purpose
Performance is a design constraint, not a final optimization phase.

## Budget dimensions
```text
FRAME_TIME
SIMULATION_TIME
JOB_TIME
MEMORY
GPU_MEMORY
STREAMING_LATENCY
NETWORK_BANDWIDTH
SAVE_TIME
LOAD_TIME
ENTITY_COUNT
CHUNK_COUNT
RESOURCE_COUNT
```

## Budget classes
Every major system should publish:
```text
TARGET
WARNING
CRITICAL
EMERGENCY
```

## Runtime separation
```text
RENDER BUDGET
SIMULATION BUDGET
STREAMING BUDGET
IO BUDGET
NETWORK BUDGET
BACKGROUND WORK BUDGET
```

A subsystem must not consume another subsystem's budget invisibly.

## Simulation scaling
Use LOD and frequency adaptation before increasing brute-force processing:
```text
FULL → high frequency / local detail
REGIONAL → reduced detail
ABSTRACT → statistical/event-driven model
UNRESIDENT → persistent summary/state only
```

## Performance invariants
- no unbounded per-frame allocations in hot paths;
- no hidden O(n²) scans in recurring world loops without an explicit justification;
- no global iteration over all entities for local queries;
- expensive work should be scheduled through the Job System where safe;
- GPU and CPU work must be profiled separately;
- budgets are measurable in tests and benchmarks.

## Acceptance
A system is not production-ready until its expected scale and measurable budget are documented.
