# NEXORA — DEPENDENCY MATRIX

## Objetivo

Registrar quem depende de quem, evitando ciclos e acessos ilegais entre sistemas.

| Camada | Pode depender de | Não deve depender diretamente de |
|---|---|---|
| Platform / Drivers | OS, APIs nativas | Gameplay, Civilization |
| RHI | Platform, Foundation | NPC, Quest |
| Foundation | Platform abstractions | Gameplay-specific systems |
| Runtime Services | Foundation, public contracts | UI-specific implementation |
| Data Runtime / ECS | Foundation, Job, Resource | Narrative presentation |
| World | Foundation, Data Runtime, Resource | Editor-only code |
| Simulation | World, Data Runtime, Jobs | UI implementation |
| Gameplay | World, Simulation, public APIs | Platform driver internals |
| Society | Simulation, Gameplay contracts, History/Knowledge | Renderer internals |
| Presentation | Public state/read models, RHI | Direct mutation of authoritative state |
| Editor | Public runtime contracts, tools APIs | Private runtime internals |
| Mods | Public mod API | Private engine internals |

## Direção

```text
Platform
  ↓
Foundation
  ↓
Runtime
  ↓
Data / World
  ↓
Simulation
  ↓
Gameplay
  ↓
Society
  ↓
Presentation / Tools
```

## Regras

1. Dependência deve apontar para uma abstração estável, não para estado privado.
2. Sistemas inferiores não conhecem conceitos de gameplay superiores.
3. UI e editor não possuem autoridade sobre estado de simulação.
4. Mods usam APIs públicas e permissões explícitas.
5. Uma dependência entre módulos deve estar documentada.
6. Ciclos exigem refatoração ou uma interface de fronteira.

## Registro por sistema

Cada sistema deve possuir:

```text
OWNER
INPUTS
OUTPUTS
DEPENDS_ON
PUBLIC_API
COMMANDS
EVENTS
QUERIES
THREADING
PERSISTENCE
NETWORK
LOD
MOD_BOUNDARY
```

## Objetivo de revisão

A matriz deve ser atualizada sempre que um novo sistema criar uma dependência entre camadas ou introduzir uma nova API transversal.
