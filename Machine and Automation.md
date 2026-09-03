Sim. **Machines / Automation** provavelmente será um dos maiores sistemas do NEXORA, porque ele conecta quase tudo que já definimos:

```text
Resources
   ↓
Machines
   ↓
Energy / Fluids
   ↓
Processing
   ↓
Automation
   ↓
Storage
   ↓
Logistics
   ↓
Production
   ↓
Economy
   ↓
Civilization
```

Eu faria ele como um sistema **genérico de máquinas e redes**, sem criar uma implementação separada para cada família tecnológica.

# NEXORA — MACHINES / AUTOMATION MASTER PLAN

## 1. Objetivo

O sistema deve permitir:

```text
máquinas
geradores
motores
processadores
fornos
bombas
turbinas
montadores
reatores
armazenamento
tubulações
transportadores
redes de energia
redes de fluidos
redes logísticas
controladores
sensores
automação
fábricas
```

E suportar diferentes paradigmas:

```text
energia elétrica-like
potência mecânica
fluidos
calor
magia
sinais
logística
```

A regra central:

> **Machine System fornece máquinas e redes. Cada conteúdo decide como sua máquina funciona através de APIs.**

---

# 2. Arquitetura geral

```text
                         MACHINE SYSTEM
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
      MACHINES              NETWORKS              LOGIC
        │                      │                      │
    Processor              Energy Network       Controllers
    Generator              Fluid Network       Sensors
    Storage                Logistics Network   Signals
    Reactor                Mechanical Network  Automation
        │                      │                      │
        └──────────────────────┼──────────────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          │                    │                    │
       ENERGY                FLUIDS             MATERIAL
          │                    │                    │
      generators            pipes               inputs
      batteries              tanks               outputs
      machines               pumps              inventory
          │                    │                    │
          └────────────────────┼────────────────────┘
                               │
                         CRAFTING / PROCESSING
                               │
                        STORAGE / LOGISTICS
                               │
                          ECONOMY / WORLD
```

---

# 3. MACHINE-0 — Machine Core

Criar:

```text
MachineSystem
MachineDefinition
MachineInstance
MachineContext
MachineState
```

Fluxo:

```text
Load Machine
 ↓
Initialize
 ↓
Connect Networks
 ↓
Receive Inputs
 ↓
Process
 ↓
Produce Outputs
 ↓
Update State
 ↓
Save
```

---

# 4. MACHINE-1 — Machine Definition

Definição estática:

```text
MachineDefinition
├── id
├── capabilities
├── inventory
├── energy
├── fluids
├── processing
├── control
├── visuals
└── rules
```

---

# 5. MACHINE-2 — Machine Instance

Estado individual:

```text
MachineInstance
├── id
├── position
├── orientation
├── state
├── inventory
├── networkRefs
└── customData
```

---

# 6. MACHINE-3 — Machine Lifecycle

Estados:

```text
CREATED
PLACED
INITIALIZING
IDLE
RUNNING
PAUSED
BLOCKED
ERROR
DISABLED
UNLOADING
```

---

# 7. MACHINE-4 — Machine Capabilities

Uma máquina pode oferecer:

```text
Processing
Generating
Storage
Pumping
Transport
Assembly
Smelting
Cooling
Heating
Charging
Control
```

Uma máquina pode ter várias.

---

# 8. MACHINE-5 — Processing Machine

Base:

```text
Processor
```

Entrada:

```text
items
fluids
energy
```

Saída:

```text
items
fluids
byproducts
```

---

# 9. MACHINE-6 — Processing Recipe

Utilizar o Crafting System:

```text
Machine
 ↓
Recipe
 ↓
Process
```

Não criar um sistema de receitas totalmente separado.

---

# 10. MACHINE-7 — Processing State

```text
ProcessingState
├── recipe
├── progress
├── duration
├── inputs
└── outputs
```

---

# 11. MACHINE-8 — Machine Tick

Não atualizar máquinas necessariamente a cada frame.

```text
Renderer
→ frame

Machine Simulation
→ tick

Network
→ tick

Production
→ scheduled tick
```

---

