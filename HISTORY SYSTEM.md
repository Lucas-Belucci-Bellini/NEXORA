# NEXORA — HISTORY SYSTEM

> **Princípio central:**
> **A História não é uma campanha pré-escrita. É o registro causal, persistente e reconstruível dos acontecimentos que realmente ocorreram em uma instância do mundo.**

O History System transforma mudanças relevantes produzidas pela simulação em uma memória histórica estruturada.

Ele não controla guerras, governos, economia, IA ou clima. Esses sistemas são autoridades sobre seus próprios domínios. O History System observa resultados, estabelece relações causais, registra importância e preserva a trajetória histórica do mundo.

---

# 1. Objetivo

O sistema deve permitir que NEXORA responda:

```text
O que aconteceu?
Quando aconteceu?
Onde aconteceu?
Quem participou?
Por que aconteceu?
O que causou?
O que mudou depois?
Quem sabe disso?
Qual evidência existe?
Como esse acontecimento afetou o mundo?
```

A História deve existir mesmo quando nenhum jogador presencia o acontecimento.

---

# 2. História não é Lore

```text
HISTORY
→ registro do estado factual da simulação

KNOWLEDGE
→ o que uma entidade conhece ou acredita conhecer

LORE
→ narrativas, interpretações, tradições, mitos e versões culturais
```

Um fato histórico pode gerar várias interpretações.

```text
HISTORICAL FACT
      ↓
KNOWLEDGE
      ↓
INTERPRETATION
      ↓
LORE
```

A Lore pode estar errada. O registro histórico interno não deve ser reescrito para concordar com uma crença.

---

# 3. Fontes de acontecimentos

O History System pode receber fatos de:

```text
World Events
Civilization
Politics
Economy
Industry
Technology
Research
Migration
Settlement
Ecology
Climate
Disasters
Exploration
Space
Dimensions
Player Actions
Quest Outcomes
Institutional Decisions
```

Fluxo:

```text
SYSTEM OF RECORD
      ↓
EVENT / STATE CHANGE
      ↓
HISTORY ADAPTER
      ↓
VALIDATION
      ↓
HISTORICAL RECORD
```

---

# 4. History Event

Um acontecimento histórico é uma entidade persistente.

```text
HistoryEvent
├── id
├── worldId
├── eraId
├── timestamp
├── duration
├── type
├── scope
├── location
├── actors
├── organizations
├── causes
├── effects
├── consequences
├── parentEvents
├── childEvents
├── evidenceRefs
├── importance
├── certainty
├── visibility
├── tags
└── provenance
```

---

# 5. Causalidade

A História deve preservar causalidade suficiente para reconstrução.

```text
CONDITION
   ↓
CAUSE
   ↓
EVENT
   ↓
EFFECT
   ↓
CONSEQUENCE
   ↓
SECONDARY EVENT
```

Exemplo:

```text
High taxation
    ↓
public dissatisfaction
    ↓
political organization
    ↓
settlements coordinate
    ↓
rebellion
    ↓
civil conflict
    ↓
government collapse
    ↓
new political order
```

Não é necessário afirmar causalidade absoluta quando a simulação não possui evidência suficiente. Pode existir:

```text
CONFIRMED
STRONG
PROBABLE
POSSIBLE
UNKNOWN
```

---

# 6. Historical Graph

A história será representada como um grafo temporal e causal.

```text
                 EVENT A
                /      \
             cause      cause
              ↓          ↓
           EVENT B     EVENT C
              \          /
               consequence
                    ↓
                 EVENT D
```

Isso permite:

- timelines;
- genealogias;
- cadeias de guerra e tratados;
- evolução de governos;
- surgimento e queda de cidades;
- descoberta e adoção de tecnologias;
- crises econômicas;
- migrações;
- causas de mudanças territoriais.

---

# 7. Escala histórica

Nem toda mudança merece o mesmo peso.

```text
MICRO
LOCAL
SETTLEMENT
REGIONAL
NATIONAL
CONTINENTAL
GLOBAL
DIMENSIONAL
COSMIC
```

Um evento pode existir em múltiplas escalas por meio de efeitos derivados.

Exemplo:

```text
Local drought
→ crop failure
→ regional food crisis
→ migration
→ political instability
```

---

# 8. Importância

Importância não é a mesma coisa que magnitude.

```text
MAGNITUDE
→ tamanho físico/intensidade do acontecimento

IMPACT
→ dano ou alteração efetivamente produzido

HISTORICAL IMPORTANCE
→ relevância para a trajetória futura do mundo
```

Um evento pequeno pode possuir enorme importância histórica.

Exemplo:

```text
Uma descoberta aparentemente pequena
        ↓
novo processo industrial
        ↓
queda no custo de produção
        ↓
expansão econômica
        ↓
mudança de poder entre civilizações
```

---

# 9. Eventos do jogador

Ações do jogador devem entrar no mesmo sistema histórico quando produzem mudanças relevantes.

```text
PLAYER ACTION
      ↓
GAME SYSTEM
      ↓
WORLD STATE CHANGE
      ↓
HISTORICAL EVENT
```

O jogador não recebe tratamento especial no modelo histórico.

```text
NPC_ACTION
PLAYER_ACTION
CIVILIZATION_ACTION
NATURAL_EVENT
```

Todos podem produzir consequências persistentes.

---

# 10. Exemplo: queda de um império

