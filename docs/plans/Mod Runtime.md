# NEXORA — MOD RUNTIME

> **Princípio central:**
> **Mods são cidadãos de primeira classe do NEXORA.**
>
> O jogo oficial não deve possuir uma API “especial” que mods não conseguem usar.
> O conteúdo oficial deve utilizar as mesmas APIs públicas de Registry, Event Bus, Entity, Block, Item, WorldGen, UI, Audio, Animation, Networking etc.

A arquitetura-base fica:

```text id="mod01"
NEXORA CORE
      │
      ▼
PUBLIC APIs
      │
      ▼
REGISTRY / EVENTS / DATA
      │
 ┌────┴─────┐
 ▼          ▼
VANILLA    MODS
CONTENT    CONTENT
```

E o **Mod Runtime** fica entre o Core/API e os módulos carregados:

```text id="mod02"
                    NEXORA
                       │
                     CORE
                       │
                PUBLIC API LAYER
                       │
                MOD RUNTIME
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
     LOADER        SANDBOX        SERVICES
        │              │              │
        └──────────────┼──────────────┘
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
        MOD A        MOD B        MOD C
```

---

# 1. O que é o Mod Runtime?

O Mod Runtime é a infraestrutura que permite:

```text
descobrir mods
↓
validar mods
↓
resolver dependências
↓
carregar mods
↓
registrar conteúdo
↓
inicializar código
↓
executar
↓
monitorar
↓
desativar
↓
descarregar
```

Ele não deve implementar a mecânica de cada mod.

Por exemplo:

```text
Mod Runtime ≠ sistema de máquinas
Mod Runtime ≠ sistema de biomas
Mod Runtime ≠ sistema de magia
Mod Runtime ≠ sistema de armas
```

Ele fornece o ambiente para que esses sistemas existam como módulos.

---

# 2. Filosofia

O NEXORA precisa evitar esse modelo:

```text id="mod03"
CORE
├── Vanilla
├── Mod A
├── Mod B
├── Mod C
└── Mod D
```

Porque isso cria dependências dentro do Core.

Queremos:

```text id="mod04"
CORE
   │
   ├── PUBLIC APIS
   │
   └── MOD RUNTIME
            │
     ┌──────┼──────┐
     ▼      ▼      ▼
   MOD A  MOD B  MOD C
```

Assim:

```text
adicionar mod
≠
alterar Core
```

---

# 3. Objetivos

O Mod Runtime precisa resolver:

```text id="mod05"
Mod Discovery
Mod Manifest
Dependency Resolution
Version Compatibility
Loading
Initialization
Lifecycle
Isolation
Permissions
Registries
Events
Resources
Assets
Data
Scripting
Networking
Persistence
Hot Reload
Diagnostics
Error Handling
Security
Performance
```

---

# 4. Mod ≠ Content

Esse é um dos conceitos mais importantes.

Um mod é um **pacote executável/declarativo que pode fornecer conteúdo, comportamento ou ambos**.

Um mod pode conter:

```text id="mod06"
Blocks
Items
Entities
Biomes
Structures
Recipes
Machines
Sounds
Animations
UI
Quests
Dimensions
Scripts
Data
Shaders
Particles
Languages
Textures
Models
Networking
```

Mas o conteúdo continua pertencendo aos sistemas especializados.

Por exemplo:

```text
Mod
 ↓
Block Definition
 ↓
Block Registry
```

e não:

```text
Mod Runtime
 ↓
"Block"
```

---

# 5. Tipos de Mods

Podemos definir categorias:

```text id="mod07"
CONTENT
SYSTEM
LIBRARY
WORLDGEN
CLIENT
SERVER
GAMEPLAY
UI
COSMETIC
AUDIO
SCRIPT
INTEGRATION
TOOLS
TOTAL CONVERSION
```

Mas essas categorias devem ser classificações, não necessariamente arquiteturas completamente diferentes.

---

# 6. Mod Manifest

Cada mod precisa de um manifesto.

Exemplo conceitual:

```yaml id="mod08"
id: example:industrial_expansion
name: Industrial Expansion
version: 1.4.0

game:
  min: 0.6.0
  max: <1.0.0

runtime:
  api: 7

dependencies:
  - nexora:core >= 0.6.0
  - nexora:energy >= 2.0.0

optionalDependencies:
  - example:vehicles >= 1.2.0

entrypoints:
  server: example.server.Main
  client: example.client.Main

side:
  - server
  - client

permissions:
  - registry.register
  - event.subscribe
```

---

# 7. Mod Identity

Precisamos separar:

```text id="mod09"
ModID
ModVersion
ModInstanceID
Namespace
```

Identidade pública:

```text
namespace:mod_id
```

Exemplo:

```text
example:industrial_expansion
```

Nunca depender somente de nome amigável.

---

# 8. Namespace

Cada mod recebe namespace.

