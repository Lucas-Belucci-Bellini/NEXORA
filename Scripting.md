# NEXORA — SCRIPTING SYSTEM

> **Princípio central:**
> **Scripting permite que conteúdo e mods expressem comportamento através de APIs controladas, sem precisar alterar o Core do NEXORA.**
>
> Script não é o motor do jogo.
> Script é uma **camada de extensibilidade**.

A separação deve ser:

```text
CORE
 ↓
PUBLIC APIs
 ↓
SCRIPTING RUNTIME
 ↓
SCRIPT API
 ↓
MOD / CONTENT / GAMEPLAY
```

E nunca:

```text
SCRIPT
 ↓
raw engine memory
 ↓
everything
```

---

# 1. O que é o Scripting System

O Scripting System deve permitir coisas como:

```text
novos comportamentos
novas interações
quests
eventos
máquinas
IA complementar
UI lógica
world events
automação
regras de gameplay
ferramentas de mod
```

sem exigir que o autor escreva código nativo do engine.

Exemplo:

```text
script
 ↓
"quando o jogador entrar nesta região"
 ↓
consultar estado
 ↓
executar comando
 ↓
publicar evento
```

---

# 2. O que o Scripting NÃO é

Scripting não deve controlar diretamente:

```text
render pipeline
memory allocator
network transport
physics internals
chunk storage internals
GPU
threads arbitrários
save file internals
```

Esses continuam protegidos.

---

# 3. Arquitetura

```text id="scr01"
                    NEXORA CORE
                         │
                   PUBLIC APIs
                         │
                  SCRIPTING SYSTEM
                         │
       ┌─────────────────┼─────────────────┐
       ▼                 ▼                 ▼
    LANGUAGE          RUNTIME           API BINDINGS
     LAYER               │                 │
       │                 │                 │
       └─────────────────┼─────────────────┘
                         │
                      SCRIPTS
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
        MODS          CONTENT        TOOLS
```

---

# 4. Objetivo arquitetural

O sistema deve conseguir suportar diferentes linguagens sem que o Core conheça todas.

```text
SCRIPT
 ↓
LANGUAGE ADAPTER
 ↓
SCRIPT RUNTIME
 ↓
NEXORA SCRIPT API
 ↓
ENGINE
```

Assim podemos ter futuramente:

```text
Lua
WASM
JavaScript
TypeScript subset
NEXORA Script
ou outra VM
```

sem mudar os sistemas do jogo.

---

# 5. Não escolher a linguagem no Core

Evitar:

```text
Core
└── LuaEverything
```

Preferir:

```text
Scripting
├── Runtime
├── API
├── Binding
└── Adapters
    ├── Language A
    ├── Language B
    └── Language C
```

Isso mantém liberdade tecnológica.

---

# 6. Camadas

Sugestão:

```text
SCRIPTING
├── SCRIPT DISCOVERY
├── SCRIPT MANIFEST
├── LANGUAGE ADAPTER
├── SCRIPT VM
├── EXECUTION CONTEXT
├── API BINDINGS
├── EVENT BINDINGS
├── COMMAND BINDINGS
├── QUERY API
├── TASK API
├── DATA API
├── SECURITY
├── RESOURCE LIMITS
├── ERROR HANDLING
├── DEBUGGING
├── PROFILING
├── HOT RELOAD
├── PERSISTENCE BRIDGE
├── NETWORK BRIDGE
└── MOD API
```

---

# 7. Script Definition

Um script precisa de identidade.

```text id="scr02"
ScriptID
Namespace
Version
Language
EntryPoint
Permissions
Dependencies
```

Exemplo:

```yaml
id: example:weather_event
version: 1.0.0
language: wasm

permissions:
  - world.read
  - event.subscribe
```

---

# 8. Script vs Mod

Um Mod pode conter scripts.

```text id="scr03"
MOD
├── native code
├── scripts
├── resources
├── data
└── content
```

Mas um pacote também pode ser predominantemente script:

```text
script-heavy mod
```

A distinção:

```text
MOD
→ pacote/ecossistema

SCRIPT
→ unidade executável
```

---

# 9. Script Lifecycle

```text id="scr04"
DISCOVERED
↓
PARSED
↓
VALIDATED
↓
DEPENDENCIES RESOLVED
↓
COMPILED
↓
LOADED
↓
INITIALIZED
↓
READY
↓
RUNNING
↓
STOPPING
↓
UNLOADED
```

