# NEXORA

# MASTER PLAN — DROP & LOOT SYSTEM

> O Drop & Loot System controla tudo aquilo que pode ser obtido a partir do mundo.
>
> Seu objetivo é criar um sistema flexível, determinístico, configurável e extensível capaz de gerar recompensas coerentes para criaturas, blocos, mineração, agricultura, estruturas, NPCs, máquinas, eventos e regiões especiais.
>
> O sistema deve evitar drops completamente previsíveis e também evitar aleatoriedade sem propósito.

---

# 1. OBJETIVO

O sistema deverá controlar:

```text id="7k5w0e"
MOB DROPS
BLOCK DROPS
ORE DROPS
PLANT DROPS
FARM DROPS
CHEST LOOT
STRUCTURE LOOT
NPC REWARDS
QUEST REWARDS
BOSS LOOT
EVENT REWARDS
RARE LOOT
DIMENSION LOOT
FAR LANDS LOOT
VOID LOOT
```

---

# 2. PRINCÍPIO FUNDAMENTAL

Drop não deve ser apenas:

```text id="3tkh1m"
Mob morreu
↓
item X
```

O resultado deve poder depender de:

```text id="m5h2k6"
source
tool
player
biome
dimension
difficulty
luck
profession
event
rarity
conditions
```

---

# 3. DROP CONTEXT

Criar:

```text id="j0n5s4"
DropContext
```

com:

```text id="js4l20"
sourceType
sourceId
position
dimension
biome
killer
tool
damageType
difficulty
luck
gameTime
event
```

---

# 4. DROP SOURCE

Toda origem de recompensa deve possuir um ID.

Exemplo:

```text id="q3d8j5"
nexora:mob/example_creature
nexora:block/example_ore
nexora:chest/example_ruin
nexora:quest/example
```

---

# 5. DROP TABLE

Criar:

```text id="8m9p7z"
DropTable
```

com entradas:

```text id="q0f2ca"
item
quantity
weight
chance
conditions
functions
```

---

# 6. DROP ENTRY

Cada entrada:

```text id="3g4xv7"
DropEntry
├── item
├── quantity
├── probability
├── weight
├── conditions
└── transformations
```

---

# 7. RANDOM SYSTEM

Usar RNG determinístico quando o contexto exigir reprodutibilidade.

Não usar:

```text id="7g8kq2"
Math.random()
```

espalhado pelo código.

---

# 8. SEEDED LOOT

A seed do mundo poderá participar do resultado.

Exemplo:

```text id="k0p2j4"
worldSeed
+
source
+
position
+
event
```

geram resultado determinístico quando apropriado.

---

# 9. DROP ROLLS

Uma DropTable pode realizar múltiplos rolls.

Exemplo:

```text id="c5h8b2"
rolls = 3
```

Cada roll usa regras definidas.

---

# 10. WEIGHTED LOOT

Em vez de apenas porcentagens:

```text id="4s1v6q"
Common = 100
Rare = 20
Epic = 2
```

permitir tabelas ponderadas.

---

# 11. GUARANTEED DROPS

Algumas recompensas são obrigatórias.

```text id="c7b9n1"
guaranteed
+
random bonus
```

---

# 12. OPTIONAL DROPS

Exemplo:

```text id="m4j9f7"
meat
hide
rare organ
```

com diferentes probabilidades.

---

# 13. QUANTITY RULES

Quantidade pode ser:

```text id="v7p5m0"
fixed
range
weighted
formula
```

Exemplo:

```text id="d1y3q8"
2–5 items
```

---

# 14. LOOT FUNCTIONS

Criar funções pós-drop.

Exemplo:

```text id="b6r8l9"
multiply
cap
bonus
replace
filter
convert
```

---

# 15. PLAYER LUCK

O sistema pode considerar:

```text id="x1f3m8"
Luck
```

mas com limites para evitar quebrar o balanceamento.

---

# 16. EQUIPMENT MODIFIERS

Equipamentos podem influenciar drops.

Exemplo:

```text id="n8k5m2"
luck accessory
harvesting tool
special collector
```

---

# 17. TOOL-DEPENDENT DROPS

Alguns recursos podem depender da ferramenta.

Exemplo:

```text id="p9f1r5"
basic tool
→ basic material

advanced tool
→ advanced material
```

