# NEXORA — DOCUMENTATION INDEX

## Objetivo

Mapear a documentação do projeto e distinguir documentos de arquitetura, sistemas, engenharia, conteúdo, arte, decisões e governança.

## Hierarquia de autoridade

```text
NEXORA MASTER ARCHITECTURE
        ↓
ARCHITECTURE / ENGINEERING RULES
        ↓
SYSTEM SPECIFICATIONS
        ↓
IMPLEMENTATION DOCS
        ↓
EXPERIMENTS / NOTES
```

Em conflito, um documento inferior não substitui silenciosamente uma regra superior.

## Núcleo arquitetural

- NEXORA MASTER ARCHITECTURE.md
- ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md
- NEXORA DEPENDENCY MATRIX.md
- NEXORA DATA OWNERSHIP AND SOURCE OF TRUTH.md
- NEXORA PUBLIC API AND CONTRACTS.md
- NEXORA THREADING AND CONCURRENCY MODEL.md
- NEXORA MEMORY AND RESOURCE OWNERSHIP.md
- NEXORA LANGUAGE AND FFI BOUNDARY.md
- NEXORA ARCHITECTURE FREEZE CHECKLIST.md

## Engenharia

- NEXORA ENGINEERING PLANNING.md
- NEXORA DEVELOPMENT ROADMAP.md
- NEXORA RUNTIME LIFECYCLE.md
- NEXORA WORLD STATE LIFECYCLE.md
- NEXORA PERFORMANCE BUDGETS.md
- NEXORA FAILURE AND RECOVERY ARCHITECTURE.md
- NEXORA DATA VALIDATION AND INVARIANTS.md
- NEXORA SAVE FORMAT AND COMPATIBILITY.md
- NEXORA REPLAY AND DETERMINISM.md
- NEXORA TESTING AND VALIDATION STRATEGY.md
- NEXORA CONTENT PIPELINE SPECIFICATION.md
- NEXORA TOOLING AND DEVELOPER EXPERIENCE.md
- NEXORA BUILD CI AND RELEASE ARCHITECTURE.md
- NEXORA OBSERVABILITY AND DEBUGGING.md
- NEXORA OBSERVABILITY DATA MODEL.md
- NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md
- NEXORA AI DECISION ARCHITECTURE.md
- NEXORA SECURITY THREAT MODEL.md
- NEXORA MOD COMPATIBILITY AND API VERSIONING.md
- NEXORA TECHNOLOGY BENCHMARK PLAN.md

## Conteúdo, arte e proveniência

- NEXORA ORIGINAL CONTENT AND ASSET POLICY.md
- NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md
- NEXORA ART DIRECTION AND PROCEDURAL VARIATION.md

## História e mundo vivo

- WORLD EVENTS SYSTEM.md
- HISTORY SYSTEM.md
- KNOWLEDGE AND INFORMATION SYSTEM.md
- LORE SYSTEM.md
- ARCHIVE AND HISTORICAL EVIDENCE SYSTEM.md
- WORLD CONTINUITY AND PLAYER INDEPENDENCE.md
- HISTORY AND LORE MASTER PLAN.md

## Processo e governança

- NEXORA ARCHITECTURE RULES.md
- NEXORA ADR INDEX.md
- NEXORA DEFINITION OF DONE.md
- NEXORA CHANGE MANAGEMENT.md
- NEXORA TECHNICAL DEBT REGISTER.md
- NEXORA NAMING AND TERMINOLOGY.md
- NEXORA DEVELOPER WORKFLOW AND CHANGE PROCESS.md

## Sistemas existentes

O repositório também contém as especificações individuais de Core, World, Voxel, Biomes, Caves, Climate, Water/Fluid, Vegetation, Physics, Lighting, Renderer/Graphics, Player, Combat, Tools/Weapons, Crafting, Machines, Energy, Inventory, Items, Entities, Registry, Event Bus, Persistence, Animation, Audio, UI, Dimensions, Structures, Networking, Server, Mod Runtime, Scripting, Commands, Security, Progression, Quest, Social/Factions, Research/Knowledge, Space, Vehicles, World Events, Industry, Civilization e demais sistemas do projeto.

## Implementação

A Phase 0 existe como código. O mapa entre documento normativo e crate:

| Camada | Crate | Documentos que implementa |
| --- | --- | --- |
| Foundation | `engine/foundation` | CORE.md §14–§15, NAMING AND TERMINOLOGY, SPATIAL AND COORDINATE SYSTEM, TIME AND CALENDAR SYSTEM, WORLD GENERATION SEED AND REPRODUCIBILITY, DIAGNOSTICS AND OBSERVABILITY, OBSERVABILITY DATA MODEL, CONFIGURATION AND SETTINGS SYSTEM |
| Persistence | `engine/persistence` | SAVE FORMAT AND COMPATIBILITY, SECURITY THREAT MODEL (entrada não confiável) |
| Runtime | `engine/runtime` | RUNTIME LIFECYCLE, ENGINE MODULE SYSTEM, Registry System, Event Bus, JOB SYSTEM, THREADING AND CONCURRENCY MODEL |
| World | `engine/world` | CHUNK & VOXEL ENGINE, WORLD STATE LIFECYCLE, WORLD GENERATION |
| Entity | `engine/entity` | Entity System, ECS AND DATA ORIENTED RUNTIME (identidade pública; layout é detalhe) |
| Physics | `engine/physics` | PHYSICS (PHY-0 a PHY-14 e PHY-22), REPLAY AND DETERMINISM (passo fixo) |
| Streaming | `engine/streaming` | STREAMING SYSTEM, WORLD CONTINUITY AND PLAYER INDEPENDENCE §18 (identidade sobrevive ao despejo) |
| Simulation | `engine/simulation` | DEPENDENCY MATRIX (o único ponto de composição entre mundo, física e streaming) |
| Benchmark | `engine/benchmark` | TECHNOLOGY BENCHMARK PLAN, PERFORMANCE BUDGETS (medição, não imposição) |
| Slice | `engine/headless` | DEFINITION OF DONE (verificação executável), TESTING AND VALIDATION STRATEGY |

Decisões que sustentam esse código estão em [`docs/adr/`](docs/adr/). O estado
real de cada contrato — definido versus construído — está em
[`NEXORA ARCHITECTURE FREEZE CHECKLIST.md`](NEXORA%20ARCHITECTURE%20FREEZE%20CHECKLIST.md).

Regra em vigor: **um arquivo de código nomeia o documento cujo contrato
implementa.** Quando os dois divergem, o documento vence até que um ADR diga o
contrário.

## Regra de atualização

Este índice deve ser atualizado quando:
- um novo documento se torna normativo;
- uma fronteira arquitetural muda;
- um sistema novo ganha especificação própria;
- um documento é substituído, descontinuado ou renomeado.

## Status

Este é um documento vivo. A ausência de um tópico no índice não significa que o sistema não exista; significa que a documentação de navegação precisa ser atualizada.
