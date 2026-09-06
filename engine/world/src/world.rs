//! World runtime: identity, lifecycle, block registry, chunks and generation.
//!
//! Implements `NEXORA WORLD STATE LIFECYCLE.md` and the deterministic
//! generation contract in `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md`.
//!
//! The lifecycle document lists streaming, LOD transition and checkpointing
//! inside its sequence. Those are *repeating operations* on a live world rather
//! than one-way phases, so [`WorldPhase`] models the linear progression and
//! streaming happens within [`WorldPhase::Active`]. The distinction matters:
//! a phase you can re-enter is not a phase, it is an operation.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::rng::{positional_rng, ReproductionKey, SeedStream};
use nexora_foundation::spatial::{BlockPos, ChunkCoord, ChunkShape};
use nexora_foundation::time::{CalendarConfig, WorldClock, WorldTime};
use nexora_foundation::version::{ContentVersion, GeneratorVersion, ENGINE_VERSION};
use nexora_runtime::registry::Registry;

use crate::chunk::{Chunk, ChunkState};
use crate::voxel::{BlockStateId, Section, AIR};

/// Version of the terrain algorithm in this build.
///
/// Bumping this is mandatory whenever generation changes: the same seed is only
/// guaranteed to reproduce a world under the same generator version.
pub const GENERATOR_VERSION: GeneratorVersion = GeneratorVersion(1);

pub use nexora_foundation::ident::WorldId;

/// The vertical extent of a dimension, in blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldBounds {
    /// Lowest buildable block.
    pub min_y: i64,
    /// Highest buildable block.
    pub max_y: i64,
}

impl WorldBounds {
    /// The default deep world: 1920 blocks up and down.
    pub const DEFAULT: Self = Self {
        min_y: -1920,
        max_y: 1920,
    };

    /// Validate the bounds.
    ///
    /// # Errors
    ///
    /// Returns an error when the range is empty or inverted.
    pub fn validate(self) -> Result<Self> {
        if self.min_y >= self.max_y {
            return Err(Error::new(
                Domain::World,
                "world-bounds",
                "the vertical range must be non-empty and ascending",
            )
            .with_recovery(Recovery::Reject)
            .with_context("min_y", self.min_y.to_string())
            .with_context("max_y", self.max_y.to_string()));
        }
        Ok(self)
    }

    /// Whether a block position is inside the buildable range.
    #[must_use]
    pub const fn contains_y(self, y: i64) -> bool {
        y >= self.min_y && y <= self.max_y
    }
}

/// A registered block type.
///
/// Deliberately minimal for Phase 0: the Core knows that something is
/// registered, not what it means (`CORE.md` §5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDefinition {
    /// Whether the block obstructs movement. Consumed by physics, not by storage.
    pub solid: bool,
}

/// The linear phases of a world's existence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorldPhase {
    /// Requested but not yet checked.
    Requested,
    /// Configuration validated.
    Validated,
    /// Seed and generator version fixed; from here they are immutable.
    Seeded,
    /// Root state generated or loaded.
    RootReady,
    /// Registered with the runtime and addressable.
    Registered,
    /// Simulating. Streaming, LOD transitions and checkpoints happen here.
    Active,
    /// Not simulating; state intact.
    Suspended,
    /// Flushed and closed.
    Closed,
}

/// Everything needed to identify and recreate a world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDescriptor {
    /// Persistent identity.
    pub id: WorldId,
    /// Human-readable name.
    pub name: String,
    /// Immutable generation seed.
    pub seed: u64,
    /// Generator algorithm version.
    pub generator_version: GeneratorVersion,
    /// Section extent.
    pub shape: ChunkShape,
    /// Vertical extent.
    pub bounds: WorldBounds,
}

impl WorldDescriptor {
    /// Build a descriptor with this build's generator and the default shape.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is empty or the bounds are invalid.
    pub fn new(name: &str, seed: u64) -> Result<Self> {
        if name.trim().is_empty() {
            return Err(
                Error::new(Domain::World, "world-descriptor", "a world needs a name")
                    .with_recovery(Recovery::Reject),
            );
        }
        Ok(Self {
            id: WorldId::derive(name, seed),
            name: name.to_owned(),
            seed,
            generator_version: GENERATOR_VERSION,
            shape: ChunkShape::cubic_default(),
            bounds: WorldBounds::DEFAULT.validate()?,
        })
    }

