//! # NEXORA headless vertical slice
//!
//! The Phase 0 proof described in the startup brief §13 and §37: that the
//! foundation actually holds together end to end.
//!
//! ```text
//! boot the runtime lifecycle
//!   -> resolve and initialize engine modules
//!   -> create a world from a seed
//!   -> generate chunks in parallel through the job system
//!   -> mutate voxels
//!   -> drop a character onto the terrain and simulate it
//!   -> walk an observer away and back, streaming chunks out and in
//!   -> advance the world clock
//!   -> save
//!   -> shut down in reverse order
//!   -> reopen, load, and verify the state survived
//!   -> check every memory pool at rest: none over its ceiling, none leaking
//! ```
//!
//! **There is no window and no renderer here, by design.** The RHI boundary is
//! specified in `RENDER HARDWARE INTERFACE.md` but not implemented: a renderer
//! cannot be verified in a headless environment, and claiming a subsystem works
//! without running it is exactly what `NEXORA DEFINITION OF DONE.md` forbids.
//! What this slice proves is the part that can be run and checked.
//!
//! The verification step compares blocks by their **namespaced identifier**, not
//! by runtime id, so a save that came back with plausible-looking integers
//! pointing at the wrong content still fails.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;

use nexora_foundation::diagnostics::{Category, Diagnostics, Level, Record, StderrSink};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::memory::{
    MemoryBudget, MemoryClass, MemoryLedger, MemoryPool, MemoryReport, PoolSpec, Pressure,
};
use nexora_foundation::spatial::{BlockPos, ChunkCoord};
use nexora_foundation::time::{CalendarConfig, TimeScale, WorldDuration, WorldTime};
use std::path::Path;

use nexora_asset::texture::MapRole;
use nexora_command::identity::{Actor, Source};
use nexora_foundation::version::QueryVersion;
use nexora_image::TextureLoader;
use nexora_mesh::mesh::SurfaceId;
use nexora_mesh::mesh_region;
use nexora_mesh::view::Extent;
use nexora_persistence::journal::{self, Journal, SnapshotId};
use nexora_persistence::SaveContainer;
use nexora_physics::body::BodyDescriptor;
use nexora_physics::collision::overlaps_solid;
use nexora_physics::math::Vec3;
use nexora_physics::world::PhysicsWorld;
use nexora_query::{QueryRequest, QueryService, QueryStats};
use nexora_resource::ResourceManager;
use nexora_runtime::jobs::{JobOutcome, JobSystem, Priority};
use nexora_runtime::lifecycle::{Lifecycle, Phase, RuntimeMode};
use nexora_runtime::module::{
    EngineModule, ModuleContext, ModuleDependency, ModuleId, ModuleManager, ModuleSides,
};
use nexora_simulation::queries::WORLD_QUERY_VERSION;
use nexora_simulation::WorldQueries;
use nexora_simulation::{
    BlockContent, PhysicsModule, RetainedChunks, WorldResidency, WorldSurfaces, WorldVoxels,
    UNMAPPED_SURFACE,
};
use nexora_streaming::budget::StreamingBudget;
use nexora_streaming::interest::{InterestId, InterestSource, LodRadii};
use nexora_streaming::system::StreamingSystem;
use nexora_world::persist;
use nexora_world::voxel::BlockStateId;
use nexora_world::world::{BlockDefinition, World, WorldDescriptor};

/// How the slice should be run.
#[derive(Debug, Clone)]
pub struct SliceConfig {
    /// World generation seed.
    pub seed: u64,
    /// Chunk radius to generate around the origin.
    pub radius: i64,
    /// Where the save is written.
    pub save_path: PathBuf,
    /// Worker threads for chunk generation.
    pub worker_threads: usize,
    /// Whether to emit diagnostics to standard error.
    pub verbose: bool,
    /// A block content document to register, place and verify, if any.
    ///
    /// Optional so that the slice's own numbers stay what the README records
    /// when no content is given; with it, every content block crosses the
    /// same save, reload and recovery boundaries as the slice's own edits,
    /// and is checked through the mesher.
    pub content: Option<PathBuf>,
    /// A resource root (with its `resources.json`) holding the content's
    /// textures, if any. Requires `content`: every content block's texture is
    /// then resolved by identifier, verified against the index and decoded.
    pub resources: Option<PathBuf>,
}

impl Default for SliceConfig {
    fn default() -> Self {
        Self {
            seed: 0x4E45_584F_5241,
            radius: 2,
            save_path: PathBuf::from("run/world.nxsv"),
            worker_threads: 4,
            verbose: true,
            content: None,
            resources: None,
        }
    }
}

/// What the slice observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceReport {
    /// The world's persistent identity.
    pub world_id: u64,
    /// Seed used.
    pub seed: u64,
    /// Chunk columns generated.
    pub chunks_generated: usize,
    /// Voxel edits applied before saving.
    pub blocks_edited: usize,
    /// World ticks advanced before saving.
    pub ticks_advanced: u64,
    /// Size of the save file on disk, in bytes.
    pub save_bytes: usize,
    /// Approximate in-memory voxel storage, in bytes.
    pub storage_bytes: usize,
    /// Non-air blocks across resident chunks.
    pub non_air_blocks: u64,
    /// Physics substeps run.
    pub physics_substeps: u32,
    /// Bodies simulated.
    pub physics_bodies: usize,
    /// Contacts resolved against the voxel grid.
    pub physics_contacts: u32,
    /// Bodies that settled and went to sleep.
    pub physics_settled: usize,
    /// How far the dropped character fell before landing, in centimetres.
    pub physics_drop_cm: i64,
    /// Streaming ticks run while the observer walked away and back.
    pub streaming_ticks: u32,
    /// Chunks generated by streaming rather than restored.
    pub streaming_generated: u64,
    /// Chunks brought back from retention.
    pub streaming_restored: u64,
    /// The most chunks held outside the world at once, because their content
    /// could not be regenerated from the seed.
    pub streaming_retained_peak: usize,
    /// Chunks streaming released.
    pub streaming_evicted: u32,
    /// Edits the engine recorded in the journal since the checkpoint.
    pub journal_edits: u64,
    /// Probes that held after rebuilding the world from checkpoint + journal.
    ///
    /// Equal to `probes_verified` when recovery reproduced everything the save
    /// contains, which is the property the whole mechanism exists for.
    pub recovered_probes: usize,
    /// Commands that passed every validation layer and changed the world.
    pub commands_accepted: usize,
    /// Commands refused, at any layer. Non-zero on purpose: the slice submits
    /// requests that must be turned away, because a pipeline that has only
    /// ever accepted things has not been shown to refuse anything.
    pub commands_refused: usize,
    /// Probes checked after reloading.
    pub probes_verified: usize,
    /// Queries answered: every probe is read back through `nexora:block_at`.
    pub queries_answered: u64,
    /// Queries refused. Non-zero on purpose, like `commands_refused`.
    pub queries_refused: u64,
    /// Content blocks registered from the content document.
    pub content_blocks: usize,
    /// Distinct surfaces those blocks showed in a mesh of the placed row.
    pub content_surfaces: usize,
    /// Content textures resolved, verified and decoded through the resource
    /// system.
    pub content_textures: usize,
    /// Every memory pool the slice opened, at the end of the run.
    pub memory: MemoryReport,
    /// Lifecycle phases entered, in order.
    pub phases: Vec<&'static str>,
}

