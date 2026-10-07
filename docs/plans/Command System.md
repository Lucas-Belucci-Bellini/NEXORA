# NEXORA — COMMAND SYSTEM

> **Princípio central:**
> **Commands representam intenção; sistemas especializados validam e executam; eventos comunicam o resultado.**

O fluxo fundamental do NEXORA passa a ser:

```text
CLIENT / SCRIPT / ADMIN / AI / NPC
                 │
                 ▼
             COMMAND
                 │
                 ▼
            COMMAND BUS
                 │
                 ▼
           SERVER AUTHORITY
                 │
                 ▼
             VALIDATION
                 │
                 ▼
        SPECIALIZED SYSTEM
                 │
                 ▼
            STATE CHANGE
                 │
                 ▼
             EVENT BUS
                 │
       ┌─────────┴─────────┐
       ▼                   ▼
 PERSISTENCE           NETWORKING
```

A função do Command System é criar uma **fronteira universal entre “quero fazer alguma coisa” e “o mundo realmente mudou”**.

---

# 1. O que é o Command System?

Um Command representa uma intenção operacional.

Exemplos:

```text
BreakBlockCommand
PlaceBlockCommand
MoveCommand
AttackCommand
CraftCommand
TransferItemCommand
UseItemCommand
EnterVehicleCommand
TravelDimensionCommand
TradeCommand
BuildStructureCommand
ExecuteAdminCommand
```

O Command **não é o resultado**.

Exemplo:

```text
BreakBlockCommand
```

significa:

> “Quero quebrar este bloco.”

Não significa:

> “O bloco foi quebrado.”

O resultado somente existe depois da execução autoritativa.

---

# 2. Command ≠ Event

Essa separação precisa ser absoluta.

```text
COMMAND
→ intenção / solicitação

EVENT
→ fato ocorrido
```

Exemplo:

```text
BreakBlockCommand
        ↓
Build & Destruction
        ↓
BlockBrokenEvent
```

Nunca:

```text
BreakBlockEvent
→ tenta executar o rompimento
```

---

# 3. Command ≠ Query

Teremos três conceitos:

```text
COMMAND
→ mudar / solicitar

QUERY
→ consultar

EVENT
→ informar
```

Exemplo:

```text
getPlayerInventory()
→ Query

MoveItemCommand
→ Command

ItemTransferredEvent
→ Event
```

---

# 4. Filosofia

A arquitetura do NEXORA fica:

```text
Commands solicitam.
Systems executam.
Events informam.
Queries consultam.
Server autoriza.
Persistence preserva.
Networking transporta.
```

Isso é uma das regras mais importantes para evitar acoplamento.

---

# 5. Arquitetura

```text
                         COMMAND SYSTEM
                                │
       ┌────────────────────────┼────────────────────────┐
       ▼                        ▼                        ▼
    DEFINITION                ROUTING                  QUEUE
       │                        │                        │
       ▼                        ▼                        ▼
   VALIDATION              DISPATCHER                SCHEDULER
       │                        │                        │
       └────────────────────────┼────────────────────────┘
                                ▼
                           EXECUTION
                                │
                                ▼
                         RESULT / FAILURE
                                │
                                ▼
                            EVENT BUS
```

---

# 6. Tipos de Command

Precisamos classificar comandos.

```text
Gameplay
World
Entity
Inventory
Crafting
Machine
Vehicle
Economy
Civilization
Quest
Research
Dimension
Admin
Mod
System
```

---

# 7. Command Definition

Cada comando deve possuir uma definição.

```text
CommandDefinition
├── CommandID
├── Version
├── Schema
├── SourcePolicy
├── PermissionPolicy
├── AuthorityPolicy
├── ValidationPolicy
├── ExecutionPolicy
├── TransactionPolicy
└── ReplicationPolicy
```

Exemplo conceitual:

```yaml
id: nexora:break_block
version: 1

source:
  player: true
  script: true
  npc: true
  admin: true

authority:
  server: required
```

---

# 8. Command Instance

A definição é o “tipo”.

A instância é a operação real.

```text
BreakBlockCommand
```

vira:

```text
CommandInstance
├── CommandID
├── CommandInstanceID
├── Actor
├── Timestamp
├── Target
├── Parameters
├── CorrelationID
├── CausationID
└── Context
```

---

# 9. Command ID

Precisamos de:

```text
CommandID
```

no formato:

```text
namespace:id
```

Exemplo:

```text
nexora:break_block
example:activate_machine
```

Assim mods podem registrar comandos sem entrar no Core.

---

# 10. Command Instance ID

Cada execução recebe um ID único:

```text
CommandInstanceID
```

Isso permite:

```text
deduplication
tracing
replay
audit
idempotency
```

---

# 11. Actor

Todo comando precisa saber:

> quem solicitou?

Pode ser:

```text
Player
NPC
AI
Script
Machine
WorldEvent
Admin
ServerSystem
Mod
```

