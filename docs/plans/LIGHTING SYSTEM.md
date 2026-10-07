Sim. **Iluminação** merece um sistema próprio no NEXORA, separado do Renderer, porque ela vai precisar funcionar em superfície, cavernas, oceano, Deep World, dimensões, máquinas emissivas e espaço.

A separação que eu faria é:

```text
WORLD
  ↓
LIGHTING SYSTEM
  ├── Light Sources
  ├── Light Propagation
  ├── Sky Light
  ├── Block Light
  ├── Dynamic Lights
  ├── Shadows
  ├── Ambient Lighting
  ├── Volumetric Light
  └── Dimension Lighting
        ↓
RENDERER
```

# NEXORA — LIGHTING MASTER PLAN

## 1. Objetivo

A iluminação deve transformar o estado físico do mundo em informação visual de luz.

Ela precisa responder:

```text
Onde existe luz?
Quanto de luz existe?
De onde ela veio?
Como ela se propaga?
O que bloqueia a luz?
Como a luz muda?
Como o Renderer deve enxergá-la?
```

E principalmente:

> **Iluminação não deve ser apenas um efeito visual. Ela precisa fazer parte do estado do mundo.**

Isso é importante para cavernas, plantas, mobs, NPCs, máquinas, agricultura, clima e exploração subterrânea.

---

# 2. Arquitetura geral

```text
                    LIGHTING
                        │
        ┌───────────────┼────────────────┐
        │               │                │
      SOURCE         PROPAGATION       BLOCKING
        │               │                │
   Sun / Moon       Sky Light          Voxel
   Torch            Block Light        Structure
   Machine          Dynamic Light     Terrain
   Magic            Local Lights
   Vehicle
        │               │                │
        └───────────────┼────────────────┘
                        │
                 LIGHT REPRESENTATION
                        │
          ┌─────────────┼─────────────┐
          │             │             │
       WORLD         PHYSICS       RENDERER
          │             │             │
      simulation     queries       shading
```

---

# 3. LIGHT-0 — Light Data Model

Criar uma estrutura abstrata:

```text
Light
├── id
├── type
├── position
├── color
├── intensity
├── range
├── direction
├── source
└── flags
```

Tipos:

```text
SUN
MOON
BLOCK
POINT
SPOT
AREA
EMISSIVE
MAGICAL
VEHICLE
MACHINE
CUSTOM
```

Mas o sistema voxel também precisa de uma representação extremamente compacta para iluminação por célula.

---

# 4. LIGHT-1 — Voxel Light Data

Cada região do mundo pode possuir dados de iluminação associados aos voxels.

Separar:

```text
SkyLight
BlockLight
```

e futuramente outras propriedades.

Conceitualmente:

```text
Voxel
├── BlockState
├── SkyLight
└── BlockLight
```

O armazenamento real deve ser compactado.

---

# 5. LIGHT-2 — Light Level

Começar com níveis discretos para a iluminação voxel:

```text
0 → ausência de luz
...
15 → máximo
```

Essa estrutura é extremamente barata e adequada para propagação em larga escala.

Depois podemos ter uma representação mais rica no Renderer.

---

# 6. LIGHT-3 — Sky Light

Separar a luz ambiental proveniente do céu da luz emitida por blocos.

```text
Sky Light
    ↓
Atmosphere
    ↓
World
```

Ela depende de:

```text
sun position
weather
clouds
time
dimension
atmosphere
```

---

# 7. LIGHT-4 — Sunlight

Criar:

```text
SunLight
```

com:

```text
direction
intensity
color
angle
```

A direção muda conforme o relógio do mundo.

```text
World Time
   ↓
Sun Position
   ↓
Lighting
   ↓
Renderer
```

---

# 8. LIGHT-5 — Moonlight

Mesma arquitetura para Lua:

```text
MoonLight
```

Pode depender de:

```text
moon phase
time
dimension
atmosphere
```

---

# 9. LIGHT-6 — Day / Night

Criar um perfil temporal:

```text
DAWN
DAY
DUSK
NIGHT
```

Mas evitar transições abruptas.

Usar curvas:

```text
World Time
 ↓
Lighting Curve
 ↓
Intensity / Color
```

---

# 10. LIGHT-7 — Sun Color

A cor solar pode variar durante o dia.

Conceitualmente:

```text
sunrise
→ warm

day
→ neutral

sunset
→ warm

night
→ moon dominated
```

O sistema deve ser parametrizado, não hardcoded em cada shader.

---

# 11. LIGHT-8 — Weather Lighting

Clima altera a iluminação:

```text
clear
cloudy
storm
rain
snow
fog
```

Exemplo:

```text
Clouds
 ↓
Reduced direct light
 ↓
Changed ambient light
```

---

# 12. LIGHT-9 — Block Emission

Blocos podem emitir luz.

Um `BlockDefinition` pode possuir propriedades como:

```text
emissionLevel
emissionColor
emissionProfile
```

Exemplos conceituais:

```text
torch
lantern
crystal
reactor
machine
magic structure
```

---

# 13. LIGHT-10 — Emissive Objects

Além dos blocos:

```text
Entity
Vehicle
Machine
Item
NPC
```

podem emitir luz.

```text
Entity
 ↓
LightEmitterComponent
 ↓
Lighting System
```

---

# 14. LIGHT-11 — Dynamic Lights

Criar um sistema para fontes que mudam de posição:

```text
DynamicLight
```

Exemplos:

```text
player-held light
vehicle headlight
moving machine
magical object
```

Não atualizar todo o mundo a cada frame.

Separar:

```text
Voxel lighting
```

de:

```text
Real-time dynamic lighting
```

---

# 15. LIGHT-12 — Light Propagation

A propagação voxel pode ser incremental.

```text
Light Source
 ↓
Queue
 ↓
Neighbors
 ↓
Propagation
 ↓
Updated voxels
```

Quando a fonte aumenta:

```text
propagate
```

Quando diminui:

```text
remove
recalculate
```

---

# 16. LIGHT-13 — Flood Fill / Propagation

A propagação pode usar filas especializadas.

Conceitualmente:

```text
Add Queue
Remove Queue
```

para evitar recalcular o mundo inteiro.

---

# 17. LIGHT-14 — Light Blocking

Cada voxel precisa informar como interfere com a luz.

Propriedades:

```text
opaque
transparent
translucent
emissive
```

Podemos ter também:

```text
lightOpacity
```

Assim:

```text
stone
→ blocks strongly

glass
→ allows passage

leaves
→ partially blocks
```

---

# 18. LIGHT-15 — Partial Transparency

Nem tudo é:

```text
0%
ou
100%
```

Alguns materiais podem absorver parcialmente:

```text
fog-like blocks
ice
leaves
water
special materials
```

---

# 19. LIGHT-16 — Colored Light

A arquitetura deve suportar luz colorida.

```text
Light
├── intensity
└── RGB
```

Exemplos:

```text
red crystal
blue reactor
green magical source
```

Na parte voxel, a representação precisa ser compacta e cuidadosamente limitada para não explodir a memória.

---

# 20. LIGHT-17 — Color Propagation

Futuramente:

```text
Source Color
 ↓
Propagation
 ↓
Accumulation
```

Quando várias fontes se encontram:

```text
red + blue
→ combined contribution
```

O método exato deve ser escolhido com testes de custo/qualidade.

---

# 21. LIGHT-18 — Cave Lighting

Esse será um dos maiores testes do NEXORA.

Uma caverna:

```text
surface
 ↓
entrance
 ↓
deep cave
 ↓
mega cave
 ↓
deep underground
```

deve perder gradualmente a influência da luz natural.

Isso precisa continuar funcionando em áreas enormes e complexas.

---

# 22. LIGHT-19 — Deep World Lighting

Nas camadas mais profundas:

```text
depth
 ↓
sky contribution decreases
 ↓
artificial / biological light becomes dominant
```

Isso conecta iluminação ao Deep World sem o Lighting System precisar conhecer “camada 12”, por exemplo.

Ele recebe apenas:

```text
sky availability
environment
sources
```

---

# 23. LIGHT-20 — Ocean Lighting

No oceano:

```text
surface
 ↓
water volume
 ↓
light attenuation
 ↓
depth
```

A iluminação deve diminuir conforme a profundidade.

---

# 24. LIGHT-21 — Underwater Light

Separar:

```text
surface lighting
```

de:

```text
underwater lighting
```

com integração com:

```text
Water Renderer
Atmosphere
Fog
Depth
```

---

