//! The voxel collision boundary (PHY-10).
//!
//! `PHYSICS.md` §11 asks for a `VoxelCollisionProvider` rather than a direct
//! reach into the world, and the closing rule of that document says physics
//! must not know what a block *means*. [`VoxelSource`] is that boundary: the
//! solver asks "what shape is at this cell", and something above answers.
//!
//! This is why `nexora-physics` does not depend on `nexora-world`. The same
//! solver runs against the real world, against a flat test floor, and against a
//! synthetic benchmark terrain, and none of those three know about each other.
//!
//! ## The contract for a source that cannot answer
//!
//! A source asked about a cell it has not loaded **must return a solid shape**.
//! Returning empty would let a body walk into terrain that has not arrived yet
//! and then be ejected when it does; returning solid stops it at the edge of
//! what is known. The refusal is a physics answer, not a swallowed error — a
//! source that wants the failure reported should count it and surface it
//! through its own diagnostics.

use nexora_foundation::spatial::BlockPos;

use crate::material::MaterialId;

/// What occupies one voxel cell, as far as collision is concerned.
///
/// Non-exhaustive on purpose: `PHYSICS.md` §13 (PHY-12) calls for slabs, ramps,
/// wedges and stairs, and each is a new variant here rather than a new trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum VoxelShape {
    /// Nothing to collide with.
    #[default]
    Empty,
    /// The whole cell is solid.
    Cube(MaterialId),
}

impl VoxelShape {
    /// A solid cell of the default material.
    pub const SOLID: Self = Self::Cube(MaterialId::DEFAULT);

    /// Whether this shape obstructs movement.
    #[must_use]
    pub const fn is_solid(self) -> bool {
        matches!(self, Self::Cube(_))
    }

    /// The material of the surface, if there is one.
    #[must_use]
    pub const fn material(self) -> Option<MaterialId> {
        match self {
            Self::Empty => None,
            Self::Cube(material) => Some(material),
        }
    }
}

/// Somewhere the solver can ask what a voxel cell contains.
///
/// Implementations must be **pure and stable within a step**: asking twice
/// about the same cell without an intervening world edit has to give the same
/// answer, or collision resolution can move a body into a wall that existed
/// half a sweep ago. `NEXORA REPLAY AND DETERMINISM.md` depends on it.
pub trait VoxelSource {
    /// What occupies the cell whose lower corner is `position`.
    ///
    /// Return [`VoxelShape::SOLID`] for a cell that cannot be answered for —
    /// see the module documentation for why.
    fn shape_at(&self, position: BlockPos) -> VoxelShape;

    /// Whether the cell obstructs movement.
    fn is_solid(&self, position: BlockPos) -> bool {
        self.shape_at(position).is_solid()
    }
}

impl<T: VoxelSource + ?Sized> VoxelSource for &T {
    fn shape_at(&self, position: BlockPos) -> VoxelShape {
        (**self).shape_at(position)
    }
}

/// Nothing anywhere: bodies fall forever.
///
/// Useful for testing integration without collision, and as the honest answer
/// for a world that has not been generated yet.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptySpace;

impl VoxelSource for EmptySpace {
    fn shape_at(&self, _position: BlockPos) -> VoxelShape {
        VoxelShape::Empty
    }
}

/// An infinite floor: every cell below `surface_y` is solid.
///
/// This is a **fixture, not a mock**. It implements the trait completely and
/// correctly for the world it describes; nothing about it is a placeholder for
/// an unwritten implementation, which is what the startup brief §43 forbids.
#[derive(Debug, Clone, Copy)]
pub struct FlatGround {
    /// The first empty cell. Cells at `surface_y - 1` and below are solid.
    pub surface_y: i64,
    /// The material of the floor.
    pub material: MaterialId,
}

impl FlatGround {
    /// A floor whose top face is the plane `y = surface_y`.
    #[must_use]
    pub const fn at(surface_y: i64) -> Self {
        Self {
            surface_y,
            material: MaterialId::DEFAULT,
        }
    }

    /// The same floor made of a different material.
    #[must_use]
    pub const fn of(mut self, material: MaterialId) -> Self {
        self.material = material;
        self
    }
}

impl VoxelSource for FlatGround {
    fn shape_at(&self, position: BlockPos) -> VoxelShape {
        if position.y < self.surface_y {
            VoxelShape::Cube(self.material)
        } else {
            VoxelShape::Empty
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_space_never_obstructs() {
        let source = EmptySpace;
        assert!(!source.is_solid(BlockPos::new(0, 0, 0)));
        assert!(!source.is_solid(BlockPos::new(-9_000, -9_000, 9_000)));
    }

    #[test]
    fn a_flat_floor_is_solid_below_its_surface_and_empty_at_it() {
        let ground = FlatGround::at(64);
        assert!(ground.is_solid(BlockPos::new(0, 63, 0)));
        // The surface cell itself is the first standable air.
        assert!(!ground.is_solid(BlockPos::new(0, 64, 0)));
        assert!(!ground.is_solid(BlockPos::new(0, 65, 0)));
    }

    #[test]
    fn a_floor_carries_its_material_into_the_shape() {
        let ice = MaterialId(7);
        let ground = FlatGround::at(0).of(ice);
        assert_eq!(
            ground.shape_at(BlockPos::new(0, -1, 0)).material(),
            Some(ice)
        );
        assert_eq!(ground.shape_at(BlockPos::new(0, 0, 0)).material(), None);
    }

    #[test]
    fn the_default_shape_is_empty_so_a_forgotten_cell_is_not_a_wall() {
        assert_eq!(VoxelShape::default(), VoxelShape::Empty);
        assert!(!VoxelShape::default().is_solid());
        assert!(VoxelShape::SOLID.is_solid());
    }

    #[test]
    fn a_reference_forwards_to_the_source() {
        let ground = FlatGround::at(0);
        let by_reference: &dyn VoxelSource = &ground;
        assert!(by_reference.is_solid(BlockPos::new(0, -1, 0)));
    }
}
