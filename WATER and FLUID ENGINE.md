Sim. **Água / Fluidos** precisa ser um sistema grande no NEXORA, e eu não trataria “água” como um bloco com uma animação. O ideal é criar um **Fluid Engine** genérico capaz de representar água, lava, combustíveis, fluidos industriais, químicos, gases e fluidos especiais.

A separação seria:

```text id="f8q2mc"
                 NEXORA FLUID ENGINE
                         │
          ┌──────────────┼──────────────┐
          │              │              │
       DEFINITION     SIMULATION      STORAGE
          │              │              │
       FluidType       Flow             Voxels
       Properties      Pressure         Networks
       Temperature     Sources          Containers
          │              │              │
          └──────────────┼──────────────┘
                         │
        ┌────────────────┼─────────────────┐
        │                │                 │
     WORLD            PHYSICS          RENDERER
        │                │                 │
    lakes/rivers      buoyancy          surface
    oceans            drag              transparency
    caves             pressure          refraction
        │
      GAMEPLAY
        │
  machines/farming/
  chemistry/economy
```

# NEXORA — WATER / FLUID ENGINE MASTER PLAN

## 1. Objetivo

O sistema deve representar:

```text id="q8jd1m"
água
lava
gases
combustíveis
óleos
fluidos industriais
fluidos mágicos
fluidos alienígenas
```

sem criar um sistema diferente para cada um.

A regra é:

> **Água é um tipo de fluido. O motor de fluidos é a infraestrutura.**

---

# 2. FLUID-0 — Fluid Definition

Criar:

```text id="p4z61t"
FluidDefinition
├── id
├── density
├── viscosity
├── temperatureRange
├── pressureBehavior
├── color
├── opacity
├── flowProperties
├── thermalProperties
└── specialProperties
```

Exemplo conceitual:

```text id="8v3t0d"
nexora:water
nexora:lava
nexora:oil
```

---

# 3. FLUID-1 — Fluid State

O tipo de fluido é uma definição.

O estado é:

```text id="m2b4vl"
FluidState
├── type
├── amount
├── level
├── temperature
├── pressure
└── velocity
```

Assim o mesmo `water` pode estar:

```text id="a9nv31"
calm
flowing
hot
cold
pressurized
contaminated
```

---

# 4. FLUID-2 — Voxel Integration

Como o mundo é voxel, o fluido pode ocupar células.

```text id="gn9s5v"
Voxel
├── BlockState
└── FluidState
```

Mas nem todo voxel precisa armazenar um FluidState completo.

O sistema pode usar armazenamento compacto:

```text id="s0r1bz"
EMPTY
FULL
PARTIAL
```

com dados complementares apenas quando necessários.

---

# 5. FLUID-3 — Fluid Volume

Uma unidade básica:

```text id="5h9s2n"
FluidVolume
```

com:

```text id="6m4g53"
amount
level
density
```

Isso permite representar volumes parciais.

---

# 6. FLUID-4 — Fluid Level

A água não precisa ser binária:

```text id="c0uw4h"
tem água
não tem água
```

Pode possuir:

```text id="2h5jyi"
0
1
2
...
N
```

níveis de preenchimento.

---

# 7. FLUID-5 — Source Blocks

Distinguir:

```text id="5r0e7u"
SOURCE
FLOWING
```

Uma fonte pode alimentar uma região.

```text id="ec38j1"
source
 ↓
flow
 ↓
neighbor
 ↓
flow
```

---

# 8. FLUID-6 — Flow Solver

Fluxo básico:

```text id="n7ka4x"
Fluid
 ↓
neighbors
 ↓
available space
 ↓
pressure / gravity
 ↓
new state
```

Mas precisamos evitar recalcular o mundo inteiro.

---

# 9. FLUID-7 — Flow Queue

Criar filas:

```text id="b7j3yq"
FlowQueue
├── add
├── remove
└── priority
```

Quando algo muda:

```text id="vq1yk6"
voxel changed
 ↓
enqueue affected cells
 ↓
simulate only affected region
```

---

# 10. FLUID-8 — Gravity

O fluxo deve ter uma diretriz vertical.

```text id="ql1o8z"
gravity
 ↓
downward flow
```

Mas o sistema deve aceitar outras forças.

---

