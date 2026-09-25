# ADR-0027 — The window host is winit, and it owns the event loop

- **Status:** ACCEPTED
- **Date:** 2026-09-25
- **Decides:** the fourth question ADR-0025 left open and ADR-0026 deferred:
  who owns the window, and therefore the event loop
- **Amends:** ADR-0002 (a second external dependency, in one crate); ADR-0026
  (the native backend presents when it is given a window)
- **Touches:** a new crate `engine/window`; `engine/rhi` (one shared rule, one
  more conformance check); `engine/rhi-wgpu` (surface and `present`);
  foundation (`Domain::Platform`); CI; `scripts/local-validation.py`

## Context

After ADR-0026 the freeze gate's second blocker reads *"the RHI has met a
driver, but not a window"*. The native backend passes the conformance suite
on Vulkan, Direct3D 12 and Metal, but every one of those runs had
`presents: false`: no window, no surface, nothing shown.

That was listed as waiting for a window host, which is code, not hardware.
Two facts measured in the development container on 2026-09-25 settle whether
it can also be *verified* here:

1. **Xvfb** installs in the container and on GitHub's Ubuntu runners: a real
   X server whose screen is memory. `winit` 0.30.13 opens an X11 window on it.
2. On that window, `wgpu` 30 over **lavapipe** creates a Vulkan surface,
   configures a swapchain (`Bgra8UnormSrgb`, FIFO) and presents frames. The
   surface allows copies out of its images, so what reached the window can be
   read back.

So the window, the swapchain and presentation are `NOT_IMPLEMENTED`, not
`BLOCKED_BY_ENVIRONMENT`, and CI can check them.

The window host also matters beyond the RHI. DEBT-0041 (no process runs
frames against a real clock) and DEBT-0043 (no real device has produced an
input signal) both wait for *"a host"*, and DEBT-0043 predicts it is one host:
*whoever has the clock has the devices*. Whoever owns the window owns the event
loop, and on macOS the event loop must own the **main thread**.

## Options

1. **`winit`**: Rust, the window layer `wgpu` is built and tested against. One
   API over Win32, AppKit, X11 and Wayland, event loop included. It speaks
   `raw-window-handle` 0.6, which is how `wgpu` receives a window, so the RHI
   never names it.
2. **SDL2 or GLFW**: mature C libraries. A C toolchain and a native library on
   every machine, and a language boundary (ADR-0009) for window events that
   arrive every frame: high frequency, low volume, the wrong shape for a
   boundary.
3. **Each platform's API directly** (`windows`, `objc2`, `x11rb`): three
   implementations of one concept, most of them `unsafe`, and the workspace
   forbids `unsafe`.

## Decision

Option 1, **`winit` pinned to `=0.30.13`**, in one crate: `engine/window`
(`nexora-window`). Default features are off. On are `rwh_06`, which hands the
window to `wgpu`, and `x11`.

- **Wayland is left out, for now.** Its client libraries must be present when
  the crate is *built*. With it on, the workspace no longer builds on a Linux
  machine without them, and the container is one. X11 works under XWayland on
  every mainstream Wayland desktop. Windows and macOS need no feature: their
  back ends are always compiled.
- **The host runs the loop, and calls back.** `nexora_window::run(spec,
  client)` opens the window, opens a `WgpuRhi` on it, and calls a `Client`:
  `opened` once, then `frame` for every redraw until it returns `Flow::Exit`
  or the window closes. It resizes the surface when the window changes size.
  This callback is where the frame clock (DEBT-0041) and real input
  (DEBT-0043) will enter. Neither is built here.
- **The RHI does not know about `winit`.** `WgpuRhi::with_surface` takes
  anything `wgpu` can make a surface from. `nexora-rhi-wgpu` gains no
  dependency, and `nexora-rhi` still has none.
- **`present` copies with a draw.** The surface's images belong to the window
  system, in a format it picks (BGRA here). `present` draws the render target
  onto the next image with one full-screen triangle, sampled *nearest*, which
  converts channel order, colour space and size. Then it shows the image.
  Nearest sampling keeps 16×16 art exact at any window size, and it lets a
  readback be checked texel by texel. The surface prefers an 8-bit sRGB format
  and FIFO, the one present mode every platform guarantees.
