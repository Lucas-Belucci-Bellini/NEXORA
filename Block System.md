Perfeito. Para o NEXORA, eu separaria **Block API** de **Block System**, exatamente como fizemos com Fluid, Energy, Combat e Tool/Weapon.

A ideia central seria:

> **Block System define o que um bloco é e como ele existe no mundo. Block API define como outros sistemas e mods podem consultar, registrar e interagir com esses blocos.**

# NEXORA — BLOCK API / BLOCK SYSTEM

## 1. Princípio central

```text
BLOCK SYSTEM
├── IDENTIDADE
├── DEFINIÇÃO
├── ESTADO
├── COMPORTAMENTO
├── ARMAZENAMENTO
└── INTEGRAÇÃO

BLOCK API
├── REGISTRO
├── CONSULTA
├── CAPABILITIES
├── INTERAÇÃO
├── EVENTOS
└── MODDING
```

O sistema precisa permitir que:

```text
WORLD
 ↓
BLOCK STATE
 ↓
BLOCK SYSTEM
 ↓
PUBLIC BLOCK API
 ↓
VANILLA CONTENT
+
COMMUNITY MODS
```

O Core não deve precisar conhecer:

```text
Stone
Grass
Wood
Copper
Machine
Door
Chest
Ore
Glass
```

Esses são conteúdos.

O Core deve conhecer conceitos como:

```text
BlockID
BlockState
BlockDefinition
BlockRegistry
BlockPosition
BlockProperties
BlockInteraction
```

---

# 2. O que é um Block?

Um Block não é simplesmente uma textura.

Ele representa uma célula do mundo com:

```text
ID
STATE
PROPERTIES
SHAPE
MATERIAL
BEHAVIOR
TAGS
COLLISION
LIGHT
PHYSICS
DROPS
INTERACTION
```

Exemplo conceitual:

```text
Block:
    id = nexora:copper_ore

State:
    exposed = false
```

Outro bloco:

```text
Block:
    id = nexora:door

State:
    open = true
    facing = north
    half = lower
```

Portanto:

> **BlockDefinition define o tipo. BlockState representa a instância daquele tipo em uma posição específica.**

---

# 3. BlockDefinition

Representa a definição estática.

```text
BlockDefinition

id
displayName
properties
material
hardness
resistance
friction
flammability
lightEmission
lightAbsorption
soundProfile
renderProfile
collisionProfile
interactionProfile
harvestProfile
dropProfile
tags
capabilities
```

Exemplo:

```text
BlockDefinition
    id = nexora:granite

    hardness = ...
    resistance = ...
    friction = ...

    material = stone

    tags:
        rock
        building_material
        natural

    capabilities:
        mineable
        placeable
        solid
```

---

# 4. BlockID

Todo bloco possui identidade global.

Formato:

```text
namespace:block
```

Exemplos:

```text
nexora:stone
nexora:dirt
nexora:oak_log
nexora:copper_ore
```

Mods:

```text
examplemod:steel_block
examplemod:reinforced_glass
```

Nunca confiar em números locais como identidade pública.

Internamente pode existir:

```text
RuntimeBlockID
```

para performance.

---

# 5. BlockState

O mesmo bloco pode possuir diferentes estados.

Exemplo:

```text
Door
├── open
├── facing
├── half
└── powered
```

Portanto:

```text
BlockDefinition
        ↓
BlockStateSchema
        ↓
BlockState
```

Exemplo:

```text
oak_door
{
    open: true,
    facing: east,
    half: upper
}
```

---

# 6. BlockStateSchema

Define quais propriedades um bloco pode possuir.

```text
PropertyDefinition
    name
    type
    allowedValues
    defaultValue
```

Tipos:

```text
Boolean
Integer
Enum
Direction
Axis
Float
Custom
```

Exemplo:

```text
facing:
    north
    south
    east
    west
```

Isso permite um sistema totalmente data-driven.

---

# 7. BlockPosition

Posição absoluta:

```text
BlockPos

x
y
z
dimension
```

Pode ser convertida para:

```text
ChunkPos
LocalBlockPos
RegionPos
```

Isso conecta diretamente com:

* Chunk & Voxel Engine
* World Generation
* Lighting
* Physics
* Build & Destruction
* Fluids
* Vegetation

---

# 8. Block Storage

O Block System não deve armazenar o mundo inteiro como objetos individuais.

Não fazer:

```text
new Block()
new Block()
new Block()
...
```

para cada posição.

O mundo deve usar armazenamento compacto.

Conceito:

```text
Chunk
 └── BlockStorage
      ├── palette
      ├── indices
      └── metadata
```

Exemplo:

```text
Palette

0 = air
1 = stone
2 = dirt
3 = grass
4 = copper_ore
```

O chunk armazena referências compactas.

---

# 9. Block Palette

Muito importante para eficiência.

Um chunk pode ter:

```text
1 milhão de blocos
```

mas talvez só utilize:

```text
37 tipos
```

Então:

```text
BlockState
      ↓
Palette Index
      ↓
Compact Storage
```

A palette deve poder crescer dinamicamente.

---

# 10. Air

Air precisa ser tratado como um bloco válido conceitualmente.

```text
nexora:air
```

Mas pode possuir representação especial de storage:

```text
EMPTY
```

Isso permite otimizações enormes.

---

# 11. Block Tags

Blocos devem possuir tags.

Exemplo:

```text
#rock
#stone
#ore
#wood
#flammable
#transparent
#fluid_passable
#building_material
#plant_support
#requires_pickaxe
```

Isso é muito mais importante que hardcode.

Uma receita pode pedir:

```text
#iron_ore
```

em vez de:

```text
nexora:iron_ore
nexora:deep_iron_ore
modx:iron_ore
...
```

---

# 12. Material System

Block não deve definir física repetidamente.

Criar:

