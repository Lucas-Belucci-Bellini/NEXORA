# NEXORA

## MASTER PLAN — V1.0

> **NEXORA** é um sandbox voxel independente, modular e extensível, projetado desde o primeiro dia para possuir um sistema de conteúdo/mods nativo.
>
> O conteúdo oficial do jogo e os conteúdos externos devem utilizar os mesmos sistemas fundamentais sempre que tecnicamente possível.
>
> O objetivo não é reproduzir outro jogo existente, mas construir um **novo jogo voxel com arquitetura própria**, capaz de crescer para centenas de sistemas e conteúdos sem transformar o núcleo em um monólito.

---

# 1. VISÃO

Construir um sandbox voxel:

```text
NEXORA
│
├── Core Engine
├── World Engine
├── Renderer
├── Physics
├── Entity System
├── Item System
├── Block System
├── Inventory
├── Crafting
├── Procedural Generation
├── Save System
├── Networking
├── Server
├── UI
├── Audio
├── Registry
├── Mod Runtime
├── Scripting
└── Vanilla Content
```

Com a seguinte filosofia:

```text
ENGINE
  ↓
APIs
  ↓
REGISTRIES
  ↓
VANILLA CONTENT
  +
COMMUNITY CONTENT
```

O conteúdo do jogo não deve precisar conhecer detalhes internos do engine.

---

# 2. OBJETIVO PRINCIPAL

Criar um jogo voxel jogável e uma plataforma tecnológica que permita:

* criar blocos;
* criar itens;
* criar receitas;
* criar entidades;
* criar mundos;
* criar estruturas;
* criar sistemas;
* modificar o mundo;
* adicionar conteúdo;
* criar servidores;
* carregar mods;
* atualizar mods;
* remover mods;
* versionar conteúdo.

Tudo sem exigir alteração manual do núcleo do engine para cada novo conteúdo.

---

# 3. PRINCÍPIO FUNDAMENTAL

## MODS NÃO SÃO UM RECURSO ADICIONADO DEPOIS

O sistema de mods nasce junto com o jogo.

A arquitetura deve pensar em:

```text
Vanilla Module
=
Mod Module
```

A diferença principal é a origem e o nível de confiança, não a estrutura básica do conteúdo.

---

# 4. OBJETIVO ARQUITETURAL

A arquitetura deve permitir:

```text
adicionar conteúdo
sem modificar o Core
```

Exemplo:

```text
Core
  ↓
Registry
  ↓
Nova mod
  ↓
registra:
  ├── blocks
  ├── items
  ├── recipes
  ├── entities
  └── systems
```

Se adicionar um novo bloco exigir editar dezenas de arquivos do engine:

**a arquitetura falhou.**

---

# 5. ESCOPO DO PROJETO

O projeto será dividido em:

```text
Phase 0 — Foundation
Phase 1 — Engine
Phase 2 — World
Phase 3 — Player
Phase 4 — Content
Phase 5 — Systems
Phase 6 — Mod Runtime
Phase 7 — Multiplayer
Phase 8 — Tooling
Phase 9 — Content Expansion
Phase 10 — V1
```

---

# 6. PHASE 0 — FOUNDATION

Objetivo:

criar a base do projeto.

Criar:

```text
repository
build system
source layout
tests
CI
logging
configuration
versioning
documentation
```

Estrutura inicial:

```text
nexora/
├── engine/
├── game/
├── client/
├── server/
├── mods/
├── assets/
├── tools/
├── tests/
├── docs/
└── scripts/
```

Criar:

```text
MASTER_PLAN.md
ARCHITECTURE.md
CONTRIBUTING.md
MOD_API.md
LICENSE
README.md
```

---

# 7. PHASE 1 — ENGINE MÍNIMO

Primeiro objetivo:

```text
abrir o jogo
↓
criar janela
↓
inicializar renderer
↓
câmera
↓
input
↓
loop principal
↓
encerrar corretamente
```

Nada de multiplayer ainda.

---

# 8. RENDERER

Implementar inicialmente:

* câmera;
* frustum;
* meshes;
* materiais;
* texturas;
* iluminação básica;
* transparência;
* depth buffer;
* sky;
* chunks.

O renderer deve ser separado do restante do jogo.

---

# 9. VOXEL ENGINE

Criar:

```text
Block
Chunk
ChunkColumn
World
Region
```

O mundo não deve ser representado por um único array gigantesco.

