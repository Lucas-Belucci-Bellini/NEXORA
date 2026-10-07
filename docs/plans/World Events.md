# NEXORA — WORLD EVENTS SYSTEM

> **Princípio central:**
> **World Events transformam mudanças do mundo em acontecimentos persistentes, sistêmicos e observáveis. O mundo não apenas possui sistemas; ele também produz acontecimentos que alteram o estado do mundo e geram consequências.**

O **World Events System** não deve virar um “gerador de eventos aleatórios”.

Ele será a camada responsável por representar acontecimentos como:

```text
chuvas extremas
tempestades
enchentes
secas
incêndios
erupções vulcânicas
terremotos
deslizamentos
epidemias
descobertas científicas
guerras
revoluções
eleições
crises econômicas
falências
greves
migrações
descobertas arqueológicas
nascimento de civilizações
queda de cidades
abertura de rotas
descoberta de recursos
colapso de infraestrutura
acidentes industriais
falhas de máquinas
expedições
anomalias
eventos dimensionais
eventos espaciais
descobertas tecnológicas
eventos ambientais
eventos raros
eventos cósmicos
```

E o mais importante:

```text
EVENTO
↓
ALTERAÇÃO DO MUNDO
↓
SISTEMAS REAGEM
↓
NPCs REAGEM
↓
CIVILIZAÇÕES REAGEM
↓
JOGADOR DESCOBRE
↓
HISTÓRIA DO MUNDO MUDA
```

---

# 1. O que é um World Event?

Um **World Event** representa um acontecimento reconhecido pelo mundo.

Exemplo:

```text
Uma seca começa.
```

Isso não é apenas:

```text
Weather = DRY
```

O World Event transforma isso em algo histórico:

```text
WorldEvent
ID: evt-928381
Type: DROUGHT
Region: REGION-421
StartedAt: WorldDay 1823
Severity: HIGH
Duration: 47 days
Cause: LOW_RAINFALL
Status: ACTIVE
```

Então outros sistemas podem reagir:

```text
CLIMATE
   ↓
WORLD EVENT
   ↓
HYDROLOGY
   ↓
AGRICULTURE
   ↓
FOOD SHORTAGE
   ↓
ECONOMY
   ↓
QUEST
   ↓
MIGRATION
   ↓
POLITICS
```

---

# 2. World Events não substituem outros sistemas

Essa separação é extremamente importante.

```text
Climate System
→ simula clima

Physics
→ simula física

Civilization
→ simula civilizações

Economy
→ simula economia

War/Combat
→ resolve combate

World Events
→ registra, coordena e representa acontecimentos
```

Por exemplo:

### Terremoto

Physics não deveria criar uma civilização destruída.

World Events também não deve calcular terremoto.

O fluxo seria:

```text
TECTONICS / PHYSICS
        ↓
EARTHQUAKE DETECTED
        ↓
WORLD EVENT
        ↓
STRUCTURES
        ↓
BUILD / DESTRUCTION
        ↓
CIVILIZATION
        ↓
ECONOMY
        ↓
QUEST
        ↓
HISTORY
```

---

# 3. Evento não é simplesmente Event Bus Event

NEXORA já possui:

```text
EVENT BUS
```

Mas:

```text
Event Bus Event
≠
World Event
```

### Event Bus Event

Comunicação interna.

Exemplo:

```text
BlockBrokenEvent
```

Vida muito curta.

### World Event

Objeto persistente da simulação.

Exemplo:

```text
EarthquakeEvent
```

Pode durar:

```text
horas
dias
anos
séculos
```

Pode gerar consequências.

---

# 4. Arquitetura

```text
                         WORLD EVENTS
                              │
              ┌───────────────┼────────────────┐
              ▼               ▼                ▼
          DETECTION        GENERATION        HISTORY
              │               │                │
              └───────────────┼────────────────┘
                              ▼
                           EVENT
                              │
         ┌────────────────────┼────────────────────┐
         ▼                    ▼                    ▼
      SCOPE                 STATE               EFFECTS
         │                    │                    │
         ▼                    ▼                    ▼
      REGION              ACTIVE               WORLD
      CHUNK               PAUSED               SYSTEMS
      WORLD               RESOLVED             NPCs
      DIMENSION            CANCELLED            CIVILIZATION
      UNIVERSE              FAILED              ECONOMY
                                                   │
                                                   ▼
                                               CONSEQUENCES
                                                   │
                         ┌─────────────────────────┼────────────────────┐
                         ▼                         ▼                    ▼
                       QUEST                    SOCIAL              HISTORY
                         │                         │                    │
                         └─────────────────────────┼────────────────────┘
                                                   ▼
                                                PLAYER
```

---

# 5. Event Definition

Define o tipo do evento.

```ts
interface WorldEventDefinition {
    id: ResourceID;
    type: WorldEventType;

    category: WorldEventCategory;

    minSeverity: Severity;
    maxSeverity: Severity;

    duration: DurationPolicy;

    triggerConditions: EventCondition[];

    effects: EventEffect[];

    propagation: PropagationPolicy;

    visibility: VisibilityPolicy;

    persistence: PersistencePolicy;

    cooldown: CooldownPolicy;
}
```

Exemplo:

```text
nexora:event/drought
```

---

# 6. Event Instance

A definição é o molde.

A instância é o acontecimento real.

