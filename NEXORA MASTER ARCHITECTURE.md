# NEXORA — MASTER ARCHITECTURE

> This document is the integration map for all NEXORA systems. Individual system documents define local responsibilities; this document defines boundaries, dependency direction and the implementation order.

## 1. Architectural layers
```text
PLATFORM
  ↓
FOUNDATION
  ↓
RUNTIME SERVICES
  ↓
WORLD / SIMULATION
  ↓
GAMEPLAY
  ↓
SOCIETY
  ↓
PRESENTATION / TOOLS
```

## 2. Foundation
```text
Core
Registry
Event Bus
Command System
Save / Persistence
Security
Job / Task System
Versioning / Migration
Diagnostics
```

Foundation owns contracts and orchestration primitives, not game content.

## 3. Runtime services
```text
Resource / Asset
Streaming
Performance
Navigation / Pathfinding
Content Pipeline
Mod Runtime
Scripting
Networking
Server
```

## 4. World
```text
Chunk / Voxel
World Generation
Biomes
Caves / Deep World
Climate / Atmosphere
Water / Fluids
Vegetation
Lighting
Structures
Dimensions
Space
World Events
```

## 5. Simulation
```text
Entity
Physics
AI / Perception
Navigation
Vehicles
Interaction
Machines
Energy
Fluid
Automation
```

## 6. Gameplay
```text
Player
Items
Inventory / Equipment
Tools / Weapons
Crafting
Combat
Agriculture
Fishing
Hunting / Wildlife
Magic
Enchanting
Quest
Progression / Technology
Achievements
```

## 7. Society
```text
Social / Factions
Economy
Advanced Industry
Advanced Civilization
Research / Knowledge
Communication
Sensors
Cartography
```

## 8. Presentation
```text
Renderer / Graphics
Animation
Audio
UI
Menus
Localization
Accessibility
Trading UI
Civilization UI
```

## 9. Modding and authoring
```text
Mod API
Mod Runtime
Mod SDK
Resource Packs
Content Pipeline
World / Structure Editors
Tooling
```

## 10. Platform / distribution
```text
Launcher
Installer / Updater
Mod Distribution
Dedicated Server
Release Artifacts
```

## 11. Dependency rules
```text
Core
↓
Public APIs
↓
Registries / Events / Commands
↓
Systems
↓
Official Content + Mods
```

Domain systems may depend on lower layers and public neighboring contracts, but lower layers must not depend on higher-level gameplay concepts.

## 12. Authority
```text
Client → request
Command → intent
Server → authority
Simulation → result
Event Bus → communication
Persistence → durability
Networking → transport
```

## 13. Data ownership
Each subsystem owns its authoritative state. Other systems consume snapshots, queries, capabilities or events rather than hidden mutable state.

## 14. LOD
```text
FULL → REGIONAL → ABSTRACT → UNRESIDENT
```
LOD changes simulation representation, not logical existence or persistent identity.

## 15. Implementation order
```text
A Foundation
B Job / Resource / Streaming / Performance
C Voxel + Entity + Physics
D WorldGen + Rendering + Persistence
E Player + Interaction + Item + Inventory
F Crafting + Machines + Energy + Fluid
G AI + Navigation + Vehicles + Railway
H Economy + Research + Technology + Civilization
I World Events + Communication + Sensors + Cartography
J Mod Runtime + Scripting + SDK
K Multiplayer + Server hardening
L Editors + Launcher + Distribution
M Content expansion
```

## 16. Golden integration loop
```text
Generate world
→ load chunk
→ create entity
→ player interacts
→ command validated
→ system changes state
→ event emitted
→ save dirty state
→ network replication
→ unload
→ reload
→ state preserved
```

## 17. Security boundary
No client, script or untrusted mod receives authoritative world mutation without explicit permission and server validation. Resource, event, job, command and storage budgets are mandatory.

## 18. Performance boundary
Every system exposes measurable budgets and supports batching/LOD. High-frequency data uses local buffers or batches rather than unbounded global event traffic.

## 19. Test boundary
Every system requires unit/contract tests, at least one integration slice, persistence coverage where stateful, fault tests and a representative benchmark.

## 20. Definition of architecture complete
A system is considered architecturally complete when its responsibilities, non-responsibilities, state ownership, API, events/commands, persistence policy, networking policy, security limits, LOD strategy, tests and integration slices are documented.

## Final principle
> **The Core provides the rules and contracts. Systems provide capabilities. Content uses those capabilities. The world changes through validated operations and keeps living through simulation.**