/// A single expectation carried across the save/load boundary.
#[derive(Debug, Clone)]
struct Probe {
    position: BlockPos,
    expected: Identifier,
}

/// A module that owns nothing but proves the graph resolves and orders.
///
/// Real subsystems replace these as they arrive; the point today is that the
/// lifecycle, ordering and rollback paths are exercised by something rather
/// than sitting untested until the first real module needs them.
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

/// A client-only module, present to prove a headless run skips it rather than
/// failing on it (`ENGINE MODULE SYSTEM.md` invariants).
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
    fn initialize(&mut self, _context: &ModuleContext) -> Result<()> {
        Err(Error::new(
            Domain::Module,
            "module/renderer",
            "the renderer has no implementation yet and must never initialize",
        )
        .fatal())
    }
}

/// Run the full slice and verify it.
///
/// # Errors
///
/// Returns an error at the first step that fails, including a verification
/// mismatch after reloading.
pub fn run_slice(config: &SliceConfig) -> Result<SliceReport> {
    let diagnostics = if config.verbose {
        Diagnostics::new(vec![Box::new(StderrSink)], Level::Info)
    } else {
        Diagnostics::silent()
    };

    let mut lifecycle = Lifecycle::new(RuntimeMode::Headless);
    lifecycle.advance_through(Phase::FoundationInit)?;
    log(&diagnostics, "runtime foundation initialized");

    // --- modules -----------------------------------------------------------
    let mut modules = ModuleManager::new();
    modules.register(Box::new(WorldModule))?;
    modules.register(Box::new(RendererModule))?;
    modules.register(Box::new(PhysicsModule))?;
    modules.register(Box::new(FoundationModule))?;
    lifecycle.advance_to(Phase::ModuleDiscovery)?;

    modules.resolve(RuntimeMode::Headless)?;
    lifecycle.advance_to(Phase::ModuleResolution)?;

    let module_context = ModuleContext::new(RuntimeMode::Headless, diagnostics.clone());
    modules.initialize_all(&module_context)?;
    lifecycle.advance_through(Phase::RuntimeInit)?;
    diagnostics
        .counters()
        .add("modules.initialized", modules.resolved_order().len() as u64);
    log(&diagnostics, "engine modules initialized");

    // --- world -------------------------------------------------------------
    let content = config
        .content
        .as_deref()
        .map(BlockContent::load)
        .transpose()?;
    let content_blocks = content
        .as_ref()
        .map(BlockContent::block_definitions)
        .unwrap_or_default();
    let descriptor = WorldDescriptor::new("nexora-slice", config.seed)?;
    let world_id = descriptor.id.0;
    let mut world = World::create_with(descriptor, CalendarConfig::earthlike(), &content_blocks)?;
    lifecycle.advance_to(Phase::WorldAttach)?;
    world.bring_online()?;
    lifecycle.advance_to(Phase::SimulationRunning)?;
    log(&diagnostics, "world attached and simulating");

    // --- generation through the job system ---------------------------------
    let coords: Vec<ChunkCoord> = (-config.radius..=config.radius)
        .flat_map(|x| (-config.radius..=config.radius).map(move |z| ChunkCoord::new(x, z)))
        .collect();
    let chunks_generated = coords.len();
    let memory = SliceMemory::open(chunks_generated as u64, config.resources.is_some())?;

    let generated = generate_in_parallel(world, &coords, config.worker_threads, &diagnostics)?;
    world = generated;
    memory.world.record(world.storage_bytes() as u64);
    diagnostics
        .counters()
        .add("chunks.generated", chunks_generated as u64);
    log(&diagnostics, "chunks generated");

    // --- checkpoint ----------------------------------------------------------
    // A snapshot of the freshly generated world, taken BEFORE any edit, and a
    // journal bound to it. This is the real shape of `Snapshot + Journal ->
    // Recovery`: checkpoint, then record what happens next. Every edit from
    // here on is journalled by `World::set_block` itself -- the slice never
    // mentions the journal again until it verifies recovery at the end.
    let checkpoint = persist::save(&world)?.encode();
    let checkpoint_id = SnapshotId::of(world.descriptor().id.0, &checkpoint);
    let journal_path = config.save_path.with_extension("nxjr");
    world.attach_journal(Journal::create(&journal_path, checkpoint_id)?);
    log(&diagnostics, "checkpoint taken, journal open");

    // --- mutation ----------------------------------------------------------
    let content_ids: Vec<Identifier> = content_blocks.iter().map(|(id, _)| id.clone()).collect();
    let probes = apply_edits(&mut world, &coords, &content_ids)?;
    memory.world.record(world.storage_bytes() as u64);
    let content_surfaces = match &content {
        Some(content) => verify_content_surfaces(&world, content)?,
        None => 0,
    };
    let content_textures = match (&content, &config.resources) {
        (Some(content), Some(root)) => {
            let pool = memory
                .textures
                .as_ref()
                .ok_or_else(|| mismatch("the texture pool was not opened"))?;
            load_content_textures(content, root, pool, &diagnostics)?
        }
        (None, Some(_)) => {
            return Err(mismatch(
                "a resource root was given without content to look up",
            ))
        }
        _ => 0,
    };
    let blocks_edited = probes.len();
    diagnostics
        .counters()
        .add("blocks.edited", blocks_edited as u64);

    // --- commands ----------------------------------------------------------
    // The reachable part of `Command System.md` §115: intent, validated by the
    // full pipeline, executed by exactly one authority, changing the world.
    // Networking, Build & Destruction, Loot and Item do not exist yet.
    let (returned, commands) = run_command_stage(world, &diagnostics)?;
    world = returned;
    diagnostics
        .counters()
        .add("commands.accepted", commands.accepted as u64);
    diagnostics
        .counters()
        .add("commands.refused", commands.refused as u64);
    log(&diagnostics, "commands dispatched");

    // --- physics -----------------------------------------------------------
    // Runs against the world but never writes to it, so the save below - and
    // the byte-identical determinism check over it - is unaffected.
    let physics = simulate_physics(&world, &diagnostics)?;
    diagnostics
        .counters()
        .add("physics.substeps", u64::from(physics.substeps));
    log(&diagnostics, "physics settled");

    // --- streaming ---------------------------------------------------------
    // Walks an observer away from the edited region and back. Everything the
    // walk touches is regenerable except the columns edited above, so this is
    // where "logical identity survives eviction" either holds or does not - and
    // the reload verification at the end of the slice is what checks it.
    let streaming = stream_a_walk(&mut world, config.radius, &memory, &diagnostics)?;
    diagnostics
        .counters()
        .add("streaming.generated", streaming.generated);
    log(&diagnostics, "observer walked away and back");

    // --- time --------------------------------------------------------------
    let before = world.clock().now();
    world.clock_mut().advance_by(TimeScale::Hour, 7)?;
    let ticks_advanced = world.clock().now().since(before).ticks();

    let storage_bytes = world.storage_bytes();
    memory.world.record(storage_bytes as u64);
    let non_air_blocks: u64 = world
        .loaded_chunks()
        .iter()
        .filter_map(|c| world.chunk(*c))
        .map(|c| c.non_air_count())
        .sum();
    let saved_time = world.clock().now();

    // --- save --------------------------------------------------------------
    let container = persist::save(&world)?;
    let save_bytes = container.encode().len();
    container.write_atomic(&config.save_path)?;

    // The commit boundary. One fsync covers every edit since the checkpoint;
    // measured at 203 us against 672 ns per append, which is why the slice
    // syncs here rather than per edit.
    let journal_edits = world.unsynced_edits();
    world.sync_journal()?;
    let journal = world.detach_journal();
    diagnostics.counters().add("journal.edits", journal_edits);
    log(&diagnostics, "world saved");

    // --- shutdown ----------------------------------------------------------
    lifecycle.advance_through(Phase::WorldFlush)?;
    modules.shutdown_all(&module_context);
    lifecycle.advance_through(Phase::ProcessExit)?;
    drop(world);
    memory.world.record(0);
    log(&diagnostics, "runtime shut down");

    // --- reopen and verify -------------------------------------------------
    let reopened = SaveContainer::read(&config.save_path)?;
    let restored = persist::load_with(&reopened, &content_blocks)?;
    memory.world.record(restored.storage_bytes() as u64);

    if restored.clock().now() != saved_time {
        return Err(mismatch("world time did not survive the save")
            .with_context("expected", saved_time.ticks().to_string())
            .with_context("found", restored.clock().now().ticks().to_string()));
    }
    if restored.chunk_count() != chunks_generated {
        return Err(mismatch("chunk count did not survive the save")
            .with_context("expected", chunks_generated.to_string())
            .with_context("found", restored.chunk_count().to_string()));
    }
    if restored.descriptor().id.0 != world_id {
        return Err(mismatch("world identity did not survive the save"));
    }

    // Read back through the query contract, as any other system would: the
    // slice is a consumer of the world like the rest, not a privileged one.
    let mut query_service = QueryService::new()?;
    let world_queries = WorldQueries::register(&mut query_service)?;
    query_service.freeze();
    let server = Actor::server();
    for probe in &probes {
        let found_id = query_service
            .ask(
                &world_queries.block_at,
                &restored,
                &QueryRequest {
                    actor: &server,
                    source: Source::Local,
                    version: WORLD_QUERY_VERSION,
                    input: probe.position,
                },
            )
            .map_err(|failure| {
                mismatch("a probe could not be read back through the query contract")
                    .with_context("position", describe(probe.position))
                    .with_context("failure", failure.to_string())
            })?;
        if found_id != probe.expected {
            return Err(mismatch("a block changed across the save/load boundary")
                .with_context("position", describe(probe.position))
                .with_context("expected", probe.expected.to_string())
                .with_context("found", found_id.to_string()));
        }
    }
    // Two questions the contract must turn away, for the reason the command
    // stage refuses three: a gate only ever shown opening has not been shown
    // to close. A player asking to scan far more than a query may, and a
    // client built against a version of the contract that does not exist.
    let player = Actor::player(1);
    let refusals = [
        query_service
            .ask(
                &world_queries.blocks_in_region,
                &restored,
                &QueryRequest {
                    actor: &player,
                    source: Source::Network,
                    version: WORLD_QUERY_VERSION,
                    input: (BlockPos::new(0, 0, 0), BlockPos::new(63, 63, 63)),
                },
            )
            .err(),
        query_service
            .ask(
                &world_queries.world_time,
                &restored,
                &QueryRequest {
                    actor: &player,
                    source: Source::Network,
                    version: QueryVersion(WORLD_QUERY_VERSION.0 + 1),
                    input: (),
                },
            )
            .err(),
    ];
    if refusals.iter().any(Option::is_none) {
        return Err(mismatch("a query that must be refused was answered"));
    }
    let query_stats = query_service.stats();
    query_service.publish(diagnostics.counters(), QueryStats::default());
    log(&diagnostics, "reload verified");

    // --- recovery ------------------------------------------------------------
    // Snapshot + Journal -> Recovery, proved against this run's own data: take
    // the pre-edit checkpoint, replay the journal the engine wrote while
    // editing, dispatching commands and streaming, and require every probe to
    // hold. If this passes, a process that died after the last commit boundary
    // would have come back with its edits.
    let recovered_probes = verify_recovery(
        &checkpoint,
        checkpoint_id,
        &journal_path,
        &probes,
        &content_blocks,
        &diagnostics,
    )?;
    drop(journal);
    log(&diagnostics, "recovery verified");

    // --- memory --------------------------------------------------------------
    // Every stage is over, so this is a point at rest: what must have drained
    // has to have drained, and no pool may have gone over its ceiling.
    let memory = memory.verify()?;
    log(&diagnostics, "memory budgets held");

    Ok(SliceReport {
        world_id,
        seed: config.seed,
        chunks_generated,
        blocks_edited,
        ticks_advanced,
        save_bytes,
        storage_bytes,
        non_air_blocks,
        journal_edits,
        recovered_probes,
        commands_accepted: commands.accepted,
        commands_refused: commands.refused,
        physics_substeps: physics.substeps,
        physics_bodies: physics.bodies,
        physics_contacts: physics.contacts,
        physics_settled: physics.settled,
        physics_drop_cm: physics.drop_cm,
        streaming_ticks: streaming.ticks,
        streaming_generated: streaming.generated,
        streaming_restored: streaming.restored,
        streaming_retained_peak: streaming.retained_peak,
        streaming_evicted: streaming.evicted,
        probes_verified: probes.len(),
        queries_answered: query_stats.answered,
        queries_refused: query_stats.refused,
        content_blocks: content_blocks.len(),
        content_surfaces,
        content_textures,
        memory,
        phases: lifecycle
            .history()
            .iter()
            .map(|phase| phase.as_str())
            .collect(),
    })
}

