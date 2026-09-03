# NEXORA — ADVANCED INTERACTION SYSTEM

> **Princípio central:**
> **Interaction System transforma intenção contextual em interação com o mundo.**
>
> Ele não é “o sistema de clicar em coisas”.
>
> Ele responde:
>
> **“O que existe aqui, o que pode interagir com o quê, em qual contexto, e qual ação deve ser solicitada?”**

Isso é especialmente importante no NEXORA porque o mundo não será composto apenas de:

```text
bloco
↓
quebrar
```

Teremos:

```text
Player
NPC
Mob
Item
Block
Machine
Vehicle
Structure
Fluid
Energy
Container
Door
Terminal
Computer
Railway
Building
Faction
Quest
Research
Environment
```

e tudo isso poderá possuir diferentes formas de interação.

---

# 1. O problema que o Advanced Interaction resolve

Sem um sistema próprio, cada sistema acaba criando:

```text
Player.interactNPC()
Player.openChest()
Player.useMachine()
Player.enterVehicle()
Player.activateDoor()
Player.talkToNPC()
```

Isso rapidamente vira uma arquitetura difícil de manter.

Queremos:

```text
INPUT
 ↓
INTERACTION DISCOVERY
 ↓
INTERACTION CONTEXT
 ↓
AVAILABLE ACTIONS
 ↓
PLAYER / AI / SCRIPT SELECTS
 ↓
INTERACTION REQUEST
 ↓
COMMAND SYSTEM
 ↓
SPECIALIZED SYSTEM
```

---

# 2. Arquitetura

```text id="int01"
                    INTERACTION SYSTEM
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
     DISCOVERY          CONTEXT            TARGET
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
                       OPTIONS
                           │
                           ▼
                    ACTION SELECTION
                           │
                           ▼
                    INTERACTION REQUEST
                           │
                           ▼
                     COMMAND SYSTEM
                           │
                           ▼
                  SPECIALIZED SYSTEM
                           │
                           ▼
                     STATE CHANGE
                           │
                           ▼
                      EVENT BUS
```

---

# 3. Interaction ≠ Command

Essa distinção é fundamental.

### Interaction

Responde:

> “O que posso fazer com isso?”

### Command

Responde:

> “Quero fazer isso.”

Exemplo:

```text id="int02"
NPC
 ↓
Interaction Discovery
 ↓
Talk
Trade
AskQuest
AskDirection
Hire
```

O jogador escolhe:

```text id="int03"
Trade
```

e isso gera:

```text id="int04"
TradeCommand
```

---

# 4. Interaction ≠ Input

Input é:

```text id="int05"
mouse
keyboard
gamepad
touch
```

Interaction é a interpretação contextual:

```text id="int06"
olhando para uma porta
+
pressionando ação
=
abrir porta
```

---

# 5. Interaction ≠ Gameplay

Interaction descobre e solicita.

Gameplay executa.

```text id="int07"
Interaction
→ "Open"

Command
→ "quero abrir"

Door System
→ decide se abre

Event
→ DoorOpened
```

---

# 6. Entidades interativas

Qualquer coisa pode implementar interação:

```text id="int08"
Player
NPC
Mob
Block
Item
Machine
Vehicle
Structure
Container
Fluid Device
Terminal
Portal
Environment
```

---

# 7. Interaction Provider

A principal abstração:

```text
IInteractionProvider
```

Um objeto pode fornecer ações.

Exemplo:

```text id="int09"
Door
 ├── Open
 ├── Close
 ├── Lock
 └── Inspect
```

---

# 8. Interaction Definition

```text id="int10"
InteractionDefinition
├── InteractionID
├── Name
├── Category
├── Requirements
├── Conditions
├── Targeting
├── Cost
├── Duration
├── Animation
├── Audio
├── Feedback
└── Command
```

---

# 9. Interaction Instance

```text id="int11"
InteractionInstance
├── InstanceID
├── DefinitionID
├── Actor
├── Target
├── Context
├── State
├── StartedAt
├── Progress
└── Metadata
```

---

# 10. Actor

O actor pode ser:

```text id="int12"
Player
NPC
AI
Script
Machine
Vehicle
WorldEvent
```

Isso é muito importante.

Não queremos:

```text id="int13"
Interaction = Player only
```

Queremos:

```text id="int14"
Interaction = Actor ↔ Target
```

---

# 11. Target

Target pode ser:

```text id="int15"
Entity
Block
Item
Structure
Machine
Container
Fluid
World Location
UI Object
```

---

# 12. Context

A mesma ação pode mudar dependendo do contexto.

