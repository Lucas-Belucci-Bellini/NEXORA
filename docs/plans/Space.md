# NEXORA — SPACE SYSTEM

> **Princípio central:**
> **Space System transforma o espaço além do mundo em um ambiente jogável, simulável e persistente — sem transformar o universo em apenas uma sequência de “dimensões com céu estrelado”.**

No NEXORA, espaço deve ser uma extensão natural de:

```text id="space01"
WORLD
 ↓
ATMOSPHERE
 ↓
HIGH ALTITUDE
 ↓
ORBIT
 ↓
SPACE
 ↓
PLANETS
 ↓
MOONS
 ↓
ASTEROIDS
 ↓
STAR SYSTEMS
 ↓
INTERSTELLAR
 ↓
DIMENSIONAL FRONTIER
```

Mas existe uma separação fundamental:

```text id="space02"
DIMENSION
→ uma realidade/espaço lógico do NEXORA

PLANET
→ corpo celeste

ORBIT
→ relação orbital

STAR SYSTEM
→ conjunto astronômico

SPACE
→ domínio de simulação/transporte/interação espacial
```

---

# 1. O que é o Space System?

Ele gerencia a infraestrutura espacial:

```text id="space03"
Celestial Bodies
Orbits
Star Systems
Planets
Moons
Asteroids
Comets
Space Stations
Spacecraft
Space Lanes
Orbital Mechanics
Space Environment
Atmosphere Transition
Vacuum
Radiation
Temperature
Gravity Context
Navigation
Space Travel
Orbital Infrastructure
Landing / Launch
Interplanetary Travel
Interstellar Travel
Sensors
Space Communication
```

Mas não deve ser dono de:

```text id="space04"
Vehicle physics
Combat
Technology
Inventory
World generation
Economy
Civilization
```

Esses sistemas continuam especializados.

---

# 2. Arquitetura principal

```text id="space05"
                         SPACE SYSTEM
                              │
       ┌──────────────────────┼──────────────────────┐
       ▼                      ▼                      ▼
   ASTRONOMY              SPACE ENVIRONMENT       NAVIGATION
       │                      │                      │
       ▼                      ▼                      ▼
 CELESTIAL BODIES          VACUUM                ORBITS
       │                      │                      │
       └──────────────────────┼──────────────────────┘
                              ▼
                         SPACE TRAVEL
                              │
               ┌──────────────┼──────────────┐
               ▼              ▼              ▼
             ORBIT         PLANETARY      INTERSTELLAR
                              │
                              ▼
                          SPACECRAFT
                              │
                       VEHICLE SYSTEM
```

---

# 3. Space ≠ Dimension

Essa distinção precisa estar no projeto desde o começo.

Uma estação orbital pode ser:

```text id="space06"
Space Object
```

sem ser obrigatoriamente:

```text id="space07"
Dimension
```

Uma dimensão pode conter:

```text id="space08"
planet
space region
unique physics
```

e uma dimensão pode até representar um espaço abstrato.

---

# 4. Universe Model

Teremos uma estrutura:

```text id="space09"
Universe
├── Galaxy
│   ├── Star System
│   │   ├── Star
│   │   ├── Planet
│   │   ├── Moon
│   │   ├── Asteroid Belt
│   │   └── Stations
│   │
│   └── ...
└── ...
```

Não significa que toda entidade astronômica precise existir fisicamente em memória.

---

# 5. Hierarquia astronômica

```text id="space10"
Universe
↓
Galaxy
↓
StarSystem
↓
Star
↓
PlanetarySystem
↓
Planet
↓
Moon
```

Além disso:

```text id="space11"
Asteroid
Comet
ArtificialSatellite
SpaceStation
Megastructure
```

---

# 6. Celestial Body

```text id="space12"
CelestialBody
├── BodyID
├── Type
├── Mass
├── Radius
├── Position
├── Velocity
├── Rotation
├── Composition
├── Atmosphere
├── Gravity
├── OrbitalState
└── GenerationProfile
```

---

# 7. Body Types

```text id="space13"
STAR
PLANET
DWARF_PLANET
MOON
ASTEROID
COMET
ARTIFICIAL_BODY
STATION
MEGASTRUCTURE
ANOMALOUS_BODY
```

Mods podem adicionar tipos.

---

# 8. Star System

```text id="space14"
StarSystem
├── SystemID
├── Star
├── Bodies
├── Orbits
├── Resources
├── NavigationNodes
├── Infrastructure
└── GenerationVersion
```

---

# 9. Orbital Mechanics

Não precisamos começar com uma simulação astrodinâmica perfeita.

Devemos ter uma abstração:

```text id="space15"
OrbitalState
├── SemiMajorAxis
├── Eccentricity
├── Inclination
├── ArgumentOfPeriapsis
├── LongitudeOfAscendingNode
├── MeanAnomaly
└── Epoch
```

ou uma representação equivalente.

---

# 10. Determinismo

Órbitas geradas devem depender de:

```text id="space16"
UniverseSeed
SystemSeed
BodyID
GenerationVersion
```

