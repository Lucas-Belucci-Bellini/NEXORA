Sim. O **Combat System** precisa ser outro sistema grande e modular. No NEXORA ele deve funcionar tanto para combate corpo a corpo, criaturas, NPCs e equipamentos quanto para veículos e sistemas de jogo mais avançados, sem colocar toda a lógica dentro do Player ou da Física.

A separação ideal:

```text id="4x8m2q"
                         COMBAT SYSTEM
                              │
       ┌──────────────────────┼──────────────────────┐
       │                      │                      │
     COMBAT                DAMAGE                 DEFENSE
       │                      │                      │
     Attacks              Damage Types            Armor
     Actions              Resistances             Shields
     Combos               Penetration             Barriers
     Timing               Criticals               Protection
       │                      │                      │
       └──────────────────────┼──────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
       WEAPONS              AI                  STATUS
          │                   │                   │
       Tools              Decisions            Effects
       Melee              Targeting            Conditions
       Ranged             Tactics              Debuffs
       Energy             Positioning           Buffs
          │                   │                   │
          └───────────────────┼───────────────────┘
                              │
             ┌────────────────┼────────────────┐
             │                │                │
          PHYSICS          PLAYER            MOB/NPC
             │                │                │
          hit tests        input             behavior
          collision        abilities         tactics
          projectiles      equipment         factions
```

# NEXORA — COMBAT SYSTEM MASTER PLAN

## 1. Objetivo

O sistema deve controlar:

```text
ataques
alvos
dano
defesa
armadura
resistências
escudos
status
alcance
precisão
cooldowns
stamina/energia
IA de combate
progressão
loot/rewards
```

A regra principal:

> **Combat System decide o resultado do confronto; Physics determina a interação física; Renderer mostra o que aconteceu; Health/Status mantém o estado do alvo.**

---

# 2. COMBAT-0 — Combat Core

Criar:

```text
CombatSystem
CombatContext
CombatAction
CombatResult
CombatRules
```

Fluxo básico:

```text id="5m8q2x"
Attack Request
 ↓
Validate
 ↓
Acquire Target
 ↓
Resolve Hit
 ↓
Calculate Damage
 ↓
Apply Defense
 ↓
Apply Result
 ↓
Generate Events
```

---

# 3. COMBAT-1 — Combatant

Qualquer entidade pode participar de combate.

```text id="7x3m9q"
Combatant
├── combatId
├── faction
├── health
├── defenses
├── abilities
├── equipment
└── combatState
```

Assim:

```text
Player
Mob
NPC
Boss
Vehicle
Turret
Machine
```

podem usar a mesma infraestrutura.

---

# 4. COMBAT-2 — Combat State

Estados:

```text id="4m7q1x"
IDLE
ALERT
TARGETING
ATTACKING
DEFENDING
STUNNED
DISABLED
RETREATING
DEFEATED
```

---

# 5. COMBAT-3 — Target System

Criar:

```text id="8q2m5x"
TargetQuery
TargetSelection
TargetLock
```

Critérios:

```text id="m9x3q7"
distance
line of sight
visibility
faction
threat
health
priority
```

---

# 6. COMBAT-4 — Factions

Combatentes podem pertencer a:

```text id="5q8m2x"
player
civilization
faction
species
organization
```

Relações:

```text id="x4m7q1"
ALLY
NEUTRAL
HOSTILE
UNKNOWN
```

O Diplomacy System pode alterar essas relações.

---

# 7. COMBAT-5 — Aggro / Threat

Criar:

```text id="m8q3x5"
ThreatTable
```

com fatores como:

```text id="7x1m9q"
damage
proximity
healing
actions
taunt
history
```

---

# 8. COMBAT-6 — Attack Definition

Um ataque não deve ser hardcoded no Player.

```text id="4m8q2x"
AttackDefinition
├── id
├── type
├── range
├── cost
├── cooldown
├── hitPolicy
├── damageProfile
└── effects
```

---

# 9. COMBAT-7 — Attack Types

Suportar:

```text id="x7m3q9"
MELEE
RANGED
AREA
CONTACT
PROJECTILE
ENERGY
MAGICAL
ENVIRONMENTAL
VEHICLE
```

---

