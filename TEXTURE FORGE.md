# NEXORA — TEXTURE FORGE

A ferramenta oficial de geração, transformação, validação, organização,
versionamento e registro de materiais de superfície do NEXORA.

> O objetivo **não** é um script que cospe PNGs. Um material aqui é uma
> **entidade** — tem identidade, categoria, escala física, parâmetros PBR,
> revisão e origem — e os pixels são o que se deriva dela, não o contrário.

```sh
cargo run -p nexora-texture-forge -- batch content/manifests/temperate_forest.json
```

---

## 1. Onde ela mora, e por quê

```text
content/materials/*.json     ← o que uma pessoa escreve. Versionado, pequeno.
content/manifests/*.json     ← um conjunto inteiro num arquivo só.
        │
        ▼  nexora-texture-forge
assets/materials/            ← gerado. NÃO versionado: é derivado e reprodutível.
        └── <namespace>/<caminho>/
              albedo.png  normal.png  roughness.png  metallic.png
              height.png  ambient_occlusion.png  preview.png  material.json
```

`tools/texture-forge/` — **ferramenta de conteúdo, não crate de engine**. A
matriz de dependências (ADR-0003) põe em `engine/` o que o jogo linka; isto
não é. Depende de `nexora-asset` (os contratos) e `nexora-foundation`, e de
nada que precise de um jogo rodando.

`engine/asset/` — os contratos que o runtime **também** precisa: o que é um
material, o que é um mapa, o que é uma procedência, o que é uma validação, e
as duas traits que definem os pontos de extensão.

## 2. As regras que a ferramenta impõe

| regra | onde vive | o que acontece se violada |
| --- | --- | --- |
| Nada é sobrescrito em silêncio | `Forge::generate` | recusa por nome, dizendo qual campo mudou e qual flag permitiria |
| Textura inválida não é aceita | `Forge::generate` | **não é escrita**; escrever e reportar depois é aceitar |
| Origem não pode mentir | `GeneratedMaterial::assemble` | quarentena; nada chega ao disco |
| Um backend não libera o próprio conteúdo | idem | quarentena |
| Caminho de textura não é hardcoded no bloco | `SurfaceTable` | bloco → material id → registry → textura |
| Capacidade nova se declara, não se espalha | `MaterialRegistry` | um material `Blocked` é recusado no registro |

## 3. O modelo: material é entidade

```rust
SurfaceMaterial {
    id: Identifier,            // nexora:material/temperate_forest/oak_bark
    category: MaterialCategory, // 15 famílias
    revision: Revision,
    resolution: Resolution,     // potências de dois, 4..8192
    physical_scale: PhysicalScale, // metros por ladrilho
    seamless: bool,
    blend: BlendMode,           // opaque | cutout | transparent | emissive | translucent
    pbr: PbrParameters,
    wanted_maps: Vec<MapRole>,
    provenance: Provenance,     // obrigatória: não dá para construir sem
}
```

**A definição não carrega pixels.** O caminho de runtime é
`ResourceID → Manifest → Resolver → Loader → Cache → Runtime Handle`: o que se
registra é uma identidade, os bytes chegam depois, por um cache que pode
despejá-los. Um registro segurando pixels congelaria toda textura em memória.

**Por que `SurfaceMaterial` e não `Material`:** `nexora_physics::MaterialId` já
significa atrito, restituição, densidade e arrasto. As regras de nomenclatura
proíbem um nome cobrindo dois conceitos. Um bloco pode ter os dois; eles nunca
se encontram.

## 4. Os três modos diferem em uma coisa só: de onde vem a semente

| modo | semente |
| --- | --- |
| `generate` | derivada da identidade e da aparência do próprio material |
| `variant` | derivada da identidade **da fonte** e do índice da variante |
| `repair` | lida **literalmente** do registro da geração sendo reparada |

Tudo depois da semente é um caminho de código só. Um modo que renderizasse por
um segundo caminho seria um segundo renderizador para manter em sincronia.

- **Variante** semeia pela fonte, então *"variante 3 do carvalho"* nomeia uma
  superfície específica seja lá como o resultado se chame. Semear pelo nome
  próprio significaria que renomear um material o repinta.