/// Generate chunks on the worker pool and fold them back into the world.
///
/// Generation only reads the world, so it parallelizes cleanly behind an `Arc`.
/// The world is reclaimed afterwards, which also asserts that no job leaked a
/// reference to it.
fn generate_in_parallel(
    world: World,
    coords: &[ChunkCoord],
    worker_threads: usize,
    diagnostics: &Diagnostics,
) -> Result<World> {
    let pool = JobSystem::new(worker_threads)?;
    let shared = Arc::new(world);
    let (sender, receiver) = mpsc::channel();

    let handles: Vec<_> = coords
        .iter()
        .map(|coord| {
            let coord = *coord;
            let world = shared.clone();
            let sender = sender.clone();
            pool.submit(Priority::High, move |token| {
                if token.is_cancelled() {
                    return Ok(());
                }
                let chunk = world.generate_chunk(coord)?;
                sender.send(chunk).map_err(|_| {
                    Error::new(Domain::Job, "slice", "chunk receiver went away").fatal()
                })
            })
        })
        .collect();

    // Drop the extra sender so the channel closes once every job has finished.
    drop(sender);
    pool.barrier();

    for handle in handles {
        match pool.wait(handle) {
            JobOutcome::Completed => {}
            other => {
                return Err(Error::new(
                    Domain::Job,
                    "slice",
                    "chunk generation job did not complete",
                )
                .with_recovery(Recovery::Retry)
                .with_context("outcome", format!("{other:?}")))
            }
        }
    }

    let metrics = pool.metrics();
    diagnostics
        .counters()
        .add("jobs.completed", metrics.completed);
    drop(pool);

    let chunks: Vec<_> = receiver.iter().collect();

    let mut world = Arc::try_unwrap(shared).map_err(|_| {
        Error::new(
            Domain::World,
            "slice",
            "a generation job outlived the world it borrowed",
        )
        .fatal()
    })?;
    for chunk in chunks {
        world.insert_chunk(chunk);
    }
    Ok(world)
}

