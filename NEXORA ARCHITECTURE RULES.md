# NEXORA — ARCHITECTURE RULES

## 1. Core

Core fornece regras e contratos; conteúdo não deve invadir o Core com lógica específica de mundo.

## 2. Ownership

Todo estado autoritativo possui um único owner.

## 3. Authority

Client requests; server validates and owns authoritative multiplayer state; simulation produces results.

## 4. Commands / Events / Queries

```text
Command = intenção
Event = fato ocorrido
Query = leitura
```

Não usar Event como comando disfarçado.

## 5. Dependency

Dependências apontam para contratos estáveis e não criam ciclos.

## 6. Concurrency

Dados compartilhados mutáveis precisam de ownership e sincronização explícitos.

## 7. Persistence

Caches e dados derivados não são fontes de verdade do save.

## 8. LOD

Representação pode mudar; identidade lógica e regras persistentes não desaparecem.

## 9. Modding

Mods usam APIs públicas, namespaces, permissões e versionamento. Não depender de detalhes privados do engine.

## 10. Editor

Editor e runtime devem compartilhar o mesmo modelo lógico quando aplicável.

## 11. Language boundaries

Linguagens diferentes comunicam por ABI/API/protocolo documentado. Evitar chamadas cruzadas em hot loops.

## 12. Observability

Sistemas críticos devem ser diagnosticáveis sem depender de estado secreto.

## 13. Experiments

Protótipos podem quebrar regras dentro de uma área explicitamente experimental; integração em runtime exige revisão.
