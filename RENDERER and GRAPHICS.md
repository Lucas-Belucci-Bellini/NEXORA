Sim. O **Renderer / Gráficos** merece um Master Plan bem maior, porque no NEXORA ele vai ter que lidar com um mundo voxel gigantesco, cavernas profundas, oceanos, cidades, milhares de entidades, veículos, dimensões e espaço — sem virar um gargalo.

Eu separaria **Rendering** do restante da Engine assim:

```text
NEXORA ENGINE
│
├── WORLD
├── PHYSICS
├── AUDIO
├── INPUT
│
└── RENDERER
    ├── Render Core
    ├── Render Pipeline
    ├── Voxel Renderer
    ├── Entity Renderer
    ├── Terrain Renderer
    ├── Water Renderer
    ├── Sky / Atmosphere
    ├── Lighting
    ├── Shadows
    ├── Materials
    ├── Particles
    ├── Vegetation
    ├── Structures
    ├── Vehicles
    ├── UI Renderer
    ├── Post Processing
    ├── LOD
    ├── Occlusion
    ├── Streaming
    ├── GPU Resource Management
    ├── Debug Rendering
    └── Mod Rendering API
```

# NEXORA — RENDERER / GRAPHICS MASTER PLAN

## 1. Objetivo

O Renderer deve transformar o estado do NEXORA em imagem.

A arquitetura ideal:

```text
WORLD STATE
    ↓
RENDER EXTRACTION
    ↓
RENDER WORLD
    ↓
VISIBILITY
    ↓
PASS PREPARATION
    ↓
GPU
    ↓
FRAME
```

A regra é:

> **O mundo não renderiza diretamente. O Renderer extrai uma representação visual do mundo.**

Isso permite que a simulação continue funcionando mesmo quando a representação gráfica muda.

---

# 2. RENDER-0 — Render Core

Criar o núcleo do sistema:

```text
Renderer
RenderDevice
RenderContext
RenderFrame
RenderView
RenderTarget
RenderPass
```

Responsabilidades:

```text
initialize
beginFrame
render
endFrame
resize
shutdown
```

---

# 3. RENDER-1 — Graphics Backend

O Renderer deve ter uma camada abstrata sobre a API gráfica.

Conceitualmente:

```text
NEXORA RENDER API
       │
 ┌─────┼─────┐
 │     │     │
Vulkan DirectX Metal
```

Não colocar chamadas específicas de GPU espalhadas pelo código inteiro.

Criar abstrações:

```text
GPUBuffer
GPUTexture
GPUSampler
GPUShader
GPUPipeline
GPUFramebuffer
GPUCommandBuffer
```

Isso também deixa o projeto mais preparado para diferentes plataformas.

---

# 4. RENDER-2 — Render Device

Gerencia a GPU:

```text
RenderDevice
├── capabilities
├── memory
├── queues
├── command buffers
├── textures
├── buffers
└── pipelines
```

Deve detectar:

```text
GPU
VRAM
feature level
supported formats
ray tracing support
compute support
```

---

# 5. RENDER-3 — Frame System

Cada frame possui um contexto próprio.

```text
RenderFrame
├── camera
├── visibleChunks
├── visibleEntities
├── lights
├── particles
├── renderPasses
└── postProcess
```

Pipeline:

```text
Frame Start
↓
Visibility
↓
Geometry
↓
Lighting
↓
Transparency
↓
Particles
↓
Sky
↓
Post Processing
↓
UI
↓
Present
```

---

# 6. RENDER-4 — Render Graph

Eu colocaria um **Render Graph** desde cedo.

Em vez de:

```text
pass1
pass2
pass3
pass4
```

ter:

```text
Render Graph
├── resources
├── dependencies
├── passes
└── synchronization
```

Exemplo:

```text
Shadow Pass
     ↓
Depth
     ↓
GBuffer
     ↓
Lighting
     ↓
Transparency
     ↓
Post Processing
```

Isso facilita reorganizar a pipeline sem reescrever tudo.

---

# 7. RENDER-5 — Camera System

Criar:

```text
Camera
PerspectiveCamera
OrthographicCamera
CameraController
```

