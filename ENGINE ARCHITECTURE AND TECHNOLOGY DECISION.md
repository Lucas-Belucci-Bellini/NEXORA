# NEXORA — ENGINE ARCHITECTURE / TECHNOLOGY DECISION

> This document defines what NEXORA needs from an engine/runtime architecture and how implementation languages are assigned to responsibilities. It is inspired by proven industry patterns, not copied source code or proprietary implementations.

## 1. Decision status

**Architecture direction: MULTI-LANGUAGE BY RESPONSIBILITY.**

**Final language selection: NOT YET LOCKED.**

NEXORA should not force every subsystem into one language. The architectural boundary is defined first; each responsibility is then assigned to the language/runtime that best satisfies its requirements.

The preferred current hypothesis is:

```text
RUST
→ core engine + high-cost runtime + simulation + server + world

C / C++ / RUST
→ platform and graphics backend integration where native GPU/API access justifies it

TYPESCRIPT + RUST
→ editor + SDK + developer tooling + user-facing tools

PYTHON
→ research + generation experiments + analysis + offline automation
```

This is a **working architecture hypothesis**, not a premature final commitment. A benchmark must validate the boundaries before the implementation stack is frozen.

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

## 3. Core principle — split by responsibility, not by fashion

The project must not start with:

```text
"Rust because Rust is fast"
"C++ because engines use C++"
"TypeScript because the editor is easy"
```

Instead:

```text
Architecture contract
        ↓
Responsibility / workload
        ↓
Performance + safety + tooling requirements
        ↓
Language/runtime selection
```

A language boundary is justified only when it provides a meaningful engineering advantage without creating excessive FFI, build, debugging, deployment or maintenance cost.

## 4. Proposed language responsibility map

### 4.1 Rust — primary NEXORA runtime

Rust is the strongest current candidate for the main engine/runtime because NEXORA needs large amounts of concurrent, persistent, data-oriented simulation with explicit ownership and long-term safety.

Planned Rust responsibility areas:

```text
Rust Runtime
├── Engine Core
├── ECS / Data-Oriented Runtime
├── Job / Task System
├── Registry
├── Event Bus
├── Command System
├── Save / Persistence
├── Serialization / Migration
├── Resource / Asset Runtime
├── Streaming
├── Time / Calendar
├── Spatial / Coordinate Runtime
├── Voxel / Chunk Runtime
├── World Generation
├── Biomes
├── Caves / Deep World
├── Climate / Atmosphere
├── Water / Fluid Simulation
├── Vegetation Simulation
├── Lighting Simulation
├── Physics
├── AI / Perception
├── Navigation / Pathfinding
├── Vehicles
├── Railway
├── Machines / Automation
├── Energy / Fluid Networks
├── World Events
├── Economy Simulation
├── Industry Simulation
├── Civilization Simulation
├── Population Simulation
├── Social / Faction Simulation
├── Research / Knowledge Simulation
├── Networking Core
├── Dedicated Server
├── Headless Runtime
├── Mod Runtime Host
├── Script Runtime Host
└── Security / Authority Enforcement
```

The objective is that **the expensive, continuously running simulation lives in one coherent runtime domain** instead of crossing multiple language boundaries every frame/tick.

## 5. Graphics and platform boundary

Graphics are different from gameplay simulation because native APIs and platform SDKs can require C/C++ interfaces or specialized low-level bindings.

Preferred architecture:

```text
Gameplay / Simulation
        ↓
Renderer API
        ↓
Render Graph
        ↓
RHI
        ↓
Native graphics abstraction
        ↓
Vulkan / DirectX / Metal / platform API
        ↓
GPU
```

The implementation may use:

```text
Rust
+ C ABI
+ selected C/C++ bindings where necessary
```

The rule is **not** "NEXORA must contain C++". The rule is:

> Use C/C++ only where its native ecosystem or API boundary creates a measurable advantage.

If Rust provides a clean and sufficiently mature path for a subsystem, avoid adding another language merely for tradition.

## 6. TypeScript — editor and development experience

TypeScript is a strong candidate for high-level tooling where rapid iteration, rich UI frameworks and developer productivity matter more than hot-loop execution.

Planned responsibilities:

```text
TypeScript
├── Editor UI
├── World Editor UI
├── Structure / Prefab Editor UI
├── Project Manager
├── Mod SDK UI
├── Asset Browser
├── Inspector / Property Panels
├── Debug Dashboards
├── Profiling UI
├── Documentation Tools
├── Content Authoring Tools
├── Launcher UI where appropriate
└── Development Services
```

When tooling needs high-performance native processing, it should call a Rust tool/backend rather than reimplementing engine logic in TypeScript.

## 7. Python — research and offline automation

Python should normally stay outside the critical runtime path.

Planned responsibilities:

```text
Python
├── Procedural-generation experiments
├── Dataset generation
├── Simulation analysis
├── Benchmark analysis
├── World statistics
├── AI / ML experimentation
├── Asset processing helpers
├── Validation scripts
├── Content-generation utilities
├── Test orchestration
└── Developer automation
```

Typical flow:

```text
Python experiment
      ↓
Generated parameters / dataset / test case
      ↓
Rust runtime
      ↓
Real NEXORA simulation
      ↓
Telemetry / results
      ↓
Python analysis
```

Python must not become the hidden dependency for core gameplay execution.

## 8. Native boundary rules

The architecture must avoid arbitrary chains such as:

```text
Rust → C++ → Python → TypeScript → Rust
```

Instead, each language should have a clear boundary:

```text
                 PUBLIC CONTRACT
                       │
        ┌──────────────┼──────────────┐
        │              │              │
       Rust        Native API      TypeScript
        │              │              │
     Runtime        Graphics       Tools/UI
        │
       FFI
        │
   platform/native

Python
  ↑
offline analysis / research
```

Prefer stable contracts based on:

```text
C-compatible ABI
or
versioned IPC
or
well-defined network protocol
or
serialized artifact/data format
```

The chosen boundary depends on latency and lifecycle requirements.

## 9. What must NOT cross the boundary frequently

Avoid crossing language boundaries inside hot loops such as:

```text
per-entity simulation
per-voxel updates
per-particle updates
per-frame gameplay calls
per-tick physics calls
per-cell AI updates
```

Hot data should remain in the owning runtime.

For example:

```text
GOOD
Rust ECS
→ processes 100,000 entities
→ emits aggregated telemetry
→ TypeScript visualizes telemetry

BAD
Rust ECS
→ calls TypeScript once per entity
→ TypeScript returns state
→ Rust resumes simulation
```

## 10. Ownership rule

Every subsystem must have one authoritative owner.

Example:

```text
Physics
→ owns physical state transition

AI
→ owns decision generation

Navigation
→ owns route computation

Economy
→ owns economic simulation

Renderer
→ owns graphical representation

Editor
→ owns authoring state before runtime commit
```

Another language may inspect or request operations, but it must not create a second competing source of truth.

## 11. Runtime architecture

```text
                         NEXORA
                            │
                  ┌─────────┴─────────┐
                  │     RUNTIME       │
                  └─────────┬─────────┘
                            │
                           Rust
                            │
        ┌───────────────────┼────────────────────┐
        │                   │                    │
      WORLD              SIMULATION           SERVER
        │                   │                    │
     Voxel             ECS / Jobs          Networking
     Terrain            Physics            Authority
     Biomes             AI                 Security
     Caves              Economy             Persistence
     Dimensions         Civilization        Replication

                            │
                           APIs
                            │
             ┌──────────────┼──────────────┐
             │                             │
         Graphics / Platform            Tools
             │                             │
       Rust / C / C++                 TS + Rust
                                             │
                                           Editor
                                           SDK
                                           Debug

                            │
                         Offline
                            │
                         Python
                            │
                  Research / Analysis
```

## 12. Modding architecture

Mod support must not assume that every mod uses the engine language.

Target trust levels:

```text
DATA-ONLY
  ↓
SANDBOXED SCRIPT / WASM / MANAGED
  ↓
TRUSTED NATIVE
```

Possible implementation model:

```text
Rust engine
    ↓
Mod API / ABI / sandbox boundary
    ↓
mod runtime
    ├── data
    ├── script
    └── native extension
```

Native extensions must never bypass server authority, resource limits, security policy or lifecycle management merely because they are native.

## 13. Editor architecture