```text id="mod10"
nexora:
example:
anothermod:
```

Isso evita:

```text
item:hammer
```

conflitando com:

```text
another_mod:hammer
```

O padrão:

```text
namespace:id
```

vale para:

```text
items
blocks
entities
biomes
recipes
sounds
dimensions
structures
```

etc.

---

# 9. Discovery

O Loader precisa encontrar mods.

Fontes possíveis:

```text id="mod11"
mods/
local repository
development environment
server package
client package
embedded packages
future remote repository
```

Pipeline:

```text
filesystem
 ↓
candidate
 ↓
manifest detection
 ↓
signature/hash verification
 ↓
manifest parsing
```

---

# 10. Mod Package

Uma estrutura possível:

```text id="mod12"
industrial-expansion/
├── mod.yaml
├── code/
├── assets/
├── data/
├── scripts/
├── config/
└── licenses/
```

O formato exato do pacote deve ser definido depois.

O Runtime não deve assumir que mods são necessariamente ZIP, JAR ou outro formato.

---

# 11. Validation

Antes de carregar:

```text id="mod13"
Manifest
 ↓
Schema Validation
 ↓
ID Validation
 ↓
Version Validation
 ↓
Dependency Validation
 ↓
Permission Validation
 ↓
Entrypoint Validation
 ↓
Package Integrity
```

Falhou?

```text
REJECT
```

Não carregar parcialmente sem política explícita.

---

# 12. Dependências

Exemplo:

```text id="mod14"
Industrial Expansion
       │
       ├── Energy API
       ├── Fluid API
       └── Core Library
```

O Runtime monta um grafo:

```text id="mod15"
Core
 ├── Energy
 │    └── Industrial
 └── Fluid
      └── Industrial
```

---

# 13. Dependency Graph

Precisamos detectar:

```text id="mod16"
missing dependency
version mismatch
circular dependency
duplicate module
conflicting module
optional dependency absence
```

Exemplo inválido:

```text
A → B
B → C
C → A
```

Resultado:

```text
DEPENDENCY CYCLE
```

---

# 14. Load Order

A ordem será derivada do grafo.

```text id="mod17"
Core Library
     ↓
Energy
     ↓
Fluid
     ↓
Industrial
```

Não devemos confiar em:

```text
alfabético
timestamp
ordem do diretório
```

---

# 15. Lifecycle

O ciclo completo:

```text id="mod18"
DISCOVERED
↓
VALIDATED
↓
RESOLVED
↓
LOADED
↓
INITIALIZED
↓
REGISTERING
↓
READY
↓
RUNNING
↓
STOPPING
↓
STOPPED
↓
UNLOADED
```

Possível falha:

```text
FAILED
QUARANTINED
```

---

# 16. Fases do Mod

Uma separação útil:

### Bootstrap

```text
preload
```

### Registration

```text
registerContent
```

### Initialization

```text
initialize
```

### Runtime

```text
tick/events/services
```

### Shutdown

```text
stop
cleanup
```

---

# 17. Registry Integration

Mods devem registrar:

```text id="mod19"
Item
Block
Entity
Biome
Fluid
Machine
Recipe
Structure
Dimension
Sound
Particle
Animation
Component
Capability
```

através do:

```text
Registry System
```

Exemplo:

```text id="mod20"
Mod
 ↓
RegistrationContext
 ↓
ItemRegistry
 ↓
example:copper_wire
```

---

# 18. Registro oficial = registro de mod

Vanilla deve seguir o mesmo caminho:

```text id="mod21"
NEXORA CONTENT
        ↓
Registration API
        ↓
Registry System
```

E:

```text
COMMUNITY MOD
        ↓
Registration API
        ↓
Registry System
```

O principal privilégio do vanilla é **estar dentro da distribuição oficial**, não possuir uma API secreta.

---

# 19. RegistrationContext

Um mod não deveria receber acesso irrestrito ao mundo inteiro.

Ele recebe um contexto:

```text id="mod22"
RegistrationContext
├── ModID
├── Logger
├── RegistryAccess
├── EventAccess
├── ConfigAccess
├── ResourceAccess
├── PermissionAccess
└── Services
```

---

# 20. Runtime Context

Durante execução:

```text id="mod23"
ModRuntimeContext
├── ModID
├── Services
├── Scheduler
├── Registry
├── Events
├── Configuration
├── Storage
├── Networking
├── Commands
└── Permissions
```

---

# 21. Mod API Surface

Não expor:

```text id="mod24"
raw pointers everywhere
internal memory structures
private engine state
```

Expor:

```text
interfaces
handles
snapshots
commands
queries
events
capabilities
registries
services
```

Isso preserva compatibilidade.

---

# 22. Capability-Based Access

Um mod pode receber capabilities específicas:

```text id="mod25"
RegistryCapability
EventCapability
NetworkCapability
WorldQueryCapability
PersistenceCapability
CommandCapability
UICapability
AudioCapability
```