```text
MaterialDefinition
```

Exemplo:

```text
STONE
WOOD
METAL
GLASS
ICE
SAND
SOIL
ORGANIC
CRYSTAL
MAGICAL
LIQUID
GAS
CUSTOM
```

Material pode influenciar:

```text
hardness
friction
sound
thermal
flammability
density
conductivity
structural behavior
```

---

# 13. Hardness

Hardness é propriedade de gameplay.

Não é necessariamente força estrutural.

Exemplo:

```text
hardness
```

determina dificuldade para quebrar.

Já:

```text
structuralStrength
```

pertence ao sistema de estabilidade.

Assim:

```text
Block System
→ hardness

Build & Destruction
→ structural behavior
```

---

# 14. Blast / Damage Resistance

Pode existir:

```text
damageResistance
```

mas a aplicação fica em sistemas especializados.

Por exemplo:

```text
Explosion
    ↓
Build & Destruction
    ↓
query Block API
    ↓
resistance
    ↓
damage operation
```

O Block System não precisa conhecer "explosões".

---

# 15. Friction

Exposto para Physics.

```text
BlockPhysicsProfile
```

Pode definir:

```text
friction
slipperiness
surfaceBounce
drag
```

---

# 16. Collision

Collision deve possuir definição própria.

```text
BlockCollisionProfile
```

Pode ser:

```text
FULL_CUBE
EMPTY
CUSTOM
MULTI_SHAPE
```

Exemplo:

```text
slab
```

pode possuir uma caixa parcial.

---

# 17. Voxel Shape

Para blocos complexos:

```text
VoxelShape
```

representa volumes.

Pode haver:

```text
CollisionShape
SelectionShape
OcclusionShape
InteractionShape
```

Isso evita obrigar Renderer, Physics e interação a usar a mesma geometria.

---

# 18. Block Occlusion

Renderer precisa saber:

```text
este bloco esconde a face daquele bloco?
```

API:

```text
IOcclusionShape
```

Isso reduz drasticamente geometria desnecessária.

---

# 19. Block Rendering

Block System não renderiza.

Ele fornece:

```text
RenderProfile
```

Renderer consome.

Exemplo:

```text
RenderProfile

model
material
transparency
cutout
emissive
animated
renderLayer
```

Portanto:

```text
Block
 ↓
RenderProfile
 ↓
Renderer
```

---

# 20. Block Models

Modelos podem ser:

```text
CUBE
CUSTOM
MULTIPART
PROCEDURAL
CONNECTED
```

Isso permite:

* máquinas
* tubos
* portas
* escadas
* pontes
* blocos decorativos
* blocos gigantes

---

# 21. Textures

Texture não deve estar hardcoded no Block System.

```text
AssetID
```

Exemplo:

```text
nexora:block/stone
```

Renderer resolve o asset.

---

# 22. Animated Blocks

Alguns blocos podem possuir animações.

Exemplo:

```text
machine
portal
fluid_container
energy_core
```

API:

```text
BlockAnimationProfile
```

O Animation System executa a animação.

---

# 23. Lighting

O bloco fornece:

```text
lightEmission
lightAbsorption
```

Lighting System calcula a propagação.

Exemplo:

```text
torch
    emission = 14
```

Não:

```text
torch.updateLighting()
```

A separação é:

```text
Block
→ propriedades

Lighting
→ simulação
```

---

# 24. Light Passability

Pode existir:

```text
opacity
transparency
```

Valores podem representar:

```text
fully opaque
semi transparent
transparent
custom
```

---

# 25. Fluid Interaction

O Block API precisa permitir consultar:

```text
fluid permeability
fluid interaction
fluid containment
fluid displacement
```

Mas a simulação fica no Fluid Engine.

Exemplo:

```text
Water
 ↓
Fluid Engine
 ↓
Block API
 ↓
canContain?
canPass?
interaction?
```

---

# 26. Thermal Properties

Como NEXORA possui Energy API e sistemas industriais, blocks podem ter:

```text
thermalConductivity
heatCapacity
meltingPoint
boilingPoint
ignitionPoint
```

Mas Thermal Simulation deve permanecer em outro sistema.

---

# 27. Electrical Properties

Alguns blocos podem possuir:

```text
conductivity
resistance
insulation
```

Energy API interpreta isso.

Não criar:

```text
Block.isElectric()
```

quando uma capability é melhor.

---

# 28. Block Capabilities

Essa é uma das partes mais importantes.

Um bloco pode fornecer capacidades.

```text
IBlockCapability
```

Exemplos:

```text
Container
Machine
EnergyPort
FluidPort
Pipe
CraftingStation
HeatSource
RedstoneLikeSignal
Interactable
Climbable
Seat
Storage
```

Assim:

```text
Block
 +
Capability
```

fica muito mais extensível.

---

# 29. Block Entity

Nem todo dado complexo deve estar no BlockState.

Blocos como:

```text
chest
machine
reactor
controller
storage
computer
```

podem precisar de dados dinâmicos.

Então:

```text
Block
+
BlockEntity
```

Exemplo:

```text
Block:
    nexora:machine_casing

BlockEntity:
    machineType
    inventory
    energy
    fluid
    progress
```

---

# 30. BlockEntity System

BlockEntity deve ser associado ao:

```text
BlockPosition
```

e possuir:

```text
BlockEntityID
PersistentData
Components
```

Isso conversa diretamente com o Entity System.

Mas BlockEntity não precisa virar uma Entity comum.

Podemos manter:

```text
BlockEntity
```

como entidade especializada do mundo.

---

# 31. Block Interaction

Public API:

```text
IBlockInteraction
```

Eventos:

```text
onPlace
onBreak
onUse
onInteract
onNeighborChanged
onStateChanged
onScheduledTick
```

---

# 32. Player Interaction

Player apenas solicita:

```text
InteractionRequest
```

Exemplo:

```text
Player
 ↓
Interaction API
 ↓
Block API
 ↓
Block behavior
```

Isso evita colocar toda lógica em Player.

---

# 33. Neighbor Updates

Quando um bloco muda:

```text
stone
 ↓
changed
```

vizinhos podem precisar saber.

Eventos:

```text
BlockNeighborChanged
```

Usado por:

* redstone-like systems
* fluid
* plants
* machines
* doors
* structural systems
* automation

---

# 34. Scheduled Block Updates

Alguns blocos precisam de atualizações.

Exemplo:

```text
plant
crop
machine
fire
fluid
```

API:

```text
scheduleBlockUpdate(
    position,
    delay
)
```

Mas o scheduler pertence ao Core/Event/Scheduler.

---

# 35. Block Ticking

Não permitir que milhões de blocos façam:

```text
tick()
tick()
tick()
```

a cada frame.

Isso seria um desastre.

Usar:

```text
event-driven updates
scheduled updates
simulation LOD
```

---

# 36. Simulation LOD

Blocos também precisam de LOD.

```text
FULL
REGIONAL
ABSTRACT
```

Exemplo:

Uma fábrica distante:

```text
FULL
```

vira:

```text
production rate
inventory summary
energy consumption
```

em vez de simular cada componente.

---

# 37. Random Tick

Caso seja necessário:

```text
RandomTickSystem
```

seleciona blocos.

Mas o Block System apenas informa:

```text
supportsRandomTick = true
```

---

# 38. Block Lifecycle

Blocos podem passar por:

```text
UNREGISTERED
REGISTERED
GENERATED
PLACED
ACTIVE
CHANGED
REMOVED
```

---

# 39. Placement

Colocar bloco deve ser uma operação transacional.

```text
PlaceBlockRequest
```

Contém:

```text
actor
position
blockState
tool
context
```

---

# 40. Placement Validation

Antes de colocar:

```text
canPlace?
```

Pode depender de:

```text
collision
support
dimension
ownership
permission
fluid
temperature
special rules
```

---

# 41. Breaking

Quebrar também é uma operação.

```text
BreakBlockRequest
```

O Build & Destruction Engine executa.

Block API fornece informações.

---

# 42. Replacement

Trocar:

```text
stone → machine
```

deve ser explicitamente uma operação.

```text
ReplaceBlockRequest
```

---

# 43. Batch Operations

Construções enormes precisam:

```text
BatchPlace
BatchBreak
BatchReplace
```

Para:

* construção
* WorldEdit-like tools
* NPC construction
* terraformação
* estruturas
* máquinas

---

# 44. Atomic Operations

Uma operação pode afetar milhares de blocos.

Então:

```text
BlockTransaction
```

com:

```text
begin
validate
reserve
apply
commit
rollback
```

---

# 45. Multiplayer

O servidor deve ser autoridade.

```text
Client
 ↓
BlockOperationRequest
 ↓
Server
 ↓
Validation
 ↓
Apply
 ↓
Replication
```

Nunca confiar no cliente.

---

# 46. Block Change Events

Eventos principais:

```text
BlockPlaced
BlockBroken
BlockChanged
BlockReplaced
BlockStateChanged
BlockNeighborChanged
BlockEntityCreated
BlockEntityRemoved
```

---

# 47. Registry

Criar:

```text
BlockRegistry
```

Operações:

```text
register
get
contains
remove
freeze
iterate
```

Durante runtime normal:

```text
Registry = read optimized
```

---

# 48. Component Registry

Separado:

```text
BlockCapabilityRegistry
BlockComponentRegistry
BlockPropertyRegistry
```

---

# 49. Mod API

Mods podem registrar:

```text
BlockDefinition
BlockState
BlockCapability
BlockEntity
BlockTag
BlockMaterial
BlockRenderProfile
BlockInteraction
```

Tudo utilizando API pública.

---

# 50. Official Content

Conteúdo oficial deve utilizar exatamente a mesma API.

```text
PUBLIC BLOCK API
       ↓
┌───────────────┐
│ Vanilla       │
│ Mod A         │
│ Mod B         │
│ Mod C         │
└───────────────┘
```

Não:

```text
Vanilla → private API
Mods → limited API
```

---

# 51. Data-Driven Blocks

Idealmente um bloco pode ser descrito por dados.

Exemplo conceitual:

```json
{
  "id": "examplemod:steel_block",
  "material": "metal",
  "hardness": 8,
  "resistance": 12,
  "tags": [
    "metal",
    "building_material"
  ]
}
```

O runtime transforma isso em estruturas internas.

---

# 52. Block Inheritance

Evitar herança profunda:

```text
Block
 └── MetalBlock
      └── MachineBlock
           └── ReactorBlock
```

Preferir composição.

```text
BlockDefinition
+
Capabilities
+
Components
+
Profiles
```

Isso combina melhor com a filosofia do NEXORA.

---

# 53. Block Components

Exemplos:

```text
StorageComponent
EnergyComponent
FluidComponent
HeatComponent
MachineComponent
InventoryComponent
SignalComponent
GrowthComponent
LightComponent
```

---

# 54. Block State vs Block Component

Regra importante.

Use **State** para informação pequena, discreta e necessária para representar o estado físico do bloco.

Use **Component/BlockEntity** para dados maiores ou dinâmicos.

Exemplo:

```text
door.open
```

→ State

Mas:

```text
chest.inventory
```

→ BlockEntity

---

# 55. Block Metadata

Pode existir:

```text
BlockMetadata
```

para pequenas informações auxiliares.

Mas não utilizar metadata como "depósito universal".

---

# 56. Block Serialization

Um bloco precisa ser serializável.

```text
BlockStateSerializer
```

Formato conceitual:

```text
BlockID
State
Version
```

---

