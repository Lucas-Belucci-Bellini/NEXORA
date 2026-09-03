# NEXORA — LORE SYSTEM

> **Princípio central:**
> **Lore é a forma como um mundo interpreta, preserva, transmite e reconta sua própria história. Ela nasce dos fatos, do conhecimento e da cultura, mas não é obrigada a ser objetivamente correta.**

O Lore System fica acima de History e Knowledge.

```text
SIMULATION
   ↓
HISTORY
   ↓
KNOWLEDGE
   ↓
LORE
```

---

# 1. Objetivo

Transformar acontecimentos e conhecimento em conteúdo cultural observável:

```text
books
chronicles
archives
monuments
traditions
myths
biographies
political narratives
local stories
quest context
dialogue topics
museum entries
education material
```

A Lore não deve ser apenas texto escrito manualmente pelos desenvolvedores.

Parte importante dela deve ser produzida a partir do estado histórico de cada mundo.

---

# 2. Três camadas

```text
CANON
→ fatos e regras fixados pelo design do universo

WORLD HISTORY
→ fatos produzidos por uma instância específica do mundo

IN-WORLD LORE
→ narrativas produzidas e preservadas pelos habitantes
```

Exemplo:

```text
Canon:
Existem 16 dimensões.

World History:
A Civilização A abriu uma passagem para a Dimensão 7 no ano 4812.

Lore:
"Os ancestrais abriram um caminho proibido para além do céu."
```

---

# 3. Lore não altera a verdade histórica

Regra de integridade:

```text
LORE
≠
HISTORY
```

Uma civilização pode acreditar que um governante venceu uma guerra sozinho mesmo quando os registros históricos mostram uma coalizão inteira.

---

# 4. Tipos de narrativa

```text
CHRONICLE
HISTORICAL ACCOUNT
BIOGRAPHY
AUTOBIOGRAPHY
MYTH
LEGEND
FOLKTALE
PROPAGANDA
MEMORIAL
RELIGIOUS/CULTURAL TRADITION
NEWS REPORT
ACADEMIC THEORY
PERSONAL STORY
```

Cada tipo possui regras diferentes de confiabilidade e linguagem.

---

# 5. Lore Item

```text
LoreItem
├── id
├── type
├── subject
├── sourceClaims
├── author
├── culture
├── faction
├── creationTime
├── lastRevision
├── audience
├── confidence
├── popularity
├── politicalAlignment
├── evidenceRefs
└── provenance
```

---

# 6. Geração procedural

A geração de Lore segue:

```text
HISTORICAL EVENT
       ↓
KNOWN FACTS
       ↓
LOCAL CONTEXT
       ↓
CULTURAL FILTER
       ↓
AUTHOR / INSTITUTION
       ↓
NARRATIVE FORM
       ↓
LORE ITEM
```

O mesmo evento pode produzir materiais diferentes em culturas diferentes.

---

# 7. Cultura altera narrativa

Exemplo:

```text
FACT:
Empire lost a war.
```

A cultura derrotada pode registrar:

```text
"The Great Betrayal"
```

A cultura vencedora:

```text
"The Liberation"
```

Uma universidade:

```text
"The Border War of 481"
```

Todos apontam para o mesmo conjunto de fatos históricos.

---

# 8. Personagens históricos

O sistema deve produzir biografias derivadas da vida real simulada de indivíduos relevantes.

```text
NPC
↓
life events
↓
relationships
↓
career
↓
political actions
↓
major decisions
↓
death / legacy
↓
BIOGRAPHY
```

Uma biografia pode possuir erros ou perspectivas diferentes.

---

# 9. Legado

Após a morte ou desaparecimento de um personagem importante:

```text
legacy
├── achievements
├── failures
├── descendants
├── institutions
├── monuments
├── controversies
└── myths
```

A importância de um personagem pode crescer depois da morte.

---

# 10. Mito e lenda

Mitos surgem quando:

```text
historical fact
+
retelling
+
uncertainty
+
cultural symbolism
```

Eles podem divergir muito da verdade sem alterar o registro histórico.

---

# 11. Propaganda

Instituições podem produzir narrativas para objetivos políticos.

```text
historical event
↓
selected facts
↓
framing
↓
public narrative
```