```ts
interface WorldEventInstance {
    eventId: EventInstanceID;

    definitionId: ResourceID;

    worldId: WorldID;

    dimensionId: DimensionID;

    scope: EventScope;

    location: EventLocation;

    startedAt: WorldTime;

    predictedEnd?: WorldTime;

    actualEnd?: WorldTime;

    severity: Severity;

    state: WorldEventState;

    causes: EventCause[];

    effects: AppliedEffect[];

    participants: EventParticipant[];

    witnesses: EntityID[];

    consequences: EventConsequence[];

    history: EventHistory[];
}
```

---

# 7. Estados

```text
DETECTED
   ↓
PROPOSED
   ↓
VALIDATING
   ↓
SCHEDULED
   ↓
STARTING
   ↓
ACTIVE
   ↓
ESCALATING
   ↓
RESOLVING
   ↓
RESOLVED
```

Também:

```text
CANCELLED
FAILED
SUPPRESSED
QUARANTINED
```

---

# 8. Categorias

Eu dividiria em:

```text
NATURAL
ENVIRONMENTAL
GEOLOGICAL
HYDROLOGICAL
CLIMATE
BIOLOGICAL
ECOLOGICAL
SOCIAL
POLITICAL
ECONOMIC
MILITARY
CIVILIZATION
TECHNOLOGICAL
INDUSTRIAL
INFRASTRUCTURE
DISCOVERY
EXPLORATION
SCIENTIFIC
ARCHAEOLOGICAL
DIMENSIONAL
SPACE
COSMIC
PLAYER
SYSTEM
CUSTOM
```

---

# 9. Eventos naturais

Exemplos:

```text
Earthquake
Volcano
Tsunami
Avalanche
Landslide
MeteorImpact
Sinkhole
Wildfire
DustStorm
Tornado
Hurricane
LightningStorm
ExtremeCold
HeatWave
Flood
Drought
Blizzard
```

Mas eles não devem ser tratados como simples RNG.

---

# 10. Causalidade

O evento deve possuir causas.

```text
CAUSE
  ↓
EVENT
  ↓
EFFECT
  ↓
CONSEQUENCE
```

Exemplo:

```text
Climate change
    ↓
drought
    ↓
river level decreases
    ↓
crop production falls
    ↓
food prices rise
    ↓
city becomes unstable
    ↓
migration begins
```

Assim o mundo pode formar cadeias causais.

---

# 11. Event Chain

Um evento pode gerar outro evento.

```text
EARTHQUAKE
     ↓
BUILDING_COLLAPSE
     ↓
INFRASTRUCTURE_DAMAGE
     ↓
FOOD_SHORTAGE
     ↓
ECONOMIC_CRISIS
     ↓
POLITICAL_UNREST
```

Isso é uma das partes mais importantes do sistema.

---

# 12. Consequências

Eventos podem ter consequências:

```ts
interface EventConsequence {
    id: ConsequenceID;

    sourceEvent: EventInstanceID;

    type: ConsequenceType;

    severity: number;

    scope: EventScope;

    duration: Duration;

    reversible: boolean;

    conditions: EventCondition[];

    effects: EventEffect[];
}
```

Exemplo:

```text
EVENT:
Drought

CONSEQUENCE:
CropFailure

CONSEQUENCE:
FoodShortage

CONSEQUENCE:
Migration

CONSEQUENCE:
PoliticalInstability
```

---

# 13. Escopo

Um evento pode existir em várias escalas.

```text
CHUNK
REGION
SETTLEMENT
CITY
PROVINCE
FACTION
CIVILIZATION
DIMENSION
WORLD
STAR_SYSTEM
GALAXY
UNIVERSE
```

Exemplo:

### Local

```text
incêndio em uma floresta
```

### Regional

```text
seca em 500 km
```

### Mundial

```text
alteração climática global
```

### Dimensional

```text
instabilidade dimensional
```

### Cósmico

```text
supernova
```

---

# 14. Propagação

Eventos podem se espalhar.

```text
EVENT
 ↓
REGION A
 ↓
REGION B
 ↓
REGION C
```

Isso pode ocorrer através de:

```text
vento
água
animais
NPCs
mercadores
comunicação
migração
doenças
comércio
guerra
informação
infraestrutura
dimensões
espaço
```

---

# 15. Propagação não significa que todo evento chega a todo lugar

O sistema precisa considerar:

```text
distance
terrain
transport
communication
weather
population
infrastructure
faction relations
technology
```

Exemplo:

Uma epidemia:

```text
Cidade A
  ↓
estrada
  ↓
Cidade B
  ↓
mercadores
  ↓
Cidade C
```

Mas uma região isolada pode continuar saudável.

---

# 16. Descoberta do evento

Nem todo evento deve ser imediatamente conhecido.

Esse é um ponto essencial para combinar com:

```text
Knowledge
Research
Social
Civilization
Quest
Communication
```

Podemos ter:

```text
REALITY
   ↓
EVENT
   ↓
DETECTION
   ↓
OBSERVATION
   ↓
KNOWLEDGE
   ↓
BELIEF
   ↓
PUBLIC AWARENESS
```

Exemplo:

```text
Vulcão começou a ficar instável.

Evento:
ACTIVE

NPC próximo:
observou fumaça

Conhecimento local:
"algo está errado"

Merchant:
leva informação

Cidade:
descobre

Civilização:
recebe relatório
```

---

# 17. Informação do evento

Separar:

```text
EVENT TRUTH
```

de:

```text
EVENT KNOWLEDGE
```

O evento realmente existe:

```text
Meteor approaching
```

Mas um NPC pode acreditar:

```text
"É um sinal divino."
```

Outro:

```text
"É apenas uma estrela."
```

Outro:

```text
"Objeto artificial."
```

Isso combina perfeitamente com o:

```text
RESEARCH / KNOWLEDGE SYSTEM
```

---

