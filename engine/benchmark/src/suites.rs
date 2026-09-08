//! The individual measurements.
//!
//! Each suite maps to a stage of the vertical slice in
//! `NEXORA TECHNOLOGY BENCHMARK PLAN.md`. Stages Phase 0 has no implementation
//! for are returned by [`unmeasured_stages`] instead of being skipped.

use std::path::Path;

use nexora_entity::components::{Bounds, TagSet, Velocity};
use nexora_entity::id::EntityTypeId;
use nexora_entity::persist as entity_persist;
use nexora_entity::query::{EntityFilter, Query};
use nexora_entity::store::{EntityStore, SpawnContext, SpawnReason};
use nexora_foundation::error::Result;
use nexora_foundation::hashing::crc32;
use nexora_foundation::ident::Identifier;
use nexora_foundation::ident::WorldId;
use nexora_foundation::rng::Rng;
use nexora_foundation::spatial::WorldPosition;
use nexora_foundation::spatial::{BlockPos, ChunkCoord, ChunkShape, SectionCoord};
use nexora_foundation::time::CalendarConfig;
use nexora_foundation::time::WorldDuration;
use nexora_persistence::journal::{self, Journal, SnapshotId};
use nexora_persistence::SaveContainer;
use nexora_physics::body::BodyDescriptor;
use nexora_physics::character::{CharacterController, MoveIntent};
use nexora_physics::collision::{depenetrate, resolve, sweep_axis};
use nexora_physics::gravity::GravityField;
use nexora_physics::math::{Aabb, Axis, Vec3};
use nexora_physics::query::raycast;
use nexora_physics::step::FixedStep;
use nexora_physics::voxel::FlatGround;
use nexora_physics::world::PhysicsWorld;
use nexora_runtime::jobs::{JobSystem, Priority};
use nexora_simulation::{RetainedChunks, WorldResidency, WorldVoxels};
use nexora_streaming::backend::{MemoryBackend, ResidencyBackend};
use nexora_streaming::budget::StreamingBudget;
use nexora_streaming::interest::{InterestId, InterestSource, LodRadii};
use nexora_streaming::lod::Lod;
use nexora_streaming::system::StreamingSystem;
use nexora_streaming::target::StreamTarget;
use nexora_world::persist;
use nexora_world::voxel::{BlockStateId, Section, AIR};
use nexora_world::world::{World, WorldDescriptor};

use crate::conformance::SweepFixture;
use crate::{
    consume, measure, measure_throughput, record_bytes, record_quantity, Budget, Measurement,
    Unmeasured,
};

/// Seed used for every world the benchmark builds, so runs are comparable.
const BENCH_SEED: u64 = 0x0BEEF_0BEEF;

/// Chunk radius generated for the world-level measurements.
const BENCH_RADIUS: i64 = 1;

fn bench_world(shape_size: u32) -> Result<World> {
    let mut descriptor = WorldDescriptor::new("benchmark", BENCH_SEED)?;
    descriptor.shape = ChunkShape::new(shape_size, shape_size, shape_size)?;
    let mut world = World::create(descriptor, CalendarConfig::earthlike())?;
    world.bring_online()?;
    Ok(world)
}

fn populated_world() -> Result<World> {
    let mut world = bench_world(32)?;
    for x in -BENCH_RADIUS..=BENCH_RADIUS {
        for z in -BENCH_RADIUS..=BENCH_RADIUS {
            world.load_or_generate(ChunkCoord::new(x, z))?;
        }
    }
    Ok(world)
}

/// World generation: the "16³ voxel chunk" stage of the plan.
///
/// Measured at the plan's 16³ and at the engine's default 32³, because the cost
/// is not linear in the cell count — the vertical range is fixed, so a smaller
/// section size means more sections per column.
///
/// # Errors
///
/// Returns an error when a benchmark world cannot be built.
pub fn worldgen(budget: Budget) -> Result<Vec<Measurement>> {
    let mut out = Vec::new();

    for (size, name, note) in [
        (
            16u32,
            "worldgen.chunk_16",
            "Generate one chunk column, 16³ sections (the plan's size)",
        ),
        (
            32u32,
            "worldgen.chunk_32",
            "Generate one chunk column, 32³ sections (engine default)",
        ),
    ] {
        let world = bench_world(size)?;
        let mut next = 0i64;
        out.push(measure(name, note, budget, || {
            // Walk fresh coordinates so no run reuses a warm cache line from the
            // previous one; generation is position-seeded, so this is fair.
            next += 1;
            let chunk = world
                .generate_chunk(ChunkCoord::new(next, next))
                .expect("generation");
            consume(chunk.non_air_count());
        }));
    }

    let world = bench_world(32)?;
    out.push(measure(
        "worldgen.surface_height",
        "One column's terrain height from the position-seeded stream",
        Budget {
            iterations_per_sample: 10_000,
            ..budget
        },
        || {
            consume(world.surface_height(consume(12_345), consume(-6_789)));
        },
    ));

    Ok(out)
}

/// Voxel storage: the hot path every other world system sits on.
///
/// # Errors
///
/// Returns an error when a benchmark world cannot be built.
pub fn voxel(budget: Budget) -> Result<Vec<Measurement>> {
    let shape = ChunkShape::cubic_default();
    let stone = BlockStateId(1);
    let dirt = BlockStateId(2);
    let mut out = Vec::new();

    // A paletted section, which is the representation a live chunk spends most
    // of its time in.
    let mut paletted = Section::empty(shape);
    for index in 0..64u32 {
        paletted
            .set(
                nexora_foundation::spatial::LocalPos::new(index % 32, 0, index / 32),
                stone,
            )
            .expect("seed the palette");
    }

    let read_at = nexora_foundation::spatial::LocalPos::new(5, 0, 0);
    out.push(measure(
        "voxel.get_paletted",
        "Read one cell from a palette-indexed section",
        Budget {
            iterations_per_sample: 20_000,
            ..budget
        },
        || {
            consume(paletted.get(consume(read_at)).expect("in bounds"));
        },
    ));

    let uniform = Section::uniform(shape, stone);
    out.push(measure(
        "voxel.get_uniform",
        "Read one cell from a uniform section (solid rock, the common case)",
        Budget {
            iterations_per_sample: 20_000,
            ..budget
        },
        || {
            consume(uniform.get(consume(read_at)).expect("in bounds"));
        },
    ));

    let mut writable = paletted.clone();
    let mut toggle = false;
    out.push(measure(
        "voxel.set_existing_state",
        "Write a cell whose state is already in the palette",
        Budget {
            iterations_per_sample: 20_000,
            ..budget
        },
        || {
            toggle = !toggle;
            let state = if toggle { stone } else { dirt };
            consume(writable.set(read_at, state).expect("in bounds"));
        },
    ));

    out.push(measure(
        "voxel.first_write_to_uniform",
        "Break a uniform section into a palette: the allocation the air optimization defers",
        Budget {
            iterations_per_sample: 200,
            ..budget
        },
        || {
            let mut section = Section::uniform(shape, AIR);
            consume(section.set(read_at, stone).expect("in bounds"));
            consume(section.non_air_count());
        },
    ));

    out.push(measure(
        "voxel.compact_section",
        "Rebuild a section's palette from what is actually referenced",
        Budget {
            iterations_per_sample: 100,
            ..budget
        },
        || {
            let mut section = paletted.clone();
            section.compact();
            consume(section.storage_bytes());
        },
    ));

    Ok(out)
}