Assim:

```text
mod cosmetic
```

não precisa:

```text
WorldWrite
```

---

# 23. Permissions

Modelo:

```text id="mod26"
Permission
```

Exemplo:

```text
registry.read
registry.register
world.read
world.write
entity.spawn
network.register
filesystem.read
filesystem.write
command.register
```

---

# 24. Principle of Least Privilege

O mod recebe somente o que precisa.

Exemplo:

```text id="mod27"
UI Mod
```

pode receber:

```text
UI
Localization
Audio
Events
```

mas não necessariamente:

```text
raw world storage
```

---

# 25. Sandboxing

O nível de sandbox depende da tecnologia usada para mods.

Precisamos projetar uma camada:

```text id="mod28"
MOD
 ↓
RUNTIME API
 ↓
SANDBOX / HOST
 ↓
ENGINE
```

Nunca:

```text
MOD
 ↓
ARBITRARY EVERYTHING
```

---

# 26. Tipos de execução

Poderíamos permitir múltiplos modelos:

```text id="mod29"
Native
Managed
Scripted
Data-driven
WASM
```

Não precisamos implementar todos inicialmente.

Mas a arquitetura deve permitir adapters.

---

# 27. Scripting Runtime

Isso prepara o sistema para:

```text id="mod30"
Lua
JavaScript
WASM
NEXORA Script
```

Mas o Mod Runtime não deve depender de uma linguagem única.

Futuro:

```text
Mod
 ↓
Script Adapter
 ↓
Public API
```

---

# 28. Native Modules

Mods nativos podem existir.

Mas devem ser considerados de maior privilégio:

```text id="mod31"
Native
→ high trust
```

e:

```text
Script/WASM
→ constrained
```

Isso permite escolher conforme o tipo de mod.

---

# 29. Client / Server Side

Mods podem declarar:

```text id="mod32"
CLIENT
SERVER
BOTH
```

Um mod server-only:

```text
server
```

não deve precisar estar no cliente caso não forneça algo necessário para o cliente.

Um mod client-only:

```text
UI
Audio
Cosmetics
```

pode existir sem server logic.

---

# 30. Mod Compatibility

Handshake:

```text id="mod33"
Client Modpack
        ↓
Server Modpack
        ↓
Compare
```

Possíveis resultados:

```text
MATCH
MISSING_CLIENT_MOD
MISSING_SERVER_MOD
VERSION_MISMATCH
API_MISMATCH
OPTIONAL_DIFFERENCE
INCOMPATIBLE
```

---

# 31. Mod Fingerprint

Criar:

```text id="mod34"
ModpackFingerprint
```

gerado usando dados como:

```text
ModID
Version
Content Schema
Dependency versions
Runtime API version
```

Isso permite verificar compatibilidade.

---

# 32. Content Fingerprint

Também podemos criar:

```text id="mod35"
ContentRegistryFingerprint
```

Se:

```text
Client Registry != Server Registry
```

o handshake detecta antes de começar a jogar.

---

# 33. Missing Mod

Se um save contém:

```text id="mod36"
example:quantum_reactor
```

e o mod não está instalado:

não apagar silenciosamente.

Usar:

```text
MissingContentEnvelope
```

para preservar:

```text
namespace
id
version
raw data
```

até o conteúdo poder ser restaurado.

---

# 34. Mod Removal

Não permitir simplesmente:

```text
delete mod folder
```

e assumir que o mundo continua intacto.

Precisamos de:

```text id="mod37"
dependency scan
save compatibility check
content usage scan
migration policy
```

---

# 35. Mod Migration

Versão:

```text
1.0
```

para:

```text
2.0
```

pode exigir:

```text
Migration
```

Fluxo:

```text id="mod38"
Old Mod Data
 ↓
Migration
 ↓
New Schema
 ↓
Validation
 ↓
Commit
```

---

# 36. Save Ownership

Dados do mod devem possuir namespace.

```text id="mod39"
world/moddata/example/
```

conceitualmente.

O Persistence System continua responsável pelo formato físico.

---

# 37. Mod Data Storage

API:

```text id="mod40"
IModStorage
```

deve fornecer operações como:

```text
get
put
remove
exists
transaction
```

Mas com limites.

---

# 38. Quotas

Um mod não deve poder ocupar espaço infinito.

Limites:

```text id="mod41"
Storage
Memory
CPU
Events
Network
Entities
Tasks
Logs
```

Isso é extremamente importante.

---

# 39. CPU Budget

Cada mod pode receber métricas:

```text id="mod42"
CPU time
callbacks
tasks
events
tick time
```

Se um mod consumir demais:

```text
warning
 ↓
throttle
 ↓
degrade
 ↓
quarantine
```

dependendo da política.

---

# 40. Event Bus Integration

Mods podem:

```text
subscribe
publish allowed events
```

Exemplo:

```text id="mod43"
BlockBrokenEvent
 ↓
Mod listener
 ↓
Custom reaction
```

Mas o Event Bus deve respeitar:

```text
permissions
event scope
quotas
subscription lifecycle
```

---

# 41. Mod unload

Quando um mod é removido:

```text id="mod44"
STOP CALLBACKS
 ↓
UNSUBSCRIBE EVENTS
 ↓
CANCEL TASKS
 ↓
CLOSE NETWORK CHANNELS
 ↓
RELEASE RESOURCES
 ↓
SAVE MOD STATE
 ↓
UNREGISTER TEMPORARY SERVICES
 ↓
UNLOAD
```

---

# 42. Event Subscription Ownership

Cada subscription precisa saber:

```text id="mod45"
quem a criou
```

Exemplo:

```text
Subscription
└── owner = example:industrial
```

Assim o Runtime pode limpar tudo no unload.

Evita:

```text dangling listeners
```

---

# 43. Task Ownership

Mesma ideia:

```text id="mod46"
Task
└── owner = mod_id
```

Unload:

```text
cancel all mod-owned tasks
```

---

# 44. Resource Ownership

Tudo deve possuir ownership quando necessário:

```text id="mod47"
Event
Task
Network Channel
Registry Entry
UI Screen
Audio Source Definition
Storage Handle
```

Isso simplifica cleanup.

---

# 45. Hot Reload

É tentador permitir:

```text
edit mod
↓
reload
```

Mas nem tudo pode ser hot reload.

Devemos separar:

```text id="mod48"
SAFE RELOAD
PARTIAL RELOAD
RESTART REQUIRED
```

---

# 46. Safe Reload

Algumas coisas podem ser recarregadas:

```text
textures
localization
sounds
data definitions
recipes
UI definitions
```

---

# 47. Runtime Code Reload

Código executável é mais difícil.

Pode exigir:

```text
restart module
server restart
```

Não assumir hot reload universal.

---

# 48. Development Mode

O Runtime deve possuir:

```text
DEV_MODE
```

com ferramentas extras:

```text
hot reload
logging
profiling
mod inspector
registry reload
resource reload
```

Mas isso não pode virar comportamento obrigatório em produção.

---

# 49. Mod Config

Cada mod pode ter:

```text id="mod49"
config schema
default config
server config
client config
world config
```

Exemplo:

```yaml
machines:
  processing_speed: 1.0
  energy_efficiency: 0.85
```

---

# 50. Config Lifecycle

```text id="mod50"
Schema Load
 ↓
Defaults
 ↓
User Config
 ↓
Validation
 ↓
Migration
 ↓
Runtime Config
```

---

# 51. Resource System

Mods precisam registrar recursos:

```text id="mod51"
Textures
Models
Animations
Sounds
Particles
Shaders
Fonts
Localization
UI Assets
```

Mas o Asset/Resource System deve possuir a lógica de carregamento.

---

# 52. Resource Registry

Exemplo:

```text id="mod52"
example:machine_idle
```

apontando para:

```text
assets/example/animations/machine_idle...
```

---

# 53. Data-driven Mods

Um mod pode não precisar de código.

Exemplo:

```text id="mod53"
JSON/YAML
 ↓
BlockDefinition
 ↓
Registry
```

Isso permite:

```text
content packs
```

muito mais simples.

---

# 54. Mod Libraries

Podemos ter mods que não adicionam conteúdo ao mundo.

Exemplo:

```text id="mod54"
example:technology-api
```

somente fornece:

```text
API
utilities
framework
shared components
```

Outros mods dependem dela.

---

# 55. Mod Categories

Podemos usar tags:

```text
library
content
gameplay
worldgen
server
client
ui
audio
script
integration
```

para descoberta e diagnóstico.

---

# 56. Mod API Versioning

A API pública precisa ser versionada.

```text id="mod57"
Mod API 1
Mod API 2
Mod API 3
```

Não fazer:

```text
game version = API version
```

São coisas diferentes.

---

# 57. Compatibility Adapters

Podemos suportar:

```text id="mod58"
API v5
 ↓
Compatibility Adapter
 ↓
API v6
```

mas isso deve ser controlado.

---

# 58. Deprecation

API antiga:

```text
registerThing()
```

pode virar:

```text
registerCapability()
```

Primeiro:

```text
deprecated
```

depois:

```text
removed
```

com ferramenta de diagnóstico.

---

# 59. Mod Error Handling

Erro:

```text
Mod throws exception
```

não deve automaticamente significar:

```text
server crash
```

O Runtime captura em fronteiras apropriadas.

```text id="mod59"
MOD
 ↓
BOUNDARY
 ↓
ERROR
 ↓
LOG
 ↓
METRIC
 ↓
ISOLATE
```

---

# 60. Quarantine

