Perfeito. O **Event Bus** é a peça que vai impedir o NEXORA de virar uma teia de dependências diretas.

A regra central:

> **Sistemas publicam fatos e assinam eventos; eles não precisam conhecer diretamente quem vai reagir a esses fatos.**

Isso encaixa muito bem com a arquitetura que já definimos para **Registry, Entity, Block, Item, Fluid, Energy, Machines, Combat, Civilization e World Simulation**.

# NEXORA — EVENT BUS

## 1. Princípio central

```text id="event-01"
                    NEXORA CORE
                         │
                     EVENT BUS
                         │
      ┌──────────────────┼──────────────────┐
      ↓                  ↓                  ↓
   PUBLISH            ROUTE              SUBSCRIBE
      │                  │                  │
      └──────────────────┼──────────────────┘
                         ↓
               NEXORA SYSTEMS
```

Exemplo:

```text id="event-02"
Player quebra bloco
       ↓
Build System
       ↓
BlockBrokenEvent
       ↓
Event Bus
       ├── Loot
       ├── Lighting
       ├── Fluid
       ├── Physics
       ├── AI
       ├── History
       └── Networking
```

O Build System não precisa fazer:

```text id="event-03"
loot.onBlockBroken()
lighting.onBlockBroken()
fluid.onBlockBroken()
physics.onBlockBroken()
...
```

Isso seria forte acoplamento.

---

# 2. O que é um Event?

Um Event representa um **fato ou solicitação**.

Exemplo de fato:

```text id="event-04"
BlockBrokenEvent
```

significa:

> Um bloco foi quebrado.

Exemplo de solicitação:

```text id="event-05"
BlockBreakRequest
```

significa:

> Alguém está solicitando que um bloco seja quebrado.

Esses dois conceitos precisam ser separados.

---

# 3. Event ≠ Command

### Command

```text id="event-06"
"faça X"
```

Exemplo:

```text id="event-07"
BreakBlockCommand
```

### Event

```text id="event-08"
"X aconteceu"
```

Exemplo:

```text id="event-09"
BlockBrokenEvent
```

Pipeline:

```text id="event-10"
Command
 ↓
System validates
 ↓
Operation
 ↓
Event
```

---

# 4. Event Bus não executa gameplay

O Event Bus deve:

```text id="event-11"
receber
rotear
ordenar
entregar
monitorar
```

Não deve:

```text id="event-12"
decidir loot
decidir combat
decidir physics
decidir economy
```

Ele transporta eventos.

---

# 5. Arquitetura

```text id="event-13"
EVENT BUS
├── EVENT
├── EVENT TYPE
├── EVENT CHANNEL
├── SUBSCRIBER
├── PUBLISHER
├── ROUTER
├── QUEUE
├── DISPATCHER
├── FILTER
├── PRIORITY
├── SCHEDULER
├── TRANSACTION
├── SNAPSHOT
├── REPLAY
├── DEBUG
└── MOD API
```

---

# 6. EventDefinition

Cada evento possui um tipo conhecido.

```text id="event-14"
EventType<T>
```

Exemplos:

```text id="event-15"
BlockPlacedEvent
EntitySpawnedEvent
ItemTransferredEvent
FluidChangedEvent
EnergyNetworkChangedEvent
RecipeCompletedEvent
```

---

# 7. Event Instance

É a ocorrência concreta.

```text id="event-16"
EventInstance

type
id
timestamp
source
context
payload
metadata
```

---

# 8. Event ID

Cada evento concreto pode possuir:

```text id="event-17"
EventID
```

Útil para:

```text id="event-18"
debug
deduplication
replay
tracing
networking
```

---

# 9. Correlation ID

Um evento pode fazer parte de uma operação maior.

Exemplo:

```text id="event-19"
Player breaks block
 ↓
BlockBrokenEvent
 ↓
LootGenerated
 ↓
ItemCreated
 ↓
ItemDropped
 ↓
Pickup
```

Todos podem compartilhar:

```text id="event-20"
CorrelationID
```

Isso permite reconstruir a cadeia causal.

---

# 10. Causation ID

Também recomendo:

```text id="event-21"
CausationID
```

Exemplo:

```text id="event-22"
ItemCreated
caused by
LootGenerated
```

---

# 11. Source

Todo evento precisa saber sua origem.

Exemplo:

```text id="event-23"
Player
NPC
Machine
WorldGen
Climate
Fluid
Physics
Command
Mod
Server
```

---

# 12. Event Context

```text id="event-24"
EventContext

world
dimension
position
actor
source
correlationId
causationId
timestamp
```

Nem todos os eventos precisam de todos esses campos.

---

# 13. Event Payload

Os dados específicos ficam no payload.

Exemplo:

```text id="event-25"
BlockBrokenEvent
{
    position
    previousState
    actor
    tool
    cause
}
```

---

# 14. Strongly Typed Events

Evitar:

```text id="event-26"
Event
{
    type: "something",
    data: {}
}
```

para tudo.

Preferir:

```text id="event-27"
BlockBrokenEvent
ItemCreatedEvent
EntityMovedEvent
```

com schemas bem definidos.

---

# 15. Generic Event Interface

Conceitualmente:

```text id="event-28"
IEvent
```

e:

```text id="event-29"
IEventHandler<T>
```

---

# 16. Publisher

```text id="event-30"
IEventPublisher
```

com:

```text id="event-31"
publish(event)
publishBatch(events)
```

---

# 17. Subscriber

```text id="event-32"
IEventSubscriber
```

ou:

```text id="event-33"
subscribe<T>(handler)
```

---

# 18. Subscription

Uma assinatura precisa registrar:

```text id="event-34"
event type
handler
priority
filter
phase
owner
```

---

# 19. Subscription Handle

Quando alguém faz:

```text id="event-35"
subscribe(...)
```

