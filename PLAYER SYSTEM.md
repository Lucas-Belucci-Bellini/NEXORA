Sim. O **Player System** precisa ser separado do Inventory, Physics, Input e Character Controller. Ele é o sistema que representa o jogador como uma entidade persistente, com estado, progressão, interação e identidade dentro do mundo.

# NEXORA — PLAYER SYSTEM MASTER PLAN

A arquitetura principal:

```text id="p9k4x7"
                         PLAYER SYSTEM
                              │
       ┌──────────────────────┼──────────────────────┐
       │                      │                      │
    IDENTITY              CHARACTER              PROGRESSION
       │                      │                      │
    PlayerID              Movement               Skills
    Profile               Position               Experience
    Permissions           Stamina                Knowledge
    Settings              Health                 Unlocks
       │                      │                      │
       └──────────────────────┼──────────────────────┘
                              │
       ┌──────────────────────┼──────────────────────┐
       │                      │                      │
    INVENTORY              INTERACTION            CAMERA
       │                      │                      │
    Equipment              Building                First/Third
    Backpacks               Mining                  Spectator
    Loadouts                Use/Interact
       │                      │
       └──────────────────────┼──────────────────────┘
                              │
                     WORLD / SIMULATION
```

A regra principal:

> **Player System representa o jogador; outros sistemas representam as capacidades que o jogador utiliza.**

Então:

```text
Player
 ├── Inventory → Inventory System
 ├── Physics → Physics System
 ├── Input → Input System
 ├── Rendering → Renderer
 ├── Skills → Progression
 ├── Building → Build System
 └── Knowledge → Knowledge System
```

---

# 1. PLAYER-0 — Player Identity

Cada jogador precisa de identidade persistente:

```text id="2x8m5q"
PlayerID
ProfileID
AccountID
CharacterID
```

Separar:

```text
Account
```

de:

```text
Character
```

Assim uma conta pode eventualmente possuir mais de um personagem.

---

# 2. PLAYER-1 — Player Profile

```text id="7m4q9x"
PlayerProfile
├── playerId
├── displayName
├── characterId
├── preferences
├── progression
├── unlocks
└── metadata
```

Não colocar todo o save dentro desse objeto.

---

# 3. PLAYER-2 — Character State

O personagem terá:

```text id="x5m8q2"
CharacterState
├── position
├── rotation
├── velocity
├── health
├── stamina
├── status
├── inventoryRef
├── equipmentRef
└── progressionRef
```

---

# 4. PLAYER-3 — Spawn System

Definir onde o personagem começa.

```text id="m7q3x1"
SpawnPoint
├── dimension
├── position
├── rotation
└── rules
```

Possíveis fontes:

```text id="8x2m6q"
world spawn
settlement
bed/rest point
checkpoint
custom spawn
```

---

# 5. PLAYER-4 — Respawn

Fluxo:

```text id="q9m4x7"
Player incapacitated
 ↓
Respawn Rules
 ↓
select spawn
 ↓
restore state
 ↓
spawn
```

A regra de “morte” deve ser separada das consequências específicas de gameplay.

---

# 6. PLAYER-5 — Player State Machine

Estados:

```text id="4m8x2q"
SPAWNING
ACTIVE
SLEEPING
SWIMMING
CLIMBING
FALLING
RIDING
DOWNED
RESPAWNING
SPECTATING
DISCONNECTED
```

---

# 7. PLAYER-6 — Input Mapping

Input não deve estar hardcoded no Player.

```text id="x1m7q5"
Input System
 ↓
Action
 ↓
Player
```

Ações:

```text id="m8q2x4"
Move
Look
Jump
Crouch
Sprint
Interact
Attack
Use
Inventory
Map
```

---

# 8. PLAYER-7 — Movement

Criar abstração:

```text id="7q3m9x"
MovementController
```

Suportar:

```text id="4m1x8q"
walk
run
sprint
crouch
jump
fall
swim
climb
```

