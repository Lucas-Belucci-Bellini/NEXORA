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
- **RESOLUÇÃO PARCIAL (2026-09-15):** existe agora um
  `nexora_world::region::RegionStore` — um diretório com **um arquivo por
  região**, cada um um `SaveContainer` comum. Decisões em
  [ADR-0014](docs/adr/ADR-0014-a-region-file-is-authoritative-for-its-region.md).

  **Salvar depois de um bloco mudar: 12,8–13,2 ms → 2,9 ms, e 40,8 KiB →
  4,7 KiB.** Cerca de 4,5× menos tempo e 8,7× menos bytes, no mundo de
  benchmark agrupado a duas colunas por região. A região limpa não é
  codificada, não é deflacionada, não é escrita e não é relida — o arquivo dela
  não é aberto.

  **O número que não depende da máquina é uma contagem:**
  `save.regions_written_per_edit` é **1** de 4, em qualquer caixa e em qualquer
  build. É também a confirmação independente de que a sujeira por seção — que
  já era rastreada e nunca tinha sido lida na hora de salvar — agora é lida.

  **O preço, medido e não escondido: a escrita completa custa 13–17% a mais** e
  o total cresce 2,2%. Cinco arquivos em vez de um são cinco molduras, cinco
  `fsync`, cinco releituras de verificação e cinco fluxos deflate que não
  compartilham dicionário; a paleta também é escrita uma vez por região. A
  escrita completa é exatamente o caso para o qual este arranjo **não** é.

  **Dividir a seção dentro do mesmo contêiner não resolveria nada hoje** — o
  contêiner codifica como uma unidade, então toda seção deflaciona a cada
  `encode` e o arquivo inteiro é escrito e verificado. O custo segue o arquivo,
  então as regiões precisavam ser arquivos.
- **STATUS:** OPEN (reduzido) — o mecanismo existe e está medido, mas
  **nada no motor ainda salva por ele**: o slice escreve o contêiner único e o
  `RegionStore` roda ao lado como verificação. Fechar exige escolher o
  `RegionStore` como o formato de save do runtime. O `DEBT-0020` — a coluna
  despejada ir para o arquivo de região em vez do `BTreeMap` — foi remediado em
  seguida e é **opt-in**; enquanto o padrão for a memória e o save for o
  contêiner único, o store continua sendo um destino que o motor sabe escrever
  e não o lugar de onde ele lê. No extent padrão de 32 colunas, todo mundo deste
  repositório cabe numa região só — a economia é real a partir da escala em que
  um mundo passa de uma região, e não antes.

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
- **RESOLUÇÃO (2026-09-14):** a remediação proposta, **num lugar diferente do que
  esta entrada supôs**. O `SYSTEM` dizia `engine/world::persist`; a costura certa
  é o contêiner. Toda seção passa pela mesma moldura — chunks hoje, entidades
  amanhã — e resolver no contêiner faz isso uma vez em vez de cada produtor
  decidir por si. O `world::persist` não mudou uma linha.

  **O save do slice: 744.954 → 116.904 bytes, 6,4× menor.** *(Corrigido em
  2026-09-15: a publicação original emparelhava 744.954, que é o slice na seed
  padrão, com 117.908, que é o slice do smoke de determinismo na seed
  987654321 — dois mundos diferentes. Ambos os números estavam certos; o par
  não estava. Na seed padrão, o antes e o depois são 744.954 e 116.904.)* O
  codec é o mesmo do
  DEBT-0034 (1,02× do zlib), que estava em `tools/texture-forge` onde só o PNG o
  alcançava; foi promovido para `nexora_foundation::deflate`. **Zero arestas
  novas no grafo de crates** — persistência e texture-forge já dependiam de
  foundation. Os 12 PNGs saem byte-idênticos antes e depois da mudança de casa.

  **Comprimir é mantido só quando ganha.** Bytes de alta entropia deflacionam
  para um pouco mais do que eram, e escrever isso pioraria o formato exatamente
  nas entradas em que ele já é pior. `Coding::Stored` não é caminho de falha —
  não há falha — é a resposta quando comprimir não pagou.

  **Formato 2, e o formato 1 continua legível.** `MIN_SUPPORTED_SAVE_FORMAT`
  ficou em 1 de propósito: ler um save antigo custa um `if` no decodificador, e
  recusá-lo jogaria fora todo mundo escrito antes da mudança por nada além de
  conveniência. Verificado contra um arquivo formato 1 **real**, escrito pelo
  build anterior, além do teste que monta a moldura à mão.

  **Um achado no caminho, de um teste que falhou.** O primeiro teste de dano
  virou um bit "no meio do arquivo" e acertou o byte de codificação — pego, mas
  pelo guarda errado. Isso expôs que os campos novos da moldura estavam **fora**
  do checksum da seção, cujo comentário já dizia que o nome entra nele
  justamente para não ser renomeado em silêncio. A mesma razão vale para a
  codificação e o comprimento: um bit virado ali não parece dano, parece
  instrução diferente — um fluxo deflate virando "isto é cru". Agora os quatro
  campos estão dentro do `frame_crc`, e há teste para o byte de codificação.
- **STATUS:** **CLOSED** — o save deixou de gravar palavras empacotadas cruas. O
  algoritmo está na moldura de cada seção, então trocá-lo é uma versão de
  formato e não uma migração.

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
- **MEDIÇÃO (2026-09-16):** medir esta entrada achou um defeito **maior do que
  ela**, e que ela não nomeia. O teto era imposto com `Vec::remove(0)`, então
  toda escrita passada a marca movia as outras 4.095 entradas:

  | | mediana |
  | --- | ---: |
  | `chunk.change_feed_append` (feed abaixo do teto) | 40,7 ns |
  | `chunk.change_feed_at_cap` (feed cheio, descarta a cada escrita) | **49,95 µs** |

  **1.227×**, pago exatamente pela coluna que está sendo mais editada — a que
  chega ao teto. São ~190 KB de `memmove` por voxel escrito.
- **RESOLUÇÃO PARCIAL (2026-09-16):** duas das três metades. A terceira está
  bloqueada e o bloqueio é o que esta entrada sempre disse.

  **O teto ficou de graça: 49,95 µs → 42,0 ns.** O feed virou `VecDeque` e o
  descarte é `pop_front`. O número que importa não é o 1.190× — é que
  `change_feed_at_cap` (42,0 ns) e `change_feed_append` (40,9 ns) agora são **o
  mesmo número**: descartar deixou de custar. Os controles não se mexeram
  (`voxel.get_paletted` 6,6 → 6,5 ns, `get_uniform` 3,2 → 3,2 ns,
  `set_existing_state` 13,5 → 14,1 ns, `compact_section` 164 → 169 µs). Isto é
  provado pelo benchmark e não por teste: trocar a estrutura de dados não muda
  comportamento, e os testes de correção passam nas duas versões.

  **O silêncio parcial fechou, pelo tipo.** `take_journal` devolvia
  `Vec<VoxelChange>` **e zerava** `dropped_journal_entries` — então quem drenava
  sem olhar antes destruía o único registro de que faltava coisa. Agora devolve
  um `ChangeFeed { changes, dropped }`, `#[must_use]`: a lacuna sai junto com o
  dado ou não sai. Os dois lugares que descartavam o retorno em silêncio
  (`persist::decode_chunks` e `World::generate_chunk`) foram apontados pelo
  próprio `must_use` e viraram `clear_journal()`, que é o que eles queriam dizer
  — um chunk recém-gerado ou recém-lido não tem feed para ninguém consumir.

  **E alguém lê o sinal.** `World::change_feed_gaps()` soma as lacunas dos chunks
  residentes, e o slice falha se houver alguma. Com 76 edições em 25 colunas
  contra um teto de 4.096 isso não dispara hoje — o ponto é que passou a ser
  **verificado** em vez de apenas contável.

  **Uma correção de leitura:** uma lacuna aqui **não é perda de durabilidade**.
  `World::set_block` grava no journal de save (ADR-0011) **antes** de tocar o
  chunk; o feed do chunk é alimentação de mudanças para History e replicação.
  Perder uma entrada custa história, nunca o bloco.
- **STATUS:** OPEN (bloqueado) — **o feed não tem nenhum consumidor real**, e é
  isso que sobra. `take_journal` não é chamado por nada no motor: o History
  System é Phase 11 e não existe, e a replicação não chegou. A remediação
  proposta (drenar por tick para o History) continua sendo a certa e continua
  esperando o History; construir um consumidor agora seria construir na frente
  da evidência. O que mudou é que, quando ele chegar, o teto não custa nada e a
  lacuna não tem como passar despercebida.

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
- **PROGRESS (2026-09-26):** a etapa **RHI** deixou de ser "presa no
  hardware". A ADR-0026 construiu o backend nativo e a ADR-0027 a janela, e o
  lavapipe (Vulkan por software) roda os dois sem GPU. O benchmark agora mede
  a etapa RHI (`nexora_benchmark::gpu`): abrir o device, round-trip de fence,
  criar e destruir textura, upload de uma textura 16×16, upload das dezesseis
  da primeira geração num fence só, upload do vertex buffer da região 16³
  meshada, e um draw 16×16. Antes de cronometrar, ele confere as respostas
  (leitura de volta do upload e do draw). O relatório imprime o adaptador, e
  no CI ele é `llvmpipe … (vulkan, cpu)`: número de CPU, não de GPU. Resultado
  em [Apêndice J](docs/benchmarks/PHASE-0-BASELINE.md), achado 28: o fence
  domina o que é pequeno, e o upload da fatia em um fence só custa um sexto de
  dezesseis uploads separados. O que ainda falta desta dívida: **frame time**
  e **câmera** (não existe renderer), a etapa **window** dentro do benchmark,
  números **em GPU real** (a próxima execução de `local-validation.py` na
  RX 6650 XT mede a etapa RHI no `benchmark_cpu`) e a comparação em **escala
  de motor** do ADR-0009.
