# NEXORA — ADVANCED VEHICLES SYSTEM

> **Princípio central:**
> **Vehicles são entidades móveis complexas que combinam física, controle, energia, fluidos, inventário, passageiros, automação, navegação e interação.**
>
> O Vehicle System não deve ser simplesmente:
>
> ```text
> veículo = entidade + velocidade
> ```
>
> Deve ser:
>
> ```text
> VEHICLE
> ├── BODY
> ├── MOVEMENT MODEL
> ├── CONTROL
> ├── POWER
> ├── FUEL
> ├── STORAGE
> ├── PASSENGERS
> ├── MODULES
> ├── DAMAGE
> ├── MAINTENANCE
> ├── NAVIGATION
> ├── DOCKING
> ├── NETWORKING
> └── PERSISTENCE
> ```
>
> E, principalmente:
>
> **Vehicle System descreve e coordena o veículo; Physics calcula o movimento real.**

---

# 1. O que é o Advanced Vehicles System?

Ele gerencia:

```text
Vehicle Definitions
Vehicle Instances
Vehicle Parts
Chassis
Engines
Propulsion
Steering
Transmission
Suspension
Wheels
Tracks
Rotors
Wings
Thrusters
Fuel
Energy
Fluid Systems
Cargo
Passengers
Seats
Modules
Controls
Damage
Repair
Maintenance
Upgrades
Customization
Automation
Autopilot
Navigation
Docking
Transport
Convoys
Vehicle AI
Vehicle Ownership
Vehicle Lifecycle
Vehicle Persistence
Vehicle Networking
```

E deve suportar desde:

```text
carrinho
bicicleta
motocicleta
carro
caminhão
ônibus
trem
navio
submarino
helicóptero
avião
drone
rover
spacecraft
```

até veículos extremamente grandes:

```text
carrier
mobile factory
airship
orbital station vehicle
colonization ship
```

---

# 2. Vehicle ≠ Physics

Essa separação precisa ficar clara:

```text
VEHICLE SYSTEM
→ "como este veículo é construído e controlado?"

PHYSICS
→ "como ele se move?"

SPACE
→ "em qual ambiente espacial está?"

FLUID
→ "como combustível/líquido funciona?"

ENERGY
→ "como energia é transportada?"

COMBAT
→ "como o veículo participa do combate?"

ENTITY
→ "qual entidade representa o veículo?"
```

---

# 3. Arquitetura

```text
                     ADVANCED VEHICLES
                            │
       ┌────────────────────┼─────────────────────┐
       ▼                    ▼                     ▼
   DEFINITION            INSTANCE               CONTROL
       │                    │                     │
       ▼                    ▼                     ▼
     PARTS              COMPONENTS            INPUT / AI
       │                    │                     │
       └────────────────────┼─────────────────────┘
                            ▼
                        PROPULSION
                            │
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
            ENERGY         FLUID         PHYSICS
              │             │             │
              └─────────────┼─────────────┘
                            ▼
                         MOVEMENT
                            │
          ┌─────────────────┼─────────────────┐
          ▼                 ▼                 ▼
       CARGO             PASSENGERS         MODULES
          │                 │                 │
          └─────────────────┼─────────────────┘
                            ▼
                      VEHICLE STATE
                            │
             ┌──────────────┼──────────────┐
             ▼              ▼              ▼
          NETWORK        PERSISTENCE      AUDIO
```

---

# 4. Vehicle Definition

A definição descreve o tipo de veículo.

```text
VehicleDefinition
├── VehicleID
├── Category
├── Mass
├── Dimensions
├── Parts
├── Controls
├── Propulsion
├── Storage
├── Seats
├── Modules
├── DamageModel
├── PowerProfile
└── EnvironmentProfile
```

---

# 5. Vehicle Instance

A instância é o veículo real.

```text
VehicleInstance
├── EntityID
├── PersistentVehicleID
├── DefinitionID
├── Transform
├── Velocity
├── Condition
├── FuelState
├── EnergyState
├── FluidState
├── CargoState
├── CrewState
├── ModuleState
├── DamageState
└── Owner
```

---

# 6. Vehicle Categories

```text
GROUND
├── Bicycle
├── Motorcycle
├── Car
├── Truck
├── Bus
├── Tank-like
├── Rover
└── Utility

RAIL
├── Locomotive
├── Passenger
├── Cargo
└── Specialized

WATER
├── Boat
├── Ship
├── Submarine
└── Hydrofoil

AIR
├── Helicopter
├── Plane
├── Airship
├── VTOL
└── Drone

SPACE
├── Shuttle
├── Lander
├── Rover
├── Spacecraft
├── Cargo Ship
└── Exploration Ship
```

Mods podem registrar novas categorias.

---

# 7. Component Architecture

Um veículo deve ser composto de componentes.

```text
Vehicle
├── Chassis
├── Propulsion
├── Steering
├── Suspension
├── Power
├── Fuel
├── Cooling
├── Storage
├── Seats
├── Controls
└── Modules
```

Isso facilita modularidade.

---

# 8. Vehicle Parts

Uma parte:

```text
VehiclePart
├── PartID
├── Definition
├── Position
├── Orientation
├── Condition
├── Connection
└── State
```

---

# 9. Modular Vehicles

Um jogador pode montar:

```text
Chassis
+
Engine
+
Transmission
+
Wheels
+
Battery
+
Cargo
```

e produzir um veículo diferente.

---

# 10. Part Compatibility

Cada componente pode declarar:

```text
MountType
PowerRequirement
FuelType
SizeClass
Mass
ConnectionType
Capability
```

---

# 11. Vehicle Blueprint

Antes de criar:

```text
VehicleBlueprint
```

permite:

```text
design
validate
simulate
build
```

---

# 12. Vehicle Construction

Fluxo:

```text
Blueprint
 ↓
Part Validation
 ↓
Compatibility
 ↓
Resources
 ↓
Build & Destruction
 ↓
Vehicle Instance
```

---

# 13. Vehicle ≠ Structure

Um veículo pode ser uma estrutura móvel, mas não devemos confundir.

```text
STRUCTURE
→ unidade arquitetônica

VEHICLE
→ unidade móvel controlável
```

Vehicle pode **usar** Structure/Build para construção.

---

# 14. Transform

Veículo possui:

```text
Position
Rotation
Velocity
AngularVelocity
```

Physics calcula os resultados.

---

# 15. Coordinate System

```text
Vehicle
 ↓
Vehicle Local Space
 ↓
World Space
```

---

# 16. Vehicle Physics Profile

Cada veículo pode fornecer:

```text
mass
inertia
collision profile
drag
lift
traction
buoyancy
thrust
```

Physics usa esses dados.

---

# 17. Control System

O controle deve ser separado:

```text
VehicleControl
├── Steering
├── Throttle
├── Brake
├── Gear
├── Clutch
├── VerticalControl
├── Pitch
├── Yaw
├── Roll
└── SpecialActions
```

---

# 18. Control Input

Entrada pode vir de:

```text
Player
NPC
AI
Autopilot
Script
Remote Control
Automation
```

---

# 19. Controle não significa Input

O Input System produz:

```text
Input
```

Vehicle transforma em:

```text
Control Intent
```

Exemplo:

```text
mouse/keyboard
 ↓
steering intent
 ↓
vehicle control
 ↓
physics
```

---

# 20. Vehicle Control Intent

```text
VehicleControlIntent
├── throttle
├── steering
├── brake
├── vertical
├── pitch
├── yaw
├── roll
└── actions
```

---

# 21. Physics Loop

```text
Control Intent
 ↓
Vehicle Controller
 ↓
Force / Torque Requests
 ↓
Physics
 ↓
Vehicle State
```

---

# 22. Engine System

Um motor pode possuir:

```text
EngineDefinition
├── Power
├── Torque
├── RPMRange
├── Efficiency
├── FuelType
├── Cooling
└── Wear
```

---

# 23. Engine Types

```text
MUSCLE
ELECTRIC
COMBUSTION
STEAM
TURBINE
ROCKET
MAGICAL
NUCLEAR
CUSTOM
```

Mods podem adicionar.

---

# 24. Propulsion

Separar:

```text
Engine
→ gera força/energia mecânica

Propulsion
→ transforma isso em movimento
```

---

# 25. Ground Propulsion

```text
Engine
 ↓
Transmission
 ↓
Axle
 ↓
Wheel
 ↓
Traction
 ↓
Physics
```

---

# 26. Rail Propulsion

```text
Locomotive
 ↓
Power
 ↓
Traction
 ↓
Rail
 ↓
Physics
```

---

# 27. Water Propulsion

```text
Engine
 ↓
Propeller / Jet
 ↓
Fluid
 ↓
Water Interaction
 ↓
Physics
```

---

# 28. Air Propulsion

```text
Engine
 ↓
Propeller / Turbine
 ↓
Air
 ↓
Lift / Thrust
 ↓
Physics
```

---

# 29. Space Propulsion

```text
Engine
 ↓
Thruster
 ↓
Propellant
 ↓
Thrust
 ↓
Physics
 ↓
Orbit
```

---

# 30. Transmission

Para veículos terrestres:

```text
Transmission
├── Gear
├── Ratio
├── Clutch
├── Differential
└── Reverse
```

Não hardcodar transmissão no Physics.

---

# 31. Wheels

Wheel pode possuir:

```text
radius
width
traction
steering
suspension
brake
powered
```

---

# 32. Tracks

Veículos de esteira:

```text
Track
├── segments
├── drive sprocket
├── rollers
├── tension
└── traction
```

---

# 33. Suspension

```text
Suspension
├── spring
├── damping
├── travel
└── load
```

---

# 34. Tire / Surface Interaction

Physics pode consultar:

```text
SurfaceProfile
```

e calcular:

```text
traction
slip
friction
rolling resistance
```

---

# 35. Surface Integration

```text
Grass
Mud
Sand
Rock
Ice
Snow
Metal
Concrete
```

podem possuir propriedades.

---

# 36. Vehicle Damage

Damage System continua separado.

Vehicle fornece:

```text
Damageable Components
```

Combat/Physics/Event podem causar dano.

---

# 37. Damage Model

