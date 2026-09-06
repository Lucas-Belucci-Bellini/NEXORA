//! Voxel storage for one chunk section.
//!
//! Implements `CHUNK & VOXEL ENGINE.md` §5-§10. The storage question that
//! document poses is how to hold tens of thousands of cells without allocating
//! an object per block, so a section stores one of three representations and
//! moves between them as its contents change:
//!
//! ```text
//! Uniform    - every cell is the same state. An all-air section costs one u32.
//! Paletted   - few distinct states; cells are indices packed at the minimum
//!              bit width the palette needs.
//! Direct     - too many distinct states for a palette to pay for itself.
//! ```
//!
//! Promotion is one-way within a mutation: a section that has grown a palette
//! does not shrink back on every write, only when [`Section::compact`] is
//! called at a quiet moment.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{ChunkShape, LocalPos};

/// A block state handle, valid within one runtime session.
///
/// This is a registry runtime id (`Registry System.md` §11). It is never
/// written to a save directly - see [`crate::persist`], which maps it through a
/// table of namespaced identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockStateId(pub u32);

/// The empty block.
///
/// Air is pinned to runtime id 0 by [`crate::world::World`], which registers it
/// first. Sections rely on that: "is this section empty" must be answerable
/// without consulting a registry.
pub const AIR: BlockStateId = BlockStateId(0);

/// Largest palette index width before a section switches to direct storage.
///
/// Twelve bits is 4096 distinct states in one section. Beyond that the index
/// packing costs more than it saves.
pub const MAX_PALETTE_BITS: u8 = 12;

/// Largest palette a section will hold before promoting to direct storage.
pub const MAX_PALETTE_LEN: usize = 1 << MAX_PALETTE_BITS;

/// Which representation a section is currently using.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageKind {
    /// Every cell holds the same state.
    Uniform,
    /// Cells are packed palette indices.
    Paletted,
    /// Cells are full state ids.
    Direct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Storage {
    Uniform(BlockStateId),
    Paletted {
        palette: Vec<BlockStateId>,
        bits: u8,
        words: Vec<u64>,
    },
    Direct(Vec<BlockStateId>),
}

/// A cubic block of voxels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    shape: ChunkShape,
    storage: Storage,
    non_air: u32,
}

impl Section {
    /// A section entirely filled with one state.
    #[must_use]
    pub fn uniform(shape: ChunkShape, state: BlockStateId) -> Self {
        let non_air = if state == AIR {
            0
        } else {
            shape.volume() as u32
        };
        Self {
            shape,
            storage: Storage::Uniform(state),
            non_air,
        }
    }

    /// An empty section.
    #[must_use]
    pub fn empty(shape: ChunkShape) -> Self {
        Self::uniform(shape, AIR)
    }

    /// The section's voxel extent.
    #[must_use]
    pub const fn shape(&self) -> ChunkShape {
        self.shape
    }

    /// The representation currently in use.
    #[must_use]
    pub const fn storage_kind(&self) -> StorageKind {
        match self.storage {
            Storage::Uniform(_) => StorageKind::Uniform,
            Storage::Paletted { .. } => StorageKind::Paletted,
            Storage::Direct(_) => StorageKind::Direct,
        }
    }

    /// The single state filling this section, if it is uniform.
    #[must_use]
    pub const fn uniform_state(&self) -> Option<BlockStateId> {
        match self.storage {
            Storage::Uniform(state) => Some(state),
            _ => None,
        }
    }

    /// How many cells hold something other than air.
    #[must_use]
    pub const fn non_air_count(&self) -> u32 {
        self.non_air
    }

