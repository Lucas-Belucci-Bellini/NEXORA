# NEXORA WORLD GENERATION

# MASTER PLAN — SISTEMA DE GERAÇÃO DE MUNDO

> O World Generation System é um dos sistemas centrais do NEXORA.
>
> Seu objetivo não é apenas criar terrenos proceduralmente. Ele deve gerar um mundo coerente, persistente, explorável e capaz de produzir geografia, recursos, ecossistemas, assentamentos, infraestrutura e regiões especiais.
>
> O sistema deve ser determinístico, modular, extensível e preparado para as 16 dimensões do NEXORA.

---

# 1. VISÃO

O mundo do NEXORA não será apenas uma coleção de chunks.

A geração deve produzir:

```text
GEOGRAFIA
+
CLIMA
+
BIOMAS
+
RECURSOS
+
CAVERNAS
+
OCEANOS
+
VEGETAÇÃO
+
ESTRUTURAS
+
ASSENTAMENTOS
+
NPCs
+
ECONOMIA
+
INFRAESTRUTURA
+
FRONTEIRAS
+
DIMENSÕES
```

Portanto:

```text
World Generation
+
World Simulation
```

devem ser projetados para trabalhar juntos.

---

# 2. OBJETIVO PRINCIPAL

O gerador deve transformar:

```text
SEED
+
WORLD CONFIGURATION
+
WORLD VERSION
+
DIMENSION
+
COORDINATES
```

em:

```text
WORLD STATE
```

de maneira determinística.

Mesma seed + mesma versão + mesmas regras:

```text
→ mesmo mundo
```

quando as condições de geração forem equivalentes.

---

# 3. PIPELINE PRINCIPAL

O pipeline oficial será:

```text
SEED
 ↓
WORLD PARAMETERS
 ↓
WORLD TOPOLOGY
 ↓
CONTINENT / MACRO TERRAIN
 ↓
TECTONIC / GEOLOGICAL HISTORY
 ↓
TERRAIN SHAPE
 ↓
EROSION
 ↓
HYDROLOGY
 ↓
CLIMATE
 ↓
BIOMES
 ↓
3D DENSITY FIELD
 ↓
CAVES
 ↓
AQUIFERS
 ↓
OCEANS / RIVERS / LAKES
 ↓
SURFACE MATERIALS
 ↓
SUBSURFACE MATERIALS
 ↓
ORE GENERATION
 ↓
VEGETATION
 ↓
STRUCTURES
 ↓
SETTLEMENTS
 ↓
NPC POPULATION
 ↓
ECONOMY
 ↓
ROADS / RAIL / INFRASTRUCTURE
 ↓
SPECIAL REGIONS
 ↓
FAR LANDS
 ↓
BEYONDLANDS
 ↓
DIMENSIONAL FRONTIER
 ↓
CHUNK DATA
 ↓
CHUNK MESH
```

---

# 4. WORLD PARAMETERS

Antes do terreno:

```text
WorldParameters
```

deve definir:

* seed;
* tamanho lógico do mundo;
* escala;
* altura mínima;
* altura máxima;
* clima global;
* nível dos oceanos;
* frequência de estruturas;
* densidade populacional;
* recursos;
* regras da dimensão;
* versão do generator.

---

# 5. WORLD VERSION

A geração deve possuir versão.

Exemplo:

```text
WorldGen 1.0
WorldGen 1.1
WorldGen 2.0
```

Um save deve registrar:

```text
worldGenVersion
```

Isso permitirá futuras alterações sem perder a capacidade de interpretar mundos antigos.

---

# 6. DETERMINISMO

O sistema deve ser determinístico.

Evitar:

```text
Math.random()
```

sem seed.

Usar um RNG determinístico derivado da seed e do contexto.

Exemplo conceitual:

```text
worldSeed
+
dimensionId
+
chunkX
+
chunkZ
+
generationStage
```

gera um estado determinístico.

---

# 7. SEED DERIVATION

Criar um sistema:

```text
SeedContext
```

que permita gerar seeds independentes para:

```text
terrain
biome
ores
structures
vegetation
settlements
```

Sem que alterar a geração de vegetação modifique todo o terreno.

---

# 8. CHUNK GENERATION

Cada chunk possui pipeline próprio.

```text
Chunk
 ↓
height / density
 ↓
terrain
 ↓
biome
 ↓
materials
 ↓
caves
 ↓
ores
 ↓
vegetation
 ↓
structures
 ↓
settlements
 ↓
entities
```

A geração deve poder acontecer de forma incremental.

---

# 9. MULTI-STAGE GENERATION

Não fazer uma função gigantesca:

```text
generateWorld()
```

Criar estágios:

```text
MacroTerrainStage
TerrainStage
HydrologyStage
ClimateStage
BiomeStage
DensityStage
CaveStage
OreStage
VegetationStage
StructureStage
SettlementStage
EntityStage
InfrastructureStage
```

---

# 10. WORLD GENERATOR API

Criar uma API comum:

```text
WorldGenerator
```

Cada dimensão poderá fornecer seu próprio generator.

Exemplo:

```text
OverworldGenerator
FrontierGenerator
DimensionGenerator
SpaceGenerator
```

---

# 11. MACRO TERRAIN

Antes de gerar cada chunk:

criar uma visão global do terreno.

Determinar:

* continentes;
* arquipélagos;
* oceanos;
* cadeias montanhosas;
* grandes planícies;
* desertos;
* regiões costeiras.

O chunk deve conhecer seu contexto regional.

---

# 12. CONTINENTS

Criar:

```text
ContinentMap
```

Cada continente poderá possuir:

* tamanho;
* forma;
* altitude;
* clima;
* geologia;
* rios;
* recursos;
* densidade de população.

---

# 13. GEOLOGIA

Criar uma camada geológica.

Exemplo:

```text
Geology
├── plates
├── regions
├── rockTypes
├── mineralZones
└── geologicalAge
```

Isso poderá influenciar:

* montanhas;
* cavernas;
* minérios;
* solo;
* estruturas naturais.

---

# 14. EROSION

O terreno deve possuir aparência natural.

Criar modelos de:

* erosão;
* sedimentação;
* encostas;
* vales;
* cânions;
* planaltos;
* bacias.

Evitar apenas empilhar noise layers sem relação.

---

# 15. HYDROLOGY

Criar:

```text
HydrologySystem
```

gerando:

```text
oceans
rivers
lakes
streams
watersheds
```

Os rios devem possuir relação com:

```text
altitude
rainfall
terrain
basins
```

---

# 16. CLIMATE

Criar mapas climáticos.

Variáveis possíveis:

```text
temperature
rainfall
humidity
wind
seasonality
```

Esses valores alimentarão os biomas.

---

# 17. BIOME SYSTEM

Criar:

```text
BiomeRegistry
```

Cada biome terá:

```text
id
climateRange
terrainRules
surfaceRules
vegetationRules
entityRules
structureRules
```

---

# 18. BIOME BLENDING

Evitar fronteiras artificiais.

Criar transições entre biomas.

Exemplo:

```text
forest
→ woodland
→ plains
```

ou:

```text
desert
→ dryland
→ savanna
```

---

# 19. 3D DENSITY FIELD

A geração do mundo deve possuir uma representação volumétrica.

Conceito:

```text
Density(x,y,z)
```

permitindo:

* montanhas;
* cavernas;
* saliências;
* túneis;
* overhangs;
* grandes vazios;
* terrenos subterrâneos.

---

# 20. CAVES

Criar sistema próprio de cavernas.

Tipos:

```text
small caves
large caves
tunnels
chambers
vertical shafts
deep caverns
```

As cavernas não devem ser somente buracos aleatórios.

---

# 21. AQUIFERS

Criar:

```text
AquiferSystem
```

para controlar água subterrânea.

Aquifers podem depender de:

* altura;
* pressão;
* região;
* água próxima;
* profundidade.

