Perfeito. O **UI System** deve ser a camada que transforma estado do jogo em interface para jogador, administrador, servidor, ferramentas e mods.

A regra principal:

> **UI apresenta e coleta intenção do usuário; ela não deve ser dona da lógica do mundo.**

# NEXORA — UI SYSTEM / UI API

## 1. Arquitetura central

```text id="ui-01"
                         NEXORA
                           │
                      UI SYSTEM
                           │
          ┌────────────────┼────────────────┐
          ↓                ↓                ↓
       UI DATA          UI INPUT         UI RENDER
          │                │                │
          ↓                ↓                ↓
      View Model       Commands/Input      Renderer
          │                │                │
          └────────────────┼────────────────┘
                           ↓
                     GAME SYSTEMS
```

O fluxo ideal:

```text id="ui-02"
GAME STATE
    ↓
UI MODEL
    ↓
UI VIEW
    ↓
USER INPUT
    ↓
UI ACTION
    ↓
COMMAND / API
    ↓
GAME SYSTEM
    ↓
EVENT
    ↓
UI UPDATE
```

Isso evita transformar a interface em uma segunda implementação da lógica do jogo.

---

# 2. UI não é gameplay

O UI System não deve decidir:

```text id="ui-03"
quanto dano uma espada causa
quanto custa um item
quem pode quebrar um bloco
como uma máquina produz energia
como um NPC pensa
```

Ele deve perguntar aos sistemas apropriados.

---

# 3. UI também não é Renderer

Separação:

```text id="ui-04"
UI System
→ estrutura, estado, interação

Renderer
→ desenha a UI
```

Podemos ter:

```text id="ui-05"
UI Definition
        ↓
UI Runtime
        ↓
Render Commands
        ↓
Renderer
```

---

# 4. UI pode ser 2D e 3D

Suportar:

```text id="ui-06"
2D HUD
Menus
Windows
Inventory
Maps
Screens
Dialogs
Tooltips
Notifications
```

e:

```text id="ui-07"
3D World UI
Nameplates
Health bars
Markers
Interaction hints
World labels
Machine displays
```

---

# 5. UI Architecture

```text id="ui-08"
UI SYSTEM
├── UI CORE
├── UI TREE
├── WIDGETS
├── LAYOUT
├── STYLE
├── INPUT
├── FOCUS
├── NAVIGATION
├── DATA BINDING
├── VIEW MODEL
├── COMMANDS
├── EVENTS
├── WINDOWS
├── HUD
├── MENUS
├── INVENTORY UI
├── MAP UI
├── DEBUG UI
├── ACCESSIBILITY
├── LOCALIZATION
├── ANIMATION
├── THEME
├── RESPONSIVE LAYOUT
├── MOD API
└── RENDER BACKEND
```

---

# 6. UI Root

Toda interface parte de uma raiz:

```text id="ui-09"
UIRoot
```

Exemplo:

```text id="ui-10"
UIRoot
├── HUD
├── Overlay
├── Notifications
├── ModalLayer
└── Cursor
```

---

# 7. UI Tree

A UI deve ser hierárquica.

```text id="ui-11"
Screen
└── Panel
    ├── Label
    ├── Button
    └── InventoryGrid
```

Cada elemento possui:

```text id="ui-12"
parent
children
layout
style
state
input behavior
```

---

# 8. Widget

Widget é a unidade básica de interface.

```text id="ui-13"
IWidget
```

Tipos:

```text id="ui-14"
Panel
Label
Image
Button
Toggle
Slider
TextInput
ProgressBar
List
Grid
ScrollView
Tree
Tooltip
Dialog
Tabs
Dropdown
Viewport
```

---

# 9. Custom Widget

Mods e sistemas podem criar:

```text id="ui-15"
CustomWidget
```

sem alterar Core.

---

# 10. Widget Lifecycle

```text id="ui-16"
CREATED
 ↓
MOUNTED
 ↓
ACTIVE
 ↓
UPDATED
 ↓
UNMOUNTED
 ↓
DESTROYED
```

---

# 11. Screen

Uma tela completa:

```text id="ui-17"
UIScreen
```

Exemplos:

```text id="ui-18"
MainMenu
Inventory
Crafting
Map
Settings
QuestLog
Research
Machine
ServerBrowser
```

---

# 12. Screen Manager

```text id="ui-19"
UIScreenManager
```

controla:

```text id="ui-20"
push
pop
replace
close
current
stack
```

---

# 13. Screen Stack

Exemplo:

```text id="ui-21"
Gameplay
 ↓
Inventory
 ↓
ItemTooltip
```

Ao fechar:

```text id="ui-22"
ItemTooltip
 ↓
Inventory
```

---

# 14. Modal

Modal bloqueia interação com o resto da tela.

```text id="ui-23"
Modal
```

Exemplo:

```text id="ui-24"
Delete World?
Confirm / Cancel
```

---

# 15. Overlay

Não bloqueia necessariamente a interação.

Exemplo:

```text id="ui-25"
notification
damage indicator
quest update
weather warning
```

---

# 16. Layer System

A UI deve possuir camadas:

```text id="ui-26"
World UI
HUD
Gameplay Overlay
Menus
Modal
Tooltip
Debug
Cursor
```

---

# 17. UI Z-Order

Cada camada possui ordem explícita.

```text id="ui-27"
World
 < HUD
 < Menu
 < Modal
 < Tooltip
 < Cursor
```

---

# 18. Layout

Não posicionar tudo manualmente em pixels.

Suportar:

```text id="ui-28"
absolute
relative
flex
grid
stack
anchor
overlay
```

---

# 19. Anchors

Exemplos:

```text id="ui-29"
top-left
top-center
top-right
center
bottom-left
bottom-center
bottom-right
```

---

# 20. Responsive UI

A interface precisa funcionar em diferentes:

```text id="ui-30"
resolutions
aspect ratios
window sizes
DPI
scales
```

---

# 21. UI Scale

Usuário pode definir:

```text id="ui-31"
75%
100%
125%
150%
200%
```

