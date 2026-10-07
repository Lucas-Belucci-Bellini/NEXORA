# NEXORA

# MASTER PLAN — MOB ECOLOGY + CIVILIZATION + ECONOMY + EMERGENT AI

> Este sistema define a camada viva do NEXORA.
>
> O objetivo é transformar mobs, NPCs, vilas, profissões, economia, política e conhecimento em partes interligadas de um mundo simulado.
>
> O jogador continua sendo extremamente importante, mas deixa de ser a única entidade capaz de produzir mudanças significativas no mundo.
>
> NPCs e criaturas devem possuir objetivos, necessidades, memória, relações e capacidade limitada de tomar decisões.

---

# 1. OBJETIVO

O sistema deve produzir:

```text id="m5m1yv"
FAUNA
+
MOBS HOSTIS
+
MOBS AMIGÁVEIS
+
NPCs
+
PROFISSÕES
+
RECURSOS
+
ECONOMIA
+
COMÉRCIO
+
VILAS
+
CIDADES
+
POLÍTICA
+
ELEIÇÕES
+
CONHECIMENTO
+
DOENÇAS
+
TRATAMENTOS
+
EVENTOS
+
HISTÓRIA EMERGENTE
```

---

# 2. PRINCÍPIO FUNDAMENTAL

O mundo não deve depender exclusivamente do jogador.

Em vez de:

```text id="eq1u6l"
Player
↓
gera evento
↓
NPC reage
```

ter também:

```text id="grkdx9"
Mundo
↓
evento
↓
NPC percebe
↓
NPC reage
↓
outros NPCs respondem
↓
cidade muda
```

O jogador continua podendo interferir em tudo isso.

---

# 3. SEPARAÇÃO ARQUITETURAL

```text id="ol7h5n"
CAVE ENGINE
│
└── Geometry / Environment

DEEP WORLD SYSTEM
│
└── Regions / Depth / World State

MOB ECOLOGY ENGINE
│
└── Creatures / Behavior / Population

CIVILIZATION ENGINE
│
└── Settlements / Society / Politics

ECONOMY ENGINE
│
└── Resources / Production / Trade

KNOWLEDGE AI
│
└── Learning / Discovery / Decision Support
```

---

# 4. CAVE ENGINE NÃO CONHECE CIVILIZAÇÃO

O Cave Engine não deve conter lógica como:

```text id="s2fp2p"
if cave.containsVillage...
```

Ele fornece somente informações ambientais.

Exemplo:

```text id="30r7wl"
CaveMetadata
├── size
├── depth
├── temperature
├── humidity
├── water
├── resources
├── entrances
└── geometry
```

O Civilization Engine decide se uma civilização pode ocupar a região.

---

# 5. MOB ECOLOGY ENGINE

Criar:

```text id="p04i4e"
MobEcologyEngine
```

Responsável por:

* população;
* reprodução;
* alimentação;
* migração;
* caça;
* fuga;
* território;
* predadores;
* presas;
* comportamento social;
* doenças;
* relação com ambiente.

---

# 6. MAIS DE 2.000 CRIATURAS

O NEXORA terá como meta:

> **mais de 2.000 tipos de criaturas/mobs.**

Esse número não deve ser atingido criando 2.000 classes independentes.

Usar:

```text id="1jytvl"
Entity Definition
+
Components
+
Behavior Profiles
+
Variants
```

---

# 7. MOB COMPOSITION

Uma criatura pode ser composta por:

```text id="y0wlq2"
Movement
Health
Needs
Diet
Combat
Senses
Memory
Social
Reproduction
Territory
Growth
Drops
```

Assim várias espécies podem reutilizar componentes.

---

# 8. MOB SPECIES

Criar:

```text id="9mygq4"
SpeciesRegistry
```

Cada espécie possui:

```text id="b3n1s5"
id
name
habitat
diet
size
temperatureRange
reproduction
socialStructure
behavior
rarity
resources
```

---

# 9. VARIAÇÕES

Uma espécie poderá ter:

```text id="i2m8ny"
regional variants
age variants
color variants
behavior variants
seasonal variants
```

---

# 10. MOBS AMIGÁVEIS

