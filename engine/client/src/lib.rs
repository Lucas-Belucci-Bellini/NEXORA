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
//!    once into an [`Intent`]. The player is handed only its
//!    [`Intent::walk`]; exit and the held count stay with the frame loop.
//! 2. **Simulation**: the fixed steps the real time since the last frame
//!    buys ([`FrameSchedule`]): the world clock advances one tick a step.
//! 3. **Physics**: the player (ADR-0035) — a character body the simulation
//!    steers — runs one tick for each of those steps, against one view of
//!    the world built for the frame. Nothing edits the client's world, so a
//!    tick that had to push the body out of terrain is a defect, not a fix.
//! 4. **Render prep**: the camera is made from the player's eye, and never
//!    the other way round; the render origin follows it (ADR-0029), it is
//!    sampled, and the frame is recorded by the first render pass
//!    (ADR-0030).
//! 5. **Render**: one submission, and the frame presented to the window. The
//!    part of it the backend spent blocked handing the frame to the window
//!    system (`WgpuRhi::last_present_wait`) is declared as presentation's,
//!    so the budget classifies the frame's work, not the monitor's refresh.
//!
//! The time between frames is measured with the host's clock and handed in;
//! the loop itself still reads no clock (ADR-0017). This is the first process
//! that runs `runtime::frame` against a real clock (DEBT-0041). A step is one
//! world tick, and the client refuses to start if the calendar says
//! otherwise: the player's physics is paced by ticks, and a step that was not
//! one would walk it at the wrong speed in silence.
//!
//! The first frame shown — from the player's eye, before any step — is read
//! back from the surface and, after the window closes, checked against the
//! CPU ray cast (`nexora_render::reference`) over every drawn column: the
//! client does not claim to draw the world unless the pixels say so.
//!
//! Not built: streaming into the pass (the world is generated and meshed
//! before the first frame), textures, pointer look, interpolation between
//! ticks (DEBT-0049), the player as an entity or in the save (DEBT-0050),
//! audio. The frame budget is the arithmetic [`FrameBudget::doubling_from`]
//! the step, not a measured one.

pub mod scene;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use nexora_camera::{Camera, Projection, RenderOrigin};
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
use nexora_simulation::{
    Controls, Eye, Intent, PhysicsModule, Player, PlayerState, WorldSurfaces, WorldVoxels,
};
use nexora_window::input::InputCounts;
use nexora_window::{run, Client, Flow, WindowFacts, WindowSpec};
use nexora_world::persist;
use nexora_world::world::{World, WorldDescriptor};

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
    /// Frame work, the wall time less the wait on presentation (what the
    /// budget classifies): median, 95th percentile, maximum.
    pub work: [Duration; 3],
    /// Waits on presentation: median, 95th percentile, maximum.
    pub presentation_wait: [Duration; 3],
    /// Time inside frames no stage claimed, summed.
    pub unattributed: Duration,
    /// The last frame's pass statistics.
    pub drawn: FrameStats,
    /// Device events, as the host counted them.
    pub input: InputCounts,
    /// Frames in which at least one client action was held.
    pub active_frames: u64,
    /// Where the camera went: forward, right and up of where it started, in
    /// the starting camera's horizontal frame, and how far it turned. The
    /// camera is the player's eye, so this is the eye's path as drawn.
    pub moved: [f64; 3],
    /// Yaw turned, in degrees.
    pub turned: f64,
    /// What the player's body did, read from the body rather than the camera.
    pub player: PlayerSummary,
    /// The flushed world's size, and where it was written.
    pub flushed: (usize, Option<PathBuf>),
}

