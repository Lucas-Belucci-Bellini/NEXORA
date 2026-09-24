//! How the mesher reads voxels, and the region it reads.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::BlockPos;

use crate::mesh::{RenderLayer, SurfaceId};

/// Terrain, as the mesher needs to see it.
///
/// Neighbour-aware by construction: the mesher asks about **world positions**,
/// including ones outside the region being meshed. `CHUNK & VOXEL ENGINE.md`
/// CHUNK-27 and CHUNK-28 require exactly that — a face on a chunk border can
/// only be culled if the cell on the other side can be read, and CHUNK-28 asks
/// for an API that crosses the boundary so the consumer never has to.
///
/// A view that answered only within one chunk would emit a wall of faces at
/// every seam, and the world would look like a grid of boxes.
pub trait VoxelView {
    /// What surface the cell shows, or `None` when it is empty.
    fn surface_at(&self, position: BlockPos) -> Option<SurfaceId>;

    /// Whether this cell hides the face of the neighbour touching it.
    ///
    /// Defaults to "anything present hides what is behind it", which is the
    /// only answer today's block data supports: `BlockDefinition` carries no
    /// render layer, so there is no way to know that glass is see-through.
    /// A view whose blocks gain one overrides this; nothing else changes.
    fn occludes(&self, position: BlockPos) -> bool {
        self.surface_at(position).is_some()
    }

    /// Which pass a surface is drawn in.
    ///
    /// Keyed by surface rather than by position, unlike [`Self::occludes`], and
    /// the difference is not an inconsistency. Occlusion is asked of an
    /// arbitrary neighbour cell that may be empty; a layer is only ever asked
    /// of a surface that exists. Keying it by surface is also what makes the
    /// split free: greedy merging joins faces only when their surfaces are
    /// equal, so every merged rectangle has exactly one layer by construction.
    ///
    /// Defaults to [`RenderLayer::Opaque`], the same posture `occludes` takes:
    /// a view whose surfaces can say otherwise overrides this, and nothing
    /// else changes.
    fn layer_of(&self, surface: SurfaceId) -> RenderLayer {
        let _ = surface;
        RenderLayer::Opaque
    }
}

/// The box of cells to mesh.
///
/// Separate from `ChunkShape` on purpose: the mesher does not care whether it
/// is given a section, a chunk column or an arbitrary box, and tying it to one
/// of those would make it untestable on anything smaller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent {
    /// The cell at the minimum corner.
    pub origin: BlockPos,
    /// Cells along each axis. Every component must be non-zero.
    pub size: [u32; 3],
}

/// Largest region the mesher will accept on one axis.
///
/// The mask it builds is proportional to the two in-plane extents, so an
/// unbounded size is an unbounded allocation. 512 is far past any chunk the
/// engine defines and far below anything that matters as a memory budget.
pub const MAX_EXTENT: u32 = 512;

impl Extent {
    /// A region of `size` cells starting at `origin`.
    ///
    /// # Errors
    ///
    /// Returns an error when any component is zero or exceeds [`MAX_EXTENT`].
    /// A zero extent is almost always a caller bug rather than a request to
    /// mesh nothing, so it is refused rather than silently producing an empty
    /// mesh that looks like correctly-culled empty space.
    pub fn new(origin: BlockPos, size: [u32; 3]) -> Result<Self> {
        for (index, component) in size.iter().enumerate() {
            if *component == 0 || *component > MAX_EXTENT {
                return Err(Error::new(
                    Domain::Spatial,
                    "mesh-extent",
                    "mesh extent must be between one and MAX_EXTENT on every axis",
                )
                .with_recovery(Recovery::Reject)
                .with_context("axis", ["x", "y", "z"][index])
                .with_context("size", component.to_string())
                .with_context("maximum", MAX_EXTENT.to_string()));
            }
        }
        Ok(Self { origin, size })
    }

    /// A cubic region.
    ///
    /// # Errors
    ///
    /// As [`Extent::new`].
    pub fn cubic(origin: BlockPos, size: u32) -> Result<Self> {
        Self::new(origin, [size, size, size])
    }

    /// How many cells the region holds.
    #[must_use]
    pub const fn cells(&self) -> u64 {
        self.size[0] as u64 * self.size[1] as u64 * self.size[2] as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zero_extent_is_refused_rather_than_meshing_nothing() {
        // An empty mesh from a zero extent is indistinguishable from an empty
        // mesh from correctly-culled empty space, so the caller would never
        // learn about the bug.
        assert!(Extent::new(BlockPos::new(0, 0, 0), [16, 0, 16]).is_err());
    }

    #[test]
    fn an_extent_past_the_cap_is_refused() {
        assert!(Extent::cubic(BlockPos::new(0, 0, 0), MAX_EXTENT + 1).is_err());
        assert!(Extent::cubic(BlockPos::new(0, 0, 0), MAX_EXTENT).is_ok());
    }

    #[test]
    fn cells_is_the_product_of_the_three_sizes() {
        let extent = Extent::new(BlockPos::new(0, 0, 0), [2, 3, 4]).expect("valid");
        assert_eq!(extent.cells(), 24);
    }

    #[test]
    fn occlusion_defaults_to_anything_present() {
        struct OneBlock;
        impl VoxelView for OneBlock {
            fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
                (position == BlockPos::new(0, 0, 0)).then_some(SurfaceId(1))
            }
        }
        assert!(OneBlock.occludes(BlockPos::new(0, 0, 0)));
        assert!(!OneBlock.occludes(BlockPos::new(1, 0, 0)));
    }

    #[test]
    fn a_view_can_make_a_block_visible_without_making_it_solid() {
        // The override a transparent block will need: present, but not hiding
        // what is behind it.
        struct Glass;
        impl VoxelView for Glass {
            fn surface_at(&self, _: BlockPos) -> Option<SurfaceId> {
                Some(SurfaceId(7))
            }
            fn occludes(&self, _: BlockPos) -> bool {
                false
            }
        }
        assert!(Glass.surface_at(BlockPos::new(0, 0, 0)).is_some());
        assert!(!Glass.occludes(BlockPos::new(0, 0, 0)));
    }
}
