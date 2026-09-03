# NEXORA — DIAGNOSTICS / OBSERVABILITY

> Makes engine behavior measurable, traceable and debuggable without becoming gameplay logic.

## Signals
Logs, metrics, traces, profiles, crash reports, health checks, save/network/mod diagnostics and event traces.

## Correlation
Commands, events, transactions, saves and network requests use correlation/causation IDs where relevant.

## Health
Expose READY, DEGRADED and failure conditions for storage, network, simulation, resources and server runtime.

## Privacy/security
Never log secrets or authentication tokens. Optional remote telemetry must be explicit and privacy-minimized.

## API
```ts
interface IDiagnostics {
  log(entry: LogEntry): void;
  metric(metric: MetricSample): void;
  trace(scope: TraceSpan): void;
  health(): HealthReport;
  dump(scope: DiagnosticScope): DiagnosticBundle;
}
```

## Tests
Redaction, rate limiting, correlation propagation, crash context and diagnostics under resource pressure.