Usar chunks.

Exemplo:

```text
World
├── Chunk
├── Chunk
├── Chunk
└── Chunk
```

---

# 10. CHUNK SYSTEM

Cada chunk deve possuir:

* dados dos blocos;
* posição;
* estado;
* geração;
* mesh;
* iluminação;
* dirty state;
* persistência.

Estados:

```text
UNLOADED
LOADING
GENERATING
READY
DIRTY
SAVING
UNLOADING
```

---

# 11. GERAÇÃO PROCEDURAL

Criar uma geração determinística:

```text
seed
↓
noise
↓
terrain
↓
biomes
↓
features
```

Mesma seed deve produzir o mesmo mundo sob as mesmas regras de geração.

Preparar arquitetura para múltiplos generators.

---

# 12. BIOMES

Criar sistema de biomas modular:

```text
Biome
├── climate
├── terrain
├── vegetation
├── blocks
├── structures
└── entities
```

O bioma não deve ser hardcoded no renderer.

---

# 13. PLAYER

Implementar:

```text
movimento
gravidade
colisão
câmera
interação
```

Depois:

```text
correr
pular
nadar
voar/debug
```

---

# 14. BLOCK SYSTEM

Criar um registry:

```text
BlockRegistry
```

Exemplo:

```text
nexora:stone
nexora:dirt
nexora:grass
nexora:sand
```

E futuramente:

```text
mod.example:steel_block
mod.example:crystal
```

O engine não deve depender de uma lista fixa de blocos.

---

# 15. ITEM SYSTEM

Criar:

```text
ItemRegistry
```

Suportar:

* item;
* stack;
* durabilidade;
* propriedades;
* tags;
* metadata.

---

# 16. INVENTÁRIO

Criar sistema independente:

```text
Inventory
Slot
ItemStack
Container
```

Preparar diferentes inventários:

```text
player
chest
machine
vehicle
NPC
```

---

# 17. CRAFTING

Criar uma API de receitas.

Exemplo conceitual:

```text
Recipe
├── id
├── inputs
├── output
├── category
└── conditions
```

Receitas não devem ficar hardcoded dentro da UI.

---

# 18. TAG SYSTEM

Criar tags para reduzir dependências diretas.

Exemplo:

```text
#wood
#stone
#metal
#food
#fuel
```

Uma receita pode exigir:

```text
#wood
```

em vez de depender de um item específico.

Isso será importante para mods.

---

# 19. ENTITY SYSTEM

Criar:

```text
Entity
LivingEntity
Player
Animal
NPC
ItemEntity
Projectile
```

O sistema deve ser modular.

---

# 20. AI

Primeiro:

```text
idle
wander
follow
flee
```

Depois:

```text
pathfinding
goals
states
perception
```

Não construir uma IA gigantesca na V1 inicial.

---

# 21. SAVE SYSTEM

Criar uma camada independente:

```text
SaveManager
WorldStorage
PlayerStorage
RegionStorage
```

Suportar:

```text
world
player
entities
chunks
metadata
```

---

# 22. VERSIONAMENTO DE SAVE

Todo save deve possuir versão:

```text
saveVersion
```

E permitir migração.

Exemplo:

```text
save v1
↓
migration
↓
save v2
```

Nunca assumir que saves antigos podem ser simplesmente lidos como se fossem atuais.

---

# 23. REGISTRY SYSTEM

O Registry será uma das peças mais importantes da arquitetura.

Criar registries para:

```text
blocks
items
entities
recipes
biomes
commands
dimensions
sounds
particles
screens
components
```

Todos com IDs estáveis:

```text
namespace:id
```

---

# 24. DATA-DRIVEN CONTENT

Sempre que possível, conteúdo deverá ser definido por dados.

Exemplo:

```text
data/
├── blocks
├── items
├── recipes
├── biomes
├── entities
└── loot
```

Isso permite criar conteúdo sem recompilar o engine sempre que possível.

---

# 25. MOD API

A Mod API deve permitir:

```text
registerBlock()
registerItem()
registerRecipe()
registerEntity()
registerBiome()
registerCommand()
registerEventHandler()
registerUI()
```

---

# 26. MOD LIFECYCLE

Todo mod deverá possuir:

```text
load
initialize
enable
disable
shutdown
```

E informações:

```text
id
name
version
author
dependencies
optionalDependencies
gameVersion
apiVersion
permissions
capabilities
```

---

# 27. DEPENDÊNCIAS

