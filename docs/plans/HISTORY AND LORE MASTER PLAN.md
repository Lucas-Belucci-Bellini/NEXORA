# NEXORA — HISTORY AND LORE MASTER PLAN

> **Visão:**
> **NEXORA não terá uma única história linear. Cada instância do mundo possui uma trajetória própria, produzida pela geração do mundo, pela simulação, pelas decisões de NPCs e civilizações e pelas ações do jogador.**

Este documento integra:

```text
World Events
History
Knowledge
Lore
Archives / Evidence
Civilization
AI
Time
Simulation LOD
Player Agency
```

---

# 1. Objetivo narrativo

NEXORA deve construir um mundo que:

```text
nasce
→ funciona
→ muda
→ lembra
→ esquece
→ registra
→ interpreta
→ conta histórias
```

A história não deve ser uma camada artificial colocada sobre o gameplay.

Ela deve ser uma consequência do próprio mundo.

---

# 2. Princípio de desenvolvimento

O jogo será construído primeiro como sistema.

A narrativa será construída em paralelo a partir daquilo que o sistema é capaz de produzir.

```text
GAME SYSTEMS
      ↓
SIMULATION
      ↓
EMERGENT EVENTS
      ↓
OBSERVATION
      ↓
HISTORICAL RECORD
      ↓
LORE
```

Isso significa:

- primeiro definimos as regras que permitem uma sociedade existir;
- depois observamos o que essas regras produzem;
- acontecimentos interessantes tornam-se parte da história daquele mundo;
- novos sistemas narrativos são criados quando surgirem necessidades reais.

---

# 3. Não existe uma única timeline universal

Existe:

```text
UNIVERSE CANON
```

que define princípios, cosmologia e fatos essenciais estabelecidos pelo design.

Depois existem infinitas:

```text
WORLD HISTORIES
```

Uma para cada instância do mundo.

Exemplo:

```text
WORLD 001
→ Empire X survives

WORLD 002
→ Empire X collapses

WORLD 003
→ Empire X never forms
```

A mesma semente ou configuração deve produzir resultados reprodutíveis quando as regras exigirem determinismo, mas mundos diferentes podem seguir trajetórias diferentes.

---

# 4. Camadas narrativas

```text
CANON
 ↓
WORLD RULES
 ↓
SIMULATION
 ↓
HISTORY
 ↓
KNOWLEDGE
 ↓
LORE
 ↓
PLAYER EXPERIENCE
```

### Canon

Define o universo e suas regras fundamentais.

### World Rules

Definem o conjunto de sistemas disponíveis naquela configuração do mundo.

### Simulation

Produz estados e acontecimentos.

### History

Registra o que efetivamente aconteceu.

### Knowledge

Define quem descobriu ou recebeu informação.

### Lore

Transforma informação em narrativas culturais.

### Player Experience

É o que o jogador consegue observar e interpretar.

---

# 5. O mundo funciona sem o jogador

O estado do mundo não pode depender da presença do jogador.

```text
PLAYER ONLINE
≠
WORLD ACTIVE
```

Civilizações devem continuar:

```text
produzindo
negociando
pesquisando
migrando
tomando decisões
mudando governos
criando infraestrutura
entrando em conflitos
```

quando o jogador estiver longe.

---

# 6. O jogador não é um protagonista obrigatório

O jogador pode se tornar:

```text
irrelevante
localmente importante
regionalmente importante
historicamente importante
globalmente importante
```

Isso deve depender das consequências reais das ações.

Não existe obrigação de transformar toda campanha em “a história do herói”.

---

# 7. História emergente

O sistema deve gerar cadeias causais.

Exemplo genérico:

```text
policy change
↓
public dissatisfaction
↓
organization
↓
coalition
↓
political conflict
↓
change of government
↓
migration
↓
economic restructuring
↓
new historical era
```

Nenhum desses passos precisa ser uma missão pré-escrita.

---

# 8. Exemplo: império das 25 vilas

Cenário inicial:

```text
25 settlements
1 central government
shared economy
shared infrastructure
```

Ações do jogador:

```text
centralization
higher taxes
reduced autonomy
political repression
```

A simulação pode produzir:

```text
local dissatisfaction
↓
communication among settlements
↓
political coordination
↓
coalition
```

Cada vila avalia individualmente:

```text
join rebellion
remain neutral
support government
seek negotiation
migrate
```

Possíveis resultados:

```text
0 rebels
5 rebels
12 rebels
23 rebels
25 rebels
```

Não existe resultado obrigatório.

---

# 9. Segredo e descoberta

Uma ação importante pode permanecer desconhecida.

```text
PLAYER ACTION
↓
NO WITNESS
↓
NO EVIDENCE RECOVERED
↓
NO KNOWLEDGE
```

Ou:

```text
PLAYER ACTION
↓
WITNESS
↓
MEMORY
↓
MESSAGE
↓
FACTION KNOWLEDGE
↓
INVESTIGATION
↓
WORLD RESPONSE
```

A informação precisa possuir um caminho.

---

# 10. Exemplo: instalação secreta

```text
player builds hidden facility
↓
physical traces exist
↓
local NPC notices unusual movement
↓
rumor appears
↓
investigator checks evidence
↓
report sent to faction
↓
faction evaluates response
```

Também é válido:

```text
no one notices
↓
facility remains secret
```

O sistema não pode dar conhecimento telepático aos NPCs apenas para gerar gameplay.

---

# 11. Conhecimento limitado

A regra fundamental da IA é:

```text
WORLD TRUTH
≠
NPC KNOWLEDGE
≠
PLAYER KNOWLEDGE
```

Um NPC toma decisões sobre seu modelo mental do mundo, não sobre o banco de dados global.

---

# 12. A mesma história pode possuir versões diferentes

Exemplo:

```text
FACT:
Empire collapsed after a coalition war.
```

Narrativas possíveis:

```text
Government:
"Foreign powers caused the collapse."

Rebels:
"The people liberated themselves."

Academic institution:
"The collapse had multiple structural causes."

Village tradition:
"Our ancestors broke the emperor's army."
```

O fato histórico continua sendo um objeto separado.

---

# 13. História e evidência

A arquitetura completa é:

```text
EVENT
↓
HISTORY
↓
EVIDENCE
↓
KNOWLEDGE
↓
LORE
```

Nem todo evento possui boa evidência.

Nem toda evidência é autêntica.

Nem todo conhecimento é correto.

Nem toda lore é factual.

---

# 14. Histórico reconstruível

Eventos importantes devem possuir explicações causais suficientes para responder:

```text
What happened?
Why?
Who participated?
What changed?
What happened afterward?
```

Ferramentas internas devem conseguir navegar:

```text
cause
→ event
→ consequence
→ secondary event
```

---

# 15. Histórico como banco de dados do mundo

O jogo deve poder construir consultas como:

```text
History of settlement X
History of civilization Y
Wars involving faction Z
Origin of current border
Origin of technology Q
Events that caused city decline
People responsible for institution founding
```

Essas consultas podem alimentar gameplay e ferramentas.

---

# 16. Lore como conteúdo derivado

Lore pode aparecer como:

```text
books
archives
monuments
museum entries
news
biographies
folk stories
songs/poetry metadata
rituals
education
quests
dialogue
```

A implementação textual deve ser baseada em dados estruturados.

---

# 17. Geração de texto

Quando uma camada generativa for usada:

```text
SOURCE FACTS
+
CULTURAL CONTEXT
+
AUTHOR PROFILE
+
NARRATIVE FORM
+
STYLE RULES
=
TEXT
```

A camada generativa não pode criar fatos históricos fora dos dados autorizados.

---

# 18. Cultura

Cada cultura pode possuir filtros:

```text
values
symbols
language
traditions
political memory
religious beliefs
historical grievances
```

Isso altera como os eventos são narrados e preservados.

---

# 19. Memória social

A morte de indivíduos não deve apagar automaticamente a memória de acontecimentos.

Instituições podem preservar:

```text
archives
monuments
books
records
traditions
```

Enquanto comunidades sem instituições podem perder detalhes históricos ao longo do tempo.

---

# 20. Esquecimento

O esquecimento é permitido.

```text
FACT
↓
RECORD LOST
↓
MEMORY DECLINES
↓
MYTH REMAINS
```

Décadas ou séculos depois:

```text
new discovery
↓
old event reinterpreted
```

---

# 21. Importância histórica

O sistema deve identificar eventos com impacto potencial em:

```text
politics
economy
population
technology
territory
culture
ecology
infrastructure
dimensions
space
```

A importância pode aumentar depois de um evento aparentemente pequeno produzir consequências grandes.

---

# 22. Tempo histórico

O sistema usa o World Time/Calendar como referência.

Eventos importantes possuem:

```text
world time
duration
era
relative order
causal order
```

A história nunca deve depender apenas do horário real do computador.

---

# 23. LOD e continuidade histórica

Uma região abstrata pode continuar tendo história.

```text
FULL
→ detailed simulation

REGIONAL
→ summarized simulation

ABSTRACT
→ statistical / causal simulation
```

Ao acordar uma região, o sistema reconcilia a história produzida em baixa resolução com o estado detalhado.

---

# 24. História da sociedade

Devemos preservar:

```text
settlement histories
civilization histories
faction histories
government histories
institution histories
family / dynasty histories
technology histories
industrial histories
migration histories
```

---

# 25. História do jogador

O jogador terá uma trajetória histórica própria derivada de seus atos.

