Sim. **Chunk & Voxel Engine** é outro sistema que precisa ser extremamente detalhado no NEXORA, porque ele fica no meio de quase tudo:

```text id="2l6h9m"
WORLDGEN
   ↓
CHUNK / VOXEL ENGINE
   ↓
PHYSICS
   ↓
RENDERER
   ↓
SIMULATION
   ↓
SAVE / NETWORK
```

Ele não deve saber que existe “Mekanism”, “aldeão”, “dragão” ou qualquer conteúdo específico. Ele precisa fornecer a infraestrutura espacial sobre a qual esses sistemas trabalham.

# NEXORA — CHUNK & VOXEL ENGINE MASTER PLAN

## 1. Objetivo

O sistema precisa responder a cinco perguntas:

```text id="v1z7ud"
Onde existe o mundo?
O que existe em cada posição?
Como carregamos esse mundo?
Como modificamos esse mundo?
Como salvamos/transmitimos esse mundo?
```

Arquitetura:

```text id="5b5m4n"
                    VOXEL ENGINE
                         │
          ┌──────────────┼──────────────┐
          │              │              │
       WORLD MAP      CHUNKS          CELLS
          │              │              │
       Regions        Sections         Voxels
          │              │              │
          └──────────────┼──────────────┘
                         │
          ┌──────────────┼──────────────┐
          │              │              │
       STORAGE        STREAMING       ACCESS
          │              │              │
        Save          Load/Unload      Queries
          │              │              │
          └──────────────┼──────────────┘
                         │
              WORLD RUNTIME / WORLDGEN
```

---

# 2. CHUNK-0 — World Coordinate System

Primeiro precisamos definir como o espaço é representado.

Uma posição deve poder ser representada por:

```text id="6k38l9"
WorldPosition
├── x
├── y
└── z
```

Mas internamente não queremos trabalhar sempre com números gigantes.

Criar:

```text id="fn5q8a"
WorldCoordinate
ChunkCoordinate
SectionCoordinate
LocalVoxelCoordinate
```

Exemplo conceitual:

```text id="qb6lbr"
World Position
      ↓
Chunk Position
      ↓
Section
      ↓
Local Voxel
```

---

# 3. CHUNK-1 — Chunk Dimensions

Precisamos escolher uma unidade padrão de chunk.

Eu deixaria isso como configuração arquitetural, mas começaria com algo como:

```text id="rh7m5t"
32 × 32 × 32
```

ou uma divisão vertical independente.

Importante: **não deixar o código inteiro assumir que o chunk sempre possui o mesmo tamanho**.

Criar:

```text id="hzn3gr"
ChunkShape
ChunkDimension
ChunkLayout
```

Assim o engine pode evoluir.

---

# 4. CHUNK-2 — Vertical Sections

Por causa dos:

```text id="7obw8y"
+1920
   ↓
surface
   ↓
-1920
```

eu não colocaria um chunk gigantesco inteiro em uma única estrutura.

Usaria seções:

```text id="q5g9g4"
Chunk
├── Section
├── Section
├── Section
├── ...
└── Section
```

Por exemplo:

```text id="2h8jnd"
Chunk
   │
   ├── Section Y=0
   ├── Section Y=1
   ├── Section Y=2
   └── ...
```

Isso permite carregar apenas partes relevantes.

---

# 5. CHUNK-3 — Voxel Cell

A célula precisa ser compacta.

Não:

```text id="0g1hqr"
new BlockObject()
```

para cada posição.

Preferir:

```text id="ej0j11"
BlockStateID
```

ou estrutura compacta equivalente.

Conceito:

```text id="j0g83b"
Voxel
├── blockState
├── metadata
└── optional data reference
```

---

# 6. CHUNK-4 — Block Definition × Block State

Separar claramente:

```text id="y4o6if"
BlockDefinition
```

de:

```text id="g5mk75"
BlockState
```

Exemplo:

```text id="v5eym9"
BlockDefinition
→ stone

BlockState
→ stone
```

ou:

```text id="r8e3a4"
BlockDefinition
→ log

BlockState
→ oak_log + axis=X
```

O voxel armazena o **state**, não uma cópia da definição completa.

---

# 7. CHUNK-5 — Palette System

Chunks normalmente não precisam de IDs gigantescos.

Em uma região cheia de:

```text id="z1y2nf"
stone
stone
stone
stone
```

usar uma paleta:

```text id="a8tkm1"
Palette
├── 0 = air
├── 1 = stone
├── 2 = dirt
├── 3 = grass
```

E os voxels guardam índices compactos.

---

# 8. CHUNK-6 — Palette Compression

Quando aparecem poucos estados:

```text id="c7w3ob"
bits por voxel ↓
```

Quando aparecem muitos:

```text id="5y3e3d"
palette expand
```

Possíveis modos:

```text id="7snk2p"
Single Value
Indirect Palette
Direct IDs
```

Isso economiza memória em regiões homogêneas.

---

# 9. CHUNK-7 — Air Optimization

O ar não deve custar o mesmo que um bloco.

Possíveis otimizações:

```text id="8lvazn"
all-air section
single-state section
compressed empty region
```

Uma section totalmente vazia pode ser representada apenas como:

```text id="m7z55x"
AIR
```

sem armazenar milhões de entradas inúteis.

---

# 10. CHUNK-8 — Sparse Data

Algumas áreas serão muito vazias.

Exemplo:

```text id="0z9m83"
caverna gigante
espaço
Void
```

Então podemos ter representação esparsa quando for vantajoso.

Arquitetura:

```text id="5fk8tm"
Dense Storage
Sparse Storage
```

e o chunk escolhe conforme a densidade.

---

# 11. CHUNK-9 — Block Metadata

Nem todo bloco precisa de dados extras.

Mas alguns podem precisar:

```text id="8yhh1p"
orientation
variant
waterlogged
growth stage
connection state
```

Dados maiores devem ficar separados:

```text id="q2igyw"
Voxel
 ↓
BlockState
 ↓
BlockEntity/Data
```

---

# 12. CHUNK-10 — Block Entities

Alguns blocos possuem estado complexo.

Exemplo:

```text id="vq5qsa"
Machine
Chest
Storage
Controller
Reactor
Crafting Station
```

Não colocar isso dentro do array bruto de voxels.

Usar:

```text id="c6tpjs"
BlockEntity
├── position
├── type
└── state
```

---

# 13. CHUNK-11 — Chunk State

Cada chunk precisa de estado de runtime.

```text id="x1tqca"
UNLOADED
REQUESTED
LOADING
GENERATING
GENERATED
LOADED
ACTIVE
INACTIVE
UNLOADING
SAVED
ERROR
```

---

# 14. CHUNK-12 — Chunk Lifecycle

Fluxo:

```text id="teh2g9"
Requested
 ↓
Load
 ↓
Generate if absent
 ↓
Populate
 ↓
Activate
 ↓
Simulate
 ↓
Deactivate
 ↓
Save
 ↓
Unload
```

---

# 15. CHUNK-13 — Chunk Manager

Criar:

```text id="o6s02o"
ChunkManager
```

Responsável por:

```text id="41f85e"
request
load
generate
activate
deactivate
save
unload
```

Mas ele não deve ser responsável por WorldGen.

---

# 16. CHUNK-14 — Chunk Streaming

Baseado em distância/prioridade.

```text id="65xjy9"
Player
 ↓
Streaming Manager
 ↓
Requested Chunks
```

Prioridades:

```text id="v4tkx9"
Current
Adjacent
Near
Far
Background
```

---

# 17. CHUNK-15 — Multiple Radii

Separar:

```text id="j5n4dr"
Render Distance
Simulation Distance
Generation Distance
Persistence Distance
Physics Distance
```

Exemplo:

```text id="1yz6ac"
Render
→ 24 chunks

Simulation
→ 12

Physics
→ 8

Generation
→ 16

Persistence
→ 32
```

Os valores seriam configuráveis.

---

# 18. CHUNK-16 — Priority Queue

Chunks devem entrar em uma fila.

```text id="axqkqf"
ChunkRequest
├── position
├── priority
├── reason
├── requester
└── deadline
```

Motivos:

```text id="rvgh6j"
player movement
rendering
physics
worldgen
AI
vehicle
network
save
```

---

# 19. CHUNK-17 — Predictive Loading

O sistema não deve esperar o jogador chegar.

Se ele está indo para:

```text id="r3i3o6"
X + direction
```

podemos antecipar chunks nessa direção.

```text id="g4hm4o"
movement prediction
 ↓
prefetch
 ↓
generation/load
```