---

# 18. TOOL TIER

Criar:

```text id="w3j7s4"
toolTier
```

e:

```text id="a9q2c6"
requiredTier
```

---

# 19. SILK/EXACT-COLLECTION STYLE SYSTEM

O NEXORA pode futuramente possuir ferramentas que alterem o resultado normal da coleta.

Exemplo:

```text id="3x7p0e"
normal mining
→ processed resource

special tool
→ original block/material
```

A mecânica deve ser própria do NEXORA.

---

# 20. MOB DROPS

Cada espécie possuirá:

```text id="e8f6b3"
DropTable
```

---

# 21. MOB ECOLOGY CONNECTION

Drops precisam conversar com o sistema ecológico.

Exemplo:

```text id="j7c2q5"
animal
↓
meat
↓
food economy
```

---

# 22. HUNTING ECONOMY

Alguns recursos de caça poderão alimentar:

```text id="1r8u5o"
food
clothing
medicine
crafting
trade
```

---

# 23. MOB AGE

Drops podem depender da idade:

```text id="v9y2a0"
juvenile
adult
elder
```

---

# 24. MOB STATE

Drops podem depender de:

```text id="z6d4m8"
health
disease
mutation
domestication
special status
```

---

# 25. BOSS DROPS

Bosses poderão possuir:

```text id="4m5q1x"
guaranteed reward
rare reward
unique reward
```

---

# 26. BOSS PROGRESSION

Boss loot pode liberar:

```text id="b8x7j4"
new equipment
new recipes
new abilities
new access
```

---

# 27. BLOCK DROPS

Blocos podem definir o que produzem quando quebrados.

---

# 28. ORE DROPS

Minérios podem produzir:

```text id="2u6c9a"
raw ore
processed material
rare bonus
```

dependendo da ferramenta e do sistema de mineração.

---

# 29. DEPTH-BASED DROPS

Recursos profundos podem variar por camada.

```text id="d8g5y1"
Deep Layer
↓
exclusive resources
```

---

# 30. CAVE DROPS

Criaturas e formações subterrâneas podem fornecer recursos exclusivos.

---

# 31. FAR LANDS DROPS

As Far Lands poderão ter loot exclusivo.

Exemplo:

```text id="n5p8x2"
frontier material
rare organism
ancient component
```

---

# 32. VOID DROPS

A Void Dimension possuirá tabelas próprias.

---

# 33. DIMENSION DROPS

Cada dimensão poderá registrar:

```text id="m1j4c7"
dimensionLootTable
```

---

# 34. BIOME DROPS

Alguns drops dependerão do bioma.

Exemplo:

```text id="t6w3p5"
desert creature
→ desert materials

ocean creature
→ marine materials
```

---

# 35. SEASONAL DROPS

Alguns recursos podem variar por estação.

```text id="k5v7h2"
spring
summer
autumn
winter
```

---

# 36. WEATHER DROPS

Eventos climáticos podem alterar certas recompensas.

---

# 37. STRUCTURE LOOT

Estruturas poderão possuir:

```text id="b7q0e3"
loot containers
hidden loot
rare loot
puzzle rewards
```

---

# 38. CHEST SYSTEM

Criar:

```text id="x8n4g6"
LootContainer
```

com:

```text id="1w6p9r"
table
seed
location
state
```

---

# 39. CHEST PERSISTENCE

Depois que um container foi aberto:

o estado deve persistir.

---

# 40. CHEST REGENERATION

Quando aplicável, definir se:

```text id="q4y7f2"
never respawn
respawn after event
respawn after cycle
```

---

# 41. STRUCTURE-SPECIFIC LOOT

Um laboratório não deve ter necessariamente o mesmo loot que uma mina.

---

# 42. NPC REWARDS

NPCs podem entregar:

```text id="k7m5x1"
money
items
recipes
equipment
knowledge
unlock items
```

---

# 43. QUEST REWARDS

Quest rewards deverão usar o Drop System.

---

# 44. ECONOMY INTEGRATION

Itens obtidos entram no:

```text id="c1p8q9"
Inventory
```

e podem entrar no:

```text id="v7n4m2"
Economy
```

---

# 45. TRADE GOOD DROPS