- **PROGRESS (2026-09-26, relatórios locais 4 e 5):** a etapa RHI tem
  números **de GPU real**: a RX 6650 XT do operador, Vulkan, `discretegpu`,
  duas execuções do mesmo código
  ([Apêndice K](docs/benchmarks/PHASE-0-BASELINE.md), achado 29). O achado 28
  se confirma e fica mais forte: num dispositivo do outro lado do PCIe, o
  round-trip de fence (112–162 µs) é **maior** que no lavapipe (68 µs), e um
  draw 16×16 (192–205 µs) custa pouco mais que um fence. Em GPU real o custo
  pequeno é espera, não trabalho: o renderer precisa de **uma submissão por
  quadro**, não de uma por draw. Nenhum orçamento: entre as duas execuções o
  fence variou 45%. Continua faltando: **frame time**, **câmera**, a etapa
  **window** no benchmark e a comparação em **escala de motor**.
- **PROGRESS (2026-09-26, ADR-0029):** a etapa **câmera** existe e é medida:
  `engine/camera` (visão, projeção reverse-Z, frustum e origem de render
  inteira flutuante) e `suites::camera`, com a câmera a 2^40 blocos. Resolver
  a câmera custa ~0,1 µs e testar as 625 colunas de um observador de raio 12
  custa 3–7 µs: menos de 0,05% de um quadro de 60 Hz
  ([Apêndice L](docs/benchmarks/PHASE-0-BASELINE.md), achado 30). Continua
  faltando: **frame time** (nada desenha um quadro pela câmera ainda), a etapa
  **window** no benchmark e a comparação em **escala de motor**.
- **TARGET STAGE:** antes da Phase 2
- **STATUS:** IN PROGRESS — a segunda linguagem e a etapa RHI estão medidas;
  faltam frame time e câmera (código: não há renderer), números em GPU real
  (relatório local) e a comparação em escala de motor

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
- **RESOLUÇÃO PARCIAL (2026-09-19):** a hipótese acima foi **medida antes de ser
  consertada**, e estava certa sobre o despertar e errada sobre o mutex ser a
  causa. Ver o achado 23 do `PHASE-0-BASELINE.md`.

  **Uma submissão é 2% enfileirar e 98% acordar um worker.** Medindo `submit()`
  em três estados que só diferem no que o pool está fazendo:

  | `submit()` com… | mediana |
  | --- | ---: |
  | todo worker **parado** na fila — um despertar por submissão | **12,20 µs** |
  | todo worker **ocupado** dentro de um job — ninguém para acordar | **288 ns** |
  | workers **drenando** (o que o `jobs.submit_only` mede) | **12,18 µs** |

  42× entre as duas primeiras, e a única diferença é existir uma thread para
  acordar. As peças somadas à parte dão ~142 ns; um `notify_one` para uma thread
  parada custa 646 ns sozinho e ~11,9 µs dentro do pool. A diferença é o
  handoff: o worker acordado vai imediatamente buscar o mesmo mutex que o
  produtor precisa para a próxima submissão. **O mutex é o amplificador, não a
  causa** — é ele que transforma um despertar numa ida e volta inteira.

  **`JobSystem::submit_all` acorda uma vez por onda, não por job.** Um lock,
  a onda inteira enfileirada, e `min(jobs, workers)` despertares:

  | | um a um | `submit_all` | |
  | --- | ---: | ---: | ---: |
  | 1.000 jobs submetidos e executados | 11,40 ms | **1,88 ms** | **6,1×** |
  | custo do produtor por job | 10,95 µs | **103 ns** | **106×** |
  | **despertares por 1.000 jobs** | **1 000** | **4** | contagem |

  A última linha é a que não é desta máquina. Os controles — `jobs.submit_only`
  e `jobs.submit_wait_roundtrip`, que continuam no caminho antigo — não se
  mexeram (10,92/10,87 → 10,95 µs e 36,26/34,44 → 36,31 µs), e é isso que diz
  que refatorar o `submit` para compartilhar o enfileiramento não custou nada.

  **A nota de arquitetura continua valendo inteira.** 103 ns por job contra
  ~3 ns para simular uma entidade inline ainda são ~35×: um job por entidade
  segue sendo anti-padrão. O que o lote conserta é submeter uma **onda de
  trabalho real** — as 25 colunas do slice agora vão como um lote só.
- **STATUS:** OPEN (reduzido) — o caminho de onda está consertado e medido; o
  `submit()` de um job isolado continua custando ~11 µs quando há worker parado,
  e ninguém mediu ainda se dá para baixar isso sem filas por worker. O segundo
  candidato da remediação (work-stealing) não foi construído: não há caso medido
  que o exija depois que a onda deixou de ser o gargalo.

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
- **RESOLUÇÃO (2026-09-16):** existe índice espacial, e ele foi medido **antes**
  — que era o que a remediação pedia. Ver [ADR-0015](docs/adr/ADR-0015-a-spatial-index-is-a-loose-grid-and-a-query-may-decline-it.md)
  e o achado 10l do `PHASE-0-BASELINE.md`.

  **A extrapolação acima foi conferida e quase toda ela se sustentou.** Medindo a
  varredura em 1.000, 10.000 e 100.000: `within_radius` custa 9,8–10,9 ns por
  entidade e `in_chunk` 16,6–17,3 ns, estáveis em duas ordens de grandeza. A
  linha que **não** se sustentou foi `by_type`: 15,7 ns por entidade em 1.000 e
  **40,7 ns** em 100.000, porque ela casa com a população inteira e o custo está
  em montar o resultado, não em examinar. A nota equivalente da ADR-0006 errou
  para menos pelo mesmo motivo.

  **Isso mudou o que foi construído.** Consulta sem posição não tem índice que
  ajude — nenhum arranjo encolhe uma resposta que já é tudo. Então `matching`,
  `count`, `by_type` e `by_tag` continuam varredura de propósito, e o controle
  provou que continuaram: `by_type` em 100.000 ficou em 4.069 → 4.196 µs.

  **O que o índice comprou**, com população espalhada como um mundo espalha:

  | | varredura | índice | |
  | --- | ---: | ---: | ---: |
  | `within_radius(16)`, 100.000 | 979,7 µs | **782 ns** | 1.253× |
  | `in_chunk`, 100.000 | 1.655,4 µs | **625 ns** | 2.647× |

  **O número que não é desta máquina é 41.** `entity.query_radius_candidates_100k`
  = 41 entidades examinadas, de 100.000, para responder um raio de 16 blocos. É
  contagem: igual em qualquer build, em qualquer caixa. E o tempo deixou de
  crescer com a população — 430, 654 e 782 ns para 1.000, 10.000 e 100.000.

  **O preço, dito inteiro:** `entity.step_1000` foi de 2,07–2,11 µs para
  7,2–8,7 µs, ~4×, que são ~6,5 ns por entidade por tick. Metade da piora era
  `f64::floor`: o baseline x86-64 não tem `roundsd` (é SSE4.1), então `floor` é
  chamada de libm — 4,94 ns contra 1,54 ns na forma com `as i64` mais correção.
  O pior caso, toda entidade mudando de célula todo tick, é 170,9 µs por 1.000.

  **E a parte incômoda:** no estágio de 1.000 entidades do próprio plano, este
  índice é **prejuízo líquido de ~6,5 µs por tick**. A fixture do plano empacota
  1.000 entidades em doze células, então o raio de 16 blocos ali pergunta pela
  população quase toda e não há o que excluir. A fixture **não** foi trocada —
  trocá-la tornaria toda linha de 1.000 entidades do baseline incomparável com
  todas as execuções anteriores, para fazer uma mudança parecer melhor. As linhas
  novas medem população espalhada ao lado dela, e as duas estão publicadas.
  6,5 µs é 0,013% de um tick de 50 ms.

  **Manter ligado sempre não é decisão de número, é de obsolescência.** Índice
  que a store mantém só às vezes é índice que a consulta não pode confiar, e
  índice espacial errado devolve entidade errada em silêncio. A ADR-0015 fecha
  isso.
- **STATUS:** **CLOSED** para as consultas espaciais. O que fica registrado, e
  não é o mesmo defeito: a remoção de uma célula varre o vetor daquela célula
  para achar o slot, então uma célula com dezenas de milhares de entidades torna
  cada `despawn` ou travessia proporcional a ela. Com densidade de mundo isso
  são dezenas de entradas; o gatilho é uma célula passar de ~1.000. Não vira
  entrada nova porque não há caso medido — é o mesmo erro que esta entrada
  acabou de evitar.

### DEBT-0040 — O job system guarda o resultado de todo job para sempre

- **SYSTEM:** `engine/runtime::jobs`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** encontrado ao medir o DEBT-0009, não por suspeita de projeto.
  `worker_loop` faz `state.results.insert(handle, outcome)` ao terminar cada job,
  e **nada remove**: `JobSystem::wait` lê com `get` e clona, porque um segundo
  `wait` no mesmo handle tem de continuar funcionando. O sistema não tem como
  saber que ninguém mais vai perguntar.
- **IMPACT:** um `(JobHandle, JobOutcome)` por job já submetido, para sempre, num
  `HashMap` dentro do mutex do escalonador. A 113.000 jobs/s isso é da ordem de
  dezenas de MB por hora de sessão. **Não custa vazão**: medido, `submit()` lê
  12,15 / 10,99 / 12,28 µs com 0, 100.000 e 500.000 resultados retidos — plano,
  dentro do ruído. É defeito de memória, e só.
- **RISK:** baixo numa sessão curta; alto num servidor dedicado, que é
  exatamente onde o processo não reinicia.
- **PROPOSED REMEDIATION:** decidir a semântica antes de mexer, porque as opções
  não são equivalentes. `wait` que **consome** o resultado é uma mudança de API
  observável (o segundo `wait` deixa de achar). Reter só os handles que alguém
  pode ainda esperar exige saber isso, e o sistema não sabe. Um teto com
  descarte do mais antigo troca vazamento por resposta perdida em silêncio — o
  mesmo defeito que o DEBT-0004 fechou em outro lugar. Provavelmente é ADR.
