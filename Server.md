# NEXORA — SERVER SYSTEM

> **Princípio central:**
> **O Server é a autoridade do mundo.**
>
> Networking transporta dados.
> Simulation calcula o mundo.
> Persistence preserva o estado.
> **Server coordena a execução autoritativa dessas partes.**

A ideia principal é evitar que o Server vire outro “God System”.

```text id="server01"
SERVER
├── Bootstrap
├── Runtime
├── Session Management
├── Player Management
├── World Authority
├── Simulation Scheduling
├── Command Processing
├── Validation
├── Permissions
├── Network Integration
├── Persistence Integration
├── Mod Runtime Integration
├── Tick Management
├── Resource Budgets
├── Monitoring
├── Recovery
├── Shutdown
└── Administration
```

---

# 1. O que o Server é

O Server representa o **processo que executa uma instância autoritativa do NEXORA**.

Ele deve responder:

> “Qual estado real o mundo possui neste momento?”

Mas não deve implementar diretamente:

> “Como funciona cada sistema?”

Por exemplo:

```text id="server02"
Server
   │
   ├── chama Combat
   ├── chama Physics
   ├── chama World
   ├── chama Entity
   ├── chama Economy
   ├── chama Civilization
   └── chama Persistence
```

O Server coordena.

Os sistemas especializados fazem o trabalho.

---

# 2. Arquitetura principal

```text id="server03"
                     NEXORA SERVER
                          │
          ┌───────────────┼────────────────┐
          ▼               ▼                ▼
       BOOTSTRAP         RUNTIME        CONFIG
          │               │
          └───────┬───────┘
                  ▼
              WORLD INSTANCE
                  │
        ┌─────────┼─────────┐
        ▼         ▼         ▼
    PLAYERS    ENTITIES   WORLD
        │         │         │
        └─────────┼─────────┘
                  ▼
             SIMULATION
                  │
       ┌──────────┼──────────┐
       ▼          ▼          ▼
   COMMANDS     EVENTS    SYSTEMS
       │          │          │
       └──────────┼──────────┘
                  ▼
             PERSISTENCE
                  │
                  ▼
              NETWORKING
```

---

# 3. Server ≠ Networking

Isso precisa ficar extremamente rígido.

```text id="server04"
NETWORKING
→ transporta

SERVER
→ coordena e governa

SIMULATION
→ calcula
```

Exemplo:

```text id="server05"
Client
 ↓
Networking
 ↓
Server
 ↓
Command
 ↓
Simulation
 ↓
World Change
 ↓
Event Bus
 ↓
Networking
 ↓
Clients
```

O Networking não deve decidir quando um jogador pode quebrar um bloco.

O Server também não deve saber como o algoritmo de mineração funciona.

Quem resolve isso:

```text id="server06"
Build & Destruction
+
Tool API
+
Block System
+
Physics
```

---

# 4. Dedicated Server

O Server deve ter um modo dedicado real.

```text id="server07"
NEXORA SERVER
```

deve funcionar sem:

```text id="server08"
Renderer
Audio Backend
UI
GPU
Window
Animation Presentation
```

Pode carregar alguns tipos de dados compartilhados, mas não deve depender desses subsistemas.

Isso permite:

```text id="server09"
Linux
Windows
Containers
Cloud
Physical Server
Headless VM
```

---

# 5. Server Modes

Podemos ter:

```text id="server10"
Dedicated
Listen
Local
Singleplayer
Test
Headless
Replay
Benchmark
```

### Dedicated

```text
Servidor separado
```

### Listen

```text
Servidor + cliente no mesmo processo
```

### Local

```text
loopback
sem rede externa
```

### Test

```text
determinístico
controlado
```

Mas todos devem usar o mesmo modelo de autoridade.

---

# 6. Bootstrap

O boot do servidor não deve simplesmente:

```text
start()
```

Deve possuir fases.

