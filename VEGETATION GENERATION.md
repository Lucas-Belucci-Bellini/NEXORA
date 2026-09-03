Sim. **Vegetação Dinâmica** deve ser um sistema próprio, não apenas uma lista de árvores que o WorldGen coloca no mapa.

No NEXORA, eu faria a vegetação funcionar como parte de um **ecossistema vivo**:

```text
WORLDGEN
   ↓
VEGETATION GENERATION
   ↓
VEGETATION SYSTEM
   ├── plants
   ├── growth
   ├── reproduction
   ├── competition
   ├── seasons
   ├── resources
   ├── disease
   ├── fire response
   └── ecosystem interaction
           ↓
      WORLD SIMULATION
```

A ideia central:

> **WorldGen planta o mundo. A Vegetation System decide como ele cresce, morre, se espalha e muda.**

# NEXORA — DYNAMIC VEGETATION MASTER PLAN

## 1. Objetivo

O sistema precisa representar:

```text
árvores
arbustos
gramíneas
flores
fungos
cultivos
musgos
plantas aquáticas
algas
plantas subterrâneas
plantas gigantes
vegetação alienígena
vegetação dimensional
```

Mas também:

```text
crescimento
morte
reprodução
sementes
competição
dispersão
doenças
pragas
estações
água
temperatura
luz
solo
incêndios
herbivoria
regeneração
```

---

# 2. Arquitetura

```text
                    VEGETATION SYSTEM
                           │
          ┌────────────────┼─────────────────┐
          │                │                 │
      SPECIES           ECOLOGY           GROWTH
          │                │                 │
      PlantType       Competition       Lifecycle
      Genetics        Population        Stages
      Traits          Succession        Reproduction
          │                │                 │
          └────────────────┼─────────────────┘
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
     SOIL                CLIMATE             WATER
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                       VEGETATION
                           │
            ┌──────────────┼──────────────┐
            │              │              │
          PLAYER           MOBS        CIVILIZATION
            │              │              │
        harvesting      grazing        farming
        planting       herbivory       forestry
```

---

# 3. VEG-0 — Plant Definition

Criar:

```text
PlantDefinition
├── id
├── species
├── growthRate
├── lifespan
├── preferredTemperature
├── preferredMoisture
├── lightRequirement
├── soilRequirements
├── growthStages
├── reproduction
├── resilience
└── ecologicalTraits
```

---

# 4. VEG-1 — Plant Instance

Não confundir espécie com indivíduo.

```text
PlantDefinition
        ↓
PlantInstance
```

Uma árvore individual pode ter:

```text
idade
altura
saúde
energia
água
estado
genética
posição
```

---

# 5. VEG-2 — Plant Lifecycle

Estados:

```text
SEED
 ↓
GERMINATING
 ↓
SEEDLING
 ↓
JUVENILE
 ↓
MATURE
 ↓
FLOWERING
 ↓
FRUITING
 ↓
AGING
 ↓
DEAD
 ↓
DECAYING
```

Nem toda planta precisa usar todos os estágios.

---

# 6. VEG-3 — Growth

Crescimento depende de:

```text
luz
água
temperatura
nutrientes
solo
idade
estação
saúde
competição
```

Conceitualmente:

```text
Growth Potential
=
Environment
+
Genetics
+
Resources
-
Stress
```

---

# 7. VEG-4 — Growth Simulation

Não atualizar cada planta a cada frame.

Usar:

```text
frame
→ visual animation

minute/hour
→ local growth

day
→ plant lifecycle

season
→ large ecological changes
```

---

# 8. VEG-5 — Growth LOD

Perto:

```text
FULL
→ indivíduo
```

Médio:

```text
REGIONAL
→ população agregada
```

Longe:

```text
ABSTRACT
→ biomass / vegetation density
```

---

# 9. VEG-6 — Plant Population

Para áreas enormes:

```text
VegetationPopulation
├── species
├── density
├── averageAge
├── biomass
├── health
└── regenerationRate
```

Não precisamos manter milhões de indivíduos ativos.