A física resolve colisões; Player decide intenção de movimento.

---

# 9. PLAYER-8 — Movement Modes

```text id="m5q2x9"
Grounded
Airborne
Swimming
Climbing
Vehicle
Flying
ZeroGravity
```

Cada modo pode ter regras próprias.

---

# 10. PLAYER-9 — Character Controller Integration

```text id="x8m4q1"
Player
 ↓
Movement Intent
 ↓
Character Controller
 ↓
Physics
 ↓
Final Position
```

O Player System não implementa colisão.

---

# 11. PLAYER-10 — Camera

Criar:

```text id="q3m7x5"
PlayerCamera
```

Modos:

```text id="1m8q2x"
First Person
Third Person
Free Camera
Vehicle Camera
Spectator
Cinematic
```

---

# 12. PLAYER-11 — Camera Collision

Na terceira pessoa:

```text id="x5q9m2"
camera
 ↓
raycast
 ↓
wall
 ↓
camera reposition
```

---

# 13. PLAYER-12 — Look System

Separar:

```text id="7m3x8q"
Input
 ↓
Look Intent
 ↓
Camera
```

Permitir:

```text id="9x2m6q"
sensitivity
invertY
FOV
smoothing
```

---

# 14. PLAYER-13 — Interaction

Sistema central:

```text id="m4q8x1"
Player
 ↓
Interaction Ray
 ↓
Target
 ↓
Interaction API
```

Pode encontrar:

```text id="7x3m9q"
block
entity
NPC
machine
container
vehicle
structure
```

---

# 15. PLAYER-14 — Interaction Context

```text id="2m7q5x"
InteractionContext
├── player
├── target
├── position
├── face
├── heldItem
└── mode
```

---

# 16. PLAYER-15 — Use Action

Uma interação pode significar:

```text id="8x4m1q"
open
activate
talk
harvest
ride
craft
inspect
```

O Player System só encaminha.

---

# 17. PLAYER-16 — Block Interaction

```text id="m9q2x7"
Player
 ↓
Target Block
 ↓
Block Interaction API
```

---

# 18. PLAYER-17 — Entity Interaction

```text id="5x8m3q"
Player
 ↓
Entity
 ↓
Entity Interaction
```

---

# 19. PLAYER-18 — NPC Interaction

NPC pode fornecer:

```text id="q1m6x9"
Dialogue
Trade
Quest
Information
Service
```

mas essas funcionalidades continuam em sistemas próprios.

---

# 20. PLAYER-19 — Combat Interface

O Player System fornece intenção:

```text id="3x7m4q"
Attack Intent
```

Combat System resolve:

```text id="m8q2x5"
range
target
damage
cooldown
hit
```

---

# 21. PLAYER-20 — Tool Interface

Ferramenta segurada:

```text id="9m4x7q"
HeldItem
```

pode implementar:

```text id="x2q8m1"
ToolAction
```

Player dispara a ação.

---

# 22. PLAYER-21 — Build Interface

```text id="6m3q9x"
Player
 ↓
Build Intent
 ↓
Build Engine
```

Player não altera voxel diretamente.

---

# 23. PLAYER-22 — Mining Interface

```text id="4x8m2q"
Player
 ↓
Mining Intent
 ↓
Build/Destruction Engine
```

---

# 24. PLAYER-23 — Inventory Integration

Player possui uma referência:

```text id="m7q1x5"
Inventory
```

mas o armazenamento pertence ao Inventory System.

---

# 25. PLAYER-24 — Equipment

A integração deve suportar:

```text id="8m4x2q"
Armor
Accessories
Backpack
Loadout
Held Item
```

O sistema de slots já planejado fica no Inventory/Equipment System.

---

# 26. PLAYER-25 — Loadout Switching

Player pode solicitar:

```text id="x5m7q3"
switchLoadout(2)
```

Inventory valida e executa.

Exemplo:

```text id="9q2m8x"
Mining
Builder
Explorer
Space
```

