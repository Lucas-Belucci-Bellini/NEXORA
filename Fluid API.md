Sim. E aqui eu faria uma distinção importante em relação ao que já fizemos:

**Water / Fluid Engine** = implementação da simulação dos fluidos.
**Fluid API** = contrato público que permite que **Machines, Blocks, Tanks, Pipes, Vehicles, Mods, Physics e Automation** utilizem fluidos sem conhecer a implementação interna.

A Fluid API precisa ser bem detalhada justamente para impedir que cada sistema crie seu próprio “mini sistema de fluidos”.

# NEXORA — FLUID API MASTER PLAN

```text id="f7k2m9"
                         FLUID API
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
      DEFINITIONS          STATE            ACCESS
          │                 │                 │
      FluidType         FluidStack        Reader
      Properties        Amount            Writer
      Tags              Temperature       Query
      Phases            Pressure          Snapshot
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
             ┌──────────────┼──────────────┐
             │              │              │
         CONTAINER        PORT           NETWORK
             │              │              │
          Tank          Input/Output      Pipe
          Battery       Connection        Routing
          Reservoir     Filter            Graph
             │              │              │
             └──────────────┼──────────────┘
                            │
             ┌──────────────┼───────────────┐
             │              │               │
          MACHINE         PHYSICS        RENDERER
             │              │               │
         processing      buoyancy         surface
         cooling         drag             waves
         heating         pressure         effects
             │
             └──────────────┬──────────────┐
                            │              │
                         MOD API        AUTOMATION
                            │              │
                         custom         sensors
                         fluids         controllers
                         rules          routing
```

---

# 1. Objetivo da API

A API deve permitir que qualquer sistema diga:

```text
"preciso de 500 unidades de água"
```

ou:

```text
"eu forneço um fluido compatível com #coolant"
```

sem saber como o Fluid Engine calcula fluxo, pressão ou armazenamento internamente.

A regra:

> **A API define o contrato. O Fluid Engine implementa o comportamento.**

---

# 2. FLUID-API-0 — Fluid ID

Todo fluido possui um ID global:

```text
nexora:water
nexora:lava
nexora:steam
```

Mods:

```text
example:coolant
example:bio_fluid
```

Usar namespace obrigatório.

---

# 3. FLUID-API-1 — Fluid Definition

Contrato:

```text
FluidDefinition
├── id
├── displayName
├── tags
├── phase
├── density
├── viscosity
├── temperatureProfile
├── pressureProfile
├── visualProfile
└── physicsProfile
```

---

# 4. FLUID-API-2 — Fluid Phase

Fases:

```text
SOLID
LIQUID
GAS
PLASMA
CUSTOM
```

Não obrigar todos os fluidos a usar todas.

---

# 5. FLUID-API-3 — Fluid Tags

Em vez de fazer:

```text
if fluid == "water"
```

usar:

```text
#water
#liquid
#coolant
#drinkable
#industrial
```

Isso deixa a API extensível.

---

# 6. FLUID-API-4 — Fluid Properties

Propriedades genéricas:

```text
density
viscosity
specificHeat
thermalConductivity
flammability
corrosiveness
opacity
```

A API deve permitir propriedades adicionais.

---

# 7. FLUID-API-5 — Fluid Stack

A principal unidade de transporte:

```text
FluidStack
├── fluid
├── amount
├── temperature
├── pressure
└── metadata
```

Isso é semelhante conceitualmente a um `ItemStack`, mas para fluidos.

---

# 8. FLUID-API-6 — Amount

Quantidade deve possuir uma unidade lógica consistente.

```text
FluidAmount
```

Não deixar:

```text
machine A = litros
machine B = buckets
machine C = unidades
```

sem uma conversão padronizada.

---

# 9. FLUID-API-7 — Unit Conversion

Criar:

```text
FluidUnit
FluidConverter
```

para representar unidades diferentes quando necessário.

---

# 10. FLUID-API-8 — Temperature

`FluidStack` pode possuir temperatura.

```text
fluid.temperature
```

Mas a definição também pode especificar limites:

```text
minimum
maximum
preferred
```

