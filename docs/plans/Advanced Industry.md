# NEXORA — ADVANCED INDUSTRY SYSTEM

> **Princípio central:**
> **Advanced Industry transforma recursos, energia, fluidos, máquinas, conhecimento, mão de obra, infraestrutura e logística em capacidade produtiva persistente. Indústria não é apenas uma máquina processando receita; é uma rede econômica e física que pode crescer, falhar, especializar-se, automatizar-se e transformar civilizações.**

A ideia central é fazer a indústria do NEXORA chegar de:

```text
forno
→ máquina
→ fábrica
```

até:

```text
mina
→ extração
→ beneficiamento
→ refinaria
→ fundição
→ fabricação
→ montagem
→ armazenamento
→ logística
→ distribuição
→ mercado
→ consumo
→ pesquisa
→ nova tecnologia
→ nova indústria
```

E tudo isso precisa existir no mesmo mundo persistente.

---

# 1. O que é Advanced Industry?

O **Advanced Industry System** será a camada que coordena produção industrial em escala.

Ele trabalha com:

```text
RESOURCE
MATERIAL
RECIPE
MACHINE
ENERGY
FLUID
STRUCTURE
LOGISTICS
STORAGE
LABOR
KNOWLEDGE
TECHNOLOGY
ECONOMY
CIVILIZATION
MAINTENANCE
QUALITY
WASTE
BYPRODUCTS
```

Mas cada sistema continua responsável pela própria especialidade.

```text
Industry
→ coordena produção

Machine
→ executa processo

Crafting
→ define transformação

Energy
→ fornece energia

Fluid
→ fornece/transfere fluidos

Inventory
→ armazena itens

Vehicle
→ transporta

Economy
→ precifica/troca

Civilization
→ organiza infraestrutura e indústria
```

---

# 2. Regra arquitetural

Não devemos criar:

```text
AdvancedIndustryGodSystem
```

que faz tudo.

O correto:

```text
                    ADVANCED INDUSTRY
                           │
          ┌────────────────┼─────────────────┐
          ▼                ▼                 ▼
      PRODUCTION        LOGISTICS         PLANNING
          │                │                 │
          ▼                ▼                 ▼
       MACHINES         STORAGE          PRODUCTION
          │                │              NETWORK
          ▼                ▼                 │
       ENERGY           VEHICLES           ▼
          │                │             ECONOMY
          ▼                ▼
        FLUID           STRUCTURE
```

---

# 3. Industry como rede

Uma fábrica individual é apenas um nó.

A verdadeira unidade industrial é:

```text
INDUSTRIAL NETWORK
```

Exemplo:

```text
IRON ORE MINE
      ↓
CRUSHER
      ↓
SEPARATOR
      ↓
FURNACE
      ↓
STEEL MILL
      ↓
CASTING
      ↓
MACHINING
      ↓
ASSEMBLY
      ↓
WAREHOUSE
      ↓
RAILWAY
      ↓
CITY
```

---

# 4. Industrial Site

Criar um conceito acima de Machine:

```text
IndustrialSite
```

Pode representar:

```text
mine
factory
refinery
power plant
chemical plant
shipyard
rail depot
warehouse
research complex
spaceport
orbital factory
```

Ele não substitui Structure.

Ele é uma entidade lógica que agrupa:

```text
structures
machines
networks
storage
workers
vehicles
production lines
```

---

# 5. Industrial Organization

Uma indústria pode pertencer a:

```text
PLAYER
COMPANY
FACTION
SETTLEMENT
CITY
CIVILIZATION
GOVERNMENT
COOPERATIVE
NPC
```

Exemplo:

```text
IronWorks Corporation
```

possui:

```text
Mine A
Steel Plant B
Rail Depot C
Warehouse D
Factory E
```

---

# 6. Production Definition

Define o que uma linha pode produzir.

```ts
interface ProductionDefinition {
    id: ResourceID;

    recipe: RecipeID;

    requiredTechnology?: TechnologyID[];

    requiredFacilities?: FacilityRequirement[];

    inputRequirements: ResourceRequirement[];

    outputProducts: ResourceOutput[];

    byproducts: ResourceOutput[];

    waste: WasteProfile;

    processTime: Duration;

    energyRequirement: EnergyRequirement;

    fluidRequirement: FluidRequirement[];

    qualityProfile: QualityProfile;
}
```

---

# 7. Production Line

A linha de produção é uma unidade operacional.

```text
ProductionLine
```

Exemplo:

```text
STEEL PRODUCTION LINE

ore
 ↓
crusher
 ↓
separator
 ↓
furnace
 ↓
converter
 ↓
caster
 ↓
rolling mill
```

Cada estágio pode ser:

```text
Machine
Process
Buffer
Transport
QualityControl
```

---

# 8. Production Stage

```ts
interface ProductionStage {
    id: StageID;

    process: ProcessDefinition;

    machines: MachineID[];

    inputBuffers: StorageNode[];

    outputBuffers: StorageNode[];

    requiredEnergy: EnergyProfile;

    requiredFluids: FluidProfile[];

    cycleTime: Duration;
}
```

---

# 9. Industrial Process

É importante separar:

```text
Recipe
```

de:

```text
Industrial Process
```

Recipe:

```text
Iron + Carbon
→ Steel
```

Process:

```text
temperature
pressure
time
atmosphere
energy
machine type
cooling
feed rate
```

Assim podemos ter:

```text
same recipe
+
different process
=
different quality
```

---

# 10. Process Parameters

