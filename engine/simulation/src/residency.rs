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
//!
//! ## Where retained chunks can live instead of memory
//!
//! Holding them in a `BTreeMap` bounds the growth by *how much was edited*
//! rather than by how far the world was explored, which is the right shape —
//! and it is still memory, and it still dies with the process. `DEBT-0020`
//! measured it at **20.0 KiB per edited column**.
//!
//! [`RetainedChunks::backed_by`] attaches a [`RegionStore`] (ADR-0014).
//! Eviction still hands the chunk here, because eviction runs inside the
//! streaming budget and a file write does not belong there; what changes is
//! that [`RetainedChunks::flush_to_store`] writes the held columns into their
//! region files and drops them, and [`ResidencyBackend::activate`] reads a
//! column back from disk when memory no longer has it. Memory is then bounded
//! by *what was evicted since the last flush*, not by everything ever edited.

use std::collections::{BTreeMap, BTreeSet};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::ChunkCoord;
use nexora_streaming::backend::ResidencyBackend;
use nexora_streaming::lod::Lod;
use nexora_streaming::target::StreamTarget;
use nexora_world::chunk::Chunk;
use nexora_world::recovery::PendingEdits;
use nexora_world::region::RegionStore;
use nexora_world::world::World;

/// What one flush to the region store moved.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlushReport {
    /// Columns written out and dropped from memory.
    pub columns: usize,
    /// Region files touched. Columns that share a region cost one write
    /// between them, which is what makes a per-tick flush affordable: the
    /// expensive part is the file, not the column.
    pub regions: usize,
    /// Bytes written across those files.
    pub bytes: u64,
}

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
    /// Where held columns go when they are flushed, if anywhere.
    store: Option<RegionStore>,
    generated: u64,
    restored: u64,
    flushed: u64,
    read_back: u64,
}

impl RetainedChunks {
    /// An empty store, holding evicted columns in memory only.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty store whose held columns can be flushed to region files.
    ///
    /// The store is where a flushed column lives and where
    /// [`ResidencyBackend::activate`] looks for it when memory does not have
    /// it. Attaching one changes nothing until [`Self::flush_to_store`] is
    /// called: eviction runs inside the streaming budget, and a file write does
    /// not belong there.
    #[must_use]
    pub fn backed_by(store: RegionStore) -> Self {
        Self {
            store: Some(store),
            ..Self::default()
        }
    }

    /// The region store backing this one, if any.
    #[must_use]
    pub const fn store(&self) -> Option<&RegionStore> {
        self.store.as_ref()
    }

    /// Columns written out to region files and dropped from memory.
    #[must_use]
    pub const fn flushed(&self) -> u64 {
        self.flushed
    }

    /// Columns brought back from a region file rather than from memory.
    #[must_use]
    pub const fn read_back(&self) -> u64 {
        self.read_back
    }

    /// Write every held column into its region file and drop it from memory.
    ///
    /// Without a store attached this is a no-op reporting nothing, so a caller
    /// can flush unconditionally.
    ///
    /// The columns are dropped **only after** the files are written and
    /// verified: a failed write leaves them held, which is recoverable, where
    /// dropping first would lose the edit outright. `world` supplies the
    /// identifier palette the region files are written with.
    ///
    /// # Errors
    ///
    /// Returns an error when a region file cannot be read, written or verified.
    pub fn flush_to_store(&mut self, world: &World) -> Result<FlushReport> {
        let Some(store) = self.store.as_ref() else {
            return Ok(FlushReport::default());
        };
        if self.held.is_empty() {
            return Ok(FlushReport::default());
        }

        let columns: Vec<Chunk> = self.held.values().cloned().collect();
        let count = columns.len();
        let written = store.store_columns(world, columns)?;

        self.held.clear();
        self.flushed += count as u64;
        Ok(FlushReport {
            columns: count,
            regions: written.written.len(),
            bytes: written.bytes,
        })
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

    /// Put every retained chunk back into the world, wherever it is held.
    ///
    /// **Call this before a single-container save.** A save written while
    /// chunks are retained is missing their edits. Returns how many chunks were
    /// returned.
    ///
    /// With a store attached this also reads back the columns
    /// [`Self::flush_to_store`] wrote out, because "every retained chunk" has
    /// to keep meaning that — otherwise a flush would quietly empty the set
    /// this call looks at, and the save would be wrong in exactly the way the
    /// call exists to prevent. A caller whose save *is* the region store does
    /// not call this at all: those columns are already in their region files.
    ///
    /// # Errors
    ///
    /// Returns an error when a region file cannot be read or fails
    /// verification.
    pub fn flush_into(&mut self, world: &mut World) -> Result<usize> {
        let mut count = self.held.len();
        for (_, chunk) in core::mem::take(&mut self.held) {
            world.insert_chunk(chunk);
        }

        if let Some(store) = self.store.as_ref() {
            // Only columns that were evicted are in `edited`, and one that has
            // since been activated is resident again and needs nothing.
            let absent: Vec<ChunkCoord> = self
                .edited
                .iter()
                .copied()
                .filter(|coord| world.chunk(*coord).is_none())
                .collect();
            for coord in absent {
                if let Some(chunk) = store.read_column(world, coord)? {
                    world.insert_chunk(chunk);
                    self.read_back += 1;
                    count += 1;
                }
            }
        }
        Ok(count)
    }
}

/// One tick's view of a world as a residency backend.
#[derive(Debug)]
pub struct WorldResidency<'a> {
    world: &'a mut World,
    retained: &'a mut RetainedChunks,
    pending: Option<&'a mut PendingEdits>,
}