Um mod que falha repetidamente pode entrar:

```text id="mod60"
QUARANTINED
```

Com:

```text
ModID
Reason
Crash count
Last error
Compatibility data
```

---

# 61. Crash Report

Cada mod deve aparecer claramente em crashes:

```text id="mod61"
NEXORA Crash

Suspected Module:
example:industrial

API:
7

Version:
1.4.0

Last Event:
MachineCompleted

Last Callback:
onMachineComplete()
```

Sem atribuir culpa automaticamente quando a evidência não for suficiente.

---

# 62. Mod Diagnostics

Comandos:

```text id="mod62"
nexora mod list
nexora mod inspect <id>
nexora mod dependencies <id>
nexora mod graph
nexora mod permissions <id>
nexora mod resources <id>
nexora mod memory <id>
nexora mod performance <id>
nexora mod events <id>
nexora mod reload <id>
nexora mod disable <id>
```

---

# 63. Mod Dependency Graph UI

Idealmente:

```text id="mod63"
Core
│
├── Energy API
│   ├── Machines
│   └── Industrial
│
├── Fluid API
│   └── Chemical
│
└── WorldGen Library
    └── Alien Worlds
```

Isso facilita debugging.

---

# 64. Mod Networking

O Mod Runtime conversa com Networking:

```text id="mod64"
Mod
 ↓
Network Channel Registry
 ↓
Networking
```

O mod pode registrar:

```text
channel
message
serializer
handler
replication
```

com quotas.

---

# 65. Mod UI

Mesmo princípio:

```text id="mod65"
Mod
 ↓
UI Registry
 ↓
Screen
Widget
HUD
```

O mod não precisa acessar diretamente o renderer.

---

# 66. Mod Commands

Mod pode registrar:

```text id="mod66"
example:machine
example:research
example:debug
```

através do Command System.

---

# 67. Mod Entities

```text id="mod67"
Mod
 ↓
Entity Registry
 ↓
Entity Definition
 ↓
Components
 ↓
AI Profile
 ↓
Spawn Rules
```

E o mod não precisa modificar o Entity System.

---

# 68. Mod Biomes

```text id="mod68"
Mod
 ↓
Biome Registry
 ↓
Biome Definition
 ↓
WorldGen
```

O World Generator simplesmente enxerga o conteúdo registrado.

---

# 69. Mod Dimensions

```text id="mod69"
Mod
 ↓
Dimension Registry
 ↓
Dimension Definition
 ↓
Dimension System
```

O Server hospeda a dimensão.

---

# 70. Mod Structures

```text id="mod70"
Mod
 ↓
Structure Registry
 ↓
Structure Definition
 ↓
WorldGen
```

---

# 71. Mod Machines

```text id="mod71"
Mod
 ↓
Machine Registry
 ↓
Machine Definition
 ↓
Energy / Fluid / Crafting
```

---

# 72. Mod Integration

Um mod pode integrar sistemas:

```text id="mod72"
Industrial Mod
├── Energy
├── Fluid
├── Machine
├── Item
├── Crafting
├── UI
├── Audio
└── Networking
```

Sem alterar esses sistemas.

---

# 73. Public API Principle

Se o mod precisa acessar alguma função:

```text
não copie um hack interno
```

Crie uma API pública apropriada.

Isso é fundamental para a saúde do projeto.

---

# 74. Official Content como Mods

Uma arquitetura ainda mais poderosa:

```text id="mod73"
NEXORA
│
├── Core APIs
│
├── Mod Runtime
│
└── Official Modules
      ├── Base Content
      ├── Technology
      ├── Vehicles
      ├── Space
      ├── Magic
      └── ...
```

Ou seja:

```text
OFFICIAL MOD
```

não precisa significar um formato separado.

---

# 75. Mas Core continua especial

Não devemos forçar:

```text
Core
```

a ser um mod comum.

Porque Core fornece:

```text
memory
threads
registries
event bus
storage primitives
runtime
```

Então:

```text
Core ≠ Mod
```

Mas:

```text
Official Content = Modules using public APIs
```

---

# 76. Mod Security Boundary

O Runtime deve considerar:

```text
trusted
semi-trusted
sandboxed
untrusted data-only
```

Exemplo:

```text
Data-only mod
→ low risk

WASM mod
→ controlled

Native mod
→ high trust
```

---

# 77. Determinism

Mods que participam de simulação devem seguir regras:

```text id="mod74"
deterministic APIs where required
no uncontrolled randomness
no direct wall-clock dependence
no unsafe threading
```

RNG deve ser fornecido pelo Runtime quando apropriado:

```text
ModRandom
```

com seed/context.

---

# 78. Threading para Mods

Mods não devem simplesmente criar:

```text
∞ threads
```

O Runtime deve oferecer:

```text id="mod75"
TaskScheduler
WorkerPool
Async API
```