---

# 20. CHUNK-18 — Chunk Cache

Criar diferentes caches:

```text id="cvbx84"
RAM Cache
Disk Cache
GPU Mesh Cache
Generation Cache
```

---

# 21. CHUNK-19 — Memory Budget

Definir:

```text id="1x6e2k"
max RAM
max chunk count
max active chunks
max pending generation
```

Quando atingir limite:

```text id="5j5sa6"
evict low priority chunks
```

---

# 22. CHUNK-20 — Chunk Unloading

Não apagar o mundo.

Apenas:

```text id="wl9aeg"
runtime state
 ↓
serialize
 ↓
disk
 ↓
remove RAM
```

Ao voltar:

```text id="7x1wbe"
disk
 ↓
RAM
 ↓
active
```

---

# 23. CHUNK-21 — Chunk Serialization

Definir formato versionado.

```text id="yi8wyw"
ChunkFile
├── version
├── coordinates
├── sections
├── block entities
├── entities
├── metadata
└── custom data
```

---

# 24. CHUNK-22 — Compression

Separar compressão física da estrutura lógica.

```text id="rxg0cp"
Chunk Data
 ↓
Serializer
 ↓
Compressor
 ↓
Disk
```

Permitir evoluir a técnica de compressão sem quebrar o formato lógico.

---

# 25. CHUNK-23 — Checksums / Integrity

Cada chunk persistido pode possuir:

```text id="1j43dm"
checksum
version
size
timestamp
```

Para detectar corrupção.

---

# 26. CHUNK-24 — Crash Recovery

O jogo não pode corromper o mundo inteiro porque fechou no momento errado.

Arquitetura:

```text id="69jjmm"
write temporary
 ↓
flush
 ↓
validate
 ↓
atomic replace
```

Ou equivalente seguro para a plataforma alvo.

---

# 27. CHUNK-25 — Region Files

Milhões de arquivos seriam ruins.

Agrupar chunks:

```text id="2tb3r0"
World
└── Regions
    ├── Region 0
    ├── Region 1
    └── ...
```

Cada Region contém diversos chunks.

---

# 28. CHUNK-26 — Region Cache

O runtime pode manter regiões recentes em cache.

Isso ajuda:

```text id="b7q8g4"
train
vehicle
player
camera
```

que atravessam várias fronteiras de chunk.

---

# 29. CHUNK-27 — Chunk Borders

Uma alteração em:

```text id="6vb3af"
x = max
```

pode afetar o chunk vizinho.

Então:

```text id="k8tzcj"
Chunk A changed
 ↓
Neighbor invalidation
 ↓
Chunk B update
```

Muito importante para:

```text id="6z9q8u"
meshing
lighting
fluid
physics
```

---

# 30. CHUNK-28 — Neighbor Access

Criar uma API:

```text id="nrhpq6"
getBlock(x,y,z)
```

que saiba atravessar automaticamente a fronteira de chunks.

Assim o consumidor não precisa manualmente resolver:

```text id="o0j8xj"
"estou em x=31, preciso do chunk seguinte"
```

---

# 31. CHUNK-29 — Fast Voxel Access

Operações extremamente frequentes:

```text id="b3egvy"
getBlock
setBlock
getState
hasBlock
isAir
```

precisam ser altamente otimizadas.

---

# 32. CHUNK-30 — Batch Access

Em vez de:

```text id="3wae3q"
getBlock()
getBlock()
getBlock()
...
```

permitir:

```text id="a44u6r"
getRegion
getSlice
getVolume
```

Útil para:

```text id="js4ym4"
meshing
worldgen
lighting
physics
AI
```

---

# 33. CHUNK-31 — Mutation API

Criar alterações transacionais:

```text id="5jvrb8"
WorldEditTransaction
```

Exemplo:

```text id="g6bq7j"
set block
set block
remove block
 ↓
validate
 ↓
commit
```

Muito útil para construções e operações grandes.

---

# 34. CHUNK-32 — Atomic Changes

Sistemas como estruturas ou geração podem precisar atualizar milhares de blocos.

Queremos:

```text id="gw55j9"
prepare
 ↓
validate
 ↓
commit
 ↓
events
```

Em vez de deixar metade da estrutura aplicada.

---

# 35. CHUNK-33 — Change Tracking

Cada chunk deve saber o que mudou.

