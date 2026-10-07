# NEXORA — NETWORKING SYSTEM

> **Princípio central:**
> **Networking transporta e sincroniza dados entre processos. Ele não é dono da simulação do mundo.**

A divisão fundamental do NEXORA deve ser:

```text
CLIENT
   │
   ▼
NETWORKING
   │
   ▼
COMMAND / REQUEST
   │
   ▼
SERVER
   │
   ▼
SIMULATION
   │
   ├── WORLD
   ├── ENTITY
   ├── BLOCK
   ├── ITEM
   ├── PHYSICS
   ├── FLUID
   ├── ENERGY
   ├── MACHINE
   ├── COMBAT
   └── ...
   │
   ▼
STATE CHANGE
   │
   ▼
EVENT BUS
   │
   ▼
NETWORK REPLICATION
   │
   ▼
CLIENTS
```

---

# 1. O que o Networking é

O Networking precisa responder:

> **“Como o estado e as operações chegam de um processo para outro?”**

Ele não deve responder:

> “O jogador pode quebrar esse bloco?”

Isso é responsabilidade do servidor + Build & Destruction.

Não deve responder:

> “Quanto dano essa espada causa?”

Isso é Combat.

Não deve responder:

> “Como esse NPC pensa?”

Isso é AI.

Não deve responder:

> “Como esse bloco existe no mundo?”

Isso é Block System.

Networking somente transporta as informações necessárias.

---

# 2. Arquitetura

```text
NETWORKING
├── Connection
├── Session
├── Transport
├── Protocol
├── Packet
├── Message
├── Channel
├── Serialization
├── Handshake
├── Synchronization
├── Replication
├── Interest Management
├── Input
├── Commands
├── Prediction
├── Reconciliation
├── Interpolation
├── Authority
├── Network Clock
├── Scheduler
├── Bandwidth Control
├── Reliability
├── Compression
├── Security
├── Voice
├── Mod Networking
├── Replay
├── Metrics
├── Diagnostics
└── API
```

---

# 3. Separação CLIENT / SERVER

O NEXORA deve trabalhar com uma separação muito forte.

```text
CLIENT
├── Input
├── UI
├── Renderer
├── Audio
├── Animation
├── Local Prediction
├── Interpolation
└── Presentation

SERVER
├── World Simulation
├── Physics Authority
├── Entity Authority
├── Block Authority
├── Inventory Authority
├── Combat Authority
├── Economy Authority
├── Civilization
├── AI
└── Persistence
```

O cliente pode prever algumas coisas para reduzir latência.

Mas:

```text
CLIENT ≠ AUTORIDADE
```

Por padrão:

```text
SERVER = AUTORIDADE
```

---

# 4. Dedicated Server

Isso é extremamente importante para NEXORA.

O servidor dedicado deve poder funcionar sem:

```text
Renderer
Audio
UI
Animation
GPU
Windowing
```

Arquitetura:

```text
NEXORA SERVER
├── Core
├── Registry
├── Event Bus
├── World
├── Entity
├── Physics
├── Block
├── Item
├── Fluid
├── Energy
├── Machine
├── Civilization
├── AI
├── Persistence
├── Networking
└── Mod Runtime
```

Sem:

```text
Renderer
Audio Backend
UI
GPU
```

Isso permite servidores enormes.

---

# 5. Connection

Uma conexão representa o canal físico/lógico entre dois processos.

```text
Connection
├── ConnectionID
├── RemoteEndpoint
├── LocalEndpoint
├── State
├── Protocol
├── Statistics
├── Encryption
├── Queues
├── Channels
└── Session
```

Estados:

```text
DISCONNECTED
    ↓
CONNECTING
    ↓
CONNECTED
    ↓
AUTHENTICATING
    ↓
SYNCHRONIZING
    ↓
READY
    ↓
DISCONNECTING
    ↓
DISCONNECTED
```

---

# 6. Session

Connection e Session não devem ser a mesma coisa.

Uma conexão pode cair.

A sessão pode continuar existindo por alguns segundos/minutos.

```text
Account
   │
   ▼
Session
   │
   ├── Player
   ├── Permissions
   ├── Network State
   ├── World State
   └── Replication State
        │
        ▼
Connection
```

Isso permite:

```text
disconnect
↓
reconnect
↓
restore session
```

---

# 7. Identidades

Não misture:

```text
AccountID
PlayerID
EntityID
ConnectionID
SessionID
PersistentEntityUUID
```

Cada um possui função diferente.

Exemplo:

```text
AccountID
    ↓
PlayerID
    ↓
EntityID
    ↓
ConnectionID
```

Um player pode trocar de conexão.

Uma entidade pode existir sem player.

---

# 8. Transport Layer

Networking não deve ficar preso a uma biblioteca específica.

Criamos uma abstração:

```text
INetworkTransport
```

Possíveis implementações futuras:

```text
UDP
QUIC
TCP
WebSocket
InProcess
Loopback
TestTransport
```

O restante do NEXORA não deve depender diretamente de uma implementação.

---

# 9. Protocol Layer

Acima do transporte:

```text
Transport
    ↓
Protocol
    ↓
Packet
    ↓
Message
```

Protocol controla:

```text
version
framing
sequence
acknowledgement
compression
serialization
message types
channels
compatibility
```

---

# 10. Packet

Um packet é a unidade transportada.

Exemplo conceitual:

```text
Packet
├── ProtocolVersion
├── ConnectionID
├── ChannelID
├── SequenceNumber
├── Flags
├── PayloadLength
├── Payload
└── Check/Integrity
```

Nunca assuma que:

```text
1 Packet = 1 Game Event
```

Um packet pode carregar vários messages.

---

# 11. Message

Mensagem é a unidade lógica.

Exemplos:

```text
HelloMessage
AuthenticationMessage
RegistrySyncMessage
WorldSyncMessage
PlayerInputMessage
BlockBreakRequest
BlockChangeMessage
EntitySpawnMessage
EntityDeltaMessage
InventoryTransactionMessage
ChunkDataMessage
ChunkUnloadMessage
DimensionTransferMessage
```

---

# 12. Channels

O NEXORA deve ter diferentes classes de tráfego.

```text
CONTROL
RELIABLE
RELIABLE_ORDERED
RELIABLE_UNORDERED
UNRELIABLE
UNRELIABLE_SEQUENCED
STATE
INPUT
BULK
VOICE
```

Não devemos usar o mesmo tratamento para tudo.

Por exemplo:

### Controle

```text
Handshake
Authentication
Disconnect
Dimension Transfer
```

Precisa de alta confiabilidade.

### Movimento

Pode usar:

```text
Unreliable + Sequence
```

Porque uma posição velha pode simplesmente ser descartada.

### Inventário

```text
Reliable
```

Porque perder uma transação pode causar inconsistência.

---

# 13. Sequence Numbers

Mensagens precisam de sequência.

```text
1
2
3
4
5
```

Se chegar:

```text
1
2
4
5
```

sabemos que:

```text
3
```

está faltando.

Para certos tipos de dados:

```text
Mensagem 5
```

pode tornar:

```text
Mensagem 4
```

irrelevante.

Isso é extremamente útil para:

```text
posição
rotação
estado visual
alguns estados temporais
```

---

# 14. ACK

Sistema de acknowledgement:

```text
Packet
   ↓
ACK
```

Permite detectar:

```text
loss
timeout
retransmission
```

Mas não precisamos implementar confiabilidade absoluta para todos os canais.

---

# 15. Network Clock

Cliente e servidor possuem relógios diferentes.

Precisamos de:

```text
NetworkClock
```

Com:

```text
ServerTick
ServerTime
ClientEstimatedServerTime
RTT
Jitter
Offset
InterpolationDelay
```

Isso será essencial para:

```text
Physics
Combat
Animation
Projectiles
Vehicles
Prediction
Replay
```

---

# 16. Replication System

Essa é uma das partes mais importantes do Networking.

O servidor possui:

```text
MUNDO INTEIRO
```

Mas não envia:

```text
MUNDO INTEIRO
```

para cada jogador.

Precisamos de:

```text
ReplicationManager
```

---

# 17. Interest Management

Cada cliente recebe apenas o que precisa.

```text
World
├── Region A
├── Region B
├── Region C
├── Region D
└── ...
```

Jogador:

```text
PLAYER
   │
   ▼
INTEREST MANAGER
   │
   ├── Nearby Chunks
   ├── Nearby Entities
   ├── Relevant Structures
   ├── Dimension
   ├── Visible Events
   └── Permission Scope
```

---

# 18. Network Distance ≠ Render Distance

Não devemos misturar:

```text
Render Distance
Simulation Distance
Network Distance
```

Exemplo:

```text
Render Distance = 16 chunks
Network Distance = 24 chunks
Simulation Distance = 12 chunks
```

Pode haver situações em que:

```text
algo não está sendo renderizado
```

mas ainda precisa estar sendo sincronizado.

---

# 19. Interest Management por camadas

```text
Chunk Interest
Entity Interest
Structure Interest
Event Interest
Dimension Interest
Permission Interest
Knowledge Interest
```

Isso é especialmente importante para o sistema de civilizações.

Um jogador pode estar perto de uma cidade.

Ele pode receber:

```text
edifícios
NPCs próximos
eventos importantes
comércio relevante
```

mas não precisa receber todos os detalhes da economia mundial.

---

# 20. Replication LOD

Seguindo a arquitetura geral do NEXORA:

```text
FULL
REGIONAL
ABSTRACT
```

### FULL

```text
NPC individual
posição
rotação
animação
estado
equipamentos
```

### REGIONAL

```text
população
atividade
produção
movimentação agregada
```