# 12. MACHINE-9 — Parallel Processing

Máquinas independentes podem ser processadas em paralelo:

```text
Machine A
Machine B
Machine C
```

desde que não exista conflito de dados.

---

# 13. MACHINE-10 — Machine Scheduling

Criar:

```text
MachineScheduler
```

com prioridades:

```text
CRITICAL
HIGH
NORMAL
LOW
BACKGROUND
```

---

# 14. MACHINE-11 — Energy API

Máquinas devem poder consumir/produzir energia através de uma API genérica:

```text
EnergyNode
├── capacity
├── inputRate
├── outputRate
├── stored
└── mode
```

---

# 15. MACHINE-12 — Energy Network

```text
Generator
 ↓
Network
 ├── Machine
 ├── Battery
 └── Consumer
```

---

# 16. MACHINE-13 — Energy Transfer

Criar operação:

```text
EnergyTransfer
├── source
├── destination
├── amount
└── transactionId
```

---

# 17. MACHINE-14 — Energy Priorities

Consumidores podem ter prioridade:

```text
CRITICAL
HIGH
NORMAL
LOW
```

Assim uma rede pode priorizar:

```text
life support
 ↓
factory
 ↓
decorative lighting
```

---

# 18. MACHINE-15 — Energy Storage

Criar:

```text
Battery
EnergyCell
Capacitor
Accumulator
```

através de uma API comum.

---

# 19. MACHINE-16 — Energy Loss

Redes podem possuir:

```text
transferLoss
distanceLoss
conversionLoss
```

dependendo da tecnologia.

---

# 20. MACHINE-17 — Fluid Network

Integração com o Fluid Engine:

```text
Pump
 ↓
Pipe
 ↓
Tank
 ↓
Machine
```

---

# 21. MACHINE-18 — Fluid Node

Uma máquina pode possuir:

```text
FluidPort
├── input
├── output
├── capacity
└── acceptedTags
```

---

# 22. MACHINE-19 — Fluid Routing

A rede calcula:

```text
source
 ↓
network
 ↓
destination
```

usando o Fluid Engine para o estado dos fluidos.

---

# 23. MACHINE-20 — Thermal System

Máquinas podem possuir:

```text
temperature
heatCapacity
heatGeneration
coolingRate
```

---

# 24. MACHINE-21 — Heat Network

```text
Heat Source
 ↓
Thermal Network
 ↓
Machine
```

Pode funcionar junto com:

```text
Fluid
Energy
Environment
```

---

# 25. MACHINE-22 — Thermal Exchange

```text
Hot Fluid
 ↓
Machine
 ↓
Heat Exchange
 ↓
Cooler Fluid
```

---

# 26. MACHINE-23 — Pressure

Máquinas de fluidos podem possuir:

```text
pressure
maxPressure
minPressure
```

---

# 27. MACHINE-24 — Mechanical Power

Para sistemas mecânicos:

```text
MechanicalNode
├── torque
├── speed
├── power
```

---

# 28. MACHINE-25 — Mechanical Networks

```text
Engine
 ↓
Shaft
 ↓
Gear
 ↓
Machine
```

---

# 29. MACHINE-26 — Gears

Criar:

```text
Gear
Gearbox
Transmission
```

como componentes de rede.

---

# 30. MACHINE-27 — Rotational State

```text
rotationSpeed
torque
direction
```

---

# 31. MACHINE-28 — Mechanical Load

Cada máquina pode exigir:

```text
requiredPower
requiredTorque
```

e parar se não receber o suficiente.

---

# 32. MACHINE-29 — Power Conversion

Um sistema pode converter:

```text
mechanical
→ energy

energy
→ mechanical

fluid pressure
→ mechanical
```

através de máquinas específicas.

---

# 33. MACHINE-30 — Generator

Gerador genérico:

```text
Generator
├── input
├── conversion
└── output
```

---

# 34. MACHINE-31 — Fuel System

Geradores podem consumir:

```text
Fuel
```

através de uma API de recursos.

O combustível não precisa ser hardcoded.

---

# 35. MACHINE-32 — Fuel Processing

Fluxo:

