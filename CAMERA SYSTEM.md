# NEXORA — CAMERA SYSTEM

> Camera converts gameplay/world state into a view without becoming authoritative gameplay logic.

## Responsibilities
- perspective/orthographic cameras;
- target tracking;
- view and projection state;
- first/third-person modes;
- interpolation and camera-relative input;
- collision-aware camera placement;
- cinematic/debug cameras;
- large-world precision support.

## Separation
Player/Vehicle define desired viewpoint context. Camera resolves presentation. Renderer consumes the final camera state.

## API
```ts
interface ICameraSystem {
  create(definition: CameraDefinition): CameraID;
  setMode(id: CameraID, mode: CameraMode): void;
  setTarget(id: CameraID, target: CameraTarget): void;
  sample(id: CameraID, time: RenderTime): CameraState;
}
```

## Large worlds
Camera transforms must work with the Spatial/Coordinate System and floating-origin/local-frame rules.

## Tests
Mode switching, interpolation, obstacle avoidance, high-coordinate precision, vehicle camera, replay camera and split-view readiness.

## Invariants
- Camera does not own gameplay transforms.
- Rendering uses presentation state derived from authoritative state.