ou escala contínua controlada.

---

# 22. Safe Areas

Especialmente para:

```text id="ui-32"
ultrawide
notches
TVs
mobile/console
```

---

# 23. Layout Constraints

Exemplo:

```text id="ui-33"
minWidth
maxWidth
minHeight
maxHeight
aspectRatio
```

---

# 24. Measurement Pass

UI deve possuir pelo menos:

```text id="ui-34"
measure
layout
render
```

---

# 25. Render Pass

Pipeline:

```text id="ui-35"
UI State
 ↓
Measure
 ↓
Layout
 ↓
Build Render Commands
 ↓
Renderer
```

---

# 26. Retained vs Immediate Mode

Eu usaria um modelo predominantemente **retained-mode** para UI normal:

```text id="ui-36"
Widget Tree
```

com uma camada de geração de render commands.

Para debug/immediate UI:

```text id="ui-37"
debug draw
```

pode ser imediata.

---

# 27. Data Binding

UI não deve buscar o estado do jogo de qualquer lugar diretamente.

Usar:

```text id="ui-38"
ViewModel
```

---

# 28. ViewModel

Exemplo:

```text id="ui-39"
InventoryViewModel
```

possui:

```text id="ui-40"
slots
selectedSlot
weight
capacity
filters
```

---

# 29. UI Model

O model adapta dados do gameplay para UI.

```text id="ui-41"
Gameplay State
 ↓
ViewModel
 ↓
UI
```

---

# 30. Não duplicar estado autoritativo

Errado:

```text id="ui-42"
UI inventory = 63 iron
game inventory = 62 iron
```

O ViewModel deve refletir a fonte autoritativa.

---

# 31. Reactive Updates

Quando o estado muda:

```text id="ui-43"
InventoryChangedEvent
 ↓
InventoryViewModel
 ↓
UI updates
```

---

# 32. Event Bus Integration

UI escuta eventos:

```text id="ui-44"
InventoryChanged
QuestUpdated
MachineStateChanged
PlayerHealthChanged
WeatherChanged
```

---

# 33. UI Input

Criar abstração:

```text id="ui-45"
IUIInput
```

eventos:

```text id="ui-46"
pointer move
click
double click
drag
key down
key up
text input
gamepad button
gamepad axis
```

---

# 34. Input não é gameplay Input

Separar:

```text id="ui-47"
Raw Input
 ↓
UI Input
 ↓
UI Action
```

e:

```text id="ui-48"
Raw Input
 ↓
Gameplay Input
```

---

# 35. Input Routing

Quando Inventory está aberta:

```text id="ui-49"
input
 ↓
UI
```

em vez de:

```text id="ui-50"
input
 ↓
Player moves
```

---

# 36. Input Capture

Widgets podem capturar:

```text id="ui-51"
pointer
keyboard
gamepad
```

---

# 37. Focus System

Criar:

```text id="ui-52"
UIFocusManager
```

controla:

```text id="ui-53"
focused widget
focus path
focus history
```

---

# 38. Keyboard Navigation

Permitir:

```text id="ui-54"
Tab
Shift+Tab
Arrow keys
Enter
Escape
```

---

# 39. Gamepad Navigation

Suportar:

```text id="ui-55"
D-pad
stick
A/B
X/Y
shoulders
triggers
```

---

# 40. Focus Navigation Graph

Para interfaces complexas:

```text id="ui-56"
Button A
 ↓
Button B
 ↓
Button C
```

---

# 41. Focus Policies

```text id="ui-57"
AUTO
MANUAL
NONE
```

---

# 42. Cursor

Criar:

```text id="ui-58"
CursorManager
```

com:

```text id="ui-59"
default
hover
drag
resize
disabled
```

---

# 43. Drag and Drop

Essencial para inventário.

```text id="ui-60"
DragSource
DropTarget
DragContext
```

---

# 44. Inventory Drag

```text id="ui-61"
Item slot
 ↓
drag
 ↓
drop target
 ↓
Inventory Command
```

UI não move o item por conta própria.

---

# 45. UI Commands

Criar:

```text id="ui-62"
UICommand
```

Exemplos:

```text id="ui-63"
MoveItemCommand
CraftCommand
EquipCommand
SelectRecipeCommand
OpenChestCommand
```

Depois:

```text id="ui-64"
UI
 ↓
Command
 ↓
Game System
```

---

# 46. Command Result

Sistema retorna:

```text id="ui-65"
success
failure
reason
updated state
```

UI apresenta.

---

# 47. UI Error

Não fazer:

```text id="ui-66"
if inventory full
```

em vários lugares.

Backend retorna:

```text id="ui-67"
InventoryFull
```

UI traduz.

---

# 48. Localization

Tudo textual deve usar chave:

```text id="ui-68"
ui.inventory.full
```

---

# 49. Localization Integration

UI resolve:

```text id="ui-69"
translationKey
+
locale
```

---

# 50. Dynamic Text

Suportar:

```text id="ui-70"
"Weight: {current}/{max}"
```

---

# 51. Pluralization

Necessário para:

```text id="ui-71"
1 item
2 items
```

seguindo regras da língua.

---

# 52. RTL

Preparar layout para:

```text id="ui-72"
right-to-left languages
```

mesmo que o suporte inicial seja limitado.

---

# 53. Fonts

Não hardcode fontes no Widget.

Usar:

```text id="ui-73"
FontRegistry
```

ou Asset System.

---

# 54. Theme

Criar:

```text id="ui-74"
UITheme
```

com:

```text id="ui-75"
font
size
spacing
radius
border
background
states
```

---

# 55. Tokens

Design system:

```text id="ui-76"
spacing-xs
spacing-sm
spacing-md
spacing-lg

text-small
text-normal
text-large
```

---

# 56. Color Tokens

Exemplo:

```text id="ui-77"
accent
warning
danger
success
background
surface
text
```

A UI usa tokens, não valores espalhados.

---

# 57. Component States

Button:

```text id="ui-78"
normal
hover
pressed
focused
disabled
```

---

# 58. Widget Style