# 25. LIGHT-22 — Fluid Attenuation

Diferentes fluidos podem alterar iluminação.

Criar algo como:

```text
LightAttenuationProfile
```

que o Fluid System fornece.

Assim:

```text
water
lava
special fluid
```

podem ter comportamentos diferentes.

---

# 26. LIGHT-23 — Ambient Light

Além das fontes explícitas:

```text
AmbientLight
```

representa iluminação indireta simplificada.

Pode vir de:

```text
sky
world environment
dimension
biome
weather
GI approximation
```

---

# 27. LIGHT-24 — Ambient Occlusion

Separar AO de iluminação direta.

```text
Geometry
 ↓
Neighbor data
 ↓
AO
 ↓
Renderer
```

No voxel world:

```text
corner
edge
cavity
```

podem ficar naturalmente mais escuros.

---

# 28. LIGHT-25 — Global Illumination Layer

O sistema deve ser preparado para diferentes níveis de GI:

```text
LEVEL 0
basic ambient

LEVEL 1
voxel indirect approximation

LEVEL 2
advanced probes / screen-space methods

LEVEL 3
hardware-accelerated options
```

Assim não obrigamos todas as máquinas a usar a técnica mais cara.

---

# 29. LIGHT-26 — Light Probes

Para objetos móveis:

```text
LightProbe
```

pode fornecer:

```text
ambient color
directional lighting
environment lighting
```

Útil para:

```text
NPC
vehicle
player
items
```

---

# 30. LIGHT-27 — Probe Volumes

Para interiores:

```text
Building
 ↓
Probe Volume
 ↓
Local lighting
```

Assim um NPC entrando em uma casa não precisa depender apenas da luz do sol.

---

# 31. LIGHT-28 — Interior Lighting

Edifícios podem possuir:

```text
ceiling lights
wall lights
windows
machines
screens
```

e o sistema deve tratar interiores e exteriores sem criar duas engines diferentes.

---

# 32. LIGHT-29 — Shadow Interface

Aqui entra uma separação importante:

**Lighting calcula a iluminação.
Renderer calcula a representação visual das sombras.**

Portanto:

```text
Lighting
→ light sources / direction / intensity

Renderer
→ shadow maps / filtering / rendering
```

---

# 33. LIGHT-30 — Shadow Caster

Objetos podem marcar:

```text
castShadow
receiveShadow
```

Isso vale para:

```text
blocks
entities
vehicles
structures
vegetation
```

---

# 34. LIGHT-31 — Sun Shadows

O sol precisa possuir uma sombra global.

Como o mundo é enorme, usar arquitetura baseada em:

```text
cascades
LOD
camera-relative shadowing
```

---

# 35. LIGHT-32 — Local Shadows

Fontes próximas podem possuir sombras locais.

Exemplo:

```text
lamp
 ↓
nearby wall
 ↓
shadow
```

Não precisa transformar toda luz pequena em um shadow map caro.

---

# 36. LIGHT-33 — Shadow Priority

Prioridade:

```text
Sun
→ highest

Large nearby lights
→ high

Medium lights
→ normal

Tiny distant lights
→ low / no shadow
```

---

# 37. LIGHT-34 — Lighting LOD

A iluminação também precisa de LOD.

```text
Near
→ full dynamic lighting

Medium
→ simplified

Far
→ baked/proxy

Very Far
→ ambient approximation
```

---

# 38. LIGHT-35 — Simulation LOD Integration

O mundo distante não precisa recalcular cada fonte de luz individual.

```text
FULL
→ individual sources

REGIONAL
→ aggregated light data

ABSTRACT
→ environment lighting
```

Isso combina com o sistema de simulação que planejamos.

---

# 39. LIGHT-36 — Chunk Lighting

Cada chunk pode manter:

```text
LightingState
├── sky light
├── block light
├── dirty regions
├── dynamic lights
└── metadata
```

---

# 40. LIGHT-37 — Lighting Dirty State

Quando há alteração:

```text
Torch removed
 ↓
Lighting Dirty
```

ou:

```text
Block placed
 ↓
Light blockage changed
 ↓
Lighting Dirty
```

---

# 41. LIGHT-38 — Partial Recalculation

Não atualizar o chunk inteiro quando possível.

```text
dirty voxel
 ↓
affected region
 ↓
recalculate
```

---

