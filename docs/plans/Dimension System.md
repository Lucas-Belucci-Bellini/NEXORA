Perfeito. O **Dimension System** é especialmente importante no NEXORA porque aqui "dimensão" não deve ser apenas uma área separada com outro céu.

Ela precisa ser uma **instância de regras de mundo**.

A regra central:

> **Dimension System define o espaço lógico, as regras ambientais e as fronteiras de uma dimensão; World Generation cria seu mundo, os sistemas simulam seu estado e o Portal/Traversal System controla como entidades atravessam entre dimensões.**

# NEXORA — DIMENSION SYSTEM

## 1. Arquitetura central

```text id="dim-01"
                         NEXORA CORE
                              │
                      DIMENSION SYSTEM
                              │
       ┌──────────────────────┼──────────────────────┐
       ↓                      ↓                      ↓
  DIMENSION DEFINITION   DIMENSION INSTANCE      DIMENSION MANAGER
       │                      │                      │
       ↓                      ↓                      ↓
    RULES                  STATE                 LOADING
    LIMITS                 TIME                  STREAMING
    ATMOSPHERE             SEED                  LIFECYCLE
    GENERATION             ENTITIES              TRANSFER
```

A relação mais importante:

```text id="dim-02"
DimensionDefinition
        ↓
DimensionInstance
        ↓
World / Chunks / Entities
```

---

# 2. DimensionDefinition

Define **o que é uma dimensão**.

Exemplo conceitual:

```text id="dim-03"
DimensionDefinition

id
displayName
generator
worldRules
environment
atmosphere
climate
physicsProfile
lightingProfile
fluidProfile
mobProfile
spawnProfile
timeProfile
skyProfile
audioProfile
progressionProfile
travelProfile
```

---

# 3. DimensionID

Usar Registry:

```text id="dim-04"
nexora:overworld
nexora:void
nexora:deep_world
examplemod:crystal_realm
```

O ID é identidade pública.

---

# 4. Definition ≠ Instance

Isso é fundamental.

```text id="dim-05"
DimensionDefinition
        │
        ├── Instance A
        ├── Instance B
        └── Instance C
```

Exemplo:

```text id="dim-06"
examplemod:planet
```

poderia gerar mundos individuais:

```text id="dim-07"
planet instance 001
planet instance 002
```

---

# 5. DimensionInstance

Representa a dimensão realmente existente.

```text id="dim-08"
DimensionInstance

instanceId
definitionId
seed
worldVersion
generationVersion
timeState
environmentState
chunkState
entityState
dimensionState
```

---

# 6. Persistent Dimension ID

Cada instância pode possuir:

```text id="dim-09"
PersistentDimensionID
```

Isso permite que duas dimensões tenham a mesma definição, mas estados completamente diferentes.

---

# 7. DimensionManager

```text id="dim-10"
DimensionManager

create
load
unload
get
list
transfer
```

---

# 8. Dimension Registry

O Registry System passa a possuir:

```text id="dim-11"
DimensionRegistry
```

que registra definições.

---

# 9. Dimension Lifecycle

```text id="dim-12"
DEFINED
 ↓
REGISTERED
 ↓
INITIALIZING
 ↓
LOADING
 ↓
ACTIVE
 ↓
DORMANT
 ↓
UNLOADING
 ↓
UNLOADED
```

---

# 10. Active

Dimensão está sendo utilizada.

Pode possuir:

```text id="dim-13"
chunks
entities
simulation
climate
fluid
machines
```

---

# 11. Dormant

Dimensão existe, mas não está totalmente carregada.

Isso é importante para múltiplas dimensões.

```text id="dim-14"
Overworld
→ ACTIVE

Void
→ DORMANT
```

---

# 12. Unloaded

Nada precisa estar em memória.

O estado está no Persistence System.

---

# 13. Dimension Scope

Cada sistema pode saber:

```text id="dim-15"
dimensionId
dimensionInstanceId
```

Isso evita confundir:

```text id="dim-16"
"qual dimensão?"
```

com:

```text id="dim-17"
"qual instância dessa dimensão?"
```

---

# 14. Dimension Rules

Cada dimensão pode possuir regras próprias:

```text id="dim-18"
WorldRules

dayNight
respawn
damage
weather
mobSpawning
blockPlacement
fluidBehavior
physics
timeScale
```

---

# 15. Time Profile

Dimensões podem ter:

```text id="dim-19"
timeScale
dayLength
nightLength
calendar
```

Exemplo:

```text id="dim-20"
Overworld
1.0x

Other Dimension
0.5x

Special Dimension
2.0x
```

---

# 16. Não desacoplar Time do World Simulation

Dimension define política.

Simulation Scheduler executa.

```text id="dim-21"
Dimension
→ timeScale

Scheduler
→ actual execution
```

---

# 17. Day/Night Cycle

Dimension pode definir:

```text id="dim-22"
dayNightEnabled
dayDuration
nightDuration
sunPath
```

---

# 18. Custom Sky

Cada dimensão pode possuir:

```text id="dim-23"
SkyProfile
```

com:

```text id="dim-24"
sun
moon
stars
clouds
color grading
celestial bodies
```

---

# 19. Multiple Suns / Moons

O sistema não deve assumir:

```text id="dim-25"
1 sun
1 moon
```

Pode haver:

```text id="dim-26"
multiple celestial bodies
```

---

# 20. Celestial System

Criar:

```text id="dim-27"
CelestialProfile
```

que pode definir:

```text id="dim-28"
position
orbit
size
light
color
cycle
```

---

# 21. Atmosphere Profile

Cada dimensão pode ter:

```text id="dim-29"
AtmosphereProfile
```

com:

```text id="dim-30"
pressure
density
composition
temperature
visibility
```

---

# 22. Space

Uma dimensão pode ser:

```text id="dim-31"
atmosphere = vacuum
```

sem precisar de um sistema diferente.

---

# 23. Underwater Dimension

Também:

```text id="dim-32"
atmosphere
→ dense fluid environment
```

mas Fluid System continua responsável pela água.

---

# 24. Physics Profile

Dimensão pode definir:

```text id="dim-33"
gravity
terminalVelocity
frictionMultiplier
timeScale
```

---

# 25. Gravity Vector

Não assumir que gravidade sempre é:

```text id="dim-34"
(0, -1, 0)
```

