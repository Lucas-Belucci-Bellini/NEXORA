# NEXORA — QUEST SYSTEM

> **Princípio central:**
> **Quests são objetivos e oportunidades que surgem do estado do mundo, das pessoas, das descobertas e das decisões dos participantes.**
>
> Uma quest não deve ser apenas:
>
> ```text
> NPC
> ↓
> "mate 10 monstros"
> ↓
> recompensa
> ```
>
> No NEXORA, queremos:
>
> ```text
> WORLD STATE
>     ↓
> EVENT / NEED / DISCOVERY
>     ↓
> QUEST GENERATION
>     ↓
> OBJECTIVES
>     ↓
> PLAYER / NPC ACTION
>     ↓
> WORLD CHANGE
>     ↓
> CONSEQUENCES
>     ↓
> NEW QUESTS
> ```

A grande diferença é que **quest não precisa existir esperando o jogador**. Uma guerra, uma cidade sem comida, uma doença, uma descoberta científica, uma expedição, um desastre, uma construção ou uma disputa comercial podem gerar objetivos naturalmente.

---

# 1. O que é o Quest System?

O Quest System gerencia:

```text id="qst01"
Quest Definitions
Quest Instances
Objectives
Conditions
Stages
Dependencies
Participants
Assignments
Rewards
Failures
Deadlines
Consequences
Quest Chains
Dynamic Generation
World Integration
History
Persistence
Networking
```

Ele responde:

> **“Quais objetivos relevantes existem, quem está envolvido e como eles evoluem?”**

Mas não deve ser dono de:

```text id="qst02"
Combat
Economy
Inventory
Crafting
AI
Civilization
WorldGen
Dialogue
```

Ele conversa com esses sistemas.

---

# 2. Quest ≠ Mission

Podemos usar os dois conceitos.

```text id="qst03"
Quest
→ objetivo narrativo/estrutural

Mission
→ objetivo operacional específico
```

Porém, para o Core, `Quest` pode ser o conceito principal e `Mission` uma categoria.

---

# 3. Arquitetura principal

```text id="qst04"
                         QUEST SYSTEM
                              │
       ┌──────────────────────┼──────────────────────┐
       ▼                      ▼                      ▼
    DEFINITION             INSTANCE              GENERATOR
       │                      │                      │
       ▼                      ▼                      ▼
  OBJECTIVES              STATE MACHINE          CONTEXT
       │                      │                      │
       └──────────────────────┼──────────────────────┘
                              ▼
                         CONDITIONS
                              │
                              ▼
                         ASSIGNMENT
                              │
                              ▼
                         EXECUTION
                              │
                  ┌───────────┼───────────┐
                  ▼           ▼           ▼
               REWARD      FAILURE    CONSEQUENCE
                  │           │           │
                  └───────────┼───────────┘
                              ▼
                          WORLD STATE
                              │
                           EVENT BUS
```

---

# 4. Quest Definition

A definição é o modelo.

```text id="qst05"
QuestDefinition
├── QuestID
├── Version
├── Category
├── Tags
├── Prerequisites
├── Trigger
├── Objectives
├── Stages
├── Participants
├── Rewards
├── FailureRules
├── Deadline
└── Consequences
```

---

# 5. Quest Instance

A instância é uma quest realmente existente no mundo.

```text id="qst06"
QuestInstance
├── QuestInstanceID
├── DefinitionID
├── Owner
├── Participants
├── State
├── CurrentStage
├── ObjectiveState
├── Variables
├── Deadline
├── Progress
├── History
└── WorldContext
```

Isso permite que a mesma definição seja usada milhares de vezes.

---

# 6. Exemplo

Definição:

```text id="qst07"
rescue_village
```

Instância:

```text id="qst08"
quest-83921
```

pode representar:

```text id="af5d8z"
Village A
→ food shortage
→ trader requests grain
```

Outra instância:

```text id="s5bjhq"
Village B
→ flood
→ same quest template
```

A definição é a mesma; o contexto é diferente.

---

# 7. Quest Categories

```text id="qst09"
MAIN
SIDE
PERSONAL
WORLD
CIVILIZATION
EXPLORATION
DISCOVERY
RESEARCH
COMMERCE
CRAFTING
BUILDING
COMBAT
DIPLOMACY
ESCORT
DELIVERY
SURVIVAL
RECOVERY
ARCHAEOLOGY
EVENT
EMERGENCY
REPEATABLE
PROCEDURAL
```

Mods podem criar novas categorias.

---

# 8. Quest Sources

Uma quest pode surgir de:

```text id="qst10"
NPC
Player
Civilization
Faction
Guild
Profession
World Event
Research
Discovery
Economy
Structure
Location
Item
Dialogue
Script
Mod
```

