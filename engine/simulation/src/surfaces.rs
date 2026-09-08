//! The bridge between the voxel world and the mesher.
//!
//! `nexora-mesh` depends on `nexora-foundation` and nothing else, so it cannot
//! reach a `World` — the same arrangement physics has with `VoxelSource`
//! (ADR-0007) and streaming with `ResidencyBackend` (ADR-0008). This is the
//! adapter, and it lives here because `nexora-simulation` is the one crate
//! allowed to see both sides.
//!
//! # What a `SurfaceId` means
//!
//! `nexora_mesh::mesh::SurfaceId` was defined as *"an opaque handle: the caller
//! decides what the number means"*, and for one increment the caller had
//! nothing better to decide than the block's own state id. It now means **a
//! material's runtime id**, resolved through [`SurfaceTable`].
//!
//! Two consequences, and both of them are the point:
//!
//! * **Two blocks that share a material merge into one rectangle.** Cobble and
//!   mossy cobble drawn with the same stone material are one quad, not two.
//! * **A material declares how it composites**, so `occludes` can finally
//!   answer honestly. That is what closes DEBT-0026: glass stops hiding what
//!   is behind it, and the mesher needed no change at all — the trait method
//!   had a default for exactly this day.
//!
//! # A chunk that is not resident is not empty
//!
//! The dangerous default, and unchanged. If an unloaded neighbour read as
//! *empty*, every face along that border would survive culling, and the moment
//! the chunk arrived the wall would vanish — a visible flash at every streaming
//! boundary. So a non-resident cell **occludes**: faces appear when the data
//! does, never the other way round. Physics makes the same call for the same
//! reason.

use nexora_asset::registry::MaterialRegistry;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::BlockPos;
use nexora_mesh::mesh::SurfaceId;
use nexora_mesh::view::VoxelView;
use nexora_world::voxel::AIR;
use nexora_world::world::World;

/// The surface shown by a block whose material could not be resolved.
///
/// Reserved rather than absent. A block with no material is a **content
/// error**, and the two wrong answers are to draw nothing (the block silently
/// disappears from the world) and to pick some other material (it silently
/// becomes stone). Showing a distinct surface keeps the block visible and
/// keeps it distinguishable, and [`SurfaceTable::unmapped`] names every block
/// it happened to, so the error can be reported by someone who can return one.
pub const UNMAPPED_SURFACE: SurfaceId = SurfaceId(u32::MAX);

/// What each block state shows, and whether it hides what is behind it.
///
/// Resolved once into flat tables indexed by block state id, because the
/// mesher asks about a cell far more often than the registries change — which,
/// after they are frozen, is never. `WorldVoxels` resolves physics materials
/// the same way, for the same reason.
#[derive(Debug, Clone, Default)]
pub struct SurfaceTable {
    surface: Vec<Option<SurfaceId>>,
    occludes: Vec<bool>,
    unmapped: Vec<Identifier>,
}

impl SurfaceTable {
    /// The table this project had before materials existed.
    ///
    /// Every block shows its own state id and everything occludes. Kept
    /// because a world can be meshed without a material registry — the
    /// benchmark does it, and so does any test that cares about geometry
    /// rather than appearance — and because it makes what changed visible: the
    /// difference between this and [`SurfaceTable::resolved`] is exactly what
    /// materials bought.
    #[must_use]
    pub fn untextured(world: &World) -> Self {
        let count = world.blocks().len() + 1;
        let occludes = vec![true; count];
        let surface = (0..count)
            .map(|index| (index != AIR.0 as usize).then_some(SurfaceId(index as u32)))
            .collect();
        Self {
            surface,
            occludes,
            unmapped: Vec::new(),
        }
    }

