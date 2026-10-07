Perfeito. O **Save / Persistence System** é o próximo pilar que fecha o ciclo dos sistemas que já projetamos.

A regra central seria:

> **Persistence System transforma o estado do NEXORA em dados duráveis e reconstrói esse estado de forma segura, versionada, incremental e compatível com futuras versões do jogo e dos mods.**

Ele não deve decidir como o mundo funciona. Ele deve garantir que **o mundo continue existindo depois que o processo terminar**.

# NEXORA — SAVE / PERSISTENCE SYSTEM

## 1. Princípio central

```text id="save-01"
             NEXORA RUNTIME
                   │
          CURRENT WORLD STATE
                   │
             PERSISTENCE
                   │
      ┌────────────┼────────────┐
      ↓            ↓            ↓
   SNAPSHOT      JOURNAL      INDEX
      │            │            │
      └────────────┼────────────┘
                   ↓
              SAVE STORAGE
                   │
                   ↓
              LOAD / RECOVERY
                   │
                   ↓
          WORLD STATE RESTORED
```

A ideia não é:

```text
save()
save()
save()
```

mas sim:

```text
WORLD STATE
   ↓
DIRTY TRACKING
   ↓
SERIALIZATION
   ↓
SNAPSHOT / DELTA
   ↓
ATOMIC COMMIT
   ↓
DURABLE STORAGE
```

---

# 2. O que Persistence precisa salvar?

Praticamente tudo que precisa sobreviver entre sessões:

```text id="save-02"
World
Dimensions
Chunks
Blocks
BlockEntities
Entities
Items
Inventories
Players
NPCs
Civilizations
Economies
Machines
Energy Networks
Fluid State
Climate State
Vegetation State
Quests
Research
Progression
Structures
Transport
World History
Mod State
Registry Compatibility
```

Mas **nem tudo precisa ser salvo com a mesma frequência ou da mesma maneira**.

---

# 3. Estado persistente vs estado derivado

Essa distinção é fundamental.

### Persistente

Informação que realmente precisa ser armazenada:

```text id="save-03"
player inventory
chunk block states
NPC identity
machine progress
world time
civilization treasury
```

### Derivado

Informação que pode ser reconstruída:

```text id="save-04"
render mesh
lighting cache
spatial index
network cache
temporary pathfinding cache
```

Regra:

> **Não salvar aquilo que o engine consegue reconstruir corretamente.**

---

# 4. Persistence não é simplesmente "serialização"

```text id="save-05"
Serialization
= transformar estado em bytes/dados

Persistence
= decidir o que salvar, quando, onde, como versionar, recuperar e validar
```

---

# 5. Arquitetura principal

```text id="save-06"
PERSISTENCE SYSTEM
├── SAVE MANAGER
├── WORLD STORAGE
├── CHUNK STORAGE
├── ENTITY STORAGE
├── PLAYER STORAGE
├── REGISTRY SNAPSHOT
├── SERIALIZATION
├── VERSIONING
├── MIGRATION
├── JOURNAL
├── SNAPSHOT
├── CHECKPOINT
├── TRANSACTION
├── RECOVERY
├── INTEGRITY
├── COMPRESSION
├── ENCRYPTION/PROTECTION POLICY
├── BACKUP
├── STREAMING
├── ASYNC I/O
├── DIRTY TRACKING
└── DEBUG/TOOLS
```

---

# 6. Save Manager

Ponto central:

```text id="save-07"
ISaveManager
```

responsável por coordenar:

```text id="save-08"
startSave()
save()
checkpoint()
flush()
load()
recover()
```

Mas não serializa tudo sozinho.

Ele orquestra especialistas.

---

# 7. World Save

O mundo deve possuir metadados:

```text id="save-09"
WorldSaveMetadata

worldId
worldName
gameVersion
worldFormatVersion
worldSeed
worldGenerationVersion
createdAt
lastSavedAt
dimensions
modpackFingerprint
registryFingerprint
```

---

# 8. World ID

Cada mundo possui identidade estável.

```text id="save-10"
PersistentWorldID
```

Não depender do nome do diretório.

---

# 9. World Seed

Salvar:

```text id="save-11"
seed
```

mas também:

```text id="save-12"
worldGenerationVersion
```

Porque o mesmo seed com um generator diferente pode produzir outro mundo.

---

# 10. World Generation Version

Muito importante:

```text id="save-13"
seed
+
generation version
+
dimension generation version
```

Isso mantém o histórico do mundo.

---

# 11. World Time

Salvar:

```text id="save-14"
worldTime
simulationTick
calendar
season
```

Se o NEXORA possui simulação de dia/semana/estação:

```text id="save-15"
second
minute
hour
day
week
season
year
```

podem ser derivados de um relógio lógico principal.

---

# 12. Dimension Persistence

Cada dimensão possui:

```text id="save-16"
DimensionSave

dimensionId
generationVersion
worldState
chunkIndex
specialState
```

Exemplo:

```text id="save-17"
nexora:overworld
nexora:void
nexora:custom_dimension
```

---

# 13. Chunk Save

Chunk será provavelmente a unidade mais importante de persistência do mundo.

```text id="save-18"
ChunkSave

chunkPosition
dimension
generationStatus
blockStorage
blockEntities
entities/reference
heightmaps
biomeData
localState
version
```

---

# 14. Chunk State

Chunk pode ter:

```text id="save-19"
GENERATED
LOADED
DIRTY
SAVING
UNLOADED
CORRUPTED
```

---

# 15. Chunk Dirty Tracking

Quando algo muda:

```text id="save-20"
block changed
 ↓
chunk.dirty = true
```

ou categorias:

```text id="save-21"
BLOCK_DIRTY
ENTITY_DIRTY
FLUID_DIRTY
LIGHTING_DIRTY
VEGETATION_DIRTY
MACHINE_DIRTY
METADATA_DIRTY
```

---

# 16. Dirty Flags

Não basta `dirty = true`.

Precisamos saber **por quê**.

```text id="save-22"
DirtyMask
```

Isso permite saves seletivos.

---

# 17. Incremental Save

Não salvar o mundo inteiro a cada alteração.

```text id="save-23"
World
├── Chunk A dirty
├── Chunk B clean
├── Chunk C clean
└── Chunk D dirty

SAVE
→ A + D
```

---

# 18. Save Queue

Criar:

```text id="save-24"
SaveQueue
```

que ordena trabalhos por:

```text id="save-25"
priority
age
distance
dirty level
criticality
```

