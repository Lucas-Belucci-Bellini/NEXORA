//! The world the client shows: generated around the origin before the frame
//! loop starts, meshed once, one region per chunk column — and where in it
//! the player starts.
//!
//! Generation and meshing happen before the first frame, on purpose. Both
//! still run on the calling thread (DEBT-0018, DEBT-0027), and a frame loop
//! that generated while it ran would measure them rather than itself
//! (DEBT-0041). Streaming into the pass is not built, so the player can walk
//! off the drawn columns onto the generated ring, which is not drawn, and is
//! stopped where generation ends — an unloaded column is solid to physics.

use std::collections::BTreeSet;
use std::f64::consts::PI;

use nexora_camera::Projection;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{BlockPos, ChunkCoord};
use nexora_mesh::{mesh_region, ChunkMesh, DenseSnapshot, Extent};
use nexora_render::atlas::Atlas;
use nexora_render::{chunk_vertices, corners_in, textured_vertices, Corners};
use nexora_simulation::{
    find_walkable_run, ColumnArea, EditArea, Run, SurfaceTable, WorldSurfaces, WorldVoxels,
};
use nexora_world::world::World;

/// Blocks of margin above the highest surface and below the lowest in the
/// drawn band.
pub const BAND_MARGIN: i64 = 8;

/// The shortest run of drawn columns the player may start on.
///
/// Five, so the first second of W — about 4.2 blocks at 20 ticks — never
/// reaches the end of the columns known to be walkable: from the centre of
/// the first column the body's leading face has 4.2 blocks of run ahead.
pub const CLIENT_RUN: u32 = 5;

/// How far the player starts looking down, in radians: 15°.
///
/// The one tunable of the first frame, which is judged from the eye: level
/// would put the horizon mid-screen over a field of pillars, and a little
/// down puts the run being walked in view.
pub const SPAWN_PITCH: f64 = -15.0 * PI / 180.0;

/// What the client draws: one meshed region per chunk column, and the region
/// that holds them all.
#[derive(Debug)]
pub struct Scene {
    /// Each drawn column's region and its opaque mesh.
    pub regions: Vec<(Extent, ChunkMesh)>,
    /// The union of the drawn regions: what the reference ray cast walks.
    pub bounds: Extent,
    /// Columns generated: the drawn ones and a ring around them, so the
    /// drawn edge is meshed against real neighbours.
    pub generated: usize,
    /// Where the player starts: a run of at least [`CLIENT_RUN`] drawn
    /// columns, found in the terrain rather than invented.
    pub spawn: Run,
    /// How the view projects: 70° high, near 0.1, far 512.
    pub projection: Projection,
    /// What each block shows: its material's surface when content gives it
    /// one, its own state id otherwise (`SurfaceTable::untextured`).
    pub table: SurfaceTable,
    /// The albedo of those surfaces, when the client draws textures
    /// (ADR-0037); `None` draws faces in their direction's colour.
    pub atlas: Option<Atlas>,
}

impl Scene {
    /// Mesh again every drawn region an edit to `cells` can change, and
    /// return their indices, ascending, each with the mesh it had.
    ///
    /// A region changes when it holds an edited cell, or holds one of the
    /// cell's six neighbours: the neighbour's face towards the cell appears
    /// or disappears with it. Cells outside every drawn region change
    /// nothing drawn.
    ///
    /// Each region is read once into a [`DenseSnapshot`] and meshed from it:
    /// the same mesh as meshing the world (the mesh crate holds the two
    /// equal), at a tenth of the cost (DEBT-0029), which is what an edit
    /// costs a frame.
    ///
    /// # Errors
    ///
    /// A region is larger than one snapshot holds.
    pub fn remesh(&mut self, world: &World, cells: &[BlockPos]) -> Result<Vec<(usize, ChunkMesh)>> {
        let mut touched = BTreeSet::new();
        for cell in cells {
            for [dx, dy, dz] in [
                [0, 0, 0],
                [1, 0, 0],
                [-1, 0, 0],
                [0, 1, 0],
                [0, -1, 0],
                [0, 0, 1],
                [0, 0, -1],
            ] {
                let near = BlockPos::new(cell.x + dx, cell.y + dy, cell.z + dz);
                if let Some(index) = self
                    .regions
                    .iter()
                    .position(|(region, _)| holds(region, near))
                {
                    touched.insert(index);
                }
            }
        }
        let view = self.view(world);
        let mut remeshed = Vec::with_capacity(touched.len());
        for index in touched {
            let region = self.regions[index].0;
            let snapshot = DenseSnapshot::read(&view, region)?;
            let fresh = mesh_region(&snapshot, region).opaque;
            remeshed.push((index, std::mem::replace(&mut self.regions[index].1, fresh)));
        }
        Ok(remeshed)
    }