# 11. FLUID-9 — Pressure

Criar:

```text id="r5d3lp"
PressureField
```

A pressão pode depender de:

```text id="0ny8st"
fluid column
depth
density
container
source
dimension
```

Isso é especialmente importante no oceano e no Deep World.

---

# 12. FLUID-10 — Pressure Gradient

O fluido pode se mover devido a diferenças:

```text id="56c6us"
high pressure
 ↓
low pressure
```

Isso abre espaço para hidráulica mais sofisticada.

---

# 13. FLUID-11 — Hydrostatic Pressure

Água profunda:

```text id="e8r4ny"
surface
 ↓
depth
 ↓
pressure
```

Outros sistemas podem consultar:

```text id="s1o9yd"
getPressure(position)
```

---

# 14. FLUID-12 — Temperature

Cada fluido pode possuir:

```text id="7q3i8v"
temperature
```

E o mundo pode possuir:

```text id="4nmw6p"
environment temperature
```

Então:

```text id="j6zq9c"
fluid
+
environment
=
new temperature
```

---

# 15. FLUID-13 — Heat Transfer

Criar integração com um sistema térmico:

```text id="i4dj98"
Fluid
 ↔
Block
 ↔
Machine
 ↔
Environment
```

Exemplo:

```text id="m5c8s2"
hot fluid
 ↓
pipe
 ↓
machine
 ↓
heat exchange
```

---

# 16. FLUID-14 — Phase Changes

Arquitetura preparada para:

```text id="r1vn47"
liquid
 ↔
gas
```

e futuramente:

```text id="7vsm8e"
solid
 ↔
liquid
 ↔
gas
```

Por exemplo, água poderia eventualmente:

```text id="gxqcm2"
freeze
melt
evaporate
condense
```

---

# 17. FLUID-15 — Fluid Transformation

Criar:

```text id="k4gy8x"
FluidTransformation
```

Exemplo conceitual:

```text id="6k1r2w"
water
 ↓ temperature
steam
```

ou processos industriais.

---

# 18. FLUID-16 — Mixing

Fluidos podem interagir.

```text id="8i4a8t"
Fluid A
+
Fluid B
 ↓
Mixture
```

Isso não precisa significar que todos os fluidos podem misturar.

Cada definição pode declarar:

```text id="lx92ji"
canMix
mixRule
result
```

---

# 19. FLUID-17 — Reaction System

Separar reação química da física de fluxo.

```text id="p4qz5u"
Fluid Engine
 ↓
Reaction API
```

Aí um módulo de química pode implementar:

```text id="4umg7d"
A + B
→ C
```

Sem colocar química dentro do núcleo do Fluid Engine.

---

# 20. FLUID-18 — Contamination

Suporte a estados:

```text id="qf3d9m"
clean
contaminated
polluted
treated
```

Isso poderia conectar com:

```text id="c6fr99"
ecology
civilization
health
industry
```

---

# 21. FLUID-19 — Water Cycle

No sistema ambiental, a água pode circular:

```text id="g8m7z9"
Ocean
 ↓
Evaporation
 ↓
Cloud
 ↓
Rain
 ↓
River
 ↓
Lake
 ↓
Groundwater
 ↓
Ocean
```

Aqui temos uma separação importante:

**Fluid Engine** simula o fluido local.

**Climate/Hydrology System** simula o ciclo planetário.

---

# 22. FLUID-20 — Rain Integration

Clima pode gerar entradas:

```text id="g3ccy0"
rain
 ↓
surface water
```

O Fluid Engine recebe:

```text id="9f78ul"
fluid injection
```

---

# 23. FLUID-21 — Snow / Ice

O sistema deve poder representar:

```text id="a7nqsg"
snow
ice
meltwater
```

com integração com temperatura.

---

# 24. FLUID-22 — Groundwater

Isso é importante para seu WorldGen.

Criar:

```text id="f8kt1a"
Aquifer
```

que é conceitualmente diferente de um lago superficial.

```text id="x2d7fq"
Rain
 ↓
ground
 ↓
aquifer
 ↓
underground flow
```

---

# 25. FLUID-23 — Underground Rivers

Permitir:

```text id="qb8yx4"
underground river
underground lake
cave waterfall
aquifer connection
```

---