Quando apropriado.

---

# 11. Orbit vs Position

Não confundir:

```text id="space17"
ORBIT
→ regra matemática

POSITION
→ estado atual
```

A posição pode ser derivada da órbita + tempo.

---

# 12. Time Integration

O Space System consulta:

```text id="space18"
WorldTime
SimulationTime
```

para atualizar corpos.

---

# 13. Não atualizar tudo por frame

Planetas distantes podem usar:

```text id="space19"
analytical orbital calculation
```

em vez de física completa.

---

# 14. Space LOD

Essencial para escala.

```text id="space20"
MICRO
LOCAL
REGIONAL
SYSTEM
GALACTIC
ABSTRACT
```

Ou conceitualmente:

```text id="space21"
FULL
REGIONAL
ABSTRACT
```

integrado ao restante do NEXORA.

---

# 15. Planetary Local Space

Quando o jogador está perto:

```text id="space22"
Planet
 ↓
Local Coordinate System
 ↓
Player
```

Isso evita problemas de precisão numérica em distâncias enormes.

---

# 16. Floating Origin

Muito importante.

Em vez de armazenar:

```text id="space23"
x = 7,000,000,000
```

podemos recentralizar:

```text id="space24"
WORLD ORIGIN
     ↓
PLAYER-CENTERED LOCAL FRAME
```

O universo continua mantendo coordenadas astronômicas separadamente.

---

# 17. Coordinate Systems

Teremos pelo menos:

```text id="space25"
Universal Coordinates
System Coordinates
Orbital Coordinates
Body Coordinates
Local World Coordinates
Vehicle Local Coordinates
```

---

# 18. Coordinate Transform

```text id="space26"
Universe
 ↓
Star System
 ↓
Planet
 ↓
Local World
 ↓
Chunk
 ↓
Voxel
```

---

# 19. Planet Surface

Planeta pode possuir:

```text id="space27"
World Instance
```

gerada pelo World Generation.

Então:

```text id="space28"
Space System
→ diz onde está o planeta

World System
→ gera a superfície
```

---

# 20. Atmosphere Transition

Fluxo:

```text id="space29"
SURFACE
 ↓
UPPER ATMOSPHERE
 ↓
EDGE OF SPACE
 ↓
SPACE
```

Climate/Atmosphere fornecem os dados ambientais.

---

# 21. Vacuum

Space Environment pode definir:

```text id="space30"
pressure
temperature
radiation
composition
visibility
```

---

# 22. Space Environment

```text id="space31"
SpaceEnvironment
├── Pressure
├── Temperature
├── Radiation
├── Microgravity
├── MagneticField
├── SolarFlux
├── Dust
├── Visibility
└── HazardProfile
```

---

# 23. Radiation

A radiação pode afetar:

```text id="space32"
equipment
electronics
living entities
materials
sensors
```

Mas os efeitos concretos pertencem aos sistemas correspondentes.

---

# 24. Gravity

Space System pode fornecer:

```text id="space33"
GravityContext
```

para Physics.

Exemplo:

```text id="space34"
Planet = 1.0g
Moon = 0.16g
Asteroid = very low
```

Physics executa as consequências.

---

# 25. Microgravity

Em espaço:

```text id="space35"
Physics
← Space gravity context
```

Isso permite:

```text id="space36"
floating objects
space movement
orbital vehicles
```

---

# 26. Planetary Gravity

Gravity pode mudar conforme a posição.

```text id="space37"
distance from body
 ↓
gravity context
```

---

# 27. Atmosphere Physics

Space System pode fornecer:

```text id="space38"
air density
pressure
```

Physics usa isso para:

```text id="space39"
drag
lift
fall
flight
```

---

# 28. Launch

Lançamento deve integrar:

```text id="space40"
Vehicle
Physics
Atmosphere
Energy
Fuel
Navigation
Space
```

Fluxo:

```text id="space41"
Launch Vehicle
 ↓
Atmosphere
 ↓
Physics
 ↓
Altitude
 ↓
Orbital State
```

---

# 29. Orbit Insertion

Um veículo pode entrar em:

```text id="space42"
SUBORBITAL
ORBITAL
ESCAPE
```

dependendo do estado físico.

---

# 30. Landing

```text id="space43"
SPACE
 ↓
ATMOSPHERE
 ↓
DESCENT
 ↓
LANDING
 ↓
PLANETARY WORLD
```

---

# 31. Spacecraft

Space System não deve virar Vehicle System.

Ele define o ambiente:

```text id="space44"
Vehicle System
→ vehicle mechanics

Space System
→ spatial/orbital context
```

---

# 32. Spacecraft State

Pode fornecer:

```text id="space45"
Orbit
Velocity
Fuel
Destination
Docking State
Navigation State
```

Mas fuel/energy vêm de:

```text id="space46"
Fluid / Energy / Inventory
```

---

# 33. Space Travel

Teremos uma abstração:

```text id="space47"
TravelRequest
```

com:

```text id="space48"
Origin
Destination
TravelMode
Vehicle
Actor
Route
```

---