Separar:

```text id="ui-79"
structure
style
state
```

---

# 59. UI Animation

A UI deve possuir animação própria.

Exemplo:

```text id="ui-80"
panel opens
tooltip fades
notification slides
```

Pode reutilizar o Animation System onde fizer sentido, mas não tornar UI dependente de animações 3D.

---

# 60. UI Animation API

```text id="ui-81"
animate
transition
tween
```

---

# 61. UI Transition

```text id="ui-82"
fade
slide
scale
expand
collapse
```

---

# 62. Notification System

Criar:

```text id="ui-83"
NotificationManager
```

Tipos:

```text id="ui-84"
info
success
warning
error
quest
system
```

---

# 63. Toast

```text id="ui-85"
Toast
```

com:

```text id="ui-86"
message
icon
duration
priority
action
```

---

# 64. Notification Priority

```text id="ui-87"
low
normal
important
critical
```

---

# 65. Tooltip

Criar:

```text id="ui-88"
TooltipSystem
```

Pode mostrar:

```text id="ui-89"
item
block
entity
recipe
machine
technology
```

---

# 66. Tooltip Data

UI recebe:

```text id="ui-90"
TooltipData
```

e renderiza.

---

# 67. Item Tooltip

Item System fornece:

```text id="ui-91"
ItemDisplayData
```

UI transforma em:

```text id="ui-92"
name
rarity
weight
stats
components
description
```

---

# 68. Block Inspector

Block API pode fornecer dados.

UI mostra:

```text id="ui-93"
block
state
material
hardness
capabilities
```

---

# 69. Entity Nameplate

Entity System fornece:

```text id="ui-94"
display name
health
faction
```

UI apresenta sobre a entidade.

---

# 70. World Marker

```text id="ui-95"
WorldMarker
```

exemplos:

```text id="ui-96"
quest
objective
base
enemy
resource
waypoint
```

---

# 71. Marker LOD

Não renderizar:

```text id="ui-97"
10,000 markers
```

individualmente.

---

# 72. Marker Priority

```text id="ui-98"
critical quest
nearby resource
distant location
```

---

# 73. HUD

O HUD pode possuir módulos:

```text id="ui-99"
HUD
├── Health
├── Stamina
├── Oxygen
├── Hunger/Needs
├── Hotbar
├── Compass
├── Minimap
├── Notifications
├── Quest Tracker
└── Interaction Prompt
```

---

# 74. Modular HUD

O usuário pode configurar:

```text id="ui-100"
position
visibility
scale
enabled
```

---

# 75. HUD Layout Persistence

Salvar configuração:

```text id="ui-101"
HUDLayout
```

no perfil do jogador.

---

# 76. Hotbar

Hotbar usa:

```text id="ui-102"
Inventory/Equipment state
```

e apresenta:

```text id="ui-103"
slots
counts
selection
cooldown
durability
```

---

# 77. Inventory UI

Inventário deve ser data-driven.

```text id="ui-104"
InventoryScreen
├── Player Inventory
├── Equipment
├── Accessories
├── Loadouts
├── Specialized Backpacks
└── Search / Filter
```

---

# 78. Inventory Grid

```text id="ui-105"
GridWidget
```

com:

```text id="ui-106"
ItemStackView
```

---

# 79. Item Stack View

Mostra:

```text id="ui-107"
icon
quantity
durability
quality
selection
```

---

# 80. Item UI não possui Item Logic

Exemplo:

```text id="ui-108"
click item
```

gera comando.

---

# 81. Equipment UI

Pode apresentar:

```text id="ui-109"
armor
accessory slots
backpack
loadouts
```

---

# 82. Crafting UI

Crafting fornece:

```text id="ui-110"
available recipes
ingredients
result
station
progress
```

UI apresenta.

---

# 83. Recipe Search

```text id="ui-111"
search
filter
tag
category
availability
```

---

# 84. Machine UI

Machine System fornece:

```text id="ui-112"
inputs
outputs
energy
fluid
temperature
progress
errors
```

UI visualiza.

---

# 85. Energy UI

Pode mostrar:

```text id="ui-113"
power
stored
input
output
network status
overload
```

---

# 86. Fluid UI

```text id="ui-114"
tank level
fluid type
temperature
pressure
```

---

# 87. Map UI

Criar:

```text id="ui-115"
MapSystem
```

ou UI especializada sobre Map APIs.

Pode mostrar:

```text id="ui-116"
terrain
markers
roads
rails
cities
quests
exploration
```

---

# 88. Map Data

Map UI não gera o mundo.

Consome:

```text id="ui-117"
MapSnapshot
```

do World/Map systems.

---

# 89. Tactical Map

Futuramente:

```text id="ui-118"
TacticalMap
```

com:

```text id="ui-119"
units
routes
terrain
objectives
```

---

# 90. Atlas

Para exploração:

```text id="ui-120"
WorldAtlas
```

---

# 91. Quest UI

```text id="ui-121"
QuestLog
QuestTracker
QuestDetails
```

---

# 92. Research UI

Como o NEXORA terá conhecimento e pesquisa:

```text id="ui-122"
ResearchTree
ResearchNode
Discovery
Experiment
```

---

# 93. Technology UI

```text id="ui-123"
TechnologyTree
```

---

# 94. Civilization UI

O jogador pode consultar:

```text id="ui-124"
settlements
factions
leaders
laws
trade
relationships
```

---

# 95. Economy UI

```text id="ui-125"
market
prices
supply
demand
trade routes
```

---

# 96. NPC UI

```text id="ui-126"
NPCPanel

name
profession
faction
reputation
dialogue
trade
quests
```

---

# 97. Dialogue UI

```text id="ui-127"
DialogueScreen
```

com:

```text id="ui-128"
speaker
text
choices
portraits
voice state
```

---

# 98. Dialogue choice

Escolher diálogo gera:

```text id="ui-129"
DialogueChoiceCommand
```

---

# 99. Server UI

Para multiplayer:

```text id="ui-130"
ServerBrowser
ServerInfo
PlayerList
Chat
```