# 57. BlockEntity Serialization

BlockEntity:

```text
BlockEntityID
BlockPosition
Type
Version
ComponentData
```

---

# 58. Save Compatibility

Quando uma definição muda:

```text
old block
 ↓
migration
 ↓
new block
```

Criar:

```text
BlockMigration
```

---

# 59. Unknown Blocks

O que acontece quando o jogo abre mundo com mod removido?

Não apagar silenciosamente.

Usar:

```text
UnknownBlock
```

ou:

```text
MissingBlockState
```

para preservar dados.

Isso é essencial para modded worlds.

---

# 60. Missing Block Policy

Estados possíveis:

```text
AVAILABLE
MISSING
INVALID
QUARANTINED
MIGRATABLE
```

---

# 61. World Compatibility

Um mundo deve registrar:

```text
block registry snapshot
mod dependencies
world version
block versions
```

---

# 62. Chunk Compatibility

Chunk precisa saber se seus blocos continuam válidos.

```text
Chunk
 ↓
Block Registry
 ↓
Validation
```

---

# 63. Procedural Generation

WorldGen pode solicitar:

```text
BlockState
```

e colocar no chunk.

Exemplo:

```text
WorldGen
 ↓
BlockRegistry
 ↓
nexora:granite
 ↓
BlockState
 ↓
ChunkStorage
```

---

# 64. Ore Generation

Ore Generator não deveria conhecer código específico para cada minério.

Pode usar:

```text
BlockTag
```

e:

```text
BlockState
```

---

# 65. Vegetation

Vegetation pode consultar:

```text
supportsPlant
soilType
waterRetention
lightRequirement
```

por API.

---

# 66. Fluid Integration

Fluid Engine pode consultar:

```text
IFluidInteractionProvider
```

Exemplo:

```text
water
+
lava
+
block
```

e determinar uma reação.

---

# 67. Physics Integration

Physics consulta:

```text
collision shape
friction
density
surface
```

---

# 68. Build & Destruction Integration

Build System consulta:

```text
hardness
resistance
support
replaceability
interaction
drop
```

---

# 69. Lighting Integration

Lighting consulta:

```text
emission
opacity
transmission
```

---

# 70. Rendering Integration

Renderer consulta:

```text
render profile
model
texture
material
animation
occlusion
```

---

# 71. Drop & Loot Integration

Block System não deve decidir diretamente o loot.

Ele fornece:

```text
DropSource
```

ou:

```text
BlockDropContext
```

e o Loot System decide.

---

# 72. Tool & Weapon Integration

Ferramenta consulta:

```text
harvestProfile
requiredCapability
toolLevel
```

Depois:

```text
Tool API
 ↓
Build & Destruction
 ↓
Block API
```

---

# 73. Crafting Integration

Crafting consulta blocos por:

```text
BlockTag
BlockID
Material
```

---

# 74. Machines Integration

Machine Blocks podem registrar:

```text
MachineCapability
```

e então:

```text
Machine System
```

gerencia a máquina.

O Block System não vira o sistema industrial.

---

# 75. Energy Integration

Um bloco pode possuir:

```text
EnergyPort
```

e a Energy API faz o resto.

---

# 76. Fluid Integration

Da mesma forma:

```text
FluidPort
Tank
Pipe
Pump
```

são capabilities.

---

# 77. Interaction API

Criar:

```text
IBlockInteractable
```

com:

```text
canInteract()
interact()
```

Contexto:

```text
actor
position
face
item
world
```

---

# 78. Block Faces

API:

```text
BlockFace

UP
DOWN
NORTH
SOUTH
EAST
WEST
```

Além de:

```text
Axis
Direction
```

---

# 79. Neighbor Queries

```text
getNeighbor(pos, direction)
```

e:

```text
getBlockState(pos)
```

precisam ser extremamente eficientes.

---

# 80. Region Queries

Permitir consultas:

```text
findBlocks
findByTag
findByCapability
findByState
```

Mas nunca varrer o mundo inteiro sem propósito.

---

# 81. Block Query API

Criar:

```text
IBlockQuery
```

Exemplos:

```text
query.box(...)
query.radius(...)
query.tag(...)
query.capability(...)
```

---

# 82. Caching

Informações muito consultadas podem possuir cache:

```text
collision
render
occlusion
state lookup
material
```

Mas cache precisa ser invalidado quando necessário.

---

# 83. Thread Safety

Leitura pode ser paralelizada.

Modificações devem passar por mecanismos controlados.

Modelo:

```text
World Snapshot
      ↓
READ
      ↓
Operations Queue
      ↓
WRITE
```

---

# 84. Deferred Updates

Não modificar estruturas críticas durante uma iteração.

Usar:

```text
BlockOperationQueue
```

Exemplo:

```text
Machine
 ↓
requests block change
 ↓
queue
 ↓
commit phase
```

---

# 85. Cross-Chunk Changes

Um bloco pode afetar outro chunk.

Exemplo:

```text
redstone
fluid
structure
plant
large block
```

Portanto a operação deve suportar:

```text
cross-chunk dependency
```

---

# 86. Large Structures

Alguns elementos ocupam vários blocos.

Exemplo:

```text
multiblock reactor
bridge
giant tree
elevator
factory
```

Block System representa os blocos.

Structure System coordena a estrutura.

---

# 87. Multi-block System

Pode existir:

```text
MultiBlockDefinition
```

com:

```text
pattern
controller
parts
validation
activation
```

---

# 88. Connected Blocks

Blocos podem possuir conectividade:

```text
pipes
rails
wires
walls
fences
roads
```

Mas a topologia deve ficar no sistema responsável.

Block API só fornece os estados necessários.

---

# 89. Gravity Blocks

Não criar um sistema de gravidade dentro de Block.

Em vez disso:

```text
BlockDefinition
    physicsTags
```

