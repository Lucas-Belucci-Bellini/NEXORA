Sim. **Entity System** é um dos documentos mais importantes que faltavam, porque praticamente tudo no NEXORA que “vive” no mundo precisa passar por ele.

# NEXORA — ENTITY SYSTEM MASTER PLAN

A ideia central:

> **Entity System define quem existe no mundo, onde está, qual é seu ciclo de vida e quais componentes possui.**

Ele **não** deve implementar sozinho combate, IA, física, inventário ou renderização.

A arquitetura:

```text id="h6q1z8"
                         ENTITY SYSTEM
                              │
                 ┌────────────┼────────────┐
                 │            │            │
              IDENTITY      STATE       COMPONENTS
                 │            │            │
              EntityID      Transform      AI
              TypeID        Velocity       Health
              UUID          Lifecycle      Inventory
                             Flags          Equipment
                 │            │            │
                 └────────────┼────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
      PLAYER                 MOB                   NPC
        │                     │                     │
      input                  AI                  society
      skills                ecology             profession
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
           ┌──────────────────┼──────────────────┐
           │                  │                  │
        PHYSICS            RENDERER           NETWORK
           │                  │                  │
       collision            visual            sync
       movement             model             replication
```

---

# 1. ENTITY-0 — Entity Core

Criar:

```text id="6m3q9x"
Entity
EntityType
EntityId
EntityManager
EntityRegistry
EntityContext
EntityState
```

Toda entidade começa com uma identidade e um ciclo de vida.

---

# 2. ENTITY-1 — Entity ID

Cada entidade precisa de identificador único.

```text id="x7m2q5"
EntityID
├── worldId
├── entityId
└── generation
```

Para rede/saves pode existir também:

```text id="4q8m1x"
PersistentEntityUUID
```

---

# 3. ENTITY-2 — Entity Type

Separar:

```text id="m9x3q7"
Entity Type
```

de:

```text id="7m2q9x"
Entity Instance
```

Exemplo:

```text id="x5m8q1"
nexora:deer
```

é o tipo.

O animal específico no mundo é uma instância.

---

# 4. ENTITY-3 — Entity Definition

```text id="4m8q2x"
EntityDefinition
├── id
├── components
├── capabilities
├── dimensions
├── spawnRules
├── persistenceRules
└── visualProfile
```

---

# 5. ENTITY-4 — Entity Instance

```text id="m7x3q9"
EntityInstance
├── entityId
├── typeId
├── position
├── rotation
├── state
└── components
```

---

# 6. ENTITY-5 — Lifecycle

Estados:

```text id="x4m8q1"
CREATED
SPAWNING
ACTIVE
INACTIVE
DESPAWNING
REMOVING
REMOVED
```

---

# 7. ENTITY-6 — Spawn

Criar:

```text id="5m9q2x"
SpawnContext
```

com:

```text id="7x3m8q"
dimension
position
reason
source
rules
```

Fontes:

```text id="m4q8x1"
WorldGen
Player
Spawner
Breeding
Quest
Structure
Mod
```

---

# 8. ENTITY-7 — Despawn

Despawn não significa necessariamente destruir permanentemente.

Estados:

```text id="x5m2q7"
active
sleeping
unloaded
removed
```

Isso é muito importante para mobs e simulação distante.

---

# 9. ENTITY-8 — Persistence Policy

Cada entidade declara:

```text id="9q3m7x"
PERSISTENT
TEMPORARY
REGIONAL
ABSTRACTABLE
```

---

# 10. ENTITY-9 — Persistent Entity

Exemplos:

```text id="m8x4q1"
NPC importante
player
vehicle
machine
boss
named creature
```

---

# 11. ENTITY-10 — Temporary Entity

Exemplos:

```text id="7m2q9x"
particle-like entity
temporary effect
short-lived projectile
```

---

# 12. ENTITY-11 — Abstractable Entity

Uma população distante pode deixar de ser representada individualmente.

```text id="x4m8q1"
1,000 mobs
 ↓
regional population state
```

Isso será essencial para o objetivo de milhares de criaturas.

---

# 13. ENTITY-12 — Component Architecture

Eu usaria composição:

```text id="m5q8x2"
Entity
├── TransformComponent
├── PhysicsComponent
├── HealthComponent
├── AIComponent
├── InventoryComponent
├── EquipmentComponent
└── ...
```

A entidade não precisa herdar 30 classes.

---

# 14. ENTITY-13 — Base Components

Componentes básicos:

```text id="4x7m2q"
Transform
Identity
Lifecycle
Tags
Metadata
Persistence
```