Um processo industrial pode possuir:

```text
temperature
pressure
flow
speed
feed rate
voltage
current
purity
concentration
atmosphere
humidity
cooling rate
mechanical force
magnetic field
```

Isso permite indústria avançada sem transformar tudo em receitas simplistas.

---

# 11. Process Control

Criar:

```text
ProcessController
```

Ele pode ajustar:

```text
target temperature
production rate
energy consumption
fluid flow
machine speed
quality target
safety limits
```

---

# 12. Automation

Aqui entra o:

```text
Automation System
```

Já planejado no Machines.

Industry usa:

```text
Machine
+
Sensors
+
Controllers
+
Signals
+
Logic
+
Production Planner
```

---

# 13. Industrial Control Layer

Arquitetura:

```text
SENSORS
   ↓
CONTROL SYSTEM
   ↓
DECISION
   ↓
MACHINE COMMAND
   ↓
PROCESS
   ↓
MEASUREMENT
   ↓
FEEDBACK
```

Isso permite:

```text
automatic production
adaptive production
failure detection
quality control
```

---

# 14. Bottleneck Detection

Uma das funções mais importantes.

Exemplo:

```text
Mine
100 ore/min

Crusher
100 ore/min

Furnace
80 ore/min

Rolling mill
120 ore/min
```

O sistema identifica:

```text
BOTTLENECK = FURNACE
```

E pode calcular:

```text
lost production
idle machines
buffer growth
energy waste
```

---

# 15. Production Graph

Toda indústria pode virar um grafo:

```text
RESOURCE
   ↓
PROCESS
   ↓
PROCESS
   ↓
PRODUCT
```

Exemplo:

```text
Iron Ore
   ↓
Iron Concentrate
   ↓
Pig Iron
   ↓
Steel
   ↓
Plate
   ↓
Machine Part
   ↓
Machine
```

---

# 16. Dependency Graph

Uma fábrica complexa pode depender de centenas de produtos.

```text
Rocket
├── Steel
├── Aluminum
├── Electronics
├── Fuel
├── Composites
├── Navigation Computer
├── Sensors
└── Life Support
```

Cada componente possui outra árvore.

---

# 17. Industrial Planning

Criar:

```text
ProductionPlanner
```

Entrada:

```text
desiredOutput
```

Exemplo:

```text
100 rockets/week
```

Saída:

```text
required materials
required machines
required energy
required fluids
required facilities
required logistics
required workforce
```

---

# 18. Backward Planning

Muito importante.

```text
TARGET
 ↓
dependencies
 ↓
subdependencies
 ↓
raw resources
```

Exemplo:

```text
100 engines
 ↓
10,000 engine components
 ↓
steel
 ↓
iron ore
```

---

# 19. Forward Simulation

Também precisamos do fluxo contrário:

```text
RAW RESOURCES
 ↓
INDUSTRIAL NETWORK
 ↓
CURRENT CAPACITY
 ↓
PREDICTED OUTPUT
```

Assim:

```text
Planner
```

pode responder:

> “Consigo produzir 100 motores por semana?”

---

# 20. Capacity Planning

Cada indústria possui:

```text
capacity
throughput
utilization
efficiency
downtime
quality
```

Exemplo:

```text
Nominal:
1000 units/day

Actual:
812 units/day

Utilization:
81.2%

Maintenance:
6%

Material shortage:
4%
```

---

# 21. Resource Flow

Precisamos tratar recursos como fluxo industrial.

```text
Mining
 ↓
Buffer
 ↓
Transport
 ↓
Factory
 ↓
Buffer
 ↓
Transport
 ↓
Consumer
```

O fluxo pode ser:

```text
continuous
batch
discrete
scheduled
priority
emergency
```

---

# 22. Batch Production

Exemplo:

```text
Steel furnace
```

produz:

```text
10 tons/batch
```

Não precisa simular cada grama.

Podemos ter:

```text
MaterialBatch
```

com:

```text
amount
composition
purity
temperature
quality
origin
```

---

# 23. Material Provenance

Isso combina muito com NEXORA.

Um produto pode saber:

```text
ore:
mine A

refinery:
plant B

steel:
plant C

machine:
factory D
```

Isso possibilita:

```text
quality tracking
recalls
trade history
technology provenance
```

---

# 24. Material Quality

A indústria pode produzir:

```text
LOW
STANDARD
HIGH
PREMIUM
EXPERIMENTAL
```

Qualidade depende de:

```text
purity
process control
temperature
machine condition
operator skill
material quality
contamination
```

---

# 25. Imperfections

Produção não precisa ser perfeita.

Podem surgir:

```text
defects
contamination
warping
cracks
incorrect dimensions
weakness
electrical faults
```

O sistema de qualidade detecta.

---

# 26. Quality Control

```text
PRODUCTION
 ↓
INSPECTION
 ↓
PASS
   OR
FAIL
```

Falha pode resultar em:

```text
scrap
rework
downgrade
repair
recycling
```

---

# 27. Rework

Um produto defeituoso não precisa ser destruído.

```text
DEFECT
 ↓
ANALYSIS
 ↓
REWORK
 ↓
PASS
```

Isso cria indústria mais interessante.

---

# 28. Waste System

Advanced Industry precisa possuir:

```text
WASTE
```

Tipos:

```text
solid
liquid
gas
thermal
radioactive
chemical
electronic
biological
```

O Waste não desaparece magicamente.

---

# 29. Recycling