    /// After `remeshed` changed (each with the mesh it had): the regions
    /// whose vertices may have changed, ascending.
    ///
    /// A region's vertices are its mesh split at the corners lying in its
    /// closed box ([`Corners::touching`]). So they change when its own mesh
    /// did, or when a neighbour's corners on their shared boundary did — and
    /// for no other reason: an edit away from a region's edges leaves its
    /// neighbours' vertices as they were, and they are not uploaded again.
    #[must_use]
    pub fn reupload(&self, remeshed: &[(usize, ChunkMesh)]) -> Vec<usize> {
        let mut changed: BTreeSet<usize> = remeshed.iter().map(|(index, _)| *index).collect();
        for (index, old) in remeshed {
            let new = &self.regions[*index].1;
            for neighbour in self.around(&[*index]) {
                if changed.contains(&neighbour) {
                    continue;
                }
                let boundary = self.regions[neighbour].0;
                if corners_in(old, boundary) != corners_in(new, boundary) {
                    changed.insert(neighbour);
                }
            }
        }
        changed.into_iter().collect()
    }

    /// Region `index`'s vertex bytes: its mesh split at every corner of it
    /// and its neighbours that lies in its closed box (DEBT-0047). The same
    /// bytes as splitting at the corners of every drawn mesh, from a tenth
    /// of the quads.
    ///
    /// # Errors
    ///
    /// The mesh does not belong to its region.
    pub fn vertices(&self, index: usize) -> Result<Vec<u8>> {
        let (region, mesh) = &self.regions[index];
        let meshes: Vec<&ChunkMesh> = self
            .around(&[index])
            .into_iter()
            .map(|near| &self.regions[near].1)
            .collect();
        let corners = Corners::touching(&meshes, *region);
        match &self.atlas {
            Some(atlas) => textured_vertices(mesh, region.origin, &corners, atlas),
            None => chunk_vertices(mesh, region.origin, &corners),
        }
    }

    /// The world as this scene's surfaces: what is meshed, and what the
    /// reference ray cast reads.
    #[must_use]
    pub fn view<'a>(&self, world: &'a World) -> WorldSurfaces<'a> {
        WorldSurfaces::new(world, self.table.clone())
    }

    /// The regions within one column of any of `indices`, themselves
    /// included, ascending.
    fn around(&self, indices: &[usize]) -> Vec<usize> {
        let column = |index: usize| {
            let region = self.regions[index].0;
            (
                region.origin.x.div_euclid(i64::from(region.size[0])),
                region.origin.z.div_euclid(i64::from(region.size[2])),
            )
        };
        let centres: Vec<(i64, i64)> = indices.iter().map(|&index| column(index)).collect();
        (0..self.regions.len())
            .filter(|&index| {
                let (x, z) = column(index);
                centres
                    .iter()
                    .any(|(cx, cz)| (x - cx).abs() <= 1 && (z - cz).abs() <= 1)
            })
            .collect()
    }

    /// The cells the player may edit: every drawn cell (ADR-0036). An edit
    /// outside them could not be shown (DEBT-0052: what is drawn is fixed
    /// when the client starts).
    #[must_use]
    pub fn edit_area(&self) -> EditArea {
        let origin = self.bounds.origin;
        let [sx, sy, sz] = self.bounds.size.map(i64::from);
        EditArea {
            min: origin,
            max: BlockPos::new(origin.x + sx, origin.y + sy, origin.z + sz),
        }
    }
}

