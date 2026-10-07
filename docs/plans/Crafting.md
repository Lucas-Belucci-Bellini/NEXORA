Sim. **Crafting** também merece um sistema próprio, e no NEXORA ele deve ser muito mais amplo que uma simples “grade 3×3”.

A arquitetura ideal seria:

```text
                     CRAFTING SYSTEM
                            │
      ┌─────────────────────┼─────────────────────┐
      │                     │                     │
   RECIPES              MATERIALS              STATIONS
      │                     │                     │
   Shaped                Tags                 Workbench
   Shapeless              Traits              Furnace-like
   Machines               Quality              Forge
   Multiblock             Purity               Chemistry
   Processing                                  Assembly
      │                     │                     │
      └─────────────────────┼─────────────────────┘
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
       CRAFTING          PROCESSING       ASSEMBLY
          │                 │                 │
       Manual            Smelting          Machines
       Player            Refining          Multiblocks
       NPC               Chemistry        Automation
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
                  INVENTORY / ECONOMY /
                  KNOWLEDGE / PROGRESSION
```

# NEXORA — CRAFTING MASTER PLAN

## 1. Objetivo

O Crafting System deve controlar:

```text
receitas
materiais
estações
processamento
montagem
desmontagem
qualidade
subprodutos
consumo de recursos
ferramentas necessárias
energia
fluidos
tempo
temperatura
pressão
conhecimento
progressão
```

Mas a regra mais importante é:

> **Crafting decide como algo é produzido; Inventory fornece os materiais; Machines fornecem os processos industriais; Progression/Knowledge determina o que o personagem sabe fazer.**

---

# 2. CRAFT-0 — Crafting Core

Criar:

```text
CraftingSystem
Recipe
RecipeContext
CraftingOperation
CraftingResult
CraftingStation
```

Fluxo:

```text
Recipe Request
 ↓
Validate
 ↓
Check Inputs
 ↓
Check Requirements
 ↓
Reserve Resources
 ↓
Process
 ↓
Produce Outputs
 ↓
Consume Inputs
 ↓
Commit
```

---

# 3. CRAFT-1 — Recipe Definition

Uma receita:

```text
Recipe
├── id
├── type
├── inputs
├── outputs
├── requirements
├── time
├── station
├── conditions
└── byproducts
```

---

# 4. CRAFT-2 — Recipe Types

Suportar:

```text
SHAPED
SHAPELESS
PROCESSING
ASSEMBLY
DISASSEMBLY
SMELTING
ALLOY
CHEMICAL
MULTIBLOCK
RESEARCH
CUSTOM
```

---

# 5. CRAFT-3 — Shaped Recipes

Formato espacial:

```text
A B
C D
```

A posição importa.

---

# 6. CRAFT-4 — Shapeless Recipes

A posição não importa:

```text
A + B + C
```

---

# 7. CRAFT-5 — Recipe Ingredients

Não exigir item exato em todas as receitas.

Uma entrada pode ser:

```text
Ingredient
├── item
├── tag
├── alternatives
├── amount
└── conditions
```

Exemplo conceitual:

```text
#metal
```

pode aceitar vários materiais compatíveis.

---

# 8. CRAFT-6 — Item Tags

Criar tags:

```text
#wood
#stone
#metal
#glass
#fiber
#food
#ore
```

Mods podem adicionar tags.

---

# 9. CRAFT-7 — Material Tags

Separar item de material.

Um item pode conter:

```text
material = copper-like
```

Isso permite receitas genéricas.

---

# 10. CRAFT-8 — Ingredient Alternatives

Uma receita pode aceitar:

```text
copper-like
OU
bronze-like
OU
advanced conductive metal
```

dependendo da regra.

---

# 11. CRAFT-9 — Quantity

Cada ingrediente possui:

```text
amount
```

E pode possuir:

```text
minimum
maximum
exact
```

---

# 12. CRAFT-10 — Output

Produto:

```text
CraftResult
├── item
├── amount
└── data
```

`data` pode preservar:

```text
durability
modules
quality
customization
```

para ferramentas e equipamentos.

---

# 13. CRAFT-11 — Byproducts

Alguns processos geram:

```text
input
 ↓
main product
+
byproduct
```

Exemplo conceitual:

```text
ore
 ↓
metal
+
slag-like residue
```

---

# 14. CRAFT-12 — Waste

Processos podem produzir:

```text
waste
```

e outros sistemas podem reutilizá-los.

---

# 15. CRAFT-13 — Processing

Diferenciar:

```text
crafting
```

de:

```text
processing
```

Exemplo:

```text
ore
 ↓
smelting
 ↓
metal
```

---

# 16. CRAFT-14 — Processing Recipe

```text
ProcessingRecipe
├── input
├── output
├── byproducts
├── time
├── temperature
├── energy
└── fluid
```

---

# 17. CRAFT-15 — Crafting Time

Receitas podem levar:

```text
instant
short
medium
long
```

ou tempo numérico.

---

# 18. CRAFT-16 — Station

Uma receita pode exigir:

```text
CraftingStation
```

Exemplos:

```text
basic_workbench
forge
assembler
laboratory
magic_station
```

---

# 19. CRAFT-17 — Station Interface

Criar:

```text
CraftingStation
├── inputs
├── outputs
├── capabilities
└── rules
```

---

# 20. CRAFT-18 — Station Capabilities

Uma estação pode oferecer:

```text
SMELTING
ASSEMBLY
CHEMISTRY
CUTTING
GRINDING
PRESSING
ALLOYING
```

---

# 21. CRAFT-19 — Machine Crafting

Máquinas usam:

```text
MachineRecipe
```

mas continuam sob o Machine API.

```text
Machine
 ↓
Crafting Recipe
```

---

# 22. CRAFT-20 — Energy Requirement

Receita pode exigir:

```text
energy
powerRate
```

O Energy System fornece o recurso.

---

# 23. CRAFT-21 — Fluid Requirement

Receita pode exigir:

```text
fluid
amount
pressure
temperature
```

Fluid Engine fornece.

---

# 24. CRAFT-22 — Temperature Requirement

Processo pode exigir:

```text
minimumTemperature
maximumTemperature
```

---

# 25. CRAFT-23 — Pressure Requirement

Processo pode exigir:

```text
minimumPressure
```

---

# 26. CRAFT-24 — Environment Requirement

Alguns processos podem exigir:

```text
biome
dimension
altitude
environment
```

---

# 27. CRAFT-25 — Player Requirements

Receita pode exigir:

```text
skill
knowledge
progression
technology
```

---

# 28. CRAFT-26 — Knowledge Unlock

O jogador pode conhecer uma receita porque:

```text
observed
discovered
researched
taught
purchased
```

---

# 29. CRAFT-27 — Recipe Discovery

Uma receita pode começar:

```text
UNKNOWN
```

e virar:

```text
DISCOVERED
```

---

# 30. CRAFT-28 — Research

Receitas avançadas podem exigir:

```text
Research
 ↓
Discovery
 ↓
Recipe Unlock
```

Isso conecta ao Knowledge System.

---

# 31. CRAFT-29 — Recipe Book

Criar:

```text
RecipeBook
```

que mostra:

```text
known recipes
available recipes
unknown recipes
```

---

# 32. CRAFT-30 — Search

Pesquisar por:

```text
item
category
station
ingredient
tag
```

---

# 33. CRAFT-31 — Filtering

Filtros:

```text
craftable
known
station
category
favorites
recent
```

---

# 34. CRAFT-32 — Craftable Check

Uma receita deve responder:

```text
canCraft(context)
```

considerando:

```text
inventory
station
requirements
knowledge
progression
environment
```

---

# 35. CRAFT-33 — Ingredient Reservation

Antes de iniciar uma operação:

```text
Inventory
 ↓
reserve
 ↓
Crafting
```

Isso evita conflitos.

---

# 36. CRAFT-34 — Transaction

A operação deve ser transacional:

```text
BEGIN
 ↓
RESERVE
 ↓
PROCESS
 ↓
OUTPUT
 ↓
COMMIT
```

---

# 37. CRAFT-35 — Failure Recovery

Se falhar:

```text
ROLLBACK
```

sem perder materiais indevidamente.

---

# 38. CRAFT-36 — Batch Crafting

Permitir:

```text
craft x1
craft x10
craft x100
```

---

# 39. CRAFT-37 — Batch Optimization

Não executar 100 operações independentes quando podem ser agrupadas.

---

# 40. CRAFT-38 — Queue

Estações podem possuir:

```text
CraftQueue
```

Exemplo:

```text
Recipe A
Recipe B
Recipe C
```

---

# 41. CRAFT-39 — Queue Priority

Permitir:

```text
priority
pause
resume
cancel
reorder
```

---

# 42. CRAFT-40 — Machine Automation

Máquinas podem pegar receitas automaticamente:

```text
input
 ↓
recipe selection
 ↓
processing
 ↓
output
```

---

# 43. CRAFT-41 — Automation Rules

Máquina pode declarar:

```text
autoProcess
autoOutput
autoInput
```

---

# 44. CRAFT-42 — Recipe Conditions

Criar:

```text
RecipeCondition
```

Possibilidades:

```text
hasSkill
hasKnowledge
dimension
biome
temperature
time
season
equipment
machineTier
energy
fluid
```

---

# 45. CRAFT-43 — Conditional Recipes

Uma mesma receita pode ter:

```text
base version
advanced version
special version
```

sem duplicar completamente o sistema.

---

# 46. CRAFT-44 — Recipe Priority

Quando várias receitas poderiam aceitar a mesma entrada:

```text
priority
specificity
station
```

determinam qual é selecionada.

---

# 47. CRAFT-45 — Recipe Conflicts

Detectar:

```text
two recipes
same input pattern
```

e gerar diagnóstico.

---

# 48. CRAFT-46 — Recipe Validation

Ao carregar o jogo/mod:

```text
missing item
missing station
invalid ingredient
invalid output
circular dependency
```

deve ser detectado antes do gameplay.

---

# 49. CRAFT-47 — Recipe Registry

```text
RecipeRegistry
```

similar aos outros registries do NEXORA.

---

# 50. CRAFT-48 — Dynamic Recipe Registration

Mods podem registrar receitas:

```text
registerRecipe(...)
```

sem alterar o Core.

---

# 51. CRAFT-49 — Recipe Removal

Mods podem remover/desabilitar receitas através da API permitida.

---

# 52. CRAFT-50 — Recipe Modification

Permitir alterar:

```text
inputs
outputs
time
requirements
```

sem editar o arquivo original.

---

# 53. CRAFT-51 — Recipe Layers

Receitas podem vir de:

```text
base game
official modules
mod
server
world
```

com prioridade controlada.

---

# 54. CRAFT-52 — Recipe Namespace

IDs:

```text
nexora:...
modname:...
```

---

# 55. CRAFT-53 — Recipe Versioning

```text
RecipeVersion
```

para saves antigos.

---

# 56. CRAFT-54 — Recipe Migration

Se receita muda:

```text
old recipe
 ↓
migration rules
 ↓
new recipe
```

---

# 57. CRAFT-55 — Custom Data

Receitas podem gerar objetos com estado:

```text
Tool
Weapon
Machine
Armor
Vehicle component
```

---

# 58. CRAFT-56 — Quality System

Alguns processos podem ter:

```text
quality
```

resultado:

```text
LOW
NORMAL
HIGH
MASTERWORK
```

Isso deve ser opcional.

---

# 59. CRAFT-57 — Quality Factors

Qualidade pode depender de:

```text
materials
station
skill
temperature
precision
process
```

