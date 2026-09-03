Sim. **Destruição / Construção** é um dos sistemas centrais do NEXORA, porque ele precisa transformar o voxel world em algo realmente manipulável pelo jogador, NPCs, máquinas e eventos do mundo.

Eu não colocaria “quebrar bloco” dentro do Player System. Faria um **Build & Destruction Engine** próprio.

```text id="r4m8x2"
                 BUILD / DESTRUCTION
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
     BUILDING         DESTRUCTION       STRUCTURE
        │                 │                 │
     Placement          Mining           Integrity
     Editing             Damage          Support
     Validation          Fracture        Collapse
     Schematics          Debris          Stability
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
              ┌───────────┼────────────┐
              │           │            │
            VOXELS      PHYSICS      WORLD
              │           │            │
           chunks      collisions    save
           states      forces        history
                          │
                    GAME SYSTEMS
```

# NEXORA — DESTRUCTION / CONSTRUCTION MASTER PLAN

## 1. Objetivo

O sistema precisa permitir:

```text
colocar
remover
substituir
copiar
mover
rotacionar
construir
demolir
danificar
reparar
escavar
terraformar
```

E também:

```text
estruturas grandes
construção automática
máquinas
pontes
ferrovias
cidades
cavernas
desabamentos
```

A regra principal:

> **Construção modifica o mundo de forma controlada. Destruição modifica o mundo de forma causal.**

---

# 2. BUILD-0 — Build Core

Criar o núcleo:

```text id="s7p2n4"
BuildEngine
BuildContext
BuildOperation
BuildResult
BuildRule
```

Toda modificação passa por esse sistema.

Fluxo:

```text id="j5x8c1"
Request
 ↓
Validate
 ↓
Reserve
 ↓
Apply
 ↓
Update Systems
 ↓
Commit
```

---

# 3. BUILD-1 — Placement

Operação básica:

```text id="2x9k4m"
placeBlock(position, blockState)
```

Mas nunca simplesmente:

```text
world.setBlock(...)
```

O sistema deve verificar:

```text id="8m4q6z"
posição válida
chunk disponível
permissão
colisão
suporte
orientação
regras do bloco
```

---

# 4. BUILD-2 — Placement Context

Criar:

```text id="w3n7p2"
PlacementContext
├── actor
├── position
├── face
├── rotation
├── tool
├── source
└── mode
```

Isso permite comportamento diferente dependendo da maneira como o bloco foi colocado.

---

# 5. BUILD-3 — Orientation

Blocos podem considerar:

```text id="1q6m8x"
face
rotation
axis
mirror
```

Exemplos:

```text id="8y5z2k"
pipe
machine
stairs
door
rail
```

---

# 6. BUILD-4 — Placement Rules

Cada tipo de bloco pode registrar:

```text id="7m3x9p"
canPlace
getPlacementState
validateSupport
getCollisionShape
```

O Build Engine executa as regras.

---

# 7. BUILD-5 — Support

Alguns blocos precisam de suporte.

```text id="4x8m2q"
Block
 ↓
Support Query
 ↓
valid?
```

Isso é importante para:

```text id="9p5k7x"
torches
plants
decorations
bridges
structures
```

---

# 8. BUILD-6 — Multi-Block Structures

Alguns sistemas não são um bloco:

```text id="m4x8q1"
Machine
Reactor
Door
Elevator
Large Structure
```

Criar:

```text id="8t2m6p"
MultiblockDefinition
MultiblockInstance
```

---

# 9. BUILD-7 — Multiblock Validation

```text id="2q9x5m"
Controller
 ↓
scan structure
 ↓
check pattern
 ↓
valid
 ↓
activate
```

Isso combina com:

```text id="0x3m7k"
machines
reactors
industrial systems
```

---

# 10. BUILD-8 — Breaking

Operação:

```text id="7m4x1q"
breakBlock(position)
```

Fluxo:

```text id="8q3n6z"
Request
 ↓
Validate
 ↓
Calculate resistance
 ↓
Apply damage
 ↓
Break
 ↓
Drops
 ↓
World Update
```

