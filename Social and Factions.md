# NEXORA — SOCIAL / FACTIONS SYSTEM

> **Princípio central:**
> **Social System simula relações entre indivíduos e grupos; Factions System organiza identidade coletiva, interesses, poder e cooperação.**
>
> O objetivo não é criar apenas:
>
> ```text
> NPC → reputação → +10
> ```
>
> e sim:
>
> ```text
> INDIVÍDUOS
>     ↓
> RELAÇÕES
>     ↓
> GRUPOS
>     ↓
> FAÇÕES
>     ↓
> INTERESSES
>     ↓
> PODER / REPUTAÇÃO / POLÍTICA
>     ↓
> DECISÕES
>     ↓
> AÇÕES
>     ↓
> MUNDO MUDA
> ```

Esse sistema é uma das peças que faltavam para transformar a civilização do NEXORA em algo realmente vivo.

---

# 1. O que o Social / Factions System faz?

Ele gerencia:

```text
Social Relationships
Trust
Reputation
Friendship
Rivalry
Family
Groups
Factions
Organizations
Guilds
Political Parties
Military Groups
Religious/Cultural Groups
Corporations
Settlements
Civilizations
Diplomacy
Alliances
Treaties
Conflicts
Membership
Influence
Leadership
Internal Politics
Elections
Loyalty
Defection
Migration
Recruitment
Social Memory
```

Mas não deve assumir a responsabilidade por:

```text
Combat
Economy
Quest
Technology
AI
Civilization simulation
```

Ele fornece relações e estruturas sociais; os outros sistemas utilizam essas informações.

---

# 2. Social ≠ Faction

São conceitos diferentes.

```text
SOCIAL
→ relação entre indivíduos e grupos

FACTION
→ entidade coletiva organizada
```

Exemplo:

```text
Lucas
  ↓
friendship
  ↓
NPC A
```

é Social.

Já:

```text
NPC A
  ↓
member of
  ↓
Miners Guild
```

é Faction/Organization.

---

# 3. Architecture

```text id="soc01"
                    SOCIAL / FACTIONS
                           │
          ┌────────────────┼─────────────────┐
          ▼                ▼                 ▼
      RELATIONS         GROUPS            FACTIONS
          │                │                 │
          ▼                ▼                 ▼
       TRUST           MEMBERSHIP        INTERESTS
          │                │                 │
          └────────────────┼─────────────────┘
                           ▼
                        INFLUENCE
                           │
                           ▼
                        POLITICS
                           │
                 ┌─────────┴─────────┐
                 ▼                   ▼
             DIPLOMACY           INTERNAL
                 │               GOVERNANCE
                 ▼                   │
             EXTERNAL               ▼
            RELATIONS           DECISIONS
                 │                   │
                 └─────────┬─────────┘
                           ▼
                         ACTION
                           │
                           ▼
                        WORLD
```

---

# 4. Relationship Graph

O fundamento social deve ser um grafo.

```text id="soc02"
NPC A
 ├── friend → NPC B
 ├── distrust → NPC C
 ├── family → NPC D
 └── member → Faction X
```

E:

```text id="soc03"
Faction X
 ├── ally → Faction Y
 ├── rival → Faction Z
 └── trade → Faction W
```

---

# 5. Relationship Definition

Uma relação deve possuir:

```text id="soc04"
RelationshipType
Strength
Trust
Sentiment
History
Visibility
LastInteraction
Origin
Decay
Flags
```

---

# 6. Relationship Types

```text id="soc05"
FAMILY
FRIEND
ACQUAINTANCE
MENTOR
STUDENT
COLLEAGUE
EMPLOYER
EMPLOYEE
ALLY
RIVAL
ENEMY
TRADER
CLIENT
NEIGHBOR
LEADER
FOLLOWER
MEMBER
```

Mods podem adicionar novos tipos.

---

# 7. Sentiment ≠ Trust

Não reduzir tudo a uma barra.

Um NPC pode:

```text id="soc06"
trust = high
sentiment = negative
```

por exemplo:

> “Não gosto dele, mas confio que cumprirá o acordo.”

Ou:

```text id="soc07"
sentiment = positive
trust = low
```

> “Gosto dele, mas não confiaria meu dinheiro a ele.”

Isso produz comportamento muito mais interessante.

---

# 8. Reputation

Reputação também precisa ser contextual.

Não:

```text id="soc08"
Player Reputation = 83
```

Mas:

```text id="soc09"
Faction A = 80
Faction B = 15
Merchant Guild = 62
Village C = 94
```

---

# 9. Reputation Scope

```text id="soc10"
PERSON
FAMILY
PROFESSION
SETTLEMENT
FACTION
REGION
CIVILIZATION
GLOBAL
```

---

# 10. Reputation não precisa ser global

Um jogador pode ser:

```text id="soc11"
herói em uma cidade
```

e:

```text id="soc12"
procurado em outra
```

O mundo não precisa compartilhar uma reputação universal.

---

# 11. Social Memory

NPCs devem lembrar acontecimentos relevantes.

```text id="soc13"
SocialMemory
├── event
├── actor
├── target
├── time
├── emotional weight
├── confidence
└── decay
```

Exemplo:

```text id="soc14"
Player helped family
↓
memory created
↓
trust increases
```

---

# 12. Memory Decay

Nem toda memória precisa durar para sempre.

```text id="soc15"
Recent
 ↓
Remembered
 ↓
Faded
 ↓
Forgotten
```

Mas acontecimentos muito importantes podem possuir:

```text id="soc16"
permanent memory
```

---

# 13. Social Events

```text id="soc17"
Helped
Insulted
Betrayed
Saved
Attacked
Traded
Negotiated
Gifted
Taught
Lied
Protected
Abandoned
```

Esses eventos podem alterar relações.

---

# 14. Social Event → Relationship

```text id="soc18"
Player saves NPC
        ↓
Social Event
        ↓
Relationship Resolver
        ↓
Trust +++
        ↓
Reputation +
```

---

