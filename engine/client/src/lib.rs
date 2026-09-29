//! The runtime in client mode (ADR-0032).
//!
//! `NEXORA DEVELOPMENT ROADMAP.md`, Phase 1's exit: *"runtime mínimo inicia
//! em modo client/headless"*. Headless has run since the first slice. This
//! crate is the other half, and it is not a second engine: it walks the same
//! checked [`Lifecycle`] in [`RuntimeMode::Client`], resolves the same module
//! graph (where the client-only renderer module now initializes instead of
//! being skipped), creates and brings online the same [`World`], and then
//! does what headless cannot: opens a window (ADR-0027), enters
//! [`Phase::PresentationRunning`], and runs frames.
//!
//! A frame, in `CORE.md` §16's order, through `runtime::frame`:
//!
//! 1. **Input**: the window host's signals for the frame (ADR-0031) sampled
//!    once into an [`controls::Intent`].
//! 2. **Simulation**: the fixed steps the real time since the last frame
//!    buys ([`FrameSchedule`]): the world clock advances one tick a step, and
//!    the camera moves by the intent for each step.
//! 3. **Render prep**: the render origin follows the camera (ADR-0029), the
//!    camera is sampled, and the frame is recorded by the first render pass
//!    (ADR-0030).
//! 4. **Render**: one submission, and the frame presented to the window.
//!
//! The time between frames is measured with the host's clock and handed in;
//! the loop itself still reads no clock (ADR-0017). This is the first process
//! that runs `runtime::frame` against a real clock (DEBT-0041).
//!
//! The first frame shown is read back from the surface and, after the window
//! closes, checked against the CPU ray cast (`nexora_render::reference`) over
//! every drawn column: the client does not claim to draw the world unless the
//! pixels say so.
//!
//! Not built: streaming into the pass (the world is generated and meshed
//! before the first frame), textures, pointer look, a player body, physics in
//! the frame, audio. The frame budget is the arithmetic
//! [`FrameBudget::doubling_from`] the step, not a measured one.

pub mod controls;
pub mod scene;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use nexora_camera::{Camera, RenderOrigin};
use nexora_foundation::diagnostics::{Category, Diagnostics, Level, StderrSink};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::time::{CalendarConfig, TimeScale};
use nexora_mesh::ChunkMesh;
use nexora_render::reference::{check_frame, FrameCheck};
use nexora_render::{ChunkPass, Corners, FrameStats, GpuChunk};
use nexora_rhi::{CommandList, Rhi, TextureDesc, TextureFormat, TextureHandle, Usage};
use nexora_rhi_wgpu::WgpuRhi;
use nexora_runtime::frame::{BudgetClass, FrameBudget, FrameLoop, FrameSchedule, FrameStage};
use nexora_runtime::input::InputFrame;
use nexora_runtime::lifecycle::{Lifecycle, Phase, RuntimeMode};
use nexora_runtime::module::{
    EngineModule, ModuleContext, ModuleDependency, ModuleId, ModuleManager, ModuleSides,
};
use nexora_simulation::WorldSurfaces;
use nexora_window::input::InputCounts;
use nexora_window::{run, Client, Flow, WindowFacts, WindowSpec};
use nexora_world::persist;
use nexora_world::world::{World, WorldDescriptor};

use crate::controls::{Controls, Intent};
use crate::scene::Scene;

/// The window's title.
pub const TITLE: &str = "NEXORA";

/// One fixed step: a world tick, 20 Hz.
pub const STEP: Duration = Duration::from_millis(50);

/// Steps one frame may run before the rest are dropped (ADR-0017).
pub const MAX_STEPS: u32 = 4;

