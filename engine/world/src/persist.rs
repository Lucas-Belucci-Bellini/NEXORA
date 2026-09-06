//! World serialization.
//!
//! Bridges [`crate::world::World`] and the versioned container in
//! `nexora-persistence`. The decision this module exists to enforce comes from
//! `Registry System.md` §12: **a save stores namespaced identifiers, never
//! runtime ids.**
//!
//! Runtime ids depend on which content happened to register first. Writing them
//! into a chunk means that installing a mod, or removing one, silently turns
//! every stone block in the world into whatever now occupies that integer. So a
//! save carries a palette of identifiers, chunk data indexes into that palette,
//! and loading remaps those indices onto the current registry - failing loudly
//! when an identifier no longer exists.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{ChunkCoord, ChunkShape};
use nexora_foundation::time::{CalendarConfig, WorldTime};
use nexora_foundation::version::GeneratorVersion;
use nexora_persistence::codec::{Reader, Writer};
use nexora_persistence::SaveContainer;

use crate::chunk::{Chunk, ChunkState};
use crate::voxel::{BlockStateId, Section, SectionParts};
use crate::world::{World, WorldBounds, WorldDescriptor, WorldId};

/// Section holding world identity, seed, bounds, clock and calendar.
pub const SECTION_WORLD_HEADER: &str = "nexora:save/world_header";

/// Section holding the identifier palette that chunk data indexes into.
pub const SECTION_BLOCK_PALETTE: &str = "nexora:save/block_palette";

/// Section holding every resident chunk.
pub const SECTION_CHUNKS: &str = "nexora:save/chunks";

/// Storage tag for a uniform section.
const TAG_UNIFORM: u8 = 0;
/// Storage tag for a paletted section.
const TAG_PALETTED: u8 = 1;
/// Storage tag for a direct section.
const TAG_DIRECT: u8 = 2;

fn section_id(raw: &str) -> Result<Identifier> {
    Identifier::parse(raw)
}

/// Serialize a world into a save container.
///
/// # Errors
///
/// Returns an error when a resident block state has no registered identifier,
/// which would mean the world is holding a state the registry does not know.
pub fn save(world: &World) -> Result<SaveContainer> {
    let mut container = SaveContainer::new();
    container.put(section_id(SECTION_WORLD_HEADER)?, encode_header(world));
    container.put(section_id(SECTION_BLOCK_PALETTE)?, encode_palette(world)?);
    container.put(section_id(SECTION_CHUNKS)?, encode_chunks(world));
    Ok(container)
}

/// Rebuild a world from a save container.
///
/// # Errors
///
/// Returns an error when a section is missing, malformed, or references content
/// that is not registered in this build.
pub fn load(container: &SaveContainer) -> Result<World> {
    container.compatibility()?;

    let (descriptor, calendar, now) =
        decode_header(container.require(&section_id(SECTION_WORLD_HEADER)?)?)?;
    let mut world = World::resumed(descriptor, calendar, now)?;

    let saved_palette = decode_palette(container.require(&section_id(SECTION_BLOCK_PALETTE)?)?)?;

    // Resolve the saved identifiers against *this* build's registry. A save that
    // names content we do not have stops here, with the name, rather than
    // loading a world full of quietly wrong blocks.
    let remap: Vec<BlockStateId> = saved_palette
        .iter()
        .map(|id| {
            world.block_id(id).map_err(|cause| {
                Error::new(
                    Domain::Save,
                    "world-persist",
                    "the save references block content that is not registered in this build",
                )
                .with_recovery(Recovery::Quarantine)
                .with_context("block", id.to_string())
                .with_source(cause)
            })
        })
        .collect::<Result<_>>()?;

    let chunks = decode_chunks(
        container.require(&section_id(SECTION_CHUNKS)?)?,
        world.descriptor().shape,
        &remap,
    )?;
    for chunk in chunks {
        world.insert_chunk(chunk);
    }

    Ok(world)
}