---

# 9. Quest Trigger

O gatilho responde:

> por que esta quest existe?

Exemplos:

```text id="qst11"
PlayerEnteredRegion
NPCRequestedHelp
FoodShortage
StormOccurred
ResearchCompleted
AncientStructureFound
TradeRouteInterrupted
FactionWarStarted
PlayerDiscoveredTechnology
```

---

# 10. Dynamic Quest Generation

O NEXORA deve permitir gerar quests a partir do mundo.

Exemplo:

```text id="qst12"
CIDADE
 ↓
food production falls
 ↓
SHORTAGE EVENT
 ↓
Quest Generator
 ↓
"Find alternative food source"
```

Isso é muito mais interessante do que 500 NPCs com quests pré-escritas.

---

# 11. Quest Generator

```text id="qst13"
IQuestGenerator
```

recebe:

```text id="qst14"
WorldContext
ActorContext
NeedContext
EventContext
KnowledgeContext
EconomyContext
```

e produz:

```text id="qst15"
QuestCandidate
```

---

# 12. Quest Candidate

Antes da quest virar uma instância oficial:

```text id="qst16"
Candidate
 ↓
Validate
 ↓
Score
 ↓
Accept
 ↓
Instantiate
```

Isso evita gerar milhares de quests irrelevantes.

---

# 13. Quest Relevance

Cada candidato pode possuir:

```text id="qst17"
relevance
urgency
difficulty
rewardValue
distance
risk
relationship
```

O sistema pode priorizar as mais relevantes.

---

# 14. Objectives

Objetivos são unidades menores.

Tipos:

```text id="qst18"
Reach
Find
Collect
Deliver
Craft
Build
Break
Protect
Defeat
Escort
Discover
Research
Talk
Trade
Buy
Sell
Repair
Survive
Explore
Observe
Wait
Choose
Decide
```

---

# 15. Objective Definition

```text id="qst19"
ObjectiveDefinition
├── ObjectiveID
├── Type
├── Target
├── Conditions
├── Quantity
├── Scope
├── ProgressRule
└── CompletionRule
```

---

# 16. Objective State

```text id="qst20"
ObjectiveState
├── Status
├── CurrentValue
├── RequiredValue
├── StartedAt
├── CompletedAt
└── Metadata
```

---

# 17. Objectives não precisam ser lineares

Quest pode ser:

```text id="qst21"
A
├── B
├── C
└── D
```

e:

```text id="qst22"
B OR C
```

pode completar uma etapa.

---

# 18. Objective Graph

Em vez de apenas:

```text id="qst23"
1 → 2 → 3
```

usar:

```text id="qst24"
       START
         │
      ┌──┴──┐
      ▼     ▼
    FIND   TALK
      │     │
      └──┬──┘
         ▼
       DECIDE
       ├──→ A
       └──→ B
```

---

# 19. Quest State Machine

Estados:

```text id="qst25"
DRAFT
OFFERED
AVAILABLE
ACCEPTED
ACTIVE
PAUSED
COMPLETED
FAILED
EXPIRED
ABANDONED
CANCELLED
```

---

# 20. Quest Progression

Uma quest pode ter:

```text id="qst26"
Stage 1
Stage 2
Stage 3
Stage 4
```

Mas internamente usamos o grafo de objetivos.

---

# 21. Stages

Stage pode representar uma macrofase.

```text id="qst27"
Stage 1
→ Investigate

Stage 2
→ Find evidence

Stage 3
→ Make decision

Stage 4
→ Resolve consequence
```

---

# 22. Branching

Uma decisão pode alterar o caminho:

```text id="qst28"
DECISION
├── SUPPORT A
│    └── Path A
└── SUPPORT B
     └── Path B
```

---

# 23. Consequences

O mais importante:

**Escolha não precisa apenas mudar a recompensa.**

Pode mudar:

```text id="qst29"
Faction Reputation
NPC Relationship
Economy
Civilization
Territory
Trade
Technology
Knowledge
Structures
Future Quests
World Events
```

---

# 24. Quest Choices

```text id="qst30"
Choice
├── ChoiceID
├── Requirements
├── Effects
├── Consequences
└── Availability
```

---

# 25. Choice Conditions

Exemplo:

```text id="qst31"
Choice:
give technology to faction

requires:
knowledge == known
relationship >= 50
```

---

# 26. Permanent Consequences

Algumas escolhas podem criar alterações persistentes:

```text id="qst32"
Faction relationship
Settlement ownership
Trade route
Technology diffusion
Structure construction
NPC survival
```

---

# 27. Não existe “reset automático”

Uma quest concluída não precisa desaparecer da história.