---

# 15. ENTITY-14 — Transform

```text id="m9q3x7"
TransformComponent
├── position
├── rotation
├── scale
└── parent
```

---

# 16. ENTITY-15 — Position

Usar:

```text id="x5m8q1"
WorldPosition
```

integrado ao sistema de chunks.

---

# 17. ENTITY-16 — Rotation

Suportar:

```text id="4m8q2x"
yaw
pitch
roll
```

conforme o tipo de entidade.

---

# 18. ENTITY-17 — Scale

Nem toda entidade será exatamente do tamanho de um bloco.

```text id="m7x3q9"
scale
bounds
```

---

# 19. ENTITY-18 — Parent / Child

Algumas entidades precisam de hierarquia:

```text id="x2m6q8"
Vehicle
 ├── Turret
 ├── Seat
 └── Attachment
```

---

# 20. ENTITY-19 — Entity Relationships

Relacionamentos:

```text id="4q8m1x"
owner
passenger
parent
child
leader
member
target
```

---

# 21. ENTITY-20 — Tags

Usar:

```text id="m9x3q7"
#living
#mob
#npc
#vehicle
#projectile
#machine
```

para consultas e sistemas.

---

# 22. ENTITY-21 — Capability System

Entidades podem declarar capacidades:

```text id="x5m8q1"
Combatant
InventoryHolder
FluidContainer
EnergyNode
Rideable
Climbable
Interactable
```

---

# 23. ENTITY-22 — Capability Discovery

Outro sistema pode perguntar:

```text id="4m7q2x"
entity.hasCapability(X)
```

sem fazer casts específicos.

---

# 24. ENTITY-23 — Living Entity

Criar uma camada:

```text id="m8x3q9"
LivingEntity
```

com possíveis componentes:

```text id="7q2m5x"
Health
Needs
Movement
Status
AI
```

---

# 25. ENTITY-24 — Player

Player é uma composição especializada:

```text id="x4m8q1"
Player
 ↓
Entity
 ↓
LivingEntity
 ↓
Player Components
```

---

# 26. ENTITY-25 — Mob

```text id="m5q7x2"
Mob
 ↓
LivingEntity
 ↓
Mob-specific components
```

---

# 27. ENTITY-26 — NPC

NPC:

```text id="9x3m7q"
NPC
 ↓
LivingEntity
 ↓
Profession
Memory
Knowledge
Social
```

---

# 28. ENTITY-27 — Animal

Animais podem possuir:

```text id="x5m2q8"
Ecology
Reproduction
Needs
AI
```

---

# 29. ENTITY-28 — Vehicle

Veículos:

```text id="m7q4x1"
VehicleEntity
├── Physics
├── Seats
├── Inventory
├── Fuel
├── Energy
└── Control
```

---

# 30. ENTITY-29 — Projectile

```text id="4x8m2q"
ProjectileEntity
├── Physics
├── Owner
├── Lifetime
└── CombatCapability
```

---

# 31. ENTITY-30 — Item Entity

Itens no mundo:

```text id="m9x3q7"
ItemEntity
├── ItemStack
├── Position
├── PickupRules
└── Lifetime
```

---

# 32. ENTITY-31 — Block Entity

Precisa separar conceito de:

```text id="x5m8q1"
Voxel Block
```

e:

```text id="4q7m2x"
Block Entity
```

Exemplo:

```text id="m7q3x9"
Chest
Machine
Controller
```

---

# 33. ENTITY-32 — Block Entity Integration

Block Entity pode ser gerenciada pelo Entity/Block sistema conforme a arquitetura final, mas precisa possuir identidade e persistência próprias quando necessário.

---

# 34. ENTITY-33 — Spatial Index

O Entity System precisa de uma forma rápida de localizar entidades.

```text id="x8m2q5"
SpatialIndex
```

baseada em:

```text id="4m7q1x"
chunk
region
grid
```

---

# 35. ENTITY-34 — Entity Query

API:

```text id="m9x3q7"
queryArea
queryRadius
queryChunk
queryType
queryTag
```

---

# 36. ENTITY-35 — Nearby Entities

Exemplo:

```text id="x5m8q1"
findNearbyEntities(position, radius)
```

usado por:

```text id="4q7m2x"
AI
Combat
Player
Audio
Renderer
Physics
```

---

# 37. ENTITY-36 — Query Filters

Filtros:

```text id="m7x3q9"
type
tag
faction
component
distance
visibility
```

---

# 38. ENTITY-37 — Chunk Ownership

