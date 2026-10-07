Perfeito. O **Audio System** deve seguir a mesma arquitetura dos anteriores: ele não deve "pertencer" ao Player, Entity ou Renderer. Ele será a infraestrutura responsável por transformar eventos, estados e ambientes em **som espacial, música, voz, efeitos e paisagens sonoras**.

A regra central:

> **Audio System determina quais sons existem, como são selecionados, espacializados, misturados e reproduzidos. Os sistemas de gameplay fornecem os eventos e estados; o Audio System produz a experiência sonora.**

# NEXORA — AUDIO SYSTEM / AUDIO API

## 1. Arquitetura central

```text id="aud-01"
                         NEXORA
                           │
                    AUDIO SYSTEM
                           │
       ┌───────────────────┼────────────────────┐
       ↓                   ↓                    ↓
    CONTENT             RUNTIME              OUTPUT
       │                   │                    │
   Audio Assets       Audio Events         Audio Device
   Sound Profiles     Audio Sources        Audio Mixer
   Music             Emitters             Spatialization
   Voice             Ambience             DSP
```

Fluxo principal:

```text id="aud-02"
WORLD / ENTITY / ITEM / BLOCK / CLIMATE / COMBAT
                         │
                         ↓
                    EVENT BUS
                         │
                         ↓
                    AUDIO SYSTEM
                         │
        ┌────────────────┼────────────────┐
        ↓                ↓                ↓
     SELECT            MIX             SPATIALIZE
        └────────────────┼────────────────┘
                         ↓
                    AUDIO OUTPUT
```

---

# 2. O que o Audio System controla?

```text id="aud-03"
Sound Effects
Ambient Sounds
Music
Voice
UI Audio
Footsteps
Weapons
Machines
Vehicles
Weather
Nature
Fluids
Caves
Cities
Space
Events
Dynamic Music
Environmental Audio
Dialogue
Radio
Communication
```

E futuramente:

```text id="aud-04"
Acoustic Simulation
Occlusion
Portals
Underground Acoustics
Planetary Atmospheres
Vacuum Audio Rules
Dimensional Audio
```

---

# 3. Audio não é apenas "tocar MP3"

O sistema precisa separar:

```text id="aud-05"
Audio Asset
Sound Definition
Audio Event
Audio Source
Audio Emitter
Audio Bus
Audio Mixer
Audio Listener
Spatialization
Acoustics
```

---

# 4. Audio Asset

É o recurso bruto.

Pode ser:

```text id="aud-06"
wav
ogg
flac
compressed runtime format
```

O jogo deve trabalhar com uma abstração:

```text id="aud-07"
AudioAssetID
```

---

# 5. Audio Definition

Define como o som deve funcionar.

```text id="aud-08"
AudioDefinition

id
asset
volume
pitch
loop
spatial
priority
bus
attenuation
randomization
variation
```

---

# 6. Audio ID

Usar Registry:

```text id="aud-09"
nexora:stone_break
nexora:forest_ambience
examplemod:laser_fire
```

---

# 7. Audio Registry

Criar:

```text id="aud-10"
AudioRegistry
```

e:

```text id="aud-11"
AudioBusRegistry
AudioProfileRegistry
MusicRegistry
VoiceRegistry
```

Isso encaixa diretamente no Registry System.

---

# 8. Sound Effect

Um efeito simples:

```text id="aud-12"
stone_break
door_open
machine_start
footstep
weapon_impact
```

---

# 9. Audio Event

Um evento semântico:

```text id="aud-13"
BlockBroken
EntityFootstep
WeaponFired
MachineStarted
StormStarted
```

O Audio System pode escutar o Event Bus e selecionar o som apropriado.

---

# 10. Audio Event ≠ Audio Asset

Exemplo:

```text id="aud-14"
BlockBroken
   ↓
material = stone
   ↓
nexora:stone_break_01
```

ou:

```text id="aud-15"
material = metal
   ↓
nexora:metal_break_03
```

---

# 11. Audio Event Definition

Criar:

```text id="aud-16"
AudioEventDefinition
```

com:

```text id="aud-17"
conditions
variations
routing
priority
spatialization
randomization
```

---

# 12. Random Variation

Não queremos:

```text id="aud-18"
10,000 footsteps
→ exatamente mesmo som
```

Uma definição pode possuir:

```text id="aud-19"
variants:
    step_01
    step_02
    step_03
    step_04
```

---

# 13. Weighted Variants

Cada variante pode possuir peso:

```text id="aud-20"
step_01 = 40%
step_02 = 30%
step_03 = 20%
step_04 = 10%
```

---

# 14. Deterministic Random

Para situações onde é útil:

```text id="aud-21"
EntityID
+
event sequence
```

pode gerar uma escolha determinística.

---

# 15. Audio Source

Representa uma reprodução.

```text id="aud-22"
AudioSource

clip
position
volume
pitch
loop
state
bus
```

---

# 16. Audio Emitter

Representa algo que produz sons no mundo.

```text id="aud-23"
AudioEmitter

owner
position
profile
sources
```

Exemplo:

```text id="aud-24"
Machine
 ↓
AudioEmitter
 ↓
motor_loop
```

---

# 17. Entity Audio

Entity pode possuir:

```text id="aud-25"
AudioComponent
```

mas a Entity System não executa áudio.

Ela apenas referencia o emitter.

---

# 18. Player Audio

Player pode produzir:

```text id="aud-26"
footsteps
jump
land
hurt
equipment
voice
```

Audio System decide como reproduzir.

---

# 19. Mob Audio

Mob pode ter:

```text id="aud-27"
idle
alert
attack
damage
death
movement
environmental
```

---

# 20. NPC Audio

NPC pode produzir:

```text id="aud-28"
speech
work
movement
social
emotion
```

---

# 21. Block Audio

Blocks podem possuir:

```text id="aud-29"
BlockSoundProfile
```

Exemplo:

```text id="aud-30"
stone
wood
metal
glass
ice
```

com:

```text id="aud-31"
break
place
hit
step
slide
```

---

# 22. Item Audio

Items:

```text id="aud-32"
pickup
drop
equip
use
reload
charge
repair
```

---

# 23. Tool Audio

Tool API pode informar:

```text id="aud-33"
tool operation
```

Audio reproduz o perfil.

---

# 24. Weapon Audio

Combat informa:

```text id="aud-34"
weapon fired
impact
reload
empty
mechanism
```

Audio escolhe os assets.

---

# 25. Machine Audio

Machine System informa:

```text id="aud-35"
started
stopped
processing
overheated
failure
```

Audio pode criar:

```text id="aud-36"
motor hum
fan
alarm
mechanical movement
```

---

# 26. Vehicle Audio

Vehicle pode possuir:

```text id="aud-37"
engine
transmission
wheel
track
brake
suspension
horn
```

---

# 27. Fluid Audio

Fluid Engine pode publicar:

```text id="aud-38"
flow
pressure
splash
mix
steam
```

Audio transforma isso em som.

---

# 28. Climate Audio

Climate fornece:

```text id="aud-39"
rain
wind
thunder
hail
snow
storm
```

---

# 29. Environment Audio

O ambiente pode possuir:

```text id="aud-40"
EnvironmentalAudioProfile
```

com:

```text id="aud-41"
forest
desert
ocean
cave
city
underground
alien
volcanic
```

---

# 30. Biome Audio

Biome Definition pode referenciar:

```text id="aud-42"
ambient profile
day profile
night profile
weather profile
```

Audio System combina esses dados.

---

# 31. Layered Ambience

Em vez de um único:

```text id="aud-43"
forest.wav
```

usar camadas:

```text id="aud-44"
wind
birds
insects
leaves
distant animals
water
```

---

# 32. Ambient Zones

Mundo pode possuir:

```text id="aud-45"
AudioZone
```

Exemplos:

```text id="aud-46"
forest
city
cave
mine
ocean
interior
```

---

# 33. Zone Blending

Ao entrar numa cidade:

```text id="aud-47"
Forest
   ↓
crossfade
   ↓
City
```

---

# 34. Interior Audio

Dentro de uma construção:

```text id="aud-48"
exterior ambience
 ↓
occlusion
 ↓
interior ambience
```

---

# 35. Cave Audio

Cavernas podem ter:

```text id="aud-49"
water drops
wind
echo
distant sounds
rockfall
underground fauna
```

---

# 36. Underground Acoustics

Como o NEXORA possui um mundo subterrâneo gigantesco:

```text id="aud-50"
surface
 ↓
cave
 ↓
deep world
 ↓
underground city
```

cada camada pode possuir perfil acústico diferente.

---

# 37. Audio Propagation

O som pode possuir alcance:

```text id="aud-51"
source
 ↓
distance
 ↓
attenuation
```

---

# 38. Attenuation

Modelos:

```text id="aud-52"
linear
inverse
custom curve
```

---

# 39. Distance Model

Uma fonte pode definir:

```text id="aud-53"
minDistance
maxDistance
falloffCurve
```

---

# 40. Spatial Audio

Suportar:

```text id="aud-54"
mono
stereo
3D
ambisonic
```

dependendo do asset e da plataforma.

---

# 41. Audio Listener

Normalmente associado à câmera/jogador:

```text id="aud-55"
AudioListener
```

com:

```text id="aud-56"
position
orientation
velocity
```

---

# 42. Player Camera

O Camera System atualiza:

```text id="aud-57"
listener transform
```

Audio usa isso.

---

# 43. Velocity

Listener e source podem possuir:

```text id="aud-58"
velocity
```

para efeitos Doppler.

---

# 44. Doppler

Pode suportar:

```text id="aud-59"
DopplerProfile
```

para:

```text id="aud-60"
vehicles
projectiles
fast aircraft
spacecraft
```

---

# 45. Occlusion

Um som atrás de uma parede não deve ser igual ao som em espaço aberto.

```text id="aud-61"
Source
 ↓
ray / acoustic query
 ↓
obstacle
 ↓
filter
```

---

# 46. Acoustic Materials

Block/Material podem fornecer:

```text id="aud-62"
acousticProfile
```

como:

```text id="aud-63"
stone
wood
metal
glass
soil
ice
water
```

---

# 47. Acoustic Profile

Pode definir:

```text id="aud-64"
transmission
absorption
reflection
lowFrequencyLoss
highFrequencyLoss
```

---

# 48. Audio Occlusion ≠ Physics

Physics responde:

```text id="aud-65"
há colisão?
```

Audio pode perguntar:

```text id="aud-66"
há obstrução acústica?
```

Pode reutilizar dados do mundo, mas são problemas diferentes.

---

# 49. Acoustic Portals

Em interiores:

```text id="aud-67"
Room A
   │
 doorway
   │
Room B
```

o som pode atravessar.

---

# 50. Acoustic Rooms

Criar:

```text id="aud-68"
AcousticRoom
```

com:

```text id="aud-69"
volume
reverb
occlusion
portal connections
```

---

# 51. Reverb

Ambientes podem definir:

```text id="aud-70"
ReverbProfile
```

Exemplos:

```text id="aud-71"
small_room
hall
cave
cathedral
tunnel
open_field
```

---

# 52. Dynamic Reverb

Entrar numa caverna:

```text id="aud-72"
open world
 ↓
crossfade
 ↓
cave reverb
```

---

# 53. Deep World Reverb

Grandes cavernas podem possuir:

```text id="aud-73"
long decay
distant reflections
low-frequency emphasis
```

---

# 54. Underwater Audio

Dentro da água:

```text id="aud-74"
underwater audio profile
```

com:

```text id="aud-75"
high frequency attenuation
muffled sounds
different propagation
```

---

# 55. Atmosphere Integration

Atmosphere pode informar:

```text id="aud-76"
density
pressure
composition
```

Audio usa para selecionar o modelo apropriado.

---

# 56. Space

Em vácuo, não assumir automaticamente áudio físico realista para gameplay.

O jogo pode definir uma política:

```text id="aud-77"
Space Audio Policy
```

por exemplo:

```text id="aud-78"
external sounds
→ cinematic/gameplay abstraction
internal ship sounds
→ normal propagation
```

---

# 57. Planet Audio

Cada planeta/dimensão pode ter:

```text id="aud-79"
atmospheric audio profile
```

---

# 58. Dimension Audio

Dimension Definition pode referenciar:

```text id="aud-80"
ambient profile
reverb
music
acoustic rules
```

---

# 59. Void Audio

Void pode possuir:

```text id="aud-81"
unique acoustic profile
```

sem precisar alterar o Audio Core.

---

# 60. Audio Bus

O mixer precisa ser hierárquico.

```text id="aud-82"
Master
├── Music
├── SFX
├── Voice
├── UI
├── Ambience
├── Vehicles
└── Environment
```

---

# 61. Bus Volume

Cada Bus possui:

```text id="aud-83"
volume
mute
solo
```

---

# 62. Bus Ducking

Quando alguém fala:

```text id="aud-84"
Voice
 ↓
duck
 ↓
Music / Ambience
```

---

# 63. Sidechain

Pode suportar:

```text id="aud-85"
Voice
→ duck Music

Alarm
→ duck Ambience
```

---

# 64. Mixer

Criar:

```text id="aud-86"
AudioMixer
```

com:

```text id="aud-87"
buses
effects
routing
ducking
```

---

# 65. DSP

Suportar uma cadeia de efeitos:

```text id="aud-88"
Source
 ↓
EQ
 ↓
Compressor
 ↓
Reverb
 ↓
Limiter
 ↓
Bus
```

---

# 66. DSP não precisa existir em todas as fontes

Só quando necessário.

---

# 67. Effects

Possíveis:

```text id="aud-89"
EQ
Compressor
Limiter
LowPass
HighPass
Reverb
Delay
Distortion
Chorus
Custom
```

---

# 68. Low-pass

Occlusion pode usar:

```text id="aud-90"
LowPassFilter
```

para sons atrás de paredes.

---

# 69. Distance Filtering

Distância pode alterar:

```text id="aud-91"
volume
high frequencies
reverb
```

---

# 70. Environmental Filtering

Bioma/caverna pode alterar:

```text id="aud-92"
EQ
reverb
ambience
```

---

# 71. Music System

