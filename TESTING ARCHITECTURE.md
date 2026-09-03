# NEXORA — TESTING ARCHITECTURE

> Tests are part of the engine contract and must cover correctness, integration, determinism, failure and scale.

## Layers
`Static/Schema → Unit → Contract → Integration → Golden/Determinism → Simulation → Stress → Fault Injection → Multiplayer → Release Validation`.

## Golden slices
World load, block transaction, item transfer, entity spawn, crafting, machine processing, vehicle movement, world-event chain, civilization decision and save/reload.

## Determinism
Fixed seeds, simulation versions and clocks. Authoritative results cannot depend on worker order.

## Faults
Corrupt saves, missing resources/mods, failed jobs, full queues, network loss, interrupted writes and invalid commands.

## Performance
Representative benchmark fixtures are release gates.

## API
```ts
interface ITestHarness {
  createWorld(seed: number, version: string): TestWorld;
  runScenario(scenario: TestScenario): TestResult;
  injectFault(fault: Fault): void;
  assertDeterministic(scenario: TestScenario): void;
}
```

## CI
`format/lint → unit → contract → integration → golden → security → benchmark → package verification`.

## Invariants
Flaky tests are not silently accepted; expected golden state is versioned; fault tests never touch production saves.