---

# 11. FLUID-API-9 — Pressure

Suporte a:

```text
pressure
```

e limites:

```text
minimumPressure
maximumPressure
```

---

# 12. FLUID-API-10 — Fluid Container

Contrato:

```text
IFluidContainer
```

operações:

```text
fill()
drain()
getAmount()
getCapacity()
getFluid()
```

---

# 13. FLUID-API-11 — Fill Result

`fill()` deve retornar resultado estruturado:

```text
FluidFillResult
├── accepted
├── amount
└── remainder
```

---

# 14. FLUID-API-12 — Drain Result

```text
FluidDrainResult
├── removed
├── stack
└── remainder
```

---

# 15. FLUID-API-13 — Simulation Mode

Operações podem ser simuladas:

```text
simulateFill()
simulateDrain()
```

para perguntar:

> “Quanto caberia?”

sem modificar o estado.

Isso será extremamente importante para automação.

---

# 16. FLUID-API-14 — Atomic Transfer

Criar:

```text
FluidTransfer
```

com:

```text
source
destination
amount
transactionId
```

---

# 17. FLUID-API-15 — Transfer Transaction

Fluxo:

```text
BEGIN
 ↓
VALIDATE
 ↓
RESERVE
 ↓
TRANSFER
 ↓
COMMIT
```

---

# 18. FLUID-API-16 — Rollback

Se o destino falhar:

```text
ROLLBACK
```

e o fluido não deve desaparecer.

---

# 19. FLUID-API-17 — Fluid Port

Máquinas precisam de portas:

```text
IFluidPort
```

com:

```text
INPUT
OUTPUT
BIDIRECTIONAL
```

---

# 20. FLUID-API-18 — Port Capacity

Cada port pode informar:

```text
capacity
flowRate
pressureLimit
temperatureLimit
```

---

# 21. FLUID-API-19 — Port Filters

Portas podem aceitar:

```text
fluid ID
fluid tag
phase
temperature range
```

---

# 22. FLUID-API-20 — Fluid Handler

Criar uma interface generalizada:

```text
IFluidHandler
```

capaz de:

```text
fill
drain
query
simulate
```

Assim uma máquina não precisa saber se está falando com:

```text
Tank
Pipe
Machine
Reservoir
Vehicle
```

---

# 23. FLUID-API-21 — Tank API

```text
ITank
```

operações:

```text
fill
drain
capacity
contents
temperature
pressure
```

---

# 24. FLUID-API-22 — Multi-Tank

Uma máquina pode possuir:

```text
Tank 0
Tank 1
Tank 2
```

cada um com filtros diferentes.

---

# 25. FLUID-API-23 — Tank Roles

Tanks podem ser:

```text
INPUT
OUTPUT
BUFFER
COOLANT
WASTE
MIXER
```

---

# 26. FLUID-API-24 — Pipe API

```text
IPipe
```

com:

```text
capacity
flow
pressure
connections
filters
```

---

# 27. FLUID-API-25 — Pipe Connections

Direções:

```text
NORTH
SOUTH
EAST
WEST
UP
DOWN
```

---

# 28. FLUID-API-26 — Connection State

Cada conexão:

```text
CONNECTED
DISCONNECTED
BLOCKED
FILTERED
```

---

# 29. FLUID-API-27 — Fluid Network

```text
IFluidNetwork
```

fornece:

```text
nodes
ports
connections
throughput
pressure
```

---

# 30. FLUID-API-28 — Network Node

```text
IFluidNode
```

com:

```text
nodeId
ports
position
capabilities
```

---

# 31. FLUID-API-29 — Network Graph

```text
Node
 +
Edge
 →
Fluid Network
```

isso serve para:

```text
routing
diagnostics
topology
```

---

# 32. FLUID-API-30 — Network Join

Ao conectar dois componentes:

```text
A + B
 ↓
Network
```

---

# 33. FLUID-API-31 — Network Split

Quando desconecta:

```text
Network
 ↓
A + B
```

---

# 34. FLUID-API-32 — Network Merge

Duas redes podem virar uma:

```text
Network A
+
Network B
 ↓
Network C
```