Dados:

```text
position
rotation
FOV
nearPlane
farPlane
projection
view
viewProjection
```

Também:

```text
frustum
jitter
camera-relative coordinates
```

---

# 8. RENDER-6 — Floating Origin

Isso é especialmente importante para o NEXORA.

Como o mundo pode ser gigantesco, coordenadas enormes começam a perder precisão em ponto flutuante.

Então usar:

```text
World Coordinates
        ↓
Origin Cell
        ↓
Local Render Coordinates
        ↓
GPU
```

O jogador fica perto da origem local enquanto o mundo continua enorme.

Isso também ajuda Far Lands, Beyondlands e exploração espacial.

---

# 9. RENDER-7 — Coordinate Systems

Separar claramente:

```text
World Space
Chunk Space
Local Space
View Space
Clip Space
Screen Space
```

Evita uma enorme quantidade de bugs.

---

# 10. RENDER-8 — Voxel Renderer

Esse será um dos maiores sistemas.

Fluxo:

```text
Chunk
 ↓
Voxel Data
 ↓
Meshing
 ↓
Mesh
 ↓
GPU Buffer
 ↓
Render
```

---

# 11. RENDER-9 — Voxel Meshing

Não renderizar cada bloco como um cubo independente.

Criar:

```text
Greedy Meshing
Face Culling
Mesh Compression
Chunk Meshing
Async Meshing
```

Também preparar a arquitetura para algoritmos futuros:

```text
Greedy
Surface Nets
Marching Cubes
Dual Contouring
```

Isso será útil para cavernas e terrenos especiais.

---

# 12. RENDER-10 — Chunk Mesh

Cada chunk deve possuir:

```text
ChunkRenderData
├── opaqueMesh
├── cutoutMesh
├── transparentMesh
├── waterMesh
└── metadata
```

Estado:

```text
UNMESHED
MESHING
READY
UPDATING
DISPOSING
```

---

# 13. RENDER-11 — Chunk Rebuild

Quando um bloco muda:

```text
Block changed
 ↓
Chunk dirty
 ↓
Neighbor check
 ↓
Remesh
 ↓
GPU update
```

Não reconstruir o mundo inteiro.

---

# 14. RENDER-12 — Async Meshing

A geração da malha deve poder rodar fora da thread principal:

```text
World Thread
       ↓
Chunk Changed
       ↓
Mesh Task
       ↓
Worker Thread
       ↓
GPU Upload
```

Isso é crucial para mineração/construção sem travar a câmera.

---

# 15. RENDER-13 — Material System

Criar materiais independentes da geometria.

```text
Material
├── shader
├── textures
├── parameters
├── blendMode
├── cullingMode
└── renderingFlags
```

Tipos:

```text
Opaque
Cutout
Transparent
Emissive
Translucent
```

---

# 16. RENDER-14 — Texture System

Gerenciar:

```text
Texture2D
Texture3D
TextureArray
Cubemap
RenderTexture
```

Também:

```text
mipmaps
streaming
compression
atlas
virtual textures
```

---

# 17. RENDER-15 — Texture Atlas

Para voxel rendering:

```text
Block IDs
 ↓
Texture Atlas
 ↓
GPU
```

Mas eu deixaria suporte também para Texture Arrays.

Isso evita que o sistema fique preso a um único método.

---

# 18. RENDER-16 — Shader System

Criar:

```text
Shader
ShaderProgram
ShaderVariant
ShaderPipeline
```

Shaders separados por função:

```text
terrain
entity
water
particle
sky
shadow
postprocess
UI
```

---

# 19. RENDER-17 — Shader Variants

Evitar criar um shader gigante cheio de `if`.

Usar variantes:

```text
TERRAIN
+ EMISSIVE
+ NORMAL_MAP
+ WET
```

compilado conforme necessário.

Também deve existir cache.

---

# 20. RENDER-18 — Lighting

O sistema de iluminação precisa suportar:

```text
Sun
Moon
Block Light
Point Light
Spot Light
Area Light
Emissive
```

---

# 21. RENDER-19 — Voxel Light