Pode virar:

```text id="qst33"
Quest History
```

e afetar outras quests.

---

# 28. Quest History

Registrar:

```text id="qst34"
QuestInstanceID
Participant
Outcome
Choices
ImportantEvents
Consequences
Time
Location
```

Isso permite que o mundo “lembre” do passado.

---

# 29. Quest Memory

NPC pode lembrar:

```text id="qst35"
Player helped village
Player betrayed faction
Player delivered medicine
```

Isso alimenta:

```text id="qst36"
Relationship
Reputation
Knowledge
Future Quest Generation
```

---

# 30. NPC Quest Generation

Um NPC pode ter uma necessidade:

```text id="qst37"
Need:
medicine
```

O Civilization/Economy/Health System detecta:

```text id="mngm1f"
need unresolved
```

Quest Generator produz:

```text id="qst38"
find medicinal plant
```

---

# 31. Quest não precisa de NPC

Pode surgir:

```text id="qst39"
World Event
 ↓
quest
```

Exemplo:

```text id="0ro0ly"
Railway damaged
```

gera:

```text id="qst40"
Repair Railway
```

---

# 32. World Quest

Uma quest pode ser compartilhada por milhares de atores.

```text id="qst41"
WORLD EVENT
 ↓
global quest
 ↓
many participants
```

---

# 33. Quest Scope

```text id="qst42"
PERSONAL
PARTY
FACTION
SETTLEMENT
REGION
CIVILIZATION
WORLD
```

---

# 34. Group Quest

```text id="qst43"
Party
├── Player A
├── Player B
└── Player C
```

Objetivos podem ser:

```text id="qst44"
shared
individual
contribution-based
```

---

# 35. Multiplayer

Cada participante deve possuir:

```text id="qst45"
QuestParticipantState
```

com:

```text id="qst46"
accepted
progress
permissions
contribution
role
```

---

# 36. Quest Authority

No multiplayer:

```text id="qst47"
SERVER
→ authoritative quest state
```

Cliente:

```text id="qst48"
display
input
prediction of UI
```

não decide conclusão.

---

# 37. Quest Progress Events

Eventos relevantes:

```text id="qst49"
QuestOffered
QuestAccepted
ObjectiveStarted
ObjectiveProgressed
ObjectiveCompleted
StageChanged
QuestChoiceMade
QuestCompleted
QuestFailed
QuestExpired
QuestAbandoned
QuestCancelled
QuestConsequenceApplied
```

---

# 38. Event Bus

```text id="qst50"
Combat
 ↓
EnemyDefeatedEvent
 ↓
Quest System
 ↓
ObjectiveProgress
```

Ou:

```text id="qst51"
ResearchCompleted
 ↓
Quest Objective
```

O Quest System observa o mundo.

Ele não precisa ficar perguntando o estado de cada sistema constantemente.

---

# 39. Event-driven Objectives

Uma objective pode escutar:

```text id="qst52"
BlockBrokenEvent
ItemCraftedEvent
EntityKilledEvent
StructureBuiltEvent
ResearchCompletedEvent
TradeCompletedEvent
DiscoveryMadeEvent
```

---

# 40. Objective Conditions

Condições:

```text id="qst53"
Actor condition
Location condition
Item condition
Entity condition
Technology condition
Knowledge condition
Faction condition
Time condition
Weather condition
Quest history
World state
```

---

# 41. Condition Graph

```text id="qst54"
AND
OR
NOT
XOR
THRESHOLD
SEQUENCE
```

---

# 42. Example

```text id="qst55"
Complete research

requires:

(
  Technology: metallurgy
  AND
  ResearchProject: completed
)
OR
(
  AncientBlueprint: acquired
)
```

---

# 43. Dynamic Targets

Objetivo não precisa ser:

```text id="qst56"
Kill Zombie #4821
```

Pode ser:

```text id="qst57"
Defeat the current leader of faction X
```

Resolvido dinamicamente.

---

# 44. Target Resolver

```text id="qst58"
ITargetResolver
```

Pode retornar:

```text id="qst59"
Entity
Structure
Item
Location
Faction
Civilization
Research Project
Technology
```

---

# 45. Quest Variables

Quest pode possuir variáveis:

```text id="qst60"
enemyCount
targetFaction
chosenPath
rewardTier
deadline
evidenceFound
```

Isso permite templates dinâmicos.

---

# 46. Quest Template

```text id="qst61"
Template
+
World Context
+
Actor Context
+
Variables
↓
Quest Instance
```

---

# 47. Procedural Quest Generation

Exemplo:

```text id="qst62"
Need:
food shortage

Context:
city
winter
trade route blocked
```

Generator:

```text id="qst63"
Find alternate supply
```

Pode criar:

```text id="qst64"
travel
trade
escort
repair railway
```

dependendo das circunstâncias.

---

# 48. Quest Quality

Não gerar quests absurdas.

Exemplo ruim:

```text id="qst65"
villager
→ asks player to repair satellite
```

se o contexto não suporta isso.

---

# 49. Context-Aware Generation

Gerador consulta:

```text id="qst66"
Technology
Economy
Civilization
Location
Resources
Known Problems
NPC Capabilities
Player Capabilities
World Events
```

---

# 50. Quest Feasibility

Antes de oferecer:

```text id="qst67"
Quest
 ↓
Feasibility Check
 ↓
Can Actor reasonably do this?
```

---

# 51. Quest Difficulty

Pode derivar de:

```text id="qst68"
distance
enemies
resource rarity
technology
time
environment
political risk
economic cost
```

---

# 52. Quest Scaling

Evitar simplesmente:

```text id="qst69"
enemy HP × player level
```

O mundo pode alterar dificuldade naturalmente.

---

# 53. Rewards

Rewards são separadas do Quest System em origem, mas Quest controla a distribuição.

Tipos:

```text id="qst70"
Item
Currency
Experience
Knowledge
Technology
Reputation
Access
Blueprint
Structure
Faction Standing
Profession
Quest Unlock
```

---

# 54. Reward Definition

```text id="qst71"
RewardDefinition
├── Type
├── Amount
├── Conditions
├── Distribution
└── TransactionPolicy
```

---

# 55. Reward Transaction

Recompensa precisa ser idempotente:

```text id="qst72"
QuestCompletionID
```

para impedir recompensa duplicada.

---

# 56. Reward Distribution

Pode ser:

```text id="qst73"
PLAYER
PARTY
NPC
FACTION
CITY
CIVILIZATION
```

---

# 57. Shared Rewards

Por exemplo:

```text id="qst74"
Repair railway
```

pode recompensar:

```text id="qst75"
Player
+
Settlement
+
Trade faction
```

---

# 58. Quest Failure

Failure não precisa ser:

```text id="qst76"
GAME OVER
```

Pode gerar consequência.

```text id="qst77"
Failed escort
 ↓
merchant lost
 ↓
trade route weakens
 ↓
new quest generated
```

---

# 59. Expiration

Quest pode possuir:

```text id="qst78"
deadline
```

Mas o relógio deve usar:

```text id="qst79"
world time
```

e não apenas real-world time.

---

# 60. Failure Conditions

```text id="qst80"
NPC dies
Structure destroyed
Deadline reached
Faction war ends
Item destroyed
Player abandons
World state changes
```

---

# 61. Quest Cancellation

Quest pode ser cancelada porque:

```text id="qst81"
world changed
creator died
faction dissolved
objective became impossible
mod removed
```

---

# 62. Dynamic World

Aqui o Quest System fica realmente interessante.

Imagine:

```text id="qst82"
Civilization A
```

declara guerra a:

```text id="j6y3s0"
Civilization B
```

Isso pode gerar:

```text id="qst83"
defend city
deliver supplies
repair railway
scout border
negotiate peace
```

sem que essas quests tenham sido escritas individualmente.

---

# 63. Economy-Generated Quests

```text id="qst84"
Factory
 ↓
resource shortage
 ↓
Supply Quest
```

---

# 64. Research-Generated Quests

```text id="qst85"
Research
 ↓
unknown material
 ↓
Expedition Quest
```

---

# 65. Civilization-Generated Quests

```text id="qst86"
City
 ↓
population growth
 ↓
needs infrastructure
 ↓
Construction Quest
```

---

# 66. Ecology-Generated Quests

```text id="qst87"
Pest outbreak
 ↓
Agriculture problem
 ↓
Investigation Quest
```

---

# 67. Climate-Generated Quests

```text id="qst88"
Severe drought
 ↓
water shortage
 ↓
Water Infrastructure Quest
```

---

# 68. Structure-Generated Quests

```text id="qst89"
Ancient Structure discovered
 ↓
Archaeology Quest
```

---

# 69. Knowledge-Generated Quests

```text id="qst90"
Rumor
 ↓
uncertain information
 ↓
Investigation Quest
```

---

# 70. Player-Created Quests

No futuro, jogadores podem criar objetivos.

```text id="qst91"
Player
 ↓
Create Contract
 ↓
Quest System
```

Exemplo:

```text id="qst92"
"I need 500 copper delivered to my factory."
```

---

# 71. Contracts

Isso pode ser um subconjunto:

```text id="qst93"
Contract
```

com:

