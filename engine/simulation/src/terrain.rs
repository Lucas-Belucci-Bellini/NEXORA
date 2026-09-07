//! Voxel terrain as a physics collision source.
//!
//! Implements `PHYSICS.md` §11's `VoxelCollisionProvider`: physics asks what
//! shape is at a cell, and this answers from the world's chunk storage and
//! block registry.
//!
//! ## The two answers that are policy, not lookup
//!
//! * **Below the world floor is solid.** The world's vertical bounds are a
//!   property of the world, and a body must not fall out of the bottom of it.
//! * **An unloaded chunk is solid.** `nexora_physics::voxel` states the
//!   contract: a source that cannot answer refuses movement. Answering "empty"
//!   would let a body walk into terrain that has not arrived yet and then be
//!   ejected when it does.
//!
//! Both are decisions the *world* owns, which is exactly why they live here and
//! not in the solver.

use nexora_foundation::diagnostics::{Category, Level};
use nexora_foundation::error::Result;
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::BlockPos;
use nexora_physics::material::MaterialId;
use nexora_physics::voxel::{VoxelShape, VoxelSource};
use nexora_runtime::module::{EngineModule, ModuleContext, ModuleDependency, ModuleId};
use nexora_world::world::World;

/// A read-only view of a world that physics can collide against.
///
/// Solidity is resolved once, into a table indexed by block state id, because
/// the collision inner loop asks about a cell far more often than the block
/// registry changes — which after `bring_online` is never.
#[derive(Debug, Clone)]
pub struct WorldVoxels<'a> {
    world: &'a World,
    /// Material per block state id; `None` means the block does not obstruct.
    surfaces: Vec<Option<MaterialId>>,
    min_y: i64,
    max_y: i64,
}

impl<'a> WorldVoxels<'a> {
    /// Build a view over a world, with every solid block on the default
    /// physics material.
    #[must_use]
    pub fn new(world: &'a World) -> Self {
        let registry = world.blocks();
        let mut surfaces = vec![None; registry.len() + 1];
        for identifier in registry.identifiers() {
            let Some(entry) = registry.get(&identifier) else {
                continue;
            };
            let index = entry.runtime_id().0 as usize;
            if index >= surfaces.len() {
                surfaces.resize(index + 1, None);
            }
            if entry.value().solid {
                surfaces[index] = Some(MaterialId::DEFAULT);
            }
        }
        let bounds = world.descriptor().bounds;
        Self {
            world,
            surfaces,
            min_y: bounds.min_y,
            max_y: bounds.max_y,
        }
    }

    /// Give one registered block its own physics material.
    ///
    /// Returns whether the block was registered in this world. A caller that
    /// needs the failure to be loud should check the result; silently ignoring
    /// an unknown block would let a typo turn ice back into stone without a
    /// word.
    pub fn assign_material(&mut self, block: &Identifier, material: MaterialId) -> bool {
        let Some(entry) = self.world.blocks().get(block) else {
            return false;
        };
        let index = entry.runtime_id().0 as usize;
        if index >= self.surfaces.len() || self.surfaces[index].is_none() {
            return false;
        }
        self.surfaces[index] = Some(material);
        true
    }

    /// The world this view reads.
    #[must_use]
    pub const fn world(&self) -> &World {
        self.world
    }
}

impl VoxelSource for WorldVoxels<'_> {
    fn shape_at(&self, position: BlockPos) -> VoxelShape {
        if position.y < self.min_y {
            // The floor of the world. Nothing falls out of the bottom.
            return VoxelShape::SOLID;
        }
        if position.y > self.max_y {
            return VoxelShape::Empty;
        }
        match self.world.get_block(position) {
            Ok(state) => self
                .surfaces
                .get(state.0 as usize)
                .copied()
                .flatten()
                .map_or(VoxelShape::Empty, VoxelShape::Cube),
            // Not resident: refuse the movement rather than invent an answer.
            Err(_) => VoxelShape::SOLID,
        }
    }
}

/// An engine module that declares physics in the module graph.
///
/// It owns no state — the physics world is created by whoever runs the
/// simulation — but it puts physics in the dependency order, after the world,
/// so that `ENGINE MODULE SYSTEM.md`'s ordering is exercised by a real edge
/// rather than only by test doubles.
#[derive(Debug, Default)]
pub struct PhysicsModule;

impl PhysicsModule {
    /// The module's identifier.
    ///
    /// # Panics
    ///
    /// Never: the identifier is a compile-time constant that is known to parse,
    /// and the test below proves it.
    #[must_use]
    pub fn module_id() -> ModuleId {
        ModuleId::parse("nexora:module/physics").expect("static module id")
    }
}

impl EngineModule for PhysicsModule {
    fn id(&self) -> ModuleId {
        Self::module_id()
    }

    fn dependencies(&self) -> Vec<ModuleDependency> {
        vec![ModuleDependency::required(
            ModuleId::parse("nexora:module/world").expect("static module id"),
        )]
    }

