//! The plan's "window" stage (ADR-0027, DEBT-0008): the first render pass,
//! drawn into a real window and presented, frame after frame.
//!
//! The window host owns the event loop (ADR-0027), so this stage is a
//! [`Client`]: it opens the render pass on the window's device, draws the
//! same meshed 16³ region the frame-time stage draws, through the same camera,
//! and presents it. The first frame shown is read back **from the surface**,
//! after presentation, and checked against the CPU ray cast
//! (`nexora_render::reference`) before any number is kept. A window that shows
//! the wrong picture quickly does not get a row in the table.
//!
//! What it measures is what a player would see: the interval between frames
//! reaching the window, and how long the first one took to get there. With the
//! FIFO present mode (the one every platform guarantees), a real display
//! paces frames to its refresh rate. That is the point: on the operator's
//! desktop the interval says whether a frame of the engine fits under the
//! monitor's, and on a virtual display it says what the engine costs without
//! one.
//!
//! One event loop per process (winit's rule), so this stage runs once, from
//! the benchmark's `main`, on the main thread (macOS's rule).

use std::time::Instant;

use nexora_camera::{Camera, RenderOrigin};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_mesh::{ChunkMesh, Extent};
use nexora_render::reference::{check_frame, FrameCheck};
use nexora_render::{ChunkPass, GpuChunk};
use nexora_rhi::{CommandList, Rhi, TextureDesc, TextureFormat, TextureHandle, Usage};
use nexora_rhi_wgpu::WgpuRhi;
use nexora_simulation::WorldSurfaces;
use nexora_window::{run, Client, Flow, WindowFacts, WindowSpec};

use crate::gpu::ChunkScene;
use crate::{record_quantity, Budget, Measurement, Unit};

/// Why the stage did not run when the machine says it has no display.
pub const DECLARED_NO_DISPLAY: &str = "NEXORA_DISPLAY=none: this machine declares no display";

/// Why the stage did not run when no window could be opened.
pub const NO_DISPLAY: &str = "no display: the window host could not open a window";

/// Why the stage did not run when the surface cannot be read back.
pub const NOT_READABLE: &str =
    "this surface cannot be read back, so a shown frame cannot be checked before it is timed";

/// The window's size: the frame-time stage's, so the two stages draw the same
/// pixels.
const SIZE: u32 = 256;

/// What the window stage found on this machine.
#[derive(Debug)]
pub struct WindowStage {
    /// The measurements, empty when nothing was measured.
    pub measurements: Vec<Measurement>,
    /// Why nothing was measured, when nothing was.
    pub gap: Option<&'static str>,
}

impl WindowStage {
    fn missing(reason: &'static str) -> Self {
        Self {
            measurements: Vec::new(),
            gap: Some(reason),
        }
    }
}