```text
Player assumes government
        ↓
centralizes authority
        ↓
raises taxation
        ↓
autonomy decreases
        ↓
settlement dissatisfaction increases
        ↓
regional communication increases
        ↓
coalition forms
        ↓
political conflict begins
        ↓
empire loses control
        ↓
new governments emerge
```

O sistema não deve simplesmente executar:

```text
if tyranny > threshold:
    empire_falls = true
```

A queda deve emergir da interação entre economia, política, população, capacidade institucional, informação, diplomacia e outros sistemas.

---

# 11. O mundo não depende do jogador

O History System deve aceitar eventos produzidos sem presença do jogador.

```text
PLAYER PRESENT
→ simulation

PLAYER ABSENT
→ simulation
```

O fato de uma região estar distante do jogador não significa que sua história está congelada.

A simulação usa LOD:

```text
FULL
→ REGIONAL
→ ABSTRACT
→ UNRESIDENT
```

O histórico relevante deve sobreviver à redução de representação.

---

# 12. Histórico sem jogador

Exemplos:

```text
cidade muda de governo
nova rota comercial é aberta
guerra termina
população migra
descoberta científica acontece
infraestrutura colapsa
aliança é criada
civilização entra em decadência
novo assentamento surge
```

O jogador pode descobrir essas mudanças depois.

---

# 13. Reação atrasada

A consequência histórica pode surgir muito depois do evento original.

```text
EVENT A
↓
latent consequence
↓
100 years
↓
EVENT B
```

Isso é importante para tecnologia, ambiente, demografia, cultura e política.

---

# 14. Histórico das instituições

O sistema deve registrar trajetórias de:

```text
settlements
cities
states
empires
factions
guilds
companies
schools
research institutions
religious/cultural institutions
rail networks
industrial sites
```

Exemplo:

```text
Settlement X
→ founded
→ expanded
→ became city
→ changed government
→ entered federation
→ suffered decline
→ abandoned
→ archaeological site
```

---

# 15. Eras

O mundo pode possuir eras emergentes.

```text
EARLY SETTLEMENT ERA
        ↓
REGIONAL KINGDOM ERA
        ↓
INDUSTRIAL ERA
        ↓
SPACE ERA
        ↓
DIMENSIONAL ERA
```

As eras não precisam ser apenas datas fixas.

Podem depender de condições históricas, tecnológicas e sociais.

---

# 16. Historical Identity

Cada civilização, cidade e facção deve poder possuir um resumo histórico derivado dos registros.

```text
CivilizationHistory
├── founding
├── leaders
├── major wars
├── alliances
├── migrations
├── discoveries
├── technological milestones
├── crises
├── territorial changes
├── cultural periods
└── legacy
```

---

# 17. Historical truth boundary

O sistema deve distinguir:

```text
SIMULATION TRUTH
```

de:

```text
CHARACTER KNOWLEDGE
```

de:

```text
CULTURAL NARRATIVE
```

Isso é fundamental para que o mundo possa ter propaganda, rumores, interpretações conflitantes e versões incompletas sem corromper a simulação.

---

# 18. Descoberta e auditoria

O sistema deve permitir reconstrução interna:

```text
HistoryEvent
→ source event
→ source system
→ state transition
→ timestamp
→ evidence
```

Isso ajuda:

- debugging;
- replays;
- testes determinísticos;
- explicabilidade da IA;
- ferramentas de desenvolvimento;
- investigação no jogo.

---

# 19. Persistência

História é estado persistente.

Ela deve sobreviver a:

```text
save
reload
server restart
chunk unload
region unload
LOD conversion
world migration
version migration
```

Eventos extremamente antigos podem ser compactados em eras/resumos sem perder relações fundamentais.

---

# 20. Compactação histórica

A história terá níveis de retenção.

```text
RECENT
→ full event data

OLD
→ compressed event + causal links

ANCIENT
→ era summary + primary records
```

O objetivo é impedir crescimento ilimitado do custo de armazenamento.

---

# 21. API conceitual

```text
History.record(event)
History.linkCause(cause, effect)
History.addParticipant(event, actor)
History.attachEvidence(event, evidence)
History.setImportance(event, score)
History.queryTimeline(scope, range)
History.queryCauses(event)
History.queryConsequences(event)
History.queryActorHistory(actor)
History.querySettlementHistory(settlement)
History.queryEra(scope)
```

A implementação concreta poderá variar conforme a linguagem/runtime final.

---

# 22. Segurança e integridade

Mods e scripts não podem escrever arbitrariamente na verdade histórica.

A criação de registros deve vir de:

```text
validated game state
trusted event adapters
authorized APIs
```

Mods podem criar tipos de evento próprios, desde que respeitem o contrato do sistema e suas permissões.

---

# 23. Testes obrigatórios

```text
causal graph integrity
chronology ordering
duplicate event rejection
idempotency
save/load preservation
LOD persistence
migration
large history query
history compaction
cross-system provenance
player-generated events
NPC-generated events
```

---

# 24. Integração

```text
World Systems
      ↓
World Events / Domain Events
      ↓
History Adapter
      ↓
History Graph
      ↓
Knowledge System
      ↓
Lore System
      ↓
Archives / UI / Quests / Dialogue
```

---

# 25. Regra final

> **O NEXORA não possui uma única história pré-escrita. Cada mundo possui uma trajetória histórica própria produzida pela simulação, pelas decisões de seus habitantes e pelas ações dos jogadores.**

A história oficial do universo define possibilidades e leis. A instância do mundo define o que realmente aconteceu.
