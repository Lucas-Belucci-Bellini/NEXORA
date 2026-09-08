//! The geometry a mesher produces, and the state a chunk's mesh is in.

use nexora_foundation::spatial::Axis;

/// Which surface a face shows.
///
/// An opaque handle rather than a block state id: the mesher only needs to know
/// whether two faces can be merged into one rectangle, never what they depict.
/// The caller decides what the number means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(pub u32);

/// Which way a face points along its axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facing {
    /// Towards decreasing coordinates.
    Negative,
    /// Towards increasing coordinates.
    Positive,
}

impl Facing {
    /// Stable lowercase name, for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Negative => "negative",
            Self::Positive => "positive",
        }
    }
}

/// One merged rectangle of surface.
///
/// Coordinates are absolute world block positions, not chunk-relative: a mesh
/// that only makes sense next to the chunk it came from is a mesh that cannot
/// be checked against its neighbour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quad {
    /// World position of the cell this face belongs to, at the rectangle's
    /// minimum corner in both in-plane axes.
    pub origin: [i64; 3],
    /// The axis the face is perpendicular to.
    pub axis: Axis,
    /// Which way it points along that axis.
    pub facing: Facing,
    /// Extent along the first in-plane axis, in blocks. Never zero.
    pub width: u32,
    /// Extent along the second in-plane axis, in blocks. Never zero.
    pub height: u32,
    /// What the face shows.
    pub surface: SurfaceId,
}

impl Quad {
    /// Surface area in unit faces.
    ///
    /// The quantity greedy merging must preserve exactly: merging changes how
    /// many rectangles describe a surface, never how much surface there is.
    #[must_use]
    pub const fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
}

/// The geometry of one meshed region.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChunkMesh {
    /// The merged rectangles, in a deterministic order.
    pub quads: Vec<Quad>,
}

impl ChunkMesh {
    /// An empty mesh.
    #[must_use]
    pub const fn new() -> Self {
        Self { quads: Vec::new() }
    }

    /// How many rectangles describe the surface.
    #[must_use]
    pub fn len(&self) -> usize {
        self.quads.len()
    }

    /// Whether there is no surface at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }

    /// Total surface area, in unit faces.
    #[must_use]
    pub fn area(&self) -> u64 {
        self.quads.iter().map(Quad::area).sum()
    }

    /// Vertices a renderer would upload, at four per rectangle.
    ///
    /// Reported rather than produced: the vertex format belongs to the render
    /// backend, and this crate has no business choosing one. It is here because
    /// "how much geometry did merging save" is the question the number answers.
    #[must_use]
    pub fn vertex_count(&self) -> u64 {
        self.quads.len() as u64 * 4
    }
}

/// Where a chunk's mesh stands relative to its voxels.
///
/// `CHUNK & VOXEL ENGINE.md` CHUNK-49 and `RENDERER and GRAPHICS.md` RENDER-10
/// both name these; the two lists agree on the shape and differ in wording, so
/// this follows RENDER-10's, which is the one a renderer will read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MeshState {
    /// No mesh has ever been built.
    #[default]
    Unmeshed,
    /// A build is in flight.
    Meshing,
    /// The mesh matches the voxels.
    Ready,
    /// A mesh exists but the voxels have moved on. RENDER-11: a block changed,
    /// so this chunk — and possibly a neighbour — needs rebuilding.
    Updating,
    /// Being torn down.
    Disposing,
}

impl MeshState {
    /// Whether the mesh can be drawn as-is.
    #[must_use]
    pub const fn is_displayable(self) -> bool {
        matches!(self, Self::Ready | Self::Updating)
    }

    /// Whether a rebuild is owed.
    #[must_use]
    pub const fn needs_rebuild(self) -> bool {
        matches!(self, Self::Unmeshed | Self::Updating)
    }

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unmeshed => "unmeshed",
            Self::Meshing => "meshing",
            Self::Ready => "ready",
            Self::Updating => "updating",
            Self::Disposing => "disposing",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_is_the_product_of_the_two_extents() {
        let quad = Quad {
            origin: [0, 0, 0],
            axis: Axis::Y,
            facing: Facing::Positive,
            width: 3,
            height: 5,
            surface: SurfaceId(1),
        };
        assert_eq!(quad.area(), 15);
    }

    #[test]
    fn an_empty_mesh_has_no_area_and_no_vertices() {
        let mesh = ChunkMesh::new();
        assert!(mesh.is_empty());
        assert_eq!(mesh.area(), 0);
        assert_eq!(mesh.vertex_count(), 0);
    }

    #[test]
    fn a_stale_mesh_is_still_displayable_while_it_rebuilds() {
        // Dropping to nothing on the first edit would flicker the world every
        // time a block changed; RENDER-11 rebuilds, it does not blank.
        assert!(MeshState::Updating.is_displayable());
        assert!(MeshState::Updating.needs_rebuild());

        assert!(MeshState::Ready.is_displayable());
        assert!(!MeshState::Ready.needs_rebuild());

        assert!(!MeshState::Unmeshed.is_displayable());
        assert!(MeshState::Unmeshed.needs_rebuild());
    }

    #[test]
    fn a_disposing_mesh_is_neither_drawn_nor_rebuilt() {
        assert!(!MeshState::Disposing.is_displayable());
        assert!(!MeshState::Disposing.needs_rebuild());
    }
}