Criar:

```text id="dim-35"
GravityProfile
```

com suporte a:

```text id="dim-36"
direction
magnitude
field
custom
```

---

# 26. Local Gravity

Uma dimensão pode possuir gravidade variável:

```text id="dim-37"
gravity field
```

ou regiões com gravidade diferente.

Physics resolve o campo.

---

# 27. Dimension Physics ≠ Physics System

Dimension define:

```text id="dim-38"
configuration
```

Physics simula:

```text id="dim-39"
forces
collisions
movement
```

---

# 28. Lighting Profile

Dimensão pode definir:

```text id="dim-40"
ambientLight
skyLight
colorTemperature
lightRules
```

Lighting System calcula a iluminação.

---

# 29. Climate Profile

Uma dimensão pode ter:

```text id="dim-41"
ClimatePolicy
```

ou até:

```text id="dim-42"
no climate
```

---

# 30. Climate Delegation

```text id="dim-43"
Dimension
→ climate configuration

Climate Engine
→ weather simulation
```

---

# 31. Weather Policy

Dimensão pode definir:

```text id="dim-44"
rain
snow
storms
fog
sandstorms
custom weather
```

---

# 32. Weather Disabled

Algumas dimensões:

```text id="dim-45"
weather = none
```

---

# 33. Fluid Profile

Dimensão pode definir:

```text id="dim-46"
fluid gravity
fluid behavior
pressure baseline
phase environment
```

Fluid Engine executa a simulação.

---

# 34. Fluid Availability

Uma dimensão pode ter:

```text id="dim-47"
water available
custom fluid
no liquid
```

como conteúdo/regras.

---

# 35. World Generation Profile

Dimension define qual generator usar:

```text id="dim-48"
WorldGeneratorID
```

---

# 36. Generator

World Generation System continua sendo o responsável por:

```text id="dim-49"
terrain
biomes
caves
ores
structures
vegetation
```

---

# 37. Dimension Generator

Exemplo:

```text id="dim-50"
Overworld
→ terrestrial generator

Void
→ void generator

Alien world
→ alien generator
```

---

# 38. Generator Context

```text id="dim-51"
DimensionGenerationContext

dimension
seed
generationVersion
worldPhase
```

---

# 39. Dimension Seed

Cada instância deve possuir:

```text id="dim-52"
dimensionSeed
```

Pode ser derivado do World Seed + Dimension ID de forma determinística ou explicitamente definido.

---

# 40. Seed Isolation

Duas dimensões não devem interferir acidentalmente em sua geração.

---

# 41. Chunk Coordinates

Cada dimensão possui seu próprio espaço de chunks.

```text id="dim-53"
Overworld
chunk (10,20)

Void
chunk (10,20)
```

são dados diferentes.

---

# 42. Storage Namespace

Persistence:

```text id="dim-54"
dimensions/<dimension-id>/
```

ou equivalente.

---

# 43. Dimension-local Registry References

Uma dimensão pode referenciar:

```text id="dim-55"
biomes
blocks
fluids
entities
structures
```

mas todos continuam vindo dos registries globais.

---

# 44. Mob Spawn Profile

Cada dimensão pode definir:

```text id="dim-56"
spawn rules
population
allowed species
spawn density
```

---

# 45. Ecology

Ecology System interpreta o ambiente.

```text id="dim-57"
Dimension
→ ecological constraints

Ecology
→ population dynamics
```

---

# 46. Civilization

Uma dimensão pode possuir:

```text id="dim-58"
civilization support
```

Exemplo:

```text id="dim-59"
Overworld
→ yes

Void
→ maybe

Pocket Dimension
→ controlled
```

Mas Civilization System continua independente.

---

# 47. Economy

Cada dimensão pode ter:

```text id="dim-60"
economic rules
```

como:

```text id="dim-61"
currency
trade restrictions
resource scarcity
```

---

# 48. Resource Rules

Exemplo:

```text id="dim-62"
iron abundant
rare crystal
unique resource
```

mas a geração real pertence ao WorldGen.

---

# 49. Progression Rules

Dimension pode definir:

```text id="dim-63"
progression requirements
```

Exemplo:

```text id="dim-64"
requires:
    technology_tier_4
    research:dimensional_travel
```

Progression System valida.

---

# 50. Access Policy

Criar:

```text id="dim-65"
DimensionAccessPolicy
```

pode definir:

```text id="dim-66"
public
restricted
quest
technology
admin
event
```

---

# 51. Teleportation

Dimension System coordena transferência.

Mas o movimento da entidade pertence ao Entity System.

Fluxo:

```text id="dim-67"
TransferRequest
 ↓
DimensionAccessPolicy
 ↓
Dimension System
 ↓
Entity Transfer
```

---

# 52. Dimension Transfer

Criar:

```text id="dim-68"
DimensionTransferRequest
```

com:

```text id="dim-69"
entity
sourceDimension
targetDimension
sourcePosition
targetPosition
cause
```

---

# 53. Transfer Result

```text id="dim-70"
ACCEPTED
DENIED
PENDING
FAILED
```

---

# 54. Transfer Pipeline

```text id="dim-71"
REQUEST
 ↓
VALIDATE
 ↓
SAVE SOURCE STATE
 ↓
LOAD TARGET
 ↓
RESOLVE SPAWN POSITION
 ↓
MOVE ENTITY
 ↓
UPDATE REFERENCES
 ↓
REPLICATE
 ↓
EVENT
```

---

# 55. Atomic Transfer

Não deixar a Entity existir simultaneamente em duas dimensões.

```text id="dim-72"
source
 ↓
detached
 ↓
target attached
```

---

# 56. Cross-Dimension Entity ID

Entity ID continua o mesmo.

```text id="dim-73"
Entity 12345
dimension = overworld

→ transfer

Entity 12345
dimension = void
```

---

# 57. Persistent Entity Identity

UUID não muda apenas porque mudou de dimensão.

---

# 58. Transfer Event

Publicar:

```text id="dim-74"
EntityDimensionTransferRequested
EntityDimensionTransferred
EntityDimensionTransferFailed
```

---

# 59. Portals

Portal não precisa estar dentro do Dimension System.

Portal é um mecanismo de conteúdo/interação que chama:

```text id="dim-75"
DimensionTransferAPI
```