---

# 19. Critical Save

Algumas alterações precisam de persistência prioritária:

```text id="save-26"
player progress
important transaction
world migration
unique artifact
civilization event
```

---

# 20. Background Save

O jogo deve poder:

```text id="save-27"
continue simulation
+
save chunks in background
```

sem congelar tudo.

---

# 21. Snapshot

O sistema precisa conseguir criar um snapshot consistente:

```text id="save-28"
WorldState
      ↓
Snapshot
      ↓
Serialization
      ↓
Storage
```

---

# 22. Snapshot Consistency

Nunca capturar metade do estado em um momento e metade em outro sem controle.

Usar:

```text id="save-29"
simulation checkpoint
```

ou snapshots versionados.

---

# 23. Snapshot Boundary

Exemplo:

```text id="save-30"
SIMULATION
 ↓
SAFE SAVE POINT
 ↓
SNAPSHOT
 ↓
CONTINUE
```

---

# 24. Copy-on-write

Para mundos grandes:

```text id="save-31"
live state
+
snapshot reference
```

e apenas dados modificados são copiados.

---

# 25. Save Transaction

Cada save importante deve possuir:

```text id="save-32"
SaveTransactionID
```

---

# 26. Atomic Save

Nunca fazer:

```text id="save-33"
overwrite old save
```

diretamente.

Melhor:

```text id="save-34"
write temporary
 ↓
flush
 ↓
validate
 ↓
commit
 ↓
replace manifest
```

---

# 27. Two-phase Commit

Conceitualmente:

```text id="save-35"
PREPARE
 ↓
WRITE ALL DATA
 ↓
VERIFY
 ↓
COMMIT
```

Se falhar:

```text id="save-36"
ROLLBACK / RECOVER
```

---

# 28. Manifest

Um save precisa de um índice principal:

```text id="save-37"
save.manifest
```

Pode indicar:

```text id="save-38"
saveVersion
worldVersion
snapshotId
chunk index
registry fingerprint
mod fingerprint
checksums
```

---

# 29. Save Generation

Manter gerações:

```text id="save-39"
Generation N
Generation N+1
```

Se a gravação nova falhar:

```text id="save-40"
Generation N
```

continua disponível.

---

# 30. Save Journaling

Além de snapshots:

```text id="save-41"
Journal
```

registra mudanças recentes.

Modelo:

```text id="save-42"
Snapshot
+
Journal
```

---

# 31. Por que Journal?

Se o jogo cair depois do último snapshot:

```text id="save-43"
snapshot
+
recent changes
```

podem reconstruir o estado.

---

# 32. Journal Entries

Exemplo:

```text id="save-44"
BlockChange
ItemTransaction
EntityCreated
EntityRemoved
MachineStateChanged
EconomyTransaction
```

---

# 33. Não transformar tudo em Event Sourcing

Mesma decisão do Event Bus:

```text id="save-45"
State + Journal
```

é melhor que:

```text id="save-46"
entire world = infinite event log
```

---

# 34. Journal Granularity

Nem toda alteração precisa virar journal individual.

Pode haver:

```text id="save-47"
block batch
inventory batch
region batch
```

---

# 35. Journal Checkpoint

Periodicamente:

```text id="save-48"
journal
 ↓
checkpoint
 ↓
new snapshot
 ↓
clear old journal
```

---

# 36. Recovery

Ao carregar:

```text id="save-49"
Read Manifest
 ↓
Validate Snapshot
 ↓
Load Snapshot
 ↓
Replay Journal
 ↓
Validate State
 ↓
WORLD READY
```

---

# 37. Corrupted Save

Não apagar automaticamente.

Estados:

```text id="save-50"
VALID
PARTIALLY_CORRUPTED
RECOVERABLE
UNRECOVERABLE
```

---

# 38. Checksum

Arquivos importantes possuem:

```text id="save-51"
checksum
```

para detectar corrupção.

---

# 39. Hash

Podemos usar hash para:

```text id="save-52"
chunk
region
snapshot
manifest
journal
```

---

# 40. Integrity Verification

Durante load:

```text id="save-53"
read
 ↓
hash
 ↓
compare
 ↓
accept/reject
```

---

# 41. Chunk Checksum

Cada chunk pode possuir:

```text id="save-54"
contentHash
```

---

# 42. Region Checksum

Também:

```text id="save-55"
regionHash
```

pode acelerar verificação agregada.

---

# 43. Redundancy

Podemos ter:

```text id="save-56"
primary
backup
journal
```

mas sem duplicar desnecessariamente tudo.

---

# 44. Backup

Save System pode suportar:

```text id="save-57"
manual backup
automatic backup
rolling backup
checkpoint backup
```

---

# 45. Rolling Backup

Manter:

```text id="save-58"
save-1
save-2
save-3
```

com política configurável.

---

# 46. Backup Rotation

Exemplo:

```text id="save-59"
last 5 checkpoints
```

---

# 47. Player Save

Player deve ter arquivo/dataset próprio:

```text id="save-60"
PlayerSave

playerId
identity
position
dimension
health
status
inventory
equipment
loadouts
progression
knowledge
reputation
quests
statistics
```

---

# 48. Player Identity

Separar:

```text id="save-61"
account identity
```

de:

```text id="save-62"
character state
```

para servidores multiplayer.

---

# 49. Player Character State

```text id="save-63"
character
├── transform
├── inventory
├── equipment
├── health
├── progression
└── knowledge
```

---

# 50. Inventory Persistence

Inventory usa:

```text id="save-64"
Item Serialization
```

Não deve criar uma implementação paralela.

---

# 51. Equipment Persistence

Salvar:

```text id="save-65"
equipped items
active loadout
extra loadouts
accessories
```

---

# 52. Entity Persistence

Entity System fornece:

```text id="save-66"
EntityPersistencePolicy
```

Exemplos:

```text id="save-67"
PERSISTENT
REGIONAL
TEMPORARY
ABSTRACTABLE
```

---

# 53. Persistent Entity

Exemplos:

```text id="save-68"
named NPC
pet
vehicle
important boss
civilization leader
unique mob
```

---

# 54. Temporary Entity

Exemplo:

```text id="save-69"
particle entity
temporary projectile
cosmetic effect
```

Pode ser descartada.

---

# 55. Abstractable Entity

Perfeito para a simulação LOD:

```text id="save-70"
FULL Entity
      ↓
REGIONAL representation
      ↓
ABSTRACT state
```