- **MITIGAÇÃO ATUAL:** `JobMetrics::retained_results` publica o tamanho, então o
  crescimento é **verificável** em vez de silencioso — e há teste cobrindo isso.
  É o mesmo movimento do DEBT-0004: medir o sinal antes de ter o consumidor.
- **TRIGGER:** primeiro processo de vida longa — servidor dedicado, ou o slice
  passar a rodar por horas.
- **TARGET STAGE:** Phase 2 em diante
- **RESOLUÇÃO (2026-09-20):** [ADR-0016](docs/adr/ADR-0016-a-job-result-can-be-forgotten-and-says-so.md).
  A remediação pedia decidir a semântica antes de mexer, e as três opções que
  ela listou continuam com os defeitos que ela apontou. A escolhida foi a
  terceira — **teto com descarte do mais antigo** — com a objeção dela
  respondida em vez de ignorada: o descarte virou **resposta**, não ausência.

  `JobOutcome::Forgotten` é variante própria, e `wait` passou a ter três saídas:
  devolve o resultado se estiver retido, **continua bloqueando enquanto o pool
  ainda segura o job**, e devolve `Forgotten` quando nem um nem outro. O sinal
  que separa os dois últimos é o `tokens`, que já existia e já era limitado —
  entra no submit, sai na conclusão, então guarda exatamente os jobs em voo.
  Nenhuma estrutura nova foi necessária para responder a pergunta.

  Três coisas que são o ponto, não efeito colateral:

  - **Job esquecido nunca é reportado como sucesso.** O atalho tentador — tratar
    resultado ausente como "então deu certo" — deixou de existir:
    `is_success()` é falso para `Forgotten`, e `is_known()` existe para
    perguntar direto. Um job que falhou e foi descartado leria como sucesso, que
    é a única resposta que um escalonador não pode inventar.
  - **`wait` agora sempre termina, e antes não terminava.** Esperar por um handle
    que este pool nunca emitiu bloqueava para sempre. A mesma checagem que
    distingue "ainda rodando" de "sumiu" resolve esse caso — conserto que caiu
    do desenho, não que foi procurado.
  - **Descartes são contados.** `JobMetrics::forgotten_results` é total corrido.
    Contagem e não flag: a pergunta útil não é *se* o pool descarta, é a que
    velocidade.

  **E um segundo vazamento, apagado em vez de limitado.** O `State` também
  carregava `cancelled: HashSet<JobHandle>`, escrito pelo `cancel` e **lido por
  nada** — o cancelamento de verdade é a flag atômica do `CancellationToken`,
  setada na linha seguinte. Vazava uma entrada por job cancelado sem efeito
  nenhum. Conjunto que ninguém lê não é estado; foi removido.

  O teto é 65.536, escolhido contra a maior onda que o motor submete e não por
  ser redondo: geração de chunks num raio de interesse 12 são 625 colunas.
  **Nada no motor hoje enxerga o teto** — o slice submete 25 e coleta 25 —, e é
  por isso que os testes empurram dois tetos inteiros por um pool em vez de
  confiar em algum caminho existente exercitá-lo.
- **STATUS:** **CLOSED**

### DEBT-0041 — O loop de quadro existe, mas nenhum processo roda quadros contra um relógio

- **SYSTEM:** `engine/runtime::frame`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** o ENGINE-0 do `CORE.md` §16 foi construído
  ([ADR-0017](docs/adr/ADR-0017-a-frame-is-time-the-host-hands-in.md)) com o
  tempo entrando como argumento, de propósito: um loop que lê o relógio não é
  reproduzível. A consequência é que **alguém tem de entregar o tempo**, e o
  único chamador hoje é a caminhada do slice, que entrega um delta roteirizado
  de exatamente um passo. Nenhum processo deste repositório roda quadros contra
  um relógio real.
- **IMPACT:** três coisas que o loop mede nunca chegam a acontecer. `discarded`
  é sempre zero, porque um delta de um passo não pode atrasar. A classificação é
  sempre `Target`, porque um quadro roteirizado não estoura orçamento. E
  `unattributed` — o número que o módulo inteiro existe para produzir — nunca é
  observado num quadro de verdade, só nos testes e no benchmark. Um mecanismo de
  contabilidade que nunca contabilizou uma carga real não foi exercitado, foi
  compilado.
- **RISK:** médio. Não há sintoma hoje, porque não há quadro a perder; o risco é
  o de sempre com mecanismo não exercitado — descobrir que a contabilidade está
  errada no dia em que ela for a única coisa a explicar um travamento.
- **PROPOSED REMEDIATION:** um host que rode quadros num laço, entregando o
  tempo real medido entre eles, e um estágio do slice ou do benchmark que use
  esse host. Pode ser headless: não precisa de janela nenhuma para existir um
  laço com relógio. O que ele precisa é de um critério de parada que não dependa
  do relógio, ou o slice deixa de ser determinístico — o caminho provável é o
  benchmark, que já é medição e não prova.
- **TRIGGER:** já disparado, no sentido de que o mecanismo existe sem carga
  real. A ordem, porém, é depois do `DEBT-0018` e do `DEBT-0027`: um host que
  rode quadros enquanto a geração e o meshing ainda moram na thread do tick
  mede o atraso deles, não o loop.
- **NOTE (2026-09-25):** o host de janela existe
  ([ADR-0027](docs/adr/ADR-0027-the-window-host-is-winit-and-it-owns-the-event-loop.md)):
  `nexora_window::Client::frame` é chamado uma vez por redesenho, e é o lugar
  natural do relógio num cliente. Nada o liga ao `runtime::frame` ainda; o
  caminho headless descrito acima continua válido.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN

### DEBT-0042 — A resolução de input varre todos os bindings a cada quadro, e 70% disso é procurar o contexto

- **SYSTEM:** `engine/runtime::input` (`InputSystem::resolve`)
- **CLASS:** PERFORMANCE
- **WHY CREATED:** `resolve` percorre `self.bindings` inteiro uma vez por quadro
  e, para cada binding, procura a prioridade do contexto num
  `BTreeMap<Identifier, i32>` — ou seja, comparação de string por nível da
  árvore, 41 vezes por quadro no keymap medido. Roda mesmo quando ninguém
  apertou nada: um quadro parado paga o mesmo que um quadro de combate.
- **IMPACT:** medido, com o par que existe para isso
  (`docs/benchmarks/PHASE-0-BASELINE.md`, achado 26). Três leituras de cada
  lado, com `frame.schedule_advance` como controle que não pode se mexer — e não
  se mexeu (39–42 ns dos dois lados):

  | `input.sample_idle`, 41 bindings | leitura 1 | 2 | 3 |
  | --- | ---: | ---: | ---: |
  | como está | 322 ns | 342 ns | 356 ns |
  | sem a busca do contexto | 102 ns | 102 ns | 99 ns |

  **Cerca de 70% do custo de um quadro parado é descobrir de que contexto cada
  binding é**, ~5,4 ns por binding. O resto da varredura é o que sobra.

  A segunda dimensão nunca foi medida: `button_held` é uma varredura do conjunto
  de teclas seguradas *dentro* do laço de bindings, então o custo real é
  O(bindings × seguradas), e todas as medições têm no máximo uma tecla embaixo.
  Um keymap cheio de cordas com quatro modificadores segurados é um caso sobre o
  qual este registro não tem número nenhum.
- **RISK:** baixo hoje, e é importante dizer por quê em vez de deixar o número
  assustar: 322 ns são 0,00064% do orçamento TARGET de 50 ms de um quadro a
  20 Hz, e ~11% de um tick de streaming parado (3,0 µs, medido nas mesmas
  execuções). O risco não é o número atual, é a inclinação — ele cresce com o
  tamanho do keymap, e keymap cresce.
- **PROPOSED REMEDIATION:** um índice de contexto → bindings, mantido em `bind`,
  `unbind` e `unbind_context`, para que a busca aconteça uma vez por contexto
  ativo (1 a 3) em vez de uma vez por binding (dezenas). A varredura de teclas
  seguradas pede a mesma forma de conserto: um índice por `Source`. Nenhum dos
  dois é difícil; os dois são estrutura nova para manter, e é por isso que este
  é um registro e não um commit.
- **TRIGGER:** o keymap embarcado passar de ~150 bindings — 3,6× o medido, o que
  põe o quadro parado perto de 1,2 µs — **ou** o input aparecer com participação
  não trivial num relatório de quadro de carga real, o que depende do
  `DEBT-0041`. Antes disso, indexar seria exatamente o que o `DEBT-0010` provou
  que não se deve fazer sem número: lá a varredura custava 979,7 µs e valia o
  índice; aqui custa 322 ns e não vale.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN

### DEBT-0043 — Nenhum dispositivo real jamais produziu um sinal de input

- **SYSTEM:** `engine/runtime::input`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** o ENGINE-8 do `CORE.md` §24 e o `INPUT SYSTEM.md` foram
  construídos ([ADR-0018](docs/adr/ADR-0018-input-is-intent-the-host-hands-in.md))
  com o sinal entrando como argumento, pelo mesmo motivo do `DEBT-0041`: um
  sistema que abre o dispositivo sozinho não se reproduz. E, pelo mesmo motivo,
  **alguém tem de entregar o sinal** — e o único chamador hoje é o jogador
  roteirizado do slice, que aperta duas teclas de um teclado que não existe.
- **IMPACT:** o que nunca aconteceu, listado para não ser confundido com o que
  funciona: nenhum teclado, mouse, gamepad ou tela de toque jamais entregou um
  sinal; `DeviceKind::Mouse` e `DeviceKind::Touch` não têm um único chamador
  fora dos testes; nenhum arquivo de remap foi gravado em disco, só codificado e
  decodificado em memória; e `validate_remote` nunca examinou um snapshot que
  tivesse atravessado uma rede, porque não há rede. O que está exercitado é a
  lógica; o que não está é a borda.