Entidade tem uma relação com seu chunk atual:

```text id="x4m8q1"
Entity
 ↓
Current Chunk
```

Quando atravessa a fronteira:

```text id="m5q7x2"
Chunk A
→
Chunk B
```

a identidade permanece.

---

# 39. ENTITY-38 — Region Ownership

Para entidades de grande escala:

```text id="9x3m7q"
region
```

---

# 40. ENTITY-39 — Entity Streaming

Player entra:

```text id="x5m2q8"
region
 ↓
entities become relevant
```

---

# 41. ENTITY-40 — Entity LOD

```text id="m7q4x1"
FULL
→ individual entity

REGIONAL
→ simplified state

ABSTRACT
→ population/aggregate
```

---

# 42. ENTITY-41 — Rehydration

Quando volta para FULL:

```text id="4x8m2q"
regional state
 ↓
instantiate entities
```

---

# 43. ENTITY-42 — Entity Sleep

Uma entidade inativa pode:

```text id="m9x3q7"
SLEEP
```

e acordar devido a:

```text id="x5m8q1"
player
event
proximity
schedule
```

---

# 44. ENTITY-43 — Simulation Priority

Prioridades:

```text id="4m7q2x"
CRITICAL
HIGH
NORMAL
LOW
BACKGROUND
```

---

# 45. ENTITY-44 — Entity Tick

Entidade não precisa ter sempre tick individual.

Usar:

```text id="m7q3x9"
scheduler
```

para componentes que precisam de processamento.

---

# 46. ENTITY-45 — Component Tick

Exemplo:

```text id="x4m8q1"
AI → 10 Hz
Health → event-driven
Animation → frame
Needs → minute
```

---

# 47. ENTITY-46 — Event-Driven Components

Preferir eventos para estados que não precisam de polling constante.

---

# 48. ENTITY-47 — Component Lifecycle

```text id="m5q8x2"
Create
Attach
Initialize
Enable
Disable
Destroy
```

---

# 49. ENTITY-48 — Component Dependencies

Um componente pode declarar:

```text id="7x3m9q"
requires Transform
requires Physics
```

O sistema resolve a ordem.

---

# 50. ENTITY-49 — Dependency Validation

Detectar:

```text id="4m8q1x"
missing component
cyclic dependency
invalid component
```

---

# 51. ENTITY-50 — Entity State Machine

Uma entidade pode possuir estados:

```text id="m9x3q7"
state
```

sem obrigar todo tipo de entidade a usar os mesmos estados.

---

# 52. ENTITY-51 — Metadata

Pequenos dados:

```text id="x5m8q1"
name
custom tags
flags
```

---

# 53. ENTITY-52 — Named Entity

NPC/criatura importante pode possuir:

```text id="4m7q2x"
persistentName
```

---

# 54. ENTITY-53 — Unique Entity

Para entidades importantes:

```text id="m7q3x9"
unique=true
```

---

# 55. ENTITY-54 — Entity Ownership

Pode existir:

```text id="x4m8q1"
ownerId
```

para:

```text id="m5q7x2"
vehicle
pet-like entity
machine
structure controller
```

---

# 56. ENTITY-55 — Faction

Entidades podem possuir:

```text id="9x3m7q"
faction
```

usado por:

```text id="x5m2q8"
Combat
Civilization
Diplomacy
AI
```

---

# 57. ENTITY-56 — Relationship

Uma entidade pode armazenar referências relacionais:

```text id="m7q4x1"
friend
enemy
leader
employer
family
```

O Relationship System é o dono das regras.

---

# 58. ENTITY-57 — Health Component

```text id="4x8m2q"
HealthComponent
```

Não implementar combate dentro dele.

Ele apenas mantém:

```text id="m9x3q7"
current
maximum
state
```

---

# 59. ENTITY-58 — Status Component

```text id="x5m8q1"
StatusComponent
```

para efeitos externos.

---

# 60. ENTITY-59 — Needs Component

Entidades podem possuir:

```text id="4m7q2x"
food
water
rest
oxygen
temperature
```

conforme seu tipo.

---

# 61. ENTITY-60 — Inventory Component

Uma entidade pode implementar:

```text id="m7q3x9"
InventoryHolder
```

---

# 62. ENTITY-61 — Equipment Component

```text id="x4m8q1"
EquipmentHolder
```

---

# 63. ENTITY-62 — Fluid Component

Máquinas/veículos:

```text id="m5q7x2"
FluidContainer
```

---

# 64. ENTITY-63 — Energy Component

