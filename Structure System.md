Perfeito. O **Structure System** é o sistema que vai cuidar de tudo que representa uma **construção composta por vários blocos, entidades ou componentes relacionados**.

A distinção mais importante é:

> **Block System sabe o que cada bloco é. Structure System sabe como vários elementos formam uma estrutura coerente.**

Isso permite desde uma árvore gigante até uma cidade subterrânea, uma ponte ferroviária, uma fábrica, uma ruína, uma nave parcialmente gerada ou uma megaestrutura dimensional sem colocar essa lógica dentro do Block System.

# NEXORA — STRUCTURE SYSTEM

## 1. Arquitetura central

```text id="str-01"
                    STRUCTURE SYSTEM
                           │
        ┌──────────────────┼──────────────────┐
        ↓                  ↓                  ↓
   DEFINITION          GENERATION          INSTANCE
        │                  │                  │
        ↓                  ↓                  ↓
    TEMPLATE            PLACEMENT          STATE
    PATTERN              RULES             HISTORY
    COMPONENTS           CONSTRAINTS       OWNERSHIP
```

Fluxo:

```text id="str-02"
StructureDefinition
        ↓
Structure Generator
        ↓
Placement Plan
        ↓
Validation
        ↓
Build Operation
        ↓
World / Blocks / Entities
        ↓
StructureInstance
        ↓
Persistence
```

---

# 2. O que é uma Structure?

Uma Structure é um conjunto coordenado de elementos que possui **identidade e significado como unidade**.

Exemplos:

```text id="str-03"
house
village
city
bridge
railway station
mine
factory
dungeon
temple
tower
fortress
port
dam
farm
giant tree
cave settlement
space station
```

E estruturas muito maiores:

```text id="str-04"
megacity
underground civilization
railway network node
space elevator
dimensional gate
mega factory
```

---

# 3. Structure ≠ Building

Uma Structure pode ser muito mais ampla que um prédio.

```text id="str-05"
Structure
├── natural
├── architectural
├── industrial
├── infrastructural
├── ecological
├── archaeological
├── civilization
├── dungeon
├── procedural
└── dimensional
```

---

# 4. Structure Definition

Define o que a estrutura pode ser.

```text id="str-06"
StructureDefinition

id
category
template
generationRules
placementRules
components
anchors
bounds
biomeRules
terrainRules
dimensionRules
rotationRules
mirrorRules
variationRules
lootRules
spawnRules
metadata
```

---

# 5. StructureID

Via Registry:

```text id="str-07"
nexora:oak_house
nexora:village
nexora:rail_station
examplemod:crystal_temple
```

---

# 6. Structure Registry

Criar:

```text id="str-08"
StructureRegistry
```

integrado ao Registry System.

---

# 7. Structure Definition ≠ Instance

Assim como Entity e Dimension:

```text id="str-09"
StructureDefinition
        │
        ├── Instance A
        ├── Instance B
        └── Instance C
```

Exemplo:

```text id="str-10"
nexora:house
```

pode resultar em:

```text id="str-11"
house at (100, 70, 200)
house at (830, 72, -40)
house at (1200, 68, 900)
```

---

# 8. StructureInstance

Representa a estrutura que realmente existe no mundo.

```text id="str-12"
StructureInstance

instanceId
definitionId
position
rotation
scale
variant
dimension
bounds
state
components
children
owner
generationSource
version
```

---

# 9. Structure Identity

Uma estrutura importante pode ter:

```text id="str-13"
PersistentStructureID
```

permitindo:

```text id="str-14"
"Central Railway Station"
```

ter identidade própria.

---

# 10. Structure State

Pode possuir:

```text id="str-15"
generated
constructed
damaged
abandoned
occupied
destroyed
repaired
upgraded
expanded
```

---

# 11. Structure Lifecycle

```text id="str-16"
DEFINED
 ↓
REGISTERED
 ↓
GENERATING
 ↓
PLACING
 ↓
VALIDATING
 ↓
ACTIVE
 ↓
DAMAGED
 ↓
ABANDONED
 ↓
DESTROYED
```

Nem toda estrutura passa por todos os estados.

---

# 12. Structure Composition

Uma estrutura não precisa ser apenas uma lista de blocos.

Pode conter:

```text id="str-17"
Blocks
BlockEntities
Entities
Items
Markers
Anchors
Components
Substructures
Links
Metadata
```

---

# 13. Structure Graph

Em vez de assumir uma árvore simples:

```text id="str-18"
StructureGraph
```

permite:

```text id="str-19"
City
 ├── District A
 │    ├── Houses
 │    ├── Roads
 │    └── Market
 ├── District B
 └── Railway Station
```

---

# 14. Substructure

Criar:

```text id="str-20"
SubStructure
```

Exemplo:

```text id="str-21"
Factory
├── Main Building
├── Warehouse
├── Power Plant
├── Cooling Tower
└── Rail Terminal
```

---

# 15. Structure Component

Componentes:

```text id="str-22"
StructureComponent
```

podem representar:

```text id="str-23"
building
road
rail
storage
machine
entrance
spawn
loot
faction
ownership
```

---

# 16. Structure Template

Template descreve a composição espacial.

```text id="str-24"
StructureTemplate
```

Pode conter:

```text id="str-25"
dimensions
palette
block states
entities
markers
anchors
metadata
```

---

# 17. Structure Palette

Assim como Chunk/Block Storage:

```text id="str-26"
0 = air
1 = stone
2 = wood
3 = glass
4 = door
```

---

# 18. Block Placement Data

Template usa:

```text id="str-27"
relativePosition
blockState
```