```text id="int16"
InteractionContext
├── Actor
├── Target
├── Position
├── Direction
├── Distance
├── Dimension
├── Time
├── Environment
├── Equipment
├── Permissions
├── Knowledge
├── QuestState
└── InputContext
```

---

# 13. Contexto é o coração do sistema

Exemplo:

```text id="int17"
Door
```

pode permitir:

```text
Open
```

mas:

```text
Night
+
Security Lock
+
Player without Key
```

pode produzir:

```text
Open = unavailable
```

---

# 14. Interaction Discovery

Antes de executar qualquer coisa:

```text id="int18"
Actor
 ↓
Target Query
 ↓
Providers
 ↓
Conditions
 ↓
Available Interactions
```

Resultado:

```text id="int19"
[
  Open,
  Knock,
  Inspect
]
```

---

# 15. Discovery não executa

Extremamente importante:

```text id="int20"
DISCOVER
≠
EXECUTE
```

Discovery é somente consulta.

---

# 16. Interaction Query

API:

```text id="int21"
IInteractionQuery
```

Exemplo:

```text
getAvailableInteractions(actor, target)
```

retorna ações possíveis.

---

# 17. Raycast / Targeting

Para o jogador:

```text id="int22"
Camera
 ↓
Raycast
 ↓
Target
 ↓
Interaction Query
```

Mas Targeting não precisa ser apenas raycast.

---

# 18. Targeting Methods

```text id="int23"
RAYCAST
AREA
PROXIMITY
TOUCH
LOOK
VOLUME
AOE
TRIGGER
AUTO
CONTEXTUAL
```

---

# 19. Proximity Interaction

NPC:

```text id="int24"
Player
 ↓
within 3m
 ↓
Talk
```

---

# 20. Area Interaction

Uma máquina pode interagir com:

```text id="int25"
all entities inside area
```

---

# 21. Touch Interaction

Veículos:

```text id="int26"
Vehicle
 ↓
touch rail switch
```

---

# 22. Automatic Interaction

Uma machine controller pode usar:

```text id="int27"
auto-pickup
auto-transfer
auto-route
```

sem player input.

---

# 23. Interaction Priority

Pode existir:

```text id="int28"
InteractionPriority
```

Exemplo:

```text
Open Door
Talk to NPC
Inspect
Attack
```

dependendo do contexto.

---

# 24. Ambiguity Resolution

Imagine:

```text id="int29"
crosshair
 ↓
NPC
+
door
+
machine
```

O sistema pode escolher com base em:

```text id="int30"
distance
angle
priority
visibility
target size
actor capabilities
```

---

# 25. Interaction Scoring

Cada possível ação pode receber:

```text id="int31"
score
```

Exemplo:

```text
Open Door = 0.92
Talk NPC = 0.67
Inspect Sign = 0.21
```

Isso é ótimo para:

```text
contextual prompts
gamepad
VR futuro
AI
```

---

# 26. AI usa o mesmo sistema

NPC:

```text id="int32"
NPC sees machine
 ↓
Interaction Discovery
 ↓
Start Machine
```

Player:

```text
Player sees machine
 ↓
Interaction Discovery
 ↓
Start Machine
```

Mesma infraestrutura.

---

# 27. Script usa o mesmo sistema

```text id="int33"
Script
 ↓
query interactions
 ↓
choose interaction
 ↓
command
```

---

# 28. Mod usa o mesmo sistema

Mod pode registrar:

```text id="int34"
example:scan_artifact
```

e adicioná-la a:

```text id="int35"
Artifact
```

---

# 29. Interaction Conditions

Uma interação pode exigir:

```text id="int36"
distance
line of sight
tool
item
skill
technology
knowledge
quest
permission
energy
fluid
state
time
weather
dimension
faction
ownership
```

---

# 30. Requirements

Separar:

```text id="int37"
Requirement
```

de:

```text id="int38"
Condition
```

Requirement pode ser declarado.

Condition pode verificar o estado atual.

---

# 31. Interaction Validator

```text id="int39"
IInteractionValidator
```

verifica:

```text
canInteract?
```

Mas a execução final ainda passa pelo Command/Server.

---

# 32. Client vs Server

No multiplayer:

```text id="int40"
CLIENT
→ discovers

SERVER
→ validates and executes
```

O cliente pode mostrar:

```text
Open
```

mas isso não garante que pode abrir.

---

# 33. Interaction Request

Cliente envia:

```text id="int41"
InteractionRequest
```

contendo:

```text
InteractionID
Target
Context Reference
```

Servidor reconstrói o contexto confiável.

---

# 34. Não confiar no contexto do cliente

Cliente não pode dizer:

```text id="int42"
distance = 0
permission = true
```