retorna:

```text id="event-36"
SubscriptionHandle
```

para:

```text id="event-37"
unsubscribe()
```

---

# 20. Owner

Toda subscription deve possuir proprietário lógico:

```text id="event-38"
core
block-system
item-system
mod:examplemod
```

Isso facilita:

```text id="event-39"
shutdown
mod unload
debug
cleanup
```

---

# 21. Channels

Um Event Bus gigante pode ficar difícil de controlar.

Criar canais lógicos:

```text id="event-40"
WORLD
ENTITY
BLOCK
ITEM
PHYSICS
FLUID
ENERGY
MACHINE
COMBAT
PLAYER
NETWORK
AUDIO
RENDER
UI
CIVILIZATION
ECONOMY
DEBUG
```

---

# 22. Global vs Domain Event Bus

Eu usaria:

```text id="event-41"
Global Event Bus
       │
       ├── World Channel
       ├── Entity Channel
       ├── Block Channel
       ├── Item Channel
       └── ...
```

Não criar dezenas de implementações diferentes.

---

# 23. Event Type Registry

O próprio Event Bus precisa de um Registry.

```text id="event-42"
EventTypeRegistry
```

Isso encaixa diretamente com o Registry System.

```text id="event-43"
Registry System
      ↓
EventTypeRegistry
      ↓
Event Bus
```

---

# 24. Event Type IDs

Exemplo:

```text id="event-44"
nexora:block_broken
nexora:entity_spawned
nexora:item_created
```

Mods:

```text id="event-45"
examplemod:machine_overheated
```

---

# 25. Event Version

Todo evento público deve possuir versão.

```text id="event-46"
EventSchemaVersion
```

Isso é importante para mods e replay.

---

# 26. Event Schema

```text id="event-47"
EventSchema

fields
types
required
optional
version
```

---

# 27. Event Categories

Separar:

```text id="event-48"
FACT
COMMAND
QUERY
LIFECYCLE
TRANSACTION
SIGNAL
```

---

# 28. Fact

Algo aconteceu:

```text id="event-49"
EntityDied
BlockBroken
ItemCreated
MachineCompleted
```

---

# 29. Command

Algo deve acontecer:

```text id="event-50"
SpawnEntityCommand
BreakBlockCommand
TransferItemCommand
```

---

# 30. Query

Normalmente não deveria usar Event Bus para queries síncronas.

Preferir API direta:

```text id="event-51"
query()
```

O Event Bus não deve virar RPC universal.

---

# 31. Lifecycle Events

Exemplos:

```text id="event-52"
EntityCreated
EntityLoaded
EntityUnloaded
EntityRemoved
```

---

# 32. Transaction Events

Exemplo:

```text id="event-53"
ItemTransactionStarted
ItemTransactionCommitted
ItemTransactionRolledBack
```

---

# 33. Signals

Para automação:

```text id="event-54"
EnergyOverload
MachineSignal
RailSignal
Alarm
```

Mas sinais de alta frequência precisam de canais específicos.

---

# 34. Synchronous Events

Handler executa imediatamente:

```text id="event-55"
publish()
 ↓
handler A
 ↓
handler B
```

Útil para:

```text id="event-56"
validation
local lifecycle
deterministic operations
```

---

# 35. Asynchronous Events

O evento entra em fila:

```text id="event-57"
publish()
 ↓
queue
 ↓
dispatcher
 ↓
handlers
```

Útil para:

```text id="event-58"
analytics
logging
audio
notifications
background processing
```

---

# 36. Nunca usar Async para tudo

O NEXORA possui sistemas determinísticos.

Algumas operações precisam acontecer dentro de uma fase controlada.

---

# 37. Deferred Events

Um evento pode ser publicado para o próximo tick/fase:

```text id="event-59"
publishDeferred(event)
```

---

# 38. Scheduled Events

Eventos podem ter timestamp lógico:

```text id="event-60"
schedule(event, tick)
```

ou:

```text id="event-61"
scheduleAfter(event, duration)
```

---

# 39. Event Priority

Handlers podem ter prioridade:

```text id="event-62"
EARLY
NORMAL
LATE
FINAL
```

Ou valores numéricos.

---

# 40. Priority não deve virar dependência invisível

Evitar:

```text id="event-63"
system A always depends on system B
```

só porque:

```text id="event-64"
priority = 999
```

Dependências importantes devem estar explícitas.

---

# 41. Event Phases

Pode ser melhor:

```text id="event-65"
PRE
MAIN
POST
```

Exemplo:

```text id="event-66"
BlockBreak PRE
BlockBreak MAIN
BlockBreak POST
```

---

# 42. Cancelable Events

Alguns eventos podem ser cancelados.

Exemplo:

```text id="event-67"
BlockBreakRequested
```

Um Protection System pode responder:

```text id="event-68"
cancel()
```

---

# 43. Fact Events não devem ser canceláveis

Depois que aconteceu:

```text id="event-69"
BlockBrokenEvent
```

não faz sentido cancelar.

---

# 44. Request Events

São candidatos naturais à cancelamento:

```text id="event-70"
BlockBreakRequested
ItemTransferRequested
EntitySpawnRequested
TeleportRequested
```

---

# 45. Event Result

Um handler pode produzir:

```text id="event-71"
accepted
rejected
modified
cancelled
```

Mas cuidado para não transformar o Event Bus numa função RPC.

---

# 46. Pre-Event

Modelo:

```text id="event-72"
Request
 ↓
Validation handlers
 ↓
accepted?
 ↓
operation
 ↓
Fact event
```

Esse modelo é muito bom para NEXORA.

---

# 47. Post-Event

Depois da operação:

```text id="event-73"
operation
 ↓
fact event
 ↓
reactions
```

---

# 48. Event Ordering

Precisamos de uma política clara.

