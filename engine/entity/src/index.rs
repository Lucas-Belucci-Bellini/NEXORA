//! A loose grid over entity positions.
//!
//! Implements the spatial index `Entity System.md` §34 (ENTITY-33) asks for,
//! and `DEBT-0010` deferred until the linear scan had been measured. It had, at
//! three population sizes, before any of this existed; see
//! `docs/benchmarks/PHASE-0-BASELINE.md`.
//!
//! ## What the measurement said, and what it did not
//!
//! A radius query over 100,000 entities scanned all of them for **six** hits.
//! That is the shape an index fixes: work proportional to the population when
//! the answer is proportional to the volume asked about.
//!
//! The same measurement said the opposite about `Query::by_type`, which matched
//! every entity. No index can help a query that returns everything, so the
//! non-spatial queries still scan and this module is not involved in them.
//!
//! ## Why a loose grid and not chunk columns
//!
//! `DEBT-0010` offered either. Chunk columns would make the store depend on a
//! [`ChunkShape`](nexora_foundation::spatial::ChunkShape), which is a runtime
//! value the store has never needed and would have to be threaded through every
//! constructor — and `CHUNK & VOXEL ENGINE.md` §3 forbids assuming a fixed one.
//! A grid with its own fixed cell answers a *world-space rectangle*, so
//! `Query::in_chunk` converts a column to a rectangle at the call site and any
//! chunk shape works against the same index.
//!
//! The cell is a power of two so placing a position is two shifts rather than
//! two divisions by a runtime value — the operation `step` performs for every
//! moving entity, every tick.
//!
//! ## Two dimensions, not three
//!
//! Cells address `(x, z)`. Entities in a voxel world spread horizontally for
//! kilometres and vertically for a few hundred blocks, mostly near one surface,
//! so a third axis would multiply the bookkeeping to subdivide the axis that is
//! already the least spread out. `in_chunk` asks about a column, which is the
//! same footprint.

use std::collections::BTreeMap;

use nexora_foundation::spatial::WorldPosition;

/// Cell edge, as a power-of-two shift.
pub const CELL_SHIFT: u32 = 4;

/// Cell edge in blocks.
pub const CELL_SIZE: i64 = 1 << CELL_SHIFT;

/// One cell of the grid: a `CELL_SIZE` square column, unbounded vertically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CellCoord {
    /// Cell index on the east-west axis.
    pub x: i64,
    /// Cell index on the north-south axis.
    pub z: i64,
}

impl CellCoord {
    /// Construct a cell address.
    #[must_use]
    pub const fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }

    /// The cell containing a block coordinate on one axis.
    ///
    /// An arithmetic shift right is floor division for a power of two, which is
    /// the same rounding [`WorldPosition::to_block_pos`] uses — truncation would
    /// fold the two cells either side of the origin into one.
    #[must_use]
    pub const fn of_block(block: i64) -> i64 {
        block >> CELL_SHIFT
    }

    /// The cell containing a continuous position.
    ///
    /// A non-finite coordinate cannot reach here through `Transform::validate`,
    /// and every conversion below saturates rather than being undefined if one
    /// ever does, so the worst case is a position filed in the outermost cell.
    #[must_use]
    pub fn of(position: WorldPosition) -> Self {
        Self::new(
            Self::of_block(floor_to_i64(position.x)),
            Self::of_block(floor_to_i64(position.z)),
        )
    }
}

/// `value.floor() as i64`, without calling `floor`.
///
/// `WorldPosition::to_block_pos` spells this with `f64::floor`, which is the
/// clearer way to write it and the right one nearly everywhere. Here it is on
/// `EntityStore::step`'s inner loop, once per moving entity per tick, and the
/// baseline for that loop is about 2 ns per entity — so what `floor` costs is
/// visible in the total.
///
/// It is not free: the x86-64 baseline has no `roundsd` (that is SSE4.1, which
/// the default target does not assume), so `floor` is a call into libm.
/// Measured over 1,000 placements it is **4.94 ns** each against **1.54 ns**
/// for the form below — 3.2× — and `step` performs two per entity.
///
/// `as i64` truncates toward zero, so it is already the answer for a positive
/// value and one too high for a negative non-integer. The comparison is true in
/// exactly that case. `saturating_sub` rather than `-` because a coordinate
/// below `i64::MIN` saturates the cast *and* compares less than it, and the
/// subtraction would then wrap to the far positive end of the world.
#[inline]
fn floor_to_i64(value: f64) -> i64 {
    let truncated = value as i64;
    truncated.saturating_sub(i64::from(value < truncated as f64))
}

/// A rectangle of cells, inclusive on both corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRect {
    /// Lowest cell on both axes.
    pub min: CellCoord,
    /// Highest cell on both axes.
    pub max: CellCoord,
}

impl CellRect {
    /// The cells covering a world-space rectangle, inclusive of its edges.
    ///
    /// Non-finite or inverted input yields a rectangle covering the single cell
    /// at the low corner rather than an empty or wrapping one.
    #[must_use]
    pub fn covering(min_x: f64, min_z: f64, max_x: f64, max_z: f64) -> Self {
        let low = CellCoord::of(WorldPosition::new(min_x, 0.0, min_z));
        let high = CellCoord::of(WorldPosition::new(max_x, 0.0, max_z));
        Self {
            min: low,
            max: CellCoord::new(high.x.max(low.x), high.z.max(low.z)),
        }
    }

