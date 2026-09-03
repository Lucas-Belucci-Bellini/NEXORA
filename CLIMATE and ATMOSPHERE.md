Sim. **Clima / Atmosfera** deveria ser um dos sistemas mais sofisticados do NEXORA, porque ele não deve apenas “sortear chuva”. Ele precisa conectar **geografia → atmosfera → clima → biomas → hidrologia → iluminação → agricultura → ecologia → civilização**.

Eu separaria em dois grandes subsistemas:

```text id="d8m4q2"
NEXORA ENVIRONMENT
│
├── CLIMATE ENGINE
│   ├── Temperature
│   ├── Humidity
│   ├── Pressure
│   ├── Wind
│   ├── Precipitation
│   ├── Storms
│   ├── Seasons
│   ├── Climate Zones
│   └── Long-term Climate
│
└── ATMOSPHERE ENGINE
    ├── Air
    ├── Composition
    ├── Density
    ├── Pressure
    ├── Visibility
    ├── Fog
    ├── Clouds
    ├── Aerosols
    ├── Sky
    └── Vertical Layers
```

A arquitetura geral:

```text id="x9jv5r"
                 WORLD GEOGRAPHY
                       │
          ┌────────────┴────────────┐
          │                         │
       TERRAIN                  OCEANS
          │                         │
          └────────────┬────────────┘
                       ↓
                CLIMATE ENGINE
                       ↓
              ATMOSPHERE ENGINE
                       ↓
        ┌──────────────┼──────────────┐
        │              │              │
      WEATHER      HYDROLOGY       BIOMES
        │              │              │
        └──────────────┼──────────────┘
                       ↓
           ECOLOGY / AGRICULTURE
                       ↓
              CIVILIZATION
                       ↓
              WORLD SIMULATION
```

# NEXORA — CLIMATE / ATMOSPHERE MASTER PLAN

## 1. Objetivo

O sistema precisa representar:

```text
temperatura
umidade
pressão
vento
precipitação
nuvens
tempestades
estações
climas regionais
microclimas
atmosfera
visibilidade
névoa
qualidade atmosférica
eventos climáticos
```

E principalmente separar:

**Climate** = comportamento estatístico e físico simplificado da atmosfera ao longo do tempo.

**Weather** = estado momentâneo.

**Atmosphere** = propriedades do ar e representação ambiental.

---

# 2. CAM-0 — Environmental State

Criar um estado ambiental por região:

```text id="t1y0pk"
EnvironmentalState
├── temperature
├── humidity
├── pressure
├── wind
├── precipitation
├── cloudCoverage
├── visibility
├── atmosphericComposition
└── radiation
```

Não guardar necessariamente isso em cada voxel.

Usar campos de escala apropriada.

---

# 3. CAM-1 — Climate Field

O clima precisa ser um **campo espacial**.

```text id="0c6v5g"
ClimateField
├── TemperatureField
├── HumidityField
├── PressureField
├── WindField
└── PrecipitationField
```

Isso permite:

```text
região A → quente
região B → fria
região C → úmida
```

sem cada biome possuir um clima fixo.

---

# 4. CAM-2 — Climate Resolution

Não simular clima por voxel.

Ter escalas:

```text id="7o6q2t"
Voxel
 ↓
Chunk
 ↓
Region
 ↓
Continent
 ↓
Planet
```

Por exemplo:

```text
microclimate → local
weather → regional
climate → continental
season → planetary
```

---

# 5. CAM-3 — Temperature

Criar um modelo de temperatura influenciado por:

```text id="w7f2m4"
latitude
altitude
season
time of day
distance from ocean
terrain
cloud cover
wind
ocean currents
```

Conceitualmente:

```text id="g3qk9n"
Base Temperature
        +
Latitude
        +
Altitude
        +
Season
        +
Ocean Influence
        +
Weather
        =
Local Temperature
```

---

# 6. CAM-4 — Vertical Temperature

A temperatura também varia com altitude.