---

# 22. OCEANS

O oceano deve possuir geração própria:

```text
OceanRegion
├── depth
├── floor
├── biome
├── vegetation
├── structures
└── resources
```

---

# 23. SURFACE MATERIALS

Depois do terreno:

```text
SurfaceRules
```

determina:

* solo;
* pedra;
* areia;
* neve;
* cascalho;
* materiais regionais.

---

# 24. SUBSURFACE

O subsolo deve possuir camadas.

Exemplo:

```text
surface
↓
soil
↓
sedimentary layer
↓
stone
↓
deep stone
```

A estrutura pode variar de acordo com a geologia.

---

# 25. ORE SYSTEM

Criar:

```text
OreGenerator
```

Minérios devem depender de:

```text
depth
geology
biome
temperature
dimension
rarity
```

---

# 26. RECURSOS REGIONAIS

Alguns recursos devem ser regionais.

Exemplo:

```text
mountain mineral
desert mineral
deep mineral
ocean resource
far-land resource
```

Isso cria incentivo à exploração.

---

# 27. VEGETATION

Criar:

```text
VegetationGenerator
```

que considere:

* biome;
* chuva;
* solo;
* altitude;
* temperatura;
* proximidade de água.

---

# 28. VEGETATION HIERARCHY

Gerar:

```text
grass
↓
shrubs
↓
plants
↓
trees
↓
forests
```

com regras regionais.

---

# 29. STRUCTURE SYSTEM

Criar:

```text
StructureRegistry
```

para:

* árvores especiais;
* ruínas;
* templos;
* pontes;
* cavernas estruturadas;
* postos;
* construções;
* landmarks.

---

# 30. STRUCTURE SPACING

Estruturas importantes devem utilizar regras de distribuição.

Evitar:

```text
uma cidade em cima da outra
```

e:

```text
dez estruturas iguais em 100 metros
```

---

# 31. SETTLEMENT SYSTEM

Este será um dos sistemas mais importantes depois do terreno.

Uma vila não será somente uma estrutura.

Ela será:

```text
Settlement
├── population
├── buildings
├── economy
├── resources
├── jobs
├── storage
├── roads
├── defenses
├── leadership
├── culture
└── growth
```

---

# 32. VILLAGE SIZE

Criar níveis:

```text
camp
hamlet
village
town
city
major city
```

---

# 33. SETTLEMENT FORMATION

A geração de assentamentos deve considerar:

```text
water
terrain
resources
fertile land
climate
trade routes
nearby settlements
```

---

# 34. ASSENTAMENTO NÃO É DECORATIVO

Cada assentamento deve possuir estado lógico.

Exemplo:

```text
population = 83
food = 120
iron = 15
wealth = 430
```

---

# 35. ECONOMY SYSTEM

Criar:

```text
EconomySystem
```

Cada settlement poderá possuir:

* produção;
* demanda;
* estoque;
* preços;
* comércio;
* riqueza;
* moeda.

---

# 36. CURRENCIES

Permitir moedas locais.

Exemplo:

```text
settlement-A: coin-A
settlement-B: coin-B
```

O sistema deve suportar câmbio quando necessário.

---

# 37. RESOURCE FLOW

Recursos devem circular:

```text
mine
↓
settlement
↓
warehouse
↓
market
↓
trade route
↓
another settlement
```

---

# 38. NPC POPULATION

Criar população procedural.

Cada NPC pode possuir:

```text
name
occupation
home
settlement
needs
relationships
skills
```

---

# 39. JOB SYSTEM

Profissões:

```text
farmer
miner
blacksmith
merchant
builder
guard
engineer
explorer
```

A lista deve ser extensível.

---

# 40. QUEST SYSTEM

Missões podem surgir do estado do mundo.

Exemplo:

```text
low food
↓
farmer request
```

ou:

```text
mine discovered
↓
merchant request
```

As missões devem depender do contexto.

---

# 41. SETTLEMENT GROWTH

