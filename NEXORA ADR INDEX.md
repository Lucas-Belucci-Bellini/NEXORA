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
| [0002](docs/adr/ADR-0002-zero-dependency-foundation.md) | Zero dependências externas nos crates do engine | ACCEPTED (emendada pela 0026) |
| [0003](docs/adr/ADR-0003-workspace-layout-enforces-dependency-matrix.md) | O layout do workspace faz o build recusar violações da matriz de dependências | ACCEPTED |
| [0004](docs/adr/ADR-0004-save-container-format-v1.md) | Formato do contêiner de save v1 | ACCEPTED |
| [0005](docs/adr/ADR-0005-phase-0-scope-boundary.md) | Fronteira de escopo da Phase 0 — o que **não** foi implementado, e por quê | ACCEPTED (emendada pelas 0006, 0007, 0008 e 0025) |
| [0006](docs/adr/ADR-0006-entity-identity-and-storage.md) | Identidade de entidade é API pública; layout de armazenamento não é | ACCEPTED |
| [0007](docs/adr/ADR-0007-physics-collides-against-a-provider-not-the-world.md) | Física colide contra um provedor, não contra o mundo | ACCEPTED |
| [0008](docs/adr/ADR-0008-streaming-decides-residency-and-a-backend-provides-it.md) | Streaming decide residência; um backend a executa | ACCEPTED |
| [0009](docs/adr/ADR-0009-a-second-stack-measures-kernels-not-an-engine.md) | Uma segunda stack mede kernels, não um motor — e uma crate pode dizer `unsafe` | ACCEPTED |
| [0010](docs/adr/ADR-0010-commands-are-intent-and-carry-their-own-authority.md) | Comandos são intenção, e a fronteira é imposta pelo grafo de crates | ACCEPTED |
| [0011](docs/adr/ADR-0011-a-torn-tail-is-a-crash-and-corruption-is-not.md) | Cauda truncada é crash; corrupção não é, e cada uma tem sua resposta | ACCEPTED |
| [0012](docs/adr/ADR-0012-a-mesh-is-a-data-structure-not-a-picture.md) | Uma malha é uma estrutura de dados, não uma imagem | ACCEPTED |
| [0013](docs/adr/ADR-0013-recovery-finishes-when-a-column-arrives.md) | Recuperação não é um instante: termina quando a coluna chega | ACCEPTED |
| [0014](docs/adr/ADR-0014-a-region-file-is-authoritative-for-its-region.md) | Um arquivo de região é autoritativo para a sua região | ACCEPTED |
| [0015](docs/adr/ADR-0015-a-spatial-index-is-a-loose-grid-and-a-query-may-decline-it.md) | O índice espacial é uma grade frouxa, e uma consulta pode recusá-lo | ACCEPTED |
| [0016](docs/adr/ADR-0016-a-job-result-can-be-forgotten-and-says-so.md) | Um resultado de job pode ser esquecido, e o pool diz isso | ACCEPTED |
| [0017](docs/adr/ADR-0017-a-frame-is-time-the-host-hands-in.md) | Um quadro é tempo que o host entrega, e os estágios prestam contas dele | ACCEPTED |
| [0018](docs/adr/ADR-0018-input-is-intent-the-host-hands-in.md) | Input é intenção que o host entrega, e um contexto consome a fonte | ACCEPTED |
| [0019](docs/adr/ADR-0019-a-recipe-is-data-and-a-material-names-it.md) | Uma receita é dado, e um material a nomeia — schema 3 do documento de material | ACCEPTED |
| [0020](docs/adr/ADR-0020-content-blocks-enter-through-the-api-a-mod-uses.md) | Blocos de conteúdo entram pela mesma API que um mod usa | ACCEPTED |
| [0021](docs/adr/ADR-0021-resources-are-verified-before-they-are-decoded.md) | Recursos são indexados e verificados antes de serem decodificados | ACCEPTED |
| [0022](docs/adr/ADR-0022-one-png-decoder-and-it-lives-in-the-engine.md) | Um único decodificador PNG, e ele mora no engine | ACCEPTED |
| [0023](docs/adr/ADR-0023-queries-are-reads-and-a-contract.md) | Queries são leituras, e um contrato | ACCEPTED |
| [0024](docs/adr/ADR-0024-memory-is-accounted-by-its-owner-in-one-ledger.md) | Memória é contabilizada pelo dono, num livro-razão só | ACCEPTED |
| [0025](docs/adr/ADR-0025-the-rhi-is-a-contract-a-null-backend-keeps-before-a-gpu-does.md) | O RHI é um contrato que um backend nulo cumpre antes de uma GPU | ACCEPTED |
| [0026](docs/adr/ADR-0026-the-first-native-backend-is-wgpu-and-ci-runs-it.md) | O primeiro backend nativo é wgpu, e o CI o executa | ACCEPTED |
| [0027](docs/adr/ADR-0027-the-window-host-is-winit-and-it-owns-the-event-loop.md) | O host de janela é o winit, e ele é dono do laço de eventos | ACCEPTED |
