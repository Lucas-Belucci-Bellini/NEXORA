# NEXORA — DIAGNOSTICS / OBSERVABILITY

> Diagnostics makes engine behavior measurable and debuggable without becoming part of gameplay logic.

## Signals
```text
Logs
Metrics
Traces
Profiles
Crash Reports
Health Checks
Event Traces
Save Diagnostics
Network Diagnostics
Mod Diagnostics
```

## Severity
`TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL` with subsystem/category/source metadata.

## Correlation
Commands, events, transactions, saves and network requests carry correlation/causation IDs where relevant.

## Metrics
Expose counters, gauges, histograms and sampled traces for tick time, memory, streaming, jobs, chunks, network, saves, mods and simulation LOD.

## Crash isolation
Capture safe diagnostic context without requiring gameplay state to remain valid. A crash report may reference world/mod fingerprints but must avoid secrets.

## Health
Server and local runtime expose readiness, degraded, storage, network, simulation and resource-health states.

## Telemetry policy
Diagnostics are useful locally and on servers. Any optional remote telemetry must be explicit, documented and privacy-minimized.

## API sketch
```ts
interface IDiagnostics {
  log(entry: LogEntry): void;
  metric(metric: MetricSample): void;
  trace(scope: TraceSpan): void;
  health(): HealthReport;
  dump(scope: DiagnosticScope): DiagnosticBundle;
}
```

## Security
Never log passwords, private keys, session tokens or raw sensitive payloads. Admin diagnostic endpoints require authorization.

## Tests
Log rate limiting, crash context, correlation propagation, privacy redaction and diagnostics under resource pressure.

## Invariants
- Diagnostics cannot alter authoritative state.
- Logging must not become an unbounded memory/IO sink.
- Health status is derived from observable conditions.
