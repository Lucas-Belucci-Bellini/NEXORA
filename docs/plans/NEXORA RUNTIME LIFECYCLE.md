# NEXORA — RUNTIME LIFECYCLE

## Purpose
Define the lifecycle of the engine process and prevent hidden initialization order dependencies.

## Global lifecycle
```text
PROCESS_START
→ PLATFORM_INIT
→ FOUNDATION_INIT
→ MODULE_DISCOVERY
→ MODULE_RESOLUTION
→ RESOURCE_BOOTSTRAP
→ RUNTIME_INIT
→ WORLD_ATTACH
→ SIMULATION_RUNNING
→ PRESENTATION_RUNNING
→ SHUTDOWN_REQUESTED
→ SIMULATION_STOP
→ WORLD_FLUSH
→ MODULE_STOP
→ RESOURCE_RELEASE
→ PLATFORM_SHUTDOWN
→ PROCESS_EXIT
```

## Rules
- A module may only consume dependencies that have completed required initialization.
- Shutdown occurs in reverse dependency order.
- Persistent state must be flushed before its owner is destroyed.
- Background jobs must reach a known quiescent state before dependent resources are released.
- Client, server, headless, editor and test modes may use different module selections but the same lifecycle contracts.

## Safe states
```text
BOOTING
READY
RUNNING
PAUSING
PAUSED
STOPPING
FAILED
```

## Failure handling
Initialization failure must identify the owning module, dependency chain and cleanup actions. Partial initialization must not be treated as a valid running state.

## Hot reload
Hot reload is opt-in. Any module supporting reload must declare state serialization, resource migration and thread-quiescence requirements.
