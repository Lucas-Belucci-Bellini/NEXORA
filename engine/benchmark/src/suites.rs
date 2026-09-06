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
use nexora_foundation::ident::Identifier;
use nexora_foundation::ident::WorldId;
use nexora_foundation::spatial::WorldPosition;
use nexora_foundation::spatial::{BlockPos, ChunkCoord, ChunkShape, SectionCoord};
use nexora_foundation::time::CalendarConfig;
use nexora_foundation::time::WorldDuration;
use nexora_persistence::SaveContainer;
use nexora_runtime::jobs::{JobSystem, Priority};
use nexora_world::persist;
use nexora_world::voxel::{BlockStateId, Section, AIR};
use nexora_world::world::{World, WorldDescriptor};

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

/// The stages of the plan's vertical slice that Phase 0 cannot measure.
///
/// Listed rather than skipped: a benchmark table with silent gaps reads as a
/// benchmark that covered everything.
#[must_use]
pub fn unmeasured_stages() -> Vec<Unmeasured> {
    vec![
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
            name: "mesh generation",
            reason: "no renderer to consume a mesh",
        },
        Unmeasured {
            name: "1,000 entities",
            reason: "entity system not implemented; ADR-0005",
        },
        Unmeasured {
            name: "physics",
            reason: "depends on entities",
        },
        Unmeasured {
            name: "streaming",
            reason: "chunks are loaded explicitly, no streaming manager",
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
            name: "FFI overhead",
            reason: "single-language build; nothing crosses a boundary",
        },
        Unmeasured {
            name: "incremental build",
            reason: "measured by the build system, not by this process",
        },
        Unmeasured {
            name: "debugging / tooling effort",
            reason: "qualitative; the plan scores it separately from timing",
        },
    ]
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

        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn the_unmeasured_list_covers_every_missing_slice_stage() {
        let stages = unmeasured_stages();
        for expected in [
            "window",
            "RHI",
            "mesh generation",
            "1,000 entities",
            "physics",
            "streaming",
        ] {
            assert!(
                stages.iter().any(|entry| entry.name == expected),
                "the plan's `{expected}` stage is neither measured nor declared missing"
            );
        }
        // Every gap must carry a reason; an empty one is a silent omission.
        assert!(stages.iter().all(|entry| !entry.reason.is_empty()));
    }
}