---

# 35. FLUID-API-33 — Network Rebuild

Somente a parte afetada deve ser recalculada.

---

# 36. FLUID-API-34 — Routing

API:

```text
findRoute(source, destination)
```

ou equivalente.

Critérios:

```text
capacity
pressure
priority
distance
loss
```

---

# 37. FLUID-API-35 — Transfer Rate

Uma conexão pode declarar:

```text
maxTransferRate
```

---

# 38. FLUID-API-36 — Pressure Constraints

Uma rota pode exigir:

```text
pressure >= X
```

---

# 39. FLUID-API-37 — Flow Reservation

Uma máquina pode reservar capacidade:

```text
reserveFlow(amount)
```

para garantir o fornecimento.

---

# 40. FLUID-API-38 — Priority

Transferências podem possuir:

```text
CRITICAL
HIGH
NORMAL
LOW
```

---

# 41. FLUID-API-39 — Network Scheduling

O Fluid Engine decide quando atualizar.

A API apenas permite:

```text
requestUpdate()
```

ou mecanismo equivalente.

---

# 42. FLUID-API-40 — Query API

Criar:

```text
IFluidQuery
```

com operações:

```text
getFluidAt()
getAmountAt()
getTemperatureAt()
getPressureAt()
getVelocityAt()
```

---

# 43. FLUID-API-41 — Region Query

```text
queryRegion(bounds)
```

para áreas maiores.

---

# 44. FLUID-API-42 — Fluid Source

```text
IFluidSource
```

pode representar:

```text
spring
reservoir
machine
pipe
rain
```

---

# 45. FLUID-API-43 — Fluid Sink

```text
IFluidSink
```

pode representar:

```text
drain
machine
tank
environment
```

---

# 46. FLUID-API-44 — Source Rate

Fontes podem declarar:

```text
generationRate
```

---

# 47. FLUID-API-45 — Sink Rate

Drenos podem declarar:

```text
consumptionRate
```

---

# 48. FLUID-API-46 — Infinite Sources

Uma fonte pode ser:

```text
FINITE
REGENERATING
INFINITE
```

Isso deve ser propriedade explícita.

---

# 49. FLUID-API-47 — Finite Reservoir

Reservatório:

```text
capacity
currentAmount
rechargeRate
```

---

# 50. FLUID-API-48 — Flow State

API deve expor:

```text
FluidFlowState
├── amount
├── direction
├── velocity
└── rate
```

---

# 51. FLUID-API-49 — Fluid Velocity

```text
Vector3
```

para fluxos físicos.

---

# 52. FLUID-API-50 — Fluid Volume Query

Physics pode perguntar:

```text
how much fluid occupies this volume?
```

---

# 53. FLUID-API-51 — Submersion Query

```text
getSubmersion(body)
```

para natação/buoyancy.

---

# 54. FLUID-API-52 — Density Query

```text
getDensity(position)
```

---

# 55. FLUID-API-53 — Physics Profile

A API pode expor:

```text
FluidPhysicsProfile
├── density
├── viscosity
├── drag
└── buoyancy
```

---

# 56. FLUID-API-54 — Renderer Profile

Separar:

```text
FluidRenderProfile
```

com:

```text
surface
opacity
color
emission
refraction
foam
```

---

# 57. FLUID-API-55 — Visual State

Renderer pode consultar:

```text
getRenderState()
```

sem ler dados internos da simulação.

---

# 58. FLUID-API-56 — Surface Data

Para água:

```text
FluidSurface
├── height
├── normal
├── flowDirection
└── waveData
```

---

# 59. FLUID-API-57 — Phase Transition

Criar:

```text
IFluidPhaseTransition
```

com:

```text
source
target
conditions
```

---

# 60. FLUID-API-58 — Temperature Transition

Exemplo:

```text
water
→ ice
```

ou:

```text
water
→ steam
```

dependendo da definição.

---

# 61. FLUID-API-59 — Fluid Transformation

Criar:

```text
FluidTransformation
```

que não é necessariamente mudança de fase.

```text
A
 ↓
Process
 ↓
B
```

