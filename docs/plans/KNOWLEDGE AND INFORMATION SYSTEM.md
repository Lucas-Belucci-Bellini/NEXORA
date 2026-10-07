# NEXORA — KNOWLEDGE AND INFORMATION SYSTEM

> **Princípio central:**
> **Informação não é magia. Uma entidade só deve saber aquilo que conseguiu observar, receber, deduzir, pesquisar ou preservar.**

Este sistema modela percepção social, memória, comunicação, descoberta, rumor, evidência e distribuição de conhecimento.

---

# 1. Por que este sistema existe?

NEXORA possui um mundo em que ações podem acontecer sem serem imediatamente conhecidas.

Exemplo:

```text
PLAYER ACTION
↓
EVENT
↓
Ninguém observa
↓
Public knowledge = none
```

Outro caso:

```text
PLAYER ACTION
↓
Witness
↓
Memory
↓
Communication
↓
Faction learns
↓
Public awareness grows
```

O segundo caminho deve custar tempo, depender de infraestrutura e poder falhar.

---

# 2. Verdade x conhecimento

```text
WORLD TRUTH
    ↓
OBSERVATION / EVIDENCE
    ↓
KNOWLEDGE
    ↓
BELIEF / INTERPRETATION
    ↓
LORE / PROPAGANDA / RUMOR
```

A existência de um fato não implica que todas as entidades saibam dele.

---

# 3. Tipos de informação

```text
OBSERVATION
EVIDENCE
FACT
REPORT
MEMORY
RUMOR
CLAIM
BELIEF
PROPAGANDA
WARNING
SECRET
PUBLIC_RECORD
ARCHIVE
DISCOVERY
```

---

# 4. Information Item

```text
InformationItem
├── id
├── subject
├── claims
├── source
├── timestamp
├── location
├── confidence
├── freshness
├── reliability
├── secrecy
├── audience
├── evidenceRefs
└── provenance
```

---

# 5. Conhecimento individual

Cada NPC importante pode possuir uma representação compacta de conhecimento.

```text
KnowledgeState
├── known facts
├── uncertain facts
├── beliefs
├── rumors
├── secrets
├── memories
├── skills
└── source trust
```

NPCs comuns não precisam possuir uma base de dados gigantesca. O modelo deve ser escalável e usar abstração por população/LOD quando necessário.

---

# 6. Conhecimento coletivo

Além da memória individual:

```text
PERSON
→ FAMILY
→ GROUP
→ SETTLEMENT
→ FACTION
→ CIVILIZATION
→ WORLD
```

Uma informação pode estar disponível para uma instituição mesmo quando poucos indivíduos conhecem seus detalhes.

---

# 7. Comunicação

Informação pode viajar através de:

```text
speech
messengers
letters
caravans
trade
radio / communication tech
networks
institutions
archives
books
news
signals
research
```

Cada método possui:

```text
range
latency
bandwidth
reliability
cost
security
reachability
```

---

# 8. Latência da informação

O mundo não possui comunicação instantânea por padrão.

```text
EVENT AT CITY A
↓
message created
↓
transport
↓
message arrives at CITY B
↓
recipient processes it
↓
knowledge updated
```

Tecnologia superior pode reduzir latência.

---

# 9. Segredos

Um segredo é informação com acesso limitado.

```text
Secret
├── subject
├── holders
├── access policy
├── discovery conditions
├── exposure risk
└── consequences
```

Segredos podem ser:

```text
personal
family
faction
military
industrial
scientific
political
archaeological
dimensional
```

---

# 10. Descoberta

Informação secreta pode ser descoberta por:

```text
observation
investigation
research
witness
document recovery
communication interception
deduction
exploration
accidental discovery
```

O jogo deve modelar **caminhos de descoberta**, não simplesmente revelar segredos porque uma quest foi iniciada.

---

# 11. Exemplo: atividade secreta

```text
Player builds hidden facility
        ↓
construction leaves physical traces
        ↓
nearby NPC observes unusual traffic
        ↓
rumor appears
        ↓
local investigator investigates
        ↓
partial evidence found
        ↓
faction receives report
        ↓
response becomes possible
```

Também pode acontecer:

```text
facility remains undiscovered
```

O resultado depende das evidências disponíveis.

---

# 12. Informação não cria conhecimento automaticamente

```text
Evidence exists
≠
NPC knows evidence exists
```

Exemplo:

```text
hidden location
↓
physical evidence exists
↓
no witness
↓
no report
↓
public knowledge = none
```

---

# 13. Confiança em fontes

Cada entidade pode atribuir confiança a fontes.