```text
VehicleDamage
├── Chassis
├── Engine
├── Transmission
├── Wheels
├── Electronics
├── FuelSystem
├── FluidSystem
└── Modules
```

---

# 38. Damage ≠ Destruction

Uma peça pode estar:

```text
100%
80%
40%
10%
0%
```

sem destruir o veículo inteiro imediatamente.

---

# 39. Vehicle Condition

```text
NEW
GOOD
WORN
DAMAGED
CRITICAL
DISABLED
DESTROYED
```

---

# 40. Maintenance

Veículos precisam de manutenção:

```text
inspection
repair
replacement
refuel
recharge
cooling
cleaning
```

---

# 41. Maintenance System

Pode ser parte do Vehicle System inicialmente, mas as regras de itens e recursos continuam nos sistemas correspondentes.

---

# 42. Wear

Peças podem degradar por:

```text
time
distance
heat
load
impact
poor maintenance
environment
```

---

# 43. Reliability

Cada componente pode possuir:

```text
ReliabilityProfile
```

---

# 44. Failure

Possibilidades:

```text
engine failure
brake failure
wheel damage
power loss
cooling failure
sensor failure
```

---

# 45. Fail-safe

O veículo deve possuir estados seguros quando possível:

```text
engine shutdown
brake fail-safe
emergency mode
manual override
```

---

# 46. Fuel

Fuel permanece como:

```text
Fluid
```

ou item, conforme o modelo do recurso.

Vehicle não deve criar um “segundo Fluid System”.

---

# 47. Fuel Tank

```text
FuelTank
→ IFluidContainer
```

---

# 48. Energy

Bateria:

```text
IEnergyStorage
```

integra com Energy API.

---

# 49. Hybrid Vehicles

Um veículo pode usar:

```text
Fuel
+
Battery
```

com control logic.

---

# 50. Resource Networks

Veículo pode ter:

```text
Energy Network
Fluid Network
Mechanical Network
```

internamente.

---

# 51. Ports

Conectar módulos:

```text
Fuel Port
Fluid Port
Energy Port
Data Port
Mechanical Port
```

usando APIs existentes.

---

# 52. Vehicle Modules

```text
Cargo
Fuel Tank
Battery
Generator
Sensor
Weapon Mount
Research Module
Medical Module
Mining Module
Communication Module
Navigation Module
Life Support
```

---

# 53. Module Host

```text
IModuleHost
```

permite:

```text
install
remove
configure
query
```

---

# 54. Modules ≠ Parts

Part:

```text
estrutura física
```

Module:

```text
capacidade funcional
```

---

# 55. Vehicle Cargo

Pode conter:

```text
ItemStacks
Containers
Fluid Tanks
Energy Storage
```

---

# 56. Cargo Security

Cargo continua usando:

```text
Item System
Inventory
Persistence
Security
```

---

# 57. Passenger System

Veículos podem ter:

```text
Seat
PassengerSlot
CrewSlot
```

---

# 58. Passenger Assignment

```text
Player
 ↓
Interaction
 ↓
Enter Vehicle
 ↓
Seat Assignment
```

---

# 59. Crew

Veículos grandes podem ter:

```text
Pilot
Engineer
Navigator
Mechanic
Scientist
Passenger
```

---

# 60. Occupancy

```text
VehicleOccupancy
├── Seat
├── Role
├── Entity
└── AccessPolicy
```

---

# 61. Enter / Exit

Interaction:

```text
Enter
Exit
```

Command:

```text
EnterVehicleCommand
LeaveVehicleCommand
```

---

# 62. Vehicle Ownership

Pode ser:

```text
PLAYER
GROUP
FACTION
CIVILIZATION
PUBLIC
UNOWNED
```

---

# 63. Ownership Rules

Owner pode controlar:

```text
access
storage
maintenance
configuration
```

conforme permissões.

---

# 64. Vehicle Registration

Veículos importantes podem possuir:

```text
VehicleRegistration
```

com:

```text
owner
type
registration
creation time
```

---

# 65. Persistent Vehicles

Um veículo construído deve sobreviver:

```text
save
restart
reload
```

---

# 66. Vehicle Storage

Vehicle Persistence usa:

```text
Persistence System
```

e salva apenas estado necessário.

---

# 67. Derived Vehicle Data

Não necessariamente persistir:

```text
render mesh
physics cache
navigation cache
audio state
```

---

# 68. Vehicle Networking

Servidor:

```text
authority
```

Cliente:

```text
prediction
interpolation
```

---

# 69. Vehicle Replication

Replicar:

```text
transform
velocity
control state
damage
fuel/energy relevant state
occupants
modules
```

conforme relevância.

---

# 70. High-Speed Vehicles

Para:

```text
train
aircraft
spacecraft
```

Interest Management precisa fazer:

```text
prefetch
```

com base na direção/velocidade.

---

# 71. Vehicle Prediction

```text
INPUT
 ↓
CLIENT VEHICLE PREDICTION
 ↓
SERVER SIMULATION
 ↓
AUTHORITATIVE STATE
 ↓
RECONCILIATION
```

---

# 72. Vehicle Interpolation

Outros jogadores:

```text
state A
 ↓
interpolate
 ↓
state B
```

---

# 73. Vehicle History Buffer

Para veículos rápidos:

```text
past states
```

podem ser mantidos no cliente/servidor conforme necessidade.

---

# 74. Vehicle AI

AI não controla física diretamente.

AI gera:

```text
VehicleControlIntent
```

ou comandos.

---

# 75. Driver AI

```text
AI
 ↓
Navigation
 ↓
Route
 ↓
Control Intent
 ↓
Vehicle
 ↓
Physics
```

---

# 76. Autopilot

Autopilot pode ser:

```text
Vehicle Module
```

utilizando:

```text
Navigation
Pathfinding
Control
```

---

# 77. Navigation

Vehicle System consulta:

```text
INavigation
```

para rotas.

---

# 78. Ground Navigation

Pode considerar:

```text
roads
terrain
bridges
traffic
```

---

# 79. Rail Navigation

Pode usar:

```text
Rail Graph
```

fornecido pelo Railway/Infrastructure system futuro.

---

# 80. Air Navigation

Pode usar:

```text
waypoints
altitude
weather
airspace
```

---

# 81. Space Navigation

Usa:

```text
Space Navigation
Orbit
Transfer
```

---

# 82. Traffic

Uma camada futura pode administrar:

```text
road traffic
rail traffic
air traffic
space traffic
```

Vehicle fornece estado.

---

# 83. Convoys

Veículos podem formar grupos:

```text
Convoy
├── Lead Vehicle
├── Cargo Vehicles
├── Support
└── Escort
```

---

# 84. Convoy AI

Social/Civilization pode solicitar:

```text
transport mission
```

e AI controla veículos.

---

# 85. Logistics

Economy/Logistics pode utilizar:

```text
Vehicle
+
Route
+
Cargo
```

---

# 86. Transport Contracts

Quest/Economy pode criar:

```text
"Deliver 500 iron to city B"
```

e Vehicle executa transporte.

---

# 87. Vehicle Automation

Máquinas podem interagir com:

```text
Vehicles
Loading
Unloading
Refueling
Charging
Routing
```

---

# 88. Vehicle Docking

```text
Vehicle
 ↕
Station
```

via Interaction/Command.

---

# 89. Water Docking

```text
Ship
 ↓
Port
```

---

# 90. Air Docking

```text
Aircraft
 ↓
Hangar
```

---

# 91. Space Docking

```text
Spacecraft
 ↓
Docking Port
 ↓
Station
```

---

# 92. Vehicle Building

O jogador pode construir:

```text
vehicle blueprint
```

a partir de:

```text
parts
materials
technology
skills
```

---

# 93. Vehicle Crafting

Crafting pode produzir:

```text
engine
wheel
battery
chassis
```

Vehicle System monta o veículo.

---

# 94. Technology

Progression pode desbloquear:

```text
advanced engines
electric propulsion
flight
spacecraft
automation
navigation
```

---

# 95. Research

Research pode descobrir:

```text
new propulsion
better materials
new fuel
new sensors
```

---

# 96. Social

Factions podem possuir:

```text
vehicle fleets
```

---

# 97. Civilization

Civilization pode controlar:

```text
transport network
rail network
shipping
aviation
spacecraft
```

---

# 98. Economy

Vehicles permitem:

```text
logistics
trade
transport
supply chains
```

---

# 99. Quest

Quest pode usar:

```text
escort
delivery
transport
repair
exploration
```

---

# 100. Combat

Combat pode utilizar:

```text
vehicle as combatant
vehicle weapon
vehicle armor
```

mas Combat resolve combate.

---

# 101. Vehicle as Combatant

Vehicle pode implementar capability:

```text
Combatant
```

sem o Vehicle System implementar dano.

---

# 102. Vehicle Sensor

Pode possuir:

```text
SensorModule
```

ligado ao futuro Sensor System.

---

# 103. Vehicle Communication

Pode possuir:

```text
Radio
Relay
Data Link
```

integrando Networking/Communication.

---

# 104. Vehicle Audio

Audio System recebe:

```text
RPM
speed
engine load
damage
environment
```

e produz áudio.

---

# 105. Vehicle Animation

Animation System recebe:

```text
wheel rotation
suspension
rotor
control surfaces
```

e gera pose.

---

# 106. Vehicle UI

UI pode mostrar:

```text
speed
fuel
energy
damage
navigation
cargo
```

---

# 107. Interaction

Interaction oferece:

```text
Enter
Exit
Drive
Repair
Refuel
Configure
Load
Unload
Dock
Inspect
```

---

# 108. Vehicle Commands

```text
CreateVehicleCommand
EnterVehicleCommand
ExitVehicleCommand
ControlVehicleCommand
RepairVehicleCommand
RefuelVehicleCommand
LoadCargoCommand
UnloadCargoCommand
InstallModuleCommand
RemoveModuleCommand
DockVehicleCommand
UndockVehicleCommand
```

---

# 109. Vehicle Events