```text id="qst94"
issuer
requirements
payment
deadline
penalty
```

---

# 72. Economy + Quest

Mercado pode gerar:

```text id="qst95"
delivery contracts
resource contracts
escort contracts
construction contracts
repair contracts
```

---

# 73. Faction Quest

Fações podem gerar:

```text id="qst96"
recruitment
diplomacy
espionage
research
trade
territory
```

---

# 74. Civilization Quest

Civilizações podem gerar objetivos coletivos:

```text id="qst97"
Build Railway
Expand Hospital
Secure Water
Establish Trade Route
Research Electricity
```

---

# 75. World Quest

Em escala global:

```text id="qst98"
Meteorological disaster
Dimensional instability
Mass migration
Global resource crisis
```

---

# 76. Quest Chain

Quests podem conectar:

```text id="qst99"
Quest A
 ↓
Quest B
 ↓
Quest C
```

Mas preferir dependências por estado:

```text id="qst100"
A completed
AND
faction relationship > 50
```

em vez de obrigar uma sequência fixa.

---

# 77. Emergent Quest Chains

Uma consequência pode gerar outra quest:

```text id="qst101"
Quest A
 ↓
Faction changed
 ↓
World Event
 ↓
Quest B
```

Essa cadeia não precisa estar pré-escrita.

---

# 78. Quest History → Future

Isso permite:

```text id="qst102"
player helps village
 ↓
village remembers
 ↓
years later
 ↓
descendant offers new opportunity
```

O mundo possui continuidade.

---

# 79. Repeatable Quests

Quests repetíveis podem ser:

```text id="qst103"
delivery
bounties
contracts
research tasks
resource orders
```

Mas precisam evitar gerar tarefas infinitamente idênticas.

---

# 80. Quest Diversity

Generator pode considerar:

```text id="qst104"
recent quest types
recent targets
player history
location
faction
world events
```

para evitar repetição.

---

# 81. Anti-Grind

Não queremos:

```text id="qst105"
100 quests
→ mesma coisa
```

Podemos usar:

```text id="qst106"
dynamic objectives
branching
context
world consequences
```

---

# 82. Dialogue Integration

Dialogue System deve ser separado.

Fluxo:

```text id="qst107"
NPC Dialogue
 ↓
Quest Offer
 ↓
Quest System
```

Quest pode expor:

```text id="qst108"
available choices
objective state
requirements
```

Dialogue apresenta isso.

---

# 83. UI Integration

UI mostra:

```text id="qst109"
Quest Journal
Active Quests
Objectives
Map Markers
Progress
Choices
History
Rewards
```

Mas UI não possui autoridade.

---

# 84. Map Integration

Objective pode fornecer:

```text id="qst110"
LocationReference
TargetReference
NavigationHint
```

Map System/World UI apresenta.

---

# 85. Quest Marker

Não assumir que toda quest precisa de:

```text id="qst111"
!
```

Marker pode depender de:

```text id="qst112"
distance
knowledge
visibility
urgency
discovery
```

---

# 86. Knowledge-gated Quest

O player pode não saber que a quest existe.

```text id="qst113"
Rumor unknown
 ↓
NPC mentions
 ↓
Knowledge acquired
 ↓
Quest becomes visible
```

---

# 87. Hidden Quest

Quest pode ser:

```text id="qst114"
hidden
```

até determinada descoberta.

---

# 88. Quest Discovery

Player pode descobrir através de:

```text id="qst115"
conversation
book
event
structure
research
exploration
trade
reputation
```

---

# 89. Reputation Integration

Quest pode alterar:

```text id="qst116"
NPC Reputation
Faction Standing
Civilization Relations
Profession Reputation
```

---

# 90. Consequence Graph

Uma quest pode definir:

```text id="qst117"
Consequence
├── immediate
├── delayed
├── conditional
└── cascading
```

---

# 91. Delayed Consequences

Por exemplo:

```text id="qst118"
Quest completed today
 ↓
10 game days later
 ↓
faction policy changes
 ↓
new quest
```

---

# 92. Quest Scheduler

```text id="qst119"
Immediate
Delayed
World-time
Event-driven
Regional
Civilization
Seasonal
```

---

# 93. Quest LOD

Para escalar:

```text id="qst120"
FULL
REGIONAL
ABSTRACT
```

### FULL

Quest individual de player.

### REGIONAL

Quest pool de uma cidade.

### ABSTRACT

Quest generation statistics de uma civilização distante.

---

# 94. Distant Quests

Civilização distante não precisa executar milhares de quests detalhadas.

Pode simular:

```text id="qst121"
food delivery requests
research tasks
construction projects
```

em nível abstrato.

---

# 95. Quest Simulation