/// How the client is started.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// World seed.
    pub seed: u64,
    /// Columns drawn on each side of the origin.
    pub radius: i64,
    /// Window width.
    pub width: u32,
    /// Window height.
    pub height: u32,
    /// Stop after this many frames shown; `None` runs until Escape or the
    /// window closes.
    pub frames: Option<u64>,
    /// Stop after this long in the frame loop, whatever else happens.
    pub timeout: Option<Duration>,
    /// Where the world is flushed at shutdown; `None` encodes it but writes
    /// nothing.
    pub save: Option<PathBuf>,
    /// Log lifecycle progress to standard error.
    pub verbose: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            seed: 0x4E58_4F52,
            radius: 1,
            width: 384,
            height: 256,
            frames: None,
            timeout: None,
            save: None,
            verbose: false,
        }
    }
}

/// Why the frame loop stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// `frames` were shown.
    FrameLimit,
    /// The exit action (Escape).
    ExitKey,
    /// The window was closed from outside.
    WindowClosed,
    /// `timeout` passed.
    Timeout,
}

impl Ending {
    /// A stable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FrameLimit => "frame limit",
            Self::ExitKey => "exit key",
            Self::WindowClosed => "window closed",
            Self::Timeout => "timeout",
        }
    }
}

/// What a client run did.
#[derive(Debug, Clone)]
pub struct ClientReport {
    /// Lifecycle phases entered, in order.
    pub phases: Vec<&'static str>,
    /// Modules initialized, in order.
    pub modules: Vec<String>,
    /// The adapter, as `name (backend, kind)`.
    pub adapter: String,
    /// The window.
    pub window: WindowFacts,
    /// Columns drawn, and generated (drawn plus a ring).
    pub columns: (usize, usize),
    /// The drawn band, bottom and top block.
    pub band: (i64, i64),
    /// The first frame shown against the ray cast, or `None` when the
    /// surface cannot be read back.
    pub first_frame: Option<FrameCheck>,
    /// Frames shown.
    pub frames: u64,
    /// Why the loop stopped.
    pub ending: Ending,
    /// Frames per budget class: target, warning, critical, emergency.
    pub classes: [u64; 4],
    /// World ticks advanced (fixed steps run).
    pub ticks: u64,
    /// Steps owed and dropped because a frame was too late.
    pub discarded: u64,
    /// Frame wall times: median, 95th percentile, maximum.
    pub wall: [Duration; 3],
    /// Time inside frames no stage claimed, summed.
    pub unattributed: Duration,
    /// The last frame's pass statistics.
    pub drawn: FrameStats,
    /// Device events, as the host counted them.
    pub input: InputCounts,
    /// Frames in which at least one client action was held.
    pub active_frames: u64,
    /// Where the camera went: forward, right and up of where it started, in
    /// the starting camera's horizontal frame, and how far it turned.
    pub moved: [f64; 3],
    /// Yaw turned, in degrees.
    pub turned: f64,
    /// The flushed world's size, and where it was written.
    pub flushed: (usize, Option<PathBuf>),
}

/// Start the runtime in client mode, run it until it ends, and shut it down.
///
/// Must be called from the main thread (the window host's rule).
///
/// # Errors
///
/// Any lifecycle step failed, no window could be opened, or the first frame
/// shown is not what the ray cast says it must be. The lifecycle records the
/// failure; nothing that failed is reported as running.
pub fn run_client(config: &ClientConfig) -> Result<ClientReport> {
    let diagnostics = if config.verbose {
        Diagnostics::new(vec![Box::new(StderrSink)], Level::Info)
    } else {
        Diagnostics::silent()
    };
    let mut lifecycle = Lifecycle::new(RuntimeMode::Client);
    match run_in(config, &mut lifecycle, &diagnostics) {
        Ok(report) => Ok(report),
        Err(error) => {
            lifecycle.fail(error.clone());
            Err(error)
        }
    }
}

