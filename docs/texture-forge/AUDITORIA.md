# TEXTURE FORGE — Auditoria do repositório e arquitetura proposta

- **Data:** 2026-09-08
- **Origem:** §25 do briefing *NEXORA TEXTURE FORGE* — *"NÃO comece escrevendo
  código imediatamente"*
- **Estado:** proposta. Nenhuma linha do Texture Forge foi escrita antes deste
  documento.

Este documento responde aos seis pontos exigidos pela §25, nesta ordem:
auditoria, sistemas relacionados, pontos de integração, conflitos
arquiteturais, arquivos a modificar, arquitetura proposta.

---

## 1. Auditoria — o que existe de verdade

O NEXORA hoje são **13 crates Rust**, ~33 mil linhas, 628 testes, `unsafe_code
= "forbid"` no workspace inteiro e **zero dependências externas** (ADR-0002).
O grafo de crates é a própria matriz de dependências (ADR-0003): Cargo recusa
ciclos, então a matriz é verificada pelo build e não pela revisão.

O que já existe e é **diretamente reaproveitável** pelo Texture Forge:

| peça | onde | o que é de fato |
| --- | --- | --- |
| `Identifier` (`namespace:path`) | `nexora_foundation::ident` | a única forma legítima de nomear conteúdo. Path `[a-z0-9_/.-]`, namespace `[a-z0-9_]`, ordenação estável |
| `Registry<T>` | `nexora_runtime::registry` | genérico, com `register` / `freeze` / `require` / `fingerprint`. Recusa duplicata em vez de sobrescrever |
| `RuntimeId(u32)` | idem | handle da sessão. **Nunca** vai para disco — o disco guarda o `Identifier` |
| `SurfaceId(u32)` | `nexora_mesh::mesh` | *"o que a face mostra"*. Handle opaco, o chamador decide o que o número significa |
| `WorldSurfaces` | `nexora_simulation::surfaces` | o adaptador que hoje devolve `SurfaceId(block_state.0)` — identidade |
| `crc32` | `nexora_foundation::hashing` | CRC-32/ISO-HDLC, verificado contra o vetor `0xCBF43926`. **É exatamente o CRC que o PNG exige** |
| `Fnv1a64` | idem | hash de conteúdo, com prefixo de comprimento contra colisão por concatenação |
| `Rng` / `SeedStream` | `nexora_foundation::rng` | SplitMix64 determinístico, `fork(label)`, streams nomeados |
| `contract_version!` | `nexora_foundation::version` | macro que declara um contrato versionado (`SaveFormatVersion`, `CommandVersion`, …) |
| `Error` / `Domain` / `Recovery` | `nexora_foundation::error` | `Domain::Content` já existe e é descrito como *"Content, assets and resource packs"* |
| `Writer` / `Reader` | `nexora_persistence::codec` | codec binário |
| convenção de CLI | `TOOLING ARCHITECTURE.md` + binários reais | `nexora <substantivo> <verbo>`, flags `--flag valor`, binários `nexora-<nome>` |

O que **não** existe: qualquer sistema de textura, material visual, atlas,
asset, PNG, imagem ou pipeline de conteúdo. O Texture Forge não vai substituir
nada — não há nada para substituir.

---

## 2. Sistemas relacionados — o que os documentos já decidiram

Sete documentos do repositório já legislam sobre este assunto. Nenhum deles é
opcional, e três deles **já contêm** partes do que o briefing pede.

**`Block System.md` §21 (Textures) — literalmente o §10 do briefing:**

> *"Texture não deve estar hardcoded no Block System. `AssetID`. Exemplo:
> `nexora:block/stone`. Renderer resolve o asset."*

E §153: *"Asset references — textures, models, sounds, particles, animations —
ficam fora do Block System. Ele apenas referencia os assets."*

**`Registry System.md` §3** lista `Materials` entre as coisas registráveis, e
**§70** diz que registros especializados *"ainda utilizam a infraestrutura
genérica"*. Ou seja: material vai num `Registry<T>`, não num registro novo.

