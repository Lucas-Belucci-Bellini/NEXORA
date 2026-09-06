# NEXORA — TECHNICAL DEBT REGISTER

## Objetivo

Registrar compromissos técnicos conscientes para impedir que atalhos temporários se tornem arquitetura permanente.

## Registro mínimo

```text
ID
TITLE
SYSTEM
WHY CREATED
IMPACT
RISK
OWNER
PROPOSED REMEDIATION
TARGET STAGE
STATUS
```

## Classes

```text
TEMPORARY
PERFORMANCE
ARCHITECTURAL
COMPATIBILITY
TOOLING
TEST
SECURITY
CONTENT
```

## Regras

1. Todo workaround importante deve possuir ID.
2. “Temporário” sem plano de remoção é debt permanente até prova em contrário.
3. Débito que ameaça save, networking, security ou public APIs recebe prioridade alta.
4. Fechar debt exige teste que prove a correção.

## Status

```text
OPEN
PLANNED
IN PROGRESS
BLOCKED
RESOLVED
WONT FIX
```

## Registro aberto

Débito real registrado durante a Phase 0. Cada item existe porque um atalho
consciente foi tomado, ou porque metade de um contrato foi implementada.

### DEBT-0001 — Save sem journal incremental

- **SYSTEM:** `engine/persistence`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `NEXORA SAVE FORMAT AND COMPATIBILITY.md` define
  `Snapshot + Journal → Recovery`. A Phase 0 implementou o snapshot completo
  (atômico, versionado, com checksum e quarentena); o journal incremental não.
- **IMPACT:** Recuperação depende do último snapshot íntegro. Mudanças entre
  checkpoints são perdidas se o processo cair.
- **RISK:** Médio. Cresce com o tamanho do mundo e com o intervalo de autosave.
- **PROPOSED REMEDIATION:** Journal append-only por região, com replay no load.
- **TARGET STAGE:** Phase 3 (Persistence + Simulation)
- **STATUS:** OPEN

### DEBT-0002 — Mundo inteiro em uma seção de save

- **SYSTEM:** `engine/world::persist`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** Todos os chunks residentes são escritos numa única seção
  `nexora:save/chunks`. `CHUNK & VOXEL ENGINE.md` §27 especifica region files.
- **IMPACT:** Salvar reescreve o mundo inteiro, mesmo com um bloco sujo. O
  rastreamento de sujeira por seção já existe e ainda não é aproveitado.
- **RISK:** Alto a partir de mundos grandes; irrelevante na escala da Phase 0.
- **PROPOSED REMEDIATION:** Region files com escrita apenas dos chunks sujos.
- **TARGET STAGE:** Phase 2 (Voxel Vertical Slice)
- **STATUS:** OPEN

### DEBT-0003 — Sem compressão de chunk

- **SYSTEM:** `engine/world::persist`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** `CHUNK & VOXEL ENGINE.md` §24 prevê compressão; a Phase 0
  grava as palavras empacotadas cruas para manter o formato inspecionável
  enquanto ele ainda está sendo estabilizado.
- **IMPACT:** Saves maiores que o necessário.
- **RISK:** Baixo. A compressão de paleta já elimina a maior parte do custo.
- **PROPOSED REMEDIATION:** Compressão por seção, com o algoritmo registrado no
  cabeçalho para permitir troca versionada.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN

### DEBT-0004 — Journal de chunk descarta o mais antigo em silêncio parcial

- **SYSTEM:** `engine/world::chunk`
- **CLASS:** TEMPORARY
- **WHY CREATED:** `NEXORA PERFORMANCE BUDGETS.md` proíbe crescimento ilimitado,
  então o journal tem teto. Ao estourar, a entrada mais antiga cai.
- **IMPACT:** O journal deixa de ser registro completo. O contador
  `dropped_journal_entries` expõe o fato, mas nada consome esse sinal ainda.
- **RISK:** Médio quando History e replicação passarem a depender do journal.
- **PROPOSED REMEDIATION:** Drenar o journal a cada tick para o History System
  em vez de acumular no chunk.
- **TARGET STAGE:** Phase 11 (History), ou antes se a replicação chegar primeiro
- **STATUS:** OPEN

### DEBT-0005 — Conversão de coordenadas usa divisão por valor de runtime

- **SYSTEM:** `engine/foundation::spatial`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** `ChunkShape` é um valor de runtime, exigido por
  `CHUNK & VOXEL ENGINE.md` §3 ("não deixar o código inteiro assumir que o chunk
  sempre possui o mesmo tamanho"). Isso impede substituir divisão por shift.
- **IMPACT:** Cada conversão bloco→seção é uma divisão inteira real.
- **RISK:** Desconhecido — **não medido**. A Phase 0 não trocou correção por uma
  otimização sem evidência.
- **PROPOSED REMEDIATION:** Medir primeiro. Se aparecer no perfil, guardar
  máscara e deslocamento pré-calculados em `ChunkShape` para extensões potência
  de dois, mantendo o caminho geral.
- **TARGET STAGE:** Phase 9 (Scale / Hardening)
- **STATUS:** OPEN

### DEBT-0006 — Verificação de leitura em toda escrita de save

- **SYSTEM:** `engine/persistence`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** `write_atomic` relê e decodifica o arquivo antes do rename,
  para que "nunca substituir um save válido por dados parciais" seja verdade
  demonstrável e não apenas pretendida (ADR-0004).
- **IMPACT:** Toda escrita custa uma leitura e uma decodificação a mais.
- **RISK:** Baixo hoje; cresce com o tamanho do save.
- **PROPOSED REMEDIATION:** Manter por padrão. Tornar configurável só com
  medição e ADR, porque desligar remove uma garantia de integridade.
- **TARGET STAGE:** Phase 9
- **STATUS:** OPEN

### DEBT-0007 — Event Bus só entrega fatos

- **SYSTEM:** `engine/runtime::events`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `Event Bus.md` §44–§45 define request events e resultados.
  A Phase 0 implementou apenas fatos, conforme ADR-0005.
- **IMPACT:** Sistemas que precisarem de pergunta-e-resposta ainda não têm
  caminho, e podem ser tentados a usar fatos como comandos — o que
  `NEXORA ARCHITECTURE RULES.md` §4 proíbe.
- **RISK:** Médio: o desvio é fácil e difícil de reverter depois.
- **PROPOSED REMEDIATION:** Implementar Command System com o contrato próprio de
  `Command System.md`, em vez de estender o Event Bus.
- **TARGET STAGE:** Phase 1/2
- **STATUS:** OPEN

### DEBT-0008 — Benchmark do gate de linguagem ainda não executado

- **SYSTEM:** engine (transversal)
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** ADR-0001 mantém o gate aberto. A fatia de referência existe;
  a medição comparativa exigida por
  `ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17 não foi feita.
- **IMPACT:** O stack final continua sem lock. Quanto mais código nascer antes
  da medição, mais caro fica trocar.
- **RISK:** Alto — este é o débito mais caro da lista.
- **PROPOSED REMEDIATION:** Instrumentar a fatia headless com as métricas do §17
  e executar o plano de `NEXORA TECHNOLOGY BENCHMARK PLAN.md`.
- **TARGET STAGE:** antes da Phase 2
- **STATUS:** OPEN