```text id="server11"
PROCESS START
    ↓
LOAD CONFIG
    ↓
INITIALIZE LOGGING
    ↓
INITIALIZE CORE
    ↓
INITIALIZE REGISTRY
    ↓
INITIALIZE EVENT BUS
    ↓
LOAD MODS
    ↓
VALIDATE CONTENT
    ↓
INITIALIZE WORLD
    ↓
LOAD PERSISTENCE
    ↓
INITIALIZE SIMULATION
    ↓
INITIALIZE NETWORKING
    ↓
OPEN SERVER
    ↓
ACCEPT CONNECTIONS
    ↓
ACTIVE
```

---

# 7. Estados do Server

```text id="server12"
CREATED
↓
BOOTING
↓
LOADING
↓
READY
↓
STARTING
↓
RUNNING
↓
DEGRADED
↓
STOPPING
↓
STOPPED
```

`DEGRADED` é importante.

Por exemplo:

```text
uma região do mundo está corrompida
```

O servidor pode continuar ativo enquanto aquele problema é isolado.

---

# 8. Server Runtime

O Runtime é o coração operacional.

```text id="server13"
ServerRuntime
├── TickScheduler
├── SimulationScheduler
├── CommandQueue
├── EventQueue
├── WorldManager
├── PlayerManager
├── EntityManager
├── NetworkManager
├── PersistenceManager
├── ModManager
├── Metrics
└── ShutdownController
```

Mas cada manager deve continuar dono de uma área clara.

---

# 9. World Instance

O servidor hospeda uma ou mais instâncias de mundo.

```text id="server14"
Server
├── World A
├── World B
└── World C
```

Uma World Instance pode possuir:

```text
WorldID
Seed
WorldVersion
Dimensions
Rules
SimulationState
Persistence
Players
```

Isso abre caminho para:

```text
local worlds
multiverse servers
test worlds
instanced content
```

---

# 10. Dimension Hosting

O Server hospeda o Dimension System.

```text id="server15"
WORLD
├── Dimension 0
├── Dimension 1
├── Dimension 2
└── ...
```

O número de dimensões não deve ser hardcoded.

Podemos ter:

```text
16 configured dimensions
```

sem o Server precisar conhecer a lógica de cada uma.

---

# 11. Tick System

O servidor precisa de um relógio autoritativo.

```text id="server16"
SERVER CLOCK
     │
     ▼
TICK SCHEDULER
```

Mas nem tudo precisa rodar no mesmo tick.

---

# 12. Multiple Tick Scales

O NEXORA já trabalha com LOD.

O Server pode combinar isso com várias escalas:

```text id="server17"
FRAME
SECOND
TICK
MINUTE
HOUR
DAY
WEEK
SEASON
EVENT
```

Por exemplo:

```text
Player Physics
→ high frequency

Nearby AI
→ frequent

Regional Economy
→ seconds/minutes

Distant Civilization
→ minutes/events

Historical Simulation
→ event-driven
```

---

# 13. Fixed Simulation Step

A simulação autoritativa deve ter uma base temporal estável.

Conceitualmente:

```text id="server18"
wall clock
    ↓
accumulator
    ↓
fixed simulation step
```

Renderização pode variar.

A simulação deve ser muito mais previsível.

---

# 14. Tick Budget

Cada tick possui orçamento.

```text id="server19"
TICK
├── Input
├── Commands
├── Simulation
├── Events
├── Persistence Tasks
└── Replication
```

Monitorar:

```text
tick duration
CPU time
queue depth
entity work
chunk work
network work
```

---

# 15. Tick Overrun

Se uma etapa ultrapassar o orçamento:

```text id="server20"
NORMAL
   ↓
OVER BUDGET
   ↓
DEGRADED SCHEDULING
```

Em vez de simplesmente deixar o servidor entrar em uma espiral.

Pode:

```text
adiar trabalho não crítico
reduzir LOD
agrupar updates
postergar manutenção
```

Nunca sacrificar arbitrariamente operações críticas.

---

# 16. Simulation Scheduler

O Server coordena:

```text id="server21"
FULL
REGIONAL
ABSTRACT
```

Exemplo:

```text
Player region
→ FULL

Nearby city
→ REGIONAL

Far civilization
→ ABSTRACT
```

Essa decisão pode ser feita pelo Simulation Scheduler usando os sistemas especializados.

---

# 17. Command Queue

Input de clientes não deve alterar o mundo imediatamente.

Fluxo:

```text id="server22"
NETWORK
 ↓
DECODE
 ↓
VALIDATE NETWORK MESSAGE
 ↓
COMMAND
 ↓
SERVER COMMAND QUEUE
 ↓
SIMULATION
```

Isso cria uma fronteira muito importante.

---

# 18. Command System

O Server deve consumir comandos.

Exemplos:

```text id="server23"
MovePlayer
BreakBlock
PlaceBlock
Interact
Craft
TransferItem
Attack
UseItem
EnterVehicle
LeaveVehicle
TravelDimension
ExecuteServerCommand
```

O Server não implementa todos eles.

Ele encaminha ao sistema competente.

---

# 19. Command Lifecycle

```text id="server24"
RECEIVED
 ↓
STRUCTURAL VALIDATION
 ↓
AUTHENTICATION
 ↓
AUTHORIZATION
 ↓
WORLD VALIDATION
 ↓
SIMULATION
 ↓
RESULT
 ↓
EVENT
 ↓
REPLICATION
```

---

# 20. Server Authority

Por padrão:

```text id="server25"
SERVER = SOURCE OF TRUTH
```

Isso vale para:

```text
Blocks
Items
Entities
Combat
Inventory
Machines
Vehicles
Economy
Civilization
World Events
Progression
Quests
```

---

# 21. Authority não significa centralizar lógica

Esse detalhe é importante.

Não queremos:

```text id="server26"
Server
└── 100.000 linhas de regras
```

Queremos:

```text id="server27"
Server
 ├── Command routing
 ├── Authority boundary
 └── Scheduling

Specialized systems
 ├── Combat
 ├── Physics
 ├── Inventory
 ├── Economy
 └── ...
```

---

# 22. Player Manager

O Server mantém conexão entre:

```text
Account
Session
Player
Entity
Connection
```

Mas os conceitos continuam separados.

```text id="server28"
PlayerManager
├── connect
├── authenticate
├── join world
├── leave world
├── respawn
├── transfer
└── disconnect
```

---

# 23. Join Flow

```text id="server29"
CLIENT CONNECT
      ↓
HANDSHAKE
      ↓
AUTHENTICATION
      ↓
CREATE/RESTORE SESSION
      ↓
LOAD PLAYER
      ↓
VALIDATE PLAYER STATE
      ↓
LOCATE SPAWN
      ↓
LOAD INTEREST AREA
      ↓
REPLICATE
      ↓
READY
```

---

# 24. Leave Flow

```text id="server30"
DISCONNECT
 ↓
MARK SESSION
 ↓
STOP INPUT
 ↓
SAVE PLAYER
 ↓
SAVE CRITICAL STATE
 ↓
RELEASE INTEREST
 ↓
DETACH CONNECTION
```

A entidade do jogador pode:

```text
despawn
sleep
persist
```

dependendo da política.

---

# 25. Crash Safety

Um servidor não deve depender exclusivamente de:

```text
shutdown normal
```

Precisamos de:

```text id="server31"
autosave
checkpoint
journal
critical transaction commit
```

---

# 26. Server Recovery

Após crash:

```text id="server32"
START
 ↓
LOAD MANIFEST
 ↓
VERIFY SAVE
 ↓
LOAD SNAPSHOT
 ↓
RECOVER JOURNAL
 ↓
VALIDATE
 ↓
LOAD WORLD
 ↓
START SERVER
```

Isso vem do Persistence System.

O Server apenas coordena a recuperação.

---

# 27. Graceful Shutdown

Comando:

```text id="server33"
shutdown
```

deve fazer:

```text
STOP ACCEPTING CONNECTIONS
        ↓
ANNOUNCE SHUTDOWN
        ↓
FINISH CRITICAL COMMANDS
        ↓
SAVE PLAYERS
        ↓
SAVE WORLD
        ↓
CHECKPOINT
        ↓
FLUSH
        ↓
CLOSE NETWORK
        ↓
STOP SYSTEMS
        ↓
EXIT
```

---

# 28. Emergency Shutdown

Em uma falha grave:

```text id="server34"
SAVE EMERGENCY CHECKPOINT
↓
STOP NONCRITICAL
↓
FLUSH CRITICAL STATE
↓
TERMINATE
```

