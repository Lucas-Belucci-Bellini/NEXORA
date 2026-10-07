# NEXORA — ATTRIBUTE / TAG / ABILITY SYSTEM

> NEXORA needs a generic capability/effect layer inspired by proven data-driven gameplay architecture, without copying any engine implementation. It provides reusable attributes, tags, abilities and effects across Player, NPC, mobs, vehicles and items.

## Separation
```text
Attribute → measurable state
Tag → semantic classification/state
Ability → executable capability/intent
Effect → state modification
```

## Attribute model
Base value, current value, modifiers, limits, provenance and replication policy. Examples: health, stamina, speed, resistance, machine efficiency.

## Tags
Hierarchical, namespaced, data-driven tags can describe state, category, permissions, compatibility and gameplay context.

## Abilities
Abilities express reusable actions and may coordinate Animation, Audio, Combat, Interaction, Energy, Fluid and Quest systems. They do not bypass Command or Server authority.

## Effects
Instant, duration-based or persistent effects. Effects are declarative where possible and may modify attributes/tags through validated APIs.

## API
```ts
interface IAbilitySystem {
  grant(actor: ActorID, ability: AbilityID): Result;
  activate(request: AbilityRequest): Result;
  applyEffect(request: EffectRequest): Result;
  getAttributes(actor: ActorID): AttributeSnapshot;
  getTags(actor: ActorID): TagSet;
}
```

## Networking
High-level ability intent and authoritative effect results may replicate; clients cannot authoritatively invent effects or attributes.

## Persistence
Persistent modifiers and base attributes are saved through normal entity/item persistence rules.

## Modding
Mods may register attributes, tags, abilities, effects and formulas under owned namespaces.

## Tests
Modifier stacking, tag matching, cooldown/resource checks, rollback, replication, save/reload and deterministic calculations.

## Invariants
- No hidden authority escalation through abilities.
- Attribute calculation is bounded and deterministic.
- Tags are metadata/semantics, not arbitrary executable code.
