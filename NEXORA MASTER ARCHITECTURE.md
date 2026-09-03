# NEXORA — MASTER ARCHITECTURE

> This document is the integration map for all NEXORA systems. Individual system documents define local responsibilities; this document defines boundaries, dependency direction and the implementation order.

## Architectural layers
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

## Foundation
Core, Registry, Event Bus, Command System, Save/Persistence, Security, Job/Task, Versioning/Migration and Diagnostics.

## Runtime services
Resource/Asset, Streaming, Performance, Navigation/Pathfinding, Content Pipeline, Mod Runtime, Scripting, Networking and Server.

## World
Chunk/Voxel, World Generation, Biomes, Caves/Deep World, Climate/Atmosphere, Water/Fluids, Vegetation, Lighting, Structures, Dimensions, Space and World Events.

## Simulation
Entity, Physics, AI/Perception, Navigation, Vehicles, Interaction, Machines, Energy, Fluid and Automation, with Railway as the specialized transport-network layer.

## Gameplay
Player, Items, Inventory/Equipment, Tools/Weapons, Crafting, Combat, Agriculture, Fishing, Hunting/Wildlife, Magic, Enchanting, Quest, Progression/Technology and Achievements.

## Society
Social/Factions, Economy, Advanced Industry, Advanced Civilization, Research/Knowledge, Communication, Sensors and Cartography.

## Presentation
Renderer/Graphics, Animation, Audio, UI, Menus, Localization, Accessibility, Trading UI, Civilization UI, Research UI and Social UI.

## Modding and authoring
Mod API, Mod Runtime, Mod SDK, Resource Packs, Content Pipeline, World/Structure Editors and Tooling.

## Platform / distribution
Launcher, Installer/Updater, Mod Distribution, Dedicated Server and Release Artifacts.

## Dependency direction
```text
Core
 ↓
Public APIs
 ↓
Registries / Events / Commands
 ↓
Systems
 ↓
Official Content + Community Mods
```

Lower layers must not depend on higher-level gameplay concepts. Systems consume public contracts rather than hidden mutable state.

## Authority
```text
Client → request
Command → intent
Server → authority
Simulation → result
Event Bus → communication
Persistence → durability
Networking → transport
```

## LOD
```text
FULL → REGIONAL → ABSTRACT → UNRESIDENT
```
Logical existence and persistent identity survive representation changes.

## Implementation order
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

## Golden integration loop
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

## Completion criteria
A system is architecturally complete when responsibilities, non-responsibilities, state ownership, API, commands/events, persistence, networking, security limits, LOD, tests and integration slices are documented.

## Final principle
> **The Core provides rules and contracts. Systems provide capabilities. Content uses those capabilities. The world changes through validated operations and keeps living through simulation.**
