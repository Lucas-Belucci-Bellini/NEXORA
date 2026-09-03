Perfeito. Para o NEXORA, o **Animation System** deve ser tratado como infraestrutura de animação, não como parte do Player, Entity ou Renderer.

A regra central:

> **Animation System transforma estado e dados de animação em poses ao longo do tempo. Entity, Item, Block, Vehicle e outros sistemas fornecem o estado; Renderer apenas apresenta o resultado.**

# NEXORA — ANIMATION SYSTEM / ANIMATION API

## 1. Princípio central

```text
ANIMATION SYSTEM
├── ASSET / CLIP
├── SKELETON
├── RIG
├── POSE
├── TRACK
├── CONTROLLER
├── STATE MACHINE
├── BLEND
├── IK
├── PROCEDURAL ANIMATION
├── EVENTS / MARKERS
├── LAYERING
├── LOD
├── CACHE
├── SERIALIZATION
└── MOD API
```

Fluxo:

```text
ENTITY / ITEM / BLOCK / VEHICLE
              ↓
         STATE / INTENT
              ↓
       ANIMATION SYSTEM
              ↓
       POSE / TRANSFORM
              ↓
          RENDERER
```

---

# 2. O Animation System não é o Renderer

Separação:

```text
Animation
→ calcula pose

Renderer
→ desenha pose
```

Portanto:

```text
Renderer.playAnimation()
```

não deveria ser a arquitetura principal.

Melhor:

```text
AnimationController
        ↓
Pose
        ↓
Renderer
```

---

# 3. Não é exclusivo para personagens

Animation System deve funcionar para:

```text
Player
NPC
Mob
Animal
Monster
Vehicle
Machine
Weapon
Tool
Item
Block
Plant
Creature
Robot
Spaceship
UI element
```

---

# 4. Animation Object Model

Conceito:

```text
AnimationDefinition
AnimationClip
AnimationTrack
AnimationChannel
AnimationMarker
AnimationController
AnimationGraph
Pose
Skeleton
Bone
Rig
```

---

# 5. AnimationDefinition

Define uma animação.

```text
AnimationDefinition

id
duration
loopMode
tracks
markers
playbackRate
blendProfile
```

Exemplo:

```text
nexora:player_walk
duration = ...
loop = true
```

---

# 6. Animation ID

Usar Registry:

```text
nexora:player_walk
examplemod:dragon_fly
```

Isso conecta diretamente ao Registry System.

---

# 7. Animation Registry

Criar:

```text
AnimationRegistry
```

e:

```text
AnimationGraphRegistry
SkeletonRegistry
RigRegistry
```

---

# 8. Animation Clip

Um clip representa uma sequência temporal.

```text
AnimationClip
    duration
    tracks
    markers
```

---

# 9. Track

Track controla uma propriedade.

Exemplo:

```text
Bone.Head.rotation
Bone.RightArm.rotation
Root.position
```

---

# 10. Channel

Um track pode possuir canais:

```text
position
rotation
scale
weight
visibility
custom property
```

---

# 11. Keyframes

Exemplo:

```text
0.0s → rotation 0°
0.5s → rotation 30°
1.0s → rotation 0°
```

O Animation System interpola.

---

# 12. Interpolation

Suportar:

```text
STEP
LINEAR
CUBIC
BEZIER
QUATERNION_SLERP
CUSTOM
```

---

# 13. Rotation

Evitar Euler como representação interna principal.

Preferir:

```text
Quaternion
```

para evitar problemas de interpolação e gimbal lock.

---

# 14. Transform

Uma pose de osso pode conter:

```text
Transform
├── position
├── rotation
└── scale
```

---

# 15. Skeleton

Skeleton define a estrutura hierárquica.

```text
Skeleton
├── Root
│   ├── Spine
│   │   ├── Chest
│   │   │   ├── Head
│   │   │   ├── Arm_L
│   │   │   └── Arm_R
│   │   └── ...
│   └── LegRoot
```

---

# 16. Bone

Cada bone:

```text
Bone
├── id
├── parent
├── restTransform
├── inverseBind
└── constraints
```

---

# 17. Rest Pose

Skeleton precisa possuir:

```text
RestPose
```

que representa a configuração base.

---

# 18. Bind Pose

Para skinning:

```text
BindPose
```

e matriz inversa correspondente.

---

# 19. Rig

Rig conecta:

```text
Skeleton
+
Animation Data
+
Constraints
+
Controllers
```

---

# 20. Skeleton ≠ Rig

### Skeleton

Estrutura óssea.

### Rig

Sistema que manipula a estrutura.

Isso permite rigs diferentes usando a mesma estrutura conceitual.

---

# 21. Pose

Pose representa o estado atual.

```text
Pose
    transforms[]
```

Uma animação gera:

```text
Pose(t)
```

---

# 22. Local Pose

Transform relativo ao pai.

---

# 23. Global Pose

