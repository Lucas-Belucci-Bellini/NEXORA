# NEXORA — MEMORY AND RESOURCE OWNERSHIP

## Objetivo

Controlar CPU memory, GPU memory, assets, world data, caches e buffers em uma engine de grande escala.

## Classes

```text
ENGINE / LONG-LIVED
WORLD / PERSISTENT
FRAME / TRANSIENT
STREAMING
GPU
NETWORK
SCRIPT / MOD
EDITOR
```

## Princípios

- ownership explícito;
- lifetime documentado;
- evitar cópias grandes desnecessárias;
- hot data separada de cold data;
- caches são descartáveis;
- recursos compartilhados usam handles/ref-count ou equivalente seguro;
- budgets por subsistema.

## World memory

Chunks devem possuir estados de residência e orçamento:

```text
UNRESIDENT
LOADING
RESIDENT
ACTIVE
EVICTING
```

A residência física não altera a identidade lógica do mundo.

## GPU memory

Renderer/RHI é responsável por alocação e lifetime de recursos GPU. Gameplay não manipula recursos gráficos diretamente.

## Asset cache

Caches devem possuir:

```text
capacity
priority
last-use
pinning
eviction policy
telemetry
```

## Network buffers

Possuir limites explícitos por conexão, sessão, pacote e fila para impedir crescimento não controlado.

## Mod / Script budgets

Aplicar quotas de:

```text
CPU
memory
entities
events
network
storage
execution time
```

## Debug

Cada grande pool deve expor contadores, high-water marks, leaks suspeitos e pressão de orçamento.