```text id="7x5s6z"
Altitude
 ↓
Temperature Modifier
```

Isso permite:

```text
vale quente
montanha fria
pico nevado
```

sem criar biomas artificiais.

---

# 7. CAM-5 — Depth Temperature

No Deep World:

```text id="m1c5q8"
Depth
 ↓
Geothermal Influence
 ↓
Temperature
```

Assim as camadas profundas podem ser:

```text
frias
temperadas
quentes
extremamente quentes
```

dependendo da região geológica.

---

# 8. CAM-6 — Geothermal Field

Criar:

```text id="6v2k8c"
GeothermalField
```

proveniente de:

```text
magma
tectonics
volcanic activity
depth
```

Isso integra:

```text
Geology
→ Climate/Atmosphere
→ Deep World
```

---

# 9. CAM-7 — Humidity

Representar:

```text id="2s8g3f"
absolute humidity
relative humidity
```

A umidade depende de:

```text id="5v1x09"
water bodies
temperature
evaporation
rain
vegetation
wind
```

---

# 10. CAM-8 — Evaporation

Oceano/lago/rio:

```text id="9y7g0m"
water
 ↓
temperature + radiation
 ↓
evaporation
 ↓
humidity
```

Isso alimenta o Weather System.

---

# 11. CAM-9 — Condensation

Quando condições adequadas:

```text id="t7z2f1"
humid air
 ↓
cooling
 ↓
condensation
 ↓
cloud
```

---

# 12. CAM-10 — Atmospheric Pressure

Criar:

```text id="1t6q3g"
PressureField
```

influenciado por:

```text id="w2x4q8"
altitude
temperature
air mass
weather systems
```

---

# 13. CAM-11 — Pressure Systems

Criar:

```text id="6b4y9n"
High Pressure
Low Pressure
```

Esses sistemas movimentam o clima.

```text id="h0q5pc"
Pressure Gradient
 ↓
Wind
```

---

# 14. CAM-12 — Wind Field

O vento deve ser um campo vetorial:

```text id="8p3x5m"
WindVector
├── x
├── y
└── z
```

Normalmente o componente vertical pode ser menor, mas não precisa ser sempre zero.

---

# 15. CAM-13 — Wind Generation

O vento depende de:

```text id="f9a2m6"
pressure gradients
temperature differences
terrain
planetary rotation model
weather systems
```

Não precisa ser meteorologicamente perfeito para ser convincente.

---

# 16. CAM-14 — Terrain Influence

Montanhas alteram o vento:

```text id="0h6m8k"
Mountain
 ↓
Wind deflection
```

Isso pode criar:

```text
windward side
leeward side
```

com efeitos climáticos.

---

# 17. CAM-15 — Orographic Rain

Montanha:

```text id="4b5j2r"
humid air
 ↓
mountain
 ↓
rises
 ↓
cools
 ↓
precipitation
```

E cria regiões:

```text
úmidas
secas
```

Naturalmente.

---

# 18. CAM-16 — Rain Shadow

No lado oposto:

```text id="r1k8x6"
mountain
 ↓
dry air
 ↓
arid region
```

Isso ajuda a gerar desertos naturalmente.

---

# 19. CAM-17 — Ocean Influence

Distância do oceano:

```text id="8w4t0n"
Ocean
 ↓
temperature buffering
 ↓
humidity
 ↓
regional climate
```

Regiões costeiras ficam diferentes do interior.

---

# 20. CAM-18 — Ocean Currents

O sistema deve consumir o estado de:

```text id="qs7r3m"
Ocean Current System
```

para influenciar:

```text
temperature
humidity
storms
coastal climate
```

---

# 21. CAM-19 — Seasons

Criar:

```text id="5a3x6c"
SeasonSystem
├── season
├── progress
├── length
└── astronomical state
```

Suporte:

```text
spring
summer
autumn
winter
```

mas dimensões podem possuir outras divisões.

---

# 22. CAM-20 — Planetary Seasons

As estações podem depender de:

```text id="b3n7w8"
axial tilt
orbital position
latitude
```

ou de um modelo simplificado.

---

# 23. CAM-21 — Day Length

A duração do dia pode variar por latitude e estação.

Isso influencia:

```text id="a1v5z9"
temperature
photosynthesis
mob activity
civilization
agriculture
```

---

# 24. CAM-22 — Climate Zones

Criar classificações derivadas:

```text id="j4m8qt"
tropical
arid
temperate
continental
polar
alpine
oceanic
monsoonal
```

Mas o Biome Engine continua sendo o sistema que transforma isso em biomas concretos.

---

# 25. CAM-23 — Climate ≠ Biome

Muito importante:

```text id="3m9x5d"
Climate
   ↓
Biome Generator
```

Um mesmo clima pode suportar biomas diferentes dependendo de:

```text
soil
geology
water
elevation
ecology
```

---

# 26. CAM-24 — Weather State

Weather representa o estado imediato:

```text id="k6s2p0"
Clear
Cloudy
Rain
Storm
Snow
Fog
Hail
Wind
Heatwave
ColdSnap
```

---

# 27. CAM-25 — Weather Fronts

Criar:

```text id="0x7z2c"
WarmFront
ColdFront
OcclusionFront
```

de maneira simplificada.

Isso gera transições meteorológicas mais naturais.

---

# 28. CAM-26 — Weather Cells

Um sistema meteorológico pode possuir:

```text id="h9q1b7"
WeatherCell
├── center
├── radius
├── pressure
├── movement
├── intensity
└── type
```

E se mover pelo mapa.

---

# 29. CAM-27 — Storm Formation

Tempestades podem nascer quando condições adequadas existem:

```text id="m4z6t1"
humidity
+
instability
+
temperature gradient
+
pressure
```

---

# 30. CAM-28 — Storm Lifecycle

```text id="3u7q5n"
FORMING
 ↓
GROWING
 ↓
MATURE
 ↓
WEAKENING
 ↓
DISSIPATING
```

---

# 31. CAM-29 — Rain

Rain possui:

```text id="7q8m2x"
intensity
duration
drop size
coverage
```

O Weather System injeta água no Fluid Engine.

```text id="r6y3c1"
Rain
 ↓
Fluid Engine
 ↓
surface water
rivers
soil
```

---

# 32. CAM-30 — Snow

Snow depende de:

```text id="2w8v6k"
temperature
humidity
cloud conditions
altitude
```

E pode acumular:

```text id="k7p9m2"
SnowAccumulation
```

---

# 33. CAM-31 — Hail

Pode ser uma forma especial de precipitação.

```text id="8f3r0s"
Precipitation
→ hail
```

Com efeitos visuais e ambientais.

---

# 34. CAM-32 — Fog

Fog resulta de:

```text id="q3n6t8"
humidity
temperature
ground conditions
terrain
cloud state
```

Tipos:

```text id="8k4h1s"
ground fog
valley fog
marine fog
dense fog
```

---

# 35. CAM-33 — Visibility

Criar:

```text id="72h4mf"
VisibilityField
```

que pode depender de:

```text
fog
rain
snow
dust
smoke
aerosols
distance
```

---

# 36. CAM-34 — Atmospheric Composition

Criar:

```text id="u4b7c9"
AtmosphereComposition
├── oxygen-like
├── inert gases
├── greenhouse components
├── water vapor
└── particulates
```

Não precisa representar milhares de moléculas.

É um modelo abstrato.

---

# 37. CAM-35 — Atmospheric Density

A densidade depende de:

```text id="z8q4r2"
pressure
temperature
composition
altitude
```

Isso pode alimentar Física.

---

# 38. CAM-36 — Physics Integration

Aircraft/particles/etc. podem consultar:

```text id="5x2n9j"
AirDensity
Wind
Temperature
Pressure
```

A Física decide os efeitos.

---

# 39. CAM-37 — Aerodynamics

O Weather Engine não implementa aerodinâmica.