Uma vila pode evoluir.

Exemplo:

```text
hamlet
↓
village
↓
town
↓
city
```

Condições:

* população;
* alimentação;
* riqueza;
* espaço;
* infraestrutura;
* segurança;
* comércio.

---

# 42. SETTLEMENT DECLINE

Também deve ser possível diminuir.

Exemplo:

```text
famine
↓
population loss
↓
economic decline
```

ou:

```text
resource depleted
↓
trade decreases
↓
settlement declines
```

---

# 43. ROADS

As cidades devem criar infraestrutura.

```text
Settlement
↓
Road Network
```

---

# 44. RAIL SYSTEM

O NEXORA deve possuir uma infraestrutura ferroviária capaz de conectar regiões.

```text
mine
↓
rail
↓
station
↓
city
```

---

# 45. RAIL GENERATION

Ferrovias podem:

* ser construídas pelo jogador;
* aparecer em pequenas quantidades como infraestrutura pré-gerada;
* expandir redes existentes;
* conectar centros econômicos.

O sistema deve possuir:

```text
RailNode
RailSegment
Station
Depot
```

---

# 46. LOGISTICS

Criar:

```text
LogisticsGraph
```

permitindo representar:

```text
road
rail
shipping
future transport
```

---

# 47. WORLD SIMULATION VS GENERATION

Separar:

```text
Generation
=
criação inicial
```

de:

```text
Simulation
=
evolução após criação
```

Isso é fundamental.

---

# 48. PERSISTENT WORLD

Depois que um chunk for gerado:

o estado importante deve ser salvo.

O mundo não deve regenerar a história de uma região ao descarregar o chunk.

---

# 49. PLAYER MODIFICATIONS

O gerador deve respeitar modificações do jogador.

Exemplo:

```text
original terrain
+
player railroad
+
player mine
+
player building
```

não deve ser sobrescrito pela geração.

---

# 50. FAR LANDS

As Far Lands fazem parte oficialmente do World Generation System.

Elas não representam o fim do mundo.

Representam uma mudança de regime de geração.

---

# 51. FRONTIER MODEL

Criar:

```text
WorldPhase
```

Exemplo:

```text
CORE_WORLD
WILDLANDS
FAR_LANDS
BEYONDLANDS
DEEP_FRONTIER
DIMENSIONAL_FRONTIER
```

---

# 52. FAR LANDS TRANSITION

A transição não deve ser uma parede instantânea.

Usar uma zona progressiva.

```text
normal
↓
unstable terrain
↓
extreme terrain
↓
far lands
```

---

# 53. FAR LAND TERRAIN

Nas Far Lands:

a geração pode se tornar deliberadamente extrema.

Exemplos:

* montanhas anormais;
* vales gigantes;
* estruturas naturais enormes;
* regiões extremamente acidentadas;
* cavernas de escala muito maior;
* biomas raros.

Mas precisa continuar sendo tecnicamente gerável.

---

# 54. FAR LANDS RESOURCES

Recursos especiais devem existir nessa região.

Exemplo conceitual:

```text
Far Crystal
Frontier Ore
Ancient Material
Dimensional Mineral
```

Esses nomes são placeholders de design e podem ser alterados.

---

# 55. FAR LANDS PROGRESSION

A existência dos recursos cria:

```text
Far Lands
↓
exploration
↓
resource extraction
↓
transport
↓
railway
↓
industrial expansion
```

---

# 56. BEYONDLANDS

Depois das Far Lands:

o mundo continua.

```text
Far Lands
↓
Beyondlands
```

As regras de geração podem mudar novamente.

---

# 57. BEYONDLANDS

Possíveis características:

* biomas desconhecidos;
* geologia extrema;
* estruturas únicas;
* recursos raros;
* fenômenos incomuns;
* grandes áreas inexploradas.

---

# 58. DIMENSIONAL FRONTIER

Depois das regiões terrestres extremas:

o mundo pode começar a apresentar fenômenos relacionados às dimensões.

