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

## Core model
`System → Job/Task → Queue → Scheduler → Worker → Result/continuation`.

## Job states
`CREATED → QUEUED → RUNNING → COMPLETED` with `CANCELLED`, `FAILED`, `TIMEOUT` and `ABORTED` terminal paths.

## Rules
No unbounded worker creation, no domain rules in scheduler, explicit backpressure, explicit shutdown state, and deterministic authoritative results independent of worker order.

## API
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
WorldGen, Streaming, Persistence, Renderer and Simulation submit work through this system. Mods and Scripts use scheduler services and quotas instead of arbitrary threads.

## Tests
Dependency ordering, cancellation, shutdown with pending jobs, no starvation, worker-count determinism and queue backpressure.

## Stress
1k, 10k and 100k queued jobs; sustained generation/IO pressure; mod quota enforcement.

## Security
Untrusted mods/scripts cannot create unrestricted threads or bypass scheduler budgets.