```text
Raw Resource
 ↓
Processing
 ↓
Fuel
 ↓
Generator
 ↓
Energy
```

---

# 36. MACHINE-33 — Reactor Framework

Criar infraestrutura para grandes reatores:

```text
Reactor
├── fuel
├── heat
├── energy
├── cooling
└── control
```

---

# 37. MACHINE-34 — Multiblock

Reatores e estruturas industriais podem usar:

```text
MultiblockDefinition
MultiblockInstance
```

em conjunto com Build System.

---

# 38. MACHINE-35 — Multiblock Validation

```text
Controller
 ↓
scan structure
 ↓
validate
 ↓
form machine
```

---

# 39. MACHINE-36 — Multiblock State

A estrutura inteira pode atuar como:

```text
Single Machine Runtime
```

apesar de ocupar vários chunks.

---

# 40. MACHINE-37 — Large Reactors

A arquitetura precisa permitir:

```text
small generator
medium reactor
large reactor
```

usando a mesma base.

---

# 41. MACHINE-38 — Cooling

Reatores podem possuir:

```text
coolant
heat exchanger
radiator
```

e usar Fluid/Thermal APIs.

---

# 42. MACHINE-39 — Machine Efficiency

A eficiência pode depender de:

```text
input quality
temperature
energy supply
machine state
upgrades
```

---

# 43. MACHINE-40 — Machine Upgrades

Uma máquina pode possuir:

```text
UpgradeSlot
```

e aceitar:

```text
speed
efficiency
capacity
automation
```

---

# 44. MACHINE-41 — Modules

Máquinas avançadas podem ser modulares:

```text
Core
+
Module
+
Module
```

---

# 45. MACHINE-42 — Machine Configuration

Criar:

```text
MachineConfig
```

com:

```text
input
output
priority
mode
filters
```

---

# 46. MACHINE-43 — Machine Modes

Exemplo:

```text
PROCESS
STORE
EXPORT
IMPORT
AUTO
MANUAL
```

---

# 47. MACHINE-44 — Filters

Máquinas podem aceitar filtros:

```text
item tags
fluid tags
quality
category
```

---

# 48. MACHINE-45 — Automation Core

Agora começa a parte realmente grande.

Criar:

```text
AutomationSystem
```

capaz de conectar:

```text
machines
storage
pipes
conveyors
sensors
controllers
```

---

# 49. MACHINE-46 — Signals

Criar:

```text
Signal
```

com:

```text
value
type
source
timestamp
```

---

# 50. MACHINE-47 — Logic Gates

Uma camada opcional:

```text
AND
OR
NOT
XOR
COMPARE
```

---

# 51. MACHINE-48 — Controllers

```text
Controller
 ↓
read sensors
 ↓
evaluate logic
 ↓
send commands
```

---

# 52. MACHINE-49 — Sensors

Sensores podem detectar:

```text
item count
fluid amount
energy
temperature
pressure
machine state
entity
player
environment
```

---

# 53. MACHINE-50 — Sensor API

```text
Sensor
├── query
├── interval
├── range
└── output
```

---

# 54. MACHINE-51 — Timers

Automação pode usar:

```text
Timer
Counter
Scheduler
```

---

# 55. MACHINE-52 — Event Triggers

Gatilhos:

```text
onItemReceived
onTankFull
onEnergyLow
onMachineDone
onSignal
```

---

# 56. MACHINE-53 — Logic Graph

Automação pode formar:

```text
Sensor
 ↓
Comparator
 ↓
Controller
 ↓
Machine
```

---

# 57. MACHINE-54 — Automation Graph

Um sistema completo:

```text
INPUT
 ↓
PROCESS
 ↓
CONDITION
 ↓
ACTION
```

---

# 58. MACHINE-55 — State Machines

Controladores complexos podem usar:

```text
IDLE
 ↓
PREPARE
 ↓
RUN
 ↓
OUTPUT
 ↓
ERROR
```

---

# 59. MACHINE-56 — Programmable Controllers

Criar uma abstração:

```text
ProgrammableController
```

que executa lógica limitada/sandboxed.

---

# 60. MACHINE-57 — Scripting

