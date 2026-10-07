Perfeito. O **Item System** deve seguir a mesma filosofia do Block System: ser uma fundação modular, mas sem colocar dentro dele a lógica de Crafting, Inventory, Combat, Tools ou Economy.

A regra principal seria:

> **Item System define o que um item é, sua identidade, definição, estado e capacidades. Os outros sistemas dão significado ao item.**

# NEXORA — ITEM SYSTEM / ITEM API

## 1. Arquitetura central

```text id="nex-item-01"
ITEM SYSTEM
├── IDENTITY
├── DEFINITION
├── INSTANCE
├── STATE
├── STACK
├── COMPONENTS
├── CAPABILITIES
├── CONTAINMENT
├── DURABILITY
├── CUSTOM DATA
├── SERIALIZATION
└── REGISTRY

ITEM API
├── REGISTRATION
├── QUERY
├── CREATION
├── TRANSFORMATION
├── CAPABILITIES
├── EVENTS
└── MODDING
```

Arquitetura geral:

```text id="nex-item-02"
                NEXORA CORE
                     │
               PUBLIC ITEM API
                     │
                ITEM REGISTRY
                     │
             ITEM DEFINITIONS
                     │
          ┌──────────┴──────────┐
          ↓                     ↓
     ITEM INSTANCE          ITEM STACK
          │                     │
          └──────────┬──────────┘
                     ↓
                COMPONENTS
                     │
        ┌────────────┼────────────┐
        ↓            ↓            ↓
   Inventory      Tools        Equipment
        ↓            ↓            ↓
     Crafting      Combat      Machines
        │            │            │
        └────────────┼────────────┘
                     ↓
               Vanilla + Mods
```

---

# 2. O que é um Item?

Item não é apenas uma textura e não é necessariamente algo que o jogador carrega.

Pode representar:

```text id="nex-item-03"
material
tool
weapon
armor
food
fluid_container
machine_component
resource
document
key
quest_object
currency
vehicle_part
module
artifact
```

E futuramente:

```text id="nex-item-04"
research_sample
genetic_material
alien_object
dimensional_artifact
space_component
```

O sistema não precisa conhecer semanticamente cada categoria.

Ele precisa fornecer a infraestrutura.

---

# 3. ItemDefinition

Define o tipo de item.

```text id="nex-item-05"
ItemDefinition

id
displayNameKey
descriptionKey

maxStackSize
weight
volume

tags
components
capabilities

useProfile
equipmentProfile
toolProfile
renderProfile
audioProfile

rarity
valueProfile
```

Exemplo:

```text id="nex-item-06"
nexora:copper_ingot

maxStackSize = 128
weight = ...
tags:
    metal
    ingot
    conductive_material
```

---

# 4. ItemID

Identidade pública:

```text id="nex-item-07"
namespace:item
```

Exemplos:

```text id="nex-item-08"
nexora:stone
nexora:iron_ingot
nexora:diamond
nexora:pickaxe
```

Mods:

```text id="nex-item-09"
examplemod:titanium_ingot
examplemod:plasma_cell
```

Assim como Block:

> **Runtime ID pode existir internamente, mas nunca deve ser a identidade persistente do item.**

---

# 5. ItemType ≠ ItemInstance

Essa separação é fundamental.

```text id="nex-item-10"
ItemDefinition
      │
      ├── Item Instance A
      ├── Item Instance B
      └── Item Instance C
```

Todas são:

```text
nexora:iron_pickaxe
```

mas podem possuir estados diferentes.

Exemplo:

```text id="nex-item-11"
Pickaxe A
durability = 87%

Pickaxe B
durability = 14%

Pickaxe C
durability = 100%
```

---

# 6. ItemInstance

Representa um item concreto.

```text id="nex-item-12"
ItemInstance

itemId
instanceId
quantity
state
components
customData
```

Nem todos os itens precisam necessariamente de um `instanceId`.

Itens completamente fungíveis podem ser representados compactamente.

---

# 7. ItemStack

Essa é outra entidade essencial.

```text id="nex-item-13"
ItemStack

itemId
quantity
state/components
```

Exemplo:

```text
64 x stone
```

Pode existir como:

```text id="nex-item-14"
ItemStack
    item = nexora:stone
    quantity = 64
```

sem criar 64 objetos individuais.

---

# 8. Stackability

Nem todo item pode ser empilhado.

Exemplo:

```text id="nex-item-15"
Stone
→ stackable

Iron Ingot
→ stackable

Pickaxe
→ normalmente limitado pelo estado

Unique Artifact
→ non-stackable
```

A decisão depende do estado.

---

# 9. Stack Compatibility

Duas stacks somente podem ser combinadas se seus estados forem compatíveis.

```text id="nex-item-16"
CanStack(A, B)
```

deve verificar:

```text
item ID
components
durability policy
custom data
ownership rules
quality
metadata
```

Por exemplo:

```text
pickaxe durability 100
+
pickaxe durability 87
```

pode não poder ser fundido automaticamente.

---

# 10. Quantity

Quantidade deve suportar valores seguros e consistentes.

```text id="nex-item-17"
quantity
maxStackSize
```

Operações:

```text
add
remove
split
merge
transfer
```

Todas devem ser validadas.

---

# 11. Item State

Itens podem possuir pequenos estados.

Exemplo:

```text id="nex-item-18"
battery
charged = true

door_key
used = false

food
temperature = cold
```

Mas, assim como Block, não usar State como depósito universal.

---

# 12. Item Components

A arquitetura principal deve ser composta.