em vez de coordenadas absolutas.

---

# 19. Relative Coordinates

Exemplo:

```text id="str-28"
origin = (0,0,0)

door = (4,0,0)
wall = (0,0,0)
roof = (0,5,0)
```

---

# 20. Anchors

Muito importante.

Uma estrutura pode ter pontos especiais:

```text id="str-29"
StructureAnchor

entrance
exit
spawn
road_connection
rail_connection
portal
foundation
center
```

---

# 21. Anchor Connections

Estruturas podem ser conectadas.

```text id="str-30"
Village
  ↓ road anchor
House
  ↓ road anchor
Market
```

---

# 22. Structure Ports

Podemos generalizar anchors para:

```text id="str-31"
StructurePort
```

tipos:

```text id="str-32"
road
rail
pipe
power
fluid
entrance
teleport
communication
```

---

# 23. Structure Connectors

Um sistema pode perguntar:

```text id="str-33"
canConnect(A, B)
```

e determinar compatibilidade.

---

# 24. Structure Placement

O Structure System gera um:

```text id="str-34"
StructurePlacementPlan
```

antes de alterar o mundo.

---

# 25. Placement Plan

Contém:

```text id="str-35"
blocks
blockEntities
entities
components
connections
metadata
```

---

# 26. Validation Before Placement

Verificar:

```text id="str-36"
terrain
space
collision
biome
dimension
ownership
overlap
support
environment
```

---

# 27. Structure Placement ≠ Build

Structure System decide:

```text id="str-37"
onde e qual estrutura
```

Build & Destruction executa:

```text id="str-38"
a mudança real no mundo
```

Portanto:

```text id="str-39"
Structure
 ↓
Placement Plan
 ↓
Build & Destruction
 ↓
World
```

---

# 28. Atomic Placement

Estruturas críticas podem ser colocadas como transação.

```text id="str-40"
validate
 ↓
reserve
 ↓
apply
 ↓
commit
```

---

# 29. Large Structures

Uma cidade pode conter milhões de blocos.

Nunca fazer:

```text id="str-41"
1 giant blocking operation
```

no thread principal.

---

# 30. Chunk-aware Placement

Dividir por chunks:

```text id="str-42"
Structure
 ↓
Chunk 1
Chunk 2
Chunk 3
...
```

---

# 31. Streaming Generation

Estrutura pode ser gerada em partes conforme chunks aparecem.

---

# 32. Structure Bounding Box

Cada estrutura possui:

```text id="str-43"
AABB
```

ou volume equivalente.

---

# 33. Structure Bounds

Usado para:

```text id="str-44"
overlap
queries
streaming
collision
ownership
rendering
generation
```

---

# 34. Structure Footprint

Separar:

```text id="str-45"
visual volume
physical footprint
generation footprint
```

Uma torre pode ocupar pouco terreno mas possuir grande altura.

---

# 35. Rotation

Suportar:

```text id="str-46"
0°
90°
180°
270°
```

ou transformações mais gerais quando necessário.

---

# 36. Mirror

Templates podem ser espelhados:

```text id="str-47"
none
x
z
xz
```

---

# 37. Scale

Algumas estruturas podem ser escaladas.

Mas escala arbitrária pode ser problemática para blocos voxel.

Preferir variantes discretas:

```text id="str-48"
small
medium
large
mega
```

ou regras específicas.

---

# 38. Procedural Structure Variation

A mesma estrutura pode ter variantes:

```text id="str-49"
house_small
house_medium
house_large
house_ruined
house_rich
house_poor
```

sem transformar cada variante em um sistema diferente.

---

# 39. Structure Variant

```text id="str-50"
StructureVariant

base
palette
module overrides
rules
```

---

# 40. Weighted Variants

Durante geração:

```text id="str-51"
variant A = 50%
variant B = 30%
variant C = 20%
```

---

# 41. Deterministic Selection

Usar:

```text id="str-52"
world seed
structure position
structure id
world generation version
```

para seleção determinística.

---

# 42. Structure Generator

```text id="str-53"
IStructureGenerator
```

opera sobre:

```text id="str-54"
GenerationContext
```

---

# 43. Generation Context

```text id="str-55"
StructureGenerationContext

seed
position
biome
terrain
climate
dimension
worldPhase
neighborStructures
```

---

# 44. Structure Placement Rules

Exemplos:

```text id="str-56"
minAltitude
maxAltitude
allowedBiome
allowedDimension
slope
nearWater
nearRoad
distanceFromSettlement
```

---

# 45. Terrain Validation

Uma casa pode exigir:

```text id="str-57"
slope < threshold
```

---

# 46. Water Validation

Porto:

```text id="str-58"
nearWater = true
```

---

# 47. Cave Validation

Estrutura subterrânea pode exigir:

```text id="str-59"
depth
cave volume
rock type
```

---

# 48. Underground Structures

NEXORA precisa de estruturas subterrâneas reais:

```text id="str-60"
mines
underground cities
rail tunnels
cavern settlements
research stations
```

---

# 49. Vertical Placement

Estruturas podem ter:

```text id="str-61"
surface
underground
deep
floating
underwater
```

---

# 50. Floating Structures

Exemplo:

```text id="str-62"
sky island
airship dock
floating city
```

---

# 51. Underwater Structures

```text id="str-63"
ocean station
underwater city
submarine dock
```

---

# 52. Deep World Structures

```text id="str-64"
underground civilization
deep fortress
subterranean factory
```

---

# 53. Dimensional Structures

```text id="str-65"
void station
dimensional temple
alien megastructure
```

---

# 54. Biome-aware Structures

Structure pode consultar:

```text id="str-66"
BiomeDefinition
```

mas Biome Engine continua independente.

---

# 55. Climate-aware Structures

Exemplo:

```text id="str-67"
desert city
```

pode utilizar:

```text id="str-68"
water infrastructure
```

---

# 56. Structure Adaptation

Em vez de apenas aceitar/rejeitar:

```text id="str-69"
StructureAdapter
```

pode modificar:

```text id="str-70"
foundation
roof
entrance
materials
```

dependendo do terreno.

---

# 57. Foundation Solver

Casa em encosta:

```text id="str-71"
terrain
 ↓
foundation solver
 ↓
adapted structure
```

---

# 58. Terrain Conforming

Estruturas podem adaptar partes à topografia.

---

# 59. Road Conforming

Estradas podem seguir:

```text id="str-72"
terrain
slope
river
mountains
```

---

# 60. Bridge Generation

Bridge Generator:

```text id="str-73"
anchor A
 ↓
span calculation
 ↓
anchor B
 ↓
bridge structure
```

---

# 61. Procedural Roads

Road system pode usar:

```text id="str-74"
StructureGraph
```

para criar:

```text id="str-75"
settlement
 ↓
road
 ↓
settlement
```

---

# 62. Railway

Mesmo:

```text id="str-76"
rail station
rail bridge
rail tunnel
```

---

# 63. Structure Networks

Estruturas podem se conectar através de redes:

```text id="str-77"
City
 ↔ Road
 ↔ Railway
 ↔ Factory
 ↔ Port
```

Mas o sistema de transporte administra a rede.

---

# 64. Structure Graph ≠ Transportation Graph

Structure diz:

```text id="str-78"
qual estrutura conecta a qual
```

Transport diz:

```text id="str-79"
como o tráfego funciona
```

---

# 65. Civilization Structures

Civilization System pode criar:

```text id="str-80"
houses
farms
markets
administrative buildings
walls
roads
```

---

# 66. NPC Construction

NPCs podem solicitar:

```text id="str-81"
ConstructStructureRequest
```

Structure System monta o plano.

Build executa.

---

# 67. Structure Ownership

Uma estrutura pode possuir:

```text id="str-82"
owner
faction
settlement
organization
player
```

---

# 68. Ownership Rules

Structure System registra.

Protection/Civilization define regras.

---

# 69. Structure Damage

Uma estrutura pode possuir:

```text id="str-83"
health/state
```

mas o dano físico pertence a:

```text id="str-84"
Build & Destruction
```

---

# 70. Structure Integrity

O sistema pode acompanhar:

```text id="str-85"
required components
critical blocks
```

---

# 71. Structural Failure

Se parte crítica sumiu:

```text id="str-86"
structure integrity
 ↓
degraded
```

Build/Physics decide colapso.

---

# 72. Structure Repairs

NPC/player pode:

```text id="str-87"
repair structure
```

através de Build/Crafting.

---

# 73. Structure Upgrade

Civilization pode:

```text id="str-88"
House Tier 1
 ↓
Tier 2
 ↓
Tier 3
```

Structure System aplica uma transformação.

---

# 74. Structure Evolution

Cidades podem crescer:

```text id="str-89"
village
 ↓
town
 ↓
city
 ↓
metropolis
```

Structure Graph adiciona estruturas.

---

# 75. Structure Archetypes

Criar:

```text id="str-90"
House
Farm
Market
Temple
Factory
Station
Port
Fortress
```

como categorias.

---

# 76. Natural Structures

Também:

```text id="str-91"
giant tree
coral colony
crystal formation
volcanic formation
ice cave
```

---

# 77. Natural vs Artificial

Não precisa de systems separados.

```text id="str-92"
StructureDefinition.category
```

pode indicar:

```text id="str-93"
NATURAL
ARTIFICIAL
HYBRID
```

---

# 78. Archaeological Structures

Estruturas antigas:

```text id="str-94"
ruins
collapsed city
ancient machinery
fossil site
```

---

# 79. Historical Structures

World History pode registrar:

```text id="str-95"
who built it
when
why
```

---

# 80. Structure History

Estrutura importante pode ter:

```text id="str-96"
createdAt
builder
events
ownershipChanges
destruction
repairs
```

---

# 81. Structure History ≠ Every Block Change

Não registrar cada bloco em histórico de alto nível.

---

# 82. Structure Markers

Template pode possuir:

```text id="str-97"
Marker

name
position
type
metadata
```

Tipos:

```text id="str-98"
spawn
loot
door
road
rail
portal
machine
NPC
quest
```

---

# 83. Marker Processing

Depois que a estrutura foi colocada:

```text id="str-99"
markers
 ↓
specialized systems
```

---

# 84. Structure Entities

Estrutura pode spawnar:

```text id="str-100"
NPC
mob
machine entity
vehicle
```

mas Entity System controla as entidades.

---

# 85. Structure Loot

Pode definir:

```text id="str-101"
loot markers
```

Loot System decide recompensas.

---

# 86. Structure Spawn Rules

Exemplo:

```text id="str-102"
village
→ NPC population seed
```

Civilization System decide a população.

---

# 87. Structure Machine Layout

Fábrica pode colocar:

```text id="str-103"
machine blocks
energy
fluid
storage
```

Machine System ativa.

---

# 88. Structure Template with Dependencies

Uma estrutura pode requerer:

```text id="str-104"
block
item
entity
machine
```

Registry resolve.

---

# 89. Missing Content

Se uma estrutura depender de um mod:

```text id="str-105"
missing block
```

o placement pode:

```text id="str-106"
abort
fallback
replace
quarantine
```