### ABSTRACT

```text
cidade
população aproximada
economia
eventos históricos
```

Isso permite mundos enormes.

---

# 21. Entity Replication

Entity System fornece:

```text
Entity
Components
State
Transform
```

Networking decide:

```text
o cliente precisa receber isso?
```

Exemplo:

```text
Entity
├── Transform
├── Health
├── Inventory
├── AI
├── Animation
└── Equipment
```

Um cliente pode receber:

```text
Transform
Health
Equipment
AnimationState
```

sem receber:

```text
AI internal state
```

---

# 22. Component Replication

Cada componente pode possuir política própria.

```text
ReplicationPolicy
├── NEVER
├── SERVER_ONLY
├── OWNER_ONLY
├── RELEVANT
├── ALL
└── CUSTOM
```

Exemplo:

```text
AI planning state
→ SERVER_ONLY

Position
→ RELEVANT

Inventory
→ OWNER_ONLY / RELEVANT

Cosmetic animation
→ RELEVANT
```

---

# 23. Full Snapshot

Quando um cliente entra:

```text
Server
 ↓
Snapshot
 ↓
Client
```

O snapshot pode conter:

```text
World Metadata
Dimension
Registry Fingerprint
Chunks
Entities
Player
Structures
Relevant State
```

Depois disso:

```text
Delta Updates
```

---

# 24. Delta Replication

Não precisamos enviar:

```text
Entity:
x=100
y=64
z=200
health=20
rotation=30
```

a cada tick.

Podemos enviar:

```text
x += 0.2
rotation += 3
```

Ou um delta compactado.

---

# 25. Baselines

Cliente mantém:

```text
BASELINE
```

Servidor pode enviar:

```text
DELTA FROM BASELINE
```

Isso reduz muito o tráfego.

---

# 26. Chunk Networking

Um chunk precisa de ciclo próprio:

```text
REQUEST
    ↓
GENERATE / LOAD
    ↓
SERIALIZE
    ↓
COMPRESS
    ↓
SEND
    ↓
DECODE
    ↓
VALIDATE
    ↓
INSTALL
```

Mensagens:

```text
ChunkRequest
ChunkData
ChunkDelta
ChunkUnload
ChunkResync
```

---

# 27. Chunk Prefetch

O servidor pode antecipar:

```text
player movement
vehicle movement
rail movement
teleport destination
```

e enviar chunks antes de o jogador chegar.

```text
CURRENT AREA
     ↓
PREFETCH AREA
```

Isso reduz pop-in.

---

# 28. Input Networking

O cliente envia:

```text
InputSnapshot
```

Exemplo:

```text
InputSnapshot
├── Sequence
├── Timestamp
├── Movement
├── Look
├── Buttons
├── Actions
└── Context
```

O servidor processa.

```text
CLIENT
 ↓
INPUT
 ↓
SERVER
 ↓
SIMULATION
 ↓
AUTHORITATIVE STATE
```

---

# 29. Client Prediction

Para movimento:

```text
Client
 ↓
Input
 ↓
Local Prediction
 ↓
Visual Response
```

Ao mesmo tempo:

```text
Input
 ↓
Server
 ↓
Authoritative Simulation
```

Servidor responde com:

```text
Authoritative State
```

Cliente compara.

---

# 30. Reconciliation

Se:

```text
Predicted Position ≠ Server Position
```

fazemos:

```text
Server State
↓
Reconcile
↓
Replay Unacknowledged Inputs
↓
New Predicted State
```

Isso é importante para:

```text
movement
vehicle
flying
swimming
climbing
```

---

# 31. Interpolation

Para outros jogadores:

```text
STATE A
   ↓
interpolation
   ↓
STATE B
```

Em vez de mostrar movimento quebrado.

---

# 32. Extrapolation

Quando um estado ainda não chegou:

```text
last known velocity
+
last known direction
```

pode produzir uma estimativa temporária.

Mas deve haver limites.

---

# 33. Authority

O sistema deve possuir:

```text
SERVER_AUTHORITY
OWNER_AUTHORITY
SHARED_AUTHORITY
LOCAL
```

Padrão:

```text
SERVER_AUTHORITY
```

Exemplo:

### Player Movement

```text
CLIENT
→ prediction

SERVER
→ authority
```

### Decorative Client Effect

```text
CLIENT
→ LOCAL
```

### World Block

```text
SERVER
→ AUTHORITY
```

---

# 34. Commands

O cliente nunca deveria simplesmente falar:

```text
"bloco quebrou"
```

Ele deve dizer:

```text
"quero quebrar este bloco"
```

Então:

```text
Client
 ↓
BreakBlockRequest
 ↓
Networking
 ↓
Server
 ↓
Build & Destruction
 ↓
Validation
 ↓
Block System
 ↓
Loot
 ↓
Event Bus
 ↓
Replication
```

---

# 35. Exemplo: quebrar bloco

