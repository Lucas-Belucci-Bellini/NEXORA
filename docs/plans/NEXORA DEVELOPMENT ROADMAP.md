# NEXORA — DEVELOPMENT ROADMAP

## Objetivo

Transformar a arquitetura em uma sequência de entregas verificáveis. O roadmap é orientativo; uma fase só avança quando seus critérios técnicos forem satisfeitos.

## Phase 0 — Architecture Freeze

```text
master architecture
contracts
ownership
dependencies
threading
memory
serialization
authority
RHI
mod boundary
benchmark plan
```

Exit: nenhum blocker arquitetural crítico sem decisão registrada.

## Phase 1 — Engine Bootstrap

Core, module lifecycle, jobs, resources, time, spatial, registry, events, commands, diagnostics, config e abstrações de plataforma.

Exit: runtime mínimo inicia em modo client/headless e testes básicos passam.

## Phase 2 — Voxel Vertical Slice

Chunk, voxel storage, meshing, world coordinates, camera, input, render mínimo, player e streaming inicial.

Exit: mundo pequeno navegável e carregamento/unloading estável.

## Phase 3 — Persistence + Simulation

Save/load, snapshot, journal, ECS/data runtime, physics, AI básica, 1.000 entidades e jobs paralelos.

Exit: save round-trip, recovery e benchmark reproduzíveis.

## Phase 4 — Living World

Population, settlements, economy, civilization, knowledge, communication, world events, history, archives e lore.

Exit: simulação consegue produzir cadeia causal e continuar sem o jogador em LOD apropriado.

## Phase 5 — Gameplay Expansion

Items, inventory, crafting, combat, machines, energy, fluids, agriculture, vehicles, railway e demais sistemas.

Exit: sistemas possuem integração, persistência e testes, não apenas protótipos isolados.

## Phase 6 — Multiplayer / Server

Server authority, replication, interest management, session lifecycle, anti-duplication, recovery e observability.

Exit: sessão básica cliente-servidor estável e testável.

## Phase 7 — Modding / Scripting

Public APIs, data-only mods, sandboxed scripting, permissions, API versioning, SDK e compatibility.

Exit: mod de referência produzido apenas através das APIs públicas.

## Phase 8 — Editor / Toolchain

World/structure editors, inspectors, profiler, save inspector, replay debugger, asset tooling e mod tooling.

Exit: fluxo básico de criação → validação → runtime sem modelo paralelo incompatível.

## Phase 9 — Scale / Hardening

10k active entities, large abstract populations, long simulations, stress, soak, fuzz, migration, determinism e large saves.

Exit: budgets, regressions e recovery conhecidos e monitorados.

## Phase 10 — Alpha

Conteúdo inicial, onboarding, UX, accessibility, packaging e telemetry de produção controlada.

## Phase 11 — Beta

Compatibilidade, performance, multiplayer, mods, saves, content completeness e bug reduction.

## Phase 12 — Release

Release artifacts, checksums, signatures quando aplicável, documentation, support policy e versioned compatibility.

## Regra do roadmap

Conteúdo não deve ultrapassar a maturidade das fundações a ponto de bloquear correções estruturais. Um protótipo pode existir antes do sistema completo, mas deve permanecer explicitamente experimental.
