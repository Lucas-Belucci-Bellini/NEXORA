# NEXORA — PROGRESSION / TECHNOLOGY SYSTEM

> **Princípio central:**
> **Progressão não é uma lista de níveis. É a representação do que uma sociedade, jogador, organização ou civilização aprendeu, descobriu, construiu e tornou possível.**

No NEXORA, progressão precisa ser muito maior que:

```text
XP
↓
Level
↓
Unlock
```

Queremos:

```text
EXPERIÊNCIA
+
CONHECIMENTO
+
PESQUISA
+
RECURSOS
+
TECNOLOGIA
+
INFRAESTRUTURA
+
DESCOBERTAS
+
SOCIEDADE
+
EVENTOS
↓
NOVAS POSSIBILIDADES
```

Isso combina diretamente com a ideia central do NEXORA:

> **O mundo não espera pelo jogador. O conhecimento se espalha, sociedades aprendem e a tecnologia evolui.**

---

# 1. O que é o Progression / Technology System?

Esse sistema controla a **evolução das capacidades disponíveis** no mundo.

Pode afetar:

```text
Player
NPC
Profession
Faction
Civilization
Organization
Settlement
Research Institution
Technology
Magic System
Industry
Vehicles
Space
Dimensions
```

Mas ele não deve possuir a lógica dessas coisas.

Ele deve responder:

> **“O que este ator/sociedade conhece, descobriu, domina ou desbloqueou?”**

---

# 2. Progressão não é só do Player

Esse é um dos pontos mais importantes.

```text
PLAYER
  ↓
pode progredir

NPC
  ↓
pode aprender

VILLAGE
  ↓
pode se desenvolver

CITY
  ↓
pode industrializar

CIVILIZATION
  ↓
pode pesquisar

FACTION
  ↓
pode desenvolver tecnologia

WORLD
  ↓
pode passar por eras
```

Portanto:

```text
Progression System
≠
Player Level System
```

---

# 3. Arquitetura

```text id="prog01"
                PROGRESSION SYSTEM
                        │
       ┌────────────────┼────────────────┐
       ▼                ▼                ▼
   KNOWLEDGE         RESEARCH         TECHNOLOGY
       │                │                │
       └────────────────┼────────────────┘
                        ▼
                     UNLOCKS
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
      PLAYER          NPC           CIVILIZATION
        │               │               │
        └───────────────┼───────────────┘
                        ▼
                  CAPABILITIES
                        │
        ┌───────────────┼─────────────────┐
        ▼               ▼                 ▼
      CRAFTING       MACHINES          VEHICLES
        │               │                 │
        └───────────────┼─────────────────┘
                        ▼
                     WORLD
```

---

# 4. Separação principal

Precisamos separar:

```text
Progression
Knowledge
Research
Technology
Unlock
Skill
Mastery
```

Eles são relacionados, mas não iguais.

---

# 5. Progression

Representa:

> evolução de capacidades.

Exemplo:

```text
Mining Tier 3
```

---

# 6. Knowledge

Representa:

> aquilo que um ator sabe.

Exemplo:

```text
NPC knows:
ore exists
```

Mas isso não significa:

```text
NPC knows how to refine it
```

---

# 7. Research

Representa:

> processo de transformar desconhecimento em conhecimento validado.

```text
Observation
↓
Hypothesis
↓
Experiment
↓
Result
↓
Validation
↓
Knowledge
```

---

# 8. Technology

Representa:

> aplicação organizada de conhecimento em capacidade prática.

Exemplo:

```text
Knowledge:
electromagnetic induction

Technology:
generator
```

---

# 9. Unlock

Representa:

> regra de acesso a determinada capacidade.

Exemplo:

```text
Generator unlocked
```

---

# 10. Skill

Representa:

> capacidade do indivíduo de executar uma tarefa.

Exemplo:

```text
Player knows how to make steel
```

não significa necessariamente:

```text
Player is highly skilled at steelworking
```

---

# 11. Mastery

Representa:

> nível de domínio de uma capacidade.

```text
Novice
Competent
Advanced
Expert
Master
```

---

# 12. Arquitetura conceitual

```text id="prog02"
OBSERVATION
    ↓
KNOWLEDGE
    ↓
RESEARCH
    ↓
DISCOVERY
    ↓
TECHNOLOGY
    ↓
UNLOCK
    ↓
CAPABILITY
    ↓
MASTERY
    ↓
NEW DISCOVERY
```

---

# 13. Technology Graph

Em vez de uma árvore simples:

```text
Stone
 ↓
Iron
 ↓
Steel
 ↓
Machines
```

queremos um **grafo tecnológico**.

```text id="prog03"
Metallurgy
   ├────→ Iron
   │        └────→ Steel
   │                 ├────→ Engines
   │                 └────→ Tools
   │
Energy
   ├────→ Electricity
   │        └────→ Motors
   │                 └────→ Automation
   │
Chemistry
   ├────→ Refining
   └────→ Polymers
```

---

# 14. Por que grafo?

Porque tecnologia real possui dependências cruzadas.

Exemplo:

```text id="prog04"
Rocket
```

pode exigir:

```text
Chemistry
+
Metallurgy
+
Energy
+
Navigation
+
Propulsion
+
Precision Manufacturing
```

Não uma única linha.

---

# 15. Technology Node

Cada tecnologia pode possuir:

```text id="prog05"
TechnologyID
Name
Version
Category
Prerequisites
ResearchCost
KnowledgeRequirements
ResourceRequirements
Unlocks
Effects
Tags
```

---

# 16. Technology Definition

Exemplo conceitual:

```yaml id="prog06"
id: nexora:electric_motor

category: engineering

prerequisites:
  - nexora:electricity
  - nexora:magnetism
  - nexora:precision_manufacturing

knowledge:
  - nexora:electromagnetism

unlocks:
  - nexora:motor_machine
```

---

# 17. Technology Categories

Podemos dividir em:

