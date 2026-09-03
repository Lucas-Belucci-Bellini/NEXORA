# NEXORA — RESEARCH / KNOWLEDGE FORMALIZATION SYSTEM

> **Princípio central:**
> **Conhecimento é algo que o mundo descobre, registra, verifica, transmite, contesta, perde e redescobre. Pesquisa é o processo que transforma observações em conhecimento utilizável.**
>
> O sistema não deve tratar ciência como:
>
> ```text
> XP
> ↓
> 100 pontos
> ↓
> tecnologia desbloqueada
> ```
>
> Mas como:
>
> ```text
> OBSERVATION
>      ↓
> DATA
>      ↓
> HYPOTHESIS
>      ↓
> EXPERIMENT
>      ↓
> RESULT
>      ↓
> EVIDENCE
>      ↓
> ANALYSIS
>      ↓
> KNOWLEDGE
>      ↓
> VALIDATION
>      ↓
> TECHNOLOGY / CAPABILITY
> ```
>
> E o mais importante:
>
> **o conhecimento pode estar incompleto, errado, perdido, ser contestado e ser descoberto independentemente por diferentes sociedades.**

---

# 1. O que esse sistema resolve?

Esse sistema formaliza:

```text
Research
Knowledge
Evidence
Observation
Hypothesis
Experiment
Analysis
Discovery
Verification
Theory
Documentation
Learning
Teaching
Knowledge Propagation
Uncertainty
Conflicting Knowledge
Knowledge Loss
Knowledge Recovery
Scientific Institutions
Research Projects
Researcher Specialization
```

Ele se conecta diretamente a:

```text
Progression
Technology
Civilization
AI
NPC Ecology
Economy
Quest
Structure
World Generation
Dimensions
Space
Magic
```

---

# 2. Research ≠ Progression

A separação que fizemos anteriormente fica ainda mais importante.

```text
RESEARCH
→ processo de descoberta

KNOWLEDGE
→ aquilo que foi aprendido

PROGRESSION
→ evolução das capacidades

TECHNOLOGY
→ aplicação prática do conhecimento
```

Exemplo:

```text
Research:
estudar um minério estranho

↓

Knowledge:
"o minério conduz energia de determinada forma"

↓

Technology:
"condutor avançado"

↓

Capability:
"criar circuito de alta eficiência"

↓

Machine:
"novo gerador"
```

---

# 3. Knowledge ≠ Truth

Essa é uma decisão importantíssima.

Um NPC pode acreditar que:

```text
A causa do fenômeno = X
```

e estar errado.

Portanto:

```text
BELIEF
≠
KNOWLEDGE
≠
VERIFIED KNOWLEDGE
≠
TRUTH
```

O mundo pode possuir um estado físico real, enquanto os agentes possuem conhecimento parcial sobre ele.

---

# 4. Arquitetura

```text
                    RESEARCH / KNOWLEDGE
                              │
       ┌──────────────────────┼──────────────────────┐
       ▼                      ▼                      ▼
   OBSERVATION             RESEARCH              KNOWLEDGE
       │                      │                      │
       ▼                      ▼                      ▼
      DATA                 PROJECTS              CLAIMS
       │                      │                      │
       ▼                      ▼                      ▼
   EVIDENCE              EXPERIMENTS            CONFIDENCE
       │                      │                      │
       └──────────────────────┼──────────────────────┘
                              ▼
                           ANALYSIS
                              │
                              ▼
                          VALIDATION
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
               KNOWLEDGE             THEORY
                    │                   │
                    └─────────┬─────────┘
                              ▼
                       PROPAGATION
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
        PLAYER               NPC              CIVILIZATION
```

---

# 5. Knowledge Object

O conhecimento precisa ser um objeto formal.

```text
KnowledgeRecord
├── KnowledgeID
├── Subject
├── Claims
├── Evidence
├── Confidence
├── Validity
├── Origin
├── Discoverers
├── KnownBy
├── Sources
├── Version
└── History
```

---

# 6. Knowledge Claim

Uma unidade de conhecimento pode ser uma afirmação:

```text
KnowledgeClaim
```

Exemplo:

```text
"Material X conducts electricity better when cooled."
```

O sistema armazena essa afirmação como dado estruturado, não como uma simples frase.

---

# 7. Claim Structure

```text
Claim
├── Subject
├── Property
├── Relation
├── Conditions
├── Scope
├── Evidence
├── Confidence
└── Status
```

---

# 8. Knowledge States

Uma afirmação pode estar:

```text
UNKNOWN
SUSPECTED
HYPOTHESIS
PROVISIONAL
SUPPORTED
VERIFIED
DISPUTED
REFUTED
OBSOLETE
LOST
RECONSTRUCTED
```

---

# 9. Isso permite ciência imperfeita

Exemplo:

```text
NPC A
→ hipótese X
→ confidence 0.42

NPC B
→ hipótese Y
→ confidence 0.63
```

Depois de um experimento:

```text
X
→ confidence 0.85

Y
→ confidence 0.21
```

E eventualmente:

```text
X → VERIFIED
Y → REFUTED
```

---

# 10. Belief System

Knowledge formalization precisa distinguir:

```text
Belief
Claim
Evidence
```

Um NPC pode possuir:

```text
Belief:
"tempestade foi causada por entidade X"

Evidence:
nenhuma

Confidence:
0.31
```

Isso não vira conhecimento científico automaticamente.

---

# 11. Knowledge Scope

Conhecimento possui alcance.

```text
PERSONAL
HOUSEHOLD
PROFESSION
GROUP
SETTLEMENT
FACTION
REGION
CIVILIZATION
GLOBAL
INTERDIMENSIONAL
```

---

# 12. Knowledge Ownership

Conhecimento pode ser:

```text
PUBLIC
PRIVATE
RESTRICTED
SECRET
MILITARY
ACADEMIC
COMMERCIAL
SACRED
```

---

# 13. Knowledge Provenance

Cada conhecimento deve responder:

> De onde veio isso?

Exemplo:

```text
Knowledge:
advanced metallurgy

Origin:
Research Project #832

Researchers:
NPC 182
NPC 734

Location:
University A

Evidence:
Experiment #491
Sample #83

Date:
Year 412
```

Isso combina muito com a preocupação de **proveniência** que você quer para o código do NEXORA.

---

# 14. Evidence

Evidence é uma das partes centrais.

```text
Evidence
├── EvidenceID
├── Type
├── Source
├── Data
├── Reliability
├── Context
├── Collector
├── Timestamp
└── Provenance
```

---

# 15. Evidence Types

```text
OBSERVATION
MEASUREMENT
SAMPLE
DOCUMENT
EXPERIMENT
REPRODUCTION
WITNESS
ARTIFACT
RECORD
SIMULATION
ANALOGY
```

---

# 16. Evidence Reliability

Cada evidência pode possuir:

```text
reliability
quality
independence
repeatability
```

---

# 17. Evidence não é verdade

Uma observação pode ser:

```text
válida
```

mas a interpretação:

```text
errada
```

Exemplo:

```text
Observation:
"material ficou quente"

Wrong hypothesis:
"material gera calor espontaneamente"

Later:
energy source discovered
```

---

# 18. Observation System

Research precisa registrar observações.

```text
Observation
├── Observer
├── Subject
├── Location
├── Conditions
├── Measurement
├── Timestamp
├── Tools
└── Reliability
```

---

# 19. Observations podem ser automáticas

Não apenas NPC.

```text
Sensor
Machine
Satellite
Scanner
Player
Animal
Researcher
```

podem gerar observações.

---

# 20. Research Question

Pesquisa começa com uma pergunta.

```text
ResearchQuestion
```

Exemplos:

```text
"Por que essa planta cresce apenas em cavernas?"
"Por que esse minério conduz energia?"
"De onde veio essa estrutura?"
"Como funciona essa máquina antiga?"
```

---

# 21. Hypothesis

Uma pesquisa pode possuir múltiplas hipóteses.

```text
ResearchQuestion
      │
 ┌────┼────┐
 ▼    ▼    ▼
 H1   H2   H3
```

---

# 22. Hypothesis Structure

```text
Hypothesis
├── HypothesisID
├── Claims
├── Predictions
├── Preconditions
├── Confidence
├── Evidence
└── Status
```

---

# 23. Prediction

Uma hipótese deve poder prever resultados.

Exemplo:

```text
H1:
"Temperature changes material conductivity."

Prediction:
cool sample → conductivity rises
```

---

# 24. Experiment

```text
Experiment
├── ExperimentID
├── ResearchProject
├── Hypothesis
├── Procedure
├── Inputs
├── Environment
├── Equipment
├── Participants
├── Measurements
├── Results
└── Reproducibility
```

---

# 25. Experiment ≠ instant unlock

O resultado não deve automaticamente gerar:

```text
Technology unlocked
```

O fluxo é:

```text
Experiment
↓
Result
↓
Analysis
↓
Evidence
↓
Knowledge
↓
Validation
↓
Technology
```

---

# 26. Experiment Outcomes

```text
CONFIRMS
PARTIALLY_CONFIRMS
REFUTES
INCONCLUSIVE
UNEXPECTED
CONTAMINATED
FAILED
```

---

# 27. Unexpected Discovery

Essa parte é muito importante para o NEXORA.

```text
Experiment
 ↓
unexpected measurement
 ↓
new observation
 ↓
new research question
```

Ou:

```text
Experiment
 ↓
unknown phenomenon
 ↓
new Knowledge Topic
```

---

# 28. Research Project

```text
ResearchProject
├── ProjectID
├── Topic
├── Questions
├── Hypotheses
├── Researchers
├── Institution
├── Resources
├── Experiments
├── Results
├── Evidence
├── Conclusions
└── Status
```

---

# 29. Research Lifecycle

```text
PROPOSED
↓
PLANNED
↓
ACTIVE
↓
EXPERIMENTAL
↓
ANALYSIS
↓
CONCLUSION
↓
VALIDATION
↓
PUBLISHED
↓
ARCHIVED
```

Pode terminar em:

```text
FAILED
ABANDONED
DISPROVEN
LOST
```

---

# 30. Research Institution

Uma cidade pode possuir:

```text
University
Laboratory
Observatory
Workshop
Archive
Medical Center
Engineering Institute
```

Cada um pode possuir especialização.

---

# 31. Institution Profile

```text
ResearchInstitution
├── Specializations
├── Facilities
├── Researchers
├── Equipment
├── Funding
├── Reputation
├── Knowledge
└── Projects
```

---

# 32. Research Capacity

Não usar simplesmente:

```text
100 Research Points
```

Podemos calcular capacidade através de:

```text
Researchers
Facilities
Equipment
Knowledge
Funding
Communication
Materials
Time
Institution Quality
```

---

# 33. Research Cost

Projetos podem consumir:

```text
time
materials
energy
fluid
samples
equipment
labor
money
```

---

# 34. Research Failure

Mesmo com bons recursos:

```text
experiment failed
```

pode acontecer.

Isso não precisa ser simplesmente “azar”.

Pode gerar:

```text
new evidence
```

---

# 35. Research Skill

Pesquisadores podem possuir:

```text
specialization
skill
experience
knowledge
```

---

# 36. Research Collaboration

Vários pesquisadores podem trabalhar juntos:

```text
Researcher A
+
Researcher B
+
Researcher C
```

e aumentar:

```text
breadth
verification
speed
```

dependendo do campo.

---

# 37. Research Competition

Instituições podem competir.

```text
University A
vs
University B
```

ambas pesquisando:

```text
same technology
```

---

# 38. Independent Discovery

Muito importante.

Duas civilizações podem descobrir a mesma coisa independentemente.

```text
Civilization A
→ discovers electricity

Civilization B
→ discovers electricity
```

Essas descobertas podem possuir:

```text
independent provenance
```

---

# 39. Convergent Discovery

Mesmo princípio pode surgir por caminhos diferentes.

```text
Magic observation
→ energy control

Scientific experiment
→ electrical control
```

Ambos podem liberar capabilities relacionadas.

---

# 40. Knowledge Merge

Duas linhas de pesquisa podem depois se combinar.

```text
Knowledge A
+
Knowledge B
↓
New hypothesis
```

---

# 41. Knowledge Graph

Aqui fica o coração formal.

```text
            Material X
                │
          has property
                │
                ▼
         High conductivity
                │
          enables research
                │
                ▼
          Electromagnetism
                │
          enables technology
                │
                ▼
             Generator
```

---

# 42. Knowledge Graph Nodes

```text
Phenomenon
Material
Species
Location
Property
Process
Theory
Technology
Structure
Artifact
Entity
```

---

# 43. Knowledge Graph Edges

```text
OBSERVED
SUPPORTS
REFUTES
CAUSES
CORRELATES
REQUIRES
DERIVED_FROM
DISCOVERED_BY
KNOWN_BY
TEACHES
USES
ENABLES
```

---

# 44. Knowledge Graph ≠ Technology Graph

Technology Graph:

```text
capability dependency
```

Knowledge Graph:

```text
relationships between what is known
```

---

# 45. Research Graph

Podemos manter:

```text
Knowledge Graph
Research Graph
Technology Graph
```

relacionados, mas separados.

---

# 46. Formalization Pipeline

```text
RAW OBSERVATION
      ↓
STRUCTURED DATA
      ↓
CLAIM
      ↓
EVIDENCE
      ↓
HYPOTHESIS
      ↓
EXPERIMENT
      ↓
RESULT
      ↓
ANALYSIS
      ↓
KNOWLEDGE
      ↓
VALIDATION
      ↓
TECHNOLOGY
```

---

# 47. Knowledge Confidence

Uma afirmação pode possuir:

```text
confidence = 0.74
```

Mas não devemos transformar isso em uma verdade matemática absoluta.

É um indicador do estado epistemológico da sociedade.

---

# 48. Confidence Factors

Pode depender de:

```text
evidence quality
reproducibility
source reliability
independent confirmation
sample size
observer expertise
contradictory evidence
```

---

# 49. Reproducibility

Experimento importante pode ser repetido.

```text
Experiment A
→ success

Experiment B
→ success

Experiment C
→ success
```

A confiança aumenta.

---

# 50. Contradictory Evidence

Se aparecer:

```text
Evidence contradicting Claim
```

o conhecimento pode virar:

```text
DISPUTED
```

ou ter confidence reduzida.

---

# 51. Scientific Disagreement

Duas instituições podem discordar.

```text
Institution A
→ Theory X

Institution B
→ Theory Y
```

O mundo continua funcionando.

Não precisamos escolher instantaneamente uma “verdade de gameplay”.

---

# 52. Truth Model

Para evitar inconsistência, o mundo possui:

```text
WORLD STATE
```

que representa o comportamento real.

Enquanto agentes possuem:

```text
KNOWLEDGE STATE
```

que representa o que acreditam/sabem.

---

# 53. Exemplo

Mundo:

```text
Phenomenon is caused by X
```

NPC:

```text
believes Y
```

Depois de pesquisa:

```text
learns X
```

Isso é muito melhor do que o NPC nascer sabendo X.

---

# 54. Hidden World Knowledge

O servidor sabe:

```text
actual world state
```

mas não precisa revelar isso para:

```text
players
NPCs
factions
```

---

# 55. Information Access

Uma entidade pode possuir:

```text
knowledge access
```

determinado por:

```text
location
observation
communication
research
teaching
documents
technology
```

---

# 56. Knowledge Acquisition

Pode acontecer via:

```text
observation
research
education
book
conversation
trade
exploration
experiment
artifact
technology
```

---

# 57. Teaching

```text
Teacher
 ↓
Knowledge Package
 ↓
Student
```

Student pode não absorver tudo imediatamente.

---

# 58. Learning Quality

Depende de:

```text
teacher expertise
student skill
language
resources
time
prior knowledge
```

---

# 59. Knowledge Packages

Um professor pode transmitir:

```text
KnowledgeSet
```

com vários claims.

---

# 60. Books

Livro pode carregar:

```text
KnowledgeReference
```

Não necessariamente “desbloqueia tecnologia” instantaneamente.

---

# 61. Translation

Knowledge pode enfrentar barreiras:

```text
Language A
→ Language B
```

com risco de:

```text
loss
distortion
misinterpretation
```

---

# 62. Communication Delay

Conhecimento pode levar tempo para chegar:

```text
City A
 ↓
merchant
 ↓
City B
 ↓
Faction
```

---

# 63. Knowledge Propagation

O sistema pode modelar:

```text
speed
distance
trust
communication
literacy
politics
secrecy
```

---

# 64. Knowledge Secrecy

Instituições podem restringir:

```text
classified research
military technology
commercial process
```

---

# 65. Knowledge Theft

Uma sociedade pode adquirir conhecimento através de:

```text
stolen documents
captured artifact
spies
defectors
trade
```

---

# 66. Knowledge Diffusion

Conhecimento pode espalhar-se naturalmente:

```text
Researcher
 ↓
student
 ↓
institution
 ↓
city
 ↓
civilization
```

---

# 67. Knowledge Bottleneck

Uma descoberta pode existir, mas apenas uma pessoa saber.

```text
NPC A
→ knows technique
```

Se:

```text
NPC A
dies
```

conhecimento pode ser:

```text
lost
```

caso não tenha sido documentado.

---

# 68. Knowledge Loss

Causas:

```text
death
war
library destruction
isolation
language loss
institution collapse
```

---

# 69. Knowledge Preservation

Pode existir:

```text
books
archives
libraries
records
artifacts
digital storage
oral tradition
```

---

# 70. Knowledge Recovery

Séculos depois:

```text
ruins
 ↓
documents
 ↓
research
 ↓
reconstruction
```

---

# 71. Rediscovery

O sistema deve diferenciar:

```text
DISCOVERY
```

de:

```text
REDISCOVERY
```

Exemplo:

```text
Civilization A invented X
↓
knowledge lost
↓
Civilization B rediscovers X
```

---

# 72. Provenance Chain

Pode registrar:

```text
Original Discovery
      ↓
Documentation
      ↓
Propagation
      ↓
Loss
      ↓
Rediscovery
```

---

# 73. Knowledge Versioning

O mesmo conceito pode evoluir:

```text
Knowledge v1
↓
improved measurement
↓
Knowledge v2
```

---

# 74. Scientific Theories

Para conhecimentos complexos:

```text
Theory
├── Claims
├── Supporting Evidence
├── Predictions
├── Competing Theories
└── Confidence
```

---

# 75. Theory Evolution

```text
Theory A
 ↓
anomaly
 ↓
revision
 ↓
Theory B
```

---

# 76. Anomalies

Uma observação que a teoria atual não explica:

```text
ANOMALY
```

pode criar:

```text
Research Question
```

automaticamente.

---

# 77. Discovery Engine

Precisamos de:

```text
IDiscoveryEngine
```

que identifica:

```text
unexpected patterns
novel phenomena
unexplained observations
```

---

# 78. Research Topic Generator

Pode produzir:

```text
ResearchTopic
```

a partir de:

```text
WorldObservation
KnowledgeGap
Anomaly
PlayerDiscovery
NPCCuriosity
```

---

# 79. Knowledge Gaps

Uma instituição pode detectar:

```text
known:
A

unknown:
relationship A → B
```

e gerar pesquisa.

---

# 80. Scientific Method

O sistema pode representar:

```text
Observation
→ Question
→ Hypothesis
→ Prediction
→ Experiment
→ Result
→ Analysis
→ Conclusion
→ Reproduction
→ Publication
```

---

# 81. Publication

Research pode ser publicada.

```text
ResearchPublication
├── Author
├── Institution
├── Claims
├── Evidence
├── Method
├── Results
└── Reliability
```

---

# 82. Publication não significa verdade

Uma publicação pode ser:

```text
wrong
incomplete
biased
correct
```

---

# 83. Peer Review

Instituições podem revisar:

```text
Publication
```

gerando:

```text
Review
```

---

# 84. Review

```text
Review
├── Reviewer
├── Claim
├── Evidence
├── Criticism
├── Confidence
└── Recommendation
```

---

# 85. Academic Reputation

Pesquisadores/instituições podem ganhar:

```text
Research Reputation
```

baseada em:

```text
successful discoveries
reproducibility
publications
peer review
```

Isso conversa com Social System.

---

# 86. Scientific Prestige

Não é necessariamente poder político.

Mas pode gerar:

```text
Influence
Funding
Students
Collaboration
```

---

# 87. Funding

Economy pode financiar:

```text
ResearchProject
```

```text
Economy
 ↓
Funding
 ↓
Research
```

---

# 88. Research Economics

Projetos podem competir por:

```text
budget
materials
researchers
labs
```

---

# 89. Research Priorities

Instituições podem escolher:

```text
Energy
Medicine
Agriculture
Space
Materials
```

---

# 90. Civilization Research Policy

Uma civilização pode definir:

```text
ResearchPriority
```

por exemplo:

```text
Military = high
Medicine = high
Space = low
```

---

# 91. Emergent Research

Isso é importantíssimo para o NEXORA.

O jogador não precisa entregar:

```text
"Research This"
```

para tudo.

Um NPC pode encontrar:

```text
new material
```

e o sistema gerar automaticamente:

```text
ResearchQuestion
```

---

# 92. Research Trigger Sources

```text
World Event
Observation
Anomaly
Artifact
Technology
Disease
Ecology
Climate
Space
Dimension
Player Action
NPC Action
```

---

# 93. Medicine Example

```text
NPC
 ↓
observes illness
 ↓
records symptoms
 ↓
hypothesis
 ↓
experiment
 ↓
treatment
 ↓
knowledge
 ↓
medicine technology
```

---

# 94. Ecology Example

```text
Researchers notice
plant grows differently
 ↓
collect samples
 ↓
study climate
 ↓
discover soil relationship
 ↓
agricultural knowledge
```

---

# 95. Astronomy Example

```text
Observatory
 ↓
unknown object detected
 ↓
tracking
 ↓
analysis
 ↓
new celestial knowledge
 ↓
space research
```

---

# 96. Dimension Example

```text
Explorer
 ↓
enters new dimension
 ↓
measures gravity
 ↓
unexpected physics
 ↓
research
 ↓
new physics knowledge
```

---

# 97. Magic Example

Mesmo magia pode seguir formalização:

```text
phenomenon
 ↓
observation
 ↓
ritual hypothesis
 ↓
experiment
 ↓
repeatability
 ↓
formalized magical knowledge
```

Assim magia também pode evoluir.

---

# 98. Technology Bridge

No final:

```text
Verified Knowledge
       ↓
Technology Candidate
       ↓
Engineering
       ↓
Prototype
       ↓
Production
```

---

# 99. Research → Technology

Uma pesquisa não precisa liberar tecnologia imediatamente.

Pode gerar:

```text
TechnologyCandidate
```

---

# 100. Technology Candidate

```text
TechnologyCandidate
├── RequiredKnowledge
├── RequiredMaterials
├── RequiredFacilities
├── Prototype
└── ProductionRequirements
```

---

# 101. Engineering

Outra camada pode transformar conhecimento em aplicação.

```text
Research
 ↓
Knowledge
 ↓
Engineering
 ↓
Prototype
 ↓
Technology
```

---

# 102. Research vs Engineering

```text
Research
→ "como isso funciona?"

Engineering
→ "como podemos usar isso?"
```

---

# 103. Reverse Engineering

Pode começar de:

```text
Artifact
 ↓
Observation
 ↓
Measurements
 ↓
Hypothesis
 ↓
Reconstruction
```

---

# 104. Ancient Knowledge

Um artefato antigo pode conter:

```text
compressed knowledge
```

mas o jogador ainda precisa interpretá-lo.

---

# 105. Knowledge Difficulty

Compreender algo depende de:

```text
literacy
prior knowledge
language
technology
equipment
skill
```

---

# 106. Formalization Level

Podemos ter:

```text
RAW_OBSERVATION
DESCRIPTIVE
STRUCTURED
HYPOTHETICAL
EXPERIMENTAL
SUPPORTED
FORMALIZED
VERIFIED
APPLIED
```

---

# 107. Knowledge Quality

Uma formalização avançada possui:

```text
clear conditions
repeatability
measurement
evidence
```

---

# 108. Data vs Knowledge

Isso é importante.

```text
DATA
→ "temperatura = 73"

KNOWLEDGE
→ "material changes behavior above temperature threshold"
```

---

# 109. Data Collection

Sensors podem gerar:

```text
Measurement
```

Research converte medidas em relações.

---

# 110. Knowledge Compiler

Podemos ter futuramente um:

```text
KnowledgeCompiler
```

que transforma:

```text
validated knowledge
```

em:

```text
capabilities
constraints
technology unlocks
AI knowledge
```

---

# 111. AI Knowledge Interface

AI deveria poder consultar:

```text
What do I know?
How confident am I?
What am I uncertain about?
What evidence do I have?
What should I investigate?
```

---

# 112. AI Research Planning

Um pesquisador NPC pode escolher:

```text
research question
```

baseando-se em:

```text
importance
uncertainty
resources
curiosity
institution policy
expected value
```

---

# 113. Research Prioritization

```text
ResearchPriority =
importance
+
uncertainty
+
potential_value
-
cost
-
risk
```

como modelo conceitual, não como fórmula fixa.

---

# 114. Curiosity

NPCs pesquisadores podem ter:

```text
Curiosity
```

influenciando o que estudam.

---

# 115. Research Personality

Um pesquisador pode ser:

```text
Risk-taking
Conservative
Experimental
Theoretical
Practical
```

e produzir linhas de pesquisa diferentes.

---

# 116. Institutional Bias

Instituições podem priorizar:

```text
military
economic
medical
scientific
religious
environmental
```

---

# 117. Scientific Competition

Isso pode produzir:

```text
Research Race
```

sem precisar de um sistema separado no início.

---

# 118. Research Race

```text
Institution A
     │
Institution B
     │
     └── same discovery
```

Quem valida primeiro pode obter:

```text
prestige
technology advantage
economic advantage
```

---

# 119. Knowledge Monopoly

Uma organização pode tentar controlar uma descoberta.

```text
Research Group
 ↓
Discovery
 ↓
Secret
```

---

# 120. Knowledge Diffusion vs Monopoly

Duas forças:

```text
DIFFUSION
vs
SECRECY
```

---

# 121. Knowledge Policies

Civilization pode definir:

```text
Open Science
Restricted Research
Military Secrecy
Commercial Patent
Religious Protection
```

---

# 122. Patents / Intellectual Property do mundo

Pode haver uma camada econômica/jurídica fictícia:

```text
Patent
```

representando:

```text
technology rights
commercial exclusivity
licensing
```

Economy/Politics podem utilizar isso.

---

# 123. Licensing

Uma tecnologia pode ser:

```text
public
licensed
restricted
proprietary
```

---

# 124. Knowledge Exchange

Sociedades podem trocar:

```text
KnowledgePackage
```

por:

```text
currency
resources
diplomacy
technology
```

---

# 125. Knowledge Network

```text
Researcher
 ↓
Institution
 ↓
City
 ↓
Faction
 ↓
Civilization
```

---

# 126. Knowledge Propagation Model

Propagação pode considerar:

```text
distance
communication
trust
language
literacy
secrecy
politics
```

---

# 127. Knowledge Decay

Algum conhecimento pode perder precisão:

```text
poor oral transmission
```

mas conhecimento formalizado em:

```text
book
archive
```

pode persistir melhor.

---

# 128. Knowledge Compression

Civilizações distantes podem ser simuladas como:

```text
abstract knowledge state
```

em vez de armazenar cada detalhe de cada pesquisador.

---

# 129. LOD

```text
FULL
REGIONAL
ABSTRACT
```

### FULL

```text
individual researcher
experiment
measurement
```

### REGIONAL

```text
institution
project
research field
```

### ABSTRACT

```text
civilization knowledge level
```

---

# 130. Scheduler

```text
Researcher analysis
→ periodic

Lab experiment
→ event/time based

Institution
→ minute/hour

Civilization
→ abstract/event driven
```

---

# 131. Persistence

Persistir:

```text
Knowledge Claims
Important Evidence
Active Research Projects
Major Discoveries
Publications
Research Institutions
Critical History
```

Não necessariamente todos os dados transitórios.

---

# 132. Knowledge Migration

Se a estrutura de conhecimento mudar:

```text
Knowledge Schema v1
 ↓
Migration
 ↓
Knowledge Schema v2
```

---

# 133. Networking

Para o player:

```text
known knowledge
active research
important discoveries
```

Para civilizações:

```text
relevant aggregate knowledge
```

---

# 134. Security

Cliente não pode:

```text
grantKnowledge
```

sem autoridade.

---

# 135. Anti-Exploit

Operações como:

```text
CompleteResearch
GrantTechnology
PublishDiscovery
```

passam pelo:

```text
Command System
+
Server
```

---

# 136. Event Bus

Eventos:

```text
ObservationCreated
ResearchStarted
ExperimentCompleted
EvidenceRecorded
HypothesisCreated
HypothesisSupported
HypothesisRefuted
KnowledgeFormalized
KnowledgeVerified
KnowledgeDisputed
KnowledgeLost
KnowledgeRecovered
KnowledgePublished
KnowledgeShared
```

---

# 137. Command System

Comandos:

```text
StartResearchCommand
RunExperimentCommand
RecordObservationCommand
SubmitEvidenceCommand
PublishResearchCommand
ShareKnowledgeCommand
StudyKnowledgeCommand
TeachKnowledgeCommand
```

---

# 138. Registry

Registrar:

```text
KnowledgeType
EvidenceType
ResearchType
ExperimentType
TheoryType
ResearchField
ValidationMethod
```

---

# 139. Modding

Mods podem adicionar:

```text
research fields
knowledge types
evidence types
experiments
scientific institutions
```

Exemplo:

```text
example:xenobiology
```

---

# 140. Scripting

Scripts podem:

```text
queryKnowledge
createResearchProject
recordObservation
```

usando APIs controladas.

---

# 141. API

```text
IKnowledgeSystem
IKnowledgeRecord
IKnowledgeClaim
IKnowledgeRepository
IKnowledgeResolver
IKnowledgePropagation
IKnowledgeHistory

IResearchSystem
IResearchProject
IResearchQuestion
IHypothesis
IExperiment
IResearchResult
IEvidence
IResearchInstitution

ITheorySystem
ITheory
ITheoryResolver

IDiscoveryEngine
IKnowledgeCompiler
IResearchScheduler
```

---

# 142. Organização do código

```text
src/research/

├── core/
│   ├── research-system
│   ├── research-context
│   └── research-state
│
├── knowledge/
│   ├── knowledge
│   ├── claim
│   ├── belief
│   ├── confidence
│   ├── validity
│   └── history
│
├── graph/
│   ├── knowledge-graph
│   ├── nodes
│   ├── edges
│   └── resolver
│
├── observation/
│   ├── observation
│   ├── measurement
│   └── collection
│
├── evidence/
│   ├── evidence
│   ├── reliability
│   └── provenance
│
├── hypothesis/
│
├── experiment/
│   ├── experiment
│   ├── procedure
│   ├── result
│   └── reproducibility
│
├── project/
│   ├── research-project
│   ├── question
│   └── planning
│
├── theory/
│
├── institution/
│
├── publication/
├── education/
├── propagation/
├── discovery/
├── formalization/
├── technology-bridge/
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

# 143. Dependências

```text
REGISTRY
   │
EVENT BUS
   │
PERSISTENCE
   │
ENTITY
   │
CIVILIZATION
   │
   ▼
RESEARCH / KNOWLEDGE
   │
 ┌─┼──────────────┬──────────────┐
 ▼ ▼              ▼              ▼
DATA EVIDENCE   RESEARCH       KNOWLEDGE
 │     │           │              │
 └─────┼───────────┼──────────────┘
       ▼
    ANALYSIS
       │
       ▼
  FORMALIZATION
       │
       ▼
   TECHNOLOGY
       │
       ▼
  PROGRESSION
```

---

# 144. Implementação por fases

## RESEARCH-0 — Core

```text
Knowledge
Claim
Evidence
Research
```

---

## RESEARCH-1 — Registry

```text
KnowledgeType
EvidenceType
ResearchField
```

---

## RESEARCH-2 — Observation

```text
Observation
Measurement
```

---

## RESEARCH-3 — Hypothesis

```text
Question
Hypothesis
Prediction
```

---

## RESEARCH-4 — Experiment

```text
Experiment
Result
Analysis
```

---

## RESEARCH-5 — Confidence

```text
Evidence
Confidence
Validity
```

---

## RESEARCH-6 — Knowledge Graph

```text
nodes
edges
queries
```

---

## RESEARCH-7 — Formalization

```text
Observation
→ Claim
→ Knowledge
```

---

## RESEARCH-8 — Research Projects

```text
project
researchers
institutions
```

---

## RESEARCH-9 — Publication

```text
reports
peer review
```

---

## RESEARCH-10 — Propagation

```text
teacher
student
books
trade
```

---

## RESEARCH-11 — Knowledge Loss

```text
preservation
loss
recovery
```

---

## RESEARCH-12 — Technology Bridge

```text
knowledge
→ capability
→ technology candidate
```

---

## RESEARCH-13 — Civilization

```text
institutions
funding
research policy
```

---

## RESEARCH-14 — Emergent Discovery

```text
anomalies
unexpected results
new topics
```

---

## RESEARCH-15 — Advanced Simulation

```text
Full
Regional
Abstract
```

---

# 145. Primeiro Vertical Slice

```text
NPC
 ↓
observe material
 ↓
Observation
 ↓
Research Question
 ↓
Hypothesis
 ↓
Experiment
 ↓