# 34. Travel Modes

```text id="space49"
SURFACE
ATMOSPHERIC
ORBITAL
INTERPLANETARY
INTERSTELLAR
DIMENSIONAL
```

---

# 35. Interplanetary Travel

```text id="space50"
Planet A
 ↓
Orbit
 ↓
Transfer
 ↓
Planet B Orbit
 ↓
Atmosphere
 ↓
Surface
```

---

# 36. Travel não deve teletransportar magicamente sempre

Dependendo da tecnologia:

```text id="space51"
travel time
fuel
trajectory
navigation
hazards
```

podem importar.

---

# 37. Fast Travel

Um sistema de fast travel pode existir:

```text id="space52"
advanced propulsion
```

ou:

```text id="space53"
dimension technology
```

Mas como uma capability.

---

# 38. Navigation System

Space precisa de:

```text id="space54"
INavigation
```

capaz de:

```text id="space55"
locate body
calculate route
estimate travel
avoid hazard
```

---

# 39. Navigation Nodes

Para viagens grandes:

```text id="space56"
Star System
 ├── Node A
 ├── Node B
 └── Node C
```

---

# 40. Star System Graph

```text id="space57"
System A
  │
  ├── route
  │
System B
  │
  └── route
System C
```

---

# 41. Interstellar

Não precisamos simular cada metro entre estrelas.

Podemos:

```text id="space58"
local simulation
+
abstract travel representation
```

---

# 42. Interstellar Travel State

```text id="space59"
TRAVELING
├── Origin
├── Destination
├── ETA
├── Travel Method
├── Vehicle
└── Hazards
```

---

# 43. Travel as Simulation

Durante viagens longas:

```text id="space60"
FULL
→ spacecraft interior

ABSTRACT
→ interstellar transit

FULL
→ destination
```

---

# 44. Communication

Espaço precisa suportar:

```text id="space61"
radio
laser
data links
satellites
deep-space communication
```

Mas Communication System pode ser separado.

Space fornece:

```text id="space62"
distance
line of sight
environment
```

---

# 45. Communication Delay

Em distâncias grandes:

```text id="space63"
message
 ↓
propagation delay
```

Isso pode importar para:

```text id="space64"
remote bases
space factions
civilizations
```

---

# 46. Sensors

Spacecraft podem possuir:

```text id="space65"
radar
optical sensors
spectrometers
navigation sensors
radiation sensors
```

Sensor System futuro pode consumir os dados espaciais.

---

# 47. Detection

Não revelar automaticamente:

```text id="space66"
all planets
all asteroids
all stations
```

O conhecimento deve depender de:

```text id="space67"
sensor capability
observation
knowledge
maps
communication
```

---

# 48. Astronomy

Uma estação pode descobrir:

```text id="space68"
new body
anomaly
comet
asteroid
planet
```

através de observação.

Isso conecta diretamente com:

```text id="space69"
Research / Knowledge
```

---

# 49. Space Discovery Loop

```text id="space70"
OBSERVATORY
 ↓
OBSERVATION
 ↓
RESEARCH
 ↓
KNOWLEDGE
 ↓
NAVIGATION
 ↓
EXPEDITION
```

---

# 50. Resource Asteroids

Asteroides podem possuir:

```text id="space71"
Composition
Resource Profile
Structural Properties
Generation Data
```

World/resource systems determinam o conteúdo.

---

# 51. Space Mining

Interação:

```text id="space72"
Asteroid
 ↓
Mining Vehicle
 ↓
Tool
 ↓
Build/Destruction or resource extraction
 ↓
Item
```

---

# 52. Space Stations

Estruturas artificiais:

```text id="space73"
SpaceStation
```

usam:

```text id="space74"
Structure System
+
Space System
```

---

# 53. Station Graph

Uma estação pode possuir:

```text id="space75"
modules
docking ports
energy ports
fluid ports
living areas
manufacturing
```

---

# 54. Docking

Interaction:

```text id="space76"
Dock
 ↓
Vehicle
 ↓
Station
```

e Command:

```text id="space77"
DockVehicleCommand
```

---

# 55. Orbital Infrastructure

Pode existir:

```text id="space78"
satellite
station
relay
solar collector
shipyard
orbital elevator
```

---

# 56. Orbital Elevator

Pode conectar:

```text id="space79"
Surface
 ↕
Orbit
```

como Structure + Space infrastructure.

---

# 57. Space Industry

Machines podem funcionar no espaço:

```text id="space80"
refinery
factory
shipyard
research lab
```

usando:

```text id="space81"
Energy
Fluid
Machines
Crafting
```

---

# 58. Space Economy

Economy pode utilizar:

```text id="space82"
space resources
transport
stations
trade routes
fuel
```

---

# 59. Space Factions

Social/Factions pode possuir:

```text id="space83"
Space Faction
```

controlando:

```text id="space84"
stations
routes
colonies
resources
```

---

# 60. Space Civilization

Civilizations podem tornar-se:

```text id="space85"
planetary
multi-planetary
system-wide
interstellar
```

---