# 42. LIGHT-39 — Neighbor Propagation

Luz pode atravessar fronteiras:

```text
Chunk A
   ↓
boundary
   ↓
Chunk B
```

Alterações precisam invalidar regiões vizinhas.

---

# 43. LIGHT-40 — Light Cache

Manter:

```text
Voxel Light Cache
Chunk Light Cache
Region Light Cache
Dynamic Light Cache
```

Mas evitar duplicar dados sem necessidade.

---

# 44. LIGHT-41 — Async Lighting

Atualizações maiores podem ser calculadas em workers:

```text
World Change
 ↓
Lighting Task
 ↓
Worker
 ↓
Updated Light Data
 ↓
Renderer
```

---

# 45. LIGHT-42 — Thread Safety

Usar snapshots ou regiões de trabalho:

```text
Live World
 ↓
Lighting Snapshot
 ↓
Worker
 ↓
Validated Result
 ↓
Commit
```

---

# 46. LIGHT-43 — Determinism

Com o mesmo:

```text
world state
lighting version
configuration
```

o resultado lógico deve ser reproduzível.

Muito útil para:

```text
tests
multiplayer
save/load
bug reproduction
```

---

# 47. LIGHT-44 — Multiplayer

A iluminação lógica derivada do mundo pode ser calculada de maneira consistente.

Mas efeitos puramente visuais podem ser exclusivos do cliente.

Separar:

```text
Authoritative lighting state
```

de:

```text
Client visual lighting
```

---

# 48. LIGHT-45 — Client-side Dynamic Light

Por exemplo, uma luz segurada pelo jogador pode ser puramente visual.

```text
Player
 ↓
Dynamic light
 ↓
Client Renderer
```

sem alterar o estado global do mundo, quando isso fizer sentido.

---

# 49. LIGHT-46 — Machines

Máquinas podem registrar:

```text
LightEmitter
```

Exemplo:

```text
reactor
machine
generator
control panel
```

O Machine System fornece:

```text
on/off
intensity
color
```

---

# 50. LIGHT-47 — Energy Integration

Energia e iluminação podem interagir.

Por exemplo, conceitualmente:

```text
Machine
 ↓
Energy available
 ↓
Light enabled
```

Mas a Lighting Engine não deve conhecer o Energy System internamente.

---

# 51. LIGHT-48 — Magical Lighting

O Magic System poderá fornecer:

```text
LightEmitterComponent
```

com propriedades personalizadas.

```text
magic source
 ↓
lighting
```

---

# 52. LIGHT-49 — Biological Light

Isso fica especialmente interessante para cavernas e Deep World.

Criaturas/plantas podem emitir luz:

```text
bioluminescent flora
bioluminescent organism
```

O Ecology System decide que algo é emissivo.

Lighting apenas processa:

```text
source
```

---

# 53. LIGHT-50 — Seasonal Lighting

A estação pode modificar:

```text
sun trajectory
day length
ambient conditions
```

O World Time/Climate System fornece os dados.

---

# 54. LIGHT-51 — Dimension Profiles

Cada dimensão pode registrar:

```text
DimensionLightingProfile
├── sun
├── moon
├── ambient
├── fog
├── atmosphere
├── sky
└── light rules
```

Exemplo conceitual:

```text
Surface
→ normal

Void
→ custom

Alien dimension
→ different sky

Magic dimension
→ unusual ambient light
```

---

# 55. LIGHT-52 — Dimension Transitions

Ao atravessar uma dimensão:

```text
Dimension A
 ↓
Transition
 ↓
Dimension B
```

a iluminação precisa fazer transição suave quando aplicável.

---

# 56. LIGHT-53 — Portals

Portais podem possuir:

```text
local light
emission
color
shadow interaction
```

---

# 57. LIGHT-54 — Special Regions

Far Lands/Beyondlands podem ter perfis próprios.

Não porque a Lighting Engine sabe que é Far Lands, mas porque a região/dimensão fornece um:

```text
Environment Lighting Profile
```

---

# 58. LIGHT-55 — Dynamic Exposure

O Renderer pode adaptar exposição.

```text
Dark cave
 ↓
eyes adapt

Surface daylight
 ↓
exposure changes
```

Importante: isso deve ser opcional/configurável para acessibilidade.

---

# 59. LIGHT-56 — Eye Adaptation