- **Presenting is GPU work, and is tracked like a draw.** `present` takes a
  fence, and the presented texture's memory waits for it if its handle dies
  first. The conformance suite now checks this on any backend that presents.
- **One rule moved into the shared checks**: a depth texture cannot be
  presented (`kit::presentable`). The null backend and `WgpuRhi` refuse it the
  same way, and the suite checks it.
- **No display is a failure, not a skip**, as ADR-0026 made no GPU. A machine
  without one declares it with `NEXORA_DISPLAY=none`.
- Window errors get their own domain, `Domain::Platform`.

### How it is verified

| where | what runs | what it proves |
| --- | --- | --- |
| every developer machine, and CI on Linux under Xvfb | `cargo test -p nexora-window`: a real window, conformance with `presents: true`, three frames, the first read back from the surface | a frame reaches a window, on a conformant Vulkan driver and a real X server |
| CI smoke | `nexora-window-probe` under Xvfb, grepped: `conformance 9/9 cases on wgpu, presenting`, `65536 of 65536 window texels show the 16x16 target`, `61 frames, 60 of them by the probe` | the release binary does the same |
| CI on Windows and macOS | the same test, inside `cargo test --workspace` | the host on Win32 and AppKit, on each runner's display |
| the operator's machine | `local-validation.py` → new check `window` (the same probe) | the same, on **real hardware and a real desktop**, recorded against a commit |

The probe shows a 16×16 target of four coloured quadrants, not a flat colour.
A frame that arrives upside down, mirrored or with red and blue swapped fails
the readback. Where a platform does not allow reading the surface, the probe
says `not readable on this surface` rather than claiming the check.

## Verified

- In the container, under Xvfb, on lavapipe: the probe opens a 256×256 X11
  window with a `Bgra8UnormSrgb` FIFO surface, passes the nine conformance
  cases with presentation on, and presents 61 frames. The first frame read
  back from the surface matches the target in all 65,536 texels.
- Mutation-checked. Flipping the blit's Y axis fails the readback (0 of
  65,536). Not tracking the present's fence fails the conformance case
  `present` ("a presented texture's memory outlives its handle until the frame
  is done").
- Without a display the probe and the test fail and say why (`neither
  WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set`). With
  `NEXORA_DISPLAY=none` the test reports that it did not run.
- The workspace lockfile grows from 122 packages to 235, counting every
  platform's bindings. Most are `winit`'s per-platform crates, which a given
  build compiles only for its own target.

## Consequences

- The freeze checklist's RHI row: **the native backend presents**, verified on
  a software Vulkan driver and a virtual display. It is **not yet verified on
  the operator's GPU and desktop**. The second blocker narrows to *one report
  from real hardware*: `rhi_native` and `window`.
- The local report's `window`, `swapchain` and `presentation` items leave the
  `NOT_IMPLEMENTED` list. They are one check, `window`, that can pass or fail
  on a machine. `rendering`, `input_devices`, `client_mode` and
  `benchmark_gpu_stages` stay gated: their code does not exist.
- DEBT-0041 and DEBT-0043 have a host to be built in. DEBT-0043's trigger
  (*"any process with a window or an OS event loop"*) has fired.
- The input boundary row stops being *"meaningless without a window"*: there is
  a window. It is still unbuilt.

## Not claimed

No renderer: the probe presents a texture it uploaded, and the one triangle
drawn so far is the conformance suite's. No frame pacing, no clock, no input.
And nothing has been shown on a physical monitor from this repository's CI.
Xvfb's screen is memory, and each runner's display is whatever its image
provides.

## Migration

None. Nothing persisted refers to a window.

## Compatibility

`WgpuRhi::with_surface`, `resize`, `surface` and `present_and_capture` are
additions. `WgpuRhi::new` behaves as before, with no surface. `Domain` is
`#[non_exhaustive]`, so `Platform` is additive. The new conformance checks
only apply to a backend that presents, or that offers a depth format; the null
backend passes them unchanged.