Exemplo:

```text id="event-74"
Command
 ↓
Pre Event
 ↓
Validation
 ↓
Operation
 ↓
State Mutation
 ↓
Post Event
```

---

# 49. Example — Block Breaking

```text id="event-75"
BreakBlockCommand
        ↓
BlockBreakRequested
        ↓
Protection
        ↓
Build System
        ↓
Block state changes
        ↓
BlockBrokenEvent
        ↓
Loot
Lighting
Fluid
Physics
Renderer
Network
History
```

---

# 50. Example — Item Pickup

```text id="event-76"
PickupRequest
      ↓
Inventory validation
      ↓
ItemTransferredEvent
      ↓
ItemEntity updated
      ↓
Entity despawn
```

---

# 51. Example — Entity Death

```text id="event-77"
Damage
 ↓
Health reaches zero
 ↓
EntityDiedEvent
 ↓
Loot
AI
Civilization
History
Quest
Network
```

---

# 52. Example — Machine

```text id="event-78"
MachineProcessingCompleted
 ↓
ItemCreated
 ↓
EnergyConsumed
 ↓
FluidConsumed
 ↓
ProductionUpdated
```

---

# 53. Event Chains

O Event Bus pode permitir cadeias causais:

```text id="event-79"
A
 ↓
B
 ↓
C
 ↓
D
```

Mas deve existir limite.

---

# 54. Infinite Event Loops

Exemplo perigoso:

```text id="event-80"
A triggers B
B triggers A
```

Precisamos de:

```text id="event-81"
event depth
cycle detection
per-chain budget
```

---

# 55. Event Depth

Cada evento pode possuir:

```text id="event-82"
depth
```

Exemplo:

```text id="event-83"
root = 0
child = 1
grandchild = 2
```

---

# 56. Event Budget

Uma cadeia pode possuir:

```text id="event-84"
maxEvents
maxDepth
maxTime
```

---

# 57. Event Deduplication

Alguns eventos podem ser duplicados.

```text id="event-85"
EventDeduplicationPolicy
```

Pode usar:

```text id="event-86"
eventId
transactionId
correlationId
```

---

# 58. At-Least-Once vs Exactly-Once

Internamente, para eventos de gameplay, o ideal deve ser:

```text id="event-87"
deterministic controlled delivery
```

e para operações transacionais:

```text id="event-88"
idempotent handlers
```

Não depender de "exactly once" como mágica.

---

# 59. Idempotent Handlers

Handler deve conseguir receber novamente certos eventos sem duplicar efeitos.

Exemplo:

```text id="event-89"
transactionId already processed?
→ ignore
```

---

# 60. Event Queue

Criar:

```text id="event-90"
EventQueue
```

estruturada por:

```text id="event-91"
priority
phase
tick
channel
```

---

# 61. Event Dispatcher

```text id="event-92"
EventDispatcher
```

resolve:

```text id="event-93"
event type
 ↓
subscribers
 ↓
filters
 ↓
priority
 ↓
execute
```

---

# 62. Event Router

Pode existir uma camada:

```text id="event-94"
EventRouter
```

que decide:

```text id="event-95"
channel
local bus
global bus
network
```

---

# 63. Local Events

Alguns eventos só interessam ao chunk ou região.

Exemplo:

```text id="event-96"
BlockChanged
```

Não precisa necessariamente acordar sistemas em todo o mundo.

---

# 64. Regional Event Routing

```text id="event-97"
World
├── Region A
├── Region B
└── Region C
```

Evento em A pode ser roteado somente para interessados em A.

---

# 65. Spatial Subscription

Muito útil em um mundo gigantesco.

Subscriber pode dizer:

```text id="event-98"
listen near(position, radius)
```

Exemplo:

```text id="event-99"
players
render
audio
AI
```

---

# 66. Entity Interest

Entity System pode usar eventos associados à interest management.

```text id="event-100"
entity moved
 ↓
interest set changed
 ↓
network replication
```

---

# 67. Chunk Interest

```text id="event-101"
chunk loaded
chunk unloaded
```

só deve chegar a interessados relevantes.

---

# 68. Event Filters

Subscription pode ter:

```text id="event-102"
predicate
```

Exemplo:

```text id="event-103"
subscribe<BlockBrokenEvent>(
    where tag == "ore"
)
```

---

# 69. Filter Performance

Filtros caros não devem ser executados em todos os eventos.

Podemos indexar por:

```text id="event-104"
event type
dimension
region
entity type
block tag
item tag
```

---

# 70. Typed Filters

Melhor:

```text id="event-105"
BlockBrokenEvent
```

com:

```text id="event-106"
position
blockId
actorId
```

do que um `Event` genérico.

---

# 71. Event Bus and Threads

Event Bus precisa definir:

```text id="event-107"
thread safety
execution context
ownership
```

---

# 72. Main Simulation Thread

Alguns eventos precisam de:

```text id="event-108"
simulation authority
```

Exemplo:

```text id="event-109"
Block mutation
Entity structural mutation
Inventory transaction
```

---

# 73. Worker Events

Processamentos pesados podem publicar resultados:

```text id="event-110"
worker
 ↓
result event
 ↓
main simulation commit
```

---

# 74. Deferred Structural Changes

Isso combina com Entity/Block.

Exemplo:

```text id="event-111"
EntityDeathEvent
```

não necessariamente remove a Entity durante uma iteração.

Pode gerar:

```text id="event-112"
RemoveEntityCommand
```

para commit posterior.

---

# 75. Snapshot Boundary

Event Bus pode operar entre:

```text id="event-113"
Simulation Tick
```

e:

```text id="event-114"
Commit Phase
```

---

# 76. Tick Scheduling

Possível pipeline:

```text id="event-115"
TICK START
 ↓
consume queued events
 ↓
simulate
 ↓
generate events
 ↓
commit
 ↓
post events
 ↓
TICK END
```

---

# 77. Multiple Tick Scales

NEXORA possui:

```text id="event-116"
frame
second
minute
hour
day
week
season
```

Event Bus deve suportar timestamp lógico em várias escalas.

---

# 78. Event Time

Não depender apenas do relógio do computador.

Usar:

```text id="event-117"
WorldTime
SimulationTick
```

---

# 79. Real Time Events

Alguns elementos podem ser externos:

```text id="event-118"
server shutdown
network packet
player input
```

O Event Bus converte para eventos internos.

---

# 80. Input Events

Exemplo:

```text id="event-119"
PlayerInputEvent
```

Mas não deve substituir completamente o Input System.

---

# 81. Network Events

Rede também pode alimentar o Event Bus.

```text id="event-120"
NetworkPacket
 ↓
Network System
 ↓
validated command/event
 ↓
Simulation
```

Não colocar raw packets diretamente no gameplay bus.

---

# 82. UI Events

UI pode publicar:

```text id="event-121"
MenuOpened
InventoryActionRequested
QuestSelected
```

mas UI não deve ter autoridade sobre o mundo.

---

# 83. Audio Events

Sistemas publicam:

```text id="event-122"
BlockBroken
WeaponFired
MachineStarted
```

Audio System escuta e produz som.

---

# 84. Renderer Events

Renderer pode consumir:

```text id="event-123"
BlockChanged
EntitySpawned
EntityRemoved
```

mas rendering não deve alterar gameplay diretamente.

---

# 85. Physics Events

Physics pode publicar:

```text id="event-124"
CollisionOccurred
EntityLanded
BlockImpact
FluidPressureChanged
```

---

# 86. Fluid Events

```text id="event-125"
FluidFlowChanged
FluidMixed
FluidPhaseChanged
FluidPressureChanged
```

---

# 87. Energy Events

```text id="event-126"
EnergyTransferred
NetworkFormed
NetworkSplit
Overload
Brownout
Shutdown
```

---

# 88. Machine Events

```text id="event-127"
MachineStarted
MachineStopped
MachineProgressed
MachineCompleted
MachineFailed
MachineOverheated
```

---

# 89. Combat Events

```text id="event-128"
AttackStarted
AttackHit
DamageApplied
StatusApplied
EntityDied
```

---

# 90. AI Events

```text id="event-129"
ThreatDetected
GoalChanged
DecisionMade
BehaviorStateChanged
```

Nem todos precisam ser publicados globalmente.

AI pode usar eventos internos.

---

# 91. Civilization Events

```text id="event-130"
SettlementFounded
ElectionStarted
ElectionCompleted
TradeRouteCreated
MigrationStarted
ConflictStarted
TreatySigned
```

---

# 92. Economy Events

```text id="event-131"
PriceChanged
MarketUpdated
TradeCompleted
ResourceShortage
SupplyChainBroken
```

---

# 93. Ecology Events

```text id="event-132"
SpeciesPopulationChanged
Migration
DiseaseDetected
EcosystemDisturbed
PlantPopulationChanged
```

---

# 94. Climate Events

```text id="event-133"
WeatherStarted
StormFormed
TemperatureChanged
SeasonChanged
ClimateAnomaly
```

---

# 95. World Events

```text id="event-134"
VolcanicEvent
Earthquake
MeteorEvent
DimensionalEvent
FrontierEvent
```

---

# 96. Event Aggregation

Não publicar:

```text id="event-135"
temperature changed
```

milhões de vezes por segundo.

Podemos agrupar:

```text id="event-136"
ClimateRegionUpdated
```

por região/tick.

---

# 97. Coalescing

Eventos semelhantes podem ser combinados.

Exemplo:

```text id="event-137"
BlockChanged A
BlockChanged B
BlockChanged C
```

pode virar:

```text id="event-138"
ChunkBlockBatchChanged
```

quando apropriado.

---

# 98. Batch Events

API:

```text id="event-139"
publishBatch(events)
```

e eventos especializados:

```text id="event-140"
BlockBatchChangedEvent
EntityBatchMovedEvent
```

---

# 99. Backpressure

Se um sistema gerar eventos demais:

```text id="event-141"
queue overflow
```

não pode simplesmente travar tudo.

Políticas:

```text id="event-142"
BLOCK
DROP
MERGE
SAMPLE
DEFER
```

---

# 100. Event Importance

Cada evento pode ter:

```text id="event-143"
CRITICAL
IMPORTANT
NORMAL
LOW
DEBUG
```

---

# 101. Critical Events

Nunca perder:

```text id="event-144"
save commit
transaction commit
entity death
block transaction
network authoritative changes
```

---

# 102. Low Importance

Pode ser descartável:

```text id="event-145"
visual telemetry
debug traces
cosmetic effects
```

---

# 103. Event Logging

Durante desenvolvimento:

```text id="event-146"
Event Log
```

pode registrar:

```text id="event-147"
timestamp
event
source
causation
handlers
result
duration
```

---

# 104. Event Trace

Muito importante para debugar:

```text id="event-148"
Player action
 ↓
Command
 ↓
Event A
 ↓
Handler X
 ↓
Event B
 ↓
Handler Y
```

---

# 105. Event Profiler

Métricas:

```text id="event-149"
event count
dispatch time
handler time
queue latency
dropped events
coalesced events
```

---

# 106. Slow Handler Detection

Se um handler demorou demais:

```text id="event-150"
WARNING:
BlockBrokenEvent handler = 42ms
```

---

# 107. Handler Isolation

Um handler com erro não deve automaticamente derrubar todos os outros.

Política:

```text id="event-151"
handler exception
 ↓
log
 ↓
isolate
 ↓
continue
```

Mas para eventos críticos pode ser:

```text id="event-152"
FAIL TRANSACTION
```

---

# 108. Error Policy

Por subscription:

```text id="event-153"
IGNORE
LOG
DISABLE_HANDLER
FAIL_EVENT
FAIL_TRANSACTION
```

---

# 109. Mod Errors

Um mod com handler quebrado:

```text id="event-154"
examplemod handler crashes
```

pode ser isolado sem derrubar o Core.

---

# 110. Mod Subscription

Mod registra:

```text id="event-155"
subscribe("nexora:block_broken", handler)
```

mas o runtime deve oferecer typed APIs.

---

# 111. Mod Unload

Quando mod é descarregado:

```text id="event-156"
unsubscribe all owned subscriptions
```

---

# 112. Subscription Quotas

Mods podem ter limites:

```text id="event-157"
max subscriptions
max queued events
max handler time
```

---

# 113. Event Permissions

Alguns eventos são públicos.

Outros são privilegiados.

```text id="event-158"
PUBLIC
MOD
SERVER
INTERNAL
PRIVILEGED
```

---

# 114. Sensitive Events

Exemplo:

```text id="event-159"
server auth
security
admin operations
```

não deveriam ser expostos indiscriminadamente.

---

# 115. Networking

Nem todo evento deve ser transmitido pela rede.

Cada EventType pode declarar:

```text id="event-160"
LOCAL
SERVER_ONLY
CLIENT_ONLY
REPLICATED
NETWORK_COMMAND
```

---

# 116. Replicated Events

Exemplo:

```text id="event-161"
EntitySpawned
EntityRemoved
BlockChanged
```

quando necessário.

---

# 117. Client-only Events

Exemplo:

```text id="event-162"
CameraShake
UINotification
ParticleRequest
```

---

# 118. Server-only Events

Exemplo:

```text id="event-163"
TradeCommitted
EconomyUpdated
SaveCommitted
```

---

# 119. Event Serialization

Eventos públicos precisam de serializer.

```text id="event-164"
EventSerializer<T>
```

---

# 120. Event Version Migration

Replay/network antigos:

```text id="event-165"
Event v1
 ↓
Migration
 ↓
Event v2
```

---

# 121. Event Replay

Uma enorme vantagem do Event Bus.

Podemos gravar:

```text id="event-166"
event stream
```

e reproduzir.

Útil para:

```text id="event-167"
debug
tests
AI analysis
simulation validation
```

---

# 122. Replay Determinism

Para deterministic replay:

```text id="event-168"
world seed
world version
registry snapshot
event sequence
simulation settings
```

devem ser conhecidos.

---

# 123. Event Sourcing?

Eu **não** faria do NEXORA um sistema totalmente baseado em Event Sourcing.

Usaria:

```text id="event-169"
State + Events
```

e não:

```text id="event-170"
Everything = event log
```

Porque o mundo possui um volume gigantesco de estado.

---

# 124. Event History vs World State

Estado:

```text id="event-171"
chest inventory = 42 iron
```

Evento:

```text id="event-172"
PlayerTransferredIron
```

O Save mantém o estado.

O Event Bus comunica mudanças.

---

# 125. Persistence Integration

Eventos importantes podem notificar Save System:

```text id="event-173"
WorldStateChanged
```

Mas Save System não precisa salvar cada evento.

---

# 126. Dirty Tracking

Eventos podem marcar:

```text id="event-174"
chunk dirty
entity dirty
inventory dirty
economy dirty
```

---

# 127. Save Boundary

Exemplo:

```text id="event-175"
BlockChanged
 ↓
SaveDirtyMarker
```

e depois:

```text id="event-176"
SaveCommit
```

---

# 128. Event Bus and Registry

O Registry System registra os tipos:

```text id="event-177"
EventTypeRegistry
```

O Event Bus executa o transporte.

Separação:

```text id="event-178"
Registry
= identidade

Event Bus
= comunicação
```

---

# 129. Event Bus and Entity

```text id="event-179"
Entity System
      ↓
EntitySpawnedEvent
      ↓
Event Bus
```

---

# 130. Event Bus and Block

```text id="event-180"
Block System
      ↓
BlockChangedEvent
      ↓
Event Bus
```

---

# 131. Event Bus and Item

```text id="event-181"
Item System
      ↓
ItemTransferredEvent
      ↓
Event Bus
```

---

# 132. Event Bus and Combat

```text id="event-182"
Combat
 ↓
DamageAppliedEvent
 ↓
Event Bus
```

---

# 133. Event Bus and Machines

```text id="event-183"
Machine
 ↓
MachineCompletedEvent
 ↓
Event Bus
```

---

# 134. Event Bus and Civilization

```text id="event-184"
Civilization
 ↓
TradeCompletedEvent
 ↓
Event Bus
```

---

# 135. Event Bus and AI

AI pode ouvir:

```text id="event-185"
EntityDamaged
FoodShortage
WeatherChanged
TradeCompleted
SettlementThreatened
```

e atualizar objetivos.

---

# 136. Event Bus and Ecology

Ecology pode ouvir:

```text id="event-186"
PlantDestroyed
AnimalDied
FireStarted
DiseaseDetected
ClimateChanged
```

---

# 137. Event Bus and Climate

Climate pode reagir:

```text id="event-187"
large fire
pollution
terraforming
```

e modificar condições.

---

# 138. Event Bus and World Simulation

O próprio mundo passa a ser orientado a eventos:

```text id="event-188"
WORLD
 ↓
events
 ↓
systems react
 ↓
state changes
 ↓
new events
```

---

# 139. Event Bus and Scheduler

Scheduler decide **quando**.

Event Bus decide **quem recebe**.

```text id="event-189"
Scheduler
→ timing

Event Bus
→ routing
```

Não misturar os dois.

---

