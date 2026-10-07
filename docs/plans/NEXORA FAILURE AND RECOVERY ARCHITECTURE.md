# NEXORA — FAILURE AND RECOVERY ARCHITECTURE

## Goal
Failures must be contained, observable and recoverable. A bad subsystem must not silently corrupt unrelated world state.

## Failure classes
```text
LOCAL
RESOURCE
THREADING
IO
NETWORK
SERIALIZATION
MOD/SCRIPT
SIMULATION
WORLD DATA
PROCESS
```

## Handling pipeline
```text
DETECT
→ CLASSIFY
→ CONTAIN
→ RECORD
→ RECOVER / RESTART / QUARANTINE
→ VALIDATE
→ RESUME
```

## Rules
- Never hide a data-integrity failure.
- Prefer failing a bounded operation over corrupting global state.
- Save corruption must be quarantined when possible.
- Failed mod/script execution is isolated according to trust level.
- Background workers report failures to an owning supervisor.
- Recovery must be deterministic where the affected subsystem requires determinism.

## Recovery levels
```text
RETRY
→ ROLLBACK
→ RELOAD
→ RECONSTRUCT
→ QUARANTINE
→ SAFE MODE
```

## World safety
Before applying large irreversible changes, systems should provide enough transaction/journal information to recover from interrupted writes or process termination.

## Observability
Every critical failure should expose:
```text
failure_id
system
operation
world_id
entity/resource if applicable
thread/job if applicable
version
cause
recovery action
result
```