# 15. Não fazer tudo determinístico

Uma ação não precisa sempre significar:

```text id="soc19"
save NPC
=
+20 reputation
```

O impacto pode depender de:

```text id="soc20"
prior relationship
culture
personality
faction
knowledge
visibility
importance
politics
context
```

---

# 16. Personality Integration

NPC pode possuir traços:

```text id="soc21"
Honest
Suspicious
Loyal
Ambitious
Curious
Conservative
RiskTaking
Empathetic
Competitive
```

O Personality System pode ser separado, enquanto Social consome os traços.

---

# 17. Group System

Antes de criar uma Faction, podemos possuir:

```text id="soc22"
Group
```

Um grupo pode ser:

```text id="soc23"
family
party
work team
club
guild
faction
military unit
community
```

---

# 18. Group Definition

```text id="soc24"
GroupDefinition
├── GroupID
├── Name
├── Type
├── MembershipRules
├── Roles
├── Governance
├── Resources
├── Relations
└── Policies
```

---

# 19. Group Instance

```text id="soc25"
GroupInstance
├── GroupID
├── Members
├── Leaders
├── Resources
├── Reputation
├── Goals
├── Policies
└── Relationships
```

---

# 20. Faction

Faction é um grupo com identidade e interesses relativamente definidos.

Exemplos:

```text id="soc26"
Miners Guild
Merchant League
Royal Army
Research Consortium
Local Government
Rebel Movement
Industrial Corporation
```

---

# 21. Faction Definition

```text id="soc27"
FactionDefinition
├── FactionID
├── Identity
├── Values
├── Interests
├── MembershipRules
├── Roles
├── Governance
├── Policies
├── DiplomaticRules
└── Capabilities
```

---

# 22. Faction Identity

Pode conter:

```text id="soc28"
name
symbol
culture
language
values
history
origin
```

Arte e apresentação continuam pertencendo aos sistemas correspondentes.

---

# 23. Faction Values

Uma facção pode priorizar:

```text id="soc29"
Freedom
Security
Profit
Science
Tradition
Expansion
Peace
Military
Religion
Industry
Environment
Knowledge
```

Isso influencia decisões.

---

# 24. Faction Interests

Exemplo:

```text id="soc30"
Merchant League
├── low trade taxes
├── safe roads
├── stable currency
└── new markets
```

---

# 25. Interests ≠ Goals

Interesse:

```text id="soc31"
"quer estabilidade comercial"
```

Goal:

```text id="soc32"
"secure railway route X"
```

---

# 26. Goals

Faction Goals podem ser:

```text id="soc33"
economic
territorial
political
military
technological
social
religious
environmental
research
```

---

# 27. Faction Strategy

Faction pode decidir como perseguir seus objetivos.

```text id="soc34"
Goal
 ↓
Strategy
 ↓
Decision
 ↓
Action
```

---

# 28. Faction Decision System

Isso não deve ficar todo dentro do Social System.

A ideia:

```text id="soc35"
Faction
 ↓
Decision Context
 ↓
AI / Governance
 ↓
Command
```

Social fornece relações/interesses.

AI/Governance toma decisões.

---

# 29. Membership

NPC pode:

```text id="soc36"
join
leave
be recruited
be expelled
defect
be promoted
be demoted
```

---

# 30. Membership State

```text id="soc37"
APPLIED
PROBATION
MEMBER
OFFICER
LEADER
SUSPENDED
EXPELLED
RESIGNED
```

---

# 31. Membership Conditions

Pode depender de:

```text id="soc38"
reputation
profession
technology
knowledge
relationship
wealth
culture
citizenship
quest
skills
```

---

# 32. Roles

Uma facção pode ter:

```text id="soc39"
Member
Officer
Scout
Trader
Engineer
Researcher
Commander
Leader
```

---

# 33. Role ≠ Profession

NPC:

```text id="soc40"
Profession = Engineer
Role = Guild Treasurer
```

São conceitos diferentes.

---

# 34. Governance

Factions podem usar:

```text id="soc41"
Hierarchy
Council
Democracy
Oligarchy
Meritocracy
Military Command
Corporate
Religious
Tribal
Custom
```

---

# 35. Governance Definition

```text id="soc42"
GovernanceModel
├── Leadership
├── Succession
├── Voting
├── Authority
├── Policies
└── DecisionRules
```

---

# 36. Leadership

Líder pode ser escolhido por:

```text id="soc43"
election
inheritance
appointment
combat
merit
wealth
tradition
religion
```

---

# 37. Elections

```text id="soc44"
Election
├── Candidates
├── Electors
├── VotingRules
├── Campaigns
├── Results
├── Term
└── Consequences
```

---

# 38. Elections precisam de contexto

NPC pode votar com base em:

```text id="soc45"
trust
faction loyalty
policy preference
knowledge
relationships
propaganda
personal interests
```

---

# 39. Political Parties

Dentro de uma civilização:

```text id="soc46"
Government
├── Party A
├── Party B
└── Party C
```

Essas organizações podem disputar políticas.

---

# 40. Political Influence

Cada grupo pode possuir:

```text id="soc47"
Influence
```

derivada de:

```text wealth
population
military
knowledge
reputation
infrastructure
political seats
```

---

# 41. Influence não é dinheiro

Uma pequena facção pode ter:

```text id="soc48"
high knowledge
```

e portanto enorme influência científica mesmo sem riqueza.

---

# 42. Diplomacy

Entre facções:

```text id="soc49"
Alliance
Neutral
Trade Partner
Non-Aggression
Rival
Hostile
War
```

---

# 43. Diplomatic Relationship

```text id="soc50"
DiplomaticState
├── Trust
├── Relations
├── Agreements
├── Conflicts
├── Trade
├── Military
└── History
```

---

# 44. Treaties

```text id="soc51"
Treaty
├── Parties
├── Terms
├── Duration
├── Benefits
├── Obligations
├── Violations
└── Status
```

---

# 45. Trade Treaty

Pode definir:

```text id="soc52"
tariffs
trade rights
safe routes
currency
resource access
```

Economy executa os detalhes.

---

# 46. Defense Pact

```text id="soc53"
Faction A
+
Faction B
→
Mutual Defense
```

---

# 47. Diplomatic Decisions

Uma facção pode considerar:

```text id="soc54"
Should we ally?
Should we trade?
Should we sanction?
Should we negotiate?
Should we declare war?
```

AI/Governance decide.

---

# 48. War

Social/Factions não deve implementar Combat.

Ele representa:

```text id="soc55"
political state
```

Combat/Military executa:

```text id="soc56"
battles
damage
units
```

---

# 49. War Lifecycle

```text id="soc57"
Tension
↓
Crisis
↓
Ultimatum
↓
Hostility
↓
War
↓
Ceasefire
↓
Peace
```

---

# 50. War Causes

Podem surgir de:

```text id="soc58"
territory
resources
trade
ideology
retaliation
alliances
politics
religion
history
```

---

# 51. Diplomacy Memory

Faction pode lembrar:

```text id="soc59"
betrayal
treaty
war
aid
trade
```

e isso afeta futuras negociações.

---

# 52. Trust

Diplomatic trust separado de individual trust.

```text id="soc60"
NPC Trust
Faction Trust
Institutional Trust
```

---

# 53. Faction Reputation

Player:

```text id="soc61"
Faction A reputation = 90
```

pode ser resultado de:

```text id="soc62"
quests
trade
social actions
help
betrayal
membership
```

---

# 54. Reputation Decay

Algumas reputações podem diminuir com o tempo:

```text id="soc63"
recent fame
 ↓
normal
```

Mas eventos históricos importantes podem persistir.

---

# 55. Fame vs Reputation

Separar:

```text id="soc64"
Fame
→ quanta gente conhece você

Reputation
→ o que pensam de você
```

---

# 56. Information Propagation

Como o NEXORA possui Knowledge System:

```text id="soc65"
action
 ↓
witnesses
 ↓
rumor
 ↓
information
 ↓
faction knowledge
```

Portanto nem todo evento é automaticamente conhecido por todos.

---

# 57. Visibility

Uma ação pode possuir:

```text id="soc66"
visibility
```

```text id="soc67"
private
local
regional
faction
public
global
```

---

# 58. Propaganda

Uma facção pode espalhar sua interpretação de um evento.

Exemplo:

```text id="soc68"
Battle
 ↓
Faction A says:
"victory"
 ↓
Faction B says:
"defensive success"
```

Knowledge System pode armazenar perspectivas diferentes.

---

# 59. Social Narrative

Isso permite que duas sociedades tenham memórias diferentes sobre o mesmo evento.

---

# 60. Diplomacy + Quest

Tratados e conflitos podem gerar quests:

```text id="soc69"
Alliance
 ↓
Diplomatic Quest
```

ou:

```text id="soc70"
War
 ↓
Military Quest
```

---

# 61. Faction + Quest

Faction pode gerar:

```text id="soc71"
Recruit
Trade
Scout
Negotiate
Build
Research
Protect
```

---

# 62. Faction + Economy

Uma guilda comercial pode influenciar:

```text id="soc72"
trade routes
prices
contracts
resource demand
```

Economy executa os mercados.

---

# 63. Faction + Civilization

Factions existem dentro de:

```text id="soc73"
Settlement
Civilization
```

mas podem ultrapassar fronteiras.

---

# 64. Cross-Civilization Faction

Uma guilda comercial pode existir:

```text id="soc74"
Civilization A
+
Civilization B
+
Civilization C
```

Isso é interessante para comércio internacional.

---

# 65. Organizations

Nem toda organização é uma Faction política.

Podemos ter:

```text id="soc75"
Corporation
Guild
University
Military
Religion
Union
Clan
Company
Government
Research Group
```

---

# 66. Organization Hierarchy

```text id="soc76"
Civilization
 ├── Government
 │    ├── Ministry
 │    └── Military
 │
 ├── Guild
 │    ├── Local Branch
 │    └── Headquarters
 │
 └── Company
      ├── Factory
      └── Office
```

---

# 67. Hierarchy

Grupo pode pertencer a outro:

```text id="soc77"
Group A
 ↓
member of
 ↓
Group B
```

Isso gera uma árvore/grafo social.

---

# 68. Nested Factions

```text id="soc78"
Republic
 ├── Government
 ├── Army
 ├── Merchant Guild
 └── Research Council
```

---

# 69. Internal Factions

Até uma cidade pode possuir grupos internos:

```text id="soc79"
City
├── Merchants
├── Workers
├── Researchers
├── Military
└── Nobility
```

Esses grupos podem disputar influência.

---

# 70. Political Pressure

Um grupo pode pressionar o governo:

```text id="soc80"
Workers
 ↓
demand wage increase
 ↓
political pressure
 ↓
policy decision
```

---

# 71. Strikes / Social Movements

Pode existir:

```text id="soc81"
protest
strike
boycott
petition
movement
```

Sem precisar controlar a economia diretamente.

---

# 72. Social Conflict

Conflitos podem surgir por:

```text id="soc82"
resources
beliefs
status
wealth
leadership
territory
jobs
technology
```

---

# 73. Social Stability

Civilization pode receber um indicador:

```text id="soc83"
SocialStability
```

derivado de:

```text id="soc84"
inequality
trust
conflict
security
food
employment
political legitimacy
```

Mas esses inputs vêm de outros sistemas.

---

# 74. Legitimacy

Um governo pode possuir:

```text id="soc85"
Legitimacy
```

baseada em:

```text elections
tradition
military power
public approval
economic performance
```

---

# 75. Loyalty

NPC pode possuir:

```text id="soc86"
Loyalty
```

em relação a:

```text id="soc87"
faction
leader
settlement
civilization
```

---

# 76. Loyalty ≠ Membership

NPC pode:

```text id="soc88"
member = Faction A
loyalty = low
```

e estar considerando sair.

---

