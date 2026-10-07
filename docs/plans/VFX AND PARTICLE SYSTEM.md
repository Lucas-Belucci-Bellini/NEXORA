# NEXORA — VFX / PARTICLE SYSTEM

> Visual effects represent transient visual phenomena produced from gameplay facts. VFX is presentation, never simulation authority.

## Model
```text
Gameplay / World Fact
→ VFX Event
→ Effect Definition
→ Emitter / System
→ GPU/CPU Simulation
→ Renderer
```

## Effects
Particles, trails, decals, environmental effects, impact effects, weather visuals, machine effects, magic visuals and dimensional/cosmic effects.

## Simulation levels
- cosmetic GPU effects;
- CPU effects when gameplay interaction is required;
- fully abstracted effects outside visual relevance.

## Pooling and budgets
Use effect pooling, lifetime limits, spawn budgets, distance culling, screen-size LOD and graceful degradation.

## API
```ts
interface IVFXSystem {
  register(definition: VFXDefinition): void;
  spawn(request: VFXRequest): VFXHandle;
  stop(handle: VFXHandle): void;
  setLOD(handle: VFXHandle, level: VFXLOD): void;
}
```

## Integration
Animation markers, Audio, World Events, Climate, Combat and Machines publish semantic facts. VFX decides how to visualize them.

## Networking
Replicate semantic effect triggers when necessary; never stream raw particle state for every client unless explicitly required.

## Tests
Pool exhaustion, high event rate, LOD transitions, replay determinism, resource streaming and headless-safe behavior.

## Invariants
- VFX cannot alter authoritative gameplay state.
- Effects are budgeted and degradable.
- GPU simulation is never required by the dedicated server.