dependendo da política.

---

# 90. Structure Fallback

Exemplo:

```text id="str-107"
modded steel block
 ↓
fallback metal block
```

quando explicitamente configurado.

---

# 91. Structure Compatibility

Uma definição deve declarar:

```text id="str-108"
requiredContent
optionalContent
fallbacks
```

---

# 92. Structure Version

```text id="str-109"
structureVersion
templateVersion
generationVersion
```

---

# 93. Persistent Structure

Estruturas geradas podem possuir:

```text id="str-110"
generated=true
```

e persistir.

---

# 94. Procedural Regeneration

Cuidado:

Uma estrutura gerada não deve ser simplesmente regenerada sobre modificações do jogador.

---

# 95. Base + Delta

Modelo:

```text id="str-111"
Structure Base
+
World Changes
=
Current Structure
```

---

# 96. Structure Delta

Pode registrar:

```text id="str-112"
removed blocks
added blocks
changed blocks
component state
```

quando necessário.

---

# 97. Player-built Structure

Também pode ser registrada:

```text id="str-113"
StructureOrigin
    GENERATED
    PLAYER
    NPC
    SYSTEM
    EVENT
```

---

# 98. Structure Discovery

O jogador pode descobrir:

```text id="str-114"
unknown structure
```

Knowledge/Quest pode registrar isso.

---

# 99. Structure Discovery Event

```text id="str-115"
StructureDiscoveredEvent
```

---

# 100. Structure Scan

Scanner pode consultar:

```text id="str-116"
IStructureQuery
```

---

# 101. Structure Query

Permitir:

```text id="str-117"
findStructure()
findNearest()
findByType()
findByTag()
findByOwner()
findInBounds()
```

---

# 102. Spatial Index

Estruturas precisam de índice espacial.

```text id="str-118"
StructureSpatialIndex
```

---

# 103. Why?

Para não procurar em todas as estruturas do mundo.

---

# 104. Structure Tags

```text id="str-119"
#settlement
#industrial
#railway
#dungeon
#natural
#ancient
```

---

# 105. Query Example

```text id="str-120"
findNearest(
    tag = #railway,
    position = playerPosition
)
```

---

# 106. Structure Registry vs Spatial Index

```text id="str-121"
Registry
→ definitions

Spatial Index
→ instances
```

---

# 107. Structure Graph Registry

Definitions podem declarar conexões disponíveis.

Instances mantêm conexões reais.

---

# 108. Structure Links

```text id="str-122"
StructureLink

source
target
type
anchorA
anchorB
state
```

---

# 109. Link Types

```text id="str-123"
ROAD
RAIL
PIPE
ENERGY
FLUID
PORTAL
COMMUNICATION
LOGISTICS
```

---

# 110. Structure Network

Uma cidade pode virar:

```text id="str-124"
Structure Graph
```

com:

```text id="str-125"
houses
roads
markets
stations
factories
```

---

# 111. Civilization Evolution

O Civilization System pode avaliar:

```text id="str-126"
population
wealth
food
infrastructure
security
```

e solicitar novas estruturas.

---

# 112. Structure Planner

Criar:

```text id="str-127"
IStructurePlanner
```

para planejar expansão.

---

# 113. Structure Planner

Entrada:

```text id="str-128"
settlement state
terrain
resources
population
rules
```

Saída:

```text id="str-129"
expansion plan
```

---

# 114. Structure Planner ≠ Civilization AI

Civilization decide:

```text id="str-130"
"precisamos de um mercado"
```

Planner decide:

```text id="str-131"
"este local é adequado"
```

Build decide:

```text id="str-132"
"como construir"
```

---

# 115. Structure Planning Pipeline

```text id="str-133"
Civilization
 ↓
Need
 ↓
Structure Planner
 ↓
Placement Plan
 ↓
Build
 ↓
Structure Instance
```

---

# 116. Natural Structure Planning

WorldGen pode solicitar:

```text id="str-134"
giant tree
```

mas Dynamic Vegetation pode posteriormente simular sua vida.

---

# 117. Structure Generation Phases

```text id="str-135"
MACRO
 ↓
MAJOR
 ↓
LOCAL
 ↓
DETAIL
```

---

# 118. Macro Structures

```text id="str-136"
continents
major cities
mountain facilities
```

---

# 119. Major Structures

```text id="str-137"
cities
large ruins
factories
rail hubs
```

---

# 120. Local Structures

```text id="str-138"
houses
barns
small caves
```

---

# 121. Detail Structures

```text id="str-139"
signs
small shrines
campfires
decorations
```

---

# 122. Structure Generation Priority

No WorldGen:

```text id="str-140"
terrain
 ↓
major structures
 ↓
local structures
 ↓
detail
```

---

# 123. Collision

Structure placement deve consultar:

```text id="str-141"
World
 ↓
Collision / Voxel
```

---

# 124. Overlap Policies

Quando duas estruturas se sobrepõem:

```text id="str-142"
REJECT
PRIORITIZE
MERGE
ADAPT
OVERWRITE
```

---

# 125. Priority

Exemplo:

```text id="str-143"
major city > small house
```

---

# 126. Merge

Duas estruturas podem compartilhar:

```text id="str-144"
road
wall
foundation
```

---

# 127. Structure Compatibility

Cada StructureDefinition pode declarar:

```text id="str-145"
compatibleWith
incompatibleWith
mergeRules
```

---

# 128. Terrain Carving

Algumas estruturas podem exigir:

```text id="str-146"
terrain excavation
```

como:

```text id="str-147"
mine
tunnel
railway
underground base
```

---

# 129. Terraforming