/// Coordinate conversion, and the specific question `DEBT-0005` records.
///
/// `ChunkShape` is a runtime value because `CHUNK & VOXEL ENGINE.md` §3 forbids
/// assuming a fixed chunk size, which means `section_of` performs a real
/// integer division. The debt register asks whether that costs anything. For a
/// power-of-two extent an arithmetic shift is exactly equivalent — including
/// for negative coordinates — so both are measured here and the difference is
/// the answer.
#[must_use]
pub fn spatial(budget: Budget) -> Vec<Measurement> {
    let shape = ChunkShape::cubic_default();
    let bits = 5u32; // 32 == 1 << 5
    let position = BlockPos::new(-1_234_567, 91, 7_654_321);

    // Two shift variants, so the shortcut gets its best case as well as its
    // realistic one. A `ChunkShape` is a runtime value, so a real implementation
    // would have to read the width from it (variable shift); the const version
    // is the ceiling the optimization could ever reach.
    let shift_variable = |pos: BlockPos| -> SectionCoord {
        // Arithmetic shift right is floor division by a power of two, so this
        // agrees with div_euclid on negative coordinates too.
        SectionCoord::new(pos.x >> bits, pos.y >> bits, pos.z >> bits)
    };
    const CONST_BITS: u32 = 5;
    let shift_const = |pos: BlockPos| -> SectionCoord {
        SectionCoord::new(
            pos.x >> CONST_BITS,
            pos.y >> CONST_BITS,
            pos.z >> CONST_BITS,
        )
    };

    vec![
        measure(
            "spatial.section_of_division",
            "Block → section using div_euclid by a runtime extent (what the engine does)",
            Budget {
                iterations_per_sample: 50_000,
                ..budget
            },
            || {
                consume(shape.section_of(consume(position)));
            },
        ),
        measure(
            "spatial.section_of_shift_runtime",
            "Block → section by shift, width read at runtime: what DEBT-0005 would actually build",
            Budget {
                iterations_per_sample: 50_000,
                ..budget
            },
            || {
                consume(shift_variable(consume(position)));
            },
        ),
        measure(
            "spatial.section_of_shift_const",
            "Block → section by shift with a compile-time width: the optimization's ceiling",
            Budget {
                iterations_per_sample: 50_000,
                ..budget
            },
            || {
                consume(shift_const(consume(position)));
            },
        ),
        measure(
            "spatial.index_of",
            "Flatten a local position into a storage index, bounds-checked",
            Budget {
                iterations_per_sample: 50_000,
                ..budget
            },
            || {
                consume(
                    shape
                        .index_of(consume(nexora_foundation::spatial::LocalPos::new(
                            7, 11, 13,
                        )))
                        .expect("in bounds"),
                );
            },
        ),
    ]
}

/// The job system: the plan's "jobs" stage.
///
/// # Errors
///
/// Returns an error when the worker pool cannot be created.
pub fn jobs(budget: Budget) -> Result<Vec<Measurement>> {
    let workers = std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
        .min(8);
    let pool = JobSystem::new(workers)?;
    let mut out = Vec::new();

    out.push(measure(
        "jobs.submit_wait_roundtrip",
        "Submit one trivial job and block until it completes: the full scheduling round trip",
        Budget {
            iterations_per_sample: 200,
            ..budget
        },
        || {
            let handle = pool.submit(Priority::Normal, |_| Ok(()));
            consume(pool.wait(handle));
        },
    ));

    out.push(measure(
        "jobs.submit_only",
        "Enqueue a job without waiting: the cost a producer pays",
        Budget {
            iterations_per_sample: 1_000,
            ..budget
        },
        || {
            consume(pool.submit(Priority::Background, |_| Ok(())));
        },
    ));
    pool.barrier();

    const BATCH: u32 = 1_000;
    out.push(measure(
        "jobs.batch_1000_barrier",
        "Submit 1,000 jobs and barrier: throughput across the whole pool",
        Budget {
            iterations_per_sample: 1,
            ..budget
        },
        || {
            for _ in 0..BATCH {
                pool.submit(Priority::Normal, |_| Ok(()));
            }
            pool.barrier();
        },
    ));

    Ok(out)
}

