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
- **RESOLUTION (2026-09-07):** feito. `engine/persistence::journal` é um log
  append-only com cada registro emoldurado e checksumado por si, e
  `engine/world::recovery` decide o que um registro significa. A distinção que
  faz o mecanismo valer: **escrita interrompida trunca, ela não reescreve** —
  então cauda curta é *crash* (esperado, recupera o prefixo) e registro completo
  com checksum errado é *corrupção* (barulhento, quarentena). O journal nomeia o
  snapshot de que parte e recusa qualquer outro, que é a armadilha silenciosa:
  os registros aplicariam limpos e produziriam um mundo que nunca existiu. Os
  testes obrigatórios do `NEXORA SAVE FORMAT AND COMPATIBILITY.md` — "crash
  durante save", "corrupção parcial", "recuperação de journal" — vivem em
  `engine/world/tests/crash_recovery.rs`. Ver
  [ADR-0011](docs/adr/ADR-0011-a-torn-tail-is-a-crash-and-corruption-is-not.md).
- **TARGET STAGE:** Phase 3 (Persistence + Simulation)
- **STATUS:** CLOSED (2026-09-07)

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
- **RESOLUTION (2026-09-07):** feito, com o contrato próprio. `engine/command`
  implementa CMD-0 a CMD-4 — identidade, definições, instâncias, ciclo de vida,
  validação em camadas, registry, dispatch, fila e resultados estruturados — e
  depende **só** de `nexora-foundation` e `nexora-runtime`. Com isso a lista do
  §134 ("não deve conter regras de bloco, física, worldgen, inventário…") vira
  erro de compilação em vez de comentário: as crates não são alcançáveis. Os
  handlers de bloco moram em `nexora-simulation`, exatamente a divisão do §28
  (o handler adapta, o sistema decide). Ver
  [ADR-0010](docs/adr/ADR-0010-commands-are-intent-and-carry-their-own-authority.md).
- **TARGET STAGE:** Phase 1/2
- **STATUS:** CLOSED (2026-09-07)

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
- **PROGRESS (2026-09-07):** a segunda stack existe e foi medida.
  `benchmarks/cpp/` reimplementa em C++20 os kernels quentes do motor; os doze
  digests de conformidade batem bit a bit entre Rust, g++ 13.3 e clang++ 18.1, e
  `scripts/compare-stacks.sh` **se recusa a cronometrar** enquanto um digest
  divergir. Resultado em
  [Apêndice D](docs/benchmarks/PHASE-0-BASELINE.md) (achados 19 e 20):
  em nove kernels compartilhados o Rust cai **dentro da faixa entre os dois
  builds C++** em cinco, ganha dos dois em dois e perde para o C++ mais próximo
  em dois (13% e 11%). **O backend do compilador pesa mais que a linguagem** — o
  CRC-32 mede 668,83 µs no g++ e 353,11 µs no clang++, contra 363,97 µs no Rust:
  medido só contra o g++, o "Rust é 1,8× mais rápido que C++" era GCC × LLVM.
  O custo de FFI, que a plataforma listava como métrica e era impossível de
  medir com uma linguagem só, ficou em **~1,2 ns por travessia** no piso da ABI
  C — o mesmo que qualquer chamada não-inlinada, e invisível a 4 KiB por
  travessia. É um **piso**, não uma estimativa: não inclui marshalling,
  conversão de string, cópia por ownership, panic boundary nem validação de
  ponteiro.
- **PROPOSED REMEDIATION:** o que falta agora é diferente do que faltava antes.
  Não é mais "não existe segunda stack": é (a) as etapas de GPU — RHI, janela,
  câmera, mesh — impossíveis num container headless, e (b) uma comparação em
  **escala de motor**, que o ADR-0009 declara explicitamente fora do escopo do
  reference de kernels. Enquanto (a) não for medível aqui, o gate não fecha, e
  `NEXORA LANGUAGE AND FFI BOUNDARY.md` reserva o lock do mapa de linguagens
  para o benchmark completo.
- **TARGET STAGE:** antes da Phase 2
- **STATUS:** IN PROGRESS — desbloqueado do lado da segunda linguagem; preso
  no hardware

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

### DEBT-0017 — O tick ocioso de streaming cresce mais rápido que a área de interesse

- **SYSTEM:** `engine/streaming::system` (`desired_tiers`)
- **CLASS:** PERFORMANCE
- **WHY CREATED:** cada tick enumera todas as colunas dentro do raio externo de
  cada fonte de interesse, monta um `BTreeSet` de candidatos e consulta o
  `BTreeMap` de rastreados uma vez por candidato — mesmo quando nada se moveu.