```text
Player History
├── actions
├── relationships
├── offices
├── discoveries
├── settlements
├── projects
├── conflicts
├── achievements
└── consequences
```

A relevância é calculada pelo impacto real, não por uma barra de “protagonismo”.

---

# 26. História pós-jogador

Se o jogador morrer, desaparecer ou abandonar uma região, o mundo continua.

```text
PLAYER
↓
LEGACY
↓
MEMORY
↓
INSTITUTIONAL RECORD
↓
LORE
```

A sociedade pode lembrar o jogador de maneiras positivas, negativas, neutras ou contraditórias.

---

# 27. Timeline global

Ferramentas e UI poderão construir:

```text
WORLD TIMELINE
CONTINENT TIMELINE
CIVILIZATION TIMELINE
CITY TIMELINE
FACTION TIMELINE
PLAYER TIMELINE
```

---

# 28. Event causality graph

Cada grande evento pode ser expandido:

```text
Event A
├── caused B
├── contributed to C
├── prevented D
└── enabled E
```

Isso permite analisar história complexa sem reduzir tudo a uma sequência linear.

---

# 29. Regras de autoria

A história do mundo é uma combinação de:

```text
SYSTEM RULES
+
SIMULATION OUTCOME
+
PLAYER ACTIONS
+
NPC ACTIONS
```

A Lore é produzida a partir desses dados por entidades do mundo e, quando autorizado, por ferramentas narrativas procedurais.

---

# 30. Integração com o restante do NEXORA

```text
TIME
 ↓
WORLD / CIVILIZATION / ECONOMY / AI / PLAYER
 ↓
WORLD EVENTS
 ↓
HISTORY
 ↓
KNOWLEDGE
 ↓
LORE / ARCHIVE
 ↓
QUEST / DIALOGUE / UI / EDUCATION / CULTURE
```

---

# 31. Performance

O sistema não pode registrar cada microação como documento histórico pesado.

Usar:

```text
event filtering
importance thresholds
aggregation
causal compression
LOD
cold storage
summary records
```

Eventos importantes recebem maior retenção.

---

# 32. Determinismo e reprodução

Para eventos derivados de simulação determinística, o sistema deve preservar dados suficientes para reprodução ou auditoria.

Isso ajuda:

```text
bug reproduction
replays
save/load
historical audit
benchmark
```

---

# 33. Multiplayer

Em multiplayer:

```text
SERVER
→ authority over world truth

CLIENT
→ receives permitted knowledge / presentation
```

Nenhum cliente pode fabricar fatos históricos autoritativos.

---

# 34. Mods

Mods podem adicionar:

```text
event types
cultures
narrative forms
archive types
knowledge sources
lore generators
```

Mas devem respeitar:

```text
History integrity
permissions
provenance
save schema
network authority
```

---

# 35. Development roadmap

## Phase H1 — Contracts

```text
HistoryEvent
EvidenceItem
KnowledgeItem
LoreItem
IDs
timestamps
causality
provenance
```

## Phase H2 — Recording

```text
World Events → History
Civilization → History
Player → History
```

## Phase H3 — Knowledge

```text
observation
memory
communication
rumor
source trust
```

## Phase H4 — Archive

```text
documents
records
ruins
monuments
preservation
```

## Phase H5 — Lore

```text
books
biographies
news
myths
cultural narratives
```

## Phase H6 — Dynamic history

```text
long-term consequences
historical eras
collective memory
historical reinterpretation
```

## Phase H7 — Player-driven history

```text
player legacy
political effects
faction reactions
investigations
```

## Phase H8 — Production hardening

```text
LOD
compression
performance
multiplayer
modding
migration
replay
```

---

# 36. Minimum viable historical world

Antes de buscar narrativa complexa, o engine precisa conseguir demonstrar:

```text
1 world
3+ settlements
multiple factions
population
basic economy
political decisions
World Events
persistent history
knowledge propagation
one secret
one investigation
one major political change
save/load
```

Depois disso, podemos expandir.

---

# 37. Success criteria

Consideramos o sistema bem-sucedido quando uma execução do jogo consegue produzir um caso como:

```text
Player becomes ruler
↓
policy changes world
↓
settlements react differently
↓
coalition forms
↓
conflict occurs
↓
government changes
↓
history records causal chain
↓
some NPCs know what happened
↓
others believe another version
↓
archives preserve partial evidence
↓
future NPCs reinterpret the event
```

Sem uma missão escrita especificamente para produzir essa sequência.

---

# 38. Regra final

> **A história de NEXORA não é uma sequência de capítulos pronta. É a memória de um mundo que continua vivendo.**

> **Cada seed pode criar uma história diferente. Cada decisão pode alterar a linha do tempo. Cada habitante conhece apenas uma parte. E o jogador pode ser responsável por acontecimentos que o mundo lembrará muito depois que ele já tiver partido.**
