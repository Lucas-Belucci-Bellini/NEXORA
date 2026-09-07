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
- **RISK:** Nenhum. **Medido** em
  [`docs/benchmarks/PHASE-0-BASELINE.md`](docs/benchmarks/PHASE-0-BASELINE.md).
- **MEASUREMENT:** a divisão custa **1,7 ns**. O atalho por deslocamento que este
  débito propunha custa **6,7 ns**, e mesmo com largura constante em tempo de
  compilação custa **6,8 ns** — cerca de **4× mais lento** do que aquilo que ele
  deveria melhorar. A explicação provável é que `section_of` é `const fn` e o
  compilador já dobra a divisão numa sequência ótima; a explicação é hipótese, a
  medição não é.
- **PROPOSED REMEDIATION:** nenhuma. Implementar a "otimização" seria uma
  regressão.
- **RE-OPEN TRIGGER:** profiling mostrar `section_of` quente num ponto de chamada
  onde o `ChunkShape` é opaco ao otimizador (atrás de trait object, ou lido de um
  save em runtime) — caso que esta medição não cobre.
- **TARGET STAGE:** —
- **STATUS:** WONT FIX (medido em 2026-09-06)

### DEBT-0006 — Verificação de leitura em toda escrita de save

- **SYSTEM:** `engine/persistence`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** `write_atomic` relê e decodifica o arquivo antes do rename,
  para que "nunca substituir um save válido por dados parciais" seja verdade
  demonstrável e não apenas pretendida (ADR-0004).
- **IMPACT:** Toda escrita custa uma leitura e uma decodificação a mais.
- **RISK:** Baixo hoje; cresce com o tamanho do save.
- **MEASUREMENT:** escrita a **37 MiB/s** contra leitura a **96 MiB/s** — a
  garantia custa cerca de **2,6×** na escrita, ou 4,35 ms para um save de
  152 KiB.
- **PROPOSED REMEDIATION:** Manter. O custo em milissegundos compra "nunca
  substituir um save válido por dados parciais" como propriedade demonstrável, e
  não como intenção. Reavaliar só quando o save for grande o bastante para a
  releitura dominar — e aí com ADR.
- **TARGET STAGE:** Phase 9
- **STATUS:** OPEN (quantificado em 2026-09-06)

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
- **PROGRESS (2026-09-06):** o harness existe (`engine/benchmark`) e a linha de
  base da implementação de referência está registrada em
  [`docs/benchmarks/PHASE-0-BASELINE.md`](docs/benchmarks/PHASE-0-BASELINE.md).
  Cobre aproximadamente um terço do gate: chunk, jobs, save/load, headless,
  memória, tamanho de binário e determinismo. As nove etapas restantes da fatia
  do plano estão **declaradas como não medidas**, com motivo, em vez de omitidas.
- **PROPOSED REMEDIATION:** falta o essencial — **uma segunda stack medida sob o
  mesmo workload**. A regra 5 do plano proíbe decidir por um microcaso único, e
  números absolutos de uma stack só não comparam nada. Próximos incrementos
  mensuráveis, nesta ordem: entity system (não precisa de hardware) e depois RHI
  (impossível de medir em container headless).
- **TARGET STAGE:** antes da Phase 2
- **STATUS:** IN PROGRESS

### DEBT-0009 — Job system custa ~8,8 µs por submissão

- **SYSTEM:** `engine/runtime::jobs`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** descoberto pela medição, não por suspeita.
  [`docs/benchmarks/PHASE-0-BASELINE.md`](docs/benchmarks/PHASE-0-BASELINE.md)
  mede **8,84 µs** para enfileirar um job e **8,85 ms** para 1.000 jobs triviais
  num pool de 4 workers — cerca de **113.000 jobs/s**, quase tudo overhead de
  escalonamento.
- **IMPACT:** irrelevante na granularidade de chunk (0,7% do custo de gerar um
  chunk). Fatal por entidade: 10.000 entidades como jobs individuais custariam
  88 ms só de overhead, antes de qualquer simulação. Na prática o tamanho mínimo
  útil de um job hoje é da ordem de um milissegundo.
