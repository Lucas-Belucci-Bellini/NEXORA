Perfeito. O **Registry System** é provavelmente o próximo sistema mais importante, porque ele vai servir de **ponte entre o Core e praticamente tudo que pode ser registrado no NEXORA**.

A regra central:

> **Registry System transforma IDs públicos em definições runtime estáveis, resolve dependências, valida conteúdo e fornece lookup eficiente para os demais sistemas.**

Ele não deve conhecer apenas Blocks e Items. Deve ser um sistema genérico capaz de registrar praticamente qualquer tipo extensível do NEXORA.

# NEXORA — REGISTRY SYSTEM

## 1. Princípio central

```text
                    NEXORA CORE
                         │
                  REGISTRY SYSTEM
                         │
          ┌──────────────┼──────────────┐
          ↓              ↓              ↓
      REGISTRY         REGISTRY       REGISTRY
       DOMAIN           TYPE           DATA
          │              │              │
          └──────────────┼──────────────┘
                         ↓
                    PUBLIC API
                         │
      ┌──────────────────┼───────────────────┐
      ↓                  ↓                   ↓
   VANILLA             MOD A               MOD B
```

O Registry System deve permitir:

```text
register
lookup
resolve
validate
freeze
iterate
snapshot
migrate
```

---

# 2. Por que Registry é necessário?

Sem Registry, acabaríamos fazendo:

```text
if id == "stone"
if id == "iron"
if id == "diamond"
if id == "machine"
```

ou:

```text
Map<string, Object>
```

espalhados por dezenas de sistemas.

Isso rapidamente vira caos.

O Registry centraliza:

```text
ID
→
TYPE
→
DEFINITION
→
RUNTIME HANDLE
```

---

# 3. O que pode ser registrado?

Praticamente tudo que for conteúdo ou extensão.

```text
Blocks
Items
Entities
Components
Capabilities
Biomes
Fluids
Materials
Recipes
Machines
Energy Types
Dimensions
Structures
Loot Tables
Sounds
Particles
Animations
Tags
Attributes
Damage Types
Status Effects
Professions
Factions
Quest Types
Research Nodes
Vehicles
World Events
```

E no futuro:

```text
Technology
Magic
Spacecraft Parts
Planets
Species
Civilizations
```

---

# 4. Registry ≠ Database

Isso precisa ficar explícito.

Registry é:

```text
runtime definition system
```

Não é um banco de dados de gameplay.

Exemplo:

```text
ItemRegistry
```

conhece a definição de:

```text
nexora:iron_ingot
```

Mas não deve armazenar:

```text
"Lucas possui 53 iron_ingots"
```

Isso pertence a:

```text
Inventory
Persistence
Player
```

---

# 5. Registry ≠ Save System

Registry resolve:

```text
o que é este ID?
```

Save resolve:

```text
qual estado esse objeto tinha?
```

Exemplo:

```text
nexora:iron_pickaxe
```

vem do Registry.

Já:

```text
durability = 37
```

vem do estado persistido.

---

# 6. RegistryID

Todos os elementos registrados precisam possuir identidade pública.

Formato:

```text
namespace:id
```

Exemplo:

```text
nexora:stone
nexora:iron_ingot
nexora:zombie
examplemod:steel_machine
```

---

# 7. Namespace

Namespace identifica o proprietário lógico.

Vanilla:

```text
nexora
```

Mod:

```text
examplemod
```

Não permitir colisão:

```text
nexora:stone
examplemod:stone
```

são IDs diferentes.

---

# 8. Registry Type

Cada registry representa um domínio.

```text
RegistryType<BlockDefinition>
RegistryType<ItemDefinition>
RegistryType<EntityDefinition>
```

Conceitualmente:

```text
Registry<T>
```

---

# 9. RegistryKey

Separar:

```text
RegistryType
```

de:

```text
RegistryKey
```

Exemplo:

```text
RegistryType = block
RegistryKey = nexora:stone
```

Isso elimina ambiguidades.

---

# 10. Registry Entry

Cada entrada poderia ser conceitualmente:

```text
RegistryEntry<T>

key
value
runtimeId
networkId
version
owner
metadata
```

---

# 11. Runtime ID

Para performance:

```text
nexora:stone
        ↓
runtime ID 17
```

Lookup comum pode usar:

```text
RuntimeID → Definition
```

muito mais rápido que manipular strings.

---

# 12. Persistência

Save nunca deve depender apenas do Runtime ID.

Errado:

```text
17 = stone
```

porque outro carregamento pode atribuir:

```text
17 = dirt
```

Correto:

```text
nexora:stone
```

ou outro identificador estável.

---

# 13. Network ID

Durante multiplayer pode existir:

```text
NetworkID
```

que é específico daquela sessão.

```text
Public ID
↓
Network ID
```

O servidor fornece o mapping.

---