```text
CLIENT
   │
   │ BreakBlockRequest
   ▼
SERVER
   │
   ├── Permission
   ├── Distance
   ├── Tool
   ├── World State
   └── Validation
   │
   ▼
BUILD & DESTRUCTION
   │
   ▼
BLOCK SYSTEM
   │
   ▼
WORLD STATE CHANGED
   │
   ▼
EVENT BUS
   │
   ├── Lighting
   ├── Fluid
   ├── Physics
   ├── Loot
   ├── Audio
   └── Networking
   │
   ▼
CLIENT A
CLIENT B
CLIENT C
```

---

# 36. Inventory Networking

Inventário é um ponto extremamente sensível.

Nunca confiar em:

```text
"meu inventário agora contém 999 diamantes"
```

O cliente envia uma operação:

```text
MoveItemRequest
SplitStackRequest
MergeStackRequest
CraftRequest
DropItemRequest
```

Servidor valida.

Depois:

```text
Item System
 ↓
Inventory
 ↓
Transaction
 ↓
Persistence
 ↓
Replication
```

---

# 37. Transaction ID

Operações importantes devem possuir:

```text
TransactionID
```

Exemplo:

```text
TX-839201
```

Se o cliente reenviar devido a timeout:

```text
TX-839201
```

o servidor pode detectar que aquela operação já foi aplicada.

Isso evita duplicações.

---

# 38. Dimension Transfer

Trocar de dimensão não deve ser:

```text
Entity.teleport(...)
```

isoladamente.

Deve existir uma operação de rede completa:

```text
REQUEST TRANSFER
       ↓
SERVER VALIDATION
       ↓
FREEZE/TRANSFER STATE
       ↓
REMOVE OLD INTEREST
       ↓
LOAD DESTINATION
       ↓
SPAWN/RESTORE
       ↓
NEW INTEREST
       ↓
READY
```

Durante isso não pode existir:

```text
half transferred entity
```

---

# 39. Network Security

Networking deve assumir que:

```text
CLIENT = NÃO CONFIÁVEL
```

Servidor precisa validar:

```text
movement
block interaction
inventory
crafting
combat
vehicle
energy
fluid
commands
trades
permissions
```

Não basta esconder alguma função do cliente.

---

# 40. Rate Limiting

Cada cliente possui limites.

Exemplo conceitual:

```text
Requests/sec
Packets/sec
Messages/sec
Chunk Requests/sec
Chat/sec
Inventory Operations/sec
Interaction/sec
```

Excesso:

```text
THROTTLE
↓
WARN
↓
TEMPORARY BLOCK
↓
DISCONNECT
```

conforme a política.

---

# 41. Malformed Packets

Nunca deixar um packet inválido derrubar o servidor.

Pipeline:

```text
RECEIVE
 ↓
FRAME VALIDATION
 ↓
SIZE LIMIT
 ↓
PROTOCOL VALIDATION
 ↓
DESERIALIZATION
 ↓
SCHEMA VALIDATION
 ↓
AUTHORITY VALIDATION
 ↓
QUEUE
```

Falha deve resultar em:

```text
reject
log
metric
```

e não:

```text
server crash
```

---

# 42. Compression

Tipos de dados diferentes podem possuir compressão diferente.

```text
Chunk Data
→ high compression

Snapshots
→ balanced

Movement
→ low overhead

Voice
→ dedicated codec

Inventory
→ compact serialization
```

Não devemos compressar tudo cegamente.

---

# 43. Bandwidth Scheduler

O servidor pode ter orçamento:

```text
Bandwidth Budget
```

Exemplo conceitual:

```text
HIGH
├── Critical Gameplay
├── Dimension Transfer
├── Inventory
└── Block Operations

MEDIUM
├── Entity State
└── Chunk Updates

LOW
├── Cosmetic State
└── Optional Effects
```

Se houver congestionamento:

```text
cosmetic updates
```

são sacrificados antes de:

```text
inventory transaction
```

---

# 44. Backpressure

Se o cliente não consegue acompanhar:

```text
QUEUE
 ↓
BACKPRESSURE
 ↓
COALESCE
 ↓
DROP NON-CRITICAL
 ↓
RESYNC
```

Não podemos deixar:

```text
memory
   ↑
   ↑
   ↑
   ∞
```

---

# 45. Event vs State Replication

Essa distinção é essencial.

### Event

```text
ExplosionOccurred
EntityDied
BlockBroken
MachineCompleted
```

Representa:

> algo aconteceu.

### State

```text
EntityPosition
InventoryContents
MachineTemperature
PlayerHealth
```

Representa:

> como algo está agora.

Não tratar os dois da mesma forma.

---

# 46. Event Replication

Um evento pode ser:

```text
AUTHORITATIVE
RELEVANT
COSMETIC
LOCAL
```

Exemplo:

```text
LightningStrike
```