No futuro:

```text
Automation Script
 ↓
Sandbox
 ↓
Machine APIs
```

Nunca permitir acesso irrestrito ao sistema.

---

# 61. MACHINE-58 — Conveyor Network

Criar:

```text
Conveyor
Splitter
Merger
Filter
Buffer
```

---

# 62. MACHINE-59 — Item Transport

Itens podem viajar:

```text
Storage
 ↓
Conveyor
 ↓
Machine
```

---

# 63. MACHINE-60 — Logical vs Physical Transport

Não simular cada item como rigid body.

Ter:

```text
Logical Item Transport
```

e somente renderizar os itens que precisam ser visualizados.

Isso economiza muito desempenho.

---

# 64. MACHINE-61 — Item Routing

Rotas:

```text
input
 ↓
filter
 ↓
destination
```

---

# 65. MACHINE-62 — Storage Networks

Integrar com Storage API:

```text
Warehouse
Network
Container
Machine
```

---

# 66. MACHINE-63 — Automated Crafting

Uma fábrica pode pedir:

```text
Recipe
```

e automaticamente:

```text
request ingredients
 ↓
process
 ↓
output
```

---

# 67. MACHINE-64 — Crafting Integration

```text
Machine
 ↓
Crafting System
 ↓
Recipe
```

---

# 68. MACHINE-65 — Fluid Automation

```text
Pump
 ↓
Pipe
 ↓
Machine
 ↓
Output Tank
```

---

# 69. MACHINE-66 — Energy Automation

```text
Generator
 ↓
Network
 ↓
Battery
 ↓
Machine
```

---

# 70. MACHINE-67 — Multi-Network Machines

Uma máquina pode possuir:

```text
Energy
Fluid
Item
Heat
Mechanical
```

simultaneamente.

---

# 71. MACHINE-68 — Ports

Criar:

```text
MachinePort
```

tipos:

```text
ITEM
FLUID
ENERGY
MECHANICAL
HEAT
SIGNAL
```

---

# 72. MACHINE-69 — Port Configuration

Cada porta pode possuir:

```text
INPUT
OUTPUT
BOTH
```

e filtros.

---

# 73. MACHINE-70 — Connections

Máquinas podem conectar automaticamente se:

```text
compatible
adjacent
configured
```

---

# 74. MACHINE-71 — Network Discovery

Ao colocar uma máquina:

```text
new node
 ↓
network search
 ↓
connect
```

---

# 75. MACHINE-72 — Network Merge

Duas redes podem virar uma:

```text
Network A
+
Network B
 ↓
Network C
```

---

# 76. MACHINE-73 — Network Split

Ao remover um elemento:

```text
Network
 ↓
disconnect
 ↓
Network A + Network B
```

---

# 77. MACHINE-74 — Network Rebuild

Não reconstruir tudo sempre.

Usar:

```text
dirty graph
 ↓
recalculate affected area
```

---

# 78. MACHINE-75 — Network Graph

Representação:

```text
Node
Edge
Network
```

para cada domínio.

---

# 79. MACHINE-76 — Routing

Storage/logistics precisa de:

```text
route
priority
filters
cost
```

---

# 80. MACHINE-77 — Logistics Scheduler

Decidir:

```text
what moves
where
when
priority
```

---

# 81. MACHINE-78 — Warehouses

Grandes depósitos:

```text
Warehouse
├── capacity
├── categories
├── inputs
└── outputs
```

---

# 82. MACHINE-79 — Factory

Criar conceito:

```text
Factory
```

como coleção organizada de máquinas.

---

# 83. MACHINE-80 — Production Line

```text
Input
 ↓
Machine A
 ↓
Buffer
 ↓
Machine B
 ↓
Machine C
 ↓
Output
```

---

# 84. MACHINE-81 — Production Controller

Controlar toda uma linha:

```text
ProductionController
```

---

# 85. MACHINE-82 — Bottleneck Detection

Sistema deve conseguir identificar:

```text
machine too slow
energy insufficient
fluid shortage
storage full
transport bottleneck
```

---

# 86. MACHINE-83 — Factory Analytics