- **RISK:** médio. A parte que costuma dar errado numa camada de input é
  justamente a borda — que scancode o sistema operacional manda, o que ele faz
  com repetição de tecla, se a desconexão chega como evento ou como silêncio. O
  módulo tem uma resposta declarada para cada uma dessas e nenhuma foi
  confrontada com um driver.
- **PROPOSED REMEDIATION:** um host que traduza eventos de dispositivo do
  sistema operacional em `Signal` e chame `sample` uma vez por quadro. É o mesmo
  host que o `DEBT-0041` pede, e provavelmente é um só: quem tem o relógio tem
  os dispositivos. Enquanto ele não existir, o ganho barato é gravar e ler o
  arquivo de remap de verdade, que não precisa de driver nenhum.
- **TRIGGER:** existir qualquer processo com janela ou com laço de eventos do
  sistema operacional. Depende do `DEBT-0041` pela mesma razão que ele depende
  do `DEBT-0018` e do `DEBT-0027`: medir ou exercitar a borda antes de haver
  host é medir o roteiro. **Disparado em 2026-09-25**
  ([ADR-0027](docs/adr/ADR-0027-the-window-host-is-winit-and-it-owns-the-event-loop.md)):
  o `nexora-window` roda o laço de eventos do sistema operacional, e o `winit`
  já entrega a ele eventos de teclado e mouse, que o host hoje descarta. A
  tradução para `Signal` é o próximo passo desta dívida, não deste host.
- **TARGET STAGE:** Phase 2
- **STATUS:** OPEN

### DEBT-0046 — O RHI ainda não apresenta nada, e nenhuma GPU real o executou

- **SYSTEM:** `engine/rhi`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** o contrato do RHI foi construído antes de qualquer backend
  nativo, de propósito
  ([ADR-0025](docs/adr/ADR-0025-the-rhi-is-a-contract-a-null-backend-keeps-before-a-gpu-does.md)):
  se o primeiro backend vier antes do contrato, ele *vira* o contrato, e uma
  fronteira definida pelo único backend que tem não pode falhar no contato com
  ele. O backend nulo cumpre todas as regras sem GPU, e a suíte de conformidade
  roda contra ele em todo slice.
- **IMPACT:** o que nunca aconteceu, listado para não ser confundido com o que
  funciona: nenhum buffer ou textura foi alocado num dispositivo real; nenhum
  fence foi sinalizado por um driver; nenhum shader foi compilado — o código de
  shader é bytes opacos e `validates_shaders` é falso; nenhuma perda de
  dispositivo veio de um driver, só de `lose_device`; nada foi apresentado. As
  regras são consistentes entre si e com a suíte; nenhuma foi confrontada com
  Vulkan, Direct3D 12 ou Metal.
- **RISK:** alto para o gate de congelamento, que depende exatamente disso (o
  segundo bloqueio do *Final gate*). Baixo para o que já existe: nada
  persistido ou na rede depende do RHI, então uma regra que um driver real
  desminta muda o contrato sem migração.
- **PROPOSED REMEDIATION:** a ADR do primeiro backend nativo, decidindo o que a
  ADR-0025 enumera — a dependência (justificada como a ADR-0002 pede), onde o
  `unsafe` mora (o precedente é a ADR-0009), a linguagem de shader, e o host da
  janela, que é o mesmo que o `DEBT-0041` e o `DEBT-0043` esperam. Depois, um
  check novo em `scripts/local-validation.py` que roda
  `conformance::run` contra esse backend na máquina real: só esse relatório pode
  tirar `rhi`, `gpu_context` e `texture_upload` de `NOT_IMPLEMENTED`.
- **TRIGGER:** já disparado. A máquina existe: os relatórios locais vêm de um
  Windows 10 com uma AMD Radeon RX 6650 XT, que tem drivers Vulkan e
  Direct3D 12.
- **TARGET STAGE:** Phase 0 (bloqueia o congelamento)
- **PROGRESS (2026-09-25, ADR-0026):** o backend nativo existe:
  `engine/rhi-wgpu`, `wgpu` 30 sobre Vulkan / Direct3D 12 / Metal, com código
  seguro, WGSL validado pelo naga e as regras compartilhadas com o backend nulo
  (`rhi::kit`). Ele passa nos nove casos de conformidade e envia uma textura
  16×16 e desenha um triângulo, lendo os dois de volta byte a byte, no
  **lavapipe** (Vulkan por software da Mesa) do container e do CI. Duas
  regras que só um driver teria pego foram para as regras compartilhadas.
  Continua aberto, e só isto:
  - ~~**apresentação**~~ — **construída** (2026-09-25,
    [ADR-0027](docs/adr/ADR-0027-the-window-host-is-winit-and-it-owns-the-event-loop.md)):
    `engine/window` abre uma janela com o `winit`, o `WgpuRhi` abre a surface
    nela, e `present` desenha o alvo na imagem da swapchain e a apresenta. No
    CI, sob Xvfb e lavapipe, o primeiro quadro é lido de volta da surface e
    bate nos 65.536 texels; a conformidade roda com `presents: true`;
  - ~~**uma GPU real**~~ — **verificada** (2026-09-26, relatórios locais 4 e
    5): as checagens `rhi_native` e `window` de `local-validation.py`
    passaram duas vezes no Windows 10 do operador, numa **AMD Radeon RX 6650
    XT** (Vulkan, `discretegpu`), nos commits `786f77f` e `8331c15`, cujo
    código é o mesmo do `HEAD` (`check`: `CURRENT_NO_RELEVANT_CHANGE`).
    Onze de onze casos de conformidade, sem e com apresentação; upload e
    desenho lidos de volta; a prova da ADR-0028 (256 de 256 texels
    amostrados, mantidos pela profundidade e tingidos pelo uniform); uma
    janela Win32 com surface `Bgra8UnormSrgb` FIFO, 0 redraws de espera, e o
    primeiro quadro lido da surface batendo nos 65.536 texels;
  - ~~**regra provisória de vértice**~~ — **resolvida** (2026-09-26,
    [ADR-0028](docs/adr/ADR-0028-a-draw-names-its-vertex-layout-its-bindings-and-its-depth.md)):
    o pipeline declara seus atributos de vértice, os slots de binding
    (uniform, textura, sampler) e o teste de profundidade; a regra do stride
    saiu do backend.
- **RESOLUTION (2026-09-26):** as três pendências fecharam. O que o título
  dizia que nunca tinha acontecido aconteceu, e está lido de volta: buffers e
  texturas alocados num dispositivo real, fences sinalizados por um driver
  real, WGSL traduzido pelo naga para SPIR-V e compilado pelo driver da AMD,
  um quadro apresentado num desktop. Os testes que provam isso são os mesmos do CI (`cargo test -p
  nexora-rhi-wgpu`, `-p nexora-window`) e os probes que a validação local roda
  (`nexora-rhi-probe`, `nexora-window-probe`). O que **não** está provado, para
  não ser lido como provado: perda de dispositivo vinda de um driver (só de
  `lose_device`), Direct3D 12 e Metal em hardware (só WARP e o dispositivo
  paravirtual do macOS), e qualquer outra GPU. Nada disso é débito deste
  item: são verificações de um renderer que ainda não existe.
- **STATUS:** CLOSED (2026-09-26)

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
- **PARCIALMENTE ENDEREÇADO** — a remediação proposta foi construída e medida
  (`docs/benchmarks/PHASE-0-BASELINE.md`, achado 10b). Duas correções ao
  diagnóstico acima:

  1. **São dois mapas ordenados no caminho, não um.** `World::get_block` desce
     `World::chunks` pela coluna, e depois `Chunk::get` desce `Chunk::sections`
     pela seção. A entrada original contou o primeiro e leu o segundo como
     parte da "leitura da seção paletizada".
  2. **As descidas eram cerca de um terço do lookup, não o grosso dele.**

  O que foi feito: `WorldVoxels` mantém a última **seção** resolvida — uma
  entrada, não um mapa — o que remove as duas descidas de toda pergunta
  repetida, e repetida é quase toda pergunta (um corpo tem ~0,6 × 1,8 × 0,6
  contra seções de 32³). Junto veio `ChunkShape::split_of`, que devolve
  endereço de seção e offset local de uma passagem só, porque `section_of`
  pega o quociente e `local_of` o resto das mesmas três divisões.

  | | antes | depois | |
  | --- | ---: | ---: | ---: |
  | `physics.raycast_40m` | 1,90 µs | **1,14 µs** | −40,0% |
  | `physics.depenetration_check` | 123,6 ns | **78,5 ns** | −36,5% |
  | `physics.character_step` | 743,2 ns | **545,4 ns** | −26,6% |
  | `physics.box_sweep` | 289,1 ns | **219,0 ns** | −24,3% |
  | `physics.thousand_bodies_step` | 209,8 µs | **174,3 µs** | −16,9% |
  | — só o lookup | 102,7 µs | **67,2 µs** | **−34,6%** |
  | `physics.thousand_bodies_step_flat` | 107,1 µs | 107,1 µs | — |

  A última linha é o controle: o fixture plano não passa por `WorldVoxels` e
  não se moveu. Sem ela, "a máquina ficou mais rápida" explicaria o resto.