Exemplo:

```text
Mod A
├── API >= 1.0
├── Mod B >= 2.1
└── Mod C optional
```

O loader deve detectar:

* dependência ausente;
* versão incompatível;
* ciclo;
* conflito;
* duplicate ID.

---

# 28. MOD SANDBOX / PERMISSÕES

Mods não devem possuir automaticamente acesso irrestrito ao sistema.

Criar permissões como:

```text
WORLD_READ
WORLD_WRITE
FILES_READ
FILES_WRITE
NETWORK
UI
COMMANDS
SYSTEM
```

O sistema deverá permitir políticas diferentes por mod.

---

# 29. MOD LOADER

Fluxo:

```text
scan
↓
parse metadata
↓
validate
↓
resolve dependencies
↓
check conflicts
↓
load
↓
initialize
↓
register
↓
enable
```

Se um mod falhar:

não derrubar automaticamente o engine inteiro.

---

# 30. ISOLAMENTO

Requisito:

```text
mod quebrado
↓
isolado
↓
NEXORA continua iniciando
```

Idealmente:

```text
Mod A crash
Mod B funciona
Mod C funciona
Core funciona
```

---

# 31. VANILLA COMO MOD

Todo o conteúdo inicial do jogo deve utilizar a infraestrutura de conteúdo.

Conceitualmente:

```text
Vanilla
├── blocks
├── items
├── recipes
├── entities
├── biomes
└── gameplay
```

registrados através das APIs normais.

Não fazer:

```text
if vanillaBlock...
else modBlock...
```

em todo o engine.

---

# 32. RESOURCE PACK SYSTEM

Separar dados de:

```text
textures
models
sounds
fonts
UI
```

Criar sistema para carregamento de assets.

---

# 33. ASSET REGISTRY

Assets devem possuir IDs e localização controladas.

Exemplo:

```text
nexora:textures/block/stone
```

Mods:

```text
example:textures/block/steel
```

---

# 34. AUDIO

Criar:

```text
SoundRegistry
AudioManager
```

Preparar:

* música;
* sons ambientais;
* sons de blocos;
* sons de entidades;
* UI;
* efeitos.

---

# 35. PARTICLES

Criar sistema independente:

```text
Particle
ParticleEmitter
ParticleRegistry
```

---

# 36. CLIMA

Posteriormente:

```text
clear
cloudy
rain
storm
snow
```

O clima deve afetar sistemas que explicitamente escolham consumi-lo.

Não acoplar clima diretamente ao Core.

---

# 37. DIMENSIONS

Preparar:

```text
Dimension
DimensionRegistry
```

Permitindo futuramente:

```text
overworld
underground
otherworld
custom mod dimension
```

---

# 38. REDSTONE/EQUIVALENTE

Não copiar sistemas específicos de outro jogo.

Criar um sistema próprio de:

```text
signal
power
automation
logic
```

Posteriormente.

A nomenclatura e funcionamento devem ser originais do NEXORA.

---

# 39. AUTOMAÇÃO

Criar futuramente APIs para:

```text
machines
energy
pipes
storage
automation
logic
```

Isso deve ser uma extensão natural dos sistemas de itens/blocos.

---

# 40. VEHICLES

Planejar suporte futuro para:

```text
vehicle
seat
fuel
inventory
physics
```

Veículos devem ser entidades ou uma abstração própria compatível com o Entity System.

---

# 41. QUESTS / PROGRESSION

Depois do Core:

```text
Quest
Objective
Progression
Achievement
```

Tudo modular.

---

# 42. WORLD EVENTS

Criar sistema de eventos do mundo:

```text
day
night
weather
world tick
entity spawn
chunk load
chunk unload
```

---

# 43. EVENT BUS

Criar Event Bus interno.

Exemplo:

```text
WORLD_TICK
BLOCK_PLACED
BLOCK_BROKEN
ENTITY_CREATED
ENTITY_REMOVED
PLAYER_JOINED
PLAYER_LEFT
RECIPE_CRAFTED
CHUNK_LOADED
```

Os módulos podem assinar eventos através de contratos.

---

# 44. SCRIPTING

Somente depois da API principal estar estável.

Escolher uma linguagem/script runtime adequado.

Objetivo:

permitir:

```text
recipes
quests
events
content rules
gameplay extensions
```

sem exigir compilação do engine para cada alteração.

---

# 45. MULTIPLAYER

