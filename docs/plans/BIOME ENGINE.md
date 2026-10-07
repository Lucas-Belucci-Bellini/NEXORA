# NEXORA BIOME ENGINE

# MASTER PLAN — SISTEMA DE BIOMAS

> O Biome Engine será um dos maiores sistemas do NEXORA.
>
> O objetivo não é simplesmente possuir centenas de nomes de biomas, mas criar um sistema capaz de produzir **ecossistemas coerentes, extensos e variados**, conectando clima, geologia, altitude, água, vegetação, fauna, recursos, estruturas e regiões especiais.
>
> O sistema poderá receber inspiração de grandes projetos de geração de biomas e de conceitos de ambientes terrestres, subterrâneos, oceânicos, alienígenas e experimentais, mas toda implementação do NEXORA deverá possuir código, assets, nomes e identidade próprios.

---

# 1. OBJETIVO

Criar o sistema responsável por determinar:

```text
BIOMA
+
CLIMA
+
VEGETAÇÃO
+
FAUNA
+
SOLO
+
ROCHA
+
ÁGUA
+
RECURSOS
+
ESTRUTURAS
+
ATMOSFERA
+
MÚSICA
+
PARTÍCULAS
+
EFEITOS
```

O bioma deve representar um ecossistema e não apenas uma textura diferente no terreno.

---

# 2. PRINCÍPIO FUNDAMENTAL

O Biome Engine nunca deve trabalhar isoladamente.

Ele recebe contexto do World Generator:

```text
Seed
↓
World Parameters
↓
Terrain
↓
Elevation
↓
Climate
↓
Hydrology
↓
Geology
↓
Biome Engine
```

Depois:

```text
Biome
↓
Surface
↓
Vegetation
↓
Entities
↓
Resources
↓
Structures
```

---

# 3. BIOME CONTEXT

Criar:

```text
BiomeContext
```

com informações como:

```text
temperature
humidity
rainfall
elevation
depth
slope
continentality
distanceToOcean
waterAvailability
geology
season
worldPhase
dimension
```

O biome deve ser resultado dessas condições.

---

# 4. NÃO USAR BIOMA COMO UMA SIMPLES COORDENADA

Evitar:

```text
if x > 1000 && x < 2000:
    biome = forest
```

Preferir:

```text
climate
+
terrain
+
geology
+
hydrology
+
elevation
=
biome
```

---

# 5. HIERARQUIA BIOLÓGICA

Criar uma hierarquia:

```text
Biome Family
    ↓
Biome Type
    ↓
Biome Variant
    ↓
Local Variant
```

Exemplo:

```text
Temperate Forest
    ↓
Temperate Rainforest
    ↓
Misty Temperate Rainforest
```

Isso evita precisar criar centenas de biomas completamente independentes para pequenas diferenças.

---

# 6. BIOME FAMILIES

Criar famílias amplas:

```text
FOREST
PLAINS
DESERT
SAVANNA
TUNDRA
TAIGA
MOUNTAIN
WETLAND
COAST
OCEAN
RIVER
CAVE
UNDERGROUND
MUSHROOM
VOLCANIC
GLACIAL
ARID
TEMPERATE
TROPICAL
ALIEN
DIMENSIONAL
FRONTIER
```

A lista deve continuar expansível.

---

# 7. FLORESTAS

O sistema de florestas deve ser extremamente amplo.

Possíveis categorias:

```text
temperate forest
dense forest
old-growth forest
rainforest
cloud forest
bamboo forest
redwood forest
autumn forest
enchanted forest
swamp forest
dry forest
mountain forest
```

As variações devem ser produzidas por regras de clima, altitude e solo.

---

# 8. PLANÍCIES

Criar:

```text
grassland
meadow
steppe
prairie
flower meadow
wet grassland
dry grassland
highland meadow
```

---

# 9. DESERTOS

Criar diferentes regimes:

```text
sand desert
rock desert
salt desert
red desert
cold desert
highland desert
dune sea
badlands
```