# 18. Severidade

```text
TRACE
MINOR
MODERATE
MAJOR
SEVERE
CRITICAL
CATASTROPHIC
COSMIC
```

Mas severidade precisa ser contextual.

Uma enchente:

```text
rio pequeno:
MODERATE
```

Pode ser:

```text
cidade inteira:
SEVERE
```

---

# 19. Magnitude

Separar:

```text
Severity
Magnitude
Impact
```

Porque um evento gigantesco pode atingir uma área vazia.

Exemplo:

```text
Magnitude: 10
Impact: 2
```

Porque:

```text
evento gigantesco
+
região desabitada
=
baixo impacto civilizacional
```

---

# 20. Impact Analysis

Antes de aplicar um evento grande:

```text
Event
 ↓
Impact Analyzer
 ↓
Structures
 ↓
Population
 ↓
Economy
 ↓
Ecology
 ↓
Infrastructure
 ↓
Resources
```

Isso permite:

```text
previsão
simulação
alertas
AI decisions
```

---

# 21. Eventos previsíveis

Alguns eventos podem possuir previsão.

Exemplo:

```text
storm probability:
73%
```

NPCs podem interpretar.

Civilização avançada:

```text
forecast:
85%
```

Civilização primitiva:

```text
folk prediction:
"tempestade chegando"
```

Tecnologia muda a capacidade de previsão.

---

# 22. Eventos planejados

Nem tudo nasce espontaneamente.

Exemplos:

```text
festival
election
construction
railway opening
space launch
scientific expedition
military exercise
trade fair
civilization summit
```

Fluxo:

```text
PLAN
 ↓
SCHEDULE
 ↓
PREPARE
 ↓
EVENT
 ↓
RESOLUTION
```

---

# 23. Eventos emergentes

Essa é outra categoria.

Não existe:

```text
Event = "Economic Crisis"
```

programado.

Ela aparece porque:

```text
poor harvest
+
food shortage
+
high prices
+
unemployment
+
political tension
```

ultrapassaram determinados limites.

Então:

```text
CRISIS DETECTOR
       ↓
Economic Crisis Event
```

---

# 24. Event Emergence Engine

```text
OBSERVATIONS
    ↓
SIGNALS
    ↓
PATTERN DETECTION
    ↓
THRESHOLD / MODEL
    ↓
EVENT CANDIDATE
    ↓
VALIDATION
    ↓
WORLD EVENT
```

Esse módulo será importante para o “mundo vivo”.

---

# 25. Eventos probabilísticos

Existirão eventos que possuem probabilidade.

Mas:

```text
random()
```

sozinho não é suficiente.

Usar:

```text
environmental conditions
history
world seed
population
technology
season
climate
region
previous events
risk
```

Exemplo:

```text
P(volcano eruption)
=
geology
×
volcanic activity
×
historical pattern
×
pressure
```

---

# 26. RNG determinístico

```text
World Seed
+
Event Definition
+
Location
+
World Time
+
Event Sequence
```

gera RNG determinístico.

Assim:

```text
same seed
+
same world version
+
same state
=
same result
```

Isso é importantíssimo para:

```text
replay
debugging
testing
multiplayer
server recovery
```

---

# 27. Eventos únicos

Alguns acontecimentos nunca devem repetir.

```text
FIRST_DISCOVERY
UNIQUE_ARTIFACT_FOUND
ANCIENT_CITY_DISCOVERED
SPECIAL_COSMIC_EVENT
WORLD_FOUNDATION
DIMENSION_DISCOVERY
```

Sistema:

```text
UniqueEventRegistry
```

---

# 28. Eventos recorrentes

Outros podem repetir:

```text
season
festival
storm
market
migration
election
harvest
```

Com:

```text
cooldown
frequency
seasonality
conditions
history
```

---

# 29. Eventos históricos

Eventos importantes entram na história.

```text
World History
    ↑
World Event
```

Exemplo:

```text
Year 129:

"The Great Flood destroyed
three settlements."

Year 137:

"The Eastern Railway was completed."

Year 152:

"First expedition reached the Far Lands."
```

---

# 30. Event Chronicle

Criar:

```ts
interface WorldChronicleEntry {
    id: ChronicleID;

    eventId: EventInstanceID;

    timestamp: WorldTime;

    title: string;

    summary: string;

    location: Location;

    participants: EntityID[];

    importance: number;

    visibility: Visibility;
}
```

---

# 31. História não deve ser apenas texto

O texto é apenas uma representação.

A verdadeira história vem do estado do mundo:

```text
EVENT
+
STATE CHANGES
+
ACTORS
+
CONSEQUENCES
```

Assim:

```text
World History
```

pode ser reconstruída.

---

# 32. Eventos e NPCs

NPCs podem:

```text
detectar
ignorar
reagir
investigar
explorar
fugir
ajudar
lucrar
manipular
registrar
informar
```

Exemplo:

```text
incêndio
 ↓
NPC percebe fumaça
 ↓
alarme
 ↓
cidade envia bombeiros
 ↓
mercadores desviam rota
 ↓
políticos discutem prevenção
```

---

# 33. Eventos e civilizações

Civilização pode responder:

```text
evento detectado
 ↓
policy evaluation
 ↓
response plan
 ↓
resource allocation
 ↓
actions
```

Exemplo:

```text
seca
 ↓
government
 ↓
ration food
 ↓
build reservoirs
 ↓
import grain
 ↓
raise taxes
```

---

# 34. Eventos políticos

```text
Election
Revolution
Coup
Scandal
Protest
Strike
Treaty
Alliance
WarDeclaration
PeaceTreaty
Succession
Independence
Rebellion
```

