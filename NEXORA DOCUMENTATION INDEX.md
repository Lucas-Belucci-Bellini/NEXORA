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

## Regra de atualização

Este índice deve ser atualizado quando:
- um novo documento se torna normativo;
- uma fronteira arquitetural muda;
- um sistema novo ganha especificação própria;
- um documento é substituído, descontinuado ou renomeado.

## Status

Este é um documento vivo. A ausência de um tópico no índice não significa que o sistema não exista; significa que a documentação de navegação precisa ser atualizada.
