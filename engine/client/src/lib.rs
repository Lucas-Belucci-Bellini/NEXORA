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
//!    On a frame that ticks, the block the player asked to mine or build
//!    ([`Intent::block`]) becomes a command, and the command pipeline decides
//!    (ADR-0036): the player's own [`BlockEditor`] casts the ray from the eye,
//!    and only a handler writes the world.
//! 3. **Physics**: the player (ADR-0035) — a character body the simulation
//!    steers — runs one tick for each of those steps, against one view of
//!    the world built for the frame, after the frame's edits. The authority
//!    refuses a block where the body is, so a tick that had to push the body
//!    out of terrain is still a defect, not a fix.
//! 4. **Render prep**: the regions an edit changed are meshed again, and
//!    they and the neighbours whose shared corners moved uploaded again in
//!    the frame's own list ([`Scene::remesh`], [`Scene::reupload`]); the camera is made from the
//!    player's eye, and never the other way round; the render origin follows
//!    it (ADR-0029), it is sampled, and the frame is recorded by the first
//!    render pass (ADR-0030).
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
//! back from the surface and checked against the CPU ray cast
//! (`nexora_render::reference`) over every drawn column: the client does not
//! claim to draw the world unless the pixels say so. So is the first frame
//! shown after the player first changed the world, against the ray cast of
//! the changed world: the client does not claim an edit is drawn unless the
//! pixels say so either. Each is judged the moment it is read back, against
//! the world it showed, and the time spent judging is not frame time.
//!
//! Not built: streaming into the pass (the world is generated and meshed
//! before the first frame), textures, pointer look and a crosshair (the
//! player aims with the arrow keys at the centre of the frame), interpolation
//! between ticks (DEBT-0049), the player as an entity or in the save
//! (DEBT-0050), audio. The frame budget is the arithmetic
//! [`FrameBudget::doubling_from`] the step, not a measured one.

pub mod residency;
pub mod scene;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};

use nexora_asset::texture::MapRole;
use nexora_camera::{Camera, Projection, RenderOrigin};
use nexora_foundation::diagnostics::{Category, Diagnostics, Level, StderrSink};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::ChunkCoord;
use nexora_foundation::time::{CalendarConfig, TimeScale};
use nexora_image::TextureLoader;
use nexora_mesh::SurfaceId;
use nexora_persistence::container::SaveContainer;
use nexora_render::atlas::{Atlas, TILE};
use nexora_render::reference::{check_frame_shaded, FrameCheck, Shading};
use nexora_render::{ChunkPass, FrameStats, GpuChunk};
use nexora_resource::ResourceManager;
use nexora_rhi::{CommandList, Rhi, TextureDesc, TextureFormat, TextureHandle, Usage};
use nexora_rhi_wgpu::WgpuRhi;
use nexora_runtime::frame::{BudgetClass, FrameBudget, FrameLoop, FrameSchedule, FrameStage};
use nexora_runtime::input::InputFrame;
use nexora_runtime::lifecycle::{Lifecycle, Phase, RuntimeMode};
use nexora_runtime::module::{
    EngineModule, ModuleContext, ModuleDependency, ModuleId, ModuleManager, ModuleSides,
};
use nexora_simulation::{
    load_player, save_player, BlockContent, BlockEditor, BlockIntent, Controls, EditTally, Eye,
    Intent, PhysicsModule, Player, PlayerState, SurfaceTable, Walk, WorldVoxels,
};
use nexora_window::input::InputCounts;
use nexora_window::{run, Client, Flow, WindowFacts, WindowSpec};
use nexora_world::persist;
use nexora_world::world::{World, WorldDescriptor};

use crate::residency::{ClientResidency, ResidencyTally};
use crate::scene::{Scene, Shift};

/// The window's title.
pub const TITLE: &str = "NEXORA";

/// One fixed step: a world tick, 20 Hz.
pub const STEP: Duration = Duration::from_millis(50);

/// Steps one frame may run before the rest are dropped (ADR-0017).
pub const MAX_STEPS: u32 = 4;

/// The block the player builds (ADR-0036).
///
/// One block, the generator's own stone: choosing what to build is the
/// inventory's (PLAYER-23, Phase 5), and there is no inventory yet.
pub const BUILT_BLOCK: &str = "nexora:block/stone";

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
    /// The world file: resumed from — the world and the player — when it
    /// exists, created otherwise, and flushed back to at shutdown
    /// (ADR-0036). Not given with [`ClientConfig::save`]: one place says
    /// where the world goes.
    pub world: Option<PathBuf>,
    /// A block content document (`BlockContent`): its blocks are registered
    /// and its surfaces drawn, the engine's own blocks among them when it
    /// restyles them (ADR-0037).
    pub content: Option<PathBuf>,
    /// A resource root (with its `resources.json`) holding the content's
    /// albedo maps. With it the world is drawn textured; without it, in its
    /// faces' colours. Requires `content`.
    pub resources: Option<PathBuf>,
    /// Where the frames read back are written, as binary PPM: the first
    /// frame at this path, the first after an edit beside it with `-edited`
    /// before the extension. `None` writes nothing.
    pub capture: Option<PathBuf>,
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
            world: None,
            content: None,
            resources: None,
            capture: None,
            verbose: false,
        }
    }
}

/// How a run's world and player began.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Start {
    /// A new world was generated from the seed, and the player spawned by
    /// the search.
    Created {
        /// The seed.
        seed: u64,
    },
    /// The world was read from its file and goes on from where it stopped.
    Resumed {
        /// The file.
        path: PathBuf,
        /// The world tick it was saved at.
        tick: u64,
        /// Where the player came from.
        player: Placed,
    },
}

/// Where a resumed world's player came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placed {
    /// From the save, exactly as it was (`Player::resume`).
    Saved,
    /// From the spawn search, because the save holds no player.
    NoneSaved,
    /// From the spawn search, because the save's player would be inside
    /// terrain, or is not a state a player can be in.
    Refused,
}