Transform acumulado na hierarquia.

```text
local transforms
 ↓
hierarchical solve
 ↓
global transforms
```

---

# 24. Pose Buffer

Para performance:

```text
PoseBuffer
```

reutilizável.

Evitar alocação a cada frame.

---

# 25. Animation Player

Componente simples:

```text
AnimationPlayer
```

controla:

```text
play
pause
stop
seek
speed
weight
loop
```

---

# 26. Animation State

```text
STOPPED
PLAYING
PAUSED
BLENDING
COMPLETED
```

---

# 27. Loop Modes

```text
ONCE
LOOP
PING_PONG
CLAMP
CUSTOM
```

---

# 28. Playback

Parâmetros:

```text
time
speed
direction
weight
```

---

# 29. Animation Controller

Para lógica mais complexa:

```text
AnimationController
```

recebe estado do objeto.

Exemplo:

```text
speed = 5
isGrounded = true
isSwimming = false
```

e decide animações.

---

# 30. Controller não deve conhecer gameplay demais

Evitar:

```text
if economyCollapsed()
```

dentro do Animation System.

Melhor:

```text
locomotionState = WALKING
```

ou:

```text
combatState = ATTACKING
```

---

# 31. Animation State Machine

Máquinas de estados:

```text
IDLE
 ↓
WALK
 ↓
RUN
 ↓
JUMP
 ↓
FALL
 ↓
LAND
 ↓
IDLE
```

---

# 32. State

Cada estado possui:

```text
animation
speed
blend
conditions
transitions
```

---

# 33. Transition

Exemplo:

```text
IDLE → WALK
```

quando:

```text
speed > threshold
```

---

# 34. Blend Time

Não cortar instantaneamente:

```text
Idle
 ↓ 0.2s blend
Walk
```

---

# 35. Transition Conditions

Suportar:

```text
boolean
float
integer
enum
trigger
parameter comparison
```

---

# 36. Parameters

Controller possui:

```text
speed
direction
verticalVelocity
grounded
combatState
healthState
```

Esses valores vêm dos sistemas de gameplay.

---

# 37. Triggers

Eventos momentâneos:

```text
attack
jump
hurt
land
interact
use
```

---

# 38. Animation Graph

Para sistemas complexos, usar grafo.

```text
AnimationGraph
├── Input
├── State Machine
├── Blend
├── IK
├── Layer
└── Output
```

---

# 39. Blend Tree

Permite:

```text
Idle
Walk
Run
Sprint
```

misturados conforme:

```text
speed
```

---

# 40. 1D Blend

```text
speed
0 → idle
5 → walk
10 → run
15 → sprint
```

---

# 41. 2D Blend

Exemplo:

```text
direction X
direction Y
```

para locomoção omnidirecional.

---

# 42. Additive Animation

Muito importante.

Base:

```text
walk
```

Additive:

```text
look
```

Resultado:

```text
walk + head look
```

---

# 43. Layering

Camadas:

```text
Base Locomotion
Upper Body Combat
Face
Equipment
Damage Reaction
```

---

# 44. Layer Mask

Uma animação pode afetar apenas:

```text
upper body
```

sem alterar:

```text
legs
```

---

# 45. Bone Mask

```text
BoneMask
```

define bones afetados.

---

# 46. Blend Modes

```text
OVERRIDE
ADDITIVE
MULTIPLY
```

com suporte apenas onde fizer sentido.

---

# 47. Procedural Animation

Nem tudo deve ser keyframe.

Exemplos:

```text
head tracking
foot placement
tail movement
breathing
weapon recoil
vehicle suspension
```

---

# 48. Procedural Layer

```text
Animation Graph
 ↓
keyframed pose
 ↓
procedural modifications
 ↓
final pose
```

---

# 49. IK

Inverse Kinematics:

```text
target
 ↓
solve
 ↓
bone chain
```

---

# 50. IK Examples

```text
Feet → terrain
Hands → weapon
Hands → ladder
Head → target
Tentacles → target
```

---

# 51. Foot IK

NPC andando em terreno irregular:

```text
terrain
 ↓
raycast
 ↓
foot target
 ↓
IK
```

Physics fornece consultas.

Animation resolve pose.

---

# 52. Look At

```text
LookAtConstraint
```

NPC pode olhar:

```text
player
object
sound source
enemy
friend
```

A percepção vem do AI System.

Animation apenas resolve a orientação.

---

# 53. Aim IK

Para armas:

```text
target
 ↓
aim constraint
 ↓
upper body pose
```

Combat fornece o alvo.

---

# 54. Constraints

Tipos:

```text
LookAt
Aim
Parent
CopyRotation
CopyPosition
LimitRotation
IK
TwoBoneIK
Custom
```

---

# 55. Constraint Order

A ordem importa.

Exemplo:

```text
Base Pose
 ↓
LookAt
 ↓
IK
 ↓
Limits
 ↓
Final Pose
```

