# NEXORA

# MASTER PLAN — INVENTORY + EQUIPMENT + LOADOUT SYSTEM

> O Inventory System será um dos sistemas mais completos do NEXORA.
>
> O objetivo não é apenas permitir que o jogador carregue itens, mas transformar **capacidade de carga, organização, especialização de equipamento e preparação para diferentes atividades** em parte importante do gameplay.
>
> O sistema deve permitir que o jogador configure seu personagem para mineração, exploração, agricultura, combate, construção, espaço, magia e outras atividades.

---

# 1. VISÃO GERAL

O sistema será dividido em:

```text id="9s6p4m"
INVENTORY
├── Main Inventory
├── Hotbar
├── Backpack
├── Specialized Bags
├── Equipment
├── Accessories
├── Armor Loadouts
├── Tools
├── Containers
└── Storage Networks
```

---

# 2. PRINCÍPIO FUNDAMENTAL

O jogador não deve possuir simplesmente:

```text id="c7n1hf"
"um inventário gigante"
```

Ele deverá possuir:

```text id="j6w1fv"
um sistema de armazenamento configurável.
```

---

# 3. INVENTÁRIO PRINCIPAL

Criar:

```text id="x5pj8g"
MainInventory
```

com:

* slots;
* stacks;
* filtros;
* ordenação;
* pesquisa;
* drag & drop;
* quick move;
* split stack;
* merge;
* lock slot;
* favorite item.

---

# 4. HOTBAR

Criar uma hotbar separada:

```text id="h0ub7y"
Hotbar
```

para:

* ferramentas;
* blocos;
* comida;
* itens rápidos.

---

# 5. STACK SYSTEM

Cada ItemStack possui:

```text id="6t9g9l"
itemId
quantity
durability
metadata
```

---

# 6. STACK SIZE

O tamanho máximo de stack deve ser definido pelo item/configuração.

Exemplo:

```text id="hj0r0e"
stone → stack normal
rare crystal → stack menor
special machine → stack = 1
```

---

# 7. MOCHILAS

Criar:

```text id="an99e7"
Backpack System
```

Cada mochila é um container independente.

---

# 8. MOCHILAS ESPECIALIZADAS

O jogador poderá possuir mochilas especializadas.

Exemplos:

```text id="3y6w8v"
Mining Backpack
Explorer Backpack
Builder Backpack
Farmer Backpack
Hunter Backpack
Magic Backpack
Engineer Backpack
Space Backpack
```

---

# 9. MINING BACKPACK

A mochila de mineração terá regras próprias.

Pode possuir:

```text id="ux5yha"
ore filter
stone filter
deep-resource filter
high stack capacity
```

---

# 10. STACK MULTIPLIER

Mochilas avançadas podem aumentar a capacidade efetiva de armazenamento.

Exemplo conceitual:

```text id="f1f4y8"
Tier I
×1

Tier II
×2

Tier III
×4

Tier IV
×8
```

Os valores reais devem ser balanceados.

---

# 11. SPECIALIZED INVENTORY RULES

Uma mochila pode aceitar apenas certas categorias.

Exemplo:

```text id="rq4w7k"
Mining Backpack
→ ores
→ stone
→ geological materials
```

O restante não entra automaticamente.

---

# 12. AUTO-SORT

Criar:

```text id="1qg6i4"
SortInventory()
```

Possibilidades:

* categoria;
* nome;
* quantidade;
* raridade;
* tipo.

---

# 13. AUTO-PICKUP

Criar opcionalmente regras:

```text id="bbpzpa"
auto collect compatible items
```

Exemplo:

```text id="pjq0ht"
ore
→ mining backpack

seed
→ farming backpack
```

---

# 14. ITEM DESTINATION

Criar um sistema:

```text id="f3hr54"
ItemRouting
```

para decidir onde um item deve ser armazenado.

---

# 15. CONTAINER PRIORITY

Exemplo:

```text id="jo27il"
Mining Backpack
priority = high

Main Inventory
priority = normal
```

---

# 16. EXCESS HANDLING

Se uma mochila estiver cheia:

```text id="l0al2b"
next compatible container
↓
main inventory
↓
pickup ground
```

---

# 17. ACCESSORY SYSTEM