impl Placed {
    /// A stable description.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Saved => "the player where it was saved",
            Self::NoneSaved => "no player saved, spawned by search",
            Self::Refused => "the saved player was refused, spawned by search",
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
    /// How the world and the player began.
    pub start: Start,
    /// What the frames drew with: tiles in the atlas and its size, or `None`
    /// when faces were coloured by direction.
    pub textures: Option<(u32, (u32, u32))>,
    /// Blocks drawn with no surface of their own (`SurfaceTable::unmapped`).
    pub unmapped: usize,
    /// Columns drawn, and resident at the end (drawn, a ring, and what the
    /// hysteresis has not let go of yet).
    pub columns: (usize, u32),
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
    /// What the player's block edits came to (ADR-0036).
    pub edits: EditTally,
    /// The first frame shown after the world first changed, against the ray
    /// cast of the changed world; `None` when nothing changed it, or when the
    /// surface cannot be read back.
    pub edited_frame: Option<FrameCheck>,
    /// Regions meshed again after edits, and regions uploaded again, summed.
    pub remeshed: (u64, u64),
    /// The slowest frame's meshing and recording of uploads after its edits:
    /// what an edit costs the frame it lands in, before the GPU sees it.
    pub slowest_remesh: Duration,
    /// How the drawn square followed the player (ADR-0038).
    pub streaming: StreamingSummary,
    /// The first frame shown after the drawn square first moved, against the
    /// ray cast of the square it moved to; `None` when it never moved, or
    /// when the surface cannot be read back.
    pub moved_frame: Option<FrameCheck>,
    /// The flushed world's size, and where it was written.
    pub flushed: (usize, Option<PathBuf>),
}

/// How the drawn square followed the player over a run (ADR-0038).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StreamingSummary {
    /// Times the square moved.
    pub moves: u64,
    /// Columns meshed as they entered it.
    pub meshed: u64,
    /// Regions uploaded because of a move: the columns that entered, and the
    /// kept ones whose neighbours changed.
    pub uploaded: u64,
    /// The slowest move: residency, meshing and recording the uploads.
    pub slowest: Duration,
    /// What residency did meanwhile.
    pub residency: ResidencyTally,
    /// Edited columns held out of the world at the end, before the flush
    /// put them back for the save.
    pub retained: usize,
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

    // --- world: resumed from its file, or created --------------------------
    if config.world.is_some() && config.save.is_some() {
        return Err(Error::new(
            Domain::Save,
            "client",
            "a world file and a save path both say where the world goes",
        )
        .with_recovery(Recovery::Reject));
    }
    let resumed = match config.world.as_deref().filter(|path| path.exists()) {
        Some(path) => Some((path.to_path_buf(), SaveContainer::read(path)?)),
        None => None,
    };
    let content = config
        .content
        .as_deref()
        .map(BlockContent::load)
        .transpose()?;
    let definitions = content
        .as_ref()
        .map(BlockContent::block_definitions)
        .unwrap_or_default();
    let mut world = match &resumed {
        Some((_, container)) => persist::load_with(container, &definitions)?,
        None => World::create_with(
            WorldDescriptor::new("nexora-client", config.seed)?,
            CalendarConfig::earthlike(),
            &definitions,
        )?,
    };
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
    let (table, atlas) = looks(&world, content.as_ref(), config.resources.as_deref())?;
    let unmapped = table.unmapped().len();

    // A resumed world's player goes on from where it was saved, so the drawn
    // square is centred on its column (ADR-0038); a new world's on the
    // origin's.
    let saved = match &resumed {
        Some((_, container)) => load_player(container)?,
        None => None,
    };
    scene::check_radius(config.radius)?;
    let centre = saved.as_ref().map_or(ChunkCoord::new(0, 0), |state| {
        scene::column_in(&world, state.feet.x, state.feet.z)
    });
    let mut residency = ClientResidency::new(config.radius)?;
    residency.settle(&mut world, centre)?;
    let mut scene = scene::build(&world, centre, config.radius, table, atlas)?;
    log(diagnostics, "world generated and meshed");

    // --- the player ---------------------------------------------------------
    // A resumed world's player goes on from where it was saved, if it can;
    // otherwise on the first column of a run the scene finds, at rest before
    // the first frame, so that frame shows - and is judged from - the eye it
    // starts at.
    let (player, placed) = {
        let terrain = WorldVoxels::new(&world);
        let spawn = || -> Result<Player> {
            let run = scene.spawn_run(&world)?;
            Player::spawn(&terrain, &run, scene::SPAWN_PITCH, ticks_per_second)
        };
        match saved {
            None => (spawn()?, Placed::NoneSaved),
            Some(state) => match Player::resume(&terrain, &state, ticks_per_second) {
                Ok(player) => (player, Placed::Saved),
                Err(error) if error.recovery() == Recovery::Reject => (spawn()?, Placed::Refused),
                Err(error) => return Err(error),
            },
        }
    };
    let start = player.state();
    let begun = match resumed {
        Some((path, _)) => Start::Resumed {
            path,
            tick: world.clock().now().ticks(),
            player: placed,
        },
        None => Start::Created { seed: config.seed },
    };
    log(diagnostics, "player placed");

    // --- the player's authority for edits (ADR-0036) ------------------------
    // Shared, because the command handlers are the ones that write it.
    let built = world.block_id(&Identifier::parse(BUILT_BLOCK)?)?;
    let world = Rc::new(RefCell::new(world));
    let editor = BlockEditor::new(Rc::clone(&world), built, 1)?.within(scene.edit_area());

    // --- presentation -------------------------------------------------------
    let start_camera = camera_at(player.eye(), scene.projection)?;
    let projection = scene.projection;
    let mut presentation = Presentation {
        world: Rc::clone(&world),
        lifecycle,
        scene: &mut scene,
        controls: Controls::new()?,
        editor,
        residency,
        block_latched: BlockIntent::default(),
        capture: config.capture.clone(),
        player,
        projection,
        camera: start_camera,
        origin: RenderOrigin::containing(start_camera.position())?,
        frame_loop: FrameLoop::new(
            FrameSchedule::new(STEP, MAX_STEPS)?,
            FrameBudget::doubling_from(STEP),
        ),
        pending: InputFrame::new(),
        jump_latched: false,
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
        editor,
        mut residency,
        ..
    } = presentation;
    let edits = editor.tally();
    // The handlers hold the world; they go before it is flushed.
    drop(editor);
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

    // --- what was shown: judged as it was read back, in the frame loop ------
    let first_frame = match tally.first_frame {
        Some(check) => Some(check),
        None if tally.unreadable => None,
        None => return Err(wrong("no frame reached the window")),
    };
    if tally.show_edit && !tally.unreadable {
        return Err(wrong(
            "the world was edited and no frame showing it was read back",
        ));
    }
    if tally.show_move && !tally.unreadable {
        return Err(wrong(
            "the drawn square moved and no frame showing it was read back",
        ));
    }

    // --- shutdown -----------------------------------------------------------
    lifecycle.advance_to(Phase::ShutdownRequested)?;
    lifecycle.advance_to(Phase::SimulationStop)?;
    // Evicted edited columns go back into the world first: the save writes
    // the world, and they are not in it (`RetainedChunks::flush_into`).
    let retained = residency.retained();
    let resident = residency.resident();
    residency.flush_into(&mut world.borrow_mut())?;
    let mut container = persist::save(&world.borrow())?;
    save_player(&mut container, &player)?;
    let flushed = container.encode().len();
    let target = config.world.as_ref().or(config.save.as_ref());
    if let Some(path) = target {
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
        start: begun,
        textures: scene
            .atlas
            .as_ref()
            .map(|atlas| (atlas.tiles(), atlas.size())),
        unmapped,
        columns: (scene.regions.len(), resident),
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
        edits,
        edited_frame: tally.edited_frame,
        remeshed: (tally.remeshed, tally.uploaded),
        slowest_remesh: tally.slowest_remesh,
        streaming: StreamingSummary {
            moves: tally.moves,
            meshed: tally.meshed_in,
            uploaded: tally.uploaded_in,
            slowest: tally.slowest_move,
            residency: residency.tally(),
            retained,
        },
        moved_frame: tally.moved_frame,
        flushed: (flushed, target.cloned()),
    })
}