Não começar multiplayer cedo demais.

Primeiro:

```text
single player
↓
save/load
↓
deterministic world
↓
network abstraction
↓
server
↓
client
```

---

# 46. SERVER

Criar aplicação separada:

```text
nexora-server
```

O servidor deve possuir:

```text
world
entities
gameplay
authentication layer
networking
permissions
```

O cliente não deve ser a autoridade absoluta sobre o estado do mundo.

---

# 47. NETWORK PROTOCOL

Criar protocolos versionados.

Exemplo:

```text
protocolVersion
gameVersion
modCompatibility
```

Preparar negociação:

```text
client
↓
server
↓
API version
↓
mods
↓
compatibility
```

---

# 48. MODPACKS

Criar conceito de:

```text
ModPack
```

Com:

```text
manifest
mods
versions
dependencies
gameVersion
apiVersion
```

---

# 49. PERFIL DE INSTALAÇÃO

Permitir diferentes ambientes:

```text
Vanilla
Modded
Development
Server
Test
```

---

# 50. CLIENT MODS VS SERVER MODS

Classificar mods:

```text
CLIENT
SERVER
BOTH
```

Exemplo conceitual:

```text
shader = CLIENT
world generation = SERVER/BOTH
UI = CLIENT
gameplay block = BOTH
```

---

# 51. MOD COMPATIBILITY

Definir claramente:

```text
gameVersion
modApiVersion
dependencies
```

Não simplesmente assumir que um mod de uma versão funcionará na seguinte.

---

# 52. TOOLING

Criar futuramente:

```text
Nexora Launcher
Nexora Mod Manager
Nexora Server Manager
Nexora World Tool
Nexora Asset Tool
Nexora Mod SDK
```

---

# 53. MOD SDK

O objetivo final da SDK:

```text
criar projeto
↓
adicionar dependência NEXORA
↓
registrar conteúdo
↓
build
↓
package
↓
instalar
```

---

# 54. MOD TEMPLATE

Fornecer template:

```text
nexora-mod-template/
├── manifest
├── src
├── assets
├── data
├── docs
└── tests
```

---

# 55. MOD VALIDATOR

Criar ferramenta:

```text
nexora mod validate
```

Verificar:

* manifest;
* IDs;
* dependências;
* assets;
* recipes;
* registries;
* compatibilidade.

---

# 56. PROFILING

Desde cedo medir:

```text
FPS
frame time
CPU
GPU
memory
chunk generation
chunk meshing
world save
network latency
```

Nunca otimizar somente por sensação.

---

# 57. CHUNK PERFORMANCE

O engine deverá medir:

```text
chunk generation time
mesh build time
upload time
lighting time
save time
```

Depois otimizar os gargalos reais.

---

# 58. THREADING

Preparar tarefas paralelas para:

```text
chunk generation
mesh generation
world saving
asset loading
```

Sem tocar em dados do engine de maneira insegura.

---

# 59. CACHE

Criar sistemas de cache onde necessário:

```text
asset cache
chunk cache
mesh cache
registry cache
```

---

# 60. TESTING

Todo sistema importante deve possuir testes.

Prioridade:

```text
unit
integration
contract
regression
smoke
performance
```

---

# 61. TESTES DE ARQUITETURA

Criar testes que provem:

```text
mod pode adicionar bloco
mod pode adicionar item
mod pode adicionar recipe
mod pode adicionar entidade
mod pode ouvir evento
mod com erro pode ser isolado
```

---

# 62. TESTE MAIS IMPORTANTE DA MOD API

Criar um “Example Mod” oficial.

Esse mod deve demonstrar:

```text
1 bloco
1 item
1 recipe
1 entidade
1 evento
1 configuração
1 asset
```

Se isso funcionar sem modificar o Core:

a arquitetura está indo na direção correta.

---

# 63. TESTE DE VANILLA

O conteúdo oficial deve utilizar as mesmas APIs.

Exemplo:

```text
VanillaBlock
```

deve ser registrado através da infraestrutura de blocos.

Não por caminhos especiais desnecessários.

---

# 64. SAVE COMPATIBILITY

Testar:

```text
save
↓
carregar
↓
desativar mod
↓
carregar novamente
```

Definir comportamento para conteúdo removido.

Nunca corromper silenciosamente o mundo.

---

# 65. MOD REMOVAL

Criar políticas:

```text
remove safely
remove with warning
requires migration
cannot remove without world migration
```

