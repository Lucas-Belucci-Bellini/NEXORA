# NEXORA — MAGIC SYSTEM

> Magic is a native extensibility domain for alternative rules of energy, matter, knowledge and interaction. It must use the same Registry, Command, Event, Security and Persistence foundations as all other systems.

## Core model
```text
Magic Source
→ Spell / Ritual / Effect
→ Cost / Conditions
→ Execution Request
→ Specialized Effect
→ World State Change
```

## Definitions
`MagicSchool`, `SpellDefinition`, `SpellInstance`, `RitualDefinition`, `ArcaneResource`, `MagicEffect`, `CasterProfile`, `MagicRule`.

## Sources
Arcane, elemental, dimensional, biological, spiritual or custom mod-defined sources. Sources are definitions, not assumptions about real-world claims.

## Costs
Energy, materials, time, cooldown, knowledge, environment, reagents and risk can be represented through public resource contracts.

## Effects
Teleportation, transformation, barriers, environmental changes, healing/status effects, summoning and other original gameplay effects should be implemented by specialized effect providers.

## Research
Magic can be discovered and formalized through Research/Knowledge. Failed experiments can create World Events without making Research dependent on Magic.

## Progression
Technology and Magic may be parallel or intersecting capability graphs.

## Safety / authority
Multiplayer magic is server-authoritative. Scripts and mods receive explicit capabilities and quotas. Magic must not bypass Security or command validation.

## API sketch
```ts
interface IMagicSystem {
  cast(request: SpellCastRequest): MagicResult;
  performRitual(request: RitualRequest): MagicResult;
  inspect(caster: EntityID): MagicState;
  registerSchool(definition: MagicSchoolDefinition): void;
}
```

## Integration
Item/Tool provide casting capabilities; Energy/Fluid can provide resources; Interaction creates requests; Command validates; Event Bus reports results; Renderer/Audio/Animation present effects.

## Persistence
Persist learned capabilities, cooldowns where durable, research state and magical artifacts through normal item/entity/save contracts.

## Tests
Cost validation, interruption, deterministic effects, multiplayer rejection, mod permissions and save/reload.

## Invariants
- Magic is not hardcoded into Core.
- A spell describes intent/effect; systems execute consequences.
- Untrusted scripts cannot bypass authority.