# 10. COMBAT-8 — Melee

Base:

```text id="5q2m8x"
MeleeAttack
├── reach
├── arc
├── timing
├── staminaCost
└── damageProfile
```

---

# 11. COMBAT-9 — Attack Arc

Ataques podem afetar:

```text id="m4x7q1"
frontal cone
arc
single target
sweep
```

---

# 12. COMBAT-10 — Hit Detection

Não confiar somente no contato visual do Renderer.

Usar Physics:

```text id="8m2q5x"
Attack
 ↓
Raycast / Sweep / Shape Query
 ↓
Target
```

---

# 13. COMBAT-11 — Hitbox

Criar:

```text id="x9m3q7"
Hitbox
```

para ataques que precisem de áreas específicas.

Separar:

```text id="5q7m1x"
Hitbox
Hurtbox
```

---

# 14. COMBAT-12 — Hurtbox

O alvo fornece:

```text id="m8q4x2"
Hurtbox
├── owner
├── region
├── multiplier
└── tags
```

---

# 15. COMBAT-13 — Body Zones

Suporte genérico a:

```text id="7x2m9q"
head
torso
arm
leg
custom
```

Sem exigir que toda entidade tenha as mesmas zonas.

---

# 16. COMBAT-14 — Damage Event

```text id="4m8q1x"
DamageEvent
├── source
├── target
├── amount
├── type
├── position
├── direction
├── weapon
└── metadata
```

---

# 17. COMBAT-15 — Damage Types

Criar tipos genéricos:

```text id="m7x3q8"
PHYSICAL
THERMAL
COLD
ELECTRICAL
CHEMICAL
MAGICAL
ENERGY
PRESSURE
ENVIRONMENTAL
```

O jogo pode expandir por API.

---

# 18. COMBAT-16 — Damage Pipeline

```text id="8q2m5x"
Base Damage
 ↓
Attacker Modifiers
 ↓
Weapon Modifiers
 ↓
Target Defense
 ↓
Resistance
 ↓
Penetration
 ↓
Critical Modifiers
 ↓
Final Damage
```

---

# 19. COMBAT-17 — Resistances

Um combatant pode ter:

```text id="m4x7q1"
resistance[damageType]
```

Exemplo:

```text id="x8m2q5"
physical resistance
thermal resistance
```

---

# 20. COMBAT-18 — Vulnerabilities

Além de resistência:

```text id="7q3m9x"
weakness
```

pode aumentar dano de determinados tipos.

---

# 21. COMBAT-19 — Armor

Criar:

```text id="m9x4q2"
ArmorProfile
├── protection
├── resistances
├── penetrationResistance
└── coverage
```

O equipamento já existente no Inventory fornece os dados.

---

# 22. COMBAT-20 — Armor Coverage

Uma armadura pode proteger:

```text id="x5m8q1"
head
torso
arms
legs
custom zones
```

---

# 23. COMBAT-21 — Damage Mitigation

Uma fórmula central precisa ser definida e versionada.

Conceitualmente:

```text id="4m7q2x"
raw damage
 ↓
armor
 ↓
resistance
 ↓
modifiers
 ↓
final damage
```

Evitar dezenas de sistemas alterando o número diretamente.

---

# 24. COMBAT-22 — Penetration

Criar:

```text id="m8q3x5"
PenetrationProfile
```

para ataques que interagem com proteção.

---

# 25. COMBAT-23 — Shields

Separar escudo de armadura:

```text id="7x2m9q"
Shield
├── capacity
├── recharge
├── rechargeDelay
└── absorption
```

---

# 26. COMBAT-24 — Barrier

Algumas entidades podem possuir:

```text id="4q8m1x"
Barrier
```

como proteção temporária.

---

# 27. COMBAT-25 — Block / Defense Action

Criar:

```text id="m7x3q9"
DefensiveAction
```

para sistemas de defesa.

Exemplo:

```text id="x2m5q8"
block
guard
shield
dodge
```

---

# 28. COMBAT-26 — Evasion

Uma defesa pode evitar totalmente um ataque.

```text id="4m8x1q"
attack
 ↓
evasion check
 ↓
miss
```

---

# 29. COMBAT-27 — Critical

Criar um sistema genérico:

```text id="m9q3x7"
CriticalProfile
├── chance
├── multiplier
└── conditions
```

---

# 30. COMBAT-28 — Accuracy

Ataques à distância podem possuir:

```text id="x5m7q2"
accuracy
spread
rangeFalloff
```

---

# 31. COMBAT-29 — Ranged Attacks

Fluxo:

```text id="7q2m8x"
Weapon
 ↓
Projectile
 ↓
Physics
 ↓
Collision
 ↓
Combat
```

---

# 32. COMBAT-30 — Projectile

Criar:

```text id="m4x9q1"
Projectile
├── position
├── velocity
├── owner
├── lifetime
├── collisionProfile
└── damageProfile
```

A Física cuida da trajetória.

---

# 33. COMBAT-31 — Projectile Types

```text id="x8m3q5"
physical projectile
energy projectile
magical projectile
utility projectile
```

---

# 34. COMBAT-32 — Projectile CCD

Usar integração com Physics:

```text id="7m2q9x"
continuous collision
sweep
```

para evitar que projéteis rápidos atravessem objetos.

---

# 35. COMBAT-33 — Area Effects

Criar:

```text id="m5x8q2"
AreaEffect
```

para situações em que um evento afeta uma região.

Pode ser usado por:

```text id="x4q7m1"
environment
ability
machine
magic
vehicle
```

---

# 36. COMBAT-34 — Damage Falloff

Ataques de área ou à distância podem possuir:

```text id="8m2q5x"
distance
 ↓
damage modifier
```

---

# 37. COMBAT-35 — Line of Sight

Antes de certos ataques:

```text id="m7x3q9"
attacker
 ↓
Physics Raycast
 ↓
target visible?
```

---

# 38. COMBAT-36 — Cover

Entidades podem aproveitar:

```text id="4q8m1x"
wall
terrain
structure
vehicle
```

AI pode consultar cobertura.

---

# 39. COMBAT-37 — Combat Positioning

Criar:

```text id="m9x2q7"
CombatPosition
```

com informações:

```text id="x5m8q1"
distance
height
cover
lineOfSight
terrain
```

---

# 40. COMBAT-38 — AI Combat

NPC/Mob pode fazer:

```text id="7q3m9x"
observe
 ↓
evaluate
 ↓
choose target
 ↓
choose action
 ↓
execute
 ↓
evaluate result
```

Isso conecta ao AI System.

---

# 41. COMBAT-39 — Combat Tactics

Criar ações:

```text id="m4x8q2"
attack
retreat
defend
flank
assist
protect
pursue
avoid
```

---

# 42. COMBAT-40 — Team Coordination

NPCs podem compartilhar:

```text id="x7m2q5"
target
threat
position
status
```

---

# 43. COMBAT-41 — Group Combat

Uma unidade/civilização pode possuir:

```text id="9q3m8x"
CombatGroup
├── members
├── leader
├── objective
├── formation
└── rules
```

---

# 44. COMBAT-42 — Faction Combat

Factions podem definir:

```text id="m5x1q7"
combat doctrine
allowed targets
retreat rules
```

---

# 45. COMBAT-43 — Diplomacy Integration

Combat pode receber:

```text id="4x8m2q"
relationship
```

do Diplomacy System.

---

# 46. COMBAT-44 — Surrender / Retreat

NPCs podem:

```text id="x7q3m9"
retreat
surrender
flee
```

dependendo do comportamento.

---

# 47. COMBAT-45 — Morale

Criar:

```text id="m8x4q2"
Morale
├── current
├── baseline
└── modifiers
```

e usar em decisões.

---

# 48. COMBAT-46 — Status Effects

Criar:

```text id="7m2q9x"
StatusEffect
```

com:

```text id="4x8m1q"
duration
intensity
stacking
source
```

---

# 49. COMBAT-47 — Status Types

Exemplos genéricos:

```text id="m5q7x2"
slow
stun
burning
poisoned
frozen
silenced
weakened
```

---

# 50. COMBAT-48 — Status Resistance

Combatant pode possuir:

```text id="x9m3q7"
statusResistance
```

---

# 51. COMBAT-49 — Immunity

Também:

```text id="4m8q1x"
immunity
```

contra determinadas condições.