```text
VehicleCreatedEvent
VehicleSpawnedEvent
VehicleMovedEvent
VehicleEnteredEvent
VehicleExitedEvent
VehicleDamagedEvent
VehicleDisabledEvent
VehicleRepairedEvent
VehicleRefueledEvent
VehicleDockedEvent
VehicleUndockedEvent
VehicleDestroyedEvent
```

---

# 110. Vehicle Registry

Registrar:

```text
VehicleDefinition
VehiclePart
PropulsionType
ControlType
SeatType
VehicleModule
VehicleCategory
```

---

# 111. Modding

Mods podem adicionar:

```text
car
ship
aircraft
rocket
drone
vehicle module
propulsion
fuel type
control system
```

sem modificar Vehicle Core.

---

# 112. Custom Vehicle Physics

Um mod pode registrar:

```text
CustomPropulsionModel
```

que fornece dados ao Physics System.

---

# 113. Scripted Vehicles

Scripts podem controlar:

```text
autopilot
mission
special behavior
```

mas através das APIs públicas.

---

# 114. Security

Cliente não pode simplesmente enviar:

```text
speed = 9000
```

e esperar que o servidor aceite.

Server calcula/valida a física.

---

# 115. Vehicle Anti-Cheat

Validar:

```text
control input
engine capabilities
mass
power
acceleration
environment
fuel
damage
position
```

---

# 116. Vehicle Ownership Security

Verificar:

```text
owner
seat permissions
cargo access
control permissions
```

---

# 117. Vehicle Duplication Protection

Criação/destruição utiliza:

```text
TransactionID
VehicleID
PersistentVehicleID
```

---

# 118. Vehicle Teleport

Teleporte deve passar pelo:

```text
Command System
+
Server
```

Não permitir mudança arbitrária da posição.

---

# 119. Vehicle Damage Exploits

Servidor não deve confiar no cliente sobre:

```text
damage
collision
destruction
```

---

# 120. Vehicle Performance

Vehicle System deve ser eficiente para:

```text
10 vehicles
100
1.000
10.000
100.000
```

---

# 121. LOD

```text
FULL
REGIONAL
ABSTRACT
```

### FULL

```text
physics
control
components
```

### REGIONAL

```text
traffic
route
cargo
```

### ABSTRACT

```text
fleet movement
economic transport
```

---

# 122. Distant Transport

Civilização distante pode simular:

```text
Cargo moved
Trade completed
Fuel consumed
```

sem simular cada roda.

---

# 123. Railway Example

Perto:

```text
Locomotive
+
50 wagons
+
physics
```

Longe:

```text
Train 842
Cargo: 12,000t
Route: A → B
ETA: ...
```

---

# 124. Spacecraft Example

Perto:

```text
ship physics
thrusters
modules
crew
```

Longe:

```text
ship transit state
origin
destination
ETA
```

---

# 125. Vehicle Scheduler

```text
Player vehicle
→ high frequency

Nearby AI vehicle
→ medium

Distant vehicles
→ low

Abstract logistics
→ event-driven
```

---

# 126. Vehicle Traffic

Traffic System pode existir no futuro.

Vehicle fornece:

```text
position
velocity
route
vehicle dimensions
```

---

# 127. Collision Groups

Vehicle pode usar grupos:

```text
vehicle-vehicle
vehicle-player
vehicle-world
vehicle-projectile
```

Physics determina colisões.

---

# 128. Vehicle Dimensions

Definir:

```text
length
width
height
wheelbase
clearance
```

para:

```text
collision
navigation
docking
rendering
```

---

# 129. Vehicle Attachments

Veículos podem anexar:

```text
trailer
cargo module
drone
wagon
tool
```

---

# 130. Vehicle Coupling

```text
Vehicle A
 ↓
Coupler
 ↓
Vehicle B
```

Muito importante para trens.

---

# 131. Train System

Uma composição:

```text
Train
├── Locomotive
├── Wagon
├── Wagon
├── Tank Car
└── Passenger
```

pode ser uma entidade lógica única composta de várias entidades físicas.

---

# 132. Vehicle Composition

```text
VehicleGroup
```

permite:

```text
train
convoy
ship fleet
space convoy
```

---

# 133. Vehicle Group Physics

O sistema pode tratar:

```text
logical group
```

enquanto Physics trabalha com componentes relevantes.

---

# 134. Large Vehicles

Para:

```text
airship
carrier
mobile factory
```

podemos ter:

```text
Vehicle
+
Structure
```

---

# 135. Vehicle Interiors

Interior pode ser:

```text
part of same spatial world
```

ou:

```text
instanced interior
```

dependendo da arquitetura.

O contrato deve permitir ambos.

---

# 136. Vehicle Dimension / Interior

Um veículo enorme pode possuir:

```text
outer world
+
interior space
```

mas não deve necessariamente virar uma Dimension do ponto de vista do jogador.

---

# 137. Moving World Problem

Construções dentro de veículos precisam acompanhar movimento.

Possível arquitetura:

```text
Vehicle Transform
 ↓
Vehicle Local Space
 ↓
Attached Structures/Entities
```

---

# 138. Attachment System

Isso merece uma abstração:

```text
IAttachment
```

para:

```text
player
cargo
turret
drone
trailer
structure
```

---

# 139. Moving Structure