```text
WASTE
 ↓
SORTING
 ↓
RECOVERY
 ↓
RAW MATERIAL
 ↓
PRODUCTION
```

Isso conecta:

```text
Industry
Economy
Ecology
Technology
Civilization
```

---

# 30. Byproducts

Processo:

```text
ore → steel
```

pode gerar:

```text
slag
dust
heat
gas
waste fluid
```

Alguns subprodutos podem ser úteis.

```text
BYPRODUCT
 ↓
OTHER INDUSTRY
```

---

# 31. Industrial Symbiosis

Uma fábrica pode usar o resíduo de outra.

```text
Steel Plant
→ Waste Heat
        ↓
District Heating

Chemical Plant
→ Waste Gas
        ↓
Fuel Recovery

Factory
→ Scrap Metal
        ↓
Recycling Plant
```

Isso é muito interessante para cidades industriais.

---

# 32. Energy Integration

Industry usa:

```text
Energy API
```

Exemplo:

```text
Factory
 ↓
Energy Network
 ↓
Grid
 ↓
Power Plant
```

Pode ocorrer:

```text
brownout
overload
load shedding
backup
battery
generator
```

---

# 33. Industrial Energy Priority

Nem todas as máquinas possuem a mesma prioridade.

```text
CRITICAL
HIGH
NORMAL
LOW
OPTIONAL
```

Durante escassez:

```text
life support
→ stays on

research
→ maybe reduced

cosmetic production
→ shuts down
```

---

# 34. Thermal Industry

Energia térmica pode ter ciclo:

```text
Fuel
 ↓
Heat
 ↓
Process
 ↓
Waste Heat
 ↓
Recovery
 ↓
Energy / Heating
```

---

# 35. Fluid Industry

Integração com:

```text
Fluid API
```

Permite:

```text
oil
water
steam
gas
chemical
fuel
coolant
industrial fluids
```

Processos podem exigir:

```text
temperature
pressure
purity
flow rate
```

---

# 36. Chemical Industry

Mais avançada:

```text
reactor
separator
distillation
refinery
synthesizer
catalyst
```

Processo:

```text
raw fluid
 ↓
reaction
 ↓
mixture
 ↓
separation
 ↓
products
```

---

# 37. Mining Industry

A cadeia:

```text
Prospection
 ↓
Mine Planning
 ↓
Extraction
 ↓
Crushing
 ↓
Processing
 ↓
Refining
```

Integra com:

```text
WorldGen
Geology
Cave
Vehicle
Energy
Fluid
Economy
Structure
```

---

# 38. Mining methods

Podemos ter:

```text
surface mine
quarry
underground mine
shaft
tunnel
borehole
deep extraction
offshore extraction
space mining
```

---

# 39. Industrial Construction

Uma indústria não aparece instantaneamente.

```text
PLAN
 ↓
DESIGN
 ↓
MATERIAL PROCUREMENT
 ↓
FOUNDATION
 ↓
STRUCTURE
 ↓
MACHINES
 ↓
NETWORKS
 ↓
COMMISSIONING
 ↓
OPERATION
```

Isso usa:

```text
Structure
Build
Construction
Vehicles
Economy
```

---

# 40. Commissioning

Uma fábrica precisa ser testada.

```text
construction complete
 ↓
machine tests
 ↓
network tests
 ↓
safety tests
 ↓
test production
 ↓
certification
 ↓
operational
```

Estados:

```text
PLANNED
CONSTRUCTING
ASSEMBLING
TESTING
COMMISSIONING
OPERATIONAL
MAINTENANCE
DECOMMISSIONED
```

---

# 41. Maintenance

Máquinas possuem:

```text
wear
condition
service interval
reliability
failure probability
```

Industry agenda:

```text
preventive maintenance
corrective maintenance
predictive maintenance
```

---

# 42. Predictive Maintenance

Sensores observam:

```text
temperature
vibration
pressure
power draw
cycle count
noise
```

Research/AI pode detectar:

```text
failure probability
```

Exemplo:

```text
Bearing failure:
87% within 14 hours
```

---

# 43. Spare Parts

Fábricas precisam de peças.

```text
machine
 ↓
maintenance demand
 ↓
spare parts inventory
 ↓
industry
```

Isso cria uma economia industrial real.

---

# 44. Industrial Workforce

Advanced Industry pode usar trabalhadores NPC.

Mas não devemos simular todos individualmente em fábricas gigantes.

Ter:

```text
workers
skills
specializations
shifts
productivity
safety
morale
```

---

# 45. Shift System

```text
Morning
Afternoon
Night
```

Uma fábrica pode operar:

```text
1 shift
2 shifts
3 shifts
continuous
```

---

# 46. Worker Skill

Exemplos:

```text
machining
metallurgy
chemistry
electrical
mechanical
automation
robotics
quality_control
maintenance
```

Skill influencia:

```text
quality
speed
failure rate
safety
```

---

# 47. Automation vs Workforce

A civilização pode escolher:

```text
labor intensive
hybrid
automated
fully automated
```

Tecnologia muda isso.

---

# 48. Robotics

Industrial systems podem registrar:

```text
robot
industrial_arm
autonomous_loader
inspection_drone
maintenance_bot
```

Robôs são entidades/máquinas conforme necessidade.

Industry os utiliza.

---

# 49. Factory AI

Não devemos criar uma nova IA monolítica.

Use:

```text
Automation
AI
Planning
Control
```

Uma fábrica pode possuir:

```text
ProductionAI
LogisticsAI
MaintenanceAI
QualityAI
```