Build & Destruction executa a alteração de terreno.

Structure System planeja.

---

# 130. Water Avoidance

Structures podem:

```text id="str-148"
avoid water
cross water
float
drain area
```

Mas Fluid System executa consequências.

---

# 131. Structure Environment Requirements

```text id="str-149"
temperature
humidity
pressure
depth
fluid presence
```

---

# 132. Dimension Rules

```text id="str-150"
allowedDimensions
```

Exemplo:

```text id="str-151"
space station
→ vacuum dimension
```

---

# 133. Structure Biome Rules

```text id="str-152"
allowedBiomeTags
forbiddenBiomeTags
```

---

# 134. Seasonal Structures

Algumas estruturas podem variar:

```text id="str-153"
winter camp
summer festival
```

mas o calendário vem de Climate/World Time.

---

# 135. Event Structures

World Events podem criar:

```text id="str-154"
temporary crater
event camp
meteor site
dimensional anomaly
```

---

# 136. Temporary Structure

```text id="str-155"
expirationTime
cleanupPolicy
```

---

# 137. Persistent Event Structure

Outra pode permanecer para sempre:

```text id="str-156"
meteor crater
```

---

# 138. Structure Cleanup

Temporary structures devem possuir:

```text id="str-157"
destroy
restore
leaveScar
```

---

# 139. Structure Destruction

Uma estrutura pode ser destruída parcialmente.

```text id="str-158"
Structure
 ↓
damage
 ↓
partially destroyed
```

---

# 140. Rebuilding

NPCs podem reconstruir:

```text id="str-159"
current state
 ↓
missing components
 ↓
construction plan
```

---

# 141. Structure Blueprint

Criar:

```text id="str-160"
StructureBlueprint
```

usável por:

```text id="str-161"
Player
NPC
WorldGen
Civilization
```

---

# 142. Blueprint vs Template

### Template

Como a estrutura é definida.

### Blueprint

Plano executável de construção.

---

# 143. Schematic

Schematic pode ser:

```text id="str-162"
portable representation
```

para copiar/mover estruturas.

---

# 144. Copy/Paste

Build System pode executar:

```text id="str-163"
capture
validate
place
```

via Structure API.

---

# 145. Structure Capture

Uma estrutura existente pode ser convertida em:

```text id="str-164"
StructureTemplate
```

---

# 146. Structure Save

Uma estrutura importante pode ser salva independentemente do chunk:

```text id="str-165"
StructureSnapshot
```

---

# 147. Structure Snapshot

Inclui:

```text id="str-166"
blocks
block entities
entities references
components
metadata
```

---

# 148. Structure Snapshot vs World Save

World Save:

```text id="str-167"
estado do mundo
```

Structure Snapshot:

```text id="str-168"
unidade reutilizável/diagnosticável
```

---

# 149. Persistence

StructureInstance deve possuir serializer.

```text id="str-169"
IStructureSerializer
```

---

# 150. Migration

```text id="str-170"
Structure v1
 ↓
Migration
 ↓
Structure v2
```

---

# 151. Mod Support

Mods registram:

```text id="str-171"
StructureDefinition
StructureTemplate
StructureGenerator
PlacementRule
StructureComponent
StructurePort
```

---

# 152. Official Structures

Conteúdo oficial utiliza a mesma API:

```text id="str-172"
Public Structure API
       ↓
Vanilla
+
Mods
```

---

# 153. Structure Generator Mod

Um mod pode criar:

```text id="str-173"
custom procedural generator
```

sem alterar WorldGen Core.

---

# 154. Generator Sandbox

Generators de mods devem possuir limites:

```text id="str-174"
generation budget
memory budget
time budget
```

---

# 155. Generation Failure

Se um mod falhar:

```text id="str-175"
structure generation error
```

o chunk não deve necessariamente corromper.

---

# 156. Retry Policy

Pode haver:

```text id="str-176"
retry
skip
fallback
```

dependendo da estrutura.

---

# 157. Structure Generation Ticket

Criar:

```text id="str-177"
StructureGenerationTicket
```

para tarefas assíncronas.

---

# 158. Async Structure Generation

```text id="str-178"
WorldGen Worker
 ↓
Structure Generator
 ↓
Placement Plan
 ↓
Main World Commit
```

---

# 159. Thread Safety

Template/Definition:

```text id="str-179"
immutable
```

durante runtime.

---

# 160. Structure Instance Mutations

Passam por comandos/operations controladas.

---

# 161. Structure Event Bus

Eventos:

```text id="str-180"
StructureGenerated
StructurePlaced
StructureActivated
StructureDamaged
StructureRepaired
StructureDestroyed
StructureExpanded
StructureUpgraded
StructureDiscovered
StructureOwnershipChanged
```

---

# 162. Structure Request Events

```text id="str-181"
StructurePlacementRequested
StructureRepairRequested
StructureUpgradeRequested
```

---

# 163. Structure Event Ordering

```text id="str-182"
Request
 ↓
Validation
 ↓
Planning
 ↓
Build Operation
 ↓
Structure State
 ↓
Fact Event
```

---

# 164. Structure Debug

```text id="str-183"
nexora structure list
nexora structure inspect
nexora structure find
nexora structure spawn
nexora structure capture
nexora structure export
nexora structure validate
```

---

# 165. Structure Inspector

Mostrar:

```text id="str-184"
instance ID
definition
position
bounds
owner
state
components
connections
children
generation source
```

---

# 166. Structure Graph Viewer

Visualizar:

```text id="str-185"
City
 ├── House
 ├── Market
 ├── Factory
 └── Railway Station
```

---

# 167. Structure Bounds Debug