Alguns itens podem ser valiosos economicamente sem serem úteis para crafting.

---

# 46. RARITY

Criar:

```text id="w4b9c2"
COMMON
UNCOMMON
RARE
VERY_RARE
EPIC
LEGENDARY
UNIQUE
```

A quantidade de níveis deve ser ajustada pelo balanceamento.

---

# 47. RARITY ≠ QUALIDADE AUTOMÁTICA

Um item raro não precisa sempre ser melhor.

Raridade define disponibilidade, não necessariamente poder.

---

# 48. UNIQUE DROPS

Alguns itens podem possuir:

```text id="u7d3f5"
one-per-world
one-per-region
one-per-boss
```

---

# 49. WORLD UNIQUE LOOT

Permitir recompensas extremamente raras associadas à seed/mundo.

---

# 50. PROGRESSIVE LOOT

Algumas recompensas podem melhorar conforme o estágio do mundo.

---

# 51. PLAYER PROGRESSION

Drop tables podem verificar:

```text id="d3j8n6"
progression
technology
dimension access
quest status
```

---

# 52. LOOT CONDITIONS

Criar condições como:

```text id="h5v2p9"
BiomeCondition
DimensionCondition
ToolCondition
DifficultyCondition
QuestCondition
PlayerLevelCondition
SeasonCondition
WorldPhaseCondition
```

---

# 53. CONDITION API

Mods poderão criar condições próprias.

---

# 54. DROP TRANSFORMATIONS

Permitir:

```text id="f0w6s8"
Base Drop
↓
condition
↓
modified Drop
```

---

# 55. LOOT REPLACEMENT

Uma regra pode substituir:

```text id="m1y5c8"
item A
→
item B
```

quando condições específicas forem atendidas.

---

# 56. LOOT BONUS

Adicionar:

```text id="r8g2d3"
+1
+percentage
+roll
```

---

# 57. LOOT CAPS

Impedir quantidades absurdas.

---

# 58. DUPLICATE HANDLING

Definir comportamento para recompensas únicas duplicadas.

Exemplo:

```text id="x8j4v6"
unique item already owned
↓
alternative reward
```

---

# 59. LOOT → UNLOCK

Alguns drops podem ser itens de desbloqueio.

Exemplo:

```text id="f7m2k4"
rare artifact
↓
consume
↓
unlock equipment slot
```

Isso conecta diretamente ao Character/Inventory System.

---

# 60. LOOT → RECIPE

Drops também podem desbloquear receitas.

---

# 61. LOOT → KNOWLEDGE

Um artefato pode desbloquear conhecimento.

---

# 62. LOOT → QUEST

Encontrar determinado item pode desencadear uma nova missão.

---

# 63. LOOT → CIVILIZATION

Itens raros podem influenciar economia ou tecnologia de uma civilização.

---

# 64. LOOT → TRADE

Alguns recursos podem ser altamente valorizados em determinadas regiões.

---

# 65. REGIONAL VALUE

O valor econômico de um item pode variar por localização.

---

# 66. DROP SOURCE TRANSPARENCY

O jogo deve conseguir explicar por que algo caiu.

Exemplo:

```text id="9c5kq2"
Rare Crystal
Source: Deep Cavern
Depth: Layer 7
Drop bonus: Mining Tool
```

---

# 67. DEBUG LOOT

Criar:

```text id="j3f7y8"
nexora loot inspect
```

Permitindo visualizar:

```text id="n6h4p1"
source
table
roll
conditions
result
```

---

# 68. DROP SIMULATOR

Criar ferramenta:

```text id="4x8s2v"
nexora loot simulate
```

para executar milhares de rolls e verificar distribuição.

---

# 69. BALANCE TESTING

Simular:

```text id="q6r3m9"
1,000
10,000
100,000
```

execuções.

Verificar:

* média;
* mínimo;
* máximo;
* raridade;
* frequência.

---

# 70. DROP DISTRIBUTION

Não aceitar uma tabela que na prática faça:

```text id="q2f8z5"
rare = 90%
```

quando deveria ser rara.

---

# 71. ECONOMY IMPACT

Simular como drops afetam a economia.

Exemplo:

```text id="y8b1f4"
mob drop
↓
oversupply
↓
price collapse
```

---

# 72. ECOLOGY IMPACT