# 26. FLUID-24 — Cave Water

A água deve funcionar naturalmente nas cavernas:

```text id="j5l9g6"
cave
 ↓
water source
 ↓
gravity
 ↓
flow
 ↓
underground lake
```

---

# 27. FLUID-25 — Waterfalls

O Renderer pode criar a aparência de queda.

O Fluid Engine fornece:

```text id="q3b6r9"
vertical velocity
flow rate
```

Então:

```text id="y31yq0"
Fluid
 ↓
Water Renderer
 ↓
waterfall visual
```

---

# 28. FLUID-26 — Currents

Água pode possuir vetor:

```text id="9jks71"
FluidVelocity
├── x
├── y
└── z
```

Isso permite:

```text id="z2h9ta"
rivers
currents
waterfalls
pipes
ocean movement
```

---

# 29. FLUID-27 — Ocean Currents

Para oceanos gigantes, não simular cada gota.

Criar:

```text id="kq4p2b"
OceanCurrentField
```

que representa padrões de circulação em escala regional.

---

# 30. FLUID-28 — Fluid LOD

Muito importante.

Perto:

```text id="bx9fpa"
FULL FLOW SIMULATION
```

Regional:

```text id="07sdh2"
SIMPLIFIED FLOW
```

Distante:

```text id="m7y0ln"
FIELD / AGGREGATE
```

---

# 31. FLUID-29 — Large Ocean Simulation

Não queremos:

```text id="e28o3d"
um milhão de células → uma simulação cara por frame
```

Criar escalas:

```text id="0u2xnv"
Voxel
 ↓
Chunk Fluid State
 ↓
Regional Current
 ↓
Ocean Model
```

---

# 32. FLUID-30 — Fluid Chunk

Cada chunk pode possuir:

```text id="g2c1rs"
FluidChunk
├── states
├── active cells
├── source list
├── dirty regions
└── metadata
```

---

# 33. FLUID-31 — Active Fluid Cells

Uma célula completamente estável não precisa ser calculada constantemente.

```text id="2l7h6d"
STATIC
```

Quando algo muda:

```text id="mi3f8f"
WAKE
 ↓
simulate
```

---

# 34. FLUID-32 — Sleep

Semelhante à Física:

```text id="qg1f19"
flowing
 ↓
velocity ≈ 0
 ↓
sleep
```

---

# 35. FLUID-33 — Chunk Borders

Fluxo precisa atravessar fronteiras:

```text id="3e7x1k"
Chunk A
 ↓
boundary
 ↓
Chunk B
```

Então alterações devem gerar:

```text id="w3k8n7"
neighbor wake
```

---

# 36. FLUID-34 — Fluid Sources

Fontes podem ser:

```text id="x9erj7"
World
Block
Machine
Pipe
Container
Rain
Entity
Structure
```

---

# 37. FLUID-35 — Fluid Sinks

Da mesma maneira:

```text id="qgct6p"
drain
machine
container
soil
portal
evaporation
```

---

# 38. FLUID-36 — Containers

Criar API:

```text id="wz6d96"
FluidContainer
├── capacity
├── acceptedFluids
├── currentFluid
├── amount
└── pressure
```

Isso será base para máquinas.

---

# 39. FLUID-37 — Tanks

Implementação especializada:

```text id="qgqt8e"
Tank
```

pode possuir:

```text id="h6jtc7"
input
output
capacity
pressure
temperature
filters
```

---

# 40. FLUID-38 — Pipes

Criar:

```text id="apq2cz"
Pipe
```

com:

```text id="jvqur0"
capacity
flowRate
pressure
connections
```

---

# 41. FLUID-39 — Fluid Network

Assim como energia:

```text id="2a8kcb"
Fluid Network
├── Sources
├── Pipes
├── Tanks
├── Machines
└── Consumers
```

---

# 42. FLUID-40 — Network Routing

O sistema pode calcular:

```text id="48pd2y"
source
 ↓
network
 ↓
destination
```

sem simular cada molécula.

---

# 43. FLUID-41 — Pipe Connections

Cada pipe pode decidir conexões:

```text id="4jbc7o"
north
south
east
west
up
down
```

---

# 44. FLUID-42 — Flow Rate

Criar conceito:

```text id="q38n0m"
amount / time
```