Persistence salva a representação necessária.

---

# 56. NPC Persistence

NPC pode exigir:

```text id="save-71"
identity
species
profession
needs
relationships
memory
knowledge
reputation
schedule
home
work
faction
inventory
health
history
```

---

# 57. Civilization Persistence

Salvar:

```text id="save-72"
population
settlements
leaders
laws
economy
treasury
resources
trade routes
diplomacy
elections
infrastructure
history
```

---

# 58. Economy Persistence

```text id="save-73"
markets
prices
supply
demand
currency
contracts
trade
production
consumption
```

---

# 59. Machine Persistence

Máquinas precisam salvar:

```text id="save-74"
definition
state
recipe
progress
inventory
energy
fluid
temperature
maintenance
```

---

# 60. Energy Persistence

Networks podem precisar de:

```text id="save-75"
network topology
stored energy
node states
faults
```

Mas caches de caminhos podem ser reconstruídos.

---

# 61. Fluid Persistence

Precisa salvar:

```text id="save-76"
persistent fluid volumes
tank contents
network state when required
```

Não necessariamente cada cálculo intermediário de fluxo.

---

# 62. Climate Persistence

Não precisa salvar cada nuvem.

Pode salvar:

```text id="save-77"
long-term climate state
season
weather seed/state
persistent anomalies
pollution
```

e reconstruir fenômenos transitórios.

---

# 63. Vegetation Persistence

Salvar apenas alterações importantes.

```text id="save-78"
seed populations
growth state
harvest changes
diseases
fire disturbances
persistent mutations
```

O resto pode ser regenerado.

---

# 64. WorldGen vs Persistence

WorldGen:

```text id="save-79"
generate initial state
```

Persistence:

```text id="save-80"
preserve changed state
```

Isso é crucial.

---

# 65. Chunk Generation

Quando chunk nunca existiu:

```text id="save-81"
WorldGen
```

Quando já existe:

```text id="save-82"
Load persisted chunk
```

Nunca regenerar cegamente um chunk salvo.

---

# 66. Chunk Status

```text id="save-83"
NEW
GENERATING
GENERATED
SAVED
```

---

# 67. Generated-but-not-Modified Chunks

Podemos decidir não salvar alguns chunks gerados mas imutáveis.

```text id="save-84"
regenerable
```

Mas isso exige uma política segura.

---

# 68. Regenerability

Chunk pode possuir:

```text id="save-85"
Regenerability:
    REGENERABLE
    PARTIAL
    NON_REGENERABLE
```

---

# 69. Procedural Base + Delta

Uma técnica ótima para NEXORA:

```text id="save-86"
Generated Chunk
+
Player Changes
=
Current Chunk
```

Assim, partes proceduralmente estáveis não precisam ocupar tanto espaço.

---

# 70. Chunk Delta

Exemplo:

```text id="save-87"
base worldgen
+
modified blocks
+
dynamic data
```

---

# 71. Cuidado com Dynamic World

Depois que um mundo "vive":

```text id="save-88"
forest grows
river changes
city expands
mine gets excavated
```

não podemos depender somente de regeneração procedural.

Precisamos persistir a divergência.

---

# 72. Persistence Layers

Eu usaria camadas:

```text id="save-89"
LAYER 0
Procedural Base

LAYER 1
Persistent World Data

LAYER 2
Dynamic Simulation

LAYER 3
Player/Server Data

LAYER 4
Journal
```

---

# 73. Region Storage

Para mundos gigantes:

```text id="save-90"
World
 └── Dimension
      └── Region
           └── Chunk
```

---

# 74. Region File

Uma região pode agrupar vários chunks.

```text id="save-91"
Region
├── chunk 0
├── chunk 1
├── ...
└── chunk N
```

Isso reduz overhead de arquivos.

---

# 75. Storage Backend

Criar abstração:

```text id="save-92"
IStorageBackend
```

Assim NEXORA não fica preso a um único formato.

---

# 76. Local Backend

```text id="save-93"
FileStorageBackend
```

---

# 77. Memory Backend

Para:

```text id="save-94"
tests
editor
preview
simulation
```

---

# 78. Remote Backend

Futuro servidor poderia utilizar:

```text id="save-95"
RemoteStorageBackend
```

mas isso não precisa existir na V1.

---

# 79. Cloud Backup

Também pode existir no futuro:

```text id="save-96"
BackupProvider
```

sem contaminar o Core.

---

# 80. Storage API

```text id="save-97"
read
write
delete
exists
list
rename
flush
sync
```

---

# 81. Atomic Storage API

Preferível possuir:

```text id="save-98"
writeAtomic()
```

ou equivalente.

---

# 82. File Locking

Em desktop/server:

```text id="save-99"
world.lock
```

impede dois processos escreverem o mesmo mundo sem controle.

---

# 83. Crash Recovery

Se o processo morrer durante save:

```text id="save-100"
old manifest
new temp data
```

o próximo boot identifica o estado.

---

# 84. Recovery Journal

Pode existir:

```text id="save-101"
recovery journal
```

com operações pendentes.

---

# 85. Save State Machine

```text id="save-102"
IDLE
 ↓
PREPARING
 ↓
SNAPSHOTTING
 ↓
SERIALIZING
 ↓
WRITING
 ↓
VERIFYING
 ↓
COMMITTING
 ↓
COMPLETED
```

Erro:

```text id="save-103"
ERROR
 ↓
RECOVER
```

---

# 86. Load State Machine

```text id="save-104"
DISCOVER
 ↓
READ MANIFEST
 ↓
VERIFY
 ↓
LOAD REGISTRY
 ↓
LOAD WORLD
 ↓
LOAD DIMENSIONS
 ↓
LOAD CHUNKS
 ↓
LOAD ENTITIES
 ↓
REPLAY JOURNAL
 ↓
MIGRATE
 ↓
VALIDATE
 ↓
READY
```

---

# 87. Registry Load Before World

Muito importante:

```text id="save-105"
Registry
 ↓
resolve IDs
 ↓
deserialize world
```

Não tentar interpretar dados do mundo sem conhecer as definições.

---

# 88. Mod Compatibility

Save deve registrar:

```text id="save-106"
modpack fingerprint
mod IDs
versions
content fingerprints
```

---

# 89. Missing Mod

Se um mod foi removido:

```text id="save-107"
missing content
```

preservar dados quando possível.

Isso reutiliza a filosofia de:

```text id="save-108"
MissingBlock
MissingItem
MissingEntity
```