---

# 100. Chat

Criar:

```text id="ui-131"
ChatUI
```

com canais:

```text id="ui-132"
global
local
party
faction
system
private
```

---

# 101. Admin UI

NEXORA pode possuir:

```text id="ui-133"
AdminPanel
```

mas permissões vêm do Server/Admin System.

---

# 102. Debug UI

Desenvolvimento:

```text id="ui-134"
Profiler
Entity Inspector
Block Inspector
Registry Viewer
Event Monitor
Chunk Viewer
Physics Debug
Audio Debug
Animation Debug
```

---

# 103. UI Debug Overlay

Exibir:

```text id="ui-135"
FPS
frame time
memory
loaded chunks
entities
audio voices
event queue
```

---

# 104. Developer Console

Pode existir como:

```text id="ui-136"
ConsoleWidget
```

mas Command System interpreta os comandos.

---

# 105. Editor UI

Futuro editor do NEXORA pode reutilizar:

```text id="ui-137"
Panel
Tree
Inspector
PropertyEditor
Viewport
```

---

# 106. Property Inspector

Muito importante para ferramentas:

```text id="ui-138"
PropertyInspector
```

mostrando:

```text id="ui-139"
name
type
value
editable
```

---

# 107. UI Data Binding

Pode ser:

```text id="ui-140"
one-way
two-way
event-driven
```

---

# 108. Two-way Binding

Usar com cuidado.

Exemplo:

```text id="ui-141"
volume slider
↔
Audio Settings
```

---

# 109. Gameplay Data Binding

Para gameplay crítico, preferir:

```text id="ui-142"
UI Action
→ Command
→ authoritative state
```

em vez de binding direto.

---

# 110. UI State

Widget pode ter estado transitório:

```text id="ui-143"
hovered
focused
pressed
expanded
selected
```

não persistente.

---

# 111. Persistent UI State

Algumas configurações:

```text id="ui-144"
HUD layout
window size
filters
favorite tabs
```

podem ser persistidas.

---

# 112. UI Save Policy

```text id="ui-145"
Widget transient state
→ TEMPORARY

HUD configuration
→ PERSIST

Render cache
→ DERIVE
```

---

# 113. Theme System

```text id="ui-146"
ThemeRegistry
```

pode registrar:

```text id="ui-147"
NEXORA Default
Dark
High Contrast
Mod Theme
```

---

# 114. Mod UI

Mods podem registrar:

```text id="ui-148"
widgets
screens
tabs
panels
HUD modules
themes
tooltips
```

---

# 115. Mod API

```text id="ui-149"
registerWidget()
registerScreen()
registerHUDModule()
registerTheme()
registerTooltipProvider()
```

---

# 116. Official UI

A UI oficial usa a mesma API.

```text id="ui-150"
PUBLIC UI API
       ↓
Vanilla UI
+
Mod UI
```

---

# 117. Screen Permissions

Mods podem possuir limites:

```text id="ui-151"
CREATE_SCREEN
CREATE_WIDGET
READ_UI_STATE
REGISTER_HUD
```

---

# 118. Mod Isolation

Um mod quebrado:

```text id="ui-152"
widget crash
```

não deve derrubar toda a UI.

Pode:

```text id="ui-153"
disable widget
log error
show fallback
```

---

# 119. UI Error Boundary

Cada tela/mod pode possuir:

```text id="ui-154"
UIErrorBoundary
```

---

# 120. Accessibility

Fundamental.

Suportar:

```text id="ui-155"
font scaling
high contrast
color independence
reduced animation
reduced motion
screen reader metadata
keyboard navigation
gamepad navigation
```

---

# 121. Reduced Motion

Usuário pode desativar:

```text id="ui-156"
large transitions
camera-like UI animations
```

---

# 122. High Contrast

Tema especializado:

```text id="ui-157"
HighContrastTheme
```

---

# 123. Color-independent UI

Não depender somente de:

```text id="ui-158"
red vs green
```

usar:

```text id="ui-159"
icon
text
shape
```

---

# 124. Screen Reader

Widgets podem possuir:

```text id="ui-160"
accessibleName
accessibleRole
accessibleDescription
```

---

# 125. Focusable

Widgets devem declarar:

```text id="ui-161"
focusable
```

---

# 126. Accessibility Tree

Separada da render tree:

```text id="ui-162"
UI Tree
 ↓
Accessibility Tree
```

---

# 127. Localization + Accessibility

Dynamic text precisa continuar acessível.

---

# 128. UI Sound

Audio System pode consumir:

```text id="ui-163"
UIAction
```

e tocar:

```text id="ui-164"
click
hover
error
confirm
```

---

# 129. UI + Animation

```text id="ui-165"
Widget State
 ↓
UI Animation
```

---

# 130. UI + Event Bus

```text id="ui-166"
World Event
 ↓
Event Bus
 ↓
UI Notification
```

---

# 131. UI + Registry

```text id="ui-167"
UIRegistry
```

pode ser registrado no Registry System.

---

# 132. UI + Save

```text id="ui-168"
HUD Configuration
 ↓
Persistence
```

---

# 133. UI + Entity

```text id="ui-169"
Entity
 ↓
UI ViewModel
 ↓
Nameplate
Status
Interaction
```

---

# 134. UI + Block

```text id="ui-170"
Block
 ↓
Interaction
 ↓
UI Prompt
```

---

# 135. UI + Item

```text id="ui-171"
Item
 ↓
ItemDisplayData
 ↓
Tooltip
Inventory
Equipment
```

---

# 136. UI + Crafting

```text id="ui-172"
Recipe
 ↓
RecipeViewModel
 ↓
Crafting UI
```

---

# 137. UI + Machines

```text id="ui-173"
Machine
 ↓
MachineViewModel
 ↓
Machine Screen
```

---

# 138. UI + Audio

```text id="ui-174"
Button Click
 ↓
UI Event
 ↓
Audio
```

---

# 139. UI + Climate

HUD pode mostrar:

```text id="ui-175"
temperature
weather
warning
```

---

