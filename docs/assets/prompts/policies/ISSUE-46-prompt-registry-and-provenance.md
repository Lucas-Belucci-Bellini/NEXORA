<!-- Imported from GitHub Issue #46. Original issue: https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/46 -->

# NEXORA — PROMPT REGISTRY / ASSET PROVENANCE

## Objetivo

Criar um registro permanente dos prompts usados por Claude, outras IAs e ferramentas de geração durante a produção visual do NEXORA.

Esta issue deve servir como índice e política. O conteúdo detalhado pode posteriormente ser movido para arquivos `.md` versionados.

## Regra

Nenhum prompt importante de produção deve existir somente dentro de uma conversa.

Para cada família de assets, registrar:

- objetivo;
- prompt-base;
- variações;
- resolução;
- ferramenta/modelo usado;
- data;
- referência conceitual;
- resultado aprovado;
- motivo de rejeição quando aplicável;
- caminho final do asset;
- versão.

## Estrutura recomendada

```text
docs/assets/
├── ART_DIRECTION.md
├── PROMPT_REGISTRY.md
├── PROVENANCE.md
├── textures/
│   ├── stones.md
│   ├── wood.md
│   ├── ores.md
│   ├── metals.md
│   └── ...
├── mobs/
├── equipment/
└── vfx/
```

## Regra para referências externas

Quando ATM10, Minecraft ou qualquer outro projeto for usado como referência, registrar somente a característica abstrata observada.

Exemplo:

**Permitido**
- 'alta variedade de minerais'
- 'materiais precisam ser distinguíveis em baixa resolução'
- 'máquinas devem possuir linguagem industrial consistente'
- 'blocos de uma mesma família devem compartilhar princípios visuais'

**Não permitido**
- 'copiar esta textura'
- 'fazer igual a este bloco'
- 'alterar esta PNG'
- 'usar esta textura como base'
- 'reproduzir exatamente esta paleta/padrão específico'

## Template de registro

```md
# Asset: <nome>

- Categoria:
- Resolução:
- Ferramenta:
- Modelo:
- Data:
- Versão:
- Caminho:
- Status: draft / approved / rejected

## Objetivo

<descrição>

## Prompt

<texto completo>

## Referências conceituais

<descrição abstrata>

## Restrições

- design original;
- sem tracing;
- sem cópia de assets de terceiros;
- sem logos/marcas não autorizadas;
- compatibilidade com NEXORA.

## Validação

- [ ] resolução correta
- [ ] tiling, quando aplicável
- [ ] legibilidade
- [ ] consistência de família
- [ ] integração no engine
- [ ] proveniência registrada
- [ ] revisão de originalidade
```

## Regra para Claude

Antes de gerar um novo conjunto de assets:

1. Ler esta issue.
2. Ler #45.
3. Ler as issues específicas da família.
4. Procurar prompts já existentes para evitar duplicação.
5. Analisar as PNG/SVG existentes do NEXORA.
6. Se houver material de referência externo, convertê-lo em requisitos abstratos.
7. Gerar designs independentes.
8. Registrar o prompt e a proveniência.
9. Validar os resultados.
10. Só então considerar o asset pronto.

## Critério de conclusão

- [ ] Todos os prompts de produção relevantes estão versionados.
- [ ] Referências externas são documentadas como referências, não como fontes de assets.
- [ ] Assets aprovados possuem proveniência.
- [ ] Assets rejeitados relevantes possuem motivo.
- [ ] Claude consegue reconstruir o contexto sem depender do histórico da conversa.
- [ ] A documentação permite auditoria futura do processo de criação.

---

**Source:** [GitHub Issue #46 — PROMPT REGISTRY — Documentar prompts e referências usadas na produção de assets](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/46)