# 14. Registry Lifecycle

O Registry possui fases:

```text
CREATED
↓
REGISTERING
↓
VALIDATING
↓
RESOLVING
↓
FREEZING
↓
READY
↓
SHUTDOWN
```

---

# 15. Registration Phase

Mods e conteúdo oficial podem registrar:

```text
Block
Item
Entity
Biome
Fluid
...
```

durante:

```text
REGISTERING
```

---

# 16. Validation Phase

Depois:

```text
VALIDATING
```

O sistema verifica:

```text
ID válido?
duplicado?
tipo correto?
dependências?
schema?
referências?
```

---

# 17. Resolution Phase

Depois resolve referências.

Exemplo:

```text
machine
→ recipe
→ item
→ energy type
→ fluid
```

Se alguma referência não existir:

```text
invalid dependency
```

---

# 18. Freeze

Após resolver tudo:

```text
FREEZE
```

O Registry passa a ser somente leitura na operação normal.

Isso permite otimização.

---

# 19. Por que congelar?

Porque durante runtime podemos assumir:

```text
Registry não muda
```

e construir:

```text
dense arrays
lookup tables
indexes
cache
```

sem medo de alterações estruturais.

---

# 20. Dynamic Registration

Ainda podemos permitir registros tardios em casos específicos.

Mas isso deve ser explicitamente suportado por um mecanismo diferente:

```text
DynamicRegistry
```

e não quebrar o registry principal.

---

# 21. Static vs Dynamic Registry

### Static

Conteúdo carregado no boot:

```text
Blocks
Items
Biomes
Dimensions
Entity Types
```

### Dynamic

Conteúdo criado durante runtime:

```text
Player names
World instances
Organizations
Temporary effects
Generated discoveries
```

Esses últimos normalmente **não** são Registry entries.

---

# 22. Registry Factories

Pode existir:

```text
RegistryFactory
```

responsável por criar registries.

Exemplo:

```text
createRegistry<BlockDefinition>()
createRegistry<ItemDefinition>()
```

---

# 23. Registry Manager

Uma camada superior:

```text
RegistryManager
```

mantém todos os registries.

Exemplo:

```text
RegistryManager
├── BlockRegistry
├── ItemRegistry
├── EntityRegistry
├── BiomeRegistry
├── FluidRegistry
├── RecipeRegistry
└── ...
```

---

# 24. Registry Registry

Eu evitaria criar um sistema recursivo confuso chamado literalmente "RegistryRegistry".

Melhor:

```text
RegistryManager
```

ou:

```text
RegistryCatalog
```

Ele conhece os domínios existentes.

---

# 25. Public Registry API

Interface principal:

```text
IRegistry<T>
```

operações:

```text
register()
get()
contains()
resolve()
iterate()
size()
freeze()
```

---

# 26. Registration

Conceito:

```text
registry.register(
    key,
    definition
)
```

Exemplo:

```text
blockRegistry.register(
    "nexora:stone",
    stoneDefinition
)
```

---

# 27. Duplicate Detection

Não permitir silenciosamente:

```text
nexora:stone
```

ser registrado duas vezes.

Pode resultar em:

```text
DuplicateRegistryKey
```

---

# 28. Override Policy

Mods podem precisar substituir definições.

Isso deve ser explicitamente controlado.

Políticas:

```text
REJECT
REPLACE
PATCH
EXTEND
SHADOW
```

Por padrão:

```text
REJECT
```

---

# 29. Replace

Substituição explícita:

```text
examplemod overrides X
```

deve registrar:

```text
override metadata
```

para debug.

---

# 30. Patch

Mais seguro:

```text
base definition
+
patch
```

em vez de copiar tudo.

---

# 31. Extension

Um mod pode adicionar capabilities ou componentes sem substituir a definição inteira.

Exemplo:

```text
stone
+
custom hardness modifier
```

---

# 32. Ownership

Cada entrada deve saber de onde veio:

```text
owner = nexora
owner = examplemod
```

Isso ajuda:

```text
debug
permissions
unloading
compatibility
```

---

# 33. Mod Ownership

Quando um mod é desativado:

```text
Registry
 ↓
entries owned by mod
```

podem ser identificadas.

Mas remover conteúdo de um mundo existente é outro problema e precisa passar por:

```text
Save/Persistence
Missing Content
Migration
```

---

# 34. Version

Cada registro pode possuir:

```text
contentVersion
schemaVersion
```

---

# 35. Registry Version

O próprio Registry pode possuir:

```text
registryVersion
```

para identificar mudanças estruturais.

---

# 36. Snapshot

Registry deve conseguir produzir:

```text
RegistrySnapshot
```

Exemplo:

```text
BlockRegistrySnapshot
ItemRegistrySnapshot
EntityRegistrySnapshot
```

---

# 37. Por que Snapshot?

Muito útil para:

```text
save compatibility
multiplayer
debugging
replay
migration
mod compatibility
```

---

# 38. Registry Fingerprint

Pode calcular um fingerprint:

```text
RegistryFingerprint
```

sobre:

```text
keys
versions
definitions
```

Isso permite detectar diferenças.

---

# 39. Multiplayer Handshake

Cliente:

```text
RegistryFingerprint
```

Servidor:

```text
RegistryFingerprint
```

Se incompatível:

```text
version mismatch
```

---

# 40. Registry Synchronization

O servidor pode enviar:

```text
public registry mapping
```

para o cliente:

```text
nexora:stone → networkId 12
```

---

# 41. Client Missing Content

Se o cliente não conhece:

```text
examplemod:laser_block
```

não deve interpretar incorretamente como outro bloco.

Deve:

```text
unknown content
```

e seguir a política definida.

---

# 42. Content Negotiation

Pode haver:

```text
RegistryCompatibility
```

com:

```text
required
optional
unsupported
```

---

# 43. Registry Dependencies

Uma entrada pode depender de outra.

Exemplo:

```text
Machine
 ↓
Fluid
 ↓
Energy
 ↓
Item
```

O Registry deve conseguir validar essas referências.

---

# 44. Dependency Graph

Criar:

```text
RegistryDependencyGraph
```

Exemplo:

```text
Machine
 ├── Item
 ├── Fluid
 └── Energy
```

---

# 45. Circular Dependency

Detectar:

```text
A → B
B → C
C → A
```

e rejeitar ou tratar explicitamente.

---

# 46. Load Ordering

Com dependências:

```text
Material
 ↓
Block
 ↓
Item
 ↓
Recipe
 ↓
Machine
```

A ordem pode ser calculada automaticamente.

---

# 47. Registry Phases

Uma estrutura melhor:

```text
Phase 1
Core registrations

Phase 2
Content registrations

Phase 3
Cross-content registrations

Phase 4
Validation

Phase 5
Resolution

Phase 6
Freeze
```

---

# 48. Deferred References

Uma definição pode referenciar algo que será registrado depois.

Exemplo:

```text
machine.recipe = examplemod:steel_recipe
```

durante register.

A referência é resolvida só na fase:

```text
RESOLVING
```

---

# 49. Registry Handle

Para hot path:

```text
RegistryHandle<T>
```

em vez de armazenar strings constantemente.

Conceito:

```text
"nexora:stone"
      ↓
RegistryHandle<Block>
```

---

# 50. Typed Handles

Evitar:

```text
Handle("stone")
```

que pode apontar para qualquer coisa.

Preferir:

```text
BlockHandle
ItemHandle
EntityHandle
```

ou genéricos fortemente tipados.

---

# 51. Invalid Handle

Pode existir:

```text
MissingHandle
InvalidHandle
```

para situações de conteúdo ausente.

---

# 52. Lazy Resolution

Algumas referências podem ser resolvidas apenas quando usadas.

Útil para reduzir custo de boot.

---

# 53. Eager Resolution

Outras devem falhar no boot se forem inválidas.

Exemplo:

```text
core required dependency
```

---

# 54. Validation Severity

Erros:

```text
ERROR
```

Avisos:

```text
WARNING
```

Informação:

```text
INFO
```

---

# 55. Quarantine

Conteúdo inválido pode ser colocado em:

```text
QUARANTINED
```

em vez de derrubar todo o jogo.

---

# 56. Registry Diagnostics

Comando:

```text
nexora registry list
```

---

# 57. Registry Inspect

```text
nexora registry inspect block nexora:stone
```

mostrar:

```text
owner
version
runtime ID
capabilities
references
```

---

# 58. Registry Search

```text
nexora registry search iron
```

---

# 59. Registry Dump

Pode exportar:

```text
registry_snapshot.json
```

para debugging.

---

# 60. Registry Diff

Comparar:

```text
snapshot A
vs
snapshot B
```

mostrar:

```text
added
removed
changed
```

---

# 61. Registry Diff em Updates

Muito útil para mods:

```text
v1.2
vs
v1.3
```

---

# 62. Tags Registry

Tags são tão importantes que devem ter tratamento próprio.

```text
TagRegistry<T>
```

Exemplo:

```text
#metal
#ore
#flammable
```

---

# 63. Tag Membership

```text
tag contains key
```

lookup otimizado.

---

# 64. Tag Expansion

Uma definição pode registrar:

```text
tags:
    metal
    building_material
```

O Registry resolve para conjuntos eficientes.

---

# 65. Nested Tags

Pode permitir:

```text
#metal
 └── #conductive
```

mas isso deve ser controlado para evitar ciclos.

---

# 66. Tag Cycle Detection

```text
A → B
B → A
```

deve ser detectado.

---

# 67. Data Registries

Alguns conteúdos não são objetos complexos.

