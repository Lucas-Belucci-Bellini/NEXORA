//! The geometry a mesher produces, and the state a chunk's mesh is in.

use nexora_foundation::spatial::Axis;

/// Which surface a face shows.
///
/// An opaque handle rather than a block state id: the mesher only needs to know
/// whether two faces can be merged into one rectangle, never what they depict.
/// The caller decides what the number means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SurfaceId(pub u32);

/// Which pass a surface is drawn in.
///
/// `RENDERER and GRAPHICS.md` RENDER-10 asks for one mesh per chunk per layer,
/// because a transparent surface has to be drawn after the opaque ones and
/// sorted back to front. Keeping them in one mesh makes that impossible: the
/// renderer would have to re-sort geometry it was handed already merged.
///
/// # Why three and not RENDER-10's four
///
/// RENDER-10 names `opaqueMesh`, `cutoutMesh`, `transparentMesh` and
/// `waterMesh`. The first three follow from `BlendMode`, which a material
/// declares. The fourth does not: **nothing in the engine says a block is
/// water.** There is no fluid concept in `engine/world`, none in
/// `nexora_asset`, and no `MaterialCategory` for a liquid. Emitting a water
/// layer would mean inventing the data that decides what goes in it, which is
/// the same refusal `DEBT-0026` made and the reason it stayed open as
/// `DEBT-0035`.
///
/// Water is a separate mesh for reasons a blend mode cannot express anyway —
/// its own shader, its own surface animation, its own sort order against other
/// transparency. When the fluid system exists it will say which blocks are
/// fluid, and a fourth variant here is one line, plus one arm wherever blend
/// modes are mapped onto layers — `nexora_simulation`'s `layer_of_blend`, which
/// this crate deliberately cannot see. Until then a water block lands in
/// `Transparent`, which is where it belongs among these three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum RenderLayer {
    /// Hides what is behind it. Drawn first, in any order.
    #[default]
    Opaque,
    /// Fully opaque or fully absent per texel. Drawn with alpha testing, no
    /// sorting needed, but it cannot share a draw call with the opaque pass.
    Cutout,
    /// Blended against what is behind it. Drawn last, back to front.
    Transparent,
}

impl RenderLayer {
    /// Every layer, in draw order.
    ///
    /// The order is the contract, not an implementation detail: a renderer that
    /// walks this array draws correctly, and one that does not, does not.
    pub const ALL: [Self; 3] = [Self::Opaque, Self::Cutout, Self::Transparent];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Opaque => "opaque",
            Self::Cutout => "cutout",
            Self::Transparent => "transparent",
        }
    }

    /// Whether the pass needs its geometry sorted back to front.
    #[must_use]
    pub const fn needs_depth_sorting(self) -> bool {
        matches!(self, Self::Transparent)
    }
}

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

/// One meshed region, split into the passes a renderer draws separately.
///
/// What RENDER-10 asks a mesher to hand over. The split happens at the moment a
/// merged rectangle is emitted rather than by sweeping the region three times:
/// merging only ever joins faces that show the *same* surface, and a surface
/// has exactly one layer, so every rectangle already belongs to one pass by the
/// time it exists. The sweep and the merge did not change by a line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayeredMesh {
    /// Surfaces that hide what is behind them.
    pub opaque: ChunkMesh,
    /// Alpha-tested surfaces.
    pub cutout: ChunkMesh,
    /// Blended surfaces, to be drawn last and sorted.
    pub transparent: ChunkMesh,
}

impl LayeredMesh {
    /// An empty mesh in every layer.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            opaque: ChunkMesh::new(),
            cutout: ChunkMesh::new(),
            transparent: ChunkMesh::new(),
        }
    }

    /// The mesh for one layer.
    #[must_use]
    pub const fn layer(&self, layer: RenderLayer) -> &ChunkMesh {
        match layer {
            RenderLayer::Opaque => &self.opaque,
            RenderLayer::Cutout => &self.cutout,
            RenderLayer::Transparent => &self.transparent,
        }
    }

    /// The mesh for one layer, to add to.
    pub const fn layer_mut(&mut self, layer: RenderLayer) -> &mut ChunkMesh {
        match layer {
            RenderLayer::Opaque => &mut self.opaque,
            RenderLayer::Cutout => &mut self.cutout,
            RenderLayer::Transparent => &mut self.transparent,
        }
    }

    /// Every layer with its mesh, in draw order.
    pub fn iter(&self) -> impl Iterator<Item = (RenderLayer, &ChunkMesh)> {
        RenderLayer::ALL
            .into_iter()
            .map(|layer| (layer, self.layer(layer)))
    }

    /// Rectangles across every layer.
    #[must_use]
    pub fn len(&self) -> usize {
        self.iter().map(|(_, mesh)| mesh.len()).sum()
    }

    /// Whether no layer holds anything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.iter().all(|(_, mesh)| mesh.is_empty())
    }

    /// Total surface area across every layer, in unit faces.
    ///
    /// The quantity the split must preserve: routing a rectangle to a different
    /// pass changes which mesh holds it, never how much surface exists.
    #[must_use]
    pub fn area(&self) -> u64 {
        self.iter().map(|(_, mesh)| mesh.area()).sum()
    }

    /// Vertices across every layer, at four per rectangle.
    #[must_use]
    pub fn vertex_count(&self) -> u64 {
        self.iter().map(|(_, mesh)| mesh.vertex_count()).sum()
    }

    /// Every layer's rectangles in one mesh, in draw order.
    ///
    /// For callers that genuinely want all the geometry — an area check, a
    /// benchmark, a collision proxy. Not for a renderer: flattening throws away
    /// the only thing the split exists to record.
    #[must_use]
    pub fn flattened(&self) -> ChunkMesh {
        let mut all = ChunkMesh::new();
        for (_, mesh) in self.iter() {
            all.quads.extend_from_slice(&mesh.quads);
        }
        all
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