/// Serialization and the save path: the plan's "save/load" stage.
///
/// # Errors
///
/// Returns an error when the benchmark world cannot be built or saved.
pub fn persistence(budget: Budget, scratch: &Path) -> Result<Vec<Measurement>> {
    let world = populated_world()?;
    let container = persist::save(&world)?;
    let encoded = container.encode();
    let size = encoded.len() as u64;
    let mut out = Vec::new();

    // Mirrors `save.crc32_64kib` in the C++ reference, down to the fixture: the
    // same 64 KiB drawn from the same seeded stream, so the pair compares two
    // implementations of one algorithm over one input.
    const CHECKSUM_PAYLOAD_BYTES: usize = 64 * 1024;
    const CHECKSUM_PAYLOAD_SEED: u64 = 11;
    let mut driver = Rng::from_seed(CHECKSUM_PAYLOAD_SEED);
    let payload: Vec<u8> = (0..CHECKSUM_PAYLOAD_BYTES)
        .map(|_| driver.next_u64() as u8)
        .collect();
    out.push(measure(
        "save.crc32_64kib",
        "CRC-32 over 64 KiB, the per-section integrity check (paired with the C++ reference)",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        || {
            consume(crc32(&payload));
        },
    ));

    out.push(measure_throughput(
        "save.encode",
        "Serialize the whole world to bytes, including per-section checksums",
        Budget {
            iterations_per_sample: 5,
            ..budget
        },
        size,
        || {
            consume(container.encode().len());
        },
    ));

    out.push(measure_throughput(
        "save.decode",
        "Parse and verify every checksum on the way back in",
        Budget {
            iterations_per_sample: 5,
            ..budget
        },
        size,
        || {
            consume(
                SaveContainer::decode(&encoded)
                    .expect("valid container")
                    .len(),
            );
        },
    ));

    out.push(measure(
        "save.world_load",
        "Rebuild the world from a container, remapping every block through its identifier",
        Budget {
            iterations_per_sample: 3,
            ..budget
        },
        || {
            let restored = persist::load(&container).expect("load");
            consume(restored.chunk_count());
        },
    ));

    // The read-back-and-verify cost recorded as DEBT-0006 is inside this number.
    let path = scratch.join("benchmark-world.nxsv");
    out.push(measure_throughput(
        "save.write_atomic_disk",
        "Write to disk atomically: temp file, fsync, decode to verify, rename (DEBT-0006)",
        Budget {
            iterations_per_sample: 1,
            ..budget
        },
        size,
        || {
            container.write_atomic(&path).expect("write");
        },
    ));

    // --- journalling ---------------------------------------------------------
    // `DEBT-0025` will not choose a durability policy without these. The
    // question the numbers have to answer: can the engine afford to make every
    // edit durable as it happens, or does durability have to be batched -- and
    // if batched, what does a crash then cost?
    {
        /// A representative edit record: a tag, three coordinates and a block
        /// identifier. Sized from what `EditRecord::SetBlock` actually encodes
        /// rather than from a round number.
        const EDIT_RECORD_BYTES: usize = 1 + 8 + 8 + 8 + 4 + "nexora:block/stone".len();

        /// How many records one batched sync covers.
        const BATCH_RECORDS: u32 = 64;

        let record = vec![0xA5u8; EDIT_RECORD_BYTES];
        let base = SnapshotId::of(0xBEEF, b"benchmark snapshot");

        let append_path = scratch.join("bench-append.nxjr");
        let mut appending = Journal::create(&append_path, base)?;
        out.push(measure(
            "journal.append_unsynced",
            "Frame and checksum one edit record, without making it durable",
            Budget {
                iterations_per_sample: 2_000,
                ..budget
            },
            || {
                appending.append(&record).expect("append");
            },
        ));
        drop(appending);

        let durable_path = scratch.join("bench-durable.nxjr");
        let mut durable = Journal::create(&durable_path, base)?;
        out.push(measure(
            "journal.append_durable",
            "One edit record, fsynced before returning: durability per edit",
            Budget {
                iterations_per_sample: 20,
                ..budget
            },
            || {
                durable.append_durable(&record).expect("append");
            },
        ));
        drop(durable);

        // The middle ground: many appends, one flush. What a tick-scoped or
        // batch-scoped commit would actually pay per edit.
        let batched_path = scratch.join("bench-batched.nxjr");
        let mut batched = Journal::create(&batched_path, base)?;
        out.push(measure(
            "journal.append_batched_sync",
            "One edit record when 64 share a single fsync",
            Budget {
                iterations_per_sample: 20,
                ..budget
            },
            || {
                for _ in 0..BATCH_RECORDS {
                    batched.append(&record).expect("append");
                }
                batched.sync().expect("sync");
            },
        ));
        drop(batched);

        // Replay cost decides whether a long journal is a problem on load.
        let replay_path = scratch.join("bench-replay.nxjr");
        let mut writing = Journal::create(&replay_path, base)?;
        for _ in 0..10_000 {
            writing.append(&record).expect("append");
        }
        writing.sync().expect("sync");
        drop(writing);

        out.push(measure(
            "journal.replay_10k_records",
            "Read back and verify a journal of 10,000 edit records",
            Budget {
                iterations_per_sample: 1,
                ..budget
            },
            || {
                consume(journal::replay(&replay_path, base).expect("replay").len());
            },
        ));

        out.push(record_bytes(
            "journal.record_bytes",
            "Encoded size of one block edit, framing included",
            (EDIT_RECORD_BYTES + 8) as u64,
        ));
    }

    out.push(measure_throughput(
        "save.read_disk",
        "Read and verify a save from disk",
        Budget {
            iterations_per_sample: 1,
            ..budget
        },
        size,
        || {
            consume(SaveContainer::read(&path).expect("read").len());
        },
    ));

    out.push(record_bytes(
        "save.size_9_chunks",
        "Bytes on disk for a 3×3 chunk world with the full vertical range",
        size,
    ));
    out.push(record_bytes(
        "world.voxel_storage_9_chunks",
        "In-memory voxel storage for the same world",
        world.storage_bytes() as u64,
    ));
    out.push(record_quantity(
        "world.non_air_blocks_9_chunks",
        "Non-air blocks that storage represents",
        world
            .loaded_chunks()
            .iter()
            .filter_map(|c| world.chunk(*c))
            .map(|c| c.non_air_count())
            .sum(),
    ));

    Ok(out)
}

