# NEXORA — CHANGE MANAGEMENT

## Objetivo

Controlar mudanças que podem quebrar contratos, saves, mods, multiplayer ou ferramentas.

## Tipos

```text
PATCH
MINOR ARCHITECTURE CHANGE
BREAKING CHANGE
EXPERIMENT
DEPRECATION
MIGRATION
```

## Mudança arquitetural

Uma mudança que altera ownership, API pública, serialization, threading, authority, RHI, mod boundary ou dependency graph deve possuir ADR ou atualização normativa equivalente.

## Fluxo

```text
PROPOSAL
→ IMPACT ANALYSIS
→ DECISION
→ UPDATE DOCS
→ IMPLEMENT
→ TEST
→ MIGRATION
→ DEPRECATION / RELEASE NOTES
```

## Backward compatibility

Quebras de save, mods ou rede precisam de estratégia explícita antes da implementação.

## Experimentos

Experimentos devem ser claramente marcados e não criar dependências permanentes sem revisão.