- **IMPACT:** medido (Apêndice C, achado 16):

  | raio | colunas | mediana | por coluna |
  | ---: | ---: | ---: | ---: |
  | 3 | 49 | 4,70 µs | 96 ns |
  | 12 | 625 | **167,47 µs** | **268 ns** |

  12,8× as colunas custam **35,6×** o tempo: o termo de área é multiplicado por
  um logarítmico. A 167 µs por tick, com nada acontecendo, é ~1% de um quadro a
  60 Hz gasto para concluir que nada mudou. Extrapolando, raio 24 ≈ 1 ms.
- **RISK:** baixo em raios pequenos; alto assim que a distância de visão crescer.
- **PROPOSED REMEDIATION:** o conjunto de candidatos só muda quando um
  observador cruza a fronteira de uma coluna. Manter o conjunto e atualizá-lo
  por diferença (anel que entra, anel que sai) em vez de reconstruí-lo. Medir de
  novo contra o par r3/r12 antes de manter.
- **TRIGGER:** raio de interesse passar de 12, ou streaming aparecer em perfil.
- **TARGET STAGE:** Phase 2 (Voxel Vertical Slice) ou Phase 4
- **STATUS:** OPEN (medido)

### DEBT-0018 — Streaming gera chunks na thread do tick, não no job system

- **SYSTEM:** `engine/simulation::residency` (`WorldResidency::activate`)
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `activate` chama `World::load_or_generate` de forma síncrona.
  `NEXORA THREADING AND CONCURRENCY MODEL.md` manda dividir trabalho pesado em
  jobs independentes, e o próprio slice **já** gera chunks no pool de workers em
  um estágio anterior — o caminho de streaming não.
- **IMPACT:** medido (Apêndice C, achado 18): gerar um chunk custa **2,72 ms**,
  contra **4,70 µs** de um tick ocioso — **578×**. Com o orçamento de 8
  ativações por tick que o próprio slice usa, um tick que gaste o orçamento
  inteiro custa `8 × 2,72 ms ≈ 21,8 ms`, mais que um quadro a 60 Hz, na thread
  do tick.
- **RISK:** alto assim que houver renderização: é um travamento visível a cada
  travessia de fronteira de região.
- **PROPOSED REMEDIATION:** submeter a geração como job e concluir a ativação em
  um tick posterior. A máquina de adiamento (`deferred` no relatório) já existe
  exatamente para descrever isso.
- **TRIGGER:** existir um loop de quadro. *(Correção 2026-09-07: o gatilho dizia
  também "ou o orçamento de ativação passar de 2" — mas o slice já usava 8
  quando isto foi escrito, ou seja, o gatilho nasceu satisfeito. Um gatilho que
  já é verdade no dia em que se escreve não é gatilho, é tarefa pendente
  disfarçada. A parte mensurável fica: com orçamento 8, um tick que o gaste
  inteiro custa 21,8 ms na thread do tick, e isso é intolerável assim que
  houver um quadro para perder.)*
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN (medido)

### DEBT-0019 — `Regional` e `Abstract` são estados reais sem dados próprios

- **SYSTEM:** `engine/streaming::lod`, `engine/simulation::residency`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `STREAMING SYSTEM.md` define a escada
  `FULL → REGIONAL → ABSTRACT → UNRESIDENT`. O gerenciador rastreia os quatro
  níveis e preserva identidade em todos, mas nenhum backend guarda dados
  distintos para os dois do meio: a simulação regional que os preencheria é a
  Phase 4.
- **IMPACT:** hoje um alvo em `Regional` é indistinguível de um em `Abstract`
  para o mundo. Isso está documentado no próprio `Lod::is_resident`, não
  escondido.
- **RISK:** baixo. O mecanismo é real e testado; falta o conteúdo.
- **PROPOSED REMEDIATION:** quando `WORLD CONTINUITY AND PLAYER INDEPENDENCE.md`
  §17–§18 for implementada, o backend passa a materializar resumo regional e
  agregado estatístico nesses níveis.
- **TRIGGER:** Phase 4 (Living World).
- **TARGET STAGE:** Phase 4
- **STATUS:** OPEN

### DEBT-0020 — Chunks retidos moram na memória, não em arquivos de região

- **SYSTEM:** `engine/simulation::residency` (`RetainedChunks`)
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** um chunk editado e despejado é guardado num `BTreeMap` em
  memória até o próximo save. `RegionCoord` e `RegionShape` já existem na
  foundation exatamente para o destino correto: arquivo de região em disco.