/// Apply the edits whose survival the slice verifies.
///
/// Positions are derived from the columns that were actually generated rather
/// than hard-coded, so the slice is correct at any radius - including a radius
/// of zero, where only the origin column exists.
/// Rebuild the world from checkpoint + journal, and check every probe.
///
/// This is the claim `DEBT-0025` guarded: that the save survives a process
/// crash in the *running engine*. The journal it replays was written by
/// `World::set_block` during the run — the slice never appended a record by
/// hand — so a pass here is evidence about the engine, not about the test.
///
/// # Errors
///
/// Returns an error when the journal cannot be replayed, when replay reports
/// damage or skipped records, or when a probe does not hold in the recovered
/// world.
fn verify_recovery(
    checkpoint: &[u8],
    checkpoint_id: SnapshotId,
    journal_path: &Path,
    probes: &[Probe],
    content: &[(Identifier, BlockDefinition)],
    diagnostics: &Diagnostics,
) -> Result<usize> {
    let replay = journal::replay(journal_path, checkpoint_id)?;
    if let Some(damage) = replay.damage {
        return Err(mismatch("the journal this run wrote back is damaged")
            .with_context("damage", format!("{damage:?}")));
    }

    let container = SaveContainer::decode(checkpoint)?;
    let mut rebuilt = persist::load_with(&container, content)?;
    let report = nexora_world::recovery::apply(&mut rebuilt, &replay)?;

    if !report.skipped.is_empty() {
        // Every chunk the slice edits is resident in the checkpoint, so a skip
        // here is a real defect rather than DEBT-0024's known limitation.
        return Err(mismatch("recovery could not apply every recorded edit")
            .with_context("applied", report.applied.to_string())
            .with_context("skipped", report.skipped.len().to_string()));
    }
    diagnostics
        .counters()
        .add("journal.replayed", report.applied as u64);

    let mut verified = 0usize;
    for probe in probes {
        let found = rebuilt.get_block(probe.position)?;
        let found_id = rebuilt.block_identifier(found).ok_or_else(|| {
            mismatch("a recovered block state has no registered identifier")
                .with_context("position", describe(probe.position))
        })?;
        if found_id != probe.expected {
            return Err(mismatch("recovery did not reproduce an edit")
                .with_context("position", describe(probe.position))
                .with_context("expected", probe.expected.to_string())
                .with_context("found", found_id.to_string()));
        }
        verified += 1;
    }
    Ok(verified)
}

/// What the command stage did.
struct CommandOutcome {
    accepted: usize,
    refused: usize,
}