---

# 90. Missing Entity

Uma Entity persistente cujo mod sumiu pode virar:

```text id="save-109"
MissingEntity
```

mantendo dados crus.

---

# 91. Migration Registry

Criar:

```text id="save-110"
MigrationRegistry
```

para:

```text id="save-111"
World
Chunk
Entity
Block
Item
Machine
Quest
Economy
```

---

# 92. Save Format Version

Ter uma versão global:

```text id="save-112"
SaveFormatVersion
```

---

# 93. Schema Version

Cada domínio pode possuir sua versão:

```text id="save-113"
ChunkSchemaVersion
ItemSchemaVersion
EntitySchemaVersion
```

---

# 94. Migration Pipeline

```text id="save-114"
Save v1
 ↓
Migration v1→v2
 ↓
Migration v2→v3
 ↓
Current format
```

---

# 95. Direct Migration vs Chain

Não é necessário manter migrações para sempre.

Podemos suportar:

```text id="save-115"
v1 → latest
```

quando houver ferramentas de conversão.

---

# 96. Migration Safety

Nunca migrar o save original diretamente.

```text id="save-116"
original
 ↓
backup
 ↓
working copy
 ↓
migration
 ↓
validation
 ↓
commit
```

---

# 97. Migration Dry Run

Muito útil:

```text id="save-117"
nexora save migrate --dry-run
```

mostrando:

```text id="save-118"
entries changed
missing content
warnings
errors
estimated impact
```

---

# 98. Save Validator

```text id="save-119"
nexora save validate
```

verifica:

```text id="save-120"
manifest
chunks
entities
IDs
checksums
references
versions
```

---

# 99. Save Repair

```text id="save-121"
nexora save repair
```

pode:

```text id="save-122"
rebuild index
recover journal
quarantine corrupted region
restore backup
```

---

# 100. Quarantine

Se um chunk estiver corrompido:

```text id="save-123"
quarantine chunk
```

em vez de impedir o mundo inteiro de abrir.

---

# 101. Partial World Load

Idealmente:

```text id="save-124"
corrupted chunk
```

não impede:

```text id="save-125"
other chunks
```

de serem carregados.

---

# 102. Recovery Strategy

```text id="save-126"
primary chunk
 ↓
journal
 ↓
backup
 ↓
regenerate base + apply valid delta
 ↓
quarantine
```

---

# 103. Regeneration Recovery

Se a parte procedural ainda puder ser reconstruída:

```text id="save-127"
WorldGen
+
valid persistent changes
```

Isso pode salvar mundos parcialmente corrompidos.

---

# 104. Save Checksums

Manifest pode conter:

```text id="save-128"
chunkHash
registryHash
journalHash
```

---

# 105. Compression

Chunk data provavelmente deve ser comprimido.

Arquitetura:

```text id="save-129"
serialize
 ↓
compress
 ↓
encrypt/protect if needed
 ↓
write
```

---

# 106. Compression Strategy

O sistema deve permitir:

```text id="save-130"
NONE
FAST
BALANCED
HIGH
```

dependendo da plataforma.

---

# 107. Compression Scope

Comprimir por:

```text id="save-131"
chunk
region
journal segment
```

e não necessariamente o save inteiro como um arquivo único.

---

# 108. Encryption / Protection

Para single-player geralmente não é necessária uma proteção forte.

Para servidor, dados sensíveis podem exigir mecanismos próprios.

Não misturar segurança de autenticação com o formato do mundo.

---

# 109. Data Integrity vs Security

Separar:

```text id="save-132"
Integrity
→ corrupção

Authentication
→ origem/autorização
```

---

# 110. Serialization Framework

Criar:

```text id="save-133"
ISerializer<T>
```

com:

```text id="save-134"
serialize
deserialize
```

---

# 111. Binary Format

Para runtime/save grande:

```text id="save-135"
Binary serialization
```

será provavelmente o caminho principal.

---

# 112. Human-readable Metadata

Manifests e arquivos administrativos podem usar um formato legível.

Exemplo:

```text id="save-136"
manifest
metadata
debug reports
```

---

# 113. Do not use text for everything

Milhões de blocks serializados em texto seriam muito caros.

---

# 114. Schema Definition

Cada serializable type deve possuir schema:

```text id="save-137"
fields
types
defaults
version
```

---

# 115. Optional Fields

Permitir:

```text id="save-138"
field absent
```

e defaults seguros.

Isso facilita evolução do formato.

---

# 116. Unknown Fields

Durante leitura:

```text id="save-139"
unknown field
```

pode ser:

```text id="save-140"
ignored
preserved
error
```

dependendo do schema.

---

# 117. Forward Compatibility

Não prometer compatibilidade infinita.

Mas o formato pode ser desenhado para:

```text id="save-141"
older reader
newer optional fields
```

quando possível.

---

# 118. Backward Compatibility

Mais importante:

```text id="save-142"
new game
reads older save
```

via migration.

---

# 119. World Format Contract

Documentar:

```text id="save-143"
Save Format v1
```

com:

```text magic
version
endianness
compression
sections
checksums
```

---

# 120. Magic Header

Arquivos binários podem ter um identificador:

```text id="save-144"
NEXS
```

ou outro formato oficial do projeto.

Serve para detectar arquivos inválidos rapidamente.

---

# 121. Sectioned Format

Um arquivo pode possuir:

```text id="save-145"
HEADER
METADATA
REGISTRY
WORLD
CHUNKS
ENTITIES
JOURNAL
FOOTER
```

---

# 122. Better: Region-oriented Storage

Para mundo enorme:

```text id="save-146"
World
├── metadata
├── registry
├── regions/
│    ├── r.0.0
│    ├── r.0.1
│    └── ...
└── players/
```

---

# 123. Region Index

Cada região pode possuir índice:

```text id="save-147"
chunk coordinate
offset
length
hash
version
```

---

# 124. Random Access

Assim podemos carregar:

```text id="save-148"
apenas o chunk necessário
```

sem ler o mundo inteiro.

---

# 125. Streaming

Perfeito para o NEXORA:

```text id="save-149"
Player
 ↓
moves
 ↓
chunk needed
 ↓
load async
```

---

# 126. Save Unloaded Chunks

Quando chunk fica distante:

```text id="save-150"
simulate
 ↓
checkpoint
 ↓
save
 ↓
unload
```

---

# 127. Persistence LOD

Isso combina com a simulação LOD:

```text id="save-151"
FULL
→ save detailed state

REGIONAL
→ save aggregate state

ABSTRACT
→ save abstract state
```

---

# 128. Rehydration

Quando uma região volta a ficar próxima:

```text id="save-152"
ABSTRACT
 ↓
REGIONAL
 ↓
FULL
```

usando persistent data.

---

# 129. NPC Persistence LOD

Exemplo:

```text id="save-153"
10,000 NPCs
```

distantes podem ser armazenados como:

```text id="save-154"
population statistics
roles
relationships summary
location
resources
```

e não como cada decisão intermediária.

---

# 130. Rehydration Rules

Cada entidade define:

```text id="save-155"
rehydration strategy
```

---

# 131. Important NPCs

Alguns sempre permanecem individualizados:

```text id="save-156"
named
quest
leader
unique
persistent
```

---

# 132. Simulation Checkpoint

Antes de unload:

```text id="save-157"
simulate
 ↓
aggregate
 ↓
checkpoint
 ↓
persist
```

---

# 133. Offline Simulation

Quando região está descarregada:

```text id="save-158"
Abstract Simulation
```

pode avançar:

```text id="save-159"
economy
population
weather
production
```

e persistir o resultado agregado.

---

# 134. Offline Progression

Player pode sair por:

```text id="save-160"
2 days
```

e quando retornar:

```text id="save-161"
city changed
farm grew
market shifted
machine progressed
```

porque o estado persistente continuou.

---

# 135. Player Save Frequency

Não fazer save a cada frame.

Pode haver:

```text id="save-162"
autosave interval
critical event save
manual save
shutdown save
```

---

# 136. Autosave

Exemplo conceitual:

```text id="save-163"
simulation checkpoint
 ↓
dirty state
 ↓
save
```

O intervalo exato deve ser configurável.

---

# 137. Manual Save

Comando:

```text id="save-164"
/save
```

ou:

```text id="save-165"
nexora save
```

---

# 138. Shutdown Save

Ao fechar normalmente:

```text id="save-166"
flush critical state
save dirty data
commit
close
```

Mas nunca depender exclusivamente dele.

---

# 139. Crash Resilience

O sistema deve assumir:

```text id="save-167"
power loss
process crash
OS kill
disk error
```

podem acontecer.

---

# 140. Save Barrier

Eventos críticos podem pedir:

```text id="save-168"
PersistenceBarrier
```

que garante que determinado estado chegou ao armazenamento.

---

# 141. Example — Trade

```text id="save-169"
Trade
 ↓
ItemTransaction
 ↓
Economy state
 ↓
Event
 ↓
Dirty
 ↓
Journal
```

Se o jogo cair, a transação não pode simplesmente desaparecer ou duplicar.

---

# 142. Exactly-once Transactions

Não depender do save para isso.

A transação possui:

```text id="save-170"
transactionId
```

e o estado persistente registra seu processamento.

---

# 143. Idempotent Recovery

Ao recuperar:

```text id="save-171"
transaction already committed?
→ do not apply twice
```

---

# 144. Save + Event Bus

Event Bus pode informar:

```text id="save-172"
WorldStateChanged
```

e Persistence marca dirty.

Mas:

```text id="save-173"
Event Bus
```

não sabe como salvar.

---

# 145. Save + Registry

Save depende do Registry para resolver:

```text id="save-174"
block IDs
item IDs
entity types
biomes
fluids
```

---

# 146. Save + Block

```text id="save-175"
Block State
 ↓
Block Serializer
 ↓
Chunk Storage
```

---

# 147. Save + Item

```text id="save-176"
Item Stack
 ↓
Item Serializer
 ↓
Inventory
```

---

# 148. Save + Entity

```text id="save-177"
Entity
 ↓
Entity Persistence Policy
 ↓
Entity Serializer
```

---

# 149. Save + BlockEntity

```text id="save-178"
BlockEntity
 ↓
Component serialization
 ↓
Chunk
```

---

# 150. Save + Machines

Machine serializa:

```text id="save-179"
state
progress
inventory
energy
fluids
```

---

# 151. Save + Energy

Energy Network salva:

```text id="save-180"
persistent state
```

mas topologia pode ser reconstruída quando possível.

---

# 152. Save + Fluid

Fluid containers sempre precisam preservar conteúdo.

Transient flow may be reconstructed.

---

# 153. Save + Climate

Climatic macrostate pode ser salva; pequenos detalhes podem ser regenerados deterministicamente.

---

# 154. Save + Vegetation

Alterações não regeneráveis precisam ser armazenadas.

---

# 155. Save + Civilization

Estado econômico e institucional precisa sobreviver.

---

# 156. Save + Knowledge

NPC knowledge pode ser importante para continuidade da história.

```text id="save-181"
knowledge graph
discoveries
research progress
rumors
```

---

# 157. Save + History

Eventos históricos importantes podem ser persistidos:

```text id="save-182"
wars
elections
discoveries
founding
disasters
```

Não salvar cada microevento.

---

# 158. Save + Quest

Quest state:

```text id="save-183"
active
completed
failed
progress
```

---

# 159. Save + Progression

Player/progression:

```text id="save-184"
skills
technology
unlocks
research
achievements
```

---

# 160. Save + Dimensions

Cada dimension precisa de:

```text id="save-185"
dimension state
spawn rules
generation version
persistent entities
regional state
```

---

# 161. Save + Space

Quando NEXORA chegar à parte espacial:

```text id="save-186"
planet state
orbit state
ship state
cargo
astronomical state
```

podem usar o mesmo framework.

---

# 162. Save + Far Lands

Como Far Lands fazem parte da progressão:

```text id="save-187"
frontier state
discovered resources
rail infrastructure
settlements
exploration history
```

também são persistentes.

---

# 163. Save + Beyondlands

Mesmo princípio.

---

# 164. Save + Void

Void pode possuir:

```text id="save-188"
dimension-specific state
```

sem exigir um formato totalmente separado.

---

# 165. Save + Mods

Cada mod pode registrar:

```text id="save-189"
IPersistentDataProvider
```

para seu próprio estado.

---

# 166. Mod Save Namespace

Cada mod possui:

```text id="save-190"
mods/examplemod/
```

ou namespace lógico equivalente.

---

# 167. Mod Persistence Contract

Um mod pode definir:

```text id="save-191"
schema
serializer
version
migration
```

---

# 168. Mod Isolation

Se mod falhar durante save:

```text id="save-192"
isolate mod data
```

sem corromper o mundo inteiro.