---

# 56. Animation Markers

Uma animação pode conter:

```text
footstep
attack_hit
reload
sound
particle
camera_event
```

---

# 57. Marker

```text
AnimationMarker

name
time
payload
```

---

# 58. Marker Events

Quando o tempo passa pelo marker:

```text
AnimationMarkerReached
```

vai para o Event Bus.

---

# 59. Example

```text
Sword Swing
0.0
0.2 → attack_start
0.4 → hit
0.7 → attack_end
```

Combat decide o efeito real.

Animation só publica os marcadores.

---

# 60. Animation ≠ Combat

Isso evita um acoplamento ruim.

```text
Animation
→ "hit marker"

Combat
→ valida hit
```

---

# 61. Sound Integration

Animation pode publicar:

```text
FootstepMarker
```

Audio reproduz som.

---

# 62. Particle Integration

Marker:

```text
dust
```

Particle System gera efeito.

---

# 63. Camera Integration

Marker pode produzir:

```text
camera recoil
```

mas Camera System executa.

---

# 64. Gameplay Sync

Animação não deve ser autoridade para gameplay.

Errado:

```text
sword animation visually hits
→ automatically damage
```

Melhor:

```text
Combat attack state
→ Animation
```

e:

```text
Animation marker
→ Combat timing hint
```

Combat continua autoritativo.

---

# 65. Network

Servidor deve replicar **estado relevante**, não necessariamente cada bone.

---

# 66. Network Animation State

Exemplo:

```text
locomotion = RUNNING
combat = ATTACK_1
```

Cliente resolve a animação localmente.

---

# 67. Avoid Bone-by-Bone Networking

Não enviar:

```text
Head rotation
Arm rotation
Leg rotation
...
```

todo frame.

Seria muito caro.

---

# 68. Network Pose Exceptions

Para casos especiais:

```text
full synchronized pose
```

pode existir, mas como exceção.

---

# 69. Determinism

Animation visual não precisa ser perfeitamente determinística em cada plataforma.

Mas eventos gameplay associados precisam possuir autoridade clara.

---

# 70. Server Animation

Servidor pode executar apenas o mínimo necessário:

```text
state transitions
markers relevant to gameplay
```

Cliente calcula visual completo.

---

# 71. Client Prediction

Cliente pode prever:

```text
locomotion
animations
```

e reconciliar estado do servidor.

---

# 72. Animation LOD

NEXORA terá milhares de mobs.

Não podemos animar 100.000 criaturas plenamente.

Criar:

```text
FULL
REDUCED
SIMPLIFIED
ABSTRACT
```

---

# 73. Full Animation

Perto:

```text
full skeleton
IK
layers
facial animation
procedural
```

---

# 74. Reduced

Mais longe:

```text
basic skeleton
reduced update frequency
no expensive IK
```

---

# 75. Simplified

Muito longe:

```text
low-frequency pose
minimal animation
```

---

# 76. Abstract

Quando entidade vira:

```text
REGIONAL
ABSTRACT
```

não precisa mais de pose visual.

---

# 77. Distance-based Update

A frequência pode diminuir:

```text
near
→ 60 Hz

medium
→ 30 Hz

far
→ 10 Hz

very far
→ 2 Hz
```

Os valores reais serão calibrados em profiling.

---

# 78. Visibility Culling

Se não está visível:

```text
animation update
```

pode ser reduzida ou pausada dependendo do gameplay.

---

# 79. Gameplay Critical Animation

Algumas animações continuam logicamente importantes:

```text
attack timing
reload timing
vehicle mechanism
```

Mesmo fora da câmera.

Mas nesses casos o gameplay deve possuir timers próprios.

---

# 80. Animation Time vs Gameplay Time

Separar:

```text
Gameplay Timer
Animation Timer
```

Não deixar combate depender exclusivamente do frame visual.

---

# 81. Asset Streaming

Animações podem ser carregadas sob demanda:

```text
near player
 ↓
load clips
```

---

# 82. Animation Cache

Cache:

```text
AnimationClip
Skeleton
Rig
CompiledGraph
```

---

# 83. Compiled Animation Graph

Durante loading:

```text
Graph Data
 ↓
Compile
 ↓
Optimized Graph
```

---

# 84. Graph Compilation

Pode resolver antecipadamente:

```text
state transitions
bone indices
masks
references
```

---

# 85. Runtime Graph

Depois:

```text
parameters
 ↓
compiled graph
 ↓
pose
```

sem resolver IDs/strings repetidamente.

---

# 86. Bone Indices

Strings:

```text
"head"
```

só no load.

Runtime:

```text
boneIndex = 7
```

---

# 87. Animation Parameter IDs

Mesmo princípio:

```text
speed
```

vira:

```text
parameterId
```

no runtime.

---

# 88. Hot Path

Ideal:

```text
Runtime IDs
+
float arrays
+
pose buffers
```