---

# 66. CONFIGURATION

Cada mod pode possuir configuração própria:

```text
mod config
```

Mas o sistema deve possuir:

```text
Config API
```

centralizada.

---

# 67. LOGGING

Cada mod deverá possuir contexto próprio:

```text
[NEXORA]
[NEXORA:MOD]
[NEXORA:WORLD]
[NEXORA:NETWORK]
```

Facilitar diagnóstico.

---

# 68. DIAGNOSTICS

Criar:

```text
nexora doctor
```

Verificando:

* engine;
* GPU;
* drivers;
* mods;
* dependências;
* arquivos;
* save;
* configuração.

---

# 69. CRASH REPORT

Se houver crash:

gerar relatório com:

```text
version
engine
mods
platform
stack
recent events
```

Sem expor dados pessoais desnecessários.

---

# 70. ROADMAP DE CONTEÚDO

Depois que a engine estiver estável:

### Primeira coleção

```text
stone
dirt
grass
sand
wood
leaves
water
basic ores
basic tools
basic food
```

Mas criar conteúdo suficiente para provar o jogo, não centenas de itens imediatamente.

---

# 71. GAMEPLAY LOOP

A primeira versão jogável deve possuir:

```text
spawn
↓
explorar
↓
coletar
↓
craft
↓
construir
↓
sobreviver
↓
explorar novamente
```

---

# 72. PRIMEIRA VERSÃO JOGÁVEL

A primeira milestone realmente importante:

```text
NEXORA 0.1
```

deve conseguir:

```text
abrir
gerar mundo
andar
olhar
quebrar bloco
colocar bloco
inventário
crafting
salvar
carregar
```

Sem multiplayer.

---

# 73. V0.2

Adicionar:

```text
biomes
day/night
basic entities
health
food
basic progression
```

---

# 74. V0.3

Adicionar:

```text
lighting
more entities
advanced world generation
structures
weather
```

---

# 75. V0.4

Adicionar:

```text
Mod API
Mod Loader
Registry
Example Mod
```

A partir desta versão o jogo deve realmente começar a ser uma plataforma.

---

# 76. V0.5

Adicionar:

```text
Mod SDK
Mod validation
Mod dependency system
Mod configuration
```

---

# 77. V0.6

Adicionar:

```text
server prototype
network protocol
client/server
```

---

# 78. V0.7

Adicionar:

```text
modded multiplayer
server mod validation
modpack support
```

---

# 79. V0.8

Adicionar:

```text
launcher
mod manager
world manager
diagnostics
```

---

# 80. V0.9

Polimento:

```text
performance
stability
UX
accessibility
save migrations
compatibility
```

---

# 81. V1.0

NEXORA 1.0 somente quando:

```text
engine
world
player
content
save
mods
network
server
launcher
tooling
documentation
```

estiverem estáveis.

---

# 82. O QUE NÃO FAZER NA V1

Não tentar colocar imediatamente:

* MMO;
* milhares de mobs;
* centenas de dimensões;
* marketplace;
* economia complexa;
* editor 3D completo;
* VR;
* ray tracing obrigatório;
* IA gigantesca;
* procedurally generated everything;
* dezenas de linguagens de script.

Primeiro criar uma base sólida.

---

# 83. ARQUITETURA DE CÓDIGO

Regra:

```text
Core
↓
Interfaces
↓
Services
↓
Modules
↓
Content
```

Evitar:

```text
UI
↓
Core internals
```

ou:

```text
Mod A
↓
classe privada do Mod B
```

---

# 84. DEPENDENCY RULE

Dependências permitidas:

```text
Core
← interfaces

Modules
← Core APIs

Vanilla
← Module APIs

External Mods
← Public APIs
```

Evitar dependências reversas.

---

# 85. FEATURE FLAGS

Preparar:

```text
feature flags
```

para recursos experimentais.

---

# 86. EXPERIMENTAL API

Separar:

```text
Stable API
Experimental API
Internal API
```

Mods externos não devem depender de APIs internas.

---

# 87. COMPATIBILITY LAYER

Quando uma API mudar:

```text
API v1
↓
compatibility layer
↓
API v2
```

quando isso for útil.

---

# 88. DOCUMENTAÇÃO DE API

Toda API pública deve possuir:

* descrição;
* exemplo;
* versão;
* estabilidade;
* permissões;
* limitações.

---

# 89. GOVERNANÇA DOS MODS