/// Drive real commands through the full pipeline against the live world.
///
/// The net effect on the world is **zero**: every block this places, it breaks
/// again. That is deliberate — the stage has to prove the pipeline mutates a
/// real world, without changing what the save contains, so the byte-identical
/// determinism comparison across worker counts keeps measuring what it was
/// written to measure.
///
/// It submits requests that must be **refused** as well as accepted. A pipeline
/// that has only ever been shown accepting things has not been shown to refuse
/// anything, and refusing is the half that matters for §71's security boundary.
fn run_command_stage(world: World, diagnostics: &Diagnostics) -> Result<(World, CommandOutcome)> {
    use nexora_command::dispatcher::Dispatcher;
    use nexora_command::identity::{Actor, ActorKind, CommandId, InstanceIdSource, Source};
    use nexora_command::instance::{CommandContext, CommandInstance, Target};
    use nexora_command::registry::CommandRegistry;
    use nexora_command::validation::ValidationPipeline;
    use nexora_foundation::diagnostics::CorrelationId;
    use nexora_simulation::commands::{
        register_block_commands, ActorPosition, BlockTargetValidator, BreakBlockHandler,
        PlaceBlockHandler, BREAK_BLOCK, PLACE_BLOCK,
    };

    // A cell high above the terrain, so placing into it cannot disturb
    // generated ground or any probe.
    let position = BlockPos::new(2, 210, 2);
    let stone: BlockStateId = world.block_id(&Identifier::parse("nexora:block/stone")?)?;

    // Taken by value and handed back, the same shape `generate_in_parallel`
    // uses: the handlers need shared ownership, and the slice needs the world
    // afterwards.
    let shared = std::rc::Rc::new(std::cell::RefCell::new(world));

    let mut registry = CommandRegistry::new()?;
    let (break_id, place_id) = register_block_commands(&mut registry)?;
    registry.freeze();

    let standing = ActorPosition {
        x: f64::from(position.x as i32) + 0.5,
        y: f64::from(position.y as i32) + 1.5,
        z: f64::from(position.z as i32) + 0.5,
    };
    let mut dispatcher = Dispatcher::new().with_pipeline(
        ValidationPipeline::standard().with_domain(Box::new(BlockTargetValidator {
            actor_position: Some(standing),
        })),
    );
    dispatcher.attach(
        break_id,
        Box::new(BreakBlockHandler::new(std::rc::Rc::clone(&shared))),
    )?;
    dispatcher.attach(
        place_id,
        Box::new(PlaceBlockHandler::new(std::rc::Rc::clone(&shared), stone)),
    )?;

    let mut ids = InstanceIdSource::new();
    let tick = WorldTime(1);
    let mut request =
        |command: &str, actor: Actor, source: Source, target: Target| -> Result<CommandInstance> {
            Ok(CommandInstance::new(
                CommandId::parse(command)?,
                ids.mint()?,
                target,
                Vec::new(),
                CommandContext::new(actor, source, tick, CorrelationId(1)),
            ))
        };

    let at = Target::Block {
        x: position.x,
        y: position.y,
        z: position.z,
    };
    let far = Target::Block {
        x: position.x + 500,
        y: position.y,
        z: position.z,
    };

    let attempts = vec![
        // Accepted: places a block into empty air, then breaks it again.
        request(PLACE_BLOCK, Actor::player(1), Source::Network, at.clone())?,
        request(BREAK_BLOCK, Actor::player(1), Source::Network, at.clone())?,
        // Refused: nothing there to break any more (§23 target validation).
        request(BREAK_BLOCK, Actor::player(1), Source::Network, at)?,
        // Refused: out of the actor's reach, decided by the server (§72).
        request(BREAK_BLOCK, Actor::player(1), Source::Network, far)?,
        // Refused: a script is not an allowed actor for these definitions.
        request(
            BREAK_BLOCK,
            Actor::of(ActorKind::Script),
            Source::ModRuntime,
            Target::None,
        )?,
    ];

    let mut accepted = 0usize;
    let mut refused = 0usize;
    for instance in &attempts {
        let result = dispatcher.dispatch(&registry, instance, tick);
        if result.succeeded() {
            accepted += 1;
        } else {
            refused += 1;
            diagnostics.counters().add("commands.refused.observed", 1);
        }
    }

    // The handlers hold clones of the world handle, so they have to go before
    // it can be reclaimed. Dropping the dispatcher drops them.
    drop(dispatcher);
    let world = std::rc::Rc::try_unwrap(shared)
        .map_err(|_| mismatch("the command stage leaked a handle to the world"))?
        .into_inner();

    // The stage must be a no-op on the world: it placed one block and broke it.
    if world.get_block(position)? != nexora_world::voxel::AIR {
        return Err(mismatch(
            "the command stage left a block behind; the save would no longer be \
             comparable across worker counts",
        ));
    }

    Ok((world, CommandOutcome { accepted, refused }))
}

/// Where content block `index` is placed: a row in the origin column, high
/// enough to be above any terrain and below the slice's own top-of-world edit.
fn content_position(index: usize) -> BlockPos {
    let index = index as i64;
    BlockPos::new(index % 16, CONTENT_ROW_Y, 2 + 2 * (index / 16))
}

/// The height of the content row.
const CONTENT_ROW_Y: i64 = 220;

/// Mesh the placed content row and require every content block to show its
/// own surface: none unmapped, none sharing, none missing from the mesh.
fn verify_content_surfaces(world: &World, content: &BlockContent) -> Result<usize> {
    let materials = content.material_registry()?;
    let table = content.surface_table(world, &materials)?;
    let expected: BTreeSet<SurfaceId> = content
        .blocks()
        .iter()
        .map(|block| {
            let state = world.block_id(&block.id)?;
            table
                .surface_of(state.0)
                .filter(|surface| *surface != UNMAPPED_SURFACE)
                .ok_or_else(|| {
                    mismatch("a content block has no surface")
                        .with_context("block", block.id.to_string())
                })
        })
        .collect::<Result<_>>()?;
    if expected.len() != content.blocks().len() {
        return Err(mismatch("two content blocks share a surface")
            .with_context("blocks", content.blocks().len().to_string())
            .with_context("surfaces", expected.len().to_string()));
    }

    let rows = content.blocks().len().div_ceil(16) as u32;
    let extent = Extent::new(
        BlockPos::new(0, CONTENT_ROW_Y - 1, 1),
        [16, 3, 2 * rows + 1],
    )?;
    let mesh = mesh_region(&WorldSurfaces::new(world, table), extent);
    let shown: BTreeSet<SurfaceId> = mesh.quads.iter().map(|quad| quad.surface).collect();
    if let Some(missing) = expected.iter().find(|surface| !shown.contains(surface)) {
        return Err(mismatch("a placed content block did not reach the mesh")
            .with_context("surface", missing.0.to_string()));
    }
    Ok(expected.len())
}

