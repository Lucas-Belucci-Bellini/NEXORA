//! Chunk columns and their lifecycle.
//!
//! Implements `CHUNK & VOXEL ENGINE.md` §13-§14 (chunk state and lifecycle),
//! §33-§34 (change tracking and the change journal) and §4 (vertical sections).
//!
//! A chunk is a **column**: one `(x, z)` footprint holding a sparse stack of
//! cubic sections indexed by `y`. An absent section is exactly air, which is the
//! air optimization from §9 - a world with a 3840-block vertical range does not
//! allocate storage for the empty majority of it.

use std::collections::{BTreeMap, BTreeSet};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{BlockPos, ChunkCoord, ChunkShape, SectionCoord};
use nexora_foundation::time::WorldTime;

use crate::voxel::{BlockStateId, Section, AIR};

/// Largest number of changes a chunk keeps before dropping the oldest.
///
/// `NEXORA PERFORMANCE BUDGETS.md` forbids unbounded growth in recurring loops.
/// An edited chunk would otherwise accumulate journal entries for as long as the
/// world is loaded.
pub const MAX_JOURNAL_ENTRIES: usize = 4_096;

/// Runtime state of a chunk (`CHUNK & VOXEL ENGINE.md` §13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkState {
    /// Not in memory.
    Unloaded,
    /// Queued for loading.
    Requested,
    /// Being read from storage.
    Loading,
    /// Being generated because no stored copy exists.
    Generating,
    /// Generation finished.
    Generated,
    /// In memory and readable.
    Loaded,
    /// In memory and being simulated.
    Active,
    /// In memory but not simulated.
    Inactive,
    /// Being written out and removed.
    Unloading,
    /// Written to storage.
    Saved,
    /// Failed; must not be treated as usable data.
    Error,
}

impl ChunkState {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unloaded => "unloaded",
            Self::Requested => "requested",
            Self::Loading => "loading",
            Self::Generating => "generating",
            Self::Generated => "generated",
            Self::Loaded => "loaded",
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Unloading => "unloading",
            Self::Saved => "saved",
            Self::Error => "error",
        }
    }

    /// Whether the chunk's voxels may be read or written.
    #[must_use]
    pub const fn is_resident(self) -> bool {
        matches!(
            self,
            Self::Generated | Self::Loaded | Self::Active | Self::Inactive | Self::Saved
        )
    }

    /// Whether `next` is a legal transition from this state.
    #[must_use]
    pub const fn can_transition_to(self, next: Self) -> bool {
        // Error is always reachable: any step may fail. Nothing leaves Error
        // except a fresh load, because a chunk that failed is not trustworthy
        // data and must be re-read rather than patched up.
        if matches!(next, Self::Error) {
            return true;
        }
        match self {
            Self::Unloaded => matches!(next, Self::Requested),
            Self::Requested => matches!(next, Self::Loading | Self::Generating),
            Self::Loading => matches!(next, Self::Loaded | Self::Generating),
            Self::Generating => matches!(next, Self::Generated),
            Self::Generated => matches!(next, Self::Loaded),
            Self::Loaded => matches!(next, Self::Active | Self::Inactive | Self::Unloading),
            Self::Active => matches!(next, Self::Inactive | Self::Unloading),
            Self::Inactive => matches!(next, Self::Active | Self::Unloading),
            Self::Unloading => matches!(next, Self::Saved | Self::Unloaded),
            Self::Saved => matches!(next, Self::Unloaded | Self::Loaded),
            Self::Error => matches!(next, Self::Unloaded | Self::Requested),
        }
    }
}

/// One recorded voxel mutation (`CHUNK & VOXEL ENGINE.md` §34).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoxelChange {
    /// Where the change happened.
    pub position: BlockPos,
    /// The state that was replaced.
    pub previous: BlockStateId,
    /// The state now in place.
    pub next: BlockStateId,
    /// World time of the change.
    pub at: WorldTime,
}

/// A column of voxel sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    coord: ChunkCoord,
    shape: ChunkShape,
    sections: BTreeMap<i64, Section>,
    state: ChunkState,
    dirty_sections: BTreeSet<i64>,
    journal: Vec<VoxelChange>,
    dropped_journal_entries: u64,
}

