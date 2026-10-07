# NEXORA — AI DECISION ARCHITECTURE

## Pipeline

```text
PERCEPTION
→ KNOWLEDGE
→ NEEDS / GOALS
→ CONTEXT
→ PLANNING
→ DECISION
→ INTENT
→ COMMAND
→ SIMULATION
```

## Informação limitada

NPCs devem tomar decisões somente com dados disponíveis por percepção, memória, comunicação, educação, documentos ou sensores autorizados.

```text
WORLD TRUTH
≠
NPC KNOWLEDGE
```

## Needs

Exemplos:

```text
survival
food
safety
shelter
social
wealth
status
curiosity
loyalty
political power
```

## Goals

Goals podem ser individuais, familiares, institucionais ou civis. Objetivos entram em conflito e possuem prioridade/contexto.

## Planning

Não exigir planejamento global perfeito. Usar racionalidade limitada, orçamento de CPU, horizonte limitado e decisões escaláveis por LOD.

## Intent

IA não muta diretamente o mundo. Produz intent/command sujeito às mesmas validações de jogadores e sistemas autorizados.

## Factions / Civilization

Agentes podem formar grupos, coalizões e instituições através de confiança, interesses compartilhados, relações e comunicação.

## Example

```text
high_taxation
→ dissatisfaction
→ political organization
→ information sharing
→ coalition candidate
→ planning
→ rebellion command
→ war simulation
→ history event
```

## LOD

```text
FULL → individual reasoning
REGIONAL → aggregated decision models
ABSTRACT → statistical / institutional decisions
```

## Determinism

Decisões críticas devem usar RNG e inputs versionados quando a reprodução for necessária.
