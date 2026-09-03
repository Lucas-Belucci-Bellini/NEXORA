# NEXORA — JOB / TASK SYSTEM

> The Job System provides bounded parallel execution for engine, world, simulation, streaming, generation and IO work. It schedules work; domain systems own the rules.

## Responsibilities
- worker pools and task queues;
- dependency-aware jobs;
- priorities and budgets;
- cancellation and shutdown;
- CPU affinity policy;
- async IO and background compilation;
- deterministic simulation barriers;
- diagnostics and profiling.

## Non-responsibilities
The Job System must not implement voxel generation, physics, rendering, AI or gameplay rules.

## Core model
```text
System
  ↓
Job / Task
  ↓
Queue
  ↓
Scheduler
  ↓
Worker
  ↓
Result / continuation
```

## Job states
`CREATED → QUEUED → RUNNING → COMPLETED` with `CANCELLED`, `FAILED`, `TIMEOUT` and `ABORTED` terminal paths.

## Scheduling
Support `CRITICAL`, `HIGH`, `NORMAL`, `LOW`, `BACKGROUND` priorities plus per-system budgets. Avoid unbounded queues; use backpressure.

## Dependencies
Tasks can depend on other tasks through a DAG. Cycles are rejected before execution.

## Determinism
Authoritative simulation jobs execute through explicit phase barriers. Randomness is supplied by deterministic system-owned RNG contexts; worker order must not become gameplay state.

## Safety
No arbitrary user threads from scripts or untrusted mods. Native extensions receive explicit trust and resource policies.

## Performance
Target work stealing for independent CPU jobs, batching for voxel/world work, dedicated IO workers and GPU submission boundaries owned by the renderer.

## API sketch
```ts
interface IJobSystem {
  submit(job: JobDefinition): JobHandle;
  cancel(handle: JobHandle): Result;
  wait(handle: JobHandle): JobResult;
  barrier(scope: JobScope): void;
  inspect(): JobMetrics;
}
```

## Integration
- WorldGen submits generation jobs.
- Streaming submits load/decompress jobs.
- Persistence submits IO jobs.
- Renderer submits preparation/compile work.
- Simulation submits bounded parallel jobs.
- Mod Runtime uses scheduler services rather than creating threads.

## Debug
`nexora jobs list`, `nexora jobs profile`, `nexora jobs queues`, `nexora jobs leaks`.

## Golden tests
1. Dependency ordering.
2. Cancellation.
3. Shutdown with pending work.
4. No starvation for high-priority jobs.
5. Deterministic authoritative result independent of worker count.

## Stress
1k, 10k and 100k queued jobs; sustained world-generation load; IO pressure; mod task quotas.

## Invariants
- No unbounded worker creation.
- No domain rules inside the scheduler.
- No silent task loss.
- Shutdown reaches a known state.
