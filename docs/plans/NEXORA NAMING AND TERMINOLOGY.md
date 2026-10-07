# NEXORA — NAMING AND TERMINOLOGY

## Objetivo

Manter uma linguagem técnica consistente entre código, documentos, APIs, editor, mods e ferramentas.

## Regras

- nomes de sistemas usam termos estáveis e específicos;
- IDs lógicos não dependem de nomes de arquivo;
- conceitos diferentes não usam o mesmo nome;
- abreviações só quando oficiais;
- comandos, eventos e queries devem ser semanticamente distintos.

## Termos normativos

```text
Command = intenção solicitada
Event = fato já ocorrido
Query = leitura
Resource = recurso identificado
Asset = conteúdo consumível pelo runtime
Entity = identidade lógica
Component = estado/composição de entidade
World Event = acontecimento persistente da simulação
History Fact = registro factual do que ocorreu
Knowledge = informação disponível a um agente
Lore = narrativa/interpretação derivada de fatos/conhecimento
```

## IDs

Preferir namespaces:

```text
nexora:block/*
nexora:item/*
nexora:entity/*
nexora:event/*
nexora:resource/*
```

Mods devem possuir namespace próprio.

## Documento vs código

Quando um termo mudar de significado, atualizar o glossário e referências afetadas antes de consolidar a mudança.