Não tentar executar operações perigosas quando o runtime já está corrompido.

---

# 29. Server + Persistence

Separação:

```text id="server35"
SERVER
"quando salvar?"

PERSISTENCE
"como salvar?"
```

O Server decide eventos operacionais como:

```text
scheduled save
shutdown save
checkpoint
critical save
```

Persistence executa o armazenamento.

---

# 30. Server + Event Bus

Server produz e consome eventos de alto nível.

Exemplos:

```text id="server36"
PlayerJoined
PlayerLeft
WorldLoaded
DimensionLoaded
ServerStarted
ServerStopping
TickStarted
TickCompleted
```

Mas:

```text
Server
```

não deve virar o único subscriber de todos os eventos do universo.

---

# 31. Server + Networking

Servidor recebe:

```text id="server37"
Requests
Inputs
Commands
```

e produz:

```text
Responses
State Replication
Events
Snapshots
```

O Networking faz o transporte.

---

# 32. Server + Registry

Server inicia o Registry System.

```text id="server38"
BOOT
 ↓
Registry
 ↓
Content
 ↓
Validation
 ↓
Freeze
 ↓
World
```

Depois de congelado:

```text
Registry = read-only runtime
```

exceto fluxos explicitamente suportados.

---

# 33. Server + Mod Runtime

O Server precisa hospedar mods.

Mas:

```text
Server ≠ Mod Loader
```

O Mod Runtime fornece:

```text
load
validate
initialize
sandbox
dependency
unload
```

O Server fornece o ambiente em que esses módulos rodam.

---

# 34. Mod lifecycle

```text id="server39"
DISCOVER
 ↓
LOAD
 ↓
VERIFY
 ↓
RESOLVE DEPENDENCIES
 ↓
INITIALIZE
 ↓
REGISTER CONTENT
 ↓
READY
 ↓
RUN
 ↓
STOP
 ↓
UNLOAD
```

Se um mod falhar:

```text
não significa necessariamente crash do Server
```

Ele pode entrar em:

```text
QUARANTINED
```

dependendo da gravidade.

---

# 35. Mod Failure Isolation

Precisamos distinguir:

```text id="server40"
recoverable
noncritical
critical
```

Exemplo:

```text
mod cosmetic falhou
→ server continua

mod civilization falhou
→ world feature degraded

core persistence failed
→ server stops safely
```

---

# 36. Permissions

Server deve possuir modelo de permissões.

```text id="server41"
Player
Moderator
Administrator
Console
System
Mod
```

Mas não colocar toda autorização dentro do Server.

Ele fornece:

```text
PermissionContext
```

e os sistemas validam suas próprias regras.

---

# 37. Console

O servidor deve ter interface administrativa.

Comandos:

```text id="server42"
server status
server stop
server save
server reload-config
server players
server world
server dimensions
server mods
server metrics
server tick
server profiler
```

E comandos dos sistemas:

```text
nexora entity ...
nexora block ...
nexora registry ...
nexora network ...
```

---

# 38. Admin Command Pipeline

```text id="server43"
Console
 ↓
Command Parser
 ↓
Authorization
 ↓
Command System
 ↓
Target System
 ↓
Simulation
 ↓
Event
```

Mesmo comandos administrativos importantes devem passar pelas mesmas fronteiras.

---

# 39. Monitoring

O Server precisa expor:

```text id="server44"
CPU
Memory
Tick Time
Entity Count
Chunk Count
Players
Network
Disk
Queues
Simulation Load
```

Por subsistema:

```text
Physics
Entity
AI
WorldGen
Fluid
Energy
Machines
Economy
Civilization
Networking
Persistence
```

---

# 40. Health System

Algo como:

```text id="server45"
ServerHealth
```

Estados:

```text
HEALTHY
DEGRADED
CRITICAL
STOPPING
```

Exemplo:

```text
Tick = normal
Network = normal
Disk = slow
Persistence = degraded
```

Servidor pode reportar:

```text
DEGRADED
```

antes de falhar completamente.

---

# 41. Resource Budgets

Cada servidor deve possuir limites configuráveis.