Separar:

```text
World Lighting
```

de:

```text
Camera Exposure
```

Assim a física do mundo não muda simplesmente porque a câmera mudou.

---

# 60. LIGHT-57 — Color Grading Integration

Lighting fornece dados físicos/semifísicos.

Post-processing decide:

```text
final image appearance
```

---

# 61. LIGHT-58 — Volumetric Lighting

Para certos cenários:

```text
fog
dust
smoke
cloud
```

a luz pode produzir:

```text
god rays
light shafts
volumetric glow
```

Isso pertence à integração Renderer + Lighting.

---

# 62. LIGHT-59 — Light Shafts

Exemplo:

```text
surface opening
 ↓
sunlight enters cave
 ↓
visible atmospheric beam
```

Isso seria excelente visualmente para cavernas gigantes.

---

# 63. LIGHT-60 — Dust / Particles

A iluminação deve poder afetar partículas:

```text
dust
smoke
fog
snow
rain
magic particles
```

---

# 64. LIGHT-61 — Particle Lighting

Partículas podem:

```text
receiveLight
emitLight
```

mas precisam de LOD agressivo.

---

# 65. LIGHT-62 — Vegetation Lighting

Vegetação deve reagir a:

```text
sun
ambient light
shadow
season
```

Sem precisar executar uma física visual cara em cada folha.

---

# 66. LIGHT-63 — Foliage Shadowing

Árvores podem gerar sombras simplificadas.

```text
tree canopy
 ↓
soft shadow approximation
```

Especialmente importante para florestas grandes.

---

# 67. LIGHT-64 — Building Shadowing

Cidades:

```text
building
 ↓
shadow
 ↓
street
```

precisam de técnicas eficientes para milhares de estruturas.

---

# 68. LIGHT-65 — Large-Scale Lighting

O NEXORA precisa suportar:

```text
house
 ↓
city
 ↓
mountain
 ↓
continent
```

Logo:

```text
local lighting
regional lighting
planet-scale lighting
```

não podem depender de uma única técnica.

---

# 69. LIGHT-66 — Hierarchical Lighting

Criar níveis:

```text
Voxel
 ↓
Chunk
 ↓
Region
 ↓
World
```

Cada nível armazena somente o necessário.

---

# 70. LIGHT-67 — Lighting Aggregation

Regiões distantes podem possuir:

```text
ambient intensity
dominant direction
dominant color
```

em vez de milhões de fontes individuais.

---

# 71. LIGHT-68 — Light Proxies

Objetos distantes podem virar:

```text
Light Proxy
```

Exemplo:

```text
cidade
→ aggregated city lighting
```

Em vez de:

```text
5.000 lamp posts
```

---

# 72. LIGHT-69 — Night City

Isso permite um cenário muito interessante:

```text
Day
→ city dominated by sunlight

Night
→ city becomes network of artificial lights
```

E isso pode ter impacto visual sem precisar simular cada lâmpada distante.

---

# 73. LIGHT-70 — Underground Civilization Lighting

Civilizações subterrâneas podem depender de:

```text
torches
lamps
crystals
bioluminescence
machines
magic
```

O sistema econômico/civilizacional determina o uso.

Lighting só representa.

---

# 74. LIGHT-71 — Lighting ↔ Ecology

A luz pode alimentar regras de gameplay:

```text
Crop
→ light requirement

Mob
→ light preference

Plant
→ light dependency
```

Mas:

```text
Ecology
→ consulta Lighting API
```

não o contrário.

---

# 75. LIGHT-72 — Lighting Query API

Outros sistemas podem perguntar:

```text
getLightLevel(position)
getSkyLight(position)
getBlockLight(position)
getDominantLight(position)
isDark(position)
```

Isso é muito útil para IA e agricultura.

---

# 76. LIGHT-73 — Light Exposure

Criar:

```text
LightExposure
```

que represente algo mais rico:

```text
intensity
color
direction
source distance
sky contribution
artificial contribution
```

NPCs/plantas podem consumir essa informação.

---

# 77. LIGHT-74 — Light Zones

Uma área pode possuir uma classificação:

```text
BRIGHT
DIM
DARK
PITCH_BLACK
```

mas isso deve ser derivado do estado real, não armazenado redundante em cada voxel.

---

# 78. LIGHT-75 — Debug

