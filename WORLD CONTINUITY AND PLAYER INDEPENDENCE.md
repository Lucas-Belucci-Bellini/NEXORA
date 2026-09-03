# NEXORA — WORLD CONTINUITY AND PLAYER INDEPENDENCE

> **Princípio central:**
> **O mundo não existe para servir ao jogador. O jogador é um agente dentro de um mundo que possui populações, instituições, economia, memória, conflitos e objetivos próprios.**

Este documento define como manter o mundo vivo quando o jogador está presente, ausente, distante, dormindo, explorando outra região ou completamente fora de uma linha de acontecimentos.

---

# 1. Player is not the simulation center

A simulação deve ser capaz de produzir mudanças sem intervenção do jogador.

```text
PLAYER ONLINE
→ world simulation continues

PLAYER ABSENT
→ world simulation continues
```

O jogador pode ser muito influente, mas influência não significa autoridade sobre todos os sistemas.

---

# 2. Autonomous world

Civilizações, assentamentos, NPCs, instituições, mercados, ecossistemas e outros sistemas possuem objetivos próprios.

```text
WORLD
├── populations
├── settlements
├── factions
├── governments
├── economies
├── industries
├── research
├── ecology
├── infrastructure
├── cultures
└── conflicts
```

Cada sistema pode produzir alterações sem uma missão ou script específico para o jogador.

---

# 3. Player as an agent

O jogador é tratado como mais um agente simulado.

```text
PLAYER
≠
SPECIAL WORLD AUTHORITY
```

Ele pode possuir habilidades especiais relacionadas ao controle direto, mas não recebe conhecimento absoluto, imunidade narrativa ou controle de causalidade.

---

# 4. Consequence pipeline

Toda ação relevante segue:

```text
PLAYER INTENT
↓
COMMAND
↓
AUTHORITY VALIDATION
↓
DOMAIN SYSTEM
↓
STATE CHANGE
↓
EVENT
↓
CONSEQUENCES
↓
HISTORY
↓
KNOWLEDGE
↓
WORLD RESPONSE
```

---

# 5. Exemplos de agência

Uma ação pode resultar em:

```text
reputation changes
political response
economic reaction
social reaction
investigation
migration
market changes
institutional decisions
future conflicts
historical records
```

Não há necessidade de transformar cada ação em uma quest.

---

# 6. Hidden information

O jogador não deve receber automaticamente a informação que a IA possui.

```text
NPC KNOWLEDGE
≠
PLAYER KNOWLEDGE
≠
WORLD TRUTH
```

Informações podem precisar ser descobertas por interação, observação, pesquisa ou comunicação.

---

# 7. Example: undiscovered action

```text
Player changes a hidden part of the world
↓
no observer
↓
no communication
↓
no public knowledge
```

O sistema pode continuar reagindo fisicamente mesmo sem produzir uma reação social imediata.

---

# 8. Example: investigation

```text
unexplained absence
↓
witness remembers unusual activity
↓
local investigation
↓
partial evidence
↓
faction learns
↓
possible response
```

A investigação pode falhar ou nunca encontrar uma resposta conclusiva.

---

# 9. Delayed consequences

A ação pode produzir efeitos muito depois.

```text
YEAR 100
player action

YEAR 103
institution notices

YEAR 110
political group reacts

YEAR 130
major conflict
```

A causalidade histórica deve manter o vínculo entre os acontecimentos quando aplicável.

---

# 10. Political reaction

Governos devem tomar decisões baseadas em:

```text
legitimacy
resources
public support
faction relations
intelligence
security
institutions
leadership
military/economic capacity
```

O governo não deve reagir automaticamente ao jogador só porque ele é o protagonista.

---

# 11. Coalition emergence

Uma coalizão deve emergir das relações entre agentes.

```text
pressure
+
shared interests
+
communication
+
coordination ability
+
opportunity
→ coalition candidate
```

A coalizão pode:

```text
form
remain secret
dissolve
split
negotiate
change leadership
```

---

# 12. Twenty-five settlement example

Cenário:

```text
25 settlements
↓
player government centralizes power
↓
autonomy decreases
↓
local dissatisfaction grows
```

Possíveis resultados:

```text
0 settlements rebel
5 rebel
12 rebel
23 rebel
25 rebel
```

A quantidade depende do estado simulado, e não de uma narrativa fixa.

---

# 13. Asymmetric outcomes

Nem todas as organizações devem reagir da mesma forma.

Exemplo:

```text
Settlement A → rebellion
Settlement B → neutrality
Settlement C → negotiation
Settlement D → support government
Settlement E → migration
```

