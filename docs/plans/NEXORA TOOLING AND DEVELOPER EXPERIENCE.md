# NEXORA — TOOLING AND DEVELOPER EXPERIENCE

## Objetivo

Tornar possível entender e depurar um engine que executa milhares de sistemas e uma simulação persistente.

## Ferramentas planejadas

```text
nexora-cli
World Inspector
Chunk Inspector
Entity Inspector
Component Inspector
Simulation Profiler
Job Graph Viewer
Event / History Viewer
Knowledge Viewer
Network Inspector
Save Inspector
Replay Debugger
Asset Validator
Mod Validator
Registry Viewer
Resource Monitor
```

## CLI

Comandos devem cobrir:

```text
build
run
server
headless
benchmark
test
validate
package
cook
inspect
replay
migrate
```

## Inspector principle

Ferramentas devem consultar contratos públicos e snapshots de diagnóstico, evitando hacks que dependam do layout interno do runtime.

## World debugging

Permitir selecionar:

```text
world → dimension → region → chunk → entity → component
```

## Simulation debugging

Permitir visualizar:

```text
intent
command
event
causal chain
AI decision
state change
```

## Performance

Exibir:

```text
frame time
simulation time
job utilization
streaming latency
memory
GPU memory
network queues
entity counts
chunk counts
LOD distribution
```

## Regra

Toda ferramenta importante deve ter uma fonte de dados documentada e uma versão compatível do contrato que consome.