# 77. Defection

NPC pode:

```text id="soc89"
defect
```

quando:

```text id="soc90"
trust low
opportunity high
alternative faction attractive
```

---

# 78. Recruitment

Faction pode procurar membros com:

```text id="soc91"
skills
profession
reputation
technology
relationships
```

---

# 79. Social Mobility

NPC pode mudar:

```text id="soc92"
profession
status
wealth
faction
political role
```

---

# 80. Marriage / Family?

Family relationships podem existir como uma categoria social, mas o sistema deve tratá-las como **dados sociais e históricos**, sem assumir uma mecânica central de relacionamento romântico para gameplay.

O objetivo técnico é permitir:

```text id="soc93"
parent
child
sibling
relative
household
```

para genealogia e sociedade.

---

# 81. Household

Um grupo menor:

```text id="soc94"
Household
```

pode possuir:

```text members
resources
home
relationships
```

---

# 82. Family Genealogy

```text id="soc95"
Family
 ↓
generation
 ↓
relationships
```

Isso pode alimentar:

```text id="soc96"
inheritance
reputation
knowledge
profession
```

conforme outros sistemas precisarem.

---

# 83. Social Networks

A rede inteira:

```text id="soc97"
NPC
↓
Family
↓
Guild
↓
Faction
↓
Civilization
```

Isso cria múltiplas camadas de identidade.

---

# 84. Multi-Membership

NPC pode pertencer a:

```text id="soc98"
family
+
guild
+
political party
+
military reserve
```

ao mesmo tempo.

---

# 85. Conflicting Loyalties

Isso é importante.

```text id="soc99"
NPC
├── loyalty → family
├── loyalty → guild
└── loyalty → government
```

Pode haver conflito.

---

# 86. Decision Conflict

NPC pode precisar escolher:

```text id="soc100"
protect family
VS
follow faction order
```

AI decide com base em:

```text values
loyalty
fear
reward
relationship
risk
knowledge
```

---

# 87. Social Roles

Um NPC pode possuir:

```text id="soc101"
formal role
informal role
```

Exemplo:

```text id="soc102"
formal = engineer
informal = influential advisor
```

---

# 88. Influence Network

NPCs podem ter:

```text id="soc103"
influence
```

derivada de:

```text reputation
wealth
knowledge
relationships
position
```

---

# 89. Social Hubs

Estruturas podem servir como:

```text id="soc104"
meeting place
market
guild hall
government
university
tavern
```

Structure System cria o prédio.

Social System utiliza sua função social.

---

# 90. Social + Structure

```text id="soc105"
Structure
 ↓
social function
 ↓
Group activities
```

---

# 91. Social + Audio

Eventos sociais podem gerar:

```text id="soc106"
crowd ambience
celebration
protest
announcement
```

Audio System apresenta.

---

# 92. Social + UI

UI pode mostrar:

```text id="soc107"
Relationships
Faction standing
Organizations
Diplomacy
Politics
```

---

# 93. Social + Quest

Quest pode consultar:

```text id="soc108"
relationship
reputation
membership
```

---

# 94. Social + Progression

Faction pode conhecer:

```text id="soc109"
technology
research
knowledge
```

Progression System permanece dono do progresso.

---

# 95. Social + Economy

```text id="soc110"
Organization
 ↓
Economic activity
```

Economy executa.

---

# 96. Social + Civilization

Aqui é a ligação principal:

```text id="soc111"
SOCIAL
 ↓
FACTIONS
 ↓
POLITICS
 ↓
CIVILIZATION
```

---

# 97. Faction Goals

Faction pode possuir:

```text id="soc112"
Goal
├── target
├── priority
├── deadline
├── resources
├── strategy
└── status
```

---

# 98. Goal Priority

```text id="soc113"
CRITICAL
HIGH
NORMAL
LOW
```

---

# 99. Decision Context

AI/Governance recebe:

```text id="soc114"
FactionState
Relationships
Economy
Military
Technology
Knowledge
WorldEvents
Threats
```

e escolhe ação.

---

# 100. Action

Resultado pode gerar:

```text id="soc115"
Command
```

Exemplo:

```text id="z8r71c"
Faction decision
 ↓
CreateTradeAgreementCommand
```

ou:

```text id="soc116"
RecruitMemberCommand
```

---

# 101. Command Integration

```text id="soc117"
Faction / AI
 ↓
Command
 ↓
Server
 ↓
Specialized System
```

---

# 102. Event Integration

Quando algo acontece:

```text id="soc118"
TradeAgreementSignedEvent
```

Social pode atualizar:

```text id="soc119"
DiplomaticRelation
```

---

# 103. Social Events

```text id="soc120"
RelationshipChanged
MemberJoined
MemberLeft
LeaderChanged
FactionCreated
FactionDisbanded
TreatySigned
TreatyBroken
AllianceFormed
ConflictStarted
PeaceDeclared
ElectionStarted
ElectionCompleted
```

---

# 104. Persistence

Persistir:

```text id="soc121"
important relationships
faction memberships
leaders
treaties
major reputation
political state
social history
```

Não necessariamente todos os detalhes de cada NPC distante.

---

# 105. Social LOD

O NEXORA precisa escalar.

```text id="soc122"
FULL
REGIONAL
ABSTRACT
```

### FULL

```text id="soc123"
NPC A trusts NPC B
```

### REGIONAL

```text id="soc124"
Guild has 2,300 members
```

### ABSTRACT

```text id="soc125"
Merchant faction influential
```

---

# 106. Relationship Aggregation

Distant simulation pode agregar:

```text id="soc126"
average trust
group cohesion
membership changes
political support
```

---

# 107. Social Simulation

Não executar:

```text id="soc127"
100.000 NPC relationships
```

individualmente a cada tick.

Usar:

```text id="soc128"
event-driven
regional
aggregate
```

---

# 108. Influence Simulation

Uma grande decisão:

```text id="soc129"
Faction A gains influence
```

pode ser calculada regionalmente.

---

