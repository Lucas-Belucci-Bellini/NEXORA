# NEXORA

# MASTER PLAN — CAVE ENGINE

# DEEP WORLD SYSTEM

# VOID DIMENSION

> Estes três sistemas fazem parte do World Engine do NEXORA, porém possuem responsabilidades diferentes.
>
> **Cave Engine** é responsável pela formação e estrutura subterrânea.
>
> **Deep World System** é responsável pela existência, progressão e simulação das grandes profundidades do mundo.
>
> **Void Dimension** é responsável pelo espaço além da Bedrock.
>
> Eles devem ser independentes, mas interoperáveis.

---

# PARTE I — CAVE ENGINE

# 1. OBJETIVO

O Cave Engine será o sistema responsável por criar a estrutura subterrânea do NEXORA.

Não será apenas:

```text
3D Noise
↓
remover blocos
```

Ele deverá produzir:

```text
cavernas
túneis
poços
câmaras
abismos
rios subterrâneos
lagos subterrâneos
sistemas de cavernas
mega-cavernas
formações naturais
regiões subterrâneas
```

---

# 2. RESPONSABILIDADE

O Cave Engine cuida de:

```text
FORMA
GEOMETRIA
CONECTIVIDADE
ESTRUTURA
```

Ele não deve ser responsável diretamente por:

```text
economia
civilizações
quest progression
UI
inventário
tecnologia
```

Esses sistemas pertencem ao Deep World e outros módulos.

---

# 3. CAVE GENERATION PIPELINE

```text
Seed
↓
Cave Seed
↓
Geological Context
↓
Density Field
↓
Cave Potential
↓
Cave Formation
↓
Connectivity
↓
Hydrology
↓
Cave Biome
↓
Features
↓
Structures
```

---

# 4. CAVE TYPES

Criar famílias:

```text
SMALL CAVES
TUNNELS
CHAMBERS
SHAFTS
RAVINES
MEGA CAVERNS
ABYSSAL CAVES
UNDERGROUND OCEANS
CONNECTED CAVE SYSTEMS
```

---

# 5. SMALL CAVES

Pequenas formações para quebrar a uniformidade subterrânea.

---

# 6. TUNNELS

Sistemas lineares:

```text
horizontal
vertical
diagonal
branching
```

---

# 7. CHAMBERS

Grandes salas naturais.

Possíveis características:

```text
water
lava
vegetation
crystals
structures
resources
```

---

# 8. SHAFTS

Poços verticais que podem conectar múltiplas profundidades.

---

# 9. RAVINES

Grandes cortes no terreno subterrâneo.

---

# 10. MEGA CAVERNS

O sistema deverá suportar cavidades de escala muito maior que uma caverna comum.

Exemplo:

```text
Mega Cavern
├── floor
├── ceiling
├── rivers
├── mountains
├── vegetation
├── resources
└── settlements
```

---

# 11. UNDERGROUND CONTINENTS

Em regiões extremamente profundas, uma única caverna poderá possuir escala de região.

Isso permite:

```text
florestas subterrâneas
mares subterrâneos
montanhas internas
cidades
```

---

# 12. CAVE CONNECTIVITY

As cavernas podem formar grafos.

```text
Cave A
 ├── Cave B
 ├── Cave C
 └── Cave D
```

Criar um:

```text
CaveNetwork
```

quando apropriado.

---

# 13. CAVE HYDROLOGY

Integrar ao sistema de água:

```text
rainfall
surface rivers
water table
aquifers
underground lakes
underground rivers
```

---

# 14. CAVE CLIMATE

Grandes sistemas subterrâneos poderão possuir microclimas.

Variáveis:

```text
temperature
humidity
airflow
moisture
light
```

---

# 15. CAVE BIOMES

O Cave Engine fornece geometria.

O Biome Engine pode determinar:

```text
fungal cave
crystal cave
wet cave
deep forest
lava cave
ice cave
```

---

# 16. GEOLOGICAL FORMATIONS

Gerar:

```text
stalactites
stalagmites
columns
crystals
rock arches
terraces
```

---

# 17. CAVE FEATURES

Permitir recursos especiais:

```text
glowing plants
underground waterfalls
geothermal vents
crystal formations
ancient structures
```

---