/// What the player's body did over a run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerSummary {
    /// Where the feet went: forward, right and up of where they started, in
    /// the spawn facing's horizontal frame.
    pub walked: [f64; 3],
    /// Whether the body ended the run standing on something.
    pub grounded: bool,
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
    // Physics runs inside the frame now (the player), so it is in the graph,
    // after the world it collides against.
    let mut modules = ModuleManager::new();
    modules.register(Box::new(FoundationModule))?;
    modules.register(Box::new(WorldModule))?;
    modules.register(Box::new(RendererModule))?;
    modules.register(Box::new(PhysicsModule))?;
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
    let ticks_per_second = world.clock().calendar().ticks_per_second();
    if ticks_per_second == 0 || STEP != Duration::from_secs(1) / ticks_per_second {
        return Err(Error::new(
            Domain::Time,
            "client",
            "the frame loop's step is not one world tick",
        )
        .with_recovery(Recovery::Manual)
        .with_context("step", format!("{STEP:?}"))
        .with_context("ticks_per_second", ticks_per_second.to_string()));
    }
    lifecycle.advance_to(Phase::WorldAttach)?;
    world.bring_online()?;
    lifecycle.advance_to(Phase::SimulationRunning)?;
    let scene = scene::build(&mut world, config.radius)?;
    log(diagnostics, "world generated and meshed");

    // --- the player ---------------------------------------------------------
    // On the first column of a run the scene found, at rest before the first
    // frame, so that frame shows - and is judged from - the eye it starts at.
    let player = Player::spawn(
        &WorldVoxels::new(&world),
        &scene.spawn,
        scene::SPAWN_PITCH,
        ticks_per_second,
    )?;
    let start = player.state();
    log(diagnostics, "player spawned and at rest");

    // --- presentation -------------------------------------------------------
    let start_camera = camera_at(player.eye(), scene.projection)?;
    let mut presentation = Presentation {
        world: &mut world,
        lifecycle,
        scene: &scene,
        controls: Controls::new()?,
        player,
        projection: scene.projection,
        camera: start_camera,
        origin: RenderOrigin::containing(start_camera.position())?,
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
        player,
        projection,
        tally,
        adapter,
        ..
    } = presentation;
    // The camera is derived from the eye and never written back; a run in
    // which they part ways drew something the player was not looking at.
    if camera != camera_at(player.eye(), projection)? {
        return Err(wrong("the camera is not the player's eye"));
    }
    let now = player.state();
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

    let spread = |mut samples: Vec<Duration>| {
        samples.sort_unstable();
        let percentile = |p: usize| {
            samples
                .get((samples.len().saturating_sub(1)) * p / 100)
                .copied()
                .unwrap_or_default()
        };
        [percentile(50), percentile(95), percentile(100)]
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
        wall: spread(tally.walls),
        work: spread(tally.works),
        presentation_wait: spread(tally.waits),
        unattributed: tally.unattributed,
        drawn: tally.drawn,
        input: session.input,
        active_frames: tally.active_frames,
        moved: displacement(&start_camera, &camera),
        turned: (camera.yaw() - start_camera.yaw()).to_degrees(),
        player: PlayerSummary {
            walked: walked(&start, &now),
            grounded: now.grounded,
        },
        flushed: (flushed, config.save.clone()),
    })
}

/// The camera at the player's eye: made from it every frame, never read
/// back (`CAMERA SYSTEM.md`: the camera owns no gameplay transform).
///
/// # Errors
///
/// The eye is not finite, or the projection is invalid.
fn camera_at(eye: Eye, projection: Projection) -> Result<Camera> {
    let mut camera = Camera::new(eye.position, projection)?;
    camera.set_orientation(eye.yaw, eye.pitch)?;
    Ok(camera)
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
    in_frame(start.yaw(), [b.x - a.x, b.y - a.y, b.z - a.z])
}

/// Where the feet went from `start` to `now`: forward, right and up in the
/// horizontal frame the player faced at `start`.
fn walked(start: &PlayerState, now: &PlayerState) -> [f64; 3] {
    let (a, b) = (start.feet, now.feet);
    in_frame(start.yaw, [b.x - a.x, b.y - a.y, b.z - a.z])
}