# 109. Social Propagation

Um evento pode se espalhar:

```text id="soc130"
NPC
 ↓
family
 ↓
workplace
 ↓
guild
 ↓
city
 ↓
civilization
```

com atraso.

---

# 110. Information Delay

Isso combina com Knowledge.

Uma notícia não deve aparecer magicamente no mundo inteiro.

```text id="soc131"
Event
 ↓
Witness
 ↓
Rumor
 ↓
Merchant
 ↓
City
 ↓
Faction
```

---

# 111. Reputation Propagation

Mesma ideia:

```text id="soc132"
Player action
 ↓
witnesses
 ↓
social network
 ↓
faction reputation
```

---

# 112. Propagation Accuracy

Informação pode sofrer:

```text id="soc133"
loss
distortion
bias
delay
```

---

# 113. Faction Knowledge

Cada facção pode possuir:

```text id="soc134"
known events
known players
known factions
known locations
known technology
```

---

# 114. Social Intelligence

Faction pode possuir uma capacidade de inteligência derivada de:

```text id="soc135"
spies
merchants
diplomats
researchers
communications
```

Mas os mecanismos específicos pertencem a outros sistemas.

---

# 115. Diplomats

NPC pode representar uma facção.

```text id="soc136"
Diplomat
 ↓
Faction
 ↓
Treaty
```

---

# 116. Ambassadors

Uma relação diplomática pode possuir:

```text id="soc137"
representatives
missions
communication
```

---

# 117. Social Contracts

Uma relação pode possuir termos:

```text id="soc138"
agreement
obligation
reward
penalty
duration
```

---

# 118. Violation

Se um tratado for quebrado:

```text id="soc139"
TreatyViolationEvent
```

pode reduzir trust.

---

# 119. Escalation

```text id="soc140"
Violation
 ↓
Diplomatic tension
 ↓
Sanctions
 ↓
Hostility
 ↓
Conflict
```

---

# 120. Peace

Peace pode surgir de:

```text id="soc141"
negotiation
fatigue
economic pressure
leadership change
external threat
```

---

# 121. Faction Dissolution

Uma facção pode desaparecer:

```text id="soc142"
loss of members
loss of resources
political collapse
merger
```

---

# 122. Faction Merger

```text id="soc143"
Faction A
+
Faction B
↓
Faction C
```

com histórico preservado.

---

# 123. Faction Split

```text id="soc144"
Faction A
 ↓
ideological conflict
 ↓
Faction B
+
Faction C
```

Isso é muito interessante para política emergente.

---

# 124. Faction Evolution

Uma guilda pode tornar-se:

```text id="soc145"
local guild
 ↓
regional organization
 ↓
national institution
 ↓
international corporation
```

---

# 125. Civilization Politics

O governo pode mudar:

```text id="soc146"
election
coup
revolution
reform
succession
```

Sem o Social System implementar toda a política do mundo; ele fornece as relações e estruturas.

---

# 126. Social + Migration

NPC pode migrar por:

```text id="soc147"
family
job
safety
faction
opportunity
politics
```

Population/Migration System executa o movimento.

---

# 127. Social + Profession

Faction pode recrutar:

```text id="soc148"
Engineer
Doctor
Farmer
Miner
Scientist
Trader
```

---

# 128. Social + Knowledge

Um professor pode transmitir conhecimento:

```text id="soc149"
Teacher
 ↓
Knowledge
 ↓
Student
```

---

# 129. Social + Technology

Faction pode possuir:

```text id="soc150"
technology adoption
```

mas Progression System calcula o estado tecnológico.

---

# 130. Social + Religion/Culture

Podemos ter grupos culturais:

```text id="soc151"
cultural identity
beliefs
traditions
```

sem criar um sistema rígido de “raças boas/más”.

---

# 131. Culture Integration

Um NPC pode possuir:

```text id="soc152"
culture
language
traditions
values
```

Social utiliza isso nas relações.

---

# 132. Language

Communication pode considerar:

```text id="soc153"
shared language
partial language
translator
```

Isso pode afetar comércio/diplomacia.

---

# 133. Group Communication

Faction possui:

```text id="soc154"
communication channels
```

que podem utilizar:

```text id="soc155"
messenger
radio
network
courier
```

Communication System futuro pode controlar isso.

---

# 134. Social Graph APIs

```text id="soc156"
IRelationshipGraph
IRelationship
ISocialMemory
IReputationSystem
IGroup
IFaction
IFactionRegistry
IMembership
IRole
IGovernanceModel
IDiplomacy
ITreaty
IInfluence
```

---

# 135. Faction Registry

Integrado ao Registry:

```text id="soc157"
FactionDefinition
GroupType
RelationshipType
GovernanceModel
Role
TreatyType
```

---

# 136. Modding

Mods podem registrar:

```text id="soc158"
new factions
group types
relationship types
governance models
social events
reputation rules
diplomatic relations
```

---

# 137. Script Integration

Scripts podem consultar:

```text id="soc159"
getRelationship()
getFaction()
getReputation()
getMembers()
getDiplomaticState()
```

E podem emitir Commands.

---

# 138. Security

Cliente não pode simplesmente mandar:

```text id="soc160"
setReputation(100)
```

Servidor valida:

```text id="soc161"
action
history
authority
```

---

# 139. Persistence Security

Social state precisa de:

```text id="soc162"
integrity
version
migration
transaction
```

principalmente em:

```text economy
politics
membership
treaties
```

---

# 140. UI

Podemos ter:

```text id="soc163"
Social Panel
Faction Panel
Diplomacy
Reputation
Relationships
Politics
Organizations
```

---

# 141. Debug Commands

```text id="soc164"
nexora social inspect
nexora social relations
nexora social reputation
nexora social history

nexora faction list
nexora faction inspect
nexora faction members
nexora faction relations
nexora faction influence
nexora faction diplomacy
nexora faction election
```

---

# 142. Social Graph Debugger

Visualização:

```text id="soc165"
             PLAYER
          /     |      \
       friend  trust   rival
        /        |        \
      NPC A    NPC B    NPC C
        |
     member
        |
    Guild X
        |
      ally
        |
    Faction Y
```

---

# 143. Metrics

```text id="soc166"
Relationships
Active Factions
Memberships
Diplomatic Treaties
Conflicts
Elections
Political Groups
Social Events
Reputation Changes
Influence Changes
```

---

# 144. Performance

O social graph pode ficar gigantesco.

Portanto:

```text id="soc167"
sparse graph
indexed relationships
regional partitioning
LOD
event-driven updates
```

---

# 145. Relationship Storage

Não guardar sempre:

```text id="soc168"
every NPC × every NPC
```

Preferir somente relações relevantes.

---

# 146. Relationship Decay

Relações antigas podem ser comprimidas:

```text id="soc169"
Detailed History
 ↓
Important Memories
 ↓
Aggregated Relationship
```

---

# 147. Historical Compression

Exemplo:

```text id="soc170"
1.000 small interactions
```

podem virar:

```text id="soc171"
long-term trust: high
relationship history: 3 major events
```

---

# 148. Social Event Importance

Cada evento pode possuir:

```text id="soc172"
importance
emotional weight
political impact
```

Assim eventos relevantes sobrevivem mais tempo.

---

# 149. Civilization-scale Social Model

```text id="soc173"
Individual
 ↓
Household
 ↓
Profession
 ↓
Organization
 ↓
Faction
 ↓
Settlement
 ↓
Civilization
```

---

# 150. Influence Flow

```text id="soc174"
Individual
 ↓
Group
 ↓
Faction
 ↓
Government
 ↓
Civilization
```

Mas o fluxo pode ocorrer em várias direções.

---

# 151. Bottom-Up Emergence

Uma grande decisão política pode surgir de:

```text id="soc175"
NPC opinions
 ↓
groups
 ↓
factions
 ↓
political pressure
 ↓
government decision
```

---

# 152. Top-Down Effects

E ao contrário:

```text id="soc176"
Government policy
 ↓
Faction reaction
 ↓
Group reaction
 ↓
NPC behavior
```

---

# 153. Social Feedback Loop

```text id="soc177"
POLICY
 ↓
WORLD EFFECT
 ↓
SOCIAL RESPONSE
 ↓
POLITICAL PRESSURE
 ↓
NEW POLICY
```

Isso pode gerar sistemas emergentes.

---

# 154. Example

```text id="soc178"
Government increases industrial tax
 ↓
Factories lose profitability
 ↓
Merchant faction becomes unhappy
 ↓
Political influence changes
 ↓
Election support changes
 ↓
New government
 ↓
Tax policy changes
```

Nenhuma quest específica precisou criar essa sequência.

---

# 155. Social + Economy

Outro exemplo:

```text id="soc179"
Food prices rise
 ↓
Workers become unhappy
 ↓
Union gains influence
 ↓
Government pressured
 ↓
Food policy changes
```

---

# 156. Social + Technology

```text id="soc180"
New automation technology
 ↓
Factories adopt
 ↓
some jobs disappear
 ↓
worker faction reacts
 ↓
political movement
 ↓
new policies
```

---

# 157. Social + Research

```text id="soc181"
Research breakthrough
 ↓
Faction wants control
 ↓
political dispute
 ↓
technology secrecy
```

---

# 158. Social + War

```text id="soc182"
Faction loses war
 ↓
leadership legitimacy falls
 ↓
internal faction split
 ↓
new government
```

---

# 159. Social + Quests

```text id="soc183"
Faction split
 ↓
two quest lines
 ↓
player chooses side
 ↓
political outcome
```

---

# 160. Social + World Events

```text id="soc184"
Disaster
 ↓
government response
 ↓
public reaction
 ↓
faction support
 ↓
election
```

---

# 161. Social + Far Lands

```text id="soc185"
Far Lands discovered
 ↓
new resources
 ↓
merchant interest
 ↓
exploration faction
 ↓
settlement expedition
```

---

# 162. Social + Deep World

```text id="soc186"
Underground civilization found
 ↓
diplomatic contact
 ↓
trade
 ↓
knowledge exchange
 ↓
new faction relations
```

---

# 163. Social + Dimensions

Uma dimensão pode possuir:

```text id="soc187"
independent societies
```

que possuem suas próprias:

```text id="soc188"
factions
cultures
diplomacy
```

---

# 164. Cross-Dimension Diplomacy

Mais tarde:

```text id="soc189"
Dimension A
 ↕
Dimension B
```

com:

```text id="soc190"
trade
diplomacy
research exchange
migration
```

---

# 165. Quest Generation

Quest Generator pode consultar:

```text id="soc191"
faction conflict
membership
reputation
diplomatic state
political goals
```

e criar:

```text id="soc192"
diplomatic quest
recruitment quest
political quest
trade quest
```

---

# 166. Progression

Progression pode permitir:

```text id="soc193"
new political technologies
communication
governance
```

---

# 167. Technology changes society

```text id="soc194"
Technology
 ↓
new industry
 ↓
new profession
 ↓
new faction
```

---

# 168. Faction emergence

Não precisa ser tudo pré-criado.

Um grupo pode surgir quando:

```text id="soc195"
shared interest
+
enough members
+
organization
+
leadership
```

---

# 169. Faction Creation Resolver

```text id="soc196"
Potential Group
 ↓
Threshold
 ↓
Organization
 ↓
Faction
```

---

# 170. Faction Dissolution

Se:

```text id="soc197"
members < threshold
```

ou:

```text id="soc198"
legitimacy = 0
```

pode haver dissolução.

---

# 171. Faction Emergence Example

```text id="soc199"
100 miners
 ↓
shared economic interest
 ↓
guild
 ↓
leadership
 ↓
merchant contracts
 ↓
political influence
```

---

# 172. Social Simulation Rules

Não fazer tudo emergente sem limites.

Precisamos de:

```text id="soc200"
minimum population
importance thresholds
event budgets
formation cooldown
relationship limits
```