e não objetos complexos por bone.

---

# 89. Memory Layout

Estruturas contíguas para:

```text
transforms
weights
keyframes
bone indices
```

---

# 90. Keyframe Compression

Para grandes bibliotecas:

```text
quantization
compressed rotations
delta positions
```

podem reduzir memória.

---

# 91. Animation Compression

No asset pipeline:

```text
raw animation
 ↓
compress
 ↓
runtime clip
```

---

# 92. Additive Compression

Additive clips podem ser compactados separadamente.

---

# 93. Retargeting

Uma função muito importante para milhares de criaturas:

```text
AnimationRetargeting
```

Uma animação pode ser adaptada para skeletons compatíveis.

---

# 94. Retarget Profile

```text
source skeleton
target skeleton
bone mappings
scale rules
```

---

# 95. Humanoid Retargeting

Um conjunto de animações pode ser reutilizado por:

```text
human
elf-like
robot
alien
NPC
```

desde que o perfil seja compatível.

---

# 96. Creature Retargeting

Outros rigs:

```text
quadruped
bird
insect
serpent
humanoid
```

podem possuir famílias de retargeting próprias.

---

# 97. Animation Libraries

Organizar:

```text
Locomotion
Combat
Interaction
Emotes
Damage
Death
Swimming
Climbing
Flying
Vehicle
```

---

# 98. Animation Tags

```text
#locomotion
#combat
#idle
#interaction
#facial
```

---

# 99. Animation Variants

Uma criatura pode ter:

```text
walk
walk_heavy
walk_injured
walk_cold
walk_hot
```

e o Controller escolher.

---

# 100. Species Animation Profile

Entity Definition pode referenciar:

```text
SpeciesAnimationProfile
```

que conecta:

```text
skeleton
rig
locomotion graph
combat graph
```

---

# 101. Entity Integration

Entity fornece:

```text
entity state
velocity
grounded
action
```

Animation transforma isso em pose.

---

# 102. Player Integration

Player fornece:

```text
movement
equipment
action
camera-relative orientation
```

Animation resolve:

```text
locomotion
body pose
equipment pose
```

---

# 103. Mob Integration

Mob AI fornece:

```text
behavior state
movement
combat state
```

Animation resolve apresentação.

---

# 104. NPC Integration

NPC pode possuir:

```text
profession animation profile
```

mas profissão continua sendo responsabilidade de Civilization/NPC System.

---

# 105. Vehicle Integration

Vehicles podem possuir:

```text
wheel rotation
suspension
turret
doors
tracks
thrusters
```

---

# 106. Machine Integration

Máquinas podem usar:

```text
pistons
belts
gears
arms
cooling fans
```

Animation apenas calcula movimento visual.

Machine System continua cuidando da lógica.

---

# 107. Block Integration

Blocos como portas:

```text
Block State
→ open
```

Animation:

```text
open transition
→ pose
```

---

# 108. Item Integration

Itens podem ter animação contextual:

```text
equip
swing
reload
charge
inspect
```

---

# 109. Weapon Integration

Tool/Weapon API fornece:

```text
attack mode
```

Animation resolve:

```text
attack pose
```

---

# 110. First Person Animation

Player pode possuir rig diferente:

```text
first-person arms
```

separado da terceira pessoa.

---

# 111. Third Person

```text
full character skeleton
```

---

# 112. Viewmodel

Criar:

```text
ViewModelRig
```

para primeira pessoa.

---

# 113. Camera-relative Animation

Alguns movimentos dependem da câmera:

```text
head
arms
weapon
```

Mas a pose final deve continuar dentro do Animation System.

---

# 114. Facial Animation

Sistema pode suportar:

```text
face bones
blend shapes
morph targets
```

---

# 115. Blend Shapes

Para:

```text
expressions
damage
speech
creature deformation
```

---

# 116. Facial Controller

Parâmetros:

```text
emotion
viseme
damage
attention
```

---

# 117. Visemes

Audio/Dialogue fornece:

```text
phoneme/viseme data
```

Animation converte para facial pose.

---

# 118. Speech Integration

Fluxo:

```text
Dialogue
 ↓
phoneme timing
 ↓
Animation
 ↓
facial pose
```

---

# 119. Dynamic Procedural Animation

NEXORA pode usar animação procedural para criaturas gigantes.

Exemplo:

```text
Giant creature
 ↓
terrain
 ↓
foot targets
 ↓
IK
 ↓
body stabilization
```

---

# 120. Physics Integration

Physics fornece:

```text
ground
contacts
velocity
constraints
```

Animation usa.

Não substituir Physics.

---

# 121. Animation-driven Motion

Cuidado com movimento baseado na animação.

Para personagens:

```text
gameplay movement
→ root motion optional
```

---

# 122. Root Motion

Pode existir:

```text
RootMotionProfile
```

Mas deve ser explícito.

---

# 123. Root Motion Modes

