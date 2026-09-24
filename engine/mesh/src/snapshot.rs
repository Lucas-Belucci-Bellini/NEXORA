//! The region's voxels, read once, so meshing stops paying for lookups.
//!
//! # Why this exists, in one measurement
//!
//! Appendix F, finding 22: the same 16³ region costs **3.30 ms** read from the
//! world and **295 µs** read from a dense array — **11.2×**. `cull_only_16`
//! says the same thing from the other side: greedy merging is about 3% of the
//! total. Optimising the merge would buy at most 9%; reading the voxels once
//! buys the rest. That is `DEBT-0029`, and this is its remediation.
//!
//! # It is also what makes off-thread meshing *safe*
//!
//! `DEBT-0027` wants meshing off the tick thread. The obstacle was never
//! scheduling — the mesher is already a pure function — it was that a worker
//! holding a `&World` is a worker racing the simulation. A snapshot is owned,
//! `Send`, and disconnected from the world the instant it is taken, so the
//! worker cannot touch authoritative state even by accident. Faster and safer
//! are the same change.
//!
//! # What it must capture, and what the benchmark fixture got wrong
//!
//! An earlier version of this lived in the benchmark and stored only the
//! surface per cell, deriving occlusion as *"anything present occludes"*. That
//! was true when it was written and stopped being true the day materials
//! arrived: a snapshot that re-derives occlusion makes glass solid, and one
//! that forgets layers draws everything in the opaque pass. Both silently.
//!
//! So a snapshot captures **every answer the view gives**, rather than
//! recomputing any of them: the surface, the occlusion, and the layer. The
//! test that keeps it honest is not a property but an equality — meshing
//! through a snapshot must produce exactly the mesh that meshing through the
//! source produced.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::BlockPos;

use crate::mesh::{RenderLayer, SurfaceId};
use crate::view::{Extent, VoxelView};

/// Most cells one snapshot will hold, border included.
///
/// A bound on memory, and the arithmetic behind it: a cell costs eight bytes
/// for its `Option<SurfaceId>` plus one for its occlusion flag, so two million
/// cells is about 18 MiB. That covers every region the engine actually meshes —
/// a 32³ chunk plus its border is 39,304 cells, 0.35 MiB — while refusing the
/// 512³ that [`Extent`] alone would permit, which would be a gigabyte for one
/// mesh job.
pub const MAX_CELLS: usize = 2_000_000;

/// One region's voxels, owned and detached from the world they came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseSnapshot {
    origin: BlockPos,
    size: [usize; 3],
    surface: Vec<Option<SurfaceId>>,
    /// Captured, never re-derived. See the module documentation.
    occludes: Vec<bool>,
    /// One entry per distinct surface, not per cell: a surface has exactly one
    /// layer, so a per-cell copy would be the same byte repeated.
    layers: BTreeMap<SurfaceId, RenderLayer>,
}

impl DenseSnapshot {
    /// Read `extent` plus a one-cell border out of `view`.
    ///
    /// The border is not optional. Face culling at the region's edge asks about
    /// the cell on the other side (CHUNK-27, CHUNK-28); a snapshot that stopped
    /// at the boundary would answer "unknown" there and emit a wall of faces at
    /// every seam.
    ///
    /// # Errors
    ///
    /// Returns an error when the region plus its border would exceed
    /// [`MAX_CELLS`].
    pub fn read<V: VoxelView + ?Sized>(view: &V, extent: Extent) -> Result<Self> {
        let size = [
            extent.size[0] as usize + 2,
            extent.size[1] as usize + 2,
            extent.size[2] as usize + 2,
        ];
        let cells = size[0]
            .checked_mul(size[1])
            .and_then(|product| product.checked_mul(size[2]))
            .ok_or_else(|| too_large("snapshot size overflows"))?;
        if cells > MAX_CELLS {
            return Err(too_large("snapshot would exceed the cell limit")
                .with_context("cells", cells.to_string())
                .with_context("limit", MAX_CELLS.to_string()));
        }

        let origin = BlockPos::new(
            extent.origin.x - 1,
            extent.origin.y - 1,
            extent.origin.z - 1,
        );
        let mut surface = Vec::with_capacity(cells);
        let mut occludes = Vec::with_capacity(cells);
        let mut layers = BTreeMap::new();

        // Z outermost, X innermost: the same order `index` lays them out in, so
        // the write is sequential.
        for z in 0..size[2] {
            for y in 0..size[1] {
                for x in 0..size[0] {
                    let at = BlockPos::new(
                        origin.x + x as i64,
                        origin.y + y as i64,
                        origin.z + z as i64,
                    );
                    let here = view.surface_at(at);
                    if let Some(id) = here {
                        layers.entry(id).or_insert_with(|| view.layer_of(id));
                    }
                    surface.push(here);
                    occludes.push(view.occludes(at));
                }
            }
        }

        Ok(Self {
            origin,
            size,
            surface,
            occludes,
            layers,
        })
    }