impl<'a> WorldResidency<'a> {
    /// Borrow a world and its retained chunks for one tick.
    pub fn new(world: &'a mut World, retained: &'a mut RetainedChunks) -> Self {
        Self {
            world,
            retained,
            pending: None,
        }
    }

    /// Also finish a recovery whose columns were not all resident at load.
    ///
    /// `recovery::apply` files the edits it could not place against the column
    /// they are waiting for. Handing that index here is what finishes the job:
    /// each column that becomes resident gets its edits applied at that moment,
    /// in the order they were journalled, before anything else can read it.
    ///
    /// Without this the edits are not lost — they stay in the journal and the
    /// recovery report lists them — but nobody ever re-applies them, which is
    /// what `DEBT-0024` was.
    #[must_use]
    pub fn recovering(mut self, pending: &'a mut PendingEdits) -> Self {
        self.pending = Some(pending);
        self
    }

    /// Apply whatever was waiting on a column that has just become resident.
    ///
    /// An edit that is still refused with the column resident is not waiting on
    /// residency; it is surfaced as an error rather than dropped, because the
    /// alternative is a world that quietly differs from its journal.
    fn finish_recovery(&mut self, coord: ChunkCoord) -> Result<()> {
        let Some(pending) = self.pending.as_deref_mut() else {
            return Ok(());
        };
        let recovered = pending.apply_column(self.world, coord)?;
        if recovered.refused.is_empty() {
            return Ok(());
        }
        Err(Error::new(
            Domain::World,
            "world-residency",
            "a recovered edit was refused by a column that is now resident",
        )
        .with_recovery(Recovery::Reject)
        .with_context("chunk", format!("{},{}", coord.x, coord.z))
        .with_context("refused", recovered.refused.len().to_string()))
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
        } else if let Some(chunk) = self.read_back(coord)? {
            self.world.insert_chunk(chunk);
            self.retained.restored += 1;
            self.retained.read_back += 1;
        } else {
            self.world.load_or_generate(coord)?;
            self.retained.generated += 1;
        }
        // Before the caller can read the column, and before anything can be
        // evicted again: a column that arrives already carries its journalled
        // edits or it never will.
        self.finish_recovery(coord)?;
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
    /// Look for a flushed column in the region store.
    ///
    /// Only columns known to have been edited are ever stored, so the `edited`
    /// set is what decides whether to touch the disk at all. Without that gate
    /// every generated column would cost a file read to learn that the store
    /// has nothing for it.
    fn read_back(&self, coord: ChunkCoord) -> Result<Option<Chunk>> {
        let Some(store) = self.retained.store.as_ref() else {
            return Ok(None);
        };
        if !self.retained.edited.contains(&coord) {
            return Ok(None);
        }
        store.read_column(self.world, coord)
    }

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

    /// `DEBT-0024`, from the streaming side. A journal that names a column this
    /// session has not loaded is not a loss: residency is where the column
    /// arrives, so residency is where the edit lands.
    #[test]
    fn a_column_streamed_in_carries_the_edits_that_were_waiting_for_it() {
        use nexora_world::recovery::{apply, EditRecord, JournalReplay};

        let mut world = world();
        let far = ChunkCoord::new(4, 0);
        let position = world
            .descriptor()
            .shape
            .section_origin(nexora_foundation::spatial::SectionCoord::new(4, 2, 0))
            .expect("in range");

        // Replayed against a world that has loaded nothing: the edit cannot
        // land, and is filed against the column it needs.
        let record = EditRecord::SetBlock {
            position,
            block: Identifier::parse("nexora:block/stone").expect("valid"),
        };
        let report = apply(
            &mut world,
            &JournalReplay {
                records: vec![record.encode()],
                damage: None,
            },
        )
        .expect("replayed");
        assert_eq!(report.applied, 0);
        let mut pending = report.deferred;
        assert_eq!(pending.waiting_on(far), 1);

        // Now let streaming bring the world in, with the index attached.
        let mut retained = RetainedChunks::new();
        let mut system = StreamingSystem::new();
        system
            .set_interest(observer(ChunkCoord::new(2, 0), 3))
            .expect("source");
        for _ in 0..64 {
            let mut backend =
                WorldResidency::new(&mut world, &mut retained).recovering(&mut pending);
            let report = system
                .tick(&mut backend, StreamingBudget::UNLIMITED)
                .expect("budget");
            if report.is_quiet() {
                break;
            }
        }

        assert!(
            world.chunk(far).is_some(),
            "the observer's radius has to reach the column for this to test anything"
        );
        assert_eq!(pending.applied(), 1, "the waiting edit was never applied");
        assert!(pending.is_empty());
        let found = world.get_block(position).expect("resident");
        assert_eq!(
            world.block_identifier(found),
            Some(Identifier::parse("nexora:block/stone").expect("valid")),
            "the column arrived without the edit that was waiting for it"
        );
    }