```text id="prog07"
SURVIVAL
AGRICULTURE
MATERIALS
METALLURGY
CHEMISTRY
ENERGY
MECHANICAL
ELECTRICAL
COMPUTING
COMMUNICATION
MEDICINE
BIOLOGY
CONSTRUCTION
TRANSPORT
NAVIGATION
AVIATION
ROCKETRY
SPACE
DIMENSIONAL
MAGIC
AUTOMATION
AI
CIVILIZATION
```

E mods podem adicionar categorias.

---

# 18. Não hardcodar eras

Evitar:

```text id="prog08"
Stone Age
Bronze Age
Iron Age
Modern Age
Future Age
```

como estrutura obrigatória.

Pode existir um **Era System** depois.

Technology Graph deve ser mais flexível.

---

# 19. Eras como emergentes

Uma sociedade pode ser classificada como:

```text id="prog09"
Primitive
Agricultural
Industrial
Electrical
Digital
Spacefaring
Dimensional
```

com base no conjunto de tecnologias que domina.

Então:

```text
ERA = interpretação
```

não necessariamente uma árvore rígida.

---

# 20. Technology Tier

Podemos possuir tiers para organização:

```text id="prog10"
T0
T1
T2
T3
T4
T5
...
```

Mas:

```text
Tier ≠ requisito universal
```

Porque alguns caminhos tecnológicos podem progredir de maneiras diferentes.

---

# 21. Research Points?

Podemos usar recursos abstratos para simplificação do gameplay.

Mas não quero que o sistema seja apenas:

```text
10.000 Research Points
↓
Technology unlocked
```

Porque isso transforma ciência em XP.

Melhor:

```text
Research Capacity
+
Knowledge
+
Experiments
+
Materials
+
Facilities
+
Time
```

---

# 22. Research Capacity

Uma sociedade pode possuir:

```text id="prog11"
ResearchCapacity
```

derivada de:

```text
scientists
universities
laboratories
libraries
infrastructure
funding
equipment
knowledge
communication
```

---

# 23. Research Institution

Uma cidade pode ter:

```text id="prog12"
Research Institution
├── Laboratory
├── University
├── Workshop
├── Observatory
└── Archive
```

Isso conecta diretamente:

```text
Civilization
Structure
Economy
Knowledge
Technology
```

---

# 24. Research Project

Um projeto de pesquisa:

```text id="prog13"
ResearchProject
├── ProjectID
├── Topic
├── Participants
├── Facility
├── Inputs
├── Hypothesis
├── Experiments
├── Progress
├── Confidence
└── Outcome
```

---

# 25. Research não precisa sempre funcionar

Uma pesquisa pode:

```text id="prog14"
succeed
fail
partially succeed
produce unexpected result
discover something else
```

Isso permite ciência emergente.

---

# 26. Discovery

Uma descoberta pode ocorrer sem o jogador.

```text id="prog15"
NPC
 ↓
experiment
 ↓
unexpected result
 ↓
discovery
 ↓
research
 ↓
knowledge
```

---

# 27. Knowledge Propagation

Conhecimento pode viajar por:

```text id="prog16"
NPCs
Merchants
Travelers
Books
Schools
Universities
Guilds
Trade
Diplomacy
Espionage
Communication
Exploration
```

---

# 28. Knowledge Scope

Um conhecimento pode ser:

```text id="prog17"
PERSONAL
FAMILY
PROFESSION
VILLAGE
CITY
FACTION
CIVILIZATION
REGIONAL
GLOBAL
```

---

# 29. Knowledge Confidence

NPC não precisa ter certeza absoluta.

Um conhecimento pode possuir:

```text id="prog18"
confidence
```

Exemplo:

```text
"Existe minério naquela montanha"

confidence = 0.72
```

Depois de confirmação:

```text
confidence = 0.98
```

---

# 30. Rumors

Isso pode produzir:

```text id="prog19"
Rumor
```

que circula antes de ser validado.

Exemplo:

```text
Trader
↓
ouviu que existe uma cidade subterrânea
↓
NPC B
↓
NPC C
```

Até alguém verificar.

---

# 31. Knowledge Conflict

Duas sociedades podem acreditar em coisas diferentes.

```text id="prog20"
Civilization A
→ "teoria X correta"

Civilization B
→ "teoria Y correta"
```

Pesquisa pode resolver.

---

# 32. Technology Adoption

Descobrir tecnologia ≠ adotar tecnologia.

Uma sociedade pode conhecer:

```text
eletricidade
```

mas não possuir:

```text
infraestrutura
recursos
indústria
```

para utilizá-la.

---

# 33. Technology Availability

Separar:

```text id="prog21"
DISCOVERED
RESEARCHED
UNLOCKED
AVAILABLE
ADOPTED
MASTERED
```

---

# 34. Technology Lifecycle

```text id="prog22"
UNKNOWN
 ↓
OBSERVED
 ↓
HYPOTHESIZED
 ↓
RESEARCHING
 ↓
DISCOVERED
 ↓
VALIDATED
 ↓
UNLOCKED
 ↓
AVAILABLE
 ↓
ADOPTED
 ↓
MATURE
 ↓
OBSOLETE
```

---

# 35. Obsolescence

Tecnologias podem tornar-se menos relevantes.

Exemplo:

```text id="prog23"
Steam Engine
 ↓
Internal Combustion
 ↓
Electric
```

Mas antigas podem continuar existindo.

---

# 36. Technology Branching

Exemplo:

```text id="prog24"
Energy
├── Steam
├── Electrical
├── Chemical
├── Nuclear
├── Magical
└── Dimensional
```

Uma civilização pode seguir caminhos diferentes.

---

# 37. Technology Specialization

Uma sociedade pode ser excelente em:

```text id="prog25"
metallurgy
```

mas ruim em:

```text
biology
```

Isso gera diferenças entre civilizações.

---

# 38. Civilization Technology Profile

