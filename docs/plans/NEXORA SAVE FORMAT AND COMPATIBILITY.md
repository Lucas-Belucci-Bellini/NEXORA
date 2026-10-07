# NEXORA — SAVE FORMAT AND COMPATIBILITY

## Objetivo

Garantir saves duráveis, recuperáveis e migráveis durante anos de evolução do engine.

## Camadas

```text
SAVE CONTAINER
├── metadata
├── world header
├── dimension metadata
├── persistent entities
├── world state
├── civilization state
├── history
├── knowledge / archives
├── mod state
└── journal / recovery data
```

## Versionamento

Manter pelo menos:

```text
engine_version
save_format_version
world_schema_version
content_version
mod_schema_versions
```

## Snapshot + Journal

```text
State Snapshot
      +
Incremental Journal
      ↓
Recovery
```

Snapshots reduzem custo de recuperação; journal registra mudanças entre checkpoints.

## Atomicidade

Escrita deve usar estratégia que permita:

```text
write temporary
→ validate
→ commit / replace
→ fsync when required
```

Nunca substituir um save válido por dados parcialmente escritos.

## Corruption

```text
detect
→ quarantine
→ recover latest valid snapshot/journal
→ report diagnostics
```

## Migration

Mudança de schema exige migration explícita ou rejeição segura. Não executar conversão silenciosa sem registrar versão de origem/destino.

## Compatibilidade

Definir políticas:

```text
READ CURRENT
READ LEGACY
MIGRATE
FORWARD COMPATIBILITY
BREAKING CHANGE
```

## Mods

Save deve registrar dependências de mods e suas versões. Ausência de conteúdo deve gerar estado Missing Content controlado, nunca corrupção silenciosa.

## Determinismo

Save deve preservar estado necessário para continuar a simulação sem depender de caches ou dados derivados.

## Testes obrigatórios

- save/load round trip;
- crash durante save;
- corrupção parcial;
- migration de versões;
- mods removidos/adicionados;
- mundo em diferentes LODs;
- grandes saves;
- recuperação de journal.
