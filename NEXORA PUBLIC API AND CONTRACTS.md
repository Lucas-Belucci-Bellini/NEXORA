# NEXORA — PUBLIC API AND CONTRACTS

## Principle
Public APIs are explicit contracts between Core, systems, tools, mods and scripts. Internal implementation details must not leak across boundaries without a deliberate contract.

## Contract types
```text
QUERY   → read information
COMMAND → request a state change
EVENT   → report a completed fact
RESOURCE→ address/load shared data
SERVICE → long-lived capability
```

## API rules
- Every public identifier has a documented owner.
- Runtime handles are not stable persistence identifiers unless explicitly declared.
- Public contracts must define versioning policy.
- Clients cannot bypass server authority through public APIs.
- Mods use the same public capability model as first-party content, subject to permissions.
- Internal APIs must not be exposed accidentally as stable mod APIs.

## Contract lifecycle
```text
PROPOSED
→ REVIEWED
→ EXPERIMENTAL
→ STABLE
→ DEPRECATED
→ REMOVED
```

## Compatibility
Breaking changes require an ADR or equivalent decision record, migration notes and compatibility impact analysis.

## Contract test
Each critical public contract should have:
```text
schema test
behavior test
version test
serialization test
security test
mod compatibility test
```
