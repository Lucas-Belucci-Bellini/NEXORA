# NEXORA — BUILD / CI / RELEASE ARCHITECTURE

## Objetivo

Garantir builds reproduzíveis, testes automáticos, artefatos rastreáveis e releases consistentes.

## Targets

```text
client
server
dedicated-server
headless
editor
tools
sdk
benchmark
replay-runner
```

## CI stages

```text
FORMAT / LINT
→ STATIC ANALYSIS
→ UNIT TEST
→ INTEGRATION TEST
→ SIMULATION TEST
→ SAVE / MIGRATION TEST
→ DETERMINISM TEST
→ BUILD
→ PACKAGE
→ SMOKE TEST
```

## Matriz

Testar pelo menos os targets e plataformas oficialmente suportados pelo projeto quando entrarem em escopo.

## Artifacts

Cada build release deve registrar:

```text
commit
version
build ID
platform
compiler/toolchain
content version
mod compatibility version
checksums
symbols/debug metadata
```

## Release channels

```text
nightly
experimental
alpha
beta
stable
```

## Reproducibility

Builds críticos devem ser suficientemente rastreáveis para identificar código, dependências e ferramentas usadas.

## Release gate

Não liberar sem smoke test, validação de assets, compatibilidade do save, checksums e changelog técnico.
