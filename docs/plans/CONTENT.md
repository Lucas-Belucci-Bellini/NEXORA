# NEXORA

# OFFICIAL CONTENT MASTER PLAN

## SISTEMAS NATIVOS DO JOGO

> Este documento define as grandes famílias de sistemas que farão parte do conteúdo oficial do NEXORA.
>
> Os nomes abaixo representam **referências de design e categorias de sistemas**, não uma autorização para copiar código, assets, texturas, modelos ou outros materiais protegidos.
>
> Cada sistema será implementado nativamente no NEXORA, usando sua própria engine, APIs, balanceamento, identidade visual e arquitetura.

---

# 1. OBJETIVO

O NEXORA não terá um “jogo base” extremamente limitado para depois depender de dezenas de mods obrigatórios.

A ideia é que várias das grandes categorias de gameplay normalmente adicionadas por mods já façam parte do **conteúdo oficial do jogo**.

Portanto:

```text id="2p8i72"
NEXORA
│
├── CORE GAMEPLAY
│
├── TECHNOLOGY
│
├── MAGIC
│
├── AGRICULTURE
│
├── LOGISTICS
│
├── CONSTRUCTION
│
├── ENERGY
│
├── DIMENSIONS
│
├── WARFARE
│
└── SPACE
```

Os sistemas oficiais continuam sendo construídos sobre a mesma API pública utilizada por conteúdos externos.

---

# 2. PRINCÍPIO CENTRAL

O conteúdo oficial do NEXORA deve utilizar as mesmas abstrações disponíveis para mods.

Exemplo:

```text id="p9r5uo"
NEXORA CORE
     ↓
PUBLIC GAME API
     ↓
REGISTRY / EVENTS / DATA
     ↓
OFFICIAL CONTENT
     +
COMMUNITY CONTENT
```

Não criar uma segunda engine secreta exclusivamente para o conteúdo oficial.

---

# 3. TECHNOLOGY FAMILY

## 3.1 Ender IO — inspiração de automação

Categoria:

```text id="q4u1cn"
Technology / Logistics / Automation
```

Sistemas desejados:

* máquinas compactas;
* processamento;
* conduítes/logística;
* automação;
* energia;
* transporte de itens;
* transporte de fluidos;
* transporte de energia;
* upgrades.

No NEXORA:

não copiar a implementação do mod.

Criar uma arquitetura própria de automação.

---

# 4. BIG REACTORS — ENERGIA EM GRANDE ESCALA

Categoria:

```text id="fef3zo"
Energy / Power Generation
```

Este será um dos pilares do sistema energético.

Conceito:

```text id="f8i9v4"
combustível
    ↓
reator
    ↓
calor
    ↓
energia
    ↓
rede energética
    ↓
máquinas
```

Sistemas previstos:

* geradores;
* reatores;
* estruturas modulares;
* controle;
* combustível;
* produção de energia;
* consumo;
* armazenamento;
* refrigeração;
* monitoramento;
* segurança;
* automação.

O NEXORA deve possuir um sistema energético genérico.

Então:

```text id="qk4l1h"
reator
+
solar
+
hidro
+
outras fontes
```

podem alimentar a mesma Energy API.

---

# 5. MEKANISM — TECNOLOGIA E PROCESSAMENTO

Categoria:

```text id="4e6qga"
Technology / Processing
```

Sistemas:

* processamento de minérios;
* máquinas;
* transformação;
* automação;
* energia;
* logística;
* upgrades;
* produção avançada.

O objetivo é criar uma árvore tecnológica de múltiplos níveis.

---

# 6. IMMERSIVE ENGINEERING — ENGENHARIA INDUSTRIAL

Categoria:

```text id="pz3x1z"
Industry / Engineering
```

Sistemas:

* máquinas multibloco;
* energia;
* cabos;
* infraestrutura;
* processamento;
* estruturas industriais;
* ferramentas;
* equipamentos.

O NEXORA deve ter máquinas que ocupem espaço real no mundo, e não apenas blocos abstratos.

---

# 7. CREATE — ENGENHARIA MECÂNICA

Categoria:

```text id="14nqau"
Mechanical Automation
```

Sistemas:

* movimento mecânico;
* engrenagens;
* eixos;
* transmissão;
* máquinas cinéticas;
* automação;
* transporte;
* processamento.