/// The device chunks after the drawn square moved: a region kept with the
/// vertices it had keeps its chunk, every other region is uploaded in
/// `list`, and the chunks of regions that left — or were uploaded again —
/// are released once the last frame that drew them is done.
fn restream(
    rhi: &mut WgpuRhi,
    pass: &ChunkPass,
    list: &mut CommandList,
    old: Vec<GpuChunk>,
    scene: &Scene,
    shift: &Shift,
) -> Result<Vec<GpuChunk>> {
    let mut old: Vec<Option<GpuChunk>> = old.into_iter().map(Some).collect();
    let mut chunks = Vec::with_capacity(scene.regions.len());
    for (index, from) in shift.from.iter().enumerate() {
        let kept = match from {
            Some(before) if shift.reupload.binary_search(&index).is_err() => {
                old.get_mut(*before).and_then(Option::take)
            }
            _ => None,
        };
        match kept {
            Some(chunk) => chunks.push(chunk),
            None => {
                let bytes = scene.vertices(index)?;
                chunks.push(pass.upload_vertices(rhi, list, bytes, scene.regions[index].0)?);
            }
        }
    }
    for chunk in old.into_iter().flatten() {
        chunk.destroy(rhi)?;
    }
    Ok(chunks)
}

/// The walk a frame's ticks run with.
///
/// Input is sampled every frame and the player ticks only on frames that buy
/// a step, and at ~100 Hz against a 20 Hz tick most frames buy none. A jump
/// held on one of those frames — a tap is held for exactly one frame
/// (ADR-0031) — would never reach the player. So a jump is latched until a
/// frame with steps spends it. Walking, turning and looking are held states
/// and need no latch: the next frame with steps reads them again.
fn walk_for_ticks(latched: &mut bool, mut walk: Walk, steps: u32) -> Walk {
    *latched |= walk.jump;
    if steps > 0 {
        walk.jump = std::mem::take(latched);
    }
    walk
}