como módulos.

---

# 50. Supply Chain

Essa talvez seja a parte mais importante do Advanced Industry.

```text
SUPPLIER
 ↓
RAW MATERIAL
 ↓
TRANSPORT
 ↓
PROCESSING
 ↓
MANUFACTURING
 ↓
DISTRIBUTION
 ↓
RETAIL
 ↓
CONSUMER
```

---

# 51. Supply Chain Graph

```text
SupplierNode
FactoryNode
WarehouseNode
MarketNode
TransportNode
ConsumerNode
```

Tudo conectado.

---

# 52. Logistics

Industry utiliza:

```text
roads
railways
ships
aircraft
pipes
conveyors
drones
spacecraft
```

Veículos não sabem:

> “Eu pertenço à indústria.”

Eles apenas executam transporte.

---

# 53. Logistics Contract

```ts
interface LogisticsOrder {
    id: LogisticsOrderID;

    source: NodeID;

    destination: NodeID;

    cargo: CargoRequirement;

    priority: Priority;

    deadline?: WorldTime;

    transportMode?: TransportMode;

    owner: EntityID;
}
```

---

# 54. Industrial Warehouses

Storage especializado:

```text
raw material storage
fluid tanks
fuel storage
cold storage
hazard storage
finished goods
spare parts
waste storage
```

---

# 55. Inventory ≠ Industrial Storage

Inventory é:

```text
player/NPC/container item management
```

Industrial Storage:

```text
high-volume material logistics
bulk storage
industrial buffers
```

Eles podem usar interfaces compartilhadas, mas não devem ser o mesmo sistema.

---

# 56. Buffer Management

Cada estágio pode possuir:

```text
input buffer
process buffer
output buffer
```

Isso é fundamental para evitar que uma fábrica pare imediatamente por uma pequena oscilação.

---

# 57. Throughput

Métrica:

```text
units/time
mass/time
volume/time
energy/time
```

Exemplo:

```text
100 kg/s ore
```

---

# 58. Efficiency

Separar:

```text
capacity
efficiency
utilization
quality
```

Exemplo:

```text
capacity: 1000 units/h
utilization: 70%
efficiency: 92%
quality: 97%
```

---

# 59. Factory Failure Modes

```text
power_loss
fluid_loss
material_shortage
machine_failure
network_failure
sensor_failure
control_failure
worker_shortage
transport_failure
storage_full
storage_empty
contamination
overheating
```

---

# 60. Cascading Failure

Exemplo:

```text
Power Plant
 ↓
power loss
 ↓
factory shutdown
 ↓
production shortage
 ↓
market shortage
 ↓
price increase
 ↓
civilization reaction
```

Isso gera World Events naturalmente.

---

# 61. Industry → World Events

Advanced Industry pode produzir:

```text
FactoryShutdown
ProductionCrisis
IndustrialAccident
SupplyChainCollapse
ResourceBoom
IndustrialExpansion
PollutionCrisis
```

World Events representa o acontecimento.

---

# 62. Industry → Ecology

Indústria pode produzir:

```text
pollution
heat
waste
water contamination
land transformation
resource depletion
```

Ecology/Climate/Water processam as consequências.

---

# 63. Pollution

Não fazer apenas:

```text
pollution += 1
```

Ter tipos:

```text
air
water
soil
thermal
noise
light
chemical
```

---

# 64. Industrial Zones

Civilization pode definir:

```text
residential
commercial
industrial
agricultural
research
military
space
```

Industry usa Zone/Structure.

---

# 65. Industrial City

Uma cidade avançada pode evoluir:

```text
village
 ↓
town
 ↓
city
 ↓
industrial city
 ↓
metropolis
 ↓
megacity
```

Mas crescimento precisa decorrer de:

```text
population
production
trade
energy
infrastructure
food
water
housing
technology
```

---

# 66. Industrial Geography

A indústria não deve aparecer aleatoriamente.

Localização pode depender de:

```text
resources
water
energy
transport
terrain
population
market
security
environment
regulation
```

---

# 67. Industrial Strategy

Civilizações podem especializar-se.

```text
mining civilization
agricultural civilization
industrial civilization
maritime civilization
scientific civilization
space civilization
```

---

# 68. Industrial Specialization

Uma região pode ser conhecida por:

```text
steel
electronics
shipbuilding
textiles
chemicals
vehicles
rocketry
magic
advanced materials
```

Isso gera comércio entre regiões.

---

# 69. Global Supply Chains

Exemplo:

```text
Region A
→ Iron

Region B
→ Chemicals

Region C
→ Electronics

Region D
→ Engines

Region E
→ Final Assembly
```

Uma interrupção pode afetar o planeta inteiro.

---

# 70. Economy Integration

Industry produz:

```text
supply
demand
employment
capital
goods
services
```

Economy calcula:

```text
price
trade
market
wealth
```

---

# 71. Industrial Pricing

Industry não deve controlar preço.

Ela fornece:

```text
production cost
capacity
available stock
delivery cost
quality
```

Economy calcula mercado.

---

# 72. Capital Investment

Civilization/Company pode decidir:

```text
build factory
upgrade line
expand mine
buy machines
build railway
research technology
```

---

# 73. Industrial Projects

Criar:

```text
IndustrialProject
```

Exemplo:

```text
Build Steel Plant
```

Possui:

```text
budget
materials
workers
deadline
dependencies
location
technology
```

---

# 74. Industrial Project Lifecycle