fn run_in(
    config: &ClientConfig,
    lifecycle: &mut Lifecycle,
    diagnostics: &Diagnostics,
) -> Result<ClientReport> {
    lifecycle.advance_through(Phase::FoundationInit)?;
    log(diagnostics, "runtime foundation initialized");

    // --- modules: the headless graph, with the renderer on its side ---------
    let mut modules = ModuleManager::new();
    modules.register(Box::new(FoundationModule))?;
    modules.register(Box::new(WorldModule))?;
    modules.register(Box::new(RendererModule))?;
    lifecycle.advance_to(Phase::ModuleDiscovery)?;
    modules.resolve(RuntimeMode::Client)?;
    lifecycle.advance_to(Phase::ModuleResolution)?;
    let context = ModuleContext::new(RuntimeMode::Client, diagnostics.clone());
    modules.initialize_all(&context)?;
    let initialized: Vec<String> = modules
        .resolved_order()
        .iter()
        .map(ToString::to_string)
        .collect();
    lifecycle.advance_through(Phase::RuntimeInit)?;
    log(diagnostics, "engine modules initialized");

    // --- world --------------------------------------------------------------
    let descriptor = WorldDescriptor::new("nexora-client", config.seed)?;
    let mut world = World::create(descriptor, CalendarConfig::earthlike())?;
    lifecycle.advance_to(Phase::WorldAttach)?;
    world.bring_online()?;
    lifecycle.advance_to(Phase::SimulationRunning)?;
    let scene = scene::build(&mut world, config.radius)?;
    log(diagnostics, "world generated and meshed");

    // --- presentation -------------------------------------------------------
    let start_camera = scene.camera;
    let mut presentation = Presentation {
        world: &mut world,
        lifecycle,
        scene: &scene,
        controls: Controls::new()?,
        camera: scene.camera,
        origin: RenderOrigin::containing(scene.camera.position())?,
        frame_loop: FrameLoop::new(
            FrameSchedule::new(STEP, MAX_STEPS)?,
            FrameBudget::doubling_from(STEP),
        ),
        pending: InputFrame::new(),
        limit: config.frames,
        timeout: config.timeout,
        live: None,
        adapter: String::new(),
        tally: Tally::default(),
    };
    let spec = WindowSpec {
        title: TITLE.into(),
        width: config.width,
        height: config.height,
    };
    let session = run(&spec, &mut presentation)?;
    let Presentation {
        camera,
        tally,
        adapter,
        ..
    } = presentation;
    let ending = if session.closed {
        Ending::WindowClosed
    } else {
        tally
            .ending
            .ok_or_else(|| wrong("the frame loop ended without a reason"))?
    };
    log(diagnostics, "presentation stopped");

    // --- the first frame, judged now that the world is not borrowed ---------
    let first_frame = match &tally.captured {
        Some(captured) => {
            let view = WorldSurfaces::untextured(&world);
            let check = check_frame(
                &view,
                scene.bounds,
                &captured.camera,
                (captured.width, captured.height),
                &captured.rgba,
            )?;
            if !frame_holds(&check) {
                return Err(
                    wrong("the window did not show what a ray cast says it must")
                        .with_context("judged", check.judged.to_string())
                        .with_context("matching", check.matching.to_string())
                        .with_context("backfacing", check.backfacing.to_string())
                        .with_context("snapped", check.snapped.to_string())
                        .with_context("pixels", check.pixels.to_string()),
                );
            }
            Some(check)
        }
        None if tally.unreadable => None,
        None => return Err(wrong("no frame reached the window")),
    };

    // --- shutdown -----------------------------------------------------------
    lifecycle.advance_to(Phase::ShutdownRequested)?;
    lifecycle.advance_to(Phase::SimulationStop)?;
    let container = persist::save(&world)?;
    let flushed = container.encode().len();
    if let Some(path) = &config.save {
        container.write_atomic(path)?;
    }
    lifecycle.advance_to(Phase::WorldFlush)?;
    modules.shutdown_all(&context);
    lifecycle.advance_through(Phase::ProcessExit)?;
    log(diagnostics, "runtime shut down");

    let mut walls = tally.walls;
    walls.sort_unstable();
    let percentile = |p: usize| {
        walls
            .get((walls.len().saturating_sub(1)) * p / 100)
            .copied()
            .unwrap_or_default()
    };
    Ok(ClientReport {
        phases: lifecycle
            .history()
            .iter()
            .map(|phase| phase.as_str())
            .collect(),
        modules: initialized,
        adapter,
        window: session.window,
        columns: (scene.regions.len(), scene.generated),
        band: (
            scene.bounds.origin.y,
            scene.bounds.origin.y + i64::from(scene.bounds.size[1]),
        ),
        first_frame,
        frames: tally.frames,
        ending,
        classes: tally.classes,
        ticks: tally.ticks,
        discarded: tally.discarded,
        wall: [percentile(50), percentile(95), percentile(100)],
        unattributed: tally.unattributed,
        drawn: tally.drawn,
        input: session.input,
        active_frames: tally.active_frames,
        moved: displacement(&start_camera, &camera),
        turned: (camera.yaw() - start_camera.yaw()).to_degrees(),
        flushed: (flushed, config.save.clone()),
    })
}