Isso produz política emergente.

---

# 14. Mercenary / hired agent concept

Atores externos podem receber contratos ou incentivos para executar tarefas dentro das regras de suas facções.

O sistema deve tratá-los como agentes com:

```text
contract
objective
information
loyalty
risk assessment
resources
```

Eles não devem possuir conhecimento que a organização contratante não poderia fornecer.

---

# 15. Player safety is not narrative immunity

Se o mundo possui ameaças políticas ou criminais, elas devem existir como parte do simulador e respeitar as regras de observação, informação e causalidade.

O jogador pode não saber antecipadamente que uma reação está sendo preparada.

A apresentação deve evitar conhecimento onisciente artificial apenas para criar surpresa.

---

# 16. World time

O Time/Calendar System é autoridade para:

```text
aging
travel
production
communication latency
political cycles
seasons
construction
research
historical progression
```

Eventos podem acontecer enquanto o jogador está distante.

---

# 17. Offline / absent simulation

Quando uma região não está em FULL:

```text
region
↓
regional simulation
↓
abstract simulation
```

A representação muda, mas as entidades e instituições importantes mantêm identidade e continuidade lógica.

---

# 18. LOD continuity

Cada transição deve preservar invariantes suficientes:

```text
population totals
settlement identity
government state
economic state
resource trends
major relationships
active historical events
critical infrastructure
```

Detalhes de baixo impacto podem ser agregados.

---

# 19. Simulation budgets

O mundo infinito não significa simular todos os detalhes em FULL.

O sistema distribui orçamento por:

```text
player proximity
importance
event activity
population
infrastructure
simulation dependencies
prediction value
```

Áreas historicamente importantes podem receber maior fidelidade quando necessário.

---

# 20. Wake-up conditions

Uma região abstrata pode retornar a maior resolução por:

```text
player arrival
major event
incoming dependency
high-value simulation condition
scheduled activity
investigation
network interaction
editor request
```

---

# 21. No script dependency

Um grande acontecimento não deve depender de um NPC específico existir para ocorrer.

Evitar:

```text
if Player enters region:
    start war
```

Preferir:

```text
world conditions
↓
AI / institutional decisions
↓
war candidate
↓
validation
↓
world event
```

---

# 22. Player can become central without being required

O jogador pode alcançar enorme importância se suas ações produzirem consequências suficientes.

```text
player action
↓
large state change
↓
major events
↓
historical importance
```

Mas isso é consequência do mundo, não uma obrigação narrativa.

---

# 23. Player can remain insignificant

Também é válido:

```text
player lives in village
↓
works locally
↓
never becomes famous
```

Enquanto isso:

```text
continent changes
civilizations rise
wars happen
technology advances
space exploration begins
```

---

# 24. Death / disappearance / succession

Personagens relevantes que desaparecem devem produzir consequências compatíveis com suas funções.

```text
leader disappears
↓
succession problem
↓
political reaction
↓
new leadership
```

Não deve existir regra geral de que o mundo para quando um personagem importante desaparece.

---

# 25. Player return

Ao retornar a uma região, o jogador deve receber informações baseadas naquilo que realmente se tornou observável.

```text
region state
+
historical changes
+
local knowledge
+
player relationships
=
return experience
```

A apresentação pode mostrar:

```text
new government
changed prices
damaged structures
new population
new roads
new monuments
new rumors
new historical events
```

---

# 26. History connection

Toda consequência importante pode entrar em:

```text
History System
Knowledge System
Lore System
Archive System
```

---

# 27. Debugging requirement

Desenvolvedores devem conseguir responder:

```text
Why did this faction act?
Why did this war start?
Why did this NPC know this?
Why did this city collapse?
Why did this event happen while the player was absent?
```

O sistema precisa registrar explicações, evidências, decisões e dependências suficientes para diagnóstico.

---

# 28. Determinism

Quando necessário, simulações abstratas devem preservar determinismo ou rastreabilidade suficiente para:

```text
replay
save/load
bug reproduction
benchmark
historical audit
```

---

# 29. Tests

```text
world runs without player
player absence
LOD transition
region wake-up
political independence
coalition emergence
information latency
discovery failure
delayed consequences
historical persistence
save/load
server restart
```

---

# 30. Regra final

> **O jogador pode mudar o mundo profundamente, mas nunca deve ser necessário para que o mundo exista.**

> **NEXORA deve parecer vivo não porque está sempre esperando o jogador, mas porque continua tomando decisões, sofrendo consequências, criando história e mudando enquanto o jogador está ocupado em outra coisa.**
