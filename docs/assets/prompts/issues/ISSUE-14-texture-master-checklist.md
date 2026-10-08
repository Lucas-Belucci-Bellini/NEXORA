<!-- Imported from GitHub Issue #14. Original issue: https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/14 -->

# NEXORA — TEXTURE MASTER CHECKLIST

## Objetivo
Fechar o problema de texturas do NEXORA até o final de 2026, mantendo o backlog separado por família e evitando recriação de assets que já existem.

## Já contabilizado — biblioteca inicial

```text
Pedras       16
Madeiras     16
Minérios     48
Areias       24
Vidros       30
Lãs          240
Banners      100
Mobs         600
Líquidos     80
Itens        2.500
-----------------
Total        3.654
```

Essas famílias possuem issues individuais (#4–#13).

## Conteúdo já existente adicional — NÃO DUPLICAR

Os seguintes grupos já existem no projeto e devem ser considerados cobertos para planejamento visual, mesmo que ainda precisem de validação/integração técnica:

```text
[x] armaduras
[x] armas
[x] escudos
[x] mesas de trabalho
```

Qualquer issue de equipamentos deve verificar primeiro o catálogo e os arquivos existentes antes de gerar novos assets.

## O que ainda precisa ser tratado como FAMÍLIA DE TEXTURA

### 1. Blocos e construção

```text
[ ] tijolos
[ ] concreto
[ ] cimento
[ ] pedra trabalhada
[ ] ladrilhos
[ ] azulejos
[ ] telhas
[ ] metais estruturais
[ ] chapas
[ ] vigas
[ ] cabos/condutos
[ ] blocos decorativos
[ ] pisos
[ ] paredes especiais
```
Issue: #15

### 2. Vegetação e natureza

```text
[ ] folhas
[ ] troncos especiais
[ ] arbustos
[ ] flores
[ ] plantas pequenas
[ ] gramíneas
[ ] musgos
[ ] fungos
[ ] raízes
[ ] culturas agrícolas
[ ] frutos
[ ] vegetação de ambientes extremos
```
Issue: #16

### 3. Terreno e ambiente

```text
[ ] terra seca
[ ] terra úmida
[ ] barro
[ ] lama
[ ] argila
[ ] cascalho
[ ] seixos
[ ] neve
[ ] gelo
[ ] gelo antigo
[ ] sal
[ ] cinzas
[ ] solo vulcânico
[ ] solo pantanoso
[ ] permafrost
[ ] solo alienígena
[ ] terreno dimensional
[ ] transições de biomas
```
Issue: #17

### 4. Tecnologia e indústria

```text
[ ] máquinas
[ ] componentes eletrônicos
[ ] placas/circuitos
[ ] telas técnicas
[ ] painéis
[ ] tubulações
[ ] cabos
[ ] conectores
[ ] baterias
[ ] reatores
[ ] componentes mecânicos
[ ] engrenagens
[ ] peças industriais
[ ] estruturas de fábrica
[ ] superfícies técnicas
```
Issue: #18

### 5. Equipamentos restantes

Não recriar os grupos já existentes.

```text
[x] armaduras — existentes
[x] armas — existentes
[x] escudos — existentes
[x] mesas de trabalho — existentes
[ ] ferramentas adicionais ainda não catalogadas
[ ] capacetes adicionais
[ ] mochilas
[ ] equipamentos de exploração
[ ] equipamentos industriais
[ ] equipamentos científicos
[ ] equipamentos médicos/suporte
[ ] equipamentos especiais
[ ] componentes visíveis de equipamento
[ ] tecidos técnicos
[ ] materiais protetivos
```
Issue: #19

### 6. Veículos

```text
[ ] veículos terrestres
[ ] veículos aéreos
[ ] veículos navais
[ ] trens
[ ] vagões
[ ] veículos industriais
[ ] veículos espaciais
[ ] estações
[ ] componentes de veículos
```
Issue: #20

### 7. Estruturas e arquitetura especial

```text
[ ] portas
[ ] janelas
[ ] portões
[ ] escadas
[ ] pisos especiais
[ ] paredes modulares
[ ] torres
[ ] pontes
[ ] instalações industriais
[ ] instalações científicas
[ ] instalações militares
[ ] bunkers
[ ] ruínas
[ ] estruturas subterrâneas
[ ] estruturas abandonadas
```
Issue: #21

### 8. História / Crônicas da Baluarte

```text
[ ] arquivos físicos
[ ] documentos visuais
[ ] mapas
[ ] placas
[ ] símbolos ficcionais
[ ] equipamentos abandonados
[ ] objetos históricos
[ ] ruínas específicas
[ ] instalações relacionadas às Crônicas
```
Issue: #22

### 9. Magia / dimensões / conteúdo extremo

```text
[ ] materiais mágicos
[ ] cristais
[ ] runas/símbolos originais
[ ] artefatos
[ ] estruturas mágicas
[ ] materiais dimensionais
[ ] materiais espaciais
[ ] tecnologia de endgame
```
Issue: #23

### 10. Efeitos visuais

```text
[ ] fogo
[ ] fumaça
[ ] poeira
[ ] faíscas
[ ] explosões
[ ] partículas ambientais
[ ] efeitos de energia
[ ] efeitos mágicos
[ ] impactos
[ ] dano visual de materiais
```
Issue: #24

### 11. Interface e sinalização

```text
[ ] ícones de inventário
[ ] ícones de ferramentas
[ ] ícones de status
[ ] ícones de materiais
[ ] símbolos de facções
[ ] símbolos de cidades
[ ] sinalização mundial
[ ] marcadores
[ ] mapas estilizados
```
Issue: #25

### 12. Decals e superfícies especiais

```text
[ ] faixas de segurança
[ ] marcações de piso
[ ] símbolos técnicos
[ ] padrões industriais
[ ] alertas gráficos
[ ] sujeira localizada
[ ] ferrugem localizada
[ ] arranhões
[ ] desgaste superficial
[ ] manchas
[ ] marcas de uso
[ ] padrões modulares
```
Issue: #26

## Pipeline final 16×16

```text
FAMÍLIA
↓
CATÁLOGO
↓
VERIFICAR O QUE JÁ EXISTE
↓
PROMPT BASE
↓
NANO BANANA / GERAÇÃO
↓
SELEÇÃO
↓
PADRONIZAÇÃO
↓
VALIDAÇÃO 16×16
↓
IMPORT
↓
TESTE NO NEXORA
↓
REGISTRO DE PROVENIÊNCIA
```

## Regra de resolução — 2026

```text
PRIMEIRA GERAÇÃO
→ 16×16
```

Não aumentar resolução para resolver um problema de qualidade nesta fase.

Gerações futuras:

```text
32×32
↓
64×64
↓
normal
↓
roughness
↓
height
↓
PBR / materiais avançados
```

## Critério de fechamento

```text
[ ] todas as famílias conhecidas possuem catálogo
[ ] assets existentes foram identificados e preservados
[ ] assets novos possuem identidade visual consistente
[ ] resolução-base 16×16 está definida
[ ] convenção de nomes está definida
[ ] origem/proveniência está registrada
[ ] assets principais estão dentro do jogo
[ ] faltas estão registradas em issues
[ ] nenhum sistema depende de textura placeholder desconhecida
[ ] backlog de versões futuras está catalogado
```

A meta de 2026 é **fechar a primeira geração visual 16×16**, não finalizar PBR ou resoluções superiores.

---

**Source:** [GitHub Issue #14 — TEXTURE MASTER CHECKLIST — Fechar backlog visual até 2026](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/14)