---

# 60. Portal Definition

Pode possuir:

```text id="dim-76"
source dimension
target dimension
destination rules
activation conditions
```

---

# 61. Portal Position

Destino pode ser:

```text id="dim-77"
fixed
relative
computed
nearest safe
custom
```

---

# 62. Safe Spawn

Dimension System precisa de:

```text id="dim-78"
ISpawnResolver
```

para encontrar uma posição segura.

---

# 63. Spawn Resolver

Considera:

```text id="dim-79"
collision
terrain
fluid
height
environment
permissions
```

---

# 64. Dimension Boundaries

Uma dimensão pode ter limites:

```text id="dim-80"
minX
maxX
minY
maxY
minZ
maxZ
```

---

# 65. NEXORA Vertical Range

A arquitetura que já definimos pode continuar suportando:

```text id="dim-81"
Y = -1920
até
Y = +1920
```

para dimensões que usam esse espaço.

---

# 66. Per-Dimension Bounds

Outra dimensão pode usar:

```text id="dim-82"
different vertical range
```

sem alterar o Core.

---

# 67. Infinite Dimensions

Uma dimensão pode declarar:

```text id="dim-83"
unbounded
```

em determinados eixos.

---

# 68. Finite Dimensions

Pode existir:

```text id="dim-84"
finite
```

para:

```text id="dim-85"
pocket realm
arena
asteroid
generated dungeon world
```

---

# 69. World Border

Criar:

```text id="dim-86"
DimensionBoundary
```

mas Boundary System pode controlar detalhes.

---

# 70. Boundary Behavior

Pode ser:

```text id="dim-87"
BLOCK
WRAP
TELEPORT
VOID
CLAMP
CUSTOM
```

---

# 71. Wraparound

Uma dimensão pode possuir:

```text id="dim-88"
x max
→ x min
```

sem ser impossível na arquitetura.

---

# 72. Void Behavior

Outra pode possuir:

```text id="dim-89"
outside bounds
→ void
```

---

# 73. Far Lands

Far Lands devem **continuar sendo parte da mesma dimensão**, não necessariamente uma nova dimensão.

```text id="dim-90"
Overworld
 ├── Surface
 ├── Deep World
 ├── Far Lands
 └── Beyondlands
```

---

# 74. Beyondlands

Mesmo princípio:

```text id="dim-91"
frontier region
```

pode ser uma região especial dentro da dimensão.

---

# 75. Dimensional Frontier

A fronteira dimensional pode então ser:

```text id="dim-92"
Overworld
        ↓
Far Lands
        ↓
Beyondlands
        ↓
Dimensional Gate
        ↓
Other Dimension
```

---

# 76. Dimension Graph

Muito importante.

Criar:

```text id="dim-93"
DimensionGraph
```

representando relações:

```text id="dim-94"
Overworld
 ├── Nether-like realm
 ├── Void
 ├── Moon
 └── Alien Realm
```

---

# 77. Dimension Link

```text id="dim-95"
DimensionLink

source
target
travelMethod
requirements
cost
```

---

# 78. Travel Method

Exemplos:

```text id="dim-96"
portal
vehicle
teleporter
gate
natural
event
death
admin
```

---

# 79. Travel Requirements

```text id="dim-97"
technology
item
energy
research
quest
permission
location
```

---

# 80. Travel Cost

Pode exigir:

```text id="dim-98"
energy
fuel
time
cooldown
resource
```

---

# 81. Travel Policy

Dimension System fornece:

```text id="dim-99"
canTravel()
```

mas não executa energia/combustível.

---

# 82. Travel Transaction

Viagem pode ser:

```text id="dim-100"
TravelTransaction
```

para garantir atomicidade.

---

# 83. Inter-Dimension Network

Futuro:

```text id="dim-101"
dimension gates
```

podem formar uma rede.

---

# 84. Dimension Instance Creation

Algumas dimensões podem ser criadas dinamicamente:

```text id="dim-102"
createInstance(definition, seed)
```

---

# 85. Dynamic Dimensions

Exemplos:

```text id="dim-103"
player-created realm
server event world
temporary dungeon
generated planet
```

---

# 86. Persistent Dynamic Dimension

Pode permanecer depois que todos saem.

---

# 87. Temporary Dimension

Pode possuir:

```text id="dim-104"
expiration
cleanup policy
```

---

# 88. Dimension Templates

Pode existir:

```text id="dim-105"
DimensionTemplate
```

para criar instâncias.

---

# 89. Procedural Dimensions

Uma definição pode gerar dimensões conforme seed:

```text id="dim-106"
template
+
seed
→
dimension
```

---

# 90. Planet Generation

No futuro:

```text id="dim-107"
PlanetDefinition
 ↓
DimensionDefinition
 ↓
WorldGenerator
```

Isso permite usar o mesmo framework para planetas.

---

# 91. Celestial Hierarchy

Podemos representar:

```text id="dim-108"
Star System
 ├── Star
 ├── Planet
 │    └── Dimension
 └── Moon
      └── Dimension
```

Mas Astronomy System pode possuir a física orbital.

---

# 92. Dimension ≠ Planet

Uma coisa importante:

```text id="dim-109"
Planet
= corpo astronômico

Dimension
= espaço simulado/regra de mundo
```

Uma dimensão pode representar:

```text id="dim-110"
planet surface
interior
subspace
pocket realm
```

---

# 93. Multiple Dimension Spaces

Podemos futuramente ter:

```text id="dim-111"
Overworld Surface
Overworld Deep World
Moon
Moon Interior
Void
```

sem limitar a arquitetura.

---

# 94. Dimension Environment

Criar:

```text id="dim-112"
EnvironmentProfile
```

com:

```text id="dim-113"
atmosphere
temperature
lighting
gravity
fluid
weather
audio
```

---

# 95. Dimension Tags

```text id="dim-114"
#terrestrial
#habitable
#vacuum
#underground
#alien
#dimensional
#frontier
```

---

# 96. Dimension Capabilities

Uma dimensão pode fornecer:

```text id="dim-115"
IAtmosphereProvider
IClimateProvider
IGravityProvider
ITravelProvider
ISpawnProvider
```

---

# 97. Dimension Capability Example

```text id="dim-116"
Vacuum Dimension
→ no atmosphere

Ocean Dimension
→ fluid-dominated

HighGravity Dimension
→ custom gravity
```

