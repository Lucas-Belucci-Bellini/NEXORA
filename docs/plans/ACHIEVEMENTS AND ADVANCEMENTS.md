# NEXORA — ACHIEVEMENTS / ADVANCEMENTS

> Achievements record accomplishments; Progression controls capabilities and Quest controls objectives.

## Model
`AchievementDefinition`, `AchievementInstance`, `AdvancementDefinition`, `ProgressCondition`, `RewardReference`, `VisibilityPolicy`.

## Trigger
```text
Event / Fact
→ Condition
→ Evaluation
→ Persistent Unlock
```

## Scope
Player, party, faction, settlement, civilization and world.

## Rewards
Rewards reference existing Item, Progression, Knowledge or Quest APIs and use normal transactions.

## Multiplayer
Server-authoritative and idempotent. Hidden criteria remain hidden according to policy.

## API
```ts
interface IAchievementSystem {
  register(definition: AchievementDefinition): void;
  evaluate(facts: FactBatch): AchievementResult[];
  getProfile(actor: ActorID): AchievementSnapshot;
}
```

## Tests
Duplicate triggers, reconnect, migration, hidden achievement and reward atomicity.
