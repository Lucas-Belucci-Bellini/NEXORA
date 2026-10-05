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

use std::f64::consts::PI;

use nexora_camera::Projection;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{BlockPos, ChunkCoord};
use nexora_mesh::{mesh_region, ChunkMesh, Extent};
use nexora_simulation::{find_walkable_run, ColumnArea, Run, WorldSurfaces, WorldVoxels};
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
pub fn build(world: &mut World, radius: i64) -> Result<Scene> {
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

    let view = WorldSurfaces::untextured(world);
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

    #[test]
    fn radius_one_draws_nine_columns_generated_with_a_ring() {
        let mut world = world();
        let scene = build(&mut world, 1).unwrap();
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
        let scene = build(&mut world, 1).unwrap();
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
        let scene = build(&mut world, 1).unwrap();
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

    #[test]
    fn a_radius_out_of_range_is_refused() {
        let mut world = world();
        assert!(build(&mut world, -1).is_err());
        assert!(build(&mut world, 9).is_err());
    }
}