Servidor calcula:

```text id="int43"
actual distance
actual state
actual permission
```

---

# 35. Interaction → Command

Fluxo:

```text id="int44"
Interaction
 ↓
Request
 ↓
Validation
 ↓
Command
 ↓
System
```

---

# 36. Long-running Interaction

Algumas interações demoram:

```text id="int45"
Mining
Repair
Crafting
Research
Medical Treatment
Conversation
Construction
Vehicle Boarding
```

Precisamos de uma state machine.

---

# 37. Interaction Lifecycle

```text id="int46"
DISCOVERED
↓
AVAILABLE
↓
REQUESTED
↓
VALIDATING
↓
ACCEPTED
↓
STARTED
↓
IN_PROGRESS
↓
COMPLETED
```

Falhas:

```text id="int47"
REJECTED
CANCELLED
INTERRUPTED
FAILED
EXPIRED
```

---

# 38. Progress

Uma interação pode possuir:

```text id="int48"
progress
duration
remaining
```

Exemplo:

```text
Repair Machine
████████░░ 80%
```

---

# 39. Interruption

Pode interromper por:

```text id="int49"
player moved
target moved
damage
resource lost
permission changed
machine destroyed
world changed
```

---

# 40. Resume

Algumas interações permitem:

```text id="int50"
pause
resume
```

ou possuem progresso persistente.

---

# 41. Interaction Channels

Podemos classificar:

```text id="int51"
PRIMARY
SECONDARY
PASSIVE
AUTOMATIC
SOCIAL
COMMERCE
CONTROL
INSPECTION
```

---

# 42. Primary Interaction

Exemplo:

```text
click
```

→ Open Door

---

# 43. Secondary Interaction

```text
right click
```

→ Inspect

---

# 44. Passive Interaction

Algo que acontece ao entrar em uma região:

```text id="int52"
enter trigger
 ↓
interaction
```

---

# 45. Social Interaction

NPC:

```text id="int53"
Talk
Trade
Ask
Teach
Hire
Follow
Lead
Invite
```

---

# 46. Social Actions

Social interactions podem conectar:

```text id="int54"
NPC
+
Knowledge
+
Quest
+
Reputation
+
Faction
```

---

# 47. Contextual Dialogue

Não colocar diálogo dentro do Interaction Core.

Fluxo:

```text id="int55"
Interact NPC
 ↓
Dialogue System
 ↓
possible conversation
```

---

# 48. Commerce

```text id="int56"
Interact Trader
 ↓
Trade Interaction
 ↓
TradeCommand
 ↓
Economy
```

---

# 49. Machine

```text id="int57"
Interact Machine
 ↓
Start
Stop
Configure
Inspect
Repair
```

A UI pode então abrir uma tela.

---

# 50. Container

```text id="int58"
Open Container
 ↓
Inventory UI
```

Inventory continua pertencendo ao Inventory System.

---

# 51. Vehicle

```text id="int59"
Vehicle
├── Enter
├── Exit
├── Control
├── Inspect
└── Repair
```

Vehicle System executa.

---

# 52. Structure

```text id="int60"
Structure
├── Inspect
├── Enter
├── Repair
├── Upgrade
├── Configure
└── Claim
```

---

# 53. Block Interaction

Um bloco pode fornecer:

```text id="int61"
Open
Use
Toggle
Inspect
Connect
Rotate
Configure
```

Não é necessário transformar todo block em entidade.

---

# 54. Item Interaction

Item pode:

```text id="int62"
Use
Equip
Consume
Place
Throw
Inspect
Repair
Configure
```

Item System fornece capabilities.

---

# 55. Capability Integration

Interaction pode perguntar:

```text id="int63"
Does actor have capability X?
```

Exemplo:

```text
Actor
→ ITool

Target
→ IRepairable

Interaction
→ Repair
```

---

# 56. Generic Capability Interaction

Isso combina muito bem com a arquitetura do NEXORA:

```text id="int64"
ACTOR CAPABILITY
        +
TARGET CAPABILITY
        ↓
INTERACTION
```

---

# 57. Example

```text id="int65"
Player
has:
RepairTool

Machine
has:
Repairable

↓
Repair interaction available
```

---

# 58. Ownership

Interactions podem depender de ownership.

```text id="int66"
Machine
owner = Player A
```

Player B pode:

```text
Inspect
```

mas não:

```text
Configure
```

---

# 59. Permission

Interaction pode usar:

```text id="int67"
PermissionContext
```

mas a autoridade final permanece no Command/System.

---

# 60. Contextual UI

A UI pode receber:

```text id="int68"
InteractionOption
├── label
├── icon
├── input hint
├── availability
├── reason
└── priority
```

e mostrar:

```text
[E] Open
[R] Repair
[F] Inspect
```

---

# 61. Reason for Unavailable

Não apenas esconder.

Pode indicar:

```text id="int69"
"Requires Engineering II"
"Locked"
"Too far away"
"No energy"
"Requires key"
```

Isso vem do Condition/Requirement resolver.

---

# 62. Accessibility

Interaction UI deve suportar:

```text id="int70"
keyboard
gamepad
touch
screen reader
large text
high contrast
alternative input
```

---

# 63. Localization

A Interaction Definition armazena:

```text id="int71"
labelKey
descriptionKey
```

e a Localization System traduz.

---

# 64. Animation

Uma interação pode disparar:

```text id="int72"
InteractionStarted
```

e Animation System responde.

Exemplo:

```text
Open Door
 ↓
animation
```

---

# 65. Audio

Da mesma forma:

```text id="int73"
DoorOpenedEvent
 ↓
Audio System
```

---

# 66. UI

```text id="int74"
Interaction discovered
 ↓
UI prompt
```

---

# 67. Physics

Uma interação pode resultar em:

```text id="int75"
push
pull
grab
attach
detach
```

Physics executa as consequências físicas.

---

# 68. Fluid

Exemplo:

```text id="int76"
Open Valve
```

Interaction:

```text
Open
```

Command:

```text
SetValveState
```

Fluid System altera fluxo.

---

# 69. Energy

```text id="int77"
Activate Generator
```

pode gerar:

```text
EnergyNode state change
```

---

# 70. Crafting

```text id="int78"
Interact Workbench
 ↓
Crafting UI
```

---

# 71. Research

```text id="int79"
Interact Laboratory
 ↓
Research UI
```

---

# 72. Quest

```text id="int80"
Talk NPC
 ↓
Quest/Dialogue systems
```

---

# 73. Progression

```text id="int81"
Interact with ancient device
 ↓
Knowledge discovery
 ↓
Research
```

---

# 74. Advanced Interaction Graph

Uma interação pode produzir outra:

```text id="int82"
Open Terminal
 ↓
Authenticate
 ↓
Access Interface
 ↓
Activate Machine
```

Cada etapa pode ser uma interaction state.

---

# 75. Interaction Sequence

```text id="int83"
InteractionSequence
├── Step 1
├── Step 2
├── Step 3
└── Completion
```

---

# 76. Conditional Sequences

```text id="int84"
Open Vault
 ↓
IF key
    → open

ELSE
    → keypad

IF no key AND wrong code
    → denied
```

---

# 77. Interaction Trees

Isso pode ser usado para:

```text id="int85"
dialogue
machines
terminals
quests
social actions
```

Mas o sistema continua genérico.

---

# 78. Multi-Actor Interaction

Algumas ações exigem:

```text id="int86"
Player A
+
Player B
```

Exemplo:

```text
Two-person machine
Two-player door
Cooperative structure
```

---

# 79. Interaction Participants

```text id="int87"
InteractionParticipants
├── Initiator
├── Target
├── RequiredParticipants
└── OptionalParticipants
```

---

# 80. Synchronization

Para interação cooperativa:

```text id="int88"
READY
→ START
→ SYNCHRONIZED
→ COMPLETE
```

---

# 81. NPC Interaction

NPC pode avaliar:

```text id="int89"
Can I interact?
Should I interact?
What options?
Which option is best?
```

O AI System decide.

Interaction executa a infraestrutura.

---

# 82. AI Decision Separation

```text id="int90"
AI
→ chooses

Interaction
→ describes/coordinates

Command
→ requests

System
→ executes
```

---

# 83. Script Interaction

Scripts podem consultar:

```text
availableInteractions
```

e emitir:

```text
InteractionRequest
```

---

# 84. Mod Interaction

Mods podem criar:

```text id="int91"
InteractionDefinition
InteractionProvider
InteractionCondition
InteractionResolver
```

---

# 85. Registry

Registrar:

```text id="int92"
InteractionDefinition
TargetingType
ConditionType
InteractionProvider
InteractionAction
```

---

# 86. Interaction Tags

Tags:

```text id="int93"
#openable
#repairable
#trade
#inspectable
#container
#vehicle
#social
#machine
```

facilitam consultas.

---

# 87. Generic Target Interface

```text id="int94"
IInteractable
```

pode expor:

```text
getInteractionProviders()
```

Mas não precisa ser implementado por absolutamente todos os objetos; capabilities podem fornecer interação dinamicamente.

---

# 88. Interaction Provider Composition