Possuem:

* necessidades;
* medo;
* território;
* alimento;
* reprodução;
* comportamento social;
* migração.

Não ficarão parados simplesmente porque existem.

---

# 11. MOBS HOSTIS

Também possuirão:

* território;
* comportamento;
* alimentação;
* objetivos;
* população;
* migração;
* relações com outras espécies.

Hostilidade não precisa significar atacar tudo que aparece.

---

# 12. PREDATOR-PREY SYSTEM

Criar relações:

```text id="2wopz4"
Predator
↓
Prey
↓
Population
```

Se determinada presa diminuir:

o predador pode:

* migrar;
* mudar de território;
* procurar outra presa;
* sofrer queda populacional.

---

# 13. FOOD WEB

Criar uma rede ecológica:

```text id="x2zqmd"
Plants
 ↓
Herbivores
 ↓
Predators
 ↓
Scavengers
```

---

# 14. RESOURCE DROPS

Mobs podem produzir:

```text id="q7a3sb"
food
materials
hides
bones
organic resources
rare resources
```

Esses recursos entram no sistema econômico.

---

# 15. HUNTING ECONOMY

A economia pode depender da fauna.

Exemplo:

```text id="8j7eh4"
caça
↓
carne
↓
mercado
↓
alimentação
```

ou:

```text id="w1s1we"
animal rare
↓
material
↓
artisan
↓
produto
```

---

# 16. MINING ECONOMY

A mineração alimentará:

```text id="qh8ktc"
blacksmiths
engineers
builders
traders
industrial systems
```

---

# 17. RESOURCE ECONOMY

O ciclo:

```text id="vv65i5"
resource
↓
collection
↓
processing
↓
production
↓
trade
↓
consumption
```

---

# 18. CIVILIZATION ENGINE

Criar:

```text id="0ae6o7"
CivilizationEngine
```

Responsável por:

* settlements;
* população;
* profissões;
* liderança;
* cultura;
* política;
* economia;
* infraestrutura;
* relações.

---

# 19. SETTLEMENT

Cada settlement possui:

```text id="m6fyat"
population
buildings
resources
economy
politics
culture
infrastructure
leadership
```

---

# 20. VILLAGE → TOWN → CITY

Crescimento baseado em:

```text id="mlocbw"
population
wealth
food
resources
infrastructure
security
trade
```

---

# 21. PROFISSÕES

Criar:

```text id="7w8i82"
ProfessionRegistry
```

Exemplos:

```text id="h4fz3w"
farmer
miner
hunter
fisher
blacksmith
merchant
builder
engineer
doctor
scholar
guard
leader
```

A lista será extensível.

---

# 22. PROFISSÃO + ECONOMIA

Profissões determinam produção e consumo.

Exemplo:

```text id="f4bjq8"
Miner
→ minério

Blacksmith
→ ferramentas

Farmer
→ comida

Merchant
→ comércio
```

---

# 23. PROFISSÕES DINÂMICAS

Uma população pode mudar sua distribuição de profissões dependendo de:

```text id="a7gzh4"
demanda
recursos
salários
escassez
tecnologia
```

---

# 24. MERCADO

Cada settlement poderá possuir:

```text id="ct4t6n"
market
warehouse
shops
prices
stock
```

---

# 25. PREÇO DINÂMICO

Preço deve depender de:

```text id="wio6ph"
supply
demand
distance
scarcity
production
trade
```

---

# 26. MOEDA

Cada settlement poderá possuir sua própria moeda.

```text id="03c64c"
Settlement A
→ Currency A

Settlement B
→ Currency B
```

---

# 27. CÂMBIO

Quando settlements comercializarem:

```text id="uw0jux"
Currency A
↕
Currency B
```

poderão existir taxas de câmbio.

---

# 28. TRADE NETWORK

Criar:

```text id="hz3bgx"
TradeNetwork
```

ligando:

```text id="bnzk0c"
settlement
settlement
city
mine
port
railway
```

---

# 29. VILAS CONVERSANDO ENTRE SI

As civilizações poderão trocar informações.

Exemplo:

```text id="pcp4j5"
Village A
↓
merchant
↓
Village B
```