---

# 169. Mod Save Limits

Mods podem ter:

```text id="save-193"
data size limit
write rate limit
schema constraints
```

---

# 170. Mod Unload

Se o mod for removido:

```text id="save-194"
preserve raw serialized data
```

quando possível.

---

# 171. Mod Reinstall

Quando retorna:

```text id="save-195"
missing data
 ↓
deserialize
 ↓
migration
 ↓
restore
```

---

# 172. Save Compatibility Matrix

Guardar algo como:

```text id="save-196"
Game version
World version
Modpack version
Registry version
Save format
```

---

# 173. Save Manifest Example

Conceitualmente:

```json id="save-197"
{
  "format": 1,
  "game": "0.1.0",
  "worldFormat": 3,
  "worldId": "...",
  "seed": "...",
  "registryFingerprint": "...",
  "modFingerprint": "...",
  "snapshot": "snapshot-104",
  "journal": "journal-104",
  "checksum": "..."
}
```

---

# 174. Save Directory

Conceito:

```text id="save-198"
saves/
└── MyWorld/
    ├── manifest
    ├── world/
    ├── dimensions/
    ├── regions/
    ├── players/
    ├── mods/
    ├── snapshots/
    ├── journal/
    ├── backups/
    └── logs/
```

---

# 175. Region Directory

```text id="save-199"
regions/
└── nexora_overworld/
    ├── r.0.0
    ├── r.0.1
    └── ...
```

---

# 176. Chunk Coordinates

Persistent coordinate:

```text id="save-200"
dimension
regionX
regionZ
chunkX
chunkZ
```

Para um mundo vertical grande, Y normalmente faz parte da seção/chunk interna.

---

# 177. Chunk Sections

Como NEXORA vai de `Y=-1920` a `Y=+1920`, o chunk pode ser particionado verticalmente.

```text id="save-201"
Chunk
├── Section -1920
├── Section -1888
├── ...
└── Section +1888
```

A divisão exata pode ser escolhida conforme o voxel engine.

---

# 178. Sparse Sections

Se uma seção estiver vazia:

```text id="save-202"
empty section
```

não precisa ocupar espaço significativo.

---

# 179. Palette Persistence

A palette do Block Storage pode ser salva de forma compacta.

---

# 180. Compression + Palette

Ótima combinação:

```text id="save-203"
Block Palette
 ↓
Bit-packed states
 ↓
Compression
```

---

# 181. Save Performance

Objetivos:

```text id="save-204"
minimal main-thread blocking
high sequential throughput
bounded memory
incremental writes
```

---

# 182. Async I/O

Estrutura:

```text id="save-205"
Simulation Thread
        ↓
Snapshot
        ↓
Save Worker
        ↓
Serializer
        ↓
Compression
        ↓
Storage
```

---

# 183. Backpressure

Se disk I/O ficar mais lento que simulation:

```text id="save-206"
save queue grows
```

Precisamos de:

```text id="save-207"
priority
coalescing
throttling
emergency flush
```

---

# 184. Save Queue Coalescing

Se o mesmo chunk muda 20 vezes antes de ser salvo:

```text id="save-208"
change x20
```

não precisa gravar 20 snapshots separados.

Pode gravar:

```text id="save-209"
latest consistent state
```

---

# 185. Save Prioritization

Maior prioridade:

```text id="save-210"
player critical state
transaction
world metadata
recently unloaded chunk
```

Menor:

```text id="save-211"
far low-value simulation
```

---

# 186. Save Budget

Definir:

```text id="save-212"
max I/O per tick
max save memory
max queue size
```

---

# 187. Emergency Save

Se o sistema estiver prestes a fechar:

```text id="save-213"
emergency checkpoint
```

com prioridade máxima.

---

# 188. Save Telemetry

Mostrar:

```text id="save-214"
dirty chunks
queue size
write throughput
save duration
journal size
last successful save
```

---

# 189. Save Profiler

Comando:

```text id="save-215"
nexora save profiler
```

---

# 190. Save Inspect

```text id="save-216"
nexora save inspect
```

pode mostrar:

```text id="save-217"
world
chunks
entities
mods
versions
storage
journal
```

---

# 191. Chunk Inspect

```text id="save-218"
nexora save inspect chunk <x> <z>
```

---

# 192. Save Stats

```text id="save-219"
nexora save stats
```

mostra:

```text id="save-220"
total size
world data
entities
players
mods
journal
backups
```

---

# 193. Save Validate

```text id="save-221"
nexora save validate
```

---

# 194. Save Repair

```text id="save-222"
nexora save repair
```

---

# 195. Save Backup

```text id="save-223"
nexora save backup
```

---

# 196. Save Rollback

Futuramente:

```text id="save-224"
nexora save rollback <checkpoint>
```

---

# 197. Deterministic Save Test

Executar:

```text id="save-225"
same world
same state
same save operation
```

e verificar se a representação lógica é consistente.

Bytes podem variar se compressão/metadata não forem determinísticos.

---

# 198. Round-trip Test

O teste básico:

```text id="save-226"
STATE
 ↓
SERIALIZE
 ↓
SAVE
 ↓
LOAD
 ↓
DESERIALIZE
 ↓
STATE'
```

e validar:

```text id="save-227"
STATE == STATE'
```

---

# 199. Migration Round-trip

```text id="save-228"
OLD SAVE
 ↓
MIGRATE
 ↓
LOAD
 ↓
SAVE CURRENT
 ↓
LOAD
```

sem perda indevida.

---

# 200. Corruption Test

Modificar aleatoriamente bytes:

```text id="save-229"
corrupt
 ↓
load
 ↓
detect
 ↓
recover/quarantine
```

---

# 201. Crash Test

Interromper o processo durante:

```text id="save-230"
write
verify
commit
```

e depois executar recovery.

---

# 202. Power-loss Simulation

Simular:

```text id="save-231"
random interruption
```

em pontos críticos.

---

# 203. Journal Recovery Test

```text id="save-232"
snapshot N
+
journal N→N+X
 ↓
crash
 ↓
recover
 ↓
state expected
```

---

# 204. Mod Compatibility Test

```text id="save-233"
save with Mod A
 ↓
remove Mod A
 ↓
load
 ↓
MissingContent preserved
 ↓
reinstall Mod A
 ↓
restore
```

---

# 205. Massive World Test

Simular:

```text id="save-234"
10,000 chunks
100,000 chunks
1,000,000 chunks
```

com saves incrementais.