```text id="int95"
Machine
├── IOpenable
├── IConfigurable
├── IRepairable
└── IInteractable
```

Resultando:

```text
Open
Configure
Repair
```

---

# 89. Interaction Discovery Cache

Para performance:

```text id="int96"
InteractionCache
```

pode guardar:

```text
target
actor capabilities
available actions
```

com invalidação quando o estado muda.

---

# 90. Invalidation

Cache precisa mudar quando:

```text id="int97"
inventory changes
technology changes
permission changes
target state changes
distance changes
quest state changes
```

---

# 91. Network Optimization

Não enviar a lista inteira do mundo.

Somente:

```text id="int98"
nearby relevant interactions
```

---

# 92. Client Prediction

Interaction visual pode ser predita:

```text id="int99"
button pressed
 ↓
UI feedback
```

Mas resultado autoritativo:

```text id="int100"
Server
```

---

# 93. Interaction Security

Cliente tenta:

```text id="int101"
Unlock Door
```

Servidor verifica:

```text id="int102"
key?
permission?
distance?
door state?
```

---

# 94. Anti-Spam

Interaction precisa de:

```text id="int103"
cooldown
debounce
rate limit
```

para evitar:

```text
100.000 interactions/sec
```

---

# 95. Interaction Lock

Quando necessário:

```text id="int104"
Door
→ locked by Interaction Instance
```

durante uma operação.

Mas evitar locks longos.

---

# 96. Reservation

Uma interação pode reservar:

```text id="int105"
machine
item
seat
vehicle
container
```

durante execução.

---

# 97. Interaction Transaction

Para operações complexas:

```text id="int106"
BEGIN
 ↓
RESERVE
 ↓
EXECUTE
 ↓
COMMIT
```

ou:

```text
CANCEL
 ↓
RELEASE
```

---

# 98. Queue

Algumas interações podem entrar em fila:

```text id="int107"
Machine Interaction Queue
```

---

# 99. Interaction Priority

Exemplo:

```text
Emergency Control
>
Manual Control
>
Maintenance
>
Inspection
```

Mas regras de autoridade continuam acima.

---

# 100. Long-term Interaction History

Algumas interações podem gerar histórico:

```text id="int108"
player repaired machine
player opened vault
player negotiated treaty
```

Isso pode alimentar:

```text
Quest
Knowledge
Reputation
Civilization
Audit
```

---

# 101. Interaction Audit

Para ações sensíveis:

```text id="int109"
Actor
Target
Action
Time
Result
Reason
```

---

# 102. World Consequences

A interaction pode causar:

```text id="int110"
state change
```

e o mundo reage.

Exemplo:

```text
Player shuts valve
 ↓
Fluid network changes
 ↓
Machine stops
 ↓
Factory production falls
 ↓
Economy changes
 ↓
NPCs react
```

A Interaction System não calcula toda essa cadeia.

Ela apenas iniciou a operação.

---

# 103. Interaction + Event Bus

```text id="int111"
InteractionStartedEvent
InteractionCompletedEvent
InteractionFailedEvent
InteractionCancelledEvent
```

Mas o estado específico é publicado pelos sistemas responsáveis.

---

# 104. Interaction + Quest

Quest pode observar:

```text id="int112"
InteractionCompletedEvent
```

e completar:

```text
"Activate the ancient console"
```

---

# 105. Interaction + Knowledge

```text id="int113"
InspectArtifact
 ↓
Knowledge System
```

---

# 106. Interaction + Research

```text id="int114"
AnalyzeSample
 ↓
Research
```

---

# 107. Interaction + Economy

```text id="int115"
Trade
 ↓
Economy
```

---

# 108. Interaction + Civilization

```text id="int116"
Negotiate
 ↓
Diplomacy
 ↓
Civilization
```

---

# 109. Interaction + Structure

```text id="int117"
RepairBuilding
 ↓
Structure
 ↓
Build & Destruction
```

---

# 110. Interaction + Entity

```text id="int118"
Talk
Mount
Follow
Inspect
```

Entity System fornece o alvo.

---

# 111. Interaction + Item

```text id="int119"
Use
Equip
Place
Consume
Repair
```

---

# 112. Interaction + Block

```text id="int120"
Open
Toggle
Rotate
Configure
```

---

# 113. Interaction + Vehicle

```text id="int121"
Enter
Exit
Drive
Repair
Refuel
```

---

# 114. Interaction + Dimension

```text id="int122"
EnterPortal
ActivateGateway
Travel
```

Dimension System executa a transferência.

---

# 115. Interaction + UI

UI deve ser somente apresentação:

```text id="int123"
InteractionOption
 ↓
UI
 ↓
user selection
 ↓
Command
```