    /// Start resolving a table against a material registry.
    #[must_use]
    pub fn resolved<'a>(
        world: &'a World,
        materials: &'a MaterialRegistry,
    ) -> SurfaceTableBuilder<'a> {
        let count = world.blocks().len() + 1;
        SurfaceTableBuilder {
            world,
            materials,
            table: Self {
                surface: vec![None; count],
                // An unresolved block occludes. The safe direction: a hole in
                // the world is worse than an extra hidden face.
                occludes: vec![true; count],
                unmapped: Vec::new(),
            },
        }
    }

    /// What a block state shows, or `None` when it shows nothing.
    #[must_use]
    pub fn surface_of(&self, state: u32) -> Option<SurfaceId> {
        self.surface.get(state as usize).copied().flatten()
    }

    /// Whether a block state hides the face of the neighbour touching it.
    #[must_use]
    pub fn occludes_at(&self, state: u32) -> bool {
        self.occludes.get(state as usize).copied().unwrap_or(true)
    }

    /// Every registered block that no material was assigned to.
    ///
    /// Empty is the only healthy answer once a world has content.
    #[must_use]
    pub fn unmapped(&self) -> &[Identifier] {
        &self.unmapped
    }
}

/// Assigns materials to blocks, one at a time, against a frozen registry.
#[derive(Debug)]
pub struct SurfaceTableBuilder<'a> {
    world: &'a World,
    materials: &'a MaterialRegistry,
    table: SurfaceTable,
}

impl SurfaceTableBuilder<'_> {
    /// Give a block the material it is drawn with.
    ///
    /// # Errors
    ///
    /// Returns an error when the block is not registered in this world, or the
    /// material is not registered in the material registry. Both are typos
    /// that would otherwise turn into a surface nobody chose, and
    /// `Registry System.md` §41 requires an unresolved reference to surface.
    pub fn assign(&mut self, block: &Identifier, material: &Identifier) -> Result<&mut Self> {
        let entry = self.world.blocks().get(block).ok_or_else(|| {
            unresolved("no block is registered under this id")
                .with_context("block", block.to_string())
        })?;
        let handle = self.materials.runtime_id_of(material).ok_or_else(|| {
            unresolved("no material is registered under this id")
                .with_context("block", block.to_string())
                .with_context("material", material.to_string())
        })?;
        let definition = self
            .materials
            .require(material)
            .expect("the handle resolved, so the material is present");

        let index = entry.runtime_id().0 as usize;
        if index >= self.table.surface.len() {
            self.table.surface.resize(index + 1, None);
            self.table.occludes.resize(index + 1, true);
        }
        self.table.surface[index] = Some(SurfaceId(handle.0));
        self.table.occludes[index] = definition.blend().occludes();
        Ok(self)
    }

    /// Finish, recording which blocks were left without a material.
    ///
    /// Air is never counted: the empty block has nothing to draw, and
    /// demanding a material for it would be demanding a texture for nothing.
    #[must_use]
    pub fn build(mut self) -> SurfaceTable {
        for identifier in self.world.blocks().identifiers() {
            let Some(entry) = self.world.blocks().get(&identifier) else {
                continue;
            };
            let index = entry.runtime_id().0 as usize;
            if index == AIR.0 as usize {
                self.table.surface[index] = None;
                self.table.occludes[index] = false;
                continue;
            }
            if self.table.surface[index].is_none() {
                self.table.surface[index] = Some(UNMAPPED_SURFACE);
                self.table.unmapped.push(identifier);
            }
        }
        self.table
    }
}

/// Reads a [`World`] as surfaces the mesher can consume.
#[derive(Debug)]
pub struct WorldSurfaces<'a> {
    world: &'a World,
    table: SurfaceTable,
}

impl<'a> WorldSurfaces<'a> {
    /// Borrow a world for meshing, with an explicit surface table.
    #[must_use]
    pub const fn new(world: &'a World, table: SurfaceTable) -> Self {
        Self { world, table }
    }

    /// Borrow a world for meshing with no materials assigned.
    #[must_use]
    pub fn untextured(world: &'a World) -> Self {
        Self {
            table: SurfaceTable::untextured(world),
            world,
        }
    }

    /// The table this view resolves through.
    #[must_use]
    pub const fn table(&self) -> &SurfaceTable {
        &self.table
    }
}

impl VoxelView for WorldSurfaces<'_> {
    fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
        match self.world.get_block(position) {
            Ok(state) if state != AIR => self.table.surface_of(state.0),
            // Air, or a chunk that is not resident. Neither shows a surface.
            _ => None,
        }
    }

    fn occludes(&self, position: BlockPos) -> bool {
        match self.world.get_block(position) {
            Ok(state) => state != AIR && self.table.occludes_at(state.0),
            // Not resident. Treated as occluding so the border stays hidden
            // until the neighbour loads — see the module documentation.
            Err(_) => true,
        }
    }
}