Music merece um subsistema próprio.

```text id="aud-93"
MUSIC
├── TRACKS
├── STATES
├── LAYERS
├── TRANSITIONS
├── INTENSITY
└── PLAYLISTS
```

---

# 72. Dynamic Music

Music State pode depender de:

```text id="aud-94"
exploration
combat
danger
city
cave
boss
space
event
```

---

# 73. Music State Machine

```text id="aud-95"
EXPLORATION
 ↓
DANGER
 ↓
COMBAT
 ↓
VICTORY
 ↓
EXPLORATION
```

---

# 74. Music Layers

Uma música pode possuir:

```text id="aud-96"
base
percussion
tension
melody
```

e ativar camadas conforme intensidade.

---

# 75. Music Intensity

Valor:

```text id="aud-97"
0.0 → calm
1.0 → intense
```

o sistema pode selecionar/transicionar camadas.

---

# 76. Combat Music

Combat fornece:

```text id="aud-98"
combat intensity
```

Music resolve apresentação.

---

# 77. Civilization Music

Civilization pode fornecer contexto:

```text id="aud-99"
city prosperity
festival
war
disaster
```

Music escolhe o clima.

---

# 78. World Events Music

World Event pode gerar:

```text id="aud-100"
meteor
volcano
dimensional event
```

---

# 79. Exploration Music

Pode depender de:

```text id="aud-101"
biome
dimension
depth
weather
world phase
```

---

# 80. Context Scoring

Em vez de:

```text id="aud-102"
if combat play track
```

usar:

```text id="aud-103"
MusicContext
```

com scores:

```text id="aud-104"
combat = 0.8
danger = 0.5
exploration = 0.2
```

---

# 81. Music Director

Criar:

```text id="aud-105"
MusicDirector
```

que interpreta o contexto.

---

# 82. Voice System

Voice inclui:

```text id="aud-106"
Dialogue
NPC Voice
Radio
Communication
Narration
```

---

# 83. Dialogue

Dialogue System fornece:

```text id="aud-107"
line
speaker
emotion
timing
```

Audio reproduz voice asset.

---

# 84. Subtitle Synchronization

Audio pode fornecer:

```text id="aud-108"
word/marker timing
```

UI usa para legendas.

---

# 85. Voice Ducking

Durante diálogo:

```text id="aud-109"
voice
→ duck music
```

---

# 86. Radio

Para máquinas/veículos:

```text id="aud-110"
RadioSource
```

pode aplicar:

```text id="aud-111"
band-pass
noise
compression
```

---

# 87. Communications

Multiplayer pode possuir:

```text id="aud-112"
voice channel
radio channel
proximity channel
```

O Networking apenas transporta dados; Audio reproduz.

---

# 88. Proximity Audio

Um sistema de voz pode considerar:

```text id="aud-113"
distance
occlusion
orientation
room
```

---

# 89. UI Audio

UI pode usar:

```text id="aud-114"
click
hover
open
close
error
notification
```

---

# 90. UI não controla o Audio Device

Ela publica:

```text id="aud-115"
UIAudioEvent
```

Audio resolve.

---

# 91. Footsteps

Footsteps não devem ser:

```text id="aud-116"
player.walk = footstep.wav
```

Precisam considerar:

```text id="aud-117"
surface
speed
weight
movement
environment
```

---

# 92. Surface Audio

Block System fornece:

```text id="aud-118"
surfaceAudioProfile
```

Exemplo:

```text id="aud-119"
stone
grass
sand
snow
metal
wood
mud
water
```

---

# 93. Footstep Resolver

```text id="aud-120"
Footstep
 ↓
Block beneath
 ↓
surface profile
 ↓
speed
 ↓
animation marker
 ↓
sound selection
```

---

# 94. Animation Integration

Animation System pode publicar:

```text id="aud-121"
FootstepMarker
```

Audio responde.

---

# 95. Block + Audio

Block:

```text id="aud-122"
BlockSoundProfile
```

Audio resolve.

---

# 96. Item + Audio

Item:

```text id="aud-123"
ItemSoundProfile
```

Audio resolve.

---

# 97. Entity + Audio

Entity:

```text id="aud-124"
AudioProfile
```

Audio resolve.

---

# 98. Vehicle + Audio

Vehicle:

```text id="aud-125"
VehicleAudioProfile
```

---

# 99. Machine + Audio

Machine:

```text id="aud-126"
MachineAudioProfile
```

---

# 100. Weather + Audio

Climate:

```text id="aud-127"
WeatherAudioProfile
```

---

# 101. Biome + Audio

Biome:

```text id="aud-128"
BiomeAudioProfile
```

---

# 102. Dimension + Audio

Dimension:

```text id="aud-129"
DimensionAudioProfile
```

---

# 103. Dynamic Ambient System

Criar:

```text id="aud-130"
AmbientDirector
```

que combina:

```text id="aud-131"
Biome
+
Weather
+
Time
+
Depth
+
Entities
+
Civilization
+
Dimension
```

---

# 104. Time-of-Day

Ambience pode variar:

```text id="aud-132"
morning
day
evening
night
```

---

# 105. Seasonal Audio

```text id="aud-133"
spring
summer
autumn
winter
```

---

# 106. Weather Layer

```text id="aud-134"
clear
rain
storm
snow
sandstorm
fog
```

---

# 107. Population Layer

Uma cidade muda quando está cheia:

```text id="aud-135"
population density
```

e isso afeta:

```text id="aud-136"
voices
traffic
machines
market
```

---

# 108. Civilization Soundscape

Uma cidade poderia ter:

```text id="aud-137"
market
industry
construction
crowd
vehicles
music
```

dependendo do estado econômico.

---

# 109. Economy Soundscape

Por exemplo:

```text id="aud-138"
prosperous city
vs
industrial collapse
```

pode gerar contextos diferentes.

---

# 110. Ecology Soundscape

Animal density pode influenciar:

```text id="aud-139"
forest ambience
```

---

# 111. Dynamic Population Audio

Não tocar 5.000 sons individuais.

Criar:

```text id="aud-140"
aggregate ambience
```

---

# 112. Crowd Audio

Para 1.000 NPCs:

```text id="aud-141"
NPCs
 ↓
aggregated vocal layer
```

em vez de mil fontes simultâneas.

---

# 113. Audio LOD

NEXORA precisa de LOD também para áudio.

```text id="aud-142"
FULL
REDUCED
ABSTRACT
```

---

# 114. Full Audio

Perto:

```text id="aud-143"
individual source
spatial
occlusion
reverb
```

---

# 115. Reduced Audio

Mais longe:

```text id="aud-144"
simplified spatial
fewer sources
less DSP
```

---

# 116. Abstract Audio

Muito longe:

```text id="aud-145"
aggregate ambience
```

---

# 117. Source Budget

Definir:

```text id="aud-146"
max active sources
```

por prioridade/plataforma.

---

# 118. Voice Priority

Voice geralmente deve receber prioridade sobre:

```text id="aud-147"
cosmetic ambience
```

---

# 119. Critical Sound

Eventos importantes:

```text id="aud-148"
alarm
warning
boss
critical system
```

podem receber prioridade alta.

---

# 120. Audio Virtualization