---

# 173. Social Scheduler

```text id="soc201"
FULL:
important individual relations

REGIONAL:
group changes

ABSTRACT:
civilization influence/politics
```

---

# 174. Social Persistence

```text id="soc202"
FULL:
important NPC relationships

REGIONAL:
faction structures

ABSTRACT:
political/diplomatic state
```

---

# 175. Golden Social Test

```text id="soc203"
NPC A
 ↓
helps NPC B
 ↓
relationship improves
 ↓
B joins group
 ↓
group grows
 ↓
faction forms
 ↓
faction gains influence
 ↓
election occurs
 ↓
policy changes
 ↓
world reacts
```

---

# 176. Golden Diplomacy Test

```text id="soc204"
Faction A
 ↕
Faction B

Negotiation
 ↓
Treaty
 ↓
Trade
 ↓
Trust increases
 ↓
Alliance
```

---

# 177. Golden Conflict Test

```text id="soc205"
Resource shortage
 ↓
Faction tension
 ↓
Negotiation fails
 ↓
Hostility
 ↓
Conflict
 ↓
Peace
 ↓
Treaty
```

---

# 178. Golden Reputation Test

```text id="soc206"
Player helps village
 ↓
NPC witnesses
 ↓
Social Memory
 ↓
Village reputation
 ↓
Guild hears rumor
 ↓
Regional reputation
```

com atraso e perda de informação apropriados.

---

# 179. Golden Faction Test

```text id="soc207"
NPCs
 ↓
shared interest
 ↓
group
 ↓
leadership
 ↓
organization
 ↓
faction
 ↓
governance
```

---

# 180. Stress Test

```text id="soc208"
100 players
10.000 NPCs
100.000 NPCs
1.000.000 abstract actors
100.000 groups
10.000 factions
```

com:

```text id="soc209"
relationships
reputation
diplomacy
elections
membership
```

usando LOD.

---

# 181. Graph Stress

Testar:

```text id="soc210"
1.000 relationships
100.000
1.000.000
10.000.000
```

sem fazer busca global a cada tick.

---

# 182. Political Stress

```text id="soc211"
10 civilizations
100 factions each
many elections
many treaties
many conflicts
```

---

# 183. Security Test

Tentar:

```text id="soc212"
client
 ↓
setReputation = 999999
```

Resultado:

```text id="soc213"
DENIED
```

---

# 184. Mod Test

Um mod adiciona:

```text id="soc214"
example:research_council
```

com:

```text id="soc215"
custom role
custom influence
custom governance rule
```

sem alterar o Core.

---

# 185. Script Test

Script:

```text id="soc216"
detect faction event
 ↓
issue diplomatic command
```

através de:

```text id="soc217"
Event Bus
+
Command System
```

---

# 186. Organização do código

```text id="soc218"
src/social/

├── core/
│   ├── social-system
│   ├── social-context
│   └── social-state
│
├── relationship/
│   ├── relationship
│   ├── graph
│   ├── types
│   └── resolver
│
├── memory/
│   ├── social-memory
│   ├── importance
│   └── decay
│
├── reputation/
│   ├── reputation
│   ├── scope
│   └── propagation
│
├── groups/
│   ├── group
│   ├── membership
│   ├── roles
│   └── hierarchy
│
├── factions/
│   ├── faction
│   ├── identity
│   ├── interests
│   ├── goals
│   └── influence
│
├── governance/
│   ├── governance
│   ├── leadership
│   ├── elections
│   └── policies
│
├── diplomacy/
│   ├── diplomacy
│   ├── treaty
│   ├── alliance
│   └── conflict
│
├── communication/
│
├── propagation/
│
├── simulation/
│   ├── full
│   ├── regional
│   └── abstract
│
├── networking/
├── persistence/
├── scripting/
├── mod/
├── metrics/
└── debug/
```

---

# 187. APIs

```text id="soc219"
ISocialSystem
IRelationshipGraph
IRelationshipResolver
ISocialMemory
IReputationSystem
IReputationResolver

IGroup
IGroupManager
IMembership
IRole
IOrganization

IFaction
IFactionManager
IFactionResolver
IFactionInfluence

IGovernanceSystem
IGovernanceModel
IElection
ILeadership

IDiplomacySystem
IDiplomaticRelation
ITreaty
IAlliance

ISocialPropagation
ISocialSimulator
```

---

# 188. Registry

```text id="soc220"
RelationshipType
GroupType
FactionType
RoleType
GovernanceType
TreatyType
DiplomaticState
SocialEventType
```

registrados via Registry System.

---

# 189. Implementação por fases

## SOCIAL-0 — Core

```text id="soc221"
Relationship
SocialMemory
Reputation
Group
Faction
```

---

## SOCIAL-1 — Relationship Graph

```text id="soc222"
nodes
edges
types
strength
```

---

## SOCIAL-2 — Memory

```text id="soc223"
events
importance
decay
```

---

## SOCIAL-3 — Reputation

```text id="soc224"
context
scope
propagation
```

---

## SOCIAL-4 — Groups

```text id="soc225"
membership
roles
hierarchy
```

---

## SOCIAL-5 — Factions

```text id="soc226"
identity
interests
goals
```

---

## SOCIAL-6 — Influence

```text id="soc227"
wealth
knowledge
population
position
```

---

## SOCIAL-7 — Governance

```text id="soc228"
leadership
policies
elections
```

---

## SOCIAL-8 — Diplomacy

```text id="soc229"
relations
treaties
alliances
conflict states
```

---

## SOCIAL-9 — Civilization Integration

```text id="soc230"
civilization politics
internal groups
external factions
```

---

## SOCIAL-10 — Quest

```text id="soc231"
faction quests
social quests
political quests
```

---

## SOCIAL-11 — Economy

```text id="soc232"
trade groups
commercial influence
```

---

## SOCIAL-12 — Knowledge

```text id="soc233"
rumors
propagation
information
```

---

## SOCIAL-13 — LOD