**`RENDERER and GRAPHICS.md` RENDER-13/14/15/54** define `Material { shader,
textures, parameters, blendMode, cullingMode, renderingFlags }`, os tipos
`Opaque / Cutout / Transparent / Emissive / Translucent`, o atlas de voxel e —
RENDER-54 — materiais procedurais *"para reduzir dependência de texturas
únicas"*.

**`NEXORA CONTENT PIPELINE SPECIFICATION.md`** fixa o pipeline
`SOURCE → IMPORT → NORMALIZE → VALIDATE → BUILD/COOK → COMPRESS → PACKAGE →
INDEX → RUNTIME RESOURCE` e a regra de que *"nome de arquivo não deve ser o
identificador lógico primário"*.

**`NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md`** — ver §4.3 abaixo. Já tem
o modelo de proveniência inteiro, mais completo do que o do briefing.

**`NEXORA ART DIRECTION AND PROCEDURAL VARIATION.md`** dá o modelo de ativo
(`BASE ASSET + MATERIAL PARAMETERS + PROCEDURAL VARIATION + WORLD CONTEXT +
INSTANCE STATE`) e a meta explícita: *"preferir uma biblioteca pequena de
primitivas originais de alta qualidade mais variação procedural controlada a
duplicação massiva de arquivos quase idênticos"* — que é, palavra por palavra, o
motivo pelo qual o operador pediu o Texture Forge.

**`NEXORA NAMING AND TERMINOLOGY.md`** define `Resource = recurso identificado`
e `Asset = conteúdo consumível pelo runtime`, e proíbe que *"conceitos
diferentes usem o mesmo nome"*. Essa regra tem consequência direta — ver §4.1.

---

## 3. Pontos de integração

Existem exatamente **quatro** costuras, e três delas já estão prontas para
receber o Texture Forge sem alteração de arquitetura.

```text
1. Identificação   Identifier            nexora:material/stone_rough
                     ↓
2. Registro        Registry<T>           MaterialRegistry = Registry<SurfaceMaterial>
                     ↓
3. Handle          SurfaceId             o id de runtime do material
                     ↓
4. Geometria       Quad.surface          o mesher já carrega o campo
```

A costura **3** é a descoberta principal desta auditoria. `SurfaceId` foi
escrito na semana passada como *"handle opaco: o chamador decide o que o número
significa"*. Hoje o `WorldSurfaces` devolve `SurfaceId(block_state.0)` — o
próprio estado do bloco, por falta de qualquer outra coisa. **O Texture Forge é
a coisa que dá significado a esse número.** Não é preciso inventar um handle
novo; ele já existe, vazio, esperando.

A quarta costura é o mapeamento bloco → material, e ela também já tem
precedente: `engine/simulation/src/terrain.rs` mantém uma **tabela lateral**
`assign_material(block: &Identifier, material: MaterialId)` que dá a um bloco
seu material *físico* sem tocar no `BlockDefinition`. A resposta do NEXORA para
"um bloco precisa de um atributo que outro sistema possui" já foi dada uma vez,
e é essa. O material de superfície segue o mesmo caminho.

---

## 4. Conflitos arquiteturais

### 4.1 `Material` já existe, e significa outra coisa

`nexora_physics::material::MaterialId` existe desde a fase da física e
significa **atrito, restituição, densidade e arrasto**. `PhysicsMaterial` é uma
superfície *de contato*, não de aparência.

`NEXORA NAMING AND TERMINOLOGY.md` é explícito: *"conceitos diferentes não usam
o mesmo nome"*. Chamar o material visual de `Material` criaria dois conceitos
com um nome só, em duas crates que um dia se encontram no mesmo arquivo.

**Resolução:** o material visual é `SurfaceMaterial`, e seu handle de sessão é
o `SurfaceId` que o mesher já define. O físico continua `PhysicsMaterial` /
`MaterialId`. Um bloco pode ter os dois, e eles não se confundem em nenhum
ponto do código.

### 4.2 Zero dependências externas (ADR-0002) contra "gere PNG e material.json"

O briefing pede PNG e JSON. O workspace não pode adicionar crate nenhuma.