e o Physics / Build System resolve.

---

# 90. Falling Blocks

Pode resultar em:

```text
Block
 ↓
Build/Physics
 ↓
Entity System
 ↓
FallingBlockEntity
```

Excelente exemplo da separação entre sistemas.

---

# 91. Fire

Fire não deveria ser hardcoded dentro do Core.

Pode ser:

```text
Block
+
FireComponent
```

ou um sistema especializado.

---

# 92. Plants

Plantas podem ser representadas parcialmente por blocos, mas crescimento pertence ao:

```text
Dynamic Vegetation
```

---

# 93. Doors

Door pode usar:

```text
BlockState
InteractionCapability
```

---

# 94. Chests

Chest:

```text
Block
+
BlockEntity
+
InventoryCapability
```

---

# 95. Machines

Machine:

```text
MachineBlock
+
BlockEntity
+
MachineCapability
+
EnergyCapability
+
FluidCapability
```

---

# 96. Portals

Portal:

```text
Block
+
PortalCapability
```

Dimension System executa a transferência.

---

# 97. Rails

Rail:

```text
BlockState
+
RailConnection
```

Railway System administra rede e veículos.

---

# 98. Roads

Road block pode possuir:

```text
surface quality
lane metadata
traffic tags
```

Civilization/Transport Systems interpretam.

---

# 99. Structural Support

Block pode expor:

```text
supportProfile
```

Mas quem calcula colapso é:

```text
Build & Destruction
+
Physics
```

---

# 100. Weather Interaction

Bloco pode responder a:

```text
rain
snow
temperature
humidity
```

mas Climate System decide quando esses eventos acontecem.

---

# 101. Environmental Reaction

Exemplos:

```text
snow melts
ice freezes
mud dries
metal corrodes
plant grows
```

A lógica pode ser distribuída entre:

```text
Climate
Fluid
Vegetation
Block
Materials
```

sem criar um monolito.

---

# 102. Block Events

Eventos recomendados:

```text
BlockPlacedEvent
BlockBrokenEvent
BlockChangedEvent
BlockStateChangedEvent
BlockNeighborChangedEvent
BlockInteractedEvent
BlockScheduledEvent
BlockEntityCreatedEvent
BlockEntityRemovedEvent
```

---

# 103. API Interfaces

Interface principal:

```text
IBlock
```

Definição:

```text
IBlockDefinition
```

Estado:

```text
IBlockState
```

Registro:

```text
IBlockRegistry
```

Consulta:

```text
IBlockQuery
```

Interação:

```text
IBlockInteraction
```

Operação:

```text
IBlockOperation
```

Persistência:

```text
IBlockPersistence
```

---

# 104. Capability APIs

```text
IBlockCapability
IBlockContainer
IBlockEnergyPort
IBlockFluidPort
IBlockMachine
IBlockInteractable
IBlockHeatSource
IBlockStorage
```

---

# 105. Block Context

Criar:

```text
BlockContext
```

com referências controladas para:

```text
world
dimension
position
state
actor
environment
```

---

# 106. Block Operation Context

```text
BlockOperationContext
```

Pode carregar:

```text
actor
source
tool
cause
permissions
transaction
```

---

# 107. Source of Change

Todo bloco alterado deveria saber a origem lógica.

```text
Player
NPC
Machine
Fluid
Physics
WorldGen
Weather
Mod
Command
WorldEvent
```

Isso é muito importante para debug e histórico.

---

# 108. Block History

Pode existir integração com:

```text
World History
```

para registrar:

```text
who
what
where
when
why
```

Sem guardar necessariamente cada bloco para sempre.

---

# 109. Protection

Block API pode expor:

```text
permissionContext
ownership
protection
```

Mas permissões pertencem a sistemas superiores.

---

# 110. Claims / Territory

Civilization ou Protection System pode perguntar:

```text
canModify(position, actor)
```

antes da operação.

---

# 111. Commands

Debug:

```text
nexora block get
nexora block set
nexora block break
nexora block inspect
nexora block state
nexora block query
nexora block registry
```

---

# 112. Debug Inspector

Mostrar:

```text
position
block id
state
material
tags
capabilities
collision
lighting
block entity
owner
```

---

# 113. Registry Debug

```text
nexora block registry list
```

pode mostrar:

```text
ID
runtime ID
mod
version
states
capabilities
```

---

# 114. Validation

Cada BlockDefinition deve ser validado.

Exemplos:

```text
missing ID
duplicate ID
invalid property
invalid state
missing render profile
invalid capability
invalid texture
```

---

# 115. Mod Isolation

Mod que registra bloco inválido:

```text
REGISTER
 ↓
VALIDATE
 ↓
ACCEPT
```

ou:

```text
REJECT
QUARANTINE
```

sem corromper todo o jogo.

---

# 116. Registry Freeze

Após carregamento:

```text
register phase
 ↓
validate phase
 ↓
freeze
 ↓
runtime
```

Isso permite otimização.

---

# 117. Runtime IDs

Durante runtime:

```text
BlockID
```

pode ser traduzido para:

```text
RuntimeBlockID
```

por exemplo:

```text
nexora:stone
 ↓
runtime 17
```

O runtime ID pode mudar entre instalações.

Save não deve depender dele.

---

# 118. Network IDs

Da mesma maneira:

```text
NetworkBlockID
```

pode existir durante uma sessão.

---

# 119. Save IDs

Save usa:

```text
namespace:block
```

ou identificador estável equivalente.

Nunca:

```text
runtime id = persistence id
```

---

# 120. Versioning

Cada definição pode ter:

```text
definitionVersion
dataVersion
schemaVersion
```

---

# 121. Migration

Exemplo:

```text
examplemod:old_machine
```

vira:

```text
examplemod:machine
```

através de:

```text
BlockMigrationRegistry
```

---

# 122. Performance Architecture