O personagem poderá possuir até:

```text id="5s3m0q"
32 accessory slots
```

A quantidade disponível dependerá da progressão do personagem.

---

# 18. ACCESSORY CATEGORIES

Criar categorias:

```text id="xg5f0j"
head
neck
back
hands
belt
ring
utility
tool
mobility
protection
technology
magic
```

As categorias são extensíveis.

---

# 19. 32 ACCESSORIES

Os 32 slots não devem obrigatoriamente ser todos idênticos.

Criar:

```text id="i0y5zx"
AccessorySlot
```

com possíveis restrições.

Exemplo:

```text id="be7q0q"
slot 1 = general
slot 2 = general
slot 3 = ring
slot 4 = ring
...
```

---

# 20. ACCESSORY POWER

A quantidade de acessórios é grande, portanto existirão sistemas de:

```text id="jy9q34"
weight
energy
compatibility
conflicts
```

para evitar combinações absurdas.

---

# 21. ACCESSORY CONFLICTS

Exemplo:

```text id="w4a0lq"
Accessory A
conflicts with
Accessory B
```

---

# 22. ACCESSORY SETS

Criar bônus de conjuntos.

Exemplo:

```text id="5gqvpp"
3 mining accessories
→ mining bonus
```

---

# 23. ARMOR SYSTEM

O jogador possui um conjunto equipado normalmente.

Além dele:

```text id="xkq4wy"
3 additional armor loadout slots
```

Total:

```text id="8cwn73"
4 complete armor sets
```

---

# 24. ARMOR LOADOUT

Cada conjunto deve armazenar:

```text id="y40q1e"
helmet
chest
legs
boots
accessories
```

ou a quantidade de peças definida pelo jogo.

---

# 25. LOADOUT SWITCHING

Permitir troca rápida:

```text id="69w7wy"
Loadout 1
Loadout 2
Loadout 3
Loadout 4
```

---

# 26. EXEMPLOS

### Mining

```text id="v9q0j3"
Mining armor
Mining accessories
Mining backpack
Mining tools
```

### Exploration

```text id="y8zq51"
Explorer armor
Navigation accessories
Explorer backpack
```

### Space

```text id="49mx3y"
Space suit
Life support
Space backpack
```

---

# 27. RAPID SWITCH

Criar:

```text id="30x3o4"
Quick Loadout Switch
```

Troca:

```text id="0t3vmy"
armor
accessories
backpack
tool configuration
```

de forma rápida.

---

# 28. LOADOUT SAFETY

Não permitir troca de loadout se:

* item estiver ausente;
* peça incompatível;
* requisito não atendido.

Mostrar o motivo.

---

# 29. LOADOUT STORAGE

O loadout deve ser persistente.

---

# 30. EQUIPMENT POWER

Equipamentos avançados podem possuir:

```text id="v6x7i4"
energy
heat
durability
modules
```

---

# 31. CHARACTER EQUIPMENT API

Criar:

```text id="lwt5q4"
EquipmentManager
```

responsável por:

```text id="n0w9n0"
equip
unequip
swap
validate
calculateStats
```

---

# 32. STAT SYSTEM

Criar sistema central de atributos.

Exemplo:

```text id="3k6v0x"
movementSpeed
miningSpeed
armor
resistance
storageCapacity
energyCapacity
```

---

# 33. MODIFIERS

Itens podem aplicar:

```text id="6sp4m2"
+5 mining speed
+20 storage
+10 heat resistance
```

---

# 34. EQUIPMENT SYNERGY

Combinações podem gerar efeitos adicionais.

---

# 35. WEIGHT SYSTEM

O jogo pode possuir peso opcional.

```text id="2w9m9i"
item weight
+
container capacity
=
carry load
```

---

# 36. ENCUMBRANCE

Excesso de carga pode afetar:

```text id="vc51xa"
movement
stamina
energy
```

Os valores devem ser balanceados para não tornar o inventário irritante.

---

# 37. BACKPACK CAPACITY

Cada mochila possui:

```text id="p7m28e"
capacity
stack rules
filter
tier
```

---

# 38. BACKPACK UPGRADE

Permitir:

```text id="4k9jrs"
upgrade
```

sem necessariamente substituir todo o item.

