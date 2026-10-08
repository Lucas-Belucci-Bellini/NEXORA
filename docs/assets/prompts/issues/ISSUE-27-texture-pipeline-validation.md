<!-- Imported from GitHub Issue #27. Original issue: https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/27 -->

# NEXORA — TEXTURE PIPELINE 16×16 / FECHAMENTO DA PRIMEIRA GERAÇÃO

## Objetivo
Criar o controle técnico necessário para que a biblioteca 16×16 possa realmente ser considerada fechada, reproduzível e integrada ao NEXORA.

Esta issue **não é para gerar novas categorias de arte**. Ela fecha o pipeline das issues de textura 16×16 existentes.

## Escopo
- catálogo mestre de assets
- convenção de nomes
- diretórios por família
- registro de resolução
- validação de dimensões
- validação de formato
- validação de transparência quando aplicável
- validação de tiling
- detecção de arquivos duplicados
- detecção de placeholders desconhecidos
- montagem/organização de atlas quando o sistema utilizar atlas
- importação no jogo
- teste visual dentro do NEXORA
- registro de prompt e proveniência
- status de cada asset

## Registro mínimo por asset
```text
asset_id
family
name
variant
resolution
file_path
format
alpha
seamless
source_tool
prompt_reference
generation_date
version
status
integration_status
```

## Prompt de validação para ferramenta/agente
```text
Validate this NEXORA asset as a first-generation 16×16 texture.
Requirements:
- exact 16×16 dimensions
- crisp pixel boundaries
- no accidental anti-aliasing
- no watermark
- no text unless explicitly required by the asset specification
- no third-party logo or copied design
- consistent palette and pixel density for its family
- no unexpected background when transparency is required
- correct file naming and directory placement
- no duplicate file identity
- suitable for the intended NEXORA material, icon, sprite, decal, or atlas usage
Report every failure instead of silently modifying an invalid asset.
```

## Regra de qualidade
Não aumentar resolução para corrigir um asset ruim. Primeiro corrigir a versão 16×16.

## Critério de fechamento
- [ ] catálogo mestre existe
- [ ] todas as issues 16×16 possuem famílias catalogadas
- [ ] cada asset tem ID/nome único
- [ ] cada asset possui resolução registrada
- [ ] arquivos inválidos são rejeitados
- [ ] placeholders desconhecidos foram eliminados ou explicitamente catalogados
- [ ] atlas/material importado corretamente
- [ ] assets principais aparecem no jogo
- [ ] proveniência registrada para os assets gerados
- [ ] pipeline reproduzível documentado

## Fora do escopo
```text
32×32
64×64
normal maps
roughness
height maps
PBR completo
```

Esses itens pertencem a gerações visuais futuras do NEXORA.


---

**Source:** [GitHub Issue #27 — TEXTURE PIPELINE 16×16 — Catálogo Mestre, Atlas, Validação e Proveniência](https://github.com/Lucas-Belucci-Bellini/NEXORA/issues/27)