Métricas:

```text
throughput
efficiency
uptime
downtime
input rate
output rate
```

---

# 87. MACHINE-84 — Machine Failures

Máquinas podem possuir:

```text
failureState
```

causado por:

```text
overheat
overpressure
lackOfEnergy
lackOfFluid
wear
```

---

# 88. MACHINE-85 — Maintenance

Criar:

```text
MaintenanceState
```

e:

```text
maintenanceRequired
```

---

# 89. MACHINE-86 — Wear

Máquinas podem possuir:

```text
wear
```

que cresce com uso.

---

# 90. MACHINE-87 — Repair

Integração com Tool/Build:

```text
Repair Tool
 ↓
Machine
 ↓
maintenance
```

---

# 91. MACHINE-88 — Safety

Grandes sistemas podem possuir:

```text
pressure limit
temperature limit
power limit
```

e entrar em estado:

```text
SAFE
WARNING
CRITICAL
SHUTDOWN
```

---

# 92. MACHINE-89 — Control Systems

Uma máquina pode ter:

```text
manual control
automatic control
emergency shutdown
```

---

# 93. MACHINE-90 — Alarm System

Eventos:

```text
EnergyLow
TemperatureHigh
PressureHigh
StorageFull
MachineFailure
```

---

# 94. MACHINE-91 — Machine Events

```text
MachinePlaced
MachineStarted
MachineStopped
MachineCompleted
MachineError
MachineRepaired
NetworkConnected
NetworkDisconnected
```

---

# 95. MACHINE-92 — World Integration

Máquinas podem modificar:

```text
temperature
fluid
energy
resources
environment
```

através das respectivas APIs.

---

# 96. MACHINE-93 — Civilization Integration

Civilizações podem criar:

```text
workshops
factories
power plants
water networks
industrial districts
```

---

# 97. MACHINE-94 — Economy Integration

Produção:

```text
resources
 ↓
factory
 ↓
goods
 ↓
market
```

---

# 98. MACHINE-95 — Industry Simulation

Distante do player:

```text
factory
 ↓
aggregate production
```

Não simular cada máquina individualmente quando não necessário.

---

# 99. MACHINE-96 — Machine LOD

```text
FULL
→ every machine

REGIONAL
→ production aggregate

ABSTRACT
→ industrial statistics
```

---

# 100. MACHINE-97 — Offline Production

Fábricas persistentes podem continuar progredindo durante o tempo simulado quando as regras do mundo permitirem.

---

# 101. MACHINE-98 — Player Automation

Player pode construir:

```text
automated farms
factories
storage
transport
```

---

# 102. MACHINE-99 — NPC Automation

Civilizações também podem operar:

```text
farms
mines
workshops
factories
power systems
```

---

# 103. MACHINE-100 — Civilization Industry

```text
Resources
 ↓
Industry
 ↓
Goods
 ↓
Trade
 ↓
Civilization
```

---

# 104. MACHINE-101 — Railway Integration

Trens podem transportar:

```text
raw resources
components
fuel
food
goods
```

entre fábricas.

---

# 105. MACHINE-102 — Logistics Integration

```text
Factory
 ↓
Warehouse
 ↓
Rail
 ↓
City
```

---

# 106. MACHINE-103 — Vehicles

Veículos podem fazer logística:

```text
truck
train
ship
aircraft
```

através das APIs de transporte.

---

# 107. MACHINE-104 — Machine Rendering

Cada máquina pode fornecer:

```text
MachineRenderState
```

Renderer mostra:

```text
animation
lights
fluid
moving parts
screens
```

---

# 108. MACHINE-105 — Machine Physics

Somente partes relevantes usam Physics.

Não transformar cada engrenagem visual em rigid body.

---

# 109. MACHINE-106 — Machine Audio

Machine fornece:

```text
running
startup
shutdown
alarm
```

O Audio System interpreta.

---

# 110. MACHINE-107 — Machine Configuration UI

Interface para:

```text
inputs
outputs
filters
modes
network
upgrades
```

---

# 111. MACHINE-108 — Network Visualization

Mostrar:

```text
energy network
fluid network
item network
mechanical network
```

---

# 112. MACHINE-109 — Debug

Comandos:

```text
nexora machine inspect
nexora machine network
nexora machine simulate
nexora machine stats
nexora automation inspect
nexora network diagnose
```

---

# 113. MACHINE-110 — Network Profiler

Métricas:

```text
nodes
edges
throughput
latency
energy transfer
fluid throughput
items transferred
rebuild time
```

---

# 114. MACHINE-111 — Production Profiler

```text
machine count
active machines
production rate
idle rate
failure rate
```

---

# 115. MACHINE-112 — Automation Debugger

Visualizar:

```text
sensor
 ↓
logic
 ↓
controller
 ↓
machine
```

---

# 116. MACHINE-113 — Graph Validation

Detectar:

```text
cycles
dead ends
unreachable nodes
invalid connections
overload
```

---

# 117. MACHINE-114 — Infinite Loop Protection

Automação não pode ficar:

```text
A
 ↓
B
 ↓
C
 ↓
A
```

gerando trabalho infinitamente.

---

# 118. MACHINE-115 — Tick Budget

Cada automação possui orçamento:

```text
operations/tick
```

---

# 119. MACHINE-116 — Sandbox

Controladores programáveis precisam de:

```text
instruction limits
memory limits
execution limits
API permissions
```

---

# 120. MACHINE-117 — Multiplayer Authority

Servidor controla:

```text
network topology
machine state
automation
resource transfers
```

---

# 121. MACHINE-118 — Transactions

Transferências:

```text
Item
Energy
Fluid
```

usam transações.

---

# 122. MACHINE-119 — Duplication Prevention

Especialmente:

```text
machines
storage
automation
multiplayer
```

devem validar operações atomicamente.

---

# 123. MACHINE-120 — Persistence

Salvar:

```text
machine state
network state
queues
inventory
tank contents
production progress
```

quando necessário.

---

# 124. MACHINE-121 — Network Reconstruction

No load:

```text
machines
 ↓
network discovery
 ↓
rebuild
```

---

# 125. MACHINE-122 — Versioning

```text
MachineDataVersion
NetworkDataVersion
AutomationDataVersion
```

---

# 126. MACHINE-123 — Migration

Atualizações podem transformar:

```text
old machine state
 ↓
migration
 ↓
new state
```

---

# 127. MACHINE-124 — Mod API

Mods podem registrar:

```text
MachineDefinition
MachineCapability
MachinePort
EnergyNode
FluidNode
MechanicalNode
HeatNode
Sensor
Controller
AutomationRule
NetworkType
Upgrade
```

---

# 128. MACHINE-125 — Official Content

Conteúdo oficial utiliza as mesmas interfaces.

```text
Official Machine
      ↓
Machine API

Community Machine
      ↓
Machine API
```

---

# 129. MACHINE-126 — Extensible Network Types

O NEXORA pode começar com:

```text
Energy
Fluid
Item
Mechanical
Signal
```

e adicionar outros posteriormente.

---

# 130. MACHINE-127 — Universal Node Model

Eu criaria uma abstração comum:

```text
NetworkNode
├── nodeId
├── ports
├── capabilities
└── networkRefs
```

---

# 131. MACHINE-128 — Port Model

```text
Port
├── type
├── direction
├── capacity
├── filter
└── priority
```

---

# 132. MACHINE-129 — Network API

```text
INetwork
INetworkNode
INetworkPort
INetworkGraph
INetworkTransfer
```

---

# 133. MACHINE-130 — Machine API

```text
IMachine
IMachineProcess
IMachineInventory
IMachineController
IMachineMaintenance
```

---

# 134. MACHINE-131 — Automation API

```text
IAutomationNode
ISensor
IController
ISignal
IAutomationAction
```

---

# 135. MACHINE-132 — Final Architecture