Mas o sistema não deve executar política.

Ele representa:

```text
EVENT
```

e chama:

```text
Social
Faction
Civilization
Economy
```

---

# 35. Eventos econômicos

```text
MarketCrash
FoodShortage
ResourceBoom
Bankruptcy
TradeEmbargo
SupplyChainFailure
Inflation
Deflation
IndustrialExpansion
EconomicCollapse
```

O Economy System calcula economia.

World Events registra:

```text
"economic crisis started"
```

---

# 36. Eventos de infraestrutura

```text
BridgeCollapse
DamFailure
RailwayBreak
PowerGridFailure
WaterSystemFailure
RoadBlockage
TunnelCollapse
FactoryExplosion
CommunicationOutage
```

Isso conecta:

```text
Structure
Machines
Energy
Fluid
Vehicles
Civilization
Economy
```

---

# 37. Eventos tecnológicos

```text
TechnologyDiscovered
PrototypeCompleted
TechnologyFailure
PatentCreated
AncientTechnologyRecovered
ResearchBreakthrough
ResearchAccident
IndustrialRevolution
```

Integra com:

```text
Research
Knowledge
Technology
Progression
Economy
Civilization
```

---

# 38. Eventos científicos

```text
Experiment
Discovery
Observation
Publication
Replication
Anomaly
HypothesisBreakthrough
```

Exemplo:

```text
Research
 ↓
Experiment
 ↓
UnexpectedResult
 ↓
WorldEvent
 ↓
NewResearchQuestion
```

---

# 39. Eventos arqueológicos

```text
ArtifactDiscovered
AncientCityFound
RuinsExcavated
AncientTechnologyRecovered
LostCivilizationDetected
```

Isso pode iniciar:

```text
Quest
Research
Knowledge
Economy
Exploration
Civilization
```

---

# 40. Eventos de exploração

```text
NewRegionDiscovered
FarLandsReached
BeyondlandsEntered
DeepWorldDiscovered
NewDimensionFound
PlanetDiscovered
StarSystemMapped
UnknownObjectDetected
```

---

# 41. Eventos espaciais

```text
AsteroidApproach
CometPass
SolarStorm
MeteorImpact
Supernova
PlanetaryCollision
OrbitalAnomaly
StationFailure
SpaceExpedition
FirstLanding
AlienSignal
```

O Space System continua responsável pela física/astronomia.

World Events representa:

```text
"Aconteceu."
```

---

# 42. Eventos dimensionais

```text
PortalOpened
DimensionDiscovered
DimensionInstability
RealityStorm
DimensionCollapse
CrossDimensionMigration
DimensionalInvasion
```

Sem colocar a lógica dimensional dentro do Event System.

---

# 43. Eventos de jogador

O jogador também pode causar eventos.

```text
player builds megastructure
player discovers dimension
player destroys infrastructure
player starts trade route
player founds settlement
player discovers artifact
player triggers ancient mechanism
```

Então:

```text
Player
 ↓
Command
 ↓
System
 ↓
World Event
```

---

# 44. O jogador não deve ser necessário

Esse princípio precisa ser permanente:

> **World Events devem continuar acontecendo mesmo sem jogadores próximos.**

Caso contrário o mundo vira:

```text
player online
→ world lives

player offline
→ world freezes
```

O correto:

```text
WORLD
 ↓
SIMULATION LOD
 ↓
EVENT MANAGER
 ↓
EVENTS CONTINUE
```

---

# 45. LOD

Para suportar mundos enormes:

```text
FULL
REGIONAL
ABSTRACT
```

### FULL

Eventos próximos.

```text
exact entities
exact structures
exact physics
```

### REGIONAL

```text
population aggregates
resource impacts
economic impacts
```

### ABSTRACT

```text
statistical state
event probability
high-level consequences
```

---

# 46. Exemplo de evento distante

Jogador está no Brasil do mundo NEXORA.

Na outra extremidade do mundo:

```text
CITY A
```

sofre:

```text
FOOD_SHORTAGE
```

Não precisamos simular:

```text
10.000 NPCs individualmente
```

Podemos guardar:

```text
population = 12,431

food_stock = 17%

unemployment = 28%

migration_pressure = 0.72
```

E o evento continua existindo.

---

# 47. Event Scheduler

```text
EventScheduler
```

controla:

```text
immediate events
short events
long events
scheduled events
recurring events
emergent events
```

Com diferentes frequências:

```text
FRAME
SECOND
MINUTE
HOUR
DAY
WEEK
MONTH
SEASON
YEAR
DECADE
CENTURY
```

---

# 48. Event Queue

```text
EventQueue
```

Possui:

```text
priority
time
scope
severity
causality
region
```

Exemplo:

```text
CRITICAL earthquake
```

não deve ficar atrás de:

```text
minor festival
```

---

# 49. Event Budget

Para impedir explosão:

```text
max events/tick
max active events/world
max event chain depth
max consequences/event
max propagated regions
max spawned tasks
```

---

# 50. Anti-event-storm

Imagine:

```text
evento A
 ↓
100 eventos
 ↓
cada um gera 100
 ↓
10.000
 ↓
1.000.000
```

Precisamos:

```text
EventDepthLimit
EventBudget
PropagationBudget
ConsequenceBudget
Deduplication
Cooldown
Aggregation
```

---

# 51. Event Aggregation

Ao invés de:

```text
1000 small fires
```

poderíamos consolidar:

```text
Regional Wildfire Event
```

Isso reduz custo.

---

# 52. Event Deduplication

Dois sistemas podem detectar o mesmo problema.

