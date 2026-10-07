Sim. A **Energy API** precisa ser separada do sistema de Machines, porque ela deve ser uma infraestrutura universal. No NEXORA, energia não pode significar apenas “energia elétrica”: ela precisa conseguir representar diferentes formas de energia e permitir conversões entre sistemas.

A arquitetura-base seria:

```text
                        ENERGY API
                            │
       ┌────────────────────┼────────────────────┐
       │                    │                    │
    ENERGY TYPES         ENERGY NODES        NETWORKS
       │                    │                    │
    Electrical           Generator            Grid
    Mechanical           Consumer             Local Network
    Thermal              Storage              Regional Network
    Magical               Converter           Dimension Network
    Chemical              Battery
    Kinetic               Port
       │                    │                    │
       └────────────────────┼────────────────────┘
                            │
                 ┌──────────┼───────────┐
                 │          │           │
              MACHINE     STORAGE      PHYSICS
                 │          │           │
              process    batteries    mechanical
              reactor    capacitor    kinetic
              factory                thermal
                 │
                 └──────────┬──────────┘
                            │
                     AUTOMATION
                            │
                     ECONOMY / WORLD
```

# NEXORA — ENERGY API MASTER PLAN

## 1. Objetivo

A Energy API deve resolver:

```text
quem produz energia?
quem consome?
quanto existe?
quanto pode ser armazenado?
como ela é transportada?
quanto é perdido?
como é convertida?
como é priorizada?
como falhas são tratadas?
```

E principalmente:

> **Máquinas não devem implementar seu próprio sistema de energia. Elas devem consumir a Energy API.**

---

# 2. ENERGY-0 — Energy Core

Criar:

```text
EnergySystem
EnergyType
EnergyNode
EnergyPort
EnergyNetwork
EnergyTransfer
EnergyTransaction
```

Fluxo:

```text
Producer
 ↓
Network
 ↓
Consumer
```

---

# 3. ENERGY-1 — Energy Type

Energia deve ser extensível.

Tipos base:

```text
ELECTRICAL
MECHANICAL
THERMAL
CHEMICAL
KINETIC
MAGICAL
RADIANT
CUSTOM
```

O sistema não deve assumir que esses são os únicos tipos possíveis.

---

# 4. ENERGY-2 — Energy Unit

Criar uma unidade lógica comum:

```text
EnergyAmount
```

com suporte a:

```text
quantity
rate
capacity
```

Não precisa obrigatoriamente usar uma unidade física perfeita para todos os sistemas. O importante é manter uma semântica consistente.

---

# 5. ENERGY-3 — Energy Rate

Separar:

```text
Energy
```

de:

```text
Power
```

Conceitualmente:

```text
Energy = quantidade armazenada
Power = quantidade por unidade de tempo
```

Então:

```text
Battery
→ 5000 energy

Machine
→ consumes 100 energy/s
```

---

# 6. ENERGY-4 — Energy Node

Criar:

```text
EnergyNode
├── nodeId
├── energyType
├── capacity
├── stored
├── inputRate
├── outputRate
└── ports
```

---

# 7. ENERGY-5 — Energy Port

Uma máquina pode possuir vários ports:

```text
EnergyPort
├── direction
├── type
├── capacity
├── maxInput
└── maxOutput
```

Direções:

```text
INPUT
OUTPUT
BIDIRECTIONAL
```

---

# 8. ENERGY-6 — Producer

Geradores:

```text
EnergyProducer
```

fornecem:

```text
availableEnergy
maxOutput
```

---

# 9. ENERGY-7 — Consumer

Máquinas consumidoras:

```text
EnergyConsumer
```

declaram:

```text
requestedPower
maxPower
priority
```

---

# 10. ENERGY-8 — Storage

Criar interface comum:

```text
EnergyStorage
├── stored
├── capacity
├── inputRate
└── outputRate
```

---

# 11. ENERGY-9 — Battery

Implementação:

```text
Battery
```

com:

```text
capacity
chargeRate
dischargeRate
efficiency
```

---