Ele fornece:

```text id="w3p8q7"
AtmosphericState
```

E o sistema de aviação calcula:

```text
lift
drag
control
```

---

# 40. CAM-38 — Sound Integration

Atmosfera pode afetar:

```text id="4h9t2e"
sound propagation
wind noise
storm sound
underwater acoustics
```

O Audio System consome os dados.

---

# 41. CAM-39 — Sky System

Criar:

```text id="7c1y5m"
SkyState
```

com:

```text
sun
moon
stars
clouds
horizon
atmospheric scattering
```

---

# 42. CAM-40 — Sky Color

O céu varia conforme:

```text id="m7x2p4"
time
humidity
aerosols
weather
altitude
atmosphere
```

---

# 43. CAM-41 — Cloud System

Separar:

```text id="0s8k2d"
Cloud Simulation
```

de:

```text id="h9q4v6"
Cloud Rendering
```

Simulation:

```text
formation
movement
density
precipitation
```

Renderer:

```text
shape
lighting
shadow
volume
```

---

# 44. CAM-42 — Cloud Layers

Ter diferentes altitudes:

```text id="p6n3s1"
low
medium
high
storm
```

---

# 45. CAM-43 — Cloud Coverage

O mundo pode possuir:

```text id="k5g8t0"
cloudCoverage
cloudDensity
cloudType
```

---

# 46. CAM-44 — Cloud Movement

```text id="m1c8q7"
Wind Field
 ↓
Cloud Advection
```

As nuvens se movem com os padrões atmosféricos.

---

# 47. CAM-45 — Cloud Shadows

O Renderer pode transformar:

```text id="g6j2b4"
cloud state
 ↓
shadow pattern
```

em sombras sobre o terreno.

---

# 48. CAM-46 — Storm Lighting

Tempestades podem alterar:

```text id="v7x3m2"
ambient light
sky
visibility
```

E o Renderer cria os efeitos visuais correspondentes.

---

# 49. CAM-47 — Lightning System

Separar:

```text id="4p8z1y"
Lightning Event
```

da simples animação.

O Climate System cria o evento.

Renderer/Audio representam.

---

# 50. CAM-48 — Lightning Integration

```text id="x5q9m3"
Storm
 ↓
Lightning Event
 ├── visual flash
 ├── sound
 └── temporary light
```

Isso integra diretamente com o Lighting System.

---

# 51. CAM-49 — Heatwaves

Criar evento climático:

```text id="7n2k5b"
Heatwave
```

com:

```text
duration
region
severity
```

Impactos podem ser tratados por:

```text
agriculture
ecology
civilization
health
```

---

# 52. CAM-50 — Cold Waves

Mesmo sistema:

```text id="2m7q6x"
ColdSnap
```

---

# 53. CAM-51 — Drought

Pode ser calculado por períodos longos:

```text id="p4c8w2"
precipitation deficit
+
soil moisture
+
water availability
```

e exposto ao resto do mundo.

---

# 54. CAM-52 — Flood Conditions

Climate não cria diretamente a enchente.

Ele fornece:

```text id="5y1n8s"
precipitation
```

e Fluid/Hydrology podem produzir:

```text
runoff
river overflow
flood
```

---

# 55. CAM-53 — Drought ↔ Civilization

Uma seca pode provocar:

```text id="k8m3q1"
crop failure
 ↓
food shortage
 ↓
price increase
 ↓
migration
 ↓
political pressure
```

Isso encaixa perfeitamente no sistema emergente de civilização.

---

# 56. CAM-54 — Monsoon

Criar padrões sazonais de precipitação:

```text id="h7v2c9"
wet season
dry season
```

que podem ser regionais.

---

# 57. CAM-55 — Microclimate

Além do clima regional:

```text id="3m8q7a"
MicroclimateField
```

Pode ser influenciado por:

```text
forest
lake
city
mountain
cave
valley
```

---

# 58. CAM-56 — Forest Microclimate

Uma floresta pode alterar:

```text id="p9x2j6"
humidity
temperature
wind
light
```

A Ecologia fornece cobertura vegetal; Climate calcula os efeitos ambientais agregados.

---

# 59. CAM-57 — City Microclimate

Grandes cidades podem possuir:

```text id="x4n7m1"
urban heat
wind corridors
reduced humidity
pollution
```

Isso pode criar um microclima próprio.

---

# 60. CAM-58 — Cave Atmosphere

No Deep World:

```text id="r6q8s3"
Cave
 ↓
Air Volume
 ↓
temperature
humidity
pressure
composition
```

Isso permite cavernas muito diferentes.

---

# 61. CAM-59 — Cave Airflow

Grandes cavernas podem possuir:

```text id="z1m5q8"
air circulation
```

baseada em:

```text
cave entrances
temperature gradients
pressure
```

---

# 62. CAM-60 — Underground Atmosphere

Algumas regiões profundas podem ter:

```text id="5v9x2k"
low oxygen
high CO₂-like gas
hot air
humid air
toxic atmosphere
```

Mas isso deve ser um perfil ambiental, não um hardcode.

---

# 63. CAM-61 — Atmospheric Hazards

O sistema pode expor:

```text id="g2j7p4"
AtmosphericHazard
```

como:

```text
low oxygen
toxic composition
extreme pressure
extreme temperature
```

Outro sistema decide o impacto.

---

# 64. CAM-62 — Player Integration

O Player System pode consultar:

```text id="m8c1y5"
temperature
pressure
oxygen-like availability
visibility
wind
```

e aplicar regras apropriadas.

---

# 65. CAM-63 — Agriculture

Plantas podem consultar:

```text id="q7n3x9"
temperature
humidity
rainfall
season
day length
```

para crescimento.

---

# 66. CAM-64 — Ecology

Ecologia pode consumir:

```text id="k1z6p8"
climate
weather
seasons
water
```

para:

```text
migration
reproduction
population
vegetation
```

---

# 67. CAM-65 — Mob Migration

Migração:

```text id="y5m8q3"
Climate change
 ↓
habitat suitability
 ↓
mob migration
```

---

# 68. CAM-66 — Civilizations

Civilizações também podem ser afetadas:

```text id="q2x7n1"
climate
 ↓
food
water
resources
 ↓
population
economy
migration
```

---

# 69. CAM-67 — Long-Term Climate

Não limitar o sistema ao tempo real.

Criar:

```text id="s4m9y2"
ClimateHistory
```

com:

```text
seasonal averages
rainfall history
drought history
temperature history
storm history
```

---

# 70. CAM-68 — Climate Change

O mundo pode modificar seu clima ao longo do tempo.

Fatores:

```text id="h8p2v5"
vegetation changes
oceans
volcanism
atmospheric composition
civilization activity
```

Mas usar escalas longas e modelos simplificados.

---

# 71. CAM-69 — Civilization Influence

Civilização pode produzir:

```text id="5n7k4x"
industrial emissions
land-use changes
deforestation
irrigation
urbanization
```

que alimentam um modelo ambiental.

---

# 72. CAM-70 — Ecology Feedback

A vegetação influencia:

```text id="7m2q9v"
humidity
soil moisture
evapotranspiration
```

e o Climate Engine recebe esses agregados.

---

# 73. CAM-71 — Hydrology Feedback

A Hydrology Engine informa:

```text id="1x6c8m"
surface water
soil moisture
groundwater
ocean state
```

Climate usa isso para estimar evaporação/umidade.

---

# 74. CAM-72 — Ocean ↔ Atmosphere

Criar interface:

```text id="9q3w5b"
OceanAtmosphereCoupling
```

que troca:

```text
temperature
humidity
wind
evaporation
currents
```

---

# 75. CAM-73 — Weather Forecast

O mundo pode ter previsão meteorológica.

```text id="4k8m2p"
Current State
 ↓
Forecast Model
 ↓
Predicted Weather
```