Result
 ↓
Evidence
 ↓
Knowledge
 ↓
Technology Candidate
```

---

# 146. Segundo Vertical Slice

Reprodução:

```text
Researcher A
 ↓
Discovery

Researcher B
 ↓
Independent Experiment

Researcher C
 ↓
Replication
```

Resultado:

```text
Knowledge
→ higher confidence
→ verified
```

---

# 147. Terceiro Vertical Slice

Ensino:

```text
Researcher
 ↓
Publication
 ↓
Student
 ↓
Study
 ↓
Knowledge acquired
```

---

# 148. Quarto Vertical Slice

Perda:

```text
Civilization A
 ↓
discovers technology
 ↓
documents poorly
 ↓
institution collapses
 ↓
knowledge lost
 ↓
centuries later
 ↓
artifact found
 ↓
research
 ↓
knowledge recovered
```

---

# 149. Quinto Vertical Slice

Conflito científico:

```text
Institution A
→ Theory X

Institution B
→ Theory Y

New Evidence
 ↓
Theory X weakened
 ↓
Theory Y strengthened
```

---

# 150. Sexto Vertical Slice

Descoberta emergente:

```text
Machine experiment
 ↓
unexpected anomaly
 ↓
new Research Question
 ↓
new field of study
 ↓
new technology
```

Esse é um dos testes que mais combina com o NEXORA.

---

# 151. Sétimo Vertical Slice

Civilização:

```text
Government
 ↓
funds laboratory
 ↓
research
 ↓
discovery
 ↓
technology
 ↓
industry
 ↓
economic growth
 ↓
more funding
 ↓
new research
```

Isso cria um loop de desenvolvimento.

---

# 152. Golden Research Test

```text
OBSERVATION
      ↓
RESEARCH QUESTION
      ↓
HYPOTHESIS
      ↓
EXPERIMENT
      ↓
RESULT
      ↓
EVIDENCE
      ↓
ANALYSIS
      ↓
KNOWLEDGE CLAIM
      ↓
VALIDATION
      ↓
FORMALIZED KNOWLEDGE
      ↓
TECHNOLOGY CANDIDATE
      ↓
PROTOTYPE
      ↓
TECHNOLOGY
```

---

# 153. Golden Knowledge Test

```text
NPC A
 ↓
discovers knowledge

NPC B
 ↓
learns from A

Institution
 ↓
documents

Civilization
 ↓
adopts

Player
 ↓
discovers documentation
```

Todos devem acabar com referências coerentes à mesma cadeia de proveniência.

---

# 154. Golden Rediscovery Test

```text
Civilization A
 ↓
discovers X
 ↓
knowledge lost

Civilization B
 ↓
finds artifact
 ↓
research
 ↓
reconstructs X
```

A nova descoberta não deve apagar o histórico original.

---

# 155. Golden Dispute Test

```text
Claim X
 ↓
Evidence A
 ↓
Supported

Evidence B
 ↓
Contradiction
 ↓
Disputed

Experiment C
 ↓
Resolution
```

---

# 156. Stress Test

```text
1 researcher
10
100
1.000
10.000
100.000
```

mais:

```text
research projects
observations
knowledge claims
evidence
experiments
```

---

# 157. Knowledge Graph Stress

```text
1.000 nodes
10.000
100.000
1.000.000
```

com:

```text
relationships
dependencies
contradictions
provenance
```

---

# 158. Civilization Stress

```text
100 civilizations
+
10.000 research institutions
+
millions of knowledge records
```

usando:

```text
regional
abstract
event-driven simulation
```

---

# 159. Security Test

Cliente tenta:

```text
GrantVerifiedKnowledge
```

Resultado:

```text
DENIED
```

---

# 160. Integrity Test

Alterar manualmente:

```text
Evidence
```

depois do save.

Resultado:

```text
integrity mismatch
```

dependendo do mecanismo de Persistence/Security.

---

# 161. Mod Test

Mod adiciona:

```text
example:xenobiology
```

com:

```text
new evidence type
new research field
new technology bridge
```

sem alterar o Core.

---

# 162. Script Test

Script:

```text
detect anomaly
 ↓
create research question
```

usando:

```text
Event Bus
+
Command System
+
Research API
```

---

# 163. Performance

Não podemos recalcular todo o Knowledge Graph sempre.

Usar:

```text
indexes
cached resolutions
incremental graph updates
event-driven propagation
LOD
```

---

# 164. Graph Queries

Precisamos de índices para consultas como:

```text
What supports claim X?
Who knows X?
What technologies depend on X?
What evidence contradicts X?
Where was X discovered?
Who discovered X?
```

---

# 165. Knowledge Query Examples

```text
knowledge.get(id)

knowledge.isKnown(actor, id)

knowledge.getConfidence(actor, claim)

knowledge.getEvidence(claim)

knowledge.getDiscoverers(claim)

knowledge.getDependents(claim)
```

---

# 166. Research Query Examples

```text
research.canResearch(actor, topic)

research.getProjects(actor)

research.getAvailableQuestions(institution)

research.getEvidence(project)

research.getHypotheses(project)
```

---

# 167. Formalization Resolver

```text
IFormalizationResolver
```

pode determinar:

```text
Can this observation become knowledge?
Is evidence sufficient?
Does this claim meet validation threshold?
```

---

# 168. Validation não precisa ser universal

Diferentes campos podem ter diferentes métodos:

```text
biology
physics
history
engineering
magic
astronomy
```

---

# 169. Research Method Registry

Registrar:

```text
Experimental
Observational
Historical
Simulation
Comparative
Field Study
```

---

# 170. Research Fields

```text
Physics
Chemistry
Biology
Medicine
Geology
Ecology
Astronomy
Engineering
Computer Science
Materials
Dimensional Science
Magic Studies
Xenobiology
```

Mods podem adicionar.

---

# 171. Formalization rules

Cada área pode definir:

```text
required evidence
acceptable methods
validation rules
confidence update rules
```

---

# 172. Science ≠ omniscience

Mesmo uma civilização avançada pode ter:

```text
unknown phenomena
```

Isso mantém o mundo aberto.

---

# 173. Technology Frontier

Toda civilização pode possuir uma fronteira:

```text
Known
|
| → Unknown
|
```

O que fica além pode gerar pesquisa.

---

# 174. Research Frontier

```text
CURRENT KNOWLEDGE
       │
       ▼
