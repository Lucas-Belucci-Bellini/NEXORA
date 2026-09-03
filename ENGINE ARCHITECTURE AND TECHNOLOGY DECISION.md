# NEXORA — ENGINE ARCHITECTURE / TECHNOLOGY DECISION

> This document defines what NEXORA needs from an engine/runtime architecture before selecting implementation languages and libraries. It is inspired by proven industry patterns, not copied source code or proprietary implementations.

## 1. Decision status

**Language selection: NOT YET FINAL.**

The project must choose technology only after the architecture contracts below are stable enough to evaluate real implementation trade-offs.

## 2. What NEXORA needs

```text
Large Voxel World
+ Deep Vertical World
+ Procedural Generation
+ Persistent Simulation
+ Thousands of Mob Types
+ Large NPC Populations
+ Civilizations / Economy / Industry
+ Vehicles / Railway
+ Space / Dimensions
+ Multiplayer
+ Native Modding
+ Scripting
+ Editor Tooling
+ Deterministic Save / Recovery
+ High Performance
```

## 3. Architectural inspirations

NEXORA may borrow **ideas and architectural lessons**, never protected implementation or proprietary code.

### Unreal Engine 5 lessons
Useful concepts to study include:
- World Partition-style spatial streaming and hierarchical LOD;
- Large World Coordinates / explicit large-world precision;
- data-oriented Mass Entity for high population simulation;
- Gameplay Ability / Attribute / Effect separation;
- contextual input mapping;
- mature editor/tooling separation.

These are architectural references. NEXORA must implement its own contracts and code.

### Open 3D Engine lessons
O3DE is particularly relevant to NEXORA's modular goal because it organizes functionality as modular components/Gems, uses a data-driven asset pipeline, has a modular multi-threaded renderer, entity-component architecture, networking, scripting and editor tooling.

### Bevy-style lessons
A data-oriented ECS, explicit systems, query-based processing and strong ownership boundaries are useful ideas for large simulation. NEXORA does not need to copy Bevy's API or runtime design.

### Godot-style lessons
Clear separation between scene-level objects, engine servers/subsystems and low-level platform drivers is useful for keeping rendering/audio/physics backend details out of gameplay.

## 4. Preliminary conclusion

For NEXORA, the strongest target is **not to select one existing engine and imitate it literally**.

The preferred architecture is a **hybrid design**:

```text
UE5
├── large-world concepts
├── world streaming / HLOD concepts
├── gameplay framework patterns
└── mature tool separation

O3DE
├── modular engine/component philosophy
├── asset pipeline
├── engine extensibility
└── editor/runtime separation

Data-oriented ECS
├── batch simulation
├── cache-friendly hot data
├── scalable populations
└── deterministic parallel processing

Godot-style separation
├── platform abstraction
├── subsystem boundaries
└── clean runtime drivers

NEXORA
└── original implementation + original public APIs
```

## 5. Language-neutral requirements

Before choosing language, the implementation must support or provide equivalents for:

- deterministic simulation where required;
- high-performance data-oriented storage;
- safe concurrency and a job system;
- explicit memory ownership/lifetime;
- low-overhead serialization;
- native graphics API access through an RHI abstraction;
- native file/IO access;
- asynchronous streaming;
- server/headless builds;
- tooling/editor applications;
- scripting/runtime embedding;
- C-compatible or equivalent stable plugin boundary if useful;
- debugging/profiling integrations;
- cross-platform build and packaging.

## 6. Architecture layers

```text
PLATFORM
  ↓
RHI / AUDIO / INPUT / FILESYSTEM
  ↓
FOUNDATION
  ↓
JOB / RESOURCE / TIME / SPATIAL
  ↓
RUNTIME SERVICES
  ↓
DATA-ORIENTED SIMULATION
  ↓
WORLD / GAMEPLAY
  ↓
SOCIETY / WORLD SIMULATION
  ↓
PRESENTATION / EDITOR / TOOLING
```

