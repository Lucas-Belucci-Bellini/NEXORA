# NEXORA — WORLD GENERATION SEED AND REPRODUCIBILITY

## Objetivo

Permitir recriar o mesmo mundo lógico a partir de entradas versionadas.

## Reproduction key

```text
world_seed
+ generator_version
+ engine_version
+ world_configuration
+ content_version
+ mod manifest
+ mod versions
```

## Rules

1. Mudança no algoritmo deve alterar a generator version.
2. Não assumir que a mesma seed produz o mesmo resultado entre versões incompatíveis.
3. Seeds devem ser imutáveis depois da criação do mundo.
4. Configurações relevantes devem ser persistidas no save.

## Deterministic domains

Separar streams quando necessário:

```text
terrain
biome
cave
vegetation
weather
structures
civilization
AI
history
loot
```

## Debug

Ferramentas devem mostrar seed, versões e configurações responsáveis pelo mundo.

## Network

Servidor é a autoridade de geração e estado do mundo em sessões multiplayer.
