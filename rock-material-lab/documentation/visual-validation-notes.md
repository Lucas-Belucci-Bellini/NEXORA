# Visual validation notes

Data da inspeção: 2026-09-04.

A primeira dobra carregou corretamente o cabeçalho, a hipótese, os cinco controles de parâmetros, os três métodos e os três modos de distância. A seção de amostras exibiu os três canvases e os rótulos Rock A / Rock B / Rock C. A seção seguinte exibiu a tabela comparativa, o score do método híbrido e o perfil de performance.

A renderização inicial com seed 1847, detalhe 58%, rugosidade 66%, erosão 34% e variação mineral 48% apresentou três silhuetas distintas, com granito mais frio e compacto, pedra natural mais terrosa e pedra erodida com leitura de desgaste. A inspeção também motivou o reforço de contraste de grãos, inclusões e cavidades para garantir leitura mais clara no alcance médio.

A página permaneceu responsiva na viewport de inspeção e não relatou erro de carregamento no documento visível. A validação de interação de botões e sliders será complementada por checagem automatizada e por execução no navegador após a atualização visual.

A seleção de `B — Procedural / shader` foi testada no navegador. O score mudou para 80/100, a decisão passou a descrever o trade-off de custo de shader, a memória foi exibida como 0,0 MB e o custo como alto / 8 camadas de ruído. Isso confirma que o estado visual e analítico reage ao método escolhido.

O modo `Distante` foi selecionado e atualizou todos os rótulos dos canvases para `DISTANTE`, reduziu o score para 78/100 e alterou a nota para `detalhe reduzido`. O botão `Gerar nova amostra` também foi acionado: a seed mudou de 1847 para 1898 e o conteúdo analítico permaneceu consistente. A interface segue sem erro visível.
