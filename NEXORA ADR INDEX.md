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
| [0005](docs/adr/ADR-0005-phase-0-scope-boundary.md) | Fronteira de escopo da Phase 0 — o que **não** foi implementado, e por quê | ACCEPTED (emendada pelas 0006, 0007 e 0008) |
| [0006](docs/adr/ADR-0006-entity-identity-and-storage.md) | Identidade de entidade é API pública; layout de armazenamento não é | ACCEPTED |
| [0007](docs/adr/ADR-0007-physics-collides-against-a-provider-not-the-world.md) | Física colide contra um provedor, não contra o mundo | ACCEPTED |
| [0008](docs/adr/ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md) | Streaming decide residência; um backend a executa | ACCEPTED |
| [0009](docs/adr/ADR-0009-a-second-stack-measures-kernels-not-an-engine.md) | Uma segunda stack mede kernels, não um motor — e uma crate pode dizer `unsafe` | ACCEPTED |
| [0010](docs/adr/ADR-0010-commands-are-intent-and-carry-their-own-authority.md) | Comandos são intenção, e a fronteira é imposta pelo grafo de crates | ACCEPTED |
| [0011](docs/adr/ADR-0011-a-torn-tail-is-a-crash-and-corruption-is-not.md) | Cauda truncada é crash; corrupção não é, e cada uma tem sua resposta | ACCEPTED |