Exemplo:

```text
Actor
├── ActorType
├── AccountID
├── PlayerID
├── EntityID
└── PermissionContext
```

---

# 12. Source

Não confundir Actor e Source.

O Actor pode ser:

```text
Player
```

enquanto a origem técnica seja:

```text
Network
```

Ou:

```text
Actor = Script
Source = Mod Runtime
```

Isso ajuda muito no diagnóstico.

---

# 13. Command Context

O contexto deve conter:

```text
CommandContext
├── Actor
├── Source
├── World
├── Dimension
├── SimulationTick
├── Permissions
├── CorrelationID
├── CausationID
└── CancellationToken
```

---

# 14. Correlation ID

Imagine:

```text
Player clicks button
```

Isso gera:

```text
UI Action
→ Command
→ Inventory
→ Event
→ Network
→ Persistence
```

Todos podem compartilhar:

```text
CorrelationID
```

Assim podemos rastrear uma operação inteira.

---

# 15. Causation ID

Um comando pode nascer de outro evento.

Exemplo:

```text
ClimateChangedEvent
        ↓
WorldEventCommand
        ↓
StormStartedEvent
```

`CausationID` permite saber:

> “o que causou esta operação?”

---

# 16. Command Lifecycle

```text
CREATED
   ↓
RECEIVED
   ↓
STRUCTURALLY_VALIDATED
   ↓
AUTHORIZED
   ↓
QUEUED
   ↓
VALIDATED
   ↓
EXECUTING
   ↓
COMMITTED
   ↓
COMPLETED
```

Falhas:

```text
REJECTED
INVALID
DENIED
FAILED
CANCELLED
ROLLED_BACK
EXPIRED
```

---

# 17. Pipeline completo

```text
RECEIVE
  ↓
DECODE
  ↓
SCHEMA VALIDATION
  ↓
IDENTITY
  ↓
AUTHORIZATION
  ↓
RATE LIMIT
  ↓
QUEUE
  ↓
WORLD VALIDATION
  ↓
RESOURCE VALIDATION
  ↓
EXECUTION
  ↓
COMMIT
  ↓
EVENTS
  ↓
REPLICATION
  ↓
PERSISTENCE
```

---

# 18. Validation em camadas

Não queremos uma única função:

```text
validateCommand()
```

gigantesca.

Dividir:

```text
Structural Validation
Identity Validation
Permission Validation
Context Validation
Target Validation
State Validation
Resource Validation
System Validation
```

---

# 19. Structural Validation

Exemplo:

```text
Position = inválida
Quantidade = negativa
ID = inválido
campo obrigatório ausente
```

Rejeita antes de tocar no mundo.

---

# 20. Identity Validation

Verificar:

```text
Session
Player
Entity
Connection
Actor
```

Exemplo:

```text
Connection 93
claiming Player 17
```

O servidor verifica se aquilo é verdadeiro.

---

# 21. Authorization

Pergunta:

> este actor pode executar esse comando?

Exemplo:

```text
Player
→ BreakBlock

Admin
→ GiveItem
```

---

# 22. Context Validation

O comando pode depender de:

```text
world
dimension
position
time
game mode
permissions
quest state
```

---

# 23. Target Validation

Exemplo:

```text
BreakBlockCommand
```

valida:

```text
chunk loaded?
target still exists?
distance valid?
dimension correct?
```

---

# 24. State Validation

Exemplo:

```text
CraftCommand
```

precisa verificar:

```text
recipe exists?
station exists?
ingredients available?
player allowed?
machine state valid?
```

---

# 25. Resource Validation

Por exemplo:

```text
TradeCommand
```

precisa validar:

```text
currency
items
inventory capacity
transaction state
```

---

# 26. System Validation

No final:

```text
Inventory
Crafting
Combat
Build
Machine
```

podem fazer validações específicas.

---

# 27. Command Handler

Cada comando deve apontar para um executor:

```text
ICommandHandler<TCommand>
```

Exemplo:

```text
ICommandHandler<BreakBlockCommand>
ICommandHandler<CraftCommand>
ICommandHandler<TradeCommand>
```

O handler não deve conter todas as regras.

Ele coordena a chamada do sistema correto.

---

# 28. Handler vs System

Muito importante:

```text
Command Handler
→ adapta comando para o sistema

Specialized System
→ possui a regra real
```

Exemplo:

```text
BreakBlockHandler
       ↓
BuildAndDestruction.break(...)
```

---

# 29. Dispatcher

Precisamos de:

```text
ICommandDispatcher
```

que faz:

```text
CommandID
   ↓
Handler Lookup
   ↓
Handler
```

Lookup deve usar handles/runtime IDs para o hot path.

---

# 30. Command Registry

Integrar com Registry System:

```text
CommandRegistry
```

como qualquer outra definição.

Isso permite:

```text
nexora:break_block
example:activate_machine
```

serem registrados sem alterar Core.

---

# 31. Queue