Não tratar todo deserto como areia.

---

# 10. SAVANAS

Variantes:

```text
dry savanna
wet savanna
wooded savanna
tallgrass savanna
thorn savanna
```

---

# 11. MONTANHAS

Montanhas terão subtipos:

```text
alpine
high mountain
volcanic mountain
snow mountain
rocky mountain
forest mountain
plateau mountain
canyon region
```

---

# 12. GLACIAL

Criar:

```text
ice plains
frozen forest
glacier
snow valley
frozen coast
ice mountain
subglacial region
```

---

# 13. WETLANDS

Criar:

```text
swamp
marsh
bog
mangrove
floodplain
peatland
wet forest
```

---

# 14. OCEAN BIOMES

O oceano terá seu próprio sistema.

Não usar:

```text
ocean
```

como um único biome.

Criar:

```text
coastal sea
open ocean
deep ocean
abyssal ocean
coral reef
kelp forest
thermal region
volcanic ocean
cold ocean
polar ocean
```

---

# 15. OCEAN DEPTH

O biome oceânico deve considerar:

```text
depth
temperature
light
pressure
distance from coast
```

Assim:

```text
shallow
↓
mid-depth
↓
deep
↓
abyssal
```

podem possuir ecossistemas totalmente diferentes.

---

# 16. RIVERS

Rios terão sub-biomas:

```text
mountain stream
forest river
plains river
jungle river
desert river
glacial river
delta
estuary
```

---

# 17. CAVERN BIOMES

O subterrâneo deve possuir biomas próprios.

Exemplo:

```text
limestone caves
crystal caves
fungal caves
lava caves
deep forest caves
ice caves
underground lakes
abyssal caves
```

---

# 18. CAVE ECOLOGY

Cada cavern biome pode possuir:

```text
lighting
plants
fungi
entities
resources
water
structures
```

Não tratar caverna como apenas:

```text
STONE + DARKNESS
```

---

# 19. VERTICAL BIOMES

O mesmo local pode mudar de biome conforme a altitude.

Exemplo:

```text
valley
↓
forest
↓
mountain forest
↓
alpine meadow
↓
snow
```

---

# 20. BIOME TRANSITIONS

Criar transições graduais.

Exemplo:

```text
desert
↓
dryland
↓
savanna
↓
grassland
↓
forest
```

Não criar paredes artificiais entre biomas.

---

# 21. BIOME EDGE SYSTEM

Criar regras de borda:

```text
Biome A
+
Biome B
↓
transition biome
```

Exemplos:

```text
forest ↔ swamp
forest ↔ desert
snow ↔ forest
mountain ↔ plains
ocean ↔ coast
```

---

# 22. MICROBIOMES

Permitir pequenas regiões especiais dentro de biomas maiores.

Exemplo:

```text
forest
└── flower clearing

desert
└── oasis

mountain
└── alpine lake
```

Isso torna a exploração mais variada.

---

# 23. ECOLOGY SYSTEM

Cada biome possuirá uma ecologia.

```text
BiomeEcology
├── plants
├── animals
├── fungi
├── predators
├── resources
└── environmental rules
```

---

# 24. VEGETATION PROFILE

Cada biome define:

```text
treeDensity
grassDensity
flowerDensity
shrubDensity
fungiDensity
```

---

# 25. FLORA

Criar Flora Registry.

Exemplo conceitual:

```text
nexora:oak_like
nexora:fern
nexora:desert_shrub
nexora:crystal_plant
```

Não depender de uma árvore hardcoded.

---

# 26. FAUNA

Cada biome pode definir listas de entidades possíveis.

Mas:

```text
spawn rules
```

devem considerar:

* clima;
* horário;
* altitude;
* comida;
* água;
* densidade populacional.

---

# 27. FOOD WEB

No futuro:

```text
plants
↓
herbivores
↓
predators
```

A fauna pode depender da disponibilidade real do ecossistema.

---

# 28. SEASON SYSTEM

Preparar suporte para:

```text
spring
summer
autumn
winter
```

