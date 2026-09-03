# NEXORA — SOCIAL / FACTION UI

> Social UI presents relationships, factions, reputation, diplomacy and governance state. Social/Factions remain authoritative.

## Views
Relationships, groups, factions, reputation, roles, elections, diplomacy, treaties, information and history.

## Flow
`Snapshot → ViewModel → User Intent → Command → Social/Faction System → Event → UI update`.

## Rules
Reputation is contextual, trust differs from sentiment, and hidden/uncertain information stays hidden/uncertain.

## API
```ts
interface ISocialUI {
  open(actor: ActorID): void;
  snapshot(scope: SocialScope): SocialViewModel;
  dispatch(action: SocialUIAction): void;
}
```

## Integration
Civilization, Economy, Communication, Quest, World Events, Localization and Accessibility.

## Tests
Stale diplomacy state, permissions, hidden information and failed membership actions.
