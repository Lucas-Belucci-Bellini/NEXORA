# NEXORA — MENU / SCREEN SYSTEM

> Menus present state and collect intent. They never own authoritative gameplay state.

## Screens
Boot, Main Menu, World Selection/Creation, Pause, Settings, Controls, Graphics, Audio, Accessibility, Mod Management and Server Browser.

## Flow
`Input → UI Action → Query/Command → Domain System → Event/State → ViewModel → UI`.

## Model
`Screen`, `ScreenStack`, `Route`, `Modal`, `ViewModel`, `UICommand`.

## Integration
UI, Localization, Accessibility, Networking, Server, Persistence and Config.

## Modding
Mods may register screens/widgets through public UI/Registry APIs.

## Tests
Navigation stack, failed world creation, reconnect, settings persistence and cancellation.
