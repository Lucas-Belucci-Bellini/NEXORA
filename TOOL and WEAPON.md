Sim. Esse precisa ser separado do **Combat System**, porque no NEXORA uma ferramenta não é necessariamente uma arma, e uma arma pode ser apenas uma das capacidades que um item oferece.

A ideia seria:

```text id="7m4q9x"
                     TOOL / WEAPON API
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
       ITEM                TOOL API             WEAPON API
         │                    │                    │
      ItemStack            Mining              Attacks
      Durability            Building            Damage
      Material              Harvest             Projectiles
      Tags                  Interaction         Abilities
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
       ┌──────────────────────┼──────────────────────┐
       │                      │                      │
    CAPABILITY             MATERIAL              MODIFIERS
       │                      │                      │
    Mining                durability             speed
    Building              hardness               damage
    Combat                efficiency              range
    Harvest               weight                  utility
    Utility
```

# NEXORA — TOOL / WEAPON API MASTER PLAN

## 1. Objetivo

Criar uma API unificada para itens que podem executar ações especiais:

```text id="q4x8m2"
ferramentas
armas
instrumentos
dispositivos
scanners
equipamentos industriais
equipamentos mágicos
equipamentos tecnológicos
```

Sem colocar lógica específica dentro do `Item` básico.

A regra:

> **Item define o objeto. Capability define o que ele consegue fazer.**

---

# 2. TOOL-0 — Item Capability

Um item pode possuir várias capacidades:

```text id="m7x3q9"
Item
 ├── MiningCapability
 ├── BuildingCapability
 ├── HarvestCapability
 ├── CombatCapability
 ├── InteractionCapability
 └── UtilityCapability
```

Uma picareta poderia ter:

```text id="x5m8q1"
Mining
```

Um equipamento complexo poderia ter:

```text id="4q7m2x"
Mining
Scanning
Building
```

---

# 3. TOOL-1 — ToolDefinition

Criar:

```text id="m8x2q5"
ToolDefinition
├── id
├── capabilities
├── materialProfile
├── durabilityProfile
├── efficiencyProfile
└── requirements
```

---

# 4. TOOL-2 — Tool Instance

Separar definição de instância.

```text id="7m4q9x"
ToolDefinition
        ↓
ToolInstance
```

Uma instância pode possuir:

```text id="x3m8q2"
durability
experience
upgrades
customName
modifiers
```

---

# 5. TOOL-3 — Capability API

Criar uma interface conceitual:

```text id="m6q1x8"
Capability
├── canUse
├── validate
├── execute
└── getPreview
```

Isso permite adicionar capacidades sem modificar Item Core.

---

# 6. TOOL-4 — Mining Capability

```text id="4x7m2q"
MiningCapability
├── miningPower
├── miningSpeed
├── area
├── allowedMaterials
└── specialRules
```

---

# 7. TOOL-5 — Mining Speed

A velocidade não deve ser uma propriedade fixa do bloco.

Pode resultar de:

```text id="m9x3q7"
Tool
+
Block
+
Material
+
Modifiers
```

---

# 8. TOOL-6 — Mining Power

Criar:

```text id="x5m8q1"
MiningPower
```

para determinar se uma ferramenta consegue interagir com determinado material.

---

# 9. TOOL-7 — Tool Material

Materiais podem possuir:

```text id="4m8q2x"
ToolMaterial
├── durability
├── miningPower
├── miningSpeed
├── repairability
└── modifiers
```

Exemplos seriam registrados por conteúdo, não pelo Core.

---

# 10. TOOL-8 — Tool Tiers

Em vez de hardcode:

```text id="7x2m9q"
wood
stone
iron
...
```

ter:

```text id="Tier"
```

com requisitos e propriedades.

Isso deixa mods livres para criar novos níveis.

---

# 11. TOOL-9 — Material Compatibility

Uma ferramenta pode declarar:

```text id="m5q8x1"
compatible material tags
```

O bloco fornece:

```text id="x4m7q2"
required tool tags
```

---

# 12. TOOL-10 — Durability

Criar:

```text id="9m3x7q"
Durability
├── current
├── maximum
└── degradationRules
```