O Block System precisa ser projetado para:

```text
milhões
→ bilhões
```

de blocos no mundo.

Portanto:

```text
NO object-per-block
NO tick-every-block
NO virtual-call-every-block
NO world-wide scan
```

---

# 123. Memory Strategy

Priorizar:

```text
Palette Compression
Bit Packing
Compact State IDs
Chunk-local arrays
Sparse metadata
Lazy BlockEntity allocation
```

---

# 124. Hot Path

O acesso mais comum será:

```text
getBlockState(x,y,z)
```

Isso precisa ser extremamente barato.

---

# 125. Chunk-local Access

Depois que o chunk está carregado:

```text
chunk.get(localX, localY, localZ)
```

deve ser preferido.

---

# 126. Chunk Mutation

Modificar:

```text
set(localX, localY, localZ, state)
```

deve disparar somente as atualizações necessárias.

---

# 127. Dirty Flags

Chunk pode possuir:

```text
blockDirty
meshDirty
lightingDirty
fluidDirty
saveDirty
simulationDirty
```

---

# 128. Meshing Integration

Quando bloco muda:

```text
Block change
 ↓
Chunk dirty
 ↓
Mesh invalidation
 ↓
Renderer remesh
```

Somente regiões necessárias devem ser reconstruídas.

---

# 129. Lighting Integration

Da mesma forma:

```text
block change
 ↓
lighting update
```

sem recalcular o mundo inteiro.

---

# 130. Fluid Integration

Alteração pode causar:

```text
fluid invalidation
```

somente no local afetado.

---

# 131. Neighbor Chunk

Se alteração ocorrer na borda:

```text
Chunk A
████████│
        │
        │ Chunk B
        │████████
```

B também pode precisar ser invalidado.

---

# 132. Async Operations

Operações pesadas podem ser executadas:

```text
worker thread
```

desde que o commit final seja seguro.

---

# 133. Read Snapshots

Sistemas como:

* Renderer
* Physics
* AI

podem consumir snapshots.

```text
World Snapshot
 ↓
Physics
Renderer
AI
```

---

# 134. Event Ordering

Definir ordem determinística:

```text
Validate
 ↓
Apply Block State
 ↓
Create/Remove BlockEntity
 ↓
Neighbor Events
 ↓
Lighting
 ↓
Fluid
 ↓
Mesh Invalidation
 ↓
Persistence
 ↓
Replication
```

Isso evita bugs caóticos.

---

# 135. Transaction IDs

Cada mutação pode possuir:

```text
BlockTransactionID
```

Isso ajuda em:

* multiplayer
* rollback
* debugging
* anti-duplication
* replay
* history

---

# 136. Replay

O sistema pode futuramente suportar:

```text
block change log
```

para debugging/replay.

---

# 137. Undo

Build System pode registrar:

```text
beforeState
afterState
```

e fazer:

```text
undo
redo
```

sem o Block System conhecer a interface de construção.

---

# 138. Blueprints

Blueprints usam:

```text
BlockState
```

e não implementação interna.

---

# 139. Schematics

Estrutura:

```text
Schematic
├── dimensions
├── block palette
├── block states
└── block entities
```

---

# 140. World Generation Contract

WorldGen só precisa de:

```text
resolve block
create state
write state
```

Não precisa conhecer storage interno.

---

# 141. Renderer Contract

Renderer recebe:

```text
BlockRenderData
```

e não depende diretamente de WorldGen.

---

# 142. Physics Contract

Physics recebe:

```text
BlockPhysicsData
```

---

# 143. Fluid Contract

Fluid Engine recebe:

```text
BlockFluidInteraction
```

---

# 144. Build Contract

Build System recebe:

```text
BlockHarvestData
BlockPlacementData
BlockSupportData
```

---

# 145. Loot Contract

Loot System recebe:

```text
BlockDropContext
```

---

# 146. Machine Contract

Machine System recebe:

```text
MachineCapability
```

---

# 147. Entity Contract

Entity System pode interagir com:

```text
BlockPosition
BlockCollision
BlockInteraction
BlockAttachment
```

---

# 148. Vehicle Contract

Vehicles podem consultar:

```text
surface
collision
friction
rail capability
road capability
```

---

# 149. NPC Contract

NPCs podem consultar:

```text
walkable
interactable
usable
buildable
destructible
```

---

# 150. AI Contract

AI nunca deve verificar:

```text
if block == stone
```

preferir:

```text
hasTag("solid")
hasCapability("walkable")
material == rock
```

---

# 151. Modding Contract

Um mod deveria conseguir fazer algo conceitualmente assim:

```text
registerBlock(
    BlockDefinition
)
```

e:

```text
registerCapability(
    BlockCapability
)
```

sem alterar Core.

---

# 152. Block Pack

Poderemos futuramente suportar:

```text
Block Pack
```

com:

```text
definitions
states
tags
models
textures
sounds
recipes
drops
```

---

# 153. Resource Integration

Asset references:

```text
textures
models
sounds
particles
animations
```

ficam fora do Block System.

Ele apenas referencia os assets.

---

# 154. Localization

BlockDefinition pode possuir:

```text
translationKey
```

exemplo:

```text
block.nexora.stone
```

O Localization System resolve o texto.

---

# 155. Sounds

Bloco pode possuir:

```text
BlockSoundProfile
```

com:

```text
break
place
step
hit
interact
```

Audio System reproduz.

---

# 156. Particles

Pode definir:

```text
breakParticle
placeParticle
interactionParticle
```

Particle System renderiza.

---

# 157. Animation

Block pode fornecer:

```text
BlockAnimationProfile
```

Animation System executa.

---

# 158. Registry Load Order

Pipeline:

```text
CORE
 ↓
REGISTRIES
 ↓
BLOCK MATERIALS
 ↓
BLOCK CAPABILITIES
 ↓
BLOCK DEFINITIONS
 ↓
BLOCK STATES
 ↓
BLOCK ENTITIES
 ↓
CONTENT
 ↓
WORLD
```

---

# 159. Dependency Resolution

Mods podem declarar:

```text
requires
optional
conflicts
loadAfter
loadBefore
```

---

# 160. Block API Version

A API pública precisa ter:

```text
Block API v1
Block API v2
...
```

para evitar quebra constante de mods.

---

# 161. Compatibility Layer

Futuro:

```text
Legacy Block API
        ↓
Compatibility Adapter
        ↓
Current Block API
```

---

# 162. Security

Mods não devem receber acesso irrestrito ao mundo.

Exemplo:

```text
Block API
 ├── read
 ├── write
 ├── registry
 ├── query
 └── privileged operations
```

Permissões podem ser diferentes.

---

# 163. Dedicated Server

Block System precisa funcionar sem Renderer.

```text
SERVER
 ├── Block Registry
 ├── Block State
 ├── Block Storage
 ├── Block Logic
 └── Block Persistence
```

Nenhuma dependência obrigatória de gráficos.

---

# 164. Client

Cliente possui adicionalmente:

```text
RenderProfile
Models
Textures
Animations
```

---

# 165. Server/Client Separation

```text
COMMON
├── BlockDefinition
├── BlockState
├── Tags
├── Logic
└── API

CLIENT
├── Render
├── Models
├── Textures
└── Animation
```

---

# 166. Testing

Testes obrigatórios:

```text
Registry
State
Properties
Serialization
Migration
Placement
Breaking
Transactions
Neighbor updates
Chunk boundaries
BlockEntity
Networking
Mod registration
```

---

# 167. Performance Tests

Escala:

```text
1 chunk
100 chunks
1,000 chunks
10,000 chunks
```

com:

```text
block queries
mass placement
mass destruction
lighting updates
fluid updates
mesh invalidation
```

---

# 168. Stress Tests

Exemplo:

```text
1,000,000 block changes
```

e medir:

```text
CPU
RAM
allocations
latency
chunk dirty propagation
network traffic
save time
```

---

# 169. Fuzz Testing

Gerar automaticamente:

```text
random BlockStates
random property combinations
random transactions
random mod definitions
```

para encontrar estados inválidos.

---

# 170. Determinism

Operações de bloco precisam ser determinísticas quando necessário.

Especialmente:

```text
WorldGen
server simulation
replay
tests
multiplayer
```

---

# 171. Golden Tests

Snapshots podem validar:

```text
chunk
block states
serialization
migration
```

---

# 172. Reference Implementation

Criar primeiro blocos simples:

```text
air
stone
dirt
grass
water
sand
wood
glass
```

Depois:

```text
door
chest
machine
fluid tank
energy block
```

---

# 173. Primeiro Vertical Slice

Eu faria exatamente nesta ordem:

```text
BlockRegistry
        ↓
BlockDefinition
        ↓
BlockState
        ↓
Chunk BlockStorage
        ↓
getBlock()
        ↓
setBlock()
        ↓
BlockPosition
        ↓
Basic Tags
        ↓
Collision
        ↓
Renderer
        ↓
Build & Destruction
        ↓
Save
        ↓
Load
```

---

# 174. Segundo Vertical Slice

Depois:

```text
Block
 ↓
BlockEntity
 ↓
Inventory Capability
 ↓
Chest
 ↓
Save
 ↓
Load
 ↓
Network
 ↓
Replication
```

---

# 175. Terceiro Vertical Slice

Depois:

```text
Machine Block
 ↓
BlockEntity
 ↓
Energy Capability
 ↓
Energy API
 ↓
Machine System
 ↓
Recipe
 ↓
Automation
```

---

# 176. Quarto Vertical Slice

Depois:

```text
Fluid Block
 ↓
Fluid Capability
 ↓
Fluid API
 ↓
Fluid Engine
 ↓
Pipe
 ↓
Tank
```

---

# 177. Quinto Vertical Slice

Depois:

```text
Mod
 ↓
register block
 ↓
register states
 ↓
register tags
 ↓
register capability
 ↓
load world
 ↓
save world
 ↓
reload world
```

Esse é o teste real de que a arquitetura é modular.

---

# 178. Estrutura de código sugerida

```text
src/
└── block/
    ├── core/
    │   ├── block.ts
    │   ├── block-definition.ts
    │   ├── block-state.ts
    │   ├── block-property.ts
    │   └── block-position.ts
    │
    ├── registry/
    │   ├── block-registry.ts
    │   ├── capability-registry.ts
    │   └── material-registry.ts
    │
    ├── storage/
    │   ├── block-storage.ts
    │   ├── palette.ts
    │   └── block-chunk.ts
    │
    ├── state/
    │   ├── state-schema.ts
    │   └── state-container.ts
    │
    ├── capability/
    │   ├── interactable.ts
    │   ├── container.ts
    │   ├── energy-port.ts
    │   ├── fluid-port.ts
    │   └── machine.ts
    │
    ├── entity/
    │   └── block-entity.ts
    │
    ├── interaction/
    │   ├── place.ts
    │   ├── break.ts
    │   └── interact.ts
    │
    ├── query/
    │   └── block-query.ts
    │
    ├── events/
    │   └── block-events.ts
    │
    ├── persistence/
    │   ├── serializer.ts
    │   └── migration.ts
    │
    ├── client/
    │   └── render-profile.ts
    │
    └── api/
        ├── block-api.ts
        └── capabilities.ts
```

---

# 179. Relação com os outros sistemas