- **IMPACT:** medido: **20,0 KiB por coluna editada** (Apêndice C). A retenção
  cresce com **quanto o mundo foi alterado**, não com o quanto foi explorado —
  que é o limite certo — mas ainda é memória, e não sobrevive ao processo.
- **RISK:** médio em sessão longa de construção; alto em servidor dedicado com
  muitos jogadores editando.
- **PROPOSED REMEDIATION:** escrever a coluna editada num arquivo de região no
  `persist` e lê-la de volta no `activate`. Isso também elimina a armadilha do
  `flush_into`, porque o estado deixa de depender de estar na memória na hora do
  save.
- **TRIGGER:** retenção passar de ~1.000 colunas, ou o primeiro servidor
  dedicado.
- **TARGET STAGE:** Phase 3 (Persistence + Simulation)
- **STATUS:** OPEN (medido)

### DEBT-0021 — Command System parou em CMD-4; CMD-5 a CMD-15 não existem

- **SYSTEM:** `engine/command`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `Command System.md` §114 define quinze etapas. O ADR-0010
  entrega CMD-0 a CMD-4 (contratos, registry, dispatch, validação, fila) porque
  as demais não têm chamador: networking, inventário, scripts, IA e console de
  admin ainda não existem. Construí-las agora seria adivinhar a forma da
  integração antes de haver o que integrar — startup brief §52.
- **IMPACT:** o que falta, e o que cada coisa espera:
  | etapa | o que falta | espera por |
  | --- | --- | --- |
  | CMD-5 | integração com networking | a camada de rede |
  | CMD-9 | transações: reserva, commit, rollback | inventário/crafting |
  | CMD-10/11/12 | scripts, NPC/IA, admin | mod runtime, IA, console |
  | CMD-13 | gravação e replay do log de comandos | `NEXORA REPLAY AND DETERMINISM.md` |
  | CMD-14 | dry run, preview, detecção de conflito | UI |
  | CMD-15 | batching, region affinity, orçamentos | mais de uma região |
- **RISK:** baixo hoje, e sobe junto com o primeiro consumidor de cada etapa.
  O risco real seria o inverso: um framework de transações sem nenhuma
  transação para modelar.
- **PROPOSED REMEDIATION:** cada etapa quando o seu consumidor chegar, na ordem
  do §114. A idempotência do §35 já é **detectada** (id de instância duplicado é
  recusado pela fila); falta a parte **transacional**, que é CMD-9.
- **TRIGGER:** por etapa, a existência do sistema que a consome.
- **TARGET STAGE:** Phase 2 em diante
- **STATUS:** OPEN

### DEBT-0022 — Validação de identidade não tem sessão para consultar

- **SYSTEM:** `engine/command::validation::IdentityValidator`
- **CLASS:** TEMPORARY
- **WHY CREATED:** `Command System.md` §20 quer verificar *"Connection 93
  claiming Player 17"* contra sessão, jogador, entidade e conexão reais. Não
  existe tabela de sessões ainda, então a camada verifica o invariante que dá
  para verificar: um ator que precisa de identidade tem de trazer alguma.
- **IMPACT:** um cliente que forje o `player id` de outro passa a camada de
  identidade hoje. As camadas seguintes ainda se aplicam — permissão, alcance,
  cota — mas a atribuição em si não é verificada, e é ela que decide *de quem*
  é a cota e *de quem* é a permissão.
- **RISK:** alto **assim que houver rede**, e exatamente zero antes disso: sem
  networking não existe conexão para mentir. É por isso que é `TEMPORARY` e não
  um furo aberto — a camada existe, com o nome certo, esperando a fonte de
  verdade.
- **PROPOSED REMEDIATION:** dar ao `ValidationRequest` acesso à tabela de
  sessões e checar que a conexão que trouxe o comando é dona do jogador que ele
  reivindica.
- **TRIGGER:** o primeiro comando que chegue por `Source::Network` de verdade.
- **TARGET STAGE:** CMD-5 (Server Integration)
- **STATUS:** OPEN

### DEBT-0023 — Comandos não emitem eventos; devolvem o nome deles

- **SYSTEM:** `engine/command::handler`, `engine/simulation::commands`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** §55 manda o sistema publicar um evento depois da mudança de
  estado, e §56 separa evento de resultado. O handler hoje devolve o **nome** do
  evento no `CommandResult` em vez de publicá-lo no Event Bus.
- **IMPACT:** quem quiser reagir a `nexora:block_broken` precisa ler o resultado
  do comando, o que inverte a relação: §110 diz que um evento tem muitos
  assinantes, e ninguém pode assinar um nome devolvido a um único chamador.