Pode transportar:

* mercadoria;
* preços;
* notícias;
* pedidos;
* informações.

---

# 30. ROTAS COMERCIAIS

Criar:

```text id="j0e2z0"
TradeRoute
```

com:

* origem;
* destino;
* mercadorias;
* frequência;
* segurança;
* transporte.

---

# 31. FERROVIAS E ECONOMIA

Ferrovias poderão mudar completamente a economia.

```text id="yf8dtf"
mine
↓
rail
↓
city
↓
industry
```

Isso pode aumentar:

* produção;
* comércio;
* riqueza;
* população.

---

# 32. NPCs COM NECESSIDADES

NPCs podem possuir:

```text id="q9t5gq"
food
water
shelter
safety
income
health
social
```

---

# 33. DECISION SYSTEM

Criar:

```text id="5f1rml"
DecisionEngine
```

que calcula decisões possíveis.

Não usar scripts individuais para todas as situações.

---

# 34. OBJETIVOS

Cada NPC pode possuir objetivos:

```text id="jmucv4"
survive
work
eat
protect family
earn money
trade
learn
help settlement
```

---

# 35. MEMÓRIA

NPCs poderão lembrar:

```text id="k09nlh"
player interactions
events
trades
injuries
promises
important discoveries
```

---

# 36. REPUTAÇÃO

Criar:

```text id="p3teph"
ReputationSystem
```

Possuindo:

* individual;
* settlement;
* faction;
* regional.

---

# 37. PLAYER IMPACT

O jogador pode alterar:

```text id="it9fwm"
economy
politics
population
trade
safety
resources
```

---

# 38. ELEIÇÕES

Assentamentos suficientemente desenvolvidos poderão possuir eleições.

```text id="de90q9"
Population
↓
Candidates
↓
Campaign
↓
Election
↓
Leader
```

---

# 39. CANDIDATES

Candidatos terão:

```text id="btr4dr"
goals
personality
reputation
economic policies
social policies
relations
```

---

# 40. ELEIÇÃO NÃO DEVE SER DECORATIVA

O resultado deve modificar:

* impostos;
* comércio;
* prioridades;
* infraestrutura;
* segurança;
* relações.

---

# 41. INFLUÊNCIA DO JOGADOR

O jogador poderá influenciar eleições de maneira indireta através de suas ações.

Exemplo:

```text id="1ra5dh"
ajudou settlement
↓
reputation ↑
↓
NPCs mudam opinião
```

Mas o resultado não deve ser garantido pelo jogador.

---

# 42. NPCs TAMBÉM PODEM DISCORDAR

NPCs podem:

* apoiar;
* rejeitar;
* ignorar;
* desconfiar.

---

# 43. CIVILIZATION RELATIONS

Assentamentos podem ter:

```text id="b4pr1s"
friendly
neutral
trading
rival
hostile
allied
```

---

# 44. DIPLOMACIA

Civilizações poderão:

```text id="7p9dhi"
trade
negotiate
form alliances
settle disputes
```

---

# 45. CONFLITOS

Os conflitos devem ser simulados de maneira abstrata quando necessário.

Não exigir simulação individual de cada NPC em todo lugar do mundo.

---

# 46. CITIES AS SYSTEMS

Uma cidade pode possuir:

```text id="9o1th5"
food supply
industry
housing
education
health
market
government
transport
```

---

# 47. HEALTH SYSTEM

Criar:

```text id="zjwozv"
HealthSystem
```

para NPCs e criaturas.

---

# 48. DISEASES

Doenças podem se espalhar por populações.

Exemplo:

```text id="qf0d7l"
infection
↓
symptoms
↓
population impact
```

---

# 49. ZOMBIE → NPC

Um exemplo de comportamento emergente:

```text id="fh5z5c"
Zombie encounters NPC
↓
NPC becomes infected
↓
NPC state changes
↓
settlement detects illness
↓
doctor researches problem
↓
search for possible treatment
```

---

# 50. KNOWLEDGE GAP

O NPC não deve automaticamente saber a solução.

Ele deve possuir:

```text id="9nq1us"
known information
unknown information
hypotheses
```