Visualizar:

```text id="str-186"
bounding box
anchors
ports
connections
```

---

# 168. Structure Generation Debug

Mostrar:

```text id="str-187"
candidate
accepted
rejected
reason
```

Isso será extremamente útil para debug de WorldGen.

---

# 169. Structure Placement Report

Exemplo:

```text id="str-188"
Village rejected
Reason:
terrain slope > limit
```

---

# 170. Structure Profiler

Medir:

```text id="str-189"
generation time
placement time
validation time
block count
entity count
memory
```

---

# 171. Stress Test

```text id="str-190"
1,000 structures
10,000
100,000
```

---

# 172. Large Structure Test

```text id="str-191"
1 mega-city
10 million blocks
```

ou escala equivalente conforme o engine.

---

# 173. Concurrent Generation Test

```text id="str-192"
1,000 chunks
+
many structures
```

geradas em paralelo.

---

# 174. Structure Graph Stress

```text id="str-193"
100,000 structure instances
```

com conexões.

---

# 175. Persistence Stress

Salvar:

```text id="str-194"
1 million structure instances
```

sintéticas para testar o framework.

---

# 176. Determinism

Mesma:

```text id="str-195"
seed
position
generator version
definition
```

deve resultar na mesma estrutura inicial.

---

# 177. Golden Structure Tests

Salvar estruturas esperadas:

```text id="str-196"
house
village
bridge
factory
cave settlement
```

e comparar.

---

# 178. Placement Tests

Testar:

```text id="str-197"
flat terrain
slope
water
cave
chunk border
dimension boundary
```

---

# 179. Cross-Chunk Structure Test

Estrutura que atravessa:

```text id="str-198"
Chunk A
│
└──── Structure ────┐
                    │
                  Chunk B
```

deve funcionar.

---

# 180. Cross-Region Structure Test

Megaestruturas atravessando regiões.

---

# 181. Partial Loading

Estrutura enorme pode ser:

```text id="str-199"
partially loaded
```

conforme chunks.

---

# 182. Streaming State

```text id="str-200"
FULL
PARTIAL
ABSTRACT
```

---

# 183. Structure LOD

Muito importante para megacidades.

```text id="str-201"
FULL
REGIONAL
ABSTRACT
```

---

# 184. Full

Próximo do jogador:

```text id="str-202"
blocks
block entities
entities
machines
```

---

# 185. Regional

Mais distante:

```text id="str-203"
building summary
population
production
infrastructure
```

---

# 186. Abstract

Muito distante:

```text id="str-204"
structure type
location
health
economic role
```

---

# 187. Rehydration

```text id="str-205"
ABSTRACT
 ↓
REGIONAL
 ↓
FULL
```

quando a área se torna relevante.

---

# 188. Structure Simulation

Structure System não simula:

```text id="str-206"
economy
machines
AI
```

mas fornece a identificação espacial necessária.

---

# 189. Structure + Civilization

Civilization utiliza:

```text id="str-207"
structure graph
```

para entender infraestrutura.

---

# 190. Structure + Economy

Economy pode associar:

```text id="str-208"
factory
market
warehouse
port
```

---

# 191. Structure + Machine

Machine System encontra:

```text id="str-209"
machines inside structure
```

---

# 192. Structure + Energy

Estruturas industriais podem conter:

```text id="str-210"
energy networks
```

---

# 193. Structure + Fluid

Infraestrutura pode conter:

```text id="str-211"
pipes
tanks
waterways
```

---

# 194. Structure + Rail

Railway System associa:

```text id="str-212"
station
rail segment
depot
bridge
tunnel
```

---

# 195. Structure + Vehicles

Vehicle System pode reconhecer:

```text id="str-213"
station
garage
dock
landing pad
```

---

# 196. Structure + Quest

Quest pode apontar para:

```text id="str-214"
specific structure
structure type
structure instance
```

---

# 197. Structure + Knowledge

Player/NPC pode descobrir:

```text id="str-215"
ancient structure
```

e gerar conhecimento.

---

# 198. Structure + History

World History registra:

```text id="str-216"
settlement founded
factory built
city destroyed
bridge repaired
```

---

# 199. Structure + UI

UI pode mostrar:

```text id="str-217"
Structure Name
Owner
Status
Population
Function
Connections
```

---

# 200. Structure + Map

Map pode renderizar:

```text id="str-218"
cities
stations
dungeons
landmarks
```

---

# 201. Structure + Audio

Audio recebe:

```text id="str-219"
structure audio profile
```

para soundscapes de:

```text id="str-220"
factory
city
station
cave
```

---

# 202. Structure + Animation

Machines/NPCs dentro da estrutura usam Animation normalmente.

Structure não anima nada diretamente.

---

# 203. Structure + Block

```text id="str-221"
StructureTemplate
 ↓
BlockStates
 ↓
Block System
```

---

# 204. Structure + Entity

```text id="str-222"
Structure
 ↓
spawn/associate
 ↓
Entity System
```

---

# 205. Structure + Item

Estruturas podem conter:

```text id="str-223"
loot
storage
item entities
```

Item System administra os itens.

---

# 206. Structure + Dimension

StructureDefinition pode dizer:

```text id="str-224"
allowed dimensions
```

---

# 207. Structure + World Generation

```text id="str-225"
WorldGen
 ↓
Structure System
 ↓
Structure Placement
 ↓
Build / World
```

---

# 208. Structure + Persistence

```text id="str-226"
StructureInstance
 ↓
Persistence
```

---

# 209. Structure + Registry

```text id="str-227"
StructureRegistry
 ↓
Definition
```

---