# 18. CAVE MATERIALS

A composição deve depender de:

```text
depth
geology
temperature
water
world region
```

---

# 19. CAVE RESOURCES

Permitir recursos exclusivos de determinadas profundidades e formações.

---

# 20. CAVE STRUCTURES

Estruturas podem ser naturais ou artificiais.

Exemplo:

```text
ruins
bridges
shafts
underground roads
old mines
ancient buildings
```

---

# 21. CAVE CIVILIZATION HOOK

O Cave Engine não cria a civilização.

Ele fornece locais onde o Deep World pode colocar:

```text
settlements
cities
ruins
infrastructure
```

---

# 22. CAVE GENERATION LAYERS

O Cave Engine deve permitir:

```text
surface caves
upper caves
mid caves
deep caves
abyssal caves
```

---

# 23. PERFORMANCE

A geração subterrânea deve usar:

```text
streaming
caching
parallel generation
distance priorities
```

---

# 24. CAVE CHUNK INTEGRATION

O Cave Engine deverá trabalhar com chunks sem assumir como o renderer funciona.

Contrato:

```text
CaveData
↓
World Chunk
```

---

# 25. CAVE API

Expor APIs como:

```text
registerCaveFeature()
registerCaveType()
registerCaveBiomeHook()
registerCaveStructure()
```

---

# 26. MOD SUPPORT

Mods podem adicionar:

```text
cave feature
cave biome
cave structure
cave material
```

sem modificar diretamente o Cave Engine.

---

# 27. CAVE TESTING

Testar:

```text
small caves
large caves
mega caves
cave networks
underground water
deep caves
extreme depth
```

---

# 28. CAVE ENGINE SUCCESS

O sistema estará funcional quando puder gerar:

```text
uma caverna
um sistema conectado
uma mega caverna
um ecossistema subterrâneo
```

de maneira determinística e estável.

---

# PARTE II — DEEP WORLD SYSTEM

# 29. OBJETIVO

O Deep World System transforma a profundidade do NEXORA em um **mundo jogável**.

Não será apenas:

```text
Y menor
```

Será:

```text
profundidade
+
ecologia
+
recursos
+
progressão
+
civilização
+
história
+
infraestrutura
```

---

# 30. ESCALA VERTICAL

O mundo terá:

```text
MAX HEIGHT
+1920
```

e:

```text
BEDROCK
-1920
```

Total:

```text
3840 blocos
```

---

# 31. PROFUNDITY MODEL

Criar:

```text
DepthProfile
```

capaz de determinar:

```text
surface
upper_deep
mid_deep
deep
extreme_deep
abyss
bedrock
```

---

# 32. 15 DEEP LAYERS

O mundo subterrâneo terá 15 grandes camadas conceituais.

As fronteiras exatas não precisam necessariamente ser linhas fixas.

Cada camada deve possuir:

```text
terrain
biome families
resources
temperature
pressure
light
entities
structures
progression
```

---

# 33. LAYERS 1–8

As primeiras oito camadas deverão ser alcançáveis usando tecnologia construída principalmente dentro do mundo terrestre.

Diamante será um marco importante de progressão inicial.

---

# 34. LAYERS 1–8 — PROGRESSÃO

Exemplo:

```text
surface
↓
iron
↓
steel
↓
diamond
↓
deep tools
```

Os valores reais serão determinados pelo balanceamento.

---

# 35. LAYERS 9–15

As camadas mais profundas deverão exigir recursos provenientes de sistemas mais avançados.

Podem envolver:

```text
Nether
End
Magic
Technology
Dimensions
```

---

# 36. MULTIDIMENSIONAL PROGRESSION

A progressão poderá funcionar:

```text
Deep Layer
↓
resource requirement
↓
dimension
↓
advanced resource
↓
new tool
↓
deeper layer
```

---

# 37. TINKERS INTEGRATION

O sistema de ferramentas modular poderá criar equipamentos capazes de avançar para profundidades maiores.

```text
Tool
+
Material
+
Modifier
=
Depth Capability
```

---

# 38. DEPTH CAPABILITY

Cada equipamento poderá possuir:

```text
miningStrength
temperatureResistance
pressureResistance
durability
specialCapabilities
```

---

# 39. ENVIRONMENTAL PRESSURE