Assim máquinas podem solicitar:

```text id="5gd7z4"
20 units/s
```

---

# 45. FLUID-43 — Fluid Pressure in Pipes

A rede pode ter:

```text id="08r9oz"
pressure
```

Isso permite máquinas que:

```text id="fg8np7"
require high pressure
```

ou falhem/operem diferente conforme as condições.

---

# 46. FLUID-44 — Pipe Physics

Não precisa transformar cada tubo em rígido corpo físico.

A parte visual é:

```text id="a3ff31"
Renderer
```

A parte lógica é:

```text id="m0ofqf"
Fluid Network
```

A colisão física pode ser simplificada.

---

# 47. FLUID-45 — Flow Direction

Cada célula pode ter:

```text id="kqs4cn"
flow direction
flow strength
```

Ajudando:

```text id="w3o7qq"
Renderer
Physics
AI
gameplay
```

---

# 48. FLUID-46 — Buoyancy

Integração com Física:

```text id="t0pko5"
Physics body
 ↓
Fluid query
 ↓
submerged volume
 ↓
buoyancy
```

---

# 49. FLUID-47 — Fluid Drag

Também:

```text id="5gs2nq"
fluid velocity
+
body velocity
 ↓
drag
```

Isso será importante em rios e correntes.

---

# 50. FLUID-48 — Water Pressure

A Física pode consultar:

```text id="kn4e42"
pressure
```

permitindo sistemas que reajam à profundidade.

---

# 51. FLUID-49 — Swimming

O Player System pode fazer:

```text id="42pt6j"
player position
 ↓
fluid query
 ↓
water present?
 ↓
swimming controller
```

---

# 52. FLUID-50 — Fluid Detection

API:

```text id="o8w2ot"
getFluidAt()
getFluidLevel()
getFluidAmount()
getFluidType()
getFluidVelocity()
getFluidPressure()
getFluidTemperature()
```

---

# 53. FLUID-51 — Fluid Physics Material

Cada fluido pode fornecer:

```text id="f9t8un"
density
viscosity
dragCoefficient
buoyancyFactor
```

para a Física.

---

# 54. FLUID-52 — Fluid Rendering Interface

O Renderer recebe:

```text id="7m2l7f"
FluidRenderData
```

contendo:

```text id="3o0srr"
surface height
flow vector
opacity
color
foam data
depth
```

---

# 55. FLUID-53 — Water Surface Mesh

O renderer cria:

```text id="ryah00"
WaterSurfaceMesh
```

a partir do estado do Fluid Engine.

---

# 56. FLUID-54 — Waves

Separar:

```text id="9e7y2j"
simulation flow
```

de:

```text id="x7xq1z"
visual wave animation
```

Assim as ondas visuais não precisam alterar fisicamente a água.

---

# 57. FLUID-55 — Foam

Criar dados para:

```text id="gmd1fd"
foam
```

em:

```text id="g8l6hh"
waterfall
shoreline
high-flow
collision
```

---

# 58. FLUID-56 — Underwater

O Fluid Engine informa:

```text id="d25z1o"
submersion
depth
fluid type
```

Renderer pode aplicar:

```text id="a5a0ss"
underwater fog
color absorption
refraction
caustics
```

---

# 59. FLUID-57 — Caustics

Para água:

```text id="opxz3e"
sun
 ↓
water
 ↓
caustics
```

É um efeito visual, portanto fica principalmente no Renderer.

---

# 60. FLUID-58 — Lava

Lava usa o mesmo sistema.

```text id="pz1j3v"
FluidDefinition
→ lava
```

com:

```text id="dl4hv7"
temperature
density
emission
viscosity
```

---

# 61. FLUID-59 — Gas

O sistema não pode assumir que todos os fluidos ficam embaixo.

Criar:

```text id="y51ndx"
FluidPhase
├── LIQUID
├── GAS
└── SOLID
```

Gases podem:

```text id="6b2x9p"
rise
spread
mix
pressurize
```

---

# 62. FLUID-60 — Gas Volumes

Para gases, pode ser útil representar:

```text id="3j5vvi"
GasField
```

em vez de simplesmente “bloco cheio”.

---

# 63. FLUID-61 — Gas Density

A densidade pode depender de:

```text id="7l3v1r"
temperature
pressure
composition
```

---

# 64. FLUID-62 — Toxic / Special Fluids

O Fluid Engine deve poder transportar propriedades especiais:

```text id="fu5aiq"
toxic
corrosive
flammable
radioactive-like
magical
biological
```

Mas quem interpreta isso é outro sistema.

Por exemplo:

```text id="24hymf"
Fluid
→ property = corrosive

Damage System
→ interpreta propriedade
```

---

# 65. FLUID-63 — Fluid Tags

Em vez de verificar nomes:

```text id="af3ugx"
isWater()
```

usar categorias:

```text id="03tw8v"
#water
#liquid
#flammable
#industrial
```

Isso facilita mods.

---

# 66. FLUID-64 — Fluid Compatibility

Cada recipiente/máquina pode declarar:

```text id="7x6c30"
accepted tags
rejected tags
capacity
```

---

# 67. FLUID-65 — Fluid Inventory

Para máquinas:

```text id="yh6t3d"
FluidInventory
├── tanks
├── inputs
└── outputs
```

---

# 68. FLUID-66 — Fluid Transfers

Operação atômica:

```text id="d6a8nz"
Transfer
├── source
├── destination
├── fluid
├── amount
└── transactionId
```

---

# 69. FLUID-67 — Duplication Protection

Como no Inventory:

```text id="60j4me"
transaction
 ↓
validate
 ↓
commit
```

Evitar:

```text id="qnd8tl"
duplicação
perda
double transfer
```

---

# 70. FLUID-68 — Multiplayer

O servidor valida:

```text id="i6urc8"
fluid state
transfers
network changes
container changes
```

Cliente pode prever visualmente o fluxo, mas o estado autoritativo continua no servidor.

---

# 71. FLUID-69 — Save / Load

Persistir somente o necessário:

```text id="cyz1a4"
active fluid state
persistent sources
containers
networks
```

Caches derivados podem ser reconstruídos.

---

# 72. FLUID-70 — Fluid Chunk Serialization

```text id="36wohw"
FluidChunkData
├── version
├── active cells
├── source data
└── network references
```

---

# 73. FLUID-71 — Fluid Network Persistence

Redes podem precisar persistir:

```text id="j8yq4r"
network topology
stored amounts
tank contents
```

---

# 74. FLUID-72 — Rebuild After Load

Ao carregar:

```text id="zoh7fw"
world
 ↓
fluid chunks
 ↓
network reconstruction
 ↓
wake active regions
```

---

# 75. FLUID-73 — Determinism

Para um estado igual:

```text id="s2b2j4"
same inputs
+
same fluid version
+
same timestep
=
same simulation
```

---

# 76. FLUID-74 — Tick System

Fluidos não precisam necessariamente rodar todo frame.

Pode existir:

```text id="1tp2ce"
FAST
NORMAL
SLOW
REGIONAL
```

Dependendo do sistema.

---

# 77. FLUID-75 — Adaptive Ticks

Se um fluido está parado:

```text id="4ft0h7"
slow tick
```

Se está em uma cachoeira:

```text id="svzwlb"
fast tick
```

---

# 78. FLUID-76 — Stability Detection

Detectar:

```text id="s1s0a7"
steady state
```

para dormir a região.

---

# 79. FLUID-77 — Fluid Events

Eventos:

```text id="87bd6r"
FluidCreated
FluidChanged
FluidFlowStarted
FluidFlowStopped
FluidEntered
FluidExited
FluidMixed
FluidTransformed
PressureChanged
TemperatureChanged
```

---

# 80. FLUID-78 — Event Integration

Exemplo:

```text id="e3e0qf"
Water reaches farmland
 ↓
Fluid Event
 ↓
Agriculture System
 ↓
irrigation effect
```

---

# 81. FLUID-79 — Farming

Agricultura pode consultar:

```text id="z4uvxs"
soil moisture
fluid presence
groundwater
irrigation
```

O sistema agrícola interpreta; Fluid Engine fornece os dados.

---

# 82. FLUID-80 — Ecosystem

Água pode alimentar:

```text id="4sbf2r"
plants
animals
settlements
ecosystems
```

---

# 83. FLUID-81 — Civilization

Cidades podem depender de:

```text id="6c7qjz"
water supply
rivers
wells
reservoirs
pipes
irrigation
```

Então:

```text id="4vyq9v"
Fluid Network
 ↓
Civilization/Economy
```

pode virar parte da infraestrutura econômica.

---

# 84. FLUID-82 — Pollution

Cidade/indústria pode gerar:

```text id="6r6p9x"
pollution
```

que altera propriedades de um fluido.

Isso pode alimentar:

```text id="b2y7l0"
Ecology
Health
Economy
Knowledge
Politics
```

---

# 85. FLUID-83 — Disaster Systems

O sistema deve permitir outros módulos reagirem a fluidos extremos:

```text id="5clv1p"
flood
overflow
dam failure
reservoir breach
```

A lógica de desastre fica fora do núcleo.

---

# 86. FLUID-84 — Dams / Reservoirs

Estruturas podem armazenar:

```text id="yy2mck"
water volume
pressure
```

e fornecer integração com:

```text id="gdxs0y"
energy
civilization
infrastructure
```

---

# 87. FLUID-85 — Hydraulic Machines

Suporte para máquinas que trabalham com fluido:

```text id="3ga6te"
pump
turbine
boiler
cooler
processor
separator
```

Essas máquinas usam a API de transferência.

---

# 88. FLUID-86 — Pumps

API:

```text id="xwt0eg"
Pump
├── input
├── output
├── flowRate
└── pressure
```

---

# 89. FLUID-87 — Fluid Power

O Fluid Engine pode fornecer:

```text id="kga24x"
pressure
flow
```

e um sistema de máquinas pode converter isso em trabalho.

---

# 90. FLUID-88 — Energy Integration

Assim:

```text id="rky1tm"
Fluid
 ↓
Pressure
 ↓
Turbine
 ↓
Energy
```

O Fluid Engine não conhece Energy; a Machine API faz a conversão.

---

# 91. FLUID-89 — Temperature Networks

Máquinas podem mover não só fluido, mas calor:

```text id="sbk5iv"
hot fluid
 ↓
machine
 ↓
cooled fluid
```

---

# 92. FLUID-90 — Fluid Data API

Publicamente:

```text id="b6in03"
IFluidReader
IFluidWriter
IFluidContainer
IFluidNetwork
IFluidQuery
IFluidSource
IFluidSink
```

---

# 93. FLUID-91 — Mod API

Mods podem registrar:

```text id="0o42r7"
FluidDefinition
FluidReaction
FluidTransformation
FluidContainer
FluidNetworkComponent
FluidRenderProfile
FluidPhysicsProfile
```

---

# 94. FLUID-92 — Official Content

Conteúdo oficial usa a mesma API:

```text id="nq5p5p"
NEXORA official
       ↓
Fluid API

Community mod
       ↓
Fluid API
```

---

# 95. FLUID-93 — Renderer Integration

Fluxo:

```text id="f6u2cc"
Fluid State
 ↓
Fluid Render Snapshot
 ↓
Water/Fluid Renderer
 ↓
surface
transparency
foam
refraction
```

---

# 96. FLUID-94 — Physics Integration

Fluxo:

```text id="7lxn8y"
Physics Body
 ↓
Fluid Query
 ↓
density
velocity
submersion
pressure
 ↓
forces
```

---

# 97. FLUID-95 — Chunk Integration

```text id="m61p0k"
Voxel Change
 ↓
Fluid Invalidation
 ↓
Flow Queue
 ↓
Chunk Fluid State
```

---

# 98. FLUID-96 — WorldGen Integration

WorldGen pode criar:

```text id="8mxfyt"
river
lake
ocean
aquifer
water source
```

mas ele apenas inicializa o estado.

Depois:

```text id="8i6g5q"
WorldGen
 ↓
initial fluid state
 ↓
Fluid Engine
 ↓
world continues evolving
```

Essa separação é muito importante para o conceito do NEXORA.

---

# 99. FLUID-97 — Climate Integration

```text id="sn4pr6"
Climate
 ↓
precipitation
evaporation
temperature
 ↓
Fluid Engine
```

---

# 100. FLUID-98 — Deep World

No subterrâneo:

```text id="pf7y34"
Aquifer
 ↓
Cave
 ↓
River
 ↓
Underground lake
```