Podem ser:

```text
DamageType
StatusEffect
EnergyType
```

e ainda assim usar Registry.

---

# 68. Registry Schema

Cada Registry pode declarar:

```text
expected type
validation schema
serializer
migration strategy
```

---

# 69. Generic Registry

Conceitualmente:

```text
Registry<T>
```

com:

```text
T = BlockDefinition
T = ItemDefinition
T = FluidDefinition
T = EntityDefinition
```

---

# 70. Specialized Registry

Alguns precisam de regras próprias.

Exemplo:

```text
EntityRegistry
```

pode validar:

```text
spawn profile
components
dimensions
```

Mas ainda utiliza a infraestrutura genérica.

---

# 71. Registry Events

Eventos:

```text
RegistryCreated
EntryRegistered
EntryRejected
EntryReplaced
RegistryValidated
RegistryFrozen
RegistryLoaded
RegistryMigrated
```

---

# 72. Event Bus Integration

O Registry publica no:

```text
Event Bus
```

mas Event Bus não depende de conteúdo específico.

---

# 73. Thread Safety

Durante boot:

```text
register phase
```

pode ser controlada.

Durante runtime:

```text
read-only
```

permite leituras concorrentes.

---

# 74. Mutable Registry

Quando necessário:

```text
MutableRegistry
```

para construção inicial.

Depois:

```text
FrozenRegistry
```

para runtime.

---

# 75. Copy-on-write

Caso o jogo eventualmente permita mudanças controladas:

```text
old snapshot
 ↓
new snapshot
```

em vez de modificar estruturas em uso.

---

# 76. Hot Reload

O Registry pode futuramente suportar:

```text
hot reload
```

para conteúdo data-driven.

Mas somente alguns tipos devem permitir isso.

---

# 77. Hot Reload Policy

Cada Registry define:

```text
HOT_RELOAD_SAFE
HOT_RELOAD_RESTRICTED
NO_HOT_RELOAD
```

---

# 78. World Safety

Não permitir hot reload arbitrário de algo que já está persistido sem:

```text
migration
```

---

# 79. Development Mode

No modo desenvolvimento:

```text
hot reload
verbose diagnostics
registry diff
validation
```

podem ficar habilitados.

---

# 80. Production Mode

Produção:

```text
frozen
optimized
minimal diagnostics
```

---

# 81. Registry Memory

Estruturas recomendadas:

```text
HashMap<Key, RuntimeID>
Array<Definition>
Array<EntryMetadata>
```

Lookup:

```text
Key → RuntimeID → Array
```

---

# 82. Why Two-Level Lookup?

Evita armazenar objetos grandes dentro do mapa hash.

```text
String Key
 ↓
small integer
 ↓
dense array
```

---

# 83. Integer Runtime IDs

O runtime pode usar:

```text
u16
u32
```

dependendo da escala prevista.

NEXORA deve evitar limitar cedo demais.

---

# 84. Registry Cardinality

Como o projeto pode ter:

```text
2.000+ mobs
milhares de items
milhares de blocks
```

é melhor projetar registries para dezenas ou centenas de milhares de entradas sem redesign.

---

# 85. Registry Locality

Runtime arrays devem favorecer cache locality.

---

# 86. Fast Path

O hot path ideal:

```text
RuntimeID
 ↓
array[index]
 ↓
definition
```

---

# 87. Slow Path

Lookup por string:

```text
namespace:id
```

é para:

```text
boot
commands
load/save
mod loading
debug
```

não para cada frame.

---

# 88. Persistent Mapping

Save pode registrar:

```text
Public ID
Definition Version
```

---

# 89. Runtime Mapping

Execução:

```text
Public ID
 ↓
Runtime ID
```

---

# 90. Network Mapping

Multiplayer:

```text
Public ID
 ↓
Network ID
```

Três espaços diferentes:

```text
Persistence
Runtime
Network
```

Isso precisa ficar explícito na arquitetura.

---

# 91. Registry Migration

Quando um ID muda:

```text
old:
examplemod:steel

new:
examplemod:steel_ingot
```

registrar:

```text
RegistryMigration
```

---

# 92. Alias

Pode existir:

```text
alias
```

para compatibilidade:

```text
old_name → new_name
```

---

# 93. Alias Lifetime

Aliases podem ser:

```text
temporary
deprecated
permanent
```

---

# 94. Deprecated Entries

Uma entrada pode estar:

```text
ACTIVE
DEPRECATED
REMOVED
```

com warnings no desenvolvimento.

---

# 95. Registry Removal

Remover conteúdo deve ser perigoso.

Antes:

```text
check references
check saves
check dependencies
```

---

# 96. Safe Removal

Se removido:

```text
MissingContent
```

deve preservar dados quando necessário.

---

# 97. Cross Registry References

Exemplo:

```text
Item
 ↓
Block
```