Isso cria uma transição natural para o sistema dimensional.

---

# 59. 16 DIMENSIONS

O NEXORA deverá possuir 16 dimensões oficiais.

Cada uma possuirá:

```text
DimensionDefinition
├── generator
├── climate
├── terrain
├── biomes
├── resources
├── structures
├── entities
├── rules
└── progression
```

---

# 60. DIMENSION GENERATORS

Não obrigar todas as dimensões a usar o mesmo generator.

Possíveis tipos:

```text
Planetary
Underground
Floating
Oceanic
Fractal
Void
Mechanical
Magical
Space
```

---

# 61. DIMENSION INDEPENDENCE

Cada dimensão possui:

```text
seed context
world rules
generation rules
```

Mas todas compartilham as APIs fundamentais.

---

# 62. CROSS-DIMENSION RESOURCES

Alguns recursos podem cruzar dimensões.

Exemplo:

```text
Dimension A
→ material X

Dimension B
→ material Y

Dimension C
→ material Z
```

Isso cria cadeias de progressão.

---

# 63. MULTIDIMENSIONAL LOGISTICS

Futuramente:

```text
dimension
↓
portal / gateway
↓
storage
↓
transport
```

permitindo redes de produção entre dimensões.

---

# 64. WORLD EVENTS

Preparar eventos de mundo:

```text
meteor
storm
migration
resource discovery
settlement growth
settlement decline
```

Esses eventos pertencem à simulação, não à geração inicial.

---

# 65. DISCOVERY SYSTEM

Registrar regiões descobertas pelo jogador.

Exemplo:

```text
unknown
↓
explored
↓
mapped
↓
catalogued
```

---

# 66. MAP DATA

O sistema de mundo deverá ser capaz de fornecer:

* altitude;
* bioma;
* recursos conhecidos;
* estruturas;
* assentamentos;
* estradas;
* ferrovias;
* regiões especiais.

---

# 67. SEED PREVIEW

Futuramente criar ferramenta:

```text
nexora world preview
```

para visualizar:

* mapa;
* biomas;
* continentes;
* rios;
* settlements;
* Far Lands.

Sem precisar carregar o jogo inteiro.

---

# 68. WORLDGEN DEBUG TOOLS

Criar ferramentas para visualizar:

```text
heightmap
biome map
climate map
density
ore map
structure map
settlement map
rail network
world phases
```

---

# 69. WORLDGEN TESTING

Criar testes determinísticos:

```text
same seed
+
same version
=
same output
```

---

# 70. GOLDEN SEEDS

Manter seeds de teste.

Exemplo:

```text
TEST_SEED_01
TEST_SEED_02
TEST_SEED_03
```

Cada uma deve representar cenários específicos.

---

# 71. EDGE CASES

Testar:

* montanha no spawn;
* oceano;
* cavernas gigantes;
* regiões extremamente densas;
* Far Lands;
* Beyondlands;
* settlements muito próximos;
* mapas com poucos recursos;
* dimensões;
* chunks nos limites numéricos.

---

# 72. PERFORMANCE

Medir:

```text
chunk generation
terrain generation
biome generation
cave generation
structure placement
settlement placement
mesh build
```

---

# 73. PARALLEL GENERATION

Permitir geração paralela de chunks quando possível.

Mas manter segurança de dados e determinismo.

---

# 74. CACHE

Implementar caches para:

```text
noise
height
climate
biomes
macro regions
```

somente onde medições mostrarem benefício.

---

# 75. LOD / DISTANCE

Preparar suporte para diferentes níveis de detalhe.

Distância grande:

```text
macro terrain
```

Distância próxima:

```text
full chunks
```

---

# 76. FAR-LANDS PERFORMANCE

As Far Lands não podem simplesmente destruir o desempenho.

A geração extrema precisa continuar respeitando:

* chunk budget;
* streaming;
* memória;
* cache;
* mesh generation.

---

# 77. WORLD STREAMING

O mundo deve carregar/descarregar dinamicamente conforme a posição do jogador.