# 61. Civilization Progression

Progression/Technology pode definir:

```text id="space86"
rocketry
orbital technology
space habitats
advanced propulsion
```

---

# 62. Space Technology

Technology graph:

```text id="space87"
Rocketry
 ↓
Orbital Flight
 ↓
Spacecraft
 ↓
Interplanetary
 ↓
Advanced Propulsion
 ↓
Interstellar
```

Mas pode haver caminhos alternativos.

---

# 63. Alternative Space Paths

Por exemplo:

```text id="space88"
Chemical Propulsion
        │
        ▼
Conventional Space

OR

Energy Technology
        │
        ▼
Advanced Propulsion

OR

Dimensional Technology
        │
        ▼
Instant/Shortcut Transit
```

---

# 64. Space + Magic

Uma civilização pode usar:

```text id="space89"
magic propulsion
```

sem precisar de:

```text id="space90"
chemical rocket
```

desde que possua a capability correspondente.

---

# 65. Space + Research

Unknown phenomenon:

```text id="space91"
sensor detects anomaly
 ↓
Research
 ↓
new theory
 ↓
new technology
```

---

# 66. Space Anomalies

Podem existir:

```text id="space92"
gravity anomaly
energy anomaly
unknown object
dimensional distortion
ancient structure
```

---

# 67. Anomaly ≠ scripted quest

Uma anomalia pode simplesmente existir.

Depois:

```text id="space93"
Research
+
Quest
+
Exploration
```

podem surgir naturalmente.

---

# 68. Ancient Space Civilization

Uma civilização antiga pode deixar:

```text id="space94"
orbital ruins
stations
probes
artifacts
signals
```

O jogador encontra isso por pesquisa/exploração.

---

# 69. Space History

O universo pode possuir:

```text id="space95"
discovery history
exploration history
colonization history
wars
trade
technology
```

---

# 70. Space Archaeology

```text id="space96"
Ancient Station
 ↓
Observation
 ↓
Archaeology
 ↓
Knowledge
 ↓
Technology
```

---

# 71. Planetary Colonization

```text id="space97"
Scout
 ↓
Planet Evaluation
 ↓
Landing
 ↓
Base Construction
 ↓
Infrastructure
 ↓
Population
 ↓
Colony
```

---

# 72. Colony

Colony pode ser:

```text id="space98"
Settlement
```

administrado pelo Civilization/Settlement Systems.

Space fornece o contexto.

---

# 73. Planet Evaluation

Pode considerar:

```text id="space99"
atmosphere
gravity
temperature
water
resources
radiation
terrain
```

---

# 74. Terraforming

Terraforming é uma operação complexa que integra:

```text id="space100"
Climate
Atmosphere
Water
Vegetation
Terrain
Energy
Machines
```

Space apenas fornece o contexto planetário.

---

# 75. Terraforming stages

```text id="space101"
Atmosphere
 ↓
Temperature
 ↓
Water
 ↓
Soil
 ↓
Vegetation
 ↓
Ecosystem
```

---

# 76. Space Ecosystems

Planetas podem ter:

```text id="space102"
native ecology
alien ecology
synthetic ecology
```

Ecology System continua sendo o dono da simulação biológica.

---

# 77. Space Weather

Eventos:

```text id="space103"
solar storm
radiation event
meteor shower
dust cloud
eclipse
```

Climate/Atmosphere não precisam simular isso diretamente; Space fornece eventos ambientais.

---

# 78. Solar Events

Uma estrela pode gerar:

```text id="space104"
SolarEvent
```

que afeta:

```text id="space105"
radiation
communication
power generation
electronics
```

---

# 79. Space Energy

Star/solar infrastructure pode alimentar:

```text id="space106"
solar collectors
```

Energy API calcula energia.

---

# 80. Space Fluids

Spacecraft podem transportar:

```text id="space107"
fuel
coolant
oxygen
water
```

Fluid API permanece responsável pelo recurso.

---

# 81. Space Life Support

Pode existir uma camada de infraestrutura:

```text id="space108"
LifeSupport
├── Oxygen
├── Water
├── Temperature
├── Pressure
├── Waste
└── Emergency Systems
```

Mas os sistemas Fluid, Atmosphere, Energy e Health fornecem os componentes.

---

# 82. Life Support Failure

Pode ocorrer:

```text id="space109"
oxygen shortage
power loss
pressure failure
temperature issue
```

Isso gera:

```text id="space110"
World/Entity state
```

e outros sistemas reagem.

---

# 83. Crew

Entity System pode possuir:

```text id="space111"
Crew roles
```

e Social/AI decide comportamento.

---

# 84. Space Crew

NPCs podem:

```text id="space112"
operate station
repair ship
research
trade
```

---

# 85. Ship Roles

Profession/Skills:

```text id="space113"
Pilot
Engineer
Scientist
Medic
Navigator
Mechanic
```

---

# 86. Space Quest Generation

Quest System pode gerar:

```text id="space114"
survey planet
repair station
deliver cargo
investigate anomaly
rescue expedition
research artifact
establish colony
```