# 12. ENERGY-10 — Capacitor

Outro armazenamento:

```text
Capacitor
```

pode ter:

```text
high discharge rate
lower capacity
```

dependendo da definição.

---

# 13. ENERGY-11 — Accumulator

Uma categoria genérica para armazenamento intermediário.

---

# 14. ENERGY-12 — Network

Criar:

```text
EnergyNetwork
├── nodes
├── connections
├── capacity
├── generation
├── consumption
├── storage
└── losses
```

---

# 15. ENERGY-13 — Network Graph

Representação:

```text
Node
 ↓
Edge
 ↓
Node
```

usada para:

```text
routing
connectivity
diagnostics
```

---

# 16. ENERGY-14 — Network Discovery

Quando um node é adicionado:

```text
Machine placed
 ↓
find neighbors
 ↓
compatible?
 ↓
join network
```

---

# 17. ENERGY-15 — Network Merge

```text
Network A
+
Network B
 ↓
Network C
```

---

# 18. ENERGY-16 — Network Split

Quando uma conexão é removida:

```text
Network
 ↓
disconnect
 ↓
A + B
```

---

# 19. ENERGY-17 — Network Rebuild

Só recalcular a área afetada.

```text
dirty graph
 ↓
rebuild affected region
```

---

# 20. ENERGY-18 — Transfer

Criar operação:

```text
EnergyTransfer
├── source
├── destination
├── amount
├── rate
└── transactionId
```

---

# 21. ENERGY-19 — Transfer Validation

Antes de transferir:

```text
source available?
destination capacity?
rate allowed?
same type?
permissions?
```

---

# 22. ENERGY-20 — Transaction

Transferência precisa ser transacional:

```text
BEGIN
 ↓
RESERVE
 ↓
TRANSFER
 ↓
COMMIT
```

---

# 23. ENERGY-21 — Rollback

Se alguma etapa falhar:

```text
ROLLBACK
```

sem perda ou duplicação.

---

# 24. ENERGY-22 — Energy Loss

Uma rede pode ter perdas:

```text
source
 ↓
cable
 ↓
machine
```

e:

```text
input > output
```

por causa de:

```text
transmission loss
conversion loss
heat loss
```

---

# 25. ENERGY-23 — Loss Profiles

Criar:

```text
EnergyLossProfile
```

com:

```text
fixedLoss
distanceLoss
percentageLoss
```

---

# 26. ENERGY-24 — Priority

Consumidores podem declarar:

```text
CRITICAL
HIGH
NORMAL
LOW
```

---

# 27. ENERGY-25 — Load Balancing

A rede pode distribuir energia:

```text
Generator
 ↓
Network
 ├── Machine A
 ├── Machine B
 └── Battery
```

conforme prioridade e disponibilidade.

---

# 28. ENERGY-26 — Overload

Se consumo ultrapassar produção:

```text
demand > supply
```

a rede entra em:

```text
NORMAL
WARNING
OVERLOADED
CRITICAL
```

---

# 29. ENERGY-27 — Brownout

Pode existir:

```text
partial power
```

em vez de simplesmente:

```text
ON / OFF
```

Isso permite máquinas reduzirem desempenho.

---

# 30. ENERGY-28 — Shutdown

Máquinas podem desligar quando:

```text
insufficient power
```

ou por segurança.

---

# 31. ENERGY-29 — Priority Shedding

Em sobrecarga:

```text
critical consumers
→ remain

low priority
→ shut down
```

---

# 32. ENERGY-30 — Generator

Criar interface:

```text
Generator
├── input
├── output
├── efficiency
└── state
```

---

# 33. ENERGY-31 — Fuel Generator

Fluxo:

```text
Fuel
 ↓
Generator
 ↓
Energy
```

O Fuel System pertence ao recurso/machine domain.

---

# 34. ENERGY-32 — Solar-like Generator

Uma implementação pode depender de:

```text
sun exposure
weather
time
```

Climate fornece esses dados.

---

# 35. ENERGY-33 — Wind Generator

Gerador pode consultar:

```text
wind speed
```

e converter:

```text
wind
 ↓
mechanical
 ↓
electrical
```

---

# 36. ENERGY-34 — Water Generation

Uma turbina pode consumir:

```text
fluid flow
pressure
```

e produzir:

```text
mechanical energy
```

---

# 37. ENERGY-35 — Mechanical Energy

Criar interface:

```text
MechanicalEnergyNode
```

com:

```text
torque
angularSpeed
power
```

---

# 38. ENERGY-36 — Mechanical Network

```text
Engine
 ↓
Shaft
 ↓
Gearbox
 ↓
Machine
```

---

# 39. ENERGY-37 — Gearbox

Pode converter:

```text
speed
```

e:

```text
torque
```

sob regras específicas.

---

# 40. ENERGY-38 — Mechanical Converter

Permitir:

```text
mechanical
↔
electrical
```

através de máquinas.

---

# 41. ENERGY-39 — Thermal Energy

Criar:

```text
ThermalEnergyNode
```

com:

```text
temperature
heatCapacity
storedHeat
transferRate
```

---

# 42. ENERGY-40 — Heat Transfer

```text
Hot Machine
 ↓
Heat Network
 ↓
Cooler
```

---

# 43. ENERGY-41 — Thermal Storage

Criar:

```text
ThermalStorage
```

para sistemas como:

```text
reactor
boiler
heat exchanger
```

---

# 44. ENERGY-42 — Energy Conversion

Uma máquina pode converter energia:

```text
INPUT TYPE
 ↓
Converter
 ↓
OUTPUT TYPE
```

---

# 45. ENERGY-43 — Converter

```text
EnergyConverter
├── inputType
├── outputType
├── efficiency
└── rate
```

---

# 46. ENERGY-44 — Conversion Loss

Exemplo:

```text
100 energy input
 ↓
conversion
 ↓
85 output
```

A diferença pode virar:

```text
heat
```

---

# 47. ENERGY-45 — Energy Cascade

Permitir:

```text
Chemical
 ↓
Thermal
 ↓
Mechanical
 ↓
Electrical
 ↓
Machine
```

---

# 48. ENERGY-46 — Magic Energy

O Magic System pode registrar:

```text
EnergyType = custom/magical
```

sem alterar o Core.

---

# 49. ENERGY-47 — Exotic Energy

Dimensões especiais podem registrar tipos adicionais:

```text
CUSTOM
```

---

# 50. ENERGY-48 — Energy Tags

Usar tags:

```text
#energy
#electrical
#mechanical
#thermal
#magical
```

para compatibilidade.

---

# 51. ENERGY-49 — Compatibility

Uma máquina pode aceitar:

```text
requiredEnergyTags
```

em vez de depender de um ID exato.

---

# 52. ENERGY-50 — Energy Bridge

Criar:

```text
EnergyBridge
```

que conecta sistemas diferentes.

Exemplo:

```text
Mechanical Network
 ↓
Generator
 ↓
Electrical Network
```

---

# 53. ENERGY-51 — Universal Conversion

O sistema não deve permitir automaticamente:

```text
A → B
```

só porque ambos são energy types.

Uma conversão precisa ser registrada explicitamente.

---

# 54. ENERGY-52 — Machine Integration

Machine pode declarar:

```text
EnergyConsumer
EnergyProducer
EnergyStorage
EnergyConverter
```

---

# 55. ENERGY-53 — Multi-Port Machine

Uma máquina pode possuir:

```text
electrical input
fluid input
mechanical output
heat output
```

---

# 56. ENERGY-54 — Reactor

Reator:

```text
Fuel
 ↓
Reaction
 ↓
Heat
 ↓
Energy
```

e pode produzir múltiplos outputs.

---

# 57. ENERGY-55 — Reactor Control

Criar:

```text
ReactorController
```

com:

```text
targetPower
temperature
safetyLimits
```

---

# 58. ENERGY-56 — Reactor Safety

Estados:

```text
SAFE
WARNING
CRITICAL
SHUTDOWN
```

---