/// Open a window, draw the chunk in it for a number of frames set by
/// `budget`, and measure what reached it.
///
/// `rhi_gap` is the RHI stage's: without an adapter there is nothing to draw
/// with, for the same reason.
///
/// # Errors
///
/// The frame shown in the window does not match the ray cast, or the host
/// failed after the window opened. No display is a gap, not an error, as no
/// adapter is for the RHI stage.
pub fn window(budget: Budget, rhi_gap: Option<&'static str>) -> Result<WindowStage> {
    if let Some(reason) = rhi_gap {
        return Ok(WindowStage::missing(reason));
    }
    if std::env::var("NEXORA_DISPLAY").as_deref() == Ok("none") {
        return Ok(WindowStage::missing(DECLARED_NO_DISPLAY));
    }
    let scene = ChunkScene::new()?;
    let view = WorldSurfaces::untextured(&scene.world);
    let mesh = nexora_mesh::mesh_region(&view, scene.region);
    let frames = 2 + u64::from(budget.samples) * 10;
    let mut client = ChunkWindow {
        view: &view,
        region: scene.region,
        mesh: &mesh.opaque,
        camera: scene.camera,
        frames,
        started: Instant::now(),
        live: None,
        first_frame_ns: None,
        intervals: Vec::new(),
        check: None,
        unreadable: false,
    };
    let spec = WindowSpec {
        title: "NEXORA benchmark: window stage".into(),
        width: SIZE,
        height: SIZE,
    };
    if let Err(error) = run(&spec, &mut client) {
        // The host could not give us a window at all: a machine without a
        // display, as a machine without a GPU is for the RHI stage.
        if error.domain() == Domain::Platform && client.check.is_none() && !client.unreadable {
            eprintln!("window stage not measured: {error}");
            return Ok(WindowStage::missing(NO_DISPLAY));
        }
        return Err(error);
    }
    if client.unreadable {
        return Ok(WindowStage::missing(NOT_READABLE));
    }
    let check = client
        .check
        .ok_or_else(|| wrong("the window closed before a frame was checked"))?;
    let first = client
        .first_frame_ns
        .ok_or_else(|| wrong("no frame reached the window"))?;
    if client.intervals.is_empty() {
        return Err(wrong("the window showed one frame and no more"));
    }
    Ok(WindowStage {
        measurements: vec![
            Measurement {
                name: "window.first_frame",
                note: "Open the event loop, the window, a device and its surface, upload the chunk, and show the first frame: startup, once",
                unit: Unit::TimePerOp,
                samples: vec![first],
                bytes_per_op: None,
            },
            Measurement {
                name: "window.present_chunk_16",
                note: "Interval between frames reaching a 256x256 window: the meshed 16 cubed region drawn and presented (FIFO: a real display paces it to its refresh)",
                unit: Unit::TimePerOp,
                samples: client.intervals,
                bytes_per_op: None,
            },
            record_quantity(
                "window.pixels_judged",
                "Of the first shown frame's 65,536 pixels, read back from the surface, those checked against the CPU ray cast (all matched)",
                check.judged as u64,
            ),
        ],
        gap: None,
    })
}

/// The render pass, living in a window.
struct ChunkWindow<'a, V: nexora_mesh::VoxelView + ?Sized> {
    view: &'a V,
    region: Extent,
    mesh: &'a ChunkMesh,
    camera: Camera,
    frames: u64,
    started: Instant,
    live: Option<Live>,
    first_frame_ns: Option<f64>,
    intervals: Vec<f64>,
    check: Option<FrameCheck>,
    unreadable: bool,
}

/// What exists on the device while the window is open.
struct Live {
    pass: ChunkPass,
    chunk: GpuChunk,
    target: TextureHandle,
    depth: TextureHandle,
    width: u32,
    height: u32,
    shown: u64,
    last: Option<Instant>,
}