# 140. UI + Civilization

Interface pode apresentar:

```text id="ui-176"
city statistics
population
economy
government
```

---

# 141. UI + Quest

Quest tracker:

```text id="ui-177"
Objective
 ↓
ViewModel
 ↓
HUD
```

---

# 142. UI + Research

Research UI:

```text id="ui-178"
Knowledge State
 ↓
Research ViewModel
 ↓
Tree
```

---

# 143. UI + Map

Map UI recebe:

```text id="ui-179"
map tiles
terrain data
markers
```

sem controlar a geração do mapa.

---

# 144. UI + Networking

Em multiplayer:

```text id="ui-180"
Server State
 ↓
Network
 ↓
Client ViewModel
 ↓
UI
```

---

# 145. UI não deve confiar no cliente

Toda ação importante:

```text id="ui-181"
UI
 ↓
Command
 ↓
Server validation
```

---

# 146. Prediction

UI pode mostrar estado otimista para:

```text id="ui-182"
button pressed
drag started
```

mas corrige se servidor rejeitar.

---

# 147. Latency

Para multiplayer, ViewModel pode ter:

```text id="ui-183"
authoritative
predicted
pending
```

---

# 148. Pending Action

Exemplo:

```text id="ui-184"
Craft
 ↓
pending
 ↓
server response
 ↓
confirmed
```

---

# 149. Optimistic UI

Aplicar somente a interações que podem ser revertidas de maneira segura.

---

# 150. UI State Machine

Telas podem possuir:

```text id="ui-185"
OPENING
OPEN
UPDATING
CLOSING
CLOSED
```

---

# 151. Navigation

Criar:

```text id="ui-186"
UINavigation
```

com:

```text id="ui-187"
push
pop
replace
back
forward
```

---

# 152. Deep Links

Interfaces podem abrir diretamente:

```text id="ui-188"
nexora://inventory
nexora://quest/...
nexora://research/...
```

conceitualmente.

Isso também ajuda mods.

---

# 153. UI Context

Cada screen recebe:

```text id="ui-189"
UIContext

player
world
dimension
input
theme
localization
services
```

---

# 154. Services

UI pode consumir serviços explicitamente:

```text id="ui-190"
INavigationService
INotificationService
IDialogService
ITooltipService
```

---

# 155. UI Service Locator

Eu evitaria um singleton global gigante.

Preferir:

```text id="ui-191"
UIContext
+
explicit dependencies
```

---

# 156. Reactive Store

Uma store opcional:

```text id="ui-192"
UIStore
```

para estados puramente de interface.

---

# 157. Don't duplicate gameplay state

```text id="ui-193"
UIStore
≠
World State
```

---

# 158. Virtualized Lists

Muito importante para:

```text id="ui-194"
10,000 items
10,000 quests
10,000 NPCs
```

Renderizar apenas itens visíveis.

---

# 159. Virtual Grid

Inventory gigantesco pode usar:

```text id="ui-195"
virtualized grid
```

---

# 160. Search Index

Busca textual pode usar índices em vez de criar widgets para tudo.

---

# 161. UI Performance

Evitar:

```text id="ui-196"
rebuild entire UI
every frame
```

---

# 162. Dirty UI Nodes

Somente nós alterados:

```text id="ui-197"
WidgetDirty
LayoutDirty
StyleDirty
ContentDirty
```

---

# 163. Layout Invalidation

Quando tamanho muda:

```text id="ui-198"
LayoutDirty
```

propagar apenas pelo subtree necessário.

---

# 164. Render Invalidation

Se só texto mudou:

```text id="ui-199"
RenderDirty
```

sem refazer toda a tela.

---

# 165. UI Batching

Renderer pode agrupar:

```text id="ui-200"
same texture
same material
same layer
```

---

# 166. UI Draw Commands

UI Runtime pode produzir:

```text id="ui-201"
DrawRect
DrawText
DrawImage
DrawNineSlice
DrawMesh
Clip
```

---

# 167. Clip Regions

Suportar:

```text id="ui-202"
scroll views
panels
windows
```

---

# 168. Nine-Slice

Para panels:

```text id="ui-203"
9-slice
```

evita esticar bordas.

---

# 169. Text Rendering

Text engine precisa suportar:

```text id="ui-204"
font shaping
kerning
glyph atlas
fallback fonts
```

---

# 170. Font Fallback

Para múltiplas línguas:

```text id="ui-205"
primary font
 ↓
fallback
 ↓
fallback
```

---

# 171. Rich Text

Suportar de maneira controlada:

```text id="ui-206"
bold
italic
icons
colors
links
```

---

# 172. Item Icons in Text

Useful:

```text id="ui-207"
[iron_ingot] x10
```

---

# 173. Input Icons

UI pode mostrar:

```text id="ui-208"
[E]
Use
```

ou:

```text id="ui-209"
[A]
Interact
```

dependendo do dispositivo.

---

# 174. Device-aware UI

Detectar:

```text id="ui-210"
keyboard
mouse
controller
```

e trocar prompts.

---

# 175. Context-sensitive Prompts

Exemplo:

```text id="ui-211"
Block in focus
 ↓
"Press E to interact"
```

---

# 176. Interaction Prompt

Block/Entity System fornece:

```text id="ui-212"
InteractionOption
```

UI mostra.

---

# 177. Radial Menus

Futuro:

```text id="ui-213"
RadialMenu
```

para gamepad.

---

# 178. Context Menus

```text id="ui-214"
right-click
```

pode abrir ações disponíveis.

---

# 179. Action Availability

UI deve consultar:

```text id="ui-215"
canEquip
canUse
canCraft
```

mas a validação final fica no backend.

---

# 180. Tooltips

Tooltip pode atualizar dinamicamente.

```text id="ui-216"
Item
 ↓
TooltipProvider
 ↓
TooltipView
```

---

# 181. World UI Anchoring

Um widget pode estar ligado a:

```text id="ui-217"
Entity
Block
World Position
Camera
```

---

# 182. Screen-space Projection

Para Nameplate:

```text id="ui-218"
world position
 ↓
camera projection
 ↓
screen position
```

Renderer/Camera fornece transformação.

---

# 183. Occlusion of World UI

World markers podem desaparecer quando:

```text id="ui-219"
behind geometry
too far
not relevant
```

---

# 184. World UI LOD

```text id="ui-220"
FULL
REDUCED
HIDDEN
```

---

# 185. Notification Queue

Evitar spam:

```text id="ui-221"
100 notifications/sec
```

com:

```text id="ui-222"
coalescing
priority
rate limit
```

---

# 186. Alert System

Eventos críticos:

```text id="ui-223"
machine overheating
storm
city attacked
low oxygen
```

podem gerar:

```text id="ui-224"
critical alerts
```

---

# 187. UI State Persistence

Salvar preferências:

```text id="ui-225"
HUD layout
keybind visibility
favorite tabs
map filters
```

---

# 188. Keybind UI

Criar:

```text id="ui-226"
KeybindScreen
```

---

# 189. Input Mapping

Input System define:

```text id="ui-227"
action = inventory
```

UI permite configurar:

```text id="ui-228"
I
Tab
controller button
```

---

# 190. UI does not own keybind logic

Input System é autoridade.

---

# 191. Settings UI

Categorias:

```text id="ui-229"
Graphics
Audio
Controls
Gameplay
Accessibility
Network
UI
```

---

# 192. Graphics Settings

Renderer fornece:

```text id="ui-230"
available options
```

UI mostra.

---

# 193. Audio Settings

Audio fornece:

```text id="ui-231"
volume
buses
spatial
```

UI configura via API.

---

# 194. UI Settings

```text id="ui-232"
scale
theme
opacity
HUD layout
```

---

# 195. Server Settings

Admin/Server Systems fornecem schema.

UI apenas apresenta controles.

---

# 196. Dynamic Forms

Criar:

```text id="ui-233"
FormSchema
```

para telas geradas por dados.

---

# 197. Property Editor

Muito útil para:

```text id="ui-234"
machines
admin
mod tools
editor
```

---

# 198. UI Schema

Widgets podem ser definidos por dados:

```text id="ui-235"
{
  "type": "button",
  "text": "ui.confirm"
}
```

---

# 199. Data-driven UI

Permite mods criarem:

```text id="ui-236"
screens
panels
forms
```

sem código completo.

---

# 200. Scripted UI

Mods podem usar scripts para eventos.

Mas com sandbox:

```text id="ui-237"
no arbitrary filesystem
no arbitrary network
```

conforme o Mod Runtime.

---

# 201. UI API Security

Permissões:

```text id="ui-238"
CREATE
READ
INPUT
NETWORK
SYSTEM
DEBUG
```

---

# 202. UI Events

```text id="ui-239"
ScreenOpened
ScreenClosed
WidgetFocused
WidgetClicked
ValueChanged
DragStarted
DragDropped
```

---

# 203. UI Events via Event Bus

Eventos de UI podem entrar no Event Bus:

```text id="ui-240"
UIEvent
 ↓
Event Bus
```

Mas nem todo evento de mouse precisa virar evento global.

---

# 204. Local UI Events

Clicks podem permanecer dentro da UI Tree.

---

# 205. Global UI Events

Coisas importantes:

```text id="ui-241"
ScreenOpened
SettingsChanged
UIAction
NotificationCreated
```

---

# 206. UI Debugger

```text id="ui-242"
nexora ui inspect
```

---

# 207. Widget Inspector

Mostrar:

```text id="ui-243"
id
type
bounds
layout
style
state
bindings
```

---

# 208. UI Tree Viewer

```text id="ui-244"
UIScreen
├── Panel
│   ├── Label
│   └── Button
└── InventoryGrid
```

---

# 209. Layout Debug

Visualizar:

```text id="ui-245"
bounds
anchors
padding
margin
clip
```

---

# 210. Performance Debug

```text id="ui-246"
layout time
render time
draw calls
widget count
rebuilds
```

---

# 211. UI Profiler

Medir:

```text id="ui-247"
frame time
layout cost
text shaping
draw commands
input processing
```

---

# 212. Stress Test

```text id="ui-248"
1,000 widgets
10,000
100,000 virtualized items
```

---

# 213. Inventory Stress

```text id="ui-249"
100,000 item entries
```

virtualized.

---

# 214. Map Stress

```text id="ui-250"
100,000 map markers
```

LOD/virtualization.

---

# 215. NPC Stress

```text id="ui-251"
10,000 nameplates
```

with visibility/priority management.

---

# 216. Accessibility Tests

Test:

```text id="ui-252"
keyboard-only
gamepad-only
large fonts
high contrast
reduced motion
screen reader tree
```

---

# 217. Localization Tests

Test long strings:

```text id="ui-253"
German-like long text
```

and different scripts/RTL where supported.

---

# 218. Responsive Tests

Resolutions:

```text id="ui-254"
1280x720
1920x1080
2560x1440
3840x2160
ultrawide
```

---

# 219. Dynamic UI Scaling

UI deve continuar utilizável quando:

```text id="ui-255"
font size
UI scale
window size
```

mudar.

---

# 220. UI Rendering Backend

```text id="ui-256"
IUIRenderer
```

pode produzir comandos para o Renderer principal.

---

# 221. UI Backend

Pode ser:

```text id="ui-257"
GameRenderer
EditorRenderer
HeadlessTestRenderer
```

---

# 222. Headless UI

Para testes:

```text id="ui-258"
TestUIRenderer
```

sem GPU.

---

# 223. UI Snapshot Tests

Renderizar tela e comparar:

```text id="ui-259"
expected snapshot
vs
actual
```

Muito útil.

---

# 224. Golden UI Tests

Exemplos:

```text id="ui-260"
Inventory
Crafting
Machine
Map
Settings
```

---

# 225. Interaction Tests

Automatizar:

```text id="ui-261"
open inventory
drag item
click craft
close
```

e verificar comandos.

---

# 226. UI Determinism

Dado:

```text id="ui-262"
same state
same resolution
same theme
```