---

# 10. VEG-7 — Seed System

Criar sementes como entidades/dados:

```text
Seed
├── species
├── genetics
├── viability
├── age
└── state
```

Podem:

```text
germinate
disperse
die
remain dormant
```

---

# 11. VEG-8 — Seed Dispersal

Sementes podem ser dispersadas por:

```text
vento
água
animais
frutos
queda natural
player
civilization
```

---

# 12. VEG-9 — Wind Dispersal

Climate fornece:

```text
wind direction
wind strength
```

Vegetation calcula:

```text
seed transport
```

---

# 13. VEG-10 — Water Dispersal

Rios podem transportar sementes:

```text
seed
 ↓
river
 ↓
downstream
 ↓
new habitat
```

Isso cria padrões naturais de expansão.

---

# 14. VEG-11 — Animal Dispersal

Mobs podem carregar sementes através de:

```text
feeding
fur
droppings
```

O Ecology System fornece o comportamento; Vegetation fornece a reprodução.

---

# 15. VEG-12 — Pollination

Plantas podem possuir:

```text
pollinationType
```

como:

```text
wind
animal
self
water
special
```

---

# 16. VEG-13 — Pollinator Interaction

Abelhas/outros organismos podem:

```text
flower
 ↓
pollinator
 ↓
pollination
 ↓
seed/fruit
```

Isso conecta Vegetation diretamente à Ecology.

---

# 17. VEG-14 — Reproduction

Tipos:

```text
SEED
SPORE
RUNNER
ROOT
CLONAL
FRUIT
CUSTOM
```

---

# 18. VEG-15 — Spore System

Para fungos e plantas especiais:

```text
spore
 ↓
air/water
 ↓
landing
 ↓
germination
```

---

# 19. VEG-16 — Soil System Integration

Vegetation consulta:

```text
soil type
soil moisture
nutrient availability
pH-like properties
temperature
```

O Soil System pode ser separado.

---

# 20. VEG-17 — Soil Moisture

O Fluid/Soil system fornece:

```text
moisture
```

Vegetation responde:

```text
dry
optimal
waterlogged
```

---

# 21. VEG-18 — Nutrients

Criar interface:

```text
NutrientState
├── fertility
├── mineralAvailability
└── organicMatter
```

Não precisa modelar química realista desde o início.

---

# 22. VEG-19 — Root System

Árvores grandes podem possuir:

```text
RootSystem
```

com:

```text
rootDepth
rootSpread
waterAccess
nutrientAccess
```

---

# 23. VEG-20 — Roots

Roots podem interagir com:

```text
soil
water
rocks
other plants
```

e influenciar estabilidade/saúde.

---

# 24. VEG-21 — Competition

Plantas competem por:

```text
light
water
nutrients
space
```

Exemplo:

```text
tree
 ↓
canopy
 ↓
less light
 ↓
understory plants affected
```

---

# 25. VEG-22 — Canopy

Uma árvore pode possuir:

```text
canopyDensity
canopyRadius
```

para simplificar sombra e competição.

---

# 26. VEG-23 — Vegetation Density

A região possui:

```text
vegetation density
```

derivada de:

```text
climate
soil
water
elevation
biome
ecology
history
```

---

# 27. VEG-24 — Biome Integration

O Biome Engine fornece condições.

Vegetation decide quais espécies realmente conseguem existir.

```text
Biome
 ↓
Environmental Conditions
 ↓
Vegetation Selection
 ↓
Population
```

Assim um bioma não é simplesmente:

```text
"spawn 50 árvores"
```

---

# 28. VEG-25 — Ecological Succession

Áreas podem mudar com o tempo:

```text
bare ground
 ↓
grass
 ↓
shrubs
 ↓
young forest
 ↓
mature forest
```

Depois:

```text
disturbance
 ↓
succession restarts
```

---

# 29. VEG-26 — Disturbance

Distúrbios:

```text
fire
flood
drought
storm
logging
construction
grazing
volcanism
```

podem alterar comunidades vegetais.

---

# 30. VEG-27 — Natural Regeneration