---

# 11. BUILD-9 — Block Hardness / Resistance

Um bloco pode possuir:

```text id="p5x7m2"
hardness
breakingTime
toolRequirement
```

Mas isso deve ser uma propriedade do conteúdo, não hardcoded no engine.

---

# 12. BUILD-10 — Tool Interaction

Ferramentas fornecem:

```text id="6m1q8p"
miningPower
miningSpeed
toolType
specialProperties
```

O Build Engine calcula o tempo/resultados.

---

# 13. BUILD-11 — Progressive Mining

Não quebrar instantaneamente.

```text id="w8x4m5"
start mining
 ↓
progress
 ↓
tool interaction
 ↓
block breaks
```

Permitir interrupção:

```text id="3q7n1k"
damage persists?
reset?
```

Essa regra pode ser configurável.

---

# 14. BUILD-12 — Mining State

Criar:

```text id="4x9m2p"
MiningOperation
├── actor
├── position
├── progress
├── tool
├── targetState
└── startedAt
```

---

# 15. BUILD-13 — Block Damage

Blocos podem visualizar danos:

```text id="z3m7q8"
damage 0 → 1
```

antes de quebrar.

O Renderer pode representar:

```text id="8p2x5m"
cracks
deformation
```

---

# 16. BUILD-14 — Tool Requirements

Exemplo conceitual:

```text id="q6m1x4"
requires:
  mining_tier >= X
```

O sistema verifica a ferramenta sem conhecer especificamente cada ferramenta.

---

# 17. BUILD-15 — Harvest Rules

Depois da destruição:

```text id="9x4m7q"
Block
 ↓
Harvest Rule
 ↓
Drop System
```

Assim:

```text id="6p2n8m"
block broken
→ determine drops
```

---

# 18. BUILD-16 — Silk/Precision-like Behavior

Não usar nomes de mecânicas específicas de outros jogos como arquitetura.

Criar genericamente:

```text id="5m8q2x"
HarvestModifier
```

permitindo efeitos como:

```text id="k7x3p4"
preserve target
increase drops
change drop table
```

---

# 19. BUILD-17 — Replacement

Permitir:

```text id="2x6m9q"
replaceBlock(A, B)
```

Útil para:

```text id="3m7x1k"
construction
upgrades
machines
terrain editing
```

---

# 20. BUILD-18 — Bulk Operations

Criar:

```text id="m8q4x1"
fill
replace
clear
clone
move
```

---

# 21. BUILD-19 — Transactions

Construções grandes não devem deixar metade do resultado aplicada.

```text id="7x2m5q"
BEGIN
 ↓
VALIDATE
 ↓
PREPARE
 ↓
COMMIT
```

Se falhar:

```text id="4m8q1x"
ROLLBACK
```

---

# 22. BUILD-20 — Build Transaction

```text id="m6x9p3"
BuildTransaction
├── operations[]
├── actor
├── source
├── permissions
├── cost
└── rollbackData
```

---

# 23. BUILD-21 — Atomic Building

Exemplo:

```text id="x8m2q5"
build bridge
 ↓
validate 500 blocks
 ↓
all valid
 ↓
commit
```

ou:

```text id="q3m7x1"
some invalid
 ↓
cancel
```

dependendo do modo.

---

# 24. BUILD-22 — Undo / Redo

O sistema deve manter:

```text id="5m9q2x"
operation history
```

para ferramentas/admin/debug.

Não necessariamente como uma função livre para survival gameplay.

---

# 25. BUILD-23 — History

Cada alteração importante pode registrar:

```text id="x4m7q8"
position
old state
new state
actor
source
timestamp
```

Isso também conversa com o World History System.

---

# 26. BUILD-24 — Player Building

O Player System fornece:

```text id="2m6q9x"
selected block
target position
rotation
```

Build Engine faz o trabalho real.

---

# 27. BUILD-25 — Ghost Preview

Antes de colocar:

```text id="x7m3q5"
Player
 ↓
Build Query
 ↓
Preview
```

Renderer mostra:

```text id="4n8q2m"
valid placement
invalid placement
orientation
```

