# Rock Material Experiment

## Objetivo

Este experimento investiga como o NEXORA pode produzir pedras bonitas, variadas e reproduzíveis sem acoplar a aparência a uma coleção de texturas copiadas ou a um sistema definitivo prematuro. A pergunta central é o equilíbrio entre **qualidade visual, variedade determinística e custo de runtime**.

A arquitetura existente do NEXORA é documental e já estabelece os pontos de integração que devem ser respeitados futuramente: definições de material orientadas a dados, registro de recursos, pipeline de conteúdo, procedência de assets e budgets mensuráveis. Não há renderer, build ou runtime executável neste checkout; por isso, o laboratório foi implementado como uma área estática e isolada, sem criar uma segunda arquitetura de engine.

## O que foi testado

O protótipo compara três famílias visuais:

| Amostra | Intenção visual | Sinal principal |
| --- | --- | --- |
| **Rock A — Pedra natural** | Superfície irregular, tonalidade terrosa, grão e imperfeições leves. | Leitura orgânica e mineral. |
| **Rock B — Granito compacto** | Base fria e sólida com inclusões minerais pontuais. | Compactação e microdetalhe. |
| **Rock C — Pedra erodida** | Desgaste, cavidades, estratos e contraste de exposição. | História de ambiente na superfície. |

As amostras usam a mesma função conceitual `RockGenerator(seed, type, scale, erosion, mineral_variation)`. A seed, a escala do detalhe, a rugosidade, a erosão e a variação mineral podem ser alteradas pela interface e produzem uma nova imagem de forma determinística.

Também foram comparados três métodos:

| Método | Implementação no laboratório | Vantagem | Risco observado |
| --- | --- | --- | --- |
| **A — Textura tradicional** | Campo de grãos e manchas simplificado, representando uma textura autoral rasterizada. | Leitura previsível e custo de shader mínimo. | Repetição e memória de mapas quando a escala cresce. |
| **B — Procedural / shader** | Ruído multi-escala, variação tonal, grão e minerais gerados por seed. | Variedade, reprodutibilidade e memória de bitmap próxima de zero. | Pode parecer “noise shader” e elevar o custo de fragmento. |
| **C — Híbrido** | Silhueta e cavidades na geometria, máscara/variação procedural no material e detalhe reduzido conforme a distância. | Separa forma de superfície e preserva detalhe útil no médio alcance. | Pipeline mais elaborada; exige budgets e validação de variantes. |

## Qual técnica venceu

**O método C — procedural + máscara + geometria — venceu como direção recomendada para o futuro.** Ele entrega uma silhueta reconhecível com baixa dependência de bitmap e permite que a mesma família de pedra produza instâncias diferentes a partir de seeds. A geometria assume a responsabilidade por irregularidades grandes e cavidades; o material assume tonalidade, grão, inclusões e microvariação.

A textura tradicional continua válida para assets hero ou superfícies que precisem de controle artístico absoluto. O procedural puro é uma boa referência de pesquisa, mas não deve receber automaticamente toda a forma: sem uma máscara geológica e uma disciplina de frequência, o ruído tende a ficar homogêneo, repetitivo em outro sentido ou caro demais.

## Resultado visual por distância

No **close**, o método híbrido mantém microgrão, inclusões e fissuras suficientes para evitar uma superfície lisa. No **médio**, a silhueta irregular continua legível e a variação tonal não desaparece. No **distante**, o detalhe é reduzido por parâmetro para evitar ruído e custo sem benefício visual proporcional.

A interface não tenta declarar uma medição de GPU real: ela expõe um perfil comparativo estimado para orientar a próxima etapa. A confirmação de produção deve ocorrer dentro do renderer do NEXORA, com instrumentação por backend.

## Performance

| Indicador | Perfil do protótipo | Interpretação |
| --- | --- | --- |
| Polígonos | Aproximadamente 1,3k–2,3k por amostra lógica, conforme rugosidade. | Adequado para protótipo; precisa de LOD no runtime. |
| Memória de textura | 0,0 MB no procedural; 0,5 MB no híbrido conceitual; 2,4 MB na alternativa tradicional. | O procedural reduz armazenamento, mas não elimina custo de shader. |
| Material | 0, 8 ou 5 camadas conceituais nos métodos A, B e C. | O híbrido evita levar todas as frequências para todos os pixels. |
| Draw calls | Três no cenário do laboratório, uma por família. | É uma referência de comparação; batching/instancing ainda não foi implementado. |

Os números de memória e camadas acima são **estimativas do experimento**, não benchmark do renderer. O próximo teste deve medir frame time, GPU time, memória residente, variantes compiladas e comportamento em lote com LOD.

## Problemas encontrados e limitações

O checkout atual contém especificações de arquitetura, mas não contém código de renderer, sistema de materiais ou pipeline de assets executável. Assim, não foi possível validar shader real, draw calls reais, compilação por backend ou integração com blocos/biomas. O Canvas 2D é uma aproximação deliberada para testar a linguagem visual sem contaminar o core.

As cavidades, estratos e grãos são representações 2D, não meshes ou mapas normais exportáveis. A avaliação de close é útil para direção, mas não substitui uma prova em malha 3D com iluminação física. A comparação de textura tradicional é simulada proceduralmente para manter a procedência limpa; uma futura comparação raster deve usar mapas autorais registrados.

## Próximo passo

Quando o renderer e o Content Pipeline existirem, transportar primeiro o contrato de dados de `materials/rock-materials.json` para uma definição de material versionável. Em seguida, criar uma pequena família de meshes base autorais, gerar uma máscara geológica em bake ou runtime, configurar LOD por distância e medir o método híbrido com instancing. Só depois decidir quais partes merecem entrar no core.

## O que não deve entrar no core ainda

Não incorporar agora um sistema de mineração, inventário, crafting, destruição, economia, geração mundial, centenas de tipos de rocha, uma biblioteca de shaders ampla ou uma API pública definitiva de `RockGenerator`. Também não promover as estimativas de performance deste protótipo a budgets de produção sem benchmark por backend.

## Procedência

Todos os visuais desta versão são gerados localmente em `prototype/app.js`, a partir de funções originais de ruído, composição e desenho. Nenhum bitmap externo ou textura de jogo existente foi utilizado. O shader em `shaders/rock-material-lab.glsl` é conceitual e também original.