---

# 52. COMBAT-50 — Healing

Healing passa pelo mesmo conceito de evento:

```text id="m7x2q9"
HealingEvent
```

com:

```text id="x4m8q1"
source
target
amount
type
```

---

# 53. COMBAT-51 — Damage Over Time

Criar:

```text id="9q3m7x"
PeriodicEffect
```

para efeitos temporais.

---

# 54. COMBAT-52 — Knockback

Integrar com Physics:

```text id="m5x8q2"
Combat
 ↓
Impulse
 ↓
Physics
```

---

# 55. COMBAT-53 — Stagger

Uma entidade pode receber:

```text id="7x2m9q"
stagger
```

sem necessariamente perder controle completo.

---

# 56. COMBAT-54 — Interrupt

Ataques/ações podem ser interrompidos por:

```text id="x4m8q1"
stun
damage
movement
environment
```

---

# 57. COMBAT-55 — Cooldown

Todos os ataques/abilities podem possuir:

```text id="m7q3x9"
Cooldown
```

usando o Cooldown Manager do Player/Engine.

---

# 58. COMBAT-56 — Resource Cost

Ataques podem consumir:

```text id="4x8m2q"
stamina
energy
magic
ammo-like resource
```

O sistema usa interfaces, não inventário diretamente.

---

# 59. COMBAT-57 — Equipment Integration

Armas/equipamentos fornecem:

```text id="m9x1q7"
AttackDefinition
DamageProfile
Range
Modifiers
```

---

# 60. COMBAT-58 — Tool / Weapon Separation

Nem todo item ofensivo precisa ser uma “weapon”.

Criar:

```text id="x5m8q2"
CombatCapability
```

Assim:

```text id="4q7m1x"
tool
device
ability
vehicle system
```

pode fornecer capacidades de combate.

---

# 61. COMBAT-59 — Ability System Integration

```text id="m8x3q5"
Ability
 ↓
Combat Action
```

---

# 62. COMBAT-60 — Magic Integration

Magic pode fornecer:

```text id="7m2q9x"
damage
shield
heal
control
utility
```

Combat resolve o resultado.

---

# 63. COMBAT-61 — Energy Technology

Sistemas tecnológicos podem fornecer:

```text id="x4m7q1"
energy attack
shield
turret
```

sem colocar lógica específica desses sistemas no Combat Core.

---

# 64. COMBAT-62 — Vehicle Combat

Veículos podem ser combatants:

```text id="m5q8x2"
Vehicle
 ↓
Combatant
```

com:

```text id="9x3m7q"
armor
systems
weapons
crew
```

---

# 65. COMBAT-63 — Vehicle Systems Damage

Um veículo pode separar:

```text id="x2m8q5"
engine
power
movement
control
armor
```

O Combat System gera o dano; Vehicle System interpreta quais componentes foram afetados.

---

# 66. COMBAT-64 — Turrets

Criar:

```text id="m7q3x9"
TargetingSystem
```

para:

```text id="4x8m2q"
turrets
defense systems
automated machines
```

---

# 67. COMBAT-65 — Structures

Estruturas podem participar de eventos de dano:

```text id="m9x2q7"
Structure
 ↓
Damage
 ↓
Structural System
```

Mas a estabilidade continua no Build/Destruction System.

---

# 68. COMBAT-66 — Environmental Combat

O mundo pode ser um fator do combate:

```text id="x5m8q1"
fire
water
terrain
pressure
weather
```

Combat pode consultar esses sistemas.

---

# 69. COMBAT-67 — Lighting

Visibilidade pode depender de:

```text id="4q7m2x"
Lighting Query
```

O Combat não implementa iluminação.

---

# 70. COMBAT-68 — Weather

Condições podem alterar:

```text id="m8x3q9"
accuracy
movement
visibility
```

de acordo com regras específicas de equipamento/combate.

---

# 71. COMBAT-69 — Atmosphere

Alguns ambientes podem afetar:

```text id="7m2q8x"
ranged systems
energy systems
movement
visibility
```

---

# 72. COMBAT-70 — Underwater Combat

O sistema deve permitir que:

```text id="x4m7q1"
Fluid State
 ↓
Combat Modifier
```