Para o mundo voxel:

```text
Sky Light
Block Light
```

Cada voxel pode armazenar níveis de iluminação.

A propagação precisa ser incremental:

```text
light source
 ↓
propagate
 ↓
neighbors
 ↓
update
```

---

# 22. RENDER-20 — Dynamic Lighting

Entidades e máquinas poderão possuir luzes.

Exemplos:

```text
lantern
torch
machine
vehicle
magic effect
spaceship
```

Mas luz dinâmica precisa ser controlada para evitar milhares de fontes ativas.

---

# 23. RENDER-21 — Shadows

Sistema separado:

```text
ShadowSystem
```

Possíveis técnicas:

```text
Shadow Maps
Cascaded Shadow Maps
Contact Shadows
Virtual Shadow Maps
```

Para o mundo voxel, sombras podem ter soluções híbridas.

---

# 24. RENDER-22 — Sun / Moon

Criar:

```text
CelestialSystem
├── sun
├── moon
├── stars
└── celestial bodies
```

Com integração com:

```text
World Time
Weather
Dimension
Atmosphere
```

---

# 25. RENDER-23 — Sky

Pipeline:

```text
Sky
├── atmosphere
├── sun
├── moon
├── clouds
├── stars
└── horizon
```

O céu deve variar por dimensão.

---

# 26. RENDER-24 — Atmosphere

Criar modelo para:

```text
Rayleigh-like scattering
Mie-like scattering
fog
horizon
sun tint
ambient light
```

Não precisa começar fisicamente perfeito.

Pode evoluir por versões.

---

# 27. RENDER-25 — Fog

Fog deve ser contextual.

```text
Biome
+
Weather
+
Altitude
+
Depth
+
Dimension
=
Fog
```

Exemplos:

```text
ocean depth
→ dark blue fog

cave
→ dense local fog

Far Lands
→ frontier haze

alien dimension
→ custom atmosphere
```

---

# 28. RENDER-26 — Water Renderer

Água merece um renderer próprio.

```text
WaterSurface
├── mesh
├── normals
├── reflection
├── refraction
├── depth
└── caustics
```

Também:

```text
waves
currents
foam
underwater fog
```

---

# 29. RENDER-27 — Deep Ocean Rendering

Em grandes profundidades:

```text
surface
 ↓
light attenuation
 ↓
color absorption
 ↓
pressure atmosphere
 ↓
darkness
```

Isso conecta diretamente com os oceanos profundos do WorldGen.

---

# 30. RENDER-28 — Fluid Rendering

Não limitar a água.

A arquitetura deve suportar:

```text
water
lava
oil-like fluid
chemical fluids
magic fluids
```

Cada fluido pode possuir:

```text
surface appearance
density look
color
opacity
emission
animation
```

---

# 31. RENDER-29 — Entity Renderer

Criar:

```text
EntityRenderer
```

Com:

```text
PlayerRenderer
MobRenderer
NPCRenderer
VehicleRenderer
ItemRenderer
ProjectileRenderer
```

---

# 32. RENDER-30 — Model System

Suportar:

```text
Static Mesh
Skinned Mesh
Voxel Model
Procedural Mesh
Instanced Mesh
```

---

# 33. RENDER-31 — Animation

Separar:

```text
Animation
AnimationController
Skeleton
Bone
Clip
BlendTree
```

Suporte a:

```text
idle
walk
run
jump
swim
climb
custom
```

---

# 34. RENDER-32 — Instancing

Para muitos objetos semelhantes:

```text
10.000 árvores
```

não queremos:

```text
10.000 draw calls
```

Usar:

```text
GPU Instancing
Indirect Drawing
Instance Buffers
```

Isso será muito importante para florestas e cidades.

---

# 35. RENDER-33 — Vegetation Renderer

Vegetação precisa ser tratada de forma especializada.

```text
Trees
Grass
Bushes
Crops
Flowers
```

Técnicas:

```text
instancing
LOD
billboards
wind animation
density batching
```

---

# 36. RENDER-34 — Foliage Wind

Vegetação pode responder ao ambiente:

```text
Wind Field
 ↓
Vegetation Shader
```