Em caso de falha:

```text
FAILED
QUARANTINED
```

---

# 10. Compilation

Dependendo da linguagem:

```text
Source
 ↓
Parser
 ↓
AST
 ↓
Validator
 ↓
Compiler
 ↓
Bytecode / WASM / VM code
 ↓
Runtime
```

Para scripts interpretados:

```text
Source
 ↓
Parser
 ↓
Validated Representation
 ↓
Runtime
```

O Core não deve depender de uma única estratégia.

---

# 11. Script Runtime

O Runtime fornece:

```text id="scr05"
load
execute
pause
resume
stop
reload
call
schedule
cancel
inspect
```

Interface conceitual:

```text
IScriptRuntime
```

---

# 12. Execution Context

Todo script deve executar dentro de um contexto.

```text id="scr06"
ScriptExecutionContext
├── ScriptID
├── ModID
├── WorldContext
├── EntityContext
├── EventContext
├── Permissions
├── TimeBudget
├── MemoryBudget
└── RNG
```

---

# 13. Nunca dar o mundo inteiro

Não:

```text id="scr07"
script.world.rawMemory
```

Sim:

```text id="scr08"
WorldQuery
WorldCommand
EntityQuery
EntityCommand
```

O script pode perguntar:

```text
qual bloco está aqui?
```

sem receber acesso aos internals do mundo.

---

# 14. Query API

Scripting deve possuir APIs de consulta.

Exemplo conceitual:

```text
world.getBlock(position)
world.getEntities(area)
world.getBiome(position)
world.getTime()
world.getDimension()
```

As APIs precisam respeitar:

```text
permissions
scope
performance
threading
```

---

# 15. Command API

Alterações no mundo devem ser feitas através de comandos.

```text
script
 ↓
WorldCommand
 ↓
Simulation Queue
 ↓
Server
 ↓
System
```

Exemplo:

```text
placeBlock(position, block)
```

vira internamente uma operação validável.

---

# 16. Commands vs Direct Mutation

Regra:

```text
QUERY
→ pode consultar estado

COMMAND
→ solicita mudança
```

Nunca:

```text
script
 ↓
world.blocks[index] = ...
```

Isso quebraria:

```text
authority
events
physics
lighting
persistence
network replication
```

---

# 17. Event Integration

Scripts podem observar eventos:

```text id="scr09"
PlayerJoined
BlockBroken
EntitySpawned
EntityDied
MachineCompleted
WeatherChanged
QuestCompleted
DimensionChanged
StructureCreated
```

Fluxo:

```text
SYSTEM
 ↓
EVENT BUS
 ↓
SCRIPT SUBSCRIBER
```

---

# 18. Script Events

Também podemos possuir eventos específicos do scripting:

```text id="scr10"
ScriptLoaded
ScriptStarted
ScriptStopped
ScriptError
ScriptTimeout
ScriptQuotaExceeded
```

---

# 19. Event Subscription Ownership

Toda subscription tem:

```text id="scr11"
ownerScript
ownerMod
```

Quando o script é removido:

```text
unsubscribe
cancel tasks
release resources
```

automaticamente.

---

# 20. Task Scheduler

Scripts frequentemente precisarão de:

```text
wait
delay
repeat
schedule
```

Exemplo conceitual:

```text
schedule(20 ticks)
```

Mas não pode criar milhares de timers sem limites.

---

# 21. Script Scheduler

```text id="scr12"
SCRIPT SCHEDULER
├── Immediate
├── Tick
├── Delayed
├── Periodic
├── Event-triggered
└── Background
```

Com quotas.

---

# 22. Script Time Budget

Cada execução possui:

```text id="scr13"
CPU budget
instruction budget
execution deadline
```

Se ultrapassar:

```text id="scr14"
TIMEOUT
```

O Runtime interrompe com segurança quando suportado pelo ambiente.

---

# 23. Infinite Loop Protection

Script ruim:

```text
while true
```

não pode congelar o servidor.

Arquitetura:

```text
Script
 ↓
Budget
 ↓
Exceeded
 ↓
Abort / Yield
 ↓
Error Report
```

---

# 24. Memory Limits

Cada runtime/script pode possuir:

```text id="scr15"
Memory Budget
Allocation Budget
Object Count
Data Size
```

Isso impede abuso.

---

# 25. Network Quotas

Scripts também devem ter limites para:

```text id="scr16"
custom messages
events
commands
outgoing data
incoming data
```

Um script não pode gerar tráfego infinito.

---

# 26. Entity Spawn Quota

Imagine um script com:

```text
spawn(entity)
```

Ele precisa possuir limites.

```text id="scr17"
per tick
per second
per world
per mod
per script
```

---

# 27. World Modification Quota

Mesma ideia:

```text id="scr18"
block changes
entity changes
structure operations
fluid operations
```

devem possuir budgets.

Isso também protege contra scripts mal escritos.

---

# 28. Permissions

Sistema:

```text id="scr19"
script.world.read
script.world.write
script.entity.read
script.entity.spawn
script.entity.modify
script.registry.read
script.registry.register
script.network.send
script.storage.read
script.storage.write
script.command.register
script.ui.create
```

---

# 29. Capability Security

Melhor ainda:

```text id="scr20"
Script
 ↓
Capability
```

Exemplo:

```text
WorldReadCapability
EventSubscribeCapability
EntitySpawnCapability
StorageCapability
```

O script só recebe o capability solicitado e aprovado.

---

# 30. Server vs Client Scripts

Script pode declarar:

```text id="scr21"
SERVER
CLIENT
BOTH
```

### Server

Pode participar de:

```text
simulation
gameplay
world events
economy
quests
```

### Client

Pode participar de:

```text
UI
cosmetic behavior
presentation
local effects
```

### Both

Precisa respeitar as restrições de cada lado.

---

# 31. Server Authority

Mesmo que exista script no cliente:

```text id="scr22"
CLIENT SCRIPT
```

não ganha autoridade.

Exemplo:

```text
client script
→ request break block
→ server validates
```

---

# 32. Deterministic RNG

Scripts que participam de geração/simulação precisam poder usar:

```text id="scr23"
ScriptRandom
```

derivado de:

```text
world seed
script id
context seed
tick
position
```

quando determinismo for necessário.

Não depender diretamente de:

```text
Math.random()
```

para lógica determinística.

---

# 33. Time API

Scripts não devem depender diretamente de:

```text
system clock
```

para simulação.

Disponibilizar:

```text id="scr24"
gameTick
worldTime
dimensionTime
season
simulationDelta
```

E uma API separada para tempo real quando realmente necessário.

---

# 34. Threading

Regra:

```text id="scr25"
Script
≠
thread arbitrária
```

O Runtime pode oferecer:

```text
scheduleTask
runAsync
runOnSimulation
```

Mas operações de mundo retornam para a fila apropriada.

---

# 35. Async

Exemplo:

```text
script
 ↓
request expensive calculation
 ↓
worker
 ↓
result
 ↓
script continuation
```

Mas o worker não escreve diretamente no mundo.

---

# 36. API de World Snapshot

Para cálculos:

```text
WorldSnapshot
```

pode ser fornecido.

Exemplo:

```text
AI analysis
 ↓
snapshot
 ↓
script
 ↓
result
```

Isso é mais seguro do que deixar o script segurar referências vivas ao mundo.

---

# 37. Script Persistence

Scripts podem precisar guardar estado:

```text id="scr26"
quest progress
machine configuration
NPC memory
custom variables
world event state
```

API:

```text
ScriptStorage
```

O Persistence System continua sendo responsável pelo armazenamento real.

---

# 38. Persistent Script State

Exemplo:

```json
{
  "questStage": 3,
  "completed": false,
  "counter": 87
}
```

Esse estado deve possuir:

```text id="scr27"
schemaVersion
scriptID
modID
migration
```

---

# 39. Script Migration

Quando o script muda:

```text id="scr28"
v1
 ↓
migration
 ↓
v2
```

A mudança deve ser explícita.

---

# 40. Script Versioning

Separar:

```text id="scr29"
Game Version
Mod Version
Script Version
API Version
Runtime Version
```

---

# 41. API Binding

Cada função exposta ao script passa por um binding.

```text id="scr30"
Script
 ↓
Binding
 ↓
Validation
 ↓
NEXORA API
```

O binding pode validar:

```text
types
permissions
arguments
scope
authority
quotas
```

---

# 42. Type System

A API deve fornecer tipos bem definidos.

Exemplo:

```text id="scr31"
BlockID
ItemID
EntityID
DimensionID
Position
Rotation
FluidStack
EnergyAmount
PlayerID
StructureID
```

Evitar transformar tudo em:

```text
string
```

---

# 43. Handles