```text
Climate → drought
Agriculture → drought
Hydrology → drought
```

Não criar três secas.

Usar:

```text
EventCorrelationKey
```

Exemplo:

```text
DROUGHT:
region=R21
period=1823-1825
```

---

# 53. Causal Graph

Esse sistema deve manter um grafo:

```text
EVENT GRAPH
```

Exemplo:

```text
CLIMATE_ANOMALY
       ↓
DROUGHT
       ↓
CROP_FAILURE
       ↓
FOOD_SHORTAGE
       ↓
PRICE_SPIKE
       ↓
RIOT
       ↓
GOVERNMENT_RESPONSE
```

Isso será valioso para:

```text
debugging
history
AI
quests
research
analytics
```

---

# 54. Event Correlation

Eventos relacionados recebem:

```text
CorrelationID
RootEventID
ParentEventID
CausationID
```

Assim podemos responder:

> Por que essa guerra começou?

```text
WAR
 ← RIOT
 ← FOOD_SHORTAGE
 ← DROUGHT
 ← CLIMATE_ANOMALY
```

---

# 55. Event Prediction

Outro módulo:

```text
EventPredictionEngine
```

Pode calcular:

```text
probability
confidence
time window
possible impact
```

Exemplo:

```text
Flood probability: 68%
Window: next 18 hours
Expected severity: Major
```

---

# 56. Forecast ≠ Event

Muito importante:

```text
FORECAST
≠
EVENT
```

Forecast:

```text
possible future
```

Event:

```text
actual occurrence
```

---

# 57. Event Detection

Vários sistemas podem fornecer sinais:

```text
Climate
Hydrology
Geology
Economy
Civilization
Research
Space
Physics
Ecology
Machines
Vehicles
Player
```

Todos podem produzir:

```text
EventCandidate
```

O World Events System valida.

---

# 58. Event Candidate

```ts
interface EventCandidate {
    source: EventSource;

    definition: ResourceID;

    scope: EventScope;

    location: Location;

    confidence: number;

    severity: number;

    causes: EventCause[];

    evidence: Evidence[];

    timestamp: WorldTime;
}
```

Isso evita que qualquer sistema simplesmente diga:

```text
"GERA EVENTO"
```

---

# 59. Event Validation

```text
candidate
 ↓
conditions
 ↓
permissions
 ↓
cooldown
 ↓
duplication
 ↓
resource budget
 ↓
causality
 ↓
event created
```

---

# 60. Effects

O World Event não deve esconder lógica.

Use efeitos declarativos.

```ts
interface EventEffect {
    type: EventEffectType;

    target: TargetSelector;

    magnitude: number;

    duration?: Duration;

    conditions?: EventCondition[];
}
```

Exemplo:

```text
DROUGHT

effect:
reduce_water_availability
```

Hydrology executa.

---

# 61. Melhor fluxo

```text
WORLD EVENT
    ↓
EFFECT
    ↓
SPECIALIZED SYSTEM
    ↓
STATE CHANGE
    ↓
EVENT BUS
    ↓
WORLD EVENT CONSEQUENCE
```

Isso mantém arquitetura limpa.

---

# 62. Integração com Event Bus

Exemplo:

```text
WorldEventStarted
WorldEventEscalated
WorldEventUpdated
WorldEventResolved
WorldEventCancelled
WorldEventFailed
WorldEventDiscovered
WorldEventPredictionUpdated
WorldEventConsequenceCreated
```

---

# 63. Comandos

Usando o Command System:

```text
CreateWorldEventCommand
ScheduleWorldEventCommand
CancelWorldEventCommand
ResolveWorldEventCommand
AcknowledgeWorldEventCommand
InvestigateWorldEventCommand
SimulateWorldEventCommand
```

Mas:

```text
Command
→ request

WorldEvent
→ resulting persistent world occurrence
```

---

# 64. Queries

```text
GetActiveEvents
GetEventsAtLocation
GetRecentEvents
GetEventHistory
GetEventCauses
GetEventConsequences
GetPredictedEvents
GetEventsByFaction
GetEventsByCivilization
GetEventsByCategory
```

---

# 65. Event API

```ts
interface IWorldEventSystem {

    create(candidate: EventCandidate): EventResult;

    schedule(event: ScheduledWorldEvent): EventResult;

    cancel(id: EventInstanceID): EventResult;

    resolve(id: EventInstanceID): EventResult;

    get(id: EventInstanceID): WorldEventInstance | null;

    query(query: WorldEventQuery): WorldEventInstance[];

    predict(context: EventPredictionContext): EventPrediction[];

    simulate(request: EventSimulationRequest): EventSimulationResult;
}
```

---

# 66. Debug API

CLI:

```text
nexora event list

nexora event inspect <id>

nexora event active

nexora event history

nexora event causes <id>

nexora event consequences <id>

nexora event graph <id>

nexora event predict

nexora event simulate

nexora event trigger <definition>

nexora event cancel <id>

nexora event resolve <id>

nexora event region <region>
```

---

# 67. Event Simulation

Extremamente útil.

```text
nexora event simulate drought
```

Retorna:

```text
Population affected: 82,492

Food production:
-41%

Water supply:
-36%

Migration:
+18%

Food prices:
+73%

Political instability:
+22%

Expected secondary events:
- famine
- migration
- protests
```

Sem alterar o mundo real.

---

# 68. Dry Run

Eventos importantes podem ter:

```text
PREVIEW
```

Antes de aplicar.

Isso será útil para:

```text
admin
testing
AI
research
game balancing
```

---

# 69. World Event vs Quest

Um evento:

```text
Flood
```