E variar conforme:

```text
weather
wind strength
season
biome
```

---

# 37. RENDER-35 — Structure Rendering

Cidades e estruturas grandes precisam de:

```text
visibility
batching
LOD
occlusion
instancing
```

Não podemos tratar uma cidade inteira como milhares de objetos totalmente independentes.

---

# 38. RENDER-36 — Entity LOD

Entidades podem possuir:

```text
LOD0
LOD1
LOD2
LOD3
Billboard
Hidden
```

Exemplo:

```text
NPC perto
→ modelo completo

NPC distante
→ modelo simplificado

NPC muito distante
→ billboard

NPC fora da área visual
→ não renderiza
```

---

# 39. RENDER-37 — Chunk LOD

O mundo também precisa de LOD.

```text
Near
 ↓
Full voxel mesh

Medium
 ↓
Simplified mesh

Far
 ↓
Terrain proxy

Very Far
 ↓
LOD terrain
```

Isso será muito importante para horizontes enormes.

---

# 40. RENDER-38 — Occlusion Culling

Não renderizar coisas que não podem ser vistas.

```text
Camera
 ↓
Frustum
 ↓
Occlusion
 ↓
Visible Objects
```

Em cavernas será extremamente importante.

---

# 41. RENDER-39 — Frustum Culling

Primeira filtragem:

```text
camera frustum
 ↓
chunks/entities
 ↓
visible
```

Pode eliminar enormes quantidades de objetos.

---

# 42. RENDER-40 — Portal / Region Visibility

Dimensões, prédios e cavernas podem utilizar estruturas de visibilidade.

Exemplo:

```text
Cidade
 ↓
district
 ↓
building
 ↓
room
```

Não desenhar tudo se o jogador não tem visão dele.

---

# 43. RENDER-41 — Particles

Sistema:

```text
ParticleSystem
ParticleEmitter
Particle
ParticleMaterial
```

Usos:

```text
fumaça
fogo
poeira
água
mágica
energia
weather
explosão visual
```

Sempre separando efeitos visuais da lógica do gameplay.

---

# 44. RENDER-42 — Weather Rendering

Integrar com o Weather System:

```text
Rain
Snow
Sand
Ash
Fog
Storm
```

Efeitos dependem de:

```text
biome
season
altitude
dimension
weather state
```

---

# 45. RENDER-43 — Clouds

Sistema próprio:

```text
Cloud Layer
Cloud Volume
Cloud Shadows
```

Pode começar simples e evoluir.

---

# 46. RENDER-44 — Post Processing

Pipeline:

```text
HDR
 ↓
Exposure
 ↓
Tone Mapping
 ↓
Bloom
 ↓
Color Grading
 ↓
AA
 ↓
Final Image
```

---

# 47. RENDER-45 — Anti-Aliasing

Arquitetura preparada para:

```text
MSAA
FXAA
TAA
Temporal methods
```

A escolha pode depender do hardware.

---

# 48. RENDER-46 — HDR

Internamente:

```text
HDR Render Target
```

para iluminação e efeitos.

Depois:

```text
Tone Mapping
 ↓
Display
```

---

# 49. RENDER-47 — Global Illumination

Esse seria um sistema evolutivo.

Começar com:

```text
ambient lighting
sky lighting
voxel light
```

Depois considerar:

```text
screen-space GI
voxel GI
probe-based GI
hardware accelerated options
```

Sem obrigar hardware topo de linha.

---

# 50. RENDER-48 — Ambient Occlusion

No mundo voxel, AO pode trazer enorme ganho visual.

```text
Voxel neighbors
 ↓
Ambient Occlusion
 ↓
Mesh / Shader
```

---

# 51. RENDER-49 — Emissive Materials

Blocos/objetos podem emitir luz visualmente:

```text
Material
├── emissionColor
└── emissionStrength
```

Exemplos conceituais:

```text
crystal
machine
magic block
reactor
space technology
```

---

# 52. RENDER-50 — Reflections

Pipeline escalável:

```text
Basic
→ cubemap

Intermediate
→ SSR

Advanced
→ hardware accelerated techniques
```