fn encode_header(world: &World) -> Vec<u8> {
    let descriptor = world.descriptor();
    let calendar = world.clock().calendar();
    let shape = descriptor.shape;

    let mut out = Writer::new();
    out.u64(descriptor.id.0);
    out.string(&descriptor.name);
    out.u64(descriptor.seed);
    out.u32(descriptor.generator_version.0);
    out.u32(shape.size_x());
    out.u32(shape.size_y());
    out.u32(shape.size_z());
    out.i64(descriptor.bounds.min_y);
    out.i64(descriptor.bounds.max_y);
    out.u64(world.clock().now().ticks());
    out.u32(calendar.ticks_per_second());
    out.u32(calendar.seconds_per_minute());
    out.u32(calendar.minutes_per_hour());
    out.u32(calendar.hours_per_day());
    out.u32(calendar.days_per_month());
    out.u32(calendar.months_per_year());
    out.u32(calendar.seasons_per_year());
    out.finish()
}

fn decode_header(bytes: &[u8]) -> Result<(WorldDescriptor, CalendarConfig, WorldTime)> {
    let mut reader = Reader::new(bytes);

    let id = WorldId(reader.u64()?);
    let name = reader.string()?.to_owned();
    let seed = reader.u64()?;
    let generator_version = GeneratorVersion(reader.u32()?);
    let shape = ChunkShape::new(reader.u32()?, reader.u32()?, reader.u32()?)?;
    let bounds = WorldBounds {
        min_y: reader.i64()?,
        max_y: reader.i64()?,
    }
    .validate()?;
    let now = WorldTime(reader.u64()?);
    let calendar = CalendarConfig::new(
        reader.u32()?,
        reader.u32()?,
        reader.u32()?,
        reader.u32()?,
        reader.u32()?,
        reader.u32()?,
        reader.u32()?,
    )?;
    reader.expect_exhausted()?;

    Ok((
        WorldDescriptor {
            id,
            name,
            seed,
            generator_version,
            shape,
            bounds,
        },
        calendar,
        now,
    ))
}

fn encode_palette(world: &World) -> Result<Vec<u8>> {
    let entries = world.blocks().len();
    let mut out = Writer::new();
    out.u32(entries as u32);
    for runtime_id in 0..entries as u32 {
        let identifier = world
            .block_identifier(BlockStateId(runtime_id))
            .ok_or_else(|| {
                Error::new(
                    Domain::Save,
                    "world-persist",
                    "a block state in the world has no registered identifier",
                )
                .fatal()
                .with_context("runtime_id", runtime_id.to_string())
            })?;
        out.string(&identifier.to_string());
    }
    Ok(out.finish())
}

fn decode_palette(bytes: &[u8]) -> Result<Vec<Identifier>> {
    let mut reader = Reader::new(bytes);
    let count = reader.u32()?;
    let mut palette = Vec::with_capacity(count as usize);
    for _ in 0..count {
        palette.push(Identifier::parse(reader.string()?)?);
    }
    reader.expect_exhausted()?;
    Ok(palette)
}

fn encode_chunks(world: &World) -> Vec<u8> {
    let coords = world.loaded_chunks();
    let mut out = Writer::new();
    out.u32(coords.len() as u32);

    for coord in coords {
        let Some(chunk) = world.chunk(coord) else {
            continue;
        };
        out.i64(coord.x);
        out.i64(coord.z);

        let indices = chunk.section_indices();
        out.u32(indices.len() as u32);
        for section_y in indices {
            out.i64(section_y);
            let Some(section) = chunk.section(section_y) else {
                continue;
            };
            match section.to_parts() {
                SectionParts::Uniform(state) => {
                    out.u8(TAG_UNIFORM);
                    out.u32(state.0);
                }
                SectionParts::Paletted {
                    palette,
                    bits,
                    words,
                } => {
                    out.u8(TAG_PALETTED);
                    out.u32(palette.len() as u32);
                    for entry in palette {
                        out.u32(entry.0);
                    }
                    out.u8(bits);
                    out.u32(words.len() as u32);
                    for word in words {
                        out.u64(word);
                    }
                }
                SectionParts::Direct(cells) => {
                    out.u8(TAG_DIRECT);
                    out.u32(cells.len() as u32);
                    for cell in cells {
                        out.u32(cell.0);
                    }
                }
            }
        }
    }
    out.finish()
}