```text id="9x3m7q"
EnergyNode
```

---

# 65. ENTITY-64 — Physics Component

```text id="x5m2q8"
PhysicsBody
```

gerenciado pela Physics Engine.

---

# 66. ENTITY-65 — Render Component

```text id="m7q4x1"
RenderRepresentation
```

Renderer possui o estado visual.

---

# 67. ENTITY-66 — Animation Component

```text id="4x8m2q"
AnimationState
```

Animation System executa.

---

# 68. ENTITY-67 — Audio Component

```text id="m9q3x7"
AudioEmitter
```

Audio System executa.

---

# 69. ENTITY-68 — AI Component

```text id="x5m8q1"
AIController
```

AI System executa.

---

# 70. ENTITY-69 — Combat Component

```text id="4m7q2x"
Combatant
```

Combat System executa.

---

# 71. ENTITY-70 — Interaction Component

```text id="m7q3x9"
Interactable
```

---

# 72. ENTITY-71 — Rideable Component

```text id="x4m8q1"
Rideable
```

para veículos/animais/montarias.

---

# 73. ENTITY-72 — Spawn Rules

Entidade pode definir:

```text id="m5q7x2"
allowed biomes
temperature
time
season
dimension
population
```

---

# 74. ENTITY-73 — Spawn Manager

Criar:

```text id="9x3m7q"
SpawnManager
```

que coordena:

```text id="x5m2q8"
natural spawning
structure spawning
event spawning
```

---

# 75. ENTITY-74 — Population Caps

Por região:

```text id="m7q4x1"
max population
```

mas considerando:

```text id="4x8m2q"
species
habitat
ecology
```

---

# 76. ENTITY-75 — Spawn Budget

Cada região possui limite de trabalho de spawn:

```text id="m9q3x7"
spawn budget
```

para evitar explosões de entidades.

---

# 77. ENTITY-76 — Reproduction

A Entity API oferece identidade e criação.

Ecology decide:

```text id="x5m8q1"
reproduction
```

---

# 78. ENTITY-77 — Entity Birth

Registrar:

```text id="4m7q2x"
parent entities
birth event
genetic data
```

---

# 79. ENTITY-78 — Entity Death

A morte gera:

```text id="m7q3x9"
EntityDeathEvent
```

e outros sistemas reagem:

```text id="x4m8q1"
Loot
Ecology
History
Economy
```

---

# 80. ENTITY-79 — Entity Removal

Depois de tratar os eventos:

```text id="m5q7x2"
death
 ↓
cleanup
 ↓
remove
```

---

# 81. ENTITY-80 — Death vs Despawn

Separar:

```text id="9x3m7q"
DEATH
```

de:

```text id="x5m2q8"
DESPAWN
```

Isso é essencial para mobs.

---

# 82. ENTITY-81 — Teleport

Entidades podem atravessar posições/dimensões:

```text id="m7q4x1"
Entity
 ↓
Teleport Request
 ↓
validate
 ↓
move
```

---

# 83. ENTITY-82 — Dimension Transfer

```text id="4x8m2q"
Dimension A
 ↓
Transfer
 ↓
Dimension B
```

A identidade continua.

---

# 84. ENTITY-83 — Cross-Dimension Identity

```text id="m9q3x7"
same EntityID
different Dimension
```

quando apropriado.

---

# 85. ENTITY-84 — Mount System

```text id="x5m8q1"
Player
 ↓
Mount
 ↓
Vehicle
```

com relacionamento formal.

---

# 86. ENTITY-85 — Attachments

Entidades podem possuir attachments:

```text id="4m7q2x"
weapon
turret
tool
cosmetic
sensor
```

---

# 87. ENTITY-86 — Ownership Graph

```text id="m7q3x9"
Player
 ↓
Vehicle
 ↓
Turret
```

---

# 88. ENTITY-87 — Serialization

Criar:

```text id="x4m8q1"
EntitySerializer
```

que grava:

```text id="m5q7x2"
entityId
type
transform
persistent components
```

---

# 89. ENTITY-88 — Component Serialization

Cada componente pode registrar seus próprios dados:

```text id="9x3m7q"
serialize()
deserialize()
```

---

# 90. ENTITY-89 — Versioning

```text id="x5m2q8"
EntityDataVersion
ComponentVersion
```

---

# 91. ENTITY-90 — Migration

```text id="m7q4x1"
Old Entity
 ↓
Component Migration
 ↓
New Entity
```

---

# 92. ENTITY-91 — Chunk Persistence

Quando chunk descarrega:

```text id="4x8m2q"
Entities
 ↓
Entity Storage
 ↓
Chunk/Region
```

---

# 93. ENTITY-92 — Region Persistence

Grandes entidades podem estar associadas a regiões.

---

# 94. ENTITY-93 — Save Priority

Priorizar:

```text id="m9q3x7"
player
unique NPC
vehicle
machine
persistent structures
```

---

# 95. ENTITY-94 — Network Identity

```text id="x5m8q1"
NetworkEntityID
```

separado do ID persistente quando necessário.

---

# 96. ENTITY-95 — Replication

Servidor decide:

```text id="4m7q2x"
what
when
to whom
```

será replicado.

---

# 97. ENTITY-96 — Interest Management

```text id="m7q3x9"
Player
 ↓
Interest Region
 ↓
Relevant Entities
```

---

# 98. ENTITY-97 — Entity Delta

Enviar mudanças:

```text id="x4m8q1"
position changed
health changed
state changed
```

em vez da entidade inteira.

---

# 99. ENTITY-98 — Prediction

Para entidades controladas pelo jogador:

```text id="m5q7x2"
client prediction
```

e servidor corrige quando necessário.

---

# 100. ENTITY-99 — Interpolation

Entidades remotas podem ser renderizadas interpoladamente.

---

# 101. ENTITY-100 — Network Spawn

```text id="9x3m7q"
server creates entity
 ↓
replication
 ↓
client creates proxy
```

---

# 102. ENTITY-101 — Network Despawn

```text id="x5m2q8"
server removes relevance
 ↓
client removes proxy
```

---

# 103. ENTITY-102 — Authority

Cada entidade pode possuir:

```text id="m7q4x1"
server-authoritative
client-authoritative-with-validation
local-only
```

---

# 104. ENTITY-103 — Local-only Entity

Exemplos:

```text id="4x8m2q"
visual effects
UI helpers
camera objects
```

---

# 105. ENTITY-104 — Entity Queries for AI

AI pode perguntar:

```text id="m9q3x7"
nearby entities
```

e receber apenas os objetos relevantes.

---

# 106. ENTITY-105 — Perception

Não colocar percepção dentro do Entity Core.

Criar:

```text id="x5m8q1"
Perception Component/API
```

que usa:

```text id="4m7q2x"
spatial query
lighting
audio
line of sight
```

---

# 107. ENTITY-106 — Visibility

Renderer/Lighting/Physics podem fornecer:

```text id="m7q3x9"
isVisible
```

quando necessário.

---

# 108. ENTITY-107 — Entity Physics Interaction

Physics pode perguntar:

```text id="x4m8q1"
entity bounds
body
```

Entity apenas fornece referência.

---

# 109. ENTITY-108 — Entity Bounds

```text id="m5q7x2"
BoundingBox
BoundingSphere
Capsule
```

para consultas espaciais.

---

# 110. ENTITY-109 — Collision Shape Reference

A forma real pertence à Physics.

Entity possui apenas o vínculo:

```text id="9x3m7q"
PhysicsBodyRef
```

---

# 111. ENTITY-110 — Render Representation

Entity fornece:

```text id="x5m2q8"
RenderHandle
```

Renderer controla:

```text id="m7q4x1"
mesh
material
LOD
animation
```

---

# 112. ENTITY-111 — Entity Animation

Animation recebe:

```text id="4x8m2q"
movement state
action state
equipment
```

---

# 113. ENTITY-112 — Audio

Audio recebe:

```text id="m9q3x7"
position
surface
entity state
```

---

# 114. ENTITY-113 — Entity Events

Eventos:

```text id="x5m8q1"
EntityCreated
EntitySpawned
EntityLoaded
EntityUnloaded
EntityMoved
EntityComponentAdded
EntityComponentRemoved
EntityDamaged
EntityDied
EntityDespawned
EntityRemoved
```

---

# 115. ENTITY-114 — Event Bus

Todos esses eventos entram no Event Bus global.

---

# 116. ENTITY-115 — Entity Scheduler

Criar:

```text id="4m7q2x"
EntityScheduler
```

responsável por:

```text id="m7q3x9"
tick
wake
sleep
priority
```

---

# 117. ENTITY-116 — Temporal LOD

```text id="x4m8q1"
FULL
→ high frequency

REGIONAL
→ low frequency

ABSTRACT
→ statistical
```

---

# 118. ENTITY-117 — Thousands of Mobs

Como queremos **mais de 2.000 tipos de mobs**, não podemos criar uma classe exclusiva com lógica pesada para cada espécie.