Comandos:

```text
nexora light inspect
nexora light recalculate
nexora light stats
nexora light sources
```

Visualizações:

```text
light levels
sky light
block light
source locations
propagation paths
shadow cascades
light probes
```

---

# 79. LIGHT-76 — Light Field Visualization

Mostrar no mundo:

```text
0 = dark
15 = bright
```

com cores de debug escolhidas pelo modo de desenvolvimento.

---

# 80. LIGHT-77 — Lighting Profiler

Métricas:

```text
light updates
propagation nodes
dirty chunks
dynamic lights
shadow maps
GPU lighting time
CPU lighting time
VRAM
```

---

# 81. LIGHT-78 — Stress Tests

Testar:

```text
1 torch
100 lights
1,000 lights
10,000 logical light sources
huge cave
large city
dense forest
ocean
deep underground
```

O objetivo não é exigir que todas sejam lights dinâmicas completas ao mesmo tempo; o teste deve avaliar as estratégias de LOD e agregação.

---

# 82. LIGHT-79 — Large Cave Test

Um dos testes oficiais:

```text
mega cave
 ↓
many openings
 ↓
water
 ↓
crystals
 ↓
vegetation
 ↓
civilization
 ↓
machines
```

A iluminação precisa continuar coerente.

---

# 83. LIGHT-80 — Far Lands Test

Outro teste:

```text
Surface
 ↓
Far Lands
 ↓
Beyondlands
```

A iluminação não pode quebrar por causa de coordenadas muito grandes.

---

# 84. LIGHT-81 — Precision

Assim como no Renderer, usar coordenadas locais quando necessário.

```text
World Position
 ↓
relative lighting coordinates
```

Para não perder precisão em grandes distâncias.

---

# 85. LIGHT-82 — Serialization

Salvar somente o que realmente precisa ser persistido.

Distinguir:

```text
Persistent Lighting Data
```

de:

```text
Derived Lighting Cache
```

Idealmente, muita coisa de iluminação pode ser recalculada em vez de ocupar save.

---

# 86. LIGHT-83 — Cache Rebuild

Se um cache de iluminação estiver ausente:

```text
World loaded
 ↓
Lighting cache missing
 ↓
rebuild
```

O mundo continua válido.

---

# 87. LIGHT-84 — Save Compatibility

Versões do sistema:

```text
lightingVersion
```

para migração de caches quando necessário.

---

# 88. LIGHT-85 — Mod API

Mods podem registrar:

```text
LightSource
LightEmitterComponent
LightMaterial
AttenuationProfile
LightingProfile
DimensionLightingProfile
```

---

# 89. LIGHT-86 — Custom Light Behavior

Um mod pode definir, por exemplo:

```text
source
→ pulsating intensity
```

ou:

```text
source
→ color changes with state
```

através da API.

---

# 90. LIGHT-87 — Official Content

Conteúdo oficial também usa as mesmas interfaces:

```text
Official Module
       ↓
Lighting API

Community Mod
       ↓
Lighting API
```

Assim não existe uma "iluminação secreta" do conteúdo oficial.

---

# 91. LIGHT-88 — API Boundary

Interfaces públicas:

```text
ILightSource
ILightEmitter
ILightQuery
ILightingWorld
ILightingChunk
ILightProfile
```

---

# 92. LIGHT-89 — Integration com Chunk Engine

```text
Voxel Change
 ↓
Chunk Engine
 ↓
Lighting Invalidation
 ↓
Lighting
 ↓
Chunk Lighting State
 ↓
Renderer
```

---

# 93. LIGHT-90 — Integration com Renderer

```text
Lighting State
 ↓
Render Extraction
 ↓
Light Buffers / Textures
 ↓
Shader
```

---

# 94. LIGHT-91 — Integration com Physics

Physics pode consultar:

```text
visibility
light-dependent sensors
```

mas não deve depender internamente de iluminação.

---

# 95. LIGHT-92 — Integration com AI

NPC/Mob:

```text
AI
 ↓
getLightExposure()
 ↓
decision
```

Exemplo conceitual:

```text
creature prefers dark environment
```

---

# 96. LIGHT-93 — Integration com Farming

```text
Crop
 ↓
Light Requirement
 ↓
Lighting Query
 ↓
Growth Modifier
```

---

