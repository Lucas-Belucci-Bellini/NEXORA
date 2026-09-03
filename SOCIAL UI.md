# NEXORA — SOCIAL / FACTION UI

> Social UI presents relationships, factions, reputation, diplomacy and governance state. Social/Factions remain authoritative.

## Views
```text
Relationships
Groups
Factions
Reputation
Roles
Elections
Diplomacy
Treaties
Rumors / Information
History
```

## Flow
```text
Social Snapshot
→ ViewModel
→ User intent
→ Command
→ Social/Faction System
→ Event
→ UI update
```

## Context
Reputation is contextual, trust is distinct from sentiment, and information can be uncertain or delayed.

## Actions
Offer alliance, join group, negotiate, vote, resign, accept treaty or inspect relationship. Actions require normal permissions/authority.

## Integration
Civilization supplies macro institutions; Economy supplies trade context; Communication carries information; Quest consumes social opportunities; World Events provide facts.

## API sketch
```ts
interface ISocialUI {
  open(actor: ActorID): void;
  snapshot(scope: SocialScope): SocialViewModel;
  dispatch(action: SocialUIAction): void;
}
```

## Tests
Stale diplomacy state, permission enforcement, hidden information, failed membership request and localization/accessibility.

## Invariants
- UI does not fabricate reputation or diplomatic facts.
- Hidden/uncertain information stays hidden/uncertain.