/// The decoded-texture budget the slice runs with: sixteen first-generation
/// albedos are 16 KiB of pixels, so this holds them all without eviction and
/// leaves no room for anything larger to hide in.
const TEXTURE_BUDGET: u64 = 64 * 1024;

/// Voxel storage per column, as `[target, warning, critical, emergency]`
/// (`NEXORA PERFORMANCE BUDGETS.md`).
///
/// Measured, not guessed: a generated and edited column costs 28.7-32.1 KiB at
/// every radius from 0 to 6 and every seed tried, and the figure is a property
/// of the data structures -- the same on any machine, unlike a timing. Target
/// is the worst measured column with a quarter to spare; the ceiling is one
/// 32x32x32 section stored without a palette (128 KiB), the point at which
/// palettes have stopped paying for themselves and the storage model, not the
/// budget, is what failed.
const COLUMN_BUDGET: [u64; 4] = [40 << 10, 48 << 10, 64 << 10, 128 << 10];

/// The slice's memory pools, one per owner (`NEXORA MEMORY AND RESOURCE
/// OWNERSHIP.md`, ADR-0024).
struct SliceMemory {
    ledger: MemoryLedger,
    /// Chunks resident in the world. Streaming enforces this pool's ceiling.
    world: Arc<MemoryPool>,
    /// Edited chunks held outside the world until they can be written back.
    /// Must be empty before the save, or the save is written without them.
    retained: Arc<MemoryPool>,
    /// Decoded textures in the resource cache, when the slice loads them.
    textures: Option<Arc<MemoryPool>>,
}

impl SliceMemory {
    fn open(columns: u64, textures: bool) -> Result<Self> {
        let ledger = MemoryLedger::new();
        let [target, warning, critical, emergency] = COLUMN_BUDGET.map(|b| b * columns);
        let budget = MemoryBudget::new(target, warning, critical, emergency)?;
        let world = ledger.register(PoolSpec::new("world.chunks", MemoryClass::World, budget))?;
        // The slice edits every column it generates, so every one of them may
        // be held outside the world at once -- and is, measured, at the far
        // end of the walk. The same per-column figures apply.
        let retained = ledger.register(
            PoolSpec::new("world.retained", MemoryClass::World, budget).drains_at_rest(),
        )?;
        // Opened only when the slice loads textures: a pool nobody records
        // into is refused by `verify` as a wiring mistake.
        let textures = if textures {
            Some(
                ledger.register(
                    PoolSpec::new(
                        "resource.textures",
                        MemoryClass::Streaming,
                        MemoryBudget::capacity(TEXTURE_BUDGET)?,
                    )
                    .drains_at_rest(),
                )?,
            )
        } else {
            None
        };
        Ok(Self {
            ledger,
            world,
            retained,
            textures,
        })
    }

    /// Check the ledger at rest, and hand back its report.
    fn verify(self) -> Result<MemoryReport> {
        let leaks = self.ledger.suspected_leaks();
        if !leaks.is_empty() {
            return Err(mismatch("a memory pool that must drain at rest did not")
                .with_context("pools", leaks.join(", ")));
        }
        let report = self.ledger.report();
        if let Some(pool) = report
            .pools
            .iter()
            .find(|pool| pool.worst == Pressure::Emergency)
        {
            return Err(mismatch("a memory pool went over its ceiling")
                .with_context("pool", pool.name)
                .with_context("high_water", pool.high_water.to_string())
                .with_context("ceiling", pool.budget.emergency().to_string()));
        }
        if let Some(pool) = report.pools.iter().find(|pool| pool.records == 0) {
            return Err(mismatch("a memory pool was opened and never recorded into")
                .with_context("pool", pool.name));
        }
        Ok(report)
    }
}

/// Resolve every content block's albedo by identifier, verify it against the
/// resource index, and decode it -- the runtime reaching its own textures
/// without the tool that wrote them (ADR-0021, ADR-0022).
fn load_content_textures(
    content: &BlockContent,
    root: &Path,
    pool: &Arc<MemoryPool>,
    diagnostics: &Diagnostics,
) -> Result<usize> {
    let mut resources = ResourceManager::open(root, TEXTURE_BUDGET)?;
    resources.cache_mut().attach(Arc::clone(pool))?;
    // Cross-system, before any load: the index must provide every map the
    // content asks for, and a gap is reported whole rather than as whichever
    // texture the loop below would have tripped over first.
    resources.manifest().provides(content.materials())?;
    let before = resources.cache().stats();
    // The first visual generation is 16x16; anything larger is a content
    // error, found here rather than in video memory.
    let loader = TextureLoader::new(MapRole::Albedo).at_most(16);
    for material in content.materials() {
        let id = material.map_asset_id(MapRole::Albedo)?;
        let handle = resources.resolve(&id, &loader)?;
        let map = resources.load(&handle, &loader)?;
        if map.resolution() != material.resolution() {
            return Err(mismatch("a texture is not the size its material declares")
                .with_context("texture", id.to_string()));
        }
    }
    resources.cache().publish(diagnostics.counters(), before);
    Ok(content.materials().len())
}

fn apply_edits(
    world: &mut World,
    coords: &[ChunkCoord],
    content: &[Identifier],
) -> Result<Vec<Probe>> {
    let air = Identifier::parse("nexora:block/air")?;
    let stone = Identifier::parse("nexora:block/stone")?;
    let grass = Identifier::parse("nexora:block/grass")?;
    let dirt = Identifier::parse("nexora:block/dirt")?;

    let shape = world.descriptor().shape;
    let size_x = i64::from(shape.size_x());
    let size_z = i64::from(shape.size_z());
    let bounds = world.descriptor().bounds;

    let mut edits: Vec<(BlockPos, &Identifier)> = Vec::new();
    for coord in coords {
        let origin_x = coord.x * size_x;
        let origin_z = coord.z * size_z;

        // Well above the terrain: builds a block into empty air.
        edits.push((BlockPos::new(origin_x + 1, 200, origin_z + 1), &stone));
        // The far corner of the column, on a section boundary.
        edits.push((
            BlockPos::new(origin_x + size_x - 1, 300, origin_z + size_z - 1),
            &dirt,
        ));
        // Deep underground: carves air out of solid uniform stone, which forces
        // a uniform section to grow a palette.
        edits.push((BlockPos::new(origin_x, bounds.min_y + 20, origin_z), &air));
    }
    // The very top of the world, in the origin column.
    edits.push((BlockPos::new(0, bounds.max_y, 0), &grass));
    // Every content block, in a row in the origin column. Probes like the
    // rest, so each one must survive the save, the reload and the journal.
    for (index, identifier) in content.iter().enumerate() {
        edits.push((content_position(index), identifier));
    }

    let mut probes = Vec::with_capacity(edits.len());
    for (position, identifier) in edits {
        let state: BlockStateId = world.block_id(identifier)?;
        world.set_block(position, state)?;
        probes.push(Probe {
            position,
            expected: identifier.clone(),
        });
    }
    Ok(probes)
}