/// Entities: the plan's "1,000 entities" stage.
///
/// The population size is the one `NEXORA TECHNOLOGY BENCHMARK PLAN.md` names,
/// so these numbers slot straight into the gate.
///
/// # Errors
///
/// Returns an error when a population cannot be built.
pub fn entities(budget: Budget) -> Result<Vec<Measurement>> {
    const POPULATION: usize = 1_000;
    const TICKS_PER_SECOND: u32 = 20;

    let world = WorldId::derive("benchmark", BENCH_SEED);
    let entity_type = EntityTypeId::parse("nexora:entity/bench")?;
    let living = Identifier::parse("nexora:tag/living")?;

    let populate = |count: usize| -> Result<EntityStore> {
        let mut store = EntityStore::new(world);
        for index in 0..count {
            let position = WorldPosition::new(
                (index % 64) as f64,
                64.0 + (index / 64) as f64,
                (index % 37) as f64,
            );
            store.spawn(
                SpawnContext::new(entity_type.clone(), position, SpawnReason::WorldGen)
                    .with_velocity(Velocity::new(0.5, 0.0, -0.25)?)
                    .with_bounds(Bounds::new(0.4, 0.9, 0.4)?)
                    .with_tags(TagSet::from_iter_sorted([living.clone()])),
            )?;
        }
        Ok(store)
    };

    let mut out = Vec::new();

    let mut spawn_store = EntityStore::new(world);
    let spawn_type = entity_type.clone();
    out.push(measure(
        "entity.spawn",
        "Create one entity: slot allocation, component writes, persistent id",
        Budget {
            iterations_per_sample: 500,
            ..budget
        },
        || {
            let id = spawn_store
                .spawn(SpawnContext::new(
                    spawn_type.clone(),
                    WorldPosition::ORIGIN,
                    SpawnReason::Spawner,
                ))
                .expect("spawn");
            consume(id);
        },
    ));

    let mut churn = populate(POPULATION)?;
    let churn_type = entity_type.clone();
    out.push(measure(
        "entity.spawn_despawn_cycle",
        "Spawn then destroy: the slot reuse path a busy world runs constantly",
        Budget {
            iterations_per_sample: 500,
            ..budget
        },
        || {
            let id = churn
                .spawn(SpawnContext::new(
                    churn_type.clone(),
                    WorldPosition::ORIGIN,
                    SpawnReason::Spawner,
                ))
                .expect("spawn");
            churn.despawn(id).expect("despawn");
        },
    ));

    let resolve_store = populate(POPULATION)?;
    let handle = resolve_store
        .iter()
        .nth(POPULATION / 2)
        .expect("a live entity");
    out.push(measure(
        "entity.resolve_handle",
        "Validate one handle: world, range, generation and liveness",
        Budget {
            iterations_per_sample: 20_000,
            ..budget
        },
        || {
            consume(resolve_store.resolve(consume(handle)).expect("resolves"));
        },
    ));

    let mut stepping = populate(POPULATION)?;
    out.push(measure(
        "entity.step_1000",
        "Advance 1,000 entities by one tick: the batch walk over dense columns",
        Budget {
            iterations_per_sample: 100,
            ..budget
        },
        || {
            consume(
                stepping
                    .step(WorldDuration::from_ticks(1), TICKS_PER_SECOND)
                    .expect("step"),
            );
        },
    ));

    let queried = populate(POPULATION)?;
    let query_type = entity_type.clone();
    out.push(measure(
        "entity.query_type_1000",
        "Linear scan of 1,000 entities filtering by type (no spatial index yet)",
        Budget {
            iterations_per_sample: 200,
            ..budget
        },
        || {
            consume(Query::by_type(&queried, query_type.clone()).len());
        },
    ));

    let tag_for_query = living.clone();
    out.push(measure(
        "entity.query_tag_1000",
        "Linear scan of 1,000 entities filtering by tag",
        Budget {
            iterations_per_sample: 200,
            ..budget
        },
        || {
            consume(Query::by_tag(&queried, tag_for_query.clone()).len());
        },
    ));

    out.push(measure(
        "entity.query_radius_1000",
        "Nearest-first radius query over 1,000 entities, including the sort",
        Budget {
            iterations_per_sample: 100,
            ..budget
        },
        || {
            consume(
                Query::within_radius(
                    &queried,
                    WorldPosition::new(32.0, 70.0, 18.0),
                    16.0,
                    &EntityFilter::new(),
                )
                .len(),
            );
        },
    ));

    let to_save = populate(POPULATION)?;
    let mut sizing = SaveContainer::new();
    entity_persist::save(&to_save, &mut sizing)?;
    let section_bytes = sizing
        .get(&Identifier::parse(entity_persist::SECTION_ENTITIES)?)
        .map_or(0, <[u8]>::len) as u64;

    out.push(measure_throughput(
        "entity.save_1000",
        "Serialize 1,000 entities as logical state",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        section_bytes,
        || {
            let mut container = SaveContainer::new();
            entity_persist::save(&to_save, &mut container).expect("save");
            consume(container.len());
        },
    ));

    out.push(measure_throughput(
        "entity.load_1000",
        "Restore 1,000 entities, re-resolving every persistent id",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        section_bytes,
        || {
            let mut restored = EntityStore::new(world);
            entity_persist::load(&mut restored, &sizing).expect("load");
            consume(restored.len());
        },
    ));

    out.push(record_bytes(
        "entity.save_size_1000",
        "Bytes of save section for 1,000 entities",
        section_bytes,
    ));

    Ok(out)
}

/// Cold start: process ready to a simulating world.
///
/// # Errors
///
/// Returns an error when a world cannot be created.
pub fn startup(budget: Budget) -> Result<Vec<Measurement>> {
    let air = Identifier::parse("nexora:block/air")?;
    Ok(vec![measure(
        "startup.world_create",
        "Create a world, register the block set, freeze the registry and bring it online",
        Budget {
            iterations_per_sample: 100,
            ..budget
        },
        || {
            let mut world = bench_world(32).expect("world");
            consume(world.block_id(&air).expect("air is registered"));
            world.bring_online().ok();
        },
    )])
}