Preferir:

```text id="m5q7x2"
EntityDefinition
+
Components
+
AI Profiles
+
Ecology Traits
```

---

# 119. ENTITY-118 — Species Templates

```text id="9x3m7q"
SpeciesDefinition
├── body
├── traits
├── ecology
├── behavior
├── reproduction
└── visuals
```

---

# 120. ENTITY-119 — Entity Templates

Criar templates reutilizáveis:

```text id="x5m8q1"
Template
 ↓
Entity Instance
```

---

# 121. ENTITY-120 — Behavior Profiles

Mob não precisa possuir uma classe única:

```text id="m7q4x1"
BehaviorProfile
```

pode combinar:

```text id="4x8m2q"
wander
herd
predator
prey
territorial
nocturnal
aquatic
```

---

# 122. ENTITY-121 — Genetics

Entity pode armazenar:

```text id="m9q3x7"
genetic seed
traits
```

quando Ecology precisar.

---

# 123. ENTITY-122 — Knowledge

NPC/animal inteligente pode possuir:

```text id="x5m8q1"
KnowledgeComponent
```

mas Knowledge System administra o conteúdo.

---

# 124. ENTITY-123 — Memory

```text id="4m7q2x"
MemoryComponent
```

para NPC/AI.

---

# 125. ENTITY-124 — Social Component

NPCs:

```text id="m7q3x9"
SocialComponent
├── relationships
├── group
└── reputation
```

---

# 126. ENTITY-125 — Profession Component

```text id="x4m8q1"
ProfessionComponent
```

Civilization System define profissão.

---

# 127. ENTITY-126 — Needs Component

```text id="m5q7x2"
NeedsComponent
```

---

# 128. ENTITY-127 — Schedule Component

NPC pode possuir:

```text id="9x3m7q"
daily schedule
```

---

# 129. ENTITY-128 — Location Memory

NPC pode lembrar:

```text id="x5m2q8"
home
work
city
important locations
```

---

# 130. ENTITY-129 — Entity History

Uma entidade importante pode ter:

```text id="m7q4x1"
birth
major events
relationships
achievements
```

---

# 131. ENTITY-130 — Civilization Integration

NPC/Player/Vehicle podem ser membros de:

```text id="4x8m2q"
settlement
faction
organization
```

---

# 132. ENTITY-131 — Economy Integration

NPC pode ter:

```text id="m9q3x7"
profession
inventory
wallet/resource state
```

Economy decide as regras.

---

# 133. ENTITY-132 — Commerce

Entity pode possuir:

```text id="x5m8q1"
MerchantCapability
```

---

# 134. ENTITY-133 — Entity Commerce

NPC:

```text id="4m7q2x"
Merchant
 ↓
Trade System
```

---

# 135. ENTITY-134 — Quest Integration

NPC pode:

```text id="m7q3x9"
offer quest
receive quest
track quest
```

---

# 136. ENTITY-135 — Knowledge Integration

Entidades podem descobrir:

```text id="x4m8q1"
species
location
technology
event
```

---

# 137. ENTITY-136 — Player/Mob/NPC Unification

A arquitetura final deve ser:

```text id="m5q7x2"
Entity
├── Player
├── NPC
├── Mob
├── Animal
├── Vehicle
├── Projectile
├── ItemEntity
└── Special Entity
```

Todos compartilham a infraestrutura.

---

# 138. ENTITY-137 — Special Entities

Permitir entidades especiais:

```text id="9x3m7q"
boss
world guardian
ancient machine
dimensional entity
```

sem alterar o Core.

---

# 139. ENTITY-138 — Entity API

Interfaces:

```text id="x5m2q8"
IEntity
ILivingEntity
IEntityComponent
IEntityQuery
IEntitySpawner
IEntityPersistence
IEntityNetwork
```

---

# 140. ENTITY-139 — Component API

```text id="m7q4x1"
IComponent
IComponentFactory
IComponentSerializer
IComponentSystem
```

---

# 141. ENTITY-140 — Registry

```text id="4x8m2q"
EntityRegistry
ComponentRegistry
```

---

# 142. ENTITY-141 — Mod API

Mods podem registrar:

```text id="m9q3x7"
EntityType
Component
Capability
SpawnRule
BehaviorProfile
```

---

# 143. ENTITY-142 — Official Content

Conteúdo oficial usa exatamente as mesmas APIs:

```text id="x5m8q1"
Official Mob
 ↓
Entity API

Mod Mob
 ↓
Entity API
```

---