```text
OFF
VISUAL_ONLY
AUTHORITY
HYBRID
```

---

# 124. Multiplayer Root Motion

Nunca permitir que uma animação visual do cliente seja autoridade para mover o personagem sem validação.

---

# 125. Animation Events

Exemplos:

```text
AnimationStarted
AnimationStopped
AnimationCompleted
AnimationMarkerReached
AnimationStateChanged
```

---

# 126. Event Bus Integration

Animation publica eventos relevantes:

```text
AnimationMarkerReached
```

e pode ouvir:

```text
CombatStarted
EntityStateChanged
```

---

# 127. Event Flood Protection

Não publicar cada mudança de bone no Event Bus.

---

# 128. Animation Debugger

Comando:

```text
nexora animation inspect <entity>
```

mostrar:

```text
skeleton
current graph
state
clip
time
weight
layers
IK
LOD
```

---

# 129. Animation Graph Debugger

Visualização:

```text
Idle
 ↓
Locomotion Blend
 ↓
Combat Layer
 ↓
LookAt
 ↓
IK
 ↓
Output
```

---

# 130. Bone Debugger

Renderizar:

```text
bones
axes
constraints
IK targets
```

apenas em development mode.

---

# 131. Animation Profiler

Métricas:

```text
pose evaluation time
IK time
blend time
graph time
bone count
active animations
memory
cache hits
```

---

# 132. Worst-case Test

Testar:

```text
1,000 animated entities
10,000
100,000
```

com LOD.

---

# 133. 2,000+ Mob Types

Não criar animation code para cada espécie.

Usar:

```text
EntityDefinition
+
SpeciesAnimationProfile
+
AnimationGraph
+
Rig
+
Traits
```

---

# 134. Procedural Species

Uma criatura pode definir:

```text
number of limbs
limb chains
locomotion mode
gait
tail
wings
```

e o Animation System gera/adapta partes automaticamente.

---

# 135. Gait System

Para quadrúpedes:

```text
walk
trot
run
gallop
```

pode haver geração procedural.

---

# 136. Creature Traits

```text
biped
quadruped
flying
swimming
serpentine
multi-legged
```

---

# 137. Animation Template

Exemplo:

```text
humanoid_locomotion_template
quadruped_locomotion_template
flying_locomotion_template
```

---

# 138. Template Compilation

```text
template
+
species skeleton
+
traits
↓
compiled animation graph
```

Isso é muito importante para chegar em milhares de criaturas sem milhares de sistemas exclusivos.

---

# 139. Dynamic Animation

Algumas animações podem ser geradas em runtime:

```text
tail
tentacles
cloth-like secondary motion
procedural legs
```

---

# 140. Secondary Motion

Pode existir:

```text
spring bones
verlet-like secondary animation
```

mas deve ser visual, não física principal.

---

# 141. Physics vs Secondary Animation

```text
Physics
→ world interaction

Animation
→ visual secondary motion
```

---

# 142. Cloth

Cloth pode ser outro sistema.

Animation pode fornecer:

```text
base pose
```

e Cloth resolver deformação.

---

# 143. Animation Output

O resultado deve ser algo como:

```text
AnimationPose

rootTransform
boneTransforms[]
morphWeights[]
visibility
markers
```

---

# 144. Renderer Contract

Renderer recebe:

```text
AnimationPose
```

ou estrutura equivalente.

---

# 145. No Renderer Mutation

Renderer não deve modificar o Animation Graph.

---

# 146. Serialization

Persistir animações de conteúdo?

Sim, mas principalmente definições e graphs.

Estado temporário do player pode ser persistido quando necessário:

```text
current state
time
```

Mas não precisa persistir pose completa.

---

# 147. Save Policy

Exemplo:

```text
AnimationDefinition
→ REGISTRY

AnimationGraph
→ CONTENT

Current animation state
→ OPTIONAL PERSIST

Pose
→ DERIVE
```

---

# 148. Load

Depois do load:

```text
entity state
 ↓
animation controller
 ↓
graph re-evaluates
 ↓
pose rebuilt
```

---

# 149. Missing Animation

Se um mod sumir:

```text
MissingAnimation
```

ou:

```text
fallback animation
```

dependendo da criticidade.

---

# 150. Mod API

Mods podem registrar:

```text
AnimationClip
Skeleton
Rig
AnimationGraph
AnimationController
Constraint
AnimationTemplate
```

---

# 151. Official Content

Vanilla utiliza exatamente:

```text
Public Animation API
```

que os mods utilizam.

---

# 152. Asset Separation

Animation System referencia:

```text
animation data
skeleton
rig
```

Asset System administra arquivos.

---

# 153. Format

NEXORA pode aceitar um formato de animação próprio ou adaptadores de formatos externos.

A API interna não deve depender do formato do arquivo.

---

# 154. Import Pipeline