---

# 51. DISCOVERY SYSTEM

Criar:

```text id="s8g6mm"
KnowledgeDiscoverySystem
```

NPCs podem descobrir:

```text id="4k6uq5"
new medicine
new tool
new farming technique
new material
new route
```

---

# 52. PESQUISA

NPCs poderão testar hipóteses.

Exemplo:

```text id="3r0mju"
doença
↓
observação
↓
hipótese
↓
experimento
↓
resultado
↓
conhecimento
```

---

# 53. LIMITES DA IA

O NPC não deve possuir inteligência perfeita.

Ele pode:

* errar;
* interpretar mal;
* desistir;
* tomar decisão ruim;
* aprender parcialmente.

---

# 54. O JOGADOR CONTINUA IMPORTANTE

O sistema não deve tornar o jogador inútil.

O jogador pode:

```text id="xlvp8q"
trazer conhecimento
fornecer recursos
salvar pessoas
fundar cidades
construir hospitais
criar rotas
mudar eleições
introduzir tecnologias
```

Mas o mundo também pode evoluir sem ele.

---

# 55. QUESTS EMERGENTES

Missões podem surgir do estado real.

Exemplo:

```text id="vhhjpd"
food shortage
↓
farmer asks for food
```

ou:

```text id="raf0py"
mine discovered
↓
miner asks for transport
```

---

# 56. QUEST CHAIN

Missões podem gerar novas missões.

```text id="h0a1c8"
cura
↓
ingrediente
↓
região
↓
pesquisa
↓
tratamento
```

---

# 57. WORLD KNOWLEDGE

Criar um Knowledge Graph para o mundo.

```text id="xq6b1v"
NPC
↓
knows
↓
resource
↓
location
```

---

# 58. KNOWLEDGE PROPAGATION

Informações podem viajar através de:

```text id="r8nv5w"
NPC
merchant
traveller
school
book
trade
settlement
```

---

# 59. NOTÍCIAS

Um evento importante pode gerar notícias.

Exemplo:

```text id="31n1so"
new mine discovered
↓
merchant hears
↓
village hears
↓
price changes
```

---

# 60. ECOLOGICAL CONSEQUENCES

Caçar demais uma espécie pode causar:

```text id="0ibsmh"
population decline
↓
predator starvation
↓
migration
↓
ecosystem change
```

---

# 61. MONSTER CONSEQUENCES

A presença de mobs hostis pode modificar settlements.

Exemplo:

```text id="ol1qvb"
hostile population rises
↓
settlement danger rises
↓
guard demand rises
↓
security spending rises
```

---

# 62. POPULATION MIGRATION

NPCs podem migrar por:

```text id="r4brfw"
famine
war
jobs
resources
climate
safety
```

---

# 63. CIVILIZATION FOUNDING

Novos settlements poderão surgir quando condições forem adequadas.

```text id="i58yl9"
resources
+
water
+
population
+
transport
=
new settlement
```

---

# 64. CIVILIZATION DECLINE

Também podem desaparecer.

Causas:

```text id="opqsaj"
famine
resource exhaustion
population loss
economic collapse
environmental change
```

---

# 65. HISTÓRIA EMERGENTE

Registrar:

```text id="qj1w1d"
settlement founded
leader elected
trade route opened
resource discovered
disease outbreak
population migrated
```

---

# 66. WORLD HISTORY DATABASE

Criar:

```text id="qj7e4z"
WorldHistory
```

para armazenar eventos importantes.

---

# 67. SIMULATION LEVELS

Nem tudo precisa ser simulado individualmente em todos os lugares.

Usar:

```text id="a0atc1"
FULL
REGIONAL
ABSTRACT
```

---

# 68. NEARBY SIMULATION

Perto do jogador:

```text id="xqdfh4"
NPC individual
mob individual
economy detailed
```

Longe:

```text id="ofjq7p"
population averages
production summaries
trade abstraction
```

---

# 69. PERFORMANCE

Isso é obrigatório.

Não simular:

```text id="d6e5ad"
2.000 mobs
+
10.000 NPCs
+
milhares de settlements
```

individualmente em cada tick global.