Profundidades maiores podem introduzir:

```text
pressure
temperature
darkness
oxygen restrictions
hazardous fluids
```

Isso deve ser implementado de maneira configurável.

---

# 40. UNDERGROUND ECOLOGY

Cada profundidade pode possuir:

```text
flora
fauna
fungi
predators
resources
```

---

# 41. UNDERGROUND CIVILIZATIONS

O Deep World System será responsável por possibilitar civilizações subterrâneas.

Estrutura:

```text
Underground Civilization
├── population
├── settlements
├── culture
├── technology
├── economy
├── government
├── history
└── infrastructure
```

---

# 42. ANCIENT CIVILIZATIONS

Algumas civilizações podem existir apenas como ruínas.

Mas outras poderão continuar ativas.

---

# 43. CIVILIZATION AGE

Cada sociedade pode possuir:

```text
ancient
old
established
developing
declining
```

---

# 44. UNDERGROUND SETTLEMENTS

Tipos:

```text
camp
outpost
village
town
city
underground megacity
```

---

# 45. UNDERGROUND ECONOMY

Cada settlement poderá possuir:

```text
production
consumption
storage
trade
currency
wealth
```

---

# 46. UNDERGROUND TRADE

Rotas poderão ligar:

```text
mine
settlement
city
surface
other settlements
```

---

# 47. SURFACE ↔ DEEP TRADE

O jogador poderá estabelecer comércio entre superfície e submundo.

---

# 48. QUEST SYSTEM

NPCs subterrâneos poderão oferecer missões baseadas em:

```text
resource shortages
trade
exploration
infrastructure
politics
discovery
```

---

# 49. UNDERGROUND TRANSPORT

Preparar:

```text
road
rail
minecart
cargo
elevator
```

---

# 50. VERTICAL INFRASTRUCTURE

Criar sistemas para:

```text
mine shafts
elevators
vertical railways
lifts
```

---

# 51. DEEP WORLD EVENTS

Possíveis eventos:

```text
cave collapse
migration
resource discovery
settlement growth
resource depletion
ancient site discovery
```

---

# 52. DEEP WORLD HISTORY

O mundo poderá registrar acontecimentos antigos.

Exemplo:

```text
civilization founded
war
migration
collapse
rediscovery
```

---

# 53. STORYTELLING THROUGH WORLD

A história pode aparecer através de:

```text
ruins
inscriptions
artifacts
structures
NPCs
libraries
maps
```

---

# 54. DEEP RESOURCE NETWORK

Recursos profundos devem criar motivos reais para explorar.

---

# 55. DEEP RESOURCE PROGRESSION

Exemplo:

```text
surface resources
↓
deep resources
↓
dimensional resources
↓
advanced tools
↓
deeper resources
```

---

# 56. DEPTH LOCKS

Algumas profundidades podem ser limitadas por:

```text
tool capability
environmental protection
resource requirement
world progression
```

---

# 57. NOT ARTIFICIAL WALLS

Não criar somente uma mensagem:

```text
"Você não pode descer."
```

O bloqueio deve surgir do próprio mundo.

Exemplo:

```text
pressure
temperature
hard material
environment
```

---

# 58. BEDROCK

A Bedrock fica em:

```text
-1920
```

Ela representa o limite do mundo físico principal.

---

# 59. VOID TRANSITION

Após a Bedrock existe uma transição especial:

```text
physical world
↓
bedrock
↓
void boundary
↓
void dimension
```

---

# PARTE III — VOID DIMENSION

# 60. OBJETIVO

A Void Dimension será um mundo além da geologia normal do NEXORA.

Ela não será simplesmente:

```text
céu preto
```

Será uma dimensão própria.

---

# 61. VOID DIMENSION MODEL

Possuir:

```text
terrain
regions
structures
entities
resources
physics rules
lighting
```

---

# 62. RELAÇÃO COM A BEDROCK

O acesso conceitual ocorrerá:

```text
Deep World
↓
Bedrock
↓
Void Boundary
↓
Void
```

A implementação real do acesso poderá usar uma transição dimensional apropriada.

---

# 63. VOID GENERATOR

O Void Dimension possuirá seu próprio generator.

Não reutilizar o Overworld generator automaticamente.

---