    /// The cells covering a block-space rectangle, inclusive of its edges.
    ///
    /// Integer in, integer out: a chunk column's corners are exact block
    /// coordinates and at the engine's limits an `f64` cannot resolve them.
    #[must_use]
    pub fn covering_blocks(min_x: i64, min_z: i64, max_x: i64, max_z: i64) -> Self {
        let low = CellCoord::new(CellCoord::of_block(min_x), CellCoord::of_block(min_z));
        let high = CellCoord::new(CellCoord::of_block(max_x), CellCoord::of_block(max_z));
        Self {
            min: low,
            max: CellCoord::new(high.x.max(low.x), high.z.max(low.z)),
        }
    }

    /// How many cells the rectangle spans.
    ///
    /// `i128` because a query with a radius near the coordinate limit spans more
    /// cells than a `u64` counts, and the count is only ever compared against a
    /// population — saturating it would report a number that is not the answer.
    #[must_use]
    pub const fn cell_count(self) -> i128 {
        let width = (self.max.x as i128) - (self.min.x as i128) + 1;
        let depth = (self.max.z as i128) - (self.min.z as i128) + 1;
        width * depth
    }
}

/// Slots grouped by the cell their entity occupies.
///
/// Holds slot indices, not [`EntityId`](crate::id::EntityId)s: a generation
/// would have to be kept in step with the store's, and the store is the only
/// caller.
#[derive(Debug, Default)]
pub struct SpatialIndex {
    cells: BTreeMap<CellCoord, Vec<u32>>,
    entries: usize,
    widest_half: f64,
}

impl SpatialIndex {
    /// An empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// How many cells hold at least one entity.
    #[must_use]
    pub fn occupied_cells(&self) -> usize {
        self.cells.len()
    }