Simular:

```text id="c4q7m2"
mob population
↓
drop production
↓
economy
```

---

# 73. FARMING IMPACT

Recursos cultiváveis devem possuir produção sustentável.

---

# 74. RENEWABLE VS NON-RENEWABLE

Classificar recursos:

```text id="a3j5w7"
renewable
limited
non-renewable
unique
```

---

# 75. DROP RESOURCE CLASSIFICATION

Cada drop pode possuir:

```text id="k9v4m1"
resourceClass
```

---

# 76. WORLD RESOURCE BALANCE

Impedir que o Drop System torne a economia infinita sem custo.

---

# 77. MOB RESPAWN

Drop também depende de respawn.

---

# 78. MOB POPULATION

O Mob Ecology System controla a população.

O Drop System apenas recompensa a interação.

---

# 79. NO INFINITE EXPLOIT

Não permitir:

```text id="s6n2p9"
kill
↓
instant respawn
↓
infinite resource
```

sem uma regra explícita.

---

# 80. DROPS FROM PLAYER ACTION

Algumas recompensas podem depender da ação.

Exemplo:

```text id="c3f5w8"
mining method
harvesting method
tool
```

---

# 81. ENVIRONMENTAL INTERACTION

Alguns recursos podem aparecer devido a:

```text id="r5m7g1"
storm
fire
flood
erosion
special event
```

---

# 82. EVENT LOOT

Eventos mundiais poderão gerar:

```text id="j8p4n2"
rare resources
temporary items
special rewards
```

---

# 83. DAILY/WEEKLY SYSTEM

Quando houver calendário de mundo, certas recompensas podem estar vinculadas a períodos.

---

# 84. QUEST OBJECTIVES

Quest system poderá pedir:

```text id="q1v7s4"
collect X
obtain rare Y
discover Z
```

usando o Drop System como fonte de eventos.

---

# 85. DROP EVENTS

Emitir:

```text id="z6c2h5"
DROP_GENERATED
ITEM_DROPPED
LOOT_OPENED
REWARD_GRANTED
```

---

# 86. INVENTORY INTEGRATION

Fluxo:

```text id="h5j8n3"
Drop
↓
ItemStack
↓
Inventory
```

---

# 87. PHYSICAL DROPS

Nem tudo precisa ser adicionado automaticamente ao inventário.

Alguns drops podem aparecer fisicamente no mundo.

---

# 88. ITEM ENTITY

Criar:

```text id="p7x4k2"
ItemEntity
```

para representar itens no mundo.

---

# 89. PICKUP

```text id="w2m9q7"
ItemEntity
↓
player
↓
Inventory
```

---

# 90. ITEM MAGNET / AUTO-PICKUP

Sistemas de equipamento poderão alterar coleta.

---

# 91. DROP FILTER

Criar filtros:

```text id="y4k6p0"
inventory compatible
backpack compatible
player preference
```

---

# 92. SPECIALIZED BAG INTEGRATION

Drops podem ser enviados diretamente para:

```text id="r0q8s4"
Mining Backpack
Farming Backpack
Hunter Backpack
Magic Backpack
```

quando as regras permitirem.

---

# 93. DROP PRIORITY

Ao obter item:

```text id="f2m5x8"
specialized container
↓
main inventory
↓
world
```

---

# 94. QUEST LOOT

Uma quest concluída deve possuir uma reward table.

---

# 95. CIVILIZATION REWARDS

Uma civilização pode recompensar o jogador com:

```text id="b7k4n2"
currency
items
knowledge
reputation
```

---

# 96. KNOWLEDGE REWARDS

Não limitar recompensa a itens físicos.

---

# 97. DISCOVERY REWARDS

Descobrir estruturas pode liberar:

```text id="n4m8p1"
knowledge
map
recipe
location
```

---

# 98. DIMENSION REWARDS

Exploração dimensional poderá liberar recursos únicos.

---

# 99. FAR LANDS REWARD LOOP

```text id="c8v2m6"
Far Lands
↓
rare resource
↓
advanced technology
↓
better equipment
↓
deeper exploration
```

---

# 100. VOID REWARD LOOP

```text id="h9x4q3"
Void
↓
unique resource
↓
dimensional technology
↓
new progression
```

---

# 101. DROP API