mas sem assumir que todo combate na água funciona igual.

---

# 73. COMBAT-71 — Deep World

No subterrâneo podem existir:

```text id="m5q8x2"
different combat environments
```

por conta de:

```text id="9x3m7q"
darkness
terrain
pressure
space
```

---

# 74. COMBAT-72 — Dimension Rules

Cada dimensão pode registrar:

```text id="x2m7q5"
CombatRulesProfile
```

por exemplo:

```text id="4q8m1x"
damage modifiers
allowed mechanics
special statuses
```

---

# 75. COMBAT-73 — PvE

Jogador contra:

```text id="m7x3q9"
mobs
NPCs
bosses
environment
```

---

# 76. COMBAT-74 — PvP

Opcionalmente:

```text id="9m2x7q"
PvP Rules
```

definindo:

```text id="x5m8q1"
friendly fire
duel
safe zones
faction rules
```

---

# 77. COMBAT-75 — Safe Zones

O Permission/Region System pode definir:

```text id="4q7m2x"
combat disabled
restricted
```

---

# 78. COMBAT-76 — Combat Sessions

Um confronto pode possuir:

```text id="m8x3q5"
CombatSession
├── participants
├── startTime
├── state
└── rules
```

Útil para:

```text id="7m2q9x"
boss encounters
arena
events
```

---

# 79. COMBAT-77 — Encounter System

Criar:

```text id="x4m8q1"
CombatEncounter
```

que pode representar:

```text id="m5q7x2"
boss
invasion
defense
arena
raid-like event
```

---

# 80. COMBAT-78 — Boss Framework

Bosses podem possuir:

```text id="9x3m7q"
phases
abilities
thresholds
behavior
arena rules
```

---

# 81. COMBAT-79 — Phase System

```text id="m7x2q5"
Phase 1
 ↓
condition
 ↓
Phase 2
 ↓
condition
 ↓
Phase 3
```

---

# 82. COMBAT-80 — Encounter Scaling

Dificuldade pode adaptar:

```text id="x4m8q1"
player count
difficulty
world progression
dimension
```

---

# 83. COMBAT-81 — Rewards

Combat produz eventos:

```text id="m9q3x7"
CombatResult
 ↓
Reward System
 ↓
Drop / Loot
```

---

# 84. COMBAT-82 — Loot Integration

Não colocar drops dentro do Combat.

```text id="4x8m2q"
Combat
 ↓
Defeated Event
 ↓
Loot System
```

---

# 85. COMBAT-83 — Experience

```text id="m7x3q9"
CombatOutcome
 ↓
Progression
```

para experiência/progressão.

---

# 86. COMBAT-84 — Knowledge

Encontros podem gerar informação:

```text id="x5m8q1"
observed attack
discovered weakness
identified creature
```

e o Knowledge System pode registrar isso.

---

# 87. COMBAT-85 — Combat History

Registrar eventos importantes:

```text id="4q8m2x"
who
against whom
where
when
outcome
```

para história do mundo.

---

# 88. COMBAT-86 — Civilization Warfare

Civilizações podem possuir:

```text id="m9x7q1"
army
militia
defense force
```

O combate usa os mesmos Combatants.

---

# 89. COMBAT-87 — Large-Scale Battles

Não simular cada detalhe de milhares de combatants quando estão muito longe.

```text id="x5m2q8"
FULL
→ local combat

REGIONAL
→ aggregated combat

ABSTRACT
→ battle simulation
```

---

# 90. COMBAT-88 — Battle LOD

Exemplo:

```text id="m7q3x9"
1 player nearby
→ individual combatants

large battle far away
→ unit-level simulation
```

---

# 91. COMBAT-89 — Unit System

Criar futuramente:

```text id="4m8x2q"
CombatUnit
```

para:

```text id="x7m1q5"
squad
patrol
defense group
```

---

# 92. COMBAT-90 — Command System

AI/civilization pode dar:

```text id="m9q3x7"
move
defend
attack
escort
retreat
```

---

# 93. COMBAT-91 — Morale / Cohesion

Unidades podem perder coesão.

```text id="4x8m2q"
losses
leader status
supply
morale
```

---

# 94. COMBAT-92 — Supply

Large-scale combat pode depender de:

```text id="m7x3q9"
food
equipment
energy
ammunition-like resources
transport
```

A Economia/Logistics fornece isso.

---

# 95. COMBAT-93 — Battlefield Environment

Batalhas podem ser influenciadas por:

```text id="x5m8q1"
terrain
weather
visibility
lighting
structures
```

---

# 96. COMBAT-94 — Damage Propagation

Para entidades/estruturas complexas:

```text id="4q7m2x"
damage
 ↓
component
 ↓
secondary state
```

Exemplo:

```text id="m8x3q5"
vehicle component damaged
 ↓
Vehicle System
 ↓
mobility reduced
```

---

# 97. COMBAT-95 — Disabled vs Defeated

Nem todo dano deve resultar imediatamente em derrota.

```text id="7m2q9x"
ACTIVE
 ↓
DAMAGED
 ↓
DISABLED
 ↓
DEFEATED
```

---

# 98. COMBAT-96 — Friendly Fire

Regra configurável:

```text id="x4m8q1"
friendlyFire = true/false
```

---

# 99. COMBAT-97 — Damage Permissions

Antes de aplicar dano:

```text id="m5q7x2"
Target
 ↓
Permission / Combat Rules
 ↓
Allowed?
```

---

# 100. COMBAT-98 — Anti-Cheat

No multiplayer:

```text id="9x3m7q"
client
 ↓
attack request
 ↓
server validation
 ↓
Combat System
```

O cliente não deve ser autoridade final sobre dano.

---

# 101. COMBAT-99 — Server Authority

Servidor valida:

```text id="x5m8q1"
position
reach
cooldown
target
line of sight
weapon state
resource cost
```

---

# 102. COMBAT-100 — Prediction

Clientes podem prever:

```text id="m7q3x9"
animation
input
visual feedback
```

mas o resultado final vem do servidor.

---

# 103. COMBAT-101 — Network Events

Eventos replicáveis:

```text id="4q8m2x"
AttackStarted
AttackHit
DamageApplied
StatusApplied
CombatantDefeated
AbilityUsed
```

---

# 104. COMBAT-102 — Persistence

Não precisa salvar cada hit.

Salvar:

```text id="x9m3q7"
persistent status
health when appropriate
equipment
long-term effects
combat history
```

---

# 105. COMBAT-103 — Determinism

Ataques precisam ser reproduzíveis:

```text id="m5x8q2"
same state
+
same action
+
same combat version
=
same result
```

quando determinismo for exigido.

---

# 106. COMBAT-104 — Debug

Comandos:

```text id="7q2m9x"
nexora combat inspect
nexora combat target
nexora combat simulate
nexora combat damage
nexora combat threats
```

---

# 107. COMBAT-105 — Visualization

Modo debug:

```text id="x4m7q1"
hitboxes
hurtboxes
attack arcs
target lines
line of sight
range
damage
threat
```

---

# 108. COMBAT-106 — Profiler

Métricas:

```text id="m8x3q5"
combat actions
hit tests
damage calculations
active combats
projectiles
status effects
AI decisions
```

---

# 109. COMBAT-107 — Balance Simulator

Uma ferramenta fundamental.

```text id="4q8m2x"
simulate
10,000 encounters
 ↓
damage
survival
DPS-like metrics
resource consumption
```

Sem precisar jogar manualmente.

---

# 110. COMBAT-108 — Statistical Testing

Testar:

```text id="x7m2q9"
weapon vs armor
mob vs player
mob vs mob
NPC vs NPC
vehicle vs vehicle
```

---

# 111. COMBAT-109 — AI Simulation

Rodar batalhas sem renderização:

```text id="m5q8x1"
AI
 ↓
Combat
 ↓
1000 simulations
 ↓
balance data
```

---

# 112. COMBAT-110 — Mod API

Mods podem registrar:

```text id="9x3m7q"
DamageType
AttackDefinition
DamageProfile
DefenseProfile
StatusEffect
CombatAbility
CombatRule
CombatantComponent
```

---

# 113. COMBAT-111 — Official Content

Oficial:

```text id="4m8q2x"
NEXORA Combat API
```

Mod:

```text id="x7m1q5"
NEXORA Combat API
```

mesmas interfaces.