```text
External Animation
 ↓
Importer
 ↓
Normalized Animation Data
 ↓
Validation
 ↓
Compiled Runtime Data
```

---

# 155. Runtime Format

Depois do import:

```text
AnimationClipCompiled
```

otimizado para execução.

---

# 156. Versioning

Todos os assets devem possuir:

```text
animationVersion
schemaVersion
```

---

# 157. Migration

Se o formato mudar:

```text
old animation schema
 ↓
migration
 ↓
new schema
```

---

# 158. Validation

Verificar:

```text
missing bone
invalid parent
invalid quaternion
bad keyframe
negative duration
invalid transition
cycle
missing clip
```

---

# 159. Graph Cycle

State machines naturalmente podem possuir ciclos válidos.

Então distinguir:

```text
valid state transition cycle
```

de:

```text
compile-time graph dependency cycle
```

que pode ser inválido.

---

# 160. Infinite Transition Protection

Um graph não deve permitir:

```text
A → B → C → A
```

sem condições que avancem o estado.

---

# 161. Animation Time Budget

Cada entidade pode possuir um orçamento:

```text
animation budget
```

para decidir:

```text
full evaluation
reduced evaluation
skip
```

---

# 162. Priority

Entidades importantes:

```text
Player
Boss
Nearby NPC
```

têm maior prioridade.

---

# 163. Distance + Importance

LOD deve considerar:

```text
distance
visibility
gameplay importance
camera relevance
```

não apenas distância.

---

# 164. Animation Scheduler

Pode haver:

```text
AnimationScheduler
```

mas ele não deve substituir o Scheduler global.

Ele agenda avaliação de animação.

---

# 165. Batch Evaluation

Animar grupos:

```text
100 same species
```

pode usar dados compartilhados.

---

# 166. Shared Animation Graph

Vários mobs podem compartilhar:

```text
same graph
same clip
same skeleton
```

com estados individuais.

---

# 167. Immutable Assets

Assets compilados devem ser imutáveis no runtime.

---

# 168. Instance State

Cada entidade possui apenas:

```text
currentState
parameters
time
weights
```

não uma cópia do graph.

---

# 169. Huge Performance Gain

Isso reduz muito memória para:

```text
10,000 wolves
```

por exemplo:

```text
1 graph compartilhado
+
10,000 lightweight controller states
```

---

# 170. Pose Sharing

Em entidades distantes, poses semelhantes podem ser reutilizadas ou avaliadas em baixa frequência.

---

# 171. Crowd Animation

Para milhares de NPCs:

```text
crowd animation strategy
```

pode usar:

```text
shared locomotion
phase offsets
reduced bones
```

---

# 172. Animation Phase Offset

Evita:

```text
1.000 NPCs
```

andando exatamente sincronizados.

---

# 173. Randomized Phase

Pequena variação:

```text
walk cycle phase
```

mas deterministicamente derivada de entity ID quando necessário.

---

# 174. Deterministic Variation

```text
EntityID
 ↓
stable random seed
 ↓
animation phase
```

---

# 175. Animation Style

Animation System pode suportar perfis:

```text
REALISTIC
STYLIZED
MECHANICAL
PROCEDURAL
CUSTOM
```

Mas estilo é um perfil de dados, não lógica espalhada.

---

# 176. Block Animation

Exemplo:

```text
Door
 ↓
BlockState open
 ↓
Animation Graph
 ↓
rotation 0→90°
```

---

# 177. Machine Animation

```text
MachineState
 ↓
processing
 ↓
rotor animation
```

---

# 178. Vehicle Animation

```text
VehicleState
 ↓
wheel speed
 ↓
wheel rotation
```

---

# 179. Player Animation

```text
PlayerState
 ↓
locomotion graph
 ↓
equipment layer
 ↓
combat layer
 ↓
IK
```

---

# 180. Mob Animation

```text
MobState
 ↓
Species Profile
 ↓
Locomotion
 ↓
Behavior Layer
 ↓
Combat
```

---

# 181. NPC Animation

```text
NPCState
 ↓
locomotion
 ↓
profession pose
 ↓
social interaction
```

---

# 182. Animation + AI

AI não deveria controlar bones diretamente.

AI diz:

```text
behavior = ALERT
```

Animation traduz:

```text
alert pose
```

---

# 183. Animation + Civilization

NPC profissão pode fornecer:

```text
workState
```

Animation pode reproduzir:

```text
mining
farming
building
smithing
```

---

# 184. Animation + Ecology

Animal state:

```text
grazing
sleeping
fleeing
hunting
mating
```

Animation traduz.

---

# 185. Animation + Combat

Combat state:

```text
attack
defend
stagger
```

Animation resolve apresentação.

---

# 186. Animation + Inventory

Equipamento muda:

```text
equipment state
```

Animation atualiza:

```text
holding pose
```

---

# 187. Animation + Tool API

Ferramenta define:

```text
usage profile
```

Animation define:

```text
swing / use animation
```