```text
player
↓
load radius
↓
generation queue
↓
render
```

---

# 78. GENERATION QUEUE

Criar fila de prioridades:

```text
player-near chunks
>
future movement chunks
>
background chunks
```

---

# 79. SAVE SYSTEM INTEGRATION

Separar:

```text
generated terrain
```

de:

```text
modified world state
```

para não destruir mudanças do jogador.

---

# 80. MOD INTEGRATION

Mods poderão contribuir com:

```text
Biome
Ore
Structure
Plant
Entity
Region Rule
World Feature
Dimension
```

sem editar diretamente o generator principal.

---

# 81. GENERATION HOOKS

Criar hooks como:

```text
beforeTerrain
afterTerrain
beforeBiome
afterBiome
beforeStructures
afterStructures
```

somente quando necessário.

---

# 82. MOD WORLDGEN API

Criar:

```text
Worldgen API
```

para conteúdo externo.

Exemplo:

```text
registerBiome()
registerOre()
registerStructure()
registerFeature()
registerDimension()
registerGenerator()
```

---

# 83. MOD CONFLICTS

Se dois mods registrarem regras conflitantes:

o sistema deve detectar.

Exemplo:

```text
conflicting biome rules
duplicate feature
incompatible generator
```

---

# 84. WORLDGEN VERSIONING PARA MODS

Mods devem declarar:

```text
gameVersion
worldgenApiVersion
```

---

# 85. WORLDGEN DIAGNOSTICS

Adicionar:

```text
nexora world doctor
```

para descobrir:

* seed;
* generator;
* version;
* dimension;
* chunk status;
* performance;
* erros.

---

# 86. FAILURE HANDLING

Se uma etapa falhar:

não corromper o chunk inteiro silenciosamente.

Registrar:

```text
generation stage
chunk
error
fallback
```

---

# 87. FALLBACK

Quando uma feature opcional falhar:

usar fallback seguro.

Exemplo:

```text
custom structure failed
↓
skip structure
↓
terrain continua válido
```

---

# 88. INFINITE WORLD

O mundo deverá continuar sendo gerável além da região inicial.

Não depender de um mapa pré-computado gigantesco.

---

# 89. WORLD BOUNDARIES

Não utilizar uma simples:

```text
world edge
```

como limite narrativo principal.

As regiões especiais cumprem essa função melhor.

---

# 90. FAR LANDS ALÉM DA REGIÃO INICIAL

A descoberta das Far Lands deve fazer parte da exploração.

O jogador pode:

```text
mapear
explorar
construir
colonizar
transportar
```

---

# 91. FRONTIER SETTLEMENTS

As regiões próximas às Far Lands podem possuir:

```text
frontier towns
outposts
rail stations
mining settlements
```

gerando uma espécie de fronteira econômica.

---

# 92. PLAYER COLONIZATION

Futuramente o jogador poderá construir:

```text
outpost
mine
station
settlement
industrial center
```

A geração não deve apagar essas estruturas.

---

# 93. PROCEDURAL SOCIETY

No longo prazo, assentamentos podem formar:

```text
villages
towns
cities
regions
trade networks
```

O mundo passará de:

```text
procedural terrain
```

para:

```text
procedural civilization
```

---

# 94. WORLD HISTORY

O sistema de simulação poderá registrar eventos:

```text
settlement founded
railway created
mine opened
resource discovered
settlement expanded
trade route created
```

Isso pode futuramente alimentar uma história emergente do mundo.

---

# 95. WORLDGEN + GAMEPLAY

A geração deve influenciar gameplay.

Exemplo:

```text
montanha
↓
minério
↓
mina
↓
cidade
↓
ferrovia
↓
economia
```

---

# 96. TESTE DE INTEGRAÇÃO MAIS IMPORTANTE

Gerar um mundo.

Depois:

```text
explorar
↓
encontrar settlement
↓
economia existir
↓
encontrar recurso
↓
criar infraestrutura
↓
salvar
↓
sair
↓
reabrir
↓
estado continuar
```