---

# 28. BUILD-26 — Grid Alignment

A construção pode usar:

```text id="5x1m7q"
voxel grid
half-grid
custom snap
```

dependendo do módulo.

---

# 29. BUILD-27 — Rotation

Operações:

```text id="m8q4x6"
rotateX
rotateY
rotateZ
```

quando suportado pelo conteúdo.

---

# 30. BUILD-28 — Mirroring

Estruturas podem usar:

```text id="7q2m9x"
mirrorX
mirrorY
mirrorZ
```

---

# 31. BUILD-29 — Blueprint System

Criar:

```text id="9x4m2q"
Blueprint
```

que armazena:

```text id="m5q7x1"
relative blocks
metadata
orientation
dependencies
```

---

# 32. BUILD-30 — Schematics

Uma ferramenta pode capturar:

```text id="3m8q5x"
region
 ↓
schematic
```

e depois:

```text id="7x2m4q"
schematic
 ↓
placement
```

---

# 33. BUILD-31 — Construction Planner

NPC/máquinas podem usar:

```text id="q8m1x6"
ConstructionPlan
├── steps
├── materials
├── dependencies
└── progress
```

---

# 34. BUILD-32 — Automated Construction

Uma máquina ou NPC pode:

```text id="4x7m2q"
read blueprint
 ↓
check materials
 ↓
build
```

sem o player colocar cada bloco manualmente.

---

# 35. BUILD-33 — NPC Construction

Isso é muito importante para civilizações.

```text id="9m3x8q"
Civilization
 ↓
Construction Project
 ↓
Workers
 ↓
Build Engine
```

---

# 36. BUILD-34 — City Construction

Uma cidade pode construir:

```text id="2x6m4q"
houses
roads
bridges
walls
railways
pipes
factories
```

---

# 37. BUILD-35 — Dynamic Cities

Isso conecta com Civilization:

```text id="x8m1q5"
population increases
 ↓
city requires housing
 ↓
construction project
 ↓
materials
 ↓
workers
 ↓
new buildings
```

---

# 38. BUILD-36 — Repair

Criar:

```text id="5q7m2x"
RepairOperation
```

usada por:

```text id="6x3m8q"
players
NPCs
machines
infrastructure
```

---

# 39. BUILD-37 — Maintenance

Estruturas podem possuir:

```text id="m4x9q1"
maintenance state
```

e eventualmente precisar de reparo.

---

# 40. BUILD-38 — Structural System

Aqui começa a destruição estrutural.

Separar:

```text id="7x2m8q"
Voxel Destruction
```

de:

```text id="q4m6x1"
Structural Destruction
```

---

# 41. BUILD-39 — Structural Graph

Uma estrutura pode ser representada como:

```text id="8m3q7x"
Structure
├── nodes
├── supports
├── connections
└── loads
```

---

# 42. BUILD-40 — Support Graph

Exemplo:

```text id="2x5m8q"
foundation
 ↓
columns
 ↓
beam
 ↓
floor
```

Se uma conexão essencial é removida:

```text id="m7q3x4"
support lost
```

---

# 43. BUILD-41 — Stability

Calcular:

```text id="9x2m6q"
supported
unstable
critical
```

Não precisa começar com física estrutural totalmente realista.

Pode existir um modelo progressivo.

---

# 44. BUILD-42 — Structural Load

Elementos podem possuir:

```text id="x4m8q1"
loadCapacity
currentLoad
```

---

# 45. BUILD-43 — Collapse

Quando a estabilidade falha:

```text id="5m2x7q"
structure
 ↓
unstable
 ↓
collapse event
```

---

# 46. BUILD-44 — Collapse Scheduling

Não derrubar 50.000 blocos instantaneamente.

Usar:

```text id="m8q3x6"
collapse queue
```

com etapas.

---

# 47. BUILD-45 — Collapse LOD

Perto:

```text id="x7m4q2"
detailed
```

Longe:

```text id="3q8m1x"
abstract structural event
```

---

# 48. BUILD-46 — Debris

Estruturas destruídas podem gerar:

```text id="5m9x2q"
Debris
```

Mas não transformar cada pedaço em rigid body.

Usar:

```text id="x6m3q8"
logical debris
```

quando apropriado.

---

# 49. BUILD-47 — Debris Physics

Somente pedaços relevantes podem virar:

```text id="q4m8x1"
dynamic body
```

e interagir com Physics.

---

# 50. BUILD-48 — Fracture

Alguns materiais podem se fragmentar.

Criar:

```text id="9m2x5q"
FractureProfile
```

com:

```text id="x7q3m8"
threshold
pattern
debrisCount
```

---

# 51. BUILD-49 — Damage System

Destruição deve suportar:

```text id="4m6x2q"
mining
impact
fire
heat
pressure
fluid
structural stress
```

Mas cada causa pode ser um módulo.

---

# 52. BUILD-50 — Damage Event

```text id="m8q1x5"
DamageEvent
├── target
├── amount
├── source
├── type
├── position
└── direction
```

---

# 53. BUILD-51 — Block Damage Resistance

Um bloco pode ter:

```text id="7x3m9q"
resistanceProfile
```

para diferentes tipos de dano.

---

# 54. BUILD-52 — Tool Damage

Mining usa:

```text id="4q8m2x"
DamageType = Mining
```

---

# 55. BUILD-53 — Environmental Damage

Outros sistemas podem gerar:

```text id="m7x1q6"
heat
pressure
flood
freeze
```

e Build/Destruction interpreta somente a capacidade estrutural relevante.

---

# 56. BUILD-54 — Explosion Hook

Sem colocar lógica de explosão no Build Engine.

Criar:

```text id="9m4x7q"
DestructionImpulse
```

ou uma API equivalente.

Outro sistema pode produzir o evento; o Destruction System avalia o que é quebrado.

---

# 57. BUILD-55 — Terrain Deformation

O terreno pode ser modificado.

```text id="x5m8q2"
terrain
 ↓
edit volume
```

Suporte futuro a:

```text id="7q3m1x"
dig
fill
level
carve
raise
lower
```

---

# 58. BUILD-56 — Terraforming

O sistema precisa permitir alterações naturais ou de engenharia:

```text id="m2x9q5"
road
dam
canal
mine
tunnel
```

---

# 59. BUILD-57 — Large Terrain Operations

Operações grandes devem funcionar por batches:

```text id="4x7m8q"
10.000 voxel changes
 ↓
batch transaction
```

---

# 60. BUILD-58 — Chunk-Aware Editing

Se uma operação cruza chunks:

```text id="m6q2x9"
Chunk A
Chunk B
Chunk C
```

o sistema gerencia tudo como uma única operação lógica.

---

# 61. BUILD-59 — Cross-Chunk Structures

Pontes, túneis e cidades podem cruzar:

```text id="7m3x8q"
chunks
regions
```

sem perder identidade.

---

# 62. BUILD-60 — Structure Identity

Uma construção grande pode ter:

```text id="x4q7m2"
StructureID
```

para:

```text id="9m1x6q"
tracking
ownership
maintenance
history
damage
```

---

# 63. BUILD-61 — Ownership

Estruturas podem pertencer a:

```text id="m8x3q5"
player
NPC
village
city
organization
none
```

---

# 64. BUILD-62 — Permissions

Antes de alterar:

```text id="7q2m9x"
Permission System
 ↓
allowed?
```

Isso será importante para multiplayer.

---

# 65. BUILD-63 — Protection Zones

Algumas regiões podem proibir construção:

```text id="x5m7q1"
protected
restricted
public
private
```

---

# 66. BUILD-64 — Multiplayer Authority

No multiplayer:

```text id="4m8x2q"
Client
 ↓
Build Request
 ↓
Server validation
 ↓
Build Engine
 ↓
World
 ↓
Replication
```

Nunca aceitar simplesmente:

```text id="m7q3x9"
client says block placed
```

---

# 67. BUILD-65 — Build Prediction

Cliente pode mostrar:

```text id="x2m6q8"
preview
```

e até prever pequenas alterações.

