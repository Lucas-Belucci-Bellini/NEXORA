# NEXORA — DATA OWNERSHIP AND SOURCE OF TRUTH

## Princípio

Todo dado autoritativo deve possuir exatamente um owner lógico. Outros sistemas consultam, derivam ou solicitam mudança através de contratos.

## Exemplos

| Dado | Owner primário |
|---|---|
| Entity identity | Entity System |
| Component state | ECS / Data Runtime |
| Block state | Block / Voxel System |
| World time | Time / Calendar System |
| Position / spatial identity | Spatial / Entity runtime |
| Inventory contents | Inventory System |
| Item definition | Item / Registry |
| Attribute value | Attribute System |
| AI knowledge | Knowledge System |
| Population state | Civilization System |
| Economic account | Economy System |
| Industrial process | Advanced Industry |
| Historical event | History System |
| Historical evidence | Archive System |
| Lore claim | Lore System |
| World event lifecycle | World Events |
| Save metadata | Persistence |
| Network transport state | Networking |
| Render representation | Renderer / Presentation |

## Regras de acesso

```text
READ
→ query / snapshot / public view

WRITE
→ command / transaction / owner API

DERIVED DATA
→ cache, index ou representação descartável
```

## Anti-padrões proibidos

- múltiplos owners para a mesma verdade;
- sistemas escrevendo diretamente em componentes privados;
- UI alterando estado autoritativo sem command;
- cache tratado como fonte de verdade;
- duplicação silenciosa de dados persistentes.

## Diagnóstico

Quando dois sistemas discordarem, o owner do dado é a referência. A divergência deve ser tratada como bug de integração, não resolvida por uma terceira cópia.

## LOD

Conversões FULL → REGIONAL → ABSTRACT não transferem ownership. Elas mudam representação, mantendo identidade e contrato lógico.