quando necessário.

Mas tentar minimizar acoplamento.

---

# 98. Registry Graph

Globalmente:

```text
               REGISTRY MANAGER
                      │
       ┌──────────────┼─────────────┐
       ↓              ↓             ↓
     Block          Item          Entity
       │              │             │
       └───────┬──────┘             │
               ↓                    ↓
            Recipe                AI
```

---

# 99. Registry Contracts

Outros sistemas não deveriam acessar diretamente a implementação.

Exemplo:

```text
BlockSystem
 ↓
IBlockRegistry
```

e não:

```text
BlockSystem
 ↓
HashMap
```

---

# 100. Mod API

Mods usam:

```text
Registry API
```

para registrar conteúdo.

Exemplo conceitual:

```text
mod.onRegister(context)
```

e:

```text
context.blocks.register(...)
context.items.register(...)
context.entities.register(...)
```

---

# 101. Registration Context

Criar:

```text
RegistrationContext
```

com:

```text
modId
version
permissions
registries
logger
schema
```

---

# 102. Mod Permissions

Um mod pode receber acesso:

```text
READ_REGISTRY
REGISTER_CONTENT
PATCH_CONTENT
OVERRIDE_CONTENT
```

---

# 103. Default Mod Permission

Por padrão:

```text
REGISTER_CONTENT
```

mas não:

```text
OVERRIDE
```

---

# 104. Registry Security

Validar:

```text
malformed IDs
excessive registrations
invalid schemas
cyclic dependencies
oversized metadata
```

---

# 105. Namespace Protection

Mods não devem registrar:

```text
nexora:*
```

salvo conteúdo oficial/autorizado.

---

# 106. Namespace Validation

Regex conceitual:

```text
namespace:id
```

com caracteres permitidos bem definidos.

---

# 107. Case Sensitivity

Eu recomendaria:

```text
lowercase only
```

para IDs públicos.

Evita:

```text
Stone
stone
STONE
```

como IDs diferentes.

---

# 108. Unicode

Evitaria Unicode nos IDs públicos.

Permitir apenas um conjunto simples e estável.

---

# 109. Registry Metadata

Cada entrada pode ter:

```text
createdBy
createdVersion
sourcePack
modId
schemaVersion
```

---

# 110. Debug Metadata

Não precisa entrar no save final.

Pode existir apenas no ambiente de desenvolvimento.

---

# 111. Content Pack Integration

Registry pode carregar:

```text
Content Pack
```

que fornece:

```text
definitions
tags
recipes
assets
```

---

# 112. Resource Pack ≠ Content Pack

Resource Pack:

```text
textures
models
sounds
UI
```

Content Pack:

```text
definitions
recipes
world content
```

Registry deve carregar conteúdo, não render assets diretamente.

---

# 113. Data Pack-like System

NEXORA pode futuramente possuir:

```text
Data Pack
```

que registra ou modifica:

```text
recipes
loot
tags
worldgen data
```

Registry é um dos destinos.

---

# 114. Registry Loader

Criar:

```text
RegistryLoader
```

responsável por:

```text
read
parse
validate
register
```

---

# 115. Loader Sources

Fontes podem ser:

```text
Vanilla package
Mod package
Data pack
Server data
Generated content
```

---

# 116. Parsing

Registry não deveria conhecer JSON/YAML/etc. diretamente.

Melhor:

```text
Parser
 ↓
Normalized Definition
 ↓
Registry
```

---

# 117. Schema Validation

Cada conteúdo data-driven possui schema.

```text
definition
 ↓
schema validator
 ↓
normalized
 ↓
registry
```

---

# 118. Normalization

Exemplo:

```text
"stack": 64
```

pode virar internamente:

```text
maxStackSize = 64
```

---

# 119. Defaults

Definitions podem possuir defaults:

```text
maxStackSize = 64
weight = 1
```

sem exigir todos os campos.

---

# 120. Explicit vs Default

Debug deve indicar:

```text
explicit
defaulted
inherited
```

para facilitar mod development.

---

# 121. Inheritance

Evitar herança de registry definitions.

Preferir:

```text
composition
templates
patches
```

---

# 122. Templates

Exemplo:

```text
metal_ingot_template
```

e:

```text
steel_ingot
```

usa o template.

---

# 123. Template Resolution

Templates devem ser resolvidos antes do Freeze.

---

# 124. Registry Compiler

Um conceito interessante para NEXORA:

```text
CONTENT DATA
     ↓
REGISTRY COMPILER
     ↓
VALIDATED RUNTIME REGISTRY
```

Isso pode produzir estruturas extremamente rápidas.

---

# 125. Development Compiler

Durante desenvolvimento:

```text
source data
→ runtime structures
```

---

# 126. Production Build

Futuramente:

```text
mod/content
 ↓
compile
 ↓
optimized registry data
```