---

# 70. EVENT-DRIVEN SIMULATION

Usar eventos para processos que não precisam atualizar a cada frame.

Exemplo:

```text id="92k4in"
daily economy update
weekly settlement update
seasonal ecology
```

---

# 71. AI TICK

Diferentes sistemas possuirão diferentes frequências.

```text id="q9s5fz"
frame
second
minute
hour
day
week
season
```

---

# 72. LOD DA SIMULAÇÃO

Maior detalhe perto do jogador.

Menor detalhe longe.

---

# 73. MOB SPAWNING

Spawns considerarão:

```text id="n5rc13"
biome
food
water
temperature
population
predators
human activity
```

---

# 74. MOB CARRYING CAPACITY

Cada região terá capacidade ecológica aproximada.

---

# 75. OVERPOPULATION

População excessiva poderá causar:

```text id="4yjwii"
food shortages
disease
migration
resource pressure
```

---

# 76. MOB DOMESTICATION

Algumas espécies poderão ser domesticáveis.

---

# 77. FARMING

Animais domesticados podem entrar na economia agrícola.

---

# 78. HUNTING

Caça será parte da economia, ecologia e sobrevivência.

Não permitir spawn infinito artificialmente só para satisfazer o jogador.

---

# 79. TRADE BETWEEN VILLAGES

Uma cidade pode solicitar:

```text id="8r8ygf"
food
ore
wood
medicine
tools
```

Outra pode produzir.

---

# 80. MERCHANT AI

Mercadores escolherão rotas com base em:

```text id="5ql0cz"
profit
distance
risk
demand
```

---

# 81. SUPPLY CHAINS

Exemplo:

```text id="qtnh2w"
mine
↓
ore
↓
blacksmith
↓
tools
↓
farmer
↓
food
↓
city
```

---

# 82. ECONOMIC SHOCKS

Eventos poderão alterar preços:

```text id="ffcj29"
mine collapse
crop failure
disease
new technology
new trade route
```

---

# 83. TECHNOLOGY DIFFUSION

Uma civilização que descobrir algo pode influenciar outras.

```text id="udn8g7"
settlement A
↓
discovery
↓
merchant
↓
settlement B
↓
adoption
```

---

# 84. KNOWLEDGE WITHOUT PLAYER

NPCs poderão descobrir soluções por experimentação.

Não precisarão depender exclusivamente do jogador.

---

# 85. KNOWLEDGE WITH PLAYER

O jogador pode acelerar descobertas.

Exemplo:

```text id="7wq4b9"
NPC has problem
+
player provides artifact
↓
research progresses
```

---

# 86. SOCIAL KNOWLEDGE

NPCs podem ensinar outros NPCs.

---

# 87. CIVILIZATION TECH TREE

Cada civilização pode possuir níveis de conhecimento:

```text id="71jqt7"
primitive
developing
industrial
advanced
```

O sistema é conceitual e deve ser adaptado à história do NEXORA.

---

# 88. CULTURAL DIFFERENCES

Duas civilizações podem interpretar recursos e tecnologias de maneiras diferentes.

---

# 89. POLITICAL SYSTEM

Criar:

```text id="u3g5dz"
PoliticalSystem
```

com:

```text id="1jpsqj"
leaders
parties/factions
laws
elections
approval
```

---

# 90. POLITICAL OPINION

NPCs podem possuir opiniões diferentes dependendo de:

```text id="4gqvqx"
wealth
profession
age
culture
history
player reputation
```

---

# 91. ELECTION SIMULATION

A eleição deve possuir:

```text id="q7u9u0"
voters
candidates
issues
campaigns
result
```

---

# 92. PLAYER ELECTION IMPACT

Ações do jogador podem afetar:

```text id="it8q7s"
candidate reputation
public opinion
economy
security
```

Mas o sistema continua probabilístico.

---

# 93. CIVILIZATION API

Outros módulos poderão registrar:

```text id="e3q6dk"
Profession
Settlement Type
Currency
Faction
Political System
Trade Good
```

---

# 94. MOB API

Mods poderão registrar:

```text id="42k8i3"
Species
Mob
Behavior
Habitat
Diet
Drops
Spawn Rule
```