- **SEGUNDA PARCELA ENDEREÇADA (2026-09-14)** — a leitura paletizada, medida em
  máquina diferente e mais barulhenta (achado 10e; nada abaixo é comparável com
  a tabela acima). As células empacotam `64 / bits` por palavra, e `64 / 12` é
  cinco: tanto a palavra quanto o offset dentro dela saíam de dividir por um
  divisor que nenhum compilador enxergava. As doze larguras possíveis são
  conhecidas em tempo de compilação, então cada uma virou entrada de tabela — o
  quociente é multiplicação e shift contra `ceil(2^32 / per_word)`, e o resto cai
  do quociente. A identidade é exata para todo índice que uma seção pode ter
  (`MAX_SECTION_EXTENT³ = 2^24`, e o limite de erro `(N + d - 1) · e < 2^32` sobra
  duas ordens de grandeza), e um teste percorre o último índice de cada palavra
  no topo da faixa — que é onde um recíproco aproximado quebra primeiro.

  | | antes | depois | |
  | --- | ---: | ---: | ---: |
  | `voxel.get_paletted` | 13,1 ns | **11,4 ns** | −13% |
  | `physics.voxel_lookup` | 28,3 ns | **25,9 ns** | −8,5% |
  | `voxel.get_uniform` | 5,2 ns | 5,3 ns | — |
  | `spatial.index_of` | 6,3 ns | 6,3 ns | — |

  As duas últimas são os controles e não se moveram: armazenamento uniforme
  nunca chama a leitura empacotada, e `index_of` não foi tocado.

  **CORREÇÃO (2026-09-14, um commit depois):** os −13% são precisos demais para
  o que a medida aguenta. O commit seguinte não tocou `engine/world` e mesmo
  assim `voxel.get_paletted` leu **7,8–8,2 ns**, com os mesmos dois controles
  parados de novo. Sob `lto = "thin"` e `codegen-units = 1`, religar o binário
  realoca `Section::get`, e a ~10 ns isso vale dezenas de por cento. O que a
  medida sustenta é **a direção e a ordem de grandeza — a divisão saiu e a
  leitura ficou entre ~13% e ~40% mais rápida neste box** — não um único número.
  Registrado como DEBT-0039.

  **13% é menos do que uma divisão custa, e é esse o achado.** O palpite era que
  duas divisões inteiras fossem o grosso de uma leitura de 13 ns. Valem 1,7 ns.
  A razão provável é que nunca foram duas: o `div` do x86-64 devolve quociente e
  resto da mesma instrução, então o compilador já tinha fundido o par. Isso é
  raciocínio, não medida — medido foram os 1,7 ns.
- **PROPOSED REMEDIATION (o que resta):** a aritmética de endereço. `split_of`
  abre todo lookup com três divisões pelas extensões da seção, que são valores
  de execução, e tem a mesma forma do que acabou de ser resolvido: um divisor
  fixo pela vida de um mundo que o compilador não vê. Ao contrário da largura de
  palete, não sai de doze valores, então tabela não fecha — o caminho é o mundo
  carregar o recíproco junto com o `ChunkShape`.
- **TRIGGER:** física passar de ~10% do orçamento de simulação, ou população
  acordada estável acima de 1.000.
- **TARGET STAGE:** Phase 4 (World Runtime) ou antes, se o gatilho ocorrer
- **STATUS:** OPEN (medido) — a parcela do lookup num passo de física está em
  **18–27%** pela régua refeita do DEBT-0037 (`1 000 leituras × 25,9 ns`), contra
  os 49% de origem. O 38,6% do achado 10b foi medido enquanto a varredura de
  depenetração ainda rodava e um corpo assentado fazia duas leituras por passo em
  vez de uma; não é contradição, é a metade que o DEBT-0012 levou.

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
- **RESOLUÇÃO:** a segunda remediação proposta, numa forma mais forte. Um corpo
  guarda o *par* (revisão da fonte, span de células) que foi **provado** livre
  de sólidos. A varredura é pulada só quando os dois batem. Detalhes em
  `docs/benchmarks/PHASE-0-BASELINE.md`, achado 10c.

  Três decisões que fazem isso ser seguro em vez de rápido-e-quebrado:

  1. **`VoxelSource::revision()` tem default `None`**, que significa "assuma que
     mudou". Uma fonte que não opta por participar se comporta exatamente como
     antes — verifica tudo, todo passo. Errar aqui exige implementar o método
     deliberadamente e implementá-lo errado.
  2. **A chave é o span de células, não a posição.** Um corpo cujo `center` é
     escrito de fora do solver continua dentro de células já provadas livres,
     desde que sejam as mesmas células. No instante em que não são, o span
     deixa de bater e a varredura roda. É por isso que isso é sólido sem
     precisar tornar `center` privado.
  3. **O mundo conta com folga.** `chunk_mut` incrementa a revisão ao entregar
     o empréstimo, não numa escrita que ele não consegue observar. Contar demais
     custa uma otimização perdida; contar de menos custa um corpo dentro de um
     bloco.

  | | antes | depois | |
  | --- | ---: | ---: | ---: |
  | `physics.thousand_bodies_step` | 137,3 µs | **84,6 µs** | −38,4% |
  | `physics.character_step` | 386,7 ns | **343,6 ns** | −11,1% |
  | `physics.thousand_bodies_step_flat` (controle) | 82,8 µs | 82,2 µs | −0,7% |
  | `physics.depenetration_check` (controle) | 56,1 ns | 55,2 ns | −1,6% |

  E o número que não depende da máquina: um corpo assentado faz **duas** leituras
  de mundo por passo, uma delas sendo essa verificação. Passa a fazer **uma**.
  Medido por contagem e assertado como igualdade exata (10 contra 20 células em
  dez passos), porque teste de tempo prova numa máquina e nada em outra.

- **NOTA (medida):** a `NOTA` acima continua valendo e agora tem teste próprio.
  `a_block_placed_inside_a_resting_body_still_ejects_it` põe um bloco onde um
  corpo já está e exige que ele saia — é o defeito original, e o skip não pode
  sobreviver ao mundo mudar. Um segundo achado saiu daí: **um corpo dormindo
  nunca roda a verificação de todo modo**, porque `step_once` pula corpos
  inativos. O custo desta dívida sempre foi só dos corpos acordados.
- **CUSTO:** fechar esta quebrou a medição do DEBT-0011. O par terreno/plano
  isolava o lookup porque as duas linhas diferiam em uma coisa; agora diferem em
  duas, já que só a de terreno pula a varredura. Registrado como **DEBT-0037**.
- **STATUS:** **CLOSED** — a varredura deixou de ser paga por todo corpo, todo
  passo. Um corpo que troca de células ainda paga, e isso não é dívida: são
  células novas e alguém tem de olhar para elas.

### DEBT-0037 — O par terreno/plano parou de isolar o lookup de voxel

- **SYSTEM:** `engine/benchmark::suites::physics`
- **CLASS:** MEASUREMENT
- **WHY CREATED:** `physics.thousand_bodies_step` e
  `physics.thousand_bodies_step_flat` mediam o custo do lookup de voxel porque
  diferiam em exatamente uma coisa: de onde vinha o terreno. O DEBT-0012 fez a
  linha de terreno pular a varredura de depenetração e a plana não, porque
  `FlatGround` não declara revisão. As duas linhas agora diferem em duas coisas,
  e a subtração entre elas não mede mais nada em particular.
- **IMPACT:** o número do achado 10b — **38,6% de um passo de física é o
  lookup** — não pode ser re-derivado. Não é que esteja errado; é que a régua
  que o produziu deixou de existir.
- **RISK:** baixo, e inteiramente sobre saber onde está o tempo. Nenhum
  comportamento depende disso.
- **PROPOSED REMEDIATION:** um fixture plano que declare uma revisão constante,
  para que as duas linhas voltem a diferir só na fonte do terreno. **Não** dar
  isso ao `FlatGround` da biblioteca: ele é construído em linha, e dois com
  pisos diferentes reportariam a mesma constante — que é exatamente o único
  jeito de uma revisão mentir. O fixture pertence ao benchmark.
- **TRIGGER:** a próxima vez que alguém precisar da parcela do lookup, ou antes
  de tentar mexer na leitura paletizada (que é o que sobrou do DEBT-0011).
- **TARGET STAGE:** Phase 4
- **RESOLUÇÃO (2026-09-14):** o fixture existe — `StillFloor`, no benchmark, com
  revisão derivada do plano que descreve, não constante da biblioteca. **E não
  bastou.** Medido em A/B: a linha plana lê 145,34 µs sem revisão e 142,79 µs
  com ela, 1,8% contra um espalhamento de 10–23%. A subtração não tinha
  resolução para o que restou — depois do DEBT-0012 o lookup é um quinto de um
  passo, escondido dentro de dois números de 140 µs.

  A régua foi refeita como **produto, não diferença**, com as duas metades
  medíveis em separado:

  | | valor | espalhamento |
  | --- | ---: | ---: |
  | `physics.world_reads_per_step` | **1 000** | — |
  | `physics.voxel_lookup` | **25,9 ns** | 5,8% |

  `1 000 × 25,9 ns` de um passo de `141,16 µs` = **18%**. A primeira linha é uma
  **contagem** — uma pergunta por corpo assentado por passo, o mesmo inteiro em
  qualquer máquina, e confirmação independente de que o atalho do DEBT-0012 está
  vivo. A segunda é um microbenchmark de 26 ns em vez de um de 140 µs, que é o
  ponto: é a única metade que precisa de máquina quieta, e re-medi-la é barato.

  **Quanto de quieta importa.** Uma segunda execução do mesmo binário leu
  `physics.voxel_lookup` em 38,2 ns com 26% de espalhamento, o que põe a parcela
  em 27% em vez de 18%. A contagem não mexeu um dígito. Fica registrado como
  faixa, **18–27%**, porque escolher a execução que lê melhor é como uma medida
  vira propaganda.
- **STATUS:** **CLOSED** — a parcela do lookup voltou a ser derivável, e por um
  caminho que não depende de duas medidas grandes se cancelarem. O que sobrou de
  imprecisão está na metade que é um relógio, e essa está isolada e é barata de
  repetir.

### DEBT-0038 — A prova de que um corpo está livre não diz qual fonte a produziu

- **SYSTEM:** `engine/physics::world`, `engine/physics::body`
- **CLASS:** CORRECTNESS
- **WHY CREATED:** o DEBT-0012 guarda em cada corpo `(revisão, span de células)`
  e pula a varredura quando a revisão da fonte bate com a guardada. A revisão é
  um `u64` sem dono: nada liga a prova à fonte que a fez. Duas fontes diferentes
  numerando a partir do zero — `World` conta suas edições a partir de 0 — podem
  emitir o mesmo valor para mundos diferentes.