---

# 39. BACKPACK MODULES

Mochilas avançadas podem possuir módulos:

```text id="gl7s5h"
filter
auto-sort
auto-pickup
compress
cooling
energy
```

---

# 40. ITEM CONTAINERS

Itens poderão conter outros itens.

Exemplo:

```text id="7v5l6b"
backpack
crate
container
machine
vehicle
```

---

# 41. CONTAINER NESTING

Definir limites.

Evitar:

```text id="r3ql3n"
backpack
→ backpack
→ backpack
→ backpack
→ infinite
```

---

# 42. INVENTORY SECURITY

Evitar exploits de duplicação.

Testar:

```text id="gu8ht9"
split
merge
drop
pickup
container transfer
save/load
```

---

# 43. INVENTORY TRANSACTIONS

Criar uma abstração:

```text id="6w5k9x"
InventoryTransaction
```

para operações atômicas quando necessário.

---

# 44. SAVE/LOAD

Persistir:

```text id="l6lglq"
main inventory
hotbar
backpacks
accessories
armor loadouts
equipment
```

---

# 45. EQUIPMENT PROGRESSION

A progressão do personagem desbloqueia capacidade.

---

# 46. SLOT UNLOCK SYSTEM

O sistema pode começar com poucos slots.

Exemplo:

```text id="j1pr4m"
Base
→ poucos acessórios
→ poucos loadouts
```

Depois:

```text id="u6y3xi"
consumir item de desbloqueio
↓
novo slot
```

---

# 47. CONSUMABLE UNLOCK ITEMS

Criar uma classe de item:

```text id="4zgrf4"
UnlockConsumable
```

Possíveis tipos:

```text id="cwn08x"
AccessorySlotUnlock
LoadoutUnlock
BackpackUnlock
EquipmentCapacityUnlock
```

---

# 48. CONSUMPTION RULE

Quando usado:

```text id="6i0tjr"
validate
↓
consume
↓
unlock
↓
save
```

---

# 49. PERMANENT UNLOCK

O desbloqueio deve ser permanente para o personagem, salvo regras específicas.

---

# 50. DISCOVERY

Alguns unlock items podem ser descobertos:

```text id="w06ewp"
chests
bosses
exploration
quests
rare structures
```

---

# 51. PROGRESSION NOT PAY-TO-WIN

Os desbloqueios devem ser conquistados dentro do gameplay.

---

# 52. LOADOUT UNLOCKS

Exemplo:

```text id="8k1fpr"
Base
→ Loadout 1

Unlock Item
→ Loadout 2

Another Unlock
→ Loadout 3

Rare Unlock
→ Loadout 4
```

---

# 53. ACCESSORY UNLOCKS

Mesma lógica para os 32 slots.

---

# 54. CHARACTER PROFILE

Criar:

```text id="a3h7l3"
CharacterProfile
```

com:

```text id="a28q0c"
inventory
equipment
accessories
loadouts
unlocks
stats
```

---

# 55. CHARACTER LOADOUT PRESETS

Além dos 4 conjuntos, permitir presets nomeados:

```text id="4n4xjp"
Mining
Builder
Explorer
Magic
Space
Combat
```

Um preset referencia um dos slots físicos de loadout.

---

# 56. ARMOR LOADOUT + BACKPACK

Quando trocar loadout:

```text id="xk4hnn"
armor
+
accessories
+
backpack
```

podem trocar juntos.

---

# 57. LOADOUT VALIDATION

Antes de equipar:

```text id="dy4u9w"
requirements
dependencies
conflicts
capacity
```

---

# 58. QUICK SWITCH INTERFACE

Criar uma interface rápida:

```text id="3ibqla"
[1] Mining
[2] Exploration
[3] Builder
[4] Space
```

---

# 59. INVENTORY UI

A interface deve possuir:

```text id="8ix6f7"
main inventory
hotbar
backpack
equipment
accessories
loadouts
```

---

# 60. SEARCH

Pesquisar:

```text id="9d33aw"
name
tag
category
rarity
```

---

# 61. FILTERS

Exemplos:

```text id="qf1k3f"
blocks
ores
tools
food
machines
magic
space
```

---

# 62. ITEM TAGS

Criar sistema:

```text id="hsxr8i"
#ore
#weapon
#food
#machine
#magic
#space
```

---

# 63. SMART INVENTORY

Permitir regras:

```text id="4e8h30"
"todos os minérios vão para a mochila de mineração"
```

---

# 64. INVENTORY PRESETS

Criar presets:

```text id="u9fi7k"
Mining preset
Farming preset
Exploration preset
```

---

# 65. AUTO-EQUIP

Quando pegar uma ferramenta apropriada:

possibilitar equipá-la rapidamente.

---

# 66. CONTAINER INTERACTION

Abrir:

```text id="vxd7en"
chest
machine
backpack
vehicle
```

usando a mesma Inventory API.

---

# 67. INVENTORY API

Criar API pública:

```text id="t1h86o"
Inventory
Container
ItemStack
Backpack
Equipment
Accessory
Loadout
```

---

# 68. MODDING

Mods oficiais e externos poderão criar:

```text id="g91l9r"
new backpack
new accessory
new loadout
new equipment slot
new inventory rule
```

---

# 69. NO CORE COUPLING

O Core não deve conhecer:

```text id="k3l3wi"
Mining Backpack
Space Backpack
Magic Backpack
```

Ele apenas conhece:

```text id="j3lxd5"
Container
Equipment
Item
```

---

# 70. OFFICIAL CONTENT

As mochilas oficiais do NEXORA usam a mesma API que mochilas de mods.

---

# 71. ITEM STORAGE NETWORK

Integrar futuramente com Storage Network.

---

# 72. REMOTE INVENTORY

Sistemas tecnológicos poderão permitir acesso remoto.

---

# 73. DIGITAL STORAGE

O sistema poderá suportar armazenamento digital através da Storage API.

---

# 74. VEHICLE INVENTORIES

Veículos também utilizarão a Inventory API.

---

# 75. CHEST / STORAGE BLOCKS

Todos utilizarão o mesmo sistema de containers.

---

# 76. MACHINE INVENTORIES

Máquinas:

```text id="bp98rj"
input
output
fuel
upgrade
```

usarão Inventory API.

---

# 77. TRADE

NPCs podem acessar inventários de comércio.

---

# 78. ECONOMY

O Economy Engine poderá consultar:

```text id="3m2wfh"
item quantity
```

sem acessar diretamente a implementação do inventory.

---

# 79. MOB LOOT

Drops entram pela Inventory API.

---

# 80. FARMING

Recursos agrícolas entram pela mesma API.

---

# 81. MINING

Minérios entram automaticamente em containers adequados.

---

# 82. ACCESSORY CAPACITY

A capacidade máxima de 32 acessórios deverá ser configurável.

---

# 83. PERFORMANCE

Não recalcular todos os modificadores do personagem a cada frame.

Usar atualização quando:

```text id="y6gxkw"
equip
unequip
loadout swap
unlock
```

---

# 84. INVENTORY EVENTS

Criar eventos:

```text id="z0njlr"
ITEM_ADDED
ITEM_REMOVED
ITEM_MOVED
ITEM_SPLIT
ITEM_MERGED
ITEM_EQUIPPED
ITEM_UNEQUIPPED
LOADOUT_CHANGED
SLOT_UNLOCKED
```

---

# 85. SAVE VERSIONING

Inventário deve possuir schema/versionamento.

Exemplo:

```text id="p2v1tr"
inventorySchemaVersion
equipmentSchemaVersion
```

---

# 86. MIGRATION

Mudanças futuras devem migrar:

```text id="34dsp9"
inventory v1
↓
migration
↓
inventory v2
```

---

# 87. ANTI-DUPLICATION

Testar operações concorrentes:

```text id="5ayq8i"
pickup
craft
container transfer
save
```

---

# 88. INVENTORY AUDIT

Criar ferramenta de debug:

```text id="w2d9qe"
nexora inventory inspect
```

mostrando:

* slots;
* containers;
* stacks;
* equipment;
* loadouts;
* unlocks.

---

# 89. TESTE DE MOCHILA

```text id="jpy8d7"
criar mochila
↓
adicionar item
↓
atingir capacidade
↓
tentar item incompatível
↓
trocar mochila
↓
save/load
```

---

