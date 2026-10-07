Sim. Para o NEXORA, eu trataria **Física** como um Master Plan próprio e independente do Core, mas com integração profunda com Voxel, entidades, veículos, fluidos, máquinas e mundo.

# NEXORA — PHYSICS MASTER PLAN

A arquitetura-base:

```text
NEXORA PHYSICS
│
├── Physics Core
│   ├── Time Step
│   ├── Solver
│   ├── Broadphase
│   ├── Narrowphase
│   ├── Constraints
│   └── Collision System
│
├── World Physics
│   ├── Voxel Collision
│   ├── Terrain
│   ├── Structures
│   └── Dynamic Blocks
│
├── Entity Physics
│   ├── Character
│   ├── Mob
│   ├── Item
│   ├── Projectile
│   └── Ragdoll
│
├── Material Physics
│   ├── Solid
│   ├── Fluid
│   ├── Powder
│   ├── Gas
│   └── Special
│
├── Vehicle Physics
│   ├── Wheeled
│   ├── Rail
│   ├── Boat
│   ├── Aircraft
│   └── Spacecraft
│
└── Simulation Integration
    ├── Machines
    ├── Fluids
    ├── Destruction
    ├── Weather
    └── World Events
```

## 1. PHY-0 — Physics World

Criar um ambiente físico separado do `World Runtime`.

```text
World
 ├── Terrain
 ├── Entities
 └── PhysicsWorld
```

O PhysicsWorld controla:

```text
bodies
colliders
contacts
constraints
materials
queries
simulation state
```

A física não deve modificar diretamente qualquer sistema de gameplay.

---

# 2. PHY-1 — Fixed Timestep

A física precisa de tempo determinístico.

Em vez de:

```text
physics = frame
```

usar:

```text
Game Time
    ↓
Physics Tick
    ↓
fixed timestep
```

Conceitualmente:

```text
accumulator += frameTime

while accumulator >= fixedStep:
    physics.step(fixedStep)
    accumulator -= fixedStep
```

Isso evita que a física mude drasticamente com FPS.

---

# 3. PHY-2 — Rigid Body

Base para objetos físicos.

```text
RigidBody
├── position
├── rotation
├── linearVelocity
├── angularVelocity
├── mass
├── inertia
├── gravityScale
├── damping
├── bodyType
└── sleepState
```

Tipos:

```text
STATIC
KINEMATIC
DYNAMIC
```

Exemplos:

```text
bedrock
→ STATIC

moving platform
→ KINEMATIC

falling block
→ DYNAMIC
```

---

# 4. PHY-3 — Collider System

Os objetos físicos precisam de formas de colisão.

```text
Box
Sphere
Capsule
Cylinder
Convex Hull
Compound
Voxel Shape
```

Para o mundo voxel:

```text
Block
 ↓
Collision Shape
```

Não devemos necessariamente transformar cada bloco em um collider independente.

Devemos gerar formas agrupadas/chunk-based quando possível.

---

# 5. PHY-4 — Collision Detection

Separar em duas fases.

```text
Broadphase
     ↓
possíveis colisões
     ↓
Narrowphase
     ↓
colisão real
```

Broadphase:

```text
AABB
Spatial Hash
Grid
BVH
```

A escolha poderá variar conforme o tipo de objeto.

---

# 6. PHY-5 — Collision Manifold

Quando duas formas colidem:

```text
Collision
├── bodyA
├── bodyB
├── contactPoints
├── normal
├── penetration
└── relativeVelocity
```

Isso alimenta o solver.

---

# 7. PHY-6 — Physics Solver

Resolver:

```text
collision
friction
restitution
constraints
forces
impulses
```

Conceito:

```text
detect collision
 ↓
calculate contacts
 ↓
solve constraints
 ↓
apply impulses
 ↓
integrate velocity
 ↓
integrate position
```

---

# 8. PHY-7 — Materials

A física não deve assumir que tudo é igual.

Criar:

```text
PhysicsMaterial
├── density
├── friction
├── restitution
├── drag
├── buoyancy
├── conductivity
└── specialProperties
```

Exemplo conceitual:

```text
ice
→ baixa fricção

rubber-like material
→ alta restituição

rock
→ alta densidade
```

As propriedades podem ser definidas pelos próprios módulos.

---

# 9. PHY-8 — Gravity

Gravity deve ser configurável por dimensão.

```text
GravityField
├── direction
├── magnitude
├── falloff
└── source
```

Assim uma dimensão pode ter:

```text
normal gravity
low gravity
high gravity
radial gravity
variable gravity
```

Isso será importante para Space e dimensões especiais.

---

# 10. PHY-9 — Forces

Criar API de forças:

```text
Force
Impulse
Torque
AngularImpulse
```

E sistemas capazes de aplicar:

```text
gravity
wind
explosion impulse
machine force
vehicle force
thruster force
```

Nada disso precisa ficar hardcoded no Core.

---

# 11. PHY-10 — Voxel Physics

Aqui começa a parte realmente importante para NEXORA.

O mundo não é formado principalmente por objetos sólidos convencionais.

É:

```text
Voxel World
```

Então precisamos de:

```text
VoxelCollisionProvider
VoxelShapeResolver
ChunkCollisionCache
```

Fluxo:

```text
Entity
 ↓
Physics Query
 ↓
Chunk
 ↓
Voxel Collision
 ↓
Collision Shape
 ↓
Solver
```

---

# 12. PHY-11 — Terreno

Montanhas, cavernas, árvores e construções precisam participar das colisões.

O Physics Engine precisa entender:

```text
terrain surface
cave ceiling
wall
slope
stairs
blocks
structures
```

---

# 13. PHY-12 — Slopes

Um sistema voxel não deveria limitar tudo a cubos.

Suporte a:

```text
slab
slope
ramp
wedge
stairs
custom shape
```

Isso será útil para:

```text
Create-like structures
roads
railways
machines
architecture
```

---

# 14. PHY-13 — Character Controller

Player e NPC não devem necessariamente usar um rigid body genérico.

Criar:

```text
CharacterController
├── capsule
├── stepHeight
├── slopeLimit
├── movement
├── grounding
└── climbing
```

Suporte a:

```text
walk
run
jump
fall
swim
climb
crouch
slide
```

---

# 15. PHY-14 — Ground Detection

O sistema precisa determinar:

```text
grounded
airborne
sliding
falling
```

Também:

```text
ground normal
slope angle
supporting body
```

Assim outros sistemas podem perguntar:

```text
player.isGrounded()
```

sem conhecer a implementação física.

---

# 16. PHY-15 — Friction

Fricção pode depender dos dois materiais:

```text
material A
+
material B
=
contact friction
```

Exemplo:

```text
metal + ice
metal + stone
rubber-like + stone
```

---

# 17. PHY-16 — Buoyancy

Objetos podem interagir com fluidos.

```text
Body
 ↓
Fluid Query
 ↓
submerged volume
 ↓
buoyant force
 ↓
drag
```

Isso permite:

```text
wood floating
stone sinking
boats floating
player swimming
```

O Fluid System continua independente; a Física apenas consome os dados físicos.

---

# 18. PHY-17 — Water Interaction

Integração com:

```text
water
lava
other fluids
```

Possibilitando:

```text
swimming
floating
fluid drag
pressure
currents
```

---

# 19. PHY-18 — Falling Objects

Criar sistema para objetos físicos desprendidos.

Exemplo:

```text
unsupported block
 ↓
Physics check
 ↓
becomes dynamic
 ↓
falls
 ↓
collides
 ↓
rests
```

Mas isso precisa de limites para evitar milhões de rigid bodies.

---

# 20. PHY-19 — Structural Physics

Aqui eu faria uma distinção importante.

A física comum:

```text
objeto → colisão
```

não é suficiente para simular construções.

Criar futuramente:

```text
Structural Analysis
├── support
├── load
├── stress
├── stability
└── failure
```

Assim construções podem opcionalmente ter:

```text
support points
load capacity
structural stability
```

Isso pode ser configurável por mundo/mod.

---

# 21. PHY-20 — Destruction

Sistema separado:

```text
Physics
   ↓
Destruction API
```

Suporte futuro a:

```text
fracture
break
collapse
debris
damage propagation
```

Não fazer explosão/destruição diretamente no solver.

---

# 22. PHY-21 — Projectiles

Criar sistema de queries de alta velocidade.

Problema:

```text
objeto rápido
```

pode atravessar um collider entre dois frames.

Usar:

```text
raycast
sweep
continuous collision detection
```

Especialmente para:

```text
arrows
tools
projectiles
vehicles
```

---

# 23. PHY-22 — Raycast / Queries

A Physics API precisa permitir:

```text
Raycast
SphereCast
CapsuleCast
BoxCast
OverlapSphere
OverlapBox
```

Uso:

```text
player
→ olhando para bloco

tool
→ procurando superfície

AI
→ verificando obstáculo

vehicle
→ verificando terreno
```

---

# 24. PHY-23 — Physics Layers

Colisões devem ser filtráveis.

```text
PLAYER
NPC
MOB
VEHICLE
PROJECTILE
BLOCK
ITEM
TRIGGER
MACHINE
```

Exemplo:

```text
Player
↔ Block
YES

Player
↔ Trigger
YES

Player
↔ decorative particle
NO
```

---

# 25. PHY-24 — Triggers

Objetos que detectam presença sem colisão física.

```text
Trigger
 ↓
Entity enters
 ↓
Event
```

Usos:

```text
portal
pressure area
machine area
quest region
vehicle station
dimension boundary
```

---

# 26. PHY-25 — Moving Platforms

Suporte a:

```text
elevators
conveyors
moving platforms
rotating structures
doors
machines
```

Character Controller precisa saber acompanhar uma plataforma móvel.

---

# 27. PHY-26 — Conveyors

Isso será particularmente importante para a parte industrial do NEXORA.

```text
Conveyor
 ↓
item movement
```

Mas o Item não precisa necessariamente ser um rigid body.

Pode existir:

```text
logical transport
```

e só virar física real em casos necessários.

---

# 28. PHY-27 — Vehicle Physics

Separar uma API genérica:

```text
VehicleBody
```

E implementações:

```text
WheeledVehicle
RailVehicle
Boat
Aircraft
Spacecraft
```

---

# 29. PHY-28 — Railway Physics

Como você quer uma infraestrutura ferroviária importante no mundo:

```text
Rail Network
 ↓
Track Geometry
 ↓
Rail Physics
 ↓
Vehicle
```

Suporte futuro:

```text
speed
acceleration
braking
mass
inclines
curves
coupling
```

O sistema ferroviário controla a rede; a física resolve a parte física.

---

# 30. PHY-29 — Wheels

Para veículos terrestres:

```text
WheelCollider
Suspension
TireContact
Steering
Traction
```

Isso evita transformar todo carro em simplesmente:

```text
entity + velocity
```

---

# 31. PHY-30 — Aircraft

Separar aerodinâmica do rigid body básico.

```text
Lift
Drag
Thrust
Weight
Control surfaces
```

Assim:

```text
Aircraft
 ↓
Aerodynamic Model
 ↓
Physics Engine
```

---

# 32. PHY-31 — Space Physics

No espaço:

```text
gravity
orbital motion
thrust
drag
mass
momentum
```

Mas o sistema pode usar simplificações dependendo da escala.

Não precisamos simular cada corpo celeste com precisão astronômica para criar uma experiência convincente.

---

# 33. PHY-32 — Fluids

A física não deve ser o Fluid Engine.

Mas deve possuir uma interface:

```text
FluidPhysicsAdapter
```

Que consulta:

```text
density
pressure
velocity
flow
temperature
```

Isso permite:

```text
buoyancy
drag
pressure effects
fluid interaction
```

---

# 34. PHY-33 — Wind / Atmosphere

Integração com clima:

```text
Atmosphere
 ↓
Wind Field
 ↓
Physics
```

Podemos ter:

```text
wind direction
wind speed
gusts
local turbulence
```

Isso influencia apenas objetos/sistemas que optarem por participar.

---

# 35. PHY-34 — Sleeping

Objetos parados não precisam ser simulados continuamente.

```text
Dynamic
 ↓
velocity ≈ 0
 ↓
sleep
```

Quando algo próximo muda:

```text
wake
 ↓
simulate
```

Isso será fundamental para performance.

---

# 36. PHY-35 — Active Physics Region

Não devemos simular objetos físicos no mundo inteiro.

```text
Player
 ↓
Physics Active Region
```

Dentro:

```text
FULL physics
```

Fora:

```text
REDUCED
```

Muito longe:

```text
ABSTRACT
```

Integra diretamente com a ideia de Simulation LOD do NEXORA.

---

# 37. PHY-36 — Determinism

Para multiplayer e testes:

```text
same inputs
+
same state
+
same physics version
=
same result
```

Precisamos definir o nível de determinismo desejado.

Especialmente para:

```text
server
replay
debug
multiplayer
```

---

# 38. PHY-37 — Multiplayer

A Física não deve permitir que o cliente seja autoridade final em operações críticas.

Arquitetura:

```text
Client
 ↓
Input
 ↓
Server Physics
 ↓
Authoritative State
 ↓
Replication
```

Para algumas interações locais podemos usar prediction.

---

# 39. PHY-38 — Prediction / Reconciliation

Player:

```text
Client predicts
 ↓
Server validates
 ↓
Correction if needed
```

Isso será essencial para movimentação multiplayer.

---

# 40. PHY-39 — Physics Events

Eventos:

```text
CollisionStarted
CollisionEnded
TriggerEntered
TriggerExited
BodySleep
BodyWake
Grounded
Ungrounded
```

Assim:

```text
Physics
 ↓
Event Bus
 ↓
Gameplay Systems
```

---