NPCs e civilizações podem utilizar isso.

---

# 76. CAM-74 — NPC Knowledge

NPC não deve receber diretamente:

```text
"vai chover em exatamente 14 minutos"
```

A Knowledge System pode representar previsões imperfeitas:

```text id="7p1y5n"
likely rain
storm approaching
uncertain forecast
```

---

# 77. CAM-75 — Civilization Forecasting

Uma cidade pode usar previsões para:

```text id="n8m3x6"
planting
harvest
travel
trade
construction
```

Isso conecta clima à economia.

---

# 78. CAM-76 — Weather Knowledge

Conhecimento climático pode evoluir:

```text id="2v7q9m"
observation
 ↓
pattern
 ↓
hypothesis
 ↓
forecast
```

Isso combina com o sistema de Knowledge AI do NEXORA.

---

# 79. CAM-77 — Weather Events

Eventos:

```text id="g6x4n2"
WeatherStarted
WeatherChanged
WeatherEnded
RainStarted
RainEnded
StormFormed
StormDissipated
TemperatureChanged
PressureChanged
WindChanged
```

---

# 80. CAM-78 — Scheduler

Climate não precisa ser calculado a cada frame.

Escalas:

```text id="4q9m7x"
frame
→ rendering

second
→ weather visuals

minute
→ local weather

hour
→ weather field

day
→ climate adjustments

season
→ climate cycle

year
→ long-term climate
```

---

# 81. CAM-79 — LOD

Assim como os outros sistemas:

```text id="y2m8q5"
FULL
→ local weather

REGIONAL
→ aggregated weather

ABSTRACT
→ statistical climate
```

---

# 82. CAM-80 — Regional Simulation

Uma região distante pode guardar:

```text id="6p4x9c"
temperature average
humidity
precipitation
storm state
```

em vez de simular cada nuvem.

---

# 83. CAM-81 — Weather Streaming

Quando o jogador se aproxima:

```text id="8n3m7q"
Regional weather
 ↓
load detailed state
 ↓
local simulation
```

Isso combina com Chunk Streaming.

---

# 84. CAM-82 — Atmospheric Regions

Algumas regiões podem possuir:

```text id="q5x1m8"
custom atmosphere
```

por exemplo:

```text
mountain
ocean
underground
alien
dimension
```

---

# 85. CAM-83 — Dimensions

Cada dimensão pode registrar:

```text id="7m9q2k"
AtmosphereProfile
ClimateProfile
WeatherProfile
```

---

# 86. CAM-84 — Dimension Weather

Uma dimensão pode ter:

```text id="2p6x8c"
no rain
acid rain-like
constant storm
floating clouds
strange winds
```

O sistema trata como regras configuráveis.

---

# 87. CAM-85 — Space

Espaço é um caso especial:

```text id="m7x3q1"
Atmosphere = absent/thin/custom
```

O sistema ainda fornece:

```text id="4n8p6y"
radiation
temperature
solar exposure
```

conforme o modelo escolhido.

---

# 88. CAM-86 — Planet Atmospheres

Cada planeta pode possuir:

```text id="p9q2x7"
AtmosphereProfile
├── composition
├── pressure
├── density
├── temperature
├── clouds
└── weather
```

---

# 89. CAM-87 — Atmospheric Entry

Para naves:

```text id="6m1z8k"
space
 ↓
atmosphere
 ↓
density increases
 ↓
heating / drag
```

A Física usa os dados atmosféricos.

---

# 90. CAM-88 — Volcanic Atmosphere

Vulcões podem injetar:

```text id="5x8q2m"
heat
ash
aerosols
gas
```

Isso pode gerar mudanças regionais.

---

# 91. CAM-89 — Aerosols

Criar:

```text id="7q4m9x"
AerosolField
```

para:

```text
dust
ash
pollution
smoke
```

Isso pode afetar:

```text
visibility
sky color
sunlight
weather
```

---

# 92. CAM-90 — Dust Storms

Desertos podem produzir:

```text id="3m8x6p"
DustStorm
```

dependendo de:

```text
wind
dryness
soil
vegetation cover
```

---

# 93. CAM-91 — Pollution

Cidades/indústrias:

```text id="q1n7x5"
pollution source
 ↓
Atmosphere
 ↓
aerosols
 ↓
visibility / sky / weather
```

O efeito exato deve ser configurável.

---

# 94. CAM-92 — Smoke

Incêndios florestais podem produzir:

```text id="4x9m2q"
smoke field
```

e isso alimenta o sistema atmosférico.

---

# 95. CAM-93 — Fire Integration

```text id="r7p3k8"
Vegetation/Fire
 ↓
heat
smoke
aerosols
 ↓
Atmosphere
```

Isso pode gerar eventos emergentes.

---

# 96. CAM-94 — Weather ↔ Fire

Condições:

```text id="5q8m1x"
dry
+
hot
+
wind
```

podem aumentar risco de incêndio.

A lógica de fogo fica em outro sistema.

---

# 97. CAM-95 — Climate ↔ Civilization

O clima pode influenciar:

```text id="2x7n4m"
food
water
migration
trade
settlement
infrastructure
politics
```

---

# 98. CAM-96 — Climate ↔ Infrastructure

Cidades precisam responder a:

```text id="8m5q2x"
flooding
snow
heat
storms
wind
```

Por exemplo:

```text
drenagem
reservatórios
estradas
pontes
```

---

# 99. CAM-97 — Weather Damage Hook

Não implementar dano diretamente.

Criar:

```text id="6q9x3m"
EnvironmentalEvent
```

que outros sistemas consomem.

---

# 100. CAM-98 — Save / Persistence

Salvar:

```text id="5m2x8q"
world climate state
weather cells
season
long-term anomalies
```

Não precisa salvar cada campo derivado se pode ser reconstruído.

---

# 101. CAM-99 — Determinism

O sistema deve ser reproduzível:

```text id="9x4m7q"
World Seed
+
World Time
+
Climate Version
+
Environmental State
=
same result
```

Para o multiplayer, o servidor pode ser autoridade.

---

# 102. CAM-100 — Debug Tools

Comandos:

```text id="q8m3x1"
nexora weather
nexora climate
nexora atmosphere
nexora wind
nexora pressure
nexora precipitation
```

Visualizações:

```text id="3m7x9q"
temperature field
humidity field
pressure field
wind vectors
weather cells
cloud coverage
visibility
aerosols
```

---

# 103. CAM-101 — Profiler

Métricas:

```text id="5x1m8q"
climate update time
weather update time
field resolution
active weather cells
cloud simulation cost
atmospheric queries
```

---

# 104. CAM-102 — Testing

Testes:

```text id="7q3m9x"
day/night
seasons
latitude
altitude
mountains
ocean
rain
snow
storm
fog
wind
pressure
```

---

# 105. CAM-103 — Extreme Tests

```text id="2m8x5q"
mega storm
continent-sized weather cell
deep cave
high mountain
ocean
desert
polar region
dense forest
large city
volcanic area
```

---

# 106. CAM-104 — Mod API

Mods podem registrar:

```text id="9x6m2q"
ClimateProfile
AtmosphereProfile
WeatherType
WeatherEvent
PrecipitationType
AtmosphericProperty
MicroclimateModifier
```

---

# 107. CAM-105 — Official Content

Conteúdo oficial usa exatamente as mesmas APIs:

```text id="4m7q1x"
Official Module
      ↓
Climate API

Community Mod
      ↓
Climate API
```

---

# 108. CAM-106 — Final Architecture