pode gerar:

```text
Quest:
Repair Dam
```

Então:

```text
WORLD EVENT
      ↓
QUEST GENERATOR
      ↓
QUEST
```

Quest não cria artificialmente o evento.

---

# 70. World Event vs Knowledge

Evento:

```text
Volcano erupted.
```

Knowledge:

```text
NPC believes volcano will erupt again.
```

Research:

```text
Research volcanic cycles.
```

Technology:

```text
Develop eruption prediction.
```

Esse ciclo é perfeito para NEXORA.

---

# 71. World Event vs History

```text
World Event
→ acontecimento

History
→ registro interpretável dos acontecimentos
```

---

# 72. World Event vs Social

```text
World Event:
Election
```

Social:

```text
candidates
voters
relationships
factions
```

---

# 73. World Event vs Civilization

```text
World Event:
economic crisis
```

Civilization:

```text
policy
infrastructure
migration
government response
```

---

# 74. World Event vs Economy

```text
World Event:
resource shortage
```

Economy:

```text
prices
supply
demand
trade
production
```

---

# 75. World Event vs Space

```text
Space:
asteroid trajectory

World Events:
asteroid discovered
asteroid impact predicted
asteroid impact occurred
```

---

# 76. Mod API

Mods precisam poder registrar:

```text
event definitions
triggers
conditions
effects
event categories
event handlers
forecast models
custom consequences
```

Exemplo:

```text
my_mod:ancient_machine_awakened
```

---

# 77. Evento criado por mod

```ts
registry.events.register({
    id: "example:machine_awakened",

    category: "ANOMALY",

    triggers: [
        "example:ancient_machine_active"
    ]
});
```

Mas a execução ocorre pelo sistema público.

---

# 78. Official content

O conteúdo oficial deve usar a mesma API.

```text
OFFICIAL EVENT
      ↓
PUBLIC WORLD EVENT API
```

e:

```text
MOD EVENT
      ↓
PUBLIC WORLD EVENT API
```

Nada de:

```text
CoreEventHack
```

para conteúdo vanilla.

---

# 79. Segurança

World Events são potencialmente perigosos para performance.

Um mod malicioso poderia tentar:

```text
100000 events
```

por segundo.

Portanto:

```text
event quota
chain quota
propagation quota
memory quota
CPU budget
storage quota
```

por mod.

---

# 80. Segurança de comandos

Somente autoridades apropriadas podem:

```text
force event
cancel event
modify event
```

Exemplo:

```text
PLAYER
→ cannot force supernova

ADMIN
→ possibly can

SERVER
→ can
```

---

# 81. Multiplayer

Servidor é autoridade.

```text
CLIENT
 ↓
request
 ↓
SERVER
 ↓
World Event
 ↓
Simulation
 ↓
Replication
 ↓
CLIENTS
```

Cliente não cria:

```text
EarthquakeEvent
```

sozinho.

---

# 82. Replicação

Não enviar todos os eventos para todos.

Interest management:

```text
local
regional
global
```

Jogador próximo:

```text
full event detail
```

Jogador distante:

```text
summary
```

---

# 83. Persistência

World Events precisam sobreviver ao restart quando apropriado.

Persistir:

```text
event id
definition
state
timestamps
scope
location
severity
causes
effects
participants
consequences
history reference
```

Não persistir:

```text
temporary runtime caches
derived predictions
render data
```

---

# 84. Recovery

Exemplo:

Servidor cai durante:

```text
Earthquake
```

Ao reiniciar:

```text
Load Save
 ↓
Load active events
 ↓
Validate
 ↓
Resume event
 ↓
Reconstruct derived state
```

---

# 85. Event snapshots

Eventos de longa duração:

```text
Drought
War
Climate Shift
Migration Crisis
```

podem possuir snapshots:

```text
Event Snapshot
```

para evitar reprocessar milhares de acontecimentos.

---

# 86. Long-term events

Eventos podem durar:

```text
5 seconds
10 minutes
3 days
10 years
1000 years
```

Exemplo:

```text
ClimateShift
```

pode nunca possuir um “fim” normal.

Pode virar:

```text
PERMANENT
```

ou:

```text
RESOLVED
```

através de mudança de condições.

---

# 87. Event evolution

Evento pode mudar:

```text
MINOR
 ↓
MODERATE
 ↓
MAJOR
 ↓
CRITICAL
```

ou:

```text
CRITICAL
 ↓
DECLINING
 ↓
RESOLVING
 ↓
RESOLVED
```

---

# 88. Event Escalation

Exemplo:

```text
food shortage
```

Se nada resolver:

```text
food shortage
 ↓
famine
 ↓
migration
 ↓
riots
 ↓
revolution
```

Isso gera histórias emergentes sem script fixo.

---

# 89. Event Resolution

Um evento deve ter condições para terminar.

```ts
interface ResolutionCondition {
    check(context: EventContext): boolean;
}
```

Exemplo:

```text
Flood
```

termina quando:

```text
water level < threshold
```

Mas a consequência permanece:

```text
destroyed bridge
```

---

# 90. Evento resolvido não significa mundo restaurado

Muito importante.

```text
FLOOD
resolved
```

não deve simplesmente:

```text
undoEverything()
```

O mundo mudou.

A ponte pode continuar destruída.

---

# 91. Persistência histórica

Mesmo depois:

```text
Event = RESOLVED
```

fica:

```text
History
```

Exemplo:

```text
Great Flood of Year 482
```

---

# 92. Estatísticas

Sistema pode gerar:

```text
events per year
events by category
events by region
events by severity
causal chains
most destructive events
most frequent events
```

