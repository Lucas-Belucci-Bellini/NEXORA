# NEXORA — WORLD STATE LIFECYCLE

## Purpose
Define how a world moves from creation to active simulation, streaming, persistence, suspension and recovery.

## Lifecycle
```text
WORLD_REQUEST
→ WORLD_VALIDATE
→ SEED_INITIALIZE
→ GENERATE_ROOT_STATE
→ LOAD_PERSISTED_STATE (optional)
→ REGISTER_WORLD
→ REGION_DISCOVERY
→ STREAM_IN
→ ACTIVE_SIMULATION
→ LOD_TRANSITION
→ STREAM_OUT
→ SAVE_CHECKPOINT
→ WORLD_SUSPEND
→ WORLD_RESUME
→ WORLD_CLOSE
```

## World identity
A world is identified independently of loaded chunks or processes. Its persistent identity survives server restart and spatial unloading.

## State classes
```text
STATIC
DERIVED
SIMULATED
HISTORICAL
KNOWLEDGE
TRANSIENT
```

Only authoritative persistent state is serialized. Derived caches and presentation data may be rebuilt.

## Streaming rule
Unloaded regions must preserve the logical world state needed for future simulation. LOD may compress representation but must not silently erase important consequences.

## Long absence
When the player is absent, the world may use REGIONAL or ABSTRACT simulation according to budget and importance. Major events, historical records, civilization state and other required persistent data must continue to advance.

## Recovery
A failed load must produce a recoverable state through checkpoint/journal recovery or quarantine rather than silently overwriting valid history.