Para objetos vivos:

```text id="scr32"
EntityHandle
BlockHandle
MachineHandle
StructureHandle
```

O handle pode tornar-se inválido.

O script precisa testar:

```text
handle.isValid()
```

---

# 44. Não deixar referências eternas

Um script não deve segurar um objeto interno para sempre.

Em vez disso:

```text
Handle
 ↓
resolve
 ↓
use
 ↓
release
```

Isso facilita:

```text
chunk unload
entity despawn
dimension unload
```

---

# 45. Cross-Dimension Scripts

Um script pode acompanhar:

```text
Player
```

através de dimensões.

Mas o contexto precisa atualizar:

```text
dimension
position
world
```

durante a transferência.

---

# 46. Script + Entity

Um entity pode possuir um componente/script behavior.

```text
Entity
├── Transform
├── Health
├── AI
└── ScriptBehavior
```

O script não substitui o Entity System.

Ele fornece comportamento adicional.

---

# 47. Script + AI

Possível arquitetura:

```text
AI System
 ↓
Behavior Profile
 ↓
Scripted Behavior
```

Mas o AI System continua responsável pela estrutura de decisão/percepção.

---

# 48. Script + Quest

```text
Quest System
 ↓
Quest condition
 ↓
Script
 ↓
custom logic
```

Isso permite quests mais flexíveis sem modificar Quest Core.

---

# 49. Script + Machine

Uma máquina pode utilizar script para comportamento especial:

```text
Machine
 ↓
Machine Definition
 ↓
Script Controller
```

Mas Energy/Fluid/Crafting continuam sendo os sistemas responsáveis por seus respectivos recursos.

---

# 50. Script + WorldGen

WorldGen pode permitir hooks:

```text
before chunk generation
after terrain
after biome
after structures
```

Mas com enorme cuidado de performance.

Scripts de WorldGen precisam possuir:

```text
determinism
time limits
memory limits
chunk scope
```

---

# 51. Script + Structures

Scripts podem:

```text
observe structure
modify blueprint
request placement
react to construction
```

Mas Structure System continua sendo o dono das estruturas.

---

# 52. Script + Events

Exemplo:

```text
BlockBroken
 ↓
Script
 ↓
check condition
 ↓
spawn custom reward
 ↓
Command
 ↓
Server
```

---

# 53. Script + Commands

Scripts podem registrar novos comandos:

```text
example:ritual
example:research
example:factory
```

O Command System continua responsável pelo parsing, permissionamento e execução.

---

# 54. Script + Networking

Scripts podem registrar mensagens:

```text id="scr33"
example:ui_state
example:machine_mode
```

mas através do Mod Runtime + Networking.

```text
SCRIPT
 ↓
MOD NETWORK API
 ↓
NETWORKING
```

---

# 55. Script + UI

Client script pode conectar comportamento a UI:

```text
button
 ↓
script handler
 ↓
command
```

Mas operações do servidor continuam autoritativas.

---

# 56. Script + Audio

Script pode solicitar evento sem acessar o backend diretamente:

```text
playSoundEvent(...)
```

O Audio System decide como reproduzir.

---

# 57. Script + Animation

Script pode:

```text
set animation state
trigger animation event
```

Mas não manipula diretamente o renderer.

---

# 58. Script Debugger

Precisamos de ferramentas como:

```text id="scr34"
nexora script list
nexora script inspect
nexora script reload
nexora script disable
nexora script enable
nexora script profile
nexora script trace
nexora script memory
nexora script permissions
```

---

# 59. Script Profiler

Por script:

```text id="scr35"
CPU time
calls
events handled
commands issued
allocations
memory
network traffic
timeouts
errors
```

Por mod:

```text
total script CPU
total script memory
total callbacks
```

---

# 60. Execution Trace

Exemplo:

```text
Tick 18299

example:weather
 ├── Event: ClimateChanged
 ├── Query: World
 ├── Command: SetWeather
 └── Event: WeatherChanged

CPU: 0.19ms
Memory: 12KB
```

Isso ajuda muito a descobrir loops e gargalos.

---

# 61. Script Errors

Erro deve indicar:

```text
Script ID
Mod
Version
Language
File
Line
Function
Event
Permission
```

Exemplo:

```text
SCRIPT ERROR

Mod:
example:weather

Script:
example:storm_controller

Event:
ClimateChanged

Function:
updateStorm()

Reason:
world.write permission denied
```