/// The block edits a frame's ticks run with.
///
/// The same reason as [`walk_for_ticks`]: a click is held for exactly one
/// frame, and most frames buy no tick. An edit is a command at a world tick,
/// so one asked for on a frame without a tick is latched until a frame with
/// one spends it — once, however many ticks that frame runs — and a frame
/// without a tick edits nothing.
fn block_for_ticks(latched: &mut BlockIntent, block: BlockIntent, steps: u32) -> BlockIntent {
    *latched = latched.or(block);
    if steps > 0 {
        std::mem::take(latched)
    } else {
        BlockIntent::default()
    }
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

/// At most one judged pixel in this many may be explained by a texel
/// boundary within `TEXEL_SNAP` of its centre (ADR-0037), and nothing else.
///
/// A bound of its own, because a textured frame has a texel boundary every
/// sixteenth of a block and how many fall that close to a centre depends on
/// the device's interpolation. Measured: lavapipe at worst 1 in 17,800 over
/// forty seeds at 768×512; WARP (Windows' software adapter) 1 in 4,640 at
/// 384×256, which the geometry bound above refused. Nine times WARP's rate,
/// and far below what a broken texel rule gives (a texel off by one: one
/// judged pixel in fourteen explained this way, and a sixth plainly wrong).
pub const TEXEL_SNAPPED_PER_JUDGED: usize = 500;

/// Whether a checked frame is the one the ray cast says: at least half the
/// pixels judged; every judged pixel matching except snapped edges, within
/// [`SNAPPED_PER_JUDGED`], and snapped texels, within
/// [`TEXEL_SNAPPED_PER_JUDGED`]; and **no back face at all**, which the pass
/// culls, so one on screen means culling or winding broke.
#[must_use]
pub fn frame_holds(check: &FrameCheck) -> bool {
    check.judged * 2 >= check.pixels
        && check.backfacing == 0
        && check.matching + check.snapped + check.texel_snapped == check.judged
        && check.snapped * SNAPPED_PER_JUDGED <= check.judged
        && check.texel_snapped * TEXEL_SNAPPED_PER_JUDGED <= check.judged
}

/// The decoded-texture budget the atlas is built under: 36 first-generation
/// albedos are 36 KiB of pixels; this holds a few hundred more and nothing
/// that is not a tile.
const TEXTURE_BUDGET: u64 = 1024 * 1024;

/// What the world looks like: the surface table, and the atlas of its
/// albedos when there are resources to read them from (ADR-0037).
///
/// Every material of the content is resolved by identifier through the
/// resource index, verified and decoded by the engine's own decoder
/// (ADR-0021, ADR-0022), held to 16×16, and given the tile its surface — the
/// material's runtime id — names.
///
/// # Errors
///
/// Resources without content; a surface the world does not have; a map the
/// index does not provide, that does not verify or decode, or that is not
/// one opaque 16×16 tile.
fn looks(
    world: &World,
    content: Option<&BlockContent>,
    resources: Option<&std::path::Path>,
) -> Result<(SurfaceTable, Option<Atlas>)> {
    let Some(content) = content else {
        if resources.is_some() {
            return Err(Error::new(
                Domain::Content,
                "client",
                "a resource root was given without content to look up",
            )
            .with_recovery(Recovery::Reject));
        }
        return Ok((SurfaceTable::untextured(world), None));
    };
    let materials = content.material_registry()?;
    let table = content.surface_table(world, &materials)?;
    let Some(root) = resources else {
        return Ok((table, None));
    };
    let mut resources = ResourceManager::open(root, TEXTURE_BUDGET)?;
    // Before any load: every map the content needs, named at once.
    resources.manifest().provides(content.materials())?;
    let loader = TextureLoader::new(MapRole::Albedo).at_most(TILE);
    let mut atlas = Atlas::new();
    for material in content.materials() {
        let id = material.map_asset_id(MapRole::Albedo)?;
        let handle = resources.resolve(&id, &loader)?;
        let map = resources.load(&handle, &loader)?;
        let surface = materials.runtime_id_of(material.id()).ok_or_else(|| {
            Error::new(Domain::Content, "client", "a material has no runtime id")
                .with_recovery(Recovery::Manual)
                .with_context("material", material.id().to_string())
        })?;
        atlas
            .add(SurfaceId(surface.0), &nexora_image::rgba8(&map)?)
            .map_err(|error| error.with_context("texture", id.to_string()))?;
    }
    Ok((table, Some(atlas)))
}

/// Judge a frame read back from the surface against the ray cast of the
/// world it showed, over `bounds`.
///
/// # Errors
///
/// The ray cast could not run, or the frame is not what it says
/// ([`frame_holds`]); `which` names the frame in the error.
fn judge(
    world: &World,
    scene: &Scene,
    captured: &Captured,
    which: &'static str,
) -> Result<FrameCheck> {
    let view = scene.view(world);
    let shading = match &scene.atlas {
        Some(atlas) => Shading::Albedo {
            atlas,
            encoded: captured.encoded,
        },
        None => Shading::Faces,
    };
    let check = check_frame_shaded(
        &view,
        scene.bounds,
        &captured.camera,
        (captured.width, captured.height),
        &captured.rgba,
        shading,
    )?;
    if !frame_holds(&check) {
        return Err(
            wrong("the window did not show what a ray cast says it must")
                .with_context("frame", which)
                .with_context("judged", check.judged.to_string())
                .with_context("matching", check.matching.to_string())
                .with_context("backfacing", check.backfacing.to_string())
                .with_context("snapped", check.snapped.to_string())
                .with_context("texel_snapped", check.texel_snapped.to_string())
                .with_context("pixels", check.pixels.to_string()),
        );
    }
    Ok(check)
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

/// Write a read-back frame as a binary PPM (`P6`): the simplest image a
/// viewer opens, and written before it is judged, so a frame that fails is
/// on disk to be looked at.
///
/// # Errors
///
/// The file could not be written.
fn write_ppm(path: &std::path::Path, captured: &Captured) -> Result<()> {
    let mut bytes = format!("P6\n{} {}\n255\n", captured.width, captured.height).into_bytes();
    for texel in captured.rgba.chunks_exact(4) {
        bytes.extend_from_slice(&texel[..3]);
    }
    std::fs::write(path, bytes).map_err(|cause| {
        Error::new(
            Domain::Render,
            "client",
            "a captured frame could not be written",
        )
        .with_recovery(Recovery::Manual)
        .with_context("path", path.display().to_string())
        .with_context("cause", cause.to_string())
    })
}

/// The first shown frame, as read back from the surface.
struct Captured {
    /// Whether the surface kept the target's sRGB encoding.
    encoded: bool,
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
    /// The first frame shown, judged.
    first_frame: Option<FrameCheck>,
    /// The first frame shown after the world first changed, judged.
    edited_frame: Option<FrameCheck>,
    /// The world has changed and no frame showing it has been read back.
    show_edit: bool,
    /// Regions meshed again, and uploaded again.
    remeshed: u64,
    uploaded: u64,
    slowest_remesh: Duration,
    /// The first frame shown after the drawn square first moved, judged.
    moved_frame: Option<FrameCheck>,
    /// The square has moved and no frame showing it has been read back.
    show_move: bool,
    /// Moves of the square, columns meshed into it, regions uploaded for it.
    moves: u64,
    meshed_in: u64,
    uploaded_in: u64,
    slowest_move: Duration,
    unreadable: bool,
    ending: Option<Ending>,
    started: Option<Instant>,
    last: Option<Instant>,
}

/// What exists on the device while the window is open.
struct Live {
    pass: ChunkPass,
    /// The albedo atlas the textured pass reads.
    atlas: Option<TextureHandle>,
    chunks: Vec<GpuChunk>,
    target: TextureHandle,
    depth: TextureHandle,
    width: u32,
    height: u32,
}

/// The client's side of the window host: the frame loop, living in a window.
struct Presentation<'a> {
    world: Rc<RefCell<World>>,
    lifecycle: &'a mut Lifecycle,
    scene: &'a mut Scene,
    controls: Controls,
    /// The authority the player's block edits go through (ADR-0036).
    editor: BlockEditor,
    /// What of the world is resident, following the player (ADR-0038).
    residency: ClientResidency,
    /// An edit asked for on a frame that bought no tick, waiting for one
    /// that does ([`block_for_ticks`]).
    block_latched: BlockIntent,
    /// Where judged frames are written ([`ClientConfig::capture`]).
    capture: Option<PathBuf>,
    /// The player the keys steer, which owns where the view is.
    player: Player,
    projection: Projection,
    /// The player's eye as a camera: derived each frame, never written back.
    camera: Camera,
    origin: RenderOrigin,
    frame_loop: FrameLoop,
    pending: InputFrame,
    /// A jump asked for on a frame that bought no tick, waiting for one
    /// that does ([`walk_for_ticks`]).
    jump_latched: bool,
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
        // Textured, the frame is sRGB: the albedo is, and light is added in
        // linear space between the decode and the encode (ADR-0037).
        let format = if self.scene.atlas.is_some() {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        };
        let target = rhi.create_texture(&texture("client frame", format, Usage::RENDER_TARGET))?;
        let depth = rhi.create_texture(&texture(
            "client depth",
            TextureFormat::Depth32Float,
            Usage::RENDER_TARGET,
        ))?;
        let mut upload = CommandList::new("client upload");
        let (pass, atlas) = match &self.scene.atlas {
            Some(atlas) => {
                let texture = atlas.upload(rhi, &mut upload)?;
                (ChunkPass::textured(rhi, format, texture)?, Some(texture))
            }
            None => (ChunkPass::new(rhi, format)?, None),
        };
        // Each column split at its neighbours' corners too, so the seams
        // between columns are split (DEBT-0047): the same path an edit's
        // upload takes.
        let mut chunks = Vec::with_capacity(self.scene.regions.len());
        for (index, (region, _)) in self.scene.regions.iter().enumerate() {
            let bytes = self.scene.vertices(index)?;
            chunks.push(pass.upload_vertices(rhi, &mut upload, bytes, *region)?);
        }
        let fence = rhi.submit(upload)?;
        rhi.wait(fence)?;
        self.live = Some(Live {
            pass,
            atlas,
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
        if self.live.is_none() {
            return Err(wrong("a frame before the window opened"));
        }
        let begun = Instant::now();
        let elapsed = self.tally.last.map_or(Duration::ZERO, |last| begun - last);
        self.tally.last = Some(begun);
        let mut frame = self.frame_loop.begin(elapsed);
        let plan = frame.plan();

        // Input: one sample; the player gets the walk, the editor the block
        // intent, and nothing else crosses.
        let clock = Instant::now();
        let intent: Intent = self.controls.sample(&std::mem::take(&mut self.pending));
        let walk = walk_for_ticks(&mut self.jump_latched, intent.walk(), plan.steps);
        let block = block_for_ticks(&mut self.block_latched, intent.block(), plan.steps);
        frame.record(FrameStage::Input, clock.elapsed())?;

        // Simulation: the steps real time bought, each one world tick; then
        // the edits the player asked for, as commands at the tick reached.
        let clock = Instant::now();
        {
            let mut world = self.world.borrow_mut();
            for _ in 0..plan.steps {
                world.clock_mut().advance_by(TimeScale::Tick, 1)?;
            }
        }
        let mut changed = Vec::new();
        if block.any() {
            let now = self.world.borrow().clock().now();
            for edit in self.editor.apply(&self.player, block, now)? {
                if let (true, Some(cell)) = (edit.changed(), edit.cell) {
                    changed.push(cell);
                }
            }
        }
        frame.record(FrameStage::Simulation, clock.elapsed())?;

        // World: residency follows the player, and when its column is no
        // longer the square's centre the square moves to it (ADR-0038). The
        // player's column is the one it ended the last frame in: a column is
        // wider than a frame's walk, and its neighbours are resident.
        let feet = self.player.state().feet;
        let column = self.scene.column_of(feet.x, feet.z);
        let mut shift = None;
        if column != self.scene.centre {
            let clock = Instant::now();
            self.residency
                .settle(&mut self.world.borrow_mut(), column)?;
            let moved = self.scene.recentre(&self.world.borrow(), column)?;
            self.editor.set_area(self.scene.edit_area());
            let spent = clock.elapsed();
            frame.record(FrameStage::World, spent)?;
            shift = Some((moved, spent));
        }

        // Physics: the player, one tick for each step, against one view of
        // the world built after the clock has moved and the edits landed.
        if plan.steps > 0 {
            let clock = Instant::now();
            let world = self.world.borrow();
            let terrain = WorldVoxels::new(&world);
            for _ in 0..plan.steps {
                let outcome = self.player.tick(&terrain, walk)?;
                if outcome.depenetrated > 0 {
                    return Err(Error::new(
                        Domain::Physics,
                        "client",
                        "the player was pushed out of terrain; no edit may bury it",
                    )
                    .with_recovery(Recovery::Manual)
                    .with_context("substeps", outcome.depenetrated.to_string()));
                }
            }
            frame.record(FrameStage::Physics, clock.elapsed())?;
        }

        // Render prep: what the edits changed, meshed and uploaded again in
        // the frame's own list; the camera is the player's eye, the origin
        // follows it, the pass is recorded.
        let clock = Instant::now();
        let mut list = CommandList::new("client frame");
        let live = self
            .live
            .as_mut()
            .ok_or_else(|| wrong("a frame before the window opened"))?;
        if let Some((moved, spent)) = shift {
            let uploading = Instant::now();
            let chunks = std::mem::take(&mut live.chunks);
            live.chunks = restream(rhi, &live.pass, &mut list, chunks, self.scene, &moved)?;
            self.tally.moves += 1;
            self.tally.meshed_in += moved.meshed as u64;
            self.tally.uploaded_in += moved.reupload.len() as u64;
            self.tally.slowest_move = self.tally.slowest_move.max(spent + uploading.elapsed());
            if self.tally.moved_frame.is_none() {
                self.tally.show_move = true;
            }
        }
        if !changed.is_empty() {
            let remeshing = Instant::now();
            let remeshed = self.scene.remesh(&self.world.borrow(), &changed)?;
            let upload = self.scene.reupload(&remeshed);
            for &index in &upload {
                let bytes = self.scene.vertices(index)?;
                let region = self.scene.regions[index].0;
                let fresh = live.pass.upload_vertices(rhi, &mut list, bytes, region)?;
                // Released once the last frame that drew it is done.
                std::mem::replace(&mut live.chunks[index], fresh).destroy(rhi)?;
            }
            self.tally.remeshed += remeshed.len() as u64;
            self.tally.uploaded += upload.len() as u64;
            self.tally.slowest_remesh = self.tally.slowest_remesh.max(remeshing.elapsed());
            if self.tally.edited_frame.is_none() {
                self.tally.show_edit = true;
            }
        }
        self.camera = camera_at(self.player.eye(), self.projection)?;
        self.origin = self.origin.follow(self.camera.position())?;
        let state = self.camera.sample(self.origin, live.width, live.height)?;
        let drawn = live
            .pass
            .record(&mut list, &state, live.target, live.depth, &live.chunks)?;
        frame.record(FrameStage::RenderPrep, clock.elapsed())?;

        // Render: one submission, then the window. The first frame, and the
        // first after the world changed, are read back to be judged.
        let clock = Instant::now();
        rhi.submit(list)?;
        let wanted = !self.tally.unreadable
            && (self.tally.first_frame.is_none() || self.tally.show_edit || self.tally.show_move);
        let mut captured = None;
        let shown = if wanted {
            match rhi.present_and_capture(live.target) {
                Ok(Some(image)) => {
                    if (image.width, image.height) != (live.width, live.height) {
                        return Err(wrong("the surface is not the size the frame was drawn at")
                            .with_context("surface", format!("{}x{}", image.width, image.height))
                            .with_context("frame", format!("{}x{}", live.width, live.height)));
                    }
                    captured = Some(Captured {
                        encoded: image.format.ends_with("Srgb"),
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

        // Judged against the world it showed, which no later frame has
        // changed yet. The world stands still while it is judged, so the
        // time is not the next frame's: the clock the loop reads moves past it.
        if let Some(captured) = captured {
            let judging = Instant::now();
            if self.tally.first_frame.is_none() {
                if let Some(path) = &self.capture {
                    write_ppm(path, &captured)?;
                }
                let check = judge(&self.world.borrow(), self.scene, &captured, "first")?;
                self.tally.first_frame = Some(check);
            } else {
                // One frame can show both an edit and a move; it is judged
                // once, against the world and the square it showed.
                let which = if self.tally.show_edit {
                    "edited"
                } else {
                    "moved"
                };
                if let Some(path) = &self.capture {
                    let stem = path
                        .file_stem()
                        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
                    write_ppm(
                        &path.with_file_name(format!("{stem}-{which}.ppm")),
                        &captured,
                    )?;
                }
                let check = judge(&self.world.borrow(), self.scene, &captured, which)?;
                if std::mem::take(&mut self.tally.show_edit) {
                    self.tally.edited_frame = Some(check);
                }
                if std::mem::take(&mut self.tally.show_move) {
                    self.tally.moved_frame = Some(check);
                }
            }
            if let Some(last) = self.tally.last.as_mut() {
                *last += judging.elapsed();
            }
        }

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
        if let Some(atlas) = live.atlas {
            rhi.destroy_texture(atlas)?;
        }
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
    match &report.start {
        Start::Created { seed } => {
            out.push_str(&format!(
                "start              a new world from seed {seed}\n"
            ));
        }
        Start::Resumed { path, tick, player } => out.push_str(&format!(
            "start              resumed from {} at tick {tick}; {}\n",
            path.display(),
            player.as_str()
        )),
    }
    match report.textures {
        Some((tiles, (width, height))) => out.push_str(&format!(
            "textures           {} tiles in a {width}x{height} atlas, {} blocks without a surface\n",
            tiles, report.unmapped
        )),
        None => out.push_str("textures           none: faces coloured by direction\n"),
    }
    out.push_str(&format!(
        "world              {} columns drawn of {} resident, blocks {}..{} high\n",
        report.columns.0, report.columns.1, report.band.0, report.band.1
    ));
    match report.first_frame {
        Some(check) => out.push_str(&format!(
            "first frame        {} of {} pixels judged by the ray cast, {} matching, {} snapped edges, {} snapped texels, {} back faces, read back from the surface\n",
            check.judged, check.pixels, check.matching, check.snapped, check.texel_snapped, check.backfacing
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
    let edits = report.edits;
    out.push_str(&format!(
        "edits              {} mined, {} built, {} refused, {} with nothing to aim at; {} regions meshed again, {} uploaded, slowest {}\n",
        edits.mined,
        edits.built,
        edits.refused,
        edits.unsent,
        report.remeshed.0,
        report.remeshed.1,
        ms(report.slowest_remesh)
    ));
    match (report.edited_frame, edits.mined + edits.built) {
        (Some(check), _) => out.push_str(&format!(
            "edited frame       {} of {} pixels judged by the ray cast, {} matching, {} snapped edges, {} snapped texels, {} back faces, the first frame after an edit\n",
            check.judged, check.pixels, check.matching, check.snapped, check.texel_snapped, check.backfacing
        )),
        (None, 0) => out.push_str("edited frame       none: nothing was edited\n"),
        (None, _) => out.push_str("edited frame       not readable on this surface\n"),
    }
    let streaming = report.streaming;
    out.push_str(&format!(
        "streaming          {} moves of the drawn square, {} columns meshed in, {} uploaded, slowest {}; {} columns made resident, {} evicted, {} edited ones retained\n",
        streaming.moves,
        streaming.meshed,
        streaming.uploaded,
        ms(streaming.slowest),
        streaming.residency.activated,
        streaming.residency.evicted,
        streaming.retained
    ));
    match report.moved_frame {
        Some(check) => out.push_str(&format!(
            "moved frame        {} of {} pixels judged by the ray cast, {} matching, {} snapped edges, {} snapped texels, {} back faces, the first frame after the square moved\n",
            check.judged, check.pixels, check.matching, check.snapped, check.texel_snapped, check.backfacing
        )),
        None if streaming.moves == 0 => {
            out.push_str("moved frame        none: the player stayed in its column\n");
        }
        None => out.push_str("moved frame        not readable on this surface\n"),
    }
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
    use nexora_simulation::{Walk, DEFAULT_BUTTONS, DEFAULT_KEYS};

    /// Found in review: a jump tapped on a frame that buys no tick used to be
    /// dropped. It is carried to the next frame that ticks, spent there once,
    /// and not repeated.
    #[test]
    fn a_jump_on_a_frame_without_a_tick_reaches_the_next_tick_once() {
        let jump = Walk {
            jump: true,
            ..Walk::default()
        };
        let mut latched = false;
        let carried = walk_for_ticks(&mut latched, jump, 0);
        assert!(latched, "held over a frame that bought no tick");
        assert!(carried.jump, "the frame's own walk is unchanged");
        let ticked = walk_for_ticks(&mut latched, Walk::default(), 1);
        assert!(ticked.jump, "spent on the next frame that ticks");
        assert!(!latched);
        assert!(
            !walk_for_ticks(&mut latched, Walk::default(), 1).jump,
            "and only once"
        );
    }

    /// The default seed's world, its scene, and the player standing in it.
    fn the_default_scene() -> (World, Scene, Player) {
        let mut world = World::create(
            WorldDescriptor::new("nexora-client", ClientConfig::default().seed).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        world.bring_online().unwrap();
        let table = SurfaceTable::untextured(&world);
        let radius = ClientConfig::default().radius;
        let centre = ChunkCoord::new(0, 0);
        ClientResidency::new(radius)
            .unwrap()
            .settle(&mut world, centre)
            .unwrap();
        let scene = scene::build(&world, centre, radius, table, None).unwrap();
        let player = Player::spawn(
            &WorldVoxels::new(&world),
            &scene.spawn_run(&world).unwrap(),
            scene::SPAWN_PITCH,
            DEFAULT_TICKS_PER_SECOND,
        )
        .unwrap();
        (world, scene, player)
    }

    const KEYBOARD: DeviceId = DeviceId::new(DeviceKind::Keyboard, 0);

    /// A click on a frame that buys no tick reaches the next tick, once; a
    /// frame without a tick edits nothing; and a mine and a build asked on
    /// different tickless frames both arrive.
    #[test]
    fn a_click_on_a_frame_without_a_tick_reaches_the_next_tick_once() {
        let mine = BlockIntent {
            mine: true,
            build: false,
        };
        let build = BlockIntent {
            mine: false,
            build: true,
        };
        let mut latched = BlockIntent::default();
        assert!(
            !block_for_ticks(&mut latched, mine, 0).any(),
            "no tick, no edit"
        );
        assert!(!block_for_ticks(&mut latched, build, 0).any());
        assert_eq!(
            block_for_ticks(&mut latched, BlockIntent::default(), 2),
            mine.or(build),
            "both, on the next frame that ticks, however many ticks"
        );
        assert!(
            !block_for_ticks(&mut latched, BlockIntent::default(), 1).any(),
            "and only once"
        );
        assert_eq!(
            block_for_ticks(&mut latched, mine, 1),
            mine,
            "a click on a ticking frame"
        );
    }

    /// The block a player aims at is the block under the centre of the
    /// frame: the eye's direction is the camera's at every facing a player
    /// can have (yaw in `(-π, π]`, which both wrap into).
    ///
    /// Not to the bit: the camera wraps the yaw it is given again
    /// (`rem_euclid`), which moves a negative one by a unit in the last
    /// place, as `the_camera_is_the_players_eye` already allows for. The ray
    /// is cast from the eye, by the authority; what differs is where the
    /// frame's centre is, by about 1e-16 of a radian.
    #[test]
    fn the_eye_aims_where_the_camera_looks() {
        let projection = Projection::perspective(1.0, 0.1, 100.0).unwrap();
        for yaw_step in -7..=8 {
            for pitch_step in -6..=6 {
                let eye = Eye {
                    position: WorldPosition::new(1.5, 70.0, -3.25),
                    yaw: f64::from(yaw_step) * std::f64::consts::PI / 8.0,
                    pitch: f64::from(pitch_step) * MAX_PITCH / 6.0,
                };
                let camera = camera_at(eye, projection).unwrap();
                let (a, b) = (eye.forward(), camera.forward());
                for axis in 0..3 {
                    assert!(
                        (a[axis] - b[axis]).abs() < 1e-15,
                        "yaw {} pitch {}: {a:?} against {b:?}",
                        eye.yaw,
                        eye.pitch
                    );
                }
            }
        }
    }

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
        // And the mouse: mine on the primary button, build on the secondary.
        use nexora_window::input::mouse_usage;
        use winit::event::MouseButton as M;
        let buttons = [M::Left, M::Right];
        assert_eq!(buttons.len(), DEFAULT_BUTTONS.len());
        for ((path, usage), button) in DEFAULT_BUTTONS.iter().zip(buttons) {
            assert_eq!(mouse_usage(button), ButtonCode(*usage), "{path}");
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
            texel_snapped: 0,
        };
        let texels = |judged, matching, texel_snapped| FrameCheck {
            pixels: 20_000,
            judged,
            matching,
            backfacing: 0,
            snapped: 0,
            texel_snapped,
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
        // Texel edges have their own, wider bound: WARP's first textured
        // frame snapped 21 in 97,505, past the geometry bound.
        assert!(frame_holds(&texels(10_000, 9_980, 20)));
        assert!(!frame_holds(&texels(10_000, 9_979, 21)), "past one in 500");
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

    /// A device for the GPU tests, or a skip where the machine declares it
    /// has none (`NEXORA_GPU=none`).
    fn backend() -> Option<WgpuRhi> {
        match WgpuRhi::new() {
            Ok(rhi) => Some(rhi),
            Err(error) if std::env::var("NEXORA_GPU").as_deref() == Ok("none") => {
                eprintln!("not run: NEXORA_GPU=none ({error})");
                None
            }
            Err(error) => panic!(
                "no GPU adapter: {error}\n\
                 install a Vulkan driver (Linux: mesa-vulkan-drivers) or set NEXORA_GPU=none"
            ),
        }
    }

    /// The drawn square followed through a walk — one column, diagonally,
    /// and back — with the device chunks updated by the frame loop's own
    /// [`restream`], at radius 2 so the inner columns keep their chunks: after every move the frame drawn from
    /// them, read back, holds against the ray cast of the square it moved to
    /// ([`judge`], the client's own check). The live client cannot be walked
    /// across a column yet — the terrain is a field of pillars — so this is
    /// where a move is drawn and judged.
    #[test]
    fn every_move_of_the_square_is_drawn_as_the_ray_cast_says() {
        const SIZE: (u32, u32) = (192, 144);
        let Some(mut rhi) = backend() else { return };
        let mut world = World::create(
            WorldDescriptor::new("nexora-client", ClientConfig::default().seed).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        world.bring_online().unwrap();
        let radius = 2;
        let mut residency = ClientResidency::new(radius).unwrap();
        residency.settle(&mut world, ChunkCoord::new(0, 0)).unwrap();
        let table = SurfaceTable::untextured(&world);
        let mut scene = scene::build(&world, ChunkCoord::new(0, 0), radius, table, None).unwrap();

        let texture = |rhi: &mut WgpuRhi, format, usage| {
            rhi.create_texture(&TextureDesc {
                label: "client move test".into(),
                width: SIZE.0,
                height: SIZE.1,
                format,
                usage,
            })
            .unwrap()
        };
        let target = texture(
            &mut rhi,
            TextureFormat::Rgba8Unorm,
            Usage::RENDER_TARGET | Usage::COPY_SRC,
        );
        let depth = texture(&mut rhi, TextureFormat::Depth32Float, Usage::RENDER_TARGET);
        let pass = ChunkPass::new(&mut rhi, TextureFormat::Rgba8Unorm).unwrap();
        let mut list = CommandList::new("client move test upload");
        let mut chunks = Vec::new();
        for (index, (region, _)) in scene.regions.iter().enumerate() {
            let bytes = scene.vertices(index).unwrap();
            chunks.push(
                pass.upload_vertices(&mut rhi, &mut list, bytes, *region)
                    .unwrap(),
            );
        }
        let fence = rhi.submit(list).unwrap();
        rhi.wait(fence).unwrap();

        let (_, high) = World::surface_range();
        let mut judged = 0;
        let mut kept = 0;
        for (cx, cz) in [(1, 0), (2, 1), (0, 0)] {
            let centre = ChunkCoord::new(cx, cz);
            residency.settle(&mut world, centre).unwrap();
            let shift = scene.recentre(&world, centre).unwrap();
            kept += (0..shift.from.len())
                .filter(|index| shift.from[*index].is_some() && !shift.reupload.contains(index))
                .count();
            let mut list = CommandList::new("client move test frame");
            chunks = restream(&mut rhi, &pass, &mut list, chunks, &scene, &shift).unwrap();

            // From above the square's middle, looking out and down over it:
            // every drawn column, the edges where columns entered and left
            // among them.
            let middle = |c: i64, size: u32| (c as f64 + 0.5) * f64::from(size) / 5.0;
            let [sx, _, sz] = scene.bounds.size;
            let eye = WorldPosition::new(
                scene.bounds.origin.x as f64 + middle(0, sx),
                (high + 30) as f64,
                scene.bounds.origin.z as f64 + middle(0, sz),
            );
            let mut camera = Camera::new(eye, scene.projection).unwrap();
            camera
                .look_at(WorldPosition::new(
                    scene.bounds.origin.x as f64 + middle(3, sx),
                    (high - 12) as f64,
                    scene.bounds.origin.z as f64 + middle(3, sz),
                ))
                .unwrap();
            let origin = RenderOrigin::containing(eye).unwrap();
            let state = camera.sample(origin, SIZE.0, SIZE.1).unwrap();
            let stats = pass
                .record(&mut list, &state, target, depth, &chunks)
                .unwrap();
            assert_eq!(stats.drawn + stats.culled + stats.empty, 25);
            assert!(
                stats.drawn >= 9,
                "({cx},{cz}): {} columns drawn",
                stats.drawn
            );
            let fence = rhi.submit(list).unwrap();
            rhi.wait(fence).unwrap();
            let captured = Captured {
                encoded: false,
                camera,
                width: SIZE.0,
                height: SIZE.1,
                rgba: rhi.read_texture(target).unwrap(),
            };
            let check = judge(&world, &scene, &captured, "moved")
                .unwrap_or_else(|error| panic!("({cx},{cz}): {error}"));
            assert!(
                check.judged * 2 >= check.pixels,
                "({cx},{cz}): only {} of {} pixels judged",
                check.judged,
                check.pixels
            );
            judged += check.judged;
        }
        assert!(judged > 0);
        assert!(kept > 0, "some regions kept their device chunks");
        for chunk in chunks {
            chunk.destroy(&mut rhi).unwrap();
        }
        pass.destroy(&mut rhi).unwrap();
        rhi.destroy_texture(target).unwrap();
        rhi.destroy_texture(depth).unwrap();
    }
}