# 140. Event Bus and Task Manager

Task Manager:

```text id="event-190"
"preciso executar trabalho"
```

Event Bus:

```text id="event-191"
"algo aconteceu"
```

Eles são complementares.

---

# 141. Event Bus and Command System

Command:

```text id="event-192"
player wants action
```

Event:

```text id="event-193"
action happened
```

---

# 142. Event Bus and Network

Network transport:

```text id="event-194"
packet
```

Event Bus:

```text id="event-195"
semantic event
```

Não misturar.

---

# 143. Local Event Bus

Para alguns subsistemas:

```text id="event-196"
MachineLocalBus
ChunkLocalBus
EntityLocalBus
```

Pode existir, mas todos utilizam a infraestrutura central.

---

# 144. Event Scope

Cada evento pode definir:

```text id="event-197"
GLOBAL
WORLD
DIMENSION
REGION
CHUNK
ENTITY
SYSTEM
```

---

# 145. Scope Routing

Exemplo:

```text id="event-198"
EntityDamaged
scope = entity
```

só interessados naquela entidade precisam recebê-lo.

---

# 146. Event Affinity

Evento pode possuir:

```text id="event-199"
dimension
region
chunk
entity
```

para routing eficiente.

---

# 147. Spatial Event Bus

Em um mundo enorme, isto pode ser essencial.

```text id="event-200"
Global
   ↓
Dimension
   ↓
Region
   ↓
Chunk
```

---

# 148. Event Bubbling

Um evento pode subir:

```text id="event-201"
Entity
 ↓
Chunk
 ↓
Region
 ↓
World
 ↓
Global
```

Mas isso deve ser explicitamente configurado, não automático para tudo.

---

# 149. Event Capture

Pode haver:

```text id="event-202"
capture phase
```

antes do handler local.

Útil para proteção e filtros.

---

# 150. Event Propagation

Políticas:

```text id="event-203"
STOP
CONTINUE
BUBBLE
CAPTURE
```

---

# 151. Event Context Mutation

Evitaria permitir que qualquer handler altere arbitrariamente o evento.

Melhor:

```text id="event-204"
immutable event
```

e, em request events:

```text id="event-205"
mutable request context
```

---

# 152. Immutable Fact

Depois que aconteceu:

```text id="event-206"
BlockBrokenEvent
```

não deve mudar.

---

# 153. Mutable Request

Antes de executar:

```text id="event-207"
BlockBreakRequested
```

pode ter:

```text id="event-208"
cancelled
replacement
modified parameters
```

---

# 154. Event Security

Eventos de mod devem passar por validação.

Nunca confiar que:

```text id="event-209"
mod_event
```

é seguro apenas porque vem do Event Bus.

---

# 155. Event Quarantine

Evento inválido:

```text id="event-210"
schema invalid
source unauthorized
payload invalid
```

pode ser:

```text id="event-211"
rejected
quarantined
logged
```

---

# 156. Event Schema Registry

Essa integração fica:

```text id="event-212"
Registry System
├── EventTypeRegistry
└── EventSchemaRegistry
```

---

# 157. Event Contract

Cada evento público deve documentar:

```text id="event-213"
purpose
payload
phase
scope
cancelable
network behavior
persistence behavior
version
```

---

# 158. Public Event API

Mods podem fazer:

```text id="event-214"
subscribe<BlockBrokenEvent>()
```

sem conhecer a implementação interna do Build System.

---

# 159. Official Content

Vanilla também usa:

```text id="event-215"
Public Event API
```

igual aos mods.

---

# 160. Event Bus não deve esconder dependências reais

Se:

```text id="event-216"
Machine requires Energy API
```

isso continua sendo uma dependência explícita.

Event Bus não deve ser utilizado para mascarar toda arquitetura.

---

# 161. Event Bus Anti-pattern

Evitar:

```text id="event-217"
publish("do_machine_thing")
```

para esconder chamadas normais.

---

# 162. Use Event quando

```text id="event-218"
vários sistemas podem reagir
o produtor não precisa conhecer consumidores
o fato pode ocorrer independentemente
há interesse em extensibilidade
```

---

# 163. Use API direta quando

```text id="event-219"
resultado imediato é necessário
há forte relação funcional
é uma query
é um serviço interno
latência mínima é crítica
```

---

# 164. Exemplo

Bom:

```text id="event-220"
BlockBrokenEvent
→ Loot
→ Lighting
→ History
```

Não tão bom:

```text id="event-221"
getBlockState(position)
```

via Event Bus.

Isso deve ser API direta.

---

# 165. Event Bus API

Interface principal:

```text id="event-222"
IEventBus

publish()
publishDeferred()
subscribe()
unsubscribe()
schedule()
```

---

# 166. Event Dispatcher API

```text id="event-223"
IEventDispatcher

dispatch()
dispatchBatch()
flush()
```

---

# 167. Event Router API

```text id="event-224"
IEventRouter

resolveScope()
resolveChannel()
resolveSubscribers()
```

---

# 168. Subscription API

```text id="event-225"
ISubscription

eventType
handler
priority
filter
owner
scope
```

---

# 169. Event Context API

```text id="event-226"
IEventContext

timestamp
worldTime
source
actor
dimension
position
correlationId
causationId
```

---

# 170. Event Queue API

```text id="event-227"
IEventQueue

enqueue()
dequeue()
peek()
size()
clear()
```

---

# 171. Event Serializer

```text id="event-228"
IEventSerializer<T>

serialize()
deserialize()
```

---

# 172. Event Trace

```text id="event-229"
IEventTracer

begin()
handlerStart()
handlerEnd()
complete()
```

---

# 173. Event Metrics

```text id="event-230"
IEventMetrics

count
latency
queueDepth
failures
drops
```

---

# 174. Internal Architecture