```text
PROPOSED
 ↓
APPROVED
 ↓
FINANCED
 ↓
PLANNED
 ↓
UNDER_CONSTRUCTION
 ↓
COMMISSIONING
 ↓
OPERATIONAL
```

Também:

```text
CANCELLED
SUSPENDED
FAILED
ABANDONED
```

---

# 75. Technology

Technology pode desbloquear:

```text
new process
new machine
new material
new automation
new energy source
new vehicle
new factory scale
```

Progression apenas determina:

```text
capability unlocked
```

Industry aplica essa capability.

---

# 76. Research

Research pode descobrir:

```text
better alloy
better catalyst
better battery
better manufacturing process
```

Depois:

```text
Research
 ↓
Knowledge
 ↓
Technology
 ↓
Industry
```

---

# 77. Quality of Technology

Uma tecnologia pode possuir:

```text
efficiency
maturity
reliability
cost
complexity
```

Uma tecnologia experimental pode:

```text
produce more
but fail more
```

---

# 78. Industrial R&D

Grandes indústrias podem possuir:

```text
R&D departments
laboratories
prototype workshops
testing facilities
```

Integra:

```text
Research
Knowledge
Technology
Industry
```

---

# 79. Prototype Industry

Exemplo:

```text
Research
 ↓
Prototype
 ↓
Testing
 ↓
Failure
 ↓
Iteration
 ↓
Production
```

Isso conecta perfeitamente ao sistema de pesquisa que definimos.

---

# 80. Modular Manufacturing

Uma fábrica pode trocar ferramentas/processos:

```text
factory
├── line A
├── line B
├── line C
```

E uma linha pode mudar de produto.

Isso exige:

```text
changeover time
tooling
calibration
cleaning
```

---

# 81. Flexible Manufacturing

Uma indústria avançada poderia fazer:

```text
Product A
 ↓
changeover
 ↓
Product B
 ↓
changeover
 ↓
Product C
```

O planner precisa considerar isso.

---

# 82. Mass Production

Para grandes volumes:

```text
high throughput
low unit cost
high automation
specialized tooling
```

---

# 83. Custom Manufacturing

Para produção pequena:

```text
high flexibility
low throughput
high customization
high skill
```

Isso gera trade-offs.

---

# 84. Industrial Trade-offs

Não existe uma indústria “melhor em tudo”.

Pode escolher:

```text
speed
quality
cost
flexibility
automation
energy efficiency
resource efficiency
```

---

# 85. Factory Upgrades

```text
faster machines
larger buffers
better sensors
better control
recycling
automation
energy recovery
quality control
maintenance system
```

---

# 86. Industrial Expansion

Uma fábrica pode:

```text
upgrade
duplicate lines
build new facilities
open remote sites
create supply chain
```

---

# 87. Industry Graph Across Civilizations

Isso pode escalar para:

```text
WORLD
├── Civilization A
│   ├── Industry
│   ├── Research
│   └── Trade
│
├── Civilization B
│   └── Industry
│
└── Civilization C
    └── Industry
```

---

# 88. Competition

Empresas/civilizações podem competir por:

```text
resources
markets
technology
workers
transport routes
energy
```

Social/Factions/Economy resolvem a parte social/política.

Industry fornece:

```text
production capability
```

---

# 89. Industrial Espionage como possibilidade sistêmica

Sem criar lógica especial dentro de Industry.

Podemos ter:

```text
Social
Faction
Knowledge
Security
```

interagindo com:

```text
industrial technology
```

Resultado:

```text
Technology leak
```

vira World Event ou Knowledge event.

---

# 90. Automation Contracts

Industry pode solicitar:

```text
"I need 10,000 steel plates/week."
```

O planner transforma isso em:

```text
production order
```

---

# 91. Production Order

```ts
interface ProductionOrder {
    id: ProductionOrderID;

    product: ItemID;

    quantity: number;

    quality?: QualityRange;

    deadline?: WorldTime;

    priority: Priority;

    destination?: LogisticsNodeID;

    source?: IndustrialSiteID;
}
```

---

# 92. Production Scheduling

O scheduler determina:

```text
which line
which machine
when
how much
with which inputs
```

Objetivos:

```text
maximize throughput
minimize energy cost
meet deadline
minimize waste
maximize quality
```

---

# 93. Optimization

Podemos criar diferentes estratégias:

```text
COST
SPEED
QUALITY
ENERGY
RESOURCE
BALANCED
EMERGENCY
```

---

# 94. Emergency Production

Civilização entra em emergência.

```text
WAR
DISASTER
EPIDEMIC
SPACE CRISIS
```

Pode:

```text
prioritize medicine
prioritize water
prioritize repair parts
```

Industry muda produção.

---

# 95. Industrial Blackout

Se:

```text
Energy shortage
```

o sistema pode:

```text
shed low priority lines
preserve critical systems
use backup generators
reschedule batches
```

---

# 96. Industrial Digital Twin

Criar:

```text
IndustrialSimulation
```

que representa virtualmente:

```text
machines
flows
energy
materials
production
failures
```

Pode simular:

> “O que acontece se eu dobrar a produção?”

---

# 97. Simulation Mode

```text
nexora industry simulate <site>
```

Pode calcular:

```text
output
energy
fluid
waste
cost
bottlenecks
failure probability
```

Sem alterar o mundo.

---

# 98. Industrial AI Planning

NPC/Civilization pode pedir:

```text
"Precisamos de 10.000 food units/month."
```