```text id="m8q3x6"
                         WORLD
                           │
                ┌──────────┴──────────┐
                │                     │
             TERRAIN                OCEAN
                │                     │
                └──────────┬──────────┘
                           ↓
                     CLIMATE ENGINE
                           │
          ┌────────────────┼────────────────┐
          │                │                │
      TEMPERATURE       PRESSURE          HUMIDITY
          │                │                │
          └────────────────┼────────────────┘
                           ↓
                      WIND FIELD
                           ↓
                  WEATHER SIMULATION
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
    PRECIPITATION        CLOUDS           STORMS
        │                  │                  │
        ↓                  ↓                  ↓
      FLUID             RENDERER           LIGHTING
        │
    HYDROLOGY
        │
        └──────────────┐
                       ↓
                    BIOMES
                       ↓
                  ECOLOGY
                       ↓
                  AGRICULTURE
                       ↓
                 CIVILIZATION
```

---

# 109. Ordem de implementação

Eu faria:

```text id="p5x8m2"
CLIMATE-0 Environmental State
CLIMATE-1 Climate Fields
CLIMATE-2 Temperature
CLIMATE-3 Humidity
CLIMATE-4 Pressure
CLIMATE-5 Wind
CLIMATE-6 Seasons
CLIMATE-7 Weather State
CLIMATE-8 Clouds
CLIMATE-9 Precipitation
CLIMATE-10 Rain
CLIMATE-11 Snow
CLIMATE-12 Fog
CLIMATE-13 Storms
CLIMATE-14 Weather Cells
CLIMATE-15 Ocean Coupling
CLIMATE-16 Hydrology Coupling
CLIMATE-17 Atmosphere
CLIMATE-18 Atmospheric Density
CLIMATE-19 Visibility
CLIMATE-20 Aerosols
CLIMATE-21 Microclimate
CLIMATE-22 Cave Atmosphere
CLIMATE-23 Deep World
CLIMATE-24 Dimension Profiles
CLIMATE-25 Space/Planet Atmosphere
CLIMATE-26 Agriculture/Ecology integration
CLIMATE-27 Civilization integration
CLIMATE-28 Forecasting
CLIMATE-29 Long-term Climate
CLIMATE-30 Climate Change
CLIMATE-31 LOD
CLIMATE-32 Persistence
CLIMATE-33 Multiplayer
CLIMATE-34 Debugging
CLIMATE-35 Mod API
CLIMATE-36 Stress Testing
```

# 110. Primeiro Vertical Slice

O primeiro slice não precisa ter clima realista. Precisa provar a cadeia inteira:

```text id="h7m3x9"
World Time
     ↓
Season
     ↓
Temperature Field
     ↓
Humidity
     ↓
Cloud Formation
     ↓
Rain
     ↓
Fluid Engine
     ↓
River / Soil
     ↓
Biome / Agriculture
```

Depois:

```text id="q2x8m5"
Pressure
 ↓
Wind
 ↓
Weather Cell
 ↓
Storm
 ├── Rain
 ├── Clouds
 ├── Lightning Event
 ├── Lighting
 ├── Audio
 └── Visibility
```

E o teste grande:

```text id="m9x4q7"
Ocean
 ↓
Evaporation
 ↓
Atmosphere
 ↓
Clouds
 ↓
Mountain
 ↓
Rain
 ↓
River
 ↓
Ocean
```

Isso começa a criar um **ciclo ambiental real** em vez de uma coleção de efeitos.

## A regra arquitetural

Eu colocaria:

> **Climate cria as condições. Weather descreve o estado momentâneo. Atmosphere descreve o meio. Hydrology transporta água. Lighting mostra a luz. Renderer mostra o ambiente. Ecologia e Civilização reagem.**

Assim o NEXORA pode chegar a algo muito mais interessante:

```text id="a5x8m2"
oceano aquece
 ↓
evaporação
 ↓
umidade
 ↓
pressão
 ↓
vento
 ↓
nuvem
 ↓
tempestade
 ↓
chuva
 ↓
rios
 ↓
agricultura
 ↓
produção de alimentos
 ↓
economia
 ↓
migração
 ↓
política
 ↓
história do mundo
```

E isso combina diretamente com a filosofia que já definimos para o NEXORA: **o mundo não fica parado esperando o jogador; os sistemas continuam interagindo entre si.**