    fn initialize(&mut self, context: &ModuleContext) -> Result<()> {
        context.diagnostics().log(
            Level::Debug,
            Category::World,
            "module/physics",
            "physics ready",
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::spatial::ChunkCoord;
    use nexora_foundation::time::CalendarConfig;
    use nexora_physics::collision::overlaps_solid;
    use nexora_physics::math::{Aabb, Vec3};
    use nexora_physics::query::raycast;
    use nexora_world::world::WorldDescriptor;

    fn populated_world() -> World {
        let descriptor = WorldDescriptor::new("terrain-test", 0x5EED).expect("valid");
        let mut world = World::create(descriptor, CalendarConfig::earthlike()).expect("created");
        world.bring_online().expect("online");
        world
            .load_or_generate(ChunkCoord::new(0, 0))
            .expect("generated");
        world
    }

    #[test]
    fn generated_terrain_is_solid_below_the_surface_and_open_above_it() {
        let world = populated_world();
        let surface = world.surface_height(4, 4);
        let voxels = WorldVoxels::new(&world);
        // `surface_height` names the topmost *solid* block, so the first
        // standable air is the cell above it.
        assert!(
            voxels.is_solid(BlockPos::new(4, surface, 4)),
            "the surface block itself should be solid"
        );
        assert!(voxels.is_solid(BlockPos::new(4, surface - 1, 4)));
        assert!(
            !voxels.is_solid(BlockPos::new(4, surface + 1, 4)),
            "the cell above the surface should be standable air"
        );
        assert!(!voxels.is_solid(BlockPos::new(4, surface + 20, 4)));
    }

    #[test]
    fn nothing_falls_out_of_the_bottom_of_the_world() {
        let world = populated_world();
        let bounds = world.descriptor().bounds;
        let voxels = WorldVoxels::new(&world);
        assert!(voxels.is_solid(BlockPos::new(0, bounds.min_y - 1, 0)));
        assert!(!voxels.is_solid(BlockPos::new(0, bounds.max_y + 1, 0)));
    }

    #[test]
    fn an_unloaded_chunk_refuses_movement_rather_than_inventing_empty_space() {
        let world = populated_world();
        let voxels = WorldVoxels::new(&world);
        // Far outside the one generated column.
        assert!(
            voxels.is_solid(BlockPos::new(5_000, 0, 5_000)),
            "an unanswerable cell must be solid, per the VoxelSource contract"
        );
    }

    #[test]
    fn a_body_standing_on_generated_terrain_is_not_inside_it() {
        let world = populated_world();
        let voxels = WorldVoxels::new(&world);
        // Feet on the first air cell, which is one above the topmost solid.
        let feet = (world.surface_height(4, 4) + 1) as f64;
        let standing = Aabb::from_center(Vec3::new(4.5, feet + 0.9, 4.5), Vec3::new(0.3, 0.9, 0.3))
            .expect("valid");
        assert!(!overlaps_solid(&voxels, standing));
    }

    #[test]
    fn a_ray_finds_the_surface_the_world_reports() {
        let world = populated_world();
        let voxels = WorldVoxels::new(&world);
        let surface = world.surface_height(4, 4);
        let hit = raycast(
            &voxels,
            Vec3::new(4.5, surface as f64 + 31.0, 4.5),
            Vec3::new(0.0, -1.0, 0.0),
            100.0,
        )
        .expect("terrain is below");
        assert_eq!(hit.cell, BlockPos::new(4, surface, 4));
        // The surface block's top face is at `surface + 1`, thirty metres down.
        assert!(
            (hit.distance - 30.0).abs() < 1e-9,
            "distance {}",
            hit.distance
        );
    }

    #[test]
    fn assigning_a_material_only_works_for_a_registered_solid_block() {
        let world = populated_world();
        let mut voxels = WorldVoxels::new(&world);
        let ice = MaterialId(9);
        let stone = Identifier::parse("nexora:block/stone").expect("valid identifier");
        assert!(
            voxels.assign_material(&stone, ice),
            "stone should be registered and solid"
        );
        let surface = world.surface_height(4, 4);
        assert_eq!(
            voxels.shape_at(BlockPos::new(4, surface - 8, 4)).material(),
            Some(ice),
            "deep enough to be stone rather than the dirt and grass on top"
        );
        // Air is registered but not solid, and an unknown block is not
        // registered at all: both are refused rather than silently accepted.
        let air = Identifier::parse("nexora:block/air").expect("valid identifier");
        assert!(!voxels.assign_material(&air, ice));
        let nonsense = Identifier::parse("nexora:block/not_a_block").expect("valid identifier");
        assert!(!voxels.assign_material(&nonsense, ice));
    }

    #[test]
    fn the_physics_module_declares_the_world_as_its_dependency() {
        let module = PhysicsModule;
        assert_eq!(module.id(), PhysicsModule::module_id());
        let dependencies = module.dependencies();
        assert_eq!(dependencies.len(), 1);
        assert_eq!(
            dependencies[0].id().to_string(),
            "nexora:module/world",
            "physics must resolve after the world it collides against"
        );
    }
}