---

# 60. CRAFT-58 — Material Purity

Materiais podem possuir:

```text
purity
```

que influencia processos especializados.

---

# 61. CRAFT-59 — Material Composition

Itens complexos podem conter:

```text
MaterialComposition
```

exemplo:

```text
alloy
├── metal A
└── metal B
```

---

# 62. CRAFT-60 — Alloy System

Criar:

```text
AlloyRecipe
```

para ligas.

---

# 63. CRAFT-61 — Material Transformation

```text
Ore
 ↓
Refined Material
 ↓
Alloy
 ↓
Component
 ↓
Product
```

---

# 64. CRAFT-62 — Assembly

Montagem:

```text
Component A
+
Component B
+
Component C
 ↓
Machine
```

---

# 65. CRAFT-63 — Disassembly

Também:

```text
Product
 ↓
Disassembly
 ↓
Components
```

com regras de recuperação.

---

# 66. CRAFT-64 — Salvaging

Itens danificados podem ser desmontados:

```text
Damaged Item
 ↓
Salvage
 ↓
Recovered Materials
```

---

# 67. CRAFT-65 — Repair Recipes

Reparo pode ser modelado como crafting especializado:

```text
damaged item
+
material
 ↓
repaired item
```

---

# 68. CRAFT-66 — Tool Crafting

Integra com Tool API:

```text
Recipe
 ↓
ToolDefinition / ToolInstance
```

---

# 69. CRAFT-67 — Weapon Crafting

Mesmo:

```text
Recipe
 ↓
Weapon
```

---

# 70. CRAFT-68 — Modular Equipment

Receita pode montar:

```text
Core
+
Module
+
Module
```

produzindo equipamento customizado.

---

# 71. CRAFT-69 — Tinkers-like Assembly

O NEXORA pode possuir um sistema próprio:

```text
Part
 ↓
Material
 ↓
Assembly
 ↓
Tool
```

sem depender de receitas fixas para cada combinação.

---

# 72. CRAFT-70 — Component Recipes

Componentes:

```text
gear
plate
rod
circuit
pipe
```

podem ser reutilizados por várias receitas.

---

# 73. CRAFT-71 — Machines as Crafting Nodes

Máquinas entram na cadeia:

```text
Raw Resource
 ↓
Processor
 ↓
Component
 ↓
Assembler
 ↓
Machine
```

---

# 74. CRAFT-72 — Production Chains

Criar sistema para representar:

```text
A
 ↓
B
 ↓
C
 ↓
D
```

---

# 75. CRAFT-73 — Dependency Graph

Cada receita pode formar grafo:

```text
Ore
 ├──> Metal
 │      └──> Plate
 │              └──> Machine
 └──> Byproduct
```

---

# 76. CRAFT-74 — Recipe Planner

Ferramenta:

```text
Need Machine
 ↓
calculate dependencies
 ↓
materials required
```

---

# 77. CRAFT-75 — Resource Planner

Mostrar:

```text
final output
 ↓
total raw resources
```

---

# 78. CRAFT-76 — Production Planner

Para industrialização:

```text
target = 100 units/hour
```

calcular:

```text
machines
resources
energy
fluids
```

---

# 79. CRAFT-77 — Economy Integration

Crafting consome e produz recursos.

```text
Resource
 ↓
Crafting
 ↓
Product
 ↓
Market
```

---

# 80. CRAFT-78 — NPC Crafting

NPCs também devem usar Crafting.

```text
NPC
 ↓
Knowledge
 ↓
Recipe
 ↓
Station
 ↓
Craft
```

---

# 81. CRAFT-79 — NPC Skills

Qualidade/velocidade podem depender da habilidade do NPC.

---

# 82. CRAFT-80 — Civilization Production

Civilizações podem ter:

```text
workshops
factories
smelters
laboratories
```

todos utilizando Crafting.

---

# 83. CRAFT-81 — Industrial Chains

Exemplo:

```text
Mine
 ↓
Ore
 ↓
Smelter
 ↓
Metal
 ↓
Forge
 ↓
Component
 ↓
Assembler
 ↓
Machine
```

---

# 84. CRAFT-82 — Energy Integration

Indústrias podem consumir energia:

```text
Craft
 ↓
Energy Request
```

---

# 85. CRAFT-83 — Fluid Integration

Indústrias podem consumir fluidos:

```text
Craft
 ↓
Fluid Request
```

---

# 86. CRAFT-84 — Automation Integration

Redes automatizadas podem:

```text
request recipe
provide ingredients
collect outputs
```

---

# 87. CRAFT-85 — Storage Integration

Crafting pode retirar materiais de:

```text
player inventory
machine inventory
storage network
warehouse
```

conforme permissões.

---

# 88. CRAFT-86 — Logistics Integration

Ingredientes podem viajar por:

```text
pipes
conveyors
vehicles
rail
```

antes de chegar à estação.

---

# 89. CRAFT-87 — Long Production Chains

Uma cadeia pode durar:

```text
seconds
minutes
hours
```

e continuar enquanto o jogador está longe, respeitando Simulation LOD.

---

# 90. CRAFT-88 — Offline / Distant Production

Uma cidade pode continuar produzindo:

```text
steel
food
tools
```

em simulação regional.

---

# 91. CRAFT-89 — Crafting LOD

Perto:

```text
individual machines
```

Longe:

```text
production statistics
```

---

# 92. CRAFT-90 — Production Simulation

Para uma fábrica distante:

```text
inputs
production rate
outputs
```

podem ser simulados agregadamente.

---

# 93. CRAFT-91 — Crafting Time Simulation

A produção pode continuar:

```text
start
 ↓
save
 ↓
load
 ↓
elapsed time
 ↓
resume/complete
```

---

# 94. CRAFT-92 — Offline Progress

Recomendo que isso seja limitado às máquinas/sistemas persistentes aprovados.

Não deixar qualquer recipe executar infinitamente em background.

---

# 95. CRAFT-93 — Recipe UI

Tela:

```text
inputs
 ↓
recipe
 ↓
outputs
```

com:

```text
requirements
time
station
```

---

# 96. CRAFT-94 — Crafting Preview

Mostrar:

```text
green = available
red = missing
```

e todos os requisitos.

---

# 97. CRAFT-95 — Missing Materials

Exibir cadeia:

```text
Need Machine
→ missing 4 plates
→ missing 12 metal
→ missing 24 ore
```

---

# 98. CRAFT-96 — Crafting History

Registrar:

```text
recipe
quantity
actor
time
station
result
```

para estatísticas/debug.

---

# 99. CRAFT-97 — Crafting Statistics

Exemplos:

```text
items crafted
resources consumed
production time
favorite recipes
```

---

# 100. CRAFT-98 — Debug

Comandos:

```text
nexora recipe inspect
nexora recipe validate
nexora recipe list
nexora recipe simulate
nexora crafting queue
nexora production graph
```

---

# 101. CRAFT-99 — Balance Simulator

Simular:

```text
10.000 crafts
```

para medir:

```text
resource consumption
production rate
economic impact
```

---

# 102. CRAFT-100 — Recipe Graph Analyzer

Analisar automaticamente:

```text
cycles
dead ends
unreachable recipes
overpowered chains
infinite loops
```

---

# 103. CRAFT-101 — Circular Recipes

Detectar situações como:

```text
A → B
B → C
C → A
```

quando isso gerar recursos infinitos.

---

# 104. CRAFT-102 — Resource Conservation

Ferramenta de balanceamento deve verificar:

```text
input value
+
processing
=
output value
```

segundo regras de economia.

---

# 105. CRAFT-103 — Duplication Protection

Todas as operações devem ter:

```text
transactionId
```

para impedir duplicação em multiplayer.

---

# 106. CRAFT-104 — Multiplayer Authority