```text id="server46"
MaxPlayers
MaxEntities
MaxChunkLoads
MaxChunkGeneration
MaxCommandRate
MaxNetworkBandwidth
MaxMemory
MaxSimulationTime
MaxPendingTasks
```

---

# 42. Scheduler Budgets

Podemos possuir:

```text id="server47"
WorldGen Budget
AI Budget
Physics Budget
Persistence Budget
Replication Budget
Chunk Streaming Budget
```

Assim uma máquina de geração gigantesca não consegue consumir todo o servidor.

---

# 43. Backpressure Global

Se:

```text id="server48"
Chunk generation
```

estiver excedendo capacidade:

```text
queue grows
```

o Server deve reagir:

```text
throttle
 ↓
prioritize nearby players
 ↓
defer distant work
 ↓
reduce prefetch
```

---

# 44. Multi-World

A arquitetura pode permitir:

```text id="server49"
Server Process
├── World A
├── World B
└── Test World
```

Mas isso não deve obrigatoriamente significar que todas sejam simuladas com a mesma prioridade.

---

# 45. World Scheduling

Exemplo:

```text id="server50"
ACTIVE WORLD
→ FULL

DORMANT WORLD
→ REGIONAL

EMPTY WORLD
→ ABSTRACT / PAUSED
```

Isso pode ser enorme para servidores de testes ou instâncias.

---

# 46. Empty World Behavior

Quando não há players:

```text id="server51"
No players
```

não significa necessariamente:

```text
world frozen
```

Civilizações podem continuar.

Mas em LOD:

```text
FULL → REGIONAL → ABSTRACT
```

O mundo continua vivo sem gastar CPU desnecessária.

---

# 47. Server Scaling

O primeiro desenho pode ser:

```text id="server52"
ONE SERVER
ONE WORLD
```

Depois:

```text id="server53"
ONE SERVER
MULTIPLE WORLDS
```

e futuramente:

```text id="server54"
SERVER CLUSTER
```

Mas o Core não deve depender de cluster.

---

# 48. Sharding futuro

O NEXORA pode futuramente distribuir regiões:

```text id="server55"
WORLD
├── Region A → Server 1
├── Region B → Server 2
├── Region C → Server 3
└── Region D → Server 4
```

Isso precisa influenciar o design agora, mas não precisa ser implementado inicialmente.

Por isso sistemas devem possuir interfaces como:

```text id="server56"
WorldRegionAuthority
EntityAuthority
```

em vez de assumir que:

```text
one process = entire universe
```

---

# 49. Region Transfer

Futuramente:

```text id="server57"
Server A
   ↓
Region Migration
   ↓
Server B
```

Isso é diferente de:

```text
Dimension Transfer
```

Não confundir os dois.

---

# 50. Server Cluster

Arquitetura futura:

```text id="server58"
                 CLUSTER
                    │
         ┌──────────┼──────────┐
         ▼          ▼          ▼
      SERVER A   SERVER B   SERVER C
         │          │          │
       Regions    Regions    Regions
```

Um cluster manager poderia cuidar de:

```text
placement
health
migration
routing
```

Mas isso é infraestrutura futura.

---

# 51. Anti-Cheat

O Server é a principal defesa.

Modelo:

```text id="server59"
CLIENT
  ↓
REQUEST
  ↓
SERVER VALIDATION
  ↓
SIMULATION
```

Nunca:

```text id="server60"
CLIENT
  ↓
"confie em mim"
```

---

# 52. Exploit Detection

Não precisamos transformar o Server em sistema anti-cheat gigantesco.

Ele pode fornecer sinais:

```text id="server61"
impossible movement
invalid transaction
impossible inventory
invalid interaction
rate anomaly
sequence anomaly
```

Um sistema especializado pode analisar esses sinais.

---

# 53. Persistence Barrier

Antes de certas operações críticas:

```text id="server62"
TRADE
PLAYER TRANSFER
WORLD MIGRATION
SERVER SHUTDOWN
```

o Server pode solicitar:

```text
Persistence barrier
```

Garantindo que o estado crítico tenha sido duravelmente registrado.

---

# 54. Transaction Coordinator