a partir do mundo.

---

# 87. Space Factions + Quest

```text id="space115"
Faction
 ↓
needs new orbital station
 ↓
construction quest
```

---

# 88. Space Economy + Quest

```text id="space116"
station requires fuel
 ↓
delivery contract
```

---

# 89. Space Research + Quest

```text id="space117"
anomaly
 ↓
research project
 ↓
quest
```

---

# 90. Navigation Knowledge

Descobrir uma estrela não significa que todos saibam.

```text id="space118"
Researcher
→ discovers system

Civilization
→ receives coordinates

Trader
→ learns route
```

---

# 91. Space Maps

Map System pode representar:

```text id="space119"
star map
planet map
orbital map
trade routes
navigation nodes
```

---

# 92. Space Navigation LOD

```text id="space120"
nearby:
precise trajectories

distant:
orbital abstraction

very distant:
system-level position
```

---

# 93. Huge Universe

Não precisamos armazenar:

```text id="space121"
trillions of physical objects
```

Podemos gerar:

```text id="space122"
procedurally
on demand
```

e manter somente estado persistente.

---

# 94. Procedural Star Systems

```text id="space123"
Universe Seed
 ↓
Galaxy Seed
 ↓
System Seed
 ↓
Star
 ↓
Planets
 ↓
Moons
 ↓
Asteroids
```

---

# 95. Persistence

Persistir:

```text id="space124"
discovered bodies
modified celestial objects
stations
colonies
orbital infrastructure
travel state
major events
```

---

# 96. Derived Data

Não necessariamente persistir:

```text id="space125"
orbital mesh
rendered sky
sensor cache
navigation cache
```

A Persistence System reconstrói.

---

# 97. Networking

Cliente não recebe:

```text id="space126"
universe
```

Inteiro.

Recebe:

```text id="space127"
relevant system
nearby bodies
visible objects
navigation information
```

---

# 98. Space Network Interest

```text id="space128"
Player
 ↓
Current Star System
 ↓
Relevant Bodies
 ↓
Nearby Ships
 ↓
Visible Stations
```

---

# 99. Interstellar Interest

Durante viagem:

```text id="space129"
Origin
Destination
Ship Interior
Travel State
Relevant Events
```

não todos os sistemas intermediários.

---

# 100. Server Authority

Servidor decide:

```text id="space130"
orbit state
travel
docking
landing
space structures
resource extraction
```

---

# 101. Space Physics

Physics recebe:

```text id="space131"
gravity
atmosphere
environment
```

e executa:

```text id="space132"
forces
motion
collision
```

---

# 102. Space Vehicle Physics

Vehicle System decide:

```text id="space133"
engine
thrust
control
```

Physics decide:

```text id="space134"
movement result
```

Space decide:

```text id="space135"
environment
orbital context
```

---

# 103. Combat in Space

Combat System continua separado:

```text id="space136"
Ship
 ↓
Combatant
 ↓
Combat
```

Space apenas fornece:

```text id="space137"
position
environment
navigation
```

---

# 104. Space Structures

Structure System pode representar:

```text id="space138"
stations
ships docks
orbital habitats
solar platforms
space elevators
shipyards
```

---

# 105. Space Building

Build & Destruction continua executando:

```text id="space139"
construction
modification
repair
```

---

# 106. Space Materials

Item/Block systems podem possuir:

```text id="space140"
vacuum-rated
radiation-resistant
temperature-resistant
```

como propriedades/capabilities.

---

# 107. Space Tools

Tool API pode possuir:

```text id="space141"
space mining tool
repair tool
survey tool
vacuum construction tool
```

---

# 108. Space Devices

Machines:

```text id="space142"
reactor
life support
refinery
oxygen generator
navigation computer
communication relay
```

---

# 109. Space Computing

Tecnologia pode desbloquear:

```text id="space143"
navigation computer
autopilot
sensor processing
```

---

# 110. Space Automation

Automation System pode controlar:

```text id="space144"
station
shipyard
mining drones
resource processing
```

---

# 111. Drones

Entity/Vehicle systems podem representar:

```text id="space145"
drone
```

Space fornece:

```text id="space146"
environment
navigation
```

---

# 112. Space Logistics

Economy/Logistics pode construir:

```text id="space147"
planet
→ orbit
→ station
→ asteroid
→ planet
```

rotas.

---

# 113. Trade Network

```text id="space148"
Planet A
 ↓
Station
 ↓
Asteroid Belt
 ↓
Station
 ↓
Planet B
```

---

# 114. Civilization Expansion

Uma civilização pode crescer:

```text id="space149"
Settlement
 ↓
Planetary Civilization
 ↓
Orbital Civilization
 ↓
Multi-Planetary Civilization
 ↓
Interstellar Civilization
```

---

# 115. Space Politics

Factions podem disputar:

```text id="space150"
stations
routes
asteroids
colonies
research
```

---

# 116. Space Diplomacy

```text id="space151"
Civilization A
 ↕
Civilization B
```

pode ter:

```text id="space152"
trade
navigation agreements
resource rights
research exchange
defense treaties
```