```text id="qst122"
FULL
→ objective-level

REGIONAL
→ aggregated objectives

ABSTRACT
→ outcome-level
```

---

# 96. Persistence

Salvar:

```text id="qst123"
QuestInstance
ObjectiveState
Choices
Variables
History
Deadlines
Consequences
```

---

# 97. Quest Migration

Quando a definição muda:

```text id="qst124"
Quest v1
 ↓
Migration
 ↓
Quest v2
```

---

# 98. Missing Quest Definition

Se um mod for removido:

```text id="qst125"
QuestInstance
 ↓
Missing Definition
```

Não apagar imediatamente.

Preservar:

```text id="qst126"
namespace
questID
raw state
version
```

para possível recuperação.

---

# 99. Networking

Sincronizar somente o necessário.

Player:

```text id="qst127"
active quests
objective progress
choices
rewards
```

NPC/Civilization:

```text id="qst128"
relevant quest state
```

---

# 100. Quest Authority

```text id="qst129"
SERVER
→ quest completion

CLIENT
→ request action / present state
```

---

# 101. Security

O cliente não pode enviar:

```text id="qst130"
"quest completed = true"
```

Ele envia ações:

```text id="qst131"
Interact
Deliver
Craft
Defeat
Choose
```

e o servidor calcula o progresso.

---

# 102. Reward Security

Recompensas usam:

```text id="qst132"
QuestCompletionTransactionID
```

para impedir:

```text id="qst133"
complete
disconnect
reconnect
complete again
```

---

# 103. Commands

O Quest System pode registrar comandos:

```text id="qst134"
AcceptQuestCommand
AbandonQuestCommand
ChooseQuestOptionCommand
CompleteQuestCommand
ShareQuestCommand
CreateContractCommand
```

---

# 104. Scripts

Scripts podem gerar:

```text id="qst135"
Quest
Objective
Trigger
Reward
```

através de APIs públicas.

---

# 105. Mods

Mods podem registrar:

```text id="qst136"
Quest Definitions
Objective Types
Conditions
Generators
Rewards
Consequences
```

---

# 106. Quest Definition via Data

Uma quest simples pode ser declarativa:

```yaml id="qst137"
id: example:repair_bridge

trigger:
  type: structure_damaged

objectives:
  - repair:
      structure: target
```

Mas o schema exato fica para implementação.

---

# 107. Custom Objective Types

Mod pode adicionar:

```text id="qst138"
example:scan_anomaly
```

sem modificar o Quest Core.

---

# 108. Custom Conditions

```text id="qst139"
example:technology_mastered
```

---

# 109. Custom Rewards

```text id="qst140"
example:grant_research
```

usando capability/API apropriada.

---

# 110. Quest Registry

Integrado ao Registry:

```text id="qst141"
QuestDefinition
ObjectiveType
ConditionType
RewardType
TriggerType
```

---

# 111. Quest API

```text id="qst142"
IQuestSystem
IQuestDefinition
IQuestInstance
IQuestRegistry
IQuestGenerator
IQuestObjective
IQuestObjectiveResolver
IQuestCondition
IQuestConditionResolver
IQuestReward
IQuestConsequence
IQuestAssignment
IQuestParticipant
IQuestScheduler
IQuestHistory
IQuestPersistence
```

---

# 112. Organização de código

```text
src/quest/

├── core/
│   ├── quest-system
│   ├── quest-context
│   ├── quest-state
│   └── quest-instance
│
├── definition/
│   ├── quest-definition
│   ├── objective-definition
│   └── stage-definition
│
├── registry/
│
├── objective/
│   ├── objective
│   ├── types
│   ├── resolver
│   └── progress
│
├── condition/
│   ├── condition
│   ├── combinators
│   └── resolver
│
├── trigger/
│
├── generator/
│   ├── generator
│   ├── templates
│   ├── context
│   └── scoring
│
├── assignment/
│
├── choice/
│
├── reward/
│
├── consequence/
│
├── history/
│
├── scheduler/
│
├── networking/
│
├── persistence/
│
├── mod/
│
├── scripting/
│
├── debug/
│
└── simulation/
```

---

# 113. Dependências

```text id="qst143"
REGISTRY
   │
EVENT BUS
   │
PERSISTENCE
   │
   └──────────────┐
                  ▼
             QUEST SYSTEM
                  │
       ┌──────────┼───────────┐
       ▼          ▼           ▼
   KNOWLEDGE    WORLD       PLAYER
       │          │           │
       ▼          ▼           ▼
   RESEARCH    ENTITY       SKILL
       │          │           │
       └──────────┼───────────┘
                  ▼
              QUEST STATE
                  │
       ┌──────────┼──────────┐
       ▼          ▼          ▼
    REWARD     CHOICE   CONSEQUENCE
       │          │          │
       ▼          ▼          ▼
      ITEM     FACTION     WORLD
```