reduz tempo de boot.

---

# 127. Registry Cache

Podemos ter cache de registries compilados.

Mas cache deve invalidar quando:

```text
content hash
schema
game version
mod version
```

mudarem.

---

# 128. Content Hash

Cada pacote pode possuir:

```text
contentHash
```

---

# 129. Deterministic Registry

Mesmos:

```text
game version
mod set
content version
```

devem produzir o mesmo Registry lógico.

Isso ajuda muito no multiplayer.

---

# 130. Registry Determinism

A ordem de registro física não deve alterar o significado.

Runtime IDs podem variar, mas o conteúdo lógico deve ser equivalente.

---

# 131. Stable Ordering

Para maior determinismo:

```text
namespace
id
```

podem ser ordenados em certas fases.

---

# 132. Registry Ordering

Não assumir:

```text
runtime ID 0 = stone
```

nem em código nem em save.

---

# 133. Registry Indexes

Além do ID, criar índices opcionais:

```text
byTag
byOwner
byCapability
byMaterial
```

---

# 134. Capability Index

Exemplo:

```text
all items with IFluidContainer
```

pode ser consultado rapidamente.

---

# 135. Tag Index

```text
#metal
 ↓
Runtime IDs
```

---

# 136. Search Index

Busca textual fica fora do hot path e pode possuir índice próprio.

---

# 137. Registry Query

```text
IRegistryQuery<T>
```

permite:

```text
byId()
byTag()
byCapability()
byOwner()
```

---

# 138. Registry Iterator

```text
forEach()
```

deve ser seguro após Freeze.

---

# 139. Registry Snapshot Query

Snapshots devem ser imutáveis.

Ótimos para:

```text
parallel systems
tests
debug
networking
```

---

# 140. Registry Events vs Event Bus

Registry publica eventos.

Mas:

```text
Registry Event
```

não precisa ficar acoplado ao gameplay.

O Event Bus geral pode distribuir.

---

# 141. Startup Error Policy

Dependendo do erro:

```text
CORE ERROR
→ abort boot

MOD ERROR
→ isolate mod

OPTIONAL CONTENT ERROR
→ quarantine

MISSING OPTIONAL REFERENCE
→ warning
```

---

# 142. Registry Health

Durante boot gerar:

```text
RegistryHealthReport
```

com:

```text
registered
failed
missing
deprecated
overridden
quarantined
```

---

# 143. Mod Health

Também:

```text
ModRegistryReport
```

---

# 144. Developer Diagnostics

Exemplo:

```text
Loaded 4,283 items
Loaded 3,917 blocks
Loaded 2,106 entities
Loaded 1,244 biomes
```

---

# 145. Registry Metrics

Métricas:

```text
registration time
validation time
resolution time
freeze time
memory usage
entry count
```

---

# 146. Performance Goal

Depois do boot:

```text
registry lookup
```

deve ser extremamente barato.

Idealmente:

```text
string lookup
→ development/slow path

runtime ID lookup
→ hot path
```

---

# 147. Entity Registry

Esse sistema conversa diretamente com o Entity System recém-projetado:

```text
EntityRegistry
 ↓
EntityDefinition
 ↓
EntityManager
 ↓
EntityInstance
```

---

# 148. Block Registry

```text
BlockRegistry
 ↓
BlockDefinition
 ↓
BlockState
 ↓
BlockStorage
```

---

# 149. Item Registry

```text
ItemRegistry
 ↓
ItemDefinition
 ↓
ItemStack / ItemInstance
```

---

# 150. Fluid Registry

```text
FluidRegistry
 ↓
FluidDefinition
 ↓
FluidState
```

---

# 151. Biome Registry

```text
BiomeRegistry
 ↓
BiomeDefinition
 ↓
WorldGen / Climate
```

---

# 152. Machine Registry

```text
MachineRegistry
 ↓
MachineDefinition
 ↓
MachineInstance
```

---

# 153. Recipe Registry

```text
RecipeRegistry
 ↓
RecipeDefinition
 ↓
Crafting / Machines
```

---

# 154. Dimension Registry

```text
DimensionRegistry
 ↓
DimensionDefinition
 ↓
World Generator
```

---

# 155. Structure Registry

```text
StructureRegistry
 ↓
StructureDefinition
 ↓
WorldGen / Civilization
```

---

# 156. Damage Registry

O Combat System pode possuir:

```text
DamageTypeRegistry
```

Exemplo:

```text
physical
thermal
kinetic
chemical
electric
magical
dimensional
```

---

# 157. Status Registry

```text
StatusEffectRegistry
```

---

# 158. Energy Registry

```text
EnergyTypeRegistry
```

---

# 159. Material Registry

Compartilhado conceitualmente:

```text
MaterialRegistry
```

Pode ser consumido por:

```text
Block
Item
Physics
Energy
Crafting
Machines
```