```text
SourceTrust
├── identity
├── historical reliability
├── faction relation
├── domain expertise
└── recent accuracy
```

Um NPC pode acreditar mais em seu governo do que em um comerciante estrangeiro.

---

# 14. Contradições

Informações contraditórias são permitidas.

```text
Claim A:
"The king started the war."

Claim B:
"The coalition started the war."

Truth:
unknown to this population
```

O sistema não deve forçar uma única opinião.

---

# 15. Rumor propagation

Rumor pode se espalhar como um processo simulado.

```text
SOURCE
↓
TRANSMISSION
↓
RETELLING
↓
MODIFICATION
↓
NEW AUDIENCE
```

Cada transmissão pode alterar:

```text
confidence
accuracy
emotion
framing
importance
```

---

# 16. Propaganda

Instituições podem produzir versões estratégicas dos fatos.

```text
FACT
↓
MESSAGE FRAMING
↓
TARGET AUDIENCE
↓
PROPAGATION
```

O sistema deve preservar a distinção entre:

```text
fact
claim
propaganda
belief
```

---

# 17. Memory

Memória não é perfeita.

Dependendo do modelo e LOD, uma entidade pode ter:

```text
exact memory
approximate memory
semantic memory
emotional association
forgotten event
```

Eventos importantes podem ser preservados por instituições mesmo quando indivíduos morrem.

---

# 18. Informação histórica

O sistema de História fornece fatos e evidências.

O sistema de Conhecimento decide quem os possui.

```text
HISTORY
↓
AVAILABLE RECORD
↓
DISCOVERY / COMMUNICATION
↓
KNOWLEDGE
```

---

# 19. Integração com IA

A IA não deve consultar diretamente a verdade absoluta do mundo para decidir.

Agentes devem raciocinar sobre o subconjunto de informação que possuem.

```text
WORLD STATE
    ↓
OBSERVATION
    ↓
MEMORY
    ↓
KNOWN INFORMATION
    ↓
DECISION SYSTEM
```

Ferramentas especiais podem fornecer conhecimento adicional quando uma regra explícita justificar.

---

# 20. Informação para grupos

Uma facção pode combinar relatórios.

```text
Agent A → report
Agent B → report
Agent C → observation
       ↓
Intelligence synthesis
       ↓
Faction knowledge
```

Isso permite investigações coletivas sem dar conhecimento telepático aos indivíduos.

---

# 21. Desinformação

Informações falsas podem existir como crenças.

```text
FALSE CLAIM
↓
BELIEF
↓
DECISION
↓
CONSEQUENCE
```

O sistema não precisa fazer todas as afirmações serem falsas ou verdadeiras automaticamente. A qualidade depende de evidência, fontes e percepção.

---

# 22. Pesquisa

Research pode melhorar a qualidade do conhecimento:

```text
QUESTION
↓
OBSERVATION
↓
EXPERIMENT
↓
EVIDENCE
↓
HYPOTHESIS
↓
CONFIDENCE UPDATE
```

Tecnologia, instituições e experiência mudam a capacidade de investigar.

---

# 23. Informação como recurso

Informação pode ter valor econômico e político.

```text
accurate map
market information
resource location
military intelligence
scientific discovery
secret technology
```

Instituições podem pagar, proteger, roubar, compartilhar ou restringir informação.

---

# 24. Player agency

O jogador pode produzir informação por suas ações.

```text
exploration
construction
trade
politics
research
public statements
social relationships
```

Mas o jogador não controla automaticamente como o mundo interpreta suas ações.

---

# 25. LOD

Em baixa resolução, o conhecimento deve ser agregado.

```text
FULL
→ individual observations

REGIONAL
→ settlement intelligence

ABSTRACT
→ faction-level knowledge summaries
```

Ao retornar para FULL, detalhes podem ser recuperados quando persistidos.

---

# 26. Persistência

Devem ser persistidos, conforme importância:

```text
secrets
major claims
historical discoveries
institutional knowledge
critical evidence
important memories
```

Memória efêmera pode ser comprimida ou descartada.

---

# 27. Segurança da simulação

Nenhum sistema de informação pode conceder conhecimento arbitrário ao cliente.

A informação autoritativa deve estar no lado confiável da simulação quando relevante para multiplayer.

---

# 28. Testes

```text
information latency
source reliability
rumor propagation
contradiction handling
secret exposure
memory decay
knowledge merge
LOD conversion
save/load
multiplayer authority
AI observability
```

---

# 29. Regra final

> **O mundo sabe o que aconteceu. Cada personagem sabe apenas uma parte. A História preserva os fatos; o Conhecimento distribui informação; a Lore transforma informação em narrativa.**