KNOWLEDGE GAPS
       │
       ▼
RESEARCH QUESTIONS
       │
       ▼
UNKNOWN FRONTIER
```

---

# 175. Player como catalisador

O jogador pode:

```text
discover
observe
fund
teach
steal
publish
trade
```

mas não precisa ser a fonte de toda descoberta.

---

# 176. World as Researcher

O próprio mundo continua funcionando:

```text
NPCs
Institutions
Civilizations
Machines
Sensors
Explorers
```

podem produzir conhecimento.

---

# 177. Conexão com sua filosofia

Isso concretiza:

> **“The world is generated. The world then lives.”**

Porque:

```text
WORLD GENERATION
→ cria fenômenos

SIMULATION
→ faz fenômenos acontecerem

RESEARCH
→ faz alguém tentar entendê-los

KNOWLEDGE
→ guarda o aprendizado

TECHNOLOGY
→ transforma aprendizado em capacidade

CIVILIZATION
→ muda por causa disso
```

---

# 178. Arquitetura final

```text
                         NEXORA
                            │
                   RESEARCH / KNOWLEDGE
                            │
       ┌────────────────────┼────────────────────┐
       ▼                    ▼                    ▼
   OBSERVATION          RESEARCH              KNOWLEDGE
       │                    │                    │
       ▼                    ▼                    ▼
      DATA              QUESTIONS              CLAIMS
       │                    │                    │
       ▼                    ▼                    ▼
   EVIDENCE             HYPOTHESIS           CONFIDENCE
       │                    │                    │
       └──────────────┬─────┴──────────────────┘
                      ▼
                   EXPERIMENT
                      │
                      ▼
                    RESULT
                      │
                      ▼
                   ANALYSIS
                      │
                      ▼
                 FORMALIZATION
                      │
              ┌───────┴────────┐
              ▼                ▼
           THEORY           KNOWLEDGE
              │                │
              └───────┬────────┘
                      ▼
                  VALIDATION
                      │
              ┌───────┴────────┐
              ▼                ▼
           VERIFIED          DISPUTED
              │
              ▼
       TECHNOLOGY BRIDGE
              │
              ▼
       CAPABILITY / TECHNOLOGY
              │
       ┌──────┼──────┬─────────┐
       ▼      ▼      ▼         ▼
     PLAYER   NPC  FACTION  CIVILIZATION
       │      │      │         │
       └──────┴──────┴─────────┘
                    │
             PROPAGATION
                    │
          ┌─────────┼─────────┐
          ▼         ▼         ▼
       TEACHING  BOOKS      TRADE
          │         │         │
          └─────────┼─────────┘
                    ▼
             KNOWLEDGE GRAPH
                    │
             ┌──────┴──────┐
             ▼             ▼
          DISCOVERY      LOSS
             │             │
             └──────┬──────┘
                    ▼
                REDISCOVERY
```

E a separação definitiva:

```text
OBSERVATION
→ "eu percebi algo"

DATA
→ "eu medi algo"

EVIDENCE
→ "tenho suporte para uma afirmação"

HYPOTHESIS
→ "acho que sei por quê"

EXPERIMENT
→ "vou testar"

RESULT
→ "isso aconteceu"

ANALYSIS
→ "o que isso significa?"

KNOWLEDGE
→ "isso agora é uma afirmação estruturada"

VALIDATION
→ "quão confiável é?"

THEORY
→ "como várias afirmações se conectam?"

RESEARCH
→ "processo para descobrir"

TECHNOLOGY
→ "aplicação prática"

PROGRESSION
→ "como isso muda as capacidades"

PROPAGATION
→ "como o conhecimento se espalha"

HISTORY
→ "como chegamos aqui"
```

### O grande ciclo do NEXORA

O que eu considero mais importante neste sistema é este ciclo:

```text
                    UNKNOWN
                       │
                       ▼
                  OBSERVATION
                       │
                       ▼
                 RESEARCH QUESTION
                       │
                       ▼
                   HYPOTHESIS
                       │
                       ▼
                   EXPERIMENT
                       │
             ┌─────────┴─────────┐
             ▼                   ▼
          RESULT             ANOMALY
             │                   │
             ▼                   └──────→ NEW QUESTION
          EVIDENCE
             │
             ▼
         KNOWLEDGE
             │
       ┌─────┴─────┐
       ▼           ▼
   PROPAGATE     APPLY
       │           │
       ▼           ▼
  CIVILIZATION  TECHNOLOGY
       │           │
       └─────┬─────┘
             ▼
        WORLD CHANGES
             │
             ▼
      NEW PHENOMENA
             │
             └──────────────────→ UNKNOWN
```

É aí que **Research/Knowledge formalization deixa de ser uma “árvore de tecnologia” e vira uma memória epistemológica do próprio mundo**. Uma descoberta pode começar com um NPC observando alguma coisa, virar uma hipótese, ser testada por outra geração, virar conhecimento formalizado, espalhar-se entre civilizações, originar uma tecnologia, transformar a economia e o ambiente e, décadas depois, gerar um fenômeno que ninguém entende ainda — reiniciando o ciclo.