Depois de uma floresta ser parcialmente destruída:

```text
seed bank
+
surviving plants
+
favorable climate
=
regeneration
```

---

# 31. VEG-28 — Forest Regrowth

Florestas podem voltar a crescer, mas não necessariamente iguais.

```text
old forest
 ↓
disturbance
 ↓
young forest
 ↓
different species mix
 ↓
mature forest
```

---

# 32. VEG-29 — Fire Integration

Fire System fornece:

```text
heat
burn status
burn area
```

Vegetation responde:

```text
damaged
burned
dead
regrowth
```

---

# 33. VEG-30 — Fire Resistance

Espécies podem possuir:

```text
fireResistance
regrowthAfterFire
```

Algumas plantas podem se recuperar melhor que outras.

---

# 34. VEG-31 — Disease

Criar:

```text
PlantDisease
```

com:

```text
pathogen
hostRange
transmission
severity
```

---

# 35. VEG-32 — Plant Disease Spread

Doença pode se espalhar por:

```text
wind
water
contact
animals
seeds
human activity
```

---

# 36. VEG-33 — Pests

Criar integração com mobs:

```text
Pest
 ↓
plant population
 ↓
crop damage
```

---

# 37. VEG-34 — Herbivory

Mobs podem consumir plantas:

```text
Herbivore
 ↓
Vegetation Query
 ↓
consume
 ↓
plant damage
 ↓
population effects
```

---

# 38. VEG-35 — Predator/Plant Indirect Interaction

Vegetação influencia herbívoros:

```text
plants
 ↓
food availability
 ↓
herbivore population
 ↓
predators
```

Isso alimenta o Ecology System.

---

# 39. VEG-36 — Animal Population Feedback

Ecossistema:

```text
Vegetation ↓
Herbivores ↓
Predators ↓
```

ou:

```text
Vegetation ↑
Herbivores ↑
Predators ↑
```

Isso cria ciclos.

---

# 40. VEG-37 — Seasonal Growth

Estações alteram:

```text
growth
flowering
fruiting
dormancy
leaf color
leaf drop
```

---

# 41. VEG-38 — Dormancy

Algumas plantas:

```text
winter
 ↓
dormant
```

e retomam crescimento na estação adequada.

---

# 42. VEG-39 — Deciduous Plants

Suporte a:

```text
leaf growth
leaf drop
regrowth
```

Renderer trata a aparência.

Vegetation trata o estado.

---

# 43. VEG-40 — Evergreen Plants

Algumas espécies mantêm folhas.

Isso deve ser apenas uma propriedade da espécie.

---

# 44. VEG-41 — Flowering

Estado:

```text
not flowering
 ↓
flowering
 ↓
pollinated
 ↓
fruiting
```

---

# 45. VEG-42 — Fruit

Plantas podem produzir:

```text
fruit
seed
resource
```

Isso alimenta:

```text
Loot/Drop
Farming
Economy
Mob Ecology
```

---

# 46. VEG-43 — Resource Production

Vegetação pode gerar:

```text
wood
leaves
fruit
fiber
sap
resin
seeds
medicine-like resources
```

O Drop System pode controlar os itens obtidos.

---

# 47. VEG-44 — Forestry

Civilizações podem interagir com:

```text
forest
```

para:

```text
logging
replanting
management
```

---

# 48. VEG-45 — Sustainable Forestry

Criar parâmetros:

```text
harvest rate
regrowth rate
```

Uma civilização pode:

```text
overharvest
```

ou:

```text
manage forest
```

---

# 49. VEG-46 — Deforestation

Grandes mudanças de uso do solo:

```text
forest
 ↓
clearing
 ↓
farmland/city
```

afetam:

```text
climate
ecology
soil
water
economy
```

---

# 50. VEG-47 — Reforestation

Civilização pode plantar novamente:

```text
forest restoration
```

O sistema simula o retorno ao longo do tempo.

---

# 51. VEG-48 — Agriculture

Agricultura usa uma camada especializada:

```text
AgriculturalPlant
```