# 59. ENERGY-57 — Cooling

Integração:

```text
Fluid Engine
 ↓
Coolant
 ↓
Reactor
 ↓
Heat
```

---

# 60. ENERGY-58 — Heat Rejection

Reator pode transferir calor para:

```text
radiator
environment
coolant
```

---

# 61. ENERGY-59 — Large-Scale Power

Grandes redes podem alimentar:

```text
city
factory
railway
industrial district
```

---

# 62. ENERGY-60 — Regional Grid

Criar níveis:

```text
Local Grid
Regional Grid
Large Grid
```

---

# 63. ENERGY-61 — Grid Interconnection

Redes podem conectar via:

```text
substation
converter
high-capacity link
```

---

# 64. ENERGY-62 — Energy Storage Grid

A rede pode balancear:

```text
generation
+
storage
-
consumption
```

---

# 65. ENERGY-63 — Demand

Cada consumidor publica:

```text
requestedPower
```

---

# 66. ENERGY-64 — Dynamic Demand

Uma máquina pode variar:

```text
low
normal
peak
```

conforme operação.

---

# 67. ENERGY-65 — Production Profiles

Geradores podem ser:

```text
constant
variable
weather-dependent
fuel-dependent
```

---

# 68. ENERGY-66 — Renewable-like Sources

Fontes podem depender do mundo:

```text
sun
wind
water
geothermal
```

---

# 69. ENERGY-67 — Geothermal

Integração:

```text
Deep World
 ↓
Geothermal Field
 ↓
Generator
 ↓
Energy
```

---

# 70. ENERGY-68 — Civilization Integration

Cidades podem consumir energia:

```text
houses
industry
transport
lighting
infrastructure
```

---

# 71. ENERGY-69 — Energy Economy

Energia pode virar recurso econômico:

```text
generation
 ↓
distribution
 ↓
consumption
 ↓
production
 ↓
trade
```

---

# 72. ENERGY-70 — Energy Price

Economy pode definir:

```text
energyCost
```

sem Energy API conhecer moeda.

---

# 73. ENERGY-71 — Industrial Demand

Fábricas podem aumentar a demanda regional.

---

# 74. ENERGY-72 — Grid Failure

Falhas podem resultar em:

```text
generator offline
 ↓
grid instability
 ↓
load shedding
 ↓
city effects
```

---

# 75. ENERGY-73 — Blackout Event

Criar:

```text
BlackoutEvent
```

como evento do mundo.

---

# 76. ENERGY-74 — Recovery

Após falha:

```text
repair
 ↓
restart
 ↓
reconnect
 ↓
grid stabilize
```

---

# 77. ENERGY-75 — Automation Integration

Automação pode consultar:

```text
energy stored
network load
generation
```

---

# 78. ENERGY-76 — Sensor

Sensor pode medir:

```text
power
energy
voltage-like property
load
capacity
```

Sem precisar usar física elétrica detalhada se não for necessário.

---

# 79. ENERGY-77 — Controller

```text
Sensor
 ↓
Controller
 ↓
Machine
```

---

# 80. ENERGY-78 — Energy Routing

Redes podem priorizar caminhos conforme:

```text
capacity
loss
priority
```

---

# 81. ENERGY-79 — Pathfinding

Para redes grandes:

```text
source
 ↓
network graph
 ↓
best route
 ↓
destination
```

---

# 82. ENERGY-80 — Network Topology

Detectar:

```text
isolated node
loop
bottleneck
overload
dead end
```

---

# 83. ENERGY-81 — Network Loop

Loops físicos podem existir.

O sistema deve evitar processamento infinito.

---

# 84. ENERGY-82 — Network Validation

```text
valid
warning
invalid
```

---

# 85. ENERGY-83 — Network Security

No multiplayer:

```text
server
 ↓
validates
 ↓
energy transfer
```

---

# 86. ENERGY-84 — Duplication Protection

Usar:

```text
transactionId
```

para operações de energia que alteram estado persistente.

---

# 87. ENERGY-85 — Multiplayer Authority