- **PNG:** um PNG válido é assinatura + `IHDR` + `IDAT` + `IEND`, cada chunk com
  **CRC-32/ISO-HDLC** — que `nexora_foundation::hashing::crc32` já implementa e
  já valida contra o vetor publicado. O `IDAT` é um fluxo zlib, que aceita
  **blocos DEFLATE armazenados** (`BTYPE=00`), legais e lidos por qualquer
  decodificador. Falta apenas o Adler-32, que são doze linhas. **Não há
  dependência a adicionar: só falta escrever.** O custo é tamanho de arquivo, e
  ele será medido, não estimado (ver §6, FASE 3).
- **JSON:** `Block System.md` §51 (*Data-Driven Blocks*) já mostra a definição
  de um bloco em JSON. O formato não está sendo inventado para o Texture Forge;
  ele já está previsto na arquitetura, e o Texture Forge é o primeiro
  consumidor. Um leitor/escritor de um subconjunto estrito, determinístico e
  ordenado é código contido e testável.

### 4.3 O briefing propõe uma lista de proveniência que o repositório já tem — maior

O §17 pede `GENERATED / DERIVED / IMPORTED / PROCEDURAL / AI_GENERATED /
HYBRID`. O repositório já define, em dois documentos:

```text
ProvenanceClass   ORIGINAL · OWNED_SOURCE · LICENSED_THIRD_PARTY · PUBLIC_DOMAIN
                  EDITOR_GENERATED · PROCEDURAL_DERIVATIVE · EXPERIMENTAL · REJECTED

ReleaseStatus     DRAFT · REVIEW · CLEARED · RESTRICTED · BLOCKED · REMOVED

AssetStatus       NEXORA_ORIGINAL · NEXORA_DERIVED_FROM_OWN_SOURCE
                  THIRD_PARTY_LICENSED · EDITOR_ONLY · EXPERIMENTAL · BLOCKED
```

mais o campo obrigatório de licença, atribuição, revisor e data de revisão.

O §23 do briefing proíbe duplicar o Registry System. A mesma regra vale aqui, e
com mais força: uma terceira lista de proveniência é a maneira mais rápida de
tornar impossível responder *"esse asset pode ser distribuído?"*.

**Resolução:** o Texture Forge **adota a lista existente**. `AI_GENERATED` não
vira classe: vira o campo `SourceTool` que o próprio documento já exige
(*"When a tool contributes to an asset, record the tool/workflow"*). Um material
gerado proceduralmente pelo forge é `PROCEDURAL_DERIVATIVE` + `NEXORA_ORIGINAL`
+ `SourceTool: nexora-texture-forge@<versão>`.

### 4.4 Uma ferramenta não é um crate de engine

`NEXORA DEPENDENCY MATRIX.md` põe `Editor`/`Tools` no fim da cadeia, dependendo
de *"public runtime contracts, tools APIs"* e nunca de internos privados. Um
codificador de PNG dentro de `engine/` seria uma inversão dessa direção.

**Resolução:** o sistema se parte em dois, exatamente como a física se partiu em
`nexora-physics` + adaptador (ADR-0007) e o mesher em `nexora-mesh` +
`WorldSurfaces` (ADR-0012):

- o que o **runtime** precisa saber (definição, registro, proveniência,
  resultado de validação) vira `engine/asset`;
- o que só a **ferramenta** faz (gerar pixels, escrever PNG, CLI, presets,
  batch, preview) vira `tools/texture-forge`.

### 4.5 Semente de textura ≠ semente de mundo

`SeedStream` é `#[non_exhaustive]` e enumera domínios de **geração de mundo**
(`Terrain`, `Biome`, `Cave`, …). Textura não é um deles: o forge roda offline,
antes de existir mundo, e seu determinismo é sobre a *definição do material*,
não sobre uma semente de mundo.

**Resolução:** não adicionar `SeedStream::Texture`. O forge deriva sua RNG de
`Fnv1a64(id do material + versão + parâmetros)`. Isso é o que faz duas execuções
da mesma definição produzirem bytes idênticos — e é o que dá sentido ao hash de
proveniência. A variação por instância no mundo (o modelo da art direction) é
outra camada, em runtime, com a semente do mundo.

