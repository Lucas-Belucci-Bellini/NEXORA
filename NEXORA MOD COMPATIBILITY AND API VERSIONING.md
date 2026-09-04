# NEXORA — MOD COMPATIBILITY AND API VERSIONING

## Objetivo

Permitir evolução do engine sem quebrar silenciosamente mods e conteúdo de terceiros.

## API layers

```text
ENGINE INTERNAL
PUBLIC ENGINE API
MOD API
SCRIPT API
CONTENT SCHEMA
```

Apenas APIs marcadas como públicas e estáveis podem ser usadas como contrato de mod.

## Versioning

Toda API pública possui versão. Mudanças breaking exigem nova versão ou migration.

```text
CURRENT
DEPRECATED
COMPATIBILITY
REMOVED
```

## Mod manifest

Deve declarar:

```text
mod_id
mod_version
engine_api_range
content_schema_range
dependencies
optional_dependencies
permissions
```

## Compatibility resolution

```text
DISCOVER
→ VALIDATE
→ RESOLVE DEPENDENCIES
→ CHECK API RANGE
→ CHECK PERMISSIONS
→ LOAD
```

## Missing / incompatible mod

O engine deve falhar de forma explícita e diagnóstica, preservando o save quando possível.

## Native mods

Trusted native extensions seguem contrato ABI documentado e sandbox/trust policy apropriada.

## Scripting

Script API pode evoluir independentemente da API nativa, com capability versions e migrations.

## Rule

Nenhum mod deve precisar depender de detalhes internos não documentados para funcionar.