Pode ser enviado para jogadores interessados.

---

# 47. State Synchronization

Estado importante pode ser confirmado através de snapshots.

Isso resolve casos como:

```text
packet perdido
event perdido
reconnect
desync
```

---

# 48. Resynchronization

Cliente pode pedir:

```text
RequestResync
```

Servidor pode responder:

```text
PlayerStateSnapshot
InventorySnapshot
EntitySnapshot
ChunkSnapshot
```

Não precisamos reiniciar a conexão inteira.

---

# 49. Networking + Event Bus

Event Bus não deve ser usado para absolutamente tudo.

Exemplo correto:

```text
BlockBrokenEvent
        ↓
Event Bus
        ↓
Networking subscriber
```

Networking detecta:

> “esse evento precisa ser replicado”.

Mas consulta direta pode ser melhor para:

```text
current player state
snapshot
interest query
```

---

# 50. Networking + Persistence

Networking não salva mundo.

Persistence salva.

Fluxo:

```text
NETWORK
 ↓
SERVER
 ↓
SIMULATION
 ↓
STATE
 ↓
PERSISTENCE
```

No reconnect:

```text
Persistence
 ↓
Server State
 ↓
Replication
 ↓
Client
```

---

# 51. Networking + Registry

Antes de sincronizar conteúdo:

```text
CLIENT
     ↓
Registry Fingerprint
     ↓
SERVER
```

Precisamos verificar compatibilidade.

```text
Game Version
World Version
Registry Version
Modpack Version
Protocol Version
```

São coisas diferentes.

---

# 52. Registry Sync

Cliente precisa conhecer os IDs necessários.

Exemplo:

```text
nexora:iron
mod_example:super_machine
```

Mas não podemos usar diretamente:

```text
string "mod_example:super_machine"
```

em toda atualização.

Durante a sessão:

```text
Public ID
   ↓
Network ID
```

Exemplo:

```text
Network ID 1827
```

Muito mais eficiente.

---

# 53. Mod Networking

Mods devem poder registrar:

```text
NetworkMessage
NetworkChannel
Serializer
Packet Handler
Replication Policy
```

Exemplo:

```text
my_mod:laser_state
```

O sistema deve controlar:

```text
namespace
permissions
size
rate
serialization
version
```

Um mod não pode simplesmente injetar tráfego ilimitado.

---

# 54. Versionamento

Teremos múltiplas versões.

```text
Game Version
Protocol Version
Network Schema Version
World Version
Registry Version
Mod Version
```

Não misturar.

Exemplo:

```text
Protocol 4
Game 0.8
World 17
Registry 31
```

---

# 55. Compatibility

Handshake deve descobrir:

```text
Compatible?
```

Possíveis resultados:

```text
ACCEPT
ACCEPT_WITH_ADAPTER
REJECT_PROTOCOL
REJECT_VERSION
REJECT_MODPACK
REJECT_REGISTRY
REJECT_AUTH
```

---

# 56. Handshake completo

```text
CLIENT
 ↓
Hello
 ↓
ServerHello
 ↓
Protocol Negotiation
 ↓
Authentication
 ↓
Permission Setup
 ↓
Registry Fingerprint
 ↓
Modpack Fingerprint
 ↓
World Compatibility
 ↓
Session Creation
 ↓
Player Assignment
 ↓
Initial Snapshot
 ↓
Chunk Streaming
 ↓
Ready
```

---

# 57. Chat

Chat deve ser semântico.

Networking transporta:

```text
ChatMessage
```

e não controla:

```text
UI
```

Depois:

```text
Chat Message
 ↓
Chat System
 ↓
UI
```

---

# 58. Voice

Voice pode possuir canal próprio:

```text
VOICE
```

Com requisitos diferentes:

```text
low latency
loss tolerance
compression
jitter handling
```

Networking apenas transporta.

Audio System decide como reproduzir.

---

# 59. Animation Networking

Não enviar:

```text
cada osso
cada frame
```

Enviar:

```text
Animation State
Animation ID
Parameter
Trigger
Playback State
```

Exemplo:

```text
movement = RUN
speed = 0.82
```

O Animation System local produz o pose.

---

# 60. Audio Networking

Nunca transportar:

```text
PCM do mundo inteiro
```

Transportar eventos semânticos:

```text
ExplosionOccurred
MachineStarted
Footstep
Gunshot
WeatherChanged
```

Audio System decide o que tocar.

---

# 61. Physics Networking

Servidor:

```text
AUTHORITY
```

Cliente:

```text
PREDICTION / PRESENTATION
```

O cliente não pode decidir sozinho:

```text
collision result
damage
vehicle crash
world destruction
```

---

# 62. Vehicle Networking

Veículos precisam de previsão mais sofisticada.

```text
Input
 ↓
Vehicle Prediction
 ↓
Server Simulation
 ↓
Authoritative State
 ↓
Reconciliation
```

