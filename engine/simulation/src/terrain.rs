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

use std::cell::Cell;

use nexora_foundation::diagnostics::{Category, Level};
use nexora_foundation::error::Result;
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{BlockPos, ChunkShape, SectionCoord};
use nexora_physics::material::MaterialId;
use nexora_physics::voxel::{VoxelShape, VoxelSource};
use nexora_runtime::module::{EngineModule, ModuleContext, ModuleDependency, ModuleId};
use nexora_world::world::World;
use nexora_world::Section;

/// What the world has to say about one section, resolved once.
///
/// The three cases are not an implementation detail of the cache: they are
/// exactly the three answers `shape_at` can give, decided one section at a time
/// instead of one cell at a time.
#[derive(Debug, Clone, Copy)]
enum Resident<'a> {
    /// The section holds storage, and the cell must be read from it.
    Stored(&'a Section),
    /// The column is resident and this section is absent, which *is* air —
    /// `Chunk` stores absence to mean exactly that.
    Air,
    /// The column is not resident. Per the `VoxelSource` contract, solid.
    Unloaded,
}

/// A read-only view of a world that physics can collide against.
///
/// Solidity is resolved once, into a table indexed by block state id, because
/// the collision inner loop asks about a cell far more often than the block
/// registry changes — which after `bring_online` is never.
///
/// ## Why the last section is kept
///
/// Answering one cell from the world costs two ordered-map lookups, not one:
/// the column in `World::chunks`, then the section in `Chunk::sections`. A
/// sweep asks about the cells a body's box covers, and a body is roughly
/// 0.6 × 1.8 × 0.6 against sections of 32³ — so consecutive questions land in
/// the same section almost every time, and the pair of descents is repaid for
/// nothing.
///
/// Keeping the last resolved section removes both lookups from the repeat case
/// and leaves one integer comparison. It is one entry rather than a map because
/// a second entry only helps a body straddling a boundary, and that body pays a
/// miss on the boundary cells either way.
///
/// **This is not the mesher's answer to the same word.** `DenseSnapshot`
/// (`DEBT-0029`) exists because meshing reads *every* cell of a region exactly
/// once, where a snapshot is bought back immediately. Physics reads *few* cells
/// *repeatedly*, and would pay to build a snapshot of a region it will barely
/// touch. Same phrase, opposite access shape, opposite structure.
///
/// ## Why holding a borrow into the world is sound
///
/// The view holds `&'a World`, so for as long as it exists the world cannot be
/// written to, chunks cannot be unloaded, and the section it points at cannot
/// move. The cache cannot go stale because the compiler will not let the world
/// change underneath it — the guarantee is the borrow checker's, not a rule
/// someone has to remember. `Cell` is what makes that reachable from
/// `shape_at(&self)`; it costs `Sync`, which nothing needs, and keeps `Send`,
/// which `DEBT-0027` will.
#[derive(Debug, Clone)]
pub struct WorldVoxels<'a> {
    world: &'a World,
    /// Material per block state id; `None` means the block does not obstruct.
    surfaces: Vec<Option<MaterialId>>,
    shape: ChunkShape,
    min_y: i64,
    max_y: i64,
    /// The last section resolved, and what it resolved to.
    resident: Cell<Option<(SectionCoord, Resident<'a>)>>,
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
        let descriptor = world.descriptor();
        let bounds = descriptor.bounds;
        Self {
            world,
            surfaces,
            shape: descriptor.shape,
            min_y: bounds.min_y,
            max_y: bounds.max_y,
            resident: Cell::new(None),
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

    /// Resolve a section, through the kept entry when it is the same one.
    fn resident(&self, section: SectionCoord) -> Resident<'a> {
        if let Some((kept, resident)) = self.resident.get() {
            if kept == section {
                return resident;
            }
        }
        let resident = match self.world.chunk(section.column()) {
            // Not resident: refuse the movement rather than invent an answer.
            None => Resident::Unloaded,
            Some(chunk) => chunk
                .section(section.y)
                .map_or(Resident::Air, Resident::Stored),
        };
        self.resident.set(Some((section, resident)));
        resident
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
        let (address, local) = self.shape.split_of(position);
        let section = match self.resident(address) {
            Resident::Unloaded => return VoxelShape::SOLID,
            Resident::Air => return VoxelShape::Empty,
            Resident::Stored(section) => section,
        };
        match section.get(local) {
            Ok(state) => self
                .surfaces
                .get(state.0 as usize)
                .copied()
                .flatten()
                .map_or(VoxelShape::Empty, VoxelShape::Cube),
            // A cell the section cannot address is not a cell physics may pass
            // through, for the same reason an absent column is not.
            Err(_) => VoxelShape::SOLID,
        }
    }

    /// The world's own counter, which moves on every block write and on every
    /// chunk arriving or leaving.
    ///
    /// Naming it here is what lets physics stop re-proving that a resting body
    /// is not inside terrain (`DEBT-0012`). The world bumps eagerly — a
    /// `chunk_mut` borrow counts whether or not it is written through — because
    /// the failure that matters is a change that goes uncounted.
    fn revision(&self) -> Option<u64> {
        Some(self.world.revision())
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

    /// The one property that matters about a cache: it must not change an
    /// answer. A view walked across a range accumulates cached sections; a view
    /// built fresh for each cell never has one. Every cell, both must agree.
    ///
    /// This is an equality rather than a property test on purpose — the same
    /// choice `DEBT-0029` made for `DenseSnapshot`. "Both are plausible" is what
    /// a wrong cache also satisfies.
    #[test]
    fn a_walked_view_answers_exactly_what_a_cold_view_answers() {
        let world = populated_world();
        let walked = WorldVoxels::new(&world);
        let surface = world.surface_height(4, 4);
        let mut checked = 0_u32;
        // Deep below the floor to well above the surface, and out past the edge
        // of the one generated column so the walk crosses into unloaded space.
        for y in (world.descriptor().bounds.min_y - 2)..=(surface + 3) {
            for x in [0_i64, 4, 31, 32, 33, 5_000] {
                let position = BlockPos::new(x, y, 4);
                let cold = WorldVoxels::new(&world);
                assert_eq!(
                    walked.shape_at(position),
                    cold.shape_at(position),
                    "cached and uncached disagree at {position:?}"
                );
                checked += 1;
            }
        }
        assert!(checked > 0, "the walk must actually visit cells");
    }

    /// A one-entry cache is weakest where the walk alternates, because every
    /// question evicts the previous answer. That is also where a cache that
    /// forgets to compare its key would look correct on a straight run and be
    /// wrong here.
    #[test]
    fn alternating_across_a_section_boundary_does_not_smear_one_answer_onto_the_other() {
        let world = populated_world();
        let voxels = WorldVoxels::new(&world);
        let surface = world.surface_height(4, 4);
        // Inside the generated column, and far outside it: different columns,
        // so different sections. The height is the first air cell above the
        // surface, because at the surface itself both cells answer solid — one
        // from terrain and one from refusal — and a fixture whose two halves
        // agree proves nothing. The `assert_ne!` below is what caught that.
        let inside = BlockPos::new(4, surface + 1, 4);
        let outside = BlockPos::new(5_000, surface + 1, 4);
        let inside_answer = voxels.shape_at(inside);
        let outside_answer = voxels.shape_at(outside);
        assert_ne!(
            inside_answer, outside_answer,
            "the fixture is pointless unless the two cells differ"
        );
        for _ in 0..8 {
            assert_eq!(voxels.shape_at(inside), inside_answer);
            assert_eq!(voxels.shape_at(outside), outside_answer);
        }
    }

    /// A resident column whose section is absent is air, and must not be
    /// confused with a column that is not resident at all — one lets a body
    /// through and the other stops it. The cache resolves both to one entry, so
    /// the distinction has to survive that.
    #[test]
    fn an_absent_section_inside_a_resident_column_is_air_not_refusal() {
        let world = populated_world();
        let voxels = WorldVoxels::new(&world);
        let high = BlockPos::new(4, world.descriptor().bounds.max_y, 4);
        assert!(
            !voxels.is_solid(high),
            "empty sky inside a loaded column is passable"
        );
        assert!(
            voxels.is_solid(BlockPos::new(5_000, world.descriptor().bounds.max_y, 4)),
            "the same height outside any loaded column is not"
        );
    }

    /// `Cell` costs `Sync` and keeps `Send`. `DEBT-0027` needs the second to
    /// move a job to a worker; nothing needs the first. A test rather than a
    /// comment, because the difference is invisible until something stops
    /// compiling somewhere else.
    #[test]
    fn a_view_is_still_send_so_a_worker_can_own_one() {
        const fn assert_send<T: Send>() {}
        assert_send::<WorldVoxels<'_>>();
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