Cliente não declara:

```text
"minha bateria agora tem 1 milhão"
```

Servidor calcula o estado.

---

# 88. ENERGY-86 — Network Replication

Cliente recebe apenas:

```text
network state changes
```

necessárias.

---

# 89. ENERGY-87 — Client Visualization

Renderer pode mostrar:

```text
cables
energy flow
machine indicators
```

mas isso é visual.

---

# 90. ENERGY-88 — Machine Animation

Máquinas podem reagir:

```text
no power
→ stopped

partial power
→ slow

full power
→ running
```

---

# 91. ENERGY-89 — Energy UI

Interface pode mostrar:

```text
stored
capacity
generation
consumption
network load
```

---

# 92. ENERGY-90 — Debug

Comandos:

```text
nexora energy inspect
nexora energy network
nexora energy simulate
nexora energy diagnose
nexora energy sources
nexora energy consumers
```

---

# 93. ENERGY-91 — Visualization

Mostrar:

```text
nodes
connections
direction
load
bottlenecks
losses
```

---

# 94. ENERGY-92 — Profiler

Métricas:

```text
networks
nodes
transfers
generation
consumption
rebuilds
simulation time
```

---

# 95. ENERGY-93 — Large Network Testing

Testar:

```text
10 nodes
100
1,000
10,000
100,000 logical nodes
```

---

# 96. ENERGY-94 — Factory Testing

```text
Generator
 ↓
100 Machines
 ↓
Storage
```

---

# 97. ENERGY-95 — City Grid Testing

```text
Power Plant
 ↓
Grid
 ↓
Districts
 ↓
Factories
 ↓
Homes
 ↓
Railway
```

---

# 98. ENERGY-96 — Persistence

Salvar:

```text
stored energy
network topology
machine energy state
```

quando necessário.

---

# 99. ENERGY-97 — Rebuild After Load

```text
Machines
 ↓
discover ports
 ↓
rebuild networks
```

---

# 100. ENERGY-98 — Versioning

```text
EnergyDataVersion
NetworkVersion
```

---

# 101. ENERGY-99 — Migration

```text
Old Network
 ↓
Migration
 ↓
New Network
```

---

# 102. ENERGY-100 — Mod API

Mods podem registrar:

```text
EnergyType
EnergyNode
EnergyPort
EnergyStorage
EnergyGenerator
EnergyConsumer
EnergyConverter
EnergyLossProfile
EnergyNetworkType
```

---

# 103. ENERGY-101 — Official Content

Conteúdo oficial e comunidade usam exatamente a mesma API:

```text
Official Machine
 ↓
Energy API

Community Machine
 ↓
Energy API
```

---

# 104. ENERGY-102 — Data Driven

Grande parte de tipos/configurações pode ser definida por dados:

```text
definition
 ↓
validation
 ↓
registry
 ↓
runtime
```

---

# 105. ENERGY-103 — Capability Composition

Uma máquina pode ser:

```text
Producer
+
Consumer
+
Storage
+
Converter
```

simultaneamente.

---

# 106. ENERGY-104 — Bidirectional Nodes

Suporte a:

```text
INPUT
OUTPUT
BIDIRECTIONAL
```

para baterias/conversores.

---

# 107. ENERGY-105 — Priority Storage

Baterias podem carregar/descarregar conforme prioridades da rede.

---

# 108. ENERGY-106 — Battery Degradation

Futuramente, storage pode ter:

```text
health
degradation
cycles
```

como recurso opcional.

---

# 109. ENERGY-107 — Environmental Integration

Energia pode interagir com:

```text
climate
fluid
geology
space
```

através das respectivas APIs.

---

# 110. ENERGY-108 — Space

No espaço:

```text
Solar
 ↓
Battery
 ↓
Ship
```

e outras fontes podem existir.

---

# 111. ENERGY-109 — Spacecraft Power

Uma nave pode possuir:

```text
power generation
battery
life-support consumer
propulsion
sensors
```

como consumidores diferentes.

---

# 112. ENERGY-110 — Life Support Hook