com limites.

---

# 79. World Access

Um dos pontos mais perigosos.

Não dar:

```text
raw world memory
```

Dar:

```text
WorldQuery
WorldCommand
WorldSnapshot
```

Exemplo:

```text
query block
query entities
request placement
schedule world change
```

---

# 80. Thread-safe World Access

Worker thread:

```text
query snapshot
```

não:

```text
change world arbitrarily
```

Mudanças passam por:

```text
Simulation Command Queue
```

---

# 81. Mod Event Timing

Um mod não deve poder alterar a ordem temporal de maneira imprevisível.

Eventos possuem:

```text
phase
priority
scope
```

seguindo as regras do Event Bus.

---

# 82. Mod Persistence

O mod pode possuir:

```text
persistent data
```

mas Persistence continua responsável por:

```text
versioning
checksum
atomicity
recovery
migration
```

---

# 83. Mod + Save Compatibility

Antes de abrir o mundo:

```text
Save
 ↓
Required Mods
 ↓
Installed Mods
 ↓
Version Check
 ↓
Migration Check
 ↓
Compatibility
```

---

# 84. World Mod Lock

Depois que um mundo entra em produção, podemos armazenar:

```text
World Mod Manifest
```

contendo:

```text
mod IDs
versions
API versions
content fingerprint
registry fingerprint
```

Isso evita abrir o save silenciosamente com uma configuração completamente diferente.

---

# 85. Add / Remove Mod Workflow

Adicionar:

```text id="mod76"
install
↓
dependency resolution
↓
content validation
↓
registry
↓
world compatibility
↓
load
```

Remover:

```text
disable
↓
scan content usage
↓
migration/fallback
↓
save
↓
unload
```

---

# 86. Mod Version Update

```text id="mod77"
1.2
 ↓
Migration
 ↓
2.0
 ↓
Registry Migration
 ↓
Save Migration
 ↓
Runtime Start
```

Tudo deve ser explícito.

---

# 87. Mod Resources and Licensing

O Runtime deve permitir metadados:

```text id="mod78"
license
authors
homepage
source
dependencies
attribution
```

Isso ajuda a manter o ecossistema organizado.

---

# 88. Content Provenance

Itens/blocks/entidades podem registrar:

```text id="mod79"
originMod
originVersion
definitionVersion
```

Isso ajuda:

```text debugging
save migration
missing content
compatibility
```

---

# 89. Mod Registry

Podemos ter:

```text id="mod80"
ModRegistry
```

com:

```text ModID
Version
State
Dependencies
Permissions
Entrypoints
Ownership
Fingerprint
```

---

# 90. APIs públicas

```text id="mod81"
IModRuntime
IModLoader
IModManager
IModContainer
IModManifest
IModDependencyResolver
IModLifecycle
IModPermissionManager
IModSandbox
IModRegistry
IModResourceManager
IModStorage
IModScheduler
IModDiagnostics
IModCompatibility
IModMigration
IModNetworkBridge
```

---

# 91. Organização de código

```text id="mod82"
src/mod/

├── core/
│   ├── mod-runtime
│   ├── mod-context
│   ├── mod-state
│   └── mod-config
│
├── manifest/
│   ├── manifest
│   ├── schema
│   └── parser
│
├── discovery/
│   ├── discovery
│   ├── package-scanner
│   └── candidate
│
├── validation/
│   ├── validator
│   ├── integrity
│   └── compatibility
│
├── dependency/
│   ├── graph
│   ├── resolver
│   └── version
│
├── loading/
│   ├── loader
│   ├── class/module-loader
│   └── resource-loader
│
├── lifecycle/
│   ├── lifecycle
│   ├── initialization
│   ├── shutdown
│   └── unload
│
├── permissions/
│   ├── permissions
│   ├── capabilities
│   └── policies
│
├── sandbox/
│   ├── sandbox
│   ├── native
│   ├── script
│   └── wasm
│
├── registry/
│   └── mod-registration
│
├── events/
│   └── mod-events
│
├── resources/
│   ├── assets
│   ├── data
│   └── localization
│
├── persistence/
│   ├── mod-storage
│   └── migration
│
├── networking/
│   └── mod-network
│
├── scheduler/
│
├── diagnostics/
│
└── debug/
```

---

# 92. Dependências

```text id="mod83"
CORE
├── Registry
├── Event Bus
├── Persistence
└── Public APIs
       │
       ▼
   MOD RUNTIME
       │
       ├── Server
       ├── Networking
       ├── UI
       ├── World
       └── Simulation
```

O Server hospeda o Runtime.

O Runtime usa APIs públicas.

---

# 93. Boot completo do NEXORA

