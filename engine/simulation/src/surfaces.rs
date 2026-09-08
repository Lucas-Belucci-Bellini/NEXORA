//! The bridge between the voxel world and the mesher.
//!
//! `nexora-mesh` depends on `nexora-foundation` and nothing else, so it cannot
//! reach a `World` — the same arrangement physics has with `VoxelSource`
//! (ADR-0007) and streaming with `ResidencyBackend` (ADR-0008). This is the
//! adapter, and it lives here because `nexora-simulation` is the one crate
//! allowed to see both sides.
//!
//! # A chunk that is not resident is not empty
//!
//! The dangerous default. If an unloaded neighbour read as *empty*, every face
//! along that border would survive culling, and the moment the chunk arrived
//! the wall would vanish — a visible flash at every streaming boundary. Worse,
//! a mesh built against absent data is wrong in a way nothing detects.
//!
//! So a non-resident cell **occludes**: the border is treated as hidden until
//! the neighbour actually loads. Faces appear when the data does, never the
//! other way round. Physics makes the same call for the same reason, and lands
//! on `solid` for it.

use nexora_foundation::spatial::BlockPos;
use nexora_mesh::mesh::SurfaceId;
use nexora_mesh::view::VoxelView;
use nexora_world::voxel::AIR;
use nexora_world::world::World;

/// Reads a [`World`] as surfaces the mesher can consume.
#[derive(Debug)]
pub struct WorldSurfaces<'a> {
    world: &'a World,
}

impl<'a> WorldSurfaces<'a> {
    /// Borrow a world for meshing.
    #[must_use]
    pub const fn new(world: &'a World) -> Self {
        Self { world }
    }
}

impl VoxelView for WorldSurfaces<'_> {
    fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
        match self.world.get_block(position) {
            // A block's state id is a stable enough handle for merging: two
            // faces merge when they show the same state, which is exactly the
            // question the mesher asks.
            Ok(state) if state != AIR => Some(SurfaceId(state.0)),
            // Air, or a chunk that is not resident. Neither shows a surface.
            _ => None,
        }
    }

    fn occludes(&self, position: BlockPos) -> bool {
        match self.world.get_block(position) {
            Ok(state) => state != AIR,
            // Not resident. Treated as occluding so the border stays hidden
            // until the neighbour loads — see the module documentation.
            Err(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::ident::Identifier;
    use nexora_foundation::spatial::ChunkCoord;
    use nexora_foundation::time::CalendarConfig;
    use nexora_mesh::view::Extent;
    use nexora_mesh::{mesh_region, ChunkMesh};
    use nexora_world::world::WorldDescriptor;

    fn world() -> World {
        let descriptor = WorldDescriptor::new("surfaces-test", 0xC0FFEE).expect("valid");
        let mut world = World::create(descriptor, CalendarConfig::earthlike()).expect("created");
        world.bring_online().expect("online");
        world
            .load_or_generate(ChunkCoord::new(0, 0))
            .expect("generated");
        world
    }

    fn stone(world: &World) -> nexora_world::voxel::BlockStateId {
        world
            .block_id(&Identifier::parse("nexora:block/stone").expect("valid"))
            .expect("registered")
    }

    #[test]
    fn a_region_spanning_the_terrain_surface_produces_geometry() {
        let world = world();
        let surfaces = WorldSurfaces::new(&world);
        // Straddle the real surface rather than guessing a height: the
        // generator's amplitude is its own business.
        let height = world.surface_height(8, 8);
        let extent = Extent::cubic(BlockPos::new(0, height - 8, 0), 16).expect("valid");
        let mesh: ChunkMesh = mesh_region(&surfaces, extent);
        assert!(
            !mesh.is_empty(),
            "the ground/air boundary produced no faces"
        );
    }

    #[test]
    fn a_region_buried_in_solid_rock_correctly_produces_nothing() {
        // Not a degenerate case -- the expected answer, and the reason culling
        // is worth having. Every face down here touches another solid cell, so
        // there is no surface to draw. A mesher that emitted geometry for this
        // would be drawing the inside of the world.
        let world = world();
        let surfaces = WorldSurfaces::new(&world);
        let deep = world.surface_height(8, 8) - 64;
        let extent = Extent::cubic(BlockPos::new(0, deep, 0), 16).expect("valid");
        assert!(
            mesh_region(&surfaces, extent).is_empty(),
            "solid rock has no visible surface"
        );
    }

    #[test]
    fn air_shows_no_surface_and_hides_nothing() {
        let world = world();
        let surfaces = WorldSurfaces::new(&world);
        // Well above the generated terrain.
        let sky = BlockPos::new(4, 250, 4);
        assert_eq!(world.get_block(sky).expect("resident"), AIR);
        assert!(surfaces.surface_at(sky).is_none());
        assert!(!surfaces.occludes(sky));
    }

    #[test]
    fn a_chunk_that_is_not_resident_occludes_rather_than_reading_as_empty() {
        // The flash-at-the-seam bug. Absent data must hide the border, not
        // reveal a wall that disappears when the neighbour loads.
        let world = world();
        let surfaces = WorldSurfaces::new(&world);
        let far = BlockPos::new(500_000, 64, 500_000);
        assert!(world.get_block(far).is_err(), "precondition: not resident");
        assert!(surfaces.occludes(far), "absent data must not reveal faces");
        assert!(
            surfaces.surface_at(far).is_none(),
            "and must not invent a surface either"
        );
    }

    #[test]
    fn two_different_blocks_read_as_two_different_surfaces() {
        let mut world = world();
        let stone = stone(&world);
        let dirt = world
            .block_id(&Identifier::parse("nexora:block/dirt").expect("valid"))
            .expect("registered");
        let a = BlockPos::new(2, 200, 2);
        let b = BlockPos::new(3, 200, 2);
        world.set_block(a, stone).expect("written");
        world.set_block(b, dirt).expect("written");

        let surfaces = WorldSurfaces::new(&world);
        assert_ne!(
            surfaces.surface_at(a),
            surfaces.surface_at(b),
            "distinct blocks must not merge into one rectangle"
        );
    }

    #[test]
    fn an_edit_changes_the_mesh_it_produces() {
        // RENDER-11's premise: a block changed, so the chunk is dirty.
        let mut world = world();
        let extent = Extent::cubic(BlockPos::new(0, 190, 0), 16).expect("valid");

        let before = mesh_region(&WorldSurfaces::new(&world), extent);
        let state = stone(&world);
        world
            .set_block(BlockPos::new(8, 200, 8), state)
            .expect("written");
        let after = mesh_region(&WorldSurfaces::new(&world), extent);

        assert_ne!(before, after, "placing a block changed no geometry");
        assert_eq!(after.area(), before.area() + 6, "one isolated cube in air");
    }
}