---

# 116. Interaction + Input

```text id="int124"
Input
 ↓
Targeting
 ↓
Interaction Discovery
 ↓
Selection
```

---

# 117. Advanced Context

O contexto pode considerar:

```text id="int125"
holding item
crouching
sprinting
vehicle
underwater
in atmosphere
low gravity
darkness
weather
season
faction
quest
technology
knowledge
```

---

# 118. Environment Interaction

Exemplos:

```text id="int126"
light fire
extinguish fire
collect water
open valve
enter shelter
climb surface
```

---

# 119. Contextual Environment

Uma interação pode mudar em:

```text id="int127"
underwater
```

Por exemplo:

```text
Use Item
```

pode possuir uma versão diferente.

---

# 120. Interaction Variants

```text id="int128"
InteractionDefinition
├── Default
├── Underwater
├── Vehicle
├── LowGravity
└── Emergency
```

---

# 121. Input Mapping

Não colocar tecla dentro da Interaction Definition.

Evitar:

```text
Open = E
```

Porque:

```text E
```

é Input Mapping.

Interaction deve dizer:

```text id="int129"
preferred interaction action
```

e Input/UI decide qual botão.

---

# 122. Multi-device

A mesma interaction funciona em:

```text id="int130"
keyboard
mouse
gamepad
touch
```

---

# 123. Accessibility

Podemos permitir:

```text id="int131"
single-action mode
auto-confirm
focus navigation
text descriptions
audio prompts
```

sem alterar gameplay.

---

# 124. Mod API

Interfaces:

```text id="int132"
IInteraction
IInteractionDefinition
IInteractionInstance
IInteractionProvider
IInteractionTarget
IInteractionQuery
IInteractionResolver
IInteractionCondition
IInteractionValidator
IInteractionExecutor
IInteractionContext
IInteractionResult
IInteractionReservation
IInteractionTransaction
```

---

# 125. Organização do código

```text id="int133"
src/interaction/

├── core/
│   ├── interaction-system
│   ├── interaction
│   ├── interaction-instance
│   └── interaction-context
│
├── definition/
│   ├── definition
│   ├── variants
│   └── requirements
│
├── registry/
│
├── discovery/
│   ├── discovery
│   ├── query
│   └── scoring
│
├── targeting/
│   ├── raycast
│   ├── proximity
│   ├── area
│   └── trigger
│
├── condition/
│   ├── condition
│   ├── resolver
│   └── combinators
│
├── validation/
│
├── execution/
│
├── lifecycle/
│
├── transaction/
│
├── reservation/
│
├── scheduling/
│
├── networking/
│
├── persistence/
│
├── scripting/
│
├── mod/
│
├── ui/
│
├── input/
│
├── audit/
│
├── metrics/
│
└── debug/
```

---

# 126. Dependências

```text id="int134"
CORE
 │
 ├── Registry
 ├── Event Bus
 ├── Command System
 └── Security
        │
        ▼
INTERACTION
        │
 ┌──────┼─────────────┐
 ▼      ▼             ▼
INPUT TARGETING      CONTEXT
        │
        ▼
   AVAILABILITY
        │
        ▼
    SELECTION
        │
        ▼
    COMMAND
        │
        ▼
     SERVER
        │
        ▼
SPECIALIZED SYSTEMS
```

---

# 127. Implementação por fases

## INT-0 — Contracts

```text id="aaf883"
IInteraction
IInteractionProvider
IInteractionContext
IInteractionResult
```

---

## INT-1 — Basic Discovery

```text id="c9b2px"
target
 ↓
available actions
```

---

## INT-2 — Player Targeting

```text id="int135"
camera
 ↓
raycast
 ↓
interaction
```

---

## INT-3 — Basic Execution

```text id="int136"
select
 ↓
command
 ↓
server
```

---

## INT-4 — Conditions

```text id="int137"
distance
permission
state
item
```

---

## INT-5 — UI Integration

```text id="int138"
prompt
menu
tooltip
```

---

## INT-6 — Entity Integration

```text id="int139"
NPC
ItemEntity
Vehicle
```

---

## INT-7 — Block Integration

```text id="int140"
doors
buttons
containers
machines
```

---

## INT-8 — Long Interactions

```text id="int141"
progress
cancel
interrupt
resume
```

---

## INT-9 — Multiplayer

```text id="int142"
client discovery
server validation
replication
```

---

## INT-10 — Transactions

```text id="int143"
reservation
atomic interaction
```

---

## INT-11 — AI

```text id="int144"
NPC
 ↓
discover
 ↓
choose
 ↓
execute
```

---