impl Chunk {
    /// Create an empty chunk in [`ChunkState::Unloaded`].
    #[must_use]
    pub fn new(coord: ChunkCoord, shape: ChunkShape) -> Self {
        Self {
            coord,
            shape,
            sections: BTreeMap::new(),
            state: ChunkState::Unloaded,
            dirty_sections: BTreeSet::new(),
            journal: Vec::new(),
            dropped_journal_entries: 0,
        }
    }

    /// The column address.
    #[must_use]
    pub const fn coord(&self) -> ChunkCoord {
        self.coord
    }

    /// The section extent used by this chunk.
    #[must_use]
    pub const fn shape(&self) -> ChunkShape {
        self.shape
    }

    /// The current runtime state.
    #[must_use]
    pub const fn state(&self) -> ChunkState {
        self.state
    }

    /// How many sections hold storage. Absent sections are air.
    #[must_use]
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// The `y` index of every stored section, ascending.
    #[must_use]
    pub fn section_indices(&self) -> Vec<i64> {
        self.sections.keys().copied().collect()
    }

    /// Total non-air cells across every stored section.
    #[must_use]
    pub fn non_air_count(&self) -> u64 {
        self.sections
            .values()
            .map(|section| u64::from(section.non_air_count()))
            .sum()
    }

    /// Approximate heap cost of this chunk's voxel storage, in bytes.
    #[must_use]
    pub fn storage_bytes(&self) -> usize {
        self.sections.values().map(Section::storage_bytes).sum()
    }

    /// Move to a new lifecycle state.
    ///
    /// # Errors
    ///
    /// Returns an error for an illegal transition, naming both states. An
    /// out-of-order transition means a caller is treating a chunk as ready when
    /// it is not, which is worth failing loudly over.
    pub fn transition_to(&mut self, next: ChunkState) -> Result<()> {
        if !self.state.can_transition_to(next) {
            return Err(self
                .error("illegal chunk state transition")
                .with_recovery(Recovery::Manual)
                .with_context("from", self.state.as_str())
                .with_context("to", next.as_str()));
        }
        self.state = next;
        Ok(())
    }

    /// Force a state without checking the transition.
    ///
    /// Only for reconstructing a chunk from storage, where the recorded state is
    /// the starting point rather than a step.
    pub(crate) fn force_state(&mut self, state: ChunkState) {
        self.state = state;
    }

    /// Read one block.
    ///
    /// # Errors
    ///
    /// Returns an error when the position is not inside this column.
    pub fn get(&self, position: BlockPos) -> Result<BlockStateId> {
        let (section_y, local) = self.locate(position)?;
        self.sections
            .get(&section_y)
            .map_or(Ok(AIR), |section| section.get(local))
    }

    /// Write one block and return what it replaced.
    ///
    /// Writing air into a section that does not exist is a no-op: absent already
    /// means air, so no storage is allocated for it.
    ///
    /// # Errors
    ///
    /// Returns an error when the position is not inside this column, or when the
    /// section storage refuses the write.
    pub fn set(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        at: WorldTime,
    ) -> Result<BlockStateId> {
        let (section_y, local) = self.locate(position)?;

        if state == AIR && !self.sections.contains_key(&section_y) {
            return Ok(AIR);
        }

        let shape = self.shape;
        let section = self
            .sections
            .entry(section_y)
            .or_insert_with(|| Section::empty(shape));
        let previous = section.set(local, state)?;

        if previous != state {
            self.dirty_sections.insert(section_y);
            self.record(VoxelChange {
                position,
                previous,
                next: state,
                at,
            });
        }
        Ok(previous)
    }

    /// Replace a whole section at once.
    ///
    /// Used by generation and by loading, where writing cell by cell would
    /// produce a journal entry per voxel.
    pub fn put_section(&mut self, section_y: i64, section: Section) {
        if section.is_empty() && section.uniform_state() == Some(AIR) {
            // Keep the sparse representation honest: an all-air section is
            // stored as absence, not as an allocated block of zeroes.
            self.sections.remove(&section_y);
        } else {
            self.sections.insert(section_y, section);
        }
        self.dirty_sections.insert(section_y);
    }

    /// Borrow a stored section.
    #[must_use]
    pub fn section(&self, section_y: i64) -> Option<&Section> {
        self.sections.get(&section_y)
    }