---

# 117. Space Knowledge

A descoberta de um planeta pode se tornar:

```text id="space153"
Knowledge Claim
```

como:

```text id="space154"
"Planet X has water."
```

---

# 118. Research formalization

```text id="space155"
Scan
 ↓
Observation
 ↓
Evidence
 ↓
Analysis
 ↓
Knowledge
```

---

# 119. Space Technology

```text id="space156"
Knowledge
 ↓
Research
 ↓
Propulsion
 ↓
Technology
 ↓
Travel Capability
```

---

# 120. Future Dimensions

Quando o NEXORA chegar à fronteira dimensional:

```text id="space157"
Space
 ↓
Dimensional anomaly
 ↓
Research
 ↓
Dimensional Technology
 ↓
Dimension System
```

Assim Space não precisa conhecer todas as 16 dimensões.

---

# 121. Space + Far Lands

O Far Lands pode conter:

```text id="space158"
observatories
launch sites
space infrastructure
frontier colonies
```

dependendo do conteúdo.

---

# 122. Space + Beyondlands

Beyondlands pode servir de:

```text id="space159"
frontier
industrial zone
launch region
exploration zone
```

---

# 123. Space as Frontier

A progressão espacial pode ser:

```text id="space160"
Surface
 ↓
High Atmosphere
 ↓
Orbit
 ↓
Moon
 ↓
Planetary System
 ↓
Other Stars
 ↓
Interstellar
 ↓
Unknown
```

---

# 124. Unknown Space

Não gerar tudo imediatamente.

Uma região pode ser:

```text id="space161"
UNKNOWN
```

até:

```text id="space162"
observed
mapped
visited
```

---

# 125. Discovery State

```text id="space163"
UNKNOWN
OBSERVED
DETECTED
MAPPED
VISITED
EXPLORED
COLONIZED
```

---

# 126. Space Cartography

Map data:

```text id="space164"
coordinates
routes
bodies
hazards
resources
stations
```

---

# 127. Navigation Infrastructure

Uma sociedade pode instalar:

```text id="space165"
beacons
relays
navigation stations
```

facilitando viagens futuras.

---

# 128. Travel Network Emergence

```text id="space166"
Discovery
 ↓
Station
 ↓
Trade Route
 ↓
More Traffic
 ↓
Infrastructure
 ↓
Colony
```

Isso liga Space a Civilization.

---

# 129. Orbital Economy Loop

```text id="space167"
Asteroid resources
 ↓
Mining
 ↓
Station
 ↓
Manufacturing
 ↓
Trade
 ↓
Planet
```

---

# 130. Space Research Loop

```text id="space168"
Anomaly
 ↓
Observation
 ↓
Research
 ↓
Knowledge
 ↓
Technology
 ↓
Space infrastructure
 ↓
New discoveries
```

---

# 131. Space Civilization Loop

```text id="space169"
Technology
 ↓
Spaceflight
 ↓
Exploration
 ↓
Resources
 ↓
Economy
 ↓
Colonies
 ↓
Population
 ↓
New research
 ↓
Advanced technology
```

---

# 132. APIs

```text id="space170"
ISpaceSystem
ISpaceEnvironment
ICelestialBody
ICelestialRegistry
IStarSystem
IOrbitalSystem
IOrbitResolver
ISpaceNavigator
ISpaceTravel
ISpaceRoute
ISpaceDiscovery
ISpaceSensorContext
ISpaceInfrastructure
ISpaceStation
IDockingSystem
IPlanetarySystem
ISpacePersistence
ISpaceSimulation
```

---

# 133. Registry

Registrar:

```text id="space171"
CelestialBodyType
StarType
PlanetType
OrbitalProfile
SpaceEnvironment
TravelMode
SpaceInfrastructure
NavigationNode
SpaceHazard
```

---

# 134. Organização

```text id="space172"
src/space/

├── core/
│   ├── space-system
│   ├── space-context
│   └── universe
│
├── astronomy/
│   ├── celestial-body
│   ├── star
│   ├── planet
│   ├── moon
│   ├── asteroid
│   └── system
│
├── orbit/
│   ├── orbit
│   ├── orbital-state
│   ├── resolver
│   └── propagation
│
├── coordinates/
│   ├── universal
│   ├── system
│   ├── body
│   └── local
│
├── environment/
│   ├── vacuum
│   ├── radiation
│   ├── gravity
│   └── solar
│
├── navigation/
│   ├── navigator
│   ├── route
│   ├── nodes
│   └── maps
│
├── travel/
│   ├── travel
│   ├── interplanetary
│   ├── interstellar
│   └── transfer
│
├── discovery/
│
├── infrastructure/
│   ├── station
│   ├── beacon
│   ├── relay
│   └── orbital
│
├── docking/
│
├── planetary/
│
├── simulation/
│   ├── full
│   ├── regional
│   └── abstract
│
├── networking/
├── persistence/
├── scripting/
├── mod/
├── metrics/
└── debug/
```

---

# 135. Dependências