---

# 27. PLAYER-26 — Health

Criar interface:

```text id="m4x8q2"
HealthComponent
├── current
├── maximum
└── state
```

O sistema de combate/ambiente produz os danos.

---

# 28. PLAYER-27 — Status Effects

Player pode possuir:

```text id="7m2q9x"
StatusEffect
```

como:

```text id="5x8m1q"
poisoned
wet
cold
hot
fatigued
slowed
```

O efeito é fornecido por outro sistema.

---

# 29. PLAYER-28 — Temperature

Player consulta Environment:

```text id="m9q4x7"
environment
 ↓
temperature
 ↓
Player Status
```

---

# 30. PLAYER-29 — Exposure

Pode existir:

```text id="x2m6q8"
ExposureState
```

para:

```text id="4q7m1x"
cold
heat
pressure
atmospheric conditions
radiation-like environments
```

---

# 31. PLAYER-30 — Stamina

Criar:

```text id="m5x8q2"
Stamina
├── current
├── max
├── regeneration
└── exhaustion
```

Consumida por:

```text id="9m3q7x"
sprint
climb
swim
jump-heavy actions
```

---

# 32. PLAYER-31 — Energy / Mana-like Resources

Não hardcode “mana”.

Criar recurso genérico:

```text id="x4m8q1"
ResourcePool
```

Exemplos:

```text id="7q2m5x"
stamina
energy
magic
oxygen
```

---

# 33. PLAYER-32 — Hunger / Needs

Criar interface para Survival Needs:

```text id="m9x3q7"
Need
├── type
├── current
├── max
└── decay
```

Tipos futuros:

```text id="4q8m2x"
food
water
rest
temperature
oxygen
```

Não precisa estar tudo ativo em todas as experiências/modos.

---

# 34. PLAYER-33 — Survival

Um `SurvivalProfile` pode determinar:

```text id="x5m1q8"
enabled needs
damage rules
respawn rules
resource consumption
```

---

# 35. PLAYER-34 — Sleep

Criar:

```text id="m7q4x2"
SleepSystem
```

Fluxo:

```text id="8x3m9q"
Player
 ↓
valid sleeping location
 ↓
sleep
 ↓
world time advancement rules
```

---

# 36. PLAYER-35 — Bed / Rest

Não hardcode cama.

Criar:

```text id="2m8q5x"
RestPoint
```

Uma cama, cápsula ou estrutura poderia registrar essa função.

---

# 37. PLAYER-36 — Mounting

Jogador pode entrar em:

```text id="x4m7q1"
Vehicle
Mount
Seat
```

---

# 38. PLAYER-37 — Vehicle Control

```text id="9q2m6x"
Player Input
 ↓
Vehicle Control
 ↓
Vehicle Physics
```

Player não controla rigid body diretamente.

---

# 39. PLAYER-38 — Railway Vehicles

Também:

```text id="m5x8q3"
Player
 ↓
Train Seat
 ↓
Vehicle Controller
 ↓
Rail Physics
```

---

# 40. PLAYER-39 — Swimming

```text id="7m3q9x"
Fluid Query
 ↓
Player Movement Mode
 ↓
Swimming Controller
 ↓
Physics
```

---

# 41. PLAYER-40 — Climbing

Suporte para:

```text id="x2m6q8"
ladder
vine
climbable surface
```

---

# 42. PLAYER-41 — Flying

Não necessariamente flight livre.

Criar uma capacidade:

```text id="m8q4x1"
FlightCapability
```

que pode ser fornecida por:

```text id="9x2m7q"
equipment
vehicle
dimension rule
ability
```

---

# 43. PLAYER-42 — Zero Gravity

Para espaço:

```text id="m5q8x2"
Player
 ↓
Zero-G Controller
 ↓
Physics
```

---

# 44. PLAYER-43 — Dimension Travel

```text id="7x3m9q"
Player
 ↓
Dimension Travel Request
 ↓
Dimension System
 ↓
new world
```