fn decode_chunks(bytes: &[u8], shape: ChunkShape, remap: &[BlockStateId]) -> Result<Vec<Chunk>> {
    let translate = |saved: BlockStateId| -> Result<BlockStateId> {
        remap.get(saved.0 as usize).copied().ok_or_else(|| {
            Error::new(
                Domain::Save,
                "world-persist",
                "chunk data references a palette entry the save does not contain",
            )
            .with_recovery(Recovery::Quarantine)
            .with_context("index", saved.0.to_string())
            .with_context("palette_len", remap.len().to_string())
        })
    };

    let mut reader = Reader::new(bytes);
    let chunk_count = reader.u32()?;
    let mut chunks = Vec::with_capacity(chunk_count as usize);

    for _ in 0..chunk_count {
        let coord = ChunkCoord::new(reader.i64()?, reader.i64()?);
        let mut chunk = Chunk::new(coord, shape);

        let section_count = reader.u32()?;
        for _ in 0..section_count {
            let section_y = reader.i64()?;
            let parts = match reader.u8()? {
                TAG_UNIFORM => SectionParts::Uniform(BlockStateId(reader.u32()?)),
                TAG_PALETTED => {
                    let palette_len = reader.u32()?;
                    let mut palette = Vec::with_capacity(palette_len as usize);
                    for _ in 0..palette_len {
                        palette.push(BlockStateId(reader.u32()?));
                    }
                    let bits = reader.u8()?;
                    let word_count = reader.u32()?;
                    let mut words = Vec::with_capacity(word_count as usize);
                    for _ in 0..word_count {
                        words.push(reader.u64()?);
                    }
                    SectionParts::Paletted {
                        palette,
                        bits,
                        words,
                    }
                }
                TAG_DIRECT => {
                    let cell_count = reader.u32()?;
                    if cell_count as usize != shape.volume() {
                        return Err(Error::new(
                            Domain::Save,
                            "world-persist",
                            "a stored section does not match the world's chunk shape",
                        )
                        .with_recovery(Recovery::Quarantine)
                        .with_context("expected", shape.volume().to_string())
                        .with_context("found", cell_count.to_string()));
                    }
                    let mut cells = Vec::with_capacity(cell_count as usize);
                    for _ in 0..cell_count {
                        cells.push(BlockStateId(reader.u32()?));
                    }
                    SectionParts::Direct(cells)
                }
                unknown => {
                    return Err(Error::new(
                        Domain::Save,
                        "world-persist",
                        "unknown section storage tag",
                    )
                    .with_recovery(Recovery::Quarantine)
                    .with_context("tag", unknown.to_string()))
                }
            };

            let mut section = Section::from_parts(shape, parts)?;
            section.remap(&translate)?;
            chunk.put_section(section_y, section);
        }

        // A chunk read from storage starts life already loaded and clean.
        chunk.force_state(ChunkState::Loaded);
        chunk.mark_clean();
        chunk.take_journal();
        chunks.push(chunk);
    }

    reader.expect_exhausted()?;
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::AIR;
    use crate::world::WorldPhase;
    use nexora_foundation::spatial::BlockPos;
    use nexora_foundation::time::TimeScale;

    fn built_world(seed: u64) -> World {
        let mut world = World::create(
            WorldDescriptor::new("persistence-test", seed).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        world.bring_online().unwrap();
        for x in -1..=1 {
            for z in -1..=1 {
                world.load_or_generate(ChunkCoord::new(x, z)).unwrap();
            }
        }
        world
    }

    fn block(world: &World, path: &str) -> BlockStateId {
        world.block_id(&Identifier::parse(path).unwrap()).unwrap()
    }

    #[test]
    fn a_world_survives_a_save_and_load_cycle() {
        let mut original = built_world(0xBEEF);
        original
            .clock_mut()
            .advance_by(TimeScale::Hour, 30)
            .unwrap();

        let stone = block(&original, "nexora:block/stone");
        let edits = [
            (BlockPos::new(5, 200, 5), stone),
            (BlockPos::new(-20, 64, -20), AIR),
            (
                BlockPos::new(31, 100, 31),
                block(&original, "nexora:block/grass"),
            ),
        ];
        for (position, state) in edits {
            original.set_block(position, state).unwrap();
        }

        let container = save(&original).unwrap();
        let encoded = container.encode();
        let restored = load(&SaveContainer::decode(&encoded).unwrap()).unwrap();

        assert_eq!(restored.descriptor(), original.descriptor());
        assert_eq!(restored.clock().now(), original.clock().now());
        assert_eq!(restored.chunk_count(), original.chunk_count());

        for (position, expected) in edits {
            assert_eq!(
                restored.get_block(position).unwrap(),
                expected,
                "at {position:?}"
            );
        }
    }

    #[test]
    fn every_generated_block_survives_the_round_trip() {
        let original = built_world(1234);
        let restored = load(&save(&original).unwrap()).unwrap();

        // Sample the full vertical range of one column plus the chunk corners.
        for coord in original.loaded_chunks() {
            for (dx, dz) in [(0i64, 0i64), (31, 31), (0, 31), (31, 0), (16, 16)] {
                let x = coord.x * 32 + dx;
                let z = coord.z * 32 + dz;
                for y in [-1920, -1000, -33, -1, 0, 50, 64, 76, 200, 1900] {
                    let position = BlockPos::new(x, y, z);
                    assert_eq!(
                        restored.get_block(position).unwrap(),
                        original.get_block(position).unwrap(),
                        "block at {position:?} changed across the round trip"
                    );
                }
            }
        }
    }

    #[test]
    fn a_restored_world_resumes_rather_than_restarts() {
        let mut original = built_world(9);
        original.clock_mut().advance_by(TimeScale::Day, 3).unwrap();
        original.transition_to(WorldPhase::Suspended).unwrap();

        let restored = load(&save(&original).unwrap()).unwrap();
        assert_eq!(restored.clock().date().day, original.clock().date().day);
        assert_ne!(restored.clock().now(), WorldTime::EPOCH);
        // Lifecycle phase is runtime state, not saved state: a loaded world
        // starts at the beginning of its lifecycle with its data intact.
        assert_eq!(restored.phase(), WorldPhase::Requested);
    }

    #[test]
    fn saved_chunks_come_back_clean_and_loaded() {
        let mut original = built_world(3);
        original
            .set_block(
                BlockPos::new(0, 150, 0),
                block(&original, "nexora:block/dirt"),
            )
            .unwrap();
        assert!(original.chunk(ChunkCoord::new(0, 0)).unwrap().is_dirty());

        let restored = load(&save(&original).unwrap()).unwrap();
        let chunk = restored.chunk(ChunkCoord::new(0, 0)).unwrap();
        assert_eq!(chunk.state(), ChunkState::Loaded);
        assert!(
            !chunk.is_dirty(),
            "a freshly loaded chunk has nothing to write back"
        );
        assert!(chunk.journal().is_empty());
    }

    #[test]
    fn chunk_data_is_remapped_through_identifiers_not_runtime_ids() {
        // This is the property the whole module exists for. The save below lists
        // its palette in a different order from the registry, so index 0 means
        // stone rather than air. If loading trusted the integers, the block
        // would come back as air.
        let mut header = Writer::new();
        let descriptor = WorldDescriptor::new("remap-test", 5).unwrap();
        let calendar = CalendarConfig::earthlike();
        header.u64(descriptor.id.0);
        header.string(&descriptor.name);
        header.u64(descriptor.seed);
        header.u32(descriptor.generator_version.0);
        header.u32(descriptor.shape.size_x());
        header.u32(descriptor.shape.size_y());
        header.u32(descriptor.shape.size_z());
        header.i64(descriptor.bounds.min_y);
        header.i64(descriptor.bounds.max_y);
        header.u64(0);
        header.u32(calendar.ticks_per_second());
        header.u32(calendar.seconds_per_minute());
        header.u32(calendar.minutes_per_hour());
        header.u32(calendar.hours_per_day());
        header.u32(calendar.days_per_month());
        header.u32(calendar.months_per_year());
        header.u32(calendar.seasons_per_year());

        // Deliberately permuted relative to registration order.
        let mut palette = Writer::new();
        palette.u32(4);
        for name in [
            "nexora:block/stone",
            "nexora:block/grass",
            "nexora:block/air",
            "nexora:block/dirt",
        ] {
            palette.string(name);
        }

        // One chunk, one uniform section holding saved index 0 == stone.
        let mut chunks = Writer::new();
        chunks.u32(1);
        chunks.i64(0);
        chunks.i64(0);
        chunks.u32(1);
        chunks.i64(2);
        chunks.u8(TAG_UNIFORM);
        chunks.u32(0);

        let mut container = SaveContainer::new();
        container.put(section_id(SECTION_WORLD_HEADER).unwrap(), header.finish());
        container.put(section_id(SECTION_BLOCK_PALETTE).unwrap(), palette.finish());
        container.put(section_id(SECTION_CHUNKS).unwrap(), chunks.finish());

        let world = load(&container).unwrap();
        let stone = block(&world, "nexora:block/stone");
        assert_ne!(
            stone, AIR,
            "the test is meaningless if stone is already id 0"
        );
        assert_eq!(
            world.get_block(BlockPos::new(0, 64, 0)).unwrap(),
            stone,
            "saved index 0 must resolve through the identifier, not the integer"
        );
    }

    #[test]
    fn a_save_naming_absent_content_is_refused_with_the_name() {
        let original = built_world(2);
        let mut container = save(&original).unwrap();

        // Rewrite the palette to name a block this build does not have.
        let mut palette = Writer::new();
        palette.u32(4);
        for name in [
            "nexora:block/air",
            "example:block/ruby_ore",
            "nexora:block/dirt",
            "nexora:block/grass",
        ] {
            palette.string(name);
        }
        container.put(section_id(SECTION_BLOCK_PALETTE).unwrap(), palette.finish());

        let err = load(&container).expect_err("missing content must not load silently");
        assert_eq!(err.recovery(), Recovery::Quarantine);
        assert!(err.to_string().contains("example:block/ruby_ore"), "{err}");
    }

    #[test]
    fn a_missing_section_is_refused() {
        let original = built_world(2);
        let full = save(&original).unwrap();

        for missing in [SECTION_WORLD_HEADER, SECTION_BLOCK_PALETTE, SECTION_CHUNKS] {
            let mut container = SaveContainer::new();
            for name in full.section_names() {
                if name.to_string() == missing {
                    continue;
                }
                container.put(name.clone(), full.get(&name).unwrap().to_vec());
            }
            assert!(
                load(&container).is_err(),
                "loading without {missing} must fail"
            );
        }
    }

    #[test]
    fn an_unknown_storage_tag_is_refused() {
        let original = built_world(2);
        let mut container = save(&original).unwrap();

        let mut chunks = Writer::new();
        chunks.u32(1);
        chunks.i64(0);
        chunks.i64(0);
        chunks.u32(1);
        chunks.i64(0);
        chunks.u8(99);
        container.put(section_id(SECTION_CHUNKS).unwrap(), chunks.finish());

        let err = load(&container).expect_err("an unknown tag must be refused");
        assert!(err.to_string().contains("storage tag"), "{err}");
    }

    #[test]
    fn an_empty_world_round_trips() {
        let world = World::create(
            WorldDescriptor::new("empty", 1).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        let restored = load(&save(&world).unwrap()).unwrap();
        assert_eq!(restored.chunk_count(), 0);
        assert_eq!(restored.descriptor(), world.descriptor());
    }
}
