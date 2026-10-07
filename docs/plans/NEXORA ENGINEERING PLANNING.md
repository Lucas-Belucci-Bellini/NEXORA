# NEXORA — ENGINEERING PLANNING

> Documento-mestre para transformar a arquitetura do NEXORA em um plano de engenharia executável. A arquitetura define o que existe; este documento define como o projeto será construído, validado, medido e mantido.

## 1. Objetivo

NEXORA deve ser desenvolvido como um engine/runtime modular, orientado a dados, com mundo persistente e simulação capaz de continuar sem o jogador.

O planejamento deve impedir que a escala do projeto gere dependências circulares, ownership ambíguo, dados duplicados, concorrência insegura, saves incompatíveis ou ferramentas separadas do runtime.

## 2. Regra de transição

```text
ARQUITETURA
→ CONTRATOS
→ OWNERSHIP
→ THREADING
→ MEMÓRIA
→ SERIALIZAÇÃO
→ TESTES
→ TOOLING
→ BENCHMARK
→ IMPLEMENTAÇÃO
→ CONTEÚDO
```

Não iniciar implementação de grande escala enquanto contratos críticos ainda estiverem indefinidos.

## 3. Workstreams

```text
A — Architecture Governance
B — Core Runtime
C — Data / ECS / Simulation
D — World / Voxel / Streaming
E — Rendering / Platform
F — Persistence / Versioning
G — Networking / Server
H — AI / Civilization / History
I — Gameplay
J — Modding / Scripting
K — Editor / Tooling
L — Testing / Benchmark / QA
M — Build / CI / Distribution
N — Content
```

## 4. Definition of Done de arquitetura

Um sistema só é considerado pronto para implementação quando define:

- responsabilidade;
- não-responsabilidades;
- owner de cada dado;
- estado e lifecycle;
- API pública;
- comandos/eventos;
- threading;
- memória/recursos;
- persistência;
- rede/autoridade;
- segurança;
- LOD;
- mod/script boundary;
- observabilidade;
- testes;
- vertical slice.

## 5. Ordem de execução

### Stage 0 — Freeze de contratos
Arquitetura, nomenclatura, IDs, ownership, threading, memória, serialização, autoridade e fronteiras de linguagem.

### Stage 1 — Foundation
Core, Module System, Job System, Resource System, Time, Spatial, Registry, Event Bus, Command System, Diagnostics e Configuration.

### Stage 2 — Minimal Runtime
RHI, janela, input, camera, renderer mínimo, filesystem, asset loading e headless runtime.

### Stage 3 — World Vertical Slice
Voxel chunk, mesh, player, streaming, save/load, 1.000 entidades e jobs paralelos.

### Stage 4 — Living Simulation
AI, needs, perception, knowledge, economy, settlements, population, civilization, world events e history.

### Stage 5 — Gameplay
Items, inventory, crafting, combat, machines, energy, fluids, agriculture, vehicles e demais sistemas.

### Stage 6 — Multiplayer / Server
Servidor dedicado, replication, security, interest management, persistence e recovery.

### Stage 7 — Modding / Scripting
Data-only, sandboxed scripting, APIs, permissions, SDK, compatibility e distribution.

### Stage 8 — Editor / Toolchain
World editor, structure editor, entity inspector, history viewer, profiler, asset tools e mod tools.

### Stage 9 — Production Hardening
Stress, soak, migration, fuzzing, determinism, recovery, compatibility, packaging e release.

## 6. Vertical slice obrigatório

```text
Boot
→ Window / Headless
→ RHI
→ Camera
→ Input
→ 16³ Voxel Chunk
→ Mesh
→ World
→ Entity
→ Physics
→ 1,000 Entities
→ Job System
→ Streaming
→ Save / Load
→ Server
→ Mod Boundary
→ Diagnostics
```

O slice deve funcionar antes da expansão de conteúdo.

## 7. Critério de escala

O planejamento deve validar progressivamente:

```text
1 chunk
→ 16 chunks
→ 1 region
→ many regions
→ 1k entities
→ 10k active entities
→ large abstract population
→ civilization simulation
→ long-duration simulation
```

## 8. Princípio operacional

Sistemas devem ser implementados como módulos independentes, com contracts explícitos e integração por APIs, commands, events e queries.

Nunca introduzir dependência “temporária” sem registrar a exceção e o plano de remoção.

## 9. Gate antes de linguagem final

A stack de implementação só pode ser congelada depois de benchmark comparável e análise de custo de memória, concorrência, tooling, graphics, modding, debugging e manutenção.

## 10. Resultado esperado

O objetivo não é apenas “ter muitos sistemas documentados”. O objetivo é possuir uma arquitetura que possa ser implementada incrementalmente sem perder coerência quando o NEXORA crescer de protótipo para engine de produção.