```text id="event-231"
                EVENT BUS
                    │
          ┌─────────┴─────────┐
          ↓                   ↓
      EVENT ROUTER       EVENT QUEUE
          │                   │
          └─────────┬─────────┘
                    ↓
              DISPATCHER
                    │
        ┌───────────┼───────────┐
        ↓           ↓           ↓
     FILTER      PRIORITY     PHASE
        │           │           │
        └───────────┼───────────┘
                    ↓
                HANDLERS
                    │
          ┌─────────┼─────────┐
          ↓         ↓         ↓
       CORE       VANILLA     MODS
```

---

# 175. Código

Estrutura recomendada:

```text id="event-code-01"
src/
└── event/
    ├── core/
    │   ├── event.ts
    │   ├── event-type.ts
    │   ├── event-context.ts
    │   ├── event-id.ts
    │   └── event-result.ts
    │
    ├── bus/
    │   ├── event-bus.ts
    │   ├── event-router.ts
    │   └── event-dispatcher.ts
    │
    ├── subscription/
    │   ├── subscription.ts
    │   ├── subscription-handle.ts
    │   └── subscription-manager.ts
    │
    ├── queue/
    │   ├── event-queue.ts
    │   ├── deferred-queue.ts
    │   └── scheduled-queue.ts
    │
    ├── routing/
    │   ├── scope.ts
    │   ├── channel.ts
    │   └── spatial-routing.ts
    │
    ├── filtering/
    │   └── event-filter.ts
    │
    ├── registry/
    │   └── event-type-registry.ts
    │
    ├── serialization/
    │   └── event-serializer.ts
    │
    ├── replay/
    │   ├── event-recorder.ts
    │   └── event-replayer.ts
    │
    ├── diagnostics/
    │   ├── event-tracer.ts
    │   ├── event-profiler.ts
    │   └── event-metrics.ts
    │
    └── api/
        └── event-api.ts
```

---

# 176. Boot

Agora o boot do NEXORA pode ficar:

```text id="event-boot-01"
CORE
 ↓
REGISTRY MANAGER
 ↓
EVENT TYPE REGISTRY
 ↓
CREATE EVENT BUS
 ↓
LOAD SYSTEMS
 ↓
REGISTER EVENT HANDLERS
 ↓
VALIDATE SUBSCRIPTIONS
 ↓
FREEZE CORE EVENT TYPES
 ↓
GAME
```

---

# 177. Mod Loading

```text id="event-mod-01"
Mod
 ↓
RegistrationContext
 ↓
register content
 ↓
register event types
 ↓
register handlers
 ↓
validation
 ↓
freeze
```

---

# 178. Mod Isolation

Se um mod falhar:

```text id="event-mod-02"
handler crash
 ↓
disable handler
 ↓
log error
 ↓
mod health degraded
```

sem necessariamente destruir toda a simulação.

---

# 179. Performance

O Event Bus precisa ser projetado para alto volume.

Evitar:

```text id="event-perf-01"
new object
new array
string lookup
reflection
```

em cada evento de alta frequência.

---

# 180. Hot Path

Usar:

```text id="event-perf-02"
EventTypeID
precomputed subscriber lists
typed handlers
compact queues
```

---

# 181. Low Frequency vs High Frequency

### Low frequency

```text id="event-perf-03"
ElectionCompleted
RecipeCompleted
SettlementFounded
```

Pode tolerar abstrações maiores.

### High frequency

```text id="event-perf-04"
PhysicsCollision
EntityMoved
FluidFlow
Particle events
```

precisam de pipelines especializados.

---

# 182. Não colocar tudo no Event Bus

Essa é uma regra crítica.

Não transformar:

```text id="event-perf-05"
every voxel update
every physics calculation
every render operation
```

em eventos globais.

Isso criaria um gargalo.

---

# 183. Internal Event Batches

Sistemas de alta frequência podem usar buffers locais.

```text id="event-perf-06"
Physics
 ↓
local events
 ↓
batch
 ↓
Event Bus
```

---

# 184. ECS Compatibility

O Event Bus pode funcionar muito bem com ECS.

Entidades/componenentes geram:

```text id="event-ecs-01"
EntitySpawned
ComponentAdded
ComponentRemoved
```

O Event Bus não precisa implementar ECS.

---

# 185. Data-Oriented Systems

Eventos podem usar estruturas compactas:

```text id="event-ecs-02"
arrays
pools
ring buffers
```

onde necessário.

---

# 186. Lock-Free / Low-Lock

Para filas específicas pode-se futuramente usar estruturas de baixa contenção.

Mas primeiro:

```text id="event-187"
correctness
determinism
```

depois:

```text id="event-188"
optimization
```

---

# 187. Stress Testing

Teste:

```text id="event-test-01"
1,000 events/tick
10,000 events/tick
100,000 events/tick
1,000,000 events/tick
```

medindo:

```text id="event-test-02"
dispatch latency
CPU
memory
queue depth
GC/allocation
```

---

# 188. Handler Stress

```text id="event-test-03"
1 event
10 handlers

1 event
100 handlers

10,000 events
10 handlers each
```

---

# 189. Chain Stress

```text id="event-test-04"
A → B → C → D → ...
```

testar limites.

---

# 190. Failure Tests

Testar:

```text id="event-test-05"
handler crash
queue overflow
invalid event
duplicate event
cycle
mod unload
subscription leak
```

---

# 191. Determinism Tests

Rodar:

```text id="event-test-06"
same input
same event sequence
same world state
```

várias vezes e comparar.

---

# 192. Replay Test

```text id="event-test-07"
record
 ↓
reset world
 ↓
replay
 ↓
compare state
```

---

# 193. Mod Stress

```text id="event-test-08"
1,000 synthetic mods
```

com:

```text id="event-test-09"
subscriptions
events
dependencies
unload/reload
```