- **RISK:** Alto. **Confirmado com entidades reais** (2026-09-06): na mesma
  execução, simular 1.000 entidades custa **2,95 µs** e submeter 1.000 jobs custa
  **14,48 ms** — razão de **~4.900×**. A estimativa anterior ("fatal por
  entidade") era otimista.
- **PROPOSED REMEDIATION:** causa provável é um futex wake por `notify_one` a
  cada submissão, somado à contenção de todos os workers num único mutex.
  Candidatos: API de submissão em lote, ou filas por worker com work-stealing
  para que o produtor não acorde o pool a cada job. Medir de novo depois.
- **NOTA DE ARQUITETURA:** o número não condena o job system, define a
  granularidade correta dele. Um chunk a ~2,5 ms é exatamente o tipo de trabalho
  que `NEXORA THREADING AND CONCURRENCY MODEL.md` manda dividir. **Um job por
  entidade é anti-padrão, agora com número.**
- **TARGET STAGE:** antes da Phase 5 (Entity + AI Foundation)
- **STATUS:** OPEN (medido)

### DEBT-0010 — Consultas de entidade são varredura linear, sem índice espacial

- **SYSTEM:** `engine/entity::query`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** `Entity System.md` §34 (ENTITY-33) especifica um índice
  espacial. A ADR-0006 adiou deliberadamente: construir índice antes de medir a
  varredura repetiria o erro que o DEBT-0005 pegou.
- **IMPACT:** medido em **~19 ns por entidade examinada**. Extrapolando:

  | população | uma varredura |
  | ---: | ---: |
  | 1.000 | ~19 µs |
  | 10.000 | ~190 µs |
  | 100.000 | ~1,9 ms |

- **RISK:** nenhum hoje; alto acima de ~10.000 entidades, e proibitivo se IA
  consultar por entidade a cada tick.
- **PROPOSED REMEDIATION:** índice espacial por chunk ou grade frouxa, alimentado
  pelas mudanças de transform. Medir de novo contra a varredura antes de manter.
- **TRIGGER:** população passar de 10.000, ou um perfil mostrar consulta em
  caminho quente.
- **TARGET STAGE:** Phase 5 (Entity + AI Foundation)
- **STATUS:** OPEN

### DEBT-0011 — Lookup de voxel domina o passo de física, sem cache de chunk

- **SYSTEM:** `engine/simulation::terrain` (`WorldVoxels`)
- **CLASS:** PERFORMANCE
- **WHY CREATED:** o adaptador responde cada célula com uma busca no `BTreeMap`
  de colunas seguida de leitura da seção paletizada. Não há estado entre
  consultas — e consultas consecutivas do mesmo corpo caem quase sempre no
  mesmo chunk.
- **IMPACT:** medido, com o par de medições que existe para isso
  (`docs/benchmarks/PHASE-0-BASELINE.md` Apêndice B, achado 10):

  | 1.000 corpos, um substep | mediana | por corpo |
  | --- | ---: | ---: |
  | contra terreno gerado | 293,82 µs | 294 ns |
  | contra piso plano | 149,27 µs | 149 ns |
  | diferença — o lookup | **144,55 µs** | **145 ns** |

  **49% do passo é perguntar ao mundo o que existe ali**, não resolver colisão.
- **RISK:** médio. Otimizar o solver hoje endereçaria a metade menor; o número
  existe justamente para impedir esse erro.
- **PROPOSED REMEDIATION:** manter a coluna residente entre os sweeps de um
  mesmo corpo (a caixa quase nunca cruza chunk), e medir de novo contra o par
  terreno/plano antes de manter.
- **TRIGGER:** física passar de ~10% do orçamento de simulação, ou população
  acordada estável acima de 1.000.
- **TARGET STAGE:** Phase 4 (World Runtime) ou antes, se o gatilho ocorrer
- **STATUS:** OPEN (medido)

### DEBT-0012 — Depenetração custa 42% de um sweep no caso em que nada aconteceu

- **SYSTEM:** `engine/physics::collision::depenetrate`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** `resolve` verifica, a cada passo e para cada corpo, se ele já
  começou dentro do terreno. A verificação varre todo o volume de células da
  caixa, e o caso que ela trata — bloco colocado onde o corpo está, chunk
  gerando em volta — é raro.
- **IMPACT:** **166,2 ns** contra **394,1 ns** de um sweep completo de três
  eixos, ou seja **~42% de um sweep** pago por todo corpo, todo passo, por um
  caminho que quase nunca dispara.
- **RISK:** baixo hoje; cresce linearmente com corpos acordados.
- **PROPOSED REMEDIATION:** o sweep já visita essas células. Derivar a
  sobreposição do próprio sweep em vez de uma varredura separada, ou marcar o
  corpo como suspeito apenas quando o mundo muda perto dele (o `wake_in` já
  identifica a região).
- **NOTA:** remover a passagem **não** é opção. O defeito que a motivou foi
  encontrado por teste: sem ela, um corpo que acaba dentro do terreno afunda
  para sempre.
- **TRIGGER:** junto com DEBT-0011, ou quando física aparecer em perfil.
- **TARGET STAGE:** Phase 4
- **STATUS:** OPEN (medido)

### DEBT-0013 — Física não publicou orçamento, embora agora tenha os números

- **SYSTEM:** `engine/physics`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `NEXORA PERFORMANCE BUDGETS.md` exige que todo sistema maior
  publique `TARGET`/`WARNING`/`CRITICAL`/`EMERGENCY`, e diz que um sistema não é
  *production-ready* enquanto o orçamento não estiver documentado. Física não
  publicou nenhum.
- **IMPACT:** sem orçamento não há como o CI ou o `/diagnostico` dizer que a
  física passou do aceitável; só dá para dizer que ficou mais lenta.
- **RISK:** médio. Regressão de desempenho passa despercebida até virar sintoma
  de jogo.
- **PROPOSED REMEDIATION:** publicar os quatro limiares a partir do Apêndice B —
  1.000 corpos acordados custam ~17,6 ms de CPU por segundo de relógio a 60 Hz,
  ~1,8% de um núcleo — **depois** de medir em mais de uma máquina. Fixar limiar
  a partir de uma execução de um container compartilhado seria precisão não
  merecida.
- **TRIGGER:** segunda máquina medida, ou entrada na Phase 4.
- **TARGET STAGE:** Phase 4
- **STATUS:** OPEN

### DEBT-0014 — Corpos não colidem com corpos

- **SYSTEM:** `engine/physics::world`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** todo contato desta fase é corpo-contra-voxel. `PHYSICS.md`
  §5–§7 (PHY-4, PHY-5, PHY-6) especificam broadphase, narrowphase e solver de
  pares; o *primeiro vertical slice* do próprio documento — cair, andar, pular,
  aterrissar — não precisa deles, e a ADR-0007 preferiu medir o que existe a
  construir o que ainda não tem evidência.
- **IMPACT:** não funcionam: plataforma móvel carregando passageiro (PHY-25),
  veículos (PHY-27), blocos que caem e empilham (PHY-18), gatilhos por
  sobreposição (PHY-24). Dois corpos dinâmicos se atravessam.
- **RISK:** alto **se descoberto por acidente**. Por isso está fixado por teste:
  `engine/physics/tests/phy_42_checklist.rs` tem casos `not_yet_*` que falham no
  dia em que o par solver existir, forçando a lista a ser atualizada em vez de
  ficar mentindo.
- **PROPOSED REMEDIATION:** broadphase por grade espacial frouxa sobre corpos
  acordados, narrowphase AABB-AABB, solver de impulso com atrito e restituição
  usando a mesma combinação de materiais já implementada.
- **TRIGGER:** primeiro sistema que precise — veículos, blocos que caem, ou
  plataforma móvel.
- **TARGET STAGE:** Phase 6 (Vehicles) ou antes, conforme o gatilho
- **STATUS:** OPEN

### DEBT-0015 — Corpos não têm orientação

- **SYSTEM:** `engine/physics::body`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** não há rotação, velocidade angular nem tensor de inércia.
  Toda caixa é alinhada aos eixos e todo voxel é um cubo alinhado aos eixos, de
  modo que uma rotação não teria em que agir. Carregar os campos sem código que
  os leia seria mock permanente (regra §43 do briefing).
- **IMPACT:** uma caixa que cai fica sempre alinhada; nada tomba, gira ou rola.
- **RISK:** baixo enquanto colisão for caixa-contra-cubo.
- **PROPOSED REMEDIATION:** chega junto com formas não-cúbicas e com o solver de
  pares — antes disso não há geometria capaz de aplicar torque.
- **TRIGGER:** DEBT-0014 ou DEBT-0016, o que vier primeiro.
- **TARGET STAGE:** Phase 6
- **STATUS:** OPEN

### DEBT-0016 — Só existem cubos inteiros, e isso força `step_height = 1.0`

- **SYSTEM:** `engine/physics::voxel::VoxelShape`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `PHYSICS.md` §13 (PHY-12) pede slab, rampa, cunha e escada.
  `VoxelShape` é `#[non_exhaustive]` exatamente porque essas são variantes dele,
  não um redesenho — mas nenhuma existe ainda.
- **IMPACT:** o menor degrau possível é um metro, então uma altura de passo
  sub-bloco (o 0,6 convencional, que pressupõe meia-laje) nunca conseguiria
  subir em nada. O preset de personagem usa `1.0` por isso, e está documentado
  no próprio preset. Também não há rampa para testar `slopeLimit`, que por
  consequência não existe.
- **RISK:** baixo hoje; a decisão do `step_height` volta a ficar em aberto no dia
  em que slabs chegarem.
- **PROPOSED REMEDIATION:** adicionar as variantes com as caixas parciais que
  cada uma descreve, e então reavaliar `step_height` e introduzir limite de
  inclinação.
- **TRIGGER:** primeiro bloco de meia altura no conteúdo.
- **TARGET STAGE:** Phase 4
- **STATUS:** OPEN