---

# 45. PLAYER-44 — Transition State

```text id="4m8q1x"
TRAVEL_REQUESTED
LOADING
TRANSITIONING
ARRIVED
```

---

# 46. PLAYER-45 — Chunk Awareness

Player é uma das principais fontes do Chunk Streaming.

```text id="m7x2q5"
Player Position
 ↓
Chunk Manager
 ↓
Load relevant chunks
```

---

# 47. PLAYER-46 — Simulation Anchor

O player define uma área de alta prioridade:

```text id="x4q8m2"
Player
 ↓
Simulation Anchor
```

Isso influencia:

```text id="9m3x7q"
chunks
entities
physics
weather
vegetation
AI
```

---

# 48. PLAYER-47 — Interest Management

Em multiplayer:

```text id="m8q1x5"
Player A
 ↓
Interest Region
```

O servidor envia somente o necessário.

---

# 49. PLAYER-48 — Network Identity

```text id="7x2m9q"
NetworkPlayer
├── playerId
├── connectionId
└── authority
```

---

# 50. PLAYER-49 — Server Authority

No multiplayer:

```text id="4m8q3x"
Client Input
 ↓
Server
 ↓
Player Simulation
 ↓
Authoritative State
```

---

# 51. PLAYER-50 — Client Prediction

Para ações de movimento:

```text id="x7m1q8"
input
 ↓
client prediction
 ↓
server validation
 ↓
reconciliation
```

---

# 52. PLAYER-51 — Replication

Servidor replica:

```text id="m5q9x2"
position
rotation
state
equipment
appearance
```

conforme a relevância.

---

# 53. PLAYER-52 — Interpolation

Outros jogadores:

```text id="8x4m7q"
network state
 ↓
interpolation
 ↓
smooth visual movement
```

---

# 54. PLAYER-53 — Persistence

Salvar:

```text id="m3q8x1"
location
dimension
inventory reference
equipment
progression
unlocks
statistics
```

---

# 55. PLAYER-54 — Save Separation

Não criar:

```text id="7x2m9q"
Player.json com tudo
```

Dividir por domínio:

```text id="p4x8m2"
Identity
Inventory
Equipment
Progression
Statistics
Knowledge
Settings
```

---

# 56. PLAYER-55 — Versioning

```text id="m9q3x7"
PlayerDataVersion
```

para migração.

---

# 57. PLAYER-56 — Player Migration

```text id="x4m8q1"
Old Player Data
 ↓
Migration
 ↓
New Player Data
```

---

# 58. PLAYER-57 — Progression

Criar:

```text id="7m2q5x"
ProgressionState
├── level
├── experience
├── skills
├── unlocks
└── milestones
```

---

# 59. PLAYER-58 — Skills

As skills devem ser genéricas:

```text id="m8x3q9"
Mining
Building
Farming
Engineering
Magic
Exploration
Science
Trading
```

Isso não precisa ser limitado a uma lista fixa.

---

# 60. PLAYER-59 — Skill API

```text id="4q7m1x"
Skill
├── id
├── level
├── experience
└── modifiers
```

Mods podem adicionar skills.

---

# 61. PLAYER-60 — Experience Sources

XP/Progression pode vir de:

```text id="m5x8q2"
crafting
building
exploration
research
quests
farming
trade
```

---

# 62. PLAYER-61 — Unlocks

Desbloqueios:

```text id="7x3m9q"
recipe
slot
ability
dimension
technology
structure
```

O sistema de unlock consumível do Inventory pode desbloquear slots, enquanto Progression controla outros desbloqueios.

---

# 63. PLAYER-62 — Achievements

Separar:

```text id="m4q8x1"
Achievement
```

de Progression.

Pode registrar:

```text id="9m2x7q"
conditions
progress
completion
```

---

# 64. PLAYER-63 — Quest Integration

Quest System fornece:

```text id="x5m1q8"
objectives
```