## INT-12 — Scripting

```text id="int145"
script
 ↓
query interaction
 ↓
request
```

---

## INT-13 — Modding

```text id="int146"
custom providers
custom conditions
custom interaction types
```

---

## INT-14 — Advanced Context

```text id="int147"
environment
technology
knowledge
quest
faction
```

---

## INT-15 — Optimization

```text id="int148"
cache
scoring
batch queries
LOD
```

---

# 128. Primeiro Vertical Slice

```text id="int149"
PLAYER
 ↓
LOOK AT DOOR
 ↓
RAYCAST
 ↓
DISCOVERY
 ↓
"OPEN"
 ↓
INPUT
 ↓
COMMAND
 ↓
SERVER
 ↓
DOOR SYSTEM
 ↓
STATE CHANGE
 ↓
EVENT
 ↓
ANIMATION
 ↓
AUDIO
```

---

# 129. Segundo Vertical Slice

```text id="int150"
PLAYER
 ↓
NPC
 ↓
Discovery
 ↓
Talk
 ↓
Dialogue
 ↓
Quest Offer
 ↓
AcceptQuestCommand
 ↓
Quest System
```

---

# 130. Terceiro Vertical Slice

```text id="int151"
PLAYER
 ↓
MACHINE
 ↓
Configure
 ↓
Machine UI
 ↓
Command
 ↓
Machine System
 ↓
Energy
 ↓
Event
 ↓
Persistence
```

---

# 131. Quarto Vertical Slice

```text id="int152"
NPC
 ↓
Machine
 ↓
AI discovers interaction
 ↓
Selects Repair
 ↓
RepairCommand
 ↓
Build / Machine
 ↓
Event
```

Isso demonstra que **Interaction não pertence exclusivamente ao Player**.

---

# 132. Quinto Vertical Slice

```text id="int153"
PLAYER
 ↓
ANCIENT DEVICE
 ↓
Inspect
 ↓
Knowledge
 ↓
Research
 ↓
Technology
 ↓
Quest
```

---

# 133. Sexto Vertical Slice

```text id="int154"
PLAYER
 ↓
TRADE NPC
 ↓
Trade Interaction
 ↓
Trade Command
 ↓
Economy
 ↓
Inventory
 ↓
Item Transaction
 ↓
Persistence
 ↓
Networking
```

---

# 134. Golden Test

```text id="int155"
TARGET FOUND
 ↓
AVAILABLE INTERACTIONS CORRECT
 ↓
PLAYER SELECTS
 ↓
REQUEST SENT
 ↓
SERVER VALIDATES
 ↓
COMMAND EXECUTES
 ↓
STATE CHANGES
 ↓
EVENT PUBLISHED
 ↓
UI UPDATES
 ↓
AUDIO / ANIMATION
 ↓
SAVE
```

---

# 135. Security Test

Cliente tenta:

```text id="int156"
interact(target, impossible)
```

Servidor:

```text id="int157"
invalid target
↓
reject
↓
no state change
```

---

# 136. Spam Test

```text id="int158"
100.000 interaction requests
```

Resultado:

```text id="int159"
rate limited
queues protected
server healthy
```

---

# 137. Race Test

Dois players:

```text id="int160"
A → open container
B → modify same container
```

O sistema deve resolver com:

```text id="int161"
state version
transaction
reservation
```

conforme o domínio.

---

# 138. Multiplayer Test

```text id="int162"
CLIENT A
 ↓
interacts with machine
 ↓
SERVER
 ↓
machine changes
 ↓
CLIENT B
 ↓
sees updated state
```

---

# 139. AI Test

```text id="int163"
NPC
 ↓
finds repairable machine
 ↓
discovers interaction
 ↓
AI selects
 ↓
Command
 ↓
Machine repaired
```

---

# 140. Mod Test

Mod registra:

```text id="int164"
example:scan_anomaly
```

em:

```text id="int165"
example:alien_artifact
```

e o jogador recebe:

```text
Scan Anomaly
```

sem alteração do Interaction Core.

---

# 141. Stress Test

```text id="int166"
100 players
10.000 NPCs
100.000 interactable objects
```

com consultas frequentes.

Medir:

```text id="int167"
discovery time
query count
cache hit rate
validation cost
network cost
```

---

# 142. Massive Interaction Test

```text id="int168"
1.000.000 possible interactable objects
```

mas apenas uma pequena área precisa ser consultada.

Isso força:

```text id="int169"
spatial index
interest
targeting
cache
```

---

# 143. Interaction LOD

Não precisamos atualizar interação de tudo:

```text id="int170"
FULL
NEARBY
ABSTRACT
NONE
```