Uma base móvel pode possuir:

```text
Vehicle
+
Structure
+
Machines
+
Inventory
+
Crew
```

Isso cria possibilidades enormes.

---

# 140. Mobile Factory

```text
Vehicle
 ↓
Structure
 ↓
Machines
 ↓
Energy
 ↓
Fluid
 ↓
Inventory
```

---

# 141. Mobile Colony

```text
Vehicle
+
Structure
+
Civilization
```

pode transportar população.

---

# 142. Vehicle + Space

Um spacecraft pode ser:

```text
Vehicle
+
Space Environment
+
Life Support
+
Structure
```

---

# 143. Vehicle + Dimensions

Traversal entre dimensões:

```text
Vehicle
 ↓
TravelRequest
 ↓
Dimension System
```

---

# 144. Vehicle + Far Lands

Railways/vehicles podem ser fundamentais para atravessar o mundo:

```text
Surface
 ↓
Rail
 ↓
Far Lands
 ↓
Frontier
```

---

# 145. Vehicle + Deep World

Podemos ter:

```text
subterranean train
underground rover
submarine
```

no Deep World.

---

# 146. Vehicle + Fluid

Submarines:

```text
buoyancy
pressure
ballast
```

Fluid + Physics executam.

---

# 147. Vehicle + Climate

Aircraft podem reagir a:

```text
wind
storms
temperature
air density
```

Climate fornece o ambiente.

---

# 148. Vehicle + Atmosphere

Flight physics consulta:

```text
air density
pressure
```

---

# 149. Vehicle + Audio

Áudio depende de:

```text
RPM
load
gear
damage
surface
environment
```

---

# 150. Vehicle + Animation

Animações:

```text
wheel
rotor
suspension
rudder
flaps
thrusters
```

---

# 151. Vehicle + UI

HUD:

```text
Speed
Altitude
Fuel
Energy
Temperature
Damage
Navigation
```

---

# 152. APIs

```text
IVehicleSystem
IVehicleDefinition
IVehicleInstance
IVehiclePart
IVehicleBlueprint
IVehicleController
IVehiclePropulsion
IVehiclePowertrain
IVehiclePassengerSystem
IVehicleCargo
IVehicleModuleHost
IVehicleDamage
IVehicleMaintenance
IVehicleNavigation
IVehicleDocking
IVehicleGroup
IVehicleOwnership
IVehiclePersistence
IVehicleReplication
```

---

# 153. Organização

```text
src/vehicle/

├── core/
│   ├── vehicle-system
│   ├── vehicle-definition
│   └── vehicle-instance
│
├── parts/
│   ├── part
│   ├── chassis
│   ├── wheels
│   ├── tracks
│   └── attachments
│
├── blueprint/
│   ├── blueprint
│   ├── builder
│   └── validator
│
├── control/
│   ├── control
│   ├── input
│   ├── ai
│   └── autopilot
│
├── propulsion/
│   ├── engine
│   ├── transmission
│   ├── wheel-drive
│   ├── propeller
│   ├── turbine
│   └── thruster
│
├── systems/
│   ├── power
│   ├── fuel
│   ├── cooling
│   └── mechanical
│
├── cargo/
├── passengers/
├── modules/
├── damage/
├── maintenance/
├── navigation/
├── docking/
├── groups/
├── ownership/
├── simulation/
├── networking/
├── persistence/
├── scripting/
├── mod/
├── metrics/
└── debug/
```

---

# 154. Dependências

```text
CORE
 │
 ├── Registry
 ├── Event Bus
 ├── Entity
 ├── Physics
 ├── Item
 ├── Energy
 ├── Fluid
 ├── Interaction
 ├── Command
 ├── Persistence
 └── Networking
        │
        ▼
     VEHICLE
        │
   ┌────┼──────────────────┐
   ▼    ▼                  ▼
CONTROL PROPULSION       MODULES
   │    │                  │
   └────┼──────────────────┘
        ▼
      PHYSICS
        │
   ┌────┼────────────┐
   ▼    ▼            ▼
 SPACE CLIMATE   ENVIRONMENT
        │
        ▼
   CIVILIZATION
        │
 ┌──────┼───────┐
 ▼      ▼       ▼
QUEST ECONOMY SOCIAL
```

---

# 155. Implementação por fases

## VEH-0 — Core

```text
Vehicle
VehicleDefinition
VehicleInstance
```

---

## VEH-1 — Parts

```text
Chassis
Wheel
Seat
Module
```

---

## VEH-2 — Basic Movement

Primeiro veículo:

```text
four wheels
steering
throttle
brake
```

---

## VEH-3 — Physics Integration

```text
Vehicle
 ↓
Physics
 ↓
Movement
```

---

## VEH-4 — Energy / Fuel

```text
Engine
 ↓
Energy/Fluid
```

---

## VEH-5 — Player Control

```text
Input
 ↓
Control
```

---

## VEH-6 — Interaction

```text
Enter
Exit
```

---

## VEH-7 — Inventory/Cargo

```text
Vehicle Cargo
 ↓
Inventory
```

---

## VEH-8 — Damage / Repair

```text
damage
 ↓
condition
 ↓
repair
```