- **IMPACT:** um `PhysicsWorld` alternado entre duas fontes que declarem revisão
  pode aceitar uma prova feita contra a outra e deixar de ejetar um corpo que
  está dentro de um bloco. Nada no motor faz isso hoje: o slice e a simulação
  seguram uma fonte só. É uma brecha estrutural, não um defeito observado.
- **RISK:** baixo hoje, e cresce sozinho — a segunda fonte com revisão foi
  criada nesta mesma sessão (`StillFloor`, no benchmark), e o único motivo de
  não ser um problema é que o seu valor tem o bit alto ligado, deliberadamente,
  para não encostar no contador do `World`. Manter dois espaços de numeração
  separados por convenção é exatamente o tipo de correção que depende de alguém
  lembrar, e que este repositório recusa em outros lugares.
- **PROPOSED REMEDIATION:** a prova carregar identidade além de contador. O
  caminho barato é o `PhysicsWorld` guardar de qual fonte veio o último passo e
  descartar toda prova quando ela muda; o caminho caro é a fonte devolver um par
  (identidade, revisão). O barato resolve o caso real e não pede nada de quem
  implementa `VoxelSource`.
- **TRIGGER:** a segunda fonte com revisão a ser usada em produção, ou qualquer
  código que passe fontes diferentes ao mesmo `PhysicsWorld`.
- **TARGET STAGE:** Phase 1
- **RESOLUÇÃO (2026-09-14):** nem o caminho barato nem o caro — um terceiro, que
  não compara nada.

  **Comparar endereços não funciona, e isso foi verificado antes de descartar.**
  Uma fonte construída para um passo e uma fonte *diferente* construída do mesmo
  jeito para o próximo ficam no mesmo endereço; o teste
  `two_sources_at_one_address_do_not_share_a_proof` garante isso em vez de
  torcer, porque usa a mesma variável, e as duas ainda declaram revisão 1.

  O que existe agora é `PhysicsWorld::against(&source) -> Stepper`, uma
  **sessão**. O `Stepper` toma `&'s S` emprestado enquanto a prova puder ser
  consultada, então todo substep que ele roda é respondido pelo **mesmo objeto
  vivo** — quem diz isso é o borrow checker, não uma comparação e não uma
  convenção. É o mesmo argumento que o `WorldVoxels` usa para o cache de seção.
  Abrir uma sessão cunha um número nunca usado; a prova guarda esse número, e
  uma prova de sessão anterior só pode falhar em casar.

  `step_once` e `advance` continuam existindo e abrem uma sessão só para a
  chamada: **corretos e nunca pulando**. O padrão seguro é o que não exige saber
  de nada; manter a otimização é que passou a exigir manter a sessão — o slice
  headless e o benchmark seguram uma.

  | dez passos assentados | células perguntadas |
  | --- | ---: |
  | dentro de uma sessão | **10** |
  | uma sessão por passo | **20** |

  O teste que fecha a brecha **falha sem a correção**: removida a condição
  `proof.session == session`, ele acusa exatamente "the check was skipped on the
  strength of a proof the old source made".
- **STATUS:** **CLOSED** — a prova deixou de valer por convenção de numeração. O
  `StillFloor` ainda liga o bit alto, mas agora isso é higiene, não a linha de
  defesa.

### DEBT-0039 — Microbenchmark de poucos nanossegundos não é comparável entre builds

- **SYSTEM:** `engine/benchmark`
- **CLASS:** MEASUREMENT
- **WHY CREATED:** `voxel.get_paletted` leu **11,4 ns** no commit `14086ec` e
  **7,8–8,2 ns** no commit seguinte, que **não tocou uma linha de
  `engine/world`** (verificado com `git diff --name-only`). O perfil de release
  usa `lto = "thin"` e `codegen-units = 1`, então mudar qualquer crate do
  workspace religa o binário inteiro e realoca `Section::get`. A ~10 ns,
  alinhamento e decisões de inline valem dezenas de por cento.
- **IMPACT:** o método em vigor — mover uma coisa, conferir que os controles não
  se moveram — é **necessário e insuficiente** quando a própria mudança religa o
  binário. Os controles (`spatial.index_of`, `voxel.get_uniform`) ficaram
  parados nas duas medições e mesmo assim `get_paletted` andou 30%. Nenhum
  número abaixo de ~20 ns publicado aqui deve ser lido como preciso melhor que
  uma faixa.
- **RISK:** baixo para o produto, alto para a tomada de decisão: é assim que uma
  otimização inexistente ganha crédito, e é o erro que o próprio Apêndice B
  existe para impedir.
- **PROPOSED REMEDIATION:** medir *n* builds do mesmo código-fonte, não uma, e
  publicar a faixa entre builds junto com o espalhamento dentro de uma. Um
  `--repeat-build` no runner não resolve — a variação é do link, não da
  execução. O caminho é o script de release construir duas vezes com uma
  mudança neutra no meio e reportar as duas.
- **TRIGGER:** a próxima vez que alguém quiser publicar um ganho abaixo de
  ~20 ns, ou antes de mexer em `spatial.index_of` (o que sobrou do DEBT-0011).
- **TARGET STAGE:** Phase 4
- **RESOLUÇÃO (2026-09-20):** o gatilho disparou pela minha própria mão — o ciclo
  do DEBT-0010 publicou 4,94 ns contra 1,54 ns. A ferramenta que a remediação
  pediu existe: **`scripts/build-spread.sh`** constrói *n* vezes a partir do
  **mesmo fonte**, separando cada build por um comentário neutro em
  `engine/benchmark/src/lib.rs` — crate que não contém nenhum kernel medido e é
  religado no mesmo binário, que é exatamente a forma da mudança que gerou esta
  entrada. Achado 24 do `PHASE-0-BASELINE.md`.

  **E a resposta não é a que a entrada previa.** Em três builds do mesmo fonte:

  | as seis piores linhas da suíte | faixa |
  | --- | ---: |
  | `journal.append_durable` | **36,3%** |
  | `physics.thousand_bodies_step_flat` | **29,9%** |
  | `save.region_write_one_dirty` | **23,8%** |
  | `jobs.batch_1000_barrier_submit_all` | **23,6%** |
  | `journal.append_batched_sync` | **21,3%** |
  | `jobs.submit_wait_roundtrip` | **18,3%** |

  Todas em µs ou ms, e **todas passam por `fsync`, disco ou escalonamento de
  thread**. As linhas mais estáveis da suíte inteira são os kernels aritméticos
  de dois nanossegundos: `ffi.scalar_inlined` e `ffi.scalar_opaque_rust` não
  moveram **um dígito**. O `voxel.get_paletted`, a linha que dá nome a esta
  entrada, moveu **5,7%** (5,30–5,60 ns).

  **Grandeza não prevê instabilidade; o que a linha toca, sim.** A regra que a
  entrada propôs — "nada abaixo de ~20 ns é preciso" — aponta para as linhas
  erradas, e deixa passar linhas de 36% três ordens de grandeza acima.

  **O piso cresce com o número de builds** (2 builds: 5,0% / 17,2%; 3 builds:
  10,2% / 33,1%; 3 builds de novo: 13,0% / 36,3%), que é a forma honesta disso —
  faixa é limite inferior, e amostrar mais acha mais.

  **O que isto NÃO explica:** os 11,4 → 7,8 ns originais. Fonte idêntico é um
  experimento diferente de dois commits diferentes — o commit em questão mudou
  código de verdade, e o LTO fino pode inlinar diferente por causa disso, não só
  realocar. Além disso a caixa mudou: `get_paletted` lê 5,3–5,6 ns aqui contra
  11,4 e 7,8–8,2 lá, então o par original não é mais reexecutável daqui. A
  observação continua válida como registrada; o que ela **inferiu** é que está
  contrariado.
- **REGRA EM VIGOR:** antes de publicar um ganho, rodar `scripts/build-spread.sh`
  e comparar com a faixa **daquela linha**. Ganho menor que o piso de build da
  própria linha não é achado, seja qual for a grandeza. Fora do CI de propósito:
  três builds de release do workspace são minutos de compute para um número que
  só importa quando alguém vai publicar comparação.
- **STATUS:** **CLOSED** — a ferramenta existe, o piso está medido e publicado, e
  a regra é verificável em vez de ser um limiar escolhido a olho.

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
- **RESOLUTION (2026-09-25):** a segunda máquina chegou pela ponte de validação
  local: um Ryzen 5 5500 com Windows 10, no relatório 3. Todas as linhas de
  física ficaram 1,27–1,58× mais lentas no container, na mesma ordem: a
  diferença entre as máquinas é um fator, não uma forma
  (`docs/benchmarks/PHASE-0-BASELINE.md`, Apêndice I, achado 27). O orçamento
  publicado é `nexora_physics::budget::CROWD_SUBSTEP`, para um substep de 1.000
  corpos acordados:
  - **TARGET 250 µs** — o pior p95 medido (168 µs) mais metade;
  - **WARNING 500 µs**;
  - **CRITICAL 1 ms**;
  - **EMERGENCY 2 ms** — oito substeps de recuperação de 2 ms enchem um quadro
    de 60 Hz, e a partir daí a física sozinha não deixa o quadro se recuperar.

  Os testes do módulo prendem essa derivação às constantes de onde ela vem. O
  benchmark classifica a medição contra o orçamento em toda execução
  (**Published budgets**), e o CI confere que a linha existe. A classe não é
  cobrada no CI, porque tempo, ao contrário de memória, depende do runner. Nas
  duas máquinas, mediana e p95 caem em `target`. O mesmo achado mostra por que
  **I/O e o job system não podem publicar** a partir destas duas máquinas: o
  disco é ~3× mais lento no Windows, e acordar thread é 2–4× mais lento no
  container.