---

# 114. Implementação por fases

## QUEST-0 — Core

```text
IQuest
IQuestDefinition
IQuestInstance
IQuestSystem
```

---

## QUEST-1 — Registry

```text
QuestRegistry
ObjectiveRegistry
ConditionRegistry
RewardRegistry
```

---

## QUEST-2 — Basic Objectives

Implementar:

```text
Reach
Collect
Deliver
Interact
```

---

## QUEST-3 — State Machine

```text
available
accepted
active
completed
failed
```

---

## QUEST-4 — Conditions

```text
AND
OR
NOT
state
item
location
```

---

## QUEST-5 — Rewards

```text
Item
Currency
Experience
Reputation
```

---

## QUEST-6 — Event Integration

```text
Event Bus
 ↓
Objective Progress
```

---

## QUEST-7 — Persistence

```text
save
load
migration
```

---

## QUEST-8 — Player UI Integration

```text
Journal
Objectives
Markers
Rewards
```

---

## QUEST-9 — Multiplayer

```text
server authority
replication
shared quests
```

---

## QUEST-10 — Choices

```text
branching
conditional paths
consequences
```

---

## QUEST-11 — Dynamic Generator

```text
world context
need detection
candidate generation
scoring
```

---

## QUEST-12 — NPC/Civilization

```text
NPC needs
city needs
faction objectives
```

---

## QUEST-13 — Contracts

```text
player-created
economy-generated
```

---

## QUEST-14 — Advanced History

```text
quest memory
delayed consequences
chains
```

---

## QUEST-15 — LOD

```text
full
regional
abstract
```

---

# 115. Primeiro Vertical Slice

Começaria simples:

```text id="qst144"
NPC
 ↓
offers quest
 ↓
Player accepts
 ↓
Reach location
 ↓
Objective completes
 ↓
Return to NPC
 ↓
Quest completes
 ↓
Reward transaction
 ↓
Event
 ↓
Persistence
```

---

# 116. Segundo Vertical Slice

Uma quest gerada pelo mundo:

```text id="qst145"
City
 ↓
food shortage
 ↓
World Event
 ↓
Quest Generator
 ↓
Quest Candidate
 ↓
Validate
 ↓
Offer Player
 ↓
Deliver food
 ↓
Economy changes
 ↓
Quest completes
```

---

# 117. Terceiro Vertical Slice

Quest com consequência:

```text id="qst146"
Faction A vs Faction B
 ↓
Player chooses A
 ↓
Faction relationship changes
 ↓
World state changes
 ↓
Different future quest
```

---

# 118. Quarto Vertical Slice

Pesquisa:

```text id="qst147"
Research discovery
 ↓
Quest:
"Investigate strange material"
 ↓
Explore
 ↓
Analyze
 ↓
Knowledge acquired
 ↓
Technology discovered
```

---

# 119. Quinto Vertical Slice

Civilização:

```text id="qst148"
City
 ↓
railway damaged
 ↓
repair quest
 ↓
player + NPCs
 ↓
Structure repaired
 ↓
Trade route resumes
 ↓
Economy changes
```

Esse é excelente porque conecta:

```text
Quest
Structure
Build
Economy
Civilization
NPC
Persistence
```

---

# 120. Sexto Vertical Slice

Quest de emergência:

```text id="qst149"
Drought
 ↓
water shortage
 ↓
Civilization generates quest
 ↓
build water infrastructure
 ↓
Fluid System
 ↓
Water availability rises
 ↓
Agriculture recovers
```

Aqui o Quest System realmente começa a fazer parte do mundo vivo.

---

# 121. Golden Quest Test

```text id="qst150"
WORLD
 ↓
EVENT
 ↓
QUEST GENERATED
 ↓
QUEST ACCEPTED
 ↓
OBJECTIVE PROGRESS
 ↓
OBJECTIVE COMPLETED
 ↓
QUEST COMPLETED
 ↓
REWARD COMMITTED
 ↓
CONSEQUENCE APPLIED
 ↓
EVENT PUBLISHED
 ↓
SAVE
 ↓
RESTART
 ↓
QUEST HISTORY RESTORED
```

---

# 122. Golden Multiplayer Test

```text id="qst151"
SERVER
+
CLIENT A
+
CLIENT B

A accepts quest
 ↓
Server records

A performs action
 ↓
Server validates

Objective updates
 ↓
A receives update
B receives relevant update

Quest completes
 ↓
Reward once
 ↓
Save
```

---

# 123. Duplicate Reward Test