O sistema deve preservar o vínculo com a fonte histórica sem tratar propaganda como verdade.

---

# 12. Informação perdida

Nem todo fato histórico permanece conhecido.

```text
FACT
↓
records destroyed
↓
knowledge loss
↓
legend survives
```

Séculos depois:

```text
historical truth = available internally
public knowledge = incomplete
lore = mythologized
```

---

# 13. Descoberta posterior

Uma nova descoberta pode alterar a interpretação sem alterar o fato original.

```text
new archive discovered
↓
new evidence
↓
knowledge update
↓
old interpretation challenged
↓
new lore generated
```

Isso permite revisões históricas naturais.

---

# 14. Books

Livros são instâncias geradas pelo mundo.

```text
Book
├── title
├── author
├── publicationDate
├── region
├── language
├── claims
├── citedEvidence
├── culturalContext
└── circulation
```

Um livro pode continuar circulando depois da morte do autor.

---

# 15. Jornais e comunicação pública

Sociedades avançadas podem possuir mídia.

```text
EVENT
↓
REPORTER / INSTITUTION
↓
EDITORIAL PROCESS
↓
NEWS
↓
PUBLIC KNOWLEDGE
```

Notícias podem ser incompletas ou enviesadas.

---

# 16. Monumentos

Sociedades podem codificar memória em estruturas:

```text
statue
monument
memorial
ruin
museum
memorial site
```

Esses elementos também podem ser gerados como resultado de eventos históricos.

---

# 17. Quest integration

Quests podem consultar Lore:

```text
Quest
↓
Historical context
↓
Known claims
↓
Evidence needed
↓
Player investigation
```

A quest não precisa inventar a história. Ela pode expor partes da história gerada pela simulação.

---

# 18. Dialogue integration

Diálogos devem usar:

```text
NPC knowledge
NPC beliefs
NPC culture
NPC faction
NPC memory
historical context
```

Dois NPCs podem responder de formas diferentes à mesma pergunta.

---

# 19. Lore discovery

O jogador pode descobrir lore por:

```text
conversation
books
archives
ruins
monuments
maps
research
quests
exploration
observation
social networks
```

O sistema não deve simplesmente desbloquear toda a lore ao entrar em uma região.

---

# 20. Procedural narrative generation

Textos gerados devem usar dados estruturados.

```text
FACTS
+
TEMPLATES / STYLE RULES
+
CULTURAL CONTEXT
+
AUTHOR PROFILE
+
LANGUAGE RULES
=
LORE TEXT
```

Uma futura camada generativa pode auxiliar na redação, mas não deve ter autoridade para criar fatos não presentes nos dados do mundo.

---

# 21. Author profiles

Cada autor ou instituição pode possuir:

```text
writing style
education
bias
political alignment
culture
knowledge level
preferred vocabulary
```

Isso torna as narrativas diferentes sem exigir texto manual para cada pessoa.

---

# 22. Language and localization

A Lore deve existir em uma representação semântica antes da localização textual.

```text
HISTORICAL MEANING
↓
NARRATIVE STRUCTURE
↓
LANGUAGE
↓
LOCALIZATION
```

Não devemos armazenar apenas strings como fonte de verdade.

---

# 23. Dynamic canon inside a save

Cada mundo terá uma espécie de "canon local":

```text
World Canon
→ major confirmed historical events
→ discovered technologies
→ current borders
→ notable leaders
→ known historical eras
```

Esse canon pertence apenas àquela instância do mundo.

---

# 24. Player-created lore

O jogador também pode criar cultura:

```text
build monument
write records
found institution
publish book
name settlement
create tradition
```

O sistema deve registrar autoria e contexto.

---

# 25. Lore permanence

Nem toda narrativa sobrevive.

```text
creation
↓
circulation
↓
adoption
↓
decline
↓
archival survival
```

Uma lenda pode desaparecer e depois ser redescoberta.

---

# 26. Tests

```text
fact/lore separation
procedural consistency
source attribution
contradictory narratives
cultural variation
knowledge dependency
history migration
save/load
LOD behavior
text provenance
```

---

# 27. Regra final

> **A História registra o que aconteceu. O Conhecimento determina quem sabe. A Lore determina como o mundo conta aquilo que sabe.**