Player fornece:

```text id="m7q4x2"
actions
```

Quest rastreia progresso.

---

# 65. PLAYER-64 — Knowledge

Player pode possuir:

```text id="8x3m9q"
Knowledge
├── discoveries
├── recipes
├── locations
├── species
├── technologies
└── observations
```

O Knowledge System continua sendo o dono desse domínio.

---

# 66. PLAYER-65 — Journal

Criar:

```text id="m2q7x5"
Journal
```

para registrar:

```text id="4x8m1q"
discoveries
events
locations
notes
quests
```

---

# 67. PLAYER-66 — Map

Player pode ter um mapa:

```text id="9m3q7x"
MapState
```

mas o Map System controla dados cartográficos.

---

# 68. PLAYER-67 — Exploration

Exploração pode rastrear:

```text id="x5m8q2"
discovered regions
visited biomes
visited dimensions
landmarks
```

---

# 69. PLAYER-68 — Reputation

Player pode ter reputação por grupo:

```text id="m7q2x9"
Faction
Village
City
Organization
```

---

# 70. PLAYER-69 — Relationships

NPCs podem possuir relação com o player:

```text id="4x8m1q"
friendship
trust
reputation
history
```

O Relationship System deve controlar isso.

---

# 71. PLAYER-70 — Ownership

Player pode ser proprietário de:

```text id="m9q3x7"
container
structure
vehicle
land claim
business
```

---

# 72. PLAYER-71 — Permissions

Player pode possuir:

```text id="x2m6q8"
permissions
roles
faction membership
```

mas o Permission System é a autoridade real.

---

# 73. PLAYER-72 — Character Customization

Separar visual de lógica:

```text id="m5x8q1"
CharacterAppearance
├── body
├── face
├── hair
├── clothing
├── accessories
└── cosmetic overrides
```

---

# 74. PLAYER-73 — Appearance Renderer

Renderer recebe:

```text id="7q3m9x"
PlayerRenderState
```

e gera visual.

---

# 75. PLAYER-74 — Equipment Visuals

Equipamentos podem contribuir:

```text id="x4m8q2"
armor
helmet
backpack
accessories
```

para o visual.

---

# 76. PLAYER-75 — Cosmetic System

Cosméticos ficam separados de gameplay:

```text id="m9q1x7"
Cosmetic
```

Assim aparência não precisa alterar atributos.

---

# 77. PLAYER-76 — Animation State

Player fornece:

```text id="8x2m5q"
movement state
equipment state
action state
```

Animation Controller transforma isso em animações.

---

# 78. PLAYER-77 — Emotes

Criar:

```text id="m7q4x1"
EmoteSystem
```

para ações puramente visuais.

---

# 79. PLAYER-78 — Interaction with World

O player pode atuar sobre:

```text id="x5m8q3"
blocks
entities
machines
vehicles
containers
structures
```

por interfaces públicas.

---

# 80. PLAYER-79 — Context Actions

A interface pode selecionar ações válidas:

```text id="4m9q2x"
target
 ↓
available actions
```

Exemplo:

```text id="7x3m8q"
Chest
→ Open

Machine
→ Configure

NPC
→ Talk

Plant
→ Harvest
```

---

# 81. PLAYER-80 — Interaction Priority

Quando vários objetos ocupam a mesma região:

```text id="m2x8q5"
closest
+
line of sight
+
priority
```

determinam o alvo.

---

# 82. PLAYER-81 — Reach

Criar:

```text id="9q4m7x"
InteractionReach
```

mas ferramentas/veículos podem alterar essa capacidade.

---

# 83. PLAYER-82 — Line of Sight

Consultar:

```text id="x5m2q8"
Physics Raycast
```

para validar interação.

---

# 84. PLAYER-83 — Environmental Awareness

Player pode receber:

```text id="m7q3x9"
temperature
light
fluid
pressure
weather
```

e outros sistemas podem reagir.

---

# 85. PLAYER-84 — Sensors

