# NEXORA — TECHNICAL DEBT REGISTER

## Objetivo

Registrar compromissos técnicos conscientes para impedir que atalhos temporários se tornem arquitetura permanente.

## Registro mínimo

```text
ID
TITLE
SYSTEM
WHY CREATED
IMPACT
RISK
OWNER
PROPOSED REMEDIATION
TARGET STAGE
STATUS
```

## Classes

```text
TEMPORARY
PERFORMANCE
ARCHITECTURAL
COMPATIBILITY
TOOLING
TEST
SECURITY
CONTENT
```

## Regras

1. Todo workaround importante deve possuir ID.
2. “Temporário” sem plano de remoção é debt permanente até prova em contrário.
3. Débito que ameaça save, networking, security ou public APIs recebe prioridade alta.
4. Fechar debt exige teste que prove a correção.

## Status

```text
OPEN
PLANNED
IN PROGRESS
BLOCKED
RESOLVED
WONT FIX
```
