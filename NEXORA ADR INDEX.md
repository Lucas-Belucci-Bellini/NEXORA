# NEXORA — ADR INDEX

Architecture Decision Records document decisões que possuem impacto duradouro e ajudam a explicar por que o projeto adotou determinado caminho.

## Quando criar ADR

Criar ADR para mudanças em:

```text
architecture
language / runtime
RHI
ECS / data model
threading
memory model
serialization
network protocol
save compatibility
mod API
scripting trust model
editor/runtime boundary
public IDs
security architecture
```

## Status

```text
PROPOSED
ACCEPTED
REJECTED
SUPERSEDED
DEPRECATED
```

## Campos mínimos

```text
context
problem
options
decision
consequences
migration
compatibility
```

## Regra

Uma implementação não deve contradizer uma ADR aceita sem que a ADR seja atualizada ou substituída por uma nova decisão formal.