Quando há muitas fontes:

```text id="aud-149"
virtual source
```

mantém estado sem necessariamente consumir hardware continuamente.

---

# 121. Source Pooling

Reutilizar fontes:

```text id="aud-150"
AudioSourcePool
```

para evitar alocações constantes.

---

# 122. Object Pooling

Especialmente para:

```text id="aud-151"
footsteps
impacts
particles-linked sounds
```

---

# 123. Audio Scheduler

Não iniciar centenas de sons simultaneamente.

Criar:

```text id="aud-152"
AudioScheduler
```

com:

```text id="aud-153"
priority
budget
distance
importance
```

---

# 124. Audio Culling

Eliminar sons:

```text id="aud-154"
too far
inaudible
occluded
low importance
duplicate
```

---

# 125. Audio Deduplication

Se 100 blocos iguais geram o mesmo som no mesmo instante:

```text id="aud-155"
coalesce
```

quando apropriado.

---

# 126. Example — Mining

```text id="aud-156"
10 NPCs mining
 ↓
100 block hits
```

não necessariamente 100 sons individuais.

Pode haver:

```text id="aud-157"
local impacts
+
aggregate industrial ambience
```

---

# 127. Acoustics Query

API:

```text id="aud-158"
IAcousticQuery
```

consulta:

```text id="aud-159"
occlusion
distance
room
material
reverb
```

---

# 128. Acoustic Simulation

Para áreas importantes:

```text id="aud-160"
source
 ↓
acoustic graph
 ↓
paths
 ↓
listener
```

---

# 129. Não simular tudo

Acoustic simulation cara deve ser limitada a:

```text id="aud-161"
nearby
important
audible
```

---

# 130. Acoustic Graph

Mundo pode ter:

```text id="aud-162"
rooms
portals
surfaces
```

e gerar rotas acústicas simplificadas.

---

# 131. Underground Audio

Grandes cavernas podem usar:

```text id="aud-163"
portal graph
```

para sons distantes.

---

# 132. Sound Propagation

Um som pode:

```text id="aud-164"
direct
reflect
occlude
attenuate
```

---

# 133. Reflections

Pode usar:

```text id="aud-165"
simplified reflection model
```

para cavernas e grandes salas.

---

# 134. Acoustic Zones

```text id="aud-166"
Zone A
→ reverb A

Zone B
→ reverb B
```

---

# 135. Zone Blending

Não trocar abruptamente.

```text id="aud-167"
0% zone A
 ↓
50/50
 ↓
100% zone B
```

---

# 136. Dynamic Material Acoustic Profile

Block Material pode fornecer:

```text id="aud-168"
absorption
reflection
transmission
```

---

# 137. Destruction Audio

Build System fornece:

```text id="aud-169"
material
damage
size
cause
```

Audio decide:

```text id="aud-170"
small crack
large collapse
metal impact
rockfall
```

---

# 138. Structural Collapse Audio

Build System pode publicar:

```text id="aud-171"
StructureCollapsedEvent
```

Audio cria uma sequência sonora.

---

# 139. Fluid Audio

Fluid Event pode possuir:

```text id="aud-172"
flow rate
pressure
volume
material
```

Audio usa isso para determinar intensidade.

---

# 140. Energy Audio

Energy API pode publicar:

```text id="aud-173"
overload
hum
arc
shutdown
```

Audio responde.

---

# 141. Heat Audio

Machines podem produzir:

```text id="aud-174"
vent
steam
alarm
```

---

# 142. Vehicle Engine

Não tocar apenas um arquivo em loop.

Usar parâmetros:

```text id="aud-175"
RPM
load
speed
gear
temperature
```

---

# 143. Layered Engine Audio

```text id="aud-176"
idle
low
mid
high
turbo
mechanical
```

misturados de acordo com estado.

---

# 144. Parameter-driven Audio

Criar:

```text id="aud-177"
AudioParameter
```

Exemplos:

```text id="aud-178"
RPM
speed
intensity
pressure
health
distance
```

---

# 145. Audio Curves

Parâmetros controlam:

```text id="aud-179"
volume
pitch
filter
effect send
```

---

# 146. Procedural Audio

Futuro suporte:

```text id="aud-180"
procedural oscillator
noise
synthesis
```

para sons que não precisam ser assets gravados.

---

# 147. Machine Procedural Audio

Máquinas repetitivas podem usar:

```text id="aud-181"
motor synthesis
```

com parâmetros.

---

# 148. Spacecraft Audio

Dentro da nave:

```text id="aud-182"
engine vibration
life support
electronics
warning
```

podem ser internos.

Exterior:

```text id="aud-183"
cinematic abstraction
```

conforme a política da dimensão/experiência.

---

# 149. Audio Buses por dimensão

Normalmente não é necessário.

Mas Dimension pode configurar:

```text id="aud-184"
default mix profile
```

---

# 150. Audio Profiles

Criar:

```text id="aud-185"
AudioProfile
```

que combina:

```text id="aud-186"
source
mixer
spatial
reverb
randomization
```

---

# 151. Profile Inheritance

Evitar herança profunda.

Preferir composição:

```text id="aud-187"
base profile
+
modifiers
```

---

# 152. Audio Modifiers

```text id="aud-188"
volume
pitch
filter
reverb
spatialization
```

podem ser modificados por contexto.

---

# 153. Environmental Modifier

Exemplo:

```text id="aud-189"
underwater
→ lowpass + reverb
```

---

# 154. Equipment Modifier

Capacete pode:

```text id="aud-190"
reduce external audio
```

via profile.

---

# 155. Status Effects

Certos estados podem modificar áudio:

```text id="aud-191"
stunned
underwater
vacuum
ear protection
```

Mas Status System possui o estado; Audio apenas interpreta.

---

# 156. Accessibility

Audio System precisa suportar:

```text id="aud-192"
master volume
SFX
music
voice
ambience
UI
spatial audio
dynamic range
```

---

# 157. Hearing Accessibility

Pode haver:

```text id="aud-193"
visual sound indicators
```

para sons importantes.

UI implementa a apresentação.

---

# 158. Subtitle Events

Áudio/Dialogue pode publicar:

```text id="aud-194"
SubtitleCue
```

---

# 159. Subtitle Data

```text id="aud-195"
speaker
text
time
position
importance
```

---

# 160. Positional Subtitles

Para determinados sons:

```text id="aud-196"
<som de explosão → esquerda>
```

UI resolve.

---

# 161. Audio + Event Bus

Eventos relevantes:

```text id="aud-197"
AudioRequested
AudioStarted
AudioStopped
AudioCompleted
AudioMarkerReached
```

Mas não publicar cada sample/frame.

---

# 162. Audio + Registry

```text id="aud-198"
AudioRegistry
AudioBusRegistry
MusicRegistry
ReverbRegistry
AcousticRegistry
```

---

# 163. Audio + Animation

```text id="aud-199"
AnimationMarker
 ↓
Audio System
```

Excelente para:

```text id="aud-200"
footsteps
reload
weapon mechanisms
machine timing
```

---

# 164. Audio + Entity

```text id="aud-201"
Entity
 ↓
Audio Profile
 ↓
Audio Emitter
```