The editor should consume the same contracts as the runtime wherever practical.

```text
Editor
  ↓
Public NEXORA APIs
  ↓
Rust services
  ↓
World / Registry / Resource / Prefab systems
```

Avoid creating a second incompatible world model solely for the editor.

TypeScript is the preferred current candidate for the UI layer, while heavy world queries, asset operations and simulation previews can be delegated to Rust.

## 14. Build architecture

The repository should make language boundaries explicit:

```text
engine/
  core/                 Rust
  world/                Rust
  simulation/           Rust
  server/               Rust
  networking/           Rust
  renderer/             Rust + native bindings where required

platform/
  native/               Rust/C/C++ as required

editor/
  ui/                   TypeScript
  backend/              Rust

scripts/
  research/             Python
  automation/           Python

tools/
  native/               Rust
  ui/                   TypeScript
```

Exact repository paths can evolve; the important requirement is explicit ownership and dependency direction.

## 15. Dependency direction

The dependency graph should remain one-way:

```text
Platform / Native
        ↓
RHI / Audio / Input / Filesystem
        ↓
Foundation
        ↓
Runtime Services
        ↓
Simulation / World
        ↓
Gameplay
        ↓
Society
        ↓
Presentation / Editor / Tools
```

Tooling can inspect lower layers, but lower layers must not depend on the editor UI.

## 16. Runtime modes

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

## 17. Language selection gate

The final stack must be validated by building the same vertical slice in the strongest candidate combinations.

Minimum benchmark:

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
→ one cross-language tool call
```

Measure:

```text
build time
iteration time
memory
frame time
simulation time
job overhead
serialization
FFI / IPC overhead
debugging quality
tooling effort
binary size
startup time
code complexity
```

The benchmark must compare **architecture + language boundary cost**, not just synthetic CPU speed.

## 18. Architectural rule for final language lock

NEXORA should only lock the final implementation stack when all of the following are proven:

```text
✓ Core ownership is explicit
✓ Hot loops do not cross language boundaries
✓ RHI boundary is stable
✓ Serialization/versioning is stable
✓ Mod boundary is stable
✓ Editor/runtime boundary is stable
✓ Server/headless build works
✓ Job system works
✓ Streaming works
✓ Persistence works
✓ Benchmark results are acceptable
✓ Build/debug/tooling cost is sustainable
```

## 19. Current recommendation

The current preferred direction is:

```text
                 NEXORA
                   │
        ┌──────────┼──────────┐
        │          │          │
      RUST      NATIVE      TYPESCRIPT
        │       GFX/API         │
        │       layer        Editor
        │                     Tools
        │
   Core + Simulation
   World + Server
   Persistence
   Networking
   Mod Runtime

                   │
                PYTHON
                   │
        Research / Analysis
```

This preserves a coherent high-performance runtime while allowing specialized languages where they are genuinely stronger.

## 20. Architectural inspirations

NEXORA may study:

### Unreal Engine 5 lessons
- World Partition-style spatial streaming and hierarchical LOD;
- Large World Coordinates / explicit large-world precision;
- data-oriented Mass Entity for high population simulation;
- Gameplay Ability / Attribute / Effect separation;
- contextual input mapping;
- mature editor/tooling separation.

### Open 3D Engine lessons
- modular engine/component philosophy;
- asset pipeline;
- engine extensibility;
- editor/runtime separation;
- networking and profiling architecture.

### Data-oriented ECS lessons
- batch simulation;
- cache-friendly hot data;
- scalable populations;
- explicit system ownership;
- deterministic parallel processing where required.

### Godot-style separation lessons
- platform abstraction;
- subsystem boundaries;
- clean runtime drivers.

NEXORA implements its own architecture, code, APIs and data model.

## 21. IP rule

NEXORA may study public documentation for Unreal Engine, O3DE, Bevy, Godot and other projects for architectural lessons. It must not copy source code, proprietary assets or protected implementation details in ways that violate their licenses or copyright.

## 22. Final principle

> **One world, one authoritative simulation, multiple languages only where they create a real architectural advantage.**

The objective is not to build a multilingual project for its own sake. The objective is to keep the NEXORA runtime coherent and fast while allowing specialized tooling and native integration to use the most appropriate technology.