/// Physics: the "physics" stage of the plan's vertical slice.
///
/// Measured against **generated terrain**, not a flat fixture. A flat floor
/// answers every collision query out of one uniform section, which would make
/// the voxel lookup look free — the opposite of what the gate needs to know.
/// The flat case is measured too, as a deliberate pair: the difference between
/// them is the cost of the world lookup, and without both numbers a slow result
/// cannot be attributed to either half.
///
/// # Errors
///
/// Returns an error when the benchmark world or a physics world cannot be built.
pub fn physics(budget: Budget) -> Result<Vec<Measurement>> {
    const POPULATION: usize = 1_000;
    const TICKS_PER_SECOND: u32 = 20;
    const SUBSTEP: f64 = 1.0 / 60.0;

    let world = populated_world()?;
    let voxels = WorldVoxels::new(&world);
    let surface = world.surface_height(4, 4);
    let flat = FlatGround::at(0);
    // Large enough to hold every body the suite spawns, so that re-waking is
    // not itself measuring a region test that missed.
    let everywhere = Aabb::new(Vec3::splat(-4_000.0), Vec3::splat(4_000.0))
        .expect("a region large enough for the whole population");

    let mut out = Vec::new();

    // One character: the case that carries the whole control path — gravity,
    // sweeps on three axes, step-up, the ground probe and friction.
    let mut single = PhysicsWorld::earthlike(TICKS_PER_SECOND)?;
    let character =
        single.spawn(BodyDescriptor::character().at(Vec3::new(4.5, (surface + 3) as f64, 4.5)))?;
    let controller = CharacterController::new(character);
    out.push(measure(
        "physics.character_step",
        "One character substep on generated terrain: gravity, sweep, ground, friction",
        Budget {
            iterations_per_sample: 2_000,
            ..budget
        },
        || {
            controller
                .apply(&mut single, MoveIntent::walking(Vec3::new(1.0, 0.0, 0.3)))
                .expect("live body");
            consume(single.step_once(&voxels, SUBSTEP).contacts);
        },
    ));

    // A thousand bodies: the plan's own stress figure. Sleeping is deliberately
    // defeated by re-waking them, so this is the cost of simulating a thousand
    // bodies rather than the cost of skipping them.
    let mut crowd = PhysicsWorld::earthlike(TICKS_PER_SECOND)?;
    for index in 0..POPULATION {
        let x = (index % 30) as i64;
        let z = (index / 30) as i64;
        crowd.spawn(BodyDescriptor::dynamic().at(Vec3::new(
            x as f64 + 0.5,
            (world.surface_height(x, z) + 4) as f64,
            z as f64 + 0.5,
        )))?;
    }
    out.push(measure(
        "physics.thousand_bodies_step",
        "One substep of 1,000 awake dynamic bodies against generated terrain",
        Budget {
            iterations_per_sample: 4,
            ..budget
        },
        || {
            crowd.wake_in(everywhere);
            consume(crowd.step_once(&voxels, SUBSTEP).simulated);
        },
    ));

    let mut flat_crowd = PhysicsWorld::earthlike(TICKS_PER_SECOND)?;
    for index in 0..POPULATION {
        flat_crowd.spawn(BodyDescriptor::dynamic().at(Vec3::new(
            (index % 30) as f64 + 0.5,
            4.0 + (index / 30) as f64,
            (index / 30) as f64 + 0.5,
        )))?;
    }
    out.push(measure(
        "physics.thousand_bodies_step_flat",
        "The same substep against a flat fixture: the solver without the world lookup",
        Budget {
            iterations_per_sample: 4,
            ..budget
        },
        || {
            flat_crowd.wake_in(everywhere);
            consume(flat_crowd.step_once(&flat, SUBSTEP).simulated);
        },
    ));

    // Mirrors `physics.axis_sweep` in the C++ reference: the same fixture, the
    // same box, the same axis and delta. The comparison table needs matched
    // pairs, and "roughly the same kernel" is how two different problems get
    // reported as a language difference.
    let sweep_box = Aabb::new(Vec3::new(0.2, 3.0, 0.2), Vec3::new(0.8, 4.8, 0.8))
        .expect("min is below max on every axis");
    out.push(measure(
        "physics.axis_sweep",
        "One axis of a swept box against the voxel grid (paired with the C++ reference)",
        Budget {
            iterations_per_sample: 20_000,
            ..budget
        },
        || {
            // `delta` goes through `consume`, and only `delta`. The call is pure
            // with constant arguments, so with no barrier at all LLVM hoists the
            // whole sweep out of the timing loop -- it did, reporting 2.4 ns for
            // a kernel that costs an order of magnitude more. One opaque scalar
            // the result depends on is enough to stop that, and unlike hiding
            // the `Aabb` it does not force a 48-byte round trip through memory
            // that the C++ side would not be paying. The C++ reference applies
            // the same barrier to the same argument.
            consume(sweep_axis(&SweepFixture, sweep_box, Axis::Y, consume(-0.16)).allowed);
        },
    ));

    // The primitive underneath all of it.
    let body_box = Aabb::from_center(
        Vec3::new(4.5, (surface + 3) as f64, 4.5),
        Vec3::new(0.3, 0.9, 0.3),
    )
    .expect("valid body box");
    out.push(measure(
        "physics.box_sweep",
        "Resolve one box move on three axes against generated terrain",
        Budget {
            iterations_per_sample: 5_000,
            ..budget
        },
        || {
            consume(
                resolve(&voxels, body_box, Vec3::new(0.07, -0.16, 0.07))
                    .applied
                    .y,
            );
        },
    ));

    out.push(measure(
        "physics.depenetration_check",
        "The per-body test for having started inside terrain, when it has not",
        Budget {
            iterations_per_sample: 10_000,
            ..budget
        },
        || {
            consume(depenetrate(&voxels, body_box).is_some());
        },
    ));

    // The query every tool, cursor and AI obstacle check runs.
    let ray_origin = Vec3::new(4.5, (surface + 40) as f64, 4.5);
    out.push(measure(
        "physics.raycast_40m",
        "Walk a 40 m ray down through generated terrain until it hits",
        Budget {
            iterations_per_sample: 5_000,
            ..budget
        },
        || {
            consume(
                raycast(&voxels, ray_origin, Vec3::new(0.0, -1.0, 0.0), 60.0).map(|hit| hit.cell.y),
            );
        },
    ));

    // The accumulator every one of the above runs behind.
    let mut accumulator = FixedStep::per_second(TICKS_PER_SECOND)?;
    out.push(measure(
        "physics.timestep_accumulate",
        "Fold one world tick into the fixed-step accumulator",
        Budget {
            iterations_per_sample: 20_000,
            ..budget
        },
        || {
            consume(
                accumulator
                    .accumulate(WorldDuration::from_ticks(1))
                    .substeps,
            );
        },
    ));

    // What sleeping is worth: the same population, settled.
    let mut settled = PhysicsWorld::new(
        GravityField::earthlike(),
        FixedStep::per_second(TICKS_PER_SECOND)?,
    );
    for index in 0..POPULATION {
        settled.spawn(BodyDescriptor::dynamic().at(Vec3::new(
            (index % 30) as f64 + 0.5,
            4.0,
            (index / 30) as f64 + 0.5,
        )))?;
    }
    for _ in 0..600 {
        settled.step_once(&flat, SUBSTEP);
    }
    out.push(record_quantity(
        "physics.sleeping_bodies_of_1000",
        "Bodies asleep after ten seconds: what sleeping actually saves",
        (settled.len() - settled.awake_count()) as u64,
    ));
    out.push(measure(
        "physics.thousand_sleeping_step",
        "One substep with the same 1,000 bodies asleep",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        || {
            consume(settled.step_once(&flat, SUBSTEP).simulated);
        },
    ));

    Ok(out)
}