---

# 188. Animation + Physics

Physics fornece:

```text
velocity
contacts
```

Animation fornece:

```text
pose
```

---

# 189. Animation + Renderer

```text
Animation
→ Pose
→ Renderer
```

---

# 190. Animation + Audio

Markers:

```text
footstep
impact
mechanism
```

Audio responde.

---

# 191. Animation + Particles

Markers:

```text
dust
spark
muzzle
```

Particle responde.

---

# 192. Animation + Event Bus

Somente eventos significativos:

```text
AnimationMarkerReached
AnimationStateChanged
AnimationCompleted
```

---

# 193. Animation + Save

Salvar:

```text
controller state
```

quando necessário.

Não salvar:

```text
every bone transform
```

---

# 194. Animation + Networking

Replicar:

```text
high-level state
```

e não:

```text
every pose
```

---

# 195. Animation + Mods

Mod pode criar:

```text
new species
new rig
new animations
new controller
```

sem alterar Core.

---

# 196. Animation + Registry

```text
AnimationRegistry
SkeletonRegistry
RigRegistry
```

---

# 197. Animation API

Interfaces:

```text
IAnimationClip
IAnimationTrack
IAnimationGraph
IAnimationController
ISkeleton
IRig
IPose
IAnimationPlayer
IAnimationConstraint
IAnimationRetargeter
```

---

# 198. Animation Runtime

```text
AnimationRuntime

update()
evaluate()
blend()
solveIK()
applyConstraints()
generatePose()
```

---

# 199. Animation Scheduler

```text
IAnimationScheduler

register()
unregister()
schedule()
evaluateBatch()
```

---

# 200. Animation Output

```text
IAnimationOutput

getPose()
getMarkers()
getState()
```

---

# 201. Debug API

```text
IAnimationDebugger

inspect()
traceGraph()
inspectSkeleton()
profile()
```

---

# 202. Código

Eu organizaria assim:

```text
src/
└── animation/
    ├── core/
    │   ├── animation.ts
    │   ├── animation-clip.ts
    │   ├── animation-track.ts
    │   ├── animation-channel.ts
    │   ├── animation-marker.ts
    │   └── animation-time.ts
    │
    ├── skeleton/
    │   ├── skeleton.ts
    │   ├── bone.ts
    │   ├── pose.ts
    │   ├── transform.ts
    │   └── bind-pose.ts
    │
    ├── rig/
    │   ├── rig.ts
    │   ├── constraints.ts
    │   └── retarget.ts
    │
    ├── player/
    │   ├── animation-player.ts
    │   └── playback-state.ts
    │
    ├── graph/
    │   ├── animation-graph.ts
    │   ├── state-machine.ts
    │   ├── state.ts
    │   ├── transition.ts
    │   ├── blend-tree.ts
    │   └── parameters.ts
    │
    ├── layers/
    │   ├── animation-layer.ts
    │   ├── bone-mask.ts
    │   └── additive.ts
    │
    ├── procedural/
    │   ├── procedural-layer.ts
    │   ├── look-at.ts
    │   ├── aim.ts
    │   ├── foot-ik.ts
    │   └── spring-bones.ts
    │
    ├── runtime/
    │   ├── animation-runtime.ts
    │   ├── animation-scheduler.ts
    │   ├── pose-buffer.ts
    │   ├── evaluation-context.ts
    │   └── batch-evaluator.ts
    │
    ├── lod/
    │   ├── animation-lod.ts
    │   └── animation-budget.ts
    │
    ├── registry/
    │   ├── animation-registry.ts
    │   ├── skeleton-registry.ts
    │   ├── rig-registry.ts
    │   └── graph-registry.ts
    │
    ├── serialization/
    │   ├── serializer.ts
    │   └── migration.ts
    │
    ├── networking/
    │   └── animation-replication.ts
    │
    ├── debugging/
    │   ├── animation-debugger.ts
    │   ├── graph-tracer.ts
    │   └── profiler.ts
    │
    └── api/
        └── animation-api.ts
```

---

# 203. Fronteira arquitetural

## Animation System faz

```text
clips
tracks
keyframes
skeleton
rig
pose
state machines
blend
layers
IK
constraints
retargeting
procedural animation
markers
animation LOD
animation runtime
```

## Não faz

```text
combat
AI
physics
rendering
audio playback
particles
inventory
vehicle physics
civilization
world simulation
```

---

# 204. Regra fundamental

> **Animation System converte estado e intenção em pose. Ele não decide o significado gameplay dessa pose.**

---

# 205. Segunda regra

> **Renderer apresenta a pose; Animation System calcula a pose.**

---

# 206. Terceira regra

> **Gameplay deve continuar correto mesmo quando a animação visual estiver desativada, atrasada ou em LOD reduzido.**

Isso é especialmente importante para multiplayer e para os milhares de entidades do NEXORA.

---

# 207. Ordem de implementação