---

# 62. FLUID-API-60 — Fluid Reaction

```text
FluidReaction
├── inputs
├── conditions
├── outputs
└── byproducts
```

---

# 63. FLUID-API-61 — Mixing

Criar:

```text
FluidMixRule
```

que determina:

```text
A + B
→ C
```

---

# 64. FLUID-API-62 — Fluid Mixture

Em sistemas avançados:

```text
FluidMixture
├── components
├── ratios
└── properties
```

---

# 65. FLUID-API-63 — Composition

Um fluido pode possuir composição:

```text
component A = 70%
component B = 30%
```

---

# 66. FLUID-API-64 — Contamination

Criar metadata:

```text
contaminants
```

sem obrigar todo fluido a usar isso.

---

# 67. FLUID-API-65 — Purity

```text
purity
```

pode ser relevante para processamento.

---

# 68. FLUID-API-66 — Quality

Fluido pode possuir:

```text
quality
grade
purity
```

para sistemas industriais.

---

# 69. FLUID-API-67 — Custom Data

Permitir dados extras por fluido:

```text
metadata
```

mas com limite e versionamento.

---

# 70. FLUID-API-68 — Capability

Um fluido pode oferecer capacidades:

```text
Coolant
Fuel
Lubricant
Chemical
Magical
```

---

# 71. FLUID-API-69 — Fluid Capability Tags

Em vez de:

```text
isCoolant()
```

usar:

```text
#coolant
```

e parâmetros.

---

# 72. FLUID-API-70 — Machine Integration

Máquinas usam:

```text
IFluidHandler
```

e:

```text
IFluidPort
```

---

# 73. FLUID-API-71 — Crafting Integration

Receita pode exigir:

```text
fluid input
fluid output
```

---

# 74. FLUID-API-72 — Processing Integration

Processo pode definir:

```text
fluid input
fluid output
temperature
pressure
```

---

# 75. FLUID-API-73 — Energy Integration

Turbinas e máquinas podem converter:

```text
fluid flow
 ↓
mechanical energy
```

---

# 76. FLUID-API-74 — Thermal Integration

Máquina pode dizer:

```text
accept coolant
```

e especificar:

```text
temperature range
flow rate
```

---

# 77. FLUID-API-75 — Pressure Integration

Uma máquina pode exigir:

```text
minimum pressure
```

---

# 78. FLUID-API-76 — Automation Integration

Automação pode consultar:

```text
tank level
pressure
temperature
flow
```

---

# 79. FLUID-API-77 — Sensor API

Criar:

```text
IFluidSensor
```

operações:

```text
measureAmount
measurePressure
measureTemperature
measureFlow
```

---

# 80. FLUID-API-78 — Controller Integration

Controlador pode decidir:

```text
tank full
→ close valve
```

---

# 81. FLUID-API-79 — Valve API

```text
IFluidValve
```

estados:

```text
OPEN
CLOSED
PARTIAL
```

---

# 82. FLUID-API-80 — Pump API

```text
IFluidPump
```

com:

```text
flowRate
pressure
powerRequired
```

---

# 83. FLUID-API-81 — Reservoir API

```text
IFluidReservoir
```

para:

```text
lake
tank
ocean segment
aquifer
storage structure
```

---

# 84. FLUID-API-82 — Natural Water

A Hydrology System pode usar a API para criar:

```text
river
lake
ocean
aquifer
spring
```

---

# 85. FLUID-API-83 — Climate Integration

Climate pode injetar:

```text
rain
snowmelt
evaporation
```

através da API.

---

# 86. FLUID-API-84 — Water Cycle Integration

```text
Climate
 ↓
Fluid API
 ↓
Water
 ↓
Hydrology
```

---

# 87. FLUID-API-85 — Underground Integration

Cave/Deep World pode criar:

```text
aquifer
underground river
underground lake
```

através da API.

---

# 88. FLUID-API-86 — Ocean Integration

Ocean System pode expor:

```text
current
temperature
pressure
salinity-like properties
```

pela API.

---

# 89. FLUID-API-87 — Biome Integration