Criar um sistema próprio de:

```text id="j29b3d"
Mechanical Power API
```

para que outros sistemas possam reutilizá-lo.

---

# 8. APPLIED 2 — COMPUTAÇÃO E ARMAZENAMENTO

Categoria:

```text id="iqf42g"
Storage / Networking / Computing
```

Sistemas:

* armazenamento digital;
* redes;
* terminais;
* canais;
* automação;
* exportação/importação;
* busca;
* processamento de dados.

Criar:

```text id="1p0ovg"
Storage Network API
```

independente do sistema visual.

---

# 9. ENDER CHEST — ARMAZENAMENTO REMOTO

Categoria:

```text id="g1g97x"
Remote Storage
```

Sistema oficial:

```text id="1jbzq8"
inventário vinculado
↓
canal/rede
↓
acesso remoto
```

Pode ser implementado como parte de uma família maior de armazenamento.

---

# 10. TRANSLUATOR — MOVIMENTAÇÃO DE ITENS

Categoria:

```text id="1n7fxm"
Logistics
```

Sistema:

* movimentação precisa de itens;
* conexão entre inventários;
* filtros;
* transporte curto;
* automação.

Deve conversar com:

```text id="3j6c6d"
Inventory API
Storage API
Automation API
```

---

# 11. EXTRA UTILITIES — UTILIDADES GERAIS

Categoria:

```text id="f8gl5c"
Utilities
```

O NEXORA pode possuir uma grande família de ferramentas utilitárias.

Exemplos conceituais:

* transporte;
* armazenamento;
* ferramentas especiais;
* construção;
* conveniência;
* automação;
* utilidades de exploração.

Quando a referência vier de conteúdo antigo, usar a mecânica como inspiração e reimplementar no NEXORA.

---

# 12. TINKERS — FERRAMENTAS CUSTOMIZÁVEIS

Categoria:

```text id="h0x8l6"
Equipment / Crafting
```

Sistema:

```text id="3yr4f2"
Tool
↓
Parts
↓
Materials
↓
Modifiers
↓
Stats
```

Permitir:

* montagem modular;
* materiais diferentes;
* modificadores;
* atributos;
* evolução;
* reparo.

Isso deve virar uma API própria de equipamento modular.

---

# 13. BOTANIA — MAGIA NATURAL

Categoria:

```text id="h5d9jk"
Magic / Nature
```

Sistemas:

* energia/magia;
* plantas especiais;
* estruturas;
* artefatos;
* progressão;
* rituais;
* automação mágica.

Criar uma API:

```text id="7b67s6"
Mana/Energy API
```

que seja distinta da energia industrial.

---

# 14. BLOOD MAGIC — MAGIA RITUAL

Categoria:

```text id="t11vqp"
Magic / Ritual
```

Sistemas:

* rituais;
* progressão;
* recursos;
* estruturas;
* ferramentas;
* sistemas mágicos avançados.

A magia deve possuir identidade própria e não depender da Energy API industrial.

---

# 15. MYSTICAL AGRICULTURE — AGRICULTURA AVANÇADA

Categoria:

```text id="3xk3ik"
Agriculture / Progression
```

Sistemas:

* culturas;
* sementes;
* recursos agrícolas;
* progressão;
* agricultura especializada;
* produção automática.

Criar:

```text id="m3nm3j"
Agriculture API
```

para permitir que outros conteúdos adicionem culturas.

---

# 16. DRACONIC — TECNOLOGIA EXTREMA

Categoria:

```text id="j6g6m1"
Endgame Technology
```

Sistemas:

* energia em grande escala;
* equipamentos avançados;
* armazenamento de energia;
* progressão extrema;
* conteúdo de endgame.

O sistema deve existir como uma camada de progressão final.

---

# 17. PROJECT E — TRANSMUTAÇÃO

Categoria:

```text id="xsn53s"
Economy / Conversion
```

Criar um sistema próprio de:

```text id="qvkgv4"
Value
↓
Conversion
↓
Transmutation
```

Cada recurso pode possuir um valor interno.

O jogador poderá usar essa mecânica para converter recursos, mas o balanceamento deve ser próprio do NEXORA.

---

# 18. TWILIGHT — DIMENSÃO ALTERNATIVA

Categoria:

```text id="5j84cc"
Dimensions / Exploration
```

Criar uma dimensão própria, não copiar nomes, mapas ou assets.

Componentes:

* geração própria;
* biomas;
* estruturas;
* entidades;
* progressão;
* desafios;
* atmosfera própria.

Criar:

```text id="4k2h8o"
Dimension API
```

para que futuras dimensões possam ser adicionadas.

---

# 19. WARFARE / SBW — SISTEMA DE GUERRA

Categoria:

```text id="0p1hy2"
Warfare
```

O sistema oficial poderá incluir:

* equipamentos;
* veículos;
* logística;
* estruturas;
* facções;
* mapas;
* sistemas estratégicos;
* progressão.

A implementação deve ser própria do NEXORA.

Não importar diretamente conteúdo protegido de terceiros.

---

# 20. SPACE — EXPLORAÇÃO ESPACIAL

Categoria:

```text id="r5b0uw"
Space
```

Esse será um dos sistemas de maior escala.

Possíveis componentes:

```text id="5v1lq7"
Space API
├── spacecraft
├── propulsion
├── fuel
├── navigation
├── celestial bodies
├── orbital systems
├── dimensions
├── cargo
├── stations
└── exploration
```

A exploração espacial deve ser construída depois que:

```text id="w7t3cc"
World API
Vehicle API
Inventory API
Entity API
Dimension API
```

estiverem maduras.

---

# 21. TECNOLOGIA + MAGIA

Uma das características próprias do NEXORA pode ser permitir que diferentes famílias de sistemas coexistam:

```text id="ws3rc5"
Industrial
     +
Mechanical
     +
Digital
     +
Magical
     +
Biological
     +
Extreme Energy
```

Mas isso não significa que todos precisam ser misturados em todos os itens.

---

# 22. ENERGY SYSTEM

Criar uma infraestrutura comum:

```text id="5io8vq"
Energy API
```

Mas permitir diferentes sistemas energéticos:

```text id="2s07u8"
Industrial Energy
Mechanical Power
Magical Energy
Biological Energy
```

Cada um possui regras próprias.

Quando houver compatibilidade explícita, sistemas podem converter energia.

---

# 23. FLUID SYSTEM

Criar:

```text id="a2k6im"
Fluid API
```

Suportando:

* água;
* combustíveis;
* fluidos industriais;
* substâncias mágicas;
* fluidos modulares.

---

# 24. ITEM SYSTEM

Todas as famílias compartilham a mesma:

```text id="duv4u0"
Item API
```

permitindo:

```text id="0o0y5i"
Technology item
Magic item
Agriculture item
Space item
```

sem criar inventários incompatíveis.

---

# 25. RECIPE SYSTEM

Todas as receitas usam:

```text id="0ge4xt"
Recipe API
```

Suportar:

* crafting;
* máquinas;
* multiblocos;
* magia;
* transmutação;
* agricultura;
* tecnologia avançada.

---

# 26. PROGRESSION SYSTEM

Como haverá muitos sistemas, será necessário um sistema de progressão.

Exemplo:

```text id="v6d2et"
Primitive
↓
Industrial
↓
Advanced
↓
Digital
↓
Magical
↓
Extreme
↓
Dimensional
↓
Space
```

Mas o jogador não deve necessariamente ser obrigado a seguir uma única árvore linear.

---

# 27. MULTIPLE PROGRESSION PATHS

Criar caminhos:

```text id="u2q91a"
Industrial
Mechanical
Magical
Agricultural
Digital
Exploration
```

Eles podem convergir em certos pontos.

---

# 28. AUTOMATION API

Criar:

```text id="z4s9rq"
Automation API
```

permitindo que:

* máquinas;
* armazenamento;
* transporte;
* agricultura;
* magia;
* produção

participem da mesma infraestrutura.

---

# 29. MULTIBLOCK API

Big Reactors, Immersive Engineering e futuros sistemas poderão utilizar:

```text id="v7t1rb"
Multiblock API
```

Com:

* estrutura;
* validação;
* montagem;
* desmontagem;
* estados;
* energia;
* inventário.

---

# 30. MACHINE API

Criar:

```text id="9n5g5e"
Machine API
```

Com estados:

```text id="jv6l32"
IDLE
PROCESSING
WAITING
ERROR
DISABLED
```