Planner verifica:

```text
agriculture
processing
storage
transport
```

e propõe:

```text
new factory
more farms
new railway
warehouse
```

---

# 99. Industrial Logistics AI

Pode responder:

```text
Which route is cheapest?

Which train should carry this?

Should we stockpile?

Should we build another warehouse?

Should we increase production?
```

---

# 100. Industrial State

```ts
interface IndustrialSiteState {
    status: IndustrialStatus;

    throughput: number;

    utilization: number;

    efficiency: number;

    energyDemand: number;

    fluidDemand: number;

    inventoryValue: number;

    workforce: number;

    maintenanceLoad: number;

    wasteOutput: number;

    qualityIndex: number;

    activeFailures: FailureState[];

    activeOrders: ProductionOrder[];
}
```

---

# 101. Advanced Industry APIs

```ts
interface IIndustrySystem {

    createSite(definition: IndustrialSiteDefinition): Result;

    startSite(siteId: IndustrialSiteID): Result;

    stopSite(siteId: IndustrialSiteID): Result;

    createLine(siteId: IndustrialSiteID, definition: ProductionLineDefinition): Result;

    schedule(order: ProductionOrder): Result;

    cancel(orderId: ProductionOrderID): Result;

    simulate(request: IndustrySimulationRequest): IndustrySimulationResult;

    inspect(siteId: IndustrialSiteID): IndustrialSiteState;

    analyzeBottlenecks(siteId: IndustrialSiteID): BottleneckReport;

    calculateCapacity(siteId: IndustrialSiteID): CapacityReport;

    requestExpansion(request: IndustrialExpansionRequest): Result;
}
```

---

# 102. Registries

Registrar:

```text
IndustrialSiteDefinition
ProductionLine
IndustrialProcess
ProductionStage
IndustrialModule
IndustrialSector
IndustrialZone
WasteType
QualityProfile
MaintenancePolicy
LogisticsPolicy
ProductionStrategy
```

---

# 103. Events

O Industry publica:

```text
IndustrialSiteCreated
IndustrialSiteStarted
IndustrialSiteStopped
ProductionStarted
ProductionCompleted
ProductionFailed
ProductionDelayed
MachineBottleneckDetected
MaterialShortage
EnergyShortage
FluidShortage
QualityFailure
MaintenanceRequired
IndustrialAccident
WasteProduced
IndustrialExpansionStarted
IndustrialExpansionCompleted
```

---

# 104. Commands

```text
CreateIndustrialSiteCommand
StartIndustrialSiteCommand
StopIndustrialSiteCommand
CreateProductionLineCommand
ScheduleProductionCommand
CancelProductionCommand
StartMaintenanceCommand
UpgradeIndustrialSiteCommand
ExpandIndustrialSiteCommand
MoveIndustrialOrderCommand
SimulateIndustrialProjectCommand
```

---

# 105. Persistence

Persistir:

```text
site identity
ownership
production state
orders
inventory references
machine states
maintenance
projects
contracts
network topology
```

Não persistir diretamente:

```text
render mesh
derived path cache
temporary UI data
```

---

# 106. LOD

### FULL

```text
machine
worker
conveyor
fluid
energy
production cycle
```

### REGIONAL

```text
factory throughput
inventory
employment
energy demand
```

### ABSTRACT

```text
production capacity
economic output
resource demand
```

---

# 107. Mega Factory

Para uma instalação enorme:

```text
100,000 machines
```

não simular tudo individualmente quando estiver a milhares de quilômetros do jogador.

Usar:

```text
IndustrialAggregate
```

com:

```text
throughput
capacity
energy
inventory
failure statistics
```

---

# 108. Space Industry

Industry não deve terminar no planeta.

No Space System:

```text
asteroid mine
 ↓
orbital refinery
 ↓
orbital factory
 ↓
space station
 ↓
planet
```

---

# 109. Deep World Industry

Também pode existir:

```text
underground factory
deep mining
subterranean refinery
geothermal plant
```

Isso integra:

```text
Deep World
Industry
Energy
Fluid
Civilization
```

---

# 110. Far Lands Industry

E isso fica especialmente interessante.

Far Lands podem possuir:

```text
unique resources
rare materials
special geology
dangerous environment
```

Então civilizações constroem:

```text
far-langs mine
railway
supply depot
processing plant
```

O custo logístico vira parte da progressão.

---

# 111. Beyondlands Industry

Depois:

```text
Far Lands
 ↓
Beyondlands
 ↓
dimensional resources
```

Isso cria uma indústria de fronteira.

---

# 112. Dimensional Industry

Cada dimensão pode ter:

```text
unique materials
unique energy
unique chemistry
unique machines
```

Industry usa o:

```text
Dimension System
```

para aplicar regras ambientais.

---

# 113. Industrial Technology Tree

Não deve ser uma árvore rígida.

Pode ser:

```text
Material Science
├── Alloying
├── Ceramics
└── Composites

Energy
├── Battery
├── Reactor
└── Fusion

Manufacturing
├── CNC
├── Robotics
└── Nanofabrication
```

Capability graph.

---

# 114. Industrial Research Loop

```text
PROBLEM
 ↓
RESEARCH
 ↓
EXPERIMENT
 ↓
KNOWLEDGE
 ↓
TECHNOLOGY
 ↓
PROTOTYPE
 ↓
INDUSTRIALIZATION
 ↓
MASS PRODUCTION
```

Isso fecha o ciclo:

```text
Research
→ Technology
→ Industry
→ Society
→ New Problems
→ Research
```

---

# 115. Civilizational Industrialization

Uma civilização pode atravessar fases naturalmente:

```text
manual craft
 ↓
workshops
 ↓
mechanization
 ↓
factory
 ↓
automation
 ↓
advanced manufacturing
 ↓
planetary industry
 ↓
space industry
 ↓
interdimensional industry
```

Sem precisar criar:

```text
ERA = 1, 2, 3, 4...
```

obrigatoriamente.

---

# 116. Industry e World Events

A integração mais interessante:

```text
Industrial Expansion
      ↓
Pollution
      ↓
Ecological Pressure
      ↓
Political Conflict
      ↓
New Regulations
      ↓
Industrial Adaptation
      ↓
New Technology
```

Ou:

```text
Factory Failure
 ↓
Supply Shortage
 ↓
Price Increase
 ↓
Economic Crisis
 ↓
Political Event
```

---

# 117. Estrutura de pastas

Eu colocaria:

```text
src/industry/
│
├── core/
│   ├── industry-system
│   ├── industrial-site
│   ├── industrial-definition
│   ├── industrial-state
│   └── industrial-sector
│
├── production/
│   ├── production-line
│   ├── production-stage
│   ├── production-order
│   ├── scheduler
│   └── throughput
│
├── process/
│   ├── process-definition
│   ├── process-instance
│   ├── process-parameters
│   └── process-control
│
├── planning/
│   ├── production-planner
│   ├── capacity-planner
│   ├── dependency-graph
│   └── optimization
│
├── logistics/
│   ├── logistics-order
│   ├── supply-chain
│   ├── warehouse
│   ├── transport
│   └── routing
│
├── quality/
│   ├── quality-control
│   ├── inspection
│   ├── defects
│   └── rework
│
├── maintenance/
│   ├── maintenance
│   ├── predictive
│   ├── failures
│   └── spare-parts
│
├── waste/
│   ├── waste
│   ├── byproducts
│   ├── recycling
│   └── pollution
│
├── workforce/
│   ├── workers
│   ├── skills
│   └── shifts
│
├── projects/
│   ├── industrial-project
│   ├── construction
│   └── commissioning
│
├── simulation/
│   ├── digital-twin
│   ├── simulator
│   ├── bottleneck-analysis
│   └── forecasting
│
├── networking/
├── persistence/
├── security/
├── api/
└── debug/
```

---

# 118. Dependências

```text
CORE
 │
 ├── Registry
 ├── Event Bus
 ├── Command
 ├── Persistence
 ├── Security
 └── Scheduler
       │
       ▼
 ADVANCED INDUSTRY
       │
 ├── Machines
 ├── Crafting
 ├── Energy
 ├── Fluid
 ├── Item
 ├── Structure
 ├── Vehicle
 ├── Economy
 ├── Technology
 ├── Research
 ├── Civilization
 ├── Social
 ├── World Events
 ├── Space
 └── Environment
```

---

# 119. O que Industry não pode fazer

Não colocar dentro:

```text
Physics calculations
fluid simulation
energy simulation
market pricing
NPC relationships
combat
vehicle physics
world generation
rendering
```

Industry apenas coordena e representa produção industrial.

---

# 120. Vertical Slice INDUSTRY-001

Primeira fábrica:

```text
Iron Ore
 ↓
Crusher
 ↓
Furnace
 ↓
Steel
 ↓
Warehouse
```

Precisa testar:

```text
Machine
Recipe
Energy
Fluid
Inventory
Production
Storage
Persistence
```

---

# 121. INDUSTRY-002 — Supply Chain

```text
Mine
 ↓
Truck
 ↓
Factory
 ↓
Warehouse
 ↓
Railway
 ↓
City
```

Testar:

```text
logistics
vehicles
storage
production
economy
```

---

# 122. INDUSTRY-003 — Failure

```text
Factory
 ↓
Energy shortage
 ↓
Production stops
 ↓
Warehouse drains
 ↓
Supply shortage
 ↓
Economy reacts
```

---

# 123. INDUSTRY-004 — Maintenance

```text
Machine wear
 ↓
Failure prediction
 ↓
Maintenance order
 ↓
Spare parts
 ↓
Repair
 ↓
Production resumes
```

---

# 124. INDUSTRY-005 — Quality

```text
raw material
 ↓
production
 ↓
defect
 ↓
inspection
 ↓
rework
 ↓
quality pass
```

---

# 125. INDUSTRY-006 — Industrial Expansion

```text
Civilization
 ↓
Demand increase
 ↓
Industry planner
 ↓
Expansion project
 ↓
Construction
 ↓
Commissioning
 ↓
Production
```

---

# 126. INDUSTRY-007 — Fully Autonomous Industry

Objetivo:

```text
NPC Civilization
```

decidir:

```text
"Precisamos de 20% mais steel."
```

e o sistema:

```text
detects demand
 ↓
planning
 ↓
resources
 ↓
production
 ↓
logistics
 ↓
delivery
```

sem precisar do jogador.

---

# 127. INDUSTRY-008 — Interplanetary Industry

```text
Asteroid
 ↓
Mining Ship
 ↓
Orbital Refinery
 ↓
Space Factory
 ↓
Cargo Ship
 ↓
Planet
```

Isso testa:

```text
Space
Vehicle
Industry
Energy
Fluid
Economy
Civilization
```

---

# 128. Golden Tests

### INDUSTRY-GOLD-001

```text
ore
→ steel
```