# 210. Structure + Event Bus

```text id="str-228"
Structure events
 ↓
Event Bus
```

---

# 211. Structure API

Interfaces:

```text id="str-api-01"
IStructure
IStructureDefinition
IStructureInstance
IStructureTemplate
IStructureVariant
IStructureComponent
IStructureAnchor
IStructurePort
IStructureLink
IStructureGenerator
IStructurePlanner
IStructurePlacement
IStructureQuery
IStructureSerializer
```

---

# 212. Structure Registry API

```text id="str-api-02"
IStructureRegistry

register()
get()
contains()
iterate()
```

---

# 213. Structure Generation API

```text id="str-api-03"
IStructureGenerator

canGenerate()
generate()
```

---

# 214. Placement API

```text id="str-api-04"
IStructurePlacement

validate()
plan()
commit()
rollback()
```

---

# 215. Query API

```text id="str-api-05"
IStructureQuery

nearest()
byType()
byTag()
byOwner()
inBounds()
connectedTo()
```

---

# 216. Planner API

```text id="str-api-06"
IStructurePlanner

planExpansion()
findLocation()
evaluate()
```

---

# 217. Structure Runtime

```text id="str-api-07"
StructureRuntime

generate()
place()
update()
load()
unload()
```

---

# 218. Structure Context

```text id="str-api-08"
StructureContext

definition
instance
world
dimension
registry
events
persistence
```

---

# 219. Código

Eu organizaria assim:

```text id="str-code-01"
src/
└── structure/
    ├── core/
    │   ├── structure.ts
    │   ├── structure-definition.ts
    │   ├── structure-instance.ts
    │   ├── structure-id.ts
    │   └── structure-context.ts
    │
    ├── template/
    │   ├── structure-template.ts
    │   ├── palette.ts
    │   ├── marker.ts
    │   └── variant.ts
    │
    ├── component/
    │   ├── structure-component.ts
    │   ├── building.ts
    │   ├── infrastructure.ts
    │   ├── ownership.ts
    │   └── function.ts
    │
    ├── anchor/
    │   ├── anchor.ts
    │   ├── port.ts
    │   └── link.ts
    │
    ├── generation/
    │   ├── generator.ts
    │   ├── generation-context.ts
    │   ├── placement-rules.ts
    │   ├── variation.ts
    │   └── generation-ticket.ts
    │
    ├── placement/
    │   ├── placement.ts
    │   ├── placement-plan.ts
    │   ├── validator.ts
    │   ├── overlap.ts
    │   └── foundation.ts
    │
    ├── planning/
    │   ├── structure-planner.ts
    │   ├── expansion-plan.ts
    │   └── connection-planner.ts
    │
    ├── query/
    │   ├── structure-query.ts
    │   └── spatial-index.ts
    │
    ├── graph/
    │   ├── structure-graph.ts
    │   ├── structure-link.ts
    │   └── graph-query.ts
    │
    ├── lod/
    │   ├── structure-lod.ts
    │   └── abstraction.ts
    │
    ├── persistence/
    │   ├── serializer.ts
    │   ├── migration.ts
    │   └── snapshot.ts
    │
    ├── registry/
    │   └── structure-registry.ts
    │
    ├── events/
    │   └── structure-events.ts
    │
    ├── networking/
    │   └── structure-replication.ts
    │
    ├── debugging/
    │   ├── structure-inspector.ts
    │   ├── structure-graph-viewer.ts
    │   └── profiler.ts
    │
    └── api/
        └── structure-api.ts
```

---

# 220. Fronteira arquitetural

## Structure System faz

```text id="str-boundary-01"
structure definitions
templates
instances
generation
placement planning
anchors
ports
links
structure graph
spatial queries
variants
placement rules
structure LOD
structure lifecycle
structure metadata
```

## Não faz

```text id="str-boundary-02"
block storage
physics
combat
AI
economy
civilization simulation
machine processing
fluid simulation
lighting
rendering
audio
entity lifecycle
world generation itself
```

---

# 221. Regra fundamental

> **Structure System descreve e coordena unidades compostas do mundo; os sistemas especializados continuam responsáveis pelos elementos individuais e pelo comportamento dessas estruturas.**

---

# 222. Segunda regra

> **World Generation decide quando uma estrutura deve aparecer; Structure System define como ela é representada e planejada; Build System executa sua materialização no mundo.**

---

# 223. Terceira regra

> **Uma Structure é uma unidade lógica, não necessariamente uma unidade física.**

Uma cidade pode continuar existindo como Structure mesmo quando parte dos seus blocos estiver danificada.

---

# 224. Quarta regra

> **Uma Structure pode possuir substructures, conexões e estado próprio sem transformar todos os seus componentes em um único objeto monolítico.**

---

# 225. Quinta regra

> **Estruturas oficiais e de mods usam a mesma Structure API pública.**

---

# 226. Ordem de implementação

```text id="str-order"
STR-0    Core Contracts
STR-1    StructureID
STR-2    StructureRegistry
STR-3    StructureDefinition
STR-4    StructureInstance
STR-5    StructureTemplate
STR-6    Palette
STR-7    Relative Coordinates
STR-8    Anchors
STR-9    Ports
STR-10   Links
STR-11   Components
STR-12   Variants
STR-13   Placement Rules
STR-14   Generation Context
STR-15   Generator
STR-16   Placement Plan
STR-17   Validation
STR-18   Overlap Rules
STR-19   Foundation Solver
STR-20   Build Integration
STR-21   Entity Integration
STR-22   BlockEntity Integration
STR-23   Spatial Index
STR-24   Queries
STR-25   Structure Graph
STR-26   Structure Planner
STR-27   Civilization Integration
STR-28   Infrastructure Integration
STR-29   LOD
STR-30   Streaming
STR-31   Persistence
STR-32   Migration
STR-33   Events
STR-34   Networking
STR-35   Mod API
STR-36   Debugging
STR-37   Profiling
STR-38   Determinism
STR-39   Stress Tests
STR-40   Compatibility
```