---

# 98. Dimension Simulation Policy

Cada dimensão pode declarar:

```text id="dim-117"
simulationProfile
```

com:

```text id="dim-118"
tickScale
entityLOD
climateLOD
fluidLOD
civilizationLOD
```

---

# 99. Dimension LOD

Uma dimensão distante pode ter:

```text id="dim-119"
ABSTRACT
```

quando nenhum player está presente.

---

# 100. Cross-Dimension Simulation

Exemplo:

```text id="dim-120"
Player in Overworld
Void still simulates abstractly
```

---

# 101. Dormant Simulation

Dimension Manager pode manter:

```text id="dim-121"
abstract state
```

em vez de chunks completos.

---

# 102. Dimension Scheduler

Não criar scheduler separado completo.

Usar:

```text id="dim-122"
Global Scheduler
+
Dimension Simulation Profile
```

---

# 103. Dimension Tick

```text id="dim-123"
Scheduler
 ↓
dimension profile
 ↓
systems update
```

---

# 104. Cross-Dimension Time

Uma dimensão pode avançar mais rápido que outra.

Isso é possível sem alterar o World Clock global, desde que o sistema defina claramente:

```text id="dim-124"
global time
local simulation time
```

---

# 105. Global vs Local Time

Eu separaria:

```text id="dim-125"
World Time
```

de:

```text id="dim-126"
Dimension Local Time
```

quando necessário.

---

# 106. Time Conversion

```text id="dim-127"
global tick
 ↓
dimension time scale
 ↓
local dimension tick
```

---

# 107. Dimension Weather State

Persistence salva:

```text id="dim-128"
persistent weather state
```

Climate reconstrói detalhes transitórios.

---

# 108. Dimension Audio

Audio System pode consumir:

```text id="dim-129"
DimensionAudioProfile
```

---

# 109. Dimension Lighting

Lighting usa:

```text id="dim-130"
DimensionLightingProfile
```

---

# 110. Dimension UI

UI pode mostrar:

```text id="dim-131"
dimension name
coordinates
environment
travel options
```

---

# 111. Dimension Map

Map System precisa conhecer:

```text id="dim-132"
dimension
```

para não misturar mapas.

---

# 112. Dimension Map Layers

Atlas pode mostrar:

```text id="dim-133"
Overworld map
Deep World map
Void map
```

---

# 113. Dimension Coordinates

UI pode mostrar:

```text id="dim-134"
X
Y
Z
Dimension
```

---

# 114. Coordinate Conversion

Entre dimensões:

```text id="dim-135"
source position
 ↓
DimensionLink
 ↓
destination position
```

---

# 115. Relative Transfer

Algumas dimensões podem preservar proporção:

```text id="dim-136"
scale
offset
rotation
```

---

# 116. Dimension Transform

Criar:

```text id="dim-137"
DimensionTransform
```

com:

```text id="dim-138"
scale
translation
rotation
```

---

# 117. Portal Mapping

Portal pode utilizar:

```text id="dim-139"
source transform
+
destination transform
```

---

# 118. Multi-axis Rotation

Não assumir que dimensões compartilham orientação.

---

# 119. Gravity Alignment

Se destino possui gravidade diferente:

```text id="dim-140"
Entity
 ↓
transform conversion
 ↓
gravity adaptation
```

Physics/Entity resolvem detalhes.

---

# 120. Dimension Spawn

Cada dimensão pode definir:

```text id="dim-141"
SpawnProfile
```

---

# 121. Spawn Rules

Pode determinar:

```text id="dim-142"
initialSpawn
respawn
fallbackSpawn
safeSpawn
```

---

# 122. Respawn

Player System solicita:

```text id="dim-143"
getRespawnLocation()
```

Dimension responde via policy/provider.

---

# 123. Bed / Base / Civilization Spawn

Spawn provider pode consultar:

```text id="dim-144"
player preference
settlement
bed
anchor
faction
```

---

# 124. Dimension Hazards

Uma dimensão pode declarar:

```text id="dim-145"
environmental hazards
```

como:

```text id="dim-146"
radiation
vacuum
extreme temperature
corrosive atmosphere
dimensional instability
```

---

# 125. Hazard System

Dimension define configuração.

Hazard System aplica efeitos.

---

# 126. Dimension Protection

Pode existir:

```text id="dim-147"
DimensionProtectionPolicy
```

para:

```text id="dim-148"
build
break
combat
teleport
```

---

# 127. Admin Dimensions

Servidor pode criar:

```text id="dim-149"
admin dimension
```

para testes.

---

# 128. Arena Dimensions

Uma dimensão pode ser:

```text id="dim-150"
PVP arena
```

com regras especiais.

---

# 129. Event Dimensions

World Events podem criar dimensões temporárias:

```text id="dim-151"
event realm
```

---

# 130. Dungeon Dimensions

Procedural dungeons podem utilizar:

```text id="dim-152"
temporary dimension instance
```

---

# 131. Dimension Persistence

Cada instance precisa de:

```text id="dim-153"
metadata
world state
chunk references
entities
simulation state
```

---

# 132. Persistence Integration

```text id="dim-154"
Dimension System
 ↓
Persistence
```

salva:

```text id="dim-155"
definitionId
instanceId
seed
state
version
```

---

# 133. Registry Integration

Ao carregar:

```text id="dim-156"
definitionId
 ↓
DimensionRegistry
 ↓
definition
```

---

# 134. Unknown Dimension

Se uma dimensão de mod não existir:

```text id="dim-157"
MissingDimension
```

não apagar automaticamente.

---

# 135. Missing Dimension State

Pode preservar:

```text id="dim-158"
definitionId
rawData
seed
persistent state
```

para recuperação futura.

---

# 136. Migration

Dimension schema também precisa:

```text id="dim-159"
version
migration
```

---

# 137. Dimension Version

```text id="dim-160"
dimensionVersion
environmentVersion
generationVersion
```

---

# 138. World Version

Separado:

```text id="dim-161"
worldFormatVersion
```

---

# 139. Dimension Compatibility

Antes de carregar:

```text id="dim-162"
definition exists?
generator exists?
rules valid?
version compatible?
```

---

# 140. Dimension Fingerprint

Pode gerar:

```text id="dim-163"
DimensionFingerprint
```

para detectar mudança de definição.

---

# 141. Dynamic Definition Warning

Se a definição mudou drasticamente:

```text id="dim-164"
generation mismatch
```

não regenerar chunks existentes silenciosamente.

---

# 142. Dimension State vs World State

Dimension mantém apenas:

```text id="dim-165"
dimension-wide state
```

World/Chunk mantém:

```text id="dim-166"
spatial state
```

---

# 143. Dimension Global State

Pode conter:

```text id="dim-167"
time
weather summary
world events
global modifiers
population summary
special flags
```

---

# 144. Region State

Region:

```text id="dim-168"
regional simulation state
```

---

# 145. Chunk State

Chunk:

```text id="dim-169"
local block/entity state
```

---

# 146. Hierarchy

```text id="dim-170"
Dimension
 ↓
Region
 ↓
Chunk
 ↓
Block / Entity
```

---

# 147. Dimension Events

Eventos:

```text id="dim-171"
DimensionCreated
DimensionLoaded
DimensionActivated
DimensionDormant
DimensionUnloaded
DimensionDestroyed
DimensionWeatherChanged
DimensionTimeChanged
DimensionTransferRequested
DimensionTransferred
```

---

# 148. DimensionDestroyed

Somente dimensões temporárias podem permitir destruição.

---

# 149. Cleanup

Antes de destruir:

```text id="dim-172"
save
release entities
release chunks
release caches
remove subscriptions
```

---

# 150. Dimension Ownership

Cada instance possui:

```text id="dim-173"
owner
```

quando aplicável:

```text id="dim-174"
server
player
event
system
```

---

# 151. Dimension Permissions

Podem definir:

```text id="dim-175"
who can enter
who can build
who can modify
who can destroy
```

---

# 152. Security

Cliente nunca pode simplesmente informar:

```text id="dim-176"
teleport to secret dimension
```

e esperar que servidor aceite.

---

# 153. Server Authority

```text id="dim-177"
Client
 ↓
TravelRequest
 ↓
Server
 ↓
DimensionAccessPolicy
 ↓
Transfer
```

---

# 154. Network Replication

Replicar:

```text id="dim-178"
dimension ID
instance ID
position
orientation
```

mas não toda a dimensão.

---

# 155. Dimension Interest

Networking pode usar dimensão como primeiro filtro:

```text id="dim-179"
client in overworld
```

não deve receber entidades:

```text id="dim-180"
void
```

---

# 156. Cross-Dimension Chat

Chat pode possuir escopos:

```text id="dim-181"
dimension
world
global
```

mas Chat System controla.

---

# 157. Audio

Audio usa:

```text id="dim-182"
dimension profile
```

---

# 158. UI

UI usa:

```text id="dim-183"
dimension metadata
```

---

# 159. Command System

Comandos podem especificar:

```text id="dim-184"
/teleport <dimension>
```

Command System chama Dimension API.

---

# 160. Command ≠ Dimension Logic

Dimension System apenas valida/executa transferência.

---

# 161. Dimension Query API

```text id="dim-185"
IDimensionQuery
```

pode responder:

```text id="dim-186"
getDefinition
getInstance
getEnvironment
getRules
getSpawn
canTravel
```

---

# 162. Dimension Manager API

```text id="dim-187"
IDimensionManager

load()
unload()
create()
destroy()
get()
list()
transfer()
```

---

# 163. Travel API

```text id="dim-188"
IDimensionTravel

canTravel()
resolveDestination()
transfer()
```

---

# 164. Environment API

```text id="dim-189"
IDimensionEnvironment

getAtmosphere()
getGravity()
getClimate()
getLighting()
```

---

# 165. Spawn API

```text id="dim-190"
IDimensionSpawn

resolveSpawn()
resolveRespawn()
```

---

# 166. Lifecycle API

```text id="dim-191"
IDimensionLifecycle

activate()
deactivate()
load()
unload()
```

---

# 167. Definition API

```text id="dim-192"
IDimensionDefinition
```

---

# 168. Dimension Context

```text id="dim-193"
DimensionContext

definition
instance
world
scheduler
registry
persistence
events
```

---

# 169. Dimension Service Boundary

Dimension System fornece:

```text id="dim-194"
identity
instance lifecycle
rules
environment profile
bounds
travel
spawn
simulation configuration
```

---

# 170. Não faz

```text id="dim-195"
WorldGen
Physics simulation
Climate simulation
Fluid simulation
Entity lifecycle
Rendering
Audio
AI
Economy
Civilization
```

---

# 171. WorldGen Integration

```text id="dim-196"
Dimension Definition
 ↓
World Generator
 ↓
Chunks
```

---

# 172. Entity Integration

```text id="dim-197"
Dimension
 ↓
EntityManager
 ↓
entities assigned to dimension
```

---

# 173. Block Integration

```text id="dim-198"
Dimension
 ↓
World/Chunk
 ↓
Blocks
```

---

# 174. Item Integration

Itens não pertencem necessariamente a uma dimensão até estarem:

```text id="dim-199"
stored
equipped
or physically spawned
```

---

# 175. BlockEntity Integration

BlockEntities pertencem ao chunk da dimensão.

---

# 176. Animation

Animation só precisa saber:

```text id="dim-200"
gravity orientation
environment state
```

quando relevante.

---

# 177. Audio

Audio recebe:

```text id="dim-201"
DimensionAudioProfile
```

---

# 178. UI

UI pode exibir:

```text id="dim-202"
DimensionDisplayData
```

---

# 179. Registry

```text id="dim-203"
DimensionRegistry
```

registra definitions.

---

# 180. Event Bus

```text id="dim-204"
Dimension events
```

são publicados.

---

# 181. Persistence

```text id="dim-205"
Dimension state
```

é persistido.

---

# 182. Mod API

Mods podem:

```text id="dim-206"
registerDimension()
registerEnvironment()
registerTravelProvider()
registerSpawnProvider()
registerDimensionRules()
```

---

# 183. Official Dimension

A dimensão oficial:

```text id="dim-207"
nexora:overworld
```

usa exatamente a mesma API pública.

---

# 184. Void

```text id="dim-208"
nexora:void
```

também.

Não deve existir:

```text id="dim-209"
special private void engine
```

---

# 185. Future Dimensions