sobre o mesmo lifecycle básico.

```text
Plant System
 ↓
Farm Module
```

---

# 52. VEG-49 — Crop Growth

Cultivos dependem de:

```text
water
temperature
light
soil
season
fertility
```

---

# 53. VEG-50 — Crop Rotation

Agricultura avançada pode possuir:

```text
crop rotation
```

para alterar fertilidade e produtividade.

---

# 54. VEG-51 — Irrigation

Integração:

```text
Fluid Engine
 ↓
Irrigation
 ↓
Soil Moisture
 ↓
Vegetation
```

---

# 55. VEG-52 — Greenhouses

Estruturas podem criar:

```text
controlled environment
```

com:

```text
temperature
humidity
light
```

---

# 56. VEG-53 — Underground Agriculture

No Deep World:

```text
cave
 ↓
artificial lighting
 ↓
humidity
 ↓
special crops
```

---

# 57. VEG-54 — Deep World Flora

Criar suporte a:

```text
fungus forests
crystal plants
giant roots
underground vines
bioluminescent flora
```

com definições próprias.

---

# 58. VEG-55 — Aquatic Plants

Plantas podem viver:

```text
water surface
shallow water
deep water
```

consultando o Fluid Engine.

---

# 59. VEG-56 — Algae

Criar população especializada:

```text
AlgaeField
```

importante para:

```text
oceans
lakes
rivers
```

---

# 60. VEG-57 — Ocean Vegetation

Suporte a:

```text
kelp-like organisms
seagrass
floating vegetation
deep aquatic flora
```

---

# 61. VEG-58 — Vertical Vegetation

Plantas podem crescer:

```text
up
down
sideways
```

e ocupar cavernas/encostas.

---

# 62. VEG-59 — Climbing Plants

Criar propriedades:

```text
climbing
vining
attachment
```

---

# 63. VEG-60 — Giant Flora

Algumas regiões podem possuir plantas enormes:

```text
giant tree
mega fungus
massive vines
ancient flora
```

Elas podem ocupar muitos chunks.

---

# 64. VEG-61 — Multi-Chunk Plants

Uma planta grande não pode ser simplesmente um bloco único.

Criar:

```text
PlantInstance
 ↓
PlantStructure
 ↓
multiple voxel components
```

---

# 65. VEG-62 — Plant Structure

Exemplo:

```text
Giant Tree
├── trunk
├── branches
├── roots
├── leaves
├── flowers
└── fruit
```

---

# 66. VEG-63 — Structure Persistence

Uma planta gigante precisa continuar sendo uma identidade única mesmo ocupando dezenas/centenas de chunks.

---

# 67. VEG-64 — Growth Across Chunks

Quando cresce para outro chunk:

```text
Growth
 ↓
neighbor chunk request
 ↓
plant extension
```

O Chunk Engine continua dono do armazenamento.

---

# 68. VEG-65 — Render Integration

Vegetation fornece:

```text
PlantRenderState
```

Renderer transforma em:

```text
mesh
instances
LOD
animation
```

---

# 69. VEG-66 — Vegetation Instancing

Florestas podem possuir milhares de indivíduos.

Usar:

```text
GPU instancing
```

quando possível.

---

# 70. VEG-67 — Vegetation LOD

```text
LOD0
→ full model

LOD1
→ simplified

LOD2
→ billboard/impostor

LOD3
→ density representation

Hidden
```

---

# 71. VEG-68 — Procedural Trees

Árvores podem ser definidas proceduralmente:

```text
seed
+
species
+
genetics
=
shape
```

Isso evita armazenar milhares de modelos únicos.

---

# 72. VEG-69 — Tree Genetics

Traits:

```text
height
branching
leafDensity
rootDepth
growthRate
droughtResistance
fireResistance
```

---

# 73. VEG-70 — Variation

Dois indivíduos da mesma espécie podem ser visualmente diferentes:

```text
species
+
genetic seed
=
variation
```

Isso deixa florestas menos repetitivas.

---

# 74. VEG-71 — Mutation