```text id="nex-item-19"
Item
├── DurabilityComponent
├── ToolComponent
├── WeaponComponent
├── FoodComponent
├── EquipmentComponent
├── FluidContainerComponent
├── EnergyComponent
├── StorageComponent
├── ModuleComponent
└── CustomComponent
```

Isso permite combinações.

---

# 13. Capability vs Component

Eu manteria a mesma regra do Block System.

### Component

Representa dados/comportamento interno do item.

```text id="nex-item-20"
DurabilityComponent
ToolComponent
```

### Capability

Representa algo que outros sistemas podem utilizar.

```text id="nex-item-21"
ITool
IWeapon
IFluidContainer
IEnergyStorage
IEquipable
IRepairable
IUsable
```

---

# 14. Item Capability API

Interface base:

```text id="nex-item-22"
IItemCapability
```

Exemplos:

```text id="nex-item-23"
IUsable
IEquipable
ITool
IWeapon
IRepairable
IContainer
IFluidContainer
IEnergyStorage
IModule
IScanner
IPlaceable
IThrowable
```

---

# 15. Item Use

Um item pode ser usado.

```text id="nex-item-24"
ItemUseRequest
```

Contexto:

```text
actor
item
target
position
face
world
```

Fluxo:

```text id="nex-item-25"
Player
 ↓
Item API
 ↓
validate use
 ↓
capability
 ↓
specialized system
 ↓
result
```

---

# 16. Usar ≠ Executar lógica dentro do Item System

Exemplo de uma espada.

Item System sabe:

```text
has WeaponCapability
```

Combat System sabe:

```text
como atacar
```

Então:

```text id="nex-item-26"
Item
 ↓
WeaponCapability
 ↓
Combat
```

---

# 17. Ferramentas

Tool System já existe como documento próprio.

Item System fornece:

```text id="nex-item-27"
ToolCapability
```

e o Tool API executa.

Exemplo:

```text
pickaxe
 ├── mining capability
 ├── durability
 └── material tier
```

---

# 18. Weapons

Mesmo princípio:

```text id="nex-item-28"
weapon
+
WeaponCapability
```

Combat resolve o combate.

---

# 19. Armor

Armor pode ser:

```text id="nex-item-29"
Item
+
EquipmentCapability
+
ArmorComponent
```

Equipment System decide:

```text slot
attributes
modifiers
loadout
```

Combat/Status System interpreta proteção.

---

# 20. Accessories

Como o NEXORA possui muitos slots de acessórios:

```text id="nex-item-30"
AccessoryCapability
```

O Inventory/Equipment System decide onde o item pode ser equipado.

---

# 21. Item Tags

Fundamental para extensibilidade.

```text id="nex-item-31"
#metal
#wood
#food
#tool
#weapon
#armor
#ore
#fuel
#machine_part
#building_material
#fluid_container
```

Recipe:

```text
#iron_ingot
```

em vez de uma lista enorme de IDs.

---

# 22. Item Materials

Pode existir um `MaterialDefinition` compartilhado com outros sistemas.

Exemplo:

```text id="nex-item-32"
iron
copper
steel
titanium
wood
crystal
organic
```

Tool System pode consumir propriedades materiais.

---

# 23. Material ≠ Item

Importante.

```text
iron
```

é material.

```text
iron_ingot
```

é item.

```text
iron_pickaxe
```

é item composto por material.

Não misturar os conceitos.

---

# 24. Item Weight

Peso pode existir:

```text id="nex-item-33"
weight
```

Inventory pode usar.

Physics pode usar para itens físicos.

Economy pode usar no transporte.

O Item System fornece o dado.

---

# 25. Volume

Alguns itens podem possuir:

```text id="nex-item-34"
volume
```

para sistemas avançados de inventário/logística.

Isso permite que a mochila tenha:

```text
weight limit
volume limit
slot limit
```

independentemente.

---

# 26. Rarity

Pode haver:

```text id="nex-item-35"
common
uncommon
rare
epic
legendary
unique
```

Mas rarity é uma propriedade de conteúdo.

Não deve controlar sozinha o balanceamento.

---

# 27. Quality

Separar rarity de quality.

Exemplo:

```text id="nex-item-36"
iron_sword
quality = 73%
```

Isso permite:

```text
poor
normal
fine
excellent
masterwork
```

sem transformar cada qualidade em um item diferente.

---

# 28. Durability

Pode ser um componente:

```text id="nex-item-37"
DurabilityComponent

current
maximum
damageRate
repairable
```

---

# 29. Durability Damage

Não colocar lógica de dano diretamente no item.

Tool/Weapon/Combat/Build solicita:

```text id="nex-item-38"
damageItem(item, amount)
```

Item System aplica o estado.

---

# 30. Repair

Capability:

```text id="nex-item-39"
IRepairable
```

Repair System ou Crafting/Machine decide como reparar.

---

# 31. Item Conditions

Itens podem precisar de:

```text id="nex-item-40"
temperature
charge
durability
purity
quality
contamination
```

Isso permite materiais e equipamentos complexos.

---

# 32. Purity

Especialmente importante para o NEXORA industrial.

```text id="nex-item-41"
purity = 98.4%
```

Pode afetar:

```text
crafting
machine efficiency
energy conversion
economy
research
```

---

# 33. Temperature

Um item pode possuir:

```text id="nex-item-42"
temperature
```

Exemplo:

```text
molten metal
heated ingot
frozen sample
```

Thermal System interpreta.

---

# 34. Contamination

Para ambientes complexos:

```text id="nex-item-43"
contamination
```