A arquitetura deve suportar:

```text id="dim-210"
Nether-like
End-like
Moon
Mars-like
Ocean World
Alien
Pocket Realm
Magic Realm
Machine Realm
Void
```

sem criar novos sistemas fundamentais.

---

# 186. Sixteen Dimensions

Como já definimos uma arquitetura de até **16 dimensões**, podemos simplesmente registrar 16 `DimensionDefinition`s.

O número não deve estar hardcoded no Core.

```text id="dim-211"
16 dimensions
= content/configuration
```

não:

```text id="dim-212"
for (i = 0; i < 16; i++)
```

---

# 187. Dimension Slot

Se necessário:

```text id="dim-213"
DimensionInstanceID
```

pode ser arbitrário.

---

# 188. Unlimited Instances

A arquitetura pode suportar:

```text id="dim-214"
16 definitions
+
many instances
```

sem confundir os dois limites.

---

# 189. Dimension Template

Exemplo:

```text id="dim-215"
"Planet Template"
```

pode criar:

```text id="dim-216"
planet_001
planet_002
planet_003
```

---

# 190. Procedural Dimension Creation

Pipeline:

```text id="dim-217"
DimensionTemplate
 ↓
Seed
 ↓
WorldGen
 ↓
DimensionInstance
 ↓
Persistence
```

---

# 191. Dimension State Machine

```text id="dim-218"
CREATED
 ↓
INITIALIZING
 ↓
LOADING
 ↓
ACTIVE
 ↓
DORMANT
 ↓
UNLOADING
 ↓
UNLOADED
```

---

# 192. Dimension Manager State

Manager precisa saber:

```text id="dim-219"
loaded
loading
queued
dormant
unloading
failed
```

---

# 193. Async Loading

Transferência:

```text id="dim-220"
request
 ↓
load target asynchronously
 ↓
prepare
 ↓
transfer
```

---

# 194. Preloading

Portal pode pré-carregar:

```text id="dim-221"
target dimension
```

para reduzir espera.

---

# 195. Dimension Streaming

Somente partes necessárias:

```text id="dim-222"
Dimension
 ↓
Region
 ↓
Chunk
```

---

# 196. Cross-Dimension Streaming

Não carregar a dimensão inteira ao entrar.

---

# 197. Failure During Transfer

Se target falhar:

```text id="dim-223"
transfer failed
```

entidade deve permanecer segura na origem.

---

# 198. Transfer Rollback

```text id="dim-224"
source state
 ↓
target load
 ↓
failure
 ↓
restore source
```

---

# 199. Dimension Desync

Network deve validar:

```text id="dim-225"
server dimension
client dimension
```

---

# 200. Client Loading Screen

UI pode exibir:

```text id="dim-226"
Loading Dimension...
Generating...
Streaming...
```

---

# 201. Progress Reporting

Dimension Manager pode fornecer:

```text id="dim-227"
load progress
```

para UI.

---

# 202. Generation vs Loading

Diferenciar:

```text id="dim-228"
GENERATING
```

e:

```text id="dim-229"
LOADING
```

porque são operações diferentes.

---

# 203. Dimension Cache

Dimensões recentemente usadas podem permanecer parcialmente em cache.

---

# 204. Memory Budget

Dimension Manager deve ter:

```text id="dim-230"
max active dimensions
memory budget
load priority
```

---

# 205. LOD Dimension

```text id="dim-231"
FULL
REGIONAL
ABSTRACT
```

---

# 206. Abstract Dimension

Quando ninguém está presente:

```text id="dim-232"
player absent
 ↓
aggregate simulation
```

---

# 207. Multi-Dimension Simulation

O Scheduler pode executar:

```text id="dim-233"
Overworld FULL
Void REGIONAL
Remote Planet ABSTRACT
```

simultaneamente.

---

# 208. Dimension Importance

Prioridade:

```text id="dim-234"
player present
quest active
world event
civilization active
background
```

---

# 209. Dimension Event Priority

Eventos em dimensões inativas podem ser agregados.

---

# 210. Dimension Wake

Quando jogador entra:

```text id="dim-235"
ABSTRACT
 ↓
REGIONAL
 ↓
FULL
```

---

# 211. Dimension Sleep

Quando todos saem:

```text id="dim-236"
FULL
 ↓
REGIONAL
 ↓
ABSTRACT
```

---

# 212. Persistence + Sleep

Antes de reduzir LOD:

```text id="dim-237"
checkpoint
 ↓
aggregate
 ↓
persist
```

---

# 213. Dimension History

Eventos importantes:

```text id="dim-238"
creation
destruction
major events
discoveries
```

podem alimentar World History.

---

# 214. Dimension Discoverability

Algumas dimensões podem estar:

```text id="dim-239"
unknown
discovered
accessible
known
```

---

# 215. Knowledge Integration

Knowledge System pode registrar:

```text id="dim-240"
"dimension discovered"
```

---

# 216. Quest Integration

Quest pode exigir:

```text id="dim-241"
visit dimension
discover location
activate portal
```

---

# 217. Progression Integration

Technology/Research pode desbloquear:

```text id="dim-242"
dimension travel
```

---

# 218. Economy Integration

Dimension travel pode exigir:

```text id="dim-243"
fuel
energy
rare resource
```

---

# 219. Civilization Integration

Civilizações podem:

```text id="dim-244"
discover dimension
establish outpost
create trade
migrate
```

---

# 220. Transport Integration

Naves/vehicles podem atravessar dimensões através de:

```text id="dim-245"
travel providers
```

---

# 221. Space Integration

Spacecraft pode representar:

```text id="dim-246"
travel between planetary dimensions
```

mas Navigation/Space System executa detalhes.

---

# 222. Travel Provider

Criar:

```text id="dim-247"
IDimensionTravelProvider
```

---

# 223. Travel Providers

```text id="dim-248"
PortalTravel
ShipTravel
TeleportTravel
EventTravel
AdminTravel
```

---

# 224. Travel Calculation

Provider resolve:

```text id="dim-249"
destination
cost
time
transform
```

---

# 225. Dimension Link Registry

Pode existir:

```text id="dim-250"
DimensionLinkRegistry
```

registrando conexões.

---

# 226. Link Dynamic

Links podem ser criados pelo jogo:

```text id="dim-251"
portal opened
```