```text id="qst152"
Complete Quest
 ↓
Reward = SUCCESS

Reconnect
 ↓
retry completion
 ↓
NO SECOND REWARD
```

---

# 124. Quest Generator Stress Test

```text id="qst153"
1 city
10 cities
100 cities
1.000 cities
10.000 settlements
```

com:

```text
needs
wars
trade
research
construction
disasters
```

O sistema precisa gerar apenas quests relevantes.

---

# 125. Quest Simulation Stress

```text id="qst154"
1.000 active player quests
10.000 NPC quests
100.000 regional quests
1.000.000 abstract quest states
```

usando LOD.

---

# 126. Quest Graph Stress

Testar:

```text id="qst155"
100 objectives
1.000
10.000
branching
conditions
dependencies
cycles
```

Detectar ciclos impossíveis.

---

# 127. Security Tests

Cliente tenta:

```text id="qst156"
CompleteQuestCommand
```

sem cumprir objetivo.

Resultado:

```text id="qst157"
DENIED
```

Cliente tenta:

```text id="qst158"
ClaimRewardAgain
```

Resultado:

```text id="qst159"
DUPLICATE
```

---

# 128. Missing Content Test

Remover mod que fornece quest:

```text id="qst160"
Mod removed
 ↓
Quest definition missing
 ↓
Instance preserved
 ↓
World remains loadable
```

---

# 129. Consequence Test

```text id="qst161"
Quest A
 ↓
Faction reputation +20
 ↓
new policy unlocked
 ↓
Quest B availability changes
```

---

# 130. Architecture final

```text id="qst162"
                         NEXORA
                            │
                       QUEST SYSTEM
                            │
       ┌────────────────────┼────────────────────┐
       ▼                    ▼                    ▼
   DEFINITIONS          INSTANCES            GENERATOR
       │                    │                    │
       ▼                    ▼                    ▼
  OBJECTIVES             STATE             WORLD CONTEXT
       │                    │                    │
       └────────────────────┼────────────────────┘
                            ▼
                        CONDITIONS
                            │
                            ▼
                        ASSIGNMENT
                            │
                            ▼
                         ACTIONS
                            │
                     ┌──────┴──────┐
                     ▼             ▼
                  CHOICES       PROGRESS
                     │             │
                     └──────┬──────┘
                            ▼
                         RESULT
                  ┌─────────┼─────────┐
                  ▼         ▼         ▼
               REWARD    FAILURE   CONSEQUENCE
                  │         │         │
                  └─────────┼─────────┘
                            ▼
                        WORLD STATE
                            │
                 ┌──────────┼──────────┐
                 ▼          ▼          ▼
              PLAYER      NPC       CIVILIZATION
                 │          │          │
                 └──────────┼──────────┘
                            ▼
                         EVENT BUS
                            │
                ┌───────────┼───────────┐
                ▼           ▼           ▼
           PERSISTENCE   NETWORKING   UI
```

A regra definitiva:

```text id="qst163"
QUEST
→ representa um objetivo

OBJECTIVE
→ representa uma condição de progresso

TRIGGER
→ explica por que a quest existe

GENERATOR
→ cria oportunidades baseadas no mundo

CONDITION
→ determina quando algo é válido

COMMAND
→ representa a ação solicitada

SYSTEM
→ executa a mecânica

EVENT
→ informa o que aconteceu

REWARD
→ entrega o resultado

CONSEQUENCE
→ altera o futuro

HISTORY
→ permite que o mundo se lembre
```

E isso cria uma diferença enorme em relação ao modelo tradicional:

```text
MODELO TRADICIONAL

NPC
 ↓
QUEST FIXA
 ↓
PLAYER
 ↓
RECOMPENSA
```

Enquanto o NEXORA pode funcionar como:

```text
MUNDO
 │
 ├── economia muda
 ├── clima muda
 ├── cidade cresce
 ├── guerra começa
 ├── NPC descobre tecnologia
 ├── estrutura é destruída
 └── recurso fica escasso
          │
          ▼
    QUEST GENERATOR
          │
          ▼
     OPORTUNIDADES
          │
     ┌────┴─────┐
     ▼          ▼
   PLAYER      NPC
     │          │
     └────┬─────┘
          ▼
       AÇÕES
          │
          ▼
     MUNDO MUDA
          │
          ▼
    NOVAS QUESTS
```

Esse modelo encaixa perfeitamente com **Progression/Technology, Knowledge, Civilization, Economy, Structure, Entity, Event Bus, Command System, Server e Persistence** que já definimos. A quest deixa de ser uma lista isolada de tarefas e vira **uma camada de objetivos emergentes do próprio estado do mundo**.