/// Streaming: the "streaming" stage of the plan's vertical slice.
///
/// Measured as a **pair**, the same way physics is: the manager deciding what
/// should be resident, against the world actually making it so. Deciding runs
/// every tick whether or not anything moves; making it so runs only when
/// something changes. A single combined number would hide which of the two a
/// budget has to be sized around.
///
/// # Errors
///
/// Returns an error when a benchmark world or streaming configuration cannot be
/// built.
pub fn streaming(budget: Budget) -> Result<Vec<Measurement>> {
    const TILE: u64 = 1_024;

    let radii = |reach: u32| LodRadii::new(reach, reach, reach, 0);
    let settled = |reach: u32| -> Result<(StreamingSystem, MemoryBackend)> {
        let mut system = StreamingSystem::new();
        let mut backend = MemoryBackend::new(TILE);
        system.set_interest(
            InterestSource::new(InterestId(1), ChunkCoord::new(0, 0)).with_radii(radii(reach)?),
        )?;
        for _ in 0..64 {
            if system
                .tick(&mut backend, StreamingBudget::UNLIMITED)?
                .is_quiet()
            {
                break;
            }
        }
        Ok((system, backend))
    };

    let mut out = Vec::new();

    // The floor: what a tick costs when nothing has moved. This runs every
    // frame of every session, so it is the number a frame budget cares about.
    let (mut idle, mut idle_backend) = settled(3)?;
    out.push(measure(
        "streaming.idle_tick_r3",
        "A tick with nothing to do, 7x7 columns of interest: pure decision cost",
        Budget {
            iterations_per_sample: 2_000,
            ..budget
        },
        || {
            consume(
                idle.tick(&mut idle_backend, StreamingBudget::UNLIMITED)
                    .expect("valid budget")
                    .resident,
            );
        },
    ));

    // The same tick over a much larger interest area. Candidate enumeration is
    // the square of the radius, and this is where that stops being free.
    let (mut wide, mut wide_backend) = settled(12)?;
    out.push(measure(
        "streaming.idle_tick_r12",
        "The same idle tick over 25x25 columns: how decision cost scales with radius",
        Budget {
            iterations_per_sample: 200,
            ..budget
        },
        || {
            consume(
                wide.tick(&mut wide_backend, StreamingBudget::UNLIMITED)
                    .expect("valid budget")
                    .resident,
            );
        },
    ));

    // Movement: one chunk of travel per tick, which is what a sprinting player
    // actually produces.
    let (mut walking, mut walk_backend) = settled(3)?;
    let mut step = 0i64;
    out.push(measure(
        "streaming.walk_one_chunk",
        "A tick after the observer moves one column: decide, load the new ring, release the old",
        Budget {
            iterations_per_sample: 500,
            ..budget
        },
        || {
            step += 1;
            walking
                .set_interest(
                    InterestSource::new(InterestId(1), ChunkCoord::new(step, 0))
                        .with_radii(LodRadii::new(3, 3, 3, 0).expect("valid")),
                )
                .expect("existing source");
            consume(
                walking
                    .tick(&mut walk_backend, StreamingBudget::UNLIMITED)
                    .expect("valid budget")
                    .activated,
            );
        },
    ));

    // Fast travel: the whole resident set is replaced at once.
    let (mut travelling, mut travel_backend) = settled(3)?;
    let mut destination = 0i64;
    out.push(measure(
        "streaming.fast_travel_r3",
        "Teleport and settle: release 49 columns and take 49 more, decision side only",
        Budget {
            iterations_per_sample: 100,
            ..budget
        },
        || {
            destination += 1_000;
            travelling
                .set_interest(
                    InterestSource::new(InterestId(1), ChunkCoord::new(destination, destination))
                        .with_radii(LodRadii::new(3, 3, 3, 0).expect("valid")),
                )
                .expect("existing source");
            for _ in 0..8 {
                if travelling
                    .tick(&mut travel_backend, StreamingBudget::UNLIMITED)
                    .expect("valid budget")
                    .is_quiet()
                {
                    break;
                }
            }
            consume(travelling.resident_count());
        },
    ));

    // The other half of the pair: what residency actually costs against a real
    // world. Generation dominates, which is the point of measuring it apart
    // from the decision.
    let mut world = bench_world(32)?;
    let mut retained = RetainedChunks::new();
    let generated_target = StreamTarget::Chunk(ChunkCoord::new(64, 64));
    out.push(measure(
        "streaming.chunk_generate_cycle",
        "Activate a fresh column and drop it again: the cost of an unedited chunk",
        Budget {
            iterations_per_sample: 4,
            ..budget
        },
        || {
            let mut backend = WorldResidency::new(&mut world, &mut retained);
            consume(
                backend
                    .activate(generated_target, Lod::Full)
                    .expect("generated"),
            );
            backend.evict(generated_target).expect("evicted");
        },
    ));

    // The same cycle for a column that has been edited, so it is restored from
    // retention rather than regenerated. This is what retention buys.
    let mut edited_world = bench_world(32)?;
    let mut edited_retained = RetainedChunks::new();
    let edited_target = StreamTarget::Chunk(ChunkCoord::new(-64, -64));
    {
        let mut backend = WorldResidency::new(&mut edited_world, &mut edited_retained);
        backend.activate(edited_target, Lod::Full)?;
    }
    let stone = edited_world.block_id(&Identifier::parse("nexora:block/stone")?)?;
    edited_world.set_block(BlockPos::new(-2_040, 300, -2_040), stone)?;
    out.push(measure(
        "streaming.chunk_retained_cycle",
        "Persist, drop and restore an edited column: retention instead of regeneration",
        Budget {
            iterations_per_sample: 200,
            ..budget
        },
        || {
            let mut backend = WorldResidency::new(&mut edited_world, &mut edited_retained);
            backend.persist(edited_target).expect("persisted");
            backend.evict(edited_target).expect("evicted");
            consume(
                backend
                    .activate(edited_target, Lod::Full)
                    .expect("restored"),
            );
        },
    ));

    // Sample while the column is actually held. The cycle above ends with a
    // restore, so reading the store straight after it reports zero bytes for
    // something that plainly occupies memory.
    {
        let mut backend = WorldResidency::new(&mut edited_world, &mut edited_retained);
        backend.persist(edited_target)?;
        backend.evict(edited_target)?;
    }
    let retained_bytes = edited_retained.storage_bytes() as u64;
    debug_assert!(
        retained_bytes > 0,
        "an evicted edited column cannot occupy nothing"
    );
    out.push(record_bytes(
        "streaming.retained_bytes_per_chunk",
        "Memory an edited column occupies while it is evicted",
        retained_bytes,
    ));
    out.push(record_quantity(
        "streaming.columns_of_interest_r12",
        "Columns a radius-12 observer makes the manager consider every tick",
        25 * 25,
    ));

    Ok(out)
}

/// The vertical slice of `NEXORA TECHNOLOGY BENCHMARK PLAN.md`, verbatim.
///
/// Every stage must appear in exactly one of [`measured_stages`] and
/// [`unmeasured_stages`]. Keeping the plan's own list here is what makes that
/// checkable: a stage that becomes measurable and is not moved leaves the two
/// lists overlapping, and the test below fails.
pub const PLAN_SLICE_STAGES: [&str; 13] = [
    "window",
    "input",
    "RHI",
    "camera",
    "16³ voxel chunk",
    "mesh generation",
    "1,000 entities",
    "physics",
    "jobs",
    "streaming",
    "save/load",
    "headless server",
    "mod boundary",
];

/// A region of voxels read once into a dense array.
///
/// A measurement fixture, not engine code: it exists to answer "how much of
/// meshing is the mesher and how much is the world lookup" by removing the
/// lookup. Deliberately not built into `nexora-mesh` — the answer decides
/// whether that is worth doing, and building it first would be assuming it.
struct DenseSnapshot {
    origin: BlockPos,
    size: [usize; 3],
    cells: Vec<Option<nexora_mesh::mesh::SurfaceId>>,
}

impl DenseSnapshot {
    /// Read `extent` plus a one-cell border, so border culling still works.
    fn read<V: nexora_mesh::view::VoxelView>(view: &V, extent: nexora_mesh::view::Extent) -> Self {
        let size = [
            extent.size[0] as usize + 2,
            extent.size[1] as usize + 2,
            extent.size[2] as usize + 2,
        ];
        let origin = BlockPos::new(
            extent.origin.x - 1,
            extent.origin.y - 1,
            extent.origin.z - 1,
        );
        let mut cells = Vec::with_capacity(size[0] * size[1] * size[2]);
        for z in 0..size[2] {
            for y in 0..size[1] {
                for x in 0..size[0] {
                    cells.push(view.surface_at(BlockPos::new(
                        origin.x + x as i64,
                        origin.y + y as i64,
                        origin.z + z as i64,
                    )));
                }
            }
        }
        Self {
            origin,
            size,
            cells,
        }
    }

    fn index(&self, position: BlockPos) -> Option<usize> {
        let dx = position.x - self.origin.x;
        let dy = position.y - self.origin.y;
        let dz = position.z - self.origin.z;
        if dx < 0 || dy < 0 || dz < 0 {
            return None;
        }
        let (x, y, z) = (dx as usize, dy as usize, dz as usize);
        if x >= self.size[0] || y >= self.size[1] || z >= self.size[2] {
            return None;
        }
        Some((z * self.size[1] + y) * self.size[0] + x)
    }
}

