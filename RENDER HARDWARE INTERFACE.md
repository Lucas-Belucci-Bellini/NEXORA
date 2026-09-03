# NEXORA — RENDER HARDWARE INTERFACE (RHI)

> The RHI isolates graphics API details from the Renderer. NEXORA gameplay must not depend directly on Vulkan, DirectX, Metal or another graphics API.

## Architecture
```text
Renderer
→ Render Graph / Resources
→ RHI
→ Graphics Backend
→ GPU / Driver
```

## Responsibilities
- device/adapter selection;
- buffers, textures, samplers and pipelines;
- command submission;
- synchronization/fences;
- shader resource binding;
- presentation/swapchain;
- GPU capability queries;
- debug markers and validation hooks.

## Backend model
A backend may target a platform graphics API. Backend-specific code stays below the RHI contract.

## Render Graph
Renderer builds logical passes and resource dependencies. RHI executes compiled backend commands.

## API sketch
```ts
interface IRHI {
  createBuffer(desc: BufferDesc): BufferHandle;
  createTexture(desc: TextureDesc): TextureHandle;
  createPipeline(desc: PipelineDesc): PipelineHandle;
  submit(commands: RenderCommands): Fence;
  present(target: PresentTarget): Result;
  capabilities(): RHICapabilities;
}
```

## Threading
The RHI accepts work produced by renderer jobs while preserving explicit synchronization. It must not expose unsafe driver ownership assumptions to gameplay systems.

## Headless
Server builds can use a null/headless backend without initializing a GPU.

## Tests
Resource lifetime, synchronization, device loss handling, backend parity, shader validation and headless startup.

## Invariants
- Renderer depends on RHI, never directly on a specific graphics API.
- Gameplay systems do not issue raw GPU commands.
- Backend replacement does not change simulation contracts.
