# NEXORA — MAGIC SYSTEM

> Magic is an extensible gameplay domain for alternative rules of energy, matter, knowledge and interaction. It uses normal Registry, Command, Event, Security and Persistence contracts.

## Model
`Source → Spell/Ritual → Conditions/Cost → Request → Effect → World State Change`.

## Concepts
`MagicSchool`, `SpellDefinition`, `SpellInstance`, `RitualDefinition`, `ArcaneResource`, `MagicEffect`, `CasterProfile`, `MagicRule`.

## Rules
Costs may use Energy, Fluid, items, time, knowledge, environment or other registered resources. Effects are implemented by specialized providers.

## Integration
Interaction creates requests; Command validates; Item/Tool provides capability; Progression/Research gate access; Event Bus reports results; Renderer/Audio/Animation present effects.

## Security
Multiplayer is server-authoritative. Mods/scripts receive explicit capabilities and quotas.

## API
```ts
interface IMagicSystem {
  cast(request: SpellCastRequest): MagicResult;
  performRitual(request: RitualRequest): MagicResult;
  inspect(caster: EntityID): MagicState;
}
```

## Tests
Costs, interruption, deterministic effects, multiplayer validation, mod permissions and save/reload.