### 4.6 Conteúdo externo é entrada não confiável

`NEXORA SECURITY THREAT MODEL.md` e a §29 do prompt-mestre tratam todo conteúdo
externo como potencialmente hostil. Um decodificador de imagem é um parser de
bytes arbitrários — a superfície clássica.

**Resolução:** o caminho de **geração** nunca faz parse de dado externo: ele
produz pixels a partir de números. O caminho de **importação** (classe
`IMPORTED`) é separado, explícito, com limites de tamanho e dimensão, e **não
entra nas primeiras fases**. A fronteira fica escrita antes de existir o
código que a atravessa.

---

## 5. Arquivos que precisam ser modificados

O Texture Forge é quase todo código novo. O que ele **toca** no que existe é
deliberadamente pequeno:

| arquivo | mudança | por quê |
| --- | --- | --- |
| `Cargo.toml` (workspace) | dois membros e duas entradas em `workspace.dependencies` | ADR-0003: o grafo é a matriz |
| `engine/simulation/src/surfaces.rs` | `WorldSurfaces` passa a consultar a tabela de superfície em vez de devolver `SurfaceId(state.0)` | fecha a costura 3; hoje dois blocos com a mesma textura não podem fundir |
| `engine/simulation/src/terrain.rs` *(ou vizinho)* | tabela lateral bloco → material de superfície | mesmo padrão de `assign_material` para material físico |
| `NEXORA ADR INDEX.md` + `docs/adr/README.md` | ADR nova | regra do índice |
| `NEXORA TECHNICAL DEBT REGISTER.md` | o que ficou de fora, com gatilho | regra 1 do registro |
| `.gitignore` | saída do forge | §23: não colocar arquivo gigante no Git |

**Nenhum arquivo é apagado. Nenhum sistema é substituído.** `BlockDefinition`
continua com um campo só, como `CORE.md` §5 pede.

---

## 6. Arquitetura proposta

```text
                       tools/texture-forge          (a ferramenta)
                       ┌──────────────────────────────────────┐
   presets ──────────► │ generator ─► pipeline ─► encoder      │
   manifest ─────────► │     │            │          │         │
                       │     │            ▼          ▼         │
                       │     │        validator    PNG/preview │
                       └─────┼────────────┼────────────────────┘
                             │            │
                             ▼            ▼
                       engine/asset                 (o contrato)
                       ┌──────────────────────────────────────┐
                       │ SurfaceMaterial · TextureMap          │
                       │ Provenance · TextureValidationResult  │
                       │ MaterialRegistry = Registry<T>        │
                       └──────────────────┬───────────────────┘
                                          │
                                          ▼
                       engine/simulation            (a composição)
                       ┌──────────────────────────────────────┐
                       │ bloco ──► material ──► SurfaceId      │
                       │            (tabela lateral)           │
                       └──────────────────┬───────────────────┘
                                          ▼
                                    nexora-mesh
                                    Quad.surface
```

### Crates

**`engine/asset`** — depende de `foundation` + `runtime`. Contém o que o
runtime precisa saber e mais nada: a definição do material, o mapa de textura e
seu papel, o registro de proveniência, o resultado de validação, e o codec JSON.
Não sabe gerar pixel nenhum, não sabe o que é um PNG.

**`tools/texture-forge`** — depende de `nexora-asset` + `foundation`. Contém
gerador, pipeline, presets, validador de imagem, codificador PNG, preview, CLI e
batch. **Não depende de `world`, `simulation`, `physics` ou `streaming`** — uma
ferramenta de conteúdo não precisa de um mundo, e a matriz não permitiria.

A fronteira entre as duas é a mesma usada quatro vezes: um trait declarado do
lado de baixo, o adaptador do lado que pode ver os dois.

```rust
// engine/asset — o contrato. Quem gera não é problema de quem consome.
pub trait TextureGenerator {
    fn id(&self) -> &Identifier;
    fn version(&self) -> GeneratorVersion;
    fn generate(&self, request: &GenerationRequest) -> Result<GeneratedMaterial>;
}
```