/// A world offset as forward, right and up for a facing of `yaw`: forward
/// is `(-sin, -cos)` and right `(cos, -sin)` in x and z.
fn in_frame(yaw: f64, d: [f64; 3]) -> [f64; 3] {
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
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
    works: Vec<Duration>,
    waits: Vec<Duration>,
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
    /// The player the keys steer, which owns where the view is.
    player: Player,
    projection: Projection,
    /// The player's eye as a camera: derived each frame, never written back.
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

        // Input: one sample; the player gets the walk and nothing else.
        let clock = Instant::now();
        let intent: Intent = self.controls.sample(&std::mem::take(&mut self.pending));
        let walk = intent.walk();
        frame.record(FrameStage::Input, clock.elapsed())?;

        // Simulation: the steps real time bought, each one world tick.
        let clock = Instant::now();
        for _ in 0..plan.steps {
            self.world.clock_mut().advance_by(TimeScale::Tick, 1)?;
        }
        frame.record(FrameStage::Simulation, clock.elapsed())?;

        // Physics: the player, one tick for each step, against one view of
        // the world built after the clock has moved (the clock needs the
        // world mutably; the view borrows it).
        if plan.steps > 0 {
            let clock = Instant::now();
            let terrain = WorldVoxels::new(&*self.world);
            for _ in 0..plan.steps {
                let outcome = self.player.tick(&terrain, walk)?;
                if outcome.depenetrated > 0 {
                    return Err(Error::new(
                        Domain::Physics,
                        "client",
                        "the player was pushed out of terrain that nothing edited",
                    )
                    .with_recovery(Recovery::Manual)
                    .with_context("substeps", outcome.depenetrated.to_string()));
                }
            }
            frame.record(FrameStage::Physics, clock.elapsed())?;
        }

        // Render prep: the camera is the player's eye, the origin follows
        // it, the pass is recorded.
        let clock = Instant::now();
        self.camera = camera_at(self.player.eye(), self.projection)?;
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
        // The part of render spent blocked until the window system took the
        // frame (a free surface image): not the engine's work (Finding 33).
        frame.waited_for_presentation(FrameStage::Render, rhi.last_present_wait())?;
        let report = frame.finish(begun.elapsed());

        let tally = &mut self.tally;
        tally.ticks += u64::from(plan.steps);
        tally.discarded = tally.discarded.saturating_add(plan.discarded);
        tally.classes[class_index(report.class())] += 1;
        tally.walls.push(report.wall());
        tally.works.push(report.work());
        tally.waits.push(report.presentation_wait());
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
        "frame work         median {}, p95 {}, max {}; presentation wait median {}, p95 {}, max {}\n",
        ms(report.work[0]),
        ms(report.work[1]),
        ms(report.work[2]),
        ms(report.presentation_wait[0]),
        ms(report.presentation_wait[1]),
        ms(report.presentation_wait[2])
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
    out.push_str(&format!(
        "player             walked {:.2} forward, {:.2} right, {:.2} up, {}\n",
        tidy(report.player.walked[0]),
        tidy(report.player.walked[1]),
        tidy(report.player.walked[2]),
        if report.player.grounded {
            "grounded"
        } else {
            "airborne"
        }
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
    use nexora_foundation::spatial::WorldPosition;
    use nexora_foundation::time::DEFAULT_TICKS_PER_SECOND;
    use nexora_runtime::input::{ButtonCode, DeviceId, DeviceKind, Signal};
    use nexora_simulation::player::{EYE_HEIGHT, MAX_PITCH};
    use nexora_simulation::{Walk, DEFAULT_KEYS};

    /// The default seed's world, its scene, and the player standing in it.
    fn the_default_scene() -> (World, Scene, Player) {
        let mut world = World::create(
            WorldDescriptor::new("nexora-client", ClientConfig::default().seed).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        world.bring_online().unwrap();
        let scene = scene::build(&mut world, ClientConfig::default().radius).unwrap();
        let player = Player::spawn(
            &WorldVoxels::new(&world),
            &scene.spawn,
            scene::SPAWN_PITCH,
            DEFAULT_TICKS_PER_SECOND,
        )
        .unwrap();
        (world, scene, player)
    }

    const KEYBOARD: DeviceId = DeviceId::new(DeviceKind::Keyboard, 0);

    /// The table is HID usages, the numbers the window host hands in: checked
    /// against the host's own translation, so the two cannot drift.
    #[test]
    fn the_default_keys_are_the_hosts_hid_usages() {
        use nexora_window::input::key_usage;
        use winit::keyboard::KeyCode as K;
        let keys = [
            K::KeyW,
            K::KeyS,
            K::KeyA,
            K::KeyD,
            K::Space,
            K::ArrowLeft,
            K::ArrowRight,
            K::ArrowUp,
            K::ArrowDown,
            K::Escape,
        ];
        assert_eq!(keys.len(), DEFAULT_KEYS.len());
        for ((path, usage), key) in DEFAULT_KEYS.iter().zip(keys) {
            assert_eq!(key_usage(key), Some(ButtonCode(*usage)), "{path}");
        }
    }

    /// A camera made from the eye must never clamp what the player decided,
    /// and a frame step must be one world tick, or the player walks at the
    /// wrong speed.
    #[test]
    fn the_pitch_limits_agree_and_the_step_is_one_tick() {
        assert_eq!(MAX_PITCH.to_bits(), nexora_camera::MAX_PITCH.to_bits());
        assert_eq!(STEP, Duration::from_secs(1) / DEFAULT_TICKS_PER_SECOND);
    }

    /// The eye is 0.18 below the top of the body's box and 0.3 from any wall
    /// it touches; the near plane's corner is closer than both, so nothing
    /// the body can touch is clipped away.
    #[test]
    fn the_eye_clears_the_near_plane() {
        let near: f64 = 0.1;
        let half_height = near * (35f64.to_radians()).tan();
        let half_width = half_height * 384.0 / 256.0;
        let corner = (near * near + half_height * half_height + half_width * half_width).sqrt();
        assert!((corner - 0.161).abs() < 1e-3, "{corner}");
        assert!(
            1.8 - EYE_HEIGHT > corner,
            "{} above the eye",
            1.8 - EYE_HEIGHT
        );
        assert!(0.3 > corner);
    }

    /// What CI's `camera moved [1-9]... forward` grep rests on, proved without
    /// a display: one second of W, sampled through the controls the window
    /// feeds, walks the default seed's player 3.8 to 4.3 blocks forward, and
    /// the camera made from its eye says the same.
    #[test]
    fn holding_w_for_a_second_walks_the_default_seed_forward() {
        let (world, scene, mut player) = the_default_scene();
        let terrain = WorldVoxels::new(&world);
        let mut controls = Controls::new().unwrap();
        let start = player.state();
        let start_camera = camera_at(player.eye(), scene.projection).unwrap();
        for tick in 0..DEFAULT_TICKS_PER_SECOND {
            let frame = if tick == 0 {
                InputFrame::new()
                    .with(Signal::Attached(KEYBOARD))
                    .with(Signal::button(KEYBOARD, ButtonCode(0x1A), true))
            } else {
                InputFrame::new()
            };
            let walk = controls.sample(&frame).walk();
            assert_eq!(walk.forward, 1.0, "tick {tick}: W is held");
            player.tick(&terrain, walk).unwrap();
        }
        let [forward, right, _] = walked(&start, &player.state());
        assert!((3.8..=4.3).contains(&forward), "walked {forward}");
        assert!(right.abs() < 1e-6, "drifted {right}");
        let camera = camera_at(player.eye(), scene.projection).unwrap();
        let seen = displacement(&start_camera, &camera)[0];
        assert!((seen - forward).abs() < 1e-12, "{seen} against {forward}");
    }

    #[test]
    fn the_camera_is_the_players_eye() {
        let (world, scene, mut player) = the_default_scene();
        let terrain = WorldVoxels::new(&world);
        let script = [
            (
                10,
                Walk {
                    forward: 1.0,
                    ..Walk::default()
                },
            ),
            (
                10,
                Walk {
                    forward: 1.0,
                    turn: 1.0,
                    ..Walk::default()
                },
            ),
            (
                5,
                Walk {
                    look: 1.0,
                    turn: -1.0,
                    ..Walk::default()
                },
            ),
        ];
        for (ticks, walk) in script {
            for _ in 0..ticks {
                player.tick(&terrain, walk).unwrap();
                let eye = player.eye();
                let camera = camera_at(eye, scene.projection).unwrap();
                let (at, from) = (camera.position(), eye.position);
                assert_eq!(
                    [at.x.to_bits(), at.y.to_bits(), at.z.to_bits()],
                    [from.x.to_bits(), from.y.to_bits(), from.z.to_bits()]
                );
                assert_eq!(camera.pitch().to_bits(), eye.pitch.to_bits());
                assert!((camera.yaw() - eye.yaw).abs() < 1e-12);
                // Made from its own state, the camera is the camera again:
                // nothing is lost or added on the way through.
                let again = camera_at(
                    Eye {
                        position: camera.position(),
                        yaw: camera.yaw(),
                        pitch: camera.pitch(),
                    },
                    scene.projection,
                )
                .unwrap();
                assert_eq!(again, camera);
            }
        }
    }

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
    /// skips: the same modules, a different side. Physics is in both, after
    /// the world, and sorts before the renderer — which CI's
    /// `modules .*nexora:module/renderer$` grep relies on.
    #[test]
    fn the_renderer_initializes_in_client_mode_only() {
        for (mode, expected) in [(RuntimeMode::Client, 4), (RuntimeMode::Headless, 3)] {
            let mut modules = ModuleManager::new();
            modules.register(Box::new(FoundationModule)).unwrap();
            modules.register(Box::new(WorldModule)).unwrap();
            modules.register(Box::new(RendererModule)).unwrap();
            modules.register(Box::new(PhysicsModule)).unwrap();
            modules.resolve(mode).unwrap();
            modules
                .initialize_all(&ModuleContext::new(mode, Diagnostics::silent()))
                .unwrap();
            let order: Vec<String> = modules
                .resolved_order()
                .iter()
                .map(ToString::to_string)
                .collect();
            assert_eq!(order.len(), expected, "{}", mode.as_str());
            let physics = order.iter().position(|id| id == "nexora:module/physics");
            assert!(physics.is_some(), "{order:?}");
            if mode == RuntimeMode::Client {
                assert_eq!(
                    order.last().map(String::as_str),
                    Some("nexora:module/renderer")
                );
                assert!(physics < order.iter().position(|id| id == "nexora:module/renderer"));
            }
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