---

# 206. Massive Entity Test

```text id="save-235"
10k
100k
1M abstract entities
```

e medir persistência.

---

# 207. Performance Target

Testes devem acompanhar:

```text id="save-236"
save latency
load latency
throughput
memory overhead
disk usage
```

sem impor números prematuros antes do engine existir.

---

# 208. Save API

Interface central:

```text id="save-237"
ISaveManager

save()
load()
checkpoint()
flush()
recover()
validate()
backup()
restore()
```

---

# 209. Storage API

```text id="save-238"
IStorageBackend

read()
write()
writeAtomic()
delete()
exists()
list()
flush()
```

---

# 210. Serialization API

```text id="save-239"
ISerializer<T>

serialize()
deserialize()
schema()
version()
```

---

# 211. Persistence Provider

Cada sistema pode registrar:

```text id="save-240"
IPersistenceProvider<T>
```

Exemplo:

```text id="save-241"
BlockPersistenceProvider
EntityPersistenceProvider
PlayerPersistenceProvider
CivilizationPersistenceProvider
```

---

# 212. Dirty Tracker

```text id="save-242"
IDirtyTracker

markDirty()
clearDirty()
isDirty()
getDirtySet()
```

---

# 213. Snapshot Provider

```text id="save-243"
ISnapshotProvider<T>

createSnapshot()
releaseSnapshot()
```

---

# 214. Migration Provider

```text id="save-244"
IMigrationProvider

canMigrate()
migrate()
validate()
```

---

# 215. Journal Provider

```text id="save-245"
IJournal

append()
flush()
checkpoint()
replay()
```

---

# 216. Recovery Manager

```text id="save-246"
IRecoveryManager

detectFailure()
recover()
quarantine()
restoreBackup()
```

---

# 217. Save Transaction

```text id="save-247"
ISaveTransaction

begin()
write()
verify()
commit()
rollback()
```

---

# 218. Save Context

```text id="save-248"
SaveContext

world
version
snapshot
transaction
storage
registrySnapshot
```

---

# 219. Load Context

```text id="save-249"
LoadContext

world
saveVersion
registry
mods
migration
recovery
```

---

# 220. Persistence Policy

Cada objeto pode declarar:

```text id="save-250"
PERSIST
REGENERATE
DERIVE
ABSTRACT
TEMPORARY
```

---

# 221. Persistence Policy para Blocks

```text id="save-251"
Block State
→ PERSIST

Render Mesh
→ DERIVE

Lighting Cache
→ DERIVE

WorldGen Base
→ REGENERATE
```

---

# 222. Persistence Policy para Entities

```text id="save-252"
named NPC
→ PERSIST

particle
→ TEMPORARY

regional creature
→ ABSTRACTABLE
```

---

# 223. Persistence Policy para Machines

```text id="save-253"
machine state
→ PERSIST

pathfinding cache
→ DERIVE
```

---

# 224. Persistence Policy para Climate

```text id="save-254"
season
→ PERSIST

cloud mesh
→ DERIVE
```

---

# 225. Persistence Policy para Renderer

```text id="save-255"
mesh
→ NEVER PERSIST

texture cache
→ NEVER PERSIST
```

---

# 226. Versioned Components

Components de Entity, Item e BlockEntity precisam declarar versão:

```text id="save-256"
componentVersion
```

---

# 227. Component Persistence

Cada componente pode implementar:

```text id="save-257"
IPersistentComponent
```

---

# 228. Component Migration

```text id="save-258"
Component v1
 ↓
Migration
 ↓
Component v2
```

---

# 229. Unknown Components

Mod removido:

```text id="save-259"
unknown component
```

deve poder ser preservado.

---

# 230. Preservation Envelope

Uma solução interessante:

```text id="save-260"
UnknownDataEnvelope

type
version
rawPayload
owner
```

Isso permite restaurar depois.

---

# 231. Save Compatibility with Mods

O save deve lembrar:

```text id="save-261"
which mod created what
```

por namespace/owner.

---

# 232. World Fingerprint

Combinar:

```text id="save-262"
game version
world format
registry fingerprint
mod fingerprint
worldgen fingerprint
```

em um:

```text id="save-263"
WorldCompatibilityFingerprint
```

---

# 233. Compatibility Check

Antes de abrir:

```text id="save-264"
compatible
compatible with migration
missing content
incompatible
corrupted
```

---

# 234. Safe Mode

Se algo estiver muito errado:

```text id="save-265"
Load Safe Mode
```

com:

```text id="save-266"
mods disabled
nonessential systems disabled
repair mode
```

---

# 235. Editor / Tools

O formato de save deve futuramente poder ser analisado sem iniciar o jogo completo.

Exemplo:

```text id="save-267"
NEXORA Save Inspector
```

---

# 236. Headless Validator

Servidor/CI pode executar:

```text id="save-268"
validate save
```

sem Renderer.

---

# 237. CI Save Tests

Cada release pode testar:

```text id="save-269"
old save
→ current engine
```

antes de liberar.

---

# 238. Golden Save Fixtures

Guardar pequenos mundos de teste:

```text id="save-270"
world-empty
world-basic
world-machines
world-fluid
world-civilization
world-modded
```

---

# 239. Compatibility Matrix

Por exemplo:

```text id="save-271"
Save A → Game A
Save A → Game B
Save A → Game C
```

e verificar migração.

---

# 240. Migration Fixtures

Ter saves reais de teste de:

```text id="save-272"
v0.1
v0.2
v0.5
v1.0
```

conforme o projeto evolui.

---

# 241. Save Format Governance

Mudanças no formato precisam de:

```text id="save-273"
ADR
schema update
migration
tests
```

---

# 242. Save Format Must Be Versioned Before 1.0

Não esperar o jogo estar pronto para pensar nisso.

---

# 243. First Vertical Slice

Primeiro:

```text id="save-vs-01"
World
 ↓
BlockRegistry
 ↓
Chunk
 ↓
BlockStorage
 ↓
Save
 ↓
Close
 ↓
Open
 ↓
Load
 ↓
BlockStates restored
```

---

# 244. Second Vertical Slice

```text id="save-vs-02"
Player
 ↓
Inventory
 ↓
ItemStacks
 ↓
Equipment
 ↓
Save
 ↓
Load
```

---

# 245. Third Vertical Slice

```text id="save-vs-03"
Entity
 ↓
BlockEntity
 ↓
Components
 ↓
Serialization
 ↓
Save
 ↓
Load
```