/// Pixels judged per snapped edge tolerated in the first frame.
///
/// Measured on lavapipe at 384×256 over twenty seeds, with the pass culling
/// back faces and its quads split at every corner (DEBT-0047, closed): 0 to 2
/// wrong pixels in about 79,500 judged, every one an edge the ray cast finds
/// within 1/64 px of the pixel's centre that the rasteriser placed on the
/// other side. The bound is far below the smallest wrong frame
/// mutation-checked (a `Less` depth test gets 28% wrong).
pub const SNAPPED_PER_JUDGED: usize = 5_000;

/// Whether a checked frame is the one the ray cast says: at least half the
/// pixels judged; every judged pixel matching except snapped edges, within
/// [`SNAPPED_PER_JUDGED`]; and **no back face at all**, which the pass culls,
/// so one on screen means culling or winding broke.
#[must_use]
pub fn frame_holds(check: &FrameCheck) -> bool {
    check.judged * 2 >= check.pixels
        && check.backfacing == 0
        && check.matching + check.snapped == check.judged
        && check.snapped * SNAPPED_PER_JUDGED <= check.judged
}

/// Where `now` is relative to `start`: forward, right and up in `start`'s
/// horizontal frame.
fn displacement(start: &Camera, now: &Camera) -> [f64; 3] {
    let (a, b) = (start.position(), now.position());
    let d = [b.x - a.x, b.y - a.y, b.z - a.z];
    let (sin_yaw, cos_yaw) = start.yaw().sin_cos();
    [
        -sin_yaw * d[0] - cos_yaw * d[2],
        cos_yaw * d[0] - sin_yaw * d[2],
        d[1],
    ]
}

/// The first shown frame, as read back from the surface.
struct Captured {
    camera: Camera,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// What the frames added up to.
#[derive(Default)]
struct Tally {
    frames: u64,
    classes: [u64; 4],
    ticks: u64,
    discarded: u64,
    walls: Vec<Duration>,
    unattributed: Duration,
    drawn: FrameStats,
    active_frames: u64,
    captured: Option<Captured>,
    unreadable: bool,
    ending: Option<Ending>,
    started: Option<Instant>,
    last: Option<Instant>,
}

/// What exists on the device while the window is open.
struct Live {
    pass: ChunkPass,
    chunks: Vec<GpuChunk>,
    target: TextureHandle,
    depth: TextureHandle,
    width: u32,
    height: u32,
}

/// The client's side of the window host: the frame loop, living in a window.
struct Presentation<'a> {
    world: &'a mut World,
    lifecycle: &'a mut Lifecycle,
    scene: &'a Scene,
    controls: Controls,
    camera: Camera,
    origin: RenderOrigin,
    frame_loop: FrameLoop,
    pending: InputFrame,
    limit: Option<u64>,
    timeout: Option<Duration>,
    live: Option<Live>,
    adapter: String,
    tally: Tally,
}