Commands recebidos não precisam executar imediatamente.

```text
CommandQueue
```

pode organizar:

```text
Priority
Tick
Region
Entity
Source
Deadline
```

---

# 32. Prioridades

Exemplo:

```text
CRITICAL
HIGH
NORMAL
LOW
BACKGROUND
```

Mas prioridade nunca deve permitir violar regras de segurança.

---

# 33. Scheduling

Um comando pode ser:

```text
IMMEDIATE
NEXT_TICK
SCHEDULED
DEFERRED
REGION_BOUND
ENTITY_BOUND
```

---

# 34. Expiration

Alguns comandos ficam inválidos com o tempo.

Exemplo:

```text
AttackCommand
```

recebido vários segundos depois.

Precisamos:

```text
expiresAt
maxLatency
```

quando apropriado.

---

# 35. Idempotency

Operações críticas devem possuir:

```text
IdempotencyKey
```

ou usar:

```text
CommandInstanceID
TransactionID
```

Exemplo:

```text
CraftCommand
TX-98213
```

Se chegar novamente:

```text
already applied
```

não duplica o resultado.

---

# 36. Transaction Support

Alguns Commands são transacionais.

```text
TransferItem
Trade
Craft
Purchase
MachineAssembly
```

Pipeline:

```text
BEGIN
 ↓
VALIDATE
 ↓
RESERVE
 ↓
EXECUTE
 ↓
COMMIT
```

Em falha:

```text
ROLLBACK
```

quando o domínio suportar isso.

---

# 37. Atomic Commands

Exemplo:

```text
TradeCommand
```

não pode resultar em:

```text
currency removed
+
item never delivered
```

A operação precisa de uma fronteira transacional apropriada.

---

# 38. Composite Commands

Algumas operações são compostas.

```text
BuildHouseCommand
```

pode envolver:

```text
Reserve Items
→ Place Structure
→ Update Inventory
→ Register Structure
```

Pode existir:

```text
CompositeCommand
```

Mas não transformar tudo em comando composto.

---

# 39. Command Chains

Algumas operações geram novas intenções:

```text
PlayerCommand
    ↓
MachineCommand
    ↓
EnergyCommand
```

Isso precisa possuir:

```text
CausationID
```

para rastreio.

---

# 40. Nested Commands

Cuidado para não permitir:

```text
A → B → C → D → A
```

Precisamos de:

```text
depth limit
cycle detection
execution budget
```

---

# 41. Command Cancellation

Suportar:

```text
cancel(commandId)
```

quando a operação ainda não começou ou permite cancelamento.

Estados:

```text
QUEUED
→ CANCELLED
```

Mas:

```text
COMMITTED
```

não deve simplesmente ser “desfeito” sem operação explícita.

---

# 42. Permission System

Command System conecta-se ao Permission System.

```text
Command
 ↓
Permission Check
```

Mas as regras especializadas continuam podendo negar.

Exemplo:

```text
player tem build permission
```

mas:

```text
claimed territory
```

pode bloquear.

---

# 43. Server Authority

Para comandos vindos de cliente:

```text
Client
 ↓
Networking
 ↓
Command System
 ↓
Server
```

O cliente não executa o comando final.

---

# 44. Script Commands

Scripts também podem criar:

```text
Command
```

usando a mesma API.

```text
Script
 ↓
Command API
 ↓
Command System
 ↓
Server
```

Não criar um segundo mecanismo de execução.

---

# 45. Admin Commands

Console:

```text
nexora give
nexora teleport
nexora weather
```

também devem passar pelo Command System.

```text
Console
 ↓
Command Parser
 ↓
Command
 ↓
Authority
 ↓
System
```

---

# 46. NPC Commands

NPCs também podem produzir Commands.

```text
NPC AI
 ↓
MoveCommand
TradeCommand
BuildCommand
TalkCommand
```

Isso é muito poderoso.

Player e NPC podem usar a mesma infraestrutura.

---

# 47. AI Commands

AI pode decidir:

```text
"quero atacar"
```

produzindo:

```text
AttackCommand
```

O Combat System decide o resultado.

---

# 48. Machine Commands

Máquinas podem emitir:

```text
StartProcessingCommand
TransferEnergyCommand
TransferFluidCommand
```

conforme o desenho dos sistemas.

---

# 49. World Event Commands

Um World Event pode gerar:

```text
SpawnEntityCommand
StartStormCommand
ChangeClimateCommand
TriggerStructureEventCommand
```

---

# 50. Command Result

Todo comando precisa retornar um resultado estruturado.

```text
CommandResult
├── Status
├── CommandID
├── CommandInstanceID
├── Reason
├── Data
├── Events
└── Metadata
```

---

# 51. Status

```text
SUCCESS
REJECTED
DENIED
INVALID
FAILED
CANCELLED
EXPIRED
```

---

# 52. Failure Reason

Não simplesmente:

```text
false
```

Usar algo como:

```text
PLAYER_TOO_FAR
TARGET_NOT_FOUND
NO_PERMISSION
MISSING_RESOURCE
INVALID_STATE
WORLD_NOT_LOADED
DEPENDENCY_MISSING
COOLDOWN_ACTIVE
```

Isso ajuda UI, logs, scripts e debug.

---

# 53. Error Codes

Os erros devem ser estáveis:

```text
CommandErrorCode
```

e versionados quando necessário.

---

# 54. User-Facing Message

O Command Result pode conter:

```text
localizedReasonKey
```

por exemplo:

```text
command.break_block.out_of_range
```

A UI localiza.

Command System não precisa conhecer idioma.

---

# 55. Event Generation

Após sucesso:

```text
Command
 ↓
System
 ↓
State Change
 ↓
Event
```

Exemplo:

```text
PlaceBlockCommand
 ↓
Build System
 ↓
Block placed
 ↓
BlockPlacedEvent
```

---

# 56. Event ≠ Result

`CommandResult` responde:

> a operação solicitada terminou como?

`Event` responde:

> o que aconteceu no mundo?

Eles podem ter informações diferentes.

---

# 57. Replication

Command System não deve decidir como os clientes recebem o resultado.

Depois do state change:

```text
Event Bus
 ↓
Networking
 ↓
Replication
```

---

# 58. Persistence

Após alteração crítica:

```text
Command
 ↓
System
 ↓
Committed State
 ↓
Persistence
```

O Command System não escreve save files.

---

# 59. Replay

Como cada comando possui:

```text
CommandID
Tick
Actor
Parameters
CorrelationID
```

podemos gravar comandos para replay.

```text
Checkpoint
+
Command Log
```

Isso é perfeito para debugging.

---

# 60. Deterministic Replay

No modo de teste:

```text
same world
+
same commands
+
same seed
+
same simulation version
```

deve produzir resultados equivalentes quando o sistema for determinístico.

---

# 61. Command Audit Log

Operações administrativas ou econômicas importantes podem gerar:

```text
AuditEntry
```

Exemplo:

```text
Admin
→ GiveItem
→ Player 83
→ 100 copper
```

Isso ajuda servidores públicos.

---

# 62. Rate Limiting

Command System também deve aplicar limites:

```text
commands/sec
commands/tick
per-player
per-mod
per-script
per-command-type
```

---

# 63. Command Spam

Exemplo:

```text
100.000 BreakBlockCommands
```

não pode simplesmente entupir a simulation queue.

Pode:

```text
coalesce
throttle
reject
prioritize
```

dependendo do tipo.

---

# 64. Command Batching

Para determinadas operações:

```text
BulkPlaceBlocksCommand
BulkBreakBlocksCommand
TransferStackBatchCommand
```

em vez de:

```text
10.000 commands
```

Mas o domínio deve possuir operações em lote seguras.

---

# 65. Spatial Commands

Comandos podem ser associados a:

```text
Dimension
Region
Chunk
Entity
```

Isso ajuda distribuir a carga.

---

# 66. Region Affinity

Exemplo:

```text
BreakBlockCommand
```

pertence à região:

```text
Region X
```

O scheduler pode colocá-lo no worker/queue adequado.

Isso será importante para futuro sharding.

---

# 67. Future Server Sharding

O design pode permitir:

```text
Command
 ↓
Region Authority
 ↓
Server Node
```

Hoje:

```text
1 server
```

No futuro:

```text
Server Cluster
```

sem redefinir o conceito de Command.

---

# 68. Cross-Region Commands

Exemplo:

```text
Trade
```

pode envolver:

```text
Player region
+
Market region
```

Essas operações precisarão de mecanismos distribuídos no futuro.

Não implementar isso no começo.

Mas os contratos devem reconhecer:

```text
execution scope
authority scope
```

---

# 69. Cross-Dimension Commands

```text
TravelDimensionCommand
```

precisa tratar:

```text
source dimension
destination dimension
player state
inventory
entity
interest
```

Pode usar uma transação de transferência.

---

# 70. Command Context Stack

Durante execução:

```text
Command A
 ↓
Command B
 ↓
Command C
```

podemos manter:

```text
causation chain
```

para diagnóstico.

---

# 71. Security Boundary

O Command System é uma das principais fronteiras de segurança.

Entrada externa:

```text
Network
Script
Mod
Console
```

passa por:

```text
validation
authorization
quota
authority
```

antes de afetar o mundo.

---

# 72. Untrusted Commands

Comando vindo do cliente:

```text
UNTRUSTED
```

Comando vindo de servidor:

```text
TRUSTED-ish
```

mas mesmo comandos internos devem respeitar invariantes.

Nunca assumir:

```text
internal = always valid
```

porque bugs internos também existem.

---

# 73. Command Capabilities

Um mod pode receber:

```text
CommandCapability
```

permitindo registrar tipos ou emitir certos comandos.

Por exemplo:

```text
mod
→ example:activate_machine
```

não necessariamente:

```text
admin:shutdown_server
```

---

# 74. Command Registry para Mods

Mods podem registrar:

```text
example:launch_probe
example:activate_reactor
example:research
```

usando:

```text
CommandRegistry
```

---

# 75. Command Schema

O schema deve suportar:

```text
Primitive
Enum
Vector
EntityID
BlockID
ItemID
Position
Rotation
List
Map
Optional
Nested Object
```

---

# 76. Serialization

Command precisa ser serializável para:

```text
Networking
Replay
Logging
Persistence Journal
Testing
```

Mas não significa que todo Command deva ser persistido.

---

# 77. Network Serialization

O cliente envia apenas:

```text
Command Payload
```

O servidor adiciona contexto confiável:

```text
Connection
Session
Actor
ServerTime
```

O cliente não pode escolher:

```text
"Actor = Admin"
```

---

# 78. Command Versioning

Exemplo:

```text
BreakBlock v1
BreakBlock v2
```

O protocol pode adaptar.

Importante para servidores em atualização.

---

# 79. Mod Command Versioning

Mods podem ter:

```text
example:machine_command v3
```

com schema próprio.

---

# 80. Command Translation

Compatibilidade:

```text
Old Command
 ↓
Adapter
 ↓
New Command
```

Mas isso deve ser explícito.

---

# 81. Command Preconditions

Um comando pode declarar:

```text
Preconditions
```

por exemplo:

```text
player.dimension == target.dimension
chunk.loaded == true
target.block == expectedBlock
```

Se a condição mudou:

```text
REJECTED
```

---

# 82. Compare-and-Commit

Isso é útil para evitar race conditions:

```text
Expected State
+
Requested Operation
```

Exemplo:

```text
Remove item
where slot = 4
and item = iron
and version = 12
```

Se version mudou:

```text
CONFLICT
```

---

# 83. Optimistic Concurrency

Alguns comandos podem usar:

```text
StateVersion
```

Exemplo:

```text
InventoryVersion = 42
```

Cliente envia:

```text
expectedVersion = 42
```

Servidor encontra:

```text
43
```

Resultado:

```text
CONFLICT
```

Isso ajuda muito no multiplayer.

---

# 84. Command Locking

Alguns domínios talvez precisem de locks curtos:

```text
Inventory
Machine
Structure
Trade
```

Mas preferir:

```text
version checks
transactions
atomic operations
```

quando possível.

---

# 85. Command Simulation

Podemos possuir:

```text
simulate(command)
```

que valida sem aplicar.

Ótimo para:

```text
UI preview
craft planner
building preview
admin tool
AI planning
```

Exemplo:

```text
"Posso construir esta estrutura?"
```

↓

```text
simulation
```

sem mutar o mundo.

---

# 86. Dry Run

CommandResult:

```text
WOULD_SUCCEED
```

pode indicar:

```text
cost
resources
changes
warnings
```

---

# 87. Command Preview

Muito importante para construção:

```text
BuildStructureCommand
```

pode primeiro:

```text
validate
simulate
preview
```

depois:

```text
execute
```

---

# 88. Undo

Não colocar “undo” diretamente no Command System como regra geral.

Alguns sistemas podem fornecer:

```text
inverse command
```

ou:

```text
transaction rollback
```

Build & Destruction pode ter histórico.

Isso continua sendo domínio especializado.

---

# 89. Command History

O Command System pode registrar:

```text
CommandRecord
```

mas não precisa armazenar permanentemente tudo.

Pode ser:

```text
ring buffer
journal
replay stream
audit log
```

dependendo do caso.

---

# 90. Debugging

Comandos:

```text
nexora command list
nexora command inspect
nexora command queue
nexora command history
nexora command pending
nexora command execute
nexora command simulate
nexora command cancel
nexora command trace
```

---

# 91. Command Trace

Exemplo:

```text
Command #82173
────────────────────────────

Source:
Network

Actor:
Player 42

Command:
nexora:break_block

Target:
Dimension 0
Chunk 182:77
Block 15,64,22

Validation:
✓ Identity
✓ Permission
✓ Distance
✓ Tool
✓ World State

Handler:
BuildAndDestruction

Result:
SUCCESS

Events:
BlockBroken
ItemDropped
LightingChanged

Correlation:
CORR-9382
```

Isso será ouro para debugging.

---

# 92. Metrics

Por comando:

```text
count
success
failure
denied
average latency
queue latency
execution time
timeouts
retries
```

Por origem:

```text
network
script
npc
admin
system
mod
```

---

# 93. Command Profiler

Exemplo:

```text
BreakBlock
├── validation    0.02ms
├── build         0.08ms
├── block         0.01ms
├── loot          0.03ms
├── lighting      0.06ms
└── events        0.01ms

total             0.21ms
```

---

# 94. Command Scheduler + LOD

Commands relacionados à simulação distante podem ser diferentes.

Por exemplo:

```text
AbstractCivilizationCommand
```

pode ser processado em escala regional/abstrata.

---

# 95. Civilização

Um NPC distante não precisa emitir:

```text
every tiny movement
```

Mas pode gerar:

```text
TradeCommand
MigrationCommand
BuildStructureCommand
DeclarePolicyCommand
```

em nível abstrato.

---

# 96. Economy

Comandos:

```text
BuyCommand
SellCommand
TradeCommand
TransferCurrencyCommand
CreateMarketOrderCommand
```

podem usar transações.

---

# 97. Quest

```text
AcceptQuestCommand
CompleteQuestCommand
AbandonQuestCommand
AdvanceQuestCommand
```

Mas Quest System continua dono das regras.

---

# 98. Research

```text
StartResearchCommand
SubmitExperimentCommand
CompleteResearchCommand
```

Research System executa.

---

# 99. Automation

```text
ConfigureMachineCommand
ConnectNodeCommand
SetAutomationRuleCommand
```

Machine/Automation System executa.

---

# 100. Player

```text
MoveCommand
InteractCommand
UseItemCommand
EquipCommand
UnequipCommand
```

Player System executa.

---

# 101. Item / Inventory

```text
MoveItemCommand
SplitStackCommand
MergeStackCommand
DropItemCommand
PickupItemCommand
TransferCommand
```

Inventory/Item executam.

---

# 102. Block / Build

```text
BreakBlockCommand
PlaceBlockCommand
ReplaceBlockCommand
BulkBuildCommand
TerraformCommand
```

Build & Destruction executa.

---

# 103. Entity

```text
SpawnEntityCommand
DespawnEntityCommand
TeleportEntityCommand
AttachEntityCommand
DetachEntityCommand
```

Entity System executa.

---

# 104. Dimension

```text
TravelDimensionCommand
CreateDimensionCommand
UnloadDimensionCommand
```

Dimension System executa.

---

# 105. Mod

```text
ReloadModCommand
EnableModCommand
DisableModCommand
```

Mod Runtime executa.

---

# 106. Command API

Interfaces:

```text
ICommand
ICommandDefinition
ICommandHandler<T>
ICommandDispatcher
ICommandRegistry
ICommandQueue
ICommandScheduler
ICommandValidator<T>
ICommandResult
ICommandContext
ICommandTransaction
ICommandSimulator
ICommandSerializer
ICommandMiddleware
ICommandMetrics
ICommandTracer
```

---

# 107. Middleware

Podemos ter pipeline:

```text
Command
 ↓
Logging
 ↓
Authentication
 ↓
Authorization
 ↓
RateLimit
 ↓
Validation
 ↓
Handler
```

Mas cuidado para não esconder regras de domínio importantes.

Middleware serve para preocupações transversais.

---

# 108. Command Middleware

Exemplos bons:

```text
authentication
authorization
rate limiting
metrics
tracing
deduplication
```

Exemplos ruins:

```text
"decide se bloco pode ser quebrado"
```

Isso é Build/World/Block.

---

# 109. Command Bus

Pode existir:

```text
CommandBus
```

mas não deve ser confundido com Event Bus.

```text
Command Bus
→ entrega intenção a um executor

Event Bus
→ distribui fatos
```

---

# 110. Command Bus vs Event Bus

```text
COMMAND
   ↓
ONE AUTHORITY / HANDLER

EVENT
   ↓
MANY SUBSCRIBERS
```

Essa é a distinção central.

---

# 111. Folder Structure

```text
src/command/

├── core/
│   ├── command
│   ├── command-context
│   ├── command-result
│   └── command-state
│
├── definition/
│   ├── command-definition
│   ├── schema
│   └── version
│
├── registry/
│   └── command-registry
│
├── dispatcher/
│   ├── dispatcher
│   ├── handler
│   └── routing
│
├── queue/
│   ├── queue
│   ├── priority
│   └── scheduler
│
├── validation/
│   ├── validator
│   ├── authorization
│   ├── preconditions
│   └── limits
│
├── transaction/
│   ├── transaction
│   ├── reservation
│   └── rollback
│
├── serialization/
│
├── replay/
│
├── tracing/
│
├── metrics/
│
├── middleware/
│
├── simulation/
│
├── mod/
│   └── registration
│
└── debug/
```

---

# 112. Dependências

```text
CORE
 │
 ├── Registry
 ├── Event Bus
 ├── Server
 ├── Networking
 └── Mod Runtime
        │
        ▼
   COMMAND SYSTEM
        │
        ├── Player
        ├── Entity
        ├── Block
        ├── Item
        ├── Crafting
        ├── Combat
        ├── Machine
        ├── Economy
        ├── Quest
        └── Dimension
```

---

# 113. Onde ele fica na arquitetura

