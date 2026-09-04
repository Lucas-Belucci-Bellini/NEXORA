# NEXORA — REPLAY AND DETERMINISM

## Objetivo

Permitir reproduzir uma simulação, investigar bugs, detectar divergências e validar sistemas críticos.

## Determinism classes

```text
STRICT
DETERMINISTIC WHERE REQUIRED
NON-DETERMINISTIC PRESENTATION
```

Não exigir determinismo absoluto de elementos que não precisam dele, mas o núcleo de simulação deve ter regras claras.

## Replay inputs

Um replay deve poder registrar:

```text
world seed
engine version
content version
mod set + versions
world configuration
simulation time origin
player inputs
network commands
authoritative events
RNG seeds/streams
```

## RNG

Usar streams nomeados/isolados por domínio quando necessário:

```text
worldgen
weather
AI
combat
loot
civilization
history
```

Não usar um RNG global imprevisível para toda a simulação.

## Divergence detection

Comparar checkpoints resumidos:

```text
tick
world hash
entity hash
region hash
critical subsystem hashes
```

Ao divergir, gerar diagnóstico causal.

## Replay modes

```text
LIVE
RECORD
PLAYBACK
FAST_FORWARD
STEP
PAUSE
BRANCH
```

## Uso

Replays devem servir para:

- reprodução de bugs;
- testes de regressão;
- debugging de IA;
- validação de multiplayer;
- comparação de versões;
- análise histórica;
- benchmark.

## Limite

Replay completo não precisa armazenar todo estado de cada frame se checkpoints + comandos + eventos forem suficientes para reconstrução confiável.