```text id="1f8w8c"
Dirty
```

E, idealmente:

```text id="tdnac8"
dirty sections
dirty blocks
dirty block entities
```

Não salvar tudo sempre.

---

# 36. CHUNK-34 — Change Journal

Para operações mais críticas:

```text id="t9z7cb"
Change
├── position
├── old state
├── new state
├── source
└── timestamp
```

Isso pode alimentar:

```text id="9lx2o5"
history
undo
debug
multiplayer
```

---

# 37. CHUNK-35 — Versioned World Data

O chunk deve conhecer:

```text id="z6ib4a"
world version
chunk data version
generator version
block registry version
```

Isso é essencial para updates.

---

# 38. CHUNK-36 — Migration

Ao abrir mundo antigo:

```text id="1f35rk"
Old Version
 ↓
Migration
 ↓
New Version
```

Exemplo:

```text id="5x9r9g"
block ID changed
 ↓
registry remap
 ↓
save updated
```

---

# 39. CHUNK-37 — WorldGen Interface

WorldGen deve falar com o Chunk Engine através de uma interface.

```text id="6f3gnj"
ChunkGenerationContext
```

O WorldGen produz:

```text id="l5x0vj"
terrain
biome data
caves
ores
structures
```

O Chunk Engine apenas armazena/aplica.

---

# 40. CHUNK-38 — Generation Stages

Suportar etapas:

```text id="4upmwp"
BASE_TERRAIN
GEOLOGY
CAVES
HYDROLOGY
BIOMES
ORES
VEGETATION
STRUCTURES
SETTLEMENTS
FINALIZATION
```

Isso combina diretamente com o WorldGen Master Plan.

---

# 41. CHUNK-39 — Partial Generation

Nem toda tarefa precisa estar pronta para o chunk aparecer.

Possível estado:

```text id="yce3w6"
terrain ready
but structures pending
```

Isso facilita streaming.

---

# 42. CHUNK-40 — Generation Dependencies

Algumas gerações dependem de vizinhos.

Criar:

```text id="mjrmo2"
GenerationDependency
```

Exemplo:

```text id="jpq7fd"
river continuity
biome edge
structure placement
ore veins
```

---

# 43. CHUNK-41 — Seamless Borders

O mundo não pode gerar:

```text id="8vx1ml"
chunk A → montanha
chunk B → parede artificial
```

As funções de geração precisam ser baseadas em coordenadas globais e contexto suficiente.

---

# 44. CHUNK-42 — Cross-Chunk Structures

Estruturas maiores:

```text id="0vcvgi"
cidade
montanha
ponte
ferrovia
mega caverna
```

podem cruzar centenas de chunks.

Precisamos de:

```text id="e8s8s4"
StructureInstance
StructureBounds
ChunkStructureReference
```

---

# 45. CHUNK-43 — Cross-Chunk Entities

Entidades também podem atravessar chunks:

```text id="i3zyf2"
Player
Train
Mob
Vehicle
Projectile
```

Nunca duplicar uma entidade apenas porque ela mudou de chunk.

---

# 46. CHUNK-44 — Entity Ownership

Criar uma noção de:

```text id="3bax2y"
Entity Home Chunk
```

ou região de gerenciamento.

A posição física pode mudar sem reescrever toda a identidade da entidade.

---

# 47. CHUNK-45 — Tick Management

Cada chunk possui:

```text id="w7y0or"
ACTIVE
TICKED
PASSIVE
```

Perto:

```text id="n7q7yx"
full simulation
```

Longe:

```text id="x4h7m9"
low-frequency
```

Muito longe:

```text id="p5u5fq"
abstract simulation
```

---

# 48. CHUNK-46 — Simulation LOD Integration

Integrar com o sistema que já definimos:

```text id="h8pd09"
Chunk
 ↓
Region
 ↓
World Simulation LOD
```

Um chunk distante pode não precisar executar cada entidade individual.

---

# 49. CHUNK-47 — Physics Integration

Physics pergunta:

```text id="3x08wb"
collision around position
```

Chunk Engine responde:

```text id="cxpq0w"
voxel shapes
```

Criar:

```text id="avqp19"
VoxelCollisionView
```

Não deixar Física ler diretamente o armazenamento interno do chunk.

---

# 50. CHUNK-48 — Renderer Integration

Renderer solicita:

```text id="0co7cx"
renderable voxel data
```

Chunk fornece:

```text id="3x2l4d"
ChunkRenderSnapshot
```

ou dados adequados para meshing.

---

# 51. CHUNK-49 — Mesh Dirty State

Cada chunk pode possuir:

```text id="qn1dvt"
MESH_CLEAN
MESH_DIRTY
MESHING
MESH_READY
```

Quando blocos mudam:

```text id="gt4l6w"
Voxel mutation
 ↓
dirty section
 ↓
mesh rebuild
```

---

# 52. CHUNK-50 — Lighting Integration

Alteração:

```text id="cdj7a4"
torch removed
```

gera:

```text id="cgtm3r"
light update
```

E pode atravessar fronteiras de chunks.

---

# 53. CHUNK-51 — Fluid Integration

Fluxo:

```text id="czfyyk"
Voxel change
 ↓
Fluid System notices
 ↓
flow update
 ↓
neighbor chunks
```

O Chunk Engine apenas oferece acesso seguro ao estado.

---

# 54. CHUNK-52 — Red/Automation-Like Updates

Para máquinas/automation:

```text id="8x1i5b"
block changed
 ↓
neighbor notification
 ↓
machine/network update
```

Mas a lógica da automação permanece fora do engine.

---

# 55. CHUNK-53 — Height Data

Para acelerar determinadas consultas, armazenar caches:

```text id="ms3rhv"
heightmap
surface height
opaque height
fluid height
```

Não usar como fonte definitiva do mundo, apenas como aceleração.

---

# 56. CHUNK-54 — Biome Storage

O chunk precisa armazenar/acessar dados de bioma.

Não necessariamente um biome ID por bloco.

Pode usar:

```text id="mp37iy"
Biome Palette
Biome Sampling
Biome Volume
```

dependendo da resolução necessária.

---

# 57. CHUNK-55 — Multi-Scale Voxel Metadata

Nem toda informação precisa da resolução 1 bloco.

Podemos ter:

```text id="7pmwrh"
Voxel scale
Chunk scale
Region scale
World scale
```

Exemplo:

```text id="1t4b2a"
temperature
→ regional field

block
→ voxel

population
→ settlement/region
```

---

# 58. CHUNK-56 — Region Metadata

Regiões podem guardar:

```text id="2j08ba"
biome statistics
population summary
structure index
resource summary
simulation state
```

Isso facilita simulações distantes.

---

# 59. CHUNK-57 — Spatial Queries

API:

```text id="w6r8a5"
queryBox
querySphere
queryChunk
queryRegion
queryBlocks
```

Usada por:

```text id="r5fry3"
AI
Physics
Renderer
WorldGen
Players
Structures
```

---

# 60. CHUNK-58 — Block Queries

Exemplos:

```text id="6t3oxm"
findNearestBlock
findBlocks
countBlocks
sampleBlocks
```

Útil para:

```text id="x2llm9"
AI
resource detection
debug
world analysis
```

---

# 61. CHUNK-59 — Ray/Voxel Traversal

Implementar traversal eficiente de voxels.

```text id="qrw8i4"
Ray
 ↓
Voxel traversal
 ↓
first hit
```

Isso alimenta:

```text id="7o5p6n"
interaction
mining
building
visibility queries
physics
```

---

# 62. CHUNK-60 — Block Placement Validation

Antes de colocar:

```text id="v3vbhj"
position
 ↓
world bounds
 ↓
permission
 ↓
collision
 ↓
placement rules
 ↓
commit
```

As regras de gameplay ficam no Block/Build API.

---

# 63. CHUNK-61 — Large Build Operations

O engine deve suportar operações como:

```text id="99tj7h"
fill
replace
copy
paste
generate structure
```

Mas isso deve ser uma API acima do armazenamento básico.

---

# 64. CHUNK-62 — Parallelism

Possíveis tarefas paralelas:

```text id="0j9iq3"
chunk load
chunk save
generation
meshing
compression
lighting
```

Nunca permitir corrida de dados sem controle.

---

# 65. CHUNK-63 — Thread Safety

Criar regras claras:

```text id="rfjhvi"
Read-only access
Write access
Exclusive mutation
Snapshot access
```

Não permitir cinco sistemas alterando o mesmo chunk simultaneamente sem coordenação.

---

# 66. CHUNK-64 — Snapshots