---

# 165. Audio + Block

```text id="aud-202"
Block
 ↓
Surface/Sound Profile
 ↓
Audio
```

---

# 166. Audio + Item

```text id="aud-203"
Item
 ↓
Audio Profile
 ↓
Audio
```

---

# 167. Audio + Climate

```text id="aud-204"
Climate
 ↓
Weather Event
 ↓
Ambient Director
```

---

# 168. Audio + Fluid

```text id="aud-205"
Fluid
 ↓
Flow Event
 ↓
Water/River/Ocean soundscape
```

---

# 169. Audio + Civilization

```text id="aud-206"
Civilization State
 ↓
City Soundscape
```

---

# 170. Audio + Economy

Economy pode fornecer contexto:

```text id="aud-207"
festival
war
industrial boom
collapse
```

Audio pode mudar ambiente/música.

---

# 171. Audio + World Events

```text id="aud-208"
World Event
 ↓
Audio Event
```

---

# 172. Audio + Networking

Server não precisa transmitir áudio.

Pode transmitir:

```text id="aud-209"
semantic event / state
```

Cliente gera som.

---

# 173. Client-side Audio

Em multiplayer:

```text id="aud-210"
Server
 ↓
semantic state
 ↓
Client
 ↓
Audio System
```

---

# 174. Local-only Sounds

Alguns sons são puramente locais:

```text id="aud-211"
UI
headset
camera effects
personal equipment
```

---

# 175. Networked Audio Events

Alguns eventos precisam ser sincronizados:

```text id="aud-212"
large explosion
world event
important announcement
```

---

# 176. Distance-based Network Audio

Não enviar para todo mundo.

```text id="aud-213"
event position
 ↓
interest management
 ↓
nearby clients
```

---

# 177. Audio Event Importance

```text id="aud-214"
CRITICAL
IMPORTANT
NORMAL
AMBIENT
COSMETIC
```

---

# 178. Voice Networking

Voice chat deve ficar em subsistema próprio de comunicação, mas pode utilizar a infraestrutura de:

```text id="aud-215"
Audio Device
Mixer
Spatialization
```

---

# 179. Recording

Audio System pode eventualmente oferecer gravação para ferramentas/debug.

Não precisa existir no núcleo da primeira versão.

---

# 180. Audio Asset Streaming

Música e voice podem ser grandes.

Suportar:

```text id="aud-216"
preload
stream
decode-on-demand
```

---

# 181. Preload Policy

Cada asset define:

```text id="aud-217"
PRELOAD
STREAM
ON_DEMAND
```

---

# 182. Short SFX

Normalmente:

```text id="aud-218"
preload
```

---

# 183. Music

Normalmente:

```text id="aud-219"
stream
```

---

# 184. Long Voice

Pode:

```text id="aud-220"
stream
```

---

# 185. Audio Memory Budget

```text id="aud-221"
short sounds
→ resident

large sounds
→ streaming
```

---

# 186. Audio Cache

```text id="aud-222"
AudioAssetCache
```

com:

```text id="aud-223"
LRU / priority
```

---

# 187. Decode Threads

Assets podem ser decodificados em threads separadas.

```text id="aud-224"
I/O
 ↓
Decode Worker
 ↓
Audio Buffer
 ↓
Audio Device
```

---

# 188. Main Thread

Evitar:

```text id="aud-225"
load huge audio
decode huge audio
```

na thread principal.

---

# 189. Audio Device Abstraction

Criar:

```text id="aud-226"
IAudioBackend
```

para não prender o NEXORA a uma biblioteca específica.

---

# 190. Backend

Poderão existir adapters:

```text id="aud-227"
Desktop
Console
Mobile
Server-null
```

---

# 191. Dedicated Server

Servidor precisa funcionar sem áudio:

```text id="aud-228"
NullAudioBackend
```

---

# 192. Headless

Mesmo:

```text id="aud-229"
world simulation
```

sem Audio Device.

---

# 193. Audio API

Interface principal:

```text id="aud-230"
IAudioSystem

play()
stop()
pause()
setListener()
createEmitter()
setParameter()
setBusVolume()
```

---

# 194. Audio Event API

```text id="aud-231"
IAudioEventSystem

emit()
register()
resolve()
```

---

# 195. Audio Emitter API

```text id="aud-232"
IAudioEmitter

play()
stop()
setPosition()
setParameter()
setProfile()
```

---

# 196. Audio Mixer API

```text id="aud-233"
IAudioMixer

createBus()
setVolume()
setMute()
setEffect()
```

---

# 197. Spatial Audio API

```text id="aud-234"
ISpatialAudio

setListener()
setSource()
spatialize()
```

---

# 198. Acoustic API

```text id="aud-235"
IAcousticQuery

getOcclusion()
getReverb()
getRoom()
getPropagation()
```

---

# 199. Music API

```text id="aud-236"
IMusicDirector

setContext()
setIntensity()
play()
transition()
```

---

# 200. Voice API

```text id="aud-237"
IVoicePlayer

play()
stop()
setSpeaker()
setSubtitle()
```

---

# 201. Debug API

```text id="aud-238"
IAudioDebugger

inspectSources()
inspectBuses()
traceEvent()
profile()
```

---

# 202. Audio Runtime

```text id="aud-239"
AudioRuntime

update()
processEvents()
updateEmitters()
updateSpatialization()
updateMixer()
submitAudio()
```

---

# 203. Audio Scheduler

```text id="aud-240"
AudioScheduler

schedule()
prioritize()
virtualize()
cull()
```

---

# 204. Audio Graph

Para DSP:

```text id="aud-241"
Source
 ↓
Effect
 ↓
Bus
 ↓
Effect
 ↓
Master
```

---

# 205. Audio Graph Compilation

Profiles podem ser compilados durante boot:

```text id="aud-242"
Audio Definition
 ↓
validate
 ↓
compile routing
 ↓
runtime graph
```

---

# 206. Runtime IDs

Como nos outros sistemas:

```text id="aud-243"
AudioID
 ↓
RuntimeAudioID
```

---

# 207. Hot Path

Não usar strings para cada frame:

```text id="aud-244"
Runtime IDs
+
precomputed routes
+
numeric parameters
```

---

# 208. Pooling

Pool para:

```text id="aud-245"
AudioSource
AudioEmitter
VoiceInstance
EffectInstance
```

quando aplicável.

---

# 209. Voice Concurrency

Limitar:

```text id="aud-246"
max voices
```

e priorizar.

---

# 210. Voice Stealing

Quando não houver fonte disponível:

```text id="aud-247"
lowest priority
→ virtualized / stolen
```

---

# 211. Source Priority

Pode ser baseada em:

```text id="aud-248"
distance
importance
volume
gameplay significance
visibility
user focus
```

---

# 212. Audio Focus

Som importante perto da câmera:

```text id="aud-249"
priority boost
```

---

# 213. Camera-aware Audio

Listener pode fornecer:

```text id="aud-250"
forward vector
```

para priorizar sons na frente.

---

# 214. Stereo / HRTF

Dependendo do backend:

```text id="aud-251"
HRTF
```

pode produzir espacialização binaural.

---