# 144. ENTITY-143 — Missing Mod Entities

Se um mod desaparece:

```text id="4m7q2x"
unknown entity type
```

o save precisa ter uma política.

Possibilidades:

```text id="m7q3x9"
preserve placeholder
quarantine
convert/drop
block loading
```

conforme criticidade.

---

# 145. ENTITY-144 — Entity Migration

Atualização:

```text id="x4m8q1"
old component schema
 ↓
migration
 ↓
new component schema
```

---

# 146. ENTITY-145 — Entity Corruption

Criar:

```text id="m5q7x2"
EntityValidation
```

para detectar:

```text id="9x3m7q"
invalid position
missing type
invalid component
broken reference
```

---

# 147. ENTITY-146 — Quarantine

Entidade corrompida pode ser isolada:

```text id="x5m2q8"
ERROR
 ↓
QUARANTINED
```

sem necessariamente impedir o mundo inteiro de carregar.

---

# 148. ENTITY-147 — Debug Tools

Comandos:

```text id="m7q4x1"
nexora entity inspect
nexora entity list
nexora entity spawn
nexora entity remove
nexora entity teleport
nexora entity components
nexora entity query
```

---

# 149. ENTITY-148 — Entity Visualization

Modo debug:

```text id="4x8m2q"
entity IDs
bounds
components
state
LOD
network ownership
```

---

# 150. ENTITY-149 — Profiler

Métricas:

```text id="m9q3x7"
total entities
active entities
sleeping entities
AI entities
physics entities
rendered entities
spawn/despawn rate
query cost
```

---

# 151. ENTITY-150 — Stress Testing

Testar:

```text id="x5m8q1"
100 entities
1,000
10,000
100,000
1,000,000 abstract entities
```

O último caso pode representar população agregada, não necessariamente milhões de objetos completos em memória.

---

# 152. ENTITY-151 — Performance

O sistema deve usar:

```text id="4m7q2x"
spatial partitioning
component scheduling
sleeping
LOD
batch queries
object pooling where useful
```

---

# 153. ENTITY-152 — Pooling

Entidades temporárias podem usar:

```text id="m7q3x9"
Entity Pool
```

quando isso realmente reduzir custo de alocação.

---

# 154. ENTITY-153 — Batch Spawn

```text id="x4m8q1"
spawnBatch(...)
```

para:

```text id="m5q7x2"
structures
population
events
```

---

# 155. ENTITY-154 — Batch Remove

Mesma ideia.

---

# 156. ENTITY-155 — Thread Safety

Entity Manager deve controlar:

```text id="9x3m7q"
creation
destruction
component changes
queries
```

entre threads.

---

# 157. ENTITY-156 — Snapshot

Criar:

```text id="x5m2q8"
EntitySnapshot
```

para:

```text id="m7q4x1"
Renderer
AI
Networking
Physics
```

---

# 158. ENTITY-157 — Read / Write Separation

Sistemas podem receber:

```text id="4x8m2q"
IEntityReader
```

ou:

```text id="m9q3x7"
IEntityWriter
```

---

# 159. ENTITY-158 — Structural Changes

Adicionar/remover componente durante uma iteração não deve corromper o sistema.

Usar:

```text id="x5m8q1"
deferred structural changes
```

quando necessário.

---

# 160. ENTITY-159 — Event Ordering

Definir ordem:

```text id="m7q3x9"
Spawn
 ↓
Initialize
 ↓
Register
 ↓
Active
```

e:

```text id="4m8q2x"
Deactivate
 ↓
Save
 ↓
Remove
```

---

# 161. ENTITY-160 — Final Architecture

```text id="x5m8q2"
                           ENTITY SYSTEM
                                │
                 ┌──────────────┼──────────────┐
                 │              │              │
             IDENTITY         STATE         COMPONENTS
                 │              │              │
             EntityID       Transform       Physics
             TypeID         Lifecycle       Health
             UUID           Flags           AI
                                            Inventory
                                            Equipment
                                            Combat
                                            Fluid
                                            Energy
                 │              │              │
                 └──────────────┼──────────────┘
                                │
                        ENTITY MANAGER
                                │
              ┌─────────────────┼─────────────────┐
              │                 │                 │
            SPAWN            STREAMING          QUERY
              │                 │                 │
          Spawn Rules        Chunk/Region       Radius
          Population         LOD                Type
          Lifecycle          Sleep              Tag
              │                 │                 │
              └─────────────────┼─────────────────┘
                                │
        ┌───────────────────────┼────────────────────────┐
        │                       │                        │
      PLAYER                  MOB                      NPC
        │                       │                        │
      Skills                  Ecology                  Society
      Inventory               Behavior                  Knowledge
      Equipment               Reproduction              Profession
        │                       │                        │
        └───────────────────────┼────────────────────────┘
                                │
            ┌───────────────────┼────────────────────┐
            │                   │                    │
          PHYSICS             RENDERER            NETWORK
            │                   │                    │
          body                model                sync
          collision           animation            replication
          movement            audio refs            interest
            │                   │                    │
            └───────────────────┼────────────────────┘
                                │
                         WORLD SIMULATION
                                │
             ┌──────────────────┼──────────────────┐
             │                  │                  │
          ECOLOGY          CIVILIZATION          COMBAT
             │                  │                  │
         populations          society             damage
         migration            economy              AI
         reproduction         politics             status
```