Biomas poderão mudar de aparência e comportamento.

Exemplo:

```text
forest
→ spring flowers
→ summer dense foliage
→ autumn colors
→ winter snow
```

---

# 29. BIOME WEATHER

Cada biome poderá influenciar:

```text
rain
snow
fog
storm
wind
temperature
```

---

# 30. ATMOSPHERE

Cada biome poderá ter:

```text
fog profile
sky profile
ambient light
particles
audio
music
```

---

# 31. SOUND PROFILE

Criar:

```text
BiomeSoundProfile
```

com:

```text
ambient
wind
water
animals
music
```

---

# 32. BIOME RESOURCES

Cada biome poderá influenciar a distribuição de:

```text
ores
plants
clay
stone
special resources
```

---

# 33. BIOME-GEOLOGY INTEGRATION

Um biome não pode ignorar geologia.

Exemplo:

```text
volcanic region
↓
volcanic rocks
↓
specific minerals
```

---

# 34. BIOME-HYDROLOGY INTEGRATION

Um biome deverá considerar:

```text
rainfall
water table
rivers
lakes
ocean
```

---

# 35. BIOME-ALTITUDE INTEGRATION

Criar perfis por altitude.

Exemplo:

```text
0–500
forest

500–1200
mountain forest

1200–2000
alpine

2000+
snow
```

Os valores reais devem depender do mundo.

---

# 36. BIOME SCALE

Os biomas não devem possuir todos o mesmo tamanho.

Permitir:

```text
micro
small
medium
large
continental
```

---

# 37. MEGA BIOMES

Criar regiões extremamente grandes.

Exemplo:

```text
mega rainforest
mega desert
mega taiga
mega wetlands
```

O tamanho deve ser configurável por seed/world settings.

---

# 38. BIOME CLUSTERING

Biomas relacionados podem aparecer próximos.

Exemplo:

```text
temperate forest
↓
oak woodland
↓
meadow
↓
river valley
```

---

# 39. MACRO BIOME REGIONS

Criar regiões climáticas maiores:

```text
tropical
temperate
arctic
arid
continental
```

Depois os biomas locais são derivados disso.

---

# 40. BIOME RARITY

Definir:

```text
common
uncommon
rare
extremely rare
unique
```

---

# 41. LANDMARK BIOMES

Alguns biomas podem existir principalmente para pontos marcantes.

Exemplos conceituais:

```text
giant crater
ancient forest
crystal valley
volcanic basin
massive canyon
```

---

# 42. SUBNAUTICA-INSPIRED ECOSYSTEMS

O NEXORA poderá utilizar ideias de ambientes oceânicos e alienígenas como referência para criar biomas próprios.

Não copiar mapas, nomes, criaturas, texturas ou assets.

Inspirar-se em conceitos como:

```text
deep ocean
bioluminescent ecosystem
coral-like zones
alien vegetation
thermal regions
subterranean aquatic environments
```

---

# 43. CUT / UNUSED BIOME CONCEPTS

Conceitos de ambientes que foram planejados para outros jogos, mas nunca utilizados, poderão servir como referência de design.

Regra:

```text
concept
↓
análise
↓
reinterpretation
↓
NEXORA biome
```

Não importar automaticamente conteúdo cancelado de terceiros.

---

# 44. ALIEN BIOMES

Criar uma família:

```text
ALIEN
```

com propriedades mais livres.

Exemplos:

```text
bioluminescent forest
crystal plains
floating ecosystem
toxic marsh
living canyon
```

---

# 45. DIMENSIONAL BIOMES

Cada dimensão terá seu próprio conjunto.

```text
Dimension A
→ biome family A

Dimension B
→ biome family B
```

---

# 46. SPACE BIOMES

Mesmo no sistema espacial, regiões poderão possuir classificação ambiental.

Exemplo:

```text
asteroid field
gas cloud region
ice body
alien planet
dead world
ocean world
```

---

# 47. FRONTIER BIOMES

As Far Lands e Beyondlands terão famílias específicas:

```text
FRONTIER
```

---

# 48. FAR-LANDS BIOMES

As Far Lands podem conter:

```text
extreme mountains
fractured valleys
unnatural plateaus
deep ravines
rare forests
extreme deserts
frontier wetlands
```

---

# 49. BEYONDLANDS BIOMES

Depois das Far Lands:

```text
unknown ecosystems
rare biome transitions
extreme climates
unique resources
```

---

# 50. BIOME GENERATION GRAPH

Criar um grafo:

```text
Climate
   ↓
Macro Region
   ↓
Biome Family
   ↓
Biome
   ↓
Variant
   ↓
Microbiome
```

---

# 51. BIOME REGISTRY

Criar:

```text
BiomeRegistry
```

Cada registro:

```text
id
family
climate
terrain
hydrology
ecology
resources
structures
entities
audio
visuals
rarity
```

---

# 52. DATA-DRIVEN BIOMES

Sempre que possível, os biomas devem ser definidos por dados.

Exemplo:

```yaml
id: nexora:temperate_forest
family: forest
temperature: temperate
humidity: high
elevation:
  min: 200
  max: 1200
vegetation:
  density: high
```

---

# 53. BIOME API

Mods poderão registrar:

```text
registerBiome()
registerBiomeVariant()
registerBiomeFeature()
registerBiomeTransition()
registerBiomeEntityRule()
```

---

# 54. BIOME MODIFIERS

Um mod pode alterar:

```text
temperature
vegetation
spawn rules
resources
structures
```

sem substituir o biome inteiro.

---

# 55. BIOME COMPONENTS

Criar componentes reutilizáveis:

```text
TemperatureProfile
RainfallProfile
VegetationProfile
FaunaProfile
SurfaceProfile
OreProfile
SoundProfile
WeatherProfile
```

---

# 56. BIOME COMPOSITION

Um biome deverá ser composto de componentes.

Exemplo:

```text
Biome
├── ClimateProfile
├── SurfaceProfile
├── EcologyProfile
├── ResourceProfile
├── StructureProfile
└── AtmosphereProfile
```

---

# 57. EVITAR DUPLICAÇÃO

Não criar 100 biomas com 95% das regras iguais.

Usar:

```text
inheritance
composition
templates
variants
```

quando apropriado.

---

# 58. BIOME TEMPLATES

Criar templates:

```text
ForestTemplate
DesertTemplate
MountainTemplate
OceanTemplate
SwampTemplate
```

e gerar variantes.

---

# 59. WORLDGEN ORDER

O biome não deve ser escolhido cedo demais.

Pipeline:

```text
terrain
↓
climate
↓
hydrology
↓
geology
↓
biome candidates
↓
biome selection
↓
biome blending
```

---

# 60. BIOME CANDIDATES

Um local poderá ter várias possibilidades.

Exemplo:

```text
forest 65%
woodland 25%
swamp 10%
```

Depois selecionar de maneira determinística.

---

# 61. BIOME BLENDING

Permitir influência espacial.

Exemplo:

```text
Biome A strength = 0.7
Biome B strength = 0.3
```

Usado somente no processo de seleção.

O bloco final continua determinado de maneira consistente.

---

# 62. BIOME BORDERS

Não gerar linhas duras quando não forem desejadas.

Usar:

```text
transition masks
```

---

# 63. BIOME MUTATIONS

Biomas podem possuir alterações raras.

Exemplo:

```text
Forest
→ ancient forest

Desert
→ crystal desert
```

Essas variantes devem ser controladas por seed.

---

# 64. BIOME EVENTS

Alguns eventos podem modificar temporariamente o biome.

Exemplo:

```text
drought
fire
flood
winter
```

Essas alterações pertencem à simulação, não à geração inicial.

---

# 65. PLAYER CHANGES

Se o jogador transformar uma região:

```text
forest
+
player farm
+
player city
```

o biome original continua existindo como dado de origem.

---

# 66. BIOME MAP

Criar ferramenta:

```text
nexora biome map
```

Visualizando:

```text
macro regions
biomes
transitions
rare biomes
```

---

# 67. BIOME DEBUG VIEW

Permitir visualizar:

```text
temperature
humidity
rainfall
altitude
biome
sub-biome
```

em tempo real.

---

# 68. WORLDGEN DEBUG

Criar visualizações:

```text
climate map
biome map
vegetation map
resource map
settlement map
```

---

# 69. BIOME TEST SEEDS

Manter seeds específicas:

```text
forest-heavy
desert-heavy
mountain-heavy
ocean-heavy
cave-heavy
frontier
```

---

# 70. BIOME DISTRIBUTION TEST

Executar análise estatística:

```text
percentage by biome
```

Detectar:

```text
95% forest
```

ou:

```text
biome inexistente
```

---

# 71. DIVERSIDADE

Criar métricas:

```text
biome diversity
transition diversity
rare biome count
macro region diversity
```

---

# 72. REPETIÇÃO

Detectar padrões repetitivos.

Exemplo:

```text
forest
forest
forest
forest
```

por quilômetros sem justificativa.

---

# 73. EXPLORATION QUALITY

Medir:

```text
distance between rare biomes
distance between settlements
distance between unique landmarks
```

---

# 74. BIOME + RESOURCES

Recursos devem fornecer motivos para explorar diferentes biomas.

---

# 75. BIOME + CIVILIZATION

Assentamentos devem considerar:

```text
biome
water
fertility
resources
climate
```

---

# 76. BIOME + ECONOMY

Regiões diferentes produzem coisas diferentes.

Exemplo:

```text
forest
→ wood

mountain
→ minerals

coast
→ fish

far lands
→ rare materials
```

Isso influencia a economia mundial.

---

# 77. BIOME + RAILWAY

Rotas comerciais podem atravessar:

```text
forest
mountains
desert
frontier
```

e criar necessidade de infraestrutura.

---

# 78. BIOME + QUESTS

NPCs podem gerar missões baseadas no bioma.

Exemplo:

```text
desert village
→ water shortage
```

ou:

```text
mountain town
→ mining request
```

---

# 79. BIOME + TECHNOLOGY

Alguns ambientes podem oferecer condições para recursos tecnológicos específicos.

---

# 80. BIOME + MAGIC

Regiões especiais podem possuir recursos mágicos exclusivos.

---

# 81. BIOME + SPACE

Planetas e dimensões espaciais poderão possuir seus próprios biome systems.

---

# 82. BIOME SCARCITY

Alguns biomas devem ser verdadeiramente raros.

```text
common
1 / 10
1 / 100
1 / 1000
unique
```

Os valores são exemplos e deverão ser balanceados por testes.

---

# 83. UNIQUE BIOMES

Alguns biomas podem ser:

```text
seed-unique
world-unique
dimension-unique
```

---

# 84. DISCOVERY

O jogador pode descobrir:

```text
unknown biome
↓
explored
↓
catalogued
```

---

# 85. BIOME ENCYCLOPEDIA

Futuramente:

```text
NEXORA FIELD GUIDE
```

com:

* clima;
* flora;
* fauna;
* recursos;
* localização;
* raridade.

---

# 86. PERFORMANCE

Biome generation deve ser barata o suficiente para rodar durante chunk generation.

Evitar cálculos gigantescos por bloco quando um cálculo regional resolver.

---

# 87. REGIONAL CACHE

Pré-calcular:

```text
climate region
macro biome
geology region
```

e reutilizar dentro dos chunks.

---

# 88. MULTITHREADING

Executar partes independentes em paralelo:

```text
biome candidate calculation
vegetation planning
structure planning
```

sem comprometer determinismo.

---

# 89. DETERMINISMO

Mesma:

```text
seed
world version
dimension
coordinates
```

deve produzir o mesmo biome.

---

# 90. WORLDGEN VERSION

Salvar:

```text
biomeGenerationVersion
```

separadamente quando necessário.

---