Algumas operações cruzam vários sistemas.

Exemplo:

```text id="server63"
NPC compra item
```

envolve:

```text
Economy
+
Inventory
+
Item
+
Quest
+
Persistence
+
Networking
```

O Server pode coordenar o contexto da transação.

Mas os sistemas continuam donos das regras.

---

# 55. Server Transaction

```text id="server64"
BEGIN
 ↓
Validate
 ↓
Reserve
 ↓
Execute
 ↓
Publish Events
 ↓
Commit
 ↓
Replicate
```

Em falha:

```text
ROLLBACK
```

quando suportado pelo sistema.

---

# 56. Save consistency

O Server deve evitar:

```text
network state
≠
simulation state
≠
persistence state
```

Em operações críticas:

```text
simulation commit
 ↓
persistence commit
 ↓
replication
```

ou fluxo equivalente conforme o sistema.

---

# 57. Server Event Loop

Uma organização inicial pode ser:

```text id="server65"
while running:

    receive network input

    validate/decode

    enqueue commands

    process critical commands

    run simulation tick

    dispatch events

    process persistence work

    build replication snapshot

    send network updates

    update metrics
```

O código real deve ser dividido em serviços, mas o conceito é esse.

---

# 58. Não deixar tudo síncrono

Operações potencialmente pesadas:

```text id="server66"
WorldGen
Disk I/O
Compression
Large serialization
Pathfinding
AI analysis
```

não devem bloquear desnecessariamente o tick principal.

Arquitetura:

```text
MAIN SIMULATION
       │
       ├── worker queues
       │
       └── asynchronous jobs
```

Mas o resultado volta por uma fronteira segura.

---

# 59. Determinismo

O Server deve tentar manter simulação determinística sempre que possível.

Especialmente:

```text Physics
World Events
Generation
Combat calculations
Machine processing
```

Isso ajuda:

```text testing
replay
debug
recovery
prediction
```

Mas não devemos assumir que absolutamente tudo será bit-for-bit determinístico em toda plataforma.

---

# 60. Server Replays

O Server pode gravar:

```text id="server67"
inputs
commands
important events
world checkpoints
tick identifiers
```

Depois:

```text
replay
```

para debugging.

---

# 61. Tick Replay

Um bug:

```text
NPC entrou na cidade
→ economia ficou inconsistente
```

Podemos:

```text
load checkpoint
↓
replay commands/events
↓
observe divergence
```

Isso será extremamente útil no projeto.

---

# 62. Server Debugger

Comandos:

```text id="server68"
nexora server status
nexora server health
nexora server tick
nexora server queues
nexora server systems
nexora server players
nexora server worlds
nexora server profiling
nexora server replay
```

---

# 63. Profiling por subsistema

Algo como:

```text id="server69"
Tick 183829

Physics       4.2 ms
AI            6.7 ms
Entities      3.1 ms
World         2.8 ms
Machines      1.4 ms
Fluids        0.7 ms
Networking    2.3 ms
Persistence   0.3 ms
Other         0.9 ms

Total        22.4 ms
```

Isso permite encontrar gargalos reais.

---

# 64. Watchdog

Um watchdog externo ou interno pode detectar:

```text id="server70"
tick stall
deadlock
infinite queue
unresponsive subsystem
```

Por exemplo:

```text
tick > threshold
```

gera diagnóstico.

Não deve simplesmente matar o processo sem coletar contexto.

---

# 65. Graceful degradation

A arquitetura do NEXORA já combina muito com isso.

Exemplo:

```text id="server71"
CPU overloaded
```

Scheduler pode:

```text
FULL mobs → REGIONAL
reduce distant AI
reduce cosmetic replication
defer chunk prefetch
reduce background simulation
```

Mas mantém:

```text
Player input
critical simulation
inventory
persistence
security
```

---

# 66. APIs públicas

```text id="server72"
IServer
IServerRuntime
IServerBootstrap
IServerLifecycle
IServerWorldManager
IServerPlayerManager
IServerCommandManager
IServerScheduler
IServerAuthority
IServerPermissions
IServerMetrics
IServerHealth
IServerAdmin
IServerRecovery
IServerShutdown
```

---

# 67. Organização de código