    /// The cell at the minimum corner, one outside the region that was asked for.
    #[must_use]
    pub const fn origin(&self) -> BlockPos {
        self.origin
    }

    /// Cells along each axis, border included.
    #[must_use]
    pub const fn size(&self) -> [usize; 3] {
        self.size
    }

    /// How many cells it holds.
    #[must_use]
    pub fn cells(&self) -> usize {
        self.surface.len()
    }

    /// How many distinct surfaces appear in it.
    #[must_use]
    pub fn surfaces(&self) -> usize {
        self.layers.len()
    }

    /// Bytes of cell data held, excluding the layer table.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.surface.len() * size_of::<Option<SurfaceId>>() + self.occludes.len()
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

impl VoxelView for DenseSnapshot {
    fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
        self.surface[self.index(position)?]
    }

    fn occludes(&self, position: BlockPos) -> bool {
        // Outside the snapshot is unknown, and unknown occludes — the same call
        // `WorldSurfaces` makes for a chunk that is not resident, and for the
        // same reason: a face that appears when data arrives is better than one
        // that vanishes.
        self.index(position)
            .is_none_or(|index| self.occludes[index])
    }

    fn layer_of(&self, surface: SurfaceId) -> RenderLayer {
        self.layers
            .get(&surface)
            .copied()
            .unwrap_or(RenderLayer::Opaque)
    }
}