fn describe(position: BlockPos) -> String {
    format!("{},{},{}", position.x, position.y, position.z)
}

fn mismatch(message: &'static str) -> Error {
    Error::new(Domain::World, "slice-verification", message)
        .with_recovery(Recovery::Manual)
        .fatal()
}

fn log(diagnostics: &Diagnostics, message: &'static str) {
    diagnostics.record(Record::new(
        Level::Info,
        Category::Engine,
        "headless",
        message,
    ));
}

/// Render a report as human-readable lines.
#[must_use]
pub fn format_report(report: &SliceReport) -> String {
    let mut out = String::new();
    out.push_str("NEXORA headless vertical slice\n");
    out.push_str("------------------------------\n");
    out.push_str(&format!("world id           {:#018x}\n", report.world_id));
    out.push_str(&format!("seed               {}\n", report.seed));
    out.push_str(&format!("chunks generated   {}\n", report.chunks_generated));
    out.push_str(&format!("non-air blocks     {}\n", report.non_air_blocks));
    out.push_str(&format!(
        "voxel storage      {} bytes\n",
        report.storage_bytes
    ));
    out.push_str(&format!("blocks edited      {}\n", report.blocks_edited));
    out.push_str(&format!(
        "commands           {} accepted, {} refused\n",
        report.commands_accepted, report.commands_refused
    ));
    out.push_str(&format!(
        "queries            {} answered, {} refused\n",
        report.queries_answered, report.queries_refused
    ));
    out.push_str(&format!(
        "journal            {} edits, {} probes recovered\n",
        report.journal_edits, report.recovered_probes
    ));
    out.push_str(&format!("ticks advanced     {}\n", report.ticks_advanced));
    out.push_str(&format!("save size          {} bytes\n", report.save_bytes));
    out.push_str(&format!(
        "physics bodies     {} ({} settled)\n",
        report.physics_bodies, report.physics_settled
    ));
    out.push_str(&format!(
        "physics substeps   {} ({} contacts)\n",
        report.physics_substeps, report.physics_contacts
    ));
    out.push_str(&format!(
        "character drop     {} cm\n",
        report.physics_drop_cm
    ));
    out.push_str(&format!(
        "streaming ticks    {} ({} generated, {} evicted, {} restored)\n",
        report.streaming_ticks,
        report.streaming_generated,
        report.streaming_evicted,
        report.streaming_restored
    ));
    out.push_str(&format!(
        "chunks retained    {} (peak, edits that cannot be regenerated)\n",
        report.streaming_retained_peak
    ));
    if report.content_blocks > 0 {
        out.push_str(&format!(
            "content blocks     {} ({} surfaces in the mesh)\n",
            report.content_blocks, report.content_surfaces
        ));
    }
    if report.content_textures > 0 {
        out.push_str(&format!(
            "content textures   {} (resolved, verified and decoded)\n",
            report.content_textures
        ));
    }
    out.push_str(&format!(
        "memory             {} pools, worst {}, {} suspected leaks\n",
        report.memory.pools.len(),
        report.memory.worst().as_str(),
        report.memory.suspected_leaks()
    ));
    for pool in &report.memory.pools {
        out.push_str(&format!(
            "                   {:<18} {:<9} {} now, {} peak, {} ceiling\n",
            pool.name,
            pool.class.as_str(),
            pool.current,
            pool.high_water,
            pool.budget.emergency()
        ));
    }
    out.push_str(&format!("probes verified    {}\n", report.probes_verified));
    out.push_str(&format!("lifecycle phases   {}\n", report.phases.len()));
    out.push_str(&format!(
        "                   {}\n",
        report.phases.join(" -> ")
    ));
    out
}

/// What the streaming stage observed.
struct StreamingOutcome {
    ticks: u32,
    generated: u64,
    restored: u64,
    retained_peak: usize,
    evicted: u32,
}

/// Walk an observer away from the origin and back, streaming as it goes.
///
/// This is the `streaming` stage of the vertical slice in
/// `NEXORA TECHNOLOGY BENCHMARK PLAN.md`, and it is the only place the
/// `WORLD CONTINUITY AND PLAYER INDEPENDENCE.md` §18 invariant can actually be
/// tested: a chunk has to be evicted, and the world has to still be right when
/// the observer comes back for it.
///
/// The observer ends where it started, so the resident set at the end is the
/// same one the slice generated - which is what lets the save, reload and probe
/// verification below prove that nothing was lost on the way.
fn stream_a_walk(
    world: &mut World,
    radius: i64,
    memory: &SliceMemory,
    diagnostics: &Diagnostics,
) -> Result<StreamingOutcome> {
    let reach = u32::try_from(radius.max(0)).unwrap_or(0);
    let radii = LodRadii::new(reach, reach, reach, 0)?;
    let mut system = StreamingSystem::new();
    let mut retained = RetainedChunks::new();
    // Chunk memory belongs to the world's pool; streaming decides residency,
    // so it enforces that pool's ceiling rather than keeping a count of its own.
    let budget = StreamingBudget {
        activations: 8,
        evictions: 16,
        max_bytes: memory.world.budget().emergency(),
        ..StreamingBudget::UNLIMITED
    };

    let mut ticks = 0;
    let mut evicted = 0;
    let mut retained_peak = 0;

    // Out four steps, then back to where it started.
    let mut route: Vec<i64> = (0..=4).map(|step| step * (radius + 1)).collect();
    route.extend((0..4).rev().map(|step| step * (radius + 1)));

    for stop in route {
        system.set_interest(
            InterestSource::new(InterestId(1), ChunkCoord::new(stop, 0)).with_radii(radii),
        )?;
        // Settle before moving on, so each leg of the walk completes rather
        // than leaving half-loaded regions behind.
        for _ in 0..256 {
            let mut backend = WorldResidency::new(world, &mut retained);
            let report = system.tick(&mut backend, budget)?;
            ticks += 1;
            evicted += report.evicted;
            retained_peak = retained_peak.max(retained.len());
            memory.world.record(world.storage_bytes() as u64);
            memory.retained.record(retained.storage_bytes() as u64);
            if report.is_quiet() {
                break;
            }
        }
        // A backend refusal is not something the slice should walk past: the
        // point of the stage is that residency worked.
        if let Some(failure) = system.take_failures().into_iter().next() {
            return Err(failure);
        }
    }

    // Anything still held outside the world has to go back before the save, or
    // the save is written without it. See `nexora_simulation::residency`.
    let flushed = retained.flush_into(world);
    memory.retained.record(retained.storage_bytes() as u64);
    memory.world.record(world.storage_bytes() as u64);
    if !retained.is_empty() {
        return Err(Error::new(
            Domain::World,
            "slice/streaming",
            "chunks remained retained after the flush",
        ));
    }
    diagnostics.log(
        Level::Debug,
        Category::World,
        "slice/streaming",
        "retained chunks flushed back into the world",
    );
    diagnostics
        .counters()
        .add("streaming.flushed", flushed as u64);

    Ok(StreamingOutcome {
        ticks,
        generated: retained.generated(),
        restored: retained.restored(),
        retained_peak,
        evicted,
    })
}

