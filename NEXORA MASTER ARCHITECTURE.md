# NEXORA — MASTER ARCHITECTURE

> This document is the integration map for all NEXORA systems. Individual system documents define local responsibilities; this document defines boundaries, dependency direction, runtime layers, technology decision gates and implementation order.

## Architectural layers
```text
PLATFORM / OS
  ↓
RHI / AUDIO / INPUT / FILESYSTEM
  ↓
FOUNDATION
  ↓
RUNTIME SERVICES
  ↓
DATA-ORIENTED SIMULATION
  ↓
WORLD / SIMULATION
  ↓
GAMEPLAY
  ↓
SOCIETY
  ↓
PRESENTATION / EDITOR / TOOLS
  ↓
DISTRIBUTION
```

## Foundation
Core, Registry, Event Bus, Command System, Save/Persistence, Security, Job/Task, Versioning/Migration, Diagnostics, Configuration/Settings, Time/Calendar, Spatial/Coordinate System and Engine Module System.

## Platform abstraction
RHI, Input, Camera-facing presentation contracts, filesystem/resource interfaces and audio backend boundaries isolate platform APIs from gameplay/runtime logic.

## Runtime services
Resource/Asset, Streaming, Performance, Content Pipeline, Mod Runtime, Scripting, Networking and Server.

## Data-oriented runtime
ECS/Data-Oriented Runtime provides hot-data storage, batch queries, archetype/composition management and scalable simulation. Public Entity System remains the logical identity/lifecycle API.

## World
Chunk/Voxel, World Generation, Biomes, Caves/Deep World, Climate/Atmosphere, Water/Fluids, Vegetation, Lighting, Structures, Prefab/Assembly, Scene/Level Runtime, Dimensions, Space, World Events, Navigation/Pathfinding, Map/Cartography, Communication and Sensors.

## Simulation
Entity, ECS/Data Runtime, Physics, AI/Perception, Navigation, Vehicles, Interaction, Machines, Energy, Fluid and Automation, with Railway as the specialized transport-network layer.

## Gameplay
Player, Input, Items, Inventory/Equipment, Tools/Weapons, Crafting, Combat, Attribute/Tag/Ability/Effect, Agriculture, Fishing, Hunting/Wildlife, Magic, Enchanting, Quest, Progression/Technology and Achievements.

## Society
Social/Factions, Economy, Advanced Industry, Advanced Civilization, Research/Knowledge and related communication/civilization systems.

## Rendering / presentation
Renderer/Graphics, RHI, Material/Shader, Lighting, VFX/Particles, Animation, Audio, Camera, UI, Menus, Localization, Accessibility, Trading UI, Civilization UI, Research UI and Social UI.

## Modding and authoring
Mod API, Mod Runtime, Mod SDK, Resource Packs, Content Pipeline, Prefab/Assembly, World/Structure Editors and Tooling.

## Platform / distribution
Launcher, Installer/Updater, Mod Distribution, Dedicated Server, Release Artifacts, Diagnostics and Replay/History.

## Dependency direction
```text
Platform / RHI
      ↓
Foundation
      ↓
Public APIs
      ↓
Registries / Events / Commands
      ↓
Runtime Services
      ↓
Simulation / World
      ↓
Gameplay
      ↓
Society
      ↓
Presentation / Tools
```

Lower layers must not depend on higher-level gameplay concepts. Systems consume public contracts rather than hidden mutable state.

## Authority
```text
Client → request / input snapshot
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

## Required cross-cutting contracts

Every major system must define:

```text
ownership
state model
public API
commands
events
persistence
network policy
security boundary
LOD behavior
threading behavior
resource budget
observability
mod/script boundary
tests / vertical slice
```

## Technology decision gate

Language and implementation stack are selected **after** the architecture contracts are stable enough to benchmark.

Candidate architectural lessons:

```text
UE5
→ large-world coordinates
→ World Partition/HLOD concepts
→ data-oriented Mass Entity
→ gameplay abilities/attributes/effects
→ contextual input

O3DE
→ modular engine/component model
→ Gems-style isolation
→ asset pipeline
→ editor/runtime separation

ECS/data-oriented approaches
→ batch processing
→ hot/cold data separation
→ scalable populations

Godot-style separation
→ engine servers/subsystems
→ platform drivers
```

NEXORA must implement its own code, APIs and data model.

## Benchmark before language lock

```text
window
→ RHI
→ camera
→ voxel chunk
→ mesh generation
→ player movement
→ physics
→ 1,000 entities
→ job system
→ streaming
→ save/load
→ headless server
→ basic mod boundary
```

Measure build time, iteration speed, memory, frame time, simulation time, job overhead, serialization, debugging, tooling and concurrency complexity.

## Implementation order
```text
A Architecture / contracts
B Foundation + Module + Job + Resource + Time + Spatial
C RHI + Input + Camera + Renderer + Material + VFX
D Voxel + Entity + Data Runtime + Physics
E WorldGen + Streaming + Persistence + Scene/Prefab
F Player + Interaction + Item + Inventory
G Crafting + Machines + Energy + Fluid + Automation
H AI + Navigation + Vehicles + Railway
I Economy + Research + Technology + Civilization + Social
J World Events + Communication + Sensors + Cartography
K Multiplayer + Server hardening
L Mod Runtime + Scripting + SDK + Content Pipeline
M Editors + Launcher + Distribution
N Farming + Fishing + Hunting + Magic + additional gameplay
O Content expansion
P Production hardening / benchmark / release
```

## Golden integration loop
```text
Generate world
→ load chunk
→ activate runtime scene
→ create entity
→ player input / AI intent
→ command validated
→ system changes state
→ event emitted
→ save dirty state
→ network replication
→ render/audio/VFX presentation
→ unload
→ reload
→ state preserved
```

## Completion criteria
A system is architecturally complete when responsibilities, non-responsibilities, state ownership, API, commands/events, persistence, networking, security limits, LOD, threading/budget policy, tests and integration slices are documented.

## Architecture exit criteria
The project may move from architecture into implementation-language selection when:

- major systems have explicit owners;
- no unresolved circular dependency exists;
- authority boundaries are explicit;
- RHI/platform boundary is fixed;
- data-oriented storage strategy is fixed;
- job/threading model is defined;
- serialization/versioning model is defined;
- content/mod/script boundaries are defined;
- editor/runtime relationship is defined;
- benchmark harness is defined.

## Final principle
> **The Core provides rules and contracts. Systems provide capabilities. Content uses those capabilities. The world changes through validated operations and keeps living through simulation.**