Para Renderer/Physics/Network:

```text id="x4ly5c"
Live World
    ↓
Snapshot
    ↓
consumer
```

O snapshot evita leitura inconsistente enquanto o mundo muda.

---

# 67. CHUNK-65 — Copy-on-Write

Quando útil:

```text id="9x2i4v"
Snapshot
 ↓
shared data
 ↓
write
 ↓
copy changed section
```

Pode reduzir custo de snapshots.

---

# 68. CHUNK-66 — Network Serialization

No multiplayer não queremos enviar:

```text id="98uywz"
chunk inteiro
```

sempre.

Criar:

```text id="7z83y0"
ChunkDelta
```

Exemplo:

```text id="kp8zqo"
block X changed
block Y changed
section Z updated
```

---

# 69. CHUNK-67 — Network Streaming

Servidor:

```text id="ndv6kd"
Player
 ↓
relevant chunks
 ↓
send
```

Também:

```text id="x6w2tj"
prioritize near chunks
```

---

# 70. CHUNK-68 — Client Prediction

Cliente pode gerar visualmente certas mudanças enquanto aguarda servidor, mas o servidor continua sendo a autoridade sobre estado compartilhado.

---

# 71. CHUNK-69 — Chunk Security

Servidor precisa validar:

```text id="f8c1p0"
chunk requests
block changes
entity interactions
data
```

Um cliente não pode simplesmente declarar:

```text id="q33t4m"
"esse chunk agora contém isso"
```

---

# 72. CHUNK-70 — World Border / Infinite Coordinates

Como haverá Far Lands e Beyondlands, precisamos separar:

```text id="c1r7tw"
World Reach
Generation Reach
Coordinate Limit
Dimension Limit
```

Não confundir:

```text id="xwm3cc"
Far Lands
≠
world border
```

---

# 73. CHUNK-71 — Far Lands

A Chunk Engine deve ser neutra.

Ela precisa suportar coordenadas muito grandes:

```text id="c84z5d"
surface
 ↓
far lands
 ↓
beyondlands
```

Quem define a aparência é o WorldGen.

O Chunk Engine apenas precisa continuar:

```text id="1rpsm7"
addressing
loading
saving
streaming
```

---

# 74. CHUNK-72 — Beyondlands

Mesma ideia.

A transição:

```text id="p3b7na"
normal world
→ Far Lands
→ Beyondlands
```

é uma mudança de geração, não uma mudança na infraestrutura básica de chunks.

---

# 75. CHUNK-73 — Dimensions

Cada dimensão possui seu próprio espaço:

```text id="2rn5qk"
DimensionID
+
ChunkCoordinate
```

A chave global pode ser conceitualmente:

```text id="d5w0a9"
(dimension, x, y, z)
```

Assim dois mundos podem possuir o mesmo `x,y,z` sem conflito.

---

# 76. CHUNK-74 — Dimension-specific Chunk Rules

Cada dimensão pode definir:

```text id="mb8r4j"
chunk size
vertical range
storage policy
generation pipeline
simulation rules
```

Mas tudo através da API.

---

# 77. CHUNK-75 — Void

Abaixo do Bedrock:

```text id="ik3hbi"
Surface World
      ↓
Bedrock boundary
      ↓
Dimension transition
      ↓
Void Dimension
```

A Chunk Engine precisa aceitar isso como uma dimensão independente.

---

# 78. CHUNK-76 — Deep World

Para as 15 camadas subterrâneas:

```text id="1f8jce"
Depth
 ↓
Chunk/Section
 ↓
Biome
 ↓
Resources
 ↓
Civilization
```

A infraestrutura continua a mesma.

---

# 79. CHUNK-77 — Chunk Events

Eventos:

```text id="si6d2d"
ChunkRequested
ChunkLoaded
ChunkGenerated
ChunkActivated
ChunkChanged
ChunkSaved
ChunkUnloaded
ChunkCorrupted
```

Esses eventos alimentam os outros sistemas.

---

# 80. CHUNK-78 — Metrics

Monitorar:

```text id="ja5es4"
loaded chunks
active chunks
generation queue
load queue
save queue
memory usage
dirty chunks
mesh queue
average load time
generation time
```

---

# 81. CHUNK-79 — Debug Tools

Comandos conceituais:

```text id="zzfkn8"
nexora chunk info
nexora chunk load
nexora chunk unload
nexora chunk regenerate
nexora chunk inspect
nexora chunk stats
```

Visualizações:

```text id="3b1jhb"
chunk borders
section borders
generation stage
loading state
simulation LOD
mesh state
memory usage
```

---

# 82. CHUNK-80 — Corruption Tools

Algo como:

```text id="5qg0j6"
nexora chunk verify
nexora chunk repair
nexora region verify
```

Sempre com backups e segurança antes de ações destrutivas.

---

# 83. CHUNK-81 — Testing

Testes fundamentais:

```text id="z5eg8k"
chunk coordinates
negative coordinates
chunk boundaries
vertical sections
palette
serialization
compression
load/unload
dirty tracking
neighbor access
cross-chunk structures
```

---

# 84. CHUNK-82 — Extreme Tests

Especialmente para NEXORA:

```text id="rbth0a"
surface
deep underground
Y = -1920
Y = +1920
Far Lands
Beyondlands
dimension boundaries
large cities
mega caves
ocean
space
```

---

# 85. CHUNK-83 — Performance Tests

Cenários:

```text id="hz73uw"
1 chunk
100 chunks
1,000 chunks
10,000 chunks
100,000 stored chunks
```

Não necessariamente carregados simultaneamente — o teste mede armazenamento, streaming e escalabilidade.

---

# 86. CHUNK-84 — Determinism

Acesso ao mesmo mundo deve produzir:

```text id="k8v4ug"
same coordinates
+
same world state
=
same voxel result
```

E WorldGen:

```text id="f3r1sy"
same seed
+
same generator version
=
same generated chunk
```

---

# 87. CHUNK-85 — Versioned Generation

A chave real de geração deve considerar:

```text id="0e3d2c"
seed
world version
generator version
dimension
coordinates
```

Isso evita mundos mudarem silenciosamente após atualização.

---

# 88. CHUNK-86 — Reproducible Chunks

Desenvolvimento precisa conseguir dizer:

```text id="v2f0x1"
seed = X
dimension = Y
chunk = Z
```

e reproduzir o problema.

Extremamente importante para bugs do WorldGen.

---

# 89. CHUNK-87 — Chunk Test Seeds

Ter seeds especiais:

```text id="ppg9oj"
flat
mountain
ocean
cave
deep
Far Lands
structure-heavy
city-heavy
performance
```

---

# 90. CHUNK-88 — Tooling

Criar ferramentas fora do jogo:

```text id="p9u37x"
chunk viewer
chunk inspector
world validator
region inspector
palette analyzer
memory analyzer
```

---

# 91. CHUNK-89 — Mod API

Mods devem poder:

```text id="08qka3"
read voxel
write voxel
register block
register block entity
query chunks
register chunk generation stage
listen chunk events
```

Mas operações perigosas devem passar por permissões.

---

# 92. CHUNK-90 — Official Content

Conteúdo oficial usa exatamente a mesma API:

```text id="e2jfve"
NEXORA official
      ↓
Chunk/Voxel API

Community mod
      ↓
Chunk/Voxel API
```

Não queremos:

```text id="q3j8r7"
internal secret block API
```

---

# 93. CHUNK-91 — API Boundary

O Chunk Engine fornece:

```text id="pwm0un"
IWorldReader
IWorldWriter
IChunkReader
IChunkWriter
IVoxelAccessor
IChunkQuery
```

Isso protege o armazenamento interno.

---

# 94. CHUNK-92 — Data Ownership

Regra importante:

```text id="se8c26"
Voxel Engine
→ owns voxel storage

WorldGen
→ owns generation logic

Renderer
→ owns render representation

Physics
→ owns collision representation

Fluid
→ owns fluid simulation
```

Nenhum sistema deve "roubar" o estado interno dos outros.

---

# 95. CHUNK-93 — Render Snapshot

Quando Renderer precisar:

```text id="3cx5ms"
Chunk
 ↓
Voxel snapshot
 ↓
Mesher
 ↓
Render mesh
```

---

# 96. CHUNK-94 — Physics Snapshot

Da mesma forma:

```text id="2q7tq4"
Chunk
 ↓
collision snapshot
 ↓
Physics
```

---

# 97. CHUNK-95 — Network Snapshot

E:

```text id="k0k7jg"
Chunk
 ↓
network snapshot/delta
 ↓
Client
```