    /// Whether anything has changed since the last [`Chunk::mark_clean`].
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        !self.dirty_sections.is_empty()
    }

    /// Which sections have changed.
    #[must_use]
    pub fn dirty_sections(&self) -> Vec<i64> {
        self.dirty_sections.iter().copied().collect()
    }

    /// Clear the dirty set, after a successful save.
    pub fn mark_clean(&mut self) {
        self.dirty_sections.clear();
    }

    /// The recorded changes, oldest first.
    #[must_use]
    pub fn journal(&self) -> &[VoxelChange] {
        &self.journal
    }

    /// How many journal entries were dropped to stay within the cap.
    ///
    /// A non-zero value means the journal is no longer a complete record, which
    /// a consumer needs to know rather than discover.
    #[must_use]
    pub const fn dropped_journal_entries(&self) -> u64 {
        self.dropped_journal_entries
    }

    /// Drain the journal, e.g. after forwarding it to history or replication.
    pub fn take_journal(&mut self) -> Vec<VoxelChange> {
        self.dropped_journal_entries = 0;
        std::mem::take(&mut self.journal)
    }

    /// Collapse every section to its smallest representation.
    pub fn compact(&mut self) {
        let mut emptied = Vec::new();
        for (section_y, section) in &mut self.sections {
            section.compact();
            if section.is_empty() && section.uniform_state() == Some(AIR) {
                emptied.push(*section_y);
            }
        }
        for section_y in emptied {
            self.sections.remove(&section_y);
        }
    }

    /// Resolve a world position to `(section index, section-local offset)`.
    fn locate(&self, position: BlockPos) -> Result<(i64, nexora_foundation::spatial::LocalPos)> {
        let section = self.shape.section_of(position);
        if section.column() != self.coord {
            return Err(self
                .error("block position belongs to a different chunk column")
                .with_recovery(Recovery::Reject)
                .with_context(
                    "position",
                    format!("{},{},{}", position.x, position.y, position.z),
                )
                .with_context("resolved_column", format!("{},{}", section.x, section.z)));
        }
        Ok((section.y, self.shape.local_of(position)))
    }

    fn record(&mut self, change: VoxelChange) {
        if self.journal.len() >= MAX_JOURNAL_ENTRIES {
            self.journal.remove(0);
            self.dropped_journal_entries += 1;
        }
        self.journal.push(change);
    }

    fn error(&self, message: &'static str) -> Error {
        Error::new(Domain::Chunk, "chunk", message)
            .with_context("chunk", format!("{},{}", self.coord.x, self.coord.z))
    }
}