Um backend de IA, uma API de imagem, um Stable Diffusion local ou outro gerador
procedural entram implementando esse trait — **sem reescrever o core**, que é
exatamente o que o §4 do briefing exige. E como o trait devolve
`GeneratedMaterial`, que carrega `Provenance`, é impossível um backend novo
produzir asset sem origem registrada: a §17 vira um erro de compilação, não uma
regra de revisão.

### Nomes (contra a §4.1)

```text
SurfaceMaterialId ──── não existe: é o SurfaceId que o mesher já define
SurfaceMaterial ────── a definição visual (albedo, PBR, escala física, seamless)
PhysicsMaterial ────── intocado, continua sendo atrito e restituição
TextureMap ─────────── um mapa e seu papel (Albedo, Normal, Roughness, …)
MapStatus ──────────── Required · Optional · Generated · Missing · Invalid  (§6)
Provenance ─────────── as classes que os documentos do repositório já definem
```

### CLI

`TOOLING ARCHITECTURE.md` usa `nexora <substantivo> <verbo>` e os binários
existentes são `nexora-headless` e `nexora-benchmark`. Logo:

```text
nexora-texture-forge generate  <material.json>
nexora-texture-forge variant   <id> --count 8
nexora-texture-forge repair    <id>
nexora-texture-forge validate  <id|caminho>
nexora-texture-forge inspect   <id>
nexora-texture-forge list
nexora-texture-forge batch     <manifest.json>
```

### Ordem de execução (as dez fases do briefing, ancoradas no que existe)

| fase | entrega | prova |
| --- | --- | --- |
| 1 | contratos em `engine/asset` | round-trip de serialização, id estável |
| 2 | gerador procedural | mesma definição → bytes idênticos |
| 3 | pipeline PBR + PNG | PNG relido por um decodificador independente; **tamanho medido** |
| 4 | validador (inclui seamless) | textura quebrada é reprovada, não aceita em silêncio |
| 5 | registro + bloco → material | `WorldSurfaces` passa a usar material; mesher funde por material |
| 6 | CLI | cada subcomando com teste |
| 7 | batch por manifesto | manifesto inválido falha alto |
| 8 | variante e reparo | variante ≠ original, mesmo preset |
| 9 | preview | preview determinístico |
| 10 | abstração de backend (IA/híbrido) | um backend falso prova que o core não muda |

Cada fase é **um commit**, com teste, `fmt`, `clippy -D warnings` e build antes
do push, como manda a §22 e como o CI já exige.

---

## 7. O que esta arquitetura deliberadamente **não** faz agora

Registrado aqui para virar debt com gatilho, não para ser esquecido:

- **não** decodifica imagem externa (fronteira da §4.6 escrita antes do código);
- **não** comprime o PNG além de blocos armazenados — até o tamanho ser medido;
- **não** dá textura por face (`SurfaceId` é por célula hoje: grama com topo e
  lado diferentes é uma limitação real do mesher, não do forge);
- **não** cria atlas nem array de textura (RENDER-15) — não há renderizador;
- **não** liga o forge ao seed de mundo (§4.5);
- **não** adiciona backend de IA de verdade — só a fronteira que permite um.

---

## 8. Resposta à imagem de referência

O operador anexou uma textura 1024×1024 em tons de marrom, com faixas verticais
tipo tábua/casca, e perguntou se dá para replicar o estilo.

Dá, e é exatamente a FASE 2 — sem incorporar nada externo. O desenho se
decompõe em operações que o gerador procedural executa a partir de números:
partição em faixas de largura variável, variação de valor por faixa, ruído
longitudinal para o grão, escurecimento nas bordas de cada faixa e quantização
da paleta para o visual de pixel art. O resultado é `NEXORA_ORIGINAL` +
`PROCEDURAL_DERIVATIVE`: nenhum pixel da imagem de referência entra no
repositório, e o `SourceTool` registrado é o próprio forge.

A imagem serve como **alvo de estilo**, que é o uso que a
`NEXORA ORIGINAL CONTENT AND ASSET POLICY.md` permite explicitamente:
*"architectural inspiration may be studied; implementation, assets and
distinctive protected expression must be independently created"*.