    /// Whether every cell is air.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.non_air == 0
    }

    /// Read one cell.
    ///
    /// # Errors
    ///
    /// Returns an error when the position lies outside the section.
    pub fn get(&self, local: LocalPos) -> Result<BlockStateId> {
        let index = self.shape.index_of(local)?;
        Ok(self.get_by_index(index))
    }

    fn get_by_index(&self, index: usize) -> BlockStateId {
        match &self.storage {
            Storage::Uniform(state) => *state,
            Storage::Paletted {
                palette,
                bits,
                words,
            } => {
                let palette_index = read_packed(words, *bits, index) as usize;
                // A palette index is written by this module and cannot exceed
                // the palette it was written against; air is the safe reading
                // if that invariant is ever violated by damaged data.
                palette.get(palette_index).copied().unwrap_or(AIR)
            }
            Storage::Direct(cells) => cells.get(index).copied().unwrap_or(AIR),
        }
    }

    /// Write one cell and return the state it replaced.
    ///
    /// # Errors
    ///
    /// Returns an error when the position lies outside the section, or when the
    /// palette cannot represent another distinct state.
    pub fn set(&mut self, local: LocalPos, state: BlockStateId) -> Result<BlockStateId> {
        let index = self.shape.index_of(local)?;
        let previous = self.get_by_index(index);
        if previous == state {
            return Ok(previous);
        }

        match &mut self.storage {
            Storage::Uniform(current) => {
                // First divergence: grow into a two-entry palette.
                let existing = *current;
                let volume = self.shape.volume();
                let bits = bits_for(2);
                let mut words = vec![0u64; word_count(volume, bits)];
                // Index 0 is the pre-existing state, so only the changed cell
                // needs writing.
                write_packed(&mut words, bits, index, 1);
                self.storage = Storage::Paletted {
                    palette: vec![existing, state],
                    bits,
                    words,
                };
            }
            Storage::Paletted {
                palette,
                bits,
                words,
            } => {
                let palette_index = match palette.iter().position(|entry| *entry == state) {
                    Some(found) => found,
                    None => {
                        if palette.len() >= MAX_PALETTE_LEN {
                            // Too many distinct states: stop paying for indirection.
                            let cells = self.materialize();
                            self.storage = Storage::Direct(cells);
                            return self.finish_set(index, previous, state);
                        }
                        palette.push(state);
                        let needed = bits_for(palette.len());
                        if needed > *bits {
                            *words = repack(words, *bits, needed, self.shape.volume());
                            *bits = needed;
                        }
                        palette.len() - 1
                    }
                };
                write_packed(words, *bits, index, palette_index as u64);
            }
            Storage::Direct(cells) => {
                cells[index] = state;
            }
        }

        self.apply_air_delta(previous, state);
        Ok(previous)
    }

    /// Complete a write after a storage promotion.
    fn finish_set(
        &mut self,
        index: usize,
        previous: BlockStateId,
        state: BlockStateId,
    ) -> Result<BlockStateId> {
        match &mut self.storage {
            Storage::Direct(cells) => cells[index] = state,
            _ => {
                return Err(Error::new(
                    Domain::Chunk,
                    "section-storage",
                    "storage promotion left the section in an unexpected representation",
                )
                .fatal())
            }
        }
        self.apply_air_delta(previous, state);
        Ok(previous)
    }

    fn apply_air_delta(&mut self, previous: BlockStateId, next: BlockStateId) {
        match (previous == AIR, next == AIR) {
            (true, false) => self.non_air += 1,
            (false, true) => self.non_air -= 1,
            _ => {}
        }
    }

    /// Expand the section into one state id per cell.
    fn materialize(&self) -> Vec<BlockStateId> {
        (0..self.shape.volume())
            .map(|index| self.get_by_index(index))
            .collect()
    }

    /// Collapse the section to the smallest representation that still fits.
    ///
    /// Called at quiet moments - after generation, before saving - rather than
    /// on every write, so that a single edit never triggers a full rescan.
    pub fn compact(&mut self) {
        let volume = self.shape.volume();
        if volume == 0 {
            return;
        }

        let first = self.get_by_index(0);
        if (1..volume).all(|index| self.get_by_index(index) == first) {
            self.storage = Storage::Uniform(first);
            self.non_air = if first == AIR { 0 } else { volume as u32 };
            return;
        }

        // Rebuild the palette from what is actually present: repeated edits can
        // leave entries that no cell references any more.
        let cells = self.materialize();
        let mut palette: Vec<BlockStateId> = Vec::new();
        for cell in &cells {
            if !palette.contains(cell) {
                palette.push(*cell);
                if palette.len() > MAX_PALETTE_LEN {
                    self.storage = Storage::Direct(cells);
                    return;
                }
            }
        }

        let bits = bits_for(palette.len());
        let mut words = vec![0u64; word_count(volume, bits)];
        for (index, cell) in cells.iter().enumerate() {
            let palette_index = palette
                .iter()
                .position(|entry| entry == cell)
                .unwrap_or_default() as u64;
            write_packed(&mut words, bits, index, palette_index);
        }
        self.storage = Storage::Paletted {
            palette,
            bits,
            words,
        };
    }

    /// Approximate heap cost of this section's cell storage, in bytes.
    ///
    /// Used by the memory budget in `NEXORA PERFORMANCE BUDGETS.md`.
    #[must_use]
    pub fn storage_bytes(&self) -> usize {
        match &self.storage {
            Storage::Uniform(_) => 0,
            Storage::Paletted { palette, words, .. } => {
                palette.len() * size_of::<BlockStateId>() + words.len() * size_of::<u64>()
            }
            Storage::Direct(cells) => cells.len() * size_of::<BlockStateId>(),
        }
    }

    /// Rebuild a section from the parts recorded in a save.
    ///
    /// # Errors
    ///
    /// Returns an error when the parts are inconsistent with the shape.
    pub(crate) fn from_parts(shape: ChunkShape, parts: SectionParts) -> Result<Self> {
        let volume = shape.volume();
        let storage = match parts {
            SectionParts::Uniform(state) => Storage::Uniform(state),
            SectionParts::Paletted {
                palette,
                bits,
                words,
            } => {
                if palette.is_empty() || bits == 0 || bits > MAX_PALETTE_BITS {
                    return Err(malformed("section palette metadata is out of range"));
                }
                if words.len() != word_count(volume, bits) {
                    return Err(malformed("section packing does not match the chunk shape")
                        .with_context("expected_words", word_count(volume, bits).to_string())
                        .with_context("found_words", words.len().to_string()));
                }
                Storage::Paletted {
                    palette,
                    bits,
                    words,
                }
            }
            SectionParts::Direct(cells) => {
                if cells.len() != volume {
                    return Err(
                        malformed("section cell count does not match the chunk shape")
                            .with_context("expected", volume.to_string())
                            .with_context("found", cells.len().to_string()),
                    );
                }
                Storage::Direct(cells)
            }
        };

        let mut section = Self {
            shape,
            storage,
            non_air: 0,
        };
        section.non_air = (0..volume)
            .filter(|index| section.get_by_index(*index) != AIR)
            .count() as u32;
        Ok(section)
    }

    /// Decompose the section for serialization.
    pub(crate) fn to_parts(&self) -> SectionParts {
        match &self.storage {
            Storage::Uniform(state) => SectionParts::Uniform(*state),
            Storage::Paletted {
                palette,
                bits,
                words,
            } => SectionParts::Paletted {
                palette: palette.clone(),
                bits: *bits,
                words: words.clone(),
            },
            Storage::Direct(cells) => SectionParts::Direct(cells.clone()),
        }
    }

    /// Rewrite every state id through a mapping, used when a save's palette is
    /// remapped onto the current registry.
    pub(crate) fn remap(
        &mut self,
        map: &dyn Fn(BlockStateId) -> Result<BlockStateId>,
    ) -> Result<()> {
        match &mut self.storage {
            Storage::Uniform(state) => *state = map(*state)?,
            Storage::Paletted { palette, .. } => {
                for entry in palette.iter_mut() {
                    *entry = map(*entry)?;
                }
            }
            Storage::Direct(cells) => {
                for cell in cells.iter_mut() {
                    *cell = map(*cell)?;
                }
            }
        }
        let volume = self.shape.volume();
        self.non_air = (0..volume)
            .filter(|index| self.get_by_index(*index) != AIR)
            .count() as u32;
        Ok(())
    }
}

