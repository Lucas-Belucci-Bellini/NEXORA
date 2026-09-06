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

## Registro

Os ADRs vivem em [`docs/adr/`](docs/adr/). Índice completo em
[`docs/adr/README.md`](docs/adr/README.md).

| ADR | Título | Status |
| --- | --- | --- |
| [0001](docs/adr/ADR-0001-rust-phase-0-reference-implementation.md) | Rust como implementação de referência da Phase 0 — o gate de linguagem **continua aberto** | ACCEPTED |
| [0002](docs/adr/ADR-0002-zero-dependency-foundation.md) | Zero dependências externas nos crates do engine | ACCEPTED |
| [0003](docs/adr/ADR-0003-workspace-layout-enforces-dependency-matrix.md) | O layout do workspace faz o build recusar violações da matriz de dependências | ACCEPTED |
| [0004](docs/adr/ADR-0004-save-container-format-v1.md) | Formato do contêiner de save v1 | ACCEPTED |
| [0005](docs/adr/ADR-0005-phase-0-scope-boundary.md) | Fronteira de escopo da Phase 0 — o que **não** foi implementado, e por quê | ACCEPTED |
