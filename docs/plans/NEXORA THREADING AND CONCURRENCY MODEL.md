# NEXORA — THREADING AND CONCURRENCY MODEL

## Objetivo

Definir onde o trabalho roda, quais dados podem ser acessados e quais sincronizações são permitidas.

## Classes de execução

```text
MAIN / COORDINATION
RENDER
SIMULATION
JOB WORKERS
IO / STREAMING
NETWORK
SCRIPT SANDBOX
TOOLS / EDITOR
```

## Princípio

O trabalho pesado deve ser dividido em jobs independentes sempre que possível. O código não deve assumir que uma entidade, chunk ou sistema inteiro possui exclusividade de thread sem contrato explícito.

## Modelo recomendado

```text
Main
 ↓
Frame / Tick Coordination
 ↓
Job Graph
 ├── World jobs
 ├── Entity jobs
 ├── Physics jobs
 ├── AI jobs
 ├── Streaming jobs
 ├── Serialization jobs
 └── Analysis jobs
```

## Ownership

Preferir:

```text
single-writer ownership
immutable snapshots
message passing
batch processing
explicit synchronization points
```

Evitar locks globais e estado compartilhado mutável sem necessidade.

## Fases de simulação

```text
INPUT / NETWORK
→ INTENT / COMMAND
→ SIMULATION
→ EVENTS
→ DERIVED DATA
→ PRESENTATION
→ SAVE / REPLICATION
```

A ordem deve ser determinística quando o sistema exigir reprodução consistente.

## Concorrência segura

Cada sistema deve documentar:

```text
READ SET
WRITE SET
THREAD AFFINITY
SYNC POINTS
DEFERRED WRITES
```

## Streaming

IO e descompressão não devem bloquear a thread principal. A ativação final de objetos deve ocorrer em ponto controlado pelo runtime.

## Scripts / Mods

Scripts não devem possuir acesso irrestrito ao estado concorrente do engine. Operações mutáveis passam por APIs controladas e quotas.

## Regra crítica

Se um sistema exige uma ordem global rígida, documentar o motivo. Não serializar trabalho simplesmente por conveniência de implementação.