pode ser representada por component.

Ecology, Chemistry ou Research usam.

---

# 35. Item Identity Levels

Ter três níveis:

```text id="nex-item-44"
ITEM TYPE
ITEM INSTANCE
ITEM STACK
```

Exemplo:

```text
Item Type:
nexora:diamond

Stack:
64 diamonds
```

Mas:

```text
Item Type:
nexora:relic

Instance:
UUID 823...
owner = ...
history = ...
```

---

# 36. Unique Items

Artefatos únicos podem precisar de identidade individual.

```text id="nex-item-45"
PersistentItemUUID
```

Isso permite:

```text
history
ownership
quest binding
research history
provenance
```

---

# 37. Item Provenance

Um item pode registrar sua origem:

```text id="nex-item-46"
createdBy
createdAt
source
location
recipe
machine
creator
worldPhase
```

Isso é especialmente útil para itens raros.

---

# 38. Item History

Não guardar histórico completo em todos os itens.

Somente quando necessário.

```text id="nex-item-47"
HistoryPolicy
    NONE
    OPTIONAL
    IMPORTANT
    UNIQUE
```

---

# 39. Ownership

Itens podem possuir:

```text id="nex-item-48"
owner
```

Mas Ownership System decide regras.

Exemplo:

```text
quest item
private artifact
organization property
civilization treasury
```

---

# 40. Binding

Alguns itens podem ficar vinculados:

```text id="nex-item-49"
soulbound
questbound
team-bound
machine-bound
vehicle-bound
```

Isso deve ser capability/policy, não hardcode.

---

# 41. Item Container

Certos itens podem conter itens:

```text id="nex-item-50"
backpack
crate
toolbox
portable_storage
```

Capability:

```text
IItemContainer
```

Inventory System executa o armazenamento.

---

# 42. Nested Inventory

Precisa de limites.

Nunca permitir:

```text
backpack
 → backpack
   → backpack
      → ...
```

sem limite.

Definir:

```text id="nex-item-51"
maxNestingDepth
```

e peso/volume recursivo com limites.

---

# 43. Specialized Backpacks

Isso encaixa perfeitamente no sistema que já planejamos.

Exemplo:

```text id="nex-item-52"
Mining Backpack
    capacity
    filters
    routing rules
```

O item fornece a capability.

Inventory decide a operação.

---

# 44. Routing

Items containers podem possuir:

```text id="nex-item-53"
ItemRoutingProfile
```

Exemplo:

```text
ore
→ mining backpack

food
→ food pouch

building blocks
→ builder backpack
```

Routing System executa.

---

# 45. Filters

Item pode informar:

```text id="nex-item-54"
acceptedTags
acceptedItems
rejectedTags
```

---

# 46. Item Pickup

Um Item pode existir no mundo como:

```text id="nex-item-55"
ItemEntity
```

O Entity System controla a entidade.

Item System controla o conteúdo.

Portanto:

```text
ItemEntity
    ↓
ItemStack
```

---

# 47. Drop

Quando uma entidade dropa:

```text id="nex-item-56"
Mob
 ↓
Loot System
 ↓
ItemStack
 ↓
ItemEntity
```

Muito importante manter essa separação.

---

# 48. Block Drops

Mesmo:

```text id="nex-item-57"
Block
 ↓
Loot
 ↓
ItemStack
 ↓
ItemEntity
```

---

# 49. Physical Item Entity

A entidade física possui:

```text id="nex-item-58"
position
velocity
ItemStack
pickupState
mergeState
```

Entity System controla posição/lifecycle.

Item System controla o conteúdo.

---

# 50. Item Merge

Dois ItemEntities podem se fundir.

```text id="nex-item-59"
ItemEntity A
+
ItemEntity B
↓
merge compatible stacks
↓
ItemEntity C
```

Somente se forem compatíveis.

---

# 51. Item Despawn

Policy:

```text id="nex-item-60"
temporary
persistent
quest
important
unique
```

Loot/Entity System decide lifecycle.

---

# 52. Inventory Integration

Inventory recebe:

```text id="nex-item-61"
ItemStack
```

e faz:

```text
insert
remove
move
split
merge
transfer
```

Não deve duplicar definição de item.

---

# 53. Equipment Integration

Equipment recebe:

```text id="nex-item-62"
EquipmentCapability
```

e valida slots.

---

# 54. Crafting Integration

Crafting consulta:

```text id="nex-item-63"
ItemTag
ItemID
Material
Components
```

---

# 55. Tool Integration

Tool API consulta:

```text id="nex-item-64"
ITool
```

---

# 56. Combat Integration

Combat consulta:

```text id="nex-item-65"
IWeapon
```

---

# 57. Machine Integration

Machine pode consumir/produzir:

```text id="nex-item-66"
ItemStack
```

---

# 58. Energy Integration

Itens que armazenam energia:

```text id="nex-item-67"
Battery
+
IEnergyStorage
```

Energy API administra a energia.

---

# 59. Fluid Integration

Itens que carregam líquidos:

```text id="nex-item-68"
Bucket
+
IFluidContainer
```

Fluid API administra o fluido.

---

# 60. Chemistry / Processing

Item pode possuir:

```text id="nex-item-69"
CompositionComponent
```

Exemplo:

```text
Iron Ore
Fe = 71%
Stone = 20%
Impurities = 9%
```

Processing System interpreta.

---

# 61. Research Integration

Itens podem ser fontes de conhecimento.

```text id="nex-item-70"
ResearchSampleComponent
```

Research System pode extrair:

```text
observations
composition
unknown properties
```

---

# 62. Quest Integration

Itens podem ser objetivos:

```text id="nex-item-71"
QuestItemCapability
```

Mas Quest System controla progressão.

---

# 63. Economy Integration

Item pode possuir:

```text id="nex-item-72"
EconomicProfile
```

com:

```text
baseValue
weight
scarcity
supplyClass
```

Economy pode reajustar valor dinamicamente.

---

# 64. Currency

Moeda deve ser um item ou uma abstração?

Eu usaria os dois conceitos:

```text id="nex-item-73"
CurrencyDefinition
```

que pode ser representada por:

```text
ItemStack
```

quando física:

```text
gold_coin
```

ou por saldo abstrato:

```text
wallet balance
```

Assim o sistema não fica preso a dinheiro físico.

---

# 65. Item Attributes

Itens podem possuir modificadores:

```text id="nex-item-74"
AttributeModifier
```

Exemplo:

```text
miningSpeed +10%
energyEfficiency +5%
armor +7
```

Equipment/Combat interpreta.

---

# 66. Item Enchantments / Modifiers

Evitaria colocar "enchantment" como sistema central.

Criaria:

```text id="nex-item-75"
ItemModifier
```

Exemplo:

```text
sharpness
efficiency
thermal_resistance
energy_capacity
```

Isso é mais genérico.

---

# 67. Modular Items

NEXORA pode ter equipamentos compostos:

```text id="nex-item-76"
tool
├── head
├── handle
├── module
└── power_cell
```

Item System armazena composição.

Tool API determina comportamento.

---

# 68. Item Modules

Capability:

```text id="nex-item-77"
IModuleHost
```

Um item pode aceitar:

```text
scanner
battery
cooling
processor
weapon_module
```

---

# 69. Item Socket

```text id="nex-item-78"
SocketDefinition
```

com:

```text
type
allowedTags
capacity
```

---

# 70. Item Assembly

Crafting/Assembly System executa:

```text
parts
 ↓
assembly recipe
 ↓
new item
```

Item System registra o resultado.

---

# 71. Item Transformation

Um item pode transformar-se:

```text id="nex-item-79"
raw ore
 ↓
processing
 ↓
ingot
```

ou:

```text
battery
 ↓
charging
 ↓
charged battery
```

Usar:

```text id="nex-item-80"
ItemTransformation
```

quando necessário.

---

# 72. Copy vs Mutation

Para itens valiosos, operações devem ser transacionais.

```text id="nex-item-81"
clone
transform
consume
replace
```

evitando:

```text
consume
then error
```

que poderia duplicar ou apagar item.

---

# 73. Item Transaction

```text id="nex-item-82"
ItemTransaction
```

Pipeline:

```text
validate
reserve
apply
commit
rollback
```

Útil para:

* crafting
* trading
* machine processing
* inventory
* quests
* multiplayer

---

# 74. Atomic Transfer

```text id="nex-item-83"
transfer(
    source,
    destination,
    stack
)
```

precisa ser atômico quando os sistemas exigirem.

---

# 75. Duplication Prevention

Toda transformação importante pode possuir:

```text id="nex-item-84"
transactionId
```

Isso ajuda a evitar:

```text crafting duplication
trade duplication
machine duplication
network replay
```

---

# 76. Serialization

Item stack simples:

```text id="nex-item-85"
{
    itemId,
    quantity,
    state
}
```

Item complexo:

```text id="nex-item-86"
{
    itemId,
    instanceId,
    quantity,
    components,
    customData,
    version
}
```

---

# 77. Item Version

Cada definição pode possuir:

```text id="nex-item-87"
definitionVersion
schemaVersion
dataVersion
```

---

# 78. Migration

```text id="nex-item-88"
old item
 ↓
ItemMigrationRegistry
 ↓
new item
```

---

# 79. Missing Items

Assim como Blocks:

```text id="nex-item-89"
AVAILABLE
MISSING
INVALID
QUARANTINED
MIGRATABLE
```

Nunca simplesmente apagar.

---

# 80. Unknown Item

Exemplo:

Um mundo usa:

```text
examplemod:quantum_core
```

e o mod não está instalado.

O save pode preservar:

```text id="nex-item-90"
MissingItem
    originalID
    rawData
```

e restaurar quando o mod voltar.

---

# 81. Registry

Criar:

```text id="nex-item-91"
ItemRegistry
```

Operações:

```text
register
get
contains
iterate
freeze
```

---

# 82. Capability Registry

Separado:

```text id="nex-item-92"
ItemCapabilityRegistry
ComponentRegistry
ModifierRegistry
MaterialRegistry
```

---

# 83. Registry Freeze

```text id="nex-item-93"
load
 ↓
validate
 ↓
resolve dependencies
 ↓
freeze
 ↓
runtime
```

---

# 84. Runtime ID

```text id="nex-item-94"
ItemID
 ↓
RuntimeItemID
```

para acesso rápido.

---

# 85. Network ID

Pode existir:

```text id="nex-item-95"
NetworkItemID
```

válido apenas durante a sessão.

---

# 86. Save Identity

Persistência usa:

```text id="nex-item-96"
namespace:item
```

e não runtime ID.

---

# 87. Data-driven Item

Conceitualmente:

```json id="nex-item-97"
{
  "id": "examplemod:steel_ingot",
  "max_stack": 128,
  "weight": 1.2,
  "tags": [
    "metal",
    "ingot"
  ]
}
```

O runtime converte isso para uma definição eficiente.

---

# 88. Official Content