Normalmente apenas objetos próximos precisam de descoberta detalhada.

---

# 144. Spatial Index

Usar:

```text id="int171"
chunk
region
spatial tree
entity index
structure index
```

para descobrir alvos próximos.

---

# 145. Interaction Prediction

No cliente:

```text id="int172"
likely interaction
```

pode ser mostrada imediatamente.

Mas:

```text id="int173"
server result
```

é autoridade.

---

# 146. Future VR

O sistema também deve permitir:

```text id="int174"
hand targeting
gaze targeting
gesture
proximity
```

sem alterar o gameplay.

---

# 147. Future Automation

Machines podem usar:

```text id="int175"
interaction APIs
```

para:

```text
connect
activate
transfer
inspect
```

---

# 148. Future Social Systems

NPCs poderão utilizar o mesmo sistema para:

```text id="int176"
greet
negotiate
teach
request
hire
trade
warn
ask
```

---

# 149. Regra de Ouro

O Interaction System nunca deve virar:

```text id="int177"
PLAYER SYSTEM 2.0
```

Ele deve ser:

```text id="int178"
GENERIC ACTOR ↔ TARGET INTERACTION LAYER
```

---

# 150. Arquitetura final

```text id="int179"
                         NEXORA
                            │
                         INPUT
                            │
                         TARGETING
                            │
                    INTERACTION DISCOVERY
                            │
                     ┌──────┴──────┐
                     ▼             ▼
                  CONTEXT       PROVIDERS
                     │             │
                     └──────┬──────┘
                            ▼
                        CONDITIONS
                            │
                            ▼
                         OPTIONS
                            │
                    ┌───────┴────────┐
                    ▼                ▼
                  PLAYER            AI
                    │                │
                    └───────┬────────┘
                            ▼
                         REQUEST
                            │
                            ▼
                        COMMAND
                            │
                            ▼
                          SERVER
                            │
                       AUTHORITY
                            │
                       VALIDATION
                            │
                            ▼
                    SPECIALIZED SYSTEM
                            │
         ┌──────────┬───────┼────────┬──────────┐
         ▼          ▼       ▼        ▼          ▼
       ENTITY     BLOCK    ITEM     MACHINE   ECONOMY
         │          │       │        │          │
         └──────────┴───────┴────────┴──────────┘
                            │
                       STATE CHANGE
                            │
                        EVENT BUS
                            │
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
            AUDIO       ANIMATION      PERSISTENCE
              │                           │
              └────────── NETWORKING ─────┘
```

E as regras ficam:

```text id="int180"
INPUT
→ "o jogador apertou alguma coisa"

TARGETING
→ "com o que ele está tentando interagir?"

DISCOVERY
→ "o que pode ser feito?"

CONTEXT
→ "em quais condições?"

SELECTION
→ "qual ação foi escolhida?"

COMMAND
→ "quero executar essa ação"

SERVER
→ "a operação é autoritativa e válida?"

SPECIALIZED SYSTEM
→ "como essa ação realmente funciona?"

EVENT
→ "o mundo mudou"

UI
→ "como mostrar isso?"

AUDIO / ANIMATION
→ "como apresentar isso?"
```

### A consequência arquitetural é muito boa

Com **Advanced Interaction**, o NEXORA ganha uma camada que conecta praticamente todos os sistemas sem fazer todos dependerem diretamente uns dos outros:

```text id="int181"
                 ACTOR
          ┌────────┼────────┐
          ▼        ▼        ▼
       Player     NPC     Script
          │        │        │
          └────────┼────────┘
                   ▼
             INTERACTION
                   │
             ┌─────┴─────┐
             ▼           ▼
          TARGET       CONTEXT
             │           │
             └─────┬─────┘
                   ▼
              COMMAND
                   │
                 SERVER
                   │
        ┌──────────┼──────────┐
        ▼          ▼          ▼
      WORLD      ENTITY      GAMEPLAY
        │          │          │
        └──────────┼──────────┘
                   ▼
                EVENT BUS
                   │
       ┌───────────┼───────────┐
       ▼           ▼           ▼
      QUEST     AUDIO/UI     PERSISTENCE
```

Isso também prepara muito bem o próximo conjunto de sistemas de gameplay, porque agora o NEXORA já possui uma sequência coerente:

```text
INPUT
→ INTERACTION
→ COMMAND
→ SERVER
→ SPECIALIZED SYSTEM
→ EVENT
→ PERSISTENCE / NETWORKING / PRESENTATION
```

Essa sequência pode servir como **uma das principais convenções arquiteturais do NEXORA** para impedir que cada sistema invente sua própria maneira de receber ações.
