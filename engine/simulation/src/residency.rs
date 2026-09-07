//! Voxel chunks as a streaming residency backend.
//!
//! Implements [`nexora_streaming::ResidencyBackend`] over a [`World`], which is
//! the only place that pairing is allowed to exist: `nexora-streaming` cannot
//! generate a chunk and `nexora-world` knows nothing about interest or budgets.
//!
//! ## What gets retained, and why it is only the edited chunks
//!
//! Generation is deterministic and position-seeded — `nexora-world` proves that
//! the same seed produces the same chunk regardless of the order chunks were
//! generated in. So an **unedited** chunk needs no storage at all: evicting it
//! and regenerating it later gives back the identical column.
//!
//! An **edited** chunk is different. Its content is not derivable from the seed
//! any more, so dropping it would lose the edit — which is precisely what
//! `WORLD CONTINUITY AND PLAYER INDEPENDENCE.md` §18 forbids. Those, and only
//! those, are held in [`RetainedChunks`].
//!
//! The consequence is worth stating plainly: retained memory grows with **how
//! much the world has been changed**, not with how far it has been explored.
//!
//! ## The trap this module exists to close
//!
//! An evicted chunk is not in the world. `nexora_world::persist::save` writes
//! *the world*. So saving while chunks are retained writes a save that is
//! missing every edit made in them — silently, because nothing is broken from
//! the world's point of view.
//!
//! [`RetainedChunks::flush_into`] is the answer, and it must be called before
//! any save. There is a test that saves without it and proves the edit is gone.

use std::collections::{BTreeMap, BTreeSet};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::ChunkCoord;
use nexora_streaming::backend::ResidencyBackend;
use nexora_streaming::lod::Lod;
use nexora_streaming::target::StreamTarget;
use nexora_world::chunk::Chunk;
use nexora_world::world::World;

/// Chunks that were evicted but whose content cannot be regenerated.
///
/// Lives across ticks, unlike [`WorldResidency`], which borrows the world for
/// the duration of one.
#[derive(Debug, Default)]
pub struct RetainedChunks {
    held: BTreeMap<ChunkCoord, Chunk>,
    /// Every column that has ever been edited. A chunk restored from here and
    /// evicted again looks clean, and dropping it would lose the edit a second
    /// time, so the fact that it was *ever* edited has to outlive its dirty bit.
    edited: BTreeSet<ChunkCoord>,
    generated: u64,
    restored: u64,
}

impl RetainedChunks {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many chunks are held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.held.len()
    }

    /// Whether nothing is held.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.held.is_empty()
    }

    /// Columns known to have been edited, held or not.
    #[must_use]
    pub fn edited_count(&self) -> usize {
        self.edited.len()
    }

    /// Chunks generated from the seed rather than restored.
    #[must_use]
    pub const fn generated(&self) -> u64 {
        self.generated
    }

    /// Chunks brought back from this store.
    #[must_use]
    pub const fn restored(&self) -> u64 {
        self.restored
    }

    /// Approximate bytes held outside the world.
    #[must_use]
    pub fn storage_bytes(&self) -> usize {
        self.held.values().map(Chunk::storage_bytes).sum()
    }

    /// Put every retained chunk back into the world.
    ///
    /// **Call this before saving.** A save written while chunks are retained is
    /// missing their edits. Returns how many chunks were returned.
    pub fn flush_into(&mut self, world: &mut World) -> usize {
        let count = self.held.len();
        for (_, chunk) in core::mem::take(&mut self.held) {
            world.insert_chunk(chunk);
        }
        count
    }
}

/// One tick's view of a world as a residency backend.
#[derive(Debug)]
pub struct WorldResidency<'a> {
    world: &'a mut World,
    retained: &'a mut RetainedChunks,
}

impl<'a> WorldResidency<'a> {
    /// Borrow a world and its retained chunks for one tick.
    pub fn new(world: &'a mut World, retained: &'a mut RetainedChunks) -> Self {
        Self { world, retained }
    }

    /// The world being streamed.
    #[must_use]
    pub fn world(&self) -> &World {
        self.world
    }

    fn coord(target: StreamTarget) -> ChunkCoord {
        target.column()
    }
}