e removidos.

---

# 227. Link Persistence

Links permanentes devem ser salvos.

---

# 228. Temporary Links

Eventos podem criar:

```text id="dim-252"
temporary link
```

---

# 229. Dimension Security

Acesso a dimensões pode depender de:

```text id="dim-253"
server permissions
progression
quest
technology
event
```

---

# 230. Dimension API Security

Mods não devem poder:

```text id="dim-254"
delete arbitrary dimension
```

sem permissão.

---

# 231. Debug Commands

```text id="dim-255"
nexora dimension list
nexora dimension inspect
nexora dimension load
nexora dimension unload
nexora dimension create
nexora dimension remove
nexora dimension travel
nexora dimension seed
nexora dimension graph
```

---

# 232. Dimension Graph Debug

Visualização:

```text id="dim-256"
Overworld
   │
   ├── Moon
   │
   ├── Void
   │
   └── Alien Realm
```

---

# 233. Dimension Inspector

Mostrar:

```text id="dim-257"
definition
instance
seed
state
loaded regions
entities
time
environment
rules
memory
```

---

# 234. Load Profiler

```text id="dim-258"
generation time
load time
stream time
memory
```

---

# 235. Transfer Profiler

```text id="dim-259"
validation
target load
entity serialization
network
latency
```

---

# 236. Tests

Testar:

```text id="dim-260"
registration
creation
loading
unloading
persistence
migration
travel
spawn
bounds
rules
missing dimensions
mod dimensions
```

---

# 237. Cross-Dimension Test

```text id="dim-261"
Overworld
 ↓
Portal
 ↓
Dimension B
 ↓
Entity transfer
 ↓
save
 ↓
logout
 ↓
reload
 ↓
Dimension B
```

---

# 238. Transfer Failure Test

```text id="dim-262"
source
 ↓
target fails
 ↓
rollback
 ↓
entity remains source
```

---

# 239. Persistence Test

```text id="dim-263"
Dimension
 ↓
state change
 ↓
save
 ↓
shutdown
 ↓
reload
 ↓
same state
```

---

# 240. Dynamic Dimension Test

```text id="dim-264"
create instance
 ↓
generate
 ↓
enter
 ↓
save
 ↓
leave
 ↓
reload
```

---

# 241. Missing Dimension Test

```text id="dim-265"
save with mod dimension
 ↓
remove mod
 ↓
load
 ↓
MissingDimension
 ↓
restore mod
 ↓
dimension restored
```

---

# 242. Multi-Dimension Stress

```text id="dim-266"
16 definitions
+
many instances
+
multiple active
```

testar:

```text id="dim-267"
load
unload
stream
transfer
simulation
persistence
```

---

# 243. LOD Test

```text id="dim-268"
16 dimensions
 ↓
1 FULL
3 REGIONAL
12 ABSTRACT
```

e verificar consumo.

---

# 244. Network Test

```text id="dim-269"
100 players
```

distribuídos entre dimensões.

Validar que cada cliente recebe apenas o estado relevante.

---

# 245. Determinism Test

Mesma:

```text id="dim-270"
definition
seed
generationVersion
```

deve produzir mesmo mundo inicial.

---

# 246. Migration Test

```text id="dim-271"
Dimension v1
 ↓
Migration
 ↓
Current
```

---

# 247. API principal

```text id="dim-api-01"
IDimension
IDimensionDefinition
IDimensionInstance
IDimensionManager
IDimensionRegistry
IDimensionQuery
IDimensionTravel
IDimensionTravelProvider
IDimensionSpawn
IDimensionEnvironment
IDimensionLifecycle
IDimensionLink
```

---

# 248. Dimension Runtime

```text id="dim-api-02"
DimensionRuntime

update()
load()
unload()
activate()
deactivate()
```

---

# 249. Dimension Manager

```text id="dim-api-03"
DimensionManager

createInstance()
load()
unload()
activate()
deactivate()
destroy()
transfer()
get()
list()
```

---

# 250. Dimension Context

```text id="dim-api-04"
DimensionContext

definition
instance
world
registry
events
persistence
scheduler
```

---

# 251. Código

Eu organizaria assim:

```text id="dim-code-01"
src/
└── dimension/
    ├── core/
    │   ├── dimension.ts
    │   ├── dimension-definition.ts
    │   ├── dimension-instance.ts
    │   ├── dimension-id.ts
    │   └── dimension-context.ts
    │
    ├── manager/
    │   ├── dimension-manager.ts
    │   ├── lifecycle.ts
    │   └── load-state.ts
    │
    ├── registry/
    │   ├── dimension-registry.ts
    │   └── travel-provider-registry.ts
    │
    ├── rules/
    │   ├── world-rules.ts
    │   ├── time-profile.ts
    │   ├── physics-profile.ts
    │   ├── spawn-profile.ts
    │   └── access-policy.ts
    │
    ├── environment/
    │   ├── atmosphere-profile.ts
    │   ├── climate-profile.ts
    │   ├── lighting-profile.ts
    │   ├── audio-profile.ts
    │   ├── sky-profile.ts
    │   └── gravity-profile.ts
    │
    ├── bounds/
    │   ├── dimension-bounds.ts
    │   └── boundary-policy.ts
    │
    ├── travel/
    │   ├── dimension-travel.ts
    │   ├── travel-request.ts
    │   ├── travel-result.ts
    │   ├── travel-provider.ts
    │   ├── dimension-link.ts
    │   └── link-registry.ts
    │
    ├── spawn/
    │   ├── spawn-resolver.ts
    │   └── safe-spawn.ts
    │
    ├── simulation/
    │   ├── simulation-profile.ts
    │   ├── dimension-scheduler.ts
    │   └── dimension-lod.ts
    │
    ├── persistence/
    │   ├── dimension-serializer.ts
    │   └── migration.ts
    │
    ├── networking/
    │   └── dimension-replication.ts
    │
    ├── events/
    │   └── dimension-events.ts
    │
    ├── debugging/
    │   ├── dimension-inspector.ts
    │   ├── dimension-graph.ts
    │   └── dimension-profiler.ts
    │
    └── api/
        └── dimension-api.ts
```

---

# 252. Fronteira arquitetural

## Dimension System faz

```text
identity
definitions
instances
lifecycle
bounds
rules
environment profiles
spawn policy
travel
dimension links
simulation configuration
dimension LOD
```

