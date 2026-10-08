<!-- Imported from GitHub Issue #45. Original issue: https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/45 -->

# NEXORA — ART DIRECTION / REFERÊNCIA VISUAL EXTERNA

## Objetivo

Usar uma instalação/modpack modificado do ATM10 disponibilizado pelo autor como **material de referência visual e funcional**, para ajudar a equipe/agentes a entender o nível de variedade, densidade de detalhes, legibilidade e organização visual desejado para o NEXORA.

> **Regra central:** ATM10 é referência de qualidade e observação. Os assets do NEXORA devem ser criados de forma independente.

## Material que o autor deve fornecer ao agente

O agente deverá receber, quando possível:

- uma cópia local da instalação/modpack de ATM10 modificado que o autor possui;
- screenshots próprias mostrando os materiais/blocos/itens que servem como referência;
- as PNG/SVG já existentes do NEXORA;
- a documentação atual de texturas e assets;
- a lista/catálogo de materiais que o NEXORA precisa produzir;
- informações sobre resolução desejada (atualmente 16×16 quando aplicável);
- qualquer restrição visual já definida nas issues de textura.

**Não é necessário colocar o modpack ou seus assets no repositório NEXORA.**

## O que o agente pode analisar

O agente pode observar:

- variedade de materiais;
- densidade geral de detalhes;
- leitura visual em primeira pessoa;
- contraste entre materiais;
- diferença visual entre famílias de recursos;
- consistência entre blocos de uma mesma família;
- escala aparente dos detalhes;
- organização de máquinas, minérios, blocos e equipamentos;
- quantidade de variação necessária para que o mundo não pareça repetitivo;
- características gerais de uma linguagem voxel;
- problemas de legibilidade que o NEXORA deve evitar.

## O que o agente NÃO deve fazer

O agente não deve:

- copiar texturas;
- extrair PNGs de Minecraft, mods ou modpacks;
- modificar uma textura existente para transformá-la em textura NEXORA;
- redesenhar por cima de uma textura de referência;
- reproduzir pixel a pixel ou quase pixel a pixel;
- copiar modelos, ícones, logos ou símbolos;
- copiar nomes protegidos como identidade visual do NEXORA;
- usar screenshots como camada para geração;
- pedir a uma IA para 'fazer igual' a uma textura específica;
- comparar uma textura NEXORA e uma textura de referência buscando maximizar semelhança;
- incluir assets de terceiros no repositório sem verificar a licença/direitos aplicáveis.

## Pipeline obrigatório

```text
ATM10 / referências
        ↓
OBSERVAÇÃO
        ↓
REQUISITOS VISUAIS ABSTRATOS
        ↓
ART DIRECTION NEXORA
        ↓
DESIGN ORIGINAL
        ↓
GERAÇÃO / PIXEL ART
        ↓
VALIDAÇÃO DE ORIGINALIDADE
        ↓
TESTE NO ENGINE
        ↓
ASSET NEXORA
```

Nunca:

```text
ATM10 → copiar/modificar → NEXORA
```

## Prompt-base para o agente

```text
Use o material de referência fornecido pelo autor somente para compreender o nível de complexidade visual, variedade, legibilidade, organização de materiais e qualidade esperada para o NEXORA.

Não copie, extraia, trace, redesenhe por cima, reconstrua ou procure reproduzir de perto qualquer textura, modelo, ícone, símbolo, interface, logo ou outro asset observado.

Transforme as observações em requisitos artísticos abstratos e crie o design NEXORA independentemente.

Quando houver dúvida se uma textura está excessivamente parecida com uma referência, descarte a versão e crie outra solução visual.

Priorize identidade própria do NEXORA, consistência entre famílias de materiais, legibilidade em baixa resolução, tiling quando necessário e compatibilidade técnica com o engine.

Registre a proveniência de cada asset novo.
```

## Proveniência

Cada asset produzido deve ter, quando aplicável:

- nome do asset;
- categoria;
- resolução;
- autor/ferramenta de geração;
- data;
- prompt utilizado;
- referências conceituais utilizadas;
- confirmação de que não houve cópia/trace de asset de terceiros;
- versão do asset;
- localização no repositório.

## Critérios de aceitação

- [ ] Referências externas utilizadas somente para análise.
- [ ] Nenhum asset de terceiros copiado para o NEXORA.
- [ ] Texturas novas possuem design próprio.
- [ ] PNG/SVG existentes do NEXORA foram considerados.
- [ ] Resolução e tiling foram validados.
- [ ] Proveniência foi registrada.
- [ ] O asset foi testado no engine quando tecnicamente possível.
- [ ] Dúvidas de licença/direitos foram sinalizadas antes de distribuição.

## Observação jurídica

Esta issue define uma política técnica de redução de risco, não uma garantia jurídica de ausência de processo. Assets de Minecraft, mods e modpacks podem envolver direitos e licenças diferentes. Qualquer distribuição/comercialização deve respeitar os direitos e licenças aplicáveis.

## Relação com outras issues

Consultar especialmente as issues de TEXTURES, MOBS, VFX, ARMADURAS e o catálogo de prompts antes de gerar novos assets.

A meta é:

**'mesmo nível de riqueza visual, identidade completamente NEXORA.'**

---

**Source:** [GitHub Issue #45 — ART DIRECTION — ATM10 como referência visual, não como fonte de assets](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/45)