```text id="space173"
REGISTRY
   │
EVENT BUS
   │
PERSISTENCE
   │
WORLD
   │
DIMENSION
   │
PROGRESSION
   │
   ▼
SPACE
   │
 ┌─┼───────────────┬───────────────┐
 ▼ ▼               ▼               ▼
ORBIT ENVIRONMENT NAVIGATION      TRAVEL
 │   │               │               │
 └───┴───────────────┼───────────────┘
                     ▼
                  VEHICLES
                     │
             ┌───────┼───────┐
             ▼       ▼       ▼
           ENTITY  PHYSICS STRUCTURE
             │       │       │
             └───────┼───────┘
                     ▼
                  GAMEPLAY
```

---

# 136. Implementação por fases

## SPACE-0 — Core

```text id="1w2k8c"
SpaceSystem
SpaceContext
CelestialBody
StarSystem
```

---

## SPACE-1 — Coordinates

```text id="9a5v1k"
universe
system
body
local
```

---

## SPACE-2 — Celestial Generation

```text id="lq8p37"
stars
planets
moons
asteroids
```

---

## SPACE-3 — Orbit

```text id="v1v9mu"
orbital states
trajectory
time
```

---

## SPACE-4 — Environment

```text id="isrql0"
vacuum
gravity
radiation
temperature
```

---

## SPACE-5 — Planet Transition

```text id="c98k13"
surface
→ atmosphere
→ orbit
```

---

## SPACE-6 — Navigation

```text id="7d5vdr"
map
discovery
route
```

---

## SPACE-7 — Travel

```text id="z3qujz"
orbital
interplanetary
```

---

## SPACE-8 — Vehicles

Integração com Vehicle/Physics.

---

## SPACE-9 — Stations

```text id="j6xj0r"
Structure
+
Space
```

---

## SPACE-10 — Docking

```text id="g24d9c"
ship
↕
station
```

---

## SPACE-11 — Discovery

```text id="cl0v7m"
sensor
→ observation
```

---

## SPACE-12 — Research

```text id="g7y7w0"
observation
→ research
→ knowledge
```

---

## SPACE-13 — Economy

```text id="2o5ojl"
trade
resources
routes
```

---

## SPACE-14 — Civilization

```text id="v5u6ze"
colonies
factions
infrastructure
```

---

## SPACE-15 — Interstellar

```text id="j15y2z"
star
→ star
```

---

## SPACE-16 — Galactic Scale

```text id="r6pij4"
galaxy abstraction
```

---

# 137. Primeiro Vertical Slice

```text id="space174"
Planet
 ↓
Atmosphere
 ↓
Launch Vehicle
 ↓
Physics
 ↓
Orbit
 ↓
Player sees planet
```

---

# 138. Segundo Vertical Slice

```text id="space175"
Orbit
 ↓
Navigation
 ↓
Moon
 ↓
Travel
 ↓
Moon Orbit
 ↓
Landing
 ↓
World Instance
```

---

# 139. Terceiro Vertical Slice

```text id="space176"
Asteroid
 ↓
Scan
 ↓
Observation
 ↓
Research
 ↓
Resource discovered
 ↓
Mining
 ↓
Item
```

---

# 140. Quarto Vertical Slice

```text id="space177"
Space Station
 ↓
Approach
 ↓
Dock
 ↓
Enter
 ↓
Structure
 ↓
Inventory
 ↓
Trade
```

---

# 141. Quinto Vertical Slice

```text id="space178"
Planet A
 ↓
Station
 ↓
Interplanetary route
 ↓
Planet B
 ↓
Cargo
 ↓
Economy
```

---

# 142. Sexto Vertical Slice

```text id="space179"
Unknown celestial object
 ↓
Sensor
 ↓
Observation
 ↓
Research
 ↓
Knowledge
 ↓
Quest
 ↓
Exploration
```

---

# 143. Sétimo Vertical Slice

```text id="space180"
Colony
 ↓
Infrastructure
 ↓
Population
 ↓
Faction
 ↓
Economy
 ↓
Research
 ↓
Space expansion
```

---

# 144. Oitavo Vertical Slice

```text id="space181"
Star A
 ↓
Star B
 ↓
Interstellar Travel
 ↓
New System
 ↓
New Civilization
```

---

# 145. Golden Space Test

```text id="space182"
PLANET
 ↓
LAUNCH
 ↓
ORBIT
 ↓
NAVIGATION
 ↓
TRAVEL
 ↓
MOON ORBIT
 ↓
LAND
 ↓
WORLD STATE
 ↓
SAVE
 ↓
RESTART
 ↓
RESTORE
```

---

# 146. Golden Discovery Test

```text id="space183"
SENSOR
 ↓
UNKNOWN OBJECT
 ↓
OBSERVATION
 ↓
RESEARCH
 ↓
KNOWLEDGE
 ↓
TECHNOLOGY
 ↓
NEW CAPABILITY
```

---

# 147. Golden Civilization Test