Para veículos de alta velocidade:

```text
Prefetch
Interest Expansion
```

também será necessário.

---

# 63. Entity Interest usando Structure

Imagine:

```text
uma cidade
```

O jogador entra.

Networking pode detectar:

```text
Structure
 ↓
City
 ↓
Relevant NPC population
 ↓
Nearby infrastructure
```

Em vez de tratar tudo como entidades individuais imediatamente.

Isso combina muito bem com:

```text
Structure System
Civilization System
Entity System
```

---

# 64. Networking + Civilization

Uma cidade distante pode ser:

```text
ABSTRACT
```

Então:

```text
Population = 18,430
FoodProduction = 7,200
Trade = ACTIVE
```

Quando o jogador se aproxima:

```text
ABSTRACT
   ↓
REGIONAL
   ↓
FULL
```

As entidades físicas são então reidratadas.

Isso é essencial para o NEXORA escalar.

---

# 65. Streaming adaptativo

Networking pode antecipar necessidades com base em:

```text
player velocity
camera direction
vehicle route
rail path
teleport destination
dimension transfer
```

Exemplo:

```text
Player → trem → 180 km/h

Interest Manager
       ↓
Prefetch 5 regiões à frente
```

---

# 66. Network Scheduler

O scheduler decide:

```text
quando enviar
o que enviar
quanto enviar
qual prioridade
```

Por exemplo:

```text
Tick
 ↓
Critical
 ↓
Input
 ↓
Gameplay State
 ↓
Chunks
 ↓
Entities
 ↓
Cosmetics
```

---

# 67. Entity Update Frequency

Nem tudo precisa de:

```text
60 Hz
```

Pode haver:

```text
PLAYER
60 Hz

Nearby Entity
20 Hz

Distant Entity
5 Hz

Regional Civilization
1 Hz

Abstract Civilization
event-driven
```

---

# 68. Packet Aggregation

Em vez de:

```text
1 entity → 1 packet
```

podemos ter:

```text
EntityStateBatch
```

com dezenas/centenas de atualizações.

Isso reduz overhead.

---

# 69. Batch Operations

Mesmo princípio para:

```text
Block changes
Entity spawns
Entity despawns
Chunk updates
Inventory deltas
Particles
Events
```

---

# 70. Reliable State Recovery

Um cliente desconectado:

```text
disconnect
```

não deve obrigatoriamente provocar:

```text
world rollback
```

Ao reconectar:

```text
Session
 ↓
Persistence
 ↓
Server state
 ↓
Fresh snapshot
```

---

# 71. Replay

Networking deve poder gravar:

```text
input
messages
snapshots
commands
timestamps
```

para reprodução.

Não necessariamente como gravação de vídeo.

Algo como:

```text
Network Replay
```

permitindo:

```text
reproduzir sessão
investigar bugs
debugar desync
analisar latency
testar exploits
```

---

# 72. Network Diagnostics

Comandos:

```text
nexora network status
nexora network connections
nexora network connection <id>
nexora network packets
nexora network bandwidth
nexora network latency
nexora network replication
nexora network interest
nexora network resync
nexora network dump
```

---

# 73. Métricas

Por cliente:

```text
RTT
Jitter
Packet Loss
Upload
Download
Packets/sec
Bytes/sec
Queue Size
Replication Count
Chunk Streaming
Resync Count
Prediction Error
```

Por servidor:

```text
Total Bandwidth
Connections
Packets/tick
Replication CPU
Serialization CPU
Compression CPU
Network Queue
```

---

# 74. Debug de Desync

Um dos sistemas mais importantes.

Precisamos comparar:

```text
CLIENT STATE
vs
SERVER STATE
```

Exemplo:

```text
Player Entity 837
Client Position
Server Position
Inventory Hash
Component Hash
Chunk Hash
```

Então:

```text
MATCH
MISMATCH
```

---

# 75. State Hash

Podemos criar hashes de estado.

```text
Entity State
 ↓
Hash
```

Periodicamente:

```text
ClientHash
ServerHash
```

Se:

```text
ClientHash != ServerHash
```

gera investigação/resync.

---

# 76. Threading

Networking pode possuir workers:

```text
Network I/O Thread
       ↓
Decode Thread
       ↓
Validation
       ↓
Simulation Queue
```

Na volta:

```text
Simulation Snapshot
       ↓
Encode
       ↓
Send Queue
       ↓
Network I/O
```

Mas o Networking nunca deve alterar arbitrariamente o estado da simulação em paralelo.

---

# 77. Snapshot Boundaries

Precisamos de fronteiras claras:

```text
NETWORK THREAD
       │
       ▼
DECODED DATA
       │
       ▼
SIMULATION COMMAND QUEUE
       │
       ▼
SERVER TICK
```

Isso evita:

```text
race conditions
```

---

# 78. Security Architecture

Separar:

```text
Transport Security
Authentication
Authorization
Simulation Validation
```