impl ResidencyBackend for WorldResidency<'_> {
    fn activate(&mut self, target: StreamTarget, lod: Lod) -> Result<u64> {
        if !lod.is_resident() {
            // Regional and Abstract hold no distinct voxel data yet; the
            // regional simulation that would fill them is Phase 4.
            return Ok(0);
        }
        let coord = Self::coord(target);
        if self.world.chunk(coord).is_some() {
            return Ok(self.bytes_of(coord));
        }
        if let Some(chunk) = self.retained.held.remove(&coord) {
            self.world.insert_chunk(chunk);
            self.retained.restored += 1;
        } else {
            self.world.load_or_generate(coord)?;
            self.retained.generated += 1;
        }
        Ok(self.bytes_of(coord))
    }

    fn evict(&mut self, target: StreamTarget) -> Result<()> {
        let coord = Self::coord(target);
        // A chunk still here at eviction time is one `persist` did not take,
        // which means it is regenerable. Dropping it is the point.
        let _ = self.world.unload_chunk(coord);
        Ok(())
    }

    fn persist(&mut self, target: StreamTarget) -> Result<()> {
        let coord = Self::coord(target);
        let Some(chunk) = self.world.unload_chunk(coord) else {
            return Err(Error::new(
                Domain::World,
                "world-residency",
                "cannot persist a chunk that is not resident",
            )
            .with_recovery(Recovery::Reject)
            .with_context("chunk", format!("{},{}", coord.x, coord.z)));
        };
        self.retained.edited.insert(coord);
        self.retained.held.insert(coord, chunk);
        Ok(())
    }

    fn is_dirty(&self, target: StreamTarget) -> bool {
        let coord = Self::coord(target);
        if self.retained.edited.contains(&coord) {
            // Edited once, never regenerable again — however clean the dirty
            // bit looks after a save-and-restore cycle.
            return true;
        }
        self.world.chunk(coord).is_some_and(Chunk::is_dirty)
    }
}