resultado determinístico.

### INDUSTRY-GOLD-002

```text
factory
→ save
→ restart
→ production resumes
```

### INDUSTRY-GOLD-003

```text
power failure
→ production stops
```

### INDUSTRY-GOLD-004

```text
material shortage
→ factory waits
```

### INDUSTRY-GOLD-005

```text
transport disruption
→ downstream factory starves
```

### INDUSTRY-GOLD-006

```text
defect
→ inspection
→ rework
```

### INDUSTRY-GOLD-007

```text
10k abstract factories
```

sem explosão de performance.

---

# 129. Stress Tests

Escalonar:

```text
10 machines
100
1,000
10,000
100,000
1,000,000 abstract machines
```

E:

```text
100 production lines
10k
100k
1M abstract production stages
```

Testar:

```text
CPU
memory
scheduler
network
save/load
dependency graph
logistics
failure propagation
```

---

# 130. Fault Injection

Testar:

```text
power failure
fluid failure
network disconnection
machine corruption
missing recipe
missing material
missing mod
invalid process
broken storage
broken logistics route
duplicate order
save corruption
server crash
```

---

# 131. Security

Invariantes:

```text
client cannot create resources
client cannot complete production
client cannot bypass recipe
client cannot alter machine output
client cannot duplicate industrial cargo
mod cannot create unlimited production
script cannot bypass industrial permissions
```

---

# 132. Performance Rule

Nunca simular:

```text
every gear rotation
every conveyor pixel
every chemical molecule
```

fora do nível necessário.

Usar:

```text
aggregate simulation
process equations
batch simulation
LOD
```

O objetivo é:

> **simular o resultado industrial, não desperdiçar CPU simulando detalhes invisíveis.**

---

# 133. Roadmap

```text
IND-0  Foundation
IND-1  Industrial Site
IND-2  Production Model
IND-3  Production Lines
IND-4  Process Engine
IND-5  Capacity
IND-6  Planning
IND-7  Logistics
IND-8  Warehousing
IND-9  Quality
IND-10 Maintenance
IND-11 Waste & Recycling
IND-12 Workforce
IND-13 Industrial Construction
IND-14 Automation
IND-15 Supply Chains
IND-16 Economy Integration
IND-17 Research Integration
IND-18 Civilization Integration
IND-19 Space Industry
IND-20 Deep World Industry
IND-21 Far Lands Industry
IND-22 Dimensional Industry
IND-23 Industrial Simulation
IND-24 Massive LOD
IND-25 Multiplayer
IND-26 Mod API
IND-27 Security Hardening
IND-28 Golden Tests
IND-29 Stress Tests
IND-30 Production Ready
```

---

# 134. Arquitetura final

```text
                           ADVANCED INDUSTRY
                                   │
          ┌────────────────────────┼────────────────────────┐
          ▼                        ▼                        ▼
      PRODUCTION                PLANNING                LOGISTICS
          │                        │                        │
          ▼                        ▼                        ▼
      PROCESSES                CAPACITY                 VEHICLES
      MACHINES                 OPTIMIZER                RAIL
      AUTOMATION               SCHEDULER                SHIPS
          │                        │                    AIRCRAFT
          └────────────────────────┼────────────────────────┘
                                   ▼
                              INDUSTRIAL SITE
                                   │
       ┌───────────────────────────┼─────────────────────────┐
       ▼                           ▼                         ▼
    MATERIAL                    ENERGY                    FLUID
       │                           │                         │
       └───────────────────────────┼─────────────────────────┘
                                   ▼
                              FACTORY OUTPUT
                                   │
                    ┌──────────────┼───────────────┐
                    ▼              ▼               ▼
                 QUALITY         WASTE          BYPRODUCT
                    │              │               │
                    ▼              ▼               ▼
                 MARKET         ECOLOGY        OTHER FACTORY
                    │              │               │
                    └──────────────┼───────────────┘
                                   ▼
                                ECONOMY
                                   │
                                   ▼
                              CIVILIZATION
                                   │
                     ┌─────────────┼─────────────┐
                     ▼             ▼             ▼
                  RESEARCH     TECHNOLOGY     WORLD EVENTS
                     │             │             │
                     └─────────────┼─────────────┘
                                   ▼
                             NEW INDUSTRY
```

## Regra de ouro

```text
INDUSTRY → coordena produção industrial.

MACHINE → executa trabalho.

CRAFTING → define transformação.

ENERGY → fornece energia.

FLUID → fornece/transfere fluidos.

LOGISTICS → move recursos.

VEHICLE → executa transporte físico.

ECONOMY → define valor e mercado.

RESEARCH → produz conhecimento.

TECHNOLOGY → desbloqueia capacidades.

CIVILIZATION → decide e organiza.

WORLD EVENTS → registra acontecimentos e consequências.

PERSISTENCE → faz tudo sobreviver ao restart.
```

O resultado que eu buscaria para o NEXORA é que **uma civilização possa realmente construir uma economia industrial que se comporta como parte do mundo**: uma mina distante pode precisar de uma ferrovia; a ferrovia pode consumir locomotivas e combustível; a fábrica pode parar por falta de minério; isso pode criar escassez; a escassez pode alterar preços; a crise pode gerar um evento mundial; a civilização pode investir em uma nova tecnologia; e essa tecnologia pode permitir uma indústria completamente diferente.

Esse encadeamento é o que faz **Advanced Industry** ser um sistema de mundo, e não apenas uma coleção de máquinas.