- **RISK:** médio. O `CommandResult` já carrega a informação certa, então a
  mudança é aditiva; o risco é alguém construir sobre o resultado por hábito e
  aí virar a forma de reagir a fatos.
- **PROPOSED REMEDIATION:** dar ao handler acesso ao `EventBus` e publicar de
  verdade, mantendo os nomes no resultado (§56: são respostas diferentes, as
  duas úteis). Precisa de tipos de evento concretos por domínio, que hoje não
  existem — `BlockBrokenEvent` mora em `Block System.md`, não implementado.
- **TRIGGER:** o primeiro assinante que precise reagir a um fato produzido por
  um comando.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN

### DEBT-0024 — Replay só alcança chunks residentes

- **SYSTEM:** `engine/world::recovery::apply`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `World::set_block` exige chunk residente, então um registro
  cujo chunk esta sessão ainda não carregou é reportado como `NotWritable` em
  vez de aplicado. Honesto — o relatório diz exatamente o que ficou de fora —
  mas incompleto: a recuperação vale só até onde o conjunto residente alcança.
- **IMPACT:** um mundo que salvou com 25 colunas residentes e recarrega com 9
  perde, no replay, as edições das 16 que faltam. Elas **não** somem: continuam
  no journal e o relatório as lista. Mas ninguém as reaplica depois.
- **RISK:** médio hoje (o slice recarrega o mesmo raio que salvou), e alto assim
  que o carregamento passar a ser guiado por interesse em vez de por raio fixo —
  aí o conjunto residente na recuperação quase nunca é o do save.
- **PROPOSED REMEDIATION:** indexar os registros por coluna e reaplicar os
  pendentes no momento em que cada chunk se torna residente, em vez de uma vez
  só no load. O `WorldResidency` do ADR-0008 já é o ponto onde uma coluna entra.
- **TRIGGER:** o primeiro load cujo conjunto residente difira do que escreveu o
  journal — na prática, streaming dirigir o carregamento inicial.
- **TARGET STAGE:** Phase 3
- **STATUS:** OPEN

### DEBT-0025 — O motor não escreve no journal ainda

- **SYSTEM:** `engine/world`, `engine/headless`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** o ADR-0011 construiu o formato, a gravação, o replay e a
  recuperação, com testes que exercitam os três casos obrigatórios. O que não
  fez foi ligar o `Journal` ao caminho de escrita do mundo: hoje quem journaliza
  é o teste, não o motor. Isso foi deliberado — depurar um formato novo e um
  ponto de chamada novo ao mesmo tempo é depurar dois de uma vez.
- **IMPACT:** enquanto isto não for feito, a 1.0.0 do save continua sendo
  "snapshot só" na prática, e o `DEBT-0001` estaria fechado no papel e aberto no
  comportamento. É por isso que este débito existe em vez de o anterior ficar
  entreaberto.
- **RISK:** médio. O risco não é o mecanismo estar errado — está testado — e sim
  alguém ler "journal implementado" e presumir durabilidade que o processo em
  execução ainda não tem.
- **PROPOSED REMEDIATION:** decidir a política de checkpoint (a cada quantos
  ticks, ou a cada quantos bytes de journal) e emitir um `EditRecord` em
  `World::set_block`, com `sync` na fronteira que o chamador considerar commit.
  O custo de `sync` por edição precisa ser medido antes de escolher a política —
  o harness já mede `save.write_atomic_disk` a 32 MiB/s.
- **RESOLUTION (2026-09-07):** medido primeiro, decidido depois
  ([Apêndice E](docs/benchmarks/PHASE-0-BASELINE.md), achado 21). Emoldurar e
  checksumar uma edição custa **672 ns**; torná-la durável custa **203 µs** —
  **303×**, e quase tudo custo fixo: 64 registros dividindo um flush saem a
  **4,6 µs cada**. Com isso a política se escolhe sozinha: as 78 escritas do
  slice custariam **15,8 ms** com fsync por edição (quase um quadro a 60 Hz) e
  custam **203 µs** com um flush por fronteira de commit (1,2% do quadro).
  Então `World::set_block` journaliza **sempre**, `World::sync_journal` torna
  durável, e `World::unsynced_edits` diz exatamente quanto uma queda custaria
  agora. A cauda não sincronizada é o preço, e o emolduramento do ADR-0011 é o
  que torna seguro pagá-lo. O journal mora **no** `World`, não em volta dele:
  mecanismo de durabilidade que o chamador pode esquecer de usar é mecanismo que
  vai ser esquecido — prova disso é que o estágio de comandos passou a ser
  journalizado sem que os handlers soubessem que o journal existe (78 = 76
  edições + 2 comandos).