a estrutura de UI deve ser reproduzível.

---

# 227. Mod UI Tests

```text id="ui-263"
Mod registers screen
 ↓
open
 ↓
input
 ↓
command
 ↓
unload
```

---

# 228. Missing UI Asset

Fallback:

```text id="ui-264"
MissingAsset
 ↓
placeholder
```

sem crash.

---

# 229. Missing Font

Fallback para:

```text id="ui-265"
fallback font
```

---

# 230. Missing Icon

```text id="ui-266"
placeholder icon
```

---

# 231. Save Policy para UI

```text id="ui-267"
UI definitions
→ Registry/Content

UI runtime tree
→ TEMPORARY

HUD preferences
→ PERSISTENT

render cache
→ DERIVE
```

---

# 232. UI API

Interfaces principais:

```text id="ui-233"
IUIElement
IWidget
UIScreen
IUILayout
IStyle
IUIInput
IFocusManager
INavigation
IViewModel
IUICommand
ITooltipProvider
INotificationService
IUIRenderer
IUITheme
```

---

# 233. UI Runtime

```text id="ui-234"
UIRuntime

update()
processInput()
updateBindings()
layout()
buildRenderCommands()
dispatchEvents()
```

---

# 234. UI Manager

```text id="ui-235"
UIManager

open()
close()
push()
pop()
register()
```

---

# 235. Layout Engine

```text id="ui-236"
UILayoutEngine

measure()
arrange()
invalidate()
```

---

# 236. Input Manager

```text id="ui-237"
UIInputManager

route()
capture()
release()
```

---

# 237. ViewModel API

```text id="ui-238"
IViewModel

subscribe()
get()
set()
dispose()
```

---

# 238. Command API

```text id="ui-239"
IUICommand

execute()
canExecute()
```

---

# 239. Accessibility API

```text id="ui-240"
IAccessibilityNode
IAccessibilityTree
IAccessibilityService
```

---

# 240. Notification API

```text id="ui-241"
INotificationService

show()
update()
remove()
```

---

# 241. Tooltip API

```text id="ui-242"
ITooltipService

show()
hide()
resolve()
```

---

# 242. Theme API

```text id="ui-243"
ITheme
IStyleResolver
```

---

# 243. Mod API

```text id="ui-244"
IUIRegistrationContext

registerWidget()
registerScreen()
registerTheme()
registerHUD()
```

---

# 244. Código

Eu organizaria assim:

```text id="ui-code-01"
src/
└── ui/
    ├── core/
    │   ├── ui-element.ts
    │   ├── widget.ts
    │   ├── screen.ts
    │   ├── ui-context.ts
    │   └── ui-state.ts
    │
    ├── tree/
    │   ├── ui-tree.ts
    │   ├── ui-root.ts
    │   └── layers.ts
    │
    ├── widgets/
    │   ├── panel.ts
    │   ├── label.ts
    │   ├── image.ts
    │   ├── button.ts
    │   ├── slider.ts
    │   ├── list.ts
    │   ├── grid.ts
    │   ├── scroll-view.ts
    │   ├── tooltip.ts
    │   └── custom-widget.ts
    │
    ├── layout/
    │   ├── layout-engine.ts
    │   ├── constraints.ts
    │   ├── flex.ts
    │   ├── grid.ts
    │   ├── anchors.ts
    │   └── measurement.ts
    │
    ├── style/
    │   ├── style.ts
    │   ├── theme.ts
    │   ├── tokens.ts
    │   └── state-styles.ts
    │
    ├── input/
    │   ├── ui-input.ts
    │   ├── input-routing.ts
    │   ├── focus.ts
    │   ├── navigation.ts
    │   └── cursor.ts
    │
    ├── binding/
    │   ├── view-model.ts
    │   ├── binding.ts
    │   └── reactive-store.ts
    │
    ├── commands/
    │   ├── ui-command.ts
    │   └── command-router.ts
    │
    ├── navigation/
    │   ├── screen-manager.ts
    │   ├── navigation.ts
    │   └── modal-manager.ts
    │
    ├── hud/
    │   ├── hud.ts
    │   ├── hud-module.ts
    │   └── hud-layout.ts
    │
    ├── world/
    │   ├── world-marker.ts
    │   ├── nameplate.ts
    │   └── world-ui.ts
    │
    ├── inventory/
    │   ├── inventory-view.ts
    │   ├── item-stack-view.ts
    │   └── drag-drop.ts
    │
    ├── map/
    │   ├── map-view.ts
    │   ├── markers.ts
    │   └── atlas.ts
    │
    ├── notifications/
    │   ├── notification.ts
    │   ├── notification-manager.ts
    │   └── toast.ts
    │
    ├── tooltip/
    │   └── tooltip-service.ts
    │
    ├── accessibility/
    │   ├── accessibility-node.ts
    │   ├── accessibility-tree.ts
    │   └── accessibility-service.ts
    │
    ├── animation/
    │   └── ui-animation.ts
    │
    ├── rendering/
    │   ├── ui-renderer.ts
    │   ├── draw-command.ts
    │   └── batching.ts
    │
    ├── virtualization/
    │   ├── virtual-list.ts
    │   └── virtual-grid.ts
    │
    ├── registry/
    │   ├── ui-registry.ts
    │   ├── widget-registry.ts
    │   └── theme-registry.ts
    │
    ├── serialization/
    │   └── ui-preferences.ts
    │
    ├── debugging/
    │   ├── ui-inspector.ts
    │   ├── ui-profiler.ts
    │   └── layout-debugger.ts
    │
    └── api/
        └── ui-api.ts
```

---

# 245. Fronteira arquitetural

## UI System faz

```text id="ui-boundary-01"
widgets
screens
layout
style
input routing
focus
navigation
tooltips
notifications
HUD
ViewModels
UI commands
accessibility
localization integration
world UI
UI animation
UI rendering abstraction
```

## Não faz

```text id="ui-boundary-02"
combat
AI
physics
inventory logic
crafting logic
economy
world generation
entity simulation
network authority
machine processing
audio playback
```