```text id="mod84"
PROCESS START
       ↓
CORE
       ↓
REGISTRY
       ↓
EVENT BUS
       ↓
PERSISTENCE
       ↓
MOD RUNTIME
       ↓
DISCOVER MODS
       ↓
VALIDATE
       ↓
RESOLVE DEPENDENCIES
       ↓
LOAD MODS
       ↓
REGISTER CONTENT
       ↓
VALIDATE REGISTRIES
       ↓
FREEZE REGISTRIES
       ↓
LOAD WORLD
       ↓
START SERVER
       ↓
RUN
```

---

# 94. Runtime Loop

Durante execução:

```text id="mod85"
SERVER TICK
    │
    ├── Input
    ├── Commands
    ├── Simulation
    ├── Event Bus
    ├── Mod Tasks
    ├── Persistence
    └── Replication
```

Mods entram por APIs controladas.

---

# 95. Implementação por fases

## MOD-0 — Core Contracts

Criar:

```text
IModRuntime
IMod
IModContext
IModManifest
```

---

## MOD-1 — Manifest

```text
parse
validate
version
dependencies
```

---

## MOD-2 — Discovery

```text
scan
identify
catalog
```

---

## MOD-3 — Dependency Graph

```text
graph
topological sort
cycle detection
version constraints
```

---

## MOD-4 — Basic Loader

Primeiro mod:

```text
hello-world
```

carrega e executa:

```text
onInitialize()
```

---

## MOD-5 — Lifecycle

Implementar:

```text
initialize
start
stop
unload
```

---

## MOD-6 — Registry Integration

Primeiro conteúdo:

```text
mod
 ↓
register Item
 ↓
spawn item
```

---

## MOD-7 — Event Bus

```text
mod subscribes event
↓
event fires
↓
mod responds
```

---

## MOD-8 — Resources

```text
texture
sound
language
data
```

---

## MOD-9 — Config

```text
schema
defaults
user config
migration
```

---

## MOD-10 — Permissions

```text
capabilities
permissions
denial
audit
```

---

## MOD-11 — Persistence

```text
mod storage
save
load
migration
```

---

## MOD-12 — Server Integration

```text
Server
 ↓
Mod Runtime
 ↓
Mods
```

---

## MOD-13 — Networking

```text
Mod
 ↓
Custom Message
 ↓
Network
```

---

## MOD-14 — UI

```text
Mod
 ↓
Screen
 ↓
Widget
```

---

## MOD-15 — Advanced Isolation

```text
sandbox
quotas
task limits
memory monitoring
```

---

## MOD-16 — Script Adapter

Preparar:

```text
WASM / Script runtime
```

sem acoplar ao Core.

---

## MOD-17 — Hot Reload

Somente recursos seguros inicialmente.

---

## MOD-18 — Migration Framework

```text
mod v1
 ↓
migration
 ↓
mod v2
```

---

## MOD-19 — Diagnostics

```text
mod profiler
dependency graph
permission inspector
resource inspector
```

---

## MOD-20 — Ecosystem Stress Test

Testar:

```text
10 mods
50
100
500+
```

---

# 96. Primeiro Vertical Slice

O primeiro verdadeiro teste deve ser:

```text id="mod86"
NEXORA SERVER
     ↓
MOD RUNTIME
     ↓
LOAD example:test
     ↓
REGISTER example:test_item
     ↓
REGISTRY
     ↓
CREATE ITEM
     ↓
INVENTORY
     ↓
SAVE
     ↓
RESTART
     ↓
LOAD MOD
     ↓
ITEM STILL VALID
```

Esse único teste prova:

```text
Loader
Registry
Item
Persistence
Server
```

---

# 97. Segundo Vertical Slice

```text id="mod87"
MOD
 ↓
REGISTER BLOCK
 ↓
WORLDGEN / WORLD
 ↓
BUILD
 ↓
BREAK
 ↓
LOOT
 ↓
ITEM
 ↓
EVENT
 ↓
NETWORK
```

Isso prova que um mod realmente pode participar do ecossistema inteiro usando APIs públicas.

---

# 98. Terceiro Vertical Slice

Um mod de máquina:

```text id="mod88"
Industrial Mod
 ↓
Machine Registry
 ↓
Machine
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
Networking
 ↓
Persistence
```

Sem modificar o Core.

Esse é um dos melhores testes de arquitetura do NEXORA.

---

# 99. Golden Mod Test

O CI deve executar:

```text id="mod89"
Load Mod
Register Content
Create World
Spawn Content
Use Content
Trigger Event
Save
Unload
Reload
Load Save
Verify
```

Resultado esperado:

```text
STATE BEFORE
=
STATE AFTER
```

---

# 100. Stress Tests

```text id="mod90"
1 mod
10
50
100
500
1000 content definitions
100k event callbacks
large registry
large dependency graph
many assets
many scripts
many tasks
many network messages
```

---

# 101. Fault Injection

Testar:

```text id="mod91"
missing dependency
invalid manifest
cycle
bad version
duplicate ID
invalid resource
broken migration
mod crash
event exception
network overflow
storage overflow
permission violation
```