- **TRIGGER:** antes de qualquer afirmação de que o save sobrevive a uma queda
  de processo no motor em execução.
- **TARGET STAGE:** Phase 3
- **STATUS:** CLOSED (2026-09-07)

### DEBT-0026 — O mesher só conhece uma classe de opacidade

- **SYSTEM:** `engine/mesh`, `engine/world::world::BlockDefinition`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `RENDERER and GRAPHICS.md` RENDER-10 pede quatro malhas por
  chunk — `opaqueMesh`, `cutoutMesh`, `transparentMesh`, `waterMesh`. O
  `BlockDefinition` de hoje carrega só `solid`: não há como saber que vidro é
  transparente. Construir as quatro camadas seria inventar dado de bloco, o que
  a §3 do briefing proíbe.
- **IMPACT:** vidro, folhagem, água e qualquer bloco não-opaco vão ocluir a face
  do vizinho e sumir do mundo visível. Não é um defeito do mesher — é a única
  resposta que o dado disponível permite.
- **RISK:** baixo hoje (não existe bloco não-opaco), alto no dia em que existir,
  porque o sintoma é "o mundo está sólido demais" e não um erro.
- **PROPOSED REMEDIATION:** dar ao `BlockDefinition` uma camada de render e
  sobrescrever `VoxelView::occludes` no `WorldSurfaces` — o método já é `trait`
  method com default exatamente para isso, então é **uma função**, não uma
  reescrita. Depois separar a saída por camada.
- **TRIGGER:** o primeiro bloco não-opaco no registro.
- **TARGET STAGE:** Phase 2
- **STATUS:** CLOSED (2026-09-08)
- **RESOLUÇÃO:** o `SurfaceMaterial` carrega o `BlendMode` do RENDER-13, e a
  `SurfaceTable` do `engine/simulation` resolve bloco → material → `occludes`.
  O mesher **não mudou uma linha**: o método do trait tinha default para
  exatamente este dia.
  Uma diferença em relação ao que estava proposto: a camada de render **não**
  foi para o `BlockDefinition`. Ela mora no material, e um bloco aponta para um
  material por tabela lateral — o mesmo padrão que o `WorldVoxels` usa para
  material físico. Isso mantém o `BlockDefinition` com um campo só, como o
  `CORE.md` §5 pede, e faz vidro e vidro-tingido compartilharem a decisão em
  vez de repeti-la.
  **Fecha só metade do que este item descrevia.** As quatro malhas por chunk do
  RENDER-10 continuam não existindo — ver `DEBT-0035`, fechado em 2026-09-12
  com três das quatro; a quarta virou o `DEBT-0036`.

### DEBT-0027 — Meshing roda na thread que pedir, não em worker

- **SYSTEM:** `engine/mesh`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** RENDER-12 quer a geração de malha fora da thread principal,
  *"crucial para mineração/construção sem travar a câmera"*. O mesher é uma
  função pura hoje; nada o agenda.
- **IMPACT:** medido — `mesh.region_32` custa **26,12 ms**, mais de um quadro a
  60 Hz. Uma região de 32³ remeshada na thread do tick é um engasgo visível.
- **RISK:** alto assim que houver um loop de quadro, e exatamente zero antes.
- **PROPOSED REMEDIATION:** submeter ao job system, com o snapshot do
  `DEBT-0029` como entrada — é ele que torna o meshing off-thread **seguro**,
  não só mais rápido, porque o worker deixa de tocar o mundo.
- **TRIGGER:** existir um loop de quadro.
- **TARGET STAGE:** Phase 2/5
- **STATUS:** OPEN (medido)

### DEBT-0028 — Não há malha de LOD

- **SYSTEM:** `engine/mesh`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** `RENDERER and GRAPHICS.md` prevê malha simplificada para
  distância, e o `STREAMING SYSTEM.md` já tem a escada `FULL → REGIONAL →
  ABSTRACT`. O mesher só produz detalhe completo.
- **IMPACT:** nenhum hoje — nada desenha. Quando desenhar, a contagem de
  triângulos cresce com o volume visível em vez de com o que dá para distinguir.
- **RISK:** baixo até existir renderização a distância.
- **PROPOSED REMEDIATION:** o `RENDER-9` já lista Surface Nets, Marching Cubes e
  Dual Contouring como algoritmos futuros; a `VoxelView` é a fronteira por onde
  um deles entra sem tocar no resto.