# 41. PHY-40 — Performance

O sistema deve ter:

```text
Spatial partitioning
Broadphase optimization
Sleeping
Batching
Multithreading
Chunk collision caching
LOD
priority simulation
```

E métricas:

```text
physics ms
broadphase ms
narrowphase ms
solver ms
queries ms
active bodies
contacts
```

---

# 42. PHY-41 — Debug Tools

Criar:

```text
nexora physics stats
nexora physics bodies
nexora physics colliders
nexora physics contacts
nexora physics raycast
```

Visualização:

```text
colliders
AABB
contact points
normals
velocity vectors
sleeping bodies
active regions
```

---

# 43. PHY-42 — Testing

Testes essenciais:

```text
falling body
collision
friction
bounce
slope
stairs
moving platform
vehicle
buoyancy
raycast
CCD
sleep/wake
```

Também stress tests:

```text
100 bodies
1,000 bodies
10,000 logical objects
large chunk
large structure
```

---

# 44. PHY-43 — Mod API

Mods devem poder registrar:

```text
PhysicsMaterial
ColliderShape
ForceProvider
VehiclePhysics
FluidInteraction
CollisionRule
PhysicsComponent
```

Exemplo:

```text
ExampleMod
→ registra novo material físico
→ registra veículo
→ registra collider
→ registra comportamento
```

Sem modificar o Physics Core.

---

# 45. PHY-44 — Integrações

No final:

```text
                    PHYSICS
                       │
       ┌───────────────┼────────────────┐
       │               │                │
     WORLD          ENTITIES          FLUIDS
       │               │                │
     VOXELS         PLAYER/NPC       WATER/LAVA
       │               │                │
       └───────────────┼────────────────┘
                       │
                 VEHICLES
                       │
            ┌──────────┼──────────┐
            │          │          │
          RAIL       WATER       SPACE
```

E também:

```text
Physics
 ↓
Destruction
 ↓
Structures
 ↓
World State
```

---

# 46. Estrutura de código

Eu deixaria aproximadamente:

```text
physics/
│
├── core/
│   ├── world/
│   ├── timestep/
│   ├── solver/
│   ├── bodies/
│   ├── colliders/
│   ├── materials/
│   └── constraints/
│
├── collision/
│   ├── broadphase/
│   ├── narrowphase/
│   ├── manifold/
│   └── queries/
│
├── voxel/
│   ├── shapes/
│   ├── chunk-collision/
│   └── terrain/
│
├── character/
│
├── vehicles/
│   ├── wheeled/
│   ├── rail/
│   ├── marine/
│   ├── aircraft/
│   └── spacecraft/
│
├── fluids/
├── destruction/
├── structures/
├── networking/
├── debug/
└── api/
```

# 47. Ordem de implementação

Eu faria:

```text
PHY-0 Physics World
PHY-1 Fixed Timestep
PHY-2 Rigid Body
PHY-3 Collider
PHY-4 Broadphase
PHY-5 Narrowphase
PHY-6 Solver
PHY-7 Materials
PHY-8 Gravity
PHY-9 Forces
PHY-10 Voxel Physics
PHY-11 Character Controller
PHY-12 Queries
PHY-13 Triggers
PHY-14 Sleeping
PHY-15 Physics Events
PHY-16 Physics LOD
PHY-17 Fluids integration
PHY-18 Vehicles
PHY-19 Structural Physics
PHY-20 Destruction
PHY-21 Multiplayer
PHY-22 Mod API
```

## Primeiro vertical slice

Antes de veículos, fluidos, destruição e espaço, o teste fundamental seria:

```text
NEXORA
 ↓
World
 ↓
Chunk
 ↓
Voxel Collision
 ↓
Player
 ↓
Character Controller
 ↓
Gravity
 ↓
Jump
 ↓
Slope
 ↓
Collision
 ↓
Moving Platform
 ↓
Save
```

E o teste de estresse:

```text
1.000 objetos
 ↓
physics
 ↓
sleeping
 ↓
wake-up
 ↓
collision
 ↓
no catastrophic performance drop
```

### Regra arquitetural

A Física deve conhecer **massa, posição, velocidade, colisão e forças**.

Ela **não** deve conhecer:

```text
"isso é uma espada"
"isso é um aldeão"
"isso é minério"
"isso é uma máquina do Mekanism"
```

Ela deve conhecer interfaces:

```text
Body
Collider
Material
Force
Constraint
Query
```

E os outros sistemas decidem o significado.

Isso mantém a Física reutilizável para **player, 2.000+ mobs, NPCs, máquinas, trens, barcos, aeronaves, estruturas e naves espaciais** sem transformar o Physics Core em um monstro impossível de manter.