A qualidade deve se adaptar ao hardware.

---

# 53. RENDER-51 — Transparency

Transparência é cara.

Criar categorias:

```text
Opaque
Masked
Translucent
Additive
```

Com ordenação onde necessária.

---

# 54. RENDER-52 — Decals

Suporte futuro a:

```text
signs
paint
scorch marks
logos
road markings
surface effects
```

Sem precisar modificar o mesh principal.

---

# 55. RENDER-53 — Damage / Surface Effects

Uma estrutura pode receber estados visuais:

```text
wet
dusty
burned
damaged
frozen
corroded
```

O Material System pode representar isso através de parâmetros.

---

# 56. RENDER-54 — Procedural Materials

Algumas superfícies podem utilizar dados procedurais:

```text
rock variation
terrain variation
ice
sand
alien materials
```

Para reduzir dependência de texturas únicas.

---

# 57. RENDER-55 — Resource Streaming

Não carregar todas as texturas/modelos de uma vez.

```text
Player
 ↓
Asset Priority
 ↓
Streaming
 ↓
VRAM
```

Prioridades:

```text
CRITICAL
HIGH
NORMAL
LOW
BACKGROUND
```

---

# 58. RENDER-56 — VRAM Manager

Controle:

```text
texture memory
mesh memory
buffer memory
render targets
temporary resources
```

Quando necessário:

```text
evict
reload
compress
downscale
```

---

# 59. RENDER-57 — GPU Memory Budget

Exibir:

```text
VRAM Used
VRAM Reserved
VRAM Available
Textures
Meshes
Shaders
RT
```

Ideal para diagnóstico.

---

# 60. RENDER-58 — Draw Call Management

Criar métricas:

```text
draw calls
triangles
vertices
instances
passes
GPU time
CPU render time
```

E estratégias:

```text
batching
instancing
indirect draw
mesh merging
```

---

# 61. RENDER-59 — Render Queue

Objetos podem ser classificados:

```text
BACKGROUND
OPAQUE
CUTOUT
TRANSPARENT
EMISSIVE
PARTICLE
UI
```

Ordenação pode considerar:

```text
material
shader
distance
priority
```

---

# 62. RENDER-60 — Render Extraction

Um ponto importantíssimo:

```text
Simulation
    ↓
Render Extraction
    ↓
Render Scene
```

O Renderer não deve consultar entidades uma por uma durante cada draw.

Ele recebe uma representação preparada:

```text
RenderEntity
RenderChunk
RenderLight
RenderParticle
```

---

# 63. RENDER-61 — Render World

Criar:

```text
RenderWorld
```

com:

```text
visible chunks
visible entities
lights
effects
sky state
environment
```

Ele é uma fotografia otimizada do estado visual.

---

# 64. RENDER-62 — Multiplayer Rendering

O cliente pode receber:

```text
authoritative world state
```

e produzir:

```text
interpolation
prediction
visual smoothing
```

O Renderer não precisa decidir autoridade.

---

# 65. RENDER-63 — Interpolation

Exemplo:

```text
Network State A
       ↓
Interpolation
       ↓
Network State B
```

Isso evita movimentos "saltando" pela tela.

---

# 66. RENDER-64 — Animation LOD

Uma entidade distante pode reduzir:

```text
bone updates
animation rate
particle count
```

Em vez de executar a mesma animação completa em milhares de NPCs.

---

# 67. RENDER-65 — Simulation ↔ Rendering LOD

Ter dois sistemas relacionados:

```text
Simulation LOD
```

e:

```text
Rendering LOD
```

Exemplo:

```text
NPC
near
→ FULL simulation + FULL rendering

medium
→ REGIONAL simulation + LOD rendering

far
→ ABSTRACT simulation + billboard/hidden
```

---

# 68. RENDER-66 — Resolution Scaling

Permitir:

```text
100%
90%
80%
70%
50%
```

ou resolução dinâmica.

O Renderer pode adaptar a carga conforme GPU time.

---

# 69. RENDER-67 — Dynamic Resolution

Conceito:

```text
GPU too slow
 ↓
reduce internal resolution
```

