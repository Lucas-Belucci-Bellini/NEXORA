# NEXORA — TECHNOLOGY BENCHMARK PLAN

## Objetivo

Escolher a stack de implementação por evidência, não por preferência ou popularidade.

## Candidatos

Benchmark pode comparar combinações envolvendo Rust, C/C++, TypeScript/tooling e outras alternativas apenas quando houver justificativa.

## Vertical slice

```text
window
→ input
→ RHI
→ camera
→ 16³ voxel chunk
→ mesh generation
→ 1,000 entities
→ physics
→ jobs
→ streaming
→ save/load
→ headless server
→ mod boundary
```

## Métricas

```text
build time
incremental build time
startup
RAM
CPU time
frame time
simulation time
job overhead
serialization throughput
streaming latency
FFI overhead
debugging effort
tooling effort
binary/package size
```

## Regras

1. Mesmo workload lógico.
2. Mesmas metas de qualidade.
3. Medir várias execuções.
4. Registrar metodologia e hardware.
5. Não escolher pelo benchmark de um único microcaso.

## Decisão

Resultado deve produzir:

```text
performance score
engineering complexity
tooling score
maintenance risk
platform score
modding score
```

A decisão final deve ser registrada em ADR e refletida na arquitetura.