---

## VEH-9 — Persistence

```text
save
load
```

---

## VEH-10 — Networking

```text
vehicle replication
prediction
reconciliation
```

---

## VEH-11 — AI Driving

```text
AI
 ↓
Navigation
 ↓
Control
```

---

## VEH-12 — Modular Vehicles

```text
blueprint
parts
modules
```

---

## VEH-13 — Rail

```text
locomotive
wagon
coupling
route
```

---

## VEH-14 — Water

```text
boat
submarine
buoyancy
```

---

## VEH-15 — Air

```text
aircraft
helicopter
flight
```

---

## VEH-16 — Space

```text
spacecraft
thrusters
orbit
docking
```

---

## VEH-17 — Convoys / Fleets

```text
vehicle groups
```

---

## VEH-18 — Logistics

```text
cargo
trade
transport
```

---

## VEH-19 — Advanced Automation

```text
autopilot
remote control
fleet management
```

---

# 156. Primeiro Vertical Slice

```text
PLAYER
 ↓
Vehicle
 ↓
Interaction
 ↓
Enter
 ↓
Input
 ↓
Control
 ↓
Engine
 ↓
Physics
 ↓
Vehicle moves
 ↓
Animation
 ↓
Audio
 ↓
Persistence
```

---

# 157. Segundo Vertical Slice

```text
Vehicle
 ↓
Engine
 ↓
Fuel Tank
 ↓
Fluid API
 ↓
Energy
 ↓
Physics
```

---

# 158. Terceiro Vertical Slice

```text
Vehicle
 ↓
Cargo
 ↓
Item System
 ↓
Trade
 ↓
Economy
```

---

# 159. Quarto Vertical Slice

```text
NPC
 ↓
Vehicle AI
 ↓
Navigation
 ↓
Control
 ↓
Physics
 ↓
Delivery
 ↓
Quest
```

---

# 160. Quinto Vertical Slice

```text
Locomotive
 ↓
Coupling
 ↓
Cargo Wagons
 ↓
Rail Network
 ↓
Trade Route
 ↓
Civilization
```

---

# 161. Sexto Vertical Slice

```text
Boat
 ↓
Water
 ↓
Buoyancy
 ↓
Fluid/Physics
 ↓
Navigation
 ↓
Port
```

---

# 162. Sétimo Vertical Slice

```text
Aircraft
 ↓
Atmosphere
 ↓
Flight
 ↓
Weather
 ↓
Navigation
```

---

# 163. Oitavo Vertical Slice

```text
Spacecraft
 ↓
Fuel
 ↓
Thrusters
 ↓
Physics
 ↓
Orbit
 ↓
Space Navigation
 ↓
Docking Station
```

---

# 164. Nono Vertical Slice

```text
Spacecraft
 ↓
Life Support
 ↓
Energy
 ↓
Fluid
 ↓
Crew
 ↓
Structure
 ↓
Space
```

---

# 165. Golden Vehicle Test

```text
BUILD VEHICLE
      ↓
SPAWN
      ↓
ENTER
      ↓
CONTROL
      ↓
MOVE
      ↓
USE FUEL
      ↓
DAMAGE
      ↓
REPAIR
      ↓
CARGO
      ↓
SAVE
      ↓
RESTART
      ↓
RESTORE
```

---

# 166. Golden Multiplayer Test

```text
SERVER
+
CLIENT A
+
CLIENT B

A enters vehicle
 ↓
server validates

A drives
 ↓
server simulates

B sees vehicle move
 ↓
A exits

Vehicle remains
 ↓
save
 ↓
restart
 ↓
vehicle restored
```

---

# 167. Golden Logistics Test

```text
Factory
 ↓
Cargo Vehicle
 ↓
Route
 ↓
City
 ↓
Unload
 ↓
Economy
 ↓
Trade completed
```

---

# 168. Golden Space Test

```text
Spacecraft
 ↓
Launch
 ↓
Orbit
 ↓
Navigation
 ↓
Travel
 ↓
Dock
 ↓
Unload Cargo
 ↓
Save
```

---

# 169. Security Test

Cliente tenta:

```text
velocity = impossible
```

Servidor:

```text
validate
 ↓
reject/correct
```

---

# 170. Duplication Test

```text
Cargo transfer
 ↓
disconnect
 ↓
reconnect
```

não pode duplicar carga.

---

# 171. Physics Exploit Test

```text
vehicle acceleration
```

não pode ultrapassar capacidades sem:

```text
valid propulsion
```

---

# 172. Large Vehicle Test

```text
Vehicle
+
Structure
+
Machines
+
Crew
+
Cargo
```

movendo-se como unidade coerente.

---

# 173. Performance Test

```text
100 vehicles
1.000
10.000
100.000
```

com:

```text
FULL
REGIONAL
ABSTRACT
```

---

# 174. Traffic Stress Test

```text
10.000 vehicles
+
roads
+
rail
+
ports
```

medindo:

```text
physics
navigation
replication
AI
```

---

# 175. Space Fleet Stress

```text
100 spacecraft
1.000
10.000
```

com:

```text
orbit
navigation
trade
```

e agregação para veículos distantes.