Expor:

```ts id="q0f3n8"
registerDropTable()
registerDropSource()
registerDropCondition()
registerDropFunction()
```

---

# 102. DATA-DRIVEN

Sempre que possível, DropTables deverão ser definidas por dados.

---

# 103. EXEMPLO CONCEITUAL

```yaml id="w8r4p2"
id: nexora:forest_creature
rolls: 2

entries:
  - item: nexora:meat
    quantity: 1-3
    weight: 80

  - item: nexora:hide
    quantity: 1-2
    weight: 35

  - item: nexora:rare_material
    quantity: 1
    weight: 2
```

Valores são somente exemplo.

---

# 104. MOD API

Mods poderão criar:

```text id="e7m3h6"
drop tables
loot conditions
rarities
reward types
```

---

# 105. OFFICIAL CONTENT

O conteúdo oficial do NEXORA utilizará a mesma Drop API.

---

# 106. VERSIONING

Drop tables devem possuir versão quando mudanças forem relevantes.

---

# 107. SAVE COMPATIBILITY

Mudanças em loot não podem corromper saves.

---

# 108. LOOT MIGRATIONS

Quando necessário:

```text id="q4n7c2"
Loot Schema V1
↓
Migration
↓
Loot Schema V2
```

---

# 109. SECURITY

Nunca confiar em dados enviados pelo cliente em multiplayer para determinar drops válidos.

O servidor deve validar recompensas.

---

# 110. MULTIPLAYER

No multiplayer:

```text id="j5x8p4"
SERVER
↓
resolve drop
↓
validates
↓
sends result
```

---

# 111. ANTI-CHEAT

Não aceitar:

```text id="z7m2q6"
client
→ "eu ganhei item lendário"
```

sem validação.

---

# 112. DUPLICATION PREVENTION

Testar:

```text id="p3w5r8"
loot
+
disconnect
+
retry
```

---

# 113. TRANSACTION ID

Recompensas importantes poderão possuir:

```text id="s4n7k2"
rewardTransactionId
```

para impedir duplicações.

---

# 114. PLAYER HISTORY

Registrar recompensas relevantes.

---

# 115. LOOT LOG

Criar:

```text id="m8x3q5"
Loot History
```

para diagnóstico opcional.

---

# 116. ECONOMIC ANALYTICS

Medir:

```text id="c7q1m9"
items generated/day
items consumed/day
items traded/day
```

---

# 117. ECOLOGICAL ANALYTICS

Medir:

```text id="v5n2p8"
mob kills
population changes
resource extraction
```

---

# 118. BALANCING DASHBOARD

Futuramente:

```text id="u9m4x2"
Loot Analytics
```

visualizando distribuição.

---

# 119. TEST SCENARIO — MOB

```text id="f3q7m5"
Mob
↓
death
↓
DropTable
↓
roll
↓
ItemEntity
↓
Inventory
```

---

# 120. TEST SCENARIO — ORE

```text id="n8k2v4"
Ore
↓
Tool
↓
DropTable
↓
Resource
↓
Inventory
```

---

# 121. TEST SCENARIO — STRUCTURE

```text id="x6m1p9"
Structure
↓
Chest
↓
LootTable
↓
Items
```

---

# 122. TEST SCENARIO — QUEST

```text id="b4w7q3"
Quest
↓
complete
↓
RewardTable
↓
item/currency/knowledge
```

---

# 123. TEST SCENARIO — FAR LANDS

```text id="j7x5m2"
Far Lands
↓
rare resource
↓
inventory
↓
economy
↓
progression
```

---

# 124. TEST SCENARIO — CIVILIZATION

```text id="c2v8n5"
Settlement
↓
Quest
↓
Reward
↓
Currency + Item
```

---

# 125. TEST SCENARIO — VOID

```text id="q9m4x7"
Void
↓
unique entity
↓
Drop
↓
dimensional resource
```

---

# 126. TEST DISTRIBUTION

Criar uma suite que gere milhares de resultados e verifique:

```text id="w2n6k4"
min
max
average
distribution
rarity
```

---

# 127. REGRESSION TEST

Se alterar uma DropTable importante:

o sistema deve detectar alteração significativa na distribuição.

---

# 128. DROP DESIGN PRINCIPLE