## 7. Engine boundary

Core engine owns:
- lifecycle;
- memory/resource abstractions;
- jobs;
- platform abstraction;
- time;
- spatial coordinates;
- serialization primitives;
- registries;
- event/command infrastructure;
- rendering/audio/input interfaces.

Gameplay content must remain above these contracts.

## 8. Runtime modes

The selected technology must support:

```text
CLIENT
DEDICATED SERVER
LISTEN SERVER
HEADLESS SIMULATION
EDITOR
TOOLS
TEST RUNNER
BENCHMARK RUNNER
REPLAY RUNNER
```

## 9. Rendering target

Renderer should use a backend-neutral RHI and a render graph. Required long-term features include:
- modern GPU APIs;
- deferred/forward paths as appropriate;
- virtualized/streamed geometry where justified;
- physically based materials;
- large-world precision;
- GPU-driven batching where useful;
- headless/null rendering backend.

## 10. Simulation target

NEXORA must be able to process:

```text
voxel worlds
10k+ active entities
100k+ abstract entities
large NPC populations
large civilization counts
large industrial networks
```

without requiring all state to exist as heavyweight object instances.

## 11. Modding target

The technology must support three trust levels:

```text
DATA-ONLY
SANDBOXED SCRIPT / WASM / MANAGED
TRUSTED NATIVE
```

The exact runtime implementation is a later decision.

## 12. Editor target

The editor should consume the same public resource/registry/world contracts used by runtime where possible. Avoid creating a second incompatible world model only for tools.

## 13. Selection criteria

Each candidate stack will be scored against:

| Criterion | Required |
|---|---|
| large-world precision | yes |
| voxel streaming | yes |
| data-oriented simulation | yes |
| multithreaded jobs | yes |
| custom renderer/RHI | yes |
| headless server | yes |
| native mod boundary | yes |
| scripting | yes |
| editor/tooling | yes |
| deterministic persistence | yes |
| cross-platform | yes |
| profiling/debugging | yes |
| long-term maintainability | yes |

## 14. Important rule

Do **not** choose a language because it is fashionable or because another engine uses it. Choose based on whether the language/runtime can satisfy the NEXORA architecture without creating unacceptable complexity in memory, concurrency, tooling, graphics, modding or iteration speed.

## 15. Current recommendation

Do not lock the language yet.

First lock:

```text
Architecture
→ data model
→ threading model
→ memory model
→ RHI boundary
→ asset model
→ plugin boundary
→ scripting boundary
→ editor boundary
→ server boundary
```

Then benchmark a small vertical slice in the strongest candidate stacks.

## 16. Required benchmark before final language decision

Build the same prototype in candidate stacks:

```text
window
→ RHI
→ camera
→ 16×16×16 voxel chunk
→ mesh generation
→ player movement
→ 1,000 entities
→ streaming
→ save/load
→ 8 worker jobs
→ headless server
```

Measure:
- build times;
- iteration speed;
- memory;
- frame time;
- simulation time;
- job overhead;
- serialization speed;
- debugging quality;
- tooling effort;
- code complexity.

The benchmark result, not preference alone, should choose the implementation stack.

## 17. IP rule

NEXORA may study Unreal Engine, O3DE, Bevy, Godot and other public documentation for architectural lessons. It must not copy source code, proprietary assets or distinctive implementation details protected by license or copyright.

## 18. References

Official architecture references consulted during this decision:
- Unreal Engine World Partition / HLOD documentation
- Unreal Engine Large World Coordinates documentation
- Unreal Engine Mass Entity documentation
- Unreal Engine Gameplay Ability System documentation
- Unreal Engine Enhanced Input documentation
- Open 3D Engine architecture, Gems, Atom Renderer and component documentation
- Godot architecture documentation

This file is a technology-study document, not a commitment to any specific engine or language.