/// Serialization-facing view of a section's contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SectionParts {
    Uniform(BlockStateId),
    Paletted {
        palette: Vec<BlockStateId>,
        bits: u8,
        words: Vec<u64>,
    },
    Direct(Vec<BlockStateId>),
}

fn malformed(message: &'static str) -> Error {
    Error::new(Domain::Chunk, "section-storage", message).with_recovery(Recovery::Quarantine)
}

/// Smallest index width that can address `len` palette entries.
#[must_use]
pub fn bits_for(len: usize) -> u8 {
    let mut bits = 1u8;
    while (1usize << bits) < len && bits < MAX_PALETTE_BITS {
        bits += 1;
    }
    bits
}

/// How many `u64` words hold `volume` indices of `bits` each.
///
/// Indices never straddle a word boundary. That wastes a few bits per word for
/// widths that do not divide 64, and buys branch-free reads and writes.
#[must_use]
pub fn word_count(volume: usize, bits: u8) -> usize {
    let per_word = 64 / bits as usize;
    volume.div_ceil(per_word)
}

fn read_packed(words: &[u64], bits: u8, index: usize) -> u64 {
    let per_word = 64 / bits as usize;
    let word = index / per_word;
    let offset = (index % per_word) * bits as usize;
    let mask = (1u64 << bits) - 1;
    words.get(word).map_or(0, |value| (value >> offset) & mask)
}