```text id="space184"
TECHNOLOGY
 ↓
SPACEFLIGHT
 ↓
EXPLORATION
 ↓
COLONY
 ↓
TRADE
 ↓
FACTION
 ↓
POLITICS
 ↓
NEW TECHNOLOGY
```

---

# 148. Stress Test

```text id="space185"
1 star system
10
100
1.000
100.000 generated bodies
```

mas somente os relevantes ficam em simulação detalhada.

---

# 149. Universe Scale Test

```text id="space186"
millions of star systems
```

podem existir proceduralmente sem serem todos carregados.

---

# 150. Travel Stress

```text id="space187"
100 spacecraft
1.000
10.000
```

com:

```text id="space188"
orbits
routes
navigation
communication
```

---

# 151. Colony Stress

```text id="space189"
1 colony
100
1.000
10.000
100.000
```

usando Settlement/Civilization LOD.

---

# 152. Networking Stress

```text id="space190"
100 players
+
many ships
+
stations
+
planetary systems
```

usando interest management.

---

# 153. Security Test

Cliente tenta:

```text id="space191"
teleport
planet B
```

sem capacidade.

Resultado:

```text id="space192"
DENIED
```

---

# 154. Persistence Test

```text id="space193"
Modify station
 ↓
Move asteroid resource
 ↓
Create colony
 ↓
Save
 ↓
Restart
 ↓
Everything persists
```

---

# 155. Architecture final

```text id="space194"
                           NEXORA
                              │
                           SPACE
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
    ASTRONOMY             ENVIRONMENT           NAVIGATION
        │                     │                     │
        ▼                     ▼                     ▼
 CELESTIAL BODIES          VACUUM                 ORBITS
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              ▼
                          SPACE TRAVEL
                              │
             ┌────────────────┼────────────────┐
             ▼                ▼                ▼
          ORBITAL        INTERPLANETARY     INTERSTELLAR
             │                │                │
             └────────────────┼────────────────┘
                              ▼
                          SPACECRAFT
                              │
                    ┌─────────┼─────────┐
                    ▼         ▼         ▼
                 VEHICLE    PHYSICS   ENTITY
                    │         │         │
                    └─────────┼─────────┘
                              ▼
                           STRUCTURE
                              │
                     ┌────────┼────────┐
                     ▼        ▼        ▼
                   STATION  COLONY   INFRASTRUCTURE
                     │        │        │
                     └────────┼────────┘
                              ▼
                          CIVILIZATION
                              │
                  ┌───────────┼───────────┐
                  ▼           ▼           ▼
               ECONOMY     RESEARCH     FACTIONS
                  │           │           │
                  └───────────┼───────────┘
                              ▼
                           WORLD
                              │
                           EVENT BUS
```

E as regras definitivas:

```text id="space195"
SPACE
→ define o domínio espacial

ASTRONOMY
→ descreve corpos e sistemas

ORBIT
→ descreve relações orbitais

ENVIRONMENT
→ descreve as condições do espaço

PHYSICS
→ calcula movimento e forças

VEHICLE
→ define como a nave funciona

NAVIGATION
→ calcula como chegar

TRAVEL
→ coordena deslocamento espacial

STRUCTURE
→ define estações e infraestrutura

WORLD
→ gera e simula superfícies planetárias

RESEARCH
→ transforma descobertas em conhecimento

PROGRESSION
→ transforma conhecimento em capacidades

CIVILIZATION
→ transforma capacidades em expansão

ECONOMY
→ movimenta recursos

SOCIAL / FACTIONS
→ organiza os participantes

QUEST
→ transforma acontecimentos em objetivos
```

## O grande ciclo espacial do NEXORA

```text id="space196"
                    UNKNOWN SPACE
                          │
                          ▼
                       SENSOR
                          │
                          ▼
                     OBSERVATION
                          │
                          ▼
                       RESEARCH
                          │
                          ▼
                       KNOWLEDGE
                          │
                          ▼
                      TECHNOLOGY
                          │
                          ▼
                       SPACECRAFT
                          │
                          ▼
                       EXPLORATION
                          │
               ┌──────────┴──────────┐
               ▼                     ▼
            RESOURCES             DISCOVERY
               │                     │
               ▼                     ▼
            ECONOMY                RESEARCH
               │                     │
               └──────────┬──────────┘
                          ▼
                       COLONY
                          │
                          ▼
                      CIVILIZATION
                          │
                          ▼
                       FACTIONS
                          │
                          ▼
                      POLITICS
                          │
                          ▼
                    INFRASTRUCTURE
                          │
                          ▼
                     NEW FRONTIER
                          │
                          └──────────→ UNKNOWN SPACE
```

A grande decisão aqui é que **o espaço não será simplesmente “mais um mapa gigantesco”**. Ele será outra escala de simulação: localmente pode haver física detalhada, naves, estações, astronautas e construções; em escalas muito maiores, sistemas estelares e civilizações podem ser representados de forma regional/abstrata. Isso permite que o NEXORA tenha uma progressão coerente de **superfície → órbita → sistema planetário → interplanetário → interestelar**, sem exigir que o servidor simule cada objeto do universo em detalhe o tempo inteiro.