---

# 114. COMBAT-112 — Extensibility

O sistema precisa permitir criar um novo tipo de combate sem modificar:

```text id="m9q3x7"
Combat Core
```

---

# 115. COMBAT-113 — Final Integration

```text id="4x8m2q"
                     COMBAT
                        │
       ┌────────────────┼────────────────┐
       │                │                │
    ATTACK            DAMAGE          DEFENSE
       │                │                │
    melee             types           armor
    ranged            resist           shield
    abilities         critical         barriers
       │                │                │
       └────────────────┼────────────────┘
                        │
            ┌───────────┼────────────┐
            │           │            │
         PHYSICS      HEALTH       STATUS
            │           │            │
         hit test     HP          effects
         projectile   defeat       conditions
            │           │            │
            └───────────┼────────────┘
                        │
               AI / PLAYER / MOB
                        │
          ┌─────────────┼─────────────┐
          │             │             │
       INVENTORY     EQUIPMENT     VEHICLES
          │             │             │
          └─────────────┼─────────────┘
                        │
                LOOT / PROGRESSION
                        │
              ECONOMY / CIVILIZATION
```

# 116. Ordem de implementação

```text id="m7x2q9"
COMBAT-0 Core
COMBAT-1 Combatant
COMBAT-2 Combat State
COMBAT-3 Targeting
COMBAT-4 Factions
COMBAT-5 Threat
COMBAT-6 Attack Definition
COMBAT-7 Melee
COMBAT-8 Hit Detection
COMBAT-9 Hitbox/Hurtbox
COMBAT-10 Damage Event
COMBAT-11 Damage Types
COMBAT-12 Damage Pipeline
COMBAT-13 Resistance
COMBAT-14 Armor
COMBAT-15 Penetration
COMBAT-16 Shields
COMBAT-17 Critical
COMBAT-18 Ranged
COMBAT-19 Projectile
COMBAT-20 Area Effects
COMBAT-21 Status Effects
COMBAT-22 Healing
COMBAT-23 Knockback
COMBAT-24 AI Combat
COMBAT-25 Team Combat
COMBAT-26 Vehicles
COMBAT-27 Structures
COMBAT-28 Boss/Encounters
COMBAT-29 Loot
COMBAT-30 Progression
COMBAT-31 Knowledge
COMBAT-32 Civilization Warfare
COMBAT-33 Battle LOD
COMBAT-34 Multiplayer
COMBAT-35 Persistence
COMBAT-36 Debug
COMBAT-37 Balance Simulation
COMBAT-38 Mod API
COMBAT-39 Stress Testing
```

# 117. Primeiro Vertical Slice

Começaria extremamente simples:

```text id="x5m8q1"
Player
 ↓
Input
 ↓
Attack
 ↓
Target Query
 ↓
Physics Hit Test
 ↓
Damage Calculation
 ↓
Armor
 ↓
Health
 ↓
Combat Event
 ↓
Defeat
 ↓
Loot
 ↓
Inventory
```

Depois:

```text id="m8q3x5"
Player
      ↕
      Mob
      ↕
     NPC
```

com:

```text id="7q2m9x"
targeting
AI
movement
damage
status
loot
```

Depois:

```text id="4m8x1q"
10 NPCs
 ↓
team combat
 ↓
morale
 ↓
retreat
 ↓
victory
```

E finalmente:

```text id="x7m3q9"
Civilization
 ↓
conflict
 ↓
units
 ↓
regional simulation
 ↓
battle
 ↓
economic consequences
 ↓
migration
 ↓
political consequences
 ↓
world history
```

## Regra arquitetural

A regra do NEXORA deveria ser:

> **Combat não sabe quem é o “herói” ou o “inimigo”. Ele apenas resolve interações entre Combatants.**

Então:

```text id="m5q8x2"
Player
Mob
NPC
Boss
Vehicle
Turret
Machine
```

todos podem fazer:

```text id="9x3m7q"
Attack
Defense
Damage
Status
Retreat
```

através da mesma infraestrutura.

E isso permite que o combate evolua desde **um jogador quebrando uma criatura simples** até **conflitos entre cidades e civilizações**, sem criar um segundo sistema de combate completamente diferente para cada escala.