    /// The index is opt-in, and a backend built without one behaves exactly as
    /// it did before it existed.
    #[test]
    fn residency_without_an_index_neither_recovers_nor_complains() {
        let mut world = world();
        let mut retained = RetainedChunks::new();
        let mut backend = WorldResidency::new(&mut world, &mut retained);
        assert!(backend.activate(target(0, 0), Lod::Full).is_ok());
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

        assert_eq!(retained.flush_into(&mut world).expect("flushed"), 1);
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

    /// A scratch directory that cleans itself up.
    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let base = std::env::temp_dir().join(format!(
                "nexora-residency-{label}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = std::fs::remove_dir_all(&base);
            std::fs::create_dir_all(&base).expect("create scratch directory");
            Self(base)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Two columns per region: at the engine default of 32 every column in
    /// these tests shares one region, and a flush could not be told from a
    /// whole-world write.
    fn small_store(dir: &TempDir) -> RegionStore {
        RegionStore::with_shape(
            &dir.0,
            nexora_foundation::spatial::RegionShape::new(2).expect("two columns per region"),
        )
    }

    /// Evict one edited column, with the given retention.
    fn evict_one_edited(
        world: &mut World,
        retained: &mut RetainedChunks,
    ) -> (ChunkCoord, BlockPos) {
        let coord = ChunkCoord::new(3, 0);
        world.load_or_generate(coord).expect("generate");
        let position = world
            .descriptor()
            .shape
            .section_origin(nexora_foundation::spatial::SectionCoord::new(3, 2, 0))
            .expect("in range");
        let block = stone(world);
        world.set_block(position, block).expect("edit");

        let mut backend = WorldResidency::new(world, retained);
        backend.persist(target(3, 0)).expect("persisted");
        (coord, position)
    }

    #[test]
    fn a_flushed_column_leaves_memory_and_comes_back_from_its_region_file() {
        let dir = TempDir::new("flush");
        let mut world = world();
        let mut retained = RetainedChunks::backed_by(small_store(&dir));
        let (coord, position) = evict_one_edited(&mut world, &mut retained);
        let block = stone(&world);

        assert_eq!(retained.len(), 1, "eviction holds it in memory first");
        assert_eq!(retained.flush_to_store(&world).expect("flushed").columns, 1);
        assert!(
            retained.is_empty(),
            "a flushed column is no longer costing memory"
        );
        assert_eq!(retained.storage_bytes(), 0);
        assert_eq!(retained.flushed(), 1);

        // It has to come back from disk, because memory does not have it.
        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend
            .activate(target(3, 0), Lod::Full)
            .expect("activated");
        assert_eq!(retained.read_back(), 1, "it was read, not regenerated");
        assert_eq!(retained.generated(), 0);
        assert_eq!(world.get_block(position).expect("resident"), block);
        assert_eq!(world.chunk(coord).map(Chunk::is_dirty), Some(false));
    }

    /// The whole point of `DEBT-0020`: the held bytes stop growing.
    #[test]
    fn flushing_bounds_retention_by_what_was_evicted_since_the_last_flush() {
        let dir = TempDir::new("bounded");
        let mut world = world();
        let mut retained = RetainedChunks::backed_by(small_store(&dir));
        let block = stone(&world);

        let mut peak_without_flush = 0;
        for x in 0..6i64 {
            let coord = ChunkCoord::new(x, 7);
            world.load_or_generate(coord).expect("generate");
            let position = world
                .descriptor()
                .shape
                .section_origin(nexora_foundation::spatial::SectionCoord::new(x, 2, 7))
                .expect("in range");
            world.set_block(position, block).expect("edit");

            let mut backend = WorldResidency::new(&mut world, &mut retained);
            backend
                .persist(StreamTarget::Chunk(coord))
                .expect("persisted");
            peak_without_flush = peak_without_flush.max(retained.len());
            retained.flush_to_store(&world).expect("flushed");
            assert_eq!(
                retained.len(),
                0,
                "nothing accumulates across evictions once each is flushed"
            );
        }

        assert_eq!(peak_without_flush, 1, "one column held at a time");
        assert_eq!(retained.flushed(), 6);
        assert_eq!(
            retained.edited_count(),
            6,
            "the fact of the edit still lives here"
        );

        // Every one of them is readable again, from three different regions.
        for x in 0..6i64 {
            let mut backend = WorldResidency::new(&mut world, &mut retained);
            backend
                .activate(StreamTarget::Chunk(ChunkCoord::new(x, 7)), Lod::Full)
                .expect("activated");
        }
        assert_eq!(retained.read_back(), 6);
        assert_eq!(retained.generated(), 0, "not one of them was regenerated");
    }

    /// `flush_into` has to keep meaning "every retained chunk", or a flush
    /// silently empties the set it looks at and the container save goes out
    /// missing the edits — the exact trap this module was built to close.
    #[test]
    fn flush_into_brings_back_the_columns_that_went_to_disk() {
        let dir = TempDir::new("flushback");
        let mut world = world();
        let mut retained = RetainedChunks::backed_by(small_store(&dir));
        let (coord, position) = evict_one_edited(&mut world, &mut retained);
        let block = stone(&world);

        retained.flush_to_store(&world).expect("flushed");
        assert!(retained.is_empty());
        assert!(
            world.chunk(coord).is_none(),
            "the column is not in the world"
        );

        assert_eq!(
            retained.flush_into(&mut world).expect("flushed in"),
            1,
            "a column on disk is still a retained column"
        );
        assert_eq!(world.get_block(position).expect("resident"), block);

        // And the container save now carries it, which is the thing that was
        // at stake.
        let container = persist::save(&world).expect("saved");
        let reloaded = persist::load(&container).expect("loaded");
        assert_eq!(reloaded.get_block(position).expect("resident"), block);
    }

    #[test]
    fn a_column_that_was_never_edited_is_regenerated_rather_than_looked_for() {
        let dir = TempDir::new("nolookup");
        let store = small_store(&dir);
        let mut world = world();

        // The store is written whole first, so the column really is on disk:
        // the choice under test is to regenerate it anyway, not an absence of
        // anything to find. Generation is deterministic, so both answers are
        // the same bytes and the cheaper one wins.
        world
            .load_or_generate(ChunkCoord::new(9, 9))
            .expect("generate");
        store.write_all(&mut world).expect("write");
        world.unload_chunk(ChunkCoord::new(9, 9)).expect("resident");
        assert!(store
            .read_column(&world, ChunkCoord::new(9, 9))
            .expect("read")
            .is_some());

        let mut retained = RetainedChunks::backed_by(store);
        let mut backend = WorldResidency::new(&mut world, &mut retained);
        backend
            .activate(target(9, 9), Lod::Full)
            .expect("activated");
        assert_eq!(retained.generated(), 1);
        assert_eq!(
            retained.read_back(),
            0,
            "an unedited column is regenerated, not read from disk"
        );
    }

    /// What makes a per-tick flush affordable: the expensive thing is the
    /// region file, and columns that share one are written together. A count,
    /// so it reads the same on every machine.
    #[test]
    fn columns_sharing_a_region_cost_one_write_between_them() {
        let dir = TempDir::new("batched");
        let mut world = world();
        let mut retained = RetainedChunks::backed_by(small_store(&dir));
        let block = stone(&world);

        // Eight columns, two per region on each axis: x in 0..8 at z = 5 falls
        // into regions (0,2), (1,2), (2,2) and (3,2).
        for x in 0..8i64 {
            let coord = ChunkCoord::new(x, 5);
            world.load_or_generate(coord).expect("generate");
            let position = world
                .descriptor()
                .shape
                .section_origin(nexora_foundation::spatial::SectionCoord::new(x, 2, 5))
                .expect("in range");
            world.set_block(position, block).expect("edit");
            let mut backend = WorldResidency::new(&mut world, &mut retained);
            backend
                .persist(StreamTarget::Chunk(coord))
                .expect("persisted");
        }

        let report = retained.flush_to_store(&world).expect("flushed");
        assert_eq!(report.columns, 8);
        assert_eq!(
            report.regions, 4,
            "eight columns, two to a region, is four writes and not eight"
        );
        assert!(report.bytes > 0);
    }

    #[test]
    fn without_a_store_a_flush_is_a_no_op_and_nothing_changes() {
        let mut world = world();
        let mut retained = RetainedChunks::new();
        evict_one_edited(&mut world, &mut retained);

        assert_eq!(
            retained.flush_to_store(&world).expect("no store"),
            FlushReport::default()
        );
        assert_eq!(retained.len(), 1, "memory is still the only place it lives");
        assert_eq!(retained.flushed(), 0);
        assert!(retained.store().is_none());
    }
}
