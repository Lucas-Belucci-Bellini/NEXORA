# NEXORA — INPUT SYSTEM

> Input traduz sinais de hardware e entrada local em intenções sem executar gameplay.

## Camadas
```text
Hardware
→ Raw Input
→ Device Abstraction
→ Input Actions
→ Mapping Context
→ Modifiers / Chords
→ Player / UI Intent
→ Command
```

## Responsabilidades
- keyboard, mouse, gamepad, touch e futuros dispositivos;
- remapeamento em runtime;
- contexts por estado de jogo;
- dead zones, sensitivity e curvas;
- conflitos de bindings;
- input recording para testes/replay.

## Regras
Input nunca altera o mundo diretamente. A camada local produz intenção; Command/Server determinam autoridade.

## API
```ts
interface IInputSystem {
  registerAction(action: InputActionDefinition): void;
  setContext(context: InputContextID, priority: number): void;
  bind(binding: InputBinding): Result;
  sample(frame: InputFrame): InputSnapshot;
}
```

## Integração
UI pode capturar input. Player/Vehicle/Interaction consomem ações sem depender de scancodes específicos. Accessibility pode alterar bindings.

## Tests
Binding conflict, context priority, device disconnect, remap persistence, deterministic input replay e server/client input validation.

## Security
Client input is untrusted. Server never accepts client claims of final state; only validated intent/snapshots are transported.