# 215. Hardware abstraction

Não assumir que todo hardware suporta HRTF.

```text id="aud-252"
capability detection
```

---

# 216. Dynamic Range

Usuário pode configurar:

```text id="aud-253"
dynamic range
```

---

# 217. Loudness Management

Mixer pode normalizar grupos para evitar diferenças absurdas.

---

# 218. Limiter

Master Bus deve possuir:

```text id="aud-254"
Limiter
```

para evitar clipping.

---

# 219. Accessibility Mix

Pode existir perfil:

```text id="aud-255"
headphones
speakers
night
reduced bass
reduced intensity
```

---

# 220. Audio Settings

Configurações:

```text id="aud-256"
master
music
sfx
voice
ambience
ui
voice_chat
dynamic_range
spatial_audio
```

---

# 221. Save Settings

Configurações de usuário pertencem ao:

```text id="aud-257"
Player/Settings
```

Audio persiste apenas o que for responsabilidade dele.

---

# 222. Save Game

Audio runtime não precisa ser salvo.

Salvar apenas estado necessário:

```text id="aud-258"
music state
dialogue position
```

se gameplay exigir.

---

# 223. Audio Persistence Policy

```text id="aud-259"
Audio Asset
→ REGISTRY / CONTENT

Audio Runtime Source
→ TEMPORARY

Music State
→ OPTIONAL

Audio Device State
→ NEVER
```

---

# 224. Modding

Mods podem registrar:

```text id="aud-260"
AudioDefinition
AudioEvent
AudioProfile
MusicTrack
MusicState
ReverbProfile
AcousticProfile
```

---

# 225. Mod Asset Pipeline

```text id="aud-261"
Mod
 ↓
Audio Assets
 ↓
Importer
 ↓
Validation
 ↓
Audio Registry
```

---

# 226. Missing Audio

Se asset estiver faltando:

```text id="aud-262"
MissingAudio
```

pode usar:

```text id="aud-263"
fallback
```

em vez de crashar o jogo.

---

# 227. Fallback Hierarchy

Exemplo:

```text id="aud-264"
exact sound
 ↓
material fallback
 ↓
generic fallback
 ↓
silent
```

---

# 228. Audio Validation

Verificar:

```text id="aud-265"
invalid asset
missing reference
unsupported format
bad duration
invalid loop
missing bus
bad profile
missing registry entry
```

---

# 229. Audio Schema Version

```text id="aud-266"
AudioSchemaVersion
```

---

# 230. Migration

```text id="aud-267"
old audio definition
 ↓
migration
 ↓
new definition
```

---

# 231. Audio Pack

Futuro:

```text id="aud-268"
Audio Pack
```

pode substituir sons sem alterar lógica.

---

# 232. Resource Pack

Audio também pode estar em:

```text id="aud-269"
Resource Pack
```

com referências resolvidas pelo Asset System.

---

# 233. Content vs Asset

Separar:

```text id="aud-270"
Sound Asset
= arquivo

Audio Definition
= como usar

Audio Event
= quando usar
```

---

# 234. Audio Event Example

```text id="aud-271"
BlockBroken
{
    material: stone,
    size: medium,
    position: ...
}
```

Audio resolveria:

```text id="aud-272"
stone_break_medium
```

---

# 235. Machine Example

```text id="aud-273"
MachineProcessingChanged
{
    rpm: 1800,
    load: 0.72
}
```

Audio:

```text id="aud-274"
engine loop
→ RPM + load
→ pitch + volume
```

---

# 236. Vehicle Example

```text id="aud-275"
VehicleState
{
    rpm
    speed
    gear
    throttle
}
```

Audio calcula o mix.

---

# 237. Weather Example

```text id="aud-276"
StormIntensity
```

pode controlar:

```text id="aud-277"
wind volume
rain volume
thunder probability
```

---

# 238. Thunder

O som do trovão pode ser baseado em:

```text id="aud-278"
distance to lightning
```

Isso pode gerar atraso:

```text id="aud-279"
lightning
 ↓
distance
 ↓
thunder delay
```

O próprio Climate/World Event fornece o evento; Audio calcula apresentação.

---

# 239. Lightning + Audio

```text id="aud-280"
LightningEvent
 ↓
Audio
 ↓
flash sync / thunder timing
```

---

# 240. Ocean Audio

Fluid/Climate podem fornecer:

```text id="aud-281"
wave intensity
wind
water volume
```

Ambient Director compõe.

---

# 241. River Audio

Pode considerar:

```text id="aud-282"
flow rate
width
slope
```

---

# 242. Waterfall Audio

```text id="aud-283"
fluid volume
drop height
flow
```

podem alterar intensidade.

---

# 243. Machine Factory Audio

Civilization/Economy pode produzir contexto agregado:

```text id="aud-284"
industrial activity
```

Audio pode produzir um soundscape industrial.

---

# 244. City Audio

Uma cidade pode possuir camadas:

```text id="aud-285"
traffic
crowds
market
industry
construction
animals
music
```

---

# 245. City LOD

Longe:

```text id="aud-286"
1 city ambience
```

Perto:

```text id="aud-287"
multiple local emitters
```

---

# 246. Railway Audio

```text id="aud-288"
train
rails
signals
station
brakes
cargo
```

---

# 247. Space Audio

Nave:

```text id="aud-289"
life support
engines
alerts
crew
```

Planeta:

```text id="aud-290"
atmosphere
wind
weather
wildlife
```

---

# 248. Dimension Audio Profiles

Cada dimensão pode declarar:

```text id="aud-291"
base ambience
reverb
music
acoustic rules
```

---

# 249. Audio World State

Ambient Director pode receber:

```text id="aud-292"
Biome
Climate
Time
Season
Depth
Dimension
Civilization
Ecology
PlayerState
```

---

# 250. Soundscape Resolver

Criar:

```text id="aud-293"
SoundscapeResolver
```

que produz:

```text id="aud-294"
SoundscapeState
```

com:

```text id="aud-295"
layers
weights
zones
music
reverb
```

---

# 251. Soundscape State

Pode ser:

```text id="aud-296"
forest
night
rain
deep_cave
danger
```

simultaneamente.

---

# 252. Priority Resolution

Cada camada tem peso:

```text id="aud-297"
forest = 0.8
rain = 0.7
combat = 0.9
cave = 1.0
```

O sistema mistura.

---

# 253. No Giant if/else

Evitar:

```text id="aud-298"
if cave
if forest
if rain
if combat
if city
```

Preferir:

```text id="aud-299"
profiles
tags
contexts
layers
```

---

# 254. Audio Tags

```text id="aud-300"
#ambient
#combat
#machine
#vehicle
#forest
#cave
#city
#weather
#ui
```

---

# 255. Audio Query

```text id="aud-301"
IAudioQuery
```

para:

```text id="aud-302"
find profiles
find sounds
find variants
```

---

# 256. Event Filtering

Audio pode assinar somente:

```text id="aud-303"
nearby
relevant
audible
```

eventos.

---

# 257. Spatial Event Filter

```text id="aud-304"
Event
 ↓
position
 ↓
listener radius
 ↓
Audio
```

---

# 258. Event Bus não deve carregar áudio bruto