---

# 97. WORLD GENERATION MILESTONES

## WG-0

```text
seed
chunk
terrain básico
```

## WG-1

```text
continents
climate
biomes
```

## WG-2

```text
3D density
caves
aquifers
```

## WG-3

```text
ores
vegetation
structures
```

## WG-4

```text
settlements
NPCs
economy
```

## WG-5

```text
roads
rail
logistics
```

## WG-6

```text
Far Lands
Beyondlands
```

## WG-7

```text
16 dimensions
```

## WG-8

```text
world simulation
```

## WG-9

```text
mod API
```

## WG-10

```text
optimization
tools
stability
```

---

# 98. PRIMEIRA DEMONSTRAÇÃO

Antes de qualquer sistema avançado:

o primeiro protótipo deve gerar:

```text
SEED
↓
CONTINENT
↓
MOUNTAINS
↓
PLAINS
↓
OCEANS
↓
RIVERS
↓
BIOMES
↓
CAVES
```

e permitir o jogador entrar no mundo.

---

# 99. SEGUNDA DEMONSTRAÇÃO

Depois:

```text
WORLD
↓
VILLAGE
↓
POPULATION
↓
ECONOMY
↓
RESOURCE
↓
TRADE
```

---

# 100. TERCEIRA DEMONSTRAÇÃO

Depois:

```text
PLAYER
↓
travels
↓
Far Lands
↓
discovers exclusive resource
↓
builds railway
↓
returns to civilization
```

Esse cenário será um dos testes mais importantes da visão do NEXORA.

---

# 101. DEFINIÇÃO DE QUALIDADE

Uma boa geração de mundo não é:

> “tem muito conteúdo”.

É:

> **“o mundo parece coerente quando o jogador explora.”**

---

# 102. DEFINIÇÃO DE SUCESSO

O World Generation System será considerado bem-sucedido quando:

```text
[ ] seed determinística
[ ] continentes
[ ] terreno
[ ] erosão
[ ] hidrologia
[ ] clima
[ ] biomas
[ ] density field
[ ] cavernas
[ ] aquifers
[ ] materiais
[ ] minérios
[ ] vegetação
[ ] estruturas
[ ] assentamentos
[ ] NPCs
[ ] economia
[ ] estradas
[ ] ferrovia
[ ] regiões especiais
[ ] Far Lands
[ ] Beyondlands
[ ] 16 dimensões
[ ] save/load
[ ] streaming
[ ] performance
[ ] debug tools
[ ] mod API
```

---

# 103. ARQUITETURA FINAL

```text
                    WORLD ENGINE
                         │
             ┌───────────┴───────────┐
             │                       │
        GENERATION                SIMULATION
             │                       │
     ┌───────┼────────┐       ┌──────┼─────────┐
     │       │        │       │      │         │
 Terrain  Climate  Resources  NPC   Economy  Logistics
     │       │        │       │      │         │
     └───────┴────────┴───────┴──────┴─────────┘
                         │
                    WORLD STATE
                         │
               ┌─────────┴─────────┐
               │                   │
            OVERWORLD         FRONTIERS
               │                   │
               │            Far Lands
               │            Beyondlands
               │                   │
               └──────────┬────────┘
                          │
                     DIMENSIONS
                          │
                         ×16
```

---

# 104. PRINCÍPIO FINAL

O NEXORA não deve possuir apenas:

> **procedural generation.**

Deve possuir:

> **procedural world creation + persistent world simulation.**

O terreno cria as condições.

Os recursos influenciam a economia.

A economia influencia os assentamentos.

Os assentamentos criam infraestrutura.

A infraestrutura permite exploração.

A exploração leva às Far Lands.

As Far Lands introduzem novos recursos.

Os recursos justificam novas rotas.

As fronteiras levam às dimensões.

E o mundo continua crescendo.

---

# FRASE DO WORLD ENGINE

> **The world is generated. The world then lives.**