# 97. LIGHT-94 — Integration com Civilization

Cidades podem possuir:

```text
street lighting
indoor lighting
industrial lighting
```

e isso pode ser construído, consumido e mantido pela economia.

---

# 98. LIGHT-95 — Integration com Energy

```text
Power Network
 ↓
Machine
 ↓
LightEmitter
```

Se uma rede energética cair:

```text
machine disabled
 ↓
light turns off
```

---

# 99. LIGHT-96 — Integration com Weather

```text
Weather
 ↓
Sky / Atmosphere
 ↓
Lighting
 ↓
Renderer
```

---

# 100. LIGHT-97 — Integration com Seasons

```text
Season
 ↓
Sun / atmosphere
 ↓
Lighting
```

---

# 101. LIGHT-98 — Integration com Space

Em espaço:

```text
Sun
Planet
Star
Ship
```

podem funcionar como fontes diferentes.

O sistema deve suportar:

```text
directional light
point-like sources
environment lighting
```

---

# 102. LIGHT-99 — Final Architecture

No final:

```text
                           WORLD
                             │
                       LIGHTING SYSTEM
                             │
         ┌───────────────────┼──────────────────┐
         │                   │                  │
      SOURCES            PROPAGATION         BLOCKING
         │                   │                  │
      Sun/Moon          Sky Light            Voxels
      Blocks            Block Light          Structures
      Machines          Dynamic             Terrain
      Entities          Regional
         │                   │                  │
         └───────────────────┼──────────────────┘
                             │
                     LIGHT REPRESENTATION
                             │
              ┌──────────────┼──────────────┐
              │              │              │
           WORLD           AI/FARM       RENDERER
              │              │              │
        persistence       queries       lighting
        simulation                       shadows
                                         AO
                                         GI
                                         fog
                                         water
```

---

# 103. Ordem de implementação

Eu faria:

```text
LIGHT-0 Light Data
LIGHT-1 Voxel Light Storage
LIGHT-2 Sky Light
LIGHT-3 Block Light
LIGHT-4 Light Blocking
LIGHT-5 Propagation
LIGHT-6 Removal/Recalculation
LIGHT-7 Chunk Integration
LIGHT-8 Neighbor Propagation
LIGHT-9 Dynamic Lights
LIGHT-10 Sun/Moon
LIGHT-11 Ambient Lighting
LIGHT-12 AO
LIGHT-13 Light Queries
LIGHT-14 Renderer Integration
LIGHT-15 Shadows
LIGHT-16 Water/Fluid Attenuation
LIGHT-17 Lighting LOD
LIGHT-18 Light Probes
LIGHT-19 Regional Aggregation
LIGHT-20 Weather/Season/Dimension
LIGHT-21 AI/Ecology Integration
LIGHT-22 Energy/Machine Integration
LIGHT-23 Space Lighting
LIGHT-24 Mod API
LIGHT-25 Debugging
LIGHT-26 Profiling
LIGHT-27 Stress Testing
```

# 104. Primeiro Vertical Slice

O primeiro teste deve ser:

```text
World
 ↓
Chunk
 ↓
Voxel
 ↓
Sky Light
 ↓
Block Light
 ↓
Torch
 ↓
Propagation
 ↓
Block placed
 ↓
Light recalculated
 ↓
Chunk border
 ↓
Renderer
```

E depois:

```text
Céu
 ↓
dia
 ↓
entardecer
 ↓
noite
 ↓
caverna
 ↓
tocha
 ↓
mega caverna
 ↓
água
 ↓
Deep World
```

### Regra arquitetural

A regra que eu usaria é:

> **Lighting conhece luz; Renderer conhece imagem; World conhece estado; Gameplay conhece significado.**

Então uma tocha não precisa dizer:

```text
"deixe a parede bonita"
```

Ela apenas registra:

```text
LightEmitter
intensity
color
range
```

E o resto acontece:

```text
Torch
 ↓
Lighting
 ↓
Propagation
 ↓
Chunk Light State
 ↓
Renderer
 ↓
Shadow / AO / GI / Material
 ↓
Imagem final
```

Isso deixa a iluminação preparada para **superfície → cavernas → oceanos → 15 camadas profundas → civilizações subterrâneas → dimensões → Far Lands → espaço**, sem fazer o sistema depender de nenhum conteúdo específico.