---

# 13. TOOL-11 — Durability Events

```text id="4q8m1x"
ToolUsed
 ↓
DurabilityDamage
```

e:

```text id="m7x2q5"
ToolBroken
```

---

# 14. TOOL-12 — Tool Wear

Diferentes ações podem desgastar diferente:

```text id="x5m9q2"
mining
combat
building
harvesting
```

---

# 15. TOOL-13 — Repair

Criar:

```text id="m8x3q7"
RepairProfile
```

com:

```text id="4x7m2q"
repairMaterial
repairCost
repairRate
```

---

# 16. TOOL-14 — Repair API

```text id="9m2x8q"
RepairService
```

Outros sistemas podem fornecer o método de reparo.

---

# 17. TOOL-15 — Upgrade System

Ferramentas podem receber:

```text id="x4m7q1"
Upgrade
├── id
├── level
├── modifiers
└── requirements
```

---

# 18. TOOL-16 — Modifier System

Criar modificadores genéricos:

```text id="m5q8x2"
Speed
Power
Durability
Efficiency
Range
Capacity
Accuracy
Cooldown
```

---

# 19. TOOL-17 — Modifier Stacking

Definir regras:

```text id="7x3m9q"
additive
multiplicative
capped
exclusive
```

para evitar combinações absurdas.

---

# 20. TOOL-18 — Tool Action

Criar:

```text id="4m8x1q"
ToolAction
├── input
├── target
├── context
├── cost
└── result
```

---

# 21. TOOL-19 — Target Context

Uma ferramenta precisa saber o alvo:

```text id="m7q2x5"
Block
Entity
Fluid
Structure
Empty Space
```

---

# 22. TOOL-20 — Interaction API

Ferramentas podem implementar:

```text id="x9m3q7"
Use
AltUse
Charge
Release
SneakUse
```

---

# 23. TOOL-21 — Charge Actions

Suporte a ações carregáveis:

```text id="4q8m2x"
press
 ↓
charge
 ↓
release
```

---

# 24. TOOL-22 — Cooldown

Usar o sistema de cooldown do Engine:

```text id="m5x7q1"
Tool
 ↓
Cooldown
```

---

# 25. TOOL-23 — Resource Cost

A ferramenta pode consumir:

```text id="8x2m9q"
durability
energy
fuel
fluid
magic
ammunition-like resource
```

Usando interfaces específicas.

---

# 26. TOOL-24 — Area Tools

Ferramentas podem afetar regiões:

```text id="m4q8x2"
1 block
3x3
line
cone
volume
```

O Build/Destruction Engine continua responsável pelas mudanças reais no mundo.

---

# 27. TOOL-25 — Area Mining

Fluxo:

```text id="x7m3q9"
Tool
 ↓
Mining Capability
 ↓
Target Volume
 ↓
Build/Destruction Engine
```

---

# 28. TOOL-26 — Building Tools

Criar:

```text id="m8x1q5"
BuildingCapability
```

pode fornecer:

```text id="4q7m2x"
placement assistance
rotation
blueprint placement
bulk placement
```

---

# 29. TOOL-27 — Construction Tools

Ferramentas especiais podem ajudar em:

```text id="x5m9q2"
repair
copy
paste
fill
scaffold
```

---

# 30. TOOL-28 — Building Preview

```text id="m7q3x8"
Tool
 ↓
Build Preview API
 ↓
Renderer
```

---

# 31. TOOL-29 — Harvest Capability

Criar:

```text id="4x8m1q"
HarvestCapability
```

para:

```text id="m9q2x7"
plants
crops
resources
```

---

# 32. TOOL-30 — Harvest Modifiers

Ferramentas podem modificar:

```text id="x5m8q1"
harvest speed
yield
quality
preservation
```

---

# 33. TOOL-31 — Combat Capability

Armas usam a mesma infraestrutura de item:

```text id="m4q7x2"
CombatCapability
```

O Combat System resolve o resultado.

---

# 34. TOOL-32 — Weapon Definition

```text id="7m3x9q"
WeaponDefinition
├── attacks
├── range
├── cooldown
├── resourceCost
└── combatProfile
```

---