```text id="prog26"
CivilizationTechnologyProfile
├── Known
├── Researched
├── Available
├── Adopted
├── Mastery
├── Specializations
└── Research Priorities
```

---

# 39. Player Progression

Player pode possuir:

```text id="prog27"
Skills
Knowledge
Discoveries
Research
Mastery
Unlocks
Achievements
Reputation
```

Mas não necessariamente um “level global”.

---

# 40. Skill Model

Categorias:

```text id="prog28"
Mining
Building
Farming
Crafting
Engineering
Combat
Medicine
Navigation
Research
Trading
Agriculture
Mechanics
Programming
Magic
```

Mods podem registrar novas.

---

# 41. Skill Progression

Uma skill pode evoluir por:

```text id="prog29"
practice
training
research
mentoring
books
experiments
repeated tasks
```

---

# 42. Anti-grind

Não queremos:

```text id="prog30"
mine 100.000 stone
=
master engineer
```

Experiência deve depender da ação relevante.

---

# 43. Contextual Learning

Exemplo:

```text id="prog31"
Player repeatedly repairs engines
```

Isso melhora:

```text
Mechanical Maintenance
```

não necessariamente:

```text
Combat
```

---

# 44. Knowledge vs Skill

Exemplo:

```text id="prog32"
Player
Knowledge:
"como fabricar motor"

Skill:
baixa
```

Então consegue fazer, mas com:

```text
higher time
lower quality
higher failure risk
```

dependendo da mecânica.

---

# 45. Mastery

Mastery pode influenciar:

```text id="prog33"
speed
quality
efficiency
failure probability
resource waste
precision
```

---

# 46. Research Discovery para Player

O jogador pode encontrar tecnologia sem simplesmente abrir uma árvore.

```text id="prog34"
Explore
 ↓
Find artifact
 ↓
Analyze
 ↓
Research
 ↓
Discover principle
 ↓
Technology
```

---

# 47. Technology Recipes

Crafting pode exigir:

```text id="prog35"
Recipe
+
TechnologyUnlock
+
Skill
+
Materials
```

---

# 48. Technology vs Recipe

Não confundir.

```text id="prog36"
Technology
→ capability

Recipe
→ procedure
```

---

# 49. Technology vs Item

```text id="prog37"
Technology
→ "capacidade de construir motores"

Item
→ "motor específico"
```

---

# 50. Technology vs Machine

```text id="prog38"
Technology
→ knowledge/capability

Machine
→ implementation
```

---

# 51. Technology vs Infrastructure

Uma sociedade pode possuir:

```text
Technology:
electricity
```

sem possuir:

```text
Grid Infrastructure
```

Portanto:

```text
Technology
+
Infrastructure
=
Adoption
```

---

# 52. Technology Requirements

Um desbloqueio pode exigir:

```text id="prog39"
technology
research
materials
infrastructure
facility
skill
knowledge
resource
location
```

---

# 53. Dynamic Unlocks

Alguns desbloqueios podem surgir de:

```text id="prog40"
world discovery
rare resource
ancient structure
experiment
NPC invention
artifact
dimension
```

---

# 54. Ancient Technology

Muito importante para o mundo.

Uma civilização passada pode deixar:

```text id="prog41"
artifact
blueprint
research notes
machine
ruins
```

O jogador encontra e precisa reconstruir conhecimento.

---

# 55. Reverse Engineering

```text id="prog42"
Ancient Machine
 ↓
Observation
 ↓
Analysis
 ↓
Hypothesis
 ↓
Experiment
 ↓
Partial understanding
 ↓
Technology
```

Não dar a tecnologia instantaneamente.

---

# 56. Technology Loss

Civilizações também podem perder conhecimento.

```text id="prog43"
war
disaster
population collapse
library destruction
isolation
```

podem fazer:

```text
KNOWN
→ LOST
```

---

# 57. Knowledge Recovery

Outra civilização pode redescobrir:

```text id="prog44"
lost technology
```

através de:

```text
ruins
books
archaeology
experiments
trade
```

Isso cria história emergente.

---

# 58. Technology Diffusion

Quando uma civilização entra em contato:

```text id="prog45"
Trade
+
Communication
+
Migration
+
Diplomacy
```

tecnologias podem se espalhar.

---

# 59. Technology Theft

Conhecimento também pode ser obtido por:

```text id="prog46"
espionage
capture
stolen documents
reverse engineering
```

Essa mecânica pode existir sem garantir que tudo seja instantaneamente compreendido.

---

# 60. Technology Secrecy

Uma sociedade pode marcar:

```text id="prog47"
PUBLIC
PRIVATE
RESTRICTED
MILITARY
SACRED
```

---

# 61. Technology Competition

Duas sociedades podem pesquisar:

```text id="prog48"
same technology
```

ao mesmo tempo.

Isso cria corrida tecnológica.

---

# 62. Research Competition

Exemplo:

```text id="prog49"
Civilization A
→ Energy Research

Civilization B
→ Energy Research
```

Uma pode descobrir antes.

---

# 63. Research Failure

Um laboratório pode descobrir:

```text id="prog50"
unexpected material
```

em vez da hipótese original.

O sistema deve permitir resultados emergentes.

---

# 64. Research Result

```text id="prog51"
ResearchOutcome
├── Success
├── Failure
├── Partial
├── UnexpectedDiscovery
├── NewHypothesis
└── NoConclusion
```

---

# 65. Technology Effects

Uma tecnologia pode alterar:

```text id="prog52"
Crafting
Machines
Energy
Fluid
Construction
Vehicles
Weapons
Agriculture
Medicine
AI
Civilization
Economy
Space
```

---

# 66. Technology Effects não devem ser hardcoded

Tecnologia pode fornecer:

```text
Capability
Modifier
RecipeUnlock
MachineUnlock
ResearchUnlock
PolicyOption
```

em vez de:

```text id="prog53"
if technology == "X"
```

