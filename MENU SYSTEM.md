# NEXORA — MENU / SCREEN SYSTEM

> Menus are UI screens and flows. They collect player intent and call Commands/Queries; they never own authoritative gameplay state.

## Screens
```text
Boot
Main Menu
World Selection
World Creation
Pause
Settings
Controls
Graphics
Audio
Accessibility
Mod Management
Server Browser
Character / Profile
```

## Navigation model
`Screen`, `ScreenStack`, `Route`, `Modal`, `ViewModel`, `UICommand`.

## Flow
```text
User input
→ UI action
→ command/query
→ system
→ event/state update
→ ViewModel
→ UI
```

## Persistence
Store local preferences and screen-safe settings through Config. World state remains in Persistence/Server.

## Multiplayer
Server Browser presents server metadata; connecting invokes Networking. Menus never fabricate connection authority.

## Mod screens
Mods register screens through UI/Registry contracts with permissions and namespace ownership.

## API sketch
```ts
interface IMenuSystem {
  open(screen: ScreenID): void;
  close(screen: ScreenID): void;
  push(route: UIRoute): void;
  pop(): void;
}
```

## Accessibility
All menus use Localization and Accessibility semantic actions.

## Tests
Navigation stack, failed world creation, reconnect flow, settings persistence and modal cancellation.

## Invariants
- Menus do not mutate simulation directly.
- Destructive actions require explicit confirmation/authority.