---

# 246. Regra fundamental

> **UI apresenta estado e coleta intenção; o sistema responsável valida e executa a ação.**

---

# 247. Segunda regra

> **A UI nunca deve ser a fonte autoritativa do estado do mundo.**

---

# 248. Terceira regra

> **Toda interação importante da UI deve terminar em uma API ou Command do sistema responsável, e não em mutação direta do mundo.**

---

# 249. Quarta regra

> **UI normal deve ser desacoplada da renderização e da resolução de gameplay.**

---

# 250. Quinta regra

> **A interface deve degradar graciosamente: conteúdo ausente, mod quebrado, asset faltante ou resolução diferente não devem derrubar a simulação.**

---

# 251. Ordem de implementação

```text id="ui-order"
UI-0    Core Contracts
UI-1    UI Element
UI-2    Widget
UI-3    UI Tree
UI-4    Root
UI-5    Screen
UI-6    Screen Manager
UI-7    Layers
UI-8    Panel
UI-9    Label
UI-10   Image
UI-11   Button
UI-12   Basic Layout
UI-13   Constraints
UI-14   Theme
UI-15   Style
UI-16   Input
UI-17   Focus
UI-18   Navigation
UI-19   ViewModel
UI-20   Binding
UI-21   Commands
UI-22   Tooltip
UI-23   Notifications
UI-24   UI Animation
UI-25   HUD
UI-26   Inventory UI
UI-27   Equipment UI
UI-28   Crafting UI
UI-29   Machine UI
UI-30   Map UI
UI-31   Quest UI
UI-32   Research UI
UI-33   Civilization UI
UI-34   Dialogue UI
UI-35   World UI
UI-36   Accessibility
UI-37   Localization
UI-38   Virtualization
UI-39   UI Rendering
UI-40   UI Registry
UI-41   Mod API
UI-42   Persistence
UI-43   Debugging
UI-44   Profiling
UI-45   Stress Tests
UI-46   Compatibility
```

---

# 252. Primeiro Vertical Slice

```text id="ui-vs-01"
UIRoot
 ↓
Screen
 ↓
Panel
 ↓
Button
 ↓
Input
 ↓
UICommand
 ↓
Event
 ↓
UI update
```

---

# 253. Segundo Vertical Slice

```text id="ui-vs-02"
Player
 ↓
Inventory
 ↓
InventoryViewModel
 ↓
InventoryScreen
 ↓
Drag & Drop
 ↓
MoveItemCommand
 ↓
Inventory System
 ↓
InventoryChangedEvent
 ↓
UI refresh
```

Esse é provavelmente o melhor primeiro teste porque atravessa:

```text
UI
+
Item
+
Inventory
+
Command
+
Event Bus
+
Persistence
```

---

# 254. Terceiro Vertical Slice

```text id="ui-vs-03"
Block
 ↓
Interaction API
 ↓
Interaction Prompt
 ↓
Player Input
 ↓
Command
 ↓
Block System
```

---

# 255. Quarto Vertical Slice

```text id="ui-vs-04"
Machine
 ↓
MachineViewModel
 ↓
Machine Screen
 ↓
Energy
 ↓
Fluid
 ↓
Progress
```

---

# 256. Quinto Vertical Slice

```text id="ui-vs-05"
Quest
 ↓
Quest Event
 ↓
ViewModel
 ↓
HUD Tracker
 ↓
Notification
 ↓
Audio
```

---

# 257. Sexto Vertical Slice

```text id="ui-vs-06"
World
 ↓
Entity
 ↓
World Marker
 ↓
Camera Projection
 ↓
UI
```

---

# 258. Sétimo Vertical Slice — Mod

```text id="ui-vs-07"
Mod
 ↓
register screen
 ↓
register widget
 ↓
register theme
 ↓
open screen
 ↓
receive input
 ↓
issue command
 ↓
unload mod
 ↓
UI cleanup
```

---

# 259. Teste de escala

```text id="ui-scale"
1,000 widgets
10,000 virtual items
100,000 world markers
10,000 NPC nameplates
```

com:

```text
virtualization
culling
LOD
priority
dirty updates
```

---

# 260. Arquitetura final

```text id="ui-final-01"
                         NEXORA
                           │
                      GAME STATE
                           │
                     VIEW MODELS
                           │
                      UI SYSTEM
                           │
        ┌──────────────────┼──────────────────┐
        ↓                  ↓                  ↓
      UI TREE            INPUT            COMMANDS
        │                  │                  │
        ↓                  ↓                  ↓
     LAYOUT             FOCUS           GAME SYSTEMS
        │                                     │
        ↓                                     ↓
      STYLE                                  EVENTS
        │                                     │
        └──────────────────┬──────────────────┘
                           ↓
                      UI UPDATE
                           │
                           ↓
                   RENDER COMMANDS
                           │
                           ↓
                       RENDERER
```

E a fundação passa a ficar:

```text id="ui-foundation"
NEXORA FOUNDATION
├── Core
├── Registry System
├── Event Bus
├── Save / Persistence
├── Entity System
├── Block System
├── Item System
├── Animation System
├── Audio System
└── UI System
```

A separação que eu considero mais importante para o NEXORA é esta:

```text id="ui-flow-final"
USER
 ↓
UI INPUT
 ↓
UI ACTION
 ↓
COMMAND
 ↓
SPECIALIZED SYSTEM
 ↓
STATE CHANGE
 ↓
EVENT BUS
 ↓
VIEW MODEL
 ↓
UI
```

Assim, clicar em **"Craft"**, arrastar um item, equipar uma armadura, abrir uma máquina, aceitar uma quest ou escolher uma opção de diálogo nunca exige que a UI saiba **como** o jogo executa aquilo.

E isso deixa o próximo nível da arquitetura muito bem definido: **Dimension System**, **Structure System**, **Networking/Server**, **Mod Runtime/Loader** e o restante da camada de gameplay poderão simplesmente expor dados, comandos e eventos para a UI, sem criar uma dependência circular com ela.