Servidor continua autoridade.

---

# 68. BUILD-66 — Conflict Resolution

Se dois jogadores alteram a mesma posição:

```text id="5q9m2x"
Player A
Player B
 ↓
Server
 ↓
ordering / conflict rules
```

---

# 69. BUILD-67 — Network Deltas

Não enviar mundo inteiro.

```text id="m8x4q1"
Block Change Delta
Structure Change
Destruction Event
```

---

# 70. BUILD-68 — Persistence

Salvar alterações:

```text id="7m2x5q"
voxel state
structure state
damage state
construction history
```

---

# 71. BUILD-69 — Recovery

Uma operação interrompida não pode deixar:

```text id="x4q8m1"
half-built structure
```

quando deveria ser atômica.

---

# 72. BUILD-70 — Construction Resume

Projetos longos:

```text id="m6x3q9"
construction project
 ↓
50%
 ↓
save
 ↓
reload
 ↓
continue
```

---

# 73. BUILD-71 — Material Cost

Construções podem consumir:

```text id="7q1m8x"
wood
stone
metal
glass
etc.
```

A economia fornece disponibilidade.

---

# 74. BUILD-72 — Resource Reservation

Para projetos grandes:

```text id="x3m7q5"
Construction Plan
 ↓
reserve materials
 ↓
build
```

---

# 75. BUILD-73 — Failure

Construção pode falhar por:

```text id="m8q2x4"
missing materials
invalid placement
destroyed support
permission
construction interrupted
```

---

# 76. BUILD-74 — Construction Progress

```text id="9x1m7q"
planned
 ↓
materials gathered
 ↓
under construction
 ↓
completed
```

---

# 77. BUILD-75 — Construction AI

NPCs podem:

```text id="4m8x3q"
select project
gather material
move material
build
repair
```

---

# 78. BUILD-76 — Civilization Integration

```text id="m2q9x6"
Civilization
 ↓
Infrastructure Need
 ↓
Construction Project
 ↓
Build Engine
```

---

# 79. BUILD-77 — Economy Integration

Construção consome:

```text id="7x4m1q"
resources
labor
energy
transport
```

---

# 80. BUILD-78 — Railway Construction

O sistema deve suportar:

```text id="m8q3x5"
rail placement
station construction
bridge
tunnel
signal
```

---

# 81. BUILD-79 — Road Construction

Mesma infraestrutura para:

```text id="9m2x7q"
roads
paths
bridges
highways
```

---

# 82. BUILD-80 — Infrastructure Repair

Civilização pode reagir:

```text id="x5q8m1"
bridge damaged
 ↓
repair project
```

---

# 83. BUILD-81 — Natural Collapse

O mundo pode produzir:

```text id="4m7x2q"
cave collapse
rockfall
slope failure
```

mas o evento pode vir do:

```text id="m9q3x6"
Geology
```

---

# 84. BUILD-82 — Cave Integration

Cave Engine fornece:

```text id="x2m8q5"
cave geometry
support information
```

Destruction pode alterar:

```text id="7q4m1x"
walls
ceilings
supports
```

---

# 85. BUILD-83 — Deep World

No subterrâneo:

```text id="m6x9q2"
mining
tunneling
cities
support structures
```

podem alterar o mundo permanentemente.

---

# 86. BUILD-84 — Mining Networks

Mineração em larga escala pode criar:

```text id="8m3x7q"
shafts
tunnels
mines
railways
```

Isso conecta:

```text id="5q1m9x"
Mining
→ Construction
→ Infrastructure
```

---

# 87. BUILD-85 — Persistent Scars

Uma mina abandonada continua existindo.

```text id="m7x2q4"
Mine
 ↓
abandoned
 ↓
decay
 ↓
vegetation returns
```

---

# 88. BUILD-86 — Vegetation Regrowth

Destruição pode alimentar:

```text id="4x8m1q"
Vegetation System
```

e depois:

```text id="m5q7x2"
regrowth
```

---

# 89. BUILD-87 — Hydrology Integration

Construir:

```text id="x3m9q6"
dam
canal
tunnel
```