Networking pode oferecer o canal seguro.

Mas:

```text
“esse jogador pode fazer isso?”
```

continua pertencendo à autoridade do servidor.

---

# 79. Interfaces públicas

```text
INetworkSystem

INetworkTransport

INetworkProtocol

INetworkConnection

INetworkSession

INetworkMessage

INetworkSerializer

INetworkChannel

INetworkScheduler

IReplicationManager

IInterestManager

INetworkAuthority

INetworkClock

INetworkMetrics

INetworkReplay

INetworkSecurity
```

---

# 80. Organização de código

Sugestão:

```text
src/networking/

├── core/
│   ├── network-system
│   ├── network-context
│   └── network-config
│
├── connection/
│   ├── connection
│   ├── connection-manager
│   └── session
│
├── transport/
│   ├── transport
│   ├── udp
│   ├── quic
│   └── loopback
│
├── protocol/
│   ├── protocol
│   ├── framing
│   ├── sequence
│   ├── acknowledgement
│   └── reliability
│
├── message/
│   ├── message
│   ├── registry
│   └── handlers
│
├── serialization/
│   ├── serializer
│   ├── compression
│   └── schemas
│
├── handshake/
│   ├── handshake
│   ├── compatibility
│   └── synchronization
│
├── replication/
│   ├── replication-manager
│   ├── snapshot
│   ├── delta
│   ├── baseline
│   └── component-replication
│
├── interest/
│   ├── interest-manager
│   ├── spatial-interest
│   ├── chunk-interest
│   └── entity-interest
│
├── input/
│   ├── input-snapshot
│   ├── prediction
│   └── reconciliation
│
├── authority/
│   ├── authority
│   └── ownership
│
├── streaming/
│   ├── chunk-streaming
│   └── prefetch
│
├── mod/
│   ├── mod-network
│   └── channel-registry
│
├── voice/
│
├── replay/
│
├── security/
│
├── metrics/
│
└── debug/
```

---

# 81. Dependências

Networking fica relativamente tarde na fundação.

```text
Core
 ↓
Registry
 ↓
Event Bus
 ↓
Entity
 ↓
Block
 ↓
Item
 ↓
Persistence
 ↓
Networking
```

Mas a relação final é:

```text
Core
├── Registry
├── Event Bus
├── Persistence
│
├── Entity
├── Block
├── Item
├── Physics
├── World
│
└── Networking
        ↓
      Server
        ↓
   Simulation
```

---

# 82. Implementação por fases

## NET-0 — Architecture

Criar:

```text
interfaces
core types
connection model
message model
```

---

## NET-1 — Loopback

Primeiro:

```text
Client
↕
Loopback Transport
↕
Server
```

Sem internet.

Isso permite testar toda a arquitetura.

---

## NET-2 — Connection

Implementar:

```text
connect
disconnect
state
session
```

---

## NET-3 — Protocol

Implementar:

```text
framing
sequence
packet
message
```

---

## NET-4 — Reliable Channel

Implementar:

```text
ACK
retransmission
ordering
timeout
```

---

## NET-5 — Unreliable Channel

Implementar:

```text
sequence
drop
latest-state logic
```

---

## NET-6 — Handshake

```text
hello
protocol
registry
modpack
world
session
```

---

## NET-7 — Player Synchronization

Primeiro vertical slice real:

```text
Client connects
 ↓
Player enters world
 ↓
Server creates Player Entity
 ↓
Server replicates player
 ↓
Client renders player
```

---

## NET-8 — Input

```text
Client Input
 ↓
Server
 ↓
Simulation
 ↓
Authoritative State
```

---

## NET-9 — Prediction

Adicionar:

```text
client prediction
reconciliation
interpolation
```

---

## NET-10 — Chunk Streaming

```text
ChunkRequest
ChunkData
Compression
Interest
Prefetch
```

---

## NET-11 — Entity Replication

```text
spawn
despawn
snapshot
delta
components
LOD
```

---

## NET-12 — Block Operations

Primeiro grande teste do mundo:

```text
Client requests block break
Server validates
Build executes
World changes
Event Bus
Replication
```

---

## NET-13 — Inventory

```text
transaction
authority
replication
duplication prevention
```

---

## NET-14 — Dimension Transfer

```text
Overworld
 ↓
Dimension
```

com carregamento/interest reset.

---

## NET-15 — Interest Management avançado

```text
Chunk
Entity
Structure
Dimension
Permissions
LOD
```

---

## NET-16 — Server Scaling

Testar:

```text
10 players
50
100
250
500+
```

---

## NET-17 — Mod Networking

Implementar:

```text
custom messages
custom channels
custom replication
permissions
quotas
```

---

## NET-18 — Replay + Diagnostics

```text
capture
replay
packet trace
desync detector
bandwidth profiler
```

---

# 83. Vertical Slice principal