# 162. Ordem de implementação

```text id="m7q3x9"
ENTITY-0 Core
ENTITY-1 Entity ID
ENTITY-2 Entity Type
ENTITY-3 Definition
ENTITY-4 Instance
ENTITY-5 Lifecycle
ENTITY-6 Spawn
ENTITY-7 Despawn
ENTITY-8 Persistence Policy
ENTITY-9 Components
ENTITY-10 Transform
ENTITY-11 Tags
ENTITY-12 Capabilities
ENTITY-13 Spatial Index
ENTITY-14 Queries
ENTITY-15 Chunk Integration
ENTITY-16 Streaming
ENTITY-17 Sleep/Wake
ENTITY-18 Scheduler
ENTITY-19 LivingEntity
ENTITY-20 Player Integration
ENTITY-21 Mob Integration
ENTITY-22 NPC Integration
ENTITY-23 Vehicle
ENTITY-24 Projectile
ENTITY-25 ItemEntity
ENTITY-26 BlockEntity
ENTITY-27 Health/Status
ENTITY-28 AI
ENTITY-29 Inventory
ENTITY-30 Equipment
ENTITY-31 Physics
ENTITY-32 Renderer
ENTITY-33 Animation
ENTITY-34 Audio
ENTITY-35 Combat
ENTITY-36 Ecology
ENTITY-37 Civilization
ENTITY-38 Persistence
ENTITY-39 Serialization
ENTITY-40 Migration
ENTITY-41 Networking
ENTITY-42 Replication
ENTITY-43 LOD
ENTITY-44 Debugging
ENTITY-45 Mod API
ENTITY-46 Stress Testing
```

# 163. Primeiro Vertical Slice

O primeiro teste deveria ser:

```text id="x4m8q1"
Entity Registry
 ↓
register entity type
 ↓
spawn
 ↓
Entity Manager
 ↓
Transform
 ↓
Chunk association
 ↓
Physics
 ↓
Renderer
 ↓
move entity
 ↓
cross chunk
 ↓
save
 ↓
unload
 ↓
reload
 ↓
entity restored
```

Depois:

```text id="m5q7x2"
Entity
 ↓
LivingEntity
 ↓
Health
 ↓
AI
 ↓
Mob
 ↓
Spawn
 ↓
Move
 ↓
Interact
 ↓
Death
 ↓
Loot
 ↓
Remove
```

E o teste realmente importante:

```text id="9x3m7q"
10.000 mobs
       ↓
FULL near player
       ↓
REGIONAL farther away
       ↓
ABSTRACT very far
       ↓
player travels
       ↓
rehydration
       ↓
individual mobs return
```

Isso é o que permite chegar aos **2.000+ tipos de mobs** sem tentar manter cada criatura do mundo com o mesmo nível de simulação o tempo todo.

## Regra arquitetural final

> **Entity System fornece identidade, existência, composição, localização e ciclo de vida. Os sistemas especializados dão significado à entidade.**

Então:

```text id="x5m8q1"
Entity
 ├── Physics → "como ela se move"
 ├── Renderer → "como ela aparece"
 ├── AI → "como ela decide"
 ├── Combat → "como ela luta"
 ├── Ecology → "como ela participa do ecossistema"
 ├── Civilization → "como ela participa da sociedade"
 ├── Inventory → "o que ela carrega"
 ├── Energy → "como ela usa energia"
 └── Fluid → "quais fluidos ela armazena"
```

Esse é o **elo que estava faltando entre o Core e praticamente todos os sistemas que você já colocou no NEXORA**. O repositório atual já possui os grandes planos de Player, Mob/Ecology/Civilization, Combat, Machines, etc.; o Entity System agora fecha a base conceitual entre eles.
