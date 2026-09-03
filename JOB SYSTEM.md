# NEXORA — JOB / TASK SYSTEM

> Bounded parallel execution for engine, world, simulation, streaming, generation and IO work.

## Pipeline
`System → Job/Task → Queue → Scheduler → Worker → Result/Continuation`.

## Responsibilities
Worker pools, task DAGs, priorities, budgets, cancellation, shutdown, async IO and simulation barriers.

## Rules
No unbounded worker creation; domain rules remain in domain systems; queues use backpressure; authoritative results cannot depend on worker order.

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
WorldGen, Streaming, Persistence, Renderer and Simulation. Mods/Scripts use scheduler services and quotas.

## Tests
Dependency ordering, cancellation, shutdown, starvation, deterministic results and quotas.
