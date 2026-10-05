//! The world the client shows: one meshed region per chunk column, in a
//! square around the player that follows it (ADR-0038) — and where in it the
//! player starts.
//!
//! The scene draws what residency made resident ([`crate::residency`]); it
//! never generates a column itself, because a column generated around the
//! streaming backend would come back from the seed without its edits. The
//! square moves when the player's column changes: the columns entering it
//! are meshed, the ones leaving are dropped, and the ones kept keep their
//! meshes. Meshing still runs on the calling thread (DEBT-0018, DEBT-0027),
//! and what a move costs a frame is reported.
//!
//! The vertical band is the generator's: every surface it can produce, with
//! a margin above and below, the same wherever the square is. Edits are
//! limited to it (DEBT-0052).

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

/// The widest square the client draws: 8 columns each side of the player.
pub const MAX_RADIUS: i64 = 8;

/// What the client draws: one meshed region per chunk column, and the region
/// that holds them all.
#[derive(Debug)]
pub struct Scene {
    /// Each drawn column's region and its opaque mesh, by column: x
    /// ascending, then z.
    pub regions: Vec<(Extent, ChunkMesh)>,
    /// The union of the drawn regions: what the reference ray cast walks.
    pub bounds: Extent,
    /// The column the square is centred on.
    pub centre: ChunkCoord,
    /// Columns drawn on each side of the centre.
    pub radius: i64,
    /// How the view projects: 70° high, near 0.1, far 512.
    pub projection: Projection,
    /// What each block shows: its material's surface when content gives it
    /// one, its own state id otherwise (`SurfaceTable::untextured`).
    pub table: SurfaceTable,
    /// The albedo of those surfaces, when the client draws textures
    /// (ADR-0037); `None` draws faces in their direction's colour.
    pub atlas: Option<Atlas>,
    /// A column's width and depth, in blocks.
    column: (i64, i64),
}

