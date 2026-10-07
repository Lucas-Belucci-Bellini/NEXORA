# NEXORA — DATA VALIDATION AND INVARIANTS

## Purpose
Define conditions that must always remain true across runtime, persistence, network and tools.

## Invariant classes
```text
IDENTITY
OWNERSHIP
STATE
SPATIAL
TEMPORAL
RESOURCE
NETWORK
PERSISTENCE
SECURITY
SIMULATION
```

## Examples
```text
Entity IDs are unique within their authority domain.
A resource cannot have two exclusive owners simultaneously.
A destroyed object cannot remain addressable as active state.
A persisted reference must resolve or enter an explicit missing-content state.
A client cannot elevate a command's authority by editing its payload.
Historical event IDs are immutable once committed.
```

## Validation levels
```text
PARSE
→ SCHEMA
→ DOMAIN
→ CROSS-SYSTEM
→ AUTHORITY
→ PERSISTENCE
```

## Runtime behavior
Invalid external input must be rejected or quarantined. Internal invariant failures are diagnostic faults and must identify the owner and recovery path.

## Testing
Critical invariants require automated tests and, where practical, property/fuzz tests.
