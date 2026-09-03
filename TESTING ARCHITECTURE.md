# NEXORA — TESTING ARCHITECTURE

> Tests are part of the engine contract. Every system must expose deterministic seams, golden scenarios, fault handling and measurable performance.

## Layers
```text
Static / Schema
→ Unit
→ Contract
→ Integration
→ Golden / Determinism
→ Simulation
→ Stress
→ Fault Injection
→ Multiplayer
→ Release Validation
```

## Test categories
Unit tests verify local logic. Contract tests verify APIs between systems. Integration tests verify vertical slices. Golden tests lock expected authoritative outcomes. Stress tests validate scale. Fault tests validate recovery and isolation.

## Determinism
Provide fixed seeds, simulation versions and controlled clocks. The same authoritative input/state must produce equivalent outcomes under supported worker configurations.

## Golden scenarios
Examples: world load, block transaction, item transfer, entity spawn, crafting, machine processing, vehicle movement, event chain, civilization decision, save/reload and mod lifecycle.

## Property tests
Use generated values for transactions, chunk coordinates, inventory operations, event graphs, serialization and registry IDs to catch edge cases.

## Multiplayer tests
Latency, jitter, loss, reconnect, prediction/reconciliation, invalid commands, registry mismatch and dimension transfer.

## Fault injection
Corrupt saves, missing resources, failed jobs, full queues, disconnected networks, failed machines, missing mods and interrupted writes.

## Performance tests
Every major system gets representative benchmark fixtures with CPU, memory, IO and network metrics.

## Test data
Synthetic worlds and fixtures should avoid external copyrighted game data. Seeds and expected outputs are NEXORA-authored.

## CI gates
```text
format/lint
→ unit
→ contract
→ integration
→ deterministic golden
→ security
→ benchmark regression
→ package verification
```

## API sketch
```ts
interface ITestHarness {
  createWorld(seed: number, version: string): TestWorld;
  runScenario(scenario: TestScenario): TestResult;
  injectFault(fault: Fault): void;
  assertDeterministic(scenario: TestScenario): void;
}
```

## Invariants
- A flaky test cannot be silently accepted.
- Golden expected state is versioned.
- Fault tests do not mutate production saves.
- Performance regressions are visible before release.