---

# 194. Memory Leak Test

Especialmente:

```text id="event-test-10"
subscribe
unsubscribe
reload mod
```

milhares de vezes.

Nenhuma subscription deve ficar presa.

---

# 195. Security Tests

Validar:

```text id="event-test-11"
unauthorized publish
invalid payload
forged source
forged actor
mod privilege escalation
```

---

# 196. Event API Versioning

```text id="event-api-01"
Event API v1
Event API v2
```

com adapters quando necessário.

---

# 197. Public Event Contracts

Eventos expostos a mods devem ter estabilidade maior que eventos internos.

Separar:

```text id="event-api-02"
PUBLIC EVENT
INTERNAL EVENT
EXPERIMENTAL EVENT
```

---

# 198. Event Documentation

Cada evento público deve ter documentação contendo:

```text id="event-doc-01"
Name
Purpose
Payload
Scope
Phase
Cancelable?
Authority
Network
Persistence
Version
Example
```

---

# 199. Primeiro vertical slice

```text id="event-vs-01"
EventTypeRegistry
        ↓
EventBus
        ↓
subscribe
        ↓
publish
        ↓
handler
        ↓
unsubscribe
```

---

# 200. Segundo vertical slice

```text id="event-vs-02"
Block System
 ↓
BlockBrokenEvent
 ↓
Event Bus
 ↓
Loot
 ↓
ItemStack
```

---

# 201. Terceiro vertical slice

```text id="event-vs-03"
Entity System
 ↓
EntitySpawnedEvent
 ↓
AI
Renderer
Network
```

---

# 202. Quarto vertical slice

```text id="event-vs-04"
Mod
 ↓
custom event
 ↓
register
 ↓
subscribe
 ↓
publish
 ↓
handler
 ↓
unload
 ↓
subscriptions removed
```

---

# 203. Quinto vertical slice

O teste mais importante:

```text id="event-vs-05"
Player
 ↓
Command
 ↓
Request Event
 ↓
Validation
 ↓
Operation
 ↓
Fact Event
 ↓
5+ systems react
```

sem nenhum desses sistemas conhecer diretamente todos os outros.

---

# 204. Ordem de implementação

```text id="event-order"
EVENT-0    Core Contract
EVENT-1    Event
EVENT-2    EventType
EVENT-3    EventID
EVENT-4    EventContext
EVENT-5    EventRegistry
EVENT-6    EventBus
EVENT-7    Subscribe
EVENT-8    Unsubscribe
EVENT-9    Dispatcher
EVENT-10   Queue
EVENT-11   Deferred Events
EVENT-12   Scheduled Events
EVENT-13   Priority
EVENT-14   Phases
EVENT-15   Channels
EVENT-16   Scope
EVENT-17   Filters
EVENT-18   Cancelable Requests
EVENT-19   Event Results
EVENT-20   Batch Events
EVENT-21   Coalescing
EVENT-22   Backpressure
EVENT-23   Error Isolation
EVENT-24   Mod Ownership
EVENT-25   Permissions
EVENT-26   Serialization
EVENT-27   Versioning
EVENT-28   Migration
EVENT-29   Network Events
EVENT-30   Replay
EVENT-31   Tracing
EVENT-32   Profiling
EVENT-33   Metrics
EVENT-34   Spatial Routing
EVENT-35   Determinism
EVENT-36   Stress Tests
EVENT-37   Security
EVENT-38   Compatibility
EVENT-39   Official Events
```

---

# 205. Mapa final

Com o que construímos até agora:

```text id="event-map-final"
                         NEXORA CORE
                              │
              ┌───────────────┴────────────────┐
              ↓                                ↓
        REGISTRY SYSTEM                    EVENT BUS
              │                                │
       identities/types                  communication
              │                                │
       ┌──────┼──────┐              ┌──────────┼─────────┐
       ↓      ↓      ↓              ↓          ↓         ↓
     BLOCK   ITEM  ENTITY         WORLD       SYSTEMS    MODS
       │      │      │              │          │
       └──────┴──────┴──────────────┴──────────┘
```

Agora os principais fluxos podem ser:

```text id="event-map-02"
BLOCK
 ↓
Build
 ↓
BlockBrokenEvent
 ↓
Event Bus
 ├── Loot
 ├── Lighting
 ├── Fluid
 ├── Physics
 ├── History
 └── Network
```

```text id="event-map-03"
ENTITY
 ↓
EntityDiedEvent
 ↓
Event Bus
 ├── Loot
 ├── AI
 ├── Civilization
 ├── Quest
 └── History
```

```text id="event-map-04"
ITEM
 ↓
ItemTransferredEvent
 ↓
Event Bus
 ├── Inventory
 ├── Economy
 ├── Quest
 ├── Research
 └── Network
```

---

# 206. Regra fundamental

Eu colocaria oficialmente no NEXORA:

> **Event Bus comunica fatos, solicitações e sinais entre sistemas. Ele não executa a lógica dos sistemas e não deve ser usado para esconder dependências funcionais que deveriam ser APIs explícitas.**

E uma segunda:

> **Commands solicitam. Systems executam. Events informam.**

Isso fecha uma separação muito importante:

```text id="event-final"
COMMAND
"faça isso"

API
"me dê essa informação"

EVENT
"isso aconteceu"

REGISTRY
"isso existe e significa X"
```

Com **Registry + Event Bus + Entity + Block + Item**, já temos uma base bem mais próxima de um **engine extensível de verdade**, porque agora identidade e comunicação estão desacopladas do conteúdo.

O próximo pilar estrutural é o **Save / Persistence System**, que vai definir como todo esse estado — Blocks, Items, Entities, BlockEntities, mundo, registries, versões, mods e histórico necessário — consegue sobreviver ao fechamento do jogo e a futuras versões do NEXORA.