/// What the physics stage observed.
struct PhysicsOutcome {
    substeps: u32,
    bodies: usize,
    contacts: u32,
    settled: usize,
    drop_cm: i64,
}

/// Drop a character and a handful of crates onto the generated terrain, and
/// simulate until they settle.
///
/// This is the `physics` stage of the vertical slice in
/// `NEXORA TECHNOLOGY BENCHMARK PLAN.md`. It proves the thing that cannot be
/// proved by a unit test against a flat fixture: that the solver and the real
/// generated world agree about where the ground is.
fn simulate_physics(world: &World, diagnostics: &Diagnostics) -> Result<PhysicsOutcome> {
    let voxels = WorldVoxels::new(world);
    let ticks_per_second = world.clock().calendar().ticks_per_second();
    let mut physics = PhysicsWorld::earthlike(ticks_per_second)?;

    // Spawn above the column's surface so that landing is something the solver
    // has to work out rather than something the spawn asserted.
    let surface = world.surface_height(0, 0);
    let spawn_y = (surface + 12) as f64;
    let character = physics.spawn(BodyDescriptor::character().at(Vec3::new(0.5, spawn_y, 0.5)))?;
    for index in 0..8i64 {
        // Sample the surface of the column the crate actually occupies. Taking
        // a neighbour's height buries the body at spawn, because the generator
        // lets adjacent columns differ by more than the drop clearance.
        let cell_x = index + 2;
        physics.spawn(BodyDescriptor::dynamic().at(Vec3::new(
            cell_x as f64 + 0.5,
            (world.surface_height(cell_x, 3) + 6) as f64,
            3.5,
        )))?;
    }
    let bodies = physics.len();

    let mut substeps = 0;
    let mut contacts = 0;
    // Ten world seconds, in one-tick slices, so the accumulator is exercised
    // the way a running server would exercise it rather than in one bulk call.
    for _ in 0..(ticks_per_second * 10) {
        let report = physics.advance(&voxels, WorldDuration::from_ticks(1));
        substeps += report.substeps;
        contacts += report.stats.contacts;
        if report.stats.stuck > 0 {
            return Err(Error::new(
                Domain::Physics,
                "slice/physics",
                "a body was buried in terrain and could not be freed",
            )
            .with_context("bodies", report.stats.stuck.to_string()));
        }
    }

    let landed = physics.require_body(character)?;
    if !landed.grounded {
        return Err(Error::new(
            Domain::Physics,
            "slice/physics",
            "the character never landed on the generated terrain",
        )
        .with_context("spawned_at", spawn_y.to_string())
        .with_context("ended_at", landed.center.y.to_string()));
    }
    let drop_cm = ((spawn_y - landed.center.y) * 100.0).round() as i64;

    for (id, body) in physics.iter() {
        if overlaps_solid(&voxels, body.aabb()) {
            return Err(Error::new(
                Domain::Physics,
                "slice/physics",
                "a body came to rest inside the terrain",
            )
            .with_context("body", id.index().to_string())
            .with_context(
                "at",
                format!(
                    "{:.3},{:.3},{:.3}",
                    body.center.x, body.center.y, body.center.z
                ),
            ));
        }
    }
    let settled = bodies - physics.awake_count();

    diagnostics.log(
        Level::Debug,
        Category::World,
        "slice/physics",
        "bodies settled on generated terrain",
    );

    Ok(PhysicsOutcome {
        substeps,
        bodies,
        contacts,
        settled,
        drop_cm,
    })
}

/// A world time helper used by the binary's summary output.
#[must_use]
pub fn describe_time(time: WorldTime) -> String {
    format!("tick {}", time.ticks())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slice_refuses_a_broken_ceiling_a_leak_and_an_unused_pool() {
        let over = SliceMemory::open(1, false).unwrap();
        over.world.record(COLUMN_BUDGET[3] + 1);
        over.retained.record(0);
        let err = over.verify().unwrap_err();
        assert!(err.to_string().contains("ceiling"), "{err}");

        let leak = SliceMemory::open(1, false).unwrap();
        leak.world.record(1);
        leak.retained.record(1);
        let err = leak.verify().unwrap_err();
        assert!(err.to_string().contains("drain"), "{err}");

        let unused = SliceMemory::open(1, true).unwrap();
        unused.world.record(1);
        unused.retained.record(0);
        let err = unused.verify().unwrap_err();
        assert!(err.to_string().contains("never recorded"), "{err}");

        let fine = SliceMemory::open(4, false).unwrap();
        fine.world.record(4 * COLUMN_BUDGET[0]);
        fine.retained.record(0);
        assert_eq!(fine.verify().unwrap().worst(), Pressure::Nominal);
    }
}