O primeiro verdadeiro teste deve ser:

```text
SERVER
+
CLIENT A
+
CLIENT B
```

### Cliente A

```text
entra
↓
recebe mundo
↓
anda
↓
quebra bloco
```

### Servidor

```text
recebe request
↓
valida
↓
Build & Destruction
↓
Block System
↓
Event Bus
```

### Cliente B

```text
recebe BlockChange
↓
atualiza mundo
```

Resultado:

```text
A vê bloco quebrado
B vê bloco quebrado
Servidor possui estado correto
Persistence salva estado
Servidor reinicia
A reconecta
B reconecta
Bloco continua quebrado
```

Esse é o **Golden Test do Networking**.

---

# 84. Stress Tests

Precisamos testar pelo menos:

```text
100 players
1.000 players
10.000 entities
100.000 entities
1.000.000 abstract entities
```

Além de:

```text
0% loss
1% loss
5% loss
10% loss
high latency
high jitter
packet reorder
duplicate packets
disconnect
reconnect
server restart
chunk flood
message flood
malformed packet
mod mismatch
registry mismatch
dimension transfer
```

---

# 85. Teste de congestionamento

Simular:

```text
muitos players
+
cidade gigante
+
milhares de entidades
+
chunks sendo carregados
+
máquinas
+
veículos
```

O sistema deve priorizar:

```text
gameplay
>
state
>
streaming
>
cosmetic
```

---

# 86. Regra para 2.000+ mobs

Networking nunca deve assumir:

```text
2.000 mobs
=
2.000 updates completos por tick
```

Usaremos:

```text
FULL
REGIONAL
ABSTRACT
```

e:

```text
event-driven updates
adaptive frequency
interest management
batching
```

---

# 87. Regra para mundo gigantesco

Não existe:

```text
"sincronizar o mundo"
```

Existe:

```text
sincronizar uma visão relevante do mundo
```

O cliente recebe:

```text
o que pode observar
+
o que precisa simular/apresentar
+
o que possui autorização para conhecer
```

---

# 88. Limites do Networking

### Networking NÃO deve possuir

```text
Combat Logic
AI Logic
World Generation
Physics Rules
Inventory Rules
Crafting Rules
Economy Logic
Civilization Logic
Loot Logic
Block Logic
Item Logic
```

### Networking DEVE possuir

```text
Transport
Serialization
Sessions
Connections
Packets
Channels
Replication
Interest
Synchronization
Prediction
Reconciliation
Bandwidth
Network Security
Metrics
```

---

# 89. Mapa de integração

```text
                    ┌──────────────┐
                    │    CLIENT    │
                    └──────┬───────┘
                           │
                     NETWORKING
                           │
                           ▼
                    ┌──────────────┐
                    │    SERVER    │
                    └──────┬───────┘
                           │
                       SIMULATION
                           │
       ┌──────────┬────────┼────────┬──────────┐
       ▼          ▼        ▼        ▼          ▼
     ENTITY     BLOCK     ITEM     PHYSICS    WORLD
       │          │        │        │          │
       └──────────┴────────┴────────┴──────────┘
                           │
                       EVENT BUS
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
       AUDIO           ANIMATION        PERSISTENCE
          │                │                │
          └────────────────┼────────────────┘
                           ▼
                      NETWORKING
                           │
                           ▼
                        CLIENTS
```

---

# 90. Arquitetura final

O Networking do NEXORA deve terminar conceitualmente assim:

```text
                 NEXORA NETWORKING
                        │
       ┌────────────────┼────────────────┐
       │                │                │
       ▼                ▼                ▼
   TRANSPORT        PROTOCOL        SECURITY
       │                │                │
       └────────────────┼────────────────┘
                        │
                        ▼
                    SESSION
                        │
                        ▼
                  SYNCHRONIZATION
                        │
             ┌──────────┴──────────┐
             ▼                     ▼
          INPUT                 REPLICATION
             │                     │
             ▼                     ▼
       PREDICTION             INTEREST
             │                     │
             ▼                     ▼
     RECONCILIATION             LOD
             │                     │
             └──────────┬──────────┘
                        ▼
                     SERVER
                        │
                    SIMULATION
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
       EVENT BUS      PERSISTENCE    WORLD
          │
          ▼
      REPLICATION
          │
          ▼
       CLIENTS
```

E a regra mais importante fica:

```text
NETWORKING
"como os dados chegam?"

SERVER
"quem possui autoridade?"

SIMULATION
"o que aconteceu?"

PERSISTENCE
"como o estado sobrevive?"

EVENT BUS
"quem precisa saber?"

REGISTRY
"o que esse ID significa?"
```

Esse desenho encaixa o **Networking como infraestrutura da camada multiplayer**, sem transformar Networking em um “God System”. Ele também deixa preparado o caminho para os próximos grandes blocos do NEXORA: **Server, Mod Runtime, Scripting e Command System**.