Biome pode definir compatibilidade:

```text
water availability
```

mas não deve possuir uma implementação paralela de água.

---

# 90. FLUID-API-88 — Vegetation Integration

Vegetation consulta:

```text
soil moisture
water source
fluid availability
```

---

# 91. FLUID-API-89 — Agriculture Integration

```text
crop
 ↓
fluid query
 ↓
irrigation
```

---

# 92. FLUID-API-90 — Civilization Integration

Civilizações podem usar:

```text
reservoir
pump
pipe
water network
```

---

# 93. FLUID-API-91 — Water Infrastructure

```text
Source
 ↓
Pump
 ↓
Pipe
 ↓
Reservoir
 ↓
City
```

---

# 94. FLUID-API-92 — Economy Integration

Água/combustível/fluidos industriais podem ser economicamente relevantes.

A Economy System deve consultar:

```text
production
availability
consumption
```

---

# 95. FLUID-API-93 — Logistics

Fluidos podem ser transportados por:

```text
pipe
tank
vehicle
ship
container
```

---

# 96. FLUID-API-94 — Fluid Cargo

Criar:

```text
FluidCargo
```

para transporte em veículos.

---

# 97. FLUID-API-95 — Vehicle Integration

Navios/veículos podem transportar:

```text
FluidContainer
```

---

# 98. FLUID-API-96 — Spacecraft

Naves podem usar fluidos para:

```text
cooling
propellant
life support
industrial processing
```

através da mesma API.

---

# 99. FLUID-API-97 — Mod Registration

Um mod deve poder:

```text
registerFluid()
registerTag()
registerReaction()
registerContainer()
registerPort()
registerTransferRule()
registerRenderProfile()
registerPhysicsProfile()
```

---

# 100. FLUID-API-98 — Data Driven

Grande parte da definição pode vir de dados:

```text
fluid
 ↓
validate
 ↓
registry
 ↓
runtime
```

---

# 101. FLUID-API-99 — API Version

Criar:

```text
FluidApiVersion
```

para compatibilidade entre mods.

---

# 102. FLUID-API-100 — Capability Compatibility

Um mod pode declarar:

```text
requires:
fluid-api >= X
```

---

# 103. FLUID-API-101 — Feature Detection

Mods devem conseguir perguntar:

```text
supports(feature)
```

em vez de assumir que toda instalação possui tudo.

---

# 104. FLUID-API-102 — Optional Features

Exemplo:

```text
basic fluid
advanced pressure
advanced chemistry
```

---

# 105. FLUID-API-103 — Permissions

Nem todo mod deve poder modificar qualquer fluido do mundo sem restrição.

Criar permissões:

```text
fluid.read
fluid.write
fluid.network
fluid.register
fluid.debug
```

---

# 106. FLUID-API-104 — Multiplayer

Fluxo:

```text
Client
 ↓
Fluid Request
 ↓
Server
 ↓
Fluid API
 ↓
Fluid Engine
 ↓
Result
 ↓
Replication
```

---

# 107. FLUID-API-105 — Server Authority

O cliente não pode dizer:

```text
"meu tanque recebeu 10.000 água"
```

sem validação.

---

# 108. FLUID-API-106 — Network Delta

Replicar somente mudanças importantes:

```text
tank amount changed
network topology changed
pressure changed
```

conforme necessidade.

---

# 109. FLUID-API-107 — Snapshot

Criar:

```text
FluidSnapshot
```

para:

```text
Renderer
Physics
Network
AI
```

---

# 110. FLUID-API-108 — Read-Only Access

Alguns sistemas devem receber:

```text
IFluidReader
```

sem acesso de escrita.

---

# 111. FLUID-API-109 — Write Access

Sistemas autorizados usam:

```text
IFluidWriter
```

---

# 112. FLUID-API-110 — Transaction Context

Toda alteração persistente deve possuir:

```text
FluidTransactionContext
```

---

# 113. FLUID-API-111 — Error Model

Erros padronizados:

```text
FLUID_NOT_FOUND
INVALID_AMOUNT
CAPACITY_EXCEEDED
PRESSURE_LIMIT
TEMPERATURE_LIMIT
INCOMPATIBLE_FLUID
NETWORK_UNAVAILABLE
PERMISSION_DENIED
TRANSACTION_FAILED
```

---

# 114. FLUID-API-112 — Result Objects

Evitar simplesmente:

```text
true / false
```

Retornar objetos diagnósticos:

```text
FluidOperationResult
├── success
├── amount
├── reason
└── diagnostics
```

---

# 115. FLUID-API-113 — Events

Eventos:

```text
FluidInserted
FluidRemoved
FluidTransferred
FluidMixed
FluidTransformed
FluidPressureChanged
FluidTemperatureChanged
FluidNetworkChanged
```

---

# 116. FLUID-API-114 — Event Bus

```text
Fluid Event
 ↓
Event Bus
 ├── Machine
 ├── Automation
 ├── Economy
 ├── Ecology
 └── Debug
```

---

# 117. FLUID-API-115 — Query Permissions

Algumas APIs podem ser:

```text
public read
restricted read
write
admin
```

conforme contexto.

---

# 118. FLUID-API-116 — Persistence

A API deve fornecer contratos para serialização:

```text
IFluidSerializable
```

---

# 119. FLUID-API-117 — Versioned Data

```text
FluidDataVersion
```

---

# 120. FLUID-API-118 — Migration

Mods/saves podem migrar:

```text
Old Fluid ID
 ↓
Migration
 ↓
New Fluid ID
```

---

# 121. FLUID-API-119 — Missing Fluid Handling

Se um mod que criou um fluido desaparecer:

```text
missing fluid
```

não deve necessariamente destruir o save.

Usar:

```text
MissingFluidPlaceholder
```

ou mecanismo equivalente.

---

# 122. FLUID-API-120 — Compatibility

Um save pode registrar:

```text
required fluid definitions
```

para alertar se algo estiver faltando.

---

# 123. FLUID-API-121 — Debug

Comandos:

```text
nexora fluid inspect
nexora fluid query
nexora fluid network
nexora fluid tank
nexora fluid transfer
nexora fluid simulate
```

---

# 124. FLUID-API-122 — Network Diagnostics

Mostrar:

```text
source
destination
flow
pressure
capacity
bottleneck
```

---

# 125. FLUID-API-123 — Simulation Tool

Uma ferramenta pode perguntar:

```text
"quanto de água chegaria nesse tanque?"
```

sem executar a transferência.

---

# 126. FLUID-API-124 — Testing

Testes:

```text
register fluid
fill
drain
transfer
filter
pressure
temperature
network
merge
split
```

---

# 127. FLUID-API-125 — Compatibility Tests

Testar:

```text
Machine ↔ Tank
Tank ↔ Pipe
Pipe ↔ Machine
Pipe ↔ Pipe
Vehicle ↔ Tank
Recipe ↔ Fluid
Automation ↔ Fluid
```

---

# 128. FLUID-API-126 — Performance

A API não deve exigir:

```text
object allocation
```

em cada operação.

Usar estruturas eficientes e resultados reutilizáveis quando apropriado.

---

# 129. FLUID-API-127 — Allocation Rules

Operações de consulta muito frequentes devem ser:

```text
low allocation
batch friendly
```

---

# 130. FLUID-API-128 — Batch Operations

Suportar:

```text
fillMany
drainMany
queryMany
transferBatch
```

quando fizer sentido.

---

# 131. FLUID-API-129 — Batch Transaction

```text
BEGIN
 ↓
multiple transfers
 ↓
VALIDATE
 ↓
COMMIT
```

---

# 132. FLUID-API-130 — Automation Safety

Automação não deve conseguir gerar:

```text
infinite transfer loops
```

---

# 133. FLUID-API-131 — Loop Detection

Detectar redes:

```text
A
 ↓
B
 ↓
C
 ↓
A
```

quando estiverem gerando operações infinitas.

---

# 134. FLUID-API-132 — Rate Limiting

Nós podem limitar operações:

```text
transfers/sec
queries/tick
automation operations/tick
```

---

# 135. FLUID-API-133 — Network LOD