impl<V: nexora_mesh::VoxelView + ?Sized> Client for ChunkWindow<'_, V> {
    fn opened(&mut self, rhi: &mut WgpuRhi, window: &WindowFacts) -> Result<()> {
        let (width, height) = (window.width, window.height);
        let texture = |label: &str, format, usage| TextureDesc {
            label: label.into(),
            width,
            height,
            format,
            usage,
        };
        let target = rhi.create_texture(&texture(
            "window stage",
            TextureFormat::Rgba8Unorm,
            Usage::RENDER_TARGET,
        ))?;
        let depth = rhi.create_texture(&texture(
            "window stage depth",
            TextureFormat::Depth32Float,
            Usage::RENDER_TARGET,
        ))?;
        let pass = ChunkPass::new(rhi, TextureFormat::Rgba8Unorm)?;
        let mut upload = CommandList::new("window stage upload");
        let chunk = pass.upload(
            rhi,
            &mut upload,
            self.mesh,
            self.region,
            &nexora_render::Corners::of(&[self.mesh]),
        )?;
        let fence = rhi.submit(upload)?;
        rhi.wait(fence)?;
        self.live = Some(Live {
            pass,
            chunk,
            target,
            depth,
            width,
            height,
            shown: 0,
            last: None,
        });
        Ok(())
    }

    fn frame(&mut self, rhi: &mut WgpuRhi) -> Result<Flow> {
        let live = self
            .live
            .as_mut()
            .ok_or_else(|| wrong("a frame before the window opened"))?;
        if live.shown >= self.frames {
            // Finished and released: a redraw the platform had queued must
            // not draw again (ADR-0027).
            return Ok(Flow::Exit);
        }
        let state = self.camera.sample(
            RenderOrigin::containing(self.camera.position())?,
            live.width,
            live.height,
        )?;
        let mut frame = CommandList::new("window stage frame");
        let stats = live
            .pass
            .record(&mut frame, &state, live.target, live.depth, &[live.chunk])?;
        if stats.drawn != 1 {
            return Err(wrong("the frame did not draw the region it looks at"));
        }
        rhi.submit(frame)?;

        if live.shown == 0 {
            let shown = match rhi.present_and_capture(live.target) {
                Ok(shown) => shown,
                Err(error) if error.recovery() == Recovery::Retry => return Ok(Flow::Wait),
                Err(error) => return Err(error),
            };
            self.first_frame_ns = Some(self.started.elapsed().as_nanos() as f64);
            let Some(shown) = shown else {
                self.unreadable = true;
                return finish(rhi, live);
            };
            if (shown.width, shown.height) != (live.width, live.height) {
                return Err(wrong("the surface is not the size the frame was drawn at")
                    .with_context("surface", format!("{}x{}", shown.width, shown.height))
                    .with_context("frame", format!("{}x{}", live.width, live.height)));
            }
            let rgba = to_rgba(&shown.format, shown.texels)?;
            let check = check_frame(
                self.view,
                self.region,
                &self.camera,
                (live.width, live.height),
                &rgba,
            )?;
            if check.matching != check.judged || check.judged * 2 < check.pixels {
                return Err(
                    wrong("the window does not show what a ray cast says it must")
                        .with_context("judged", check.judged.to_string())
                        .with_context("matching", check.matching.to_string())
                        .with_context("pixels", check.pixels.to_string()),
                );
            }
            self.check = Some(check);
        } else {
            match rhi.present(live.target) {
                Ok(()) => {}
                Err(error) if error.recovery() == Recovery::Retry => return Ok(Flow::Wait),
                Err(error) => return Err(error),
            }
        }
        let now = Instant::now();
        if let Some(last) = live.last {
            self.intervals.push((now - last).as_nanos() as f64);
        }
        live.last = Some(now);
        live.shown += 1;
        if live.shown < self.frames {
            return Ok(Flow::Continue);
        }
        finish(rhi, live)
    }
}

/// Release everything and end the run.
fn finish(rhi: &mut WgpuRhi, live: &mut Live) -> Result<Flow> {
    live.shown = u64::MAX;
    live.chunk.destroy(rhi)?;
    live.pass.destroy(rhi)?;
    rhi.destroy_texture(live.target)?;
    rhi.destroy_texture(live.depth)?;
    Ok(Flow::Exit)
}

/// A surface image in RGBA order, whatever order the window system chose.
fn to_rgba(format: &str, mut texels: Vec<u8>) -> Result<Vec<u8>> {
    match format {
        "Rgba8Unorm" | "Rgba8UnormSrgb" => Ok(texels),
        "Bgra8Unorm" | "Bgra8UnormSrgb" => {
            for texel in texels.chunks_exact_mut(4) {
                texel.swap(0, 2);
            }
            Ok(texels)
        }
        other => Err(wrong("a surface format this stage does not read")
            .with_context("format", other.to_string())),
    }
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Render, "benchmark-window", message).with_recovery(Recovery::Manual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bgra_surface_is_read_as_rgba() {
        let rgba = to_rgba("Bgra8UnormSrgb", vec![1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(rgba, vec![3, 2, 1, 4, 7, 6, 5, 8]);
        assert_eq!(
            to_rgba("Rgba8Unorm", vec![1, 2, 3, 4]).unwrap(),
            vec![1, 2, 3, 4]
        );
        assert!(to_rgba("Rgba16Float", vec![0; 8]).is_err());
    }

    #[test]
    fn without_an_adapter_or_a_display_the_stage_says_why_and_opens_nothing() {
        // Opening a real window needs the main thread, so the stage itself
        // runs in `tests/window_stage.rs`. Here: the gaps, which open nothing.
        let budget = Budget::coarse(1).smoke();
        let stage = window(budget, Some(crate::gpu::NO_ADAPTER)).unwrap();
        assert_eq!(stage.gap, Some(crate::gpu::NO_ADAPTER));
        assert!(stage.measurements.is_empty());
    }
}