e depois:

```text
GPU has headroom
 ↓
increase resolution
```

---

# 70. RENDER-68 — Frame Pacing

Não basta FPS alto.

Precisamos de:

```text
stable frame time
```

Monitorar:

```text
CPU frame
GPU frame
frame variance
spikes
```

---

# 71. RENDER-69 — Profiling

Criar:

```text
Render Profiler
```

Exibindo:

```text
CPU Render
GPU Render
Shadow
Terrain
Entities
Particles
Water
Post Processing
UI
```

---

# 72. RENDER-70 — Debug Rendering

Modos:

```text
Wireframe
Normals
Depth
Light
AO
LOD
Culling
Collision
Chunk boundaries
Overdraw
Material IDs
```

Muito importante para desenvolvimento.

---

# 73. RENDER-71 — Screenshot / Capture

Ferramentas:

```text
nexora render screenshot
nexora render capture
nexora render frame-debug
```

Possibilidade futura de frame capture para comparar regressões gráficas.

---

# 74. RENDER-72 — Photo / Cinematic Mode

Depois da base pronta:

```text
free camera
FOV
DOF
exposure
time control
weather control
```

Isso pode virar uma ferramenta de criação dentro do jogo.

---

# 75. RENDER-73 — UI Renderer

A UI deve ser outro pipeline:

```text
World Renderer
     +
UI Renderer
```

Com suporte a:

```text
HUD
menus
inventory
map
crafting
debug
notifications
```

---

# 76. RENDER-74 — Text Rendering

Suporte a:

```text
fonts
glyph atlas
Unicode
fallback fonts
icons
rich text
```

Isso é importante para localização.

---

# 77. RENDER-75 — Accessibility Visual

Suporte a:

```text
color adjustments
UI scaling
contrast options
motion reduction
screen effects intensity
```

---

# 78. RENDER-76 — Mod Rendering API

Mods devem conseguir registrar:

```text
CustomRenderer
CustomMaterial
CustomShader
CustomParticle
CustomModel
CustomRenderPass
PostProcessEffect
```

Mas com sandbox/permissões bem definidas.

---

# 79. RENDER-77 — Official Content

O conteúdo oficial deve usar as mesmas APIs:

```text
NEXORA official
      =
public rendering API
```

Isso é importante para não criar uma API falsa que só mods podem usar.

---

# 80. RENDER-78 — Asset Pipeline

Antes de chegar no jogo:

```text
Source Asset
 ↓
Import
 ↓
Validation
 ↓
Optimization
 ↓
Compression
 ↓
Packaging
 ↓
Runtime Asset
```

Suporte a:

```text
models
textures
animations
audio hooks
materials
shaders
```

---

# 81. RENDER-79 — Asset Validation

Verificar:

```text
missing texture
invalid mesh
bad UV
unsupported material
shader failure
incorrect skeleton
excessive polygon count
```

---

# 82. RENDER-80 — Shader Compilation Pipeline

Shaders devem ser compilados previamente quando possível.

```text
source
 ↓
compile
 ↓
validate
 ↓
cache
 ↓
package
```

Evitar travamentos durante gameplay.

---

# 83. RENDER-81 — Shader Hot Reload

Durante desenvolvimento:

```text
edit shader
 ↓
reload
 ↓
see result
```

Sem reiniciar todo o jogo.

---

# 84. RENDER-82 — Graphics Settings

Menu:

```text
Resolution
VSync
FPS
View Distance
Simulation Distance
Shadow Quality
Lighting
Water
Particles
Clouds
Vegetation
Texture Quality
Anti-Aliasing
Post Processing
LOD
```

---

# 85. RENDER-83 — Presets

```text
Low
Medium
High
Ultra
Custom
```

Mas os presets devem apenas configurar variáveis.

Não devem criar caminhos especiais dentro do código.

---

# 86. RENDER-84 — Hardware Profiles

Detectar:

```text
GPU
VRAM
CPU
RAM
```

e sugerir uma configuração inicial.

---

# 87. RENDER-85 — Scalability

O NEXORA precisa conseguir rodar em máquinas diferentes.