```text id="server73"
src/server/

├── core/
│   ├── server
│   ├── runtime
│   ├── context
│   └── config
│
├── bootstrap/
│   ├── bootstrap
│   ├── phases
│   └── startup
│
├── lifecycle/
│   ├── lifecycle
│   ├── shutdown
│   └── recovery
│
├── world/
│   ├── world-manager
│   ├── world-instance
│   └── dimension-host
│
├── player/
│   ├── player-manager
│   ├── sessions
│   └── join-leave
│
├── simulation/
│   ├── scheduler
│   ├── tick
│   ├── budgets
│   └── lod
│
├── command/
│   ├── command-manager
│   ├── queue
│   └── validation
│
├── authority/
│   ├── authority
│   ├── permissions
│   └── ownership
│
├── networking/
│   └── adapters
│
├── persistence/
│   └── adapters
│
├── mods/
│   └── runtime-adapters
│
├── monitoring/
│   ├── metrics
│   ├── health
│   └── watchdog
│
├── admin/
│   ├── console
│   └── commands
│
└── debug/
```

---

# 68. Dependências

A relação principal:

```text id="server74"
CORE
 │
 ├── Registry
 ├── Event Bus
 ├── Persistence
 ├── Entity
 ├── World
 ├── Physics
 └── ...
       │
       ▼
     SERVER
       │
       ├── Networking
       ├── Mod Runtime
       └── Command System
```

Na prática, o Server é uma camada de **runtime/orquestração** acima da fundação.

---

# 69. Ordem de implementação

## SERVER-0 — Server Core

Criar:

```text
IServer
ServerRuntime
ServerContext
ServerState
```

---

## SERVER-1 — Bootstrap

Implementar:

```text
boot phases
config
logging
lifecycle
```

---

## SERVER-2 — Tick

Criar:

```text
fixed tick
clock
scheduler
budget
```

---

## SERVER-3 — World Host

```text
WorldManager
WorldInstance
DimensionHost
```

---

## SERVER-4 — Local Loopback

```text
Local client
↕
Server
```

---

## SERVER-5 — Networking Integration

Conectar:

```text
Networking
↓
Server
```

---

## SERVER-6 — Command Queue

```text
Network request
↓
Command
↓
Queue
↓
Simulation
```

---

## SERVER-7 — Player Lifecycle

```text
join
load
spawn
play
leave
save
```

---

## SERVER-8 — Authority

Implementar:

```text
server authority
permissions
validation boundary
```

---

## SERVER-9 — Persistence

Integrar:

```text
load world
save world
checkpoint
recovery
```

---

## SERVER-10 — Mod Runtime

```text
discover
load
validate
initialize
run
stop
```

---

## SERVER-11 — Monitoring

```text
metrics
health
profiling
watchdog
```

---

## SERVER-12 — Admin

```text
console
commands
permissions
diagnostics
```

---

## SERVER-13 — LOD Scheduling

```text
FULL
REGIONAL
ABSTRACT
```

integrado ao scheduler.

---

## SERVER-14 — Recovery

```text
crash recovery
partial recovery
safe mode
```

---

## SERVER-15 — Multi-World

```text
multiple world instances
```

---

## SERVER-16 — Scale Testing

```text
10 players
50
100
250
500
1000+
```

---

# 70. Primeiro Vertical Slice

O primeiro slice real deveria ser:

```text id="server75"
SERVER
 +
CLIENT
 +
WORLD
```

Fluxo:

```text
Start Server
    ↓
Load Registry
    ↓
Load World
    ↓
Open Networking
    ↓
Client Connects
    ↓
Authenticate
    ↓
Create Session
    ↓
Load Player
    ↓
Spawn Entity
    ↓
Send Snapshot
    ↓
Client Moves
    ↓
Input
    ↓
Server Simulation
    ↓
Authoritative State
    ↓
Replication
```

---

# 71. Segundo Vertical Slice

```text id="server76"
Client A
   ↓
Break Block Request
   ↓
Server
   ↓
Build & Destruction
   ↓
Block System
   ↓
Event Bus
   ↓
Loot
   ↓
Item
   ↓
Persistence
   ↓
Replication
   ↓
Client A + Client B
```

