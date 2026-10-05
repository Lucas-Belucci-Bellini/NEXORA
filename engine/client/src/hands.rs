//! The player's hands in the window (ADR-0036): which drawn columns an edit
//! changes, which must be uploaded again, and the world as the first frame
//! showed it.
//!
//! A click becomes a block command at the next world tick
//! (`nexora_simulation::Interaction`); this module is only the client's side
//! of what follows. An accepted edit changes the faces of the column holding
//! the cell and of any column holding a cell that shares a face with it, so
//! those are meshed again. Their new quads put new corners on the edges they
//! share with every column they touch, edge or corner, and the pass splits
//! quads at every corner it is given (DEBT-0047) — so those neighbours are
//! uploaded again too, with the corners of every drawn mesh, or a seam opens.
//!
//! A cell outside the drawn band changes the world and the save, and nothing
//! on screen: the band is fixed when the scene is built, and streaming into
//! the pass is not built.

use std::collections::BTreeMap;

use nexora_foundation::spatial::BlockPos;
use nexora_mesh::{Extent, RenderLayer, SurfaceId, VoxelView};
use nexora_simulation::WorldSurfaces;
use nexora_world::voxel::{BlockStateId, AIR};

/// Whether `region` holds the cell at `position`.
#[must_use]
pub fn holds(region: &Extent, position: BlockPos) -> bool {
    let low = [region.origin.x, region.origin.y, region.origin.z];
    let cell = [position.x, position.y, position.z];
    (0..3).all(|axis| {
        cell[axis] >= low[axis] && cell[axis] < low[axis] + i64::from(region.size[axis])
    })
}

/// The drawn regions an edit at `position` changes: the one holding the
/// cell, and any holding a cell that shares a face with it. In order, each
/// once.
#[must_use]
pub fn touched(regions: &[Extent], position: BlockPos) -> Vec<usize> {
    let (x, y, z) = (position.x, position.y, position.z);
    let cells = [
        position,
        BlockPos::new(x + 1, y, z),
        BlockPos::new(x - 1, y, z),
        BlockPos::new(x, y + 1, z),
        BlockPos::new(x, y - 1, z),
        BlockPos::new(x, y, z + 1),
        BlockPos::new(x, y, z - 1),
    ];
    regions
        .iter()
        .enumerate()
        .filter(|(_, region)| cells.iter().any(|cell| holds(region, *cell)))
        .map(|(index, _)| index)
        .collect()
}

/// The regions in `indices`, and every region touching one of them along an
/// edge or at a corner: what must be uploaded again when those meshes
/// change. In order, each once.
#[must_use]
pub fn with_neighbours(regions: &[Extent], indices: &[usize]) -> Vec<usize> {
    let touching = |a: &Extent, b: &Extent| {
        [
            (0usize, a.origin.x, b.origin.x),
            (2, a.origin.z, b.origin.z),
        ]
        .iter()
        .all(|&(axis, a_low, b_low)| {
            let (a_high, b_high) = (
                a_low + i64::from(a.size[axis]),
                b_low + i64::from(b.size[axis]),
            );
            a_low <= b_high && b_low <= a_high
        })
    };
    regions
        .iter()
        .enumerate()
        .filter(|(index, region)| {
            indices
                .iter()
                .any(|&changed| changed == *index || touching(region, &regions[changed]))
        })
        .map(|(index, _)| index)
        .collect()
}

/// The world as the first frame showed it: every cell edited since reads as
/// it was, every other as it is.
///
/// The first frame is judged against a ray cast after the window closes, so
/// the world is not borrowed by the check while frames run (ADR-0032). Once
/// the hands can change it, the world at the end is not the world that
/// frame drew; this view is, exactly, because every edit after the capture
/// records what the cell held before it.
pub struct AsFirstShown<'a> {
    now: WorldSurfaces<'a>,
    before: &'a BTreeMap<BlockPos, BlockStateId>,
}

impl<'a> AsFirstShown<'a> {
    /// `now`, with each cell in `before` read as the state it maps to.
    #[must_use]
    pub const fn new(now: WorldSurfaces<'a>, before: &'a BTreeMap<BlockPos, BlockStateId>) -> Self {
        Self { now, before }
    }
}

impl VoxelView for AsFirstShown<'_> {
    fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
        match self.before.get(&position) {
            Some(state) if *state == AIR => None,
            Some(state) => self.now.table().surface_of(state.0),
            None => self.now.surface_at(position),
        }
    }

    fn occludes(&self, position: BlockPos) -> bool {
        match self.before.get(&position) {
            Some(state) => *state != AIR && self.now.table().occludes_at(state.0),
            None => self.now.occludes(position),
        }
    }

    fn layer_of(&self, surface: SurfaceId) -> RenderLayer {
        self.now.layer_of(surface)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three by three columns of sixteen, as the default client draws.
    fn grid() -> Vec<Extent> {
        let mut regions = Vec::new();
        for cx in -1..=1 {
            for cz in -1..=1 {
                regions
                    .push(Extent::new(BlockPos::new(cx * 16, 50, cz * 16), [16, 40, 16]).unwrap());
            }
        }
        regions
    }

    #[test]
    fn an_edit_inside_a_column_touches_only_that_column() {
        let regions = grid();
        assert_eq!(touched(&regions, BlockPos::new(5, 60, 5)), vec![4]);
    }

    #[test]
    fn an_edit_on_a_column_edge_touches_the_neighbour_across_it() {
        let regions = grid();
        // x = 15 is the last cell of the centre column; x = 16 the first of
        // the next one, whose face against it changes.
        assert_eq!(touched(&regions, BlockPos::new(15, 60, 5)), vec![4, 7]);
        assert_eq!(touched(&regions, BlockPos::new(0, 60, 0)), vec![1, 3, 4]);
    }

    #[test]
    fn an_edit_outside_the_band_touches_nothing_drawn() {
        let regions = grid();
        assert!(touched(&regions, BlockPos::new(5, 200, 5)).is_empty());
        // The top cell of the band does: its face against the cell above is
        // drawn.
        assert_eq!(touched(&regions, BlockPos::new(5, 89, 5)), vec![4]);
    }

    #[test]
    fn a_changed_column_is_uploaded_with_every_column_it_touches() {
        let regions = grid();
        // The centre touches all nine, edges and corners.
        assert_eq!(with_neighbours(&regions, &[4]).len(), 9);
        // A corner column touches itself and three others.
        assert_eq!(with_neighbours(&regions, &[0]), vec![0, 1, 3, 4]);
    }
}