impl nexora_mesh::view::VoxelView for DenseSnapshot {
    fn surface_at(&self, position: BlockPos) -> Option<nexora_mesh::mesh::SurfaceId> {
        self.cells[self.index(position)?]
    }

    fn occludes(&self, position: BlockPos) -> bool {
        // Outside the snapshot is unknown, and unknown occludes -- the same
        // call `WorldSurfaces` makes for a non-resident chunk.
        self.index(position)
            .is_none_or(|index| self.cells[index].is_some())
    }
}

/// Turning voxels into surfaces (RENDER-9).
///
/// Measures the two reductions separately, because they answer different
/// questions: culling decides how much surface exists, merging decides how many
/// primitives describe it. A single "meshing took N µs" would hide which half
/// to optimise — the same mistake a single physics number would have made
/// (finding 10).
///
/// # Errors
///
/// Returns an error when the benchmark world cannot be built.
pub fn meshing(budget: Budget) -> Result<Vec<Measurement>> {
    use nexora_mesh::greedy::unmerged_face_count;
    use nexora_mesh::mesh_region;
    use nexora_mesh::view::Extent;
    use nexora_simulation::WorldSurfaces;

    let world = populated_world()?;
    let surface = world.surface_height(8, 8);
    let mut out = Vec::new();

    // Straddling the ground/air boundary: the only place there is surface to
    // build. Meshing buried rock measures culling finding nothing.
    let across = Extent::cubic(BlockPos::new(0, surface - 8, 0), 16)?;
    let chunk_sized = Extent::cubic(BlockPos::new(0, surface - 16, 0), 32)?;

    out.push(measure(
        "mesh.region_16",
        "Cull and merge a 16 cubed region across the ground/air boundary",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        || {
            consume(mesh_region(&WorldSurfaces::untextured(&world), across).len());
        },
    ));

    out.push(measure(
        "mesh.region_32",
        "The same at 32 cubed, the engine's default section size",
        Budget {
            iterations_per_sample: 5,
            ..budget
        },
        || {
            consume(mesh_region(&WorldSurfaces::untextured(&world), chunk_sized).len());
        },
    ));

    out.push(measure(
        "mesh.cull_only_16",
        "Count visible faces without merging them: the culling half alone",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        || {
            consume(unmerged_face_count(
                &WorldSurfaces::untextured(&world),
                across,
            ));
        },
    ));

    // --- is the mesher slow, or is the world lookup slow? --------------------
    // The same question finding 10 asked of physics, and the pattern that has
    // produced every useful answer in this harness: run the identical workload
    // twice, differing in exactly one thing. Here that thing is where the
    // voxels come from.
    let snapshot = DenseSnapshot::read(&WorldSurfaces::untextured(&world), across);
    out.push(measure(
        "mesh.region_16_from_snapshot",
        "The same 16 cubed region, meshed from a pre-read dense array",
        Budget {
            iterations_per_sample: 20,
            ..budget
        },
        || {
            consume(mesh_region(&snapshot, across).len());
        },
    ));

    // What the two reductions actually bought, on this terrain. Recorded rather
    // than asserted: a ratio depends entirely on how smooth the ground is, and
    // quoting one as if it were a property of the mesher would be wrong.
    let surfaces = WorldSurfaces::untextured(&world);
    let mesh = mesh_region(&surfaces, across);
    let visible = unmerged_face_count(&surfaces, across);
    let total_cube_faces = across.cells() * 6;

    out.push(record_quantity(
        "mesh.cube_faces_16",
        "Faces a naive mesher would emit: every cell, all six sides",
        total_cube_faces,
    ));
    out.push(record_quantity(
        "mesh.visible_faces_16",
        "Faces left after culling",
        visible,
    ));
    out.push(record_quantity(
        "mesh.quads_16",
        "Rectangles left after greedy merging",
        mesh.len() as u64,
    ));
    out.push(record_quantity(
        "mesh.vertices_16",
        "Vertices a renderer would upload, at four per rectangle",
        mesh.vertex_count(),
    ));

    Ok(out)
}

/// What one crossing of a language boundary costs.
///
/// Three rungs of the same ladder, so the differences separate two costs the
/// word "FFI" usually runs together:
///
/// * `opaque - inlined` — what it costs to not inline a call. Not a language
///   cost; any `dyn`, any function pointer, any cross-crate call without LTO
///   pays it too.
/// * `crossing - opaque` — what it costs for the callee to have been compiled
///   by a different language's compiler. This is the number ADR-0004 needs.
///
/// Each pair is measured at two payload sizes, because that difference is the
/// whole design question: a boundary crossed once per 4 KiB and a boundary
/// crossed once per `u64` are not the same boundary.
///
/// Every rung passes its input through [`consume`] on the way in, including the
/// ones that do not need it. A call is already an optimizer barrier, so the
/// inlinable rung is the only one that could be fused across iterations -- and
/// it was: LLVM collapsed four mixes into one `imul` and reported a quarter of
/// the real cost. Applying the barrier to all three keeps the rungs comparable
/// instead of handicapping the two that have a call in them.
///
/// Returns only the Rust rungs when the build has no C++ linked; the crossing
/// is then reported by [`unmeasured_stages`] instead. Substituting a Rust
/// number under a cross-language label is the failure this guards against.
#[must_use]
pub fn ffi(budget: Budget) -> Vec<Measurement> {
    use nexora_ffi_probe::{crossing, inlined, opaque, CROSS_LANGUAGE_LINKED};

    /// A single crossing costs a handful of nanoseconds, which is below the
    /// resolution of one `Instant::now()` pair. Batched like the other
    /// nanosecond-scale kernels so the clock is amortised rather than measured.
    const SCALAR_ITERATIONS: u32 = 50_000;

    /// 4 KiB of FNV is microseconds, so a much smaller batch already dwarfs the
    /// clock.
    const BULK_ITERATIONS: u32 = 200;

    let scalar = Budget {
        iterations_per_sample: SCALAR_ITERATIONS,
        ..budget
    };
    let bulk = Budget {
        iterations_per_sample: BULK_ITERATIONS,
        ..budget
    };

    let mut measurements = Vec::new();

    // --- scalar: high frequency, nothing to amortise the crossing over -------
    let mut seed = 1u64;
    measurements.push(measure(
        "ffi.scalar_inlined",
        "Mix one u64, inlined Rust: the work with no call at all",
        scalar,
        || seed = inlined::mix(consume(seed)),
    ));
    consume(seed);

    let mut seed = 1u64;
    measurements.push(measure(
        "ffi.scalar_opaque_rust",
        "The same mix behind a Rust call the optimizer will not inline",
        scalar,
        || seed = opaque::mix(consume(seed)),
    ));
    consume(seed);

    if CROSS_LANGUAGE_LINKED {
        let mut seed = 1u64;
        measurements.push(measure(
            "ffi.scalar_crossing",
            "The same mix executed by C++, through the C ABI",
            scalar,
            || {
                seed =
                    crossing::mix(consume(seed)).expect("linked: checked by CROSS_LANGUAGE_LINKED")
            },
        ));
        consume(seed);

        // The crossing with as little work as possible on the far side, so the
        // remaining cost is the crossing and nothing else.
        let mut seed = 1u64;
        measurements.push(measure(
            "ffi.empty_crossing",
            "Into C++ and back doing almost nothing: the crossing alone",
            scalar,
            || {
                seed =
                    crossing::noop(consume(seed)).expect("linked: checked by CROSS_LANGUAGE_LINKED")
            },
        ));
        consume(seed);
    }

    // --- 4 KiB: one crossing, a page of work behind it -----------------------
    let payload: Vec<u8> = (0u32..4096)
        .map(|byte| byte.wrapping_mul(2_654_435_761) as u8)
        .collect();
    measurements.push(measure(
        "ffi.bulk_inlined",
        "Hash 4 KiB, inlined Rust",
        bulk,
        || {
            consume(inlined::digest(&payload));
        },
    ));
    measurements.push(measure(
        "ffi.bulk_opaque_rust",
        "Hash 4 KiB behind a Rust call the optimizer will not inline",
        bulk,
        || {
            consume(opaque::digest(&payload));
        },
    ));
    if CROSS_LANGUAGE_LINKED {
        measurements.push(measure(
            "ffi.bulk_crossing",
            "Hash 4 KiB in C++, the buffer passed as a pointer and a length",
            bulk,
            || {
                consume(crossing::digest(&payload));
            },
        ));
    }

    measurements
}