e a água pode participar da economia/civilização subterrânea.

---

# 101. FLUID-99 — Ocean

Oceano deve possuir:

```text id="1z3guj"
depth
currents
temperature
pressure
salinity-like properties
```

A parte de escala oceânica pode usar modelos simplificados.

---

# 102. FLUID-100 — Final Architecture

```text id="m4o8b1"
                         WORLD
                           │
                     FLUID ENGINE
                           │
          ┌────────────────┼─────────────────┐
          │                │                 │
       LOCAL FLOW       NETWORKS         ENVIRONMENT
          │                │                 │
       Voxels            Pipes            Climate
       Sources           Tanks            Hydrology
       Pressure          Machines         Ocean
       Temperature       Pumps            Aquifer
          │                │                 │
          └────────────────┼─────────────────┘
                           │
            ┌──────────────┼───────────────┐
            │              │               │
         PHYSICS        RENDERER        GAMEPLAY
            │              │               │
        buoyancy        surface          farming
        drag            waves            ecology
        pressure        foam             civilization
        swimming        refraction       economy
                           │
                        SPACE
                           │
                     gas / fluids
```

# 103. Ordem de implementação

Eu faria:

```text id="z9p06r"
FLUID-0 Definition
FLUID-1 State
FLUID-2 Voxel Integration
FLUID-3 Volume
FLUID-4 Levels
FLUID-5 Sources
FLUID-6 Flow Solver
FLUID-7 Flow Queue
FLUID-8 Gravity
FLUID-9 Pressure
FLUID-10 Temperature
FLUID-11 Chunk Integration
FLUID-12 Neighbor Flow
FLUID-13 Sleeping
FLUID-14 Queries
FLUID-15 Containers
FLUID-16 Transfers
FLUID-17 Pipes
FLUID-18 Networks
FLUID-19 Physics Integration
FLUID-20 Renderer Integration
FLUID-21 Water
FLUID-22 Lava
FLUID-23 Gas
FLUID-24 Phase Changes
FLUID-25 Mixing
FLUID-26 Reactions API
FLUID-27 Climate Integration
FLUID-28 Hydrology
FLUID-29 Ocean
FLUID-30 Groundwater
FLUID-31 Fluid LOD
FLUID-32 Machines
FLUID-33 Multiplayer
FLUID-34 Persistence
FLUID-35 Debugging
FLUID-36 Mod API
FLUID-37 Stress Testing
```

# 104. Primeiro Vertical Slice

Eu faria o primeiro teste assim:

```text id="qg2n0j"
World
 ↓
Chunk
 ↓
Water Source
 ↓
Water State
 ↓
Gravity
 ↓
Flow
 ↓
Neighbor Chunk
 ↓
River
 ↓
Physics
 ↓
Player swims
 ↓
Renderer
 ↓
Water surface
 ↓
Save
 ↓
Load
```

Depois:

```text id="7vw5jp"
Ocean
 ↓
depth
 ↓
pressure
 ↓
currents
 ↓
underwater rendering
```

E depois:

```text id="t6s2au"
Aquifer
 ↓
Cave
 ↓
Underground River
 ↓
Underground Lake
 ↓
Civilization
 ↓
Water Infrastructure
 ↓
Economy
```

## A regra arquitetural

Eu deixaria esta como a regra do sistema:

> **Fluid Engine simula matéria fluida; Physics simula interação física; Renderer mostra o fluido; Climate/WorldGen cria as condições; Gameplay decide o significado.**

Então:

```text id="j6td0b"
Water
 ↓
Fluid Engine
 ├── flow
 ├── pressure
 ├── temperature
 └── volume
       │
       ├───────────────┐
       ↓               ↓
   Physics          Renderer
       ↓               ↓
 buoyancy          surface
 swimming          waves
 drag              foam
 pressure          refraction
       │
       └───────────────┐
                       ↓
                    Gameplay
                       ↓
              farming / ecology /
              civilization / economy
```

E tem uma consequência muito boa para o NEXORA: **o mesmo motor de fluidos consegue atender um riacho na superfície, um oceano, um aquífero no Deep World, uma tubulação industrial, um reator, um fluido mágico e eventualmente sistemas de fluidos em naves**, sem criar oito sistemas incompatíveis.
