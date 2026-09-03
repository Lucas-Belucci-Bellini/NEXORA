# NEXORA — LOCALIZATION SYSTEM

> Localization turns stable text keys into language-specific presentation while keeping gameplay state language-independent.

## Core
`Locale`, `TranslationBundle`, `StringKey`, `PluralRule`, `FormatContext`, `FallbackChain`.

## Rules
Use stable keys such as `nexora.item.stone.name`. Never store translated text as authoritative game state.

## Fallback
```text
requested locale
→ language fallback
→ default locale
→ developer key
```

## Formatting
Support variables, pluralization, gender where applicable, dates, numbers, units and rich text markers using typed format data.

## Mods
Mods can ship localization bundles under their namespace. Missing translations should degrade to a safe fallback without breaking content.

## Dynamic content
NPC names, generated settlement names and procedural text can use localized templates and deterministic parameters.

## API sketch
```ts
interface ILocalizationSystem {
  setLocale(locale: LocaleID): void;
  translate(key: StringKey, args?: FormatArgs): LocalizedText;
  load(bundle: TranslationBundle): void;
  validate(bundle: TranslationBundle): ValidationReport;
}
```

## UI integration
UI consumes localized text. Quest, item, civilization and debug systems expose keys/data, not baked strings.

## Tests
Fallback, plural rules, missing keys, mod locales, formatting and locale switching without changing authoritative state.

## Invariants
- Localization never changes gameplay logic.
- Keys are stable across releases; migrations handle renames.
- Missing translation does not invalidate a world.
