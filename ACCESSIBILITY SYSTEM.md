# NEXORA — ACCESSIBILITY SYSTEM

> Accessibility provides configurable presentation and input adaptations without changing authoritative simulation state.

## Areas
```text
UI scale
Input remapping
Gamepad support
Subtitles
Visual alerts
Audio cues
Motion options
Contrast / readability
Navigation assistance
Color-independent information
```

## Architecture
Accessibility preferences are local presentation/configuration state. Gameplay systems expose semantic information through UI/audio contracts.

## Input
Support remapping, conflict detection, sensitivity, hold/toggle modes and alternative bindings.

## Audio / visual
World and gameplay facts can have semantic cues. The Audio System and Renderer choose actual presentation.

## Motion
Allow configurable camera shake, motion effects and animation intensity where supported.

## Subtitles
Audio events can publish caption metadata such as speaker, category and direction without exposing raw audio.

## Mods
Mods may register accessibility-aware UI labels and semantic actions; they cannot disable platform-level safeguards.

## API sketch
```ts
interface IAccessibilitySystem {
  getProfile(): AccessibilityProfile;
  setOption(option: AccessibilityOption, value: AccessibilityValue): void;
  resolveAction(action: SemanticAction): InputBinding[];
}
```

## Tests
Input remap conflicts, locale changes, subtitle generation, scalable UI and persistence of local preferences.

## Invariants
- Accessibility settings are presentation/input settings.
- They do not weaken multiplayer authority or security validation.
