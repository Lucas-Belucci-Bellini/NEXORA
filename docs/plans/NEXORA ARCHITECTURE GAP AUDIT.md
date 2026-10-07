# NEXORA — ARCHITECTURE GAP AUDIT

## Purpose

This document records the architecture audit performed before choosing NEXORA's implementation language and engine technology.

## Status

The previous missing-systems checklist is now obsolete. The repository contains architecture documents for the original foundation and gameplay systems plus the missing platform, simulation, tooling, content and player-experience areas.

## Newly identified engine-critical systems

```text
Input System
Camera System
ECS / Data-Oriented Runtime
Attribute / Tag / Ability System
Time / Calendar System
Spatial / Coordinate System
Render Hardware Interface
Prefab / Assembly System
Configuration / Settings System
Engine Module System
Material / Shader System
VFX / Particle System
Scene / Level Runtime
Engine Architecture / Technology Decision
```

## Architecture families

### Foundation
Core, Registry, Event Bus, Command, Persistence, Security, Job System, Resource/Asset, Streaming, Performance, Time, Spatial, Configuration and Engine Modules.

### Rendering
Renderer, Lighting, RHI, Materials/Shaders, VFX, Animation, Audio and Camera.

### World
World Generation, Biomes, Chunks/Voxels, Caves/Deep World, Climate/Atmosphere, Water/Fluids, Vegetation, Structures, Dimensions, Space, Navigation and Cartography.

### Simulation
Entity, data-oriented runtime, Physics, AI-related systems, Interaction, Vehicles, Railway, Sensors, Communication and World Events.

### Gameplay
Player, Input, Item, Inventory, Crafting, Combat, Tools/Weapons, Agriculture, Fishing, Hunting/Wildlife, Magic, Enchanting, Quests and Progression.

### Society
Social/Factions, Economy, Advanced Industry, Advanced Civilization, Research/Knowledge, Technology and civilization/research/social/trading UIs.

### Modding / Content
Mod Runtime, Scripting, Mod SDK, Resource Packs, Content Pipeline, Prefab/Assembly and World/Structure Editors.

### Platform
Networking, Server, Launcher, Installer/Updater, Mod Distribution, Diagnostics, Replay/History, Localization, Accessibility, Achievements/Advancements and Menu system.

### Engineering
Testing Architecture, Tooling Architecture, Versioning/Migration and build/release requirements documented across the platform documents.

## Remaining work is not an obvious missing top-level system

The remaining architectural work should now focus on:

1. Contract reconciliation between documents.
2. Dependency-direction validation.
3. Shared type/ID conventions.
4. Threading and job ownership rules.
5. Memory ownership rules.
6. Serialization/schema ownership.
7. Client/server authority boundaries.
8. LOD transition contracts.
9. Editor/runtime parity.
10. Mod/script trust boundaries.
11. Cross-system golden vertical slices.
12. Final technology benchmark and language selection.

## Potential future sub-systems

These should not automatically become independent top-level engines. They can be modules/subsystems under existing contracts unless implementation evidence proves otherwise:

```text
Gameplay Timer
Dialogue
Cinematics
Weather Rendering
HLOD/World Partition implementation
GPU-driven scene management
Material graph tooling
Profiling UI
Server orchestration / clustering
Voice communication
Photo/replay capture
```

## Language decision gate

NEXORA is **not ready to lock a language solely from feature count**. The architecture is ready for a technology benchmark.

Required benchmark:

```text
RHI
→ window
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

Candidate stacks should be compared against the same workload and measured for memory, frame time, simulation time, build time, iteration speed, tooling, serialization, debugging and concurrency complexity.

## Engine recommendation

Do not clone Unreal Engine 5, O3DE, Godot or Bevy.

Use their public architectural lessons where appropriate and implement NEXORA's own contracts.

The current strongest architectural direction is:

```text
UE5-inspired
large-world + streaming + gameplay framework concepts

O3DE-inspired
modularity + component/gem-style isolation + asset pipeline

ECS/data-oriented
batch simulation + hot/cold data separation

Godot-inspired
clear subsystem/platform boundaries

NEXORA
original voxel-first runtime + original APIs + original simulation model
```

## Exit criteria

The architecture phase can move to technology selection when:

- all top-level contracts have owners;
- no unresolved cyclic dependency exists;
- authoritative state ownership is explicit;
- RHI/platform boundary is fixed;
- threading model is fixed;
- serialization model is fixed;
- content/mod boundary is fixed;
- editor/runtime relationship is fixed;
- benchmark harness is defined;
- candidate language stacks can implement the same vertical slice.