```text id="soc234"
full
regional
abstract
```

---

## SOCIAL-14 — Emergent Factions

```text id="soc235"
formation
merge
split
dissolution
```

---

# 190. Primeiro Vertical Slice

```text id="soc236"
NPC A
 ↓
interacts with NPC B
 ↓
Social Event
 ↓
Relationship created
 ↓
Trust increases
 ↓
Reputation update
 ↓
Persistence
```

---

# 191. Segundo Vertical Slice

```text id="soc237"
NPCs
 ↓
shared profession
 ↓
Group
 ↓
Membership
 ↓
Leader
```

---

# 192. Terceiro Vertical Slice

```text id="soc238"
Group
 ↓
shared interest
 ↓
Faction formed
 ↓
Influence
 ↓
Government interaction
```

---

# 193. Quarto Vertical Slice

```text id="soc239"
Faction A
 ↕
Faction B
 ↓
negotiation
 ↓
treaty
 ↓
trade
 ↓
relationship
```

---

# 194. Quinto Vertical Slice

```text id="soc240"
Faction election
 ↓
candidates
 ↓
NPC preferences
 ↓
vote
 ↓
leader
 ↓
policy
 ↓
world consequence
```

---

# 195. Sexto Vertical Slice

```text id="soc241"
Technology
 ↓
economic disruption
 ↓
worker faction
 ↓
political pressure
 ↓
election
 ↓
policy
```

Esse teste mostra a interação entre:

```text id="4ouu53"
Progression
Economy
Social
Civilization
Quest
```

---

# 196. Sétimo Vertical Slice

```text id="soc242"
World Event
 ↓
Faction response
 ↓
Diplomacy
 ↓
Quest
 ↓
Player intervention
 ↓
new political state
```

---

# 197. Golden Test completo

```text id="soc243"
WORLD EVENT
      ↓
NPCS OBSERVE
      ↓
SOCIAL MEMORY
      ↓
INFORMATION PROPAGATION
      ↓
FACTION REACTION
      ↓
POLITICAL DECISION
      ↓
COMMAND
      ↓
SPECIALIZED SYSTEM
      ↓
WORLD CHANGE
      ↓
NEW SOCIAL EVENT
```

Isso é exatamente o tipo de ciclo que faz o NEXORA parecer um mundo em vez de um conjunto de NPCs esperando o jogador.

---

# 198. Arquitetura final

```text id="soc244"
                         NEXORA
                            │
                     SOCIAL SYSTEM
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
   RELATIONSHIPS          GROUPS            REPUTATION
        │                   │                   │
        ▼                   ▼                   ▼
    MEMORY             MEMBERSHIP          PROPAGATION
        │                   │                   │
        └───────────────────┼───────────────────┘
                            ▼
                         FACTIONS
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
     INTERESTS            GOALS              INFLUENCE
        │                   │                   │
        └───────────────────┼───────────────────┘
                            ▼
                        GOVERNANCE
                            │
                 ┌──────────┴──────────┐
                 ▼                     ▼
              INTERNAL             EXTERNAL
              POLITICS             DIPLOMACY
                 │                     │
                 ▼                     ▼
             ELECTIONS             TREATIES
                 │                     │
                 └──────────┬──────────┘
                            ▼
                         DECISIONS
                            │
                            ▼
                          COMMAND
                            │
                            ▼
                          SERVER
                            │
                            ▼
                       WORLD SYSTEMS
                            │
                            ▼
                         EVENT BUS
                            │
                  ┌─────────┴─────────┐
                  ▼                   ▼
               QUEST              PERSISTENCE
```

E a separação definitiva:

```text id="soc245"
RELATIONSHIP
→ "como A se relaciona com B?"

MEMORY
→ "o que A lembra?"

REPUTATION
→ "como os outros veem A?"

GROUP
→ "quem pertence a quem?"

FACTION
→ "quais interesses coletivos existem?"

INFLUENCE
→ "quem consegue afetar decisões?"

GOVERNANCE
→ "como o grupo toma decisões?"

DIPLOMACY
→ "como grupos se relacionam?"

COMMAND
→ "qual ação foi solicitada?"

SERVER
→ "essa ação é autoritativa?"

EVENT
→ "o que mudou?"
```

## O ponto mais importante para o NEXORA

O sistema não deve produzir apenas:

```text
NPC
↓
REPUTAÇÃO
↓
FAÇÃO
```

Ele deve permitir o ciclo:

```text
                 ┌───────────────┐
                 │    MUNDO      │
                 └───────┬───────┘
                         ▼
                    ACONTECIMENTO
                         │
                         ▼
                  SOCIAL MEMORY
                         │
                         ▼
                 INFORMATION FLOW
                         │
             ┌───────────┴───────────┐
             ▼                       ▼
          NPCS                    FACTIONS
             │                       │
             ▼                       ▼
       RELATIONSHIPS             INTERESTS
             │                       │
             └───────────┬───────────┘
                         ▼
                      POLITICS
                         │
                         ▼
                      DECISION
                         │
                         ▼
                      COMMAND
                         │
                         ▼
                       WORLD
                         │
                         └───────────────→ NOVO EVENTO
```

Isso fecha uma conexão muito importante com tudo que já projetamos:

```text
Entity
   ↓
Social
   ↓
Faction
   ↓
Civilization
   ↓
Economy / Technology / Politics
   ↓
Quest
   ↓
Command
   ↓
Server
   ↓
World
   ↓
Event Bus
   ↓
Social novamente
```

Ou seja: **as sociedades do NEXORA não precisam ser apenas “NPCs decorativos com nomes de facção”. Elas podem formar grupos, desenvolver interesses, disputar influência, fazer alianças, quebrar tratados, criar organizações, mudar de liderança, perder conhecimento, reagir à economia e gerar novas quests como consequência das próprias decisões.**

Isso é justamente o que conecta a ideia do NEXORA de **“o mundo não espera pelo jogador; ele continua vivendo”** ao restante da arquitetura.
