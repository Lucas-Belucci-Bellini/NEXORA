# NEXORA — PERFORMANCE ENGINE

> Performance is an architectural constraint. The Performance Engine measures budgets, detects regressions and coordinates degradation; it does not own gameplay logic.

## Budgets
```text
Frame
Simulation
WorldGen
Rendering
Memory
GPU
IO
Network
Events
Commands
Mods / Scripts
```

Each budget has target, soft limit, hard limit, priority and degradation policy.

## Metrics
Track frame time, tick time, job latency, queue depth, memory resident set, allocations, GPU time, draw submissions, chunk generation latency, streaming churn and network bandwidth.

## Adaptive quality
When a budget is exceeded, reduce optional work first:
```text
cosmetic → distant updates → simulation frequency → streaming radius → expensive effects
```
Never silently relax authoritative validation or persistence integrity.

## Profiling
Provide scoped timers, counters, traces and sampling. Every major subsystem can publish metrics without depending on a specific profiler vendor.

## Regression gates
CI/benchmarks should compare representative workloads and flag statistically meaningful regressions rather than relying only on a single frame number.

## LOD integration
Performance budgets can request lower Simulation LOD, but the owning simulation system performs the actual transition.

## API sketch
```ts
interface IPerformanceEngine {
  beginScope(name: string): ProfileScope;
  counter(name: string, value: number): void;
  budget(scope: BudgetScope): BudgetState;
  snapshot(): PerformanceSnapshot;
  report(): PerformanceReport;
}
```

## Resource exhaustion
Define emergency policies for CPU saturation, memory pressure, IO backlog and network congestion. Backpressure is preferable to unlimited queues.

## Server
Server ticks require explicit simulation budgets. A missed budget becomes `DEGRADED`, not silent time distortion.

## Client
Maintain frame pacing and responsiveness separately from simulation correctness.

## Debug
`nexora perf frame`, `tick`, `jobs`, `memory`, `streaming`, `render`, `network`, `profile`.

## Golden benchmarks
- empty world;
- chunk streaming;
- 1k/10k/100k entities with LOD;
- large factory;
- large civilization simulation;
- multiplayer traffic;
- save/load under pressure.

## Invariants
- Performance optimization cannot change authoritative outcomes.
- Budgets are observable.
- Derived caches can be dropped before persistent state.
- Every critical subsystem has at least one benchmark workload.
