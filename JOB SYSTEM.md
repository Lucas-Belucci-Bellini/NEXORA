# NEXORA — JOB / TASK SYSTEM

> The Job System provides bounded parallel execution for engine, world, simulation, streaming, generation and IO work. It schedules work; domain systems own the rules.

## Core model
`System → Job/Task → Queue → Scheduler → Worker → Result/continuation`.

## Responsibilities
Worker pools, dependency DAGs, priorities, budgets, cancellation, shutdown, async IO, background compilation and deterministic simulation barriers.

## States
`CREATED → QUEUED → RUNNING → COMPLETED` with `CANCELLED`, `FAILED`, `TIMEOUT`, `ABORTED` terminal paths.

## Rules
No unbounded worker creation, domain rules outside scheduler, explicit backpressure, deterministic authoritative results and observable queue state.

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
WorldGen, Streaming, Persistence, Renderer and Simulation submit work. Mods/Scripts use scheduler services and quotas rather than unrestricted threads.

## Tests
Dependency ordering, cancellation, shutdown, starvation, worker-count determinism, queue backpressure and mod quota enforcement.