Exatamente como Blocks:

```text id="nex-item-98"
Public Item API
      ↓
Vanilla
      +
Mods
```

Nenhum sistema especial para "itens oficiais".

---

# 89. Mod Registration

Um mod pode registrar:

```text id="nex-item-99"
ItemDefinition
Component
Capability
Modifier
Tag
Material
```

---

# 90. Mod Dependency

Suportar:

```text id="nex-item-100"
requires
optional
conflicts
loadBefore
loadAfter
```

---

# 91. Mod Validation

Antes de aceitar:

```text id="nex-item-101"
ID válido?
definition válida?
component válido?
capability válida?
stack size válido?
dependências satisfeitas?
```

---

# 92. Security

Mods não devem receber acesso irrestrito a itens de outros mods.

O API pode separar:

```text id="nex-item-102"
READ
CREATE
MODIFY
DESTROY
REGISTRY
PRIVILEGED
```

---

# 93. Client/Server

COMMON:

```text id="nex-item-103"
ItemDefinition
ItemStack
Components
Capabilities
Registry
```

CLIENT:

```text id="nex-item-104"
RenderProfile
Model
Texture
Animation
```

SERVER:

```text id="nex-item-105"
Logic
Validation
Persistence
Transactions
```

---

# 94. Rendering

Item System fornece:

```text id="nex-item-106"
ItemRenderProfile
```

Renderer resolve:

```text
model
texture
material
animation
effects
```

---

# 95. First-person / Third-person

Render Profiles podem definir referências para:

```text id="nex-item-107"
inventory
world
firstPerson
thirdPerson
equipment
```

---

# 96. Sounds

```text id="nex-item-108"
ItemSoundProfile
```

Audio System usa:

```text
use
pickup
equip
break
drop
```

---

# 97. Particles

```text id="nex-item-109"
ItemEffectProfile
```

Particle System executa.

---

# 98. Localization

ItemDefinition usa:

```text id="nex-item-110"
displayNameKey
descriptionKey
```

Exemplo:

```text
item.nexora.iron_ingot
```

---

# 99. Item Interaction

```text id="nex-item-111"
IItemInteractable
```

Pode ter:

```text
canUse
use
canEquip
equip
canConsume
consume
```

Mas a implementação pode delegar aos sistemas apropriados.

---

# 100. Consume

Food/medicine-like systems podem usar:

```text id="nex-item-112"
ConsumableCapability
```

e fornecer:

```text
duration
effects
amount
conditions
```

O Status/Gameplay System aplica os efeitos.

---

# 101. Food

```text id="nex-item-113"
FoodComponent

nutrition
hydration
temperature
spoilage
taste
quality
```

Agriculture/Food System pode interpretar.

---

# 102. Spoilage

Não colocar um relógio em cada item.

Para stacks fungíveis pode usar:

```text id="nex-item-114"
batch timestamp
```

quando necessário.

---

# 103. Batch State

Exemplo:

```text id="nex-item-115"
64 apples
harvestTime = X
quality = 87
```

Isso permite stack eficiente.

---

# 104. Per-Item State

Se cada unidade for diferente:

```text id="nex-item-116"
ItemStack
    quantity = 4
    instances = [...]
```

Mas essa estrutura só aparece quando necessária.

---

# 105. Stack Optimization

Regra:

```text id="nex-item-117"
Fungible Item
→ compact stack

Non-fungible Item
→ individual instance
```

Isso é extremamente importante para performance.

---

# 106. Item Memory Model

Evitar:

```text
object por unidade
```

para bilhões de itens fungíveis.

Preferir:

```text
ItemDefinition
+
compact stack state
```

---

# 107. Item Database

Registry em memória durante execução.

Persistência em save.

Não usar banco de dados tradicional para cada item do mundo sem necessidade.

---

# 108. Chunk Item Storage

Inventários de chunks podem armazenar stacks.

ItemEntities ficam no Entity System.

---

# 109. Item Entity vs Item Stack

Separação:

```text id="nex-item-118"
ItemStack
= conteúdo

ItemEntity
= representação física desse conteúdo no mundo
```

---

# 110. Pickup

Fluxo:

```text id="nex-item-119"
Player
 ↓
detect ItemEntity
 ↓
Inventory
 ↓
merge stack
 ↓
ItemEntity remove/reduce
```

---

# 111. Item Lock

Durante operações:

```text id="nex-item-120"
reserved
inTransit
locked
```

para impedir dupla utilização.

---

# 112. Crafting Reservation

Crafting pode reservar:

```text id="nex-item-121"
10 iron
```

antes de iniciar.

Se falhar:

```text
rollback
```

---

# 113. Machine Reservation

Mesmo para máquinas:

```text id="nex-item-122"
Input
 ↓
reserve
 ↓
process
 ↓
consume
 ↓
output
```

---

# 114. Trading

Economy pode executar:

```text id="nex-item-123"
TradeTransaction
```

com:

```text
ItemStack
+
Currency
```

---

# 115. NPC Inventory

NPC usa o mesmo:

```text id="nex-item-124"
Inventory API
```

que o jogador.

Não criar um sistema diferente.

---

# 116. Civilization Storage

Armazéns de cidade:

```text id="nex-item-125"
Warehouse
Storage
Stockpile
```

podem armazenar ItemStacks.

---

# 117. Logistics

Rail/transport pode mover:

```text id="nex-item-126"
ItemStacks
```

sem conhecer internamente todos os itens.

---

# 118. Vehicle Cargo

Veículos:

```text id="nex-item-127"
CargoContainer
```