impl Client for Presentation<'_> {
    fn opened(&mut self, rhi: &mut WgpuRhi, window: &WindowFacts) -> Result<()> {
        let adapter = rhi.adapter();
        self.adapter = format!("{} ({}, {})", adapter.name, adapter.backend, adapter.kind);
        let (width, height) = (window.width, window.height);
        let texture = |label: &str, format, usage| TextureDesc {
            label: label.into(),
            width,
            height,
            format,
            usage,
        };
        let target = rhi.create_texture(&texture(
            "client frame",
            TextureFormat::Rgba8Unorm,
            Usage::RENDER_TARGET,
        ))?;
        let depth = rhi.create_texture(&texture(
            "client depth",
            TextureFormat::Depth32Float,
            Usage::RENDER_TARGET,
        ))?;
        let pass = ChunkPass::new(rhi, TextureFormat::Rgba8Unorm)?;
        let mut upload = CommandList::new("client upload");
        // Every column's corners, so the seams between columns are split
        // too (DEBT-0047).
        let meshes: Vec<&ChunkMesh> = self.scene.regions.iter().map(|(_, mesh)| mesh).collect();
        let corners = Corners::of(&meshes);
        let mut chunks = Vec::with_capacity(self.scene.regions.len());
        for (region, mesh) in &self.scene.regions {
            chunks.push(pass.upload(rhi, &mut upload, mesh, *region, &corners)?);
        }
        let fence = rhi.submit(upload)?;
        rhi.wait(fence)?;
        self.live = Some(Live {
            pass,
            chunks,
            target,
            depth,
            width,
            height,
        });
        self.lifecycle.advance_to(Phase::PresentationRunning)?;
        self.tally.started = Some(Instant::now());
        Ok(())
    }

    fn input(&mut self, frame: &InputFrame) {
        for signal in frame.signals() {
            self.pending.push(*signal);
        }
    }

    fn frame(&mut self, rhi: &mut WgpuRhi) -> Result<Flow> {
        if self.tally.ending.is_some() {
            // Finished and released: a redraw the platform had queued must
            // not draw again (ADR-0027).
            return Ok(Flow::Exit);
        }
        let Some(live) = self.live.as_ref() else {
            return Err(wrong("a frame before the window opened"));
        };
        let begun = Instant::now();
        let elapsed = self.tally.last.map_or(Duration::ZERO, |last| begun - last);
        self.tally.last = Some(begun);
        let mut frame = self.frame_loop.begin(elapsed);
        let plan = frame.plan();

        // Input.
        let clock = Instant::now();
        let intent: Intent = self.controls.sample(&std::mem::take(&mut self.pending));
        frame.record(FrameStage::Input, clock.elapsed())?;

        // Simulation: the steps real time bought.
        let clock = Instant::now();
        for _ in 0..plan.steps {
            self.world.clock_mut().advance_by(TimeScale::Tick, 1)?;
            controls::step(&mut self.camera, intent, STEP)?;
        }
        frame.record(FrameStage::Simulation, clock.elapsed())?;

        // Render prep: the origin follows the camera, the pass is recorded.
        let clock = Instant::now();
        self.origin = self.origin.follow(self.camera.position())?;
        let state = self.camera.sample(self.origin, live.width, live.height)?;
        let mut list = CommandList::new("client frame");
        let drawn = live
            .pass
            .record(&mut list, &state, live.target, live.depth, &live.chunks)?;
        frame.record(FrameStage::RenderPrep, clock.elapsed())?;

        // Render: one submission, then the window.
        let clock = Instant::now();
        rhi.submit(list)?;
        let shown = if self.tally.captured.is_none() && !self.tally.unreadable {
            match rhi.present_and_capture(live.target) {
                Ok(Some(image)) => {
                    if (image.width, image.height) != (live.width, live.height) {
                        return Err(wrong("the surface is not the size the frame was drawn at")
                            .with_context("surface", format!("{}x{}", image.width, image.height))
                            .with_context("frame", format!("{}x{}", live.width, live.height)));
                    }
                    self.tally.captured = Some(Captured {
                        camera: self.camera,
                        width: live.width,
                        height: live.height,
                        rgba: to_rgba(&image.format, image.texels)?,
                    });
                    true
                }
                Ok(None) => {
                    self.tally.unreadable = true;
                    true
                }
                Err(error) if error.recovery() == Recovery::Retry => false,
                Err(error) => return Err(error),
            }
        } else {
            match rhi.present(live.target) {
                Ok(()) => true,
                Err(error) if error.recovery() == Recovery::Retry => false,
                Err(error) => return Err(error),
            }
        };
        frame.record(FrameStage::Render, clock.elapsed())?;
        let report = frame.finish(begun.elapsed());

        let tally = &mut self.tally;
        tally.ticks += u64::from(plan.steps);
        tally.discarded = tally.discarded.saturating_add(plan.discarded);
        tally.classes[class_index(report.class())] += 1;
        tally.walls.push(report.wall());
        tally.unattributed += report.unattributed();
        tally.drawn = drawn;
        if intent.held > 0 {
            tally.active_frames += 1;
        }
        if !shown {
            // Not taking frames yet: ask again shortly, not in a spin.
            return Ok(Flow::Wait);
        }
        tally.frames += 1;

        let timed_out = match (self.timeout, tally.started) {
            (Some(timeout), Some(started)) => started.elapsed() >= timeout,
            _ => false,
        };
        tally.ending = if intent.exit {
            Some(Ending::ExitKey)
        } else if self.limit.is_some_and(|limit| tally.frames >= limit) {
            Some(Ending::FrameLimit)
        } else if timed_out {
            Some(Ending::Timeout)
        } else {
            None
        };
        if tally.ending.is_none() {
            return Ok(Flow::Continue);
        }
        let Some(live) = self.live.take() else {
            return Ok(Flow::Exit);
        };
        for chunk in live.chunks {
            chunk.destroy(rhi)?;
        }
        live.pass.destroy(rhi)?;
        rhi.destroy_texture(live.target)?;
        rhi.destroy_texture(live.depth)?;
        Ok(Flow::Exit)
    }
}