---

# 160. Registry não deve virar "God System"

Apesar de conhecer muitos registries:

```text
RegistryManager
```

não deve implementar:

```text
WorldGen
Combat
Machine logic
Crafting
AI
```

Ele apenas organiza identidade e definições.

---

# 161. Estrutura de código

Eu faria:

```text id="reg-code-01"
src/
└── registry/
    ├── core/
    │   ├── registry.ts
    │   ├── registry-key.ts
    │   ├── registry-entry.ts
    │   ├── registry-handle.ts
    │   ├── registry-type.ts
    │   └── registry-snapshot.ts
    │
    ├── manager/
    │   ├── registry-manager.ts
    │   └── registry-catalog.ts
    │
    ├── lifecycle/
    │   ├── registration-phase.ts
    │   ├── validation-phase.ts
    │   ├── resolution-phase.ts
    │   └── freeze-phase.ts
    │
    ├── dependency/
    │   ├── dependency-graph.ts
    │   └── dependency-resolver.ts
    │
    ├── validation/
    │   ├── registry-validator.ts
    │   ├── key-validator.ts
    │   └── schema-validator.ts
    │
    ├── migration/
    │   ├── registry-migration.ts
    │   ├── alias.ts
    │   └── deprecated.ts
    │
    ├── snapshot/
    │   ├── fingerprint.ts
    │   └── diff.ts
    │
    ├── query/
    │   ├── registry-query.ts
    │   └── indexes.ts
    │
    ├── tags/
    │   ├── tag-registry.ts
    │   └── tag-index.ts
    │
    ├── loading/
    │   ├── registry-loader.ts
    │   ├── registration-context.ts
    │   └── content-source.ts
    │
    ├── diagnostics/
    │   ├── health-report.ts
    │   └── registry-inspector.ts
    │
    └── api/
        └── registry-api.ts
```

---

# 162. APIs principais

```text id="reg-api-01"
IRegistry<T>

register()
get()
resolve()
contains()
iterate()
size()
freeze()
snapshot()
```

Manager:

```text id="reg-api-02"
IRegistryManager

getRegistry()
getRegistryType()
listRegistries()
snapshotAll()
```

---

# 163. Registration Context

```text id="reg-api-03"
IRegistrationContext

modId
modVersion
register()
lookup()
createHandle()
log()
validate()
```

---

# 164. Registry Handle

```text id="reg-api-04"
RegistryHandle<T>

key
runtimeId
resolve()
isValid()
isMissing()
```

---

# 165. Registry Snapshot

```text id="reg-api-05"
RegistrySnapshot<T>

version
fingerprint
entries
metadata
```

Imutável.

---

# 166. Master Registry Map

A arquitetura pode finalmente ficar:

```text id="reg-map-final"
                         REGISTRY SYSTEM
                                │
                         REGISTRY MANAGER
                                │
       ┌──────────────┬─────────┼─────────┬──────────────┐
       ↓              ↓         ↓         ↓              ↓
     BLOCK           ITEM     ENTITY     FLUID          BIOME
       │              │         │         │              │
       ↓              ↓         ↓         ↓              ↓
   BlockDef       ItemDef   EntityDef FluidDef       BiomeDef
```

E depois:

```text id="reg-map-ext"
       ┌──────────────┬──────────────┬──────────────┐
       ↓              ↓              ↓              ↓
    RECIPE         MACHINE        DIMENSION      STRUCTURE
       ↓              ↓              ↓              ↓
   RecipeDef      MachineDef    DimensionDef   StructureDef
```

---

# 167. Boot completo

Agora conseguimos definir um boot muito mais sólido:

```text id="reg-boot-01"
NEXORA BOOT
    ↓
CORE
    ↓
REGISTRY MANAGER
    ↓
CREATE REGISTRIES
    ↓
LOAD CORE TYPES
    ↓
LOAD VANILLA
    ↓
LOAD MODS
    ↓
REGISTER CONTENT
    ↓
VALIDATE
    ↓
RESOLVE DEPENDENCIES
    ↓
BUILD INDEXES
    ↓
CREATE SNAPSHOT
    ↓
FREEZE
    ↓
WORLD SYSTEM
    ↓
GAME
```

---

# 168. Erro importante a evitar

Não fazer:

```text
Mod
 ↓
modify registry forever
```

O runtime precisa chegar a:

```text
Frozen Registry
```

e funcionar de forma previsível.

---

# 169. Regra de ouro

Eu colocaria no documento oficial:

> **Registry System é o sistema de identidade e resolução de conteúdo do NEXORA. Ele sabe qual definição corresponde a cada ID, mas não sabe o significado de gameplay dessa definição.**

---

# 170. Segunda regra

> **Registries definem conteúdo; sistemas executam comportamento.**

Exemplo:

```text
ItemRegistry
→ define Pickaxe

Tool System
→ sabe minerar

Build System
→ sabe quebrar bloco

Durability
→ controla desgaste
```

---

# 171. Terceira regra

> **Runtime IDs são otimizações. IDs públicos são identidade.**

Portanto:

```text
Public ID
→ Persistence

Runtime ID
→ CPU / Memory

Network ID
→ Multiplayer Session
```

---

# 172. Ordem de implementação

Eu dividiria assim:

```text id="reg-order"
REG-0    Core Contracts
REG-1    RegistryKey
REG-2    RegistryType
REG-3    RegistryEntry
REG-4    Generic Registry
REG-5    RegistryManager
REG-6    Register
REG-7    Lookup
REG-8    Runtime IDs
REG-9    Handles
REG-10   Validation
REG-11   Registration Phases
REG-12   Freeze
REG-13   Dependency Graph
REG-14   Resolution
REG-15   Tags
REG-16   Indexes
REG-17   Snapshots
REG-18   Fingerprints
REG-19   Diff
REG-20   Versioning
REG-21   Migration
REG-22   Aliases
REG-23   Missing Content
REG-24   Mod Ownership
REG-25   Mod Registration Context
REG-26   Content Loading
REG-27   Network Mapping
REG-28   Registry Synchronization
REG-29   Diagnostics
REG-30   Security
REG-31   Hot Reload
REG-32   Registry Compiler
REG-33   Performance
REG-34   Stress Tests
REG-35   Compatibility
```

---

# 173. Primeiro vertical slice

```text
RegistryManager
        ↓
BlockRegistry
        ↓
register nexora:stone
        ↓
validate
        ↓
freeze
        ↓
lookup
        ↓
RuntimeID
        ↓
Block System
```

---

# 174. Segundo vertical slice

```text
ItemRegistry
 ↓
register nexora:iron_ingot
 ↓
Item System
 ↓
ItemStack
 ↓
Inventory
 ↓
Save
```

---

# 175. Terceiro vertical slice

```text
EntityRegistry
 ↓
register nexora:test_mob
 ↓
Entity System
 ↓
spawn
 ↓
Entity Instance
```

---

# 176. Quarto vertical slice

```text
Mod
 ↓
RegistrationContext
 ↓
register custom block
 ↓
register custom item
 ↓
register custom entity
 ↓
validate
 ↓
freeze
 ↓
world starts
```

Esse deve ser um dos primeiros testes de toda a arquitetura de mods.

---

# 177. Teste crítico

Executar:

```text
Vanilla
+
Mod A
+
Mod B
+
Mod C
```

com:

```text
blocks
items
entities
biomes
fluids
recipes
```

e garantir:

```text
sem IDs duplicados
sem referências quebradas
sem dependências cíclicas
registry determinístico
save compatível
network mapping correto
```

---

# 178. Stress Test

Algo como:

```text
100
1.000
10.000
100.000
1.000.000
```

entries sintéticas, medindo:

```text
registration
validation
resolution
freeze
memory
lookup
snapshot
```

Não significa que NEXORA precise de um milhão de itens reais; é um teste de limite arquitetural.

---

# 179. Fuzz Testing

Gerar:

```text
invalid IDs
duplicate IDs
cycles
missing references
invalid schemas
bad aliases
```

e verificar que o Registry rejeita corretamente.

---

# 180. Arquitetura final do Registry

```text
                         NEXORA CORE
                              │
                       REGISTRY SYSTEM
                              │
                      REGISTRY MANAGER
                              │
                 ┌────────────┼─────────────┐
                 ↓            ↓             ↓
              REGISTRY     REGISTRY      REGISTRY
               DOMAIN        TYPE         DATA
                 │            │             │
                 └────────────┼─────────────┘
                              ↓
                      REGISTRY LIFECYCLE
                              │
          ┌───────────────────┼───────────────────┐
          ↓                   ↓                   ↓
      REGISTER             VALIDATE            RESOLVE
          │                   │                   │
          └───────────────────┼───────────────────┘
                              ↓
                            FREEZE
                              │
                    ┌─────────┴─────────┐
                    ↓                   ↓
               RUNTIME IDS         SNAPSHOTS
                    │                   │
                    ↓                   ↓
              GAME SYSTEMS         SAVE/NETWORK
```

E agora surge uma consequência importante para o NEXORA:

```text
CORE
 │
 ├── Registry System
 ├── Event Bus
 ├── Entity System
 ├── Block System
 ├── Item System
 └── Save/Persistence
```

Esses seis passam a formar o **núcleo estrutural da plataforma**.

O Registry sozinho ainda não fecha a arquitetura: o próximo passo natural é o **Event Bus**, porque Registry, Entity, Block, Item, Physics, Fluids, Machines, Combat e Civilization precisarão de comunicação desacoplada sem ficarem chamando diretamente uns aos outros.