## Não faz

```text
world generation
chunk storage
entity logic
physics simulation
climate simulation
fluid simulation
rendering
audio playback
AI
civilization
economy
combat
```

---

# 253. Regra fundamental

> **Dimension System define o espaço e as regras nas quais outros sistemas operam; ele não implementa os sistemas que simulam essas regras.**

---

# 254. Segunda regra

> **Uma dimensão é uma definição + uma instância + um estado persistente, não simplesmente um mapa separado.**

---

# 255. Terceira regra

> **Atravessar dimensões é uma operação transacional, e nunca deve deixar uma entidade parcialmente transferida entre dois mundos.**

---

# 256. Quarta regra

> **Dimensões oficiais e dimensões de mods utilizam a mesma API pública.**

---

# 257. Quinta regra

> **Far Lands e Beyondlands são regiões/fronteiras do mundo quando conceitualmente pertencem ao mesmo espaço; não transformar toda região especial em uma nova dimensão.**

Isso é especialmente importante para preservar a progressão que definimos:

```text
SURFACE
   ↓
DEEP WORLD
   ↓
FAR LANDS
   ↓
BEYONDLANDS
   ↓
DIMENSIONAL FRONTIER
   ↓
OTHER DIMENSION
```

---

# 258. Ordem de implementação

```text id="dim-order"
DIM-0    Core Contracts
DIM-1    DimensionID
DIM-2    DimensionDefinition
DIM-3    DimensionRegistry
DIM-4    DimensionInstance
DIM-5    DimensionManager
DIM-6    Lifecycle
DIM-7    Context
DIM-8    World Rules
DIM-9    Time Profile
DIM-10   Environment Profile
DIM-11   Atmosphere
DIM-12   Gravity
DIM-13   Lighting Profile
DIM-14   Climate Profile
DIM-15   Sky
DIM-16   Bounds
DIM-17   Spawn Profile
DIM-18   Access Policy
DIM-19   Simulation Profile
DIM-20   Dimension LOD
DIM-21   Load
DIM-22   Unload
DIM-23   Streaming
DIM-24   Persistence
DIM-25   Migration
DIM-26   Travel Request
DIM-27   Travel Validation
DIM-28   Spawn Resolution
DIM-29   Entity Transfer
DIM-30   Dimension Links
DIM-31   Travel Providers
DIM-32   Event Bus Integration
DIM-33   Networking
DIM-34   Registry Integration
DIM-35   Mod API
DIM-36   Dynamic Dimensions
DIM-37   Missing Dimensions
DIM-38   Debugging
DIM-39   Profiling
DIM-40   Stress Tests
DIM-41   Compatibility
```

---

# 259. Primeiro Vertical Slice

```text id="dim-vs-01"
DimensionRegistry
        ↓
nexora:overworld
        ↓
DimensionInstance
        ↓
WorldGenerator
        ↓
Chunks
        ↓
Entity
        ↓
Player
```

---

# 260. Segundo Vertical Slice

```text id="dim-vs-02"
Overworld
 ↓
Portal
 ↓
DimensionTravelRequest
 ↓
Validation
 ↓
Load Target
 ↓
Transfer Entity
 ↓
DimensionTransferredEvent
```

---

# 261. Terceiro Vertical Slice

```text id="dim-vs-03"
Dimension
 ↓
Gravity Profile
 ↓
Physics
 ↓
Player movement
 ↓
Renderer
```

---

# 262. Quarto Vertical Slice

```text id="dim-vs-04"
Dimension
 ↓
Atmosphere
 ↓
Climate
 ↓
Lighting
 ↓
Audio
 ↓
UI
```

---

# 263. Quinto Vertical Slice

```text id="dim-vs-05"
Custom Mod
 ↓
DimensionDefinition
 ↓
DimensionRegistry
 ↓
Custom Generator
 ↓
Custom Environment
 ↓
Portal
 ↓
Save
 ↓
Reload
```

---

# 264. Sexto Vertical Slice — escala

```text id="dim-vs-06"
16 Dimensions
       ↓
1 FULL
3 REGIONAL
12 ABSTRACT
       ↓
players travel
       ↓
load / unload
       ↓
persistent state
       ↓
return
```

Esse é um teste muito importante para provar que o NEXORA realmente suporta múltiplas dimensões sem simplesmente manter todos os mundos carregados em RAM.

---

# 265. Arquitetura final

```text id="dim-final-01"
                           NEXORA CORE
                                │
                        REGISTRY SYSTEM
                                │
                      DIMENSION SYSTEM
                                │
                  ┌─────────────┴─────────────┐
                  ↓                           ↓
         DIMENSION DEFINITION          DIMENSION INSTANCE
                  │                           │
        ┌─────────┼──────────┐       ┌────────┼─────────┐
        ↓         ↓          ↓       ↓        ↓         ↓
      RULES   ENVIRONMENT   TRAVEL   WORLD   CHUNKS   ENTITIES
        │         │          │       │        │         │
        ↓         ↓          ↓       ↓        ↓         ↓
   Time/Gravity Climate    Portal  WorldGen Persistence Entity
   Bounds       Atmosphere  Ship    Blocks    Save       AI
                Lighting    Gate   Fluids
                Audio             Machines
                Sky               Civilization
```

E o lugar dele na fundação fica:

```text id="dim-foundation"
NEXORA FOUNDATION
├── Core
├── Registry System
├── Event Bus
├── Save / Persistence
├── Entity System
├── Block System
├── Item System
├── Animation System
├── Audio System
├── UI System
└── Dimension System
```

A distinção mais importante agora é:

```text id="dim-final-02"
DIMENSION SYSTEM
"onde e sob quais regras?"

WORLD GENERATION
"como este mundo é criado?"

WORLD / CHUNK
"qual é o estado espacial atual?"

PERSISTENCE
"como esse estado sobrevive?"

ENTITY SYSTEM
"quem existe nele?"

SIMULATION SYSTEMS
"como ele se comporta?"
```

Isso permite que o NEXORA tenha **Overworld, Deep World, Far Lands, Beyondlands, Void, mundos alienígenas, planetas, dimensões mágicas ou dimensões criadas por mods** usando a mesma fundação, em vez de cada dimensão virar um engine separado.