const fn class_index(class: BudgetClass) -> usize {
    match class {
        BudgetClass::Target => 0,
        BudgetClass::Warning => 1,
        BudgetClass::Critical => 2,
        BudgetClass::Emergency => 3,
    }
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
        other => Err(wrong("a surface format the client does not read")
            .with_context("format", other.to_string())),
    }
}

/// Render a report as the lines `nexora-client` prints.
#[must_use]
pub fn format_report(report: &ClientReport) -> String {
    let ms = |d: Duration| format!("{:.2} ms", d.as_secs_f64() * 1e3);
    let mut out = String::new();
    out.push_str("mode               client\n");
    out.push_str(&format!(
        "lifecycle          {} phases: {}\n",
        report.phases.len(),
        report.phases.join(" -> ")
    ));
    out.push_str(&format!(
        "modules            {}\n",
        report.modules.join(", ")
    ));
    out.push_str(&format!("adapter            {}\n", report.adapter));
    out.push_str(&format!(
        "window             {}x{} on {}\n",
        report.window.width, report.window.height, report.window.platform
    ));
    out.push_str(&format!(
        "world              {} columns drawn of {} generated, blocks {}..{} high\n",
        report.columns.0, report.columns.1, report.band.0, report.band.1
    ));
    match report.first_frame {
        Some(check) => out.push_str(&format!(
            "first frame        {} of {} pixels judged by the ray cast, {} matching, {} snapped edges, {} back faces, read back from the surface\n",
            check.judged, check.pixels, check.matching, check.snapped, check.backfacing
        )),
        None => out.push_str("first frame        not readable on this surface\n"),
    }
    out.push_str(&format!(
        "frames             {} shown, ended by {}\n",
        report.frames,
        report.ending.as_str()
    ));
    let [target, warning, critical, emergency] = report.classes;
    out.push_str(&format!(
        "frame loop         {} ticks, {} discarded; {target} target, {warning} warning, {critical} critical, {emergency} emergency (budget: doubling from the 50 ms step, not measured)\n",
        report.ticks, report.discarded
    ));
    out.push_str(&format!(
        "frame wall         median {}, p95 {}, max {}; {} unattributed in all\n",
        ms(report.wall[0]),
        ms(report.wall[1]),
        ms(report.wall[2]),
        ms(report.unattributed)
    ));
    out.push_str(&format!(
        "last frame         {} columns drawn, {} culled, {} empty, {} vertices\n",
        report.drawn.drawn, report.drawn.culled, report.drawn.empty, report.drawn.vertices
    ));
    out.push_str(&format!(
        "input              {} signals, {} keys without a HID usage, {} repeats dropped, focus gained {} times; actions held in {} frames\n",
        report.input.delivered, report.input.unnumbered, report.input.repeats, report.input.focused, report.active_frames
    ));
    out.push_str(&format!(
        "camera             moved {:.2} forward, {:.2} right, {:.2} up; turned {:.1} degrees\n",
        tidy(report.moved[0]),
        tidy(report.moved[1]),
        tidy(report.moved[2]),
        tidy(report.turned)
    ));
    match &report.flushed.1 {
        Some(path) => out.push_str(&format!(
            "world flush        {} bytes, written to {}\n",
            report.flushed.0,
            path.display()
        )),
        None => out.push_str(&format!(
            "world flush        {} bytes encoded, not written (no --save)\n",
            report.flushed.0
        )),
    }
    out.push_str("result             OK\n");
    out
}