Isso também ajuda a balancear NEXORA.

---

# 93. World Event Analytics

```text
EventAnalytics
```

Pode descobrir:

```text
Região X sofre secas demais.
```

ou:

```text
Eventos políticos estão raros demais.
```

ou:

```text
99% das crises econômicas começam por agricultura.
```

---

# 94. Debug visual

No futuro:

```text
EVENT OVERLAY
```

Mostra:

```text
[RED]
ACTIVE EVENT

Type:
EARTHQUAKE

Magnitude:
7.2

Severity:
MAJOR

Cause:
TECTONIC PRESSURE

Affected:
12 regions

Consequences:
38
```

---

# 95. Ferramenta de causalidade

Muito importante para desenvolvimento:

```text
nexora event graph <id>
```

Resultado:

```text
CLIMATE_ANOMALY
      │
      ▼
DROUGHT
      │
      ├──> CROP_FAILURE
      │         │
      │         ▼
      │    FOOD_SHORTAGE
      │         │
      │         ▼
      │      INFLATION
      │
      └──> MIGRATION
```

---

# 96. Arquitetura interna

Eu deixaria:

```text
src/world-events/
│
├── core/
│   ├── world-event.ts
│   ├── event-definition.ts
│   ├── event-instance.ts
│   ├── event-state.ts
│   └── event-result.ts
│
├── detection/
│   ├── detector.ts
│   ├── candidate.ts
│   ├── conditions.ts
│   └── validator.ts
│
├── generation/
│   ├── generator.ts
│   ├── procedural.ts
│   ├── emergent.ts
│   └── scheduled.ts
│
├── lifecycle/
│   ├── lifecycle.ts
│   ├── escalation.ts
│   └── resolution.ts
│
├── effects/
│   ├── effect.ts
│   ├── effect-runner.ts
│   └── consequence.ts
│
├── causality/
│   ├── cause.ts
│   ├── graph.ts
│   └── correlation.ts
│
├── propagation/
│   ├── propagator.ts
│   ├── region.ts
│   └── network.ts
│
├── prediction/
│   ├── predictor.ts
│   ├── forecast.ts
│   └── impact.ts
│
├── history/
│   ├── chronicle.ts
│   └── archive.ts
│
├── scheduler/
│   ├── scheduler.ts
│   ├── queue.ts
│   └── budget.ts
│
├── replication/
│   └── event-replication.ts
│
├── persistence/
│   └── event-storage.ts
│
├── security/
│   ├── quotas.ts
│   └── permissions.ts
│
├── api/
│   └── world-event-api.ts
│
└── debug/
    └── event-debugger.ts
```

---

# 97. Dependências

```text
CORE
 │
 ├── Registry
 ├── Event Bus
 ├── Command
 ├── Persistence
 ├── Security
 └── Scheduler
        │
        ▼
 WORLD EVENTS
        │
 ├── Climate
 ├── Hydrology
 ├── Ecology
 ├── Economy
 ├── Civilization
 ├── Social
 ├── Research
 ├── Space
 ├── Structure
 ├── Machines
 ├── Vehicle
 └── Player
```

---

# 98. Dependências que não podem acontecer

World Events não deve ficar acoplado diretamente a:

```text
Renderer
UI
Audio
Animation
```

Esses recebem:

```text
Event Bus
```

Exemplo:

```text
Earthquake
 ↓
World Event
 ↓
Event Bus
 ├── Renderer
 ├── Audio
 ├── Camera
 └── UI
```

---

# 99. Vertical Slice 1 — Evento manual

```text
Registry
 ↓
Definition
 ↓
Create Event
 ↓
Active
 ↓
Resolve
 ↓
History
```

Evento simples:

```text
test:event
```

---

# 100. Vertical Slice 2 — Evento natural

```text
Climate
 ↓
Event Candidate
 ↓
World Event
 ↓
Flood
 ↓
Hydrology
 ↓
Event Bus
 ↓
History
```

---

# 101. Vertical Slice 3 — Evento econômico

```text
Agriculture
 ↓
Food Production ↓
 ↓
Economic detector
 ↓
Food shortage
 ↓
Economy
 ↓
NPC
 ↓
Quest
```

---

# 102. Vertical Slice 4 — Cadeia causal

```text
Drought
 ↓
CropFailure
 ↓
FoodShortage
 ↓
PriceIncrease
 ↓
Migration
 ↓
PoliticalInstability
```

Esse será um dos primeiros grandes testes de NEXORA.

---

# 103. Vertical Slice 5 — Evento descoberto

```text
Earthquake
 ↓
NPC observes
 ↓
Knowledge
 ↓
Communication
 ↓
Civilization knows
 ↓
Research
```

Aqui começa a aparecer o mundo realmente vivo.

---

# 104. Vertical Slice 6 — Evento espacial

```text
Space
 ↓
Asteroid detected
 ↓
World Event
 ↓
Scientists investigate
 ↓
Research
 ↓
Civilization prepares
 ↓
Quest / Expedition
```

---

# 105. Vertical Slice 7 — Evento mundial

Teste:

```text
Global Climate Shift
```

Deve afetar:

```text
Climate
Hydrology
Biomes
Vegetation
Agriculture
Animals
Economy
Civilization
Migration
Politics
Research
```

sem o World Event implementar internamente nenhum desses sistemas.

---

# 106. Golden Test principal

## WORLD-EVENT-001

```text
create event
→ persist
→ restart
→ reload
→ event still active
→ resolve
→ history exists
```

---

# 107. Golden Test de causalidade

```text
event A
→ consequence B
→ event C
→ consequence D
```