```text
                    ┌───────────────┐
                    │ WORLD GEN     │
                    └───────┬───────┘
                            ↓
                      ┌───────────┐
                      │ BLOCK API │
                      └─────┬─────┘
                            ↓
                     BLOCK SYSTEM
                            │
        ┌────────────┬──────┼───────┬───────────┐
        ↓            ↓      ↓       ↓           ↓
     PHYSICS      LIGHT   FLUID   RENDER      BUILD
        │            │      │       │           │
        └────────────┴──────┴───────┴───────────┘
                            │
                    ┌───────┴────────┐
                    ↓                ↓
               BLOCK ENTITY      CONTENT/MODS
                    │
          ┌─────────┼─────────┐
          ↓         ↓         ↓
       MACHINE    ENERGY     FLUID
```

---

# 180. Fronteira arquitetural

## Block System faz

```text
identity
state
definition
properties
storage contract
registry
capabilities
queries
block entity association
block lifecycle
block events
serialization
```

## Não faz

```text
combat
AI
physics simulation
fluid simulation
lighting simulation
rendering
inventory logic
machine processing
economy
civilization
world generation
```

Ele fornece os dados e os contratos.

---

# 181. Block API pública final

A camada pública pode chegar a algo conceitualmente próximo de:

```text
IBlockRegistry
IBlockDefinition
IBlockState
IBlockProperty
IBlockQuery
IBlockCapability
IBlockEntity
IBlockInteraction
IBlockOperation
IBlockPersistence
IBlockMigration
```

E capabilities:

```text
IContainer
IEnergyPort
IFluidPort
IMachine
IInteractable
IHeatSource
IStorage
```

---

# 182. Regra fundamental

Eu colocaria esta regra no documento oficial:

> **Um bloco não deve possuir comportamento porque é "especial". Ele possui capacidades, propriedades e estados que permitem aos sistemas do NEXORA interpretar o que ele pode fazer.**

Isso evita que o projeto evolua para:

```text
if block == X
if block == Y
if block == Z
```

e transforma o sistema em:

```text
if hasCapability(...)
if hasTag(...)
if property(...)
if component(...)
```

---

# 183. Arquitetura final

```text
                 NEXORA CORE
                      │
              PUBLIC BLOCK API
                      │
              BLOCK REGISTRY
                      │
             BLOCK DEFINITIONS
                      │
                  BLOCK STATE
                      │
            ┌─────────┴──────────┐
            ↓                    ↓
      BLOCK STORAGE         BLOCK ENTITY
            │                    │
            ↓                    ↓
        CHUNK/Voxel          Components
            │                    │
    ┌───────┼────────┬───────────┼────────┐
    ↓       ↓        ↓           ↓        ↓
 Physics  Lighting  Fluids    Machines   Build
    │       │        │           │        │
    └───────┴────────┴───────────┴────────┘
                       │
                 VANILLA CONTENT
                       +
                 COMMUNITY MODS
```

# 184. Implementação

Eu dividiria o desenvolvimento em:

```text
BLOCK-0   Core contracts
BLOCK-1   BlockID
BLOCK-2   BlockRegistry
BLOCK-3   BlockDefinition
BLOCK-4   BlockState
BLOCK-5   BlockProperty
BLOCK-6   BlockPosition
BLOCK-7   BlockStorage
BLOCK-8   Palette
BLOCK-9   Block Tags
BLOCK-10  Material
BLOCK-11  Collision
BLOCK-12  Render Profile
BLOCK-13  Interaction API
BLOCK-14  Placement
BLOCK-15  Breaking
BLOCK-16  Neighbor Updates
BLOCK-17  Block Events
BLOCK-18  BlockEntity
BLOCK-19  Components
BLOCK-20  Capabilities
BLOCK-21  Chunk Integration
BLOCK-22  Lighting Integration
BLOCK-23  Physics Integration
BLOCK-24  Fluid Integration
BLOCK-25  Build Integration
BLOCK-26  Loot Integration
BLOCK-27  Persistence
BLOCK-28  Migration
BLOCK-29  Networking
BLOCK-30  Replication
BLOCK-31  Mod API
BLOCK-32  Missing Blocks
BLOCK-33  Debugging
BLOCK-34  Performance
BLOCK-35  Stress Tests
BLOCK-36  Compatibility
BLOCK-37  Official Content
```

E o **teste de ouro** desse sistema seria:

```text
NEXORA inicia
    ↓
Block Registry carrega
    ↓
registra Stone
    ↓
registra Machine
    ↓
registra Fluid Tank
    ↓
gera Chunk
    ↓
Chunk armazena BlockStates
    ↓
Player coloca bloco
    ↓
Build System solicita operação
    ↓
Block API valida
    ↓
Block colocado
    ↓
Lighting atualiza
    ↓
Physics atualiza
    ↓
Renderer atualiza
    ↓
BlockEntity nasce
    ↓
Energy/Fluid conectam
    ↓
mundo salva
    ↓
jogo fecha
    ↓
jogo abre
    ↓
chunk recarrega
    ↓
BlockStates restaurados
    ↓
BlockEntities restauradas
    ↓
máquina continua funcionando
```

Isso transforma o **Block System** em uma das peças centrais da arquitetura do NEXORA, mas sem transformar o sistema em um monólito.

### Dependências

A posição dele no mapa geral ficaria aproximadamente:

```text
CORE
 │
 ├── Registry
 ├── Event Bus
 ├── Chunk/Voxel
 │
 └── BLOCK API
       │
       └── BLOCK SYSTEM
             ├── Build & Destruction
             ├── Lighting
             ├── Physics
             ├── Fluids
             ├── Rendering
             ├── Machines
             ├── Energy
             ├── Inventory
             ├── Crafting
             ├── Loot
             ├── WorldGen
             └── Mod Runtime
```

E isso deixa uma decisão arquitetural muito importante definida: **Block, Item, Entity, Registry, Event Bus e Save/Persistence devem virar contratos-base do NEXORA; os sistemas especializados ficam por cima deles.**