---

# 98. CHUNK-96 — Cache Invalidation

Sempre que algo mudar, sistemas interessados precisam receber a invalidação correta:

```text id="azslap"
Voxel Mutation
      ↓
Chunk dirty
      ├── Mesh invalidated
      ├── Light invalidated
      ├── Physics cache invalidated
      ├── Fluid cache invalidated
      └── Network delta created
```

Esse é um dos pontos centrais de toda a arquitetura.

---

# 99. CHUNK-97 — Dependency Graph

Em vez de todo mundo escutar tudo:

```text id="v8ms3n"
Voxel Change
 ↓
Change Dispatcher
 ├── Renderer
 ├── Physics
 ├── Lighting
 ├── Fluid
 └── Network
```

---

# 100. CHUNK-98 — Final Architecture

No final eu gostaria de chegar nisso:

```text id="rb7s21"
                         WORLD
                           │
                     WORLD RUNTIME
                           │
                   CHUNK / VOXEL ENGINE
                           │
          ┌────────────────┼────────────────┐
          │                │                │
       STORAGE          ACCESS          STREAMING
          │                │                │
       SAVE/LOAD       QUERIES          LOAD/UNLOAD
          │                │                │
          └────────────────┼────────────────┘
                           │
                  ┌────────┴────────┐
                  │                 │
               VOXELS            CHUNKS
                  │                 │
            Block States        Sections
            Block Entities      Metadata
            Biomes              Entities
            Light Data          Structures
                  │                 │
                  └────────┬────────┘
                           │
             ┌─────────────┼─────────────┐
             │             │             │
          RENDERER       PHYSICS       FLUIDS
             │             │             │
          meshing       collision       flow
             │             │             │
             └─────────────┼─────────────┘
                           │
                     SIMULATION
                           │
                CIVILIZATION / AI /
                 ECONOMY / WORLD
```

---

# 101. Ordem de implementação

Eu faria em fases:

```text id="o5or4l"
VOXEL-0 Coordinate System
VOXEL-1 Block State
VOXEL-2 Chunk
VOXEL-3 Sections
VOXEL-4 Palette
VOXEL-5 Voxel Access
VOXEL-6 Chunk Manager
VOXEL-7 Chunk Lifecycle
VOXEL-8 Streaming
VOXEL-9 Cache
VOXEL-10 Serialization
VOXEL-11 Compression
VOXEL-12 Region Storage
VOXEL-13 Dirty Tracking
VOXEL-14 Neighbor Access
VOXEL-15 WorldGen Integration
VOXEL-16 Renderer Integration
VOXEL-17 Physics Integration
VOXEL-18 Lighting Integration
VOXEL-19 Fluid Integration
VOXEL-20 Simulation LOD
VOXEL-21 Network Streaming
VOXEL-22 Migration
VOXEL-23 Debugging
VOXEL-24 Mod API
VOXEL-25 Stress Testing
```

# 102. Primeiro Vertical Slice

O primeiro teste do sistema deveria ser:

```text id="x0k5om"
Seed
 ↓
World
 ↓
Chunk Manager
 ↓
Generate Chunk
 ↓
Store Voxels
 ↓
Access Voxel
 ↓
Modify Voxel
 ↓
Mark Dirty
 ↓
Remesh
 ↓
Render
 ↓
Save
 ↓
Unload
 ↓
Reload
 ↓
Voxel survives
```

Depois:

```text id="v3j2p8"
100 chunks
 ↓
streaming
 ↓
generation
 ↓
rendering
 ↓
save/load
```

E finalmente:

```text id="u0v7am"
Player moves
 ↓
chunks stream
 ↓
physics follows
 ↓
renderer follows
 ↓
simulation follows
 ↓
world persists
```

### A regra mais importante desse sistema

> **Chunk é a unidade de armazenamento e streaming. Voxel é a unidade de mundo. Section é a unidade de processamento. Region é a unidade de persistência em escala.**

Essa separação combina muito bem com o NEXORA que você está planejando:

```text
Voxel
 ↓
Section
 ↓
Chunk
 ↓
Region
 ↓
World
 ↓
Dimension
```

E permite que o mundo vá de uma pequena caverna até **megaestruturas, cidades, oceanos, Deep World, Far Lands, Beyondlands e dimensões inteiras** usando a mesma infraestrutura.