# 35. TOOL-33 — Attack Profiles

Uma arma pode registrar múltiplas ações:

```text id="x8m2q5"
Primary
Secondary
Charged
Special
```

---

# 36. TOOL-34 — Melee Weapon API

```text id="m5q8x1"
MeleeWeapon
├── reach
├── arc
├── timing
├── staminaCost
└── combatAction
```

---

# 37. TOOL-35 — Ranged Weapon API

```text id="4x7m2q"
RangedWeapon
├── projectile
├── launchVelocity
├── accuracy
├── range
└── resource
```

---

# 38. TOOL-36 — Projectile Provider

A arma não implementa física do projétil.

Ela fornece:

```text id="m9x3q7"
ProjectileDefinition
```

e:

```text id="x5m8q1"
Physics
```

controla a trajetória.

---

# 39. TOOL-37 — Energy Weapon

Suporte:

```text id="4m8q2x"
EnergyWeapon
```

que utiliza:

```text id="m7x3q9"
Energy Resource
```

---

# 40. TOOL-38 — Magic Equipment

Uma ferramenta/equipamento mágico pode fornecer:

```text id="x2m8q5"
MagicCapability
```

sem modificar Combat Core.

---

# 41. TOOL-39 — Technology Equipment

Mesma coisa:

```text id="m5q7x2"
TechnologyCapability
```

---

# 42. TOOL-40 — Scanner

Não tudo precisa ser combate.

Criar:

```text id="9x3m7q"
ScannerCapability
```

que pode consultar:

```text id="4m8q1x"
ore
biome
mob
fluid
structure
environment
```

---

# 43. TOOL-41 — Sensor Tools

Suporte:

```text id="m7x2q9"
environment scanner
resource scanner
entity scanner
```

---

# 44. TOOL-42 — Utility Tools

Exemplos conceituais:

```text id="x5m8q2"
wrench
multitool
repair device
measuring device
```

---

# 45. TOOL-43 — Machine Interaction

Uma ferramenta pode operar máquinas:

```text id="4q7m1x"
Tool
 ↓
Machine Interaction
```

---

# 46. TOOL-44 — Machine Configuration

Ferramentas podem:

```text id="m9x3q7"
rotate component
configure machine
connect pipe
connect energy
inspect network
```

---

# 47. TOOL-45 — Network Interaction

Ferramentas podem interagir com:

```text id="x4m8q1"
Energy Network
Fluid Network
Automation Network
```

---

# 48. TOOL-46 — Context Actions

O mesmo item pode fazer coisas diferentes:

```text id="m5q7x2"
block → action A
machine → action B
entity → action C
```

usando `InteractionContext`.

---

# 49. TOOL-47 — Tool Modes

Uma ferramenta pode possuir:

```text id="7x2m9q"
mode
```

Exemplo:

```text id="4m8x1q"
Mining
Building
Repair
Scan
```

---

# 50. TOOL-48 — Mode Switching

```text id="m7x3q9"
changeMode()
```

com:

```text id="x5m2q8"
input
UI
wheel
```

---

# 51. TOOL-49 — Tool UI

HUD pode mostrar:

```text id="4q8m2x"
current mode
durability
resource
selected action
```

---

# 52. TOOL-50 — Tool Preview

Antes da ação:

```text id="m9x3q7"
Tool
 ↓
Preview
 ↓
Renderer
```

---

# 53. TOOL-51 — Tool Sounds

Tool pode fornecer:

```text id="x5m8q1"
use sound
hit sound
break sound
reload-like sound
```

mas Audio System executa.

---

# 54. TOOL-52 — Animation Hooks

Tool fornece:

```text id="m7q2x5"
animation action
```

e Animation System interpreta.

---

# 55. TOOL-53 — Visual Attachments

Ferramentas podem possuir:

```text id="4x8m1q"
attachments
modules
cosmetics
```

---

# 56. TOOL-54 — Modular Weapons

Uma arma pode possuir componentes:

```text id="m9x3q7"
Core
Module
Module
Module
```

Cada módulo fornece modificadores/capacidades.

---

# 57. TOOL-55 — Modular Tools

Mesma arquitetura para ferramentas:

```text id="x5m8q2"
Handle
Head
Module
Upgrade
```

---

# 58. TOOL-56 — Tinkers-like Architecture

Sem copiar implementação de nenhum mod, podemos ter:

```text id="4q7m1x"
Tool Parts
 ↓
Tool Assembly
 ↓
Tool Definition
```

---

# 59. TOOL-57 — Part Materials

Peças podem possuir:

```text id="m7x2q9"
durability
speed
power
modifier
```

---

# 60. TOOL-58 — Part Compatibility

Cada ferramenta define:

```text id="x5m8q1"
required part slots
```

---

# 61. TOOL-59 — Tool Assembly

```text id="4m8q2x"
Parts
 ↓
Validation
 ↓
Assembly
 ↓
Generated Tool
```

---

# 62. TOOL-60 — Generated Tool Identity

Uma ferramenta montada precisa continuar sendo uma instância:

```text id="m9x3q7"
ToolInstance
├── baseDefinition
├── parts
├── modifiers
└── durability
```

---

# 63. TOOL-61 — Repairability

O material pode determinar:

```text id="x7m2q5"
repairType
repairCost
repairLimit
```

---

# 64. TOOL-62 — Tool Weight

Peso pode influenciar outros sistemas:

```text id="4q8m1x"
Weight
```

Player/Stamina pode consumir essa informação.

---

# 65. TOOL-63 — Tool Ergonomics

Criar um modificador abstrato:

```text id="m5x8q2"
handling
```

que pode afetar:

```text id="7m3q9x"
action speed
stamina cost
```

---

# 66. TOOL-64 — Range

Uma ferramenta pode definir:

```text id="x4m7q1"
interaction range
```

mas não deve simplesmente ignorar regras do Player/Combat.

---

# 67. TOOL-65 — Reach Modifier

Player fornece alcance base.

Equipment pode modificar dentro das regras permitidas.

---

# 68. TOOL-66 — Tool Permissions

Algumas capacidades exigem:

```text id="m8q2x5"
skill level
technology unlock
permission
dimension rule
```

---

# 69. TOOL-67 — Skill Integration

```text id="4x7m1q"
Tool
+
Mining Skill
```

pode produzir modificadores.

---

# 70. TOOL-68 — Progression Integration

Equipamentos podem exigir:

```text id="m9x3q7"
technology tier
skill
knowledge
unlock
```

---

# 71. TOOL-69 — Knowledge Integration

O jogador pode descobrir como usar uma ferramenta:

```text id="x5m8q2"
discovery
 ↓
capability unlocked
```

---

# 72. TOOL-70 — Recipe Integration

Ferramentas possuem receitas através da Recipe API:

```text id="m7q3x9"
Recipe
 ↓
Tool
```

---

# 73. TOOL-71 — Inventory Integration

Tool é um `ItemStack` especial:

```text id="4x8m1q"
ItemStack
 ↓
Tool Instance Data
```

---

# 74. TOOL-72 — Equipment Integration

Ferramentas seguradas:

```text id="m5x2q7"
main hand
off hand
```

Equipamentos especializados:

```text id="x8m3q1"
armor/accessory
```

podem fornecer capabilities também.

---

# 75. TOOL-73 — Accessory Capabilities

Um acessório pode fornecer:

```text id="4q7m9x"
MiningCapability
ScannerCapability
CombatCapability
```

sem ser uma ferramenta tradicional.

---

# 76. TOOL-74 — Durability Serialization

Salvar:

```text id="m8x2q5"
current durability
upgrades
modules
custom data
```

---

# 77. TOOL-75 — Versioning

```text id="7m3q9x"
ToolDataVersion
```

para migração.

---

# 78. TOOL-76 — Multiplayer

Cliente solicita:

```text id="x4m8q1"
Tool Action
```

Servidor valida:

```text id="m5q7x2"
tool exists
capability exists
cooldown
resource
target
range
permissions
```

---

# 79. TOOL-77 — Server Authority

O cliente nunca deve simplesmente declarar:

```text id="9x3m7q"
"meu machado destruiu esses 500 blocos"
```

O servidor valida a operação.

---