- **TARGET STAGE:** Phase 4
- **STATUS:** CLOSED (2026-09-25)

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
- **STATUS:** OPEN (medido) — **o gatilho disparou em 2026-09-20.** O loop de
  quadro existe (`engine/runtime/src/frame.rs`, ENGINE-0, ADR-0017) e a própria
  caminhada do slice roda dentro dele. Isso **não** conserta nada aqui: a
  ativação continua síncrona na thread do tick, e o que o loop acrescenta é só
  que agora há um lugar onde os 21,8 ms aparecem como um quadro classificado
  `EMERGENCY` em vez de uma frase neste registro. O que falta continua sendo o
  descrito acima — submeter a geração como job e concluir a ativação num tick
  posterior — mais a pergunta que o loop torna respondível e que ainda não foi
  medida: **quantas ativações cabem num passo de 50 ms**.

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
- **RESOLUÇÃO (2026-09-15):** `RetainedChunks::backed_by(RegionStore)` existe, e
  faz o que esta entrada pediu — com uma diferença deliberada em *quando*.

  **A memória: 20,0 KiB → 0 B por coluna despejada.** `RegionStore::store_columns`
  funde as colunas nos arquivos de região sem derrubar as que já estavam lá, e
  `RegionStore::read_column` traz uma de volta. Medido:

  | | |
  | --- | ---: |
  | `streaming.retained_bytes_per_chunk` (controle, não mudou) | **20,0 KiB** |
  | `streaming.retained_bytes_after_flush` | **0 B** |
  | `streaming.chunk_retained_cycle` (memória) | 184,7 ns |
  | `streaming.chunk_flushed_cycle` (disco) | **3,08 ms** |
  | `streaming.region_writes_per_eight_columns` | **4** |

  **O preço é 16.700×, e é por isso que a decisão não é "sempre disco".** O que
  torna um flush por tick viável não é o número de nanossegundos: é que o custo
  é o **arquivo**, não a coluna. Oito colunas que caem em quatro regiões são
  quatro escritas, não oito — uma contagem, igual em qualquer máquina.

  **Despejar não escreve; o flush escreve.** O `persist` do backend continua
  entregando o chunk para a memória, porque o despejo roda dentro do orçamento
  de streaming e escrita de arquivo não cabe ali. `flush_to_store` é chamado no
  fim do tick, escreve o que aquele tick despejou e solta. O limite da memória
  passa a ser **o que foi despejado desde o último flush**, não tudo o que já
  foi editado.

  **`flush_into` continua significando "todo chunk retido".** Com um store
  anexado ele também lê de volta o que foi para o disco — senão um flush
  esvaziaria em silêncio justamente o conjunto que aquela chamada olha, e o save
  em contêiner sairia sem as edições, que é exatamente a armadilha que o módulo
  existe para fechar. Quem salva **pelo store** não chama `flush_into`: aquelas
  colunas já estão nos arquivos de região.

  **Coluna nunca editada é regerada, não procurada.** Geração é determinística,
  então as duas respostas são os mesmos bytes e a barata ganha; há teste com a
  coluna presente no arquivo provando que mesmo assim ela é regerada.

  **No slice:** o pico de retenção caiu de **25 para 15** colunas (um orçamento
  de despejo, não o mundo editado inteiro), 25 colunas foram para 12 arquivos de
  região e 25 voltaram de lá — e o **save saiu byte-idêntico**, 116.904 bytes,
  com ou sem o spill. O smoke de determinismo continua idêntico a 1 e 8 threads.
- **STATUS:** OPEN (remediado, **opt-in**) — `RetainedChunks::new()` não mudou e
  continua sendo o padrão. O gatilho desta entrada (~1.000 colunas retidas, ou o
  primeiro servidor dedicado) **não disparou**: na escala da Phase 0, 25 colunas
  são 500 KiB e 3 ms por coluna é caro demais para pagar por isso. O mecanismo
  está pronto e medido; ligá-lo por padrão é a decisão que espera o gatilho.
  A `edited` ainda é memória e ainda não sobrevive ao processo — o que sobrevive
  agora é o **dado**, que era o custo que crescia.

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
- **RESOLUÇÃO (2026-09-14):** exatamente a remediação proposta. O
  `recovery::apply` agora arquiva por coluna todo registro que só falhou por
  residência, e o relatório carrega esse índice em `RecoveryReport::deferred`.
  O `WorldResidency::recovering(&mut pending)` liga o índice ao streaming: toda
  coluna que entra recebe os seus registros **antes** que qualquer coisa possa
  lê-la, na ordem em que foram journalados.

  **Por que adiar é seguro, e não uma quebra da regra de ordem.** O `apply`
  exige ordem de journal porque uma edição antiga escrita por cima de uma nova
  produz um mundo que nunca existiu. Dois registros que podem se sobrescrever
  estão na mesma posição, e a mesma posição está na mesma coluna: preservar a
  ordem *dentro* de cada coluna basta, e entre colunas não há o que preservar.
  Um teste fixa isso escrevendo duas vezes na mesma posição e exigindo que a
  segunda vença.

  **Três decisões:**

  1. **O relatório não ficou menos honesto.** Uma edição adiada continua em
     `skipped` e `is_complete()` continua falso. O índice **acrescenta** a
     capacidade de terminar o serviço; não troca o aviso por silêncio.
  2. **Só o que espera por residência é arquivado.** Um bloco que esta sessão
     não conhece não fica conhecido esperando, então é reportado e não
     enfileirado — arquivá-lo significaria retentá-lo contra toda coluna que
     carregasse, para sempre.
  3. **Uma edição ainda recusada com a coluna residente vira erro**, não
     descarte. Ela não estava esperando residência, e engolir isso é como um
     mundo passa a divergir do próprio journal em silêncio.

  Uma coluna que ninguém traz mantém suas edições no índice, sem aplicar —
  resultado honesto e **contável** (`PendingEdits::len`), não um silêncio.

  De quebra: `nexora_world::recovery` passou a reexportar `Replay` e `Damage`
  (como `JournalReplay`/`JournalDamage`). Os dois aparecem na assinatura pública
  de `apply`, e sem a reexportação todo chamador tinha de depender de
  `nexora-persistence` por um tipo que só vê através desta API.
- **STATUS:** **CLOSED** — a recuperação deixou de valer só até onde o conjunto
  residente alcançava. O slice não muda: cada chunk que ele edita já é residente
  no checkpoint, então ele continua exigindo zero `skipped` e o save segue
  byte-idêntico. Quem exercita o caminho adiado são os testes e o streaming.

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
- **STATUS:** OPEN (medido) — **metade do obstáculo saiu em 2026-09-13.** O
  `DenseSnapshot` do `DEBT-0029` existe, é `Send`, e há um teste que falha em
  compilar se deixar de ser. O que falta é só o agendamento: submeter ao job
  system. A parte difícil — dar ao worker uma entrada que não é o mundo — está
  feita. *(2026-09-20: o gatilho "existir um loop de quadro" também disparou —
  ADR-0017. O mesher continua rodando na thread que pedir; o que mudou é que
  agora dá para dizer em que estágio ele roda e quanto do quadro ele levou.)*

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
- **STATUS:** CLOSED (2026-09-13)
- **RESOLUÇÃO:** `nexora_mesh::DenseSnapshot`. A fixture do benchmark dizia de
  si mesma que estava *"deliberadamente fora do `nexora-mesh` — a resposta
  decide se vale a pena, e construir antes seria assumir"*. A resposta veio, e
  agora é código de engine.
  **Promover não foi copiar.** A fixture guardava só a superfície por célula e
  *derivava* a oclusão como "qualquer coisa presente oclui" — verdade quando foi
  escrita, mentira desde que materiais existem: um snapshot que re-deriva torna
  o vidro sólido, e um que esquece a camada desenha tudo no passe opaco. Os
  dois em silêncio. O tipo real **captura toda resposta que a view dá** —
  superfície, oclusão e camada — em vez de recalcular qualquer uma. O teste que
  o mantém honesto não é uma propriedade e sim uma igualdade: malhar por um
  snapshot tem que produzir *exatamente* a malha que malhar pela fonte produziu.
  **Duas medições, e trocar uma pela outra seria errado.** O número do achado 22
  (11,2×) exclui o custo de *construir* o snapshot, que é ele próprio uma
  passada de leituras do mundo. Medido agora, com a construção incluída:

  | | mediana |
  | --- | ---: |
  | `mesh.region_16` (direto do mundo) | **3,32 ms** |
  | `mesh.region_16_with_snapshot` (constrói **e** malha) | **1,22 ms** |
  | `mesh.region_16_from_snapshot` (snapshot já em mãos) | **296 µs** |

  Primeira malha de uma região: **2,7×**. Remalha com o snapshot em mãos:
  **11,2×**. Construir custa 0,92 ms, 76% do tempo com snapshot. O snapshot se
  paga já na primeira vez e se paga muito mais quando a região é malhada de
  novo sem os voxels terem mudado.
  Limite de **2 milhões de células** (~18 MiB): o `Extent` sozinho permitiria
  512³, que como snapshot seria um gigabyte para um job de malha.
  **Metade do `DEBT-0027` veio junto.** Um snapshot é `Send` e desligado do
  mundo no instante em que é tirado — há um teste que falha em compilar se
  deixar de ser. O obstáculo para malhar fora da thread do tick nunca foi
  agendamento; era um worker segurando `&World`. Esse obstáculo saiu.

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
  *(Aquela diferença fechou em 2026-09-13: medido de novo sobre um conjunto PBR
  completo, o alcançável é 8,72× e este codificador entrega 8,52×.)*

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
- **PROPOSED REMEDIATION:** duas coisas independentes: **(1) correspondência
  preguiçosa** e **(2) Huffman dinâmico**. A ordem de retorno prevista aqui
  estava **invertida** — ver abaixo.