    /// The versioned inputs that make this world reproducible.
    #[must_use]
    pub fn reproduction_key(&self) -> ReproductionKey {
        ReproductionKey {
            world_seed: self.seed,
            generator_version: self.generator_version,
            engine_version: ENGINE_VERSION,
            content_version: ContentVersion(1),
            mods: Vec::new(),
        }
    }
}

/// Base terrain height, in blocks.
const TERRAIN_BASE_HEIGHT: i64 = 64;

/// Peak-to-trough variation of the terrain surface, in blocks.
const TERRAIN_AMPLITUDE: i64 = 12;

/// Depth of the soil layer beneath the surface block.
const SOIL_DEPTH: i64 = 4;

/// A live world.
#[derive(Debug)]
pub struct World {
    descriptor: WorldDescriptor,
    clock: WorldClock,
    blocks: Registry<BlockDefinition>,
    chunks: BTreeMap<ChunkCoord, Chunk>,
    phase: WorldPhase,
}

impl World {
    /// Create a world and register the built-in block set.
    ///
    /// # Errors
    ///
    /// Returns an error when block registration fails.
    pub fn create(descriptor: WorldDescriptor, calendar: CalendarConfig) -> Result<Self> {
        let mut blocks = Registry::new(Identifier::parse("nexora:registry/block")?);

        // Air must be runtime id 0: section storage answers "is this empty"
        // without a registry lookup, and that shortcut depends on this order.
        let air = blocks.register(
            Identifier::parse("nexora:block/air")?,
            BlockDefinition { solid: false },
        )?;
        debug_assert_eq!(BlockStateId(air.0), AIR, "air must occupy runtime id 0");
        blocks.register(
            Identifier::parse("nexora:block/stone")?,
            BlockDefinition { solid: true },
        )?;
        blocks.register(
            Identifier::parse("nexora:block/dirt")?,
            BlockDefinition { solid: true },
        )?;
        blocks.register(
            Identifier::parse("nexora:block/grass")?,
            BlockDefinition { solid: true },
        )?;
        blocks.freeze();

        Ok(Self {
            descriptor,
            clock: WorldClock::new(calendar),
            blocks,
            chunks: BTreeMap::new(),
            phase: WorldPhase::Requested,
        })
    }

    /// Rebuild a world at a persisted time, used when loading a save.
    ///
    /// # Errors
    ///
    /// Returns an error when block registration fails.
    pub fn resumed(
        descriptor: WorldDescriptor,
        calendar: CalendarConfig,
        now: WorldTime,
    ) -> Result<Self> {
        let mut world = Self::create(descriptor, calendar)?;
        world.clock = WorldClock::resumed_at(calendar, now);
        Ok(world)
    }

