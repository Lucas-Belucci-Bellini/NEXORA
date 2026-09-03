# NEXORA

## MASTER PLAN — V1.0

> **NEXORA** é um sandbox voxel independente, modular e extensível, projetado desde o primeiro dia para possuir um sistema de conteúdo/mods nativo.
>
> A arquitetura completa está centralizada em [`NEXORA MASTER ARCHITECTURE.md`](./NEXORA%20MASTER%20ARCHITECTURE.md). Os documentos individuais definem responsabilidades locais, APIs, persistência, segurança, LOD, testes e integração.

## Arquitetura atual

```text
FOUNDATION
Core · Registry · Event Bus · Command · Persistence · Security
Job/Task · Versioning/Migration · Diagnostics

RUNTIME
Resource/Asset · Streaming · Performance · Navigation/Pathfinding
Content Pipeline · Mod Runtime · Scripting · Networking · Server

WORLD
Voxel · WorldGen · Biome · Cave/Deep World · Climate · Water
Vegetation · Lighting · Structures · Dimensions · Space · World Events

SIMULATION
Entity · Physics · AI/Perception · Vehicles · Interaction
Machines · Energy · Fluid · Automation · Railway

GAMEPLAY
Player · Item · Inventory · Tools/Weapons · Crafting · Combat
Agriculture · Fishing · Hunting/Wildlife · Magic · Enchanting
Quest · Progression/Technology · Achievements

SOCIETY
Social/Factions · Economy · Advanced Industry · Advanced Civilization
Research/Knowledge · Communication · Sensors · Cartography

PRESENTATION
Renderer · Animation · Audio · UI · Menus · Localization · Accessibility
Trading UI · Civilization UI · Research UI · Social UI

MODDING / TOOLS
Mod API · Mod Runtime · Mod SDK · Resource Packs · Content Pipeline
World/Structure Editors · Tooling

PLATFORM
Launcher · Installer/Updater · Mod Distribution · Dedicated Server
```

## Princípio

```text
ENGINE
  ↓
PUBLIC APIs
  ↓
REGISTRIES / EVENTS / COMMANDS
  ↓
SYSTEMS
  ↓
OFFICIAL CONTENT + COMMUNITY MODS
```

## Estado do projeto

A arquitetura conceitual foi expandida para cobrir os principais sistemas previstos. O próximo passo é sair do planejamento isolado de sistemas e começar a implementar a fundação em ordem de dependência, usando o Master Architecture como referência única.

## Regras

- Core pequeno e estável.
- Conteúdo oficial usa as mesmas APIs públicas que mods sempre que possível.
- Cliente não possui autoridade sobre o mundo em multiplayer.
- Estado autoritativo pertence ao sistema responsável.
- Event Bus comunica fatos/sinais; Command representa intenção.
- Persistence preserva estado; Streaming controla residência; LOD controla representação.
- Sistemas devem possuir testes, limites de recursos, telemetria local e políticas de recuperação.