/// A value printed to two decimals, without the minus sign of a rounding
/// residue (`-0.00`).
fn tidy(value: f64) -> f64 {
    if value.abs() < 0.005 {
        0.0
    } else {
        value
    }
}

// --- modules ----------------------------------------------------------------

/// The foundation, as in headless: proves the graph resolves and orders.
struct FoundationModule;

impl EngineModule for FoundationModule {
    fn id(&self) -> ModuleId {
        ModuleId::parse("nexora:module/foundation").expect("static module id")
    }
    fn initialize(&mut self, context: &ModuleContext) -> Result<()> {
        context.diagnostics().log(
            Level::Debug,
            Category::Core,
            "module/foundation",
            "foundation services ready",
        );
        Ok(())
    }
}

/// The world runtime, as in headless.
struct WorldModule;

impl EngineModule for WorldModule {
    fn id(&self) -> ModuleId {
        ModuleId::parse("nexora:module/world").expect("static module id")
    }
    fn dependencies(&self) -> Vec<ModuleDependency> {
        vec![ModuleDependency::required(
            ModuleId::parse("nexora:module/foundation").expect("static module id"),
        )]
    }
    fn initialize(&mut self, context: &ModuleContext) -> Result<()> {
        context.diagnostics().log(
            Level::Debug,
            Category::World,
            "module/world",
            "world runtime ready",
        );
        Ok(())
    }
}

/// The renderer: client-only, the module headless skips.
///
/// It initializes here because it exists (ADR-0030). The device it draws on
/// belongs to the window host and appears when the window opens (ADR-0027),
/// so the pass itself is created then, in [`Presentation`].
struct RendererModule;

impl EngineModule for RendererModule {
    fn id(&self) -> ModuleId {
        ModuleId::parse("nexora:module/renderer").expect("static module id")
    }
    fn dependencies(&self) -> Vec<ModuleDependency> {
        vec![ModuleDependency::required(
            ModuleId::parse("nexora:module/world").expect("static module id"),
        )]
    }
    fn sides(&self) -> ModuleSides {
        ModuleSides::Only(vec![RuntimeMode::Client, RuntimeMode::ListenServer])
    }
    fn initialize(&mut self, context: &ModuleContext) -> Result<()> {
        context.diagnostics().log(
            Level::Debug,
            Category::Render,
            "module/renderer",
            "renderer ready; the pass is created on the window's device",
        );
        Ok(())
    }
}