    /// The world's descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &WorldDescriptor {
        &self.descriptor
    }

    /// The authoritative clock.
    #[must_use]
    pub const fn clock(&self) -> &WorldClock {
        &self.clock
    }

    /// Mutable access to the clock, for the runtime that owns time advancement.
    pub fn clock_mut(&mut self) -> &mut WorldClock {
        &mut self.clock
    }

    /// The block registry.
    #[must_use]
    pub const fn blocks(&self) -> &Registry<BlockDefinition> {
        &self.blocks
    }

    /// The current lifecycle phase.
    #[must_use]
    pub const fn phase(&self) -> WorldPhase {
        self.phase
    }

    /// Move the world to a new phase.
    ///
    /// # Errors
    ///
    /// Returns an error for a transition that is not allowed.
    pub fn transition_to(&mut self, next: WorldPhase) -> Result<()> {
        let allowed = match self.phase {
            WorldPhase::Requested => next == WorldPhase::Validated,
            WorldPhase::Validated => next == WorldPhase::Seeded,
            WorldPhase::Seeded => next == WorldPhase::RootReady,
            WorldPhase::RootReady => next == WorldPhase::Registered,
            WorldPhase::Registered => matches!(next, WorldPhase::Active | WorldPhase::Closed),
            // Suspend and resume may repeat; closing is always available.
            WorldPhase::Active => matches!(next, WorldPhase::Suspended | WorldPhase::Closed),
            WorldPhase::Suspended => matches!(next, WorldPhase::Active | WorldPhase::Closed),
            WorldPhase::Closed => false,
        };
        if !allowed {
            return Err(
                Error::new(Domain::World, "world", "illegal world phase transition")
                    .with_recovery(Recovery::Manual)
                    .with_context("from", format!("{:?}", self.phase))
                    .with_context("to", format!("{next:?}")),
            );
        }
        self.phase = next;
        Ok(())
    }

    /// Walk from the current phase through to [`WorldPhase::Active`].
    ///
    /// # Errors
    ///
    /// Returns an error when any step is refused.
    pub fn bring_online(&mut self) -> Result<()> {
        for phase in [
            WorldPhase::Validated,
            WorldPhase::Seeded,
            WorldPhase::RootReady,
            WorldPhase::Registered,
            WorldPhase::Active,
        ] {
            if self.phase < phase {
                self.transition_to(phase)?;
            }
        }
        Ok(())
    }

    /// Resolve a block identifier to its session-local state id.
    ///
    /// # Errors
    ///
    /// Returns an error when the block is not registered.
    pub fn block_id(&self, id: &Identifier) -> Result<BlockStateId> {
        Ok(BlockStateId(self.blocks.require(id)?.runtime_id().0))
    }

    /// The identifier behind a state id, for saving and diagnostics.
    #[must_use]
    pub fn block_identifier(&self, state: BlockStateId) -> Option<Identifier> {
        self.blocks
            .get_by_runtime_id(nexora_runtime::registry::RuntimeId(state.0))
            .map(|entry| entry.id().clone())
    }

    /// Chunk columns currently resident.
    #[must_use]
    pub fn loaded_chunks(&self) -> Vec<ChunkCoord> {
        self.chunks.keys().copied().collect()
    }

    /// How many chunk columns are resident.
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Approximate voxel storage cost across resident chunks, in bytes.
    #[must_use]
    pub fn storage_bytes(&self) -> usize {
        self.chunks.values().map(Chunk::storage_bytes).sum()
    }

    /// Borrow a resident chunk.
    #[must_use]
    pub fn chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    /// Mutably borrow a resident chunk.
    pub fn chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    /// Insert an already-built chunk, e.g. one read from a save.
    pub fn insert_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.coord(), chunk);
    }

    /// Ensure a chunk is resident, generating it if it is not.
    ///
    /// # Errors
    ///
    /// Returns an error when generation fails.
    pub fn load_or_generate(&mut self, coord: ChunkCoord) -> Result<&Chunk> {
        if !self.chunks.contains_key(&coord) {
            let chunk = self.generate_chunk(coord)?;
            self.chunks.insert(coord, chunk);
        }
        self.chunks.get(&coord).ok_or_else(|| {
            Error::new(
                Domain::World,
                "world",
                "chunk vanished immediately after insertion",
            )
            .fatal()
        })
    }

    /// Remove a chunk from memory.
    ///
    /// Returns the chunk so the caller can persist it; dropping it here would
    /// discard unsaved edits silently.
    pub fn unload_chunk(&mut self, coord: ChunkCoord) -> Option<Chunk> {
        self.chunks.remove(&coord)
    }

    /// Read a block from a resident chunk.
    ///
    /// # Errors
    ///
    /// Returns an error when the chunk is not resident or the position is out of
    /// bounds.
    pub fn get_block(&self, position: BlockPos) -> Result<BlockStateId> {
        let coord = self.descriptor.shape.section_of(position).column();
        let chunk = self.chunks.get(&coord).ok_or_else(|| {
            Error::new(Domain::World, "world", "chunk is not resident")
                .with_recovery(Recovery::Retry)
                .with_context("chunk", format!("{},{}", coord.x, coord.z))
        })?;
        chunk.get(position)
    }

    /// Write a block into a resident chunk.
    ///
    /// # Errors
    ///
    /// Returns an error when the chunk is not resident, the position is outside
    /// the world's vertical bounds, or the chunk is not in a writable state.
    pub fn set_block(&mut self, position: BlockPos, state: BlockStateId) -> Result<BlockStateId> {
        if !self.descriptor.bounds.contains_y(position.y) {
            return Err(Error::new(
                Domain::World,
                "world",
                "block position is outside the world's vertical bounds",
            )
            .with_recovery(Recovery::Reject)
            .with_context("y", position.y.to_string())
            .with_context(
                "bounds",
                format!(
                    "{}..={}",
                    self.descriptor.bounds.min_y, self.descriptor.bounds.max_y
                ),
            ));
        }

        let now = self.clock.now();
        let coord = self.descriptor.shape.section_of(position).column();
        let chunk = self.chunks.get_mut(&coord).ok_or_else(|| {
            Error::new(Domain::World, "world", "chunk is not resident")
                .with_recovery(Recovery::Retry)
                .with_context("chunk", format!("{},{}", coord.x, coord.z))
        })?;

        if !chunk.state().is_resident() {
            return Err(
                Error::new(Domain::World, "world", "chunk is not in a writable state")
                    .with_recovery(Recovery::Retry)
                    .with_context("chunk", format!("{},{}", coord.x, coord.z))
                    .with_context("state", chunk.state().as_str()),
            );
        }

        chunk.set(position, state, now)
    }

    /// The generated surface height for one world column.
    ///
    /// Derived from a position-seeded stream, so a column's height does not
    /// depend on which chunks were generated first.
    #[must_use]
    pub fn surface_height(&self, x: i64, z: i64) -> i64 {
        let mut rng = positional_rng(self.descriptor.seed, SeedStream::Terrain, x, 0, z);
        let span = (TERRAIN_AMPLITUDE * 2 + 1) as u64;
        TERRAIN_BASE_HEIGHT + (rng.next_below(span) as i64) - TERRAIN_AMPLITUDE
    }

    /// Generate one chunk from the world seed.
    ///
    /// # Errors
    ///
    /// Returns an error when section construction fails.
    pub fn generate_chunk(&self, coord: ChunkCoord) -> Result<Chunk> {
        let shape = self.descriptor.shape;
        let mut chunk = Chunk::new(coord, shape);
        chunk.transition_to(ChunkState::Requested)?;
        chunk.transition_to(ChunkState::Generating)?;

        let stone = self.block_id(&Identifier::parse("nexora:block/stone")?)?;
        let dirt = self.block_id(&Identifier::parse("nexora:block/dirt")?)?;
        let grass = self.block_id(&Identifier::parse("nexora:block/grass")?)?;

        let size_x = shape.size_x() as i64;
        let size_y = shape.size_y() as i64;
        let size_z = shape.size_z() as i64;
        let origin_x = coord.x * size_x;
        let origin_z = coord.z * size_z;

        // Heightmap first, so the vertical span to fill is known before any
        // section is allocated.
        let mut heights = Vec::with_capacity((size_x * size_z) as usize);
        for local_z in 0..size_z {
            for local_x in 0..size_x {
                heights.push(self.surface_height(origin_x + local_x, origin_z + local_z));
            }
        }
        let lowest = heights.iter().copied().min().unwrap_or(TERRAIN_BASE_HEIGHT);
        let highest = heights.iter().copied().max().unwrap_or(TERRAIN_BASE_HEIGHT);

        let first_section = self.descriptor.bounds.min_y.div_euclid(size_y);
        let last_section = highest.div_euclid(size_y);

        for section_y in first_section..=last_section {
            let section_min = section_y * size_y;
            let section_max = section_min + size_y - 1;

            // Wholly below the deepest soil: one uniform stone section, which
            // costs nothing to store.
            if section_max < lowest - SOIL_DEPTH {
                chunk.put_section(section_y, Section::uniform(shape, stone));
                continue;
            }
            // Wholly above the highest surface: leave it absent, which is air.
            if section_min > highest {
                continue;
            }

            let mut section = Section::empty(shape);
            for local_z in 0..size_z {
                for local_x in 0..size_x {
                    let surface = heights[(local_z * size_x + local_x) as usize];
                    for local_y in 0..size_y {
                        let world_y = section_min + local_y;
                        if world_y > surface {
                            continue;
                        }
                        let state = if world_y == surface {
                            grass
                        } else if world_y >= surface - SOIL_DEPTH {
                            dirt
                        } else {
                            stone
                        };
                        section.set(
                            nexora_foundation::spatial::LocalPos::new(
                                local_x as u32,
                                local_y as u32,
                                local_z as u32,
                            ),
                            state,
                        )?;
                    }
                }
            }
            section.compact();
            chunk.put_section(section_y, section);
        }

        chunk.transition_to(ChunkState::Generated)?;
        chunk.transition_to(ChunkState::Loaded)?;
        chunk.mark_clean();
        chunk.take_journal();
        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(seed: u64) -> World {
        World::create(
            WorldDescriptor::new("test-world", seed).expect("descriptor"),
            CalendarConfig::earthlike(),
        )
        .expect("world")
    }

    fn block(world: &World, path: &str) -> BlockStateId {
        world
            .block_id(&Identifier::parse(path).expect("identifier"))
            .expect("registered block")
    }

    #[test]
    fn air_is_pinned_to_runtime_id_zero() {
        // Section storage depends on this; if it ever changes, empty-section
        // detection silently starts counting air as solid.
        let world = world(1);
        assert_eq!(block(&world, "nexora:block/air"), AIR);
    }

    #[test]
    fn descriptors_reject_an_empty_name() {
        assert!(WorldDescriptor::new("", 1).is_err());
        assert!(WorldDescriptor::new("   ", 1).is_err());
        assert!(WorldDescriptor::new("valid", 1).is_ok());
    }

    #[test]
    fn the_lifecycle_runs_forward_and_supports_suspend_resume() {
        let mut world = world(1);
        assert_eq!(world.phase(), WorldPhase::Requested);

        world.bring_online().unwrap();
        assert_eq!(world.phase(), WorldPhase::Active);

        world.transition_to(WorldPhase::Suspended).unwrap();
        world.transition_to(WorldPhase::Active).unwrap();
        world.transition_to(WorldPhase::Suspended).unwrap();
        world.transition_to(WorldPhase::Closed).unwrap();

        // A closed world is finished.
        assert!(world.transition_to(WorldPhase::Active).is_err());
    }

    #[test]
    fn phases_cannot_be_skipped() {
        let mut world = world(1);
        let err = world
            .transition_to(WorldPhase::Active)
            .expect_err("must not skip");
        assert!(err.to_string().contains("from=Requested"), "{err}");
    }

    #[test]
    fn the_same_seed_generates_an_identical_chunk() {
        let first = world(0xC0FFEE);
        let second = world(0xC0FFEE);
        let coord = ChunkCoord::new(3, -5);

        let a = first.generate_chunk(coord).unwrap();
        let b = second.generate_chunk(coord).unwrap();
        assert_eq!(a, b, "the same seed must reproduce the same chunk");
    }

    #[test]
    fn a_different_seed_generates_a_different_world() {
        let a = world(1).generate_chunk(ChunkCoord::new(0, 0)).unwrap();
        let b = world(2).generate_chunk(ChunkCoord::new(0, 0)).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn generation_is_order_independent() {
        // Generating neighbours first must not change what a chunk contains.
        let world = world(42);
        let target = ChunkCoord::new(0, 0);

        let alone = world.generate_chunk(target).unwrap();
        for neighbour in [
            ChunkCoord::new(1, 0),
            ChunkCoord::new(0, 1),
            ChunkCoord::new(-1, -1),
        ] {
            let _ = world.generate_chunk(neighbour).unwrap();
        }
        let after_neighbours = world.generate_chunk(target).unwrap();

        assert_eq!(alone, after_neighbours);
    }

    #[test]
    fn generated_terrain_has_the_expected_layering() {
        let world = world(99);
        let chunk = world.generate_chunk(ChunkCoord::new(0, 0)).unwrap();

        let surface = world.surface_height(0, 0);
        let grass = block(&world, "nexora:block/grass");
        let dirt = block(&world, "nexora:block/dirt");
        let stone = block(&world, "nexora:block/stone");

        assert_eq!(chunk.get(BlockPos::new(0, surface, 0)).unwrap(), grass);
        assert_eq!(chunk.get(BlockPos::new(0, surface - 1, 0)).unwrap(), dirt);
        assert_eq!(
            chunk
                .get(BlockPos::new(0, surface - SOIL_DEPTH - 1, 0))
                .unwrap(),
            stone
        );
        assert_eq!(chunk.get(BlockPos::new(0, surface + 1, 0)).unwrap(), AIR);
        assert_eq!(chunk.get(BlockPos::new(0, 1900, 0)).unwrap(), AIR);
    }

    #[test]
    fn deep_underground_sections_cost_nothing_to_store() {
        // Uniform stone sections are the whole point of the representation:
        // 1900 blocks of solid rock must not allocate per-cell storage.
        let world = world(7);
        let chunk = world.generate_chunk(ChunkCoord::new(0, 0)).unwrap();

        let deep_section = (-1000i64).div_euclid(32);
        let section = chunk.section(deep_section).expect("deep sections exist");
        assert_eq!(section.storage_bytes(), 0);
        assert_eq!(
            section.uniform_state(),
            Some(block(&world, "nexora:block/stone"))
        );
    }

    #[test]
    fn terrain_stays_inside_its_amplitude() {
        let world = world(31_337);
        for x in -50..50i64 {
            for z in -50..50i64 {
                let height = world.surface_height(x, z);
                assert!(
                    (TERRAIN_BASE_HEIGHT - TERRAIN_AMPLITUDE
                        ..=TERRAIN_BASE_HEIGHT + TERRAIN_AMPLITUDE)
                        .contains(&height),
                    "height {height} at ({x},{z}) escaped the amplitude"
                );
            }
        }
    }

    #[test]
    fn reading_and_writing_requires_a_resident_chunk() {
        let mut world = world(5);
        world.bring_online().unwrap();
        let position = BlockPos::new(4, 70, 4);
        let stone = block(&world, "nexora:block/stone");

        assert!(world.get_block(position).is_err(), "no chunk is loaded yet");
        assert!(world.set_block(position, stone).is_err());

        world.load_or_generate(ChunkCoord::new(0, 0)).unwrap();
        world.set_block(position, stone).unwrap();
        assert_eq!(world.get_block(position).unwrap(), stone);
    }

    #[test]
    fn writing_outside_the_vertical_bounds_is_refused() {
        let mut world = world(5);
        world.bring_online().unwrap();
        world.load_or_generate(ChunkCoord::new(0, 0)).unwrap();
        let stone = block(&world, "nexora:block/stone");

        let err = world
            .set_block(BlockPos::new(0, 5_000, 0), stone)
            .expect_err("above the ceiling must be refused");
        assert!(err.to_string().contains("vertical bounds"), "{err}");
        assert!(world.set_block(BlockPos::new(0, -5_000, 0), stone).is_err());
    }

    #[test]
    fn unloading_returns_the_chunk_rather_than_discarding_edits() {
        let mut world = world(5);
        world.bring_online().unwrap();
        world.load_or_generate(ChunkCoord::new(0, 0)).unwrap();
        world
            .set_block(
                BlockPos::new(1, 100, 1),
                block(&world, "nexora:block/stone"),
            )
            .unwrap();

        let unloaded = world
            .unload_chunk(ChunkCoord::new(0, 0))
            .expect("chunk was resident");
        assert!(unloaded.is_dirty(), "the caller receives the unsaved edits");
        assert_eq!(world.chunk_count(), 0);
    }

    #[test]
    fn block_identifiers_round_trip_through_state_ids() {
        let world = world(1);
        for path in [
            "nexora:block/air",
            "nexora:block/stone",
            "nexora:block/dirt",
            "nexora:block/grass",
        ] {
            let id = Identifier::parse(path).unwrap();
            let state = world.block_id(&id).unwrap();
            assert_eq!(world.block_identifier(state), Some(id));
        }
        assert!(world
            .block_id(&Identifier::parse("example:block/ruby").unwrap())
            .is_err());
    }

    #[test]
    fn the_reproduction_key_pins_the_generator_version() {
        let world = world(11);
        let key = world.descriptor().reproduction_key();
        assert_eq!(key.world_seed, 11);
        assert_eq!(key.generator_version, GENERATOR_VERSION);
        assert_ne!(key.fingerprint(), 0);
    }

    #[test]
    fn world_bounds_are_validated() {
        assert!(WorldBounds {
            min_y: 10,
            max_y: 10
        }
        .validate()
        .is_err());
        assert!(WorldBounds {
            min_y: 10,
            max_y: -10
        }
        .validate()
        .is_err());
        assert!(WorldBounds::DEFAULT.validate().is_ok());
        assert!(WorldBounds::DEFAULT.contains_y(0));
        assert!(!WorldBounds::DEFAULT.contains_y(100_000));
    }
}
