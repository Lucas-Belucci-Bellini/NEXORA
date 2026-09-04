# NEXORA — Rock Material Lab

> Protótipo visual isolado para descobrir uma pipeline reproduzível de pedras. **Não é uma implementação definitiva do core.**

## Execução local

A interface é estática e não depende de bibliotecas ou assets externos. Na raiz do repositório, sirva a pasta com qualquer servidor HTTP estático:

```bash
python3 -m http.server 4173 --directory rock-material-lab/prototype
```

Depois, abra `http://localhost:4173` no navegador. A abertura direta via `file://` pode ser limitada por políticas do navegador; o servidor local evita essa limitação.

## O que está incluído

| Área | Conteúdo |
| --- | --- |
| `prototype/` | Laboratório navegável, com renderização procedural em Canvas 2D, controles e perfil de performance. |
| `materials/` | Definições de dados para Rock A, Rock B, Rock C e os métodos comparados. |
| `shaders/` | Shader conceitual que registra a tradução futura para uma implementação de material real. |
| `textures/` | Reservado para máscaras ou mapas autorais futuros; esta versão não carrega bitmap. |
| `documentation/` | Relatório do experimento, decisão técnica e limitações conhecidas. |
| `tests/` | Testes leves de invariantes do protótipo, executados sem dependências. |

## Interação

A interface permite alternar entre os métodos `A — Textura tradicional`, `B — Procedural / shader` e `C — Híbrido recomendado`, além de observar as famílias natural, granito e erodida nas distâncias close, médio e distante. Os parâmetros de seed, detalhe, rugosidade, erosão e variação mineral alteram todas as amostras de maneira determinística.

O botão **Gerar nova amostra** altera apenas a seed. **Variar parâmetros** cria uma combinação nova de parâmetros dentro de uma faixa útil para comparação.

## Origem dos assets

Todo o conteúdo visual é original e gerado em tempo de execução pelo `prototype/app.js`. Não foram copiadas texturas de jogos, bibliotecas proprietárias ou imagens externas. O arquivo `materials/rock-materials.json` funciona como registro de proveniência do experimento.

## Fora de escopo

O laboratório não implementa mineração, inventário, crafting, destruição, economia, geração mundial, integração com o renderer ou um sistema definitivo de materiais. A pasta existe para validar a direção visual e não deve ser tratada como contrato final do runtime.