/// Whether `region` holds `cell`.
fn holds(region: &Extent, cell: BlockPos) -> bool {
    let origin = region.origin;
    let [sx, sy, sz] = region.size.map(i64::from);
    (origin.x..origin.x + sx).contains(&cell.x)
        && (origin.y..origin.y + sy).contains(&cell.y)
        && (origin.z..origin.z + sz).contains(&cell.z)
}

/// Generate columns `-radius..=radius` on both axes and one ring more, mesh
/// the inner ones in a vertical band around the surface, and find where the
/// player starts: the first run of [`CLIENT_RUN`] walkable drawn columns,
/// searched from the centre outward.
///
/// # Errors
///
/// `radius` is negative, generation failed, a region is larger than the
/// mesher accepts, or no run that long exists in the drawn columns.
pub fn build(
    world: &mut World,
    radius: i64,
    table: SurfaceTable,
    atlas: Option<Atlas>,
) -> Result<Scene> {
    if !(0..=8).contains(&radius) {
        return Err(Error::new(
            Domain::World,
            "client-scene",
            "the radius must be 0 to 8 columns",
        )
        .with_recovery(Recovery::Reject)
        .with_context("radius", radius.to_string()));
    }
    let ring = radius + 1;
    let mut generated = 0;
    for x in -ring..=ring {
        for z in -ring..=ring {
            world.load_or_generate(ChunkCoord::new(x, z))?;
            generated += 1;
        }
    }
    let shape = world.descriptor().shape;
    let (sx, sz) = (i64::from(shape.size_x()), i64::from(shape.size_z()));
    let (min_x, max_x) = (-radius * sx, (radius + 1) * sx);
    let (min_z, max_z) = (-radius * sz, (radius + 1) * sz);
    let (mut low, mut high) = (i64::MAX, i64::MIN);
    for x in min_x..max_x {
        for z in min_z..max_z {
            let h = world.surface_height(x, z);
            low = low.min(h);
            high = high.max(h);
        }
    }
    let bottom = low - BAND_MARGIN;
    let height = u32::try_from(high + BAND_MARGIN - bottom)
        .map_err(|_| Error::new(Domain::World, "client-scene", "the band has no height"))?;

    let view = WorldSurfaces::new(world, table.clone());
    let mut regions = Vec::new();
    for cx in -radius..=radius {
        for cz in -radius..=radius {
            let region = Extent::new(
                BlockPos::new(cx * sx, bottom, cz * sz),
                [shape.size_x(), height, shape.size_z()],
            )?;
            regions.push((region, mesh_region(&view, region).opaque));
        }
    }
    let bounds = Extent::new(
        BlockPos::new(min_x, bottom, min_z),
        [
            u32::try_from(max_x - min_x).unwrap_or(u32::MAX),
            height,
            u32::try_from(max_z - min_z).unwrap_or(u32::MAX),
        ],
    )?;

    // The player starts inside what is drawn, read in the drawn band: every
    // cell between a column's top and the band's ceiling is known empty.
    let drawn = ColumnArea {
        min_x,
        min_z,
        max_x,
        max_z,
        floor_y: bottom,
        ceiling_y: bottom + i64::from(height) - 1,
    };
    let spawn = find_walkable_run(&WorldVoxels::new(world), &drawn, CLIENT_RUN)?;
    Ok(Scene {
        regions,
        bounds,
        generated,
        spawn,
        projection: Projection::perspective(70f64.to_radians(), 0.1, 512.0)?,
        table,
        atlas,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::time::CalendarConfig;
    use nexora_world::world::WorldDescriptor;

    fn world() -> World {
        let mut world = World::create(
            WorldDescriptor::new("client-scene", 7).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        world.bring_online().unwrap();
        world
    }

    /// A scene with no content: every block its own surface, faces by
    /// direction.
    fn build_plain(world: &mut World, radius: i64) -> Result<Scene> {
        let table = SurfaceTable::untextured(world);
        build(world, radius, table, None)
    }

    #[test]
    fn radius_one_draws_nine_columns_generated_with_a_ring() {
        let mut world = world();
        let scene = build_plain(&mut world, 1).unwrap();
        assert_eq!(scene.regions.len(), 9);
        assert_eq!(scene.generated, 25);
        assert_eq!(world.chunk_count(), 25);
        assert_eq!(scene.bounds.size[0], 96);
        assert_eq!(scene.bounds.size[2], 96);
        assert!(scene.regions.iter().all(|(_, mesh)| !mesh.is_empty()));
    }

    /// Every region sits inside the bounds the reference walks, and they
    /// tile it without overlap.
    #[test]
    fn the_regions_tile_the_bounds() {
        let mut world = world();
        let scene = build_plain(&mut world, 1).unwrap();
        let cells: u64 = scene.regions.iter().map(|(region, _)| region.cells()).sum();
        assert_eq!(cells, scene.bounds.cells());
        for (region, _) in &scene.regions {
            assert!(
                region.origin.x >= scene.bounds.origin.x
                    && region.origin.z >= scene.bounds.origin.z
            );
            assert_eq!(region.origin.y, scene.bounds.origin.y);
        }
    }

    /// The player starts inside what is drawn, on a run long enough for a
    /// second of W, with its eye inside the band the ray cast walks.
    #[test]
    fn the_spawn_is_inside_the_drawn_columns_with_a_clear_run() {
        use nexora_simulation::player::EYE_HEIGHT;
        let mut world = world();
        let scene = build_plain(&mut world, 1).unwrap();
        let bounds = scene.bounds;
        let (min_x, min_z) = (bounds.origin.x, bounds.origin.z);
        let (max_x, max_z) = (
            min_x + i64::from(bounds.size[0]),
            min_z + i64::from(bounds.size[2]),
        );
        let run = scene.spawn;
        assert!(run.length >= CLIENT_RUN);
        for (x, z) in run.columns() {
            assert!(
                (min_x..max_x).contains(&x) && (min_z..max_z).contains(&z),
                "({x}, {z}) is not drawn"
            );
        }
        let eye = run.feet_y as f64 + EYE_HEIGHT;
        let (bottom, top) = (
            bounds.origin.y as f64,
            (bounds.origin.y + i64::from(bounds.size[1])) as f64,
        );
        assert!(
            eye > bottom && eye < top,
            "eye {eye} outside {bottom}..{top}"
        );
    }

    // The spawn looks a little down, never up and never steeply: checked
    // when the crate compiles, not when a test runs.
    const _: () = assert!(SPAWN_PITCH < 0.0 && SPAWN_PITCH > -std::f64::consts::FRAC_PI_4);

    /// The incremental update after an edit gives every region exactly the
    /// vertices a from-scratch build of the edited world gives it, split at
    /// the corners of every drawn mesh: the regions [`Scene::reupload`] names
    /// through [`Scene::vertices`], and every other region with the vertices
    /// it already had. Edits sit on a region's corner, its edge, and in the
    /// ring outside what is drawn, at radius 2 so some regions are two
    /// columns from any change.
    #[test]
    fn an_incremental_update_draws_what_a_fresh_build_would() {
        use nexora_world::voxel::AIR;

        let all_corners = |scene: &Scene| -> Vec<Vec<u8>> {
            let meshes: Vec<&ChunkMesh> = scene.regions.iter().map(|(_, mesh)| mesh).collect();
            let corners = Corners::of(&meshes);
            scene
                .regions
                .iter()
                .map(|(region, mesh)| chunk_vertices(mesh, region.origin, &corners).unwrap())
                .collect()
        };

        let mut world = world();
        let mut scene = build_plain(&mut world, 2).unwrap();
        let mut drawn: Vec<Vec<u8>> = (0..scene.regions.len())
            .map(|index| scene.vertices(index).unwrap())
            .collect();
        assert!(
            drawn == all_corners(&scene),
            "a region's own corners give what every corner gives"
        );
        let stone = world
            .block_id(&nexora_foundation::ident::Identifier::parse("nexora:block/stone").unwrap())
            .unwrap();
        let size = i64::from(world.descriptor().shape.size_x());
        let surface = |world: &World, x: i64, z: i64| world.surface_height(x, z);
        let edits = [
            (0, 0),            // a region's corner
            (size - 1, 7),     // its edge, towards +X
            (3 * size + 8, 5), // the generated ring: not drawn
        ];
        for (x, z) in edits {
            // Two edits, each applied incrementally, then one comparison: the
            // updates compose.
            let top = surface(&world, x, z);
            let mut remeshed_any = false;
            for (cell, state) in [
                (BlockPos::new(x, top, z), AIR),
                (BlockPos::new(x, top + 2, z), stone),
            ] {
                world.set_block(cell, state).unwrap();
                let remeshed = scene.remesh(&world, &[cell]).unwrap();
                remeshed_any |= !remeshed.is_empty();
                for index in scene.reupload(&remeshed) {
                    drawn[index] = scene.vertices(index).unwrap();
                }
            }
            let mut fresh = Scene {
                regions: scene.regions.clone(),
                ..scene_shape(&scene)
            };
            let view = scene.view(&world);
            for (region, mesh) in &mut fresh.regions {
                let snapshot = DenseSnapshot::read(&view, *region).unwrap();
                *mesh = mesh_region(&snapshot, *region).opaque;
            }
            assert_eq!(
                fresh.regions, scene.regions,
                "the remeshed regions are what meshing everything again gives"
            );
            let expected = all_corners(&fresh);
            for (index, (incremental, expected)) in drawn.iter().zip(&expected).enumerate() {
                assert!(
                    incremental == expected,
                    "region {index} differs after editing column ({x}, {z})"
                );
            }
            assert_eq!(remeshed_any, x < 3 * size, "the ring is not drawn");
        }
    }

    /// A scene with no regions, everything else copied: for building a fresh
    /// comparison scene in a test.
    fn scene_shape(scene: &Scene) -> Scene {
        Scene {
            regions: Vec::new(),
            bounds: scene.bounds,
            generated: scene.generated,
            spawn: scene.spawn,
            projection: scene.projection,
            table: scene.table.clone(),
            atlas: scene.atlas.clone(),
        }
    }

    #[test]
    fn the_edit_area_is_what_is_drawn() {
        let mut world = world();
        let scene = build_plain(&mut world, 1).unwrap();
        let area = scene.edit_area();
        let bounds = scene.bounds;
        assert_eq!(area.min, bounds.origin);
        assert!(area.contains(bounds.origin));
        let [sx, sy, sz] = bounds.size.map(i64::from);
        let o = bounds.origin;
        assert!(area.contains(BlockPos::new(o.x + sx - 1, o.y + sy - 1, o.z + sz - 1)));
        assert!(!area.contains(BlockPos::new(o.x + sx, o.y, o.z)));
        assert!(!area.contains(BlockPos::new(o.x, o.y + sy, o.z)));
        assert!(!area.contains(BlockPos::new(o.x, o.y - 1, o.z)));
    }

    #[test]
    fn a_radius_out_of_range_is_refused() {
        let mut world = world();
        assert!(build_plain(&mut world, -1).is_err());
        assert!(build_plain(&mut world, 9).is_err());
    }
}