---

# 95. ECONOMY API

Mods poderão registrar:

```text id="x0y9cv"
Currency
Market
Trade Good
Profession
Production Chain
```

---

# 96. KNOWLEDGE API

Mods poderão registrar:

```text id="xqg15f"
Knowledge
Discovery
Research
Treatment
Technology
```

---

# 97. EVENT BUS

Os quatro sistemas devem conversar através de eventos.

Exemplo:

```text id="8l9b7r"
MOB_INFECTED_NPC
NPC_ILL
MEDICINE_DISCOVERED
PRICE_CHANGED
SETTLEMENT_GROWTH
ELECTION_STARTED
ELECTION_FINISHED
TRADE_ROUTE_CREATED
SPECIES_POPULATION_CHANGED
```

---

# 98. EXEMPLO COMPLETO — DOENÇA

```text id="q1tql8"
Hostile Mob
↓
NPC infection
↓
Health System
↓
NPC becomes ill
↓
Settlement notices increase in illness
↓
Doctor investigates
↓
Knowledge System
↓
Hypothesis
↓
Research
↓
Treatment discovered
↓
Knowledge spreads
↓
Disease declines
```

O jogador pode interferir.

Mas não precisa ser ele quem descobre tudo.

---

# 99. EXEMPLO COMPLETO — RECURSO

```text id="6x8p3f"
Far Lands resource
↓
miner discovers
↓
settlement economy
↓
merchant learns
↓
trade route
↓
railway
↓
other city learns
↓
technology develops
```

---

# 100. EXEMPLO COMPLETO — ECOLOGIA

```text id="wqdk3n"
player hunts species
↓
population falls
↓
predator loses food
↓
predator migrates
↓
new prey pressure
↓
biome changes
↓
settlement notices resource shortage
```

---

# 101. DEFINIÇÃO DE SUCESSO

O sistema será considerado funcional quando o mundo conseguir produzir cadeias como:

```text id="bq73ot"
MOB
↓
RESOURCE
↓
ECONOMY
↓
PROFESSION
↓
SETTLEMENT
↓
POLITICS
↓
PLAYER
↓
WORLD CHANGE
```

e:

```text id="8q1ezi"
EVENT
↓
NPC
↓
DISCOVERY
↓
KNOWLEDGE
↓
SOLUTION
↓
SOCIETY
```

sem exigir que o jogador desencadeie cada passo.

---

# 102. REGRA CONTRA "NPC BURRO"

O objetivo não é criar uma IA que saiba tudo.

O objetivo é criar NPCs que:

```text id="a7r3mf"
percebam
↓
pensem
↓
tentem
↓
errem
↓
aprendam
```

---

# 103. REGRA CONTRA "IA MÁGICA"

NPCs não devem simplesmente receber:

```text id="qkz3mx"
solution = cure
```

do nada.

Conhecimento deve ter origem.

---

# 104. CONHECIMENTO TEM CUSTO

Descoberta pode exigir:

```text id="6i1cg5"
tempo
recursos
observação
experimentos
profissionais
livros
interação
```

---

# 105. JOGADOR COMO CATALISADOR

O jogador pode acelerar sistemas:

```text id="t2cxr4"
conhecimento
economia
tecnologia
política
exploração
```

sem precisar ser a única fonte.

---

# 106. PERFORMANCE GLOBAL

Criar scheduler:

```text id="zqxpz4"
SimulationScheduler
```

com prioridades.

---

# 107. PRIORIDADES

```text id="o29k1s"
player-near simulation
>
important events
>
regional economy
>
far-away abstraction
```

---

# 108. SAVE SYSTEM

Persistir:

```text id="4grrn2"
population
settlements
economy
relationships
knowledge
politics
important events
```

Não salvar tudo por NPC se houver forma mais eficiente.

---

# 109. DETERMINISMO

A simulação deve ser reproduzível quando necessário.

---

# 110. WORLD HISTORY

Criar timeline:

```text id="x0qay4"
Year 1
Settlement founded

Year 8
Mine discovered

Year 12
Railway built

Year 17
Election

Year 25
Disease outbreak
```