- **Reparo** não re-deriva: re-derivar só coincide com o original por acidente
  de nada ter mudado. Ler a semente registrada torna o mapa restaurado idêntico
  **por construção**. Recusa quando não há registro, quando outro gerador fez o
  material, ou quando a versão do algoritmo não é esta.
- **Reparo não conserta receita ruim.** Um mapa presente, legível e reprovando
  numa checagem regenera para o mesmo mapa reprovado. Esses ficam onde estão.

## 5. Backends

| backend | reprodutível do próprio registro | instalado |
| --- | --- | --- |
| `procedural` | **sim** | sim |
| `ai` | não | não |
| `hybrid` | não | não |

Um backend sem gerador instalado é **recusado pelo nome** (saída 1), nunca
servido em silêncio pelo procedural — quem pediu um modelo e recebeu ruído
publicaria o ruído. Um nome que não é backend nenhum sai com 2.

O ponto de extensão é `TextureGenerator`, e ele é exercitado: os testes
instalam um segundo backend declarando `Backend::Ai` e rodam **todo** o
pipeline por ele — gerar, escrever, validar, listar, batch, variante, reparo e
um preview renderizado de mapas que outro gerador desenhou. Um teste
companheiro exige que os dois produzam pixels diferentes, para que o primeiro
não passe com um gerador que é secretamente o mesmo.

## 6. Procedência, e o que ela impede

Toda saída carrega um `GenerationTrace`: gerador, versão, **backend**, pipeline,
versão do pipeline, preset, semente, prompt, parâmetros e entradas. O backend é
um **campo**, não uma entrada no mapa de parâmetros: o que decide se um asset
pode ser publicado não é uma string que um gerador qualquer pode esquecer de
escrever.

Um backend que não é reprodutível do próprio registro:

- **não pode** alegar `procedural_derivative` — não foi um algoritmo que o
  derivou de parâmetros;
- **não pode** alegar `nexora_original` — não foi este repositório que computou;
- **não pode** chegar já liberado — liberação nomeia um revisor, e um gerador
  liberando a própria saída é um gerador se revisando.

As três recusas são **quarentena**, não rejeição: os pixels podem estar bons, mas
um asset cuja origem não se estabelece não entra em lugar nenhum até uma pessoa
olhar.

### Originalidade

Tudo que o backend procedural emite é **computado de números**. Nenhuma imagem
externa é decodificada, embutida, amostrada ou publicada — a saída é
`NEXORA_ORIGINAL` / `PROCEDURAL_DERIVATIVE` por construção, não por afirmação.
Uma imagem de referência pode ser *estudada* para escolher os números, que é o
que a política de conteúdo original permite: inspiração arquitetural se estuda;
implementação, assets e expressão protegida distintiva se criam
independentemente.

## 7. Validação

Catorze checagens nomeadas, cada uma com PASS / WARN / FAIL e uma explicação.
A medida de costura levou três tentativas e tem um controle que falha de
propósito — 600 amostras que ladrilham chegam no máximo a **1,082**; ruído
não-periódico dá **2,41–2,78**; recortes chegam a **7,9**. O limite é **1,5**.

Um material que reprova **não é escrito**.

## 8. Manifestos

Um arquivo declara um conjunto inteiro. `prefix` vira segmento de caminho, então
**agrupar já era dar namespace**: `nexora:material/temperate_forest/oak_bark`.
Identificador já aninha, o layout já vira diretório, o registro já ordena por
identificador. E `forest/soil` e `desert/soil` deixam de colidir.

Uma entrada herda `defaults` e sobrescreve só o que nomeia — inclusive **um**
campo dentro de `pbr`. Uma entrada ruim não para as outras: o batch segue,
coleta cada falha ao lado do id, e sai com código ≠ 0.

Detalhes e limites: [`content/manifests/README.md`](content/manifests/README.md).

## 9. Preview

Um render iluminado, **ladrilhado 2×2** — é ali que está o defeito. Um ladrilho
só esconde a costura; dois põem as duas costuras pelo meio da imagem. A luz é
constante: preview iluminado diferente a cada vez não é preview, é humor.

Não é um renderizador. É Lambert mais um lóbulo Blinn-Phong, em luz linear, com
oclusão dobrada no ambiente. Perto o bastante de que um normal map invertido ou
um roughness chapado fique óbvio, que é o trabalho inteiro.