espalhado por todo o código.

---

# 67. Capability Registry

Uma tecnologia pode liberar uma capability:

```text id="prog54"
nexora:advanced_metalworking
```

Então:

```text
Recipe
requires capability
```

---

# 68. Technology Tags

```text id="prog55"
#energy
#biology
#industrial
#space
#magic
```

facilitam consultas.

---

# 69. Progression State

Cada ator pode possuir:

```text id="prog56"
ProgressionState
├── Knowledge
├── Research
├── Technologies
├── Skills
├── Masteries
├── Unlocks
├── Discoveries
└── Preferences
```

---

# 70. Player Progression State

```text id="prog57"
PlayerProgression
├── Skills
├── Knowledge
├── Technologies
├── ResearchProjects
├── Discoveries
└── Masteries
```

---

# 71. NPC Progression State

NPC pode possuir uma versão muito menor:

```text id="prog58"
NPCProgression
├── Knowledge
├── Profession
├── Skills
├── LearnedRecipes
└── Experience
```

---

# 72. Civilization Progression

```text id="prog59"
CivilizationProgression
├── Knowledge
├── Technologies
├── ResearchInstitutions
├── Infrastructure
├── Specializations
├── Adoption
└── Policies
```

---

# 73. Faction Progression

```text id="prog60"
FactionProgression
├── Technologies
├── MilitaryDoctrine
├── Industry
├── Research
└── Knowledge
```

---

# 74. Shared Knowledge

Pode existir:

```text id="prog61"
shared technology
```

entre membros de uma sociedade.

---

# 75. Knowledge Ownership

Conhecimento não precisa possuir “dono” absoluto.

Pode haver:

```text id="prog62"
discovered by
known by
controlled by
licensed by
```

---

# 76. Research Facilities

A qualidade da instituição pode afetar:

```text id="prog63"
research speed
accuracy
failure rate
discovery chance
```

Mas sem transformar tudo em um simples multiplicador.

---

# 77. Scientists

NPC researchers podem possuir:

```text id="prog64"
specialization
knowledge
skill
curiosity
reputation
institution
```

---

# 78. Researchers Learn

Um pesquisador pode:

```text id="prog65"
observe
experiment
fail
learn
teach
publish
```

Isso conecta com Knowledge AI.

---

# 79. Teaching

Conhecimento pode ser transferido através de:

```text id="prog66"
teacher
student
book
school
lab
```

---

# 80. Books

Um livro pode conter:

```text id="prog67"
KnowledgeReference
```

mas o leitor ainda precisa possuir capacidade cognitiva/conhecimento adequado para interpretar.

---

# 81. Technology Documentation

Tecnologias podem possuir:

```text id="prog68"
documentation
blueprints
schematics
manuals
research notes
```

---

# 82. Blueprint

Blueprint pode representar:

```text id="prog69"
estrutura
máquina
vehicle
component
```

mas requer:

```text technology
+
materials
+
skills
```

---

# 83. Technology Discovery UI

UI pode mostrar:

```text
Known
Possible
Researching
Locked
Unknown
```

Mas “unknown” não precisa nem aparecer como item.

---

# 84. Fog of Knowledge

O player não precisa saber:

```text id="prog70"
todo o technology graph
```

desde o começo.

Pode descobrir nós gradualmente.

---

# 85. Hidden Technologies

Uma tecnologia pode existir no mundo:

```text id="prog71"
unknown to player
```

e só aparecer após:

```text discovery
```

---

# 86. Technology Metadata

```text id="prog72"
TechnologyDefinition
├── Identity
├── Category
├── Prerequisites
├── Research
├── Unlocks
├── Capabilities
├── Resources
├── Knowledge
├── Facilities
└── DiscoverySources
```

---

# 87. Progression Resolver

Precisamos de um serviço:

```text id="prog73"
IProgressionResolver
```

que responda:

```text
CanUse?
CanResearch?
CanCraft?
CanBuild?
CanAdopt?
IsKnown?
IsUnlocked?
IsMastered?
```

---

# 88. Technology Graph Resolver

```text id="prog74"
ITechnologyGraph
ITechnologyResolver
ITechnologyRepository
```

---

# 89. Requirements

Um requisito pode ser:

```text id="prog75"
TechnologyRequirement
KnowledgeRequirement
SkillRequirement
ResourceRequirement
FacilityRequirement
InfrastructureRequirement
ResearchRequirement
LocationRequirement
EventRequirement
QuestRequirement
```

---

# 90. Requirement Composition

Exemplo:

```text id="prog76"
Rocket

requires:
  Technology:
    propulsion
    metallurgy
    electronics

  Facility:
    advanced workshop

  Infrastructure:
    launch site

  Skill:
    engineering >= 4

  Knowledge:
    orbital mechanics
```

---

# 91. Unlock Conditions

Condições podem ser compostas:

```text id="prog77"
AND
OR
NOT
THRESHOLD
```

Exemplo:

```text
Metallurgy
AND
Electricity
AND
(Advanced Workshop OR Ancient Blueprint)
```

---

# 92. Progression Events

O sistema pode publicar:

```text id="prog78"
KnowledgeDiscovered
ResearchStarted
ResearchCompleted
TechnologyDiscovered
TechnologyUnlocked
TechnologyAdopted
TechnologyMastered
TechnologyLost
SkillImproved
DiscoveryMade
```

---

# 93. Event Bus Integration

```text id="prog79"
ResearchCompleted
 ↓
Event Bus
 ↓
Crafting
Machines
UI
Civilization
Economy
Quest
Networking
```

---

# 94. Persistence

Progression state é persistente:

```text id="prog80"
Player knowledge
NPC knowledge
Civilization tech
Research projects
Discoveries
```

Mas caches de resolução não são.

---

# 95. Networking

Não precisamos replicar tudo.

Para player:

```text id="prog81"
relevant unlocks
research
skill
knowledge
```

Para civilizações distantes:

```text id="prog82"
aggregate technological state
```

---

# 96. LOD

Também:

```text id="prog83"
FULL
REGIONAL
ABSTRACT
```

Exemplo:

### FULL

```text
Researcher A
current experiment
```

### REGIONAL

```text
City research capacity
active projects
```

### ABSTRACT

```text
Civilization technological level
```

---

# 97. Technology Simulation

Uma sociedade distante pode receber:

```text id="prog84"
research ticks
```

em escala abstrata.

Não executar cada cientista individualmente a cada frame.

---

# 98. Progression Scheduler

```text id="prog85"
Player research
→ frequent

NPC learning
→ event/time based

City research
→ minute scale

Civilization research
→ abstract/event-driven
```

---

# 99. Technology Obsolescence

Não apagar automaticamente receitas antigas.

Uma tecnologia pode virar:

```text
obsolete
superseded
legacy
```

mas permanecer utilizável.

---

# 100. Technology Compatibility

Tecnologias podem coexistir:

```text id="prog86"
steam + electric
magic + digital
ancient + modern
```

Isso permite builds únicas.

---

# 101. Tech Hybridization

Exemplo:

```text id="prog87"
Magic
+
Electrical Engineering
=
Arcane Electronics
```

Isso pode ser criado por:

```text
mod
research
discovery
```

---

# 102. Technology Emergence

Talvez nenhuma tecnologia precise estar originalmente planejada.

Sistemas podem gerar novas:

```text id="prog88"
TechnologyDefinition
```

através de pesquisa/procedural discovery controlado.

Isso deve ser mais avançado, mas a arquitetura deve permitir.

---

# 103. Research Generation

Uma área desconhecida:

```text id="prog89"
unknown material
```

pode gerar:

```text
Research Topic
```

que depois vira:

```text
Discovery
```

---

# 104. World Phase

Progressão pode considerar:

```text id="prog90"
WorldPhase
```

como:

```text
Early
Industrial
Advanced
Spacefaring
Dimensional
```

mas isso deve ser uma consequência ou ferramenta de design, não necessariamente um bloqueio global.

---

# 105. Progression + Far Lands

Aqui entra uma das ideias do NEXORA.

```text id="prog91"
Far Lands
 ↓
rare resources
 ↓
research
 ↓
new technology
 ↓
logistics
 ↓
new frontier
```

A fronteira deixa de ser apenas uma área distante.

---

# 106. Progression + Deep World

```text id="prog92"
Deep World
 ↓
new materials
 ↓
new biology
 ↓
new physics
 ↓
research
 ↓
technology
```

---

# 107. Progression + Dimensions

Uma dimensão pode apresentar:

```text id="prog93"
unique knowledge
unique physics
unique materials
```

que desbloqueiam novas tecnologias.

---

# 108. Progression + Space

Mais tarde:

```text id="prog94"
Astronomy
 ↓
Rocketry
 ↓
Orbital
 ↓
Spacecraft
 ↓
Interplanetary
 ↓
Dimensional
```

---

# 109. Technology + Crafting

Crafting consulta:

```text id="prog95"
Technology
Knowledge
Skill
```

e decide se a receita está disponível.

---

# 110. Technology + Machine

Machine Registry pode exigir:

```text id="prog96"
technology capability
```

antes de permitir a construção.

---

# 111. Technology + Energy

Tecnologias diferentes podem habilitar:

```text id="prog97"
generator
battery
reactor
transmission
```

---

# 112. Technology + Fluid

Pode habilitar:

```text id="prog98"
refinery
chemical processor
pressure systems
advanced fluid handling
```

---

# 113. Technology + Vehicles

```text id="prog99"
wheel
engine
electric motor
aircraft
rocket
```

dependem do technology graph.

---

# 114. Technology + AI

AI pode usar Technology Knowledge.

Um NPC engenheiro:

```text id="prog100"
knows engines
```

pode procurar:

```text
engine repair
```

ao invés de possuir conhecimento global.

---

# 115. Civilization + Technology

Uma civilização pode possuir:

```text id="prog101"
technology adoption rate
```

influenciada por:

```text
economy
politics
resources
culture
infrastructure
knowledge
```

---

# 116. Politics

Uma sociedade pode decidir:

```text id="prog102"
invest in science
ignore science
restrict technology
militarize research
commercialize research
```

Isso conecta:

```text
Civilization
Economy
Politics
Technology
```

---

# 117. Technology Policies

Exemplo:

```text
ResearchPriority
TradeRestriction
TechnologySecrecy
EducationPolicy
IndustrialPolicy
```

---

# 118. Technology Economics

Tecnologia altera produtividade.

Mas também possui custo:

```text id="prog103"
research
infrastructure
maintenance
training
resources
```

---

# 119. Technology Maintenance

Uma sociedade avançada precisa manter:

```text id="prog104"
machines
power grid
research facilities
education
supply chains
```

---

# 120. Regression

Civilização pode regredir em determinada área:

```text id="prog105"
knowledge loss
infrastructure loss
resource scarcity
```

sem necessariamente “voltar de era inteira”.

---

# 121. Multiple Progression Paths

Muito importante:

```text id="prog106"
No single correct tech tree.
```

Uma sociedade pode chegar ao mesmo resultado por caminhos diferentes.

Exemplo:

```text
mechanical computer
```

ou:

```text
magical computation
```

dependendo do mundo/mods.

---

# 122. Technology Convergence

Diferentes tecnologias podem fornecer a mesma capability.

```text id="prog107"
Capability:
automated control

Technology A:
mechanical control

Technology B:
electronic control

Technology C:
magical control
```

Isso é muito melhor do que hardcodar “technology X”.

---

# 123. Capability Resolver

Portanto:

```text id="prog108"
Technology
   ↓
Capability
   ↑
Alternative Technology
```

---

# 124. Progression Graph

A estrutura geral:

```text id="prog109"
Knowledge
   │
   ▼
Research
   │
   ▼
Technology
   │
   ▼
Capability
   │
   ├── Recipe
   ├── Machine
   ├── Vehicle
   ├── Structure
   ├── Weapon
   ├── Policy
   └── Profession
```

---

# 125. APIs públicas

```text id="prog110"
IProgressionSystem
IProgressionState
IProgressionResolver

IKnowledgeSystem
IKnowledgeState
IKnowledgeResolver

IResearchSystem
IResearchProject
IResearchInstitution
IResearchResolver

ITechnologySystem
ITechnologyDefinition
ITechnologyGraph
ITechnologyResolver

IUnlockSystem
IRequirement
ICapabilityResolver

ISkillSystem
ISkill
IMasterySystem
```

---

# 126. Registry Integration

Registrar:

```text id="prog111"
Technology
Knowledge Type
Research Type
Skill
Mastery
Capability
Requirement
Unlock
```

via:

```text
Registry System
```

---

# 127. Modding

Mods podem registrar:

```text id="prog112"
technology
research
skill
capability
unlock
requirements
```

Exemplo:

```text
example:quantum_computation
```

sem alterar o Core.

---

# 128. Mod Technology Example

```yaml id="prog113"
id: example:quantum_computation

requires:
  - nexora:advanced_electronics
  - nexora:precision_manufacturing

unlocks:
  - example:quantum_processor
```

---

# 129. Scripting

Scripts podem:

```text id="prog114"
start research
query knowledge
grant discovery
request unlock
simulate research
```

mas não devem simplesmente conceder tudo sem passar pelas permissões apropriadas.

---

# 130. Command System

Comandos:

```text id="prog115"
ResearchCommand
DiscoverCommand
AdoptTechnologyCommand
TrainSkillCommand
StudyKnowledgeCommand
```

---

# 131. Security

Progression precisa proteger:

```text id="prog116"
grant unlock
grant technology
grant skill
grant research
```

especialmente em multiplayer.

Cliente não pode dizer:

```text
"agora tenho todas as tecnologias."
```

---

# 132. Persistence

Salvar:

```text id="prog117"
PlayerProgressionState
CivilizationProgressionState
ResearchProjects
KnowledgeState
DiscoveryHistory
```

com versionamento.

---

# 133. Migration

Quando uma tecnologia é alterada:

```text id="prog118"
Technology v1
 ↓
Migration
 ↓
Technology v2
```

---

# 134. Debug Commands

```text
nexora progression inspect
nexora progression player <id>
nexora progression technology <id>
nexora progression graph
nexora progression unlocks
nexora progression research
nexora progression knowledge
nexora progression simulate
nexora progression grant
nexora progression revoke
```

`grant/revoke` devem exigir privilégios.

---

# 135. Technology Graph Viewer

Ferramenta excelente:

```text id="prog119"
         Electricity
              │
        ┌─────┴─────┐
        ▼           ▼
      Motor       Battery
        │           │
        └─────┬─────┘
              ▼
          Automation
              │
              ▼
          Robotics
```

---

# 136. Research Simulator

Poderíamos ter:

```text
nexora progression simulate research
```

para testar:

```text
100 researchers
10 labs
resource budget
```

e observar:

```text
research completion
failures
discoveries
```

---

# 137. Balance Tools

Útil para desenvolvimento:

```text id="prog120"
technology dependency depth
average unlock cost
dead-end nodes
overpowered path
unused technologies
```

---

# 138. Dead-End Detection

O sistema deve detectar:

```text id="prog121"
technology
→ no unlock
→ no dependency
→ no usage
```

quando isso for um erro de conteúdo.

---

# 139. Cycle Detection

Technology Graph não deve possuir ciclo impossível:

```text id="prog122"
A → B
B → C
C → A
```

O Registry/Technology resolver deve detectar.

---

# 140. Optional Dependencies

Algumas tecnologias podem ter:

```text id="prog123"
A AND B
A OR B
```

---

# 141. Alternative Technologies

Exemplo:

```text
Power Generation
requires:
  (Steam OR Solar OR Magic)
```

Isso cria liberdade.

---

# 142. Technology Synergies

Algumas tecnologias combinadas podem gerar:

```text id="prog124"
bonus capability
```

ou uma nova discovery.

---

# 143. Research Chains

```text id="prog125"
Observation
 ↓
Basic Research
 ↓
Prototype
 ↓
Field Test
 ↓
Production
 ↓
Industrialization
```

---

# 144. Prototype vs Production

Descobrir algo não significa produção em massa.

Podemos ter:

```text id="prog126"
Prototype
Production
Industrialized
```

---

# 145. Technology Maturity

```text id="prog127"
Experimental
Prototype
Early
Mature
Advanced
Obsolete
```

---

# 146. Technology Reliability

Uma tecnologia experimental pode possuir:

```text id="prog128"
higher failure chance
```

e melhorar com conhecimento/experiência.

---

# 147. Technology Certification

Sociedades podem exigir:

```text id="prog129"
certification
training
facility
```

antes de uso.

---

# 148. Profession Integration

Technology altera profissões:

```text id="prog130"
blacksmith
→ machinist
→ engineer
→ robotics engineer
```

Mas não necessariamente substitui imediatamente as profissões antigas.

---

# 149. Civilization Emergence

Uma nova profissão pode surgir porque:

```text technology
+
economic demand
```

desbloqueia uma necessidade.

---

# 150. Technology + Ecology

Tecnologia pode afetar:

```text id="prog131"
pollution
agriculture
deforestation
energy use
water use
```

Isso conecta diretamente com:

```text Climate
Vegetation
Water
Civilization
```

---

# 151. Technology Consequences

Tecnologia nunca deve ser apenas:

```text id="prog132"
+10% speed
```

Ela pode criar:

```text id="prog133"
advantages
costs
risks
externalities
new dependencies
```

---

# 152. Industrialization

Uma sociedade desbloquear:

```text
steam engine
```

pode gerar:

```text
factories
railways
coal demand
pollution
urbanization
```

Então:

```text
Technology
→ Civilization
→ World changes
```

---

# 153. Automation

Uma tecnologia pode reduzir necessidade de:

```text manual labor
```

e aumentar:

```text energy demand
maintenance
specialized jobs
```

---

# 154. Space Technology

Ao chegar:

```text id="prog134"
rocketry
```

podemos abrir:

```text
orbital infrastructure
space stations
satellites
spacecraft
planetary exploration
```

---

# 155. Dimensional Technology

Mais tarde:

```text id="prog135"
dimensional research
```

pode permitir:

```text
portals
dimensional stabilizers
cross-dimensional communication
```

---

# 156. Magic Integration

Magic também pode usar o mesmo modelo:

```text id="prog136"
Knowledge
→ Research
→ Magic Technology
→ Capability
```

Ou seja:

```text
Technology
≠
only machines
```

---

# 157. Unified Capability Model

Essa é uma decisão muito poderosa:

```text id="prog137"
TECHNOLOGY
MAGIC
SKILL
ITEM
RESEARCH
```

podem todos fornecer:

```text
CAPABILITY
```

---

# 158. Capability Example

```text
Capability:
advanced_flight

Sources:
├── aircraft_technology
├── magic_flight
└── alien_antigravity
```

Depois:

```text
Movement System
```

apenas pergunta:

```text
Does actor have advanced_flight?
```

Não precisa conhecer todas as origens.

---

# 159. Isso deixa o NEXORA extensível

Um mod pode criar:

```text
example:psychic_flight
```

que também fornece:

```text
advanced_flight
```

sem alterar Player System.

---

# 160. Performance

Não consultar o grafo inteiro toda vez.

Usar:

```text id="prog138"
resolved capability cache
runtime IDs
compiled requirements
```

---

# 161. Cache

```text id="prog139"
ProgressionCache
TechnologyResolutionCache
CapabilityCache
RequirementCache
```

São derivados.

---

# 162. Invalidation

Quando:

```text id="prog140"
research complete
technology unlocked
knowledge added
skill changed
```

o cache relevante é invalidado.

---

# 163. LOD Performance

Civilizações distantes:

```text id="prog141"
abstract tech state
```

em vez de:

```text
1000 researchers
10000 projects
```

individualmente.

---

# 164. Simulation

```text id="prog142"
FULL:
researcher-level

REGIONAL:
institution-level

ABSTRACT:
civilization-level
```

---

# 165. API de consulta

Exemplos:

```text
isKnown(actor, knowledge)
isResearched(actor, technology)
hasCapability(actor, capability)
canResearch(actor, technology)
canUse(actor, capability)
getMastery(actor, skill)
```

---

# 166. Eventos

```text id="prog143"
KnowledgeAcquiredEvent
ResearchStartedEvent
ResearchCompletedEvent
TechnologyDiscoveredEvent
TechnologyUnlockedEvent
TechnologyAdoptedEvent
TechnologyLostEvent
SkillChangedEvent
MasteryReachedEvent
CapabilityGrantedEvent
CapabilityRevokedEvent
```

---

# 167. Organização do código

```text
src/progression/

├── core/
│   ├── progression-system
│   ├── progression-state
│   └── progression-context
│
├── knowledge/
│   ├── knowledge
│   ├── knowledge-state
│   ├── knowledge-propagation
│   └── confidence
│
├── research/
│   ├── research-system
│   ├── research-project
│   ├── research-institution
│   ├── experiment
│   └── outcome
│
├── technology/
│   ├── technology-definition
│   ├── technology-graph
│   ├── resolver
│   ├── requirements
│   └── maturity
│
├── unlock/
│   ├── unlock
│   └── resolver
│
├── capability/
│   ├── capability
│   └── resolver
│
├── skill/
│   ├── skill
│   ├── mastery
│   └── training
│
├── player/
│
├── npc/
│
├── civilization/
│
├── persistence/
│
├── networking/
│
├── mod/
│
├── debug/
│
└── simulation/
```

---

# 168. Interfaces

```text
IProgressionSystem
IProgressionState
IProgressionResolver

IKnowledgeSystem
IKnowledgeRepository
IKnowledgeResolver

IResearchSystem
IResearchProject
IResearchInstitution
IResearchEngine

ITechnologySystem
ITechnologyGraph
ITechnologyResolver
ITechnologyDefinition

IUnlockSystem
IUnlockResolver

ICapabilitySystem
ICapabilityResolver

ISkillSystem
ISkillResolver
IMasterySystem
```

---

# 169. Implementação por fases

## PROG-0 — Core

```text
ProgressionState
Knowledge
Technology
Capability
```

---

## PROG-1 — Registry

Registrar:

```text
Technology
Knowledge
Skill
Capability
Requirement
```

---

## PROG-2 — Basic Unlocks

Primeiro teste:

```text
condition
 ↓
unlock
 ↓
capability
```

---

## PROG-3 — Technology Graph

```text
nodes
edges
requirements
cycle detection
```

---

## PROG-4 — Player Skills

```text
practice
skill
mastery
```

---

## PROG-5 — Research

```text
project
progress
outcome
```

---

## PROG-6 — Knowledge

```text
acquire
confidence
teach
propagate
```

---

## PROG-7 — Civilization

```text
research institutions
technology adoption
specialization
```

---

## PROG-8 — Crafting Integration

```text
technology
→ recipe
```

---

## PROG-9 — Machine Integration

```text
technology
→ machine
```

---

## PROG-10 — World Integration

```text
technology
→ civilization
→ infrastructure
→ world change
```

---

## PROG-11 — Mod Integration

```text
mod
→ technology
→ capability
```

---

## PROG-12 — Networking / Persistence

```text
save
sync
recovery
```

---

## PROG-13 — Emergent Research

```text
NPC
→ experiment
→ discovery
```