---

# 111. WORLD MEMORY

O mundo precisa lembrar das mudanças importantes.

---

# 112. DEBUG TOOLS

Criar ferramentas:

```text id="8c8tq5"
population map
mob map
economy map
trade map
political map
knowledge map
disease map
```

---

# 113. SIMULATION INSPECTOR

Permitir selecionar:

```text id="w8qv1l"
NPC
Mob
Settlement
```

e ver:

```text id="qnn6xc"
needs
goals
health
profession
knowledge
relationships
```

---

# 114. TEST WORLD

Criar um mundo pequeno especificamente para testes de simulação.

---

# 115. TEST SCENARIO — VILA

Criar:

```text id="w0w6yh"
2 villages
1 mine
1 forest
1 hostile species
1 friendly species
trade route
```

e testar crescimento.

---

# 116. TEST SCENARIO — DISEASE

Criar automaticamente:

```text id="1n2ekn"
infected NPC
doctor
settlement
unknown disease
```

Verificar se o sistema consegue investigar.

---

# 117. TEST SCENARIO — ECONOMY

Criar:

```text id="n1eqc7"
resource shortage
```

e observar:

```text id="2t7k7v"
price
production
trade
migration
```

---

# 118. TEST SCENARIO — ELECTION

Criar:

```text id="8hgryd"
settlement
candidates
population
issues
```

e verificar resultado.

---

# 119. INTEGRAÇÃO COM WORLD GENERATION

O World Generator fornece:

```text id="b9b7ad"
biome
resources
terrain
water
climate
```

O Mob/Civilization Engine consome.

---

# 120. INTEGRAÇÃO COM CAVE ENGINE

O Cave Engine fornece:

```text id="lhqs6u"
cave
depth
space
water
resources
```

O Deep World decide:

```text id="e5mksj"
biome
settlement
civilization
```

---

# 121. INTEGRAÇÃO COM DIMENSIONS

Cada dimensão pode possuir:

```text id="f2q8r4"
its own species
civilizations
economies
politics
knowledge
```

---

# 122. VOID DIMENSION

A Void Dimension poderá possuir civilizações e espécies completamente próprias.

---

# 123. FAR LANDS

As Far Lands poderão possuir:

```text id="7w6hsp"
fauna
resources
settlements
trade
rare species
```

---

# 124. MODDING

Tudo que for público deve ser extensível:

```text id="1rz8ae"
Mobs
Species
Professions
Settlements
Currencies
Trade Goods
Economies
Knowledge
Political Systems
```

---

# 125. OFFICIAL CONTENT

O conteúdo oficial do NEXORA também deve utilizar essas APIs.

Assim:

```text id="yuf4os"
Official Mob
=
External Mod Mob
```

no ponto de vista do engine.

---

# 126. DEFINIÇÃO FINAL DA ARQUITETURA

```text id="x22w8o"
                 NEXORA WORLD
                      │
            ┌─────────┴─────────┐
            │                   │
       ENVIRONMENT          SIMULATION
            │                   │
      ┌─────┴─────┐       ┌─────┴──────────────┐
      │           │       │                    │
   TERRAIN      CAVES    MOBS           CIVILIZATION
      │           │       │                    │
      └─────┬─────┘       ├──────────┬─────────┤
            │              │          │         │
            ▼              ▼          ▼         ▼
         BIOMES         ECOLOGY    ECONOMY   POLITICS
                               │
                               ▼
                           KNOWLEDGE
                               │
                               ▼
                        EMERGENT HISTORY
```

---

# 127. PRINCÍPIO FINAL

O mundo deve funcionar mesmo quando o jogador não está olhando.

Mas:

> **o mundo nunca deve ignorar o jogador.**

O jogador pode mudar uma vila.

A vila pode mudar uma região.

Uma região pode mudar uma economia.

Uma economia pode mudar uma civilização.

Uma civilização pode mudar outra.

Um mob pode provocar uma crise.

Uma crise pode provocar uma descoberta.

Uma descoberta pode mudar o futuro.

---

# FRASE DO SISTEMA

> **The world does not wait for the player.**
>
> **It reacts to the player.**