/// The plan's slice stages this build actually measures.
#[must_use]
pub fn measured_stages() -> Vec<&'static str> {
    vec![
        "16³ voxel chunk",
        "mesh generation",
        "1,000 entities",
        "physics",
        "jobs",
        "streaming",
        "save/load",
        "headless server",
    ]
}

/// The stages of the plan's vertical slice that Phase 0 cannot measure.
///
/// Listed rather than skipped: a benchmark table with silent gaps reads as a
/// benchmark that covered everything.
#[must_use]
pub fn unmeasured_stages() -> Vec<Unmeasured> {
    let mut stages = vec![
        Unmeasured {
            name: "window",
            reason: "no windowing layer; ADR-0005",
        },
        Unmeasured {
            name: "input",
            reason: "meaningless without a window",
        },
        Unmeasured {
            name: "RHI",
            reason: "boundary specified, no backend implemented",
        },
        Unmeasured {
            name: "camera",
            reason: "depends on the RHI",
        },
        Unmeasured {
            name: "mod boundary",
            reason: "mod runtime not implemented; Phase 7",
        },
        Unmeasured {
            name: "frame time",
            reason: "no render loop exists to time",
        },
        Unmeasured {
            name: "incremental build",
            reason: "measured by the build system, not by this process",
        },
        Unmeasured {
            name: "debugging / tooling effort",
            reason: "qualitative; the plan scores it separately from timing",
        },
    ];

    // Measured only when the C++ translation unit is linked in. Built without
    // the `cpp` feature there is no second language, so the honest report is
    // that the metric has no number -- not a Rust-to-Rust call wearing the
    // label.
    if !nexora_ffi_probe::CROSS_LANGUAGE_LINKED {
        stages.push(Unmeasured {
            name: "FFI overhead",
            reason: "built without the `cpp` feature; no boundary is linked in",
        });
    }

    stages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shift_shortcut_agrees_with_division_everywhere_it_is_used() {
        // The spatial benchmark compares two implementations. If they disagreed,
        // it would be timing a correct routine against a wrong one and calling
        // the difference an optimization.
        let shape = ChunkShape::cubic_default();
        let bits = 5u32;

        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        for _ in 0..20_000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let spread = |value: u64| (value as i64) % 4_000_000;
            let pos = BlockPos::new(spread(seed), spread(seed >> 16), spread(seed >> 32));

            let by_division = shape.section_of(pos);
            let by_shift = SectionCoord::new(pos.x >> bits, pos.y >> bits, pos.z >> bits);
            assert_eq!(by_division, by_shift, "disagreement at {pos:?}");
            // The const-width variant must agree too; it is timed as well.
            assert_eq!(
                by_division,
                SectionCoord::new(pos.x >> 5, pos.y >> 5, pos.z >> 5)
            );
        }

        // Explicitly including the negative boundary cases.
        for value in [-33i64, -32, -31, -1, 0, 1, 31, 32, 33] {
            let pos = BlockPos::new(value, value, value);
            assert_eq!(
                shape.section_of(pos),
                SectionCoord::new(pos.x >> bits, pos.y >> bits, pos.z >> bits),
                "disagreement at {value}"
            );
        }
    }

    #[test]
    fn every_suite_produces_measurements() {
        let budget = Budget {
            warmup_iterations: 0,
            samples: 1,
            iterations_per_sample: 1,
        };
        let scratch =
            std::env::temp_dir().join(format!("nexora-bench-test-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).expect("scratch");

        assert!(!worldgen(budget).expect("worldgen").is_empty());
        assert!(!voxel(budget).expect("voxel").is_empty());
        assert!(!spatial(budget).is_empty());
        assert!(!jobs(budget).expect("jobs").is_empty());
        assert!(!persistence(budget, &scratch)
            .expect("persistence")
            .is_empty());
        assert!(!startup(budget).expect("startup").is_empty());
        assert!(!entities(budget).expect("entities").is_empty());
        assert!(!physics(budget).expect("physics").is_empty());
        assert!(!streaming(budget).expect("streaming").is_empty());

        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn every_plan_stage_is_either_measured_or_declared_missing_and_never_both() {
        // This is the check that would have caught "1,000 entities" being left
        // in the unmeasured list after the entity suite started measuring it:
        // the report claimed the stage was missing on the same page it printed
        // numbers for it.
        let measured = measured_stages();
        let unmeasured = unmeasured_stages();

        for stage in PLAN_SLICE_STAGES {
            let is_measured = measured.contains(&stage);
            let is_declared = unmeasured.iter().any(|entry| entry.name == stage);
            assert!(
                is_measured || is_declared,
                "the plan's `{stage}` stage is neither measured nor declared missing"
            );
            assert!(
                !(is_measured && is_declared),
                "the plan's `{stage}` stage is both measured and declared missing"
            );
        }

        for stage in &measured {
            assert!(
                PLAN_SLICE_STAGES.contains(stage),
                "`{stage}` is claimed as measured but is not a stage of the plan"
            );
        }

        // Every gap must carry a reason; an empty one is a silent omission.
        assert!(unmeasured.iter().all(|entry| !entry.reason.is_empty()));
    }
}