Criar API:

```text id="4x8m1q"
PlayerSensor
```

para equipamentos/abilities.

Exemplo:

```text id="9m2q7x"
ore scanner
biome scanner
environment scanner
```

---

# 86. PLAYER-85 — Equipment Capabilities

Equipamentos podem fornecer:

```text id="m5x8q2"
FlightCapability
BreathingCapability
MiningCapability
ScannerCapability
ProtectionCapability
```

sem aumentar o código do Player.

---

# 87. PLAYER-86 — Ability System

Criar:

```text id="x7m3q9"
Ability
```

com:

```text id="4q8m1x"
cost
cooldown
requirements
effect
```

---

# 88. PLAYER-87 — Cooldowns

Sistema genérico:

```text id="m9q2x7"
CooldownManager
```

usado por:

```text id="7x4m8q"
abilities
tools
combat
machines
```

---

# 89. PLAYER-88 — Player Commands

No jogo/modding:

```text id="5m1x9q"
PlayerCommand
```

exemplos:

```text id="8q3m7x"
/home
/warp
/map
```

mas permissões são verificadas externamente.

---

# 90. PLAYER-89 — Spectator

Modo:

```text id="m4x8q2"
SpectatorController
```

sem alterar o Character State normal.

---

# 91. PLAYER-90 — Admin / Debug Player

Dev tools podem habilitar:

```text id="x7m2q5"
noclip
freecam
teleport
debug overlay
```

sempre através de permissões.

---

# 92. PLAYER-91 — Multiplayer Reconnect

Se conexão cai:

```text id="9m3q8x"
Disconnected
 ↓
session preserved
 ↓
Reconnect
 ↓
resume
```

---

# 93. PLAYER-92 — AFK

Criar:

```text id="m5q1x7"
AFK State
```

para multiplayer.

---

# 94. PLAYER-93 — Player Activity

Registrar:

```text id="4x8m2q"
movement
interaction
building
crafting
exploration
```

sem precisar guardar logs infinitos.

---

# 95. PLAYER-94 — Statistics

Exemplos:

```text id="7m3q9x"
distance traveled
blocks mined
blocks placed
time played
biomes visited
dimensions visited
```

---

# 96. PLAYER-95 — History

Player pode ter histórico:

```text id="m8x4q2"
discoveries
major events
quests
relationships
```

O History System continua sendo o dono da história global.

---

# 97. PLAYER-96 — Event Integration

Eventos:

```text id="9q2m7x"
PlayerSpawned
PlayerMoved
PlayerEnteredBiome
PlayerChangedDimension
PlayerInteracted
PlayerStartedMining
PlayerPlacedBlock
PlayerOpenedContainer
PlayerEnteredVehicle
PlayerLevelUp
```

---

# 98. PLAYER-97 — Scheduler Integration

Usar várias escalas:

```text id="x5m8q1"
frame
→ input/camera

second
→ status/resource

minute
→ some progression

hour/day
→ long-term player effects
```

---

# 99. PLAYER-98 — Persistence Architecture

Estrutura:

```text id="m7q3x9"
player/
├── identity
├── character
├── progression
├── inventory
├── equipment
├── knowledge
├── statistics
├── relationships
└── settings
```

---

# 100. PLAYER-99 — Final Architecture

```text id="q4m8x2"
                              PLAYER
                                │
                 ┌──────────────┼──────────────┐
                 │              │              │
              IDENTITY       CHARACTER      PROGRESSION
                 │              │              │
             profile        movement         skills
             permissions    health            XP
             appearance     stamina           unlocks
                 │              │              │
                 └──────────────┼──────────────┘
                                │
              ┌─────────────────┼─────────────────┐
              │                 │                 │
           INVENTORY         INTERACTION        CAMERA
              │                 │                 │
          equipment          blocks             first
          backpacks          entities           third
          loadouts           NPCs               vehicle
              │                 │
              └─────────────────┼─────────────────┘
                                │
          ┌─────────────────────┼─────────────────────┐
          │                     │                     │
       PHYSICS                WORLD                 NETWORK
          │                     │                     │
      movement              chunks                 server
      gravity               dimensions             replication
      swimming              biomes                 prediction
      vehicles              structures             authority
                                │
          ┌─────────────────────┼──────────────────────┐
          │                     │                      │
       KNOWLEDGE             ECONOMY               SOCIETY
          │                     │                      │
       discoveries           trade                 reputation
       research              jobs                  factions
```