Fluxo:

```text
Client
 ↓
Craft Request
 ↓
Server
 ↓
Validation
 ↓
Crafting
 ↓
Inventory Update
 ↓
Replication
```

---

# 107. CRAFT-105 — Concurrent Crafting

Se dois sistemas quiserem o mesmo ingrediente:

```text
Inventory
 ↓
atomic reservation
```

impede ambos de gastarem o mesmo recurso.

---

# 108. CRAFT-106 — Save

Salvar:

```text
craft queue
machine state
recipe version
progress
```

quando necessário.

---

# 109. CRAFT-107 — Version Migration

Receitas antigas devem ser migráveis.

---

# 110. CRAFT-108 — Mod API

Mods podem registrar:

```text
Recipe
Ingredient
RecipeType
CraftingStation
ProcessingType
RecipeCondition
RecipeModifier
Material
QualityRule
```

---

# 111. CRAFT-109 — Data Driven

Idealmente, grande parte do conteúdo de crafting deve vir de dados.

```text
definition
 ↓
validation
 ↓
registry
 ↓
runtime
```

Isso será particularmente importante porque o NEXORA pretende ter **muitos milhares de itens e receitas**.

---

# 112. CRAFT-110 — Official Content

Conteúdo oficial usa a mesma API:

```text
Official Module
 ↓
Crafting API

Community Mod
 ↓
Crafting API
```

---

# 113. CRAFT-111 — Recipe Categories

Categorias:

```text
Basic
Tools
Equipment
Machines
Components
Food
Agriculture
Magic
Technology
Vehicles
Space
```

---

# 114. CRAFT-112 — Search by Dependency

Permitir descobrir:

```text
"o que posso fabricar com copper?"
```

e:

```text
"onde copper é usado?"
```

---

# 115. CRAFT-113 — Reverse Recipe Graph

```text
Material
 ↓
all recipes using material
```

Extremamente útil para planejamento.

---

# 116. CRAFT-114 — Crafting Discovery

Quando o player obtém um material novo:

```text
Material discovered
 ↓
recipes become visible/unknown
```

Dependendo das regras de conhecimento.

---

# 117. CRAFT-115 — Experimentation

Um sistema opcional pode permitir:

```text
unknown recipe
 ↓
experiment
 ↓
discover result
```

Isso combina muito bem com Knowledge.

---

# 118. CRAFT-116 — Research Crafting

Alguns processos avançados podem exigir:

```text
research points
knowledge
experiments
```

---

# 119. CRAFT-117 — Quality by Skill

Artesãos diferentes podem produzir resultados diferentes dentro de limites definidos.

```text
NPC Skill
+
Recipe
+
Material
=
Quality
```

---

# 120. CRAFT-118 — Civilization Specialization

Civilizações podem se especializar:

```text
metalworking
textiles
alchemy
engineering
magic
shipbuilding
```

Isso muda o que produzem com eficiência.

---

# 121. CRAFT-119 — Regional Production

Uma região rica em certo material pode desenvolver uma cadeia industrial específica.

```text
resource abundance
 ↓
industry
 ↓
specialization
 ↓
trade
```

---

# 122. CRAFT-120 — Supply Chain

Crafting pode virar:

```text
Raw Resource
 ↓
Processing
 ↓
Intermediate
 ↓
Component
 ↓
Assembly
 ↓
Finished Product
 ↓
Market
```

---

# 123. CRAFT-121 — Final Architecture

