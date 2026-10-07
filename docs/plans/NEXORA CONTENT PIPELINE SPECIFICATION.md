# NEXORA — CONTENT PIPELINE SPECIFICATION

## Objetivo

Transformar assets e definições autorais em recursos validados e consumíveis pelo runtime, mantendo editor, jogo e mods dentro do mesmo modelo de dados.

## Pipeline

```text
SOURCE
→ IMPORT
→ NORMALIZE
→ VALIDATE
→ BUILD / COOK
→ COMPRESS
→ PACKAGE
→ INDEX
→ RUNTIME RESOURCE
```

## Tipos

```text
Textures
Meshes
Materials
Shaders
Animations
Audio
VFX
Blocks
Items
Entities
Prefabs
Structures
World data
Localization
UI assets
Mod content
```

## IDs

Todo recurso persistente deve ter Resource ID estável. Nome de arquivo não deve ser o identificador lógico primário.

## Validation

Verificar:

```text
schema
references
dependencies
bounds
formats
licenses
ownership
performance budgets
missing resources
```

## Cooking

O produto final não deve depender de parsing pesado de formatos-fonte quando um formato compilado/normalizado for superior.

## Mod content

Mods passam pelo mesmo pipeline conceitual, com namespaces, dependências, permissões e limites.

## Runtime

Runtime acessa recursos por Resource/Asset Manager e handles. Gameplay não deve conhecer caminhos físicos de pacote.

## Editor parity

O editor deve editar o mesmo modelo lógico que o runtime consome, evitando um formato paralelo incompatível.