usam Inventory + Item API.

---

# 119. Space Cargo

Mesma API:

```text id="nex-item-128"
ship
 ↓
cargo inventory
 ↓
ItemStacks
```

---

# 120. Item Metadata

Pode existir:

```text id="nex-item-129"
CustomData
```

mas com schema e limites.

Nunca deixar um campo arbitrário infinito.

---

# 121. Component Serialization

Cada componente pode registrar:

```text id="nex-item-130"
serializer
deserializer
version
migration
```

---

# 122. Component Lifecycle

```text id="nex-item-131"
created
attached
updated
removed
serialized
```

---

# 123. Item Events

Eventos principais:

```text id="nex-item-132"
ItemCreated
ItemDestroyed
ItemStacked
ItemSplit
ItemMerged
ItemTransferred
ItemUsed
ItemEquipped
ItemUnequipped
ItemConsumed
ItemTransformed
ItemDamaged
ItemRepaired
```

---

# 124. Event Bus Integration

```text id="nex-item-133"
Item System
      ↓
Event Bus
      ↓
Crafting
Inventory
Economy
Quest
Research
Machines
```

---

# 125. Audit Trail

Transações importantes podem registrar:

```text id="nex-item-134"
transactionId
actor
source
destination
quantity
item
timestamp
```

---

# 126. Replay / Debug

Com eventos suficientes pode-se futuramente reproduzir problemas:

```text id="nex-item-135"
Item created
 ↓
transferred
 ↓
modified
 ↓
consumed
```

---

# 127. Query API

Criar:

```text id="nex-item-136"
IItemQuery
```

Exemplos:

```text
query.tag("metal")
query.id(...)
query.capability(...)
query.component(...)
```

---

# 128. Search API

Inventários grandes precisam:

```text id="nex-item-137"
search
filter
sort
group
```

Mas a lógica visual fica no UI.

---

# 129. Item Comparison

API pode fornecer:

```text id="nex-item-138"
ItemSnapshot
```

para UI comparar:

```text
weight
durability
attributes
capabilities
quality
```

---

# 130. Item Description

Descrição final não precisa ser fixa.

Pode ser formada por:

```text id="nex-item-139"
base description
+
components
+
modifiers
+
state
```

---

# 131. Dynamic Tooltip Data

Item API pode fornecer:

```text id="nex-item-140"
ItemDisplayData
```

UI transforma em tooltip.

---

# 132. Item Icon

Asset reference:

```text id="nex-item-141"
item.icon
```

UI/Renderer resolve.

---

# 133. Item Registry Snapshot

Save multiplayer pode registrar:

```text id="nex-item-142"
registry snapshot
```

para compatibilidade.

---

# 134. World Save

Salvar:

```text id="nex-item-143"
item id
state
components
version
```

Não salvar necessariamente:

```text
runtime id
render data
cached data
```

---

# 135. Network Replication

Servidor pode replicar apenas:

```text id="nex-item-144"
delta
```

Exemplo:

```text
quantity 64 → 63
```

em vez de mandar o stack inteiro.

---

# 136. Client Prediction

Pode haver previsão para:

```text id="nex-item-145"
pickup
inventory movement
UI interactions
```

mas servidor valida.

---

# 137. Server Authority

```text id="nex-item-146"
Client
 ↓
Item Operation Request
 ↓
Server
 ↓
Validate
 ↓
Transaction
 ↓
Commit
 ↓
Replicate
```

---

# 138. Anti-Duplication

Sistema precisa detectar:

```text id="nex-item-147"
same transaction twice
invalid quantity
negative quantity
forged item ID
forged component
invalid ownership
invalid source
```

---

# 139. Security Boundary

Nunca permitir que o cliente simplesmente diga:

```text
give me 999 diamonds
```

A operação deve passar por autoridade do servidor.

---

# 140. Item API Versioning

```text id="nex-item-148"
Item API v1
Item API v2
```

com compatibilidade planejada.

---

# 141. Compatibility Adapter

Mods antigos:

```text id="nex-item-149"
Legacy API
 ↓
Compatibility Adapter
 ↓
Current API
```

---

# 142. Performance — Registry

Registry deve ser otimizado para:

```text id="nex-item-150"
O(1) lookup
```

ou estrutura equivalente.

---

# 143. Performance — Tags

Tags devem ser resolvidas para estruturas eficientes:

```text id="nex-item-151"
Tag
 ↓
compact set of ItemIDs
```

---

# 144. Performance — Components

Não colocar todos os componentes em todos os itens.

```text id="nex-item-152"
stone
→ minimal

pickaxe
→ durability + tool

machine module
→ energy + module + ...
```

---

# 145. Performance — Instances

Só criar identidade individual quando necessário:

```text id="nex-item-153"
fungible
→ stack

unique
→ instance
```

---

# 146. Performance — Serialization

Preferir:

```text id="nex-item-154"
compact binary representation
```

para saves e networking, mantendo formatos legíveis para definição/mod data.

---

# 147. Stress Test

Testar:

```text id="nex-item-155"
1,000,000 stacks
10,000,000 stacks
```

com:

```text
insert
merge
split
transfer
serialize
deserialize
query
```

---

# 148. Stress Test — Inventory

```text id="nex-item-156"
100,000 inventories
```

com diferentes cargas.

---

# 149. Stress Test — World Drops

```text id="nex-item-157"
100,000 ItemEntities
```

e testar:

```text
merge
pickup
despawn
streaming
networking
```

---

# 150. Fuzz Testing

Gerar aleatoriamente:

```text id="nex-item-158"
ItemStacks
Components
quantities
states
transactions
```

e garantir:

```text
quantity >= 0
no invalid state
no duplication
no lost items
```

---

# 151. Determinism

Operações críticas devem ser determinísticas:

```text id="nex-item-159"
stacking
splitting
transactions
serialization
migration
```

---

# 152. Golden Tests

Salvar snapshots esperados:

```text id="nex-item-160"
item serialization
item migration
stack operations
registry
```

---

# 153. Teste de ouro

O primeiro item oficial:

```text id="nex-item-161"
nexora:stone
```

deve conseguir:

```text
register
 ↓
create ItemStack
 ↓
place in Inventory
 ↓
save
 ↓
load
 ↓
drop
 ↓
ItemEntity
 ↓
pickup
 ↓
merge
 ↓
save again
```

---

# 154. Segundo vertical slice

```text id="nex-item-162"
iron_pickaxe
 ↓
Item
 ↓
ToolCapability
 ↓
DurabilityComponent
 ↓
Tool API
 ↓
Build & Destruction
 ↓
damage durability
 ↓
save/load
```

---

# 155. Terceiro vertical slice

```text id="nex-item-163"
battery
 ↓
EnergyStorageCapability
 ↓
Energy API
 ↓
Machine
 ↓
consume energy
 ↓
change item state
 ↓
save/load
```

---

# 156. Quarto vertical slice

```text id="nex-item-164"
bucket
 ↓
FluidContainerCapability
 ↓
Fluid API
 ↓
fill
 ↓
drain
 ↓
save/load
```

---

# 157. Quinto vertical slice

```text id="nex-item-165"
Custom Mod
 ↓
register Item
 ↓
register Component
 ↓
register Capability
 ↓
create stack
 ↓
inventory
 ↓
save
 ↓
reload
 ↓
item restored
```

Esse é o verdadeiro teste de modularidade.

---

# 158. Estrutura de código

Eu organizaria assim:

```text id="nex-item-166"
src/
└── item/
    ├── core/
    │   ├── item.ts
    │   ├── item-definition.ts
    │   ├── item-instance.ts
    │   ├── item-stack.ts
    │   ├── item-state.ts
    │   └── item-quantity.ts
    │
    ├── registry/
    │   ├── item-registry.ts
    │   ├── component-registry.ts
    │   ├── capability-registry.ts
    │   └── modifier-registry.ts
    │
    ├── components/
    │   ├── durability.ts
    │   ├── tool.ts
    │   ├── weapon.ts
    │   ├── equipment.ts
    │   ├── food.ts
    │   ├── container.ts
    │   └── custom.ts
    │
    ├── capabilities/
    │   ├── usable.ts
    │   ├── equipable.ts
    │   ├── tool.ts
    │   ├── weapon.ts
    │   ├── fluid-container.ts
    │   ├── energy-storage.ts
    │   └── module-host.ts
    │
    ├── transaction/
    │   ├── transaction.ts
    │   ├── transfer.ts
    │   ├── merge.ts
    │   └── split.ts
    │
    ├── query/
    │   └── item-query.ts
    │
    ├── events/
    │   └── item-events.ts
    │
    ├── persistence/
    │   ├── serializer.ts
    │   ├── migration.ts
    │   └── missing-item.ts
    │
    ├── networking/
    │   └── replication.ts
    │
    ├── client/
    │   ├── render-profile.ts
    │   └── display-data.ts
    │
    └── api/
        ├── item-api.ts
        └── capabilities.ts
```

---

# 159. Integração com Block System

Agora conseguimos uma arquitetura muito limpa:

```text id="nex-item-167"
BLOCK
 ↓
Break
 ↓
Loot
 ↓
ITEM STACK
```

e:

```text
ITEM
 ↓
PlaceableCapability
 ↓
BUILD SYSTEM
 ↓
BLOCK
```

Assim um item pode representar um bloco colocável sem fundir os dois sistemas.

---

# 160. Relação Block ↔ Item

Um bloco não precisa necessariamente ter um item correspondente.

Pode existir:

```text
block only
```

e um item pode também:

```text
item only
```

Exemplos:

```text
invisible gameplay block
```

ou:

```text
key
currency
research sample
```

---

# 161. Item ↔ Entity

```text id="nex-item-168"
ITEM
 ↓
ITEM ENTITY
```

A Entity System controla existência física.

---

# 162. Item ↔ Inventory

```text id="nex-item-169"
ITEM
 ↓
ITEM STACK
 ↓
INVENTORY
```

Inventory não redefine Item.

---

# 163. Item ↔ Equipment

```text id="nex-item-170"
ITEM
 ↓
EquipmentCapability
 ↓
Equipment System
```

---

# 164. Item ↔ Crafting

```text id="nex-item-171"
ITEM TAGS
 ↓
CRAFTING
```

---

# 165. Item ↔ Tools

```text id="nex-item-172"
ITEM
 ↓
ToolCapability
 ↓
Tool API
```

---

# 166. Item ↔ Combat

```text id="nex-item-173"
ITEM
 ↓
WeaponCapability
 ↓
Combat
```

---

# 167. Item ↔ Energy

```text id="nex-item-174"
ITEM
 ↓
EnergyCapability
 ↓
Energy API
```

---

# 168. Item ↔ Fluid

```text id="nex-item-175"
ITEM
 ↓
FluidContainerCapability
 ↓
Fluid API
```

---

# 169. Item ↔ Machines

```text id="nex-item-176"
ITEM
 ↓
Machine input/output
 ↓
Machine System
```