Não enviar:

```text id="aud-305"
PCM samples
```

pelo Event Bus.

Enviar:

```text id="aud-306"
semantic event + parameters
```

---

# 259. Audio Engine Boundary

```text id="aud-307"
GAME
 ↓
Audio API
 ↓
Audio Runtime
 ↓
Audio Backend
 ↓
OS / Device
```

---

# 260. Audio Runtime vs Backend

Runtime:

```text id="aud-308"
game-specific logic
```

Backend:

```text id="aud-309"
device-specific playback
```

---

# 261. Null Backend

Servidor:

```text id="aud-310"
NullAudioBackend
```

aceita chamadas sem produzir som.

---

# 262. Testing Backend

Criar:

```text id="aud-311"
TestAudioBackend
```

que grava:

```text id="aud-312"
events
sources
parameters
```

sem hardware.

---

# 263. Audio Unit Tests

Testar:

```text id="aud-313"
routing
random selection
attenuation
priority
bus mixing
parameter curves
fallbacks
```

---

# 264. Integration Tests

```text id="aud-314"
BlockBroken
 ↓
Event Bus
 ↓
Audio
 ↓
expected sound
```

---

# 265. Acoustic Tests

```text id="aud-315"
source
+
wall
+
listener
```

esperar:

```text id="aud-316"
occlusion > 0
```

---

# 266. Music Tests

```text id="aud-317"
exploration
 ↓
combat
 ↓
victory
 ↓
exploration
```

verificar transições.

---

# 267. Stress Test

```text id="aud-318"
1,000 emitters
10,000 emitters
100,000 abstract sources
```

com LOD.

---

# 268. Voice Stress

```text id="aud-319"
100
1,000
10,000
```

NPC audio sources.

---

# 269. Audio Budget Test

Garantir que:

```text id="aud-320"
active hardware voices
```

permaneçam dentro do orçamento.

---

# 270. Streaming Test

```text id="aud-321"
large music file
 ↓
stream
 ↓
seek
 ↓
pause
 ↓
resume
```

---

# 271. Mod Audio Test

```text id="aud-322"
Mod
 ↓
register Audio
 ↓
register Event
 ↓
subscribe
 ↓
emit
 ↓
play
 ↓
unload mod
```

---

# 272. Missing Asset Test

```text id="aud-323"
missing asset
 ↓
fallback
 ↓
no crash
```

---

# 273. Audio Save Test

Verificar que fechar/reabrir o jogo:

```text id="aud-324"
music state
dialogue state
```

volte corretamente quando persistido.

---

# 274. Performance Targets

Medir:

```text id="aud-325"
CPU
memory
active voices
DSP time
streaming bandwidth
latency
```

---

# 275. Audio Debugger

Comando:

```text id="aud-326"
nexora audio inspect
```

mostrar:

```text id="aud-327"
sources
emitters
listener
buses
music state
reverb
occlusion
```

---

# 276. Audio Source Debug

```text id="aud-328"
nexora audio sources
```

---

# 277. Audio Bus Debug

```text id="aud-329"
nexora audio buses
```

---

# 278. Audio Profiler

```text id="aud-330"
nexora audio profiler
```

mostra:

```text id="aud-331"
voice count
CPU
memory
DSP
streaming
source priorities
```

---

# 279. Acoustic Debug

```text id="aud-332"
nexora audio acoustics
```

pode visualizar:

```text id="aud-333"
listener
sources
rays
rooms
portals
occlusion
```

---

# 280. Music Debug

```text id="aud-334"
nexora music inspect
```

mostra:

```text id="aud-335"
current state
intensity
layers
transition
```

---

# 281. Estrutura de código

Eu organizaria assim:

```text id="aud-code-01"
src/
└── audio/
    ├── core/
    │   ├── audio.ts
    │   ├── audio-definition.ts
    │   ├── audio-event.ts
    │   ├── audio-asset.ts
    │   └── audio-parameter.ts
    │
    ├── source/
    │   ├── audio-source.ts
    │   ├── audio-emitter.ts
    │   └── source-pool.ts
    │
    ├── spatial/
    │   ├── listener.ts
    │   ├── spatializer.ts
    │   ├── attenuation.ts
    │   └── doppler.ts
    │
    ├── acoustics/
    │   ├── acoustic-query.ts
    │   ├── acoustic-room.ts
    │   ├── acoustic-portal.ts
    │   ├── occlusion.ts
    │   └── propagation.ts
    │
    ├── mixer/
    │   ├── mixer.ts
    │   ├── bus.ts
    │   ├── routing.ts
    │   ├── ducking.ts
    │   └── effects.ts
    │
    ├── ambience/
    │   ├── ambient-director.ts
    │   ├── soundscape.ts
    │   ├── zones.ts
    │   └── environment-profile.ts
    │
    ├── music/
    │   ├── music-director.ts
    │   ├── music-state.ts
    │   ├── music-layer.ts
    │   └── transition.ts
    │
    ├── voice/
    │   ├── voice-player.ts
    │   ├── dialogue-audio.ts
    │   └── subtitle-cues.ts
    │
    ├── runtime/
    │   ├── audio-runtime.ts
    │   ├── audio-scheduler.ts
    │   ├── audio-virtualization.ts
    │   └── audio-lod.ts
    │
    ├── streaming/
    │   ├── audio-stream.ts
    │   ├── decoder.ts
    │   └── cache.ts
    │
    ├── registry/
    │   ├── audio-registry.ts
    │   ├── bus-registry.ts
    │   ├── music-registry.ts
    │   └── acoustic-registry.ts
    │
    ├── networking/
    │   └── audio-replication.ts
    │
    ├── serialization/
    │   └── migration.ts
    │
    ├── backend/
    │   ├── audio-backend.ts
    │   ├── null-backend.ts
    │   └── test-backend.ts
    │
    ├── debugging/
    │   ├── audio-debugger.ts
    │   ├── audio-profiler.ts
    │   └── acoustic-debugger.ts
    │
    └── api/
        └── audio-api.ts
```

---

# 282. APIs principais

```text id="aud-api-01"
IAudioSystem
IAudioSource
IAudioEmitter
IAudioListener
IAudioMixer
IAudioBus
ISpatialAudio
IAcousticQuery
IMusicDirector
IVoicePlayer
IAudioBackend
```

Conteúdo:

```text id="aud-api-02"
IAudioDefinition
IAudioEvent
IAudioProfile
IAudioParameter
```

---

# 283. Fronteira arquitetural

## Audio System faz

```text id="aud-boundary-01"
audio asset resolution
sound selection
source management
spatialization
attenuation
occlusion
reverb
mixing
DSP
music
voice playback
ambience
soundscapes
audio LOD
streaming
audio device abstraction
```

## Não faz

```text id="aud-boundary-02"
Combat
AI
Physics
Climate simulation
Fluid simulation
Entity lifecycle
Animation
Inventory
Economy
Civilization
World Generation
```

---

# 284. Regra fundamental

> **Audio System transforma eventos e contexto do mundo em uma experiência sonora. Ele não define a lógica que causou esses eventos.**

---

# 285. Segunda regra