# 90. TESTE DOS 32 ACESSÓRIOS

```text id="z97p1q"
unlock 1
unlock 2
...
unlock 32
```

Verificar:

* persistência;
* conflitos;
* stats;
* interface.

---

# 91. TESTE DOS 4 LOADOUTS

```text id="r3wm81"
loadout 1
loadout 2
loadout 3
loadout 4
```

Testar troca.

---

# 92. TESTE DE UNLOCK

```text id="08m6su"
obter consumível
↓
consumir
↓
desbloquear slot
↓
salvar
↓
reabrir
↓
continua desbloqueado
```

---

# 93. TESTE DE INTEGRAÇÃO

```text id="z8v3q1"
Mining
↓
Mining Backpack
↓
Mining Armor
↓
Accessories
↓
Mine
↓
Ore
↓
Auto-routing
↓
Backpack
```

---

# 94. TESTE DE CIVILIZAÇÃO

NPC compra/vende itens utilizando o mesmo Inventory API.

---

# 95. TESTE DE QUEST

Quest recompensa o jogador com:

```text id="vxh2e4"
item
+
unlock item
```

---

# 96. TESTE DE MOB

Mobs deixam drops no mundo.

O jogador coleta.

O sistema roteia para o container correto.

---

# 97. TESTE DE SPACE

Space loadout:

```text id="qv5ie7"
armor
accessories
backpack
equipment
```

troca rapidamente.

---

# 98. UI MOBILE/CONTROLLER

Preparar interface para:

```text id="0xk0ma"
mouse
keyboard
controller
touch
```

---

# 99. ACCESSIBILITY

Suportar:

* atalhos;
* foco;
* remapeamento;
* navegação por teclado/controller;
* opções de tamanho.

---

# 100. DEFINIÇÃO FINAL

O Inventory System do NEXORA deve ser capaz de representar:

```text id="fg3h4x"
PLAYER
│
├── INVENTORY
│
├── HOTBAR
│
├── BACKPACKS
│
├── 32 ACCESSORIES
│
├── LOADOUT 1
├── LOADOUT 2
├── LOADOUT 3
└── LOADOUT 4
```

com progressão para desbloquear capacidade.

---

# 101. ARQUITETURA

```text id="0b6f2q"
                 CHARACTER
                     │
             ┌───────┴────────┐
             │                │
        INVENTORY          EQUIPMENT
             │                │
       ┌─────┼─────┐      ┌───┴────┐
       │     │     │      │        │
     MAIN  BAGS  HOTBAR  ARMOR  ACCESSORIES
                         │
                      LOADOUTS
                         │
                     1 2 3 4
```

---

# 102. CONEXÕES

```text id="rc4b7o"
Inventory
├── Crafting
├── Mining
├── Farming
├── Mob Loot
├── Economy
├── Trading
├── Vehicles
├── Machines
├── Storage
└── Quest Rewards
```

---

# 103. REGRA FINAL

O jogador não deve ser obrigado a trocar manualmente 40 coisas toda vez que mudar de atividade.

O sistema deve permitir preparação.

Exemplo:

```text id="1x6x3v"
"Vou minerar."
↓
Mining Loadout
↓
Mining Backpack
↓
Mining Accessories
↓
Mining Armor
↓
Mining Tools
```

Depois:

```text id="y0f8u3"
"Vou para o espaço."
↓
Space Loadout
↓
Space Armor
↓
Space Backpack
↓
Space Accessories
```

A troca deve ser rápida e segura.

---

# PRINCÍPIO DE DESIGN

> **Inventory is not storage. Inventory is preparation.**

O sistema deve fazer o jogador pensar:

> “Como vou me preparar para o que vou fazer?”

e não:

> “Onde eu coloco mais 30 itens?”

---

# OBJETIVO FINAL

O Inventory System deve ser suficientemente poderoso para suportar todo o ecossistema do NEXORA:

```text id="wni8n5"
SURVIVAL
MINING
BUILDING
AGRICULTURE
MAGIC
TECHNOLOGY
AUTOMATION
EXPLORATION
WARFARE
DIMENSIONS
SPACE
```

sem precisar criar um inventário diferente para cada sistema.

Todos devem utilizar a mesma infraestrutura.