Desliga com `--no-preview`. O custo foi medido: previews somam **40%** em cima
dos mapas — ~62 MB contra ~156 MB por dez mil materiais.

## 10. Linha de comando

```text
nexora-texture-forge <comando> [opções]

  generate <definition.json>   realiza um material e o escreve
  batch    <manifest.json>     realiza tudo que um manifesto declara
  variant  <definition.json>   outro material parecido com um já escrito
  repair   <material-id>       restaura mapas perdidos ou corrompidos
  validate <material-id>       confere o que está no disco
  inspect  <material-id>       imprime definição e origem
  list                         tudo que está escrito sob a raiz

  --out <dir>          onde os materiais moram (padrão: assets/materials)
  --seed <valor>       semente, decimal ou 0x (padrão: 0)
  --force              substitui um material já escrito
  --of <material-id>   de qual material a variante deriva (obrigatório)
  --index <n>          qual variante, contando de um (padrão: 1)
  --no-preview         pula o preview iluminado
  --backend <nome>     procedural | ai | hybrid (padrão: procedural)

  0  o comando funcionou
  1  o comando rodou e a resposta foi não
  2  a linha de comando estava errada
```

## 11. Números medidos

Medidos, não estimados. Cada um tem o teste ou o script que o produziu.

| o quê | quanto |
| --- | --- |
| material 64² com 5 mapas + definição | **17,8 KiB** |
| o mesmo, com preview | **23,2 KiB** |
| conjunto de exemplo: 8 materiais, 40 mapas | **121,7 KiB** |
| o mesmo, com os 8 previews | **170,1 KiB** |
| dez mil materiais, só mapas | ~**156 MB** |
| previews, em cima disso | +**40%** (~62 MB) |
| compressão alcançável (zlib -9) no conjunto PBR | **6,79×** |
| compressão do compressor daqui | **4,73×** (1,40× maior que zlib -9) |

> O commit da FASE 9 diz **69%** e ~107 MB para os previews. Aquele número foi
> medido *antes* das duas correções de sombreamento e ficou obsoleto no mesmo
> dia: o render corrigido está muito mais perto do albedo, e o albedo comprime
> bem. O valor certo é o desta tabela, **40%**.

O compressor próprio existe porque blocos armazenados davam 1,00–1,03× e os
mapas comprimem 12× com zlib. A diferença que sobra está registrada como
DEBT-0034 (lazy matching primeiro, Huffman dinâmico depois).

## 12. Zero dependências

`unsafe_code = "forbid"` no workspace, **nenhum crate externo** (ADR-0002). O
PNG, o DEFLATE, o inflate, o ruído, o filtro e o render são todos daqui. O
CRC-32 do PNG já existia em `nexora_foundation` — é CRC-32/ISO-HDLC, verificado
contra `0xCBF43926`; só faltava o Adler-32.

A verificação que importa: os PNGs escritos são decodificados pelo **`zlib` do
Python** — um decodificador que não é meu — com todo CRC de chunk correto e toda
scanline desfiltrando limpa. A CI roda isso.

## 13. Documentos

- [`docs/texture-forge/AUDITORIA.md`](docs/texture-forge/AUDITORIA.md) — a
  auditoria que a §25 exigiu **antes** de qualquer código, e os três achados que
  remodelaram o plano
- [`content/materials/README.md`](content/materials/README.md) — o estágio SOURCE
- [`content/manifests/README.md`](content/manifests/README.md) — conjuntos
- ADR-0002 (zero dependências) · ADR-0003 (matriz de dependências)

## 14. O que ainda não existe

Deliberadamente, e registrado em vez de esquecido:

- **Nenhum backend de IA está instalado.** O ponto de extensão está provado; o
  gerador, não. Instalar um exige decidir modelo, hospedagem, custo e — o mais
  caro — o que a procedência de um modelo treinado em dados desconhecidos pode
  honestamente afirmar.
- **Presets são código, não dados.** As catorze famílias vivem em `recipe.rs`.
  Externalizá-las é útil quando alguém de fora precisar ajustá-las.
- **A receita de `metal` usa a mesma forma de listras da madeira.** Funciona —
  lê como metal escovado — mas não é uma forma própria.