/// What moving the square did, for whoever holds the regions' vertices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shift {
    /// For each region now drawn, the index it had before the move, if it
    /// was drawn then.
    pub from: Vec<Option<usize>>,
    /// The regions whose vertices are not the ones they had: every column
    /// that entered the square, and every kept one whose neighbourhood in
    /// the square changed (its corners are split at its neighbours',
    /// DEBT-0047). Ascending.
    pub reupload: Vec<usize>,
    /// Columns meshed: those that entered the square.
    pub meshed: usize,
    /// Columns that left the square.
    pub dropped: usize,
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
            let fresh = mesh(&view, region)?;
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

    /// Move the square to `centre`: mesh the columns that enter it, drop the
    /// ones that leave, keep the rest with their meshes, and say which
    /// regions' vertices changed.
    ///
    /// A kept region's mesh is still right: a move changes no block, and the
    /// ring a region is meshed against was resident when it was meshed. Its
    /// vertices are not when a neighbour entered or left the square, because
    /// they are split at the corners of the drawn neighbours. Every other
    /// region's vertices are byte for byte the ones it had, and a test holds
    /// the result to a scene built at `centre` from scratch.
    ///
    /// # Errors
    ///
    /// A column the square or its ring needs is not resident, or a region is
    /// larger than one snapshot holds.
    pub fn recentre(&mut self, world: &World, centre: ChunkCoord) -> Result<Shift> {
        resident(world, centre, self.radius)?;
        let old_columns = columns(self.centre, self.radius);
        let new_columns = columns(centre, self.radius);
        let mut old: Vec<Option<(Extent, ChunkMesh)>> = std::mem::take(&mut self.regions)
            .into_iter()
            .map(Some)
            .collect();
        let view = self.view(world);
        let mut from = Vec::with_capacity(new_columns.len());
        let mut meshed = 0;
        for &column in &new_columns {
            match old_columns.iter().position(|&kept| kept == column) {
                Some(index) => {
                    let region = old[index].take().ok_or_else(|| {
                        Error::new(Domain::World, "client-scene", "a region was kept twice")
                    })?;
                    self.regions.push(region);
                    from.push(Some(index));
                }
                None => {
                    let region = self.region_of(column)?;
                    self.regions.push((region, mesh(&view, region)?));
                    from.push(None);
                    meshed += 1;
                }
            }
        }
        let dropped = old.iter().filter(|region| region.is_some()).count();
        let near = |of: ChunkCoord, among: &[ChunkCoord]| -> Vec<ChunkCoord> {
            among
                .iter()
                .copied()
                .filter(|other| (other.x - of.x).abs() <= 1 && (other.z - of.z).abs() <= 1)
                .collect()
        };
        let reupload = new_columns
            .iter()
            .enumerate()
            .filter(|&(index, &column)| {
                from[index].is_none() || near(column, &old_columns) != near(column, &new_columns)
            })
            .map(|(index, _)| index)
            .collect();
        self.centre = centre;
        self.bounds = bounds_of(centre, self.radius, self.column)?;
        Ok(Shift {
            from,
            reupload,
            meshed,
            dropped,
        })
    }

    /// The column a point of the world is in.
    #[must_use]
    pub fn column_of(&self, x: f64, z: f64) -> ChunkCoord {
        column_of(self.column, x, z)
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

    /// The cells the player may edit: every drawn cell (ADR-0036), wherever
    /// the square is now (ADR-0038). Not above or below the band
    /// (DEBT-0052).
    #[must_use]
    pub fn edit_area(&self) -> EditArea {
        let origin = self.bounds.origin;
        let [sx, sy, sz] = self.bounds.size.map(i64::from);
        EditArea {
            min: origin,
            max: BlockPos::new(origin.x + sx, origin.y + sy, origin.z + sz),
        }
    }

    /// Where a player with nowhere to go on from starts: the first run of
    /// [`CLIENT_RUN`] walkable drawn columns, searched from the centre
    /// outward, read in the drawn band — every cell between a column's top
    /// and the band's ceiling is known empty.
    ///
    /// # Errors
    ///
    /// No run that long exists in the drawn columns.
    pub fn spawn_run(&self, world: &World) -> Result<Run> {
        let origin = self.bounds.origin;
        let [sx, sy, sz] = self.bounds.size.map(i64::from);
        let drawn = ColumnArea {
            min_x: origin.x,
            min_z: origin.z,
            max_x: origin.x + sx,
            max_z: origin.z + sz,
            floor_y: origin.y,
            ceiling_y: origin.y + sy - 1,
        };
        find_walkable_run(&WorldVoxels::new(world), &drawn, CLIENT_RUN)
    }

    fn region_of(&self, column: ChunkCoord) -> Result<Extent> {
        region_of(column, self.column)
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

/// The band every drawn region spans: the lowest surface the generator can
/// make, less [`BAND_MARGIN`], to the highest, plus it.
#[must_use]
pub fn band() -> (i64, u32) {
    let (low, high) = World::surface_range();
    let bottom = low - BAND_MARGIN;
    // Positive by construction: `high >= low`.
    (bottom, (high + BAND_MARGIN - bottom).unsigned_abs() as u32)
}

/// The columns a square of `radius` around `centre` draws: x ascending, then
/// z.
#[must_use]
pub fn columns(centre: ChunkCoord, radius: i64) -> Vec<ChunkCoord> {
    let mut all = Vec::new();
    for x in -radius..=radius {
        for z in -radius..=radius {
            all.push(ChunkCoord::new(centre.x + x, centre.z + z));
        }
    }
    all
}

/// The columns that square needs resident: itself and one ring more, so its
/// edge is meshed against real neighbours.
#[must_use]
pub fn needed(centre: ChunkCoord, radius: i64) -> Vec<ChunkCoord> {
    columns(centre, radius + 1)
}

/// Refuse a square whose columns or ring are not resident, naming the first.
fn resident(world: &World, centre: ChunkCoord, radius: i64) -> Result<()> {
    match needed(centre, radius)
        .into_iter()
        .find(|column| world.chunk(*column).is_none())
    {
        None => Ok(()),
        Some(column) => Err(Error::new(
            Domain::World,
            "client-scene",
            "a column the drawn square needs is not resident",
        )
        .with_recovery(Recovery::Manual)
        .with_context("chunk", format!("{},{}", column.x, column.z))),
    }
}

fn region_of(column: ChunkCoord, (sx, sz): (i64, i64)) -> Result<Extent> {
    let (bottom, height) = band();
    Extent::new(
        BlockPos::new(column.x * sx, bottom, column.z * sz),
        [
            u32::try_from(sx).unwrap_or(u32::MAX),
            height,
            u32::try_from(sz).unwrap_or(u32::MAX),
        ],
    )
}

fn bounds_of(centre: ChunkCoord, radius: i64, (sx, sz): (i64, i64)) -> Result<Extent> {
    let (bottom, height) = band();
    let side = |size: i64| u32::try_from((2 * radius + 1) * size).unwrap_or(u32::MAX);
    Extent::new(
        BlockPos::new((centre.x - radius) * sx, bottom, (centre.z - radius) * sz),
        [side(sx), height, side(sz)],
    )
}

/// One region's opaque mesh, read through a snapshot (DEBT-0029).
fn mesh(view: &WorldSurfaces<'_>, region: Extent) -> Result<ChunkMesh> {
    let snapshot = DenseSnapshot::read(view, region)?;
    Ok(mesh_region(&snapshot, region).opaque)
}

/// Refuse a radius outside `0..=`[`MAX_RADIUS`]: before anything is made
/// resident for it.
///
/// # Errors
///
/// [`Recovery::Reject`], naming the radius.
pub fn check_radius(radius: i64) -> Result<()> {
    if (0..=MAX_RADIUS).contains(&radius) {
        return Ok(());
    }
    Err(Error::new(
        Domain::World,
        "client-scene",
        "the radius must be 0 to 8 columns",
    )
    .with_recovery(Recovery::Reject)
    .with_context("radius", radius.to_string()))
}

/// The column of `world` a point is in.
#[must_use]
pub fn column_in(world: &World, x: f64, z: f64) -> ChunkCoord {
    let shape = world.descriptor().shape;
    column_of((i64::from(shape.size_x()), i64::from(shape.size_z())), x, z)
}

fn column_of((sx, sz): (i64, i64), x: f64, z: f64) -> ChunkCoord {
    ChunkCoord::new(
        (x.floor() as i64).div_euclid(sx),
        (z.floor() as i64).div_euclid(sz),
    )
}

/// Mesh the square of `radius` columns around `centre`, from columns
/// residency already holds.
///
/// # Errors
///
/// `radius` is outside `0..=`[`MAX_RADIUS`], a column the square or its ring
/// needs is not resident, or a region is larger than the mesher accepts.
pub fn build(
    world: &World,
    centre: ChunkCoord,
    radius: i64,
    table: SurfaceTable,
    atlas: Option<Atlas>,
) -> Result<Scene> {
    check_radius(radius)?;
    resident(world, centre, radius)?;
    let shape = world.descriptor().shape;
    let column = (i64::from(shape.size_x()), i64::from(shape.size_z()));
    let view = WorldSurfaces::new(world, table.clone());
    let mut regions = Vec::new();
    for at in columns(centre, radius) {
        let region = region_of(at, column)?;
        regions.push((region, mesh(&view, region)?));
    }
    Ok(Scene {
        regions,
        bounds: bounds_of(centre, radius, column)?,
        centre,
        radius,
        projection: Projection::perspective(70f64.to_radians(), 0.1, 512.0)?,
        table,
        atlas,
        column,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::residency::ClientResidency;
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

    const ORIGIN: ChunkCoord = ChunkCoord::new(0, 0);

    /// A scene with no content around the origin — every block its own
    /// surface, faces by direction — over the columns residency made
    /// resident for it.
    fn build_plain(world: &mut World, radius: i64) -> Result<(Scene, ClientResidency)> {
        let mut residency = ClientResidency::new(radius)?;
        residency.settle(world, ORIGIN)?;
        let table = SurfaceTable::untextured(world);
        Ok((build(world, ORIGIN, radius, table, None)?, residency))
    }

    #[test]
    fn radius_one_draws_nine_columns_with_a_resident_ring() {
        let mut world = world();
        let (scene, _) = build_plain(&mut world, 1).unwrap();
        assert_eq!(scene.regions.len(), 9);
        assert_eq!(world.chunk_count(), 25);
        assert_eq!(scene.bounds.size[0], 96);
        assert_eq!(scene.bounds.size[2], 96);
        assert!(scene.regions.iter().all(|(_, mesh)| !mesh.is_empty()));
    }

    /// The scene draws what residency holds, and never generates: a square
    /// whose ring is missing is refused by name.
    #[test]
    fn a_square_needs_its_columns_resident() {
        let mut world = world();
        let table = SurfaceTable::untextured(&world);
        let err = build(&world, ORIGIN, 1, table.clone(), None).expect_err("nothing resident");
        assert!(err.to_string().contains("not resident"), "{err}");
        let mut residency = ClientResidency::new(0).unwrap();
        residency.settle(&mut world, ORIGIN).unwrap();
        assert!(build(&world, ORIGIN, 0, table.clone(), None).is_ok());
        let err = build(&world, ORIGIN, 1, table, None).expect_err("the ring of radius 1 is not");
        assert!(err.to_string().contains("not resident"), "{err}");
    }

    /// Every region sits inside the bounds the reference walks, and they
    /// tile it without overlap; the band is the generator's.
    #[test]
    fn the_regions_tile_the_bounds() {
        let mut world = world();
        let (scene, _) = build_plain(&mut world, 1).unwrap();
        let cells: u64 = scene.regions.iter().map(|(region, _)| region.cells()).sum();
        assert_eq!(cells, scene.bounds.cells());
        let (low, high) = World::surface_range();
        assert_eq!(scene.bounds.origin.y, low - BAND_MARGIN);
        assert_eq!(
            scene.bounds.origin.y + i64::from(scene.bounds.size[1]),
            high + BAND_MARGIN
        );
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
        let (scene, _) = build_plain(&mut world, 1).unwrap();
        let bounds = scene.bounds;
        let (min_x, min_z) = (bounds.origin.x, bounds.origin.z);
        let (max_x, max_z) = (
            min_x + i64::from(bounds.size[0]),
            min_z + i64::from(bounds.size[2]),
        );
        let run = scene.spawn_run(&world).unwrap();
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

    /// The vertices a scene built from scratch gives each region, split at
    /// the corners of every drawn mesh.
    fn all_corners(scene: &Scene) -> Vec<Vec<u8>> {
        let meshes: Vec<&ChunkMesh> = scene.regions.iter().map(|(_, mesh)| mesh).collect();
        let corners = Corners::of(&meshes);
        scene
            .regions
            .iter()
            .map(|(region, mesh)| chunk_vertices(mesh, region.origin, &corners).unwrap())
            .collect()
    }

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

        let mut world = world();
        let (mut scene, _residency) = build_plain(&mut world, 2).unwrap();
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
            (3 * size + 8, 5), // the resident ring: not drawn
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
            let fresh = build(&world, ORIGIN, 2, scene.table.clone(), None).unwrap();
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

    /// Moving the square — one column, diagonally, several at once, and back
    /// — gives every region exactly what a scene built at the new centre
    /// from scratch gives it: the same meshes, and the same vertex bytes,
    /// where the regions [`Shift::reupload`] does not name keep the bytes
    /// they had. An edit made before walking away, in a column evicted on
    /// the way, is drawn again on the way back.
    #[test]
    fn a_moved_square_draws_what_a_fresh_build_would() {
        // Radius 1 walks one column, diagonally, far enough to evict the
        // edited column, and back; radius 2 diagonally and back. Each move is
        // checked against a fresh build, which in a debug build is seconds.
        let walks: [(i64, &[(i64, i64)]); 2] = [
            (1, &[(1, 0), (2, 1), (6, -1), (0, 0)]),
            (2, &[(1, 1), (0, 0)]),
        ];
        for (radius, walk) in walks {
            let mut world = world();
            let (mut scene, mut residency) = build_plain(&mut world, radius).unwrap();
            let mut drawn: Vec<Vec<u8>> = (0..scene.regions.len())
                .map(|index| scene.vertices(index).unwrap())
                .collect();
            let stone = world
                .block_id(
                    &nexora_foundation::ident::Identifier::parse("nexora:block/stone").unwrap(),
                )
                .unwrap();
            let size = i64::from(world.descriptor().shape.size_x());
            let (x, z) = (-radius * size + 3, 5);
            let cell = BlockPos::new(x, world.surface_height(x, z) + 2, z);
            world.set_block(cell, stone).unwrap();
            let remeshed = scene.remesh(&world, &[cell]).unwrap();
            for index in scene.reupload(&remeshed) {
                drawn[index] = scene.vertices(index).unwrap();
            }

            for &(cx, cz) in walk {
                let centre = ChunkCoord::new(cx, cz);
                residency.settle(&mut world, centre).unwrap();
                let shift = scene.recentre(&world, centre).unwrap();
                let before = std::mem::take(&mut drawn);
                for index in 0..scene.regions.len() {
                    drawn.push(if shift.reupload.contains(&index) {
                        scene.vertices(index).unwrap()
                    } else {
                        let kept = shift.from[index].expect("only a kept region keeps its bytes");
                        before[kept].clone()
                    });
                }
                let fresh = build(&world, centre, radius, scene.table.clone(), None).unwrap();
                assert_eq!(fresh.bounds, scene.bounds, "({cx},{cz})");
                assert!(
                    fresh.regions == scene.regions,
                    "meshes at ({cx},{cz}), radius {radius}"
                );
                // A region's own corners give what every corner gives (the
                // edit test above holds that), and cost a tenth as much.
                let expected: Vec<Vec<u8>> = (0..fresh.regions.len())
                    .map(|index| fresh.vertices(index).unwrap())
                    .collect();
                assert!(
                    drawn == expected,
                    "vertices at ({cx},{cz}), radius {radius}"
                );
                let side = usize::try_from(2 * radius + 1).unwrap();
                assert_eq!(shift.meshed, shift.dropped, "a square stays a square");
                assert!(shift.meshed <= side * side);
            }
            if radius == 1 {
                assert!(
                    residency.tally().persisted > 0,
                    "the edited column was evicted"
                );
            }
            assert_eq!(world.get_block(cell).unwrap(), stone, "the edit is drawn");
        }
    }

    /// One column over: one row enters, one leaves, and only the regions
    /// next to either are uploaded again.
    #[test]
    fn a_one_column_move_meshes_one_row() {
        let mut world = world();
        let (mut scene, mut residency) = build_plain(&mut world, 2).unwrap();
        let centre = ChunkCoord::new(1, 0);
        residency.settle(&mut world, centre).unwrap();
        let shift = scene.recentre(&world, centre).unwrap();
        assert_eq!((shift.meshed, shift.dropped), (5, 5));
        // Five columns wide: the row that entered, the row behind it, and
        // the row at the other edge that lost its neighbours. Two rows in
        // the middle keep their bytes.
        assert_eq!(shift.reupload.len(), 15);
        assert_eq!(shift.from.iter().filter(|from| from.is_some()).count(), 20);
    }

    #[test]
    fn the_edit_area_is_what_is_drawn() {
        let mut world = world();
        let (scene, _) = build_plain(&mut world, 1).unwrap();
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
        let world = world();
        let table = SurfaceTable::untextured(&world);
        for radius in [-1, MAX_RADIUS + 1] {
            let err = build(&world, ORIGIN, radius, table.clone(), None).unwrap_err();
            assert!(err.to_string().contains("radius"), "{err}");
        }
    }

    #[test]
    fn a_point_is_in_the_column_its_floor_is() {
        let mut world = world();
        let (scene, _) = build_plain(&mut world, 0).unwrap();
        let size = f64::from(world.descriptor().shape.size_x());
        assert_eq!(scene.column_of(0.0, 0.0), ORIGIN);
        assert_eq!(scene.column_of(-0.001, 0.0), ChunkCoord::new(-1, 0));
        assert_eq!(scene.column_of(size, size - 0.5), ChunkCoord::new(1, 0));
    }
}