---

# 62. Crash Isolation

Idealmente:

```text
script error
```

não derruba o servidor.

Fluxo:

```text
SCRIPT
 ↓
ERROR
 ↓
RUNTIME BOUNDARY
 ↓
DISABLE CALLBACK / SCRIPT
 ↓
REPORT
```

---

# 63. Script Quarantine

Se um script exceder repetidamente:

```text
CPU
Memory
Events
Network
Errors
```

pode entrar em:

```text
QUARANTINED
```

---

# 64. Hot Reload

Separar:

```text
DATA RELOAD
SCRIPT RELOAD
RUNTIME RESTART
```

Alguns scripts podem ser recarregados sem reiniciar o mundo.

Mas scripts com estado persistente devem possuir política de migração.

---

# 65. Development Reload

Fluxo:

```text
edit
 ↓
compile
 ↓
validate
 ↓
replace runtime
 ↓
restore state
 ↓
continue
```

Se falhar:

```text
restore previous version
```

quando possível.

---

# 66. Script Package Format

Uma possibilidade:

```text
example-script/
├── manifest
├── scripts/
├── data/
├── assets/
├── schemas/
└── migrations/
```

O formato físico ainda pode ser decidido depois.

---

# 67. Script Dependency

Script pode depender de API/lib:

```text
script A
 ↓
script library B
 ↓
NEXORA API
```

O Mod Runtime resolve dependências.

---

# 68. Script Libraries

Bibliotecas comuns:

```text
math
world utilities
UI helpers
quest helpers
machine helpers
network helpers
```

Mas precisam continuar dentro da sandbox.

---

# 69. Standard Library

A linguagem escolhida pode possuir uma biblioteca padrão restrita.

Evitar que um script possa acessar livremente:

```text
filesystem
processes
sockets
native libraries
```

sem capability.

---

# 70. Filesystem Access

Por padrão:

```text
DENIED
```

Se necessário:

```text
sandboxed storage
```

em vez de:

```text
C:/
```

ou equivalente.

---

# 71. Process Access

Por padrão:

```text
DENIED
```

Script não deve iniciar processos externos.

Isso é especialmente importante para servidores multiplayer.

---

# 72. External Network

Por padrão:

```text
DENIED
```

Um script não deve poder fazer requisições arbitrárias à internet.

Quando houver necessidade legítima, isso deve ser uma capability explicitamente concedida e altamente limitada.

---

# 73. Security Levels

Podemos definir:

```text id="scr36"
DATA_ONLY
SANDBOXED
TRUSTED_SCRIPT
NATIVE
```

### DATA_ONLY

Nenhum código.

### SANDBOXED

API restrita.

### TRUSTED SCRIPT

Mais capacidades, para ambientes confiáveis.

### NATIVE

Acesso de maior privilégio.

---

# 74. Dedicated Server

No servidor:

```text
SCRIPT
 ↓
Server API
```

não deve depender de:

```text
Renderer
Audio
GPU
UI frontend
```

Isso permite scripts server-side em ambiente headless.

---

# 75. Client Script

No cliente:

```text
SCRIPT
 ↓
UI
Audio
Animation
Presentation
```

Mas ainda com sandbox.

---

# 76. Multiplayer

Nunca usar script cliente para validar:

```text
damage
inventory
economy
world changes
```

Servidor sempre pode negar.

---

# 77. Script API Version

A API precisa de:

```text id="scr37"
Script API v1
Script API v2
Script API v3
```

com:

```text
deprecation
migration
compatibility adapters
```

---

# 78. API Contract

Cada API deve definir:

```text id="scr38"
input
output
errors
permissions
threading
authority
performance cost
version
```

Por exemplo:

```text
world.getBlock(position)

Permission:
world.read

Thread:
simulation/query-safe

Cost:
LOW

Returns:
BlockState
```

---

# 79. Performance Classes

Muito útil definir:

```text id="scr39"
TRIVIAL
LOW
MEDIUM
HIGH
EXPENSIVE
ASYNC_ONLY
```

Um scriptificador consegue entender o custo das APIs.

---

# 80. Batch API

Para evitar:

```text
1000 calls
```

podemos fornecer:

```text
world.getBlocks(area)
entity.query(area)
```

ou APIs batch.

Isso é importante para scripts de automação e IA.

---

# 81. Event Coalescing

Se houver:

```text
1000 block updates
```

não necessariamente gerar:

```text
1000 script callbacks
```

Podemos fornecer:

```text
BlockBatchChangedEvent
```

quando apropriado.

---

# 82. Script Scheduling LOD

Scripts também podem seguir:

```text
FULL
REGIONAL
ABSTRACT
```

Não permitir que 100.000 entidades tenham scripts executados individualmente a cada tick sem controle.

---

# 83. Script AI em larga escala

Por exemplo:

```text
50.000 NPCs
```

não significa:

```text
50.000 VMs
50.000 scripts × 60 Hz
```

Em vez disso:

```text
nearby
→ FULL

regional
→ aggregated

abstract
→ event-driven
```

---

# 84. Script Instances

Podemos compartilhar código:

```text
ScriptDefinition
```

e possuir instâncias leves:

```text
ScriptInstance
```

Assim:

```text
1 ScriptDefinition
+
10.000 instances
```

não significa 10.000 cópias do bytecode.

---

# 85. Script State

Somente dados mutáveis específicos devem existir por instância:

```text
state
variables
timers
handles
```

---

# 86. Script VM Pool

Dependendo da tecnologia:

```text
VM Pool
```

pode reutilizar runtimes.

Isso deve ser medido; não assumir que pooling sempre melhora.

---

# 87. Script Cancellation

Precisa existir:

```text
cancel(script)
cancel(task)
cancel(owner)
cancel(mod)
```

Para unload:

```text
cancel all
```

---

# 88. Script Ownership

Tudo precisa ter owner:

```text
script
task
subscription
storage
network channel
entity behavior
```

Assim:

```text
disable mod
```

pode realmente limpar seus recursos.

---

# 89. Mod Runtime + Scripting

A relação correta:

```text
MOD RUNTIME
      │
      ▼
SCRIPTING RUNTIME
      │
      ▼
SCRIPT
```

Mod Runtime controla:

```text
lifecycle
dependency
ownership
permissions
resources
```

Scripting controla:

```text
execution
language
VM
bindings
script state
```

---

# 90. Server + Scripting

```text
SERVER
 ↓
MOD RUNTIME
 ↓
SCRIPTING
 ↓
SCRIPT
 ↓
PUBLIC API
 ↓
SIMULATION
```

---

# 91. Registry + Scripting

Scripts podem registrar:

```text
events
commands
behaviors
recipes
data
```

e, quando permitido, conteúdo através do Registry.

---

# 92. Event Bus + Scripting

```text
Event Bus
 ↓
Script Subscription
 ↓
Script
```

Com:

```text
scope
priority
quota
owner
```

---

# 93. Persistence + Scripting

```text
Script State
 ↓
Persistence API
 ↓
Persistence System
```

Nunca:

```text
Script → save file format
```

---

# 94. Networking + Scripting

```text
Script
 ↓
Network API
 ↓
Networking
```

Nunca:

```text
Script → socket direto
```

por padrão.

---

# 95. Primeiro Vertical Slice

O primeiro teste deve ser extremamente simples:

```text
SERVER
 ↓
MOD RUNTIME
 ↓
SCRIPT
 ↓
register event
 ↓
PlayerJoined
 ↓
script executes
 ↓
log message
```

Isso prova:

```text
Loader
Runtime
Event Bus
Server
```

---

# 96. Segundo Vertical Slice

```text
SCRIPT
 ↓
BlockBrokenEvent
 ↓
query player
 ↓
check condition
 ↓
issue Item command
 ↓
Item System
 ↓
Event
 ↓
Persistence
```

Prova que script pode participar do gameplay sem quebrar as fronteiras.

---

# 97. Terceiro Vertical Slice

Máquina:

```text
Machine
 ↓
Script Controller
 ↓
Recipe
 ↓
Energy
 ↓
Fluid
 ↓
Inventory
 ↓
UI
 ↓
Network
 ↓
Save
```

Esse teste demonstra quase toda a cadeia de extensibilidade.

---

# 98. Quarto Vertical Slice

Quest:

```text
Player enters region
 ↓
Event
 ↓
Script
 ↓
Quest condition
 ↓
Quest System
 ↓
Reward
 ↓
Item
 ↓
Persistence
```

---

# 99. Quinto Vertical Slice

World event:

```text
Climate
 ↓
Storm condition
 ↓
Script
 ↓
World Event
 ↓
Civilization
 ↓
NPC reaction
 ↓
Audio
 ↓
Network
```