Então:

```text
visual quality
≠
gameplay quality
```

Um PC fraco deve poder reduzir:

```text
shadows
view distance
particles
water effects
LOD
```

sem mudar a simulação do mundo.

---

# 88. RENDER-86 — World Scale

Especialmente importante porque o NEXORA possui:

```text
surface
deep world
far lands
beyondlands
dimensions
space
```

O Renderer precisa estar preparado para escalas muito diferentes:

```text
microscopic visual detail
     ↓
player
     ↓
city
     ↓
continent
     ↓
planet
     ↓
space
```

Isso exige LOD, floating origin e streaming desde cedo.

---

# 89. RENDER-87 — Dimension Renderer

Cada dimensão poderá definir:

```text
Sky Profile
Atmosphere Profile
Fog Profile
Lighting Profile
Post Process Profile
Water Profile
```

Assim uma dimensão pode parecer visualmente completamente diferente sem alterar o Renderer inteiro.

---

# 90. RENDER-88 — Environment Profiles

Criar algo como:

```text
EnvironmentProfile
├── sky
├── lighting
├── fog
├── weather
├── water
├── postProcess
└── ambient
```

Biome, dimensão e clima combinam esses parâmetros.

---

# 91. RENDER-89 — Biome Rendering

Biomas podem influenciar:

```text
grass color
foliage density
fog
sky tint
ambient light
particles
cloud appearance
water appearance
```

Sem o Renderer conhecer nomes específicos de biomas.

---

# 92. RENDER-90 — Seasonal Rendering

Temporadas podem alterar:

```text
foliage
snow
grass
lighting
weather
particles
```

Isso conecta o Renderer à simulação sem acoplamento direto.

---

# 93. RENDER-91 — Destruction Rendering

Quando alguma estrutura muda:

```text
World change
 ↓
Render update
```

E efeitos podem incluir:

```text
debris
dust
fracture
particles
temporary decals
```

---

# 94. RENDER-92 — Machine Rendering

Máquinas podem possuir:

```text
moving parts
lights
screens
rotating components
energy effects
pipes
fluids
```

A Machine API registra os dados; o Renderer apenas os representa.

---

# 95. RENDER-93 — Vehicle Rendering

Mesma ideia:

```text
Vehicle State
 ↓
Vehicle Renderer
```

com:

```text
wheels
tracks
doors
lights
engines
propellers
effects
```

---

# 96. RENDER-94 — Railway Rendering

Como ferrovias serão importantes:

```text
Track Renderer
Rail Mesh
Sleeper Mesh
Signal Renderer
Station Renderer
```

E trilhos distantes podem usar LOD muito agressivo.

---

# 97. RENDER-95 — Space Rendering

Para espaço:

```text
Starfield
Planet Rendering
Celestial Bodies
Nebula
Orbital Objects
Ships
```

Novamente usando o mesmo Renderer, mas com passes especializados.

---

# 98. RENDER-96 — Planet / Celestial LOD

Objeto celeste:

```text
near
→ detailed

medium
→ simplified

far
→ billboard / procedural sphere

very far
→ point/light representation
```

---

# 99. RENDER-97 — Procedural Planet Rendering

No futuro:

```text
planet
 ↓
procedural surface
 ↓
LOD
 ↓
atmosphere
 ↓
clouds
 ↓
surface
```

Isso pode ser muito importante para a parte espacial.

---

# 100. RENDER-98 — Performance Budgets

Cada frame deve possuir orçamento:

```text
CPU budget
GPU budget
VRAM budget
draw-call budget
particle budget
```

O sistema pode reduzir qualidade automaticamente quando necessário.

---

# 101. RENDER-99 — Render Scheduler

Algo como:

```text
Critical
High
Normal
Low
Background
```

Pode priorizar:

```text
chunk perto
→ Critical

NPC perto
→ High

vegetação distante
→ Low

cosmetic effect distante
→ Background
```

---

# 102. RENDER-100 — Validation

Testar:

```text
1 chunk
10 chunks
100 chunks
1.000 chunks
10.000 visible instances
1.000 entities
10.000 entities
large city
large cave
deep ocean
Far Lands
dimension transition
```