---

## PROG-14 — Advanced Technology

```text
space
magic
dimensional
```

---

# 170. Primeiro Vertical Slice

O primeiro slice deveria ser:

```text id="prog144"
PLAYER
 ↓
learns knowledge
 ↓
researches technology
 ↓
technology unlocked
 ↓
capability granted
 ↓
Crafting sees capability
 ↓
new recipe available
 ↓
player crafts item
```

---

# 171. Segundo Vertical Slice

NPC:

```text id="prog145"
NPC
 ↓
observes phenomenon
 ↓
creates hypothesis
 ↓
research
 ↓
discovery
 ↓
knowledge
 ↓
technology
 ↓
teaches NPC
```

---

# 172. Terceiro Vertical Slice

Civilização:

```text id="prog146"
City
 ↓
builds laboratory
 ↓
researchers
 ↓
technology discovered
 ↓
adopt technology
 ↓
factory created
 ↓
economy changes
 ↓
infrastructure expands
```

---

# 173. Quarto Vertical Slice

Ancient technology:

```text id="prog147"
Player
 ↓
finds ancient device
 ↓
analyzes
 ↓
research
 ↓
partial knowledge
 ↓
reverse engineering
 ↓
new technology
 ↓
new machine
```

---

# 174. Quinto Vertical Slice

Far Lands:

```text id="prog148"
Explorer reaches frontier
 ↓
finds unique resource
 ↓
research
 ↓
new material technology
 ↓
advanced machine
 ↓
new logistics network
```

---

# 175. Golden Test

```text id="prog149"
Knowledge acquired
        ↓
Research started
        ↓
Research completed
        ↓
Technology unlocked
        ↓
Capability granted
        ↓
Recipe unlocked
        ↓
Item crafted
        ↓
Progression saved
        ↓
Server restart
        ↓
State restored
```

---

# 176. Emergent Civilization Test

```text id="prog150"
NPC discovers property
 ↓
Research
 ↓
Technology
 ↓
Factory
 ↓
Production
 ↓
Trade
 ↓
City growth
 ↓
Other civilization notices
 ↓
Knowledge spreads
```

Isso seria um teste fantástico da visão original do NEXORA.

---

# 177. Stress Test

```text id="prog151"
100 players
10.000 NPCs
100 civilizations
100.000 knowledge nodes
100.000 technology nodes
millions of progression states
```

Com:

```text
research
propagation
trade
migration
wars
technology loss
rediscovery
```

---

# 178. Graph Stress

Testar:

```text id="prog152"
10 nodes
1.000
10.000
100.000
```

com:

```text
dependencies
alternative requirements
cycles
mod additions
version migrations
```

---

# 179. Save Stress

```text id="prog153"
huge civilization progression state
+
player progression
+
NPC knowledge
+
active research
```

salvar/carregar e verificar igualdade.

---

# 180. Security Test

Cliente tenta:

```text id="prog154"
grantTechnology
```

Resultado:

```text
DENIED
```

Admin ou sistema autorizado:

```text
ALLOWED
```

---

# 181. Mod Test

Mod adiciona:

```text
example:quantum_drive
```

dependendo de:

```text
nexora:advanced_electronics
```

e fornece:

```text
example:interplanetary_travel
```

sem alterar o Progression Core.

---

# 182. Scripting Test

Script:

```text
observe research completion
 ↓
execute command
```

usando:

```text
Event Bus
+
Command System
```

---

# 183. Architecture final

```text
                         NEXORA
                            │
                        PROGRESSION
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
     KNOWLEDGE           RESEARCH           TECHNOLOGY
        │                   │                   │
        │                   │                   ▼
        │                   │              REQUIREMENTS
        │                   │                   │
        └───────────────────┼───────────────────┘
                            ▼
                         UNLOCK
                            │
                            ▼
                       CAPABILITY
                            │
        ┌───────────────────┼────────────────────┐
        ▼                   ▼                    ▼
      PLAYER               NPC              CIVILIZATION
        │                   │                    │
        ▼                   ▼                    ▼
      SKILL              LEARNING            ADOPTION
        │                   │                    │
        └───────────────────┼────────────────────┘
                            ▼
                    GAMEPLAY SYSTEMS
                            │
       ┌─────────┬──────────┼──────────┬──────────┐
       ▼         ▼          ▼          ▼          ▼
   CRAFTING   MACHINES   VEHICLES    ECONOMY    WORLD
       │         │          │          │          │
       └─────────┴──────────┼──────────┴──────────┘
                            ▼
                       EVENT BUS
                            │
                  ┌─────────┴─────────┐
                  ▼                   ▼
              NETWORKING         PERSISTENCE
```

E a regra definitiva:

```text
KNOWLEDGE
→ "o que eu sei?"

RESEARCH
→ "o que estou tentando descobrir?"

TECHNOLOGY
→ "o que sei transformar em capacidade?"

UNLOCK
→ "o que agora posso acessar?"

CAPABILITY
→ "o que consigo fazer?"

SKILL
→ "quão bem consigo fazer?"

MASTERY
→ "quão profundamente domino?"

INFRASTRUCTURE
→ "consigo realmente aplicar isso em escala?"

CIVILIZATION
→ "como isso muda a sociedade?"
```

A parte mais importante para o NEXORA é que **não deve existir uma única “Tech Tree do jogador”**. O mesmo sistema precisa conseguir representar um minerador aprendendo uma técnica, um NPC descobrindo uma propriedade de um material, uma cidade construindo um laboratório, uma civilização desenvolvendo eletricidade, outra redescobrindo uma tecnologia perdida e um mod adicionando uma nova linha tecnológica — tudo usando os mesmos contratos.

Isso também prepara uma das partes mais interessantes do NEXORA: **Knowledge + Technology + Civilization + Economy + World Events**, onde a progressão deixa de ser uma tela de desbloqueios e passa a ser uma propriedade viva do mundo.