    /// How many entities are filed.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries
    }

    /// Whether nothing is filed.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries == 0
    }

    /// The largest horizontal half-extent ever filed.
    ///
    /// A box query has to widen its rectangle by this, because an entity whose
    /// centre is outside the rectangle can still overlap it. It is a running
    /// maximum and never falls: lowering it when the widest entity despawns
    /// would mean a scan of everything left, on a despawn, to save candidate
    /// cells on a query. Too wide costs work; too narrow would lose an answer.
    #[must_use]
    pub const fn widest_half_extent(&self) -> f64 {
        self.widest_half
    }

    /// Record a horizontal half-extent as possibly the widest.
    pub fn observe_half_extent(&mut self, half: f64) {
        if half.is_finite() && half > self.widest_half {
            self.widest_half = half;
        }
    }

    /// File a slot under a cell.
    pub fn insert(&mut self, cell: CellCoord, slot: u32) {
        self.cells.entry(cell).or_default().push(slot);
        self.entries += 1;
    }

    /// Remove a slot from a cell.
    ///
    /// Returns whether it was there. A cell that empties is dropped, so
    /// iterating a rectangle never walks cells that hold nothing.
    pub fn remove(&mut self, cell: CellCoord, slot: u32) -> bool {
        let Some(slots) = self.cells.get_mut(&cell) else {
            return false;
        };
        let Some(at) = slots.iter().position(|&held| held == slot) else {
            return false;
        };
        // Order within a cell is not meaningful: queries sort their own results.
        slots.swap_remove(at);
        if slots.is_empty() {
            self.cells.remove(&cell);
        }
        self.entries -= 1;
        true
    }

    /// Move a slot from one cell to another.
    pub fn relocate(&mut self, from: CellCoord, to: CellCoord, slot: u32) {
        if from == to {
            return;
        }
        self.remove(from, slot);
        self.insert(to, slot);
    }

    /// The slots filed under one cell.
    #[must_use]
    pub fn slots(&self, cell: CellCoord) -> &[u32] {
        self.cells.get(&cell).map_or(&[], Vec::as_slice)
    }

    /// Call `visit` with every slot in a rectangle of cells.
    ///
    /// Looks each cell up rather than ranging the map: the rectangle is
    /// generated from the query's own extent, so its size is bounded by the
    /// radius asked about and not by how far apart the world's entities are.
    pub fn for_each_in<F: FnMut(u32)>(&self, rect: CellRect, mut visit: F) {
        for x in rect.min.x..=rect.max.x {
            for z in rect.min.z..=rect.max.z {
                for &slot in self.slots(CellCoord::new(x, z)) {
                    visit(slot);
                }
            }
        }
    }

    /// Forget everything.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entries = 0;
        self.widest_half = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_is_floor_division_so_the_two_sides_of_the_origin_do_not_share_one() {
        assert_eq!(
            CellCoord::of(WorldPosition::new(0.0, 0.0, 0.0)),
            CellCoord::new(0, 0)
        );
        assert_eq!(
            CellCoord::of(WorldPosition::new(15.9, 0.0, 15.9)),
            CellCoord::new(0, 0)
        );
        assert_eq!(
            CellCoord::of(WorldPosition::new(16.0, 0.0, 16.0)),
            CellCoord::new(1, 1)
        );
        // Truncation would put this in cell 0 alongside the positive side.
        assert_eq!(
            CellCoord::of(WorldPosition::new(-0.5, 0.0, -0.5)),
            CellCoord::new(-1, -1)
        );
        assert_eq!(
            CellCoord::of(WorldPosition::new(-16.0, 0.0, -16.0)),
            CellCoord::new(-1, -1)
        );
        assert_eq!(
            CellCoord::of(WorldPosition::new(-16.1, 0.0, -16.1)),
            CellCoord::new(-2, -2)
        );
    }

    #[test]
    fn placing_a_position_agrees_with_the_floor_it_replaced() {
        let mut rng = nexora_foundation::rng::Rng::from_seed(0xF100_0001);
        for _ in 0..20_000 {
            let value = rng.next_f64().mul_add(1e9, -5e8);
            assert_eq!(floor_to_i64(value), value.floor() as i64, "at {value}");
        }
        // The cases a random sweep will not produce: exact integers on both
        // signs, the smallest steps either side of zero, the saturating ends,
        // and the values where the cast itself has no exact answer.
        for value in [
            0.0,
            -0.0,
            1.0,
            -1.0,
            -0.5,
            f64::MIN_POSITIVE,
            -f64::MIN_POSITIVE,
            16.0,
            -16.0,
            -16.000_000_000_1,
            9.007_199_254_740_99e15,
            -9.007_199_254_740_99e15,
            1e300,
            -1e300,
            f64::MAX,
            f64::MIN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ] {
            assert_eq!(floor_to_i64(value), value.floor() as i64, "at {value}");
        }
    }

    #[test]
    fn a_position_below_the_coordinate_floor_saturates_rather_than_wrapping_east() {
        // The naive `truncated - 1` wraps `i64::MIN` to `i64::MAX` here, which
        // files an entity at the opposite end of the world. In debug it panics
        // instead, which is how this was found.
        let far = CellCoord::of(WorldPosition::new(-1e300, 0.0, -1e300));
        assert!(far.x < 0 && far.z < 0, "got {far:?}");
        assert_eq!(
            far,
            CellCoord::of(WorldPosition::new(f64::MIN, 0.0, f64::MIN))
        );
    }

    #[test]
    fn a_rectangle_covers_both_edges_and_counts_what_it_covers() {
        let rect = CellRect::covering(-1.0, -1.0, 33.0, 17.0);
        assert_eq!(rect.min, CellCoord::new(-1, -1));
        assert_eq!(rect.max, CellCoord::new(2, 1));
        assert_eq!(rect.cell_count(), 4 * 3);
    }

    #[test]
    fn an_inverted_rectangle_does_not_wrap_into_an_enormous_one() {
        let rect = CellRect::covering(100.0, 100.0, 0.0, 0.0);
        assert_eq!(rect.min, rect.max);
        assert_eq!(rect.cell_count(), 1);
    }

    #[test]
    fn a_cell_that_empties_stops_being_iterated() {
        let mut index = SpatialIndex::new();
        index.insert(CellCoord::new(3, 4), 7);
        assert_eq!(index.occupied_cells(), 1);
        assert!(index.remove(CellCoord::new(3, 4), 7));
        assert_eq!(index.occupied_cells(), 0);
        assert_eq!(index.len(), 0);
        // Removing what is not there is not a panic and not a decrement.
        assert!(!index.remove(CellCoord::new(3, 4), 7));
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn relocating_to_the_same_cell_does_not_disturb_the_entry() {
        let mut index = SpatialIndex::new();
        index.insert(CellCoord::new(0, 0), 1);
        index.relocate(CellCoord::new(0, 0), CellCoord::new(0, 0), 1);
        assert_eq!(index.slots(CellCoord::new(0, 0)), &[1]);
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn a_rectangle_visits_every_slot_it_covers_and_nothing_outside_it() {
        let mut index = SpatialIndex::new();
        index.insert(CellCoord::new(0, 0), 1);
        index.insert(CellCoord::new(0, 0), 2);
        index.insert(CellCoord::new(1, 0), 3);
        index.insert(CellCoord::new(5, 5), 4);

        let mut seen = Vec::new();
        index.for_each_in(
            CellRect {
                min: CellCoord::new(0, 0),
                max: CellCoord::new(1, 1),
            },
            |slot| seen.push(slot),
        );
        seen.sort_unstable();
        assert_eq!(seen, vec![1, 2, 3]);
    }

    #[test]
    fn the_widest_half_extent_rises_and_ignores_a_non_finite_one() {
        let mut index = SpatialIndex::new();
        index.observe_half_extent(0.4);
        index.observe_half_extent(2.5);
        index.observe_half_extent(1.0);
        index.observe_half_extent(f64::NAN);
        index.observe_half_extent(f64::INFINITY);
        assert!((index.widest_half_extent() - 2.5).abs() < f64::EPSILON);
    }
}