# 80. TOOL-78 — Action Transaction

```text id="4m8x2q"
Tool Use
 ↓
Validate
 ↓
Resource Reservation
 ↓
Execute
 ↓
Durability
 ↓
Commit
```

---

# 81. TOOL-79 — Failure Handling

Se falhar:

```text id="m7x3q9"
no action
```

e recursos não devem desaparecer indevidamente.

---

# 82. TOOL-80 — Action Events

```text id="x5m8q1"
ToolUsed
ToolActionStarted
ToolActionCompleted
ToolActionFailed
ToolBroken
ToolModeChanged
```

---

# 83. TOOL-81 — Combat Integration

```text id="4q7m2x"
Weapon
 ↓
CombatCapability
 ↓
Combat System
```

---

# 84. TOOL-82 — Build Integration

```text id="m9x3q7"
Construction Tool
 ↓
Build System
```

---

# 85. TOOL-83 — Destruction Integration

```text id="x5m8q1"
Mining Tool
 ↓
Destruction System
```

---

# 86. TOOL-84 — Physics Integration

Tool can request:

```text id="4m8q2x"
raycast
sweep
target query
```

Physics executes.

---

# 87. TOOL-85 — Renderer Integration

Renderer recebe:

```text id="m7q3x9"
ToolRenderState
```

---

# 88. TOOL-86 — Resource Integration

Ferramentas podem consumir:

```text id="x4m8q1"
fuel
energy
fluid
```

usando as APIs apropriadas.

---

# 89. TOOL-87 — Mod API

Mods podem registrar:

```text id="m5q7x2"
ToolDefinition
ToolMaterial
ToolCapability
WeaponDefinition
AttackProfile
Upgrade
Modifier
RepairProfile
ToolMode
```

---

# 90. TOOL-88 — Capability Composition

Uma modificação importante:

```text id="4q8m1x"
Tool
├── Mining
├── Building
├── Scanning
└── Repair
```

Isso permite criar ferramentas realmente multifuncionais.

---

# 91. TOOL-89 — Capability Conflicts

Algumas capacidades podem ser incompatíveis.

Criar:

```text id="m7x3q9"
CapabilityConflict
```

---

# 92. TOOL-90 — Compatibility Rules

```text id="x5m8q2"
Capability A
+
Capability B
=
allowed / restricted / modified
```

---

# 93. TOOL-91 — Tool Categories

Tags:

```text id="4m8x1q"
#tool
#mining
#building
#harvesting
#combat
#scanner
#engineering
#magic
```

---

# 94. TOOL-92 — Weapon Categories

Tags:

```text id="m9x3q7"
#weapon
#melee
#ranged
#energy
#magic
```

O Combat System usa tags em vez de nomes hardcoded.

---

# 95. TOOL-93 — Data-Driven Definitions

Uma ferramenta deve poder ser definida por dados:

```text id="x5m8q1"
definition
 ↓
validation
 ↓
runtime
```

sem recompilar o Core.

---

# 96. TOOL-94 — Asset Integration

Tool definition pode apontar para:

```text id="4q7m2x"
model
texture
animation
sound
effects
```

---

# 97. TOOL-95 — Scripting Hooks

No futuro, capabilities podem ter scripts/controladores restritos.

```text id="m8x2q5"
onUse
onHit
onBreak
onRepair
```

mas com sandbox.

---

# 98. TOOL-96 — Debug

Comandos:

```text id="7m3q9x"
nexora tool inspect
nexora tool capabilities
nexora tool test
nexora tool durability
nexora weapon inspect
```

---

# 99. TOOL-97 — Balance Testing

Uma ferramenta precisa poder ser simulada:

```text id="x4m8q1"
10,000 mining operations
 ↓
durability
speed
resource cost
```

Armas podem usar o Combat Simulator que planejamos.

---

# 100. TOOL-98 — Stress Test

```text id="m5q7x2"
1,000 tools
10,000 tool actions
large area mining
large construction operation
```

---

# 101. TOOL-99 — Final Architecture

