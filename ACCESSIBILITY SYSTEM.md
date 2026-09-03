# NEXORA — ACCESSIBILITY SYSTEM

> Accessibility provides configurable presentation and input adaptations without changing authoritative simulation state.

## Areas
UI scale, input remapping, gamepad support, subtitles, visual alerts, audio cues, motion options, contrast/readability, navigation assistance and color-independent information.

## Architecture
Preferences are local presentation/input state. Gameplay exposes semantic actions and facts through UI/Audio contracts.

## Input
Support remapping, conflict detection, sensitivity, hold/toggle modes and alternative bindings.

## Audio/visual
Semantic world/gameplay facts can drive captions and cues; Audio and Renderer decide presentation.

## API
```ts
interface IAccessibilitySystem {
  getProfile(): AccessibilityProfile;
  setOption(option: AccessibilityOption, value: AccessibilityValue): void;
  resolveAction(action: SemanticAction): InputBinding[];
}
```

## Integration
UI, Localization, Audio, Animation and Input. Accessibility cannot weaken authority or security.

## Tests
Remap conflicts, subtitles, scalable UI, locale changes and preference persistence.

## Invariant
Accessibility settings never modify authoritative simulation rules.
