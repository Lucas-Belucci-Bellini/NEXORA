# NEXORA — ARCHITECTURE FREEZE CHECKLIST

## Goal
Provide an explicit gate before large-scale implementation and final technology lock.

## Foundation
- [ ] Core lifecycle defined
- [ ] Module lifecycle defined
- [ ] Job/threading model defined
- [ ] Resource ownership defined
- [ ] Time model defined
- [ ] Spatial model defined
- [ ] Registry/ID rules defined
- [ ] Event/Command/Query semantics defined

## Runtime
- [ ] RHI boundary defined
- [ ] Input boundary defined
- [ ] Audio boundary defined
- [ ] Asset lifecycle defined
- [ ] Streaming lifecycle defined
- [ ] Headless mode defined

## Simulation
- [ ] ECS/data model defined
- [ ] Physics ownership defined
- [ ] AI decision pipeline defined
- [ ] LOD transitions defined
- [ ] performance budgets defined
- [ ] deterministic requirements defined

## World
- [ ] Chunk lifecycle defined
- [ ] world-state lifecycle defined
- [ ] generation seed reproducibility defined
- [ ] persistence boundary defined
- [ ] world event lifecycle defined

## Living world
- [ ] civilization ownership defined
- [ ] economy ownership defined
- [ ] knowledge/information boundaries defined
- [ ] history truth model defined
- [ ] lore derivation defined
- [ ] archive/evidence model defined
- [ ] player-independence rules defined

## Network / security
- [ ] authority model defined
- [ ] replication boundaries defined
- [ ] threat model defined
- [ ] validation invariants defined
- [ ] mod/script trust model defined

## Content / tools
- [ ] content pipeline defined
- [ ] asset provenance defined
- [ ] original-content policy defined
- [ ] editor/runtime relationship defined
- [ ] mod API versioning defined

## Engineering
- [ ] save compatibility defined
- [ ] crash/recovery strategy defined
- [ ] observability defined
- [ ] testing strategy defined
- [ ] CI/build/release strategy defined
- [ ] technology benchmark defined

## Final gate
The architecture can be frozen only when unresolved items are either completed or explicitly classified as post-freeze extensions with no impact on frozen contracts.