```text
                           NEXORA
                              │
                           SERVER
                              │
                      ┌───────┴───────┐
                      ▼               ▼
                 NETWORKING       COMMAND SYSTEM
                      │               │
                      │               ▼
                      │           VALIDATION
                      │               │
                      └───────────────┤
                                      ▼
                                  SIMULATION
                                      │
            ┌─────────────┬───────────┼─────────────┐
            ▼             ▼           ▼             ▼
          ENTITY        BLOCK        ITEM          AI
            │             │           │             │
            └─────────────┴───────────┴─────────────┘
                                      │
                                  STATE CHANGE
                                      │
                                  EVENT BUS
                                      │
                          ┌───────────┴───────────┐
                          ▼                       ▼
                     PERSISTENCE             NETWORKING
```

---

# 114. Implementação por fases

## CMD-0 — Core Contracts

Criar:

```text
ICommand
ICommandHandler
ICommandDispatcher
ICommandResult
ICommandContext
```

---

## CMD-1 — Registry

```text
CommandRegistry
CommandDefinition
CommandID
```

---

## CMD-2 — Basic Dispatch

```text
create
register
dispatch
execute
result
```

---

## CMD-3 — Validation

```text
schema
identity
authority
permission
```

---

## CMD-4 — Queue

```text
enqueue
priority
schedule
cancel
```

---

## CMD-5 — Server Integration

```text
Networking
 ↓
Command
 ↓
Server
```

---

## CMD-6 — Player

```text
Move
Interact
UseItem
```

---

## CMD-7 — World

```text
BreakBlock
PlaceBlock
```

---

## CMD-8 — Inventory

```text
MoveItem
Split
Merge
Drop
```

---

## CMD-9 — Transactions

```text
idempotency
reservations
commit
rollback
```

---

## CMD-10 — Scripts

```text
Script
 ↓
Command API
```

---

## CMD-11 — NPC / AI

```text
AI
 ↓
Command
```

---

## CMD-12 — Admin

```text
Console
 ↓
Command System
```

---

## CMD-13 — Replay

```text
record
replay
trace
```

---

## CMD-14 — Simulation

```text
dry run
preview
preconditions
conflict detection
```

---

## CMD-15 — Scalability

```text
priority
batch
region affinity
budgets
```

---

# 115. Primeiro Vertical Slice

O primeiro teste:

```text
CLIENT
 ↓
Networking
 ↓
BreakBlockCommand
 ↓
Command Validation
 ↓
Server
 ↓
Build & Destruction
 ↓
Block System
 ↓
BlockBrokenEvent
 ↓
Loot
 ↓
Item
 ↓
Persistence
 ↓
Networking
 ↓
CLIENTS
```

Isso prova quase toda a arquitetura fundamental.

---

# 116. Segundo Vertical Slice

Inventário:

```text
CLIENT
 ↓
MoveItemCommand
 ↓
Server
 ↓
Inventory validation
 ↓
Item transaction
 ↓
Commit
 ↓
ItemTransferredEvent
 ↓
Persistence
 ↓
Replication
```

---

# 117. Terceiro Vertical Slice

Script:

```text
SCRIPT
 ↓
Command API
 ↓
SpawnEntityCommand
 ↓
Server validation
 ↓
Entity System
 ↓
EntitySpawnedEvent
```

---

# 118. Quarto Vertical Slice

NPC:

```text
NPC AI
 ↓
BuildStructureCommand
 ↓
Server
 ↓
Structure System
 ↓
Build & Destruction
 ↓
Event Bus
```

Isso prova que o mundo pode reagir e agir sem o player estar no centro de tudo.

---

# 119. Quinto Vertical Slice

Economia:

```text
PLAYER
 ↓
TradeCommand
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
TradeCompletedEvent
 ↓
Persistence
 ↓
Networking
```

---

# 120. Golden Test

```text
SERVER
+
CLIENT A
+
CLIENT B

A sends command
        ↓
Server receives
        ↓
Command validated
        ↓
System executes
        ↓
State changes
        ↓
Event published
        ↓
Persistence updated
        ↓
B receives replication
```

Verificar:

```text
command result
world state
event
save state
network state
```

Tudo precisa ser consistente.

---

# 121. Stress Test

```text
1 command/tick
100
1.000
10.000
100.000
1.000.000 queued abstract operations
```

Separar por categorias:

```text
player commands
NPC commands
AI commands
script commands
admin commands
mod commands
```

---

# 122. Queue Flood Test

Simular:

```text
10.000 players
+
command spam
```

O sistema deve:

```text
rate-limit
prioritize
reject
coalesce
protect critical operations
```

---

# 123. Duplicate Test

Enviar:

```text
CommandInstanceID X
```

dez vezes.

Resultado:

```text
one logical execution
```

quando o comando for idempotente.

---

# 124. Race Condition Test

Dois clients:

```text
A → take item
B → take same item
```

Servidor:

```text
State Version
```

deve garantir que somente uma operação válida seja aplicada.

---

# 125. Disconnect Test

```text
Client sends command
↓
connection drops
```

Precisamos saber se o comando:

```text
queued?
executed?
committed?
cancelled?
```

e manter o comportamento determinístico.

---

# 126. Replay Test

```text
World Snapshot
+
Command Log
```

reproduzir.

Resultado esperado:

```text
final state equivalent
```

onde o domínio permitir determinismo.

---

# 127. Mod Test

Carregar:

```text
example:magic
```

que registra:

```text
example:cast_spell
```

Fluxo:

```text
Script/UI
 ↓
Command
 ↓
Validation
 ↓
Magic System
 ↓
Event
```

sem alteração do Core.

---

# 128. Debug de um comando

O desenvolvedor deve conseguir responder:

```text
Quem pediu?
De onde veio?
Quando?
Qual versão?
Qual mundo?
Qual dimensão?
Qual entidade?
Qual estado anterior?
Qual validação falhou?
Qual sistema executou?
Qual evento surgiu?
Foi persistido?
Foi replicado?
```

Isso precisa ser possível por tracing.

---

# 129. Performance

O Command System precisa ser leve.

No hot path:

```text
CommandID
↓
Runtime Handle
↓
Handler
```

Evitar:

```text
string parsing
+
reflection
+
dynamic lookup
```

em todas as etapas.

---

# 130. Command Compilation

Uma otimização futura:

```text
Command Definition
 ↓
Compile Schema
 ↓
Runtime Validator
 ↓
Fast Dispatch
```

---

# 131. Command Batching

Exemplo de construção:

```text
1.000 blocks
```

em vez de:

```text
1.000 individual commands
```

pode existir:

```text
BulkBuildCommand
```

com:

```text
1 authorization
1 transaction
1 region scheduling
```

quando for seguro.

---

# 132. Future Distributed Server

O desenho já fica preparado para:

```text
Client
 ↓
Gateway
 ↓
Command Router
 ↓
Region Server
```

mais tarde.

---

# 133. Command Routing futuro

```text
Command
 ↓
Authority Resolver
 ↓
Region/Dimension/World
 ↓
Responsible Server
```

Isso pode ser implementado muito depois.

---

# 134. Limites do Command System

### Não deve conter:

```text
Combat logic
AI logic
Inventory rules
Block rules
Crafting rules
Economy rules
Physics
WorldGen
```

### Deve conter:

```text
Command definitions
Registry
Routing
Queue
Validation framework
Authority boundary
Transactions
Dispatch
Results
Tracing
Replay
Quotas
```

---

# 135. Arquitetura final

```text
                         NEXORA
                            │
                           CORE
                            │
                   PUBLIC INFRASTRUCTURE
                            │
       ┌────────────────────┼────────────────────┐
       ▼                    ▼                    ▼
    REGISTRY             EVENT BUS          PERSISTENCE
                            │
                            │
                        SERVER
                            │
                 ┌──────────┴──────────┐
                 ▼                     ▼
            NETWORKING              COMMAND
                 │                     │
                 └──────────┬──────────┘
                            ▼
                      VALIDATION
                            │
                      AUTHORITY
                            │
                        SCHEDULER
                            │
                         HANDLER
                            │
                            ▼
                       SIMULATION
                            │
       ┌────────┬───────────┼──────────┬─────────┐
       ▼        ▼           ▼          ▼         ▼
     ENTITY   WORLD        ITEM       COMBAT    AI
       │        │           │          │         │
       └────────┴───────────┴──────────┴─────────┘
                            │
                       STATE CHANGE
                            │
                        EVENT BUS
                            │
                  ┌─────────┴─────────┐
                  ▼                   ▼
             PERSISTENCE          NETWORKING
```

E o conjunto de regras do NEXORA passa a ficar:

```text
COMMAND
→ "eu quero fazer isso"

SERVER
→ "você pode solicitar isso?"

VALIDATION
→ "essa operação é válida agora?"

SPECIALIZED SYSTEM
→ "como essa mecânica funciona?"

STATE CHANGE
→ "o mundo mudou"

EVENT BUS
→ "quem precisa saber?"

PERSISTENCE
→ "isso precisa sobreviver"

NETWORKING
→ "quem precisa receber?"
```

### A arquitetura da fundação agora

```text
NEXORA FOUNDATION
│
├── CORE
├── REGISTRY
├── EVENT BUS
├── PERSISTENCE
│
├── ENTITY
├── BLOCK
├── ITEM
│
├── ANIMATION
├── AUDIO
├── UI
├── DIMENSION
├── STRUCTURE
│
├── NETWORKING
├── SERVER
├── MOD RUNTIME
├── SCRIPTING
└── COMMAND SYSTEM
```

Isso fecha uma das partes mais importantes da arquitetura: **qualquer origem de intenção — jogador, NPC, IA, script, mod ou administrador — pode entrar pelo mesmo mecanismo de Command, passar pela mesma fronteira de autoridade do Server e chegar ao sistema especializado sem criar caminhos paralelos escondidos.**
