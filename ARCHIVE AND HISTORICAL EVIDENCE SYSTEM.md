# NEXORA — ARCHIVE AND HISTORICAL EVIDENCE SYSTEM

> **Princípio central:**
> **Uma história confiável precisa de rastros. Arquivos, objetos, monumentos, registros e testemunhos formam a camada de evidência que permite ao mundo lembrar, esquecer, investigar e reinterpretar o passado.**

---

# 1. Objetivo

O Archive System preserva evidências relacionadas à História do mundo.

```text
EVENT
↓
EVIDENCE
↓
ARCHIVE
↓
DISCOVERY
↓
KNOWLEDGE
↓
LORE
```

Ele não é a verdade histórica em si. Ele armazena os rastros que podem comprovar, questionar ou iluminar acontecimentos.

---

# 2. Tipos de evidência

```text
DOCUMENT
LETTER
TREATY
MAP
LEDGER
DATABASE RECORD
INSCRIPTION
MONUMENT
RELIC
ARCHAEOLOGICAL SITE
PHYSICAL REMNANT
PHOTOGRAPH / CAPTURE
TESTIMONY
WITNESS MEMORY
SCIENTIFIC MEASUREMENT
INSTITUTIONAL RECORD
PLAYER RECORD
```

---

# 3. Evidence Item

```text
EvidenceItem
├── id
├── type
├── creationTime
├── origin
├── creator
├── owner
├── location
├── preservation
├── integrity
├── authenticity
├── accessibility
├── linkedEvents
├── claims
└── provenance
```

---

# 4. Integridade

Uma evidência pode:

```text
intact
partial
damaged
corrupted
altered
forged
lost
unknown
```

A História pode saber que um documento existiu mesmo quando o documento físico foi destruído.

---

# 5. Autenticidade

```text
verified
likely authentic
uncertain
suspected forgery
confirmed forgery
```

A autenticidade pode ser revisada conforme novos métodos de investigação aparecem.

---

# 6. Arquivos como instituições

Uma civilização pode possuir:

```text
archive
library
museum
registry
court records
scientific institution
military archive
private collection
```

Instituições diferentes podem preservar conjuntos diferentes de evidências.

---

# 7. Destruição e perda

Arquivos podem desaparecer por:

```text
decay
fire
flood
war
abandonment
political suppression
accident
migration
```

Isso afeta o que o futuro consegue conhecer, sem alterar o fato histórico original.

---

# 8. Descoberta arqueológica

Ruínas podem ligar o presente ao passado:

```text
EXPLORATION
↓
SITE DISCOVERY
↓
EVIDENCE RECOVERY
↓
ANALYSIS
↓
HISTORICAL CLAIM
↓
KNOWLEDGE UPDATE
```

---

# 9. Evidência física no mundo

Arquivos não precisam ser apenas UI.

Exemplos:

```text
ruined city
old railway
abandoned factory
graveyard
fortification
monument
artifact cache
damaged infrastructure
```

O mundo físico pode ser parte do registro histórico.

---

# 10. Player investigation

O jogador pode:

```text
find
inspect
compare
catalog
translate
restore
research
publish
```

A descoberta de uma evidência pode transformar uma hipótese em conhecimento confiável.

---

# 11. Conflicting evidence

Evidências podem entrar em conflito.

```text
Document A → claim X
Document B → claim Y
Witness C → remembers Z
```

O sistema deve preservar todas as fontes e permitir avaliação de confiança.

---

# 12. Provenance

Toda evidência importante precisa de rastreabilidade:

```text
created by
modified by
found by
stored by
copied from
translated by
```

Isso também auxilia auditoria e debugging.

---

# 13. Relação com Lore

```text
Evidence
↓
History interpretation
↓
Knowledge
↓
Lore
```

Um livro pode citar um documento. Um monumento pode representar uma versão cultural. Uma universidade pode rejeitar uma interpretação antiga após nova evidência.

---

# 14. Persistência

Evidências relevantes devem sobreviver a:

```text
save/load
server restart
chunk unload
region unload
LOD changes
world migration
```

---

# 15. Escala

```text
PERSONAL ARCHIVE
SETTLEMENT ARCHIVE
CIVILIZATION ARCHIVE
WORLD ARCHIVE
INTERDIMENSIONAL ARCHIVE
```

---

# 16. LOD

Quando regiões estão abstraídas, o sistema mantém resumos dos arquivos relevantes.

```text
FULL
→ physical evidence + detailed records

REGIONAL
→ institution-level archives

ABSTRACT
→ historical summary + key evidence references
```

---

# 17. Segurança

Arquivos privados ou secretos podem possuir controle de acesso.

```text
PUBLIC
RESTRICTED
PRIVATE
SECRET
CLASSIFIED
LOST
```

O conhecimento de sua existência também pode ser limitado.

---

# 18. Testes

```text
authenticity
integrity
provenance
archive loss
archive recovery
conflicting evidence
save/load
LOD
permissions
migration
```

---

# 19. Regra final

> **O passado deixa rastros. A História registra acontecimentos; o Arquivo preserva evidências; o Conhecimento depende daquilo que foi observado, transmitido ou descoberto.**
