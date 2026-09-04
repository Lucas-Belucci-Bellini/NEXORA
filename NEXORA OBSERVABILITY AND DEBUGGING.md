# NEXORA — OBSERVABILITY AND DEBUGGING

## Objetivo

Tornar observáveis as causas e consequências de problemas de performance, simulação, rede, persistência e comportamento de NPCs.

## Telemetry layers

```text
LOGS
METRICS
TRACES
EVENT TRACES
SIMULATION CHECKPOINTS
PROFILE DATA
CRASH DATA
```

## Correlation

Operações importantes devem possuir IDs correlacionáveis:

```text
command_id
transaction_id
event_id
entity_id
world_id
save_id
replay_id
```

## Causal debugging

Permitir reconstruir:

```text
input
→ intent
→ command
→ state change
→ event
→ consequence
→ downstream events
```

## AI debugging

Registrar de forma controlada:

```text
perception
knowledge used
goal
constraints
selected action
reason code
```

Sem depender de explicações textuais livres como fonte de verdade.

## Performance

Instrumentar jobs, chunks, entity processing, streaming, serialization, networking e rendering com baixo overhead configurável.

## Production safety

Telemetry deve possuir sampling, limites, redaction e desligamento configurável para evitar impacto indevido no runtime.