Suportar:

* receitas;
* energia;
* fluidos;
* inventário;
* upgrades.

---

# 31. STORAGE API

Criar:

```text id="3aj4zq"
Storage API
```

Suportar:

* inventário físico;
* armazenamento remoto;
* redes;
* filtros;
* capacidade;
* automação.

---

# 32. VEHICLE API

Necessária para:

* guerra;
* exploração;
* espaço;
* transporte.

Estrutura:

```text id="a8xe6h"
Vehicle
├── movement
├── seats
├── inventory
├── fuel
├── energy
└── modules
```

---

# 33. DIMENSION API

Necessária para:

* dimensões mágicas;
* mundos especiais;
* espaço;
* conteúdo externo.

---

# 34. OFFICIAL CONTENT REGISTRY

Criar uma categoria especial:

```text id="7m7w2i"
official
```

mas utilizando exatamente o mesmo registry geral.

Exemplo:

```text id="6yg9f4"
nexora:stone
nexora:reactor
nexora:mana_flower
nexora:storage_network
nexora:space_station
```

---

# 35. O QUE É "NATIVO"

Um sistema é considerado nativo quando:

```text id="l7pbwu"
faz parte do jogo
+
possui suporte oficial
+
é testado oficialmente
+
possui API
+
recebe manutenção
```

Não significa que precisa estar hardcoded no Core.

---

# 36. CONTEÚDO OFICIAL ≠ CORE

Mesmo que faça parte do jogo:

```text id="z6y6b6"
Big Reactor
```

não pertence necessariamente ao Core.

Pode existir como:

```text id="g3b4bi"
Official Content Module
```

O Core fornece:

```text id="w4rzhm"
Energy API
Multiblock API
Machine API
```

e o módulo implementa o sistema.

---

# 37. ARQUITETURA FINAL

```text id="01q1fp"
NEXORA
│
├── CORE
│
├── PUBLIC APIs
│   ├── Block
│   ├── Item
│   ├── Entity
│   ├── Energy
│   ├── Fluid
│   ├── Inventory
│   ├── Storage
│   ├── Recipe
│   ├── Machine
│   ├── Automation
│   ├── Vehicle
│   ├── Dimension
│   └── Event
│
└── OFFICIAL CONTENT
    │
    ├── Technology
    │   ├── Automation
    │   ├── Industrial
    │   ├── Digital
    │   └── Extreme Energy
    │
    ├── Magic
    │   ├── Nature
    │   └── Ritual
    │
    ├── Agriculture
    │
    ├── Logistics
    │
    ├── Equipment
    │
    ├── Dimensions
    │
    ├── Warfare
    │
    └── Space
```

---

# 38. FAMÍLIAS OFICIAIS

## Tecnologia

Inspirada por:

```text id="xj5skc"
Ender IO
Mekanism
Immersive Engineering
Big Reactors
Draconic
```

## Mecânica

Inspirada por:

```text id="b8l0rh"
Create
```

## Computação

Inspirada por:

```text id="u3rt0f"
Applied Energistics 2
```

## Armazenamento

Inspirada por:

```text id="jso6i9"
Ender Chest
Extra Utilities
Translocator
```

## Ferramentas

Inspirada por:

```text id="3z1a68"
Tinkers
```

## Agricultura

Inspirada por:

```text id="k4i8gx"
Mystical Agriculture
```

## Magia

Inspirada por:

```text id="7xg7ai"
Botania
Blood Magic
```

## Dimensões

Inspirada por:

```text id="i4rc5r"
Twilight
```

## Guerra

Inspirada por:

```text id="0e0qdi"
SBW
```

## Espaço

```text id="4jsnq0"
Sistema espacial próprio do NEXORA
```

---

# 39. PRIORIDADE DE IMPLEMENTAÇÃO

Não construir todos ao mesmo tempo.

Ordem recomendada:

```text id="4qx8tj"
1. Core
2. Item / Block / Recipe
3. Inventory
4. Energy
5. Machine
6. Automation
7. Storage
8. Industrial
9. Mechanical
10. Digital
11. Agriculture
12. Magic
13. Extreme Technology
14. Dimensions
15. Warfare
16. Space
```

---

# 40. BIG REACTORS COMO MARCO

O primeiro grande sistema industrial de alto nível deverá provar:

```text id="57t2zu"
Multiblock
+
Fuel
+
Heat
+
Energy
+
Control
+
Storage
+
Automation
```

Depois o mesmo framework poderá ser reutilizado por outros sistemas oficiais.

---

# 41. TESTE DE ARQUITETURA

Criar um teste:

```text id="u5pmjr"
Official Module
+
External Mod
```

usando a mesma API.

Se ambos conseguem:

```text id="uijz2k"
registrar
bloco
item
recipe
machine
evento
```

sem alterar o Core:

**arquitetura aprovada.**

---

# 42. CONTEÚDO EXTERNO

Mesmo com muito conteúdo oficial, o NEXORA continuará permitindo mods externos.

A ideia não é eliminar mods.

A ideia é:

```text id="p6w1ux"
NEXORA OFFICIAL CONTENT
          +
COMMUNITY MODS
```

---

# 43. COMPATIBILIDADE

O sistema de conteúdo oficial deverá possuir versões:

```text id="8pp6yf"
contentVersion
apiVersion
gameVersion
```

Isso permitirá evolução sem destruir saves.

---

# 44. SAVE MIGRATION

Quando um sistema oficial mudar:

```text id="m8i1sn"
content v1
↓
migration
↓
content v2
```

---

# 45. BALANCEAMENTO

O conteúdo oficial deve possuir uma progressão coerente.

Nenhuma família deve destruir automaticamente as outras.

Exemplo:

* tecnologia pode ser eficiente;
* magia pode ser flexível;
* agricultura pode facilitar recursos;
* mecânica pode automatizar processos;
* energia extrema pode sustentar infraestrutura pesada;
* espaço deve representar uma etapa avançada.

---

# 46. NÃO COPIAR MECÂNICAS LITERALMENTE

Mesmo quando a referência for importante:

não copiar:

* nomes proprietários;
* assets;
* modelos;
* texturas;
* código;
* interfaces;
* textos;
* progressão exatamente igual.

Usar como referência de design e implementar uma versão própria.

---

# 47. DOCUMENTAÇÃO DE REFERÊNCIAS

Criar:

```text id="2n3kco"
docs/content/references/
```

Cada sistema deverá possuir:

```text id="w3qj83"
REFERENCE.md
DESIGN.md
ARCHITECTURE.md
PROGRESSION.md
```

Explicando quais ideias serviram de inspiração e como foram transformadas para o NEXORA.

---

# 48. DEFINIÇÃO DE V1

A V1 não precisa possuir absolutamente tudo da lista funcionando perfeitamente.

Ela precisa possuir:

```text id="b94o7x"
Core
World
Player
Crafting
Inventory
Energy
Machines
Automation
Storage
um caminho mágico
um caminho agrícola
uma dimensão
um sistema avançado
```

Depois expandir.

---

# 49. VISÃO DE LONGO PRAZO

```text id="86km9s"
NEXORA
│
├── Survival
├── Industry
├── Automation
├── Magic
├── Agriculture
├── Logistics
├── Advanced Technology
├── Dimensions
├── Warfare
└── Space
```

Todos derivados do mesmo Core.

---

# 50. DEFINIÇÃO FINAL

O NEXORA deve chegar ao estado em que um jogador consiga instalar o jogo e encontrar, sem depender de mods externos:

```text id="d5vkm0"
automação
energia
máquinas
armazenamento avançado
agricultura
magia
equipamentos customizáveis
dimensões
tecnologia avançada
sistemas de grande energia
exploração espacial
```

Enquanto usuários avançados ainda possam instalar seus próprios mods por cima.

---

# PRINCÍPIO FINAL

> **Os mods que inspiraram o NEXORA não são a arquitetura do NEXORA.**
>
> **Eles são referências para categorias de gameplay que serão reinterpretadas e transformadas em sistemas nativos, próprios e modulares.**

O resultado esperado é:

```text id="3t4g4i"
NEXORA CORE
      ↓
PUBLIC API
      ↓
OFFICIAL CONTENT
      +
COMMUNITY MODS
      ↓
UM ÚNICO ECOSSISTEMA
```

**NEXORA não deve parecer um jogo básico com quinze mods obrigatórios.**

Ele deve parecer um jogo completo que, além de possuir seu próprio conteúdo, nasceu preparado para ser expandido.
