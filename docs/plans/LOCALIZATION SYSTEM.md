# NEXORA — LOCALIZATION SYSTEM

> Converts stable text keys into locale-specific presentation without changing authoritative state.

## Core
`Locale`, `TranslationBundle`, `StringKey`, `PluralRule`, `FormatContext`, `FallbackChain`.

## Rules
Use stable keys such as `nexora.item.stone.name`. Never store translated text as gameplay state.

## Fallback
`Requested locale → language fallback → default locale → developer key`.

## Formatting
Support variables, pluralization, dates, numbers, units and safe rich-text markers.

## Mods
Mods ship namespaced translation bundles. Missing translations fall back safely.

## API
```ts
interface ILocalizationSystem {
  setLocale(locale: LocaleID): void;
  translate(key: StringKey, args?: FormatArgs): LocalizedText;
  load(bundle: TranslationBundle): void;
}
```

## Tests
Fallback, plural rules, missing keys, mod translations and locale switching.