A API deve permitir representar redes distantes de forma agregada:

```text
RegionalFluidNetworkState
```

---

# 136. FLUID-API-134 — Regional State

Distante:

```text
total storage
production rate
consumption rate
```

em vez de cada pipe.

---

# 137. FLUID-API-135 — Rehydration

Quando a área volta para FULL:

```text
regional
 ↓
instantiate local network
```

---

# 138. FLUID-API-136 — WorldGen

WorldGen pode usar:

```text
IFluidWorldWriter
```

para criar:

```text
river
lake
ocean
aquifer
```

---

# 139. FLUID-API-137 — Hydrology

Hydrology usa a API para atualizar:

```text
surface
groundwater
rivers
```

---

# 140. FLUID-API-138 — Climate

Climate injeta:

```text
precipitation
evaporation
```

sem acessar o armazenamento interno.

---

# 141. FLUID-API-139 — Physics

Physics usa:

```text
FluidQuery
```

para:

```text
buoyancy
drag
pressure
```

---

# 142. FLUID-API-140 — Renderer

Renderer recebe:

```text
FluidRenderSnapshot
```

---

# 143. FLUID-API-141 — Vegetation

Vegetation consulta:

```text
water availability
soil moisture
```

---

# 144. FLUID-API-142 — Civilization

Civilization consulta:

```text
water infrastructure
resource availability
```

---

# 145. FLUID-API-143 — Economy

Economy consulta:

```text
resource production
consumption
shortage
```

---

# 146. FLUID-API-144 — Machines

Machines implementam:

```text
IFluidHandler
IFluidPort
```

e não precisam conhecer o Fluid Engine.

---

# 147. FLUID-API-145 — Crafting

Crafting usa:

```text
FluidIngredient
FluidOutput
```

---

# 148. FLUID-API-146 — Tool API

Ferramentas podem interagir:

```text
Fluid Tool
 ↓
Fluid API
```

Exemplo conceitual:

```text
pump tool
container
fluid scanner
```

---

# 149. FLUID-API-147 — Vehicle

Veículos podem ter:

```text
fuel tank
coolant tank
water tank
cargo tank
```

todos através da API.

---

# 150. FLUID-API-148 — Space

Naves podem usar:

```text
propellant
coolant
life support fluid
industrial fluid
```

---

# 151. FLUID-API-149 — Mod Example

Um mod hipotético poderia registrar:

```text
example:coolant
```

com:

```text
tag = #coolant
density = ...
thermal profile = ...
```

e imediatamente uma máquina compatível com `#coolant` poderia utilizar o fluido.

Esse é justamente o comportamento que queremos.

---

# 152. FLUID-API-150 — Final Contract

A API pública final teria aproximadamente:

```text
FluidAPI
│
├── FluidRegistry
├── FluidDefinition
├── FluidStack
├── FluidType
├── FluidTags
│
├── IFluidReader
├── IFluidWriter
├── IFluidHandler
├── IFluidContainer
├── IFluidPort
├── IFluidTank
│
├── IFluidSource
├── IFluidSink
├── IFluidNetwork
├── IFluidNode
├── IFluidQuery
│
├── FluidTransfer
├── FluidTransaction
├── FluidOperationResult
│
├── FluidTransformation
├── FluidReaction
├── FluidMixRule
│
├── FluidPhysicsProfile
├── FluidRenderProfile
│
├── FluidSnapshot
└── FluidEvents
```

---

# 153. Estrutura de projeto

Eu deixaria:

```text
api/
└── fluid/
    ├── FluidAPI
    ├── FluidRegistry
    ├── FluidDefinition
    ├── FluidStack
    ├── FluidTags
    │
    ├── container/
    │   ├── IFluidContainer
    │   ├── IFluidTank
    │   └── FluidOperationResult
    │
    ├── port/
    │   ├── IFluidPort
    │   └── FluidPortConfig
    │
    ├── network/
    │   ├── IFluidNetwork
    │   ├── IFluidNode
    │   └── IFluidConnection
    │
    ├── transfer/
    │   ├── FluidTransfer
    │   └── FluidTransaction
    │
    ├── query/
    │   ├── IFluidQuery
    │   └── FluidSnapshot
    │
    ├── processing/
    │   ├── FluidTransformation
    │   ├── FluidReaction
    │   └── FluidMixRule
    │
    ├── physics/
    ├── rendering/
    ├── events/
    ├── serialization/
    └── permissions/
```