# 101. Ordem de implementação

Eu faria:

```text id="x7m4q9"
PLAYER-0 Identity
PLAYER-1 Profile
PLAYER-2 Character State
PLAYER-3 Spawn
PLAYER-4 State Machine
PLAYER-5 Input Integration
PLAYER-6 Movement
PLAYER-7 Camera
PLAYER-8 Interaction
PLAYER-9 Character Controller
PLAYER-10 Health
PLAYER-11 Stamina
PLAYER-12 Status
PLAYER-13 Inventory Integration
PLAYER-14 Equipment Integration
PLAYER-15 Build Integration
PLAYER-16 Mining Integration
PLAYER-17 Vehicle Integration
PLAYER-18 Swimming
PLAYER-19 Climbing
PLAYER-20 Dimension Travel
PLAYER-21 Progression
PLAYER-22 Skills
PLAYER-23 Unlocks
PLAYER-24 Knowledge
PLAYER-25 Exploration
PLAYER-26 Reputation
PLAYER-27 Customization
PLAYER-28 Animation
PLAYER-29 Abilities
PLAYER-30 Survival
PLAYER-31 Save/Load
PLAYER-32 Multiplayer
PLAYER-33 Prediction
PLAYER-34 Replication
PLAYER-35 Statistics
PLAYER-36 History
PLAYER-37 Events
PLAYER-38 Debug
PLAYER-39 Mod API
PLAYER-40 Stress Testing
```

# 102. Primeiro Vertical Slice

O primeiro vertical slice precisa provar que o jogador realmente existe dentro do NEXORA:

```text id="m8q3x1"
START
 ↓
Create Player
 ↓
Spawn World
 ↓
Camera
 ↓
Input
 ↓
Movement
 ↓
Physics
 ↓
Chunk Streaming
 ↓
Look at Block
 ↓
Interact
 ↓
Mine Block
 ↓
Drop
 ↓
Inventory
 ↓
Place Block
 ↓
Save
 ↓
Quit
 ↓
Reload
 ↓
Player restored
```

Depois:

```text id="7x2m9q"
Player
 ↓
enter vehicle
 ↓
vehicle physics
 ↓
leave vehicle
 ↓
swim
 ↓
climb
 ↓
dimension travel
```

E então o slice completo:

```text id="q5m8x2"
PLAYER
 ↓
explores
 ↓
discovers resource
 ↓
knowledge updated
 ↓
crafts equipment
 ↓
builds structure
 ↓
joins settlement
 ↓
trades
 ↓
gains reputation
 ↓
learns technology
 ↓
changes loadout
 ↓
explores deeper
```

## Regra principal do Player System

> **O jogador é um participante do mundo, não o centro técnico do mundo.**

Ou seja:

```text id="m4x7q9"
Player
   │
   ├── requests movement → Physics
   ├── requests building → Build Engine
   ├── requests mining → Destruction
   ├── accesses items → Inventory
   ├── enters vehicle → Vehicle System
   ├── learns → Knowledge
   ├── trades → Economy
   └── interacts → World APIs
```

Isso é importante para o NEXORA porque **NPCs, mobs, veículos e outros agentes também vão utilizar grande parte dessas mesmas capacidades**. O Player System não deve ser uma coleção de funções especiais que só o jogador consegue fazer.

A meta é que o jogador seja simplesmente **uma entidade altamente capaz dentro do mesmo mundo vivo**.