```text id="4q8m2x"
                            ITEM
                              │
                       CAPABILITY SYSTEM
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
        TOOLS              WEAPONS             DEVICES
          │                   │                   │
       Mining              Combat              Scan
       Building            Attack              Repair
       Harvest             Defense             Utility
       Interaction         Ability              Machine
          │                   │                   │
          └───────────────────┼───────────────────┘
                              │
                    MODIFIER / MATERIAL
                              │
              ┌───────────────┼───────────────┐
              │               │               │
          Durability        Power           Speed
          Efficiency        Range           Capacity
              │               │               │
              └───────────────┼───────────────┘
                              │
       ┌──────────────────────┼──────────────────────┐
       │                      │                      │
     PLAYER                COMBAT                 BUILD
       │                      │                      │
     Input                 Damage                Placement
     Skills                Targets               Mining
     Equipment             Defense               Repair
       │                      │                      │
       └──────────────────────┼──────────────────────┘
                              │
                   PHYSICS / INVENTORY /
                   ENERGY / FLUID / RENDER
```

# 102. Ordem de implementação

Eu faria:

```text id="m7x3q9"
TOOL-0 Capability System
TOOL-1 Tool Definition
TOOL-2 Tool Instance
TOOL-3 Material
TOOL-4 Durability
TOOL-5 Mining Capability
TOOL-6 Mining Power
TOOL-7 Mining Speed
TOOL-8 Harvest Capability
TOOL-9 Building Capability
TOOL-10 Interaction Capability
TOOL-11 Tool Actions
TOOL-12 Modes
TOOL-13 Modifiers
TOOL-14 Repairs
TOOL-15 Upgrades
TOOL-16 Combat Capability
TOOL-17 Weapon Definition
TOOL-18 Melee
TOOL-19 Ranged
TOOL-20 Projectiles
TOOL-21 Energy
TOOL-22 Magic
TOOL-23 Scanner
TOOL-24 Engineering
TOOL-25 Modular Tools
TOOL-26 Part System
TOOL-27 Compatibility
TOOL-28 Player Integration
TOOL-29 Inventory Integration
TOOL-30 Physics Integration
TOOL-31 Build Integration
TOOL-32 Combat Integration
TOOL-33 Energy/Fluid Integration
TOOL-34 Renderer Integration
TOOL-35 Multiplayer
TOOL-36 Persistence
TOOL-37 Debug
TOOL-38 Balance Simulator
TOOL-39 Mod API
TOOL-40 Stress Testing
```

# 103. Primeiro Vertical Slice

O primeiro teste:

```text id="x5m8q1"
Tool Item
 ↓
Capability
 ↓
Player uses tool
 ↓
Target Query
 ↓
Build/Destruction
 ↓
Block changes
 ↓
Durability decreases
 ↓
Drop produced
 ↓
Inventory updated
 ↓
Renderer updated
 ↓
Save
```

Depois uma arma:

```text id="4m8q2x"
Weapon
 ↓
Combat Capability
 ↓
Attack
 ↓
Physics Query
 ↓
Target
 ↓
Combat System
 ↓
Damage
 ↓
Health
 ↓
Loot / Progression
```

E o teste mais interessante para o NEXORA:

```text id="m7q3x9"
MULTITOOL
├── Mining
├── Building
├── Repair
└── Scanner
       │
       ↓
    MODE SWITCH
       │
 ┌─────┼─────┬─────┐
 ↓     ↓     ↓     ↓
Mine Build Repair Scan
```

## Regra arquitetural final

> **O Item é o objeto. A Tool/Weapon API fornece capacidades. Os sistemas especializados executam o resultado.**

Então:

```text id="x4m8q1"
Tool
 ↓
MiningCapability
 ↓
Destruction Engine

Weapon
 ↓
CombatCapability
 ↓
Combat System

Scanner
 ↓
ScanningCapability
 ↓
World / Knowledge

Repair Tool
 ↓
RepairCapability
 ↓
Build / Machine / Vehicle
```

Isso deixa o NEXORA livre para ter desde **uma ferramenta simples de coleta até equipamentos modulares, armas, scanners, ferramentas industriais, equipamentos mágicos e dispositivos espaciais**, todos usando a mesma base de API sem transformar o `Item` ou o `Player` em um monstro.