O Runtime deve:

```text
detect
report
isolate
recover
```

quando possível.

---

# 102. Performance

O Mod Runtime não pode ficar no caminho de tudo.

Não queremos:

```text
Every block access
 ↓
Mod Runtime
 ↓
lookup permissions
```

sempre.

Em vez disso:

```text
Registration time
 ↓
compile/resolve references
 ↓
runtime handles
```

e hot paths usam referências eficientes.

---

# 103. Mod Runtime Cache

Pode existir:

```text
ModCatalog
DependencyCache
ManifestCache
ResourceIndex
CompiledSchemaCache
ScriptCache
```

Mas são derivados e reconstruíveis.

Persistence não depende deles.

---

# 104. Mod Registry vs Registry System

Não confundir:

```text
Mod Registry
```

responde:

> quais mods estão instalados?

Enquanto:

```text
Registry System
```

responde:

> o que significa `example:iron_machine`?

São sistemas relacionados, mas diferentes.

---

# 105. Mod Runtime vs Server

```text id="mod92"
SERVER
→ hospeda o runtime

MOD RUNTIME
→ gerencia módulos

MOD
→ fornece conteúdo/comportamento
```

---

# 106. Mod Runtime vs Scripting

```text id="mod93"
MOD RUNTIME
→ lifecycle e integração

SCRIPTING
→ linguagem/execução de scripts
```

Scripting será outro sistema.

O Runtime apenas fornece o adaptador.

---

# 107. Mod Runtime vs Registry

```text id="mod94"
REGISTRY
→ identidade e definição

MOD RUNTIME
→ quem registra e em qual contexto
```

---

# 108. Mod Runtime vs Persistence

```text id="mod95"
MOD RUNTIME
→ solicita armazenamento

PERSISTENCE
→ garante durabilidade
```

---

# 109. Mod Runtime vs Networking

```text id="mod96"
MOD RUNTIME
→ registra mensagens de mod

NETWORKING
→ transporta mensagens
```

---

# 110. A arquitetura final

```text id="mod97"
                         NEXORA
                            │
                         CORE
                            │
                  ┌─────────┴─────────┐
                  ▼                   ▼
            PUBLIC APIs           REGISTRY
                  │
                  ▼
             MOD RUNTIME
                  │
     ┌────────────┼────────────┐
     ▼            ▼            ▼
  LOADER       SANDBOX      SERVICES
     │            │            │
     └────────────┼────────────┘
                  │
        ┌─────────┼─────────┐
        ▼         ▼         ▼
       MOD A     MOD B     MOD C
        │         │         │
        └─────────┼─────────┘
                  ▼
       ┌─────────────────────┐
       │ PUBLIC NEXORA APIS  │
       └──────────┬──────────┘
                  │
   ┌──────────────┼─────────────────┐
   ▼              ▼                 ▼
 WORLD          ENTITY            ITEM
   │              │                 │
   ▼              ▼                 ▼
 BLOCK         MACHINES          CRAFTING
   │              │                 │
   └──────────────┼─────────────────┘
                  ▼
              EVENT BUS
                  │
       ┌──────────┼──────────┐
       ▼          ▼          ▼
    SERVER    NETWORKING PERSISTENCE
```

E a regra definitiva do NEXORA fica:

```text
CORE
→ fornece as regras fundamentais

PUBLIC APIs
→ expõem capacidades

REGISTRY
→ define identidades

EVENT BUS
→ comunica acontecimentos

MOD RUNTIME
→ hospeda extensões

MOD
→ adiciona conteúdo/comportamento

SERVER
→ fornece autoridade

SIMULATION
→ executa o mundo

PERSISTENCE
→ mantém o estado
```

### O ponto mais importante

A arquitetura do NEXORA agora começa a ficar muito forte:

```text
                    NEXORA FOUNDATION
                           │
       ┌───────────────────┼───────────────────┐
       ▼                   ▼                   ▼
    REGISTRY            EVENT BUS         PERSISTENCE
       │                   │                   │
       └───────────────────┼───────────────────┘
                           ▼
                     PUBLIC APIs
                           │
                           ▼
                      MOD RUNTIME
                           │
                  ┌────────┴────────┐
                  ▼                 ▼
            OFFICIAL CONTENT    COMMUNITY MODS
                  │                 │
                  └────────┬────────┘
                           ▼
                         SERVER
                           │
                       SIMULATION
                           │
                 ┌─────────┼─────────┐
                 ▼         ▼         ▼
               WORLD     ENTITY    GAMEPLAY
```

Com isso, **Vanilla deixa de ser uma coleção de exceções do Core e passa a ser o primeiro grande consumidor das mesmas APIs que os mods usam**. Isso é uma das decisões arquiteturais mais importantes para manter o NEXORA realmente extensível a longo prazo.