---

# 176. Vehicle AI Stress

```text
1.000 drivers
10.000
100.000 abstract transport agents
```

---

# 177. Mod Vehicle Test

Um mod adiciona:

```text
example:maglev
```

com:

```text
custom propulsion
custom module
custom control
```

sem alterar o Core.

---

# 178. Script Vehicle Test

```text
Script
 ↓
Autopilot
 ↓
Navigation
 ↓
Vehicle
```

---

# 179. Quest Vehicle Test

```text
Quest
 ↓
Deliver Cargo
 ↓
Vehicle
 ↓
Destination
 ↓
Cargo Delivered
 ↓
Reward
```

---

# 180. Civilization Vehicle Test

```text
Civilization
 ↓
Build Railway
 ↓
Purchase Locomotive
 ↓
Cargo Route
 ↓
Trade
 ↓
Economic Growth
```

---

# 181. Arquitetura final

```text
                            NEXORA
                               │
                         VEHICLE SYSTEM
                               │
       ┌───────────────────────┼────────────────────────┐
       ▼                       ▼                        ▼
   DEFINITION               INSTANCE                 BLUEPRINT
       │                       │                        │
       ▼                       ▼                        ▼
     PARTS                 COMPONENTS               VALIDATION
       │                       │                        │
       └───────────────────────┼────────────────────────┘
                               ▼
                            CONTROL
                               │
                     ┌─────────┼─────────┐
                     ▼         ▼         ▼
                   PLAYER      AI       SCRIPT
                     │         │         │
                     └─────────┼─────────┘
                               ▼
                           PROPULSION
                               │
                 ┌─────────────┼─────────────┐
                 ▼             ▼             ▼
              ENERGY         FLUID        MECHANICAL
                 │             │             │
                 └─────────────┼─────────────┘
                               ▼
                            PHYSICS
                               │
                   ┌───────────┼───────────┐
                   ▼           ▼           ▼
                GROUND       WATER         AIR
                   │           │           │
                   └───────────┼───────────┘
                               ▼
                              SPACE
                               │
                ┌──────────────┼──────────────┐
                ▼              ▼              ▼
              CARGO        PASSENGERS       MODULES
                │              │              │
                └──────────────┼──────────────┘
                               ▼
                             STATE
                               │
          ┌────────────────────┼─────────────────────┐
          ▼                    ▼                     ▼
        DAMAGE             MAINTENANCE           OWNERSHIP
          │                    │                     │
          └────────────────────┼─────────────────────┘
                               ▼
                         COMMAND / EVENT
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                ▼
           SERVER          NETWORKING       PERSISTENCE
```

E a separação definitiva:

```text
VEHICLE
→ o que é o veículo?

PARTS
→ do que ele é feito?

BLUEPRINT
→ como ele será construído?

CONTROL
→ o que ele está tentando fazer?

PROPULSION
→ como produz movimento?

ENERGY / FLUID
→ de onde vêm os recursos?

PHYSICS
→ o que realmente acontece fisicamente?

NAVIGATION
→ para onde deve ir?

MODULES
→ quais capacidades possui?

INTERACTION
→ como um ator usa o veículo?

COMMAND
→ qual ação está sendo solicitada?

SERVER
→ qual estado é autoritativo?

NETWORKING
→ quem precisa receber o estado?

PERSISTENCE
→ como o veículo sobrevive ao restart?
```

# O grande ciclo de veículos do NEXORA

```text
                  TECHNOLOGY
                      │
                      ▼
                  BLUEPRINT
                      │
                      ▼
                   CRAFTING
                      │
                      ▼
                    PARTS
                      │
                      ▼
                   VEHICLE
                      │
          ┌───────────┼───────────┐
          ▼           ▼           ▼
       ENERGY       FLUID       CONTROL
          │           │           │
          └───────────┼───────────┘
                      ▼
                   PHYSICS
                      │
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
    GROUND          WATER           AIR
       │              │              │
       └──────────────┼──────────────┘
                      ▼
                    SPACE
                      │
                 NAVIGATION
                      │
              ┌───────┴───────┐
              ▼               ▼
           CARGO           PASSENGERS
              │               │
              └───────┬───────┘
                      ▼
                  ECONOMY
                      │
                  CIVILIZATION
                      │
                   QUESTS
                      │
                    WORLD
                      │
                NEW DEMAND
                      │
                      └────────────→ TECHNOLOGY
```

A decisão que mais vale preservar é esta:

> **Um veículo não é apenas uma forma mais rápida de mover o player.**

No NEXORA, veículos podem virar **infraestrutura móvel do próprio mundo**:

```text
Player
  ↓
Carro

NPC
  ↓
Caminhão

Cidade
  ↓
Ferrovia

Civilização
  ↓
Rede logística

Exploração
  ↓
Navio / Rover

Indústria
  ↓
Fábrica móvel

Space
  ↓
Nave

Civilização espacial
  ↓
Frota + colônias + rotas
```

Isso conecta diretamente **Advanced Vehicles → Physics → Energy → Fluid → Structure → Economy → Civilization → Quest → Space**, sem transformar o Vehicle System em um sistema monolítico.