pode alterar:

```text id="7q2m8x"
water flow
```

O Fluid Engine recalcula.

---

# 90. BUILD-88 — Lighting Integration

Alterar uma parede:

```text id="m8x4q1"
cave wall removed
 ↓
Lighting
 ↓
new light path
```

---

# 91. BUILD-89 — Physics Integration

Destruição:

```text id="5m1x7q"
block removed
 ↓
Physics collision update
```

---

# 92. BUILD-90 — Renderer Integration

Construção/destruição:

```text id="x8q3m2"
Voxel changed
 ↓
Mesh dirty
 ↓
Renderer rebuild
```

---

# 93. BUILD-91 — Event Bus

Eventos:

```text id="m4x7q9"
BlockPlaced
BlockBroken
StructureBuilt
StructureDamaged
StructureCollapsed
TerrainModified
ConstructionStarted
ConstructionCompleted
RepairStarted
RepairCompleted
```

---

# 94. BUILD-92 — Build Sources

Uma alteração pode vir de:

```text id="7q1m8x"
Player
NPC
Machine
WorldGen
Terraforming
Natural Event
Mod
Admin
```

---

# 95. BUILD-93 — Operation Identity

Cada alteração importante recebe:

```text id="x5m2q8"
operationId
```

para:

```text id="m7q4x1"
logging
rollback
multiplayer
debug
duplication prevention
```

---

# 96. BUILD-94 — Anti-Duplication

Construções envolvendo itens:

```text id="8x3m9q"
consume materials
 ↓
commit build
```

de maneira transacional.

---

# 97. BUILD-95 — Atomic Inventory + Build

Isso é muito importante.

Não:

```text id="m2q7x4"
remove item
→ crash
→ block not placed
```

Usar uma transação entre:

```text id="9x1m8q"
Inventory
+
Build Engine
```

---

# 98. BUILD-96 — Blueprint Cost Validation

Antes de iniciar:

```text id="4m8x2q"
Blueprint
 ↓
calculate cost
 ↓
inventory/economy
 ↓
can afford?
```

---

# 99. BUILD-97 — Mod API

Mods podem registrar:

```text id="m7q3x5"
BuildRule
PlacementRule
BreakRule
DamageType
Structure
Blueprint
ConstructionMachine
StructuralMaterial
```

---

# 100. BUILD-98 — Debug

Comandos:

```text id="x2m6q9"
nexora build inspect
nexora build validate
nexora build preview
nexora build undo
nexora structure inspect
nexora destruction analyze
```

---

# 101. BUILD-99 — Structural Debug

Visualizar:

```text id="5q8m1x"
supports
loads
connections
unstable nodes
collapse zones
```

---

# 102. BUILD-100 — Performance

O sistema precisa evitar:

```text id="m4x9q2"
milhares de blocos
→ milhões de checks
```

Usar:

```text id="7x3q8m"
spatial partitioning
batch operations
dirty regions
async structural analysis
LOD
```

---

# 103. BUILD-101 — Large Structures

Testar:

```text id="1m8q5x"
house
tower
bridge
factory
city block
giant reactor
mega bridge
railway
underground city
```

---

# 104. BUILD-102 — Large Destruction

Testar:

```text id="x7m2q9"
mine
tunnel
dam
large structure
cave collapse
```

---

# 105. BUILD-103 — Stress Testing

Cenários:

```text id="4m8x1q"
1 block
1,000 blocks
10,000 blocks
100,000 blocks
1,000,000 logical block changes
```

com operações em lote.

---

# 106. BUILD-104 — Save Testing

```text id="m3q7x8"
build
 ↓
save
 ↓
crash simulation
 ↓
reload
 ↓
validate
```

---

# 107. BUILD-105 — Multiplayer Testing

```text id="5x9m2q"
2 players
10 players
50 players
```

realizando alterações próximas e distantes.

---

# 108. BUILD-106 — Determinism

Uma operação deve produzir o mesmo resultado com:

```text id="m8q4x2"
same world state
+
same operation
+
same build version
```

---