E medir:

```text
FPS
frame time
GPU time
CPU time
VRAM
draw calls
triangles
```

---

# 103. Estrutura de projeto

Eu deixaria algo parecido com:

```text
renderer/
│
├── core/
│   ├── device/
│   ├── context/
│   ├── frame/
│   ├── command/
│   └── resources/
│
├── pipeline/
│   ├── graph/
│   ├── passes/
│   ├── queues/
│   └── synchronization/
│
├── camera/
├── world/
├── voxel/
│   ├── meshing/
│   ├── chunks/
│   ├── terrain/
│   └── lighting/
│
├── entities/
├── models/
├── animation/
├── materials/
├── shaders/
├── lighting/
├── shadows/
├── water/
├── atmosphere/
├── sky/
├── weather/
├── vegetation/
├── structures/
├── vehicles/
├── particles/
├── postprocess/
├── lod/
├── culling/
├── streaming/
├── ui/
├── debug/
├── profiling/
├── assets/
└── api/
```

# 104. Ordem de implementação

Eu **não** começaria pelo renderer futurista.

A ordem seria:

```text
RENDER-0 Core
RENDER-1 Graphics Backend
RENDER-2 Device
RENDER-3 Frame
RENDER-4 Camera
RENDER-5 Basic Shader
RENDER-6 Basic Mesh
RENDER-7 Voxel Renderer
RENDER-8 Chunk Meshing
RENDER-9 Chunk Rendering
RENDER-10 Texture/Material
RENDER-11 Lighting
RENDER-12 Shadows
RENDER-13 Entity Renderer
RENDER-14 Animation
RENDER-15 Frustum Culling
RENDER-16 Chunk Streaming
RENDER-17 LOD
RENDER-18 Water
RENDER-19 Sky/Atmosphere
RENDER-20 Particles
RENDER-21 Post Processing
RENDER-22 Vegetation
RENDER-23 Structures
RENDER-24 Vehicles
RENDER-25 Space
RENDER-26 Debug/Profiler
RENDER-27 Mod Rendering API
```

# 105. Primeiro Vertical Slice do Renderer

O primeiro objetivo deveria ser absurdamente simples:

```text
NEXORA
 ↓
Window
 ↓
Graphics Device
 ↓
Camera
 ↓
World
 ↓
Chunk
 ↓
Voxel Mesh
 ↓
Texture
 ↓
Shader
 ↓
Lighting
 ↓
Player
 ↓
Render Frame
```

Resultado:

```text
abre o jogo
→ aparece um terreno voxel
→ câmera funciona
→ jogador anda
→ chunks entram/saem
→ blocos podem mudar
→ iluminação atualiza
→ mundo salva
```

Só depois entrariam:

```text
água
sombras avançadas
vegetação
cidades
veículos
clima
atmosfera
dimensões
espaço
```

### A arquitetura final ficaria assim

```text
                 NEXORA
                    │
             ┌──────┴──────┐
             │    WORLD    │
             └──────┬──────┘
                    │
             RENDER EXTRACTION
                    │
             ┌──────┴──────┐
             │ RENDER WORLD│
             └──────┬──────┘
                    │
       ┌────────────┼─────────────┐
       │            │             │
   VISIBILITY    GEOMETRY      LIGHTING
       │            │             │
    CULLING      VOXELS        SHADOWS
    LOD          ENTITIES      ATMOSPHERE
    STREAMING    WATER         GI
       │            │             │
       └────────────┼─────────────┘
                    │
               RENDER GRAPH
                    │
                GPU PIPELINE
                    │
             POST PROCESSING
                    │
                   UI
                    │
                 DISPLAY
```

E a regra mais importante do Renderer seria:

> **O Renderer não é o mundo. Ele é a interpretação visual do mundo.**

Isso é o que permite que o mesmo NEXORA tenha **um mundo voxel profundo, cidades com milhares de NPCs, oceanos, cavernas gigantes, Far Lands, 16 dimensões e espaço** sem precisar reescrever o motor gráfico para cada sistema.
