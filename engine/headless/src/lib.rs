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
//!   -> advance the world clock
//!   -> save
//!   -> shut down in reverse order
//!   -> reopen, load, and verify the state survived
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

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;

use nexora_foundation::diagnostics::{Category, Diagnostics, Level, Record, StderrSink};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{BlockPos, ChunkCoord};
use nexora_foundation::time::{CalendarConfig, TimeScale, WorldDuration, WorldTime};
use nexora_persistence::SaveContainer;
use nexora_physics::body::BodyDescriptor;
use nexora_physics::collision::overlaps_solid;
use nexora_physics::math::Vec3;
use nexora_physics::world::PhysicsWorld;
use nexora_runtime::jobs::{JobOutcome, JobSystem, Priority};
use nexora_runtime::lifecycle::{Lifecycle, Phase, RuntimeMode};
use nexora_runtime::module::{
    EngineModule, ModuleContext, ModuleDependency, ModuleId, ModuleManager, ModuleSides,
};
use nexora_simulation::{PhysicsModule, WorldVoxels};
use nexora_world::persist;
use nexora_world::voxel::BlockStateId;
use nexora_world::world::{World, WorldDescriptor};

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
}

impl Default for SliceConfig {
    fn default() -> Self {
        Self {
            seed: 0x4E45_584F_5241,
            radius: 2,
            save_path: PathBuf::from("run/world.nxsv"),
            worker_threads: 4,
            verbose: true,
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
    /// Probes checked after reloading.
    pub probes_verified: usize,
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
    let descriptor = WorldDescriptor::new("nexora-slice", config.seed)?;
    let world_id = descriptor.id.0;
    let mut world = World::create(descriptor, CalendarConfig::earthlike())?;
    lifecycle.advance_to(Phase::WorldAttach)?;
    world.bring_online()?;
    lifecycle.advance_to(Phase::SimulationRunning)?;
    log(&diagnostics, "world attached and simulating");

    // --- generation through the job system ---------------------------------
    let coords: Vec<ChunkCoord> = (-config.radius..=config.radius)
        .flat_map(|x| (-config.radius..=config.radius).map(move |z| ChunkCoord::new(x, z)))
        .collect();
    let chunks_generated = coords.len();

    let generated = generate_in_parallel(world, &coords, config.worker_threads, &diagnostics)?;
    world = generated;
    diagnostics
        .counters()
        .add("chunks.generated", chunks_generated as u64);
    log(&diagnostics, "chunks generated");

    // --- mutation ----------------------------------------------------------
    let probes = apply_edits(&mut world, &coords)?;
    let blocks_edited = probes.len();
    diagnostics
        .counters()
        .add("blocks.edited", blocks_edited as u64);

    // --- physics -----------------------------------------------------------
    // Runs against the world but never writes to it, so the save below - and
    // the byte-identical determinism check over it - is unaffected.
    let physics = simulate_physics(&world, &diagnostics)?;
    diagnostics
        .counters()
        .add("physics.substeps", u64::from(physics.substeps));
    log(&diagnostics, "physics settled");

    // --- time --------------------------------------------------------------
    let before = world.clock().now();
    world.clock_mut().advance_by(TimeScale::Hour, 7)?;
    let ticks_advanced = world.clock().now().since(before).ticks();

    let storage_bytes = world.storage_bytes();
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
    log(&diagnostics, "world saved");

    // --- shutdown ----------------------------------------------------------
    lifecycle.advance_through(Phase::WorldFlush)?;
    modules.shutdown_all(&module_context);
    lifecycle.advance_through(Phase::ProcessExit)?;
    drop(world);
    log(&diagnostics, "runtime shut down");

    // --- reopen and verify -------------------------------------------------
    let reopened = SaveContainer::read(&config.save_path)?;
    let restored = persist::load(&reopened)?;

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

    for probe in &probes {
        let found = restored.get_block(probe.position)?;
        let found_id = restored.block_identifier(found).ok_or_else(|| {
            mismatch("a restored block state has no registered identifier")
                .with_context("position", describe(probe.position))
        })?;
        if found_id != probe.expected {
            return Err(mismatch("a block changed across the save/load boundary")
                .with_context("position", describe(probe.position))
                .with_context("expected", probe.expected.to_string())
                .with_context("found", found_id.to_string()));
        }
    }
    log(&diagnostics, "reload verified");

    Ok(SliceReport {
        world_id,
        seed: config.seed,
        chunks_generated,
        blocks_edited,
        ticks_advanced,
        save_bytes,
        storage_bytes,
        non_air_blocks,
        physics_substeps: physics.substeps,
        physics_bodies: physics.bodies,
        physics_contacts: physics.contacts,
        physics_settled: physics.settled,
        physics_drop_cm: physics.drop_cm,
        probes_verified: probes.len(),
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
fn apply_edits(world: &mut World, coords: &[ChunkCoord]) -> Result<Vec<Probe>> {
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
    out.push_str(&format!("probes verified    {}\n", report.probes_verified));
    out.push_str(&format!("lifecycle phases   {}\n", report.phases.len()));
    out.push_str(&format!(
        "                   {}\n",
        report.phases.join(" -> ")
    ));
    out
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
