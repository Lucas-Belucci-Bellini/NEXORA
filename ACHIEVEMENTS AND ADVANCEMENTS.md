# NEXORA — ACHIEVEMENTS / ADVANCEMENTS

> Achievements and advancements record meaningful accomplishments without becoming the authoritative source of gameplay progression.

## Distinction
`Progression/Technology` controls capabilities. `Quest` controls objectives. `Achievement` records accomplishments. `Advancement` can present structured world/player milestones.

## Model
`AchievementDefinition`, `AchievementInstance`, `AdvancementDefinition`, `ProgressCondition`, `RewardReference`, `VisibilityPolicy`.

## Triggers
Prefer facts from Event Bus and queries over hardcoded polling:
```text
world event
→ condition
→ achievement evaluation
→ persistent result
```

## Scope
Player, party, faction, settlement, civilization and world achievements.

## Hidden achievements
Definitions may be secret while evaluation remains server-authoritative. Unlock should not leak hidden criteria unintentionally.

## Rewards
Rewards are references to existing item/progression/knowledge systems and must use normal transactions.

## Persistence
Unlock state is persistent and versioned. Definitions are registry content; instances belong to profiles/worlds.

## API sketch
```ts
interface IAchievementSystem {
  register(definition: AchievementDefinition): void;
  evaluate(facts: FactBatch): AchievementResult[];
  getProfile(actor: ActorID): AchievementSnapshot;
}
```

## Multiplayer
Server is authoritative for unlocks. Clients receive verified achievement facts.

## Tests
Duplicate trigger, reconnect, migration, hidden achievement, reward transaction and world-scope accomplishment.

## Invariants
- Achievement state cannot grant capabilities outside authorized progression/reward APIs.
- Unlocks are idempotent.