# 91. MOD COMPATIBILITY

Mods devem declarar:

```text
gameVersion
worldgenApiVersion
```

---

# 92. BIOME CONFLICT RESOLUTION

Se dois mods alterarem a mesma região:

resolver através de:

```text
priority
tags
compatibility
generation layer
```

e não por comportamento aleatório.

---

# 93. BIOME API TEST

Criar Example Biome Mod:

```text
1 biome
1 variant
1 plant
1 entity
1 structure
1 resource
```

O mod deve conseguir registrar tudo sem editar o Core.

---

# 94. VANILLA BIOME TEST

Os biomas oficiais do NEXORA devem utilizar a mesma API.

---

# 95. CONTENT PACKS

No futuro permitir:

```text
Biome Pack
```

que adiciona:

```text
biomes
plants
entities
structures
resources
```

---

# 96. BIOME EXPANSION PACKS

O sistema deve permitir pacotes oficiais:

```text
NEXORA: Tropical Expansion
NEXORA: Deep Ocean
NEXORA: Frontier
NEXORA: Alien Worlds
```

Os nomes são exemplos.

---

# 97. BIOME FAMILIES OFICIAIS

Primeira grande coleção:

```text
Forests
Plains
Deserts
Savannas
Tundra
Taiga
Mountains
Wetlands
Coasts
Oceans
Rivers
Caves
Underground
Volcanic
Glacial
Alien
Frontier
Dimensional
```

---

# 98. REFERÊNCIAS DE DESIGN

As seguintes obras podem ser estudadas como referências de variedade e abordagem:

```text
Biomes O' Plenty
Terralith
Regions Unexplored
Oh The Biomes We've Gone
Ad Astra
Subnautica ecosystem concepts
```

Não copiar implementação, assets ou conteúdo protegido.

A regra é:

```text
REFERENCE
↓
ANALYSIS
↓
NEXORA DESIGN
↓
NATIVE IMPLEMENTATION
```

---

# 99. CRITÉRIO DE QUALIDADE

Não medir o sistema apenas por:

```text
quantidade de biomas
```

Medir:

```text
coerência
diversidade
transição
raridade
ecologia
exploração
performance
```

---

# 100. DEFINIÇÃO FINAL

O Biome Engine deverá fazer o jogador sentir:

> “Eu estou em outro ecossistema.”

e não apenas:

> “A textura do chão mudou.”

O mundo deve apresentar diferenças de:

```text
clima
forma
vegetação
fauna
água
recursos
sons
atmosfera
estruturas
economia
```

---

# ROADMAP DO BIOME ENGINE

```text
B0
Biome API
    ↓
B1
Climate + Macro Regions
    ↓
B2
Basic Biomes
    ↓
B3
Biome Transitions
    ↓
B4
Vegetation + Ecology
    ↓
B5
Ocean + River Biomes
    ↓
B6
Cave Biomes
    ↓
B7
Rare + Micro Biomes
    ↓
B8
Alien + Frontier Biomes
    ↓
B9
Dimension Biomes
    ↓
B10
Seasonal Ecology
    ↓
B11
Biome Simulation
    ↓
B12
Mod API
    ↓
B13
Optimization
    ↓
B14
Biome Encyclopedia + Tools
```

---

# TESTE FINAL

Gerar um mundo.

Depois verificar:

```text
continents
↓
climate
↓
biomes
↓
transitions
↓
ecology
↓
resources
↓
settlements
↓
exploration
↓
Far Lands
↓
Beyondlands
↓
dimensions
```

O sistema deve produzir um mundo em que cada região tenha uma identidade clara.

---

# PRINCÍPIO FINAL

> **O NEXORA não deve ter simplesmente muitos biomas.**
>
> **Ele deve ter muitos ecossistemas.**

O objetivo é que a geração de mundo crie variedade suficiente para que viajar milhares de blocos não pareça simplesmente atravessar cópias do mesmo ambiente.

---

# FRASE DO BIOME ENGINE

> **Every region has a reason to exist.**
