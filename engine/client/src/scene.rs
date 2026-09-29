//! The world the client shows: generated around the origin before the frame
//! loop starts, meshed once, one region per chunk column.
//!
//! Generation and meshing happen before the first frame, on purpose. Both
//! still run on the calling thread (DEBT-0018, DEBT-0027), and a frame loop
//! that generated while it ran would measure them rather than itself
//! (DEBT-0041). Streaming into the pass is not built.

use nexora_camera::{Camera, Projection};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{BlockPos, ChunkCoord, WorldPosition};
use nexora_mesh::{mesh_region, ChunkMesh, Extent};
use nexora_simulation::WorldSurfaces;
use nexora_world::world::World;

/// Blocks of margin above the highest surface and below the lowest in the
/// drawn band.
pub const BAND_MARGIN: i64 = 8;

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
    /// Where the camera starts.
    pub camera: Camera,
}

/// Generate columns `-radius..=radius` on both axes and one ring more, mesh
/// the inner ones in a vertical band around the surface, and place a camera
/// above one corner looking at the centre.
///
/// # Errors
///
/// `radius` is negative, generation failed, or a region is larger than the
/// mesher accepts.
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

    // Above the band, outside one corner, looking at the middle.
    let top = (bottom + i64::from(height)) as f64;
    let mut camera = Camera::new(
        WorldPosition::new(min_x as f64 - 6.5, top + 18.0, min_z as f64 - 5.5),
        Projection::perspective(70f64.to_radians(), 0.1, 512.0)?,
    )?;
    let centre = WorldPosition::new(
        (min_x + max_x) as f64 / 2.0,
        (low + high) as f64 / 2.0,
        (min_z + max_z) as f64 / 2.0,
    );
    camera.look_at(centre)?;
    Ok(Scene {
        regions,
        bounds,
        generated,
        camera,
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

    /// The camera starts above the band, so the first frame looks at the
    /// ground from outside what it draws.
    #[test]
    fn the_camera_starts_above_the_band_looking_down() {
        let mut world = world();
        let scene = build(&mut world, 1).unwrap();
        let top = scene.bounds.origin.y + i64::from(scene.bounds.size[1]);
        assert!(scene.camera.position().y > top as f64);
        assert!(scene.camera.pitch() < 0.0);
    }

    #[test]
    fn a_radius_out_of_range_is_refused() {
        let mut world = world();
        assert!(build(&mut world, -1).is_err());
        assert!(build(&mut world, 9).is_err());
    }
}