Isso aproxima o scripting da filosofia:

> **O mundo vive e reage.**

---

# 100. Testes de segurança

Testar:

```text
infinite loop
memory allocation abuse
event recursion
task flood
entity spawn flood
block modification flood
network flood
storage flood
permission violation
invalid handle
cross-dimension stale handle
mod unload during callback
script reload during execution
```

---

# 101. Teste de estabilidade

```text
1 script
10
100
1.000
10.000
100.000 instances
```

Com:

```text
events
tasks
queries
world changes
```

---

# 102. Teste de performance

Medir:

```text
script execution time
binding overhead
VM startup
VM memory
event dispatch
query cost
command cost
serialization
state persistence
```

---

# 103. Chaos Test

```text
script crash
 ↓
mod unload
 ↓
player disconnect
 ↓
dimension unload
 ↓
world save
 ↓
server restart
```

O sistema deve permanecer consistente.

---

# 104. API pública

```text id="scr40"
IScriptingSystem
IScriptRuntime
IScript
IScriptDefinition
IScriptContext
IScriptEngine
IScriptCompiler
IScriptModule
IScriptScheduler
IScriptStorage
IScriptPermissionManager
IScriptCapabilityManager
IScriptDebugger
IScriptProfiler
IScriptSerializer
IScriptMigration
IScriptBinding
IScriptEventBridge
IScriptCommandBridge
IScriptNetworkBridge
```

---

# 105. Organização do código

```text id="scr41"
src/scripting/

├── core/
│   ├── scripting-system
│   ├── script-context
│   ├── script-state
│   └── config
│
├── manifest/
│   ├── script-manifest
│   ├── schema
│   └── validation
│
├── discovery/
│   ├── scanner
│   └── catalog
│
├── runtime/
│   ├── runtime
│   ├── instance
│   ├── lifecycle
│   └── limits
│
├── language/
│   ├── adapter
│   ├── compiler
│   └── registry
│
├── bindings/
│   ├── world
│   ├── entity
│   ├── item
│   ├── block
│   ├── machine
│   ├── quest
│   ├── ui
│   ├── audio
│   ├── animation
│   └── networking
│
├── events/
│   └── event-bridge
│
├── commands/
│   └── command-bridge
│
├── scheduler/
│   ├── scheduler
│   ├── task
│   └── budget
│
├── security/
│   ├── permissions
│   ├── capabilities
│   └── sandbox
│
├── persistence/
│   ├── state
│   └── migration
│
├── networking/
│   └── bridge
│
├── debugging/
│
├── profiling/
│
└── mod/
    └── runtime-integration
```

---

# 106. Ordem de implementação

## SCRIPT-0 — Contracts

Criar:

```text
IScript
IScriptRuntime
IScriptContext
IScriptDefinition
```

---

## SCRIPT-1 — Script Discovery

```text
manifest
scanner
catalog
```

---

## SCRIPT-2 — Basic Runtime

Primeiro script:

```text
hello.nx
```

faz:

```text
log("Hello NEXORA")
```

---

## SCRIPT-3 — Lifecycle

```text
load
initialize
start
stop
unload
```

---

## SCRIPT-4 — Event Binding

```text
subscribe
receive event
execute callback
```

---

## SCRIPT-5 — World Query

```text
world.getBlock()
world.getBiome()
world.getTime()
```

---

## SCRIPT-6 — Commands

```text
script
 ↓
command
 ↓
simulation
```

---

## SCRIPT-7 — Permissions

```text
permission
capability
denial
```

---

## SCRIPT-8 — Scheduler

```text
delay
repeat
cancel
```

---

## SCRIPT-9 — Persistence

```text
script state
save
load
migration
```

---

## SCRIPT-10 — Entity Integration

```text
Entity
 ↓
Script Behavior
```

---

## SCRIPT-11 — Mod Runtime Integration

```text
Mod
 ↓
Script
 ↓
API
```

---

## SCRIPT-12 — Networking

```text
script
 ↓
network message
```

---

## SCRIPT-13 — UI

```text
script
 ↓
UI behavior
```

---

## SCRIPT-14 — Profiling

```text
CPU
Memory
Events
Tasks
```

---

## SCRIPT-15 — Isolation

```text
limits
sandbox
quotas
```

---

## SCRIPT-16 — Hot Reload

```text
edit
 ↓
reload
 ↓
restore
```

---

## SCRIPT-17 — Advanced Runtime