Reprodução pode produzir pequenas variações.

Não precisa simular genética real detalhada; basta criar um sistema parametrizado.

---

# 75. VEG-72 — Invasive Species

Uma espécie pode expandir além de seu habitat original:

```text
introduction
 ↓
spread
 ↓
competition
 ↓
ecosystem change
```

---

# 76. VEG-73 — Seed Banks

Solo pode conter:

```text
seed bank
```

permitindo regeneração depois de eventos.

---

# 77. VEG-74 — Decomposition

Plantas mortas:

```text
dead
 ↓
decay
 ↓
organic matter
 ↓
soil nutrients
```

Isso fecha o ciclo ecológico.

---

# 78. VEG-75 — Organic Matter

A decomposição pode alimentar:

```text
soil fertility
```

e depois:

```text
plant growth
```

---

# 79. VEG-76 — Nutrient Cycle

Ciclo simplificado:

```text
Plant
 ↓
Death
 ↓
Decomposition
 ↓
Nutrients
 ↓
Soil
 ↓
New Plant
```

---

# 80. VEG-77 — Carbon-like Cycle

Podemos futuramente modelar:

```text
growth
respiration
decay
fire
atmosphere exchange
```

de forma agregada.

Isso integra com Climate.

---

# 81. VEG-78 — Climate Feedback

Florestas podem influenciar:

```text
humidity
temperature moderation
soil moisture
```

O Climate Engine recebe apenas agregados.

---

# 82. VEG-79 — Hydrology Feedback

Vegetação influencia:

```text
runoff
soil retention
evapotranspiration
```

Isso conecta Vegetation e Hydrology.

---

# 83. VEG-80 — Erosion

Vegetação pode reduzir:

```text
soil erosion
```

O Terrain/Hydrology System interpreta isso.

---

# 84. VEG-81 — Roots and Terrain

Raízes podem ajudar a estabilizar alguns terrenos.

Não transformar cada raiz em física detalhada.

Usar propriedades agregadas:

```text
rootDensity
soilStabilityModifier
```

---

# 85. VEG-82 — Weather Damage

Plantas podem reagir a:

```text
wind
snow
ice
heat
drought
flood
```

---

# 86. VEG-83 — Storm Damage

Uma tempestade pode causar:

```text
branch damage
tree fall
defoliation
```

com resultados simplificados.

---

# 87. VEG-84 — Fallen Trees

Árvores grandes podem mudar para:

```text
standing
damaged
fallen
decaying
```

---

# 88. VEG-85 — Fallen Vegetation

Árvores caídas podem virar:

```text
world object
```

ou alimentar:

```text
resource
decay
habitat
```

---

# 89. VEG-86 — Habitat

Vegetação cria habitats:

```text
forest canopy
understory
dead wood
wetland vegetation
```

Mobs podem consultar:

```text
habitat suitability
```

---

# 90. VEG-87 — Biodiversity

Uma região pode manter:

```text
species richness
biomass
population distribution
```

Isso alimenta o sistema ecológico.

---

# 91. VEG-88 — Biome Transition

À medida que clima muda:

```text
Climate
 ↓
habitat suitability
 ↓
vegetation composition
```

Então a paisagem pode mudar gradualmente.

---

# 92. VEG-89 — Seasonal Landscape

Exemplo:

```text
spring
→ growth

summer
→ flowering

autumn
→ fruit / leaves

winter
→ dormancy
```

---

# 93. VEG-90 — Extreme Climate

Em condições extremas:

```text
heatwave
 ↓
stress

drought
 ↓
mortality

flood
 ↓
water stress

cold
 ↓
freeze
```

---

# 94. VEG-91 — Fire Regeneration

Depois de incêndio:

```text
ash
 ↓
soil changes
 ↓
pioneer species
 ↓
succession
```

---

# 95. VEG-92 — Human/Civilization Interaction

Civilizações podem:

```text
plant
harvest
clear
protect
farm
trade
transport
```

---

# 96. VEG-93 — Economy

Vegetation pode alimentar economia:

```text
wood
fiber
fruit
resin
medicine resources
```

e gerar:

```text
resource availability
price
trade
```

---

# 97. VEG-94 — Knowledge

Civilizações podem descobrir:

```text
medicinal plant
fertilizer
cultivation method
new crop
```

O Knowledge System registra essas descobertas.

---

# 98. VEG-95 — Player Interaction

O jogador pode:

```text
plant
harvest
cut
collect seeds
irrigate
cultivate
```

mas todas as operações passam pelas APIs.

---

# 99. VEG-96 — Drop Integration

Quando uma planta é coletada:

```text
Vegetation
 ↓
Harvest Event
 ↓
Drop System
 ↓
Inventory
```

---

# 100. VEG-97 — Dynamic World

O mais importante:

```text
WorldGen
↓
creates initial vegetation
↓
Simulation
↓
growth
↓
death
↓
spread
↓
disturbance
↓
recovery
```

Assim duas regiões geradas originalmente iguais podem ficar diferentes séculos depois.

---

# 101. VEG-98 — Persistence

Salvar o estado necessário:

```text
population
growth stages
important individuals
seed banks
disease
disturbance history
```

Não salvar obrigatoriamente todos os detalhes visuais.

---

# 102. VEG-99 — LOD Persistence

Uma região distante pode armazenar:

```text
forest biomass
species percentages
regeneration state
```

Quando se aproxima:

```text
REGIONAL
 ↓
instantiate local plants
```

---

# 103. VEG-100 — Rehydration

Quando uma região volta a ficar ativa:

```text
regional vegetation state
 ↓
generate individuals
 ↓
continue simulation
```

Isso evita salvar milhões de entidades.

---

# 104. VEG-101 — Chunk Integration

```text
Vegetation
 ↓
Plant Instance
 ↓
Voxel/Structure representation
 ↓
Chunk Engine
```

O Chunk Engine continua sendo responsável por guardar os voxels.

---

# 105. VEG-102 — WorldGen Integration

O WorldGen fornece:

```text
initial species
initial density
initial age distribution
```

Vegetation continua a evolução.

---

# 106. VEG-103 — Climate Integration

Vegetation consulta:

```text
temperature
humidity
rainfall
season
light
```

---

# 107. VEG-104 — Fluid Integration

Vegetation consulta:

```text
soil moisture
surface water
groundwater access
```

---

# 108. VEG-105 — Lighting Integration

Plantas podem consultar:

```text
light level
daily light exposure
canopy light
```

---

# 109. VEG-106 — Ecology Integration

Ecology consulta:

```text
food availability
habitats
biomass
species
```

---

# 110. VEG-107 — Mob Integration

Mobs consultam:

```text
food
shelter
cover
habitat
```

Vegetation recebe:

```text
grazing
damage
seed dispersal
```

---

# 111. VEG-108 — Fire Integration

```text
Fire
 ↓
Vegetation
 ↓
damage/mortality
 ↓
regrowth
```

---

# 112. VEG-109 — Climate Feedback

```text
Vegetation
 ↓
evapotranspiration
soil effects
albedo-like properties
 ↓
Climate
```

---

# 113. VEG-110 — Mod API

Mods podem registrar:

```text
PlantSpecies
GrowthModel
ReproductionModel
VegetationProfile
PlantDisease
PollinatorRelationship
VegetationBiomeRule
AgricultureCrop
```

---

# 114. VEG-111 — Official Content

Conteúdo oficial e mods usam:

```text
Vegetation API
```

sem API secreta.

---

# 115. VEG-112 — Debug

Comandos:

```text
nexora vegetation inspect
nexora vegetation simulate
nexora vegetation species
nexora vegetation population
nexora vegetation health
```

Visualizações:

```text
density
species
biomass
growth
soil moisture
competition
habitat
```

---

# 116. VEG-113 — Profiler

Métricas:

```text
active plants
population simulations
growth tasks
reproduction tasks
disease simulations
vegetation memory
render instances
```

---

# 117. VEG-114 — Stress Tests

Testar:

```text
grassland
forest
jungle
wetland
ocean
cave forest
giant tree region
city
farmland
```

---

# 118. VEG-115 — Extreme World Test

```text
10.000 hectares equivalent region
 ↓
huge vegetation population
 ↓
simulation LOD
 ↓
player enters
 ↓
regional → local
```

A transição precisa ser consistente.

---

# 119. VEG-116 — Final Architecture

```text
                         ENVIRONMENT
                              │
            ┌─────────────────┼─────────────────┐
            │                 │                 │
         CLIMATE            WATER             LIGHT
            │                 │                 │
            └─────────────────┼─────────────────┘
                              ↓
                         VEGETATION
                              │
             ┌────────────────┼─────────────────┐
             │                │                 │
          GROWTH         REPRODUCTION       ECOLOGY
             │                │                 │
          stages           seeds             habitat
          health           pollen            food
          aging            spores            biomass
             │                │                 │
             └────────────────┼─────────────────┘
                              ↓
                         DISTURBANCE
             ┌────────────────┼────────────────┐
             │                │                │
           FIRE             DROUGHT          FLOOD
             │                │                │
             └────────────────┼────────────────┘
                              ↓
                          SUCCESSION
                              ↓
                           BIOMASS
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
       MOBS              AGRICULTURE          CIVILIZATION
        │                     │                     │
      food                 crops                forestry
      habitat              irrigation            trade
      migration             harvest              economy
```

# 120. Ordem de implementação

Eu faria:

```text
VEG-0 Plant Definition
VEG-1 Plant Instance
VEG-2 Lifecycle
VEG-3 Growth
VEG-4 Population
VEG-5 Seeds
VEG-6 Reproduction
VEG-7 Soil Integration
VEG-8 Climate Integration
VEG-9 Water Integration
VEG-10 Lighting Integration
VEG-11 Competition
VEG-12 Seasonal System
VEG-13 Herbivory
VEG-14 Disease
VEG-15 Fire
VEG-16 Succession
VEG-17 Regeneration
VEG-18 Decomposition
VEG-19 Biomass
VEG-20 Agriculture
VEG-21 Forestry
VEG-22 Aquatic Vegetation
VEG-23 Underground Vegetation
VEG-24 Giant Plants
VEG-25 Procedural Plants
VEG-26 Genetics/Variation
VEG-27 LOD
VEG-28 Regional Simulation
VEG-29 Persistence
VEG-30 Civilization Integration
VEG-31 Economy Integration
VEG-32 Ecology Integration
VEG-33 Renderer Integration
VEG-34 Mod API
VEG-35 Debugging
VEG-36 Stress Testing
```

# 121. Primeiro Vertical Slice

O primeiro slice deveria ser:

```text
WorldGen
 ↓
Biome
 ↓
Soil
 ↓
Temperature
 ↓
Water
 ↓
Light
 ↓
Seed
 ↓
Germination
 ↓
Plant Growth
 ↓
Mature Plant
 ↓
Reproduction
 ↓
New Seed
```

Depois:

```text
Forest
 ↓
Herbivores
 ↓
Grazing
 ↓
Plant damage
 ↓
Seed dispersal
 ↓
Regeneration
```

E o teste realmente interessante:

```text
Forest
 ↓
100 anos
 ↓
fire
 ↓
vegetation loss
 ↓
pioneer plants
 ↓
succession
 ↓
young forest
 ↓
mature forest
```

## Regra principal

Eu colocaria:

> **Vegetation não gera apenas árvores. Ela simula biomassa viva.**

Então:

```text
Climate
      ↓
Water ─────┐
           ↓
          SOIL
           ↓
         PLANTS
           ↓
      BIOMASS / HABITAT
       ↙           ↘
     MOBS         CIVILIZATION
       ↓              ↓
   herbivory       farming
   dispersal       forestry
       ↓              ↓
       └──────→ VEGETATION
                   ↑
              regeneration
```

Isso faz a vegetação virar uma peça real do **mundo vivo** do NEXORA, em vez de decoração estática.