/// The section address of a block within a chunk column.
#[must_use]
pub fn section_of(shape: ChunkShape, position: BlockPos) -> SectionCoord {
    shape.section_of(position)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STONE: BlockStateId = BlockStateId(1);
    const DIRT: BlockStateId = BlockStateId(2);

    fn chunk_at(x: i64, z: i64) -> Chunk {
        Chunk::new(ChunkCoord::new(x, z), ChunkShape::cubic_default())
    }

    /// A block position inside column `(x, z)`.
    fn inside(x: i64, z: i64, local_x: i64, y: i64, local_z: i64) -> BlockPos {
        BlockPos::new(x * 32 + local_x, y, z * 32 + local_z)
    }

    #[test]
    fn a_new_chunk_is_entirely_air_and_stores_nothing() {
        let chunk = chunk_at(0, 0);
        assert_eq!(chunk.section_count(), 0);
        assert_eq!(chunk.storage_bytes(), 0);
        assert_eq!(chunk.non_air_count(), 0);
        assert_eq!(chunk.get(inside(0, 0, 5, 100, 5)).unwrap(), AIR);
        assert!(!chunk.is_dirty());
    }

    #[test]
    fn writing_and_reading_a_block_round_trips() {
        let mut chunk = chunk_at(0, 0);
        let position = inside(0, 0, 3, 70, 9);

        assert_eq!(chunk.set(position, STONE, WorldTime(10)).unwrap(), AIR);
        assert_eq!(chunk.get(position).unwrap(), STONE);
        assert_eq!(chunk.non_air_count(), 1);
        assert!(chunk.is_dirty());
    }

    #[test]
    fn negative_columns_address_their_own_blocks() {
        // The regression this guards: floor-division mistakes make column (-1,-1)
        // claim blocks that belong to (0,0).
        let mut chunk = chunk_at(-1, -1);
        let position = BlockPos::new(-1, 64, -1);
        assert_eq!(chunk.set(position, STONE, WorldTime(0)).unwrap(), AIR);
        assert_eq!(chunk.get(position).unwrap(), STONE);

        // A block just across the boundary belongs to a different column.
        assert!(chunk.get(BlockPos::new(0, 64, 0)).is_err());
        assert!(chunk
            .set(BlockPos::new(0, 64, 0), STONE, WorldTime(0))
            .is_err());
    }

    #[test]
    fn a_position_from_another_column_is_refused() {
        let mut chunk = chunk_at(0, 0);
        let err = chunk
            .set(inside(5, 5, 0, 64, 0), STONE, WorldTime(0))
            .expect_err("cross-column writes must be refused");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("different chunk column"), "{err}");
    }

    #[test]
    fn deep_and_high_sections_coexist_sparsely() {
        let mut chunk = chunk_at(0, 0);
        chunk
            .set(inside(0, 0, 0, -1900, 0), STONE, WorldTime(0))
            .unwrap();
        chunk
            .set(inside(0, 0, 0, 1900, 0), DIRT, WorldTime(0))
            .unwrap();

        // Two sections stored out of the ~120 the vertical range spans.
        assert_eq!(chunk.section_count(), 2);
        assert_eq!(chunk.get(inside(0, 0, 0, -1900, 0)).unwrap(), STONE);
        assert_eq!(chunk.get(inside(0, 0, 0, 1900, 0)).unwrap(), DIRT);
        assert_eq!(chunk.get(inside(0, 0, 0, 0, 0)).unwrap(), AIR);
    }

    #[test]
    fn writing_air_into_empty_space_allocates_nothing() {
        let mut chunk = chunk_at(0, 0);
        assert_eq!(
            chunk
                .set(inside(0, 0, 1, 500, 1), AIR, WorldTime(0))
                .unwrap(),
            AIR
        );
        assert_eq!(chunk.section_count(), 0);
        assert!(!chunk.is_dirty(), "a no-op write must not dirty the chunk");
    }

    #[test]
    fn dirty_tracking_is_per_section() {
        let mut chunk = chunk_at(0, 0);
        chunk
            .set(inside(0, 0, 0, 0, 0), STONE, WorldTime(0))
            .unwrap();
        chunk
            .set(inside(0, 0, 0, 64, 0), STONE, WorldTime(0))
            .unwrap();

        // Sections 0 and 2 for a 32-high section stack.
        assert_eq!(chunk.dirty_sections(), vec![0, 2]);

        chunk.mark_clean();
        assert!(!chunk.is_dirty());
        assert!(chunk.dirty_sections().is_empty());

        chunk
            .set(inside(0, 0, 1, 64, 1), DIRT, WorldTime(0))
            .unwrap();
        assert_eq!(
            chunk.dirty_sections(),
            vec![2],
            "only the touched section is dirty again"
        );
    }

    #[test]
    fn rewriting_the_same_state_does_not_dirty_or_journal() {
        let mut chunk = chunk_at(0, 0);
        let position = inside(0, 0, 2, 20, 2);
        chunk.set(position, STONE, WorldTime(1)).unwrap();
        chunk.mark_clean();
        chunk.take_journal();

        assert_eq!(chunk.set(position, STONE, WorldTime(2)).unwrap(), STONE);
        assert!(!chunk.is_dirty());
        assert!(chunk.journal().is_empty());
    }

    #[test]
    fn the_journal_records_cause_and_effect() {
        let mut chunk = chunk_at(0, 0);
        let position = inside(0, 0, 4, 40, 4);

        chunk.set(position, STONE, WorldTime(100)).unwrap();
        chunk.set(position, DIRT, WorldTime(200)).unwrap();
        chunk.set(position, AIR, WorldTime(300)).unwrap();

        let journal = chunk.journal();
        assert_eq!(journal.len(), 3);
        assert_eq!(
            journal[0],
            VoxelChange {
                position,
                previous: AIR,
                next: STONE,
                at: WorldTime(100)
            }
        );
        assert_eq!(
            journal[1],
            VoxelChange {
                position,
                previous: STONE,
                next: DIRT,
                at: WorldTime(200)
            }
        );
        assert_eq!(
            journal[2],
            VoxelChange {
                position,
                previous: DIRT,
                next: AIR,
                at: WorldTime(300)
            }
        );

        let drained = chunk.take_journal();
        assert_eq!(drained.len(), 3);
        assert!(chunk.journal().is_empty());
    }

    #[test]
    fn the_journal_is_bounded_and_reports_what_it_dropped() {
        let mut chunk = chunk_at(0, 0);
        let position = inside(0, 0, 0, 0, 0);

        for tick in 0..(MAX_JOURNAL_ENTRIES as u64 + 50) {
            // Alternate so every write is a real change.
            let state = if tick % 2 == 0 { STONE } else { DIRT };
            chunk.set(position, state, WorldTime(tick)).unwrap();
        }

        assert_eq!(chunk.journal().len(), MAX_JOURNAL_ENTRIES);
        assert_eq!(chunk.dropped_journal_entries(), 50);
        // The retained window is the most recent one.
        assert_eq!(
            chunk.journal().last().unwrap().at,
            WorldTime(MAX_JOURNAL_ENTRIES as u64 + 49)
        );
    }

    #[test]
    fn lifecycle_transitions_follow_the_documented_flow() {
        let mut chunk = chunk_at(0, 0);
        for state in [
            ChunkState::Requested,
            ChunkState::Generating,
            ChunkState::Generated,
            ChunkState::Loaded,
            ChunkState::Active,
            ChunkState::Inactive,
            ChunkState::Unloading,
            ChunkState::Saved,
            ChunkState::Unloaded,
        ] {
            chunk
                .transition_to(state)
                .unwrap_or_else(|err| panic!("{state:?} rejected: {err}"));
            assert_eq!(chunk.state(), state);
        }
    }

    #[test]
    fn illegal_transitions_are_refused_with_both_states() {
        let mut chunk = chunk_at(0, 0);
        let err = chunk
            .transition_to(ChunkState::Active)
            .expect_err("an unloaded chunk cannot become active");
        assert!(err.to_string().contains("from=unloaded"), "{err}");
        assert!(err.to_string().contains("to=active"), "{err}");
        assert_eq!(
            chunk.state(),
            ChunkState::Unloaded,
            "a refused transition changes nothing"
        );
    }

    #[test]
    fn a_failed_chunk_must_be_reloaded_rather_than_resumed() {
        let mut chunk = chunk_at(0, 0);
        chunk.transition_to(ChunkState::Requested).unwrap();
        chunk.transition_to(ChunkState::Error).unwrap();

        assert!(!chunk.state().is_resident());
        assert!(chunk.transition_to(ChunkState::Active).is_err());
        assert!(chunk.transition_to(ChunkState::Loaded).is_err());
        // Only a fresh load is allowed out of Error.
        chunk.transition_to(ChunkState::Unloaded).unwrap();
    }

    #[test]
    fn putting_an_all_air_section_stores_absence() {
        let mut chunk = chunk_at(0, 0);
        let shape = ChunkShape::cubic_default();

        chunk.put_section(3, Section::uniform(shape, STONE));
        assert_eq!(chunk.section_count(), 1);

        chunk.put_section(3, Section::empty(shape));
        assert_eq!(
            chunk.section_count(),
            0,
            "an air section is stored as absence"
        );
    }

    #[test]
    fn compaction_removes_sections_that_were_dug_out() {
        let mut chunk = chunk_at(0, 0);
        let position = inside(0, 0, 7, 7, 7);

        chunk.set(position, STONE, WorldTime(0)).unwrap();
        assert_eq!(chunk.section_count(), 1);

        chunk.set(position, AIR, WorldTime(1)).unwrap();
        assert_eq!(
            chunk.section_count(),
            1,
            "the section still exists until compaction"
        );

        chunk.compact();
        assert_eq!(chunk.section_count(), 0);
        assert_eq!(chunk.storage_bytes(), 0);
        assert_eq!(chunk.non_air_count(), 0);
    }
}