Validar:

```text
correct parent
correct correlation
correct timestamps
correct history
```

---

# 108. Golden Test de duplicação

Dois sistemas detectam:

```text
same drought
```

Resultado:

```text
1 World Event
```

não:

```text
2 World Events
```

---

# 109. Golden Test determinístico

```text
seed = 12345
```

simular.

Depois:

```text
seed = 12345
```

simular novamente.

Resultado:

```text
same events
same timings
same RNG decisions
```

---

# 110. Stress Tests

Começar:

```text
1 event
10
100
1,000
10,000
100,000
1,000,000 abstract events
```

Testar:

```text
CPU
memory
scheduler
persistence
replication
causal graph
```

---

# 111. Fault Tests

Precisamos testar:

```text
corrupted event
missing definition
missing mod
invalid cause
event loop
event storm
broken dependency
server crash
partial save
duplicate event
clock corruption
invalid severity
overflow
NaN
infinite duration
invalid location
```

---

# 112. Segurança

Invariantes:

```text
client cannot create authoritative events
mod cannot exceed event quota
event chains have bounded depth
event propagation has bounded scope
invalid event cannot enter ACTIVE
event cannot commit twice
corrupted event cannot become valid active state
```

---

# 113. Implementação por fases

## WE-0 — Foundation

```text
Event
Definition
Instance
Registry
States
```

---

## WE-1 — Lifecycle

```text
start
update
escalate
resolve
cancel
```

---

## WE-2 — Scheduler

```text
queues
priority
budgets
time
```

---

## WE-3 — Detection

```text
candidates
conditions
validation
deduplication
```

---

## WE-4 — Effects

```text
effects
consequences
resolution
```

---

## WE-5 — Causality

```text
causes
parent events
correlation
causal graph
```

---

## WE-6 — Propagation

```text
regional
global
information
physical
```

---

## WE-7 — Prediction

```text
forecast
probability
impact
```

---

## WE-8 — History

```text
chronicle
archive
importance
```

---

## WE-9 — Persistence

```text
save
load
migration
recovery
```

---

## WE-10 — Multiplayer

```text
replication
interest management
server authority
```

---

## WE-11 — Mod API

```text
custom events
triggers
effects
conditions
```

---

## WE-12 — Natural Events

```text
storm
earthquake
flood
fire
volcano
```

---

## WE-13 — Ecological Events

```text
disease
infestation
species migration
ecosystem collapse
```

---

## WE-14 — Civilization Events

```text
elections
wars
migration
revolutions
economic crises
```

---

## WE-15 — Science / Technology

```text
discoveries
research breakthroughs
anomalies
```

---

## WE-16 — Space

```text
asteroids
solar events
discoveries
planetary events
```

---

## WE-17 — Dimensional

```text
dimensional anomalies
portals
reality events
```

---

## WE-18 — Emergent World

Integrar:

```text
Climate
Ecology
Economy
Civilization
Knowledge
Research
Space
Social
```

---

## WE-19 — Massive World Test

```text
10k regions
100k active/abstract events
2,000+ mob types
thousands of settlements
long simulation
restart/recovery
```

---

# 114. O objetivo final

A arquitetura completa fica:

```text
                         NEXORA WORLD
                              │
                              ▼
                        WORLD EVENTS
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
    DETECTION             GENERATION            SCHEDULE
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              ▼
                           EVENT
                              │
                    ┌─────────┼─────────┐
                    ▼         ▼         ▼
                 CAUSES     EFFECTS   STATE
                    │         │         │
                    └─────────┼─────────┘
                              ▼
                        CONSEQUENCES
                              │
       ┌──────────────────────┼────────────────────────┐
       ▼                      ▼                        ▼
    ECOLOGY               CIVILIZATION              ECONOMY
       │                      │                        │
       ├── CLIMATE            ├── SOCIAL               ├── TRADE
       ├── WATER              ├── POLITICS             ├── PRICES
       ├── BIOLOGY            ├── MIGRATION            └── INDUSTRY
       └── VEGETATION         └── WAR
                              │
                              ▼
                          KNOWLEDGE
                              │
                              ▼
                           RESEARCH
                              │
                              ▼
                         TECHNOLOGY
                              │
                              ▼
                         NEW EVENTS
```

# 115. Regra de ouro do sistema

Eu colocaria estas regras diretamente na documentação:

```text
1. World Event não é RNG solto.

2. World Event representa acontecimentos reais do mundo.

3. Event Bus comunica eventos; World Events persistem acontecimentos.

4. World Events não implementam a lógica dos sistemas afetados.

5. Consequências devem ser causais e rastreáveis.

6. Eventos importantes devem deixar história.

7. Eventos podem acontecer sem jogadores.

8. Distância altera o nível de simulação, não a existência do evento.

9. Informação sobre um evento não é o próprio evento.

10. Eventos emergentes devem poder nascer de condições sistêmicas.

11. Cadeias de eventos devem ser limitadas e observáveis.

12. Conteúdo oficial e mods usam a mesma API pública.

13. Servidor é autoridade em multiplayer.

14. Todo evento persistente precisa ser recuperável.

15. O mundo pode mudar para sempre por causa de um evento.
```

E a ideia que melhor resume esse sistema:

> **“Um World Event não é algo que acontece para o jogador. É algo que acontece com o mundo.”**

Isso encaixa muito bem com a arquitetura que já definimos para **Climate → Ecology → Civilization → Economy → Research → Knowledge → Space**, porque transforma esses sistemas de conjuntos isolados em uma **cadeia histórica capaz de produzir um mundo que realmente muda com o tempo**.