```text
                           MACHINE SYSTEM
                                │
               ┌────────────────┼────────────────┐
               │                │                │
            MACHINES         NETWORKS         CONTROL
               │                │                │
          processors         energy            sensors
          generators         fluids            signals
          storage            items             logic
          reactors           mechanical        controllers
               │                │                │
               └────────────────┼────────────────┘
                                │
                         UNIVERSAL NODE API
                                │
            ┌───────────────────┼────────────────────┐
            │                   │                    │
         ENERGY               FLUID               ITEMS
            │                   │                    │
        generator             pump               storage
        battery               pipe                conveyor
        machine               tank                warehouse
            │                   │                    │
            └───────────────────┼────────────────────┘
                                │
                            PROCESSING
                                │
                         CRAFTING SYSTEM
                                │
                       PRODUCTION CHAINS
                                │
             ┌──────────────────┼──────────────────┐
             │                  │                  │
          PLAYER             NPC/CIV            WORLD
             │                  │                  │
        factories          industry           resources
        automation         production          logistics
             │                  │                  │
             └──────────────────┼──────────────────┘
                                │
                              ECONOMY
```

---

# 136. Ordem de implementação

Eu faria em camadas:

```text
MACHINE-0 Core
MACHINE-1 Definition
MACHINE-2 Instance
MACHINE-3 Lifecycle
MACHINE-4 Capability
MACHINE-5 Machine Tick
MACHINE-6 Inventory Port
MACHINE-7 Processing
MACHINE-8 Crafting Integration
MACHINE-9 Energy Node
MACHINE-10 Energy Network
MACHINE-11 Energy Storage
MACHINE-12 Fluid Port
MACHINE-13 Fluid Network
MACHINE-14 Heat System
MACHINE-15 Pressure
MACHINE-16 Mechanical Network
MACHINE-17 Generators
MACHINE-18 Reactors
MACHINE-19 Multiblock
MACHINE-20 Machine Upgrades
MACHINE-21 Maintenance
MACHINE-22 Automation Core
MACHINE-23 Sensors
MACHINE-24 Signals
MACHINE-25 Controllers
MACHINE-26 Logic
MACHINE-27 Conveyors
MACHINE-28 Storage Networks
MACHINE-29 Logistics
MACHINE-30 Production Lines
MACHINE-31 Factory
MACHINE-32 Analytics
MACHINE-33 NPC Industry
MACHINE-34 Civilization Industry
MACHINE-35 Railway/Vehicle Logistics
MACHINE-36 LOD
MACHINE-37 Multiplayer
MACHINE-38 Persistence
MACHINE-39 Debug
MACHINE-40 Mod API
MACHINE-41 Stress Testing
```

# 137. Primeiro Vertical Slice

O primeiro slice deve provar a infraestrutura:

```text
Generator
 ↓
Energy Network
 ↓
Machine
 ↓
Recipe
 ↓
Input
 ↓
Processing
 ↓
Output
 ↓
Storage
```

Depois:

```text
Water Source
 ↓
Pump
 ↓
Pipe
 ↓
Machine
 ↓
Fluid Output
```

Depois:

```text
Generator
 ↓
Power Network
 ↓
Machine
 ↓
Controller
 ↑
Sensor
```

E finalmente:

```text
MINE
 ↓
RAW RESOURCE
 ↓
TRANSPORT
 ↓
PROCESSOR
 ↓
COMPONENT
 ↓
ASSEMBLER
 ↓
MACHINE
 ↓
ENERGY
 ↓
STORAGE
 ↓
AUTOMATION
 ↓
FACTORY
 ↓
RAILWAY
 ↓
CITY
 ↓
ECONOMY
```

## Regra arquitetural final

> **Máquina é um consumidor/produtor de recursos. Rede conecta máquinas. Automation decide quando e como elas trabalham.**

Então não teremos:

```text
MekanismSystem
CreateSystem
BigReactorsSystem
```

como sistemas fundamentais.

Teremos:

```text
Machine API
Energy API
Fluid API
Mechanical API
Storage API
Automation API
Multiblock API
```

E os módulos oficiais podem construir em cima disso.

Isso é o que vai permitir ao NEXORA ter **fornos simples → oficinas → linhas industriais → grandes fábricas → redes energéticas → sistemas de fluidos → reatores → automação → infraestrutura de cidades → produção inter-regional**, usando uma arquitetura única.