fn write_packed(words: &mut [u64], bits: u8, index: usize, value: u64) {
    let per_word = 64 / bits as usize;
    let word = index / per_word;
    let offset = (index % per_word) * bits as usize;
    let mask = (1u64 << bits) - 1;
    if let Some(slot) = words.get_mut(word) {
        *slot = (*slot & !(mask << offset)) | ((value & mask) << offset);
    }
}

fn repack(words: &[u64], from_bits: u8, to_bits: u8, volume: usize) -> Vec<u64> {
    let mut out = vec![0u64; word_count(volume, to_bits)];
    for index in 0..volume {
        write_packed(
            &mut out,
            to_bits,
            index,
            read_packed(words, from_bits, index),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const STONE: BlockStateId = BlockStateId(1);
    const DIRT: BlockStateId = BlockStateId(2);
    const GRASS: BlockStateId = BlockStateId(3);

    fn shape() -> ChunkShape {
        ChunkShape::cubic_default()
    }

    #[test]
    fn an_empty_section_costs_nothing_to_store() {
        let section = Section::empty(shape());
        assert_eq!(section.storage_kind(), StorageKind::Uniform);
        assert_eq!(
            section.storage_bytes(),
            0,
            "an all-air section must not allocate cells"
        );
        assert!(section.is_empty());
        assert_eq!(section.non_air_count(), 0);
        assert_eq!(section.get(LocalPos::new(5, 5, 5)).unwrap(), AIR);
    }

    #[test]
    fn writing_the_same_state_keeps_the_section_uniform() {
        let mut section = Section::uniform(shape(), STONE);
        section.set(LocalPos::new(1, 2, 3), STONE).unwrap();
        assert_eq!(section.storage_kind(), StorageKind::Uniform);
        assert_eq!(section.non_air_count(), shape().volume() as u32);
    }

    #[test]
    fn the_first_differing_write_grows_a_palette() {
        let mut section = Section::empty(shape());
        let previous = section.set(LocalPos::new(0, 0, 0), STONE).unwrap();

        assert_eq!(previous, AIR);
        assert_eq!(section.storage_kind(), StorageKind::Paletted);
        assert_eq!(section.get(LocalPos::new(0, 0, 0)).unwrap(), STONE);
        assert_eq!(section.get(LocalPos::new(0, 0, 1)).unwrap(), AIR);
        assert_eq!(section.non_air_count(), 1);
    }

    #[test]
    fn every_cell_is_independently_addressable() {
        // The bug this catches is packed-index arithmetic that aliases two
        // cells onto the same bits.
        let shape = ChunkShape::new(7, 5, 13).unwrap();
        let mut section = Section::empty(shape);
        let mut expected = Vec::new();

        for y in 0..shape.size_y() {
            for z in 0..shape.size_z() {
                for x in 0..shape.size_x() {
                    // Cycle through several states so the palette really grows.
                    let state = BlockStateId((x + y * 3 + z * 7) % 11);
                    section.set(LocalPos::new(x, y, z), state).unwrap();
                    expected.push((LocalPos::new(x, y, z), state));
                }
            }
        }

        for (local, state) in expected {
            assert_eq!(
                section.get(local).unwrap(),
                state,
                "cell {local:?} was clobbered"
            );
        }
    }

    #[test]
    fn the_palette_widens_as_states_accumulate() {
        let mut section = Section::empty(shape());
        // Two states fit in one bit, three need two, five need three.
        section.set(LocalPos::new(0, 0, 0), STONE).unwrap();
        section.set(LocalPos::new(1, 0, 0), DIRT).unwrap();
        section.set(LocalPos::new(2, 0, 0), GRASS).unwrap();
        for extra in 4..40u32 {
            section
                .set(LocalPos::new(extra % 32, 1, 0), BlockStateId(extra))
                .unwrap();
        }

        // Everything written earlier survived the widening repacks.
        assert_eq!(section.get(LocalPos::new(0, 0, 0)).unwrap(), STONE);
        assert_eq!(section.get(LocalPos::new(1, 0, 0)).unwrap(), DIRT);
        assert_eq!(section.get(LocalPos::new(2, 0, 0)).unwrap(), GRASS);
        assert_eq!(section.storage_kind(), StorageKind::Paletted);
    }

    #[test]
    fn bit_widths_cover_their_palette_size() {
        assert_eq!(bits_for(1), 1);
        assert_eq!(bits_for(2), 1);
        assert_eq!(bits_for(3), 2);
        assert_eq!(bits_for(4), 2);
        assert_eq!(bits_for(5), 3);
        assert_eq!(bits_for(256), 8);
        assert_eq!(bits_for(257), 9);
        assert_eq!(bits_for(MAX_PALETTE_LEN), MAX_PALETTE_BITS);
        // The width is capped; beyond it a section switches to direct storage.
        assert_eq!(bits_for(MAX_PALETTE_LEN + 1), MAX_PALETTE_BITS);

        for len in 1..=MAX_PALETTE_LEN {
            let bits = bits_for(len);
            assert!(
                (1usize << bits) >= len || bits == MAX_PALETTE_BITS,
                "width {bits} cannot address {len} entries"
            );
        }
    }

    #[test]
    fn packing_round_trips_at_every_supported_width() {
        for bits in 1..=MAX_PALETTE_BITS {
            let volume = 1000usize;
            let mut words = vec![0u64; word_count(volume, bits)];
            let max = (1u64 << bits) - 1;

            for index in 0..volume {
                write_packed(&mut words, bits, index, (index as u64) & max);
            }
            for index in 0..volume {
                assert_eq!(
                    read_packed(&words, bits, index),
                    (index as u64) & max,
                    "width {bits} lost index {index}"
                );
            }
        }
    }

    #[test]
    fn an_overfull_palette_promotes_to_direct_storage() {
        // A shape with more cells than the palette can hold distinct states.
        let shape = ChunkShape::new(64, 2, 64).unwrap();
        let mut section = Section::empty(shape);

        let mut written = Vec::new();
        let mut state = 1u32;
        'fill: for y in 0..shape.size_y() {
            for z in 0..shape.size_z() {
                for x in 0..shape.size_x() {
                    let local = LocalPos::new(x, y, z);
                    section.set(local, BlockStateId(state)).unwrap();
                    written.push((local, BlockStateId(state)));
                    state += 1;
                    if state as usize > MAX_PALETTE_LEN + 16 {
                        break 'fill;
                    }
                }
            }
        }

        assert_eq!(section.storage_kind(), StorageKind::Direct);
        // Nothing was lost in the promotion.
        for (local, expected) in written {
            assert_eq!(
                section.get(local).unwrap(),
                expected,
                "cell {local:?} lost its state"
            );
        }
    }

    #[test]
    fn non_air_count_tracks_writes_in_both_directions() {
        let mut section = Section::empty(shape());
        assert_eq!(section.non_air_count(), 0);

        section.set(LocalPos::new(0, 0, 0), STONE).unwrap();
        section.set(LocalPos::new(1, 0, 0), DIRT).unwrap();
        assert_eq!(section.non_air_count(), 2);

        // Replacing one solid with another does not change the count.
        section.set(LocalPos::new(0, 0, 0), GRASS).unwrap();
        assert_eq!(section.non_air_count(), 2);

        // Digging it out does.
        section.set(LocalPos::new(0, 0, 0), AIR).unwrap();
        assert_eq!(section.non_air_count(), 1);
        section.set(LocalPos::new(1, 0, 0), AIR).unwrap();
        assert_eq!(section.non_air_count(), 0);
        assert!(section.is_empty());
    }

    #[test]
    fn compaction_collapses_a_refilled_section() {
        let mut section = Section::empty(shape());
        section.set(LocalPos::new(0, 0, 0), STONE).unwrap();
        assert_eq!(section.storage_kind(), StorageKind::Paletted);

        // Put it back the way it was; the representation has not caught up yet.
        section.set(LocalPos::new(0, 0, 0), AIR).unwrap();
        assert_eq!(section.storage_kind(), StorageKind::Paletted);

        section.compact();
        assert_eq!(section.storage_kind(), StorageKind::Uniform);
        assert_eq!(section.uniform_state(), Some(AIR));
        assert_eq!(section.storage_bytes(), 0);
    }

    #[test]
    fn compaction_drops_palette_entries_nothing_references() {
        let mut section = Section::empty(shape());
        for state in 1..20u32 {
            section
                .set(LocalPos::new(0, 0, 0), BlockStateId(state))
                .unwrap();
        }
        // Nineteen states were written into one cell; only the last remains.
        section.compact();
        assert_eq!(
            section.get(LocalPos::new(0, 0, 0)).unwrap(),
            BlockStateId(19)
        );
        assert_eq!(section.get(LocalPos::new(1, 0, 0)).unwrap(), AIR);
        // Two live states means a one-bit palette.
        assert_eq!(section.storage_kind(), StorageKind::Paletted);
        assert!(section.storage_bytes() < shape().volume() * 4);
    }

    #[test]
    fn out_of_bounds_access_is_refused() {
        let mut section = Section::empty(shape());
        assert!(section.get(LocalPos::new(32, 0, 0)).is_err());
        assert!(section.set(LocalPos::new(0, 99, 0), STONE).is_err());
    }

    #[test]
    fn parts_round_trip_through_every_representation() {
        let shape = ChunkShape::new(8, 8, 8).unwrap();

        let uniform = Section::uniform(shape, STONE);
        let rebuilt = Section::from_parts(shape, uniform.to_parts()).unwrap();
        assert_eq!(rebuilt, uniform);

        let mut paletted = Section::empty(shape);
        paletted.set(LocalPos::new(1, 1, 1), STONE).unwrap();
        paletted.set(LocalPos::new(2, 2, 2), DIRT).unwrap();
        let rebuilt = Section::from_parts(shape, paletted.to_parts()).unwrap();
        assert_eq!(rebuilt, paletted);
        assert_eq!(rebuilt.non_air_count(), 2);

        let direct =
            Section::from_parts(shape, SectionParts::Direct(vec![STONE; shape.volume()])).unwrap();
        assert_eq!(direct.non_air_count(), shape.volume() as u32);
    }

    #[test]
    fn malformed_parts_are_refused() {
        let shape = ChunkShape::new(8, 8, 8).unwrap();
        assert!(Section::from_parts(shape, SectionParts::Direct(vec![STONE; 7])).is_err());
        assert!(Section::from_parts(
            shape,
            SectionParts::Paletted {
                palette: vec![],
                bits: 1,
                words: vec![]
            }
        )
        .is_err());
        assert!(Section::from_parts(
            shape,
            SectionParts::Paletted {
                palette: vec![AIR],
                bits: 1,
                words: vec![0; 3]
            }
        )
        .is_err());
    }
}