```text
ANIM-0    Core Contracts
ANIM-1    AnimationID
ANIM-2    AnimationRegistry
ANIM-3    AnimationClip
ANIM-4    Track
ANIM-5    Keyframes
ANIM-6    Interpolation
ANIM-7    Transform
ANIM-8    Skeleton
ANIM-9    Bone
ANIM-10   Pose
ANIM-11   AnimationPlayer
ANIM-12   Playback
ANIM-13   State
ANIM-14   StateMachine
ANIM-15   Transition
ANIM-16   Parameters
ANIM-17   Blend
ANIM-18   BlendTree
ANIM-19   Layers
ANIM-20   BoneMasks
ANIM-21   Additive
ANIM-22   AnimationGraph
ANIM-23   Markers
ANIM-24   Event Bus Integration
ANIM-25   Rig
ANIM-26   Constraints
ANIM-27   IK
ANIM-28   LookAt
ANIM-29   Aim
ANIM-30   Foot IK
ANIM-31   Retargeting
ANIM-32   Procedural Animation
ANIM-33   Animation LOD
ANIM-34   Scheduler
ANIM-35   Batch Evaluation
ANIM-36   Runtime Compilation
ANIM-37   Compression
ANIM-38   Serialization
ANIM-39   Networking
ANIM-40   Mod API
ANIM-41   Debugging
ANIM-42   Profiling
ANIM-43   Stress Tests
ANIM-44   Compatibility
```

---

# 208. Primeiro Vertical Slice

Começaria extremamente simples:

```text
AnimationRegistry
        ↓
Skeleton
        ↓
AnimationClip
        ↓
AnimationPlayer
        ↓
Pose
        ↓
Renderer
```

Com:

```text
idle
walk
```

---

# 209. Segundo Vertical Slice

```text
Player
 ↓
movement state
 ↓
Animation Controller
 ↓
Idle / Walk / Run
 ↓
Blend Tree
 ↓
Pose
 ↓
Renderer
```

---

# 210. Terceiro Vertical Slice

```text
NPC
 ↓
velocity
 ↓
locomotion
 ↓
Foot IK
 ↓
uneven terrain
 ↓
Renderer
```

---

# 211. Quarto Vertical Slice

```text
Combat
 ↓
Attack state
 ↓
Animation
 ↓
Hit marker
 ↓
Event Bus
 ↓
Combat timing
```

Sem deixar a animação ser a autoridade do dano.

---

# 212. Quinto Vertical Slice

Teste que realmente interessa:

```text
10,000 entities
        ↓
shared Animation Graph
        ↓
FULL / REDUCED / SIMPLIFIED LOD
        ↓
movement
        ↓
combat
        ↓
streaming
        ↓
camera movement
        ↓
re-evaluation
```

Objetivo: demonstrar que animação continua sendo uma camada visual escalável, e não um gargalo do mundo.

---

# 213. Testes

Testar:

```text
Registry
Clip playback
Interpolation
State transitions
Blend
Layers
IK
Retargeting
Markers
Serialization
Migration
LOD
Networking
Mod loading
```

---

# 214. Teste de determinismo

Para o mesmo:

```text
state
parameters
animation version
```

a sequência lógica de estados/markers deve ser consistente.

---

# 215. Teste de performance

Medir:

```text
1 entity
100
1,000
10,000
100,000
```

com:

```text
bones
IK
blend
graphs
markers
LOD
```

---

# 216. Teste de memória

Comparar:

```text
10,000 individual graphs
```

contra:

```text
1 shared graph
+
10,000 controller states
```

A segunda é a arquitetura desejável.

---

# 217. Teste final de integração

```text
                  ENTITY
                     │
                     ↓
               STATE / INTENT
                     │
                     ↓
              ANIMATION GRAPH
                     │
          ┌──────────┼───────────┐
          ↓          ↓           ↓
        BLEND        IK       PROCEDURAL
          └──────────┼───────────┘
                     ↓
                   POSE
                     │
              ┌──────┴──────┐
              ↓             ↓
          RENDERER       MARKERS
                            │
               ┌────────────┼─────────────┐
               ↓            ↓             ↓
            COMBAT        AUDIO        PARTICLES
```

Esse desenho mantém a filosofia do NEXORA intacta: **o Entity System diz quem está no mundo, gameplay diz o que a entidade está fazendo, Animation transforma isso em movimento visual, e Renderer mostra o resultado.**

Com isso, a fundação que estamos montando passa a ficar:

```text
NEXORA FOUNDATION
├── Core
├── Registry System
├── Event Bus
├── Save / Persistence
├── Entity System
├── Block System
├── Item System
└── Animation System
```

E o **Animation System** fica preparado desde o começo para o requisito mais pesado do NEXORA: **milhares de tipos de entidades compartilhando rigs, graphs e animações, com LOD e processamento em lote, em vez de cada entidade carregar um sistema de animação próprio.**
