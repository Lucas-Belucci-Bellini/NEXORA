# NEXORA — PERFORMANCE ENGINE

> Performance is a cross-cutting constraint. This system measures budgets, detects regressions and coordinates degradation without owning gameplay rules.

## Budgets
Frame, simulation, WorldGen, rendering, memory, GPU, IO, network, events, commands, mods/scripts.

## Metrics
Frame/tick time, queue depth, job latency, memory, allocations, GPU time, draw work, chunk generation, streaming churn and network bandwidth.

## Adaptive quality
Optional work degrades first; authoritative validation and persistence integrity are never silently weakened.

## LOD
Performance can request LOD changes; each simulation system owns the transition.

## API
```ts
interface IPerformanceEngine {
  beginScope(name: string): ProfileScope;
  counter(name: string, value: number): void;
  budget(scope: BudgetScope): BudgetState;
  snapshot(): PerformanceSnapshot;
}
```

## Tests
Chunk streaming, entity LOD, factory/civilization simulation, saves and network pressure. Benchmark regressions are release gates.

## Invariants
Budgets are observable and optimizations cannot change authoritative outcomes.