---

# 246. Fourth Vertical Slice

```text id="save-vs-04"
Mod
 ↓
register content
 ↓
save world
 ↓
remove mod
 ↓
load
 ↓
Missing Content preserved
 ↓
restore mod
 ↓
data returns
```

---

# 247. Fifth Vertical Slice

O mais importante:

```text id="save-vs-05"
World
 ↓
Simulation
 ↓
Dirty Tracking
 ↓
Snapshot
 ↓
Journal
 ↓
Atomic Commit
 ↓
Crash Simulation
 ↓
Recovery
 ↓
State Restored
```

---

# 248. Estrutura de código

Eu organizaria assim:

```text id="save-code-01"
src/
└── persistence/
    ├── core/
    │   ├── persistence.ts
    │   ├── save-context.ts
    │   ├── load-context.ts
    │   ├── persistence-policy.ts
    │   └── persistence-version.ts
    │
    ├── manager/
    │   ├── save-manager.ts
    │   ├── load-manager.ts
    │   └── checkpoint-manager.ts
    │
    ├── storage/
    │   ├── storage-backend.ts
    │   ├── file-storage.ts
    │   ├── region-storage.ts
    │   └── memory-storage.ts
    │
    ├── serialization/
    │   ├── serializer.ts
    │   ├── schema.ts
    │   └── binary-format.ts
    │
    ├── snapshot/
    │   ├── snapshot.ts
    │   ├── snapshot-provider.ts
    │   └── copy-on-write.ts
    │
    ├── journal/
    │   ├── journal.ts
    │   ├── journal-entry.ts
    │   └── journal-replay.ts
    │
    ├── transaction/
    │   ├── save-transaction.ts
    │   └── atomic-commit.ts
    │
    ├── dirty/
    │   ├── dirty-tracker.ts
    │   └── dirty-mask.ts
    │
    ├── migration/
    │   ├── migration.ts
    │   ├── migration-registry.ts
    │   └── migration-runner.ts
    │
    ├── recovery/
    │   ├── recovery-manager.ts
    │   ├── corruption-detector.ts
    │   └── quarantine.ts
    │
    ├── compatibility/
    │   ├── fingerprint.ts
    │   ├── compatibility-check.ts
    │   └── missing-content.ts
    │
    ├── compression/
    │   └── compressor.ts
    │
    ├── backup/
    │   ├── backup-manager.ts
    │   └── rotation.ts
    │
    ├── diagnostics/
    │   ├── save-profiler.ts
    │   ├── save-validator.ts
    │   └── save-inspector.ts
    │
    └── api/
        └── persistence-api.ts
```

---

# 249. Dependências

Agora a arquitetura base fica:

```text id="save-deps"
                    CORE
                     │
        ┌────────────┼────────────┐
        ↓            ↓            ↓
    REGISTRY      EVENT BUS    SCHEDULER
        │            │            │
        └────────────┼────────────┘
                     ↓
                PERSISTENCE
                     │
      ┌──────────────┼───────────────┐
      ↓              ↓               ↓
    BLOCK          ITEM           ENTITY
      │              │               │
      └──────────────┼───────────────┘
                     ↓
              WORLD / GAMEPLAY
```

---

# 250. A fronteira do sistema

## Persistence faz

```text
save
load
snapshot
journal
serialization
storage
versioning
migration
recovery
integrity
backup
dirty tracking
compatibility
```

## Persistence não faz

```text
World Generation
Combat
Physics
AI
Inventory Logic
Machine Logic
Economy
Rendering
Networking
```

Ele apenas persiste o estado que esses sistemas possuem.

---

# 251. Regra fundamental

Eu colocaria oficialmente:

> **Persistence System garante continuidade do estado do NEXORA. Ele não define o comportamento desse estado.**

---

# 252. Segunda regra

> **Tudo que pode ser reconstruído de forma determinística deve ser derivado antes de ser persistido.**

---

# 253. Terceira regra

> **Um save nunca deve depender de um Runtime ID que possa mudar entre execuções.**

Isso reaproveita a regra do Registry:

```text
PUBLIC ID
→ Persistence

RUNTIME ID
→ Performance

NETWORK ID
→ Session
```

---

# 254. Quarta regra

> **Falha durante o save nunca deve transformar automaticamente o save anterior válido em dados inválidos.**

Daí:

```text
old snapshot
+
new temp snapshot
+
validation
+
atomic commit
```

---

# 255. Quinta regra

> **Mods são cidadãos de primeira classe da persistência.**

Mas seus dados ficam isolados e versionados.

---

# 256. Arquitetura final

```text id="save-final-01"
                         NEXORA
                            │
                     WORLD STATE
                            │
                    DIRTY TRACKING
                            │
                     SAFE CHECKPOINT
                            │
                  ┌─────────┴─────────┐
                  ↓                   ↓
              SNAPSHOT             JOURNAL
                  │                   │
                  └─────────┬─────────┘
                            ↓
                       SERIALIZATION
                            ↓
                         COMPRESS
                            ↓
                         VERIFY
                            ↓
                      ATOMIC COMMIT
                            ↓
                      SAVE STORAGE
                            │
                     ┌──────┴──────┐
                     ↓             ↓
                  BACKUPS       RECOVERY
                                   │
                                   ↓
                              LOAD / MIGRATE
                                   │
                                   ↓
                           REGISTRY RESOLUTION
                                   │
                                   ↓
                            WORLD RESTORED
```

E com os quatro pilares que acabamos de construir:

```text id="save-final-02"
                 NEXORA FOUNDATION

       ┌──────────┬──────────┬──────────┬───────────┐
       │          │          │          │           │
       ↓          ↓          ↓          ↓           ↓
   REGISTRY    EVENT BUS   ENTITY     BLOCK        ITEM
       │          │          │          │           │
       └──────────┴──────────┴──────────┴───────────┘
                              │
                              ↓
                         PERSISTENCE
                              │
                              ↓
                           WORLD
```

Agora o NEXORA já tem uma fundação conceitual muito mais completa:

```text
CORE
├── Registry System
├── Event Bus
├── Save / Persistence
├── Entity System
├── Block System
└── Item System
```

A consequência mais importante é que os próximos sistemas já não precisam ser projetados "soltos": **Entity, Block, Item, Registry, Event Bus e Persistence formam a camada de infraestrutura sobre a qual Physics, Rendering, World Generation, Inventory, Crafting, Machines, Combat, Civilization e Mods vão operar.**
