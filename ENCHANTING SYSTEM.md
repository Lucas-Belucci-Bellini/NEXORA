# NEXORA — ENCHANTING SYSTEM

> Enchanting applies persistent magical modifiers to item state through validated transactions.

## Flow
`Item → Enchanting Operation → Requirements/Cost → Modifier Set → Item State`.

## Concepts
`EnchantmentDefinition`, `EnchantmentInstance`, `EnchantingStation`, `Modifier`, `CompatibilityRule`, `EnchantmentCost`.

## Rules
Compatibility uses item tags/material/type/existing modifiers and Magic/Knowledge/Technology requirements. Conflicting enchantments are explicit.

## Integration
Magic describes the magical process; Item stores persistent state; Crafting provides stations; Progression/Research gates capabilities; UI presents previews; Persistence saves results.

## API
```ts
interface IEnchantingSystem {
  evaluate(request: EnchantingRequest): EnchantingPreview;
  apply(request: EnchantingRequest): Result;
  remove(request: DisenchantingRequest): Result;
}
```

## Security
Server validates item identity, cost, compatibility and transaction ID.

## Tests
Compatibility, conflict, rollback, save/reload and mod-defined enchantments.
