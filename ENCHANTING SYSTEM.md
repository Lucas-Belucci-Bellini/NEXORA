# NEXORA — ENCHANTING SYSTEM

> Enchanting is the item-capability layer for persistent magical modifiers. It does not replace Item, Magic, Crafting or Progression.

## Model
```text
Item
→ Enchanting Operation
→ Requirements / Cost
→ Modifier Set
→ Item State
```

## Concepts
`EnchantmentDefinition`, `EnchantmentInstance`, `EnchantingStation`, `Modifier`, `Affinity`, `CompatibilityRule`, `EnchantmentCost`.

## Modifiers
Examples: durability, efficiency, protection, special capabilities and original magical properties. Modifiers are data-driven and can be supplied by mods.

## Compatibility
An item can accept enchantments according to item tags, material, type, existing modifiers, skill, knowledge and technology/magic prerequisites.

## Quality
Enchantments can have level, stability, quality and resource cost. Avoid an infinite power curve by explicit caps and compatibility rules.

## Conflict
Definitions can declare incompatible pairs/groups. The registry validates these references.

## Execution
Enchanting changes Item state transactionally. Magic describes the magical process; Item System stores persistent modifiers.

## Research
New enchantments may be discovered through Research/Knowledge before becoming craftable.

## API sketch
```ts
interface IEnchantingSystem {
  evaluate(request: EnchantingRequest): EnchantingPreview;
  apply(request: EnchantingRequest): Result;
  remove(request: DisenchantingRequest): Result;
}
```

## Integration
Crafting provides station/process concepts; Item stores state; Magic supplies magical rules; Progression/Knowledge gates; UI presents preview; Persistence saves result.

## Security
Server validates costs, item identity, compatibility and transaction IDs. Client cannot invent modifiers.

## Tests
Compatibility, conflicting enchantments, failure, transactional rollback, save/reload and mod-defined enchantments.

## Invariants
- Enchantments are part of item state, not hidden global flags.
- Operations are transactional.
- Modifiers cannot bypass authority or item validation.