Energy API fornece:

```text
available power
```

Life Support interpreta.

---

# 113. ENERGY-111 — Railway

Trens podem utilizar:

```text
electrical power
mechanical energy
fuel
```

conforme o modelo do veículo.

---

# 114. ENERGY-112 — Rail Grid

A infraestrutura ferroviária pode possuir:

```text
power substations
powered segments
```

e o sistema Energy fornece energia.

---

# 115. ENERGY-113 — Network LOD

Perto:

```text
FULL
→ node-level
```

Médio:

```text
REGIONAL
→ aggregated
```

Distante:

```text
ABSTRACT
→ generation / consumption statistics
```

---

# 116. ENERGY-114 — Simulation LOD

Uma fábrica distante pode ser:

```text
100 machines
 ↓
aggregate
```

em vez de simular cada máquina individual.

---

# 117. ENERGY-115 — Wake-up

Quando o jogador chega:

```text
REGIONAL
 ↓
FULL
```

---

# 118. ENERGY-116 — Demand Forecast

Sistema pode estimar:

```text
future demand
```

usado por civilizações/automação.

---

# 119. ENERGY-117 — Grid Planning

Civilização pode decidir:

```text
build generator
expand grid
add storage
```

---

# 120. ENERGY-118 — Energy Infrastructure

Civilizações podem construir:

```text
power plants
substations
transmission
storage
```

usando Build/Machines.

---

# 121. ENERGY-119 — Economy Feedback

```text
Energy shortage
 ↓
production falls
 ↓
goods shortage
 ↓
prices change
 ↓
civilization reacts
```

---

# 122. ENERGY-120 — Knowledge

Sociedades podem descobrir:

```text
new generator
new converter
new storage
```

através do Knowledge System.

---

# 123. ENERGY-121 — Technology Progression

Tecnologia pode liberar:

```text
basic generation
 ↓
advanced generation
 ↓
large reactors
 ↓
space power systems
```

---

# 124. ENERGY-122 — Failure Events

Criar eventos:

```text
EnergyShortage
NetworkOverload
GeneratorFailure
StorageEmpty
NetworkDisconnected
BlackoutStarted
BlackoutEnded
```

---

# 125. ENERGY-123 — Maintenance

Máquinas/redes podem exigir:

```text
maintenance
repair
replacement
```

mas essas regras ficam em Machine/Infrastructure.

---

# 126. ENERGY-124 — Diagnostic System

Uma rede deve conseguir explicar:

```text
why machine is offline
```

Exemplo conceitual:

```text
Machine A offline
 ↓
Insufficient power
 ↓
Generator B offline
 ↓
Fuel shortage
 ↓
Supply chain problem
```

Isso é muito útil para automação.

---

# 127. ENERGY-125 — Energy Analyzer

Ferramenta:

```text
nexora energy analyze
```

pode mostrar:

```text
generation
consumption
storage
loss
bottleneck
```

---

# 128. ENERGY-126 — Network Graph Export

Para debugging:

```text
network
 ↓
graph
 ↓
analysis
```

---

# 129. ENERGY-127 — Testing

Testes básicos:

```text
producer
consumer
storage
transfer
loss
overload
priority
disconnect
merge
split
```

---

# 130. ENERGY-128 — Determinism

Com:

```text
same state
+
same tick
+
same energy version
```

resultado deve ser reproduzível.

---

# 131. ENERGY-129 — Performance

O sistema precisa utilizar:

```text
dirty networks
batched transfers
sleeping
parallel processing
LOD
```

---

# 132. ENERGY-130 — Final Architecture