fn too_large(message: &'static str) -> Error {
    Error::new(Domain::Spatial, "mesh-snapshot", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::greedy::mesh_region;

    /// A view with three classes, so a snapshot that drops occlusion or layer
    /// information produces a visibly different mesh rather than a subtly one.
    struct Mixed;

    impl Mixed {
        const STONE: SurfaceId = SurfaceId(1);
        const LEAVES: SurfaceId = SurfaceId(2);
        const GLASS: SurfaceId = SurfaceId(3);
    }

    impl VoxelView for Mixed {
        fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
            if !(0..4).contains(&position.x) || position.y != 0 || position.z != 0 {
                return None;
            }
            Some(match position.x {
                0 | 1 => Self::STONE,
                2 => Self::LEAVES,
                _ => Self::GLASS,
            })
        }
        fn occludes(&self, position: BlockPos) -> bool {
            self.surface_at(position) == Some(Self::STONE)
        }
        fn layer_of(&self, surface: SurfaceId) -> RenderLayer {
            match surface {
                Self::LEAVES => RenderLayer::Cutout,
                Self::GLASS => RenderLayer::Transparent,
                _ => RenderLayer::Opaque,
            }
        }
    }

    fn extent() -> Extent {
        Extent::new(BlockPos::new(0, 0, 0), [4, 1, 1]).expect("valid")
    }

    #[test]
    fn meshing_through_a_snapshot_produces_the_same_mesh_as_the_source() {
        // The whole contract, as an equality rather than a property. Anything
        // the snapshot fails to capture shows up here as a different mesh.
        let direct = mesh_region(&Mixed, extent());
        let snapshot = DenseSnapshot::read(&Mixed, extent()).expect("fits");
        let indirect = mesh_region(&snapshot, extent());

        assert_eq!(direct, indirect);
        // And the mesh is worth comparing: all three passes carry something.
        assert!(!direct.opaque.is_empty());
        assert!(!direct.cutout.is_empty());
        assert!(!direct.transparent.is_empty());
    }

    #[test]
    fn a_snapshot_captures_occlusion_rather_than_re_deriving_it() {
        // The bug the benchmark fixture had. "Anything present occludes" was
        // true before materials and false after: it makes glass solid, and the
        // face behind it disappears.
        let snapshot = DenseSnapshot::read(&Mixed, extent()).expect("fits");
        assert!(snapshot.occludes(BlockPos::new(0, 0, 0)), "stone is solid");
        assert!(
            !snapshot.occludes(BlockPos::new(2, 0, 0)),
            "leaves must not hide what is behind them"
        );
        assert!(
            !snapshot.occludes(BlockPos::new(3, 0, 0)),
            "glass must not hide what is behind it"
        );
        // A cell that is present but not occluding is exactly the case a
        // re-derived snapshot gets wrong.
        assert!(snapshot.surface_at(BlockPos::new(3, 0, 0)).is_some());
    }

    #[test]
    fn a_snapshot_carries_the_layer_of_every_surface_it_saw() {
        let snapshot = DenseSnapshot::read(&Mixed, extent()).expect("fits");
        assert_eq!(snapshot.surfaces(), 3);
        assert_eq!(snapshot.layer_of(Mixed::STONE), RenderLayer::Opaque);
        assert_eq!(snapshot.layer_of(Mixed::LEAVES), RenderLayer::Cutout);
        assert_eq!(snapshot.layer_of(Mixed::GLASS), RenderLayer::Transparent);
        // One entry per surface, not per cell: stone occupies two cells.
        assert_eq!(snapshot.cells(), 6 * 3 * 3);
    }

    #[test]
    fn a_surface_the_snapshot_never_saw_falls_back_to_opaque() {
        // The same safe direction the table takes: wrong pass and visible
        // beats right pass and vanished.
        let snapshot = DenseSnapshot::read(&Mixed, extent()).expect("fits");
        assert_eq!(snapshot.layer_of(SurfaceId(999)), RenderLayer::Opaque);
    }

    #[test]
    fn the_border_is_read_and_outside_it_occludes() {
        let snapshot = DenseSnapshot::read(&Mixed, extent()).expect("fits");
        assert_eq!(snapshot.origin(), BlockPos::new(-1, -1, -1));
        assert_eq!(snapshot.size(), [6, 3, 3]);

        // Inside the border, and empty: the neighbour the edge face is culled
        // against. It must read as empty, not as unknown.
        assert!(snapshot.surface_at(BlockPos::new(-1, 0, 0)).is_none());
        assert!(!snapshot.occludes(BlockPos::new(-1, 0, 0)));

        // Past the border: unknown, and unknown occludes.
        assert!(snapshot.occludes(BlockPos::new(-2, 0, 0)));
        assert!(snapshot.surface_at(BlockPos::new(-2, 0, 0)).is_none());
    }

    #[test]
    fn a_snapshot_is_send_so_a_worker_can_own_one() {
        // The half of DEBT-0027 that is about safety rather than speed. If this
        // stops compiling, off-thread meshing stops being possible.
        const fn assert_send<T: Send>() {}
        assert_send::<DenseSnapshot>();
    }

    #[test]
    fn a_region_past_the_cell_limit_is_refused_rather_than_allocated() {
        // `Extent` alone permits 512³, which as a snapshot is about a gigabyte
        // for one mesh job.
        let huge = Extent::cubic(BlockPos::new(0, 0, 0), 512).expect("valid extent");
        let error = DenseSnapshot::read(&Mixed, huge).expect_err("too large");
        assert!(error.to_string().contains("cell limit"), "{error}");

        // And a region at a realistic size is nowhere near it.
        let chunk = Extent::cubic(BlockPos::new(0, 0, 0), 32).expect("valid");
        let snapshot = DenseSnapshot::read(&Mixed, chunk).expect("fits");
        assert_eq!(snapshot.cells(), 34 * 34 * 34);
        assert!(snapshot.byte_len() < MAX_CELLS * 9);
    }

    #[test]
    fn reading_the_same_region_twice_gives_the_same_snapshot() {
        let once = DenseSnapshot::read(&Mixed, extent()).expect("fits");
        let twice = DenseSnapshot::read(&Mixed, extent()).expect("fits");
        assert_eq!(once, twice);
    }
}