- **TRIGGER:** o primeiro nível de LOD que precise desenhar.
- **TARGET STAGE:** Phase 5
- **STATUS:** OPEN

### DEBT-0029 — 91% do meshing é lookup no mundo, não meshing

- **SYSTEM:** `engine/simulation::surfaces`, `engine/mesh`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** descoberto pela medição em par, não por suspeita
  ([Apêndice F](docs/benchmarks/PHASE-0-BASELINE.md), achado 22): a mesma região
  de 16³ custa **3,30 ms** lida do mundo e **295,45 µs** lida de um array denso
  pré-carregado. **11,2×.** O `mesh.cull_only_16` a 3,20 ms diz o mesmo pelo
  outro lado — a fusão gulosa é ~3% do total.
- **IMPACT:** otimizar o algoritmo do mesher renderia no máximo 9%. O ganho está
  em ler os voxels uma vez.
- **RISK:** médio, e o mesmo padrão do `DEBT-0011` (49% de um passo de física é
  o lookup de voxel). Duas medições independentes apontando para o mesmo lugar.
- **PROPOSED REMEDIATION:** um `DenseSnapshot` de verdade em `nexora-mesh` — a
  região mais uma borda de uma célula, lida de uma vez. O benchmark já tem a
  versão-fixture que produziu o número; promovê-la é pequeno. **E resolve duas
  coisas:** é também o que torna o `DEBT-0027` (meshing off-thread) seguro, já
  que o worker passa a não tocar o mundo.
- **TRIGGER:** o primeiro remesh no caminho de um quadro.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN (medido)

### DEBT-0030 — A inclinação do normal map não tem significado físico

- **SYSTEM:** `tools/texture-forge::pbr`, `engine/asset::material`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** um material declara `metres_per_tile`, mas **não declara a
  amplitude do relevo em metros**. O campo de altura vive em `0.0..=1.0` e não
  há como convertê-lo honestamente em gradiente real. O `normal_strength` é,
  portanto, um controle **estilístico**, e o pipeline diz isso em vez de
  inventar uma constante que pareceria física.
- **IMPACT:** duas texturas com o mesmo relevo aparente e escalas físicas
  diferentes produzem normais iguais. Sob luz rasante, a parede de tijolo e o
  tijolo isolado vão reagir igual, o que está errado.
- **RISK:** baixo enquanto não houver iluminação; médio no dia em que houver,
  porque o sintoma é "o relevo parece raso demais" e não um erro.
- **PROPOSED REMEDIATION:** um campo `relief_metres` na `SurfaceMaterial` (ou
  derivado de `physical_scale` × uma fração declarada), e o Sobel passa a
  produzir gradiente em metros por metro. É **um campo e três linhas** — o que
  falta é a decisão de arte sobre o valor, não o código.
- **TRIGGER:** a primeira luz direcional no renderizador.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN

### DEBT-0031 — O PNG sai sem compressão, e a compressão vale 12×

- **SYSTEM:** `tools/texture-forge::png`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** o codificador escreve blocos **deflate armazenados** —
  legais, lidos por qualquer decodificador, e sem compressão nenhuma. Foi a
  escolha certa para nascer correto; escrever um compressor antes de saber
  quanto os arquivos pesam seria adivinhar.
- **IMPACT:** medido, não estimado. Seis mapas gerados (albedo e altura de
  wood/stone a 64², brick a 128²) pesam **123 805 bytes** como estão e
  **10 220 bytes** com deflate real: **12,11× maior**. Os albedos comprimem 16
  a 48× justamente porque a quantização de paleta produz corridas longas; as
  alturas, 2 a 10×.
- **RISK:** alto na escala declarada pelo operador. Dez mil materiais × 6
  mapas × 64² são da ordem de **1 GB** como está, contra **~85 MB** comprimido.
  Isso é a diferença entre caber num repositório e não caber.
- **PROPOSED REMEDIATION:** deflate de Huffman fixo com um localizador de
  correspondências simples, **dentro deste módulo** — nada que o chame muda.
  A filtragem adaptativa do PNG só paga na frente de um compressor (medido: sem
  compressor ela não muda nada), então entra junto.