---

# 227. Primeiro Vertical Slice

```text id="str-vs-01"
StructureRegistry
        ↓
StructureDefinition
        ↓
StructureTemplate
        ↓
StructureGenerator
        ↓
PlacementPlan
        ↓
Build & Destruction
        ↓
World
```

Primeira estrutura:

```text id="str-vs-02"
nexora:test_house
```

---

# 228. Segundo Vertical Slice

```text id="str-vs-03"
Village
 ↓
Structure Graph
 ↓
House
Market
Road
 ↓
NPC population
 ↓
Civilization
```

---

# 229. Terceiro Vertical Slice

```text id="str-vs-04"
Rail Station
 ↓
Rail Anchor
 ↓
Railway System
 ↓
Rail Network
 ↓
Vehicle
```

---

# 230. Quarto Vertical Slice

```text id="str-vs-05"
Factory Structure
 ↓
Machine Blocks
 ↓
Energy Network
 ↓
Fluid Network
 ↓
Production
```

---

# 231. Quinto Vertical Slice

```text id="str-vs-06"
Ancient Ruin
 ↓
StructureInstance
 ↓
Player discovers
 ↓
Knowledge
 ↓
Quest
 ↓
Loot
 ↓
History
```

---

# 232. Sexto Vertical Slice

```text id="str-vs-07"
Player
 ↓
Structure Blueprint
 ↓
Placement Plan
 ↓
Build System
 ↓
StructureInstance
 ↓
Save
 ↓
Reload
```

---

# 233. Sétimo Vertical Slice — Mod

```text id="str-vs-08"
Mod
 ↓
StructureDefinition
 ↓
Template
 ↓
Generator
 ↓
Placement
 ↓
World
 ↓
Save
 ↓
Reload
```

---

# 234. Teste de escala

```text id="str-scale-01"
1 structure
100
1,000
10,000
100,000
```

com:

```text id="str-scale-02"
spatial queries
graph links
LOD
streaming
persistence
```

---

# 235. Megaestrutura

Teste específico:

```text id="str-scale-03"
1 mega-city
├── 100,000 buildings
├── roads
├── rail
├── factories
├── storage
└── districts
```

O Structure System deve conseguir representar isso como grafo e hierarquia sem transformar todos os blocos em objetos Structure individuais.

---

# 236. Arquitetura final

```text id="str-final-01"
                         NEXORA
                           │
                    STRUCTURE SYSTEM
                           │
             ┌─────────────┼─────────────┐
             ↓             ↓             ↓
        DEFINITIONS     GENERATION     INSTANCES
             │             │             │
         TEMPLATE       RULES          STATE
         VARIANTS       PLACEMENT      HISTORY
         COMPONENTS     VALIDATION     OWNERSHIP
         ANCHORS            │             │
         PORTS              └──────┬──────┘
                                   ↓
                              BUILD SYSTEM
                                   │
                                   ↓
                                  WORLD
```

E a arquitetura de uma cidade fica:

```text id="str-final-02"
                          CITY
                           │
                     STRUCTURE GRAPH
                           │
        ┌──────────────────┼─────────────────┐
        ↓                  ↓                 ↓
    DISTRICTS           ROADS             RAIL
        │                  │                 │
   ┌────┼────┐             │            STATIONS
   ↓    ↓    ↓             │
 HOUSE MARKET FACTORY       │
   │         │              │
   └─────────┴──────────────┘
               │
          INFRASTRUCTURE
               │
       ┌───────┼────────┐
       ↓       ↓        ↓
     ENERGY  FLUID    LOGISTICS
```

E o fluxo completo de geração:

```text id="str-final-03"
WORLD GENERATION
       ↓
"preciso gerar uma vila"
       ↓
STRUCTURE SYSTEM
       ↓
select definition
       ↓
select variant
       ↓
find location
       ↓
terrain adaptation
       ↓
placement validation
       ↓
placement plan
       ↓
BUILD & DESTRUCTION
       ↓
BLOCK / BLOCK ENTITY / ENTITY
       ↓
STRUCTURE INSTANCE
       ↓
CIVILIZATION / ECONOMY / QUEST / HISTORY
       ↓
PERSISTENCE
```

A fundação do NEXORA agora fica ainda mais interessante:

```text
NEXORA FOUNDATION
├── Core
├── Registry System
├── Event Bus
├── Save / Persistence
├── Entity System
├── Block System
├── Item System
├── Animation System
├── Audio System
├── UI System
├── Dimension System
└── Structure System
```

E uma distinção passa a ficar oficial:

```text
BLOCK
"O que existe nesta célula?"

ENTITY
"Quem existe aqui?"

STRUCTURE
"Quais elementos formam esta unidade?"

CHUNK
"Como este espaço é armazenado?"

DIMENSION
"Em qual espaço/regras isso existe?"

WORLD
"Qual é o estado geral?"

PERSISTENCE
"Como tudo isso sobrevive?"
```

Isso é particularmente importante para a visão de mundo do NEXORA: **uma cidade, uma fábrica, uma ferrovia ou uma civilização não precisam ser hardcoded como objetos gigantes no Core; elas podem emergir da combinação de Structures + Blocks + Entities + Machines + Infrastructure + Civilization.**