- **TRIGGER:** quando o repositório de assets passar de algumas centenas de
  megabytes, ou antes de empacotar uma release.
- **TARGET STAGE:** Phase 2
- **STATUS:** CLOSED (2026-09-13)
- **RESOLUÇÃO:** **1,3881× → 1,0233×.** Dos 44 316 bytes que separavam do
  `zlib -9`, sobraram 2 655 — **94% da diferença fechou**. Medido sobre um
  conjunto PBR completo: 48 imagens, 995 264 bytes de scanlines filtradas.

  | | bytes | tabela | matcher |
  | --- | ---: | --- | --- |
  | antes | 158 503 | fixa | guloso, cadeia 32 |
  | + correspondência preguiçosa | 155 779 | fixa | preguiçoso, cadeia 32 |
  | + `MAX_CHAIN` 256 | 148 500 | fixa | preguiçoso, cadeia 256 |
  | **+ Huffman dinâmico** | **116 842** | **dinâmica** | preguiçoso, cadeia 256 |
  | `zlib -9 Z_FIXED` | 145 278 | fixa | do zlib |
  | `zlib -9` | 114 187 | dinâmica | do zlib |

  **A previsão desta entrada estava errada, e o jeito de descobrir foi medir.**
  Ela dizia que a correspondência preguiçosa *"costuma valer a maior parte da
  diferença"*. Valeu 9%. O que separou os dois lados foi o **controle**: o
  `zlib` aceita `Z_FIXED`, que mantém o matcher dele e troca a tabela dinâmica
  pela fixa. Com isso uma medição virou duas, cada variável isolada por vez, e
  a resposta foi **91% tabela, 9% matching** — o inverso da ordem prevista.
  A metade descrita aqui como *"a metade cara e rende menos"* rendeu 3,5× mais
  que a outra.

  Sinal de que o diagnóstico fechava, visível antes de escrever a segunda
  metade: dos 48 arquivos, **8 não encolheram um byte** com cadeia mais funda —
  e eram justamente os de pior razão (`forest_soil/height`, `roughness`). São
  os mapas de maior entropia, onde não há correspondência para achar em cadeia
  nenhuma, e o custo é inteiramente o de codificar literais. Isto é, a tabela.

  **O que entrou:**
  - Correspondência preguiçosa: o token não é emitido na posição em que foi
    achado, e cede a um estritamente maior um byte adiante. O teste não afirma
    um tamanho, afirma a *decisão*: um oráculo guloso que compartilha a mesma
    `Chain` e os mesmos emissores, de modo que a diferença entre os dois é a
    preguiça e nada mais.
  - `MAX_CHAIN` 32 → 256, com a varredura que o comentário anterior deveria ter
    tido. O 32 era verdade sob matching guloso e deixou de ser sob preguiçoso.
  - Blocos de **Huffman dinâmico** (RFC 1951 §3.2.7): código canônico limitado a
    15 bits, sequência de comprimentos em RLE com o alfabeto de 19 símbolos,
    HLIT/HDIST/HCLEN. E o lado do `inflate`, que antes recusava bloco dinâmico
    pelo nome — os dois tipos de bloco Huffman agora dividem um só laço de
    símbolos e um só conjunto de verificações de limite.
  - `deflate` passou a **pesar os três** tipos de bloco e emitir o menor. É por
    isso que nada disto pode aumentar arquivo nenhum: o matching roda uma vez e
    os dois codificadores recebem os mesmos tokens.

  **Dois defeitos que o trabalho encontrou, e que não eram dele:**
  1. A limitação de profundidade transcrita do `zlib` estava **errada**, e o
     `assert` do somatório de Kraft pegou. Um laço guiado por uma contagem de
     códigos longos demais roda vezes de menos: em 351 de 400 histogramas
     sintéticos o resultado não era um código prefixo — códigos sobrepostos,
     fluxo que decodificador nenhum lê. O laço passou a ser guiado pelo
     **próprio somatório**, que é a condição que importa, e aí fecha em 400 de
     400.
  2. O teste `incompressible_data_falls_back_to_stored_rather_than_growing`
     tinha **parado de testar o fallback**. Ele usava uma sequência de contador
     descrita como *"sem repetições dentro da janela"*; ela é uma progressão
     aritmética módulo 256, comprime **20×**, e a asserção passava sem
     significar nada. Oito bits de entropia por byte não diz se um *localizador
     de correspondências* acha alguma coisa — são propriedades diferentes. O
     teste recebeu bytes de fato inaproveitáveis, e o contra-exemplo virou caso
     próprio.

  **Verificação.** Dois vetores de bloco dinâmico produzidos pelo `zlib`, que
  não leu este código, mais a etapa de CI que decodifica **todo PNG gerado** com
  o `zlib` do Python — conferindo CRC de cada chunk, o fluxo `IDAT` inteiro e o
  byte de filtro de cada scanline. Essa etapa é nova: o `TEXTURE FORGE.md`
  afirmava que a CI fazia isso e a CI não fazia. E ela foi conferida contra um
  byte corrompido de propósito, porque conferência que não sabe falhar não
  confere nada.

  **Fica em aberto:** 2,3%, e agora eles estão no **matcher**, não na tabela.

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

### DEBT-0044 — Uma definição não escolhe paleta: dezesseis pedras são uma pedra com dezesseis sementes

- **SYSTEM:** `tools/texture-forge::recipe`, `engine/asset::document`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** o gerador procedural escolhe a receita (rampa de cor,
  ruído, rachaduras, faixas) por `Recipe::for_category(definition.category())`
  e por nada mais. O documento de material não tem campo em que uma receita,
  uma paleta ou um preset possa ser nomeado. Apareceu ao fechar a FASE 7: o
  batch já sabe impor a primeira geração (`policy` 16×16, só albedo), mas não
  há o que catalogar por ele que seja *diferente* dentro de uma família.
- **IMPACT:** a issue #5 pede dezesseis pedras "visualmente distinguíveis" —
  granito claro, basalto, ardósia, mármore escuro. Hoje as dezesseis sairiam
  como a mesma pedra cinza mosqueada, variando só na posição dos grãos. O
  mesmo vale para as 24 areias (#7), as 48 minérios (#6) e toda família
  procedural das issues #15–#26.
- **RISK:** médio. Não quebra nada que existe; impede o caminho procedural de
  produzir o catálogo 16×16, e empurra o operador para o caminho de geração
  externa (#4) — que é legítimo, mas precisa do caminho de importação que a
  auditoria (§4.6) deliberadamente deixou para depois.
- **PROPOSED REMEDIATION:** um campo `recipe` opcional no documento de material
  (schema 3 — o 2 é o `backend` do `main` —, com migração do 1 e do 2: ausente = a receita da categoria, que é o
  comportamento de hoje) nomeando um preset por `Identifier`
  (`nexora:recipe/granite_light`) e, dentro dele, a rampa de cor e os pesos de
  ruído. `GenerationTrace.preset` já existe e já é gravado — hoje sempre com o
  preset da categoria.
- **TRIGGER:** a primeira família 16×16 catalogada pelo caminho procedural.
- **TARGET STAGE:** Phase 1 (conteúdo da primeira geração)
- **STATUS:** RESOLVED — [ADR-0019](docs/adr/ADR-0019-a-recipe-is-data-and-a-material-names-it.md).
  Receitas são documentos em `content/recipes/`, nomeados pelo campo `recipe`
  do schema 3 do material. Prova: `recipe_book::tests` (receita pálida
  renderiza pálida; caminho, categoria e identificador errados são recusados),
  `forge::tests::editing_a_named_recipe_is_noticed_though_the_material_did_not_change`
  e `batch::tests::a_missing_recipe_stops_the_batch_before_anything_is_written`.

### DEBT-0045 — O runtime carrega textura como bytes, porque o decodificador PNG mora no forge

- **SYSTEM:** `engine/resource`, `tools/texture-forge::png`, `tools/texture-forge::deflate`
- **CLASS:** ARCHITECTURAL
- **WHY CREATED:** o `ResourceManager` (ADR-0021) resolve, verifica e faz cache
  de uma textura, mas só existe um loader de bytes para ela: o único
  decodificador PNG (e o inflate que ele usa) está em `tools/texture-forge`,
  misturado com o codificador, e o engine não pode depender de uma ferramenta.
- **IMPACT:** o runtime prova que tem os bytes certos de uma textura, mas não
  consegue transformá-los em `TextureMap`. Nada consome pixels hoje (não há
  renderizador), então o custo ainda é zero — e deixa de ser no primeiro
  consumidor.
- **RISK:** médio. A tentação no primeiro renderizador será copiar o
  decodificador para o engine, e aí haverá dois — exatamente o que a regra do
  Vanguard/NEXORA contra duas implementações do mesmo cálculo proíbe.
- **PROPOSED REMEDIATION:** mover `png::decode` e `deflate::inflate` (e os
  testes deles) para uma crate de engine sobre `nexora-asset`, reexportar no
  forge, e escrever um `TextureLoader` que devolve `TextureMap` com limites de
  dimensão e de razão de descompressão (a *Security* do documento de recursos).
- **TRIGGER:** o primeiro consumidor de pixels no runtime (renderizador,
  validação de conteúdo em runtime, ou ícone de UI).
- **TARGET STAGE:** Phase 1 (Resource System)
- **STATUS:** RESOLVED — [ADR-0022](docs/adr/ADR-0022-one-png-decoder-and-it-lives-in-the-engine.md).
  `engine/image` tem o `png::decode` e o `TextureLoader`, e infla com o
  `nexora_foundation::deflate::inflate_bounded` (limitado pelo tamanho que o
  cabeçalho declara, nos três tipos de bloco — o inflater próprio da imagem
  saiu no merge de 2026-09-25); o forge reexporta, e há um decodificador só. Prova: `engine/image` (PNGs montados à mão, sem o
  encoder), `first_generation::the_runtime_reaches_every_first_generation_texture_by_identifier`
  e o slice headless com `--resources` no CI.