fn log(diagnostics: &Diagnostics, message: &'static str) {
    diagnostics.log(Level::Info, Category::Engine, "engine/client", message);
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Render, "client", message).with_recovery(Recovery::Manual)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_camera::Projection;
    use nexora_foundation::spatial::WorldPosition;

    #[test]
    fn displacement_is_in_the_starting_cameras_frame() {
        let projection = Projection::perspective(1.0, 0.1, 100.0).unwrap();
        let mut start = Camera::new(WorldPosition::new(0.0, 0.0, 0.0), projection).unwrap();
        start
            .set_orientation(std::f64::consts::FRAC_PI_2, 0.0)
            .unwrap(); // looks down -X
        let mut now = start;
        now.set_position(WorldPosition::new(-3.0, 1.0, -2.0))
            .unwrap();
        let [forward, right, up] = displacement(&start, &now);
        assert!((forward - 3.0).abs() < 1e-12, "{forward}");
        assert!((right - 2.0).abs() < 1e-12, "{right}"); // facing -X, right is -Z
        assert!((up - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_bgra_surface_is_read_as_rgba() {
        assert_eq!(
            to_rgba("Bgra8Unorm", vec![1, 2, 3, 4]).unwrap(),
            vec![3, 2, 1, 4]
        );
        assert!(to_rgba("Rgba16Float", vec![0; 8]).is_err());
    }

    /// Client mode must pass through presentation: the lifecycle refuses to
    /// shut down a client that never presented, so a client whose window never
    /// opened cannot report itself as having run.
    #[test]
    fn a_client_cannot_skip_presentation() {
        let mut lifecycle = Lifecycle::new(RuntimeMode::Client);
        lifecycle.advance_through(Phase::SimulationRunning).unwrap();
        assert_eq!(lifecycle.next_phase(), Some(Phase::PresentationRunning));
        assert!(lifecycle.advance_to(Phase::ShutdownRequested).is_err());
    }

    /// The client's module graph initializes the renderer, which headless
    /// skips: the same modules, a different side.
    #[test]
    fn the_renderer_initializes_in_client_mode_only() {
        for (mode, expected) in [(RuntimeMode::Client, 3), (RuntimeMode::Headless, 2)] {
            let mut modules = ModuleManager::new();
            modules.register(Box::new(FoundationModule)).unwrap();
            modules.register(Box::new(WorldModule)).unwrap();
            modules.register(Box::new(RendererModule)).unwrap();
            modules.resolve(mode).unwrap();
            modules
                .initialize_all(&ModuleContext::new(mode, Diagnostics::silent()))
                .unwrap();
            assert_eq!(
                modules.resolved_order().len(),
                expected,
                "{}",
                mode.as_str()
            );
        }
    }

    #[test]
    fn a_frame_holds_only_with_snapped_edges_rare_and_nothing_else_wrong() {
        let check = |judged, matching, backfacing, snapped| FrameCheck {
            pixels: 20_000,
            judged,
            matching,
            backfacing,
            snapped,
        };
        assert!(frame_holds(&check(10_000, 10_000, 0, 0)));
        assert!(frame_holds(&check(10_000, 9_998, 0, 2)));
        assert!(
            !frame_holds(&check(9_000, 9_000, 0, 0)),
            "too little judged"
        );
        assert!(
            !frame_holds(&check(10_000, 9_999, 0, 0)),
            "a wrong pixel of no class"
        );
        assert!(
            !frame_holds(&check(10_000, 9_999, 1, 0)),
            "a back face, which the pass culls"
        );
        assert!(
            !frame_holds(&check(10_000, 9_997, 0, 3)),
            "past one in 5,000"
        );
    }

    #[test]
    fn the_budget_classes_have_one_slot_each() {
        let slots: Vec<usize> = [
            BudgetClass::Target,
            BudgetClass::Warning,
            BudgetClass::Critical,
            BudgetClass::Emergency,
        ]
        .into_iter()
        .map(class_index)
        .collect();
        assert_eq!(slots, vec![0, 1, 2, 3]);
    }
}