Um drop deve possuir uma razão para existir.

Exemplo:

```text id="f7m3q8"
mob
→ resource
→ crafting
→ economy
```

ou:

```text id="x4n9c1"
structure
→ artifact
→ knowledge
→ quest
```

---

# 129. NÃO CRIAR LOOT LIXO

Evitar gerar:

```text id="v2m8p6"
100 itens aleatórios sem função
```

apenas para aumentar quantidade.

---

# 130. DROP CHAIN

Alguns recursos podem formar cadeias:

```text id="q6x1n4"
Mob
↓
Organic Material
↓
Alchemy
↓
Medicine
```

---

# 131. CROSS-SYSTEM DROPS

Um drop pode atravessar:

```text id="m7p3q5"
Mob
→ Resource
→ Economy
→ Profession
→ Civilization
```

---

# 132. DROP + TECHNOLOGY

Recursos raros podem alimentar:

```text id="b8n4x2"
machines
reactors
advanced tools
```

---

# 133. DROP + MAGIC

Recursos raros podem alimentar:

```text id="c5q7m1"
ritual
mana
magic equipment
```

---

# 134. DROP + AGRICULTURE

Recursos podem virar:

```text id="x9m2v5"
seed
fertilizer
crop
```

---

# 135. DROP + SPACE

Recursos encontrados em dimensões/espaço podem criar:

```text id="p4n8q6"
fuel
materials
modules
research
```

---

# 136. DROP + INVENTORY

Sempre terminar em uma destas possibilidades:

```text id="6x3m9q"
Inventory
World Item
Reward State
Knowledge
Currency
```

---

# 137. DROP ENGINE

Arquitetura final:

```text id="j8q4m1"
                DROP ENGINE
                     │
        ┌────────────┼────────────┐
        │            │            │
     SOURCES       RULES       TABLES
        │            │            │
        └────────────┼────────────┘
                     │
                  ROLLER
                     │
                TRANSFORM
                     │
                 RESULT
            ┌────────┼─────────┐
            │        │         │
         ITEM     CURRENCY   KNOWLEDGE
            │
        INVENTORY
```

---

# 138. RELAÇÃO COM O MOB ENGINE

```text id="p3v8n6"
Mob Ecology
      ↓
Mob
      ↓
death/event
      ↓
Drop Engine
```

O Mob Engine não deve implementar manualmente as regras de loot.

---

# 139. RELAÇÃO COM O WORLD ENGINE

```text id="g7m2q4"
World Generation
↓
Biome / Structure / Resource
↓
Drop Engine
```

---

# 140. RELAÇÃO COM O ECONOMY ENGINE

```text id="z4n8x1"
Drop
↓
Inventory
↓
Economy
```

---

# 141. RELAÇÃO COM O INVENTORY ENGINE

```text id="w6m3q9"
Drop Result
↓
ItemStack
↓
Inventory Transaction
```

---

# 142. RELAÇÃO COM QUEST ENGINE

```text id="k8q2m5"
Quest Reward
↓
RewardTable
↓
Drop Engine
```

---

# 143. RELAÇÃO COM CIVILIZATION ENGINE

```text id="x3n7p4"
Settlement
↓
Quest / Event
↓
Reward
↓
Drop Engine
```

---

# 144. DEFINIÇÃO DE "DROP SYSTEM FUNCIONAL"

O sistema estará funcional quando for possível:

```text id="m9q4x6"
criar source
↓
criar table
↓
definir regras
↓
executar roll
↓
gerar resultado
↓
entregar ao jogador
↓
persistir
↓
validar
```

---

# 145. DEFINIÇÃO FINAL

O Drop System do NEXORA deve transformar:

```text id="y6m2p8"
MUNDO
```

em:

```text id="q3x7n4"
OPORTUNIDADES
```

Cada recurso deve possuir uma função dentro do ecossistema.

---

# PRINCÍPIO FINAL

> **Loot should connect the world together.**

Um mob fornece recurso.

O recurso alimenta uma profissão.

A profissão alimenta uma economia.

A economia sustenta uma civilização.

A civilização cria demanda.

A demanda incentiva exploração.

A exploração encontra novos recursos.

E o ciclo continua.

O Drop System é a ponte entre a atividade do jogador e o restante do mundo.