---

# 154. Ordem de implementação

```text
FLUID-API-0 Registry
FLUID-API-1 Definition
FLUID-API-2 IDs/Tags
FLUID-API-3 FluidStack
FLUID-API-4 Amount
FLUID-API-5 Container
FLUID-API-6 Fill/Drain
FLUID-API-7 Simulation
FLUID-API-8 Transaction
FLUID-API-9 Ports
FLUID-API-10 Tanks
FLUID-API-11 Pipes
FLUID-API-12 Nodes
FLUID-API-13 Networks
FLUID-API-14 Connections
FLUID-API-15 Routing
FLUID-API-16 Sources
FLUID-API-17 Sinks
FLUID-API-18 Queries
FLUID-API-19 Physics Profile
FLUID-API-20 Render Profile
FLUID-API-21 Temperature
FLUID-API-22 Pressure
FLUID-API-23 Transformations
FLUID-API-24 Reactions
FLUID-API-25 Mixing
FLUID-API-26 Machine Integration
FLUID-API-27 Crafting Integration
FLUID-API-28 Energy Integration
FLUID-API-29 Automation
FLUID-API-30 Climate/Hydrology
FLUID-API-31 Vehicle
FLUID-API-32 Space
FLUID-API-33 Multiplayer
FLUID-API-34 Persistence
FLUID-API-35 Versioning
FLUID-API-36 Permissions
FLUID-API-37 Events
FLUID-API-38 Debugging
FLUID-API-39 Mod API
FLUID-API-40 LOD
FLUID-API-41 Performance
FLUID-API-42 Testing
```

# 155. Primeiro Vertical Slice

O primeiro teste da API seria:

```text
Fluid Registry
 ↓
register water
 ↓
FluidStack
 ↓
Tank
 ↓
fill()
 ↓
Pipe
 ↓
FluidPort
 ↓
Machine
 ↓
drain()
 ↓
Transfer Transaction
 ↓
commit
```

Depois:

```text
Water
 ↓
Pump
 ↓
Pipe Network
 ↓
Tank
 ↓
Machine
 ↓
Fluid consumed
```

Depois integração real:

```text
Rain
 ↓
Climate
 ↓
Fluid API
 ↓
River
 ↓
Pump
 ↓
Reservoir
 ↓
City
 ↓
Agriculture
 ↓
Economy
```

E o teste industrial:

```text
Resource
 ↓
Processing
 ↓
Fluid Output
 ↓
Tank
 ↓
Pipe Network
 ↓
Cooling Machine
 ↓
Heat Exchange
 ↓
Energy Production
```

# 156. Regra arquitetural final

Eu colocaria esta regra no contrato do NEXORA:

> **Nenhum sistema externo ao Fluid Engine deve acessar diretamente o armazenamento interno de fluidos.**

Ele sempre passa por:

```text
IFluidReader
IFluidWriter
IFluidHandler
IFluidQuery
IFluidNetwork
```

Então teremos:

```text
WORLD
  ↓
Fluid Engine
  ↓
Fluid API
  ├── Machines
  ├── Crafting
  ├── Energy
  ├── Physics
  ├── Renderer
  ├── Climate
  ├── Hydrology
  ├── Vegetation
  ├── Agriculture
  ├── Civilization
  ├── Vehicles
  ├── Space
  └── Mods
```

A diferença para o **Water / Fluid Engine** que definimos antes é fundamental:

```text
Fluid Engine
= "como o fluido se comporta"

Fluid API
= "como o resto do NEXORA conversa com o fluido"
```

Assim um mod poderá criar um fluido completamente novo e uma máquina de outro mod poderá aceitá-lo apenas por `tags/capabilities`, sem nenhum dos dois precisar conhecer a implementação interna do outro.