```text
                           CRAFTING
                              │
              ┌───────────────┼────────────────┐
              │               │                │
           RECIPES         STATIONS         MATERIALS
              │               │                │
          ingredients       player            tags
          outputs            forge             quality
          conditions         machine           purity
          versions           assembler
              │               │                │
              └───────────────┼────────────────┘
                              │
                     PROCESSING ENGINE
                              │
       ┌──────────────────────┼───────────────────────┐
       │                      │                       │
    CRAFTING              PROCESSING             ASSEMBLY
       │                      │                       │
    manual                 smelting                machines
    recipes                refining                multiblocks
    tools                  chemistry               equipment
       │                      │                       │
       └──────────────────────┼───────────────────────┘
                              │
       ┌──────────────────────┼───────────────────────┐
       │                      │                       │
   INVENTORY               ENERGY                  FLUID
       │                      │                       │
   materials               power                   inputs
                              │
                       KNOWLEDGE / SKILLS
                              │
                         PROGRESSION
                              │
                       ECONOMY / INDUSTRY
```

# 124. Ordem de implementação

```text
CRAFT-0 Core
CRAFT-1 Recipe Definition
CRAFT-2 Recipe Registry
CRAFT-3 Ingredients
CRAFT-4 Tags
CRAFT-5 Shaped Recipes
CRAFT-6 Shapeless Recipes
CRAFT-7 Outputs
CRAFT-8 Stations
CRAFT-9 Craft Validation
CRAFT-10 Transactions
CRAFT-11 Batch Crafting
CRAFT-12 Queue
CRAFT-13 Processing
CRAFT-14 Energy Integration
CRAFT-15 Fluid Integration
CRAFT-16 Temperature
CRAFT-17 Pressure
CRAFT-18 Requirements
CRAFT-19 Knowledge
CRAFT-20 Progression
CRAFT-21 Tool Crafting
CRAFT-22 Equipment Crafting
CRAFT-23 Modular Assembly
CRAFT-24 Material/Quality
CRAFT-25 Alloy
CRAFT-26 Disassembly
CRAFT-27 Salvage
CRAFT-28 Machine Crafting
CRAFT-29 Automation
CRAFT-30 Production Chains
CRAFT-31 NPC Crafting
CRAFT-32 Civilization Industry
CRAFT-33 Economy
CRAFT-34 Production LOD
CRAFT-35 Persistence
CRAFT-36 Multiplayer
CRAFT-37 Debugging
CRAFT-38 Recipe Analyzer
CRAFT-39 Balance Simulator
CRAFT-40 Mod API
CRAFT-41 Stress Testing
```

# 125. Primeiro Vertical Slice

O primeiro teste:

```text
Player
 ↓
Recipe Registry
 ↓
Recipe
 ↓
check Inventory
 ↓
check station
 ↓
reserve materials
 ↓
craft
 ↓
consume materials
 ↓
create output
 ↓
Inventory
 ↓
save
```

Depois:

```text
Ore
 ↓
Furnace
 ↓
Energy
 ↓
Heat
 ↓
Metal
 ↓
Component
 ↓
Assembler
 ↓
Machine
```

E o slice realmente importante para o NEXORA:

```text
Mine
 ↓
Ore
 ↓
Transport
 ↓
Processing
 ↓
Metal
 ↓
Components
 ↓
Factory
 ↓
Machine
 ↓
Energy
 ↓
Production
 ↓
Storage
 ↓
Economy
 ↓
Trade
 ↓
Civilization
```

## Regra arquitetural

> **Recipe descreve. Crafting executa. Inventory fornece recursos. Machines processam. Energy e Fluid fornecem recursos industriais. Knowledge e Progression determinam acesso. Economy dá valor aos resultados.**

E isso evita um problema enorme que seria comum em um jogo desse tamanho: criar centenas de sistemas separados para “crafting manual”, “fábricas”, “forjas”, “magia”, “tecnologia”, “componentes”, etc.

No NEXORA, tudo pode começar com:

```text
Ingredient
+
Process
+
Station
+
Conditions
→
Result
```

e depois escalar de:

```text
2 gravetos + 3 pedras
```

até:

```text
minério
→ refinaria
→ liga
→ componentes
→ circuito
→ máquina
→ fábrica
→ infraestrutura
→ tecnologia avançada
→ equipamento espacial
```

usando a **mesma espinha dorsal de Crafting/Processing**.