Criar manifesto padrão.

Exemplo conceitual:

```yaml
id: example.mod
name: Example Mod
version: 1.0.0
gameVersion: 1.x
apiVersion: 1.x
side: both
dependencies:
  - nexora.api
permissions:
  - WORLD_READ
```

---

# 90. PACKAGE FORMAT

Criar formato oficial de distribuição de mods.

Exemplo conceitual:

```text
example-mod.nxm
```

ou outra extensão própria escolhida durante implementação.

O package deve conter:

```text
manifest
binary/script
assets
data
metadata
```

---

# 91. INSTALAÇÃO DE MOD

Fluxo:

```text
download
↓
validate
↓
dependency check
↓
install
↓
enable
```

---

# 92. ATUALIZAÇÃO DE MOD

Fluxo:

```text
current
↓
check
↓
new version
↓
compatibility
↓
backup
↓
update
↓
validate
```

---

# 93. ROLLBACK

Se uma atualização falhar:

```text
previous version
↓
restore
```

---

# 94. MOD STORE — FUTURO

Somente depois de o ecossistema estar maduro.

Possível futuro:

```text
mod browser
repository
ratings
downloads
updates
dependencies
```

Não é requisito da V1.

---

# 95. OPEN SOURCE / COMMUNITY

Projetar documentação para que terceiros consigam criar conteúdo.

O sucesso do NEXORA depende de:

```text
boa API
boa documentação
boas ferramentas
boa estabilidade
```

mais do que da quantidade inicial de conteúdo.

---

# 96. LICENCIAMENTO

O projeto precisa possuir licença própria e clara.

Não reutilizar código, assets, modelos, texturas ou sons de terceiros sem verificar as permissões correspondentes.

O projeto deve possuir identidade visual e assets próprios.

---

# 97. DIFERENCIAÇÃO

NEXORA não deve existir apenas como:

> “outro jogo de blocos”.

A identidade deve vir de:

```text
arquitetura modular
modding nativo
sistemas próprios
progressão própria
conteúdo próprio
mundo próprio
ferramentas próprias
```

---

# 98. MASTER TEST

Um dia deverá existir um teste que prove:

```text
criar novo mod
↓
registrar bloco
↓
registrar item
↓
registrar recipe
↓
adicionar comportamento
↓
carregar mod
↓
gerar mundo
↓
jogar
↓
salvar
↓
fechar
↓
abrir
↓
continuar
```

sem editar o Core do NEXORA.

Esse será um dos maiores testes da arquitetura.

---

# 99. DEFINIÇÃO DE SUCESSO

NEXORA não será considerado bem-sucedido por possuir:

```text
mais blocos
mais mobs
mais biomas
```

O sucesso será:

> **ser possível adicionar sistemas novos sem precisar reconstruir o engine inteiro.**

---

# 100. REGRA FINAL

Nunca transformar o NEXORA em um monólito.

Sempre perguntar:

```text
isso pertence ao Core?
ou
isso deveria ser um módulo?
```

Sempre preferir:

```text
Core pequeno
+
APIs fortes
+
Registry
+
Module Runtime
+
conteúdo modular
```

sobre:

```text
Core enorme
+
if para cada mod
+
código acoplado
```

---

# ROADMAP RESUMIDO

```text
PHASE 0
Foundation
        ↓
PHASE 1
Engine
        ↓
PHASE 2
Voxel World
        ↓
PHASE 3
Player
        ↓
PHASE 4
Gameplay
        ↓
PHASE 5
Mod API
        ↓
PHASE 6
Mod Loader
        ↓
PHASE 7
Mod SDK
        ↓
PHASE 8
Multiplayer
        ↓
PHASE 9
Tooling
        ↓
PHASE 10
V1.0
```

---

# PRIMEIRO MARCO

Não começar fazendo o jogo completo.

O primeiro objetivo é criar:

```text
NEXORA 0.0.1
```

com:

```text
✅ janela
✅ renderer
✅ câmera
✅ input
✅ chunk
✅ bloco
✅ mundo mínimo
✅ player mínimo
✅ logging
✅ testes
✅ build reproducível
```

Depois:

```text
0.0.2
```

com mundo realmente jogável.

Depois:

```text
0.1
```

com o primeiro gameplay loop.

Só depois:

```text
0.2+
```

começar a expandir o ecossistema.

---

# FRASE DO PROJETO

> **NEXORA — Build the world. Extend the rules.**