# 64. VOID TERRAIN

O terreno pode ser radicalmente diferente.

Exemplos:

```text
floating islands
void plains
fractured terrain
fragmented structures
```

Os exemplos não são requisitos fixos.

---

# 65. VOID PHYSICS

A dimensão pode possuir regras próprias:

```text
gravity
movement
fall behavior
light
```

---

# 66. VOID BIOMES

Criar famílias próprias.

Exemplo:

```text
void forest
void crystal field
dark expanse
floating ecosystem
```

---

# 67. VOID RESOURCES

Recursos exclusivos.

Eles podem alimentar:

```text
dimensional technology
advanced tools
space progression
deep-world systems
```

---

# 68. VOID ENTITIES

Criar fauna/entidades próprias do NEXORA.

---

# 69. VOID STRUCTURES

Possíveis:

```text
ancient stations
ruins
floating structures
dimensional gates
```

---

# 70. VOID CIVILIZATIONS

Pode existir uma civilização adaptada à dimensão.

Ela possui:

```text
economy
culture
technology
settlements
history
```

---

# 71. VOID HISTORY

A Void Dimension poderá guardar pistas sobre:

```text
origem do mundo
civilizações antigas
dimensões
Far Lands
```

A história deve ser criada para o universo do NEXORA.

---

# 72. VOID ECONOMY

Se houver civilização:

```text
currency
trade
resources
markets
```

---

# 73. VOID QUESTS

Missões relacionadas a:

```text
exploration
relics
resources
ancient structures
dimensional events
```

---

# 74. VOID LOGISTICS

Preparar transporte dimensional futuramente.

---

# 75. VOID → OTHER DIMENSIONS

A Void Dimension pode funcionar como uma zona de ligação para partes do universo dimensional.

Não precisa ser um hub obrigatório.

---

# 76. VOID + SPACE

A dimensão pode possuir relação narrativa e tecnológica com o sistema espacial.

---

# 77. VOID + FAR LANDS

As Far Lands podem servir como uma das pistas que levam ao conceito de Void.

Não tornar obrigatoriamente o único caminho.

---

# 78. VOID DISCOVERY

A existência da dimensão deve ser descoberta, não necessariamente revelada imediatamente.

---

# 79. WORLDGEN INTEGRATION

Arquitetura:

```text
World Generator
      │
      ├── Cave Engine
      │
      ├── Deep World System
      │
      └── Dimension System
               │
               └── Void Generator
```

---

# 80. CONTRATO ENTRE CAVE ENGINE E DEEP WORLD

O Cave Engine fornece:

```text
CaveGeometry
CaveNetwork
CaveFeatures
CaveMetadata
```

O Deep World consome esses dados para criar:

```text
biomes
resources
settlements
civilizations
```

---

# 81. CONTRATO ENTRE DEEP WORLD E VOID

O Deep World fornece:

```text
depth state
bedrock boundary
discovery state
dimensional transition
```

O Void fornece:

```text
VoidWorld
```

---

# 82. NÃO ACOPLAR SISTEMAS

Não permitir:

```text
Cave Engine
↓
diretamente
Economy
```

ou:

```text
Void Dimension
↓
diretamente
Inventory internals
```

Usar APIs.

---

# 83. TESTE DE INTEGRAÇÃO

Criar:

```text
World
↓
Cave
↓
Deep Layer
↓
Underground Settlement
↓
Deep Resource
↓
Advanced Tool
↓
Bedrock
↓
Void
```

---

# 84. SAVE/LOAD

Salvar:

```text
cave state
deep layer state
settlement state
economy state
void state
```

---

# 85. PLAYER MODIFICATION

O jogador pode alterar:

```text
caves
settlements
railways
mines
structures
```

e essas alterações devem persistir.

---

# 86. PERFORMANCE

Todos os três sistemas devem suportar streaming.

Não gerar os 3840 blocos verticais completos de uma vez.

---

# 87. GENERATION ON DEMAND

Gerar conforme:

```text
player
exploration
simulation
```

e necessidade do mundo.

---

# 88. DEBUG TOOLS

Criar ferramentas:

```text
depth map
cave map
cave network
underground biome map
settlement map
resource map
void map
```

---

# 89. TEST SEEDS

Manter seeds específicas para:

```text
mega cave
underground ocean
civilization
deep resources
bedrock
void transition
```

---

# 90. METRICS

Medir:

```text
cave generation time
deep chunk generation
memory
streaming
save time
void generation
```

---

# 91. MOD API

Mods poderão adicionar:

```text
cave features
deep biomes
deep resources
underground structures
deep civilizations
void biomes
void structures
void resources
```

sem modificar o Core.

---

# 92. CONTENT SEPARATION

A separação conceitual será:

```text
CAVE ENGINE
→ como o espaço subterrâneo existe

DEEP WORLD
→ o que existe e acontece dentro dele

VOID DIMENSION
→ o que existe além do mundo físico
```

---

# 93. ROADMAP

## CAVE-0

```text
basic caves
```

## CAVE-1

```text
cave networks
```

## CAVE-2

```text
mega caves
```

## CAVE-3

```text
hydrology
```

## CAVE-4

```text
cave biomes
```

## CAVE-5

```text
structures
```

---

## DEEP-0

```text
depth model
```

## DEEP-1

```text
15 layers
```

## DEEP-2

```text
resources
```

## DEEP-3

```text
environmental progression
```

## DEEP-4

```text
settlements
```

## DEEP-5

```text
economy
```

## DEEP-6

```text
civilizations
```

## DEEP-7

```text
infrastructure
```

## DEEP-8

```text
story/history
```

---

## VOID-0

```text
Void API
```

## VOID-1

```text
Void Generator
```

## VOID-2

```text
Void Biomes
```

## VOID-3

```text
Void Resources
```

## VOID-4

```text
Void Structures
```

## VOID-5

```text
Void Civilization
```

## VOID-6

```text
Dimensional Integration
```

---

# 94. PRIMEIRO OBJETIVO DO SISTEMA

O primeiro protótipo não precisa possuir todas as 15 camadas.

Precisa provar:

```text
surface
↓
cave
↓
deep layer
↓
mega cave
↓
deeper layer
↓
bedrock
↓
void
```

---

# 95. SEGUNDO OBJETIVO

Provar:

```text
deep layer
↓
biome
↓
resources
↓
settlement
↓
NPCs
↓
economy
```

---

# 96. TERCEIRO OBJETIVO

Provar:

```text
surface
↓
railway
↓
deep civilization
↓
resource extraction
↓
return to surface
```

---

# 97. QUARTO OBJETIVO

Provar:

```text
deep world
↓
bedrock
↓
void
↓
void resource
↓
return
```

---

# 98. TESTE DE LONGA DISTÂNCIA

O jogador deverá conseguir explorar grandes profundidades sem:

```text
teleporte obrigatório
geração incoerente
perda de estado
corromper save
```

---

# 99. DEFINIÇÃO DE SUCESSO

Os sistemas estarão maduros quando:

```text
[ ] cave generation
[ ] cave networks
[ ] mega caves
[ ] underground hydrology
[ ] deep biome integration
[ ] 15 layers
[ ] deep progression
[ ] deep resources
[ ] underground settlements
[ ] underground economy
[ ] civilizations
[ ] infrastructure
[ ] Bedrock boundary
[ ] Void Dimension
[ ] Void generation
[ ] Void biomes
[ ] Void resources
[ ] Void structures
[ ] save/load
[ ] streaming
[ ] performance
[ ] debug tools
[ ] mod API
```

---

# 100. REGRA FINAL

Os três sistemas nunca devem virar um único monstro de código.

A separação deve permanecer:

```text
                NEXORA WORLD ENGINE
                       │
        ┌──────────────┼──────────────┐
        │              │              │
        ▼              ▼              ▼
   CAVE ENGINE     DEEP WORLD      VOID DIMENSION
        │              │              │
     geometry       simulation      dimension
        │              │              │
        └──────────────┼──────────────┘
                       │
                 SHARED WORLD APIs
```

O Cave Engine sabe **como criar o espaço subterrâneo**.

O Deep World System sabe **como esse espaço funciona como mundo**.

A Void Dimension sabe **como existe o mundo além da Bedrock**.

Nenhum dos três deve precisar conhecer as implementações internas dos outros.

---

# FRASE DOS TRÊS SISTEMAS

> **Beneath the world, the world continues.**
