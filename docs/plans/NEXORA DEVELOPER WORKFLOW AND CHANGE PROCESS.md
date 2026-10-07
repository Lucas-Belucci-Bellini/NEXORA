# NEXORA — DEVELOPER WORKFLOW AND CHANGE PROCESS

## Principle
Every meaningful engineering change must be discoverable, reviewable and reversible.

## Change flow
```text
IDEA
→ ISSUE / DESIGN NOTE
→ ARCHITECTURE IMPACT CHECK
→ IMPLEMENTATION
→ TESTS
→ BENCHMARK IF RELEVANT
→ REVIEW
→ MERGE
→ DOCUMENTATION UPDATE
```

## Change classes
```text
DOC
BUGFIX
FEATURE
ARCHITECTURE
PERFORMANCE
SECURITY
DATA/MIGRATION
BREAKING API
CONTENT
```

## Required architecture review
Changes must identify whether they alter:
- ownership;
- dependency direction;
- public API;
- persistence;
- networking;
- threading;
- security;
- LOD;
- mod compatibility;
- language/FFI boundary.

## Breaking changes
A breaking architectural/API change requires an ADR, migration strategy and compatibility impact record.

## Documentation rule
Implementation that invalidates an architectural document must update that document in the same change whenever practical.

## Experimental work
Experiments may live separately from stable contracts. An experiment must not silently become normative architecture.

## Reversibility
Prefer small commits, explicit migration steps and feature boundaries that allow rollback.