---

# 170. Item ↔ Economy

```text id="nex-item-177"
ITEM
 ↓
Economic Profile
 ↓
Economy
```

---

# 171. Item ↔ Research

```text id="nex-item-178"
ITEM
 ↓
Research Capability
 ↓
Knowledge System
```

---

# 172. Item ↔ Quest

```text id="nex-item-179"
ITEM
 ↓
Quest condition
 ↓
Quest System
```

---

# 173. Item ↔ Civilization

NPCs podem manipular:

```text id="nex-item-180"
ItemStacks
```

exatamente como jogadores.

Isso é importante para a arquitetura de economia do NEXORA.

---

# 174. Item ↔ Logistics

```text id="nex-item-181"
Mine
 ↓
ItemStack
 ↓
Storage
 ↓
Rail
 ↓
City
 ↓
Factory
```

Isso conecta diretamente mineração → indústria → civilização.

---

# 175. Item ↔ World Simulation

Um recurso:

```text ore block
 ↓
break
 ↓
ore item
 ↓
processing
 ↓
ingot
 ↓
machine
 ↓
product
 ↓
economy
```

Assim o Item System vira uma das principais pontes econômicas do mundo.

---

# 176. Item Lifecycle

```text id="nex-item-182"
DEFINED
 ↓
CREATED
 ↓
STACKED
 ↓
STORED
 ↓
TRANSFERRED
 ↓
USED
 ↓
TRANSFORMED
 ↓
CONSUMED
```

Para itens físicos:

```text id="nex-item-183"
CREATED
 ↓
DROPPED
 ↓
ITEM ENTITY
 ↓
PICKED UP
 ↓
STORED
```

---

# 177. Item Lifecycle não significa destruir fisicamente tudo

Um stack de 64 pode virar:

```text
64 → 63
```

sem criar/destruir 64 objetos individuais.

Isso preserva performance.

---

# 178. API pública final

Eu fecharia a API principal com:

```text id="nex-item-184"
IItem
IItemDefinition
IItemInstance
IItemStack
IItemState

IItemRegistry
IItemQuery

IItemComponent
IItemCapability

IItemTransaction
IItemTransfer

IItemSerializer
IItemMigration

IItemInteraction
```

Capabilities:

```text id="nex-item-185"
IUsable
IEquipable
ITool
IWeapon
IRepairable
IContainer
IFluidContainer
IEnergyStorage
IModuleHost
IPlaceable
```

---

# 179. Ordem de implementação

```text id="nex-item-186"
ITEM-0   Core Contracts
ITEM-1   ItemID
ITEM-2   ItemRegistry
ITEM-3   ItemDefinition
ITEM-4   ItemInstance
ITEM-5   ItemStack
ITEM-6   Quantity
ITEM-7   ItemState
ITEM-8   Tags
ITEM-9   Components
ITEM-10  Capabilities
ITEM-11  Durability
ITEM-12  Item Transactions
ITEM-13  Split/Merge
ITEM-14  Transfer
ITEM-15  Query
ITEM-16  Inventory Integration
ITEM-17  Equipment Integration
ITEM-18  Tool Integration
ITEM-19  Combat Integration
ITEM-20  Crafting Integration
ITEM-21  Block Integration
ITEM-22  ItemEntity Integration
ITEM-23  Energy Integration
ITEM-24  Fluid Integration
ITEM-25  Machine Integration
ITEM-26  Economy Integration
ITEM-27  Quest Integration
ITEM-28  Research Integration
ITEM-29  Serialization
ITEM-30  Migration
ITEM-31  Missing Items
ITEM-32  Networking
ITEM-33  Replication
ITEM-34  Client Rendering
ITEM-35  Mod API
ITEM-36  Validation
ITEM-37  Debugging
ITEM-38  Performance
ITEM-39  Stress Tests
ITEM-40  Compatibility
```

---

# 180. Arquitetura final

```text id="nex-item-187"
                         NEXORA CORE
                              │
                      PUBLIC ITEM API
                              │
                       ITEM REGISTRY
                              │
                     ITEM DEFINITIONS
                              │
                ┌─────────────┴─────────────┐
                ↓                           ↓
          ITEM INSTANCE                 ITEM STACK
                │                           │
                └─────────────┬─────────────┘
                              ↓
                        COMPONENTS
                              │
                        CAPABILITIES
                              │
        ┌─────────┬───────────┼───────────┬─────────┐
        ↓         ↓           ↓           ↓         ↓
   INVENTORY   EQUIPMENT    TOOL       COMBAT    CRAFTING
        │         │           │           │         │
        └─────────┴───────────┴───────────┴─────────┘
                              │
           ┌──────────────────┼───────────────────┐
           ↓                  ↓                   ↓
        ENERGY             FLUID               MACHINES
           │                  │                   │
           └──────────────────┼───────────────────┘
                              ↓
                      ECONOMY / RESEARCH
                              │
                              ↓
                    VANILLA + COMMUNITY MODS
```

E a regra que eu colocaria no documento:

> **Item System fornece identidade, definição, estado, composição, capacidades e operações fundamentais de itens. Inventory armazena. Equipment equipa. Tool API dá capacidades de ferramenta. Combat resolve combate. Crafting transforma. Machines processam. Energy e Fluid fornecem infraestrutura. Economy atribui valor. Entity System representa itens físicos no mundo.**

Isso fecha a segunda grande peça depois do **Block System** e deixa o caminho pronto para os próximos pilares: **Registry System → Event Bus → Save/Persistence**.