```text
                           ENERGY API
                               │
                  ┌────────────┼────────────┐
                  │            │            │
                TYPES        NODES       NETWORKS
                  │            │            │
            electrical     producer        local
            mechanical     consumer        regional
            thermal        storage         large
            chemical       converter
            magical
                  │            │            │
                  └────────────┼────────────┘
                               │
                       TRANSFER SYSTEM
                               │
                      VALIDATION / TX
                               │
              ┌────────────────┼────────────────┐
              │                │                │
           ENERGY            POWER            HEAT
              │                │                │
          quantity/s        rate/s           thermal
              │                │                │
              └────────────────┼────────────────┘
                               │
                      MACHINE SYSTEM
                               │
        ┌──────────────────────┼───────────────────────┐
        │                      │                       │
     GENERATORS            STORAGE                 CONVERTERS
        │                      │                       │
      fuel                  battery               mechanical
      solar                 capacitor             thermal
      wind                  reactor               magical
      water
        │                      │                       │
        └──────────────────────┼───────────────────────┘
                               │
                       AUTOMATION SYSTEM
                               │
                   ┌───────────┼───────────┐
                   │           │           │
                SENSOR      CONTROL      LOGIC
                   │           │           │
                   └───────────┼───────────┘
                               │
                    INDUSTRY / CIVILIZATION
                               │
                          ECONOMY / TRADE
```

# 133. Ordem de implementação

```text
ENERGY-0 Core
ENERGY-1 Energy Type
ENERGY-2 Energy Amount
ENERGY-3 Power Rate
ENERGY-4 Node
ENERGY-5 Port
ENERGY-6 Producer
ENERGY-7 Consumer
ENERGY-8 Storage
ENERGY-9 Transfer
ENERGY-10 Transaction
ENERGY-11 Network
ENERGY-12 Graph
ENERGY-13 Discovery
ENERGY-14 Merge/Split
ENERGY-15 Loss
ENERGY-16 Priority
ENERGY-17 Load Balancing
ENERGY-18 Overload
ENERGY-19 Batteries
ENERGY-20 Mechanical
ENERGY-21 Thermal
ENERGY-22 Conversion
ENERGY-23 Generators
ENERGY-24 Reactor
ENERGY-25 Cooling Integration
ENERGY-26 Machine Integration
ENERGY-27 Automation Integration
ENERGY-28 Civilization Integration
ENERGY-29 Economy Integration
ENERGY-30 Railway Integration
ENERGY-31 Space Integration
ENERGY-32 LOD
ENERGY-33 Multiplayer
ENERGY-34 Persistence
ENERGY-35 Diagnostics
ENERGY-36 Debugging
ENERGY-37 Mod API
ENERGY-38 Balance/Stress Tests
```

# 134. Primeiro Vertical Slice

Primeiro, algo simples:

```text
Generator
 ↓
Energy Node
 ↓
Cable/Network
 ↓
Battery
 ↓
Machine
```

A máquina:

```text
requests power
 ↓
network checks supply
 ↓
energy transferred
 ↓
machine runs
 ↓
energy consumed
```

Depois:

```text
Generator
 ↓
Energy Network
 ↓
Battery
 ↓
Machine
 ↓
Controller
 ↑
Sensor
```

E então o vertical slice industrial:

```text
RESOURCE
   ↓
PROCESSING
   ↓
FUEL / ENERGY SOURCE
   ↓
GENERATOR
   ↓
ENERGY GRID
   ↓
STORAGE
   ↓
FACTORY
   ↓
AUTOMATION
   ↓
RAILWAY
   ↓
CITY
```

## Regra arquitetural

> **Energy API transporta e gerencia energia. Machine System decide como uma máquina usa energia. Physics decide movimento e forças. Fluid Engine decide fluidos. Economy decide o valor da energia.**

E o ponto mais importante para o NEXORA é deixar a API **agnóstica ao tipo de tecnologia**:

```text
Generator
      ↓
Energy API
      ↓
Machine
```

pode representar:

```text
combustível
solar
vento
água
geotérmica
reator
energia mecânica
energia térmica
energia mágica
energia de tecnologia avançada
```

sem criar um `ElectricalSystem`, `MagicEnergySystem`, `ReactorEnergySystem` etc. completamente independentes.

Assim a mesma infraestrutura pode sustentar desde **uma pequena máquina em uma oficina até uma rede elétrica regional, um reator gigantesco, uma cidade inteira ou o sistema de energia de uma nave espacial**.