- **TRIGGER:** já disparado pela medição acima.
- **TARGET STAGE:** imediatamente após a FASE 3
- **STATUS:** CLOSED (2026-09-08)
- **RESOLUÇÃO:** `tools/texture-forge/src/deflate.rs` — Huffman fixo com
  localizador por cadeia de hash, mais a escolha de filtro por scanline no
  `png.rs`. Vinte PNGs de um conjunto PBR completo foram inflados pelo `zlib`
  do Python e cada pixel voltou idêntico; o codificador escolheu quatro dos
  cinco filtros.
  **E uma correção ao número acima:** os 12,11× foram medidos sobre seis mapas
  que eram só albedo e altura, e albedo quantizado é o melhor caso que existe.
  Sobre o conjunto PBR inteiro — vinte mapas, com normal, roughness e oclusão,
  que são contínuos — o alcançável é **6,79×** (zlib -9) e este codificador
  entrega **4,73×**. Ver o `DEBT-0034` para a diferença que sobra.

### DEBT-0032 — Metallic é constante porque nenhuma receita tem metal por texel

- **SYSTEM:** `tools/texture-forge::pbr`, `tools/texture-forge::recipe`
- **CLASS:** CONTENT
- **WHY CREATED:** o pipeline emite o valor declarado pelo material em todo
  texel. Não é um mock: é a resposta correta para uma superfície uniforme, e é
  tudo que o dado disponível permite. O que não existe é receita que produza
  metal **variável** — ferrugem, veio de minério, verniz descascado.
- **IMPACT:** um bloco de minério não pode ter o minério metálico e a rocha
  dielétrica. O mapa gasta bytes para dizer uma constante.
- **RISK:** baixo. O caminho está pronto: basta uma receita emitir uma máscara.
- **PROPOSED REMEDIATION:** uma quarta saída opcional da `Recipe` (uma máscara
  em `0.0..=1.0`), consumida pelo pipeline no lugar da constante. E, quando o
  mapa for constante, **não emiti-lo** — o valor já está na definição.
- **TRIGGER:** o primeiro material com metal parcial.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN

### DEBT-0033 — O codificador escreve oito bits por canal e recusa dezesseis

- **SYSTEM:** `tools/texture-forge::png`
- **CLASS:** TEMPORARY
- **WHY CREATED:** `TextureFormat::sixteen_bit` existe no contrato porque um
  mapa de altura com banding visível é um problema real, mas **nada produz 16
  bits ainda**. Escrever a codificação seria código não testado por dado que
  não existe; o codificador recusa alto em vez disso.
- **IMPACT:** um mapa de altura de 8 bits tem 256 degraus. Em relevo suave e
  larga escala isso aparece como faixas no normal derivado.
- **RISK:** baixo hoje. As alturas atuais vêm de receitas com quantização de
  paleta, onde 256 degraus não é o limitante.
- **PROPOSED REMEDIATION:** PNG guarda amostras de 16 bits em big-endian; são
  ~5 linhas no laço de scanline mais o `bit_depth` no IHDR. O que falta é um
  gerador que produza o dado, para que exista teste.
- **TRIGGER:** o primeiro mapa de 16 bits, ou banding visível num normal.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN


### DEBT-0034 — O compressor é 1,40× pior que o zlib -9

- **SYSTEM:** `tools/texture-forge::deflate`
- **CLASS:** PERFORMANCE
- **WHY CREATED:** o compressor usa **Huffman fixo** (a tabela que os dois lados
  já conhecem, sem árvore no fluxo) e **correspondência gulosa** (a primeira
  correspondência mais longa encontrada, sem olhar se adiar um byte renderia
  uma melhor). O `zlib -9` faz Huffman dinâmico e correspondência preguiçosa.
- **IMPACT:** medido sobre as mesmas vinte scanlines filtradas de um conjunto
  PBR completo: **59 518 bytes** contra **42 472** do `zlib -9` — 1,40× maior,
  com o pior arquivo individual a 1,50× (`sand_64_height`). Na escala de dez
  mil materiais isso é da ordem de 120 MB contra 85 MB.
- **RISK:** baixo. É espaço, não correção, e o formato de saída continua sendo
  PNG válido para qualquer decodificador.
- **PROPOSED REMEDIATION:** duas coisas independentes, nesta ordem de retorno:
  **(1) correspondência preguiçosa** — adiar um byte quando a posição seguinte
  oferece uma correspondência maior; é ~20 linhas e costuma valer a maior parte
  da diferença. **(2) Huffman dinâmico** — contar frequências, construir o
  código canônico e emitir a árvore; é a metade cara e rende menos.
- **TRIGGER:** quando o repositório de assets passar de algumas centenas de
  megabytes, ou antes de empacotar uma release.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN (medido)

### DEBT-0035 — A malha ainda sai numa camada só, não nas quatro do RENDER-10