Adicionar adapters de linguagem adicionais.

---

# 107. Golden Script Test

O CI deve executar:

```text
START SERVER
      ↓
LOAD MOD
      ↓
LOAD SCRIPT
      ↓
REGISTER EVENT
      ↓
CREATE WORLD
      ↓
TRIGGER EVENT
      ↓
SCRIPT EXECUTES
      ↓
SCRIPT QUERIES WORLD
      ↓
SCRIPT ISSUES COMMAND
      ↓
WORLD CHANGES
      ↓
SAVE
      ↓
RESTART
      ↓
LOAD SCRIPT STATE
      ↓
VERIFY
```

---

# 108. Teste de isolamento

Executar:

```text
while true {}
```

e garantir:

```text
server continues
script terminated
error recorded
```

---

# 109. Teste de autoridade

Cliente tenta:

```text
client script
 ↓
modify inventory directly
```

Resultado:

```text
DENIED
```

Cliente precisa:

```text
request
 ↓
server
 ↓
validation
 ↓
simulation
```

---

# 110. Teste de unload

```text
Load Mod
 ↓
Script
 ↓
Events
 ↓
Tasks
 ↓
Network
 ↓
Unload Mod
```

Depois verificar:

```text
0 orphan events
0 orphan tasks
0 orphan channels
0 orphan handles
```

---

# 111. Arquitetura final

```text
                         NEXORA
                            │
                           CORE
                            │
                     PUBLIC APIs
                            │
            ┌───────────────┴──────────────┐
            ▼                              ▼
        MOD RUNTIME                    SCRIPTING
            │                              │
     ┌──────┼──────┐               ┌──────┼──────┐
     ▼      ▼      ▼               ▼      ▼      ▼
   LOADER  MOD   SERVICES        VM     BINDINGS SECURITY
            │                              │
            └──────────────┬───────────────┘
                           ▼
                        SCRIPTS
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
           EVENTS       QUERIES      COMMANDS
              │            │            │
              ▼            ▼            ▼
          EVENT BUS      WORLD       SERVER
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
      ENTITY             BLOCK              ITEM
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
                       SIMULATION
```

E a regra definitiva:

```text
CORE
→ fornece infraestrutura

PUBLIC API
→ define o que extensões podem usar

MOD RUNTIME
→ gerencia módulos

SCRIPTING
→ executa comportamento extensível

SCRIPT
→ expressa comportamento

COMMAND
→ solicita mudança

SYSTEM
→ executa a mudança

EVENT BUS
→ informa o resultado

SERVER
→ mantém autoridade

PERSISTENCE
→ mantém o estado
```

## Como isso encaixa no NEXORA

A fundação agora está formando uma camada de extensibilidade muito clara:

```text
                    NEXORA FOUNDATION
                           │
      ┌────────────────────┼────────────────────┐
      ▼                    ▼                    ▼
   REGISTRY             EVENT BUS          PERSISTENCE
      │                    │                    │
      └────────────────────┼────────────────────┘
                           ▼
                      PUBLIC APIs
                           │
              ┌────────────┴────────────┐
              ▼                         ▼
         MOD RUNTIME                SCRIPTING
              │                         │
       ┌──────┼──────┐                  │
       ▼      ▼      ▼                  ▼
     MOD A  MOD B   MOD C            SCRIPT
       │      │      │                  │
       └──────┴──────┴──────────┬───────┘
                                ▼
                              SERVER
                                │
                            SIMULATION
                                │
        ┌──────────┬────────────┼────────────┬──────────┐
        ▼          ▼            ▼            ▼          ▼
      WORLD      ENTITY       ITEM         PHYSICS    AI
```

A decisão arquitetural mais importante aqui é **não transformar Scripting em um “segundo Core”**. O script precisa ser poderoso o bastante para construir sistemas complexos, mas sempre através das fronteiras públicas do NEXORA.

Com isso, o próximo componente natural é o **Command System**, porque agora existe o caminho completo:

```text
CLIENT / SCRIPT / ADMIN
          ↓
       COMMAND
          ↓
        SERVER
          ↓
      VALIDATION
          ↓
      SPECIALIZED SYSTEM
          ↓
       STATE CHANGE
          ↓
       EVENT BUS
          ↓
 NETWORKING + PERSISTENCE + OTHER SYSTEMS
```

Esse é o mecanismo que começa a ligar praticamente toda a arquitetura do NEXORA.