> **Gameplay produz fatos e estados; Audio interpreta esses fatos e estados para produzir som.**

---

# 286. Terceira regra

> **O áudio deve continuar sendo opcional para a simulação. Desligar ou remover o backend de áudio não pode quebrar o mundo.**

Essa terceira é importantíssima para:

```text id="aud-final-01"
Dedicated Server
Headless
CI
Automated Tests
AI Simulation
World Generation
```

---

# 287. Quarta regra

> **O NEXORA não deve criar uma fonte de áudio física para tudo que existe no mundo. Fontes próximas recebem detalhe individual; grandes quantidades distantes são agregadas em soundscapes.**

Isso combina diretamente com a arquitetura LOD do Entity System e da simulação.

---

# 288. Ordem de implementação

```text id="aud-order"
AUDIO-0    Core Contracts
AUDIO-1    AudioID
AUDIO-2    AudioRegistry
AUDIO-3    AudioAsset
AUDIO-4    AudioDefinition
AUDIO-5    AudioEvent
AUDIO-6    AudioSource
AUDIO-7    AudioEmitter
AUDIO-8    Listener
AUDIO-9    Playback
AUDIO-10   Audio Pool
AUDIO-11   Mixer
AUDIO-12   Bus
AUDIO-13   Routing
AUDIO-14   Volume
AUDIO-15   Pitch
AUDIO-16   Random Variants
AUDIO-17   Parameters
AUDIO-18   Spatialization
AUDIO-19   Attenuation
AUDIO-20   Event Bus Integration
AUDIO-21   Block Audio
AUDIO-22   Item Audio
AUDIO-23   Entity Audio
AUDIO-24   Animation Markers
AUDIO-25   Footsteps
AUDIO-26   Machine Audio
AUDIO-27   Vehicle Audio
AUDIO-28   Climate Audio
AUDIO-29   Ambient Director
AUDIO-30   Soundscapes
AUDIO-31   Reverb
AUDIO-32   Occlusion
AUDIO-33   Acoustic Rooms
AUDIO-34   Acoustic Portals
AUDIO-35   Music
AUDIO-36   Dynamic Music
AUDIO-37   Voice
AUDIO-38   Subtitles
AUDIO-39   Streaming
AUDIO-40   Cache
AUDIO-41   LOD
AUDIO-42   Virtualization
AUDIO-43   Networking
AUDIO-44   Serialization
AUDIO-45   Mod API
AUDIO-46   Backend
AUDIO-47   Null Backend
AUDIO-48   Debugging
AUDIO-49   Profiling
AUDIO-50   Stress Tests
AUDIO-51   Compatibility
```

---

# 289. Primeiro Vertical Slice

```text id="aud-vs-01"
AudioRegistry
       ↓
AudioDefinition
       ↓
AudioSource
       ↓
AudioBackend
       ↓
Sound
```

---

# 290. Segundo Vertical Slice

```text id="aud-vs-02"
BlockBrokenEvent
       ↓
Event Bus
       ↓
Audio
       ↓
BlockSoundProfile
       ↓
stone_break
```

---

# 291. Terceiro Vertical Slice

```text id="aud-vs-03"
Player
 ↓
Animation
 ↓
FootstepMarker
 ↓
Audio
 ↓
Block Surface
 ↓
Footstep Sound
```

Esse é um teste excelente porque conecta **Entity + Animation + Block + Event Bus + Audio**.

---

# 292. Quarto Vertical Slice

```text id="aud-vs-04"
Biome
+
Climate
+
Time
+
Depth
 ↓
Ambient Director
 ↓
Soundscape
 ↓
Mixer
 ↓
Output
```

---

# 293. Quinto Vertical Slice

```text id="aud-vs-05"
Combat
 ↓
Combat State
 ↓
Music Director
 ↓
Intensity
 ↓
Music Layers
 ↓
Transition
 ↓
Output
```

---

# 294. Sexto Vertical Slice

```text id="aud-vs-06"
Machine
 ↓
RPM / Load
 ↓
Audio Parameters
 ↓
Layered Engine Sound
 ↓
Spatialization
 ↓
Occlusion
 ↓
Reverb
```

---

# 295. Sétimo Vertical Slice — Modding

```text id="aud-vs-07"
Mod
 ↓
Audio Asset
 ↓
Audio Definition
 ↓
Audio Event
 ↓
Registry
 ↓
Event Bus
 ↓
Audio
 ↓
Unload Mod
 ↓
Subscriptions/assets safely removed
```

---

# 296. Teste de escala

```text id="aud-scale-01"
10 sources
100
1,000
10,000
100,000 abstract emitters
```

com:

```text id="aud-scale-02"
spatialization
LOD
virtualization
aggregation
priority
```

---

# 297. Teste crítico de soundscape

Simular:

```text id="aud-scale-03"
cidade com 10.000 NPCs
500 máquinas
200 veículos
chuva
vento
ferrovia
mercado
```

e não permitir que:

```text id="aud-scale-04"
10.000 + 500 + 200
```

virem milhares de fontes de áudio físicas simultâneas.

O sistema deve transformar grande parte disso em:

```text id="aud-scale-05"
local sources
+
crowd layer
+
industrial layer
+
traffic layer
+
weather layer
```

---

# 298. Arquitetura final

```text id="aud-final-02"
                         NEXORA
                           │
                     EVENT BUS
                           │
                    AUDIO SYSTEM
                           │
          ┌────────────────┼─────────────────┐
          ↓                ↓                 ↓
      SOUND EVENTS      CONTEXT          CONTENT
          │                │                 │
          │                ├── Biome         │
          │                ├── Climate       │
          │                ├── Entity        │
          │                ├── Block         │
          │                ├── Combat        │
          │                └── World         │
          │                                  │
          └──────────────────┬───────────────┘
                             ↓
                     SOUND RESOLUTION
                             ↓
                ┌────────────┼────────────┐
                ↓            ↓            ↓
            SPATIAL       MIXER       MUSIC
                │            │            │
                ↓            ↓            ↓
             ACOUSTICS       DSP       LAYERS
                └────────────┼────────────┘
                             ↓
                       AUDIO BACKEND
                             ↓
                        AUDIO DEVICE
```

E a fundação agora fica:

```text id="aud-foundation"
NEXORA FOUNDATION
├── Core
├── Registry System
├── Event Bus
├── Save / Persistence
├── Entity System
├── Block System
├── Item System
├── Animation System
└── Audio System
```

A separação mais importante entre **Animation e Audio** fica:

```text id="aud-animation-link"
Animation
    │
    └── Marker: Footstep
                    ↓
                 Event Bus
                    ↓
                  Audio
```

e entre **Block e Audio**:

```text id="aud-block-link"
Block
 └── Surface/Sound Profile
             ↓
          Audio
```

e entre **Climate e Audio**:

```text id="aud-climate-link"
Climate
 └── Weather State
          ↓
      Ambient Director
          ↓
       Soundscape
```

Assim o Audio System consegue suportar desde **um passo sobre grama** até **uma cidade industrial subterrânea inteira, uma tempestade, uma ferrovia, uma caverna gigantesca ou uma nave espacial**, sem transformar esses sistemas em dependências diretas do motor de áudio.