fn unresolved(message: &'static str) -> Error {
    Error::new(Domain::Content, "surface-table", message).with_recovery(Recovery::Quarantine)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_asset::material::{BlendMode, MaterialCategory, SurfaceMaterial};
    use nexora_asset::provenance::Provenance;
    use nexora_asset::texture::Resolution;
    use nexora_foundation::spatial::{BlockPos, ChunkCoord};
    use nexora_foundation::time::CalendarConfig;
    use nexora_mesh::mesh::{Facing, Quad};
    use nexora_mesh::view::Extent;
    use nexora_mesh::{mesh_region, ChunkMesh};
    use nexora_world::voxel::BlockStateId;
    use nexora_world::world::WorldDescriptor;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn world() -> World {
        let descriptor = WorldDescriptor::new("surfaces-test", 0x00C0_FFEE).expect("valid");
        let mut world = World::create(descriptor, CalendarConfig::earthlike()).expect("created");
        world.bring_online().expect("online");
        world
            .load_or_generate(ChunkCoord::new(0, 0))
            .expect("generated");
        world
    }

    fn material(path: &str, blend: BlendMode) -> SurfaceMaterial {
        SurfaceMaterial::builder(
            id(path),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .blended(blend)
        .build()
        .expect("valid material")
    }

    fn registry(entries: &[(&str, BlendMode)]) -> MaterialRegistry {
        let mut registry = MaterialRegistry::new().expect("registry");
        for (path, blend) in entries {
            registry
                .register(material(path, *blend))
                .expect("registration");
        }
        registry.freeze();
        registry
    }

    /// The state id of a built-in block, by its bare name.
    fn state_of(world: &World, block: &str) -> BlockStateId {
        BlockStateId(
            world
                .blocks()
                .runtime_id_of(&id(&format!("nexora:block/{block}")))
                .expect("a built-in block")
                .0,
        )
    }

    /// A row of blocks in open air, and the extent that covers exactly it.
    ///
    /// The height is taken from the tallest column the row spans, not from the
    /// first: terrain varies by the full amplitude across four columns, and a
    /// row placed relative to one of them can end up with rock sitting on top
    /// of another — which culls a face and makes the test count the wrong
    /// thing for the wrong reason.
    fn row(world: &mut World, blocks: &[&str]) -> (i64, Extent) {
        let y = (0..blocks.len() as i64)
            .map(|x| world.surface_height(x, 0))
            .max()
            .expect("a non-empty row")
            + 4;
        for (offset, block) in blocks.iter().enumerate() {
            world
                .set_block(BlockPos::new(offset as i64, y, 0), state_of(world, block))
                .expect("placed");
        }
        (
            y,
            Extent::new(BlockPos::new(0, y, 0), [blocks.len() as u32, 1, 1]).expect("extent"),
        )
    }

    fn top_faces(mesh: &ChunkMesh) -> Vec<&Quad> {
        mesh.quads
            .iter()
            .filter(|quad| {
                quad.axis == nexora_foundation::spatial::Axis::Y && quad.facing == Facing::Positive
            })
            .collect()
    }

    #[test]
    fn two_blocks_that_share_a_material_merge_into_one_rectangle() {
        // The payoff. Before materials, cobble and mossy cobble were two
        // surfaces because they were two block states, and the mesher could
        // not merge them however identical they looked.
        let mut world = world();
        let (_, extent) = row(&mut world, &["stone", "dirt", "stone", "dirt"]);

        let one = registry(&[("nexora:material/rock", BlendMode::Opaque)]);
        let mut table = SurfaceTable::resolved(&world, &one);
        table
            .assign(&id("nexora:block/stone"), &id("nexora:material/rock"))
            .unwrap();
        table
            .assign(&id("nexora:block/dirt"), &id("nexora:material/rock"))
            .unwrap();
        table
            .assign(&id("nexora:block/grass"), &id("nexora:material/rock"))
            .unwrap();
        let shared = table.build();
        assert!(shared.unmapped().is_empty(), "{:?}", shared.unmapped());

        let mesh = mesh_region(&WorldSurfaces::new(&world, shared), extent);
        let faces = top_faces(&mesh);
        assert_eq!(faces.len(), 1, "one material must merge to one rectangle");
        assert_eq!(faces[0].width * faces[0].height, 4);

        // And with two materials, the alternating blocks cannot merge.
        let materials = registry(&[
            ("nexora:material/rock", BlendMode::Opaque),
            ("nexora:material/soil", BlendMode::Opaque),
        ]);
        let mut table = SurfaceTable::resolved(&world, &materials);
        table
            .assign(&id("nexora:block/stone"), &id("nexora:material/rock"))
            .unwrap();
        table
            .assign(&id("nexora:block/dirt"), &id("nexora:material/soil"))
            .unwrap();
        table
            .assign(&id("nexora:block/grass"), &id("nexora:material/soil"))
            .unwrap();
        let split = table.build();

        let mesh = mesh_region(&WorldSurfaces::new(&world, split), extent);
        assert_eq!(
            top_faces(&mesh).len(),
            4,
            "four alternating materials cannot merge"
        );
    }

    #[test]
    fn a_transparent_material_stops_hiding_what_is_behind_it() {
        // DEBT-0026. The mesher assumed everything occludes because
        // `BlockDefinition` carries only `solid` and nothing could say
        // otherwise. A material can, and the mesher needed no change: the
        // trait method had a default for exactly this.
        let mut world = world();
        // Stone beside dirt; the dirt is what becomes glass.
        let (_, extent) = row(&mut world, &["stone", "dirt"]);

        let materials = registry(&[
            ("nexora:material/rock", BlendMode::Opaque),
            ("nexora:material/glass", BlendMode::Transparent),
        ]);

        let mut opaque = SurfaceTable::resolved(&world, &materials);
        opaque
            .assign(&id("nexora:block/stone"), &id("nexora:material/rock"))
            .unwrap();
        opaque
            .assign(&id("nexora:block/dirt"), &id("nexora:material/rock"))
            .unwrap();
        opaque
            .assign(&id("nexora:block/grass"), &id("nexora:material/rock"))
            .unwrap();
        let opaque = mesh_region(&WorldSurfaces::new(&world, opaque.build()), extent);

        let mut glazed = SurfaceTable::resolved(&world, &materials);
        glazed
            .assign(&id("nexora:block/stone"), &id("nexora:material/rock"))
            .unwrap();
        glazed
            .assign(&id("nexora:block/dirt"), &id("nexora:material/glass"))
            .unwrap();
        glazed
            .assign(&id("nexora:block/grass"), &id("nexora:material/rock"))
            .unwrap();
        let glazed = mesh_region(&WorldSurfaces::new(&world, glazed.build()), extent);

        // The face between the two cells is hidden when both are opaque and
        // survives when the second is transparent, so the glazed mesh has more
        // surface than the opaque one.
        assert!(
            glazed.area() > opaque.area(),
            "transparent {} should expose more surface than opaque {}",
            glazed.area(),
            opaque.area()
        );
        assert_eq!(
            glazed.area() - opaque.area(),
            2,
            "exactly the two faces either side of the shared plane"
        );
    }

    #[test]
    fn a_block_with_no_material_is_visible_and_reported() {
        // The two wrong answers: draw nothing, and the block vanishes from the
        // world; pick something, and it silently becomes stone.
        let mut world = world();
        let (_, extent) = row(&mut world, &["stone", "dirt"]);

        let materials = registry(&[("nexora:material/rock", BlendMode::Opaque)]);
        let mut table = SurfaceTable::resolved(&world, &materials);
        table
            .assign(&id("nexora:block/stone"), &id("nexora:material/rock"))
            .unwrap();
        let table = table.build();

        let unmapped: Vec<String> = table.unmapped().iter().map(ToString::to_string).collect();
        assert_eq!(unmapped, ["nexora:block/dirt", "nexora:block/grass"]);
        // Air is never counted: the empty block has nothing to draw.
        assert!(!unmapped.contains(&"nexora:block/air".to_owned()));

        // And the unmapped block is still there to be seen.
        let mesh = mesh_region(&WorldSurfaces::new(&world, table), extent);
        assert!(mesh
            .quads
            .iter()
            .any(|quad| quad.surface == UNMAPPED_SURFACE));
    }

    #[test]
    fn assigning_something_that_does_not_exist_is_refused() {
        let world = world();
        let materials = registry(&[("nexora:material/rock", BlendMode::Opaque)]);

        let mut table = SurfaceTable::resolved(&world, &materials);
        let err = table
            .assign(&id("nexora:block/ruby"), &id("nexora:material/rock"))
            .expect_err("an unknown block must be refused");
        assert!(err.to_string().contains("nexora:block/ruby"), "{err}");

        let err = table
            .assign(&id("nexora:block/stone"), &id("nexora:material/ruby"))
            .expect_err("an unknown material must be refused");
        assert_eq!(err.recovery(), Recovery::Quarantine);
        assert!(err.to_string().contains("nexora:material/ruby"), "{err}");
    }

    #[test]
    fn air_shows_nothing_and_hides_nothing() {
        let world = world();
        let materials = registry(&[("nexora:material/rock", BlendMode::Opaque)]);
        let mut table = SurfaceTable::resolved(&world, &materials);
        for block in ["stone", "dirt", "grass"] {
            table
                .assign(
                    &id(&format!("nexora:block/{block}")),
                    &id("nexora:material/rock"),
                )
                .unwrap();
        }
        let table = table.build();

        assert!(table.surface_of(AIR.0).is_none());
        assert!(!table.occludes_at(AIR.0));

        // In the world, an air cell reads the same way.
        let view = WorldSurfaces::new(&world, table);
        let sky = BlockPos::new(0, world.surface_height(0, 0) + 20, 0);
        assert!(view.surface_at(sky).is_none());
        assert!(!view.occludes(sky));
    }

    #[test]
    fn a_chunk_that_is_not_resident_still_occludes() {
        // Unchanged, and the reason is unchanged: a mesh built against absent
        // data is wrong in a way nothing detects, and the symptom is a wall
        // that vanishes the moment the chunk arrives.
        let world = world();
        let view = WorldSurfaces::untextured(&world);
        let far = BlockPos::new(10_000, 64, 10_000);
        assert!(view.surface_at(far).is_none());
        assert!(
            view.occludes(far),
            "an absent neighbour must hide the border"
        );
    }

    #[test]
    fn the_untextured_table_is_what_this_project_had_before_materials() {
        let world = world();
        let table = SurfaceTable::untextured(&world);
        // Every block shows its own state id, and everything occludes.
        for block in ["stone", "dirt", "grass"] {
            let state = state_of(&world, block);
            assert_eq!(table.surface_of(state.0), Some(SurfaceId(state.0)));
            assert!(table.occludes_at(state.0));
        }
        assert!(table.unmapped().is_empty());
    }

    #[test]
    fn a_material_registry_and_a_block_registry_number_things_independently() {
        // The reason a surface id is resolved rather than assumed: the two
        // registries assign their own runtime ids, and nothing makes them
        // agree. A table that used the block's id as a surface id would be
        // right only by coincidence.
        let world = world();
        let materials = registry(&[
            ("nexora:material/alpha", BlendMode::Opaque),
            ("nexora:material/beta", BlendMode::Opaque),
            ("nexora:material/rock", BlendMode::Opaque),
        ]);
        let mut table = SurfaceTable::resolved(&world, &materials);
        for block in ["stone", "dirt", "grass"] {
            table
                .assign(
                    &id(&format!("nexora:block/{block}")),
                    &id("nexora:material/rock"),
                )
                .unwrap();
        }
        let table = table.build();

        let stone = state_of(&world, "stone");
        let rock = materials
            .runtime_id_of(&id("nexora:material/rock"))
            .unwrap();
        assert_eq!(table.surface_of(stone.0), Some(SurfaceId(rock.0)));
        assert_ne!(stone.0, rock.0, "the two registries disagree, as they may");
    }
}