---

# 72. Terceiro Vertical Slice

Economia:

```text id="server77"
NPC
 ↓
Trade Request
 ↓
Server
 ↓
Economy
 ↓
Inventory
 ↓
Item
 ↓
Transaction
 ↓
Persistence
 ↓
Event
 ↓
Replication
```

Isso prova que o Server consegue coordenar múltiplos sistemas sem possuir as regras deles.

---

# 73. Testes críticos

### Boot test

```text
Server starts
Registry ready
World ready
Networking ready
```

### Join test

```text
connect
load
spawn
replicate
```

### Save test

```text
modify
save
restart
restore
```

### Authority test

```text
fake client command
→ rejected
```

### Recovery test

```text
crash
restart
journal recovery
world valid
```

---

# 74. Stress Tests

```text id="server78"
100 players
1.000 entities
10.000 entities
100.000 abstract entities
millions of regional/abstract entities
large cities
large automation networks
mass chunk generation
many dimensions
```

E combinar:

```text
players
+
vehicles
+
cities
+
machines
+
AI
+
network traffic
+
saving
```

---

# 75. Chaos Tests

Muito importante para Server:

```text id="server79"
random disconnects
packet loss
high latency
chunk generation failures
disk delay
mod failure
entity explosion in count
memory pressure
network flood
save interruption
server restart
```

O objetivo é descobrir:

> “O que acontece quando tudo dá errado ao mesmo tempo?”

---

# 76. Golden Server Test

Esse deve ficar no CI.

```text id="server80"
1 Server
2 Clients
1 World
1 Dimension

Client A joins
Client B joins

A places block
B observes

A moves
B observes

B picks item
A observes

World saves

Server shuts down

Server restarts

A reconnects
B reconnects

State remains correct
```

---

# 77. Regra de isolamento

O Server nunca deve virar isso:

```text id="server81"
SERVER
├── Combat code
├── Physics code
├── AI code
├── Economy code
├── Inventory code
├── Block code
├── Item code
└── EVERYTHING
```

Deve permanecer:

```text id="server82"
SERVER
├── Lifecycle
├── Scheduling
├── Authority
├── Coordination
├── Runtime
├── Recovery
└── Integration
```

---

# 78. Arquitetura final

```text id="server83"
                         NEXORA
                            │
                     ┌──────┴──────┐
                     │     CORE    │
                     └──────┬──────┘
                            │
         ┌──────────────────┼──────────────────┐
         ▼                  ▼                  ▼
      REGISTRY          EVENT BUS          PERSISTENCE
         │                  │                  │
         └──────────────────┼──────────────────┘
                            │
                         SERVER
                            │
       ┌────────────┬───────┼────────┬─────────────┐
       ▼            ▼       ▼        ▼             ▼
   SCHEDULER     COMMAND  WORLD    PLAYER      AUTHORITY
       │            │       │        │             │
       └────────────┴───────┼────────┴─────────────┘
                            ▼
                       SIMULATION
                            │
        ┌─────────┬─────────┼─────────┬─────────┐
        ▼         ▼         ▼         ▼         ▼
      ENTITY    PHYSICS    BLOCK     ITEM      AI
        │         │         │         │         │
        └─────────┴─────────┴─────────┴─────────┘
                            │
                            ▼
                       EVENT BUS
                            │
                    ┌───────┴───────┐
                    ▼               ▼
              PERSISTENCE       NETWORKING
                                      │
                                      ▼
                                   CLIENT
```

E a regra definitiva fica:

```text
NETWORKING
→ "como os dados chegam?"

SERVER
→ "quem coordena e possui autoridade?"

SIMULATION
→ "o que aconteceu?"

SYSTEMS
→ "como cada mecânica funciona?"

PERSISTENCE
→ "como o estado sobrevive?"

EVENT BUS
→ "quem precisa saber?"

REGISTRY
→ "o que cada ID significa?"
```

O próximo nível natural da fundação é o **Mod Runtime/Loader**, porque o Server já está desenhado para hospedar conteúdo modular, e depois o **Command System** e **Scripting**, que fecham a camada de extensibilidade do NEXORA.