impl WorldResidency<'_> {
    fn bytes_of(&self, coord: ChunkCoord) -> u64 {
        self.world
            .chunk(coord)
            .map(|chunk| chunk.storage_bytes() as u64)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::ident::Identifier;
    use nexora_foundation::spatial::BlockPos;
    use nexora_foundation::time::CalendarConfig;
    use nexora_streaming::budget::StreamingBudget;
    use nexora_streaming::interest::{InterestId, InterestSource, LodRadii};
    use nexora_streaming::system::StreamingSystem;
    use nexora_world::persist;
    use nexora_world::world::WorldDescriptor;

    fn world() -> World {
        let descriptor = WorldDescriptor::new("residency-test", 0xC0FFEE).expect("valid");
        let mut world = World::create(descriptor, CalendarConfig::earthlike()).expect("created");
        world.bring_online().expect("online");
        world
    }

    fn stone(world: &World) -> nexora_world::voxel::BlockStateId {
        world
            .block_id(&Identifier::parse("nexora:block/stone").expect("valid"))
            .expect("registered")
    }

    fn target(x: i64, z: i64) -> StreamTarget {
        StreamTarget::Chunk(ChunkCoord::new(x, z))
    }

    fn observer(at: ChunkCoord, radius: u32) -> InterestSource {
        InterestSource::new(InterestId(1), at)
            .with_radii(LodRadii::new(radius, radius, radius, 0).expect("valid"))
    }

    fn settle(system: &mut StreamingSystem, world: &mut World, retained: &mut RetainedChunks) {
        for _ in 0..64 {
            let mut backend = WorldResidency::new(world, retained);
            let report = system
                .tick(&mut backend, StreamingBudget::UNLIMITED)
                .expect("budget");
            if report.is_quiet() {
                return;
            }
        }
        panic!("streaming never settled");
    }

    #[test]
    fn activation_generates_a_chunk_and_eviction_releases_it() {
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let mut backend = WorldResidency::new(&mut world, &mut retained);

        let bytes = backend
            .activate(target(0, 0), Lod::Full)
            .expect("generated");
        assert!(bytes > 0, "a generated chunk should occupy storage");
        assert!(backend.world().chunk(ChunkCoord::new(0, 0)).is_some());

        backend.evict(target(0, 0)).expect("evicted");
        assert!(backend.world().chunk(ChunkCoord::new(0, 0)).is_none());
        assert_eq!(retained.generated(), 1);
        assert_eq!(retained.len(), 0, "a clean chunk must not be retained");
    }

    #[test]
    fn a_non_resident_tier_neither_generates_nor_costs() {
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let mut backend = WorldResidency::new(&mut world, &mut retained);
        for tier in [Lod::Regional, Lod::Abstract, Lod::Unresident] {
            assert_eq!(backend.activate(target(5, 5), tier).expect("ok"), 0);
            assert!(backend.world().chunk(ChunkCoord::new(5, 5)).is_none());
        }
        assert_eq!(retained.generated(), 0);
    }

    #[test]
    fn an_unedited_chunk_regenerates_byte_identically_after_eviction() {
        // The property that lets the backend throw clean chunks away. If it
        // ever stops holding, retention has to cover every explored column.
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let coord = ChunkCoord::new(3, -2);

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend
            .activate(target(3, -2), Lod::Full)
            .expect("generated");
        // Inside chunk (3, -2): at 32 columns per chunk that is x 96..=127,
        // z -64..=-33.
        let probe = |y: i64| BlockPos::new(100, y, -50);
        let before: Vec<_> = (0..16)
            .map(|y| world.get_block(probe(y)).expect("resident"))
            .collect();

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend.evict(target(3, -2)).expect("evicted");
        assert!(world.chunk(coord).is_none());

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend
            .activate(target(3, -2), Lod::Full)
            .expect("regenerated");
        let after: Vec<_> = (0..16)
            .map(|y| world.get_block(probe(y)).expect("resident"))
            .collect();

        assert_eq!(before, after);
        assert_eq!(retained.generated(), 2, "it was regenerated, not restored");
    }

    #[test]
    fn an_edited_chunk_is_retained_and_comes_back_with_its_edit() {
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let position = BlockPos::new(4, 200, 4);
        let block = stone(&world);

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend
            .activate(target(0, 0), Lod::Full)
            .expect("generated");
        world.set_block(position, block).expect("edited");

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        assert!(backend.is_dirty(target(0, 0)), "the edit was not noticed");
        backend.persist(target(0, 0)).expect("persisted");
        backend.evict(target(0, 0)).expect("evicted");
        assert_eq!(retained.len(), 1);
        assert!(world.chunk(ChunkCoord::new(0, 0)).is_none());

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend.activate(target(0, 0), Lod::Full).expect("restored");
        assert_eq!(retained.restored(), 1);
        assert_eq!(world.get_block(position).expect("resident"), block);
    }

    #[test]
    fn an_edited_chunk_stays_unregenerable_across_repeated_cycles() {
        // The trap: a restored chunk looks clean, so a second eviction would
        // drop it and the third activation would regenerate the terrain
        // without the edit.
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let position = BlockPos::new(4, 200, 4);
        let block = stone(&world);

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend
            .activate(target(0, 0), Lod::Full)
            .expect("generated");
        world.set_block(position, block).expect("edited");

        for cycle in 0..4 {
            let mut backend = WorldResidency::new(&mut world, &mut retained);
            assert!(
                backend.is_dirty(target(0, 0)),
                "cycle {cycle}: an edited chunk reported itself regenerable"
            );
            backend.persist(target(0, 0)).expect("persisted");
            backend.evict(target(0, 0)).expect("evicted");

            let mut backend = WorldResidency::new(&mut world, &mut retained);
            backend.activate(target(0, 0), Lod::Full).expect("restored");
            assert_eq!(
                world.get_block(position).expect("resident"),
                block,
                "cycle {cycle}: the edit was lost"
            );
        }
    }

    #[test]
    fn persisting_a_chunk_that_is_not_resident_is_refused() {
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let mut backend = WorldResidency::new(&mut world, &mut retained);
        let err = backend.persist(target(9, 9)).expect_err("not resident");
        assert_eq!(err.domain(), Domain::World);
    }

    #[test]
    fn a_save_written_while_chunks_are_retained_loses_their_edits() {
        // Documents the trap `flush_into` exists to close. If this ever starts
        // failing, the world has learned to save retained chunks by itself and
        // the flush is no longer load-bearing.
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let position = BlockPos::new(4, 200, 4);
        let block = stone(&world);

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend
            .activate(target(0, 0), Lod::Full)
            .expect("generated");
        world.set_block(position, block).expect("edited");
        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend.persist(target(0, 0)).expect("persisted");
        backend.evict(target(0, 0)).expect("evicted");

        let container = persist::save(&world).expect("saved");
        let reloaded = persist::load(&container).expect("loaded");
        assert_eq!(
            reloaded.chunk_count(),
            0,
            "the retained chunk should be absent from an unflushed save"
        );
    }

    #[test]
    fn flushing_before_the_save_carries_every_edit_across() {
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let position = BlockPos::new(4, 200, 4);
        let block = stone(&world);

        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend
            .activate(target(0, 0), Lod::Full)
            .expect("generated");
        world.set_block(position, block).expect("edited");
        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend.persist(target(0, 0)).expect("persisted");
        backend.evict(target(0, 0)).expect("evicted");

        assert_eq!(retained.flush_into(&mut world), 1);
        assert!(retained.is_empty());

        let container = persist::save(&world).expect("saved");
        let reloaded = persist::load(&container).expect("loaded");
        assert_eq!(
            reloaded.get_block(position).expect("resident"),
            block,
            "the edit did not survive the save"
        );
    }

    #[test]
    fn streaming_a_moving_observer_over_a_world_keeps_edits_and_bounds_retention() {
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let mut system = StreamingSystem::new();
        let block = stone(&world);

        // Walk out, editing one column on the way.
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 1))
            .expect("source");
        settle(&mut system, &mut world, &mut retained);
        let edited = BlockPos::new(4, 300, 4);
        world.set_block(edited, block).expect("edited");

        for step in 1..=6 {
            system
                .set_interest(observer(ChunkCoord::new(step * 3, 0), 1))
                .expect("source");
            settle(&mut system, &mut world, &mut retained);
        }

        assert!(
            world.chunk(ChunkCoord::new(0, 0)).is_none(),
            "still resident"
        );
        assert_eq!(
            retained.len(),
            1,
            "retention should hold the edited column and nothing else"
        );
        assert!(
            retained.generated() > 10,
            "the walk should have generated many columns"
        );

        // Walk back; the edit is there.
        system
            .set_interest(observer(ChunkCoord::new(0, 0), 1))
            .expect("source");
        settle(&mut system, &mut world, &mut retained);
        assert_eq!(world.get_block(edited).expect("resident"), block);
        assert_eq!(retained.restored(), 1);
        assert!(retained.is_empty(), "the restored chunk is still held");
    }
}
