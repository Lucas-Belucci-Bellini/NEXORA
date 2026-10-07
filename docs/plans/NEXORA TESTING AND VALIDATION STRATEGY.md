# NEXORA — TESTING AND VALIDATION STRATEGY

## Objetivo

Validar engine, simulação, conteúdo, persistência, rede, modding e ferramentas em várias escalas.

## Pirâmide

```text
UNIT
↓
COMPONENT / SYSTEM
↓
INTEGRATION
↓
SIMULATION
↓
END-TO-END
```

## Classes especiais

```text
PROPERTY
FUZZ
STRESS
SOAK
DETERMINISM
SAVE/LOAD
MIGRATION
NETWORK
SECURITY
MOD COMPATIBILITY
PERFORMANCE
REPLAY
```

## Testes de simulação

Executar mundos controlados por longos períodos e verificar invariantes:

```text
population >= 0
resources >= 0 where required
ownership unique
time monotonic
history causal links valid
entity IDs unique
no invalid references
```

## Stress

Escalar progressivamente:

```text
1k entities
10k
100k abstract
large settlements
large history graphs
large save files
many simultaneous events
```

## Soak

Simulações de muitas horas/dias de runtime para detectar:

```text
memory leaks
queue growth
fragmentation
state drift
history bloat
performance degradation
```

## Test fixtures

Manter mundos pequenos e reproduzíveis para cada subsistema crítico.

## CI gates

Nenhum merge/release deve avançar com regressão crítica conhecida sem exceção registrada.