- **SYSTEM:** `engine/mesh`, `engine/simulation::surfaces`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** a metade restante do `DEBT-0026`. O `RENDER-10` pede
  `opaqueMesh`, `cutoutMesh`, `transparentMesh` e `waterMesh` por chunk, porque
  superfícies transparentes precisam ser desenhadas depois das opacas e
  ordenadas de trás para frente. O `mesh_region` devolve **uma** `ChunkMesh`
  com tudo dentro.
- **IMPACT:** com o `BlendMode` já disponível, o dado para separar existe — o
  que não existe é a separação. Um vidro desenhado junto com a pedra vai
  compor errado assim que houver blending de verdade.
- **RISK:** baixo hoje (não há renderizador), alto no primeiro frame com
  transparência, e o sintoma é "o vidro está preto" ou "o vidro some quando
  olho de certo ângulo" — nenhum dos dois parece um bug de meshing.
- **PROPOSED REMEDIATION:** `mesh_region` passa a devolver uma malha por
  camada, e o `VoxelView` ganha um método que diz a camada de uma superfície
  (com default `Opaque`, como o `occludes` teve). A `SurfaceTable` já sabe a
  resposta; é propagá-la.
- **TRIGGER:** o primeiro renderizador que faça blending, ou o primeiro bloco
  `Cutout` (folhagem) no conteúdo.
- **TARGET STAGE:** Phase 2
- **STATUS:** CLOSED (2026-09-12)
- **RESOLUÇÃO:** o gatilho disparou por conta própria — o manifesto de exemplo
  do Texture Forge trouxe `leaf_canopy` com `blend: cutout`, que é literalmente
  *"o primeiro bloco `Cutout` no conteúdo"*.
  `mesh_region` devolve `LayeredMesh` e o `VoxelView` ganhou `layer_of`, com
  default `Opaque` — o mesmo padrão que fez o `occludes` do `DEBT-0026` ser uma
  função e não uma reescrita.
  **A varredura e a fusão não mudaram uma linha.** A separação acontece no
  momento em que um retângulo é emitido, não varrendo a região três vezes: a
  fusão gulosa só junta faces de superfícies **iguais**, e uma superfície tem
  exatamente uma camada, então todo retângulo já pertence a um passe quando
  passa a existir. `layer_of` é chaveado por `SurfaceId` por isso — não por
  posição, como o `occludes`.
  Medido: a geometria é a mesma de antes — **807 retângulos, 3 228 vértices**
  na região de 16³, os números já registrados — e `mesh.region_16` menos
  `mesh.cull_only_16` é **0,03 ms**, que é a fusão e o roteamento juntos.
  **Três camadas, não as quatro do RENDER-10.** A `waterMesh` continua sem
  existir e não por esquecimento: **nada no motor diz que um bloco é água.**
  Não há conceito de fluido em `engine/world`, nem em `nexora_asset`, nem uma
  `MaterialCategory` de líquido. Emitir a camada seria inventar o dado que
  decide o que entra nela — a mesma recusa que o `DEBT-0026` fez. Ver
  `DEBT-0036`.

### DEBT-0036 — A camada de água do RENDER-10 não tem dado que a defina

- **SYSTEM:** `engine/mesh`, `engine/simulation::surfaces`, sistema de fluidos
  (inexistente)
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** o resto do `DEBT-0035`. O `RENDER-10` pede quatro malhas por
  chunk e três foram entregues. A quarta, `waterMesh`, não sai do `BlendMode`:
  água é uma categoria de **conteúdo**, não um modo de composição. Um bloco de
  água é `Transparent` como o vidro é, e ainda assim precisa de malha própria —
  shader próprio, animação de superfície própria, ordem própria contra o resto
  da transparência. Nada disso um `BlendMode` consegue dizer.
- **IMPACT:** hoje um bloco de água cairia em `Transparent`, que é onde ele
  pertence entre as três que existem. Isso está certo até haver um shader de
  água; a partir daí a água precisa ser desenhada separada e ordenada contra a
  outra transparência, e uma malha só não permite isso.
- **RISK:** zero hoje (não há fluidos nem renderizador), médio no primeiro
  shader de água.
- **PROPOSED REMEDIATION:** quando o sistema de fluidos existir, é ele que diz
  quais blocos são fluido. Aí `RenderLayer` ganha uma variante e o mapeamento
  em `layer_of_blend` ganha um braço — e nada mais, porque o roteamento já
  acontece por superfície.
- **TRIGGER:** o sistema de fluidos, ou o primeiro shader de água.
- **TARGET STAGE:** Phase 2/3
- **STATUS:** OPEN