# 109. BUILD-107 — Final Architecture

```text id="q4m9x2"
                           BUILD / DESTRUCTION
                                   │
                ┌──────────────────┼──────────────────┐
                │                  │                  │
             BUILDING          MINING/DAMAGE      STRUCTURES
                │                  │                  │
            Placement           Breaking          Support
            Blueprint            Damage            Stability
            Construction         Harvest           Collapse
            Repair              Terrain            Fracture
                │                  │                  │
                └──────────────────┼──────────────────┘
                                   │
                         TRANSACTION SYSTEM
                                   │
                 ┌─────────────────┼────────────────┐
                 │                 │                │
              INVENTORY          WORLD           HISTORY
                 │                 │                │
              materials          voxels          operations
                                   │
               ┌───────────────────┼───────────────────┐
               │                   │                   │
            PHYSICS             FLUIDS              LIGHTING
               │                   │                   │
            collapse             water              shadows
               │                   │                   │
               └───────────────────┼───────────────────┘
                                   │
                              SIMULATION
                                   │
                       CIVILIZATION / ECONOMY
```

# 110. Ordem de implementação

```text id="m7q3x9"
BUILD-0 Core
BUILD-1 Placement
BUILD-2 Placement Context
BUILD-3 Orientation
BUILD-4 Placement Rules
BUILD-5 Support Rules
BUILD-6 Breaking
BUILD-7 Mining State
BUILD-8 Tool Integration
BUILD-9 Harvest
BUILD-10 Replacement
BUILD-11 Transactions
BUILD-12 Bulk Operations
BUILD-13 History
BUILD-14 Blueprint
BUILD-15 Schematics
BUILD-16 Structure System
BUILD-17 Multiblock
BUILD-18 Damage System
BUILD-19 Structural Graph
BUILD-20 Stability
BUILD-21 Collapse
BUILD-22 Debris
BUILD-23 Terrain Editing
BUILD-24 Construction Projects
BUILD-25 NPC Construction
BUILD-26 Repair/Maintenance
BUILD-27 Infrastructure
BUILD-28 Physics Integration
BUILD-29 Fluid Integration
BUILD-30 Lighting Integration
BUILD-31 Renderer Integration
BUILD-32 Multiplayer
BUILD-33 Persistence
BUILD-34 Mod API
BUILD-35 Debugging
BUILD-36 Stress Testing
```

# 111. Primeiro Vertical Slice

Eu começaria com:

```text id="x8m2q5"
Player
 ↓
select block
 ↓
placement preview
 ↓
validate
 ↓
consume item
 ↓
place voxel
 ↓
Chunk marked dirty
 ↓
Lighting update
 ↓
Physics update
 ↓
Renderer remesh
 ↓
save
```

Depois mineração:

```text id="m4q9x1"
Player
 ↓
tool
 ↓
break target
 ↓
damage progression
 ↓
block destroyed
 ↓
Drop System
 ↓
Inventory
 ↓
Chunk update
 ↓
Lighting
 ↓
Physics
 ↓
Renderer
```

E então o primeiro teste realmente especial do NEXORA:

```text id="7x3m8q"
BUILD BRIDGE
      ↓
structure graph
      ↓
remove support
      ↓
stability analysis
      ↓
collapse
      ↓
physics
      ↓
debris
      ↓
voxel changes
      ↓
fluid/lighting/renderer updates
      ↓
world saved
```

### Regra arquitetural principal

> **Construção e destruição são operações sobre o mundo, não funções do jogador.**

Então:

```text id="5q8m2x"
Player
NPC
Machine
World Event
Mod
Admin
```

todos podem solicitar:

```text id="m7x3q9"
BuildOperation
DestructionOperation
RepairOperation
TerraformOperation
```

e o mesmo motor valida e executa.

Isso é especialmente importante para o NEXORA porque, futuramente, **o próprio mundo poderá construir e destruir coisas sem o jogador estar presente**: cidades crescendo, ferrovias sendo expandidas, minas sendo escavadas, estruturas sofrendo desgaste, túneis colapsando e vegetação retomando áreas abandonadas.
