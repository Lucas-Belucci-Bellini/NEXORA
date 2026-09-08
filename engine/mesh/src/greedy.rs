//! Face culling and greedy merging (RENDER-9).
//!
//! # The two reductions, and why they are separate
//!
//! **Culling** decides which unit faces exist at all: a face exists where a
//! filled cell touches one that does not occlude it. A solid 32³ region has
//! 32³ × 6 = 196,608 cube faces and exactly 6,144 real ones — the interior is
//! not surface, and drawing it would be drawing the inside of a rock.
//!
//! **Merging** then describes those faces with as few rectangles as possible.
//! It changes the *count* of primitives and never the *area* of surface, which
//! is the property the tests pin: after merging, that same solid region is 6
//! rectangles rather than 6,144 quads, covering the same 6,144 unit faces.
//!
//! # The sweep
//!
//! For each axis, the mesher walks the planes *between* cells rather than the
//! cells themselves. At plane `i` along axis `d`, the cell below it and the
//! cell above it decide what happens:
//!
//! ```text
//! below filled, above does not occlude  ->  face on `below`, pointing +d
//! above filled, below does not occlude  ->  face on `above`, pointing -d
//! ```
//!
//! Walking planes instead of cells is what makes borders fall out for free.
//! The first and last plane of a region reach one cell outside it, the view
//! answers (CHUNK-28), and a seam between two solid chunks produces nothing —
//! no special case, no "am I at the edge" branch.
//!
//! # Determinism
//!
//! Axes in coordinate order, planes ascending, and within a plane the mask is
//! swept in row-major order. `NEXORA REPLAY AND DETERMINISM.md` wants the same
//! inputs to produce the same state, and a mesh assembled in whatever order a
//! hash map iterated is not reproducible — nor is it diffable between two runs,
//! which is how the tests below compare merged output against unmerged.

use nexora_foundation::spatial::{Axis, BlockPos};

use crate::mesh::{ChunkMesh, Facing, Quad, SurfaceId};
use crate::view::{Extent, VoxelView};

/// Build the surface of `extent` as seen through `view`.
///
/// Reads one cell beyond the region on every side, so faces on the boundary are
/// culled against their real neighbours rather than against nothing.
#[must_use]
pub fn mesh_region<V: VoxelView + ?Sized>(view: &V, extent: Extent) -> ChunkMesh {
    let mut mesh = ChunkMesh::new();
    for axis in Axis::ALL {
        sweep_axis(view, extent, axis, &mut mesh);
    }
    mesh
}

/// Sweep every plane perpendicular to `axis`.
fn sweep_axis<V: VoxelView + ?Sized>(view: &V, extent: Extent, axis: Axis, mesh: &mut ChunkMesh) {
    let [first, second] = axis.others();
    let depth = extent.size[axis.index()];
    let width = extent.size[first.index()] as usize;
    let height = extent.size[second.index()] as usize;

    // One mask per facing, reused across planes to avoid reallocating per slice.
    let mut positive: Vec<Option<SurfaceId>> = vec![None; width * height];
    let mut negative: Vec<Option<SurfaceId>> = vec![None; width * height];

    // Planes 0..=depth: `depth + 1` boundaries between and around `depth` cells.
    for plane in 0..=depth {
        positive.fill(None);
        negative.fill(None);

        for v in 0..height {
            for u in 0..width {
                let below = cell_at(extent, axis, plane as i64 - 1, first, u, second, v);
                let above = cell_at(extent, axis, plane as i64, first, u, second, v);

                // A face belongs to the cell that owns it, and only cells
                // inside the region are meshed here — the neighbour's own mesh
                // owns the faces on its side.
                if plane >= 1 {
                    if let Some(surface) = view.surface_at(below) {
                        if !view.occludes(above) {
                            positive[v * width + u] = Some(surface);
                        }
                    }
                }
                if plane < depth {
                    if let Some(surface) = view.surface_at(above) {
                        if !view.occludes(below) {
                            negative[v * width + u] = Some(surface);
                        }
                    }
                }
            }
        }

        merge_mask(&mut positive, width, height, |u, v, w, h, surface| {
            mesh.quads.push(Quad {
                origin: world_of(extent, axis, plane as i64 - 1, first, u, second, v),
                axis,
                facing: Facing::Positive,
                width: w,
                height: h,
                surface,
            });
        });
        merge_mask(&mut negative, width, height, |u, v, w, h, surface| {
            mesh.quads.push(Quad {
                origin: world_of(extent, axis, plane as i64, first, u, second, v),
                axis,
                facing: Facing::Negative,
                width: w,
                height: h,
                surface,
            });
        });
    }
}

/// World position of the cell at `(along, u, v)` relative to the region.
fn cell_at(
    extent: Extent,
    axis: Axis,
    along: i64,
    first: Axis,
    u: usize,
    second: Axis,
    v: usize,
) -> BlockPos {
    let [x, y, z] = world_of(extent, axis, along, first, u, second, v);
    BlockPos::new(x, y, z)
}

/// The same, as a raw triple.
fn world_of(
    extent: Extent,
    axis: Axis,
    along: i64,
    first: Axis,
    u: usize,
    second: Axis,
    v: usize,
) -> [i64; 3] {
    let mut position = [extent.origin.x, extent.origin.y, extent.origin.z];
    position[axis.index()] += along;
    position[first.index()] += u as i64;
    position[second.index()] += v as i64;
    position
}

/// Merge a mask of unit faces into maximal rectangles.
///
/// Grows each rectangle as far right as the surface matches, then as far down
/// as every cell of the next row matches, clearing what it consumes. Sweeping
/// row-major means the result depends only on the mask, never on iteration
/// order.
fn merge_mask<F>(mask: &mut [Option<SurfaceId>], width: usize, height: usize, mut emit: F)
where
    F: FnMut(usize, usize, u32, u32, SurfaceId),
{
    for v in 0..height {
        let mut u = 0usize;
        while u < width {
            let Some(surface) = mask[v * width + u] else {
                u += 1;
                continue;
            };

            // Widen.
            let mut run = 1usize;
            while u + run < width && mask[v * width + u + run] == Some(surface) {
                run += 1;
            }

            // Deepen, but only by whole rows: a partial row would leave a
            // ragged rectangle, and rectangles are what a renderer wants.
            let mut rows = 1usize;
            'grow: while v + rows < height {
                for offset in 0..run {
                    if mask[(v + rows) * width + u + offset] != Some(surface) {
                        break 'grow;
                    }
                }
                rows += 1;
            }

            for row in 0..rows {
                for offset in 0..run {
                    mask[(v + row) * width + u + offset] = None;
                }
            }

            emit(u, v, run as u32, rows as u32, surface);
            u += run;
        }
    }
}

/// Count unit faces without merging them.
///
/// The control the merged output is checked against: merging must produce the
/// same total area. Kept public because the benchmark uses it to report what
/// merging actually saved, rather than asserting a ratio nobody measured.
#[must_use]
pub fn unmerged_face_count<V: VoxelView + ?Sized>(view: &V, extent: Extent) -> u64 {
    let mut faces = 0u64;
    for axis in Axis::ALL {
        let [first, second] = axis.others();
        let depth = extent.size[axis.index()];
        for plane in 0..=depth {
            for v in 0..extent.size[second.index()] as usize {
                for u in 0..extent.size[first.index()] as usize {
                    let below = cell_at(extent, axis, plane as i64 - 1, first, u, second, v);
                    let above = cell_at(extent, axis, plane as i64, first, u, second, v);
                    if plane >= 1 && view.surface_at(below).is_some() && !view.occludes(above) {
                        faces += 1;
                    }
                    if plane < depth && view.surface_at(above).is_some() && !view.occludes(below) {
                        faces += 1;
                    }
                }
            }
        }
    }
    faces
}

#[cfg(test)]
mod tests {
    use super::*;

    const STONE: SurfaceId = SurfaceId(1);
    const DIRT: SurfaceId = SurfaceId(2);

    /// A solid axis-aligned box; everything outside it is empty.
    struct SolidBox {
        min: [i64; 3],
        max: [i64; 3],
        surface: SurfaceId,
    }

    impl VoxelView for SolidBox {
        fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
            let p = [position.x, position.y, position.z];
            (0..3)
                .all(|i| p[i] >= self.min[i] && p[i] <= self.max[i])
                .then_some(self.surface)
        }
    }

    fn solid(min: [i64; 3], max: [i64; 3]) -> SolidBox {
        SolidBox {
            min,
            max,
            surface: STONE,
        }
    }

    fn region(origin: [i64; 3], size: u32) -> Extent {
        Extent::cubic(BlockPos::new(origin[0], origin[1], origin[2]), size).expect("valid")
    }

    // --- culling ------------------------------------------------------------

    #[test]
    fn a_single_cube_has_six_faces() {
        let view = solid([0, 0, 0], [0, 0, 0]);
        let mesh = mesh_region(&view, region([0, 0, 0], 1));
        assert_eq!(mesh.len(), 6);
        assert_eq!(mesh.area(), 6);
    }

    #[test]
    fn empty_space_produces_nothing() {
        struct Empty;
        impl VoxelView for Empty {
            fn surface_at(&self, _: BlockPos) -> Option<SurfaceId> {
                None
            }
        }
        assert!(mesh_region(&Empty, region([0, 0, 0], 16)).is_empty());
    }

    #[test]
    fn a_solid_region_emits_only_its_shell_not_its_interior() {
        // The whole point of culling. A 16-cube has 16^3 * 6 = 24,576 cube
        // faces and 16*16*6 = 1,536 real ones; the rest is the inside of a
        // rock. Merging then describes those 1,536 with 6 rectangles.
        let size = 16u32;
        let view = solid([0, 0, 0], [15, 15, 15]);
        let extent = region([0, 0, 0], size);

        let unmerged = unmerged_face_count(&view, extent);
        assert_eq!(unmerged, u64::from(size * size * 6), "culling is wrong");

        let mesh = mesh_region(&view, extent);
        assert_eq!(mesh.len(), 6, "a cube's shell is six rectangles");
        assert_eq!(mesh.area(), unmerged, "merging changed the surface area");
    }

    #[test]
    fn merging_never_creates_or_destroys_surface() {
        // Checked across shapes that merge well and shapes that do not.
        let cases: [(&str, Box<dyn VoxelView>); 3] = [
            ("solid", Box::new(solid([0, 0, 0], [7, 7, 7]))),
            (
                "hollow",
                Box::new(Hollow {
                    min: [0, 0, 0],
                    max: [7, 7, 7],
                }),
            ),
            ("checker", Box::new(Checkerboard)),
        ];
        for (name, view) in cases {
            let extent = region([0, 0, 0], 8);
            let merged = mesh_region(view.as_ref(), extent);
            let unmerged = unmerged_face_count(view.as_ref(), extent);
            assert_eq!(
                merged.area(),
                unmerged,
                "{name}: area changed under merging"
            );
        }
    }

    /// A box with its interior hollowed out.
    struct Hollow {
        min: [i64; 3],
        max: [i64; 3],
    }

    impl VoxelView for Hollow {
        fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
            let p = [position.x, position.y, position.z];
            let inside = (0..3).all(|i| p[i] >= self.min[i] && p[i] <= self.max[i]);
            if !inside {
                return None;
            }
            let on_shell = (0..3).any(|i| p[i] == self.min[i] || p[i] == self.max[i]);
            on_shell.then_some(STONE)
        }
    }

    /// Alternating filled and empty cells: nothing can merge.
    struct Checkerboard;

    impl VoxelView for Checkerboard {
        fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
            ((position.x + position.y + position.z).rem_euclid(2) == 0).then_some(STONE)
        }
    }

    #[test]
    fn a_checkerboard_cannot_merge_and_is_not_merged() {
        // Guards the opposite failure from under-merging: a greedy pass that
        // merged across differing cells would report less area than exists.
        let extent = region([0, 0, 0], 8);
        let merged = mesh_region(&Checkerboard, extent);
        let unmerged = unmerged_face_count(&Checkerboard, extent);
        assert_eq!(merged.area(), unmerged);
        assert_eq!(
            merged.len() as u64,
            unmerged,
            "no two faces of a checkerboard are adjacent, so none may merge"
        );
    }

    #[test]
    fn different_surfaces_do_not_merge_into_one_rectangle() {
        struct TwoTone;
        impl VoxelView for TwoTone {
            fn surface_at(&self, position: BlockPos) -> Option<SurfaceId> {
                if position.y != 0 {
                    return None;
                }
                Some(if position.x < 4 { STONE } else { DIRT })
            }
        }

        let extent = Extent::new(BlockPos::new(0, 0, 0), [8, 1, 1]).expect("valid");
        let mesh = mesh_region(&TwoTone, extent);
        let top: Vec<&Quad> = mesh
            .quads
            .iter()
            .filter(|q| q.axis == Axis::Y && q.facing == Facing::Positive)
            .collect();
        assert_eq!(top.len(), 2, "one rectangle per surface, not one for both");
        assert_eq!(top.iter().map(|q| q.area()).sum::<u64>(), 8);
    }

    // --- borders (CHUNK-27, CHUNK-28) ---------------------------------------

    #[test]
    fn a_face_on_the_border_is_culled_by_the_neighbouring_chunk() {
        // The seam test. Two adjacent 8-cubes are one solid 8x8x16 slab; the
        // plane between them is interior and must produce nothing. A mesher
        // that only saw its own chunk would emit 64 faces there, and the world
        // would look like a grid of boxes.
        let view = solid([0, 0, 0], [7, 7, 15]);

        let near = mesh_region(&view, region([0, 0, 0], 8));
        let far = mesh_region(&view, region([0, 0, 8], 8));

        let seam_faces = near
            .quads
            .iter()
            .chain(far.quads.iter())
            .filter(|q| q.axis == Axis::Z && (q.origin[2] == 7 || q.origin[2] == 8))
            .filter(|q| {
                (q.origin[2] == 7 && q.facing == Facing::Positive)
                    || (q.origin[2] == 8 && q.facing == Facing::Negative)
            })
            .count();
        assert_eq!(
            seam_faces, 0,
            "the seam between two solid chunks is interior"
        );
    }

    #[test]
    fn the_two_halves_together_equal_meshing_the_whole_slab() {
        // Stronger than the seam test: no surface is lost or invented by
        // splitting the work in two.
        let view = solid([0, 0, 0], [7, 7, 15]);
        let whole = Extent::new(BlockPos::new(0, 0, 0), [8, 8, 16]).expect("valid");

        let together = mesh_region(&view, whole).area();
        let split = mesh_region(&view, region([0, 0, 0], 8)).area()
            + mesh_region(&view, region([0, 0, 8], 8)).area();
        assert_eq!(together, split);
    }

    #[test]
    fn a_region_whose_neighbour_is_empty_keeps_its_boundary_faces() {
        // The other direction: culling against nothing must not cull.
        let view = solid([0, 0, 0], [7, 7, 7]);
        let mesh = mesh_region(&view, region([0, 0, 0], 8));
        assert_eq!(mesh.area(), 8 * 8 * 6);
    }

    #[test]
    fn a_region_offset_from_the_origin_meshes_the_same_shape() {
        // Negative coordinates are where truncating arithmetic goes wrong.
        let here = solid([0, 0, 0], [7, 7, 7]);
        let there = solid([-100, -64, -100], [-93, -57, -93]);
        assert_eq!(
            mesh_region(&here, region([0, 0, 0], 8)).area(),
            mesh_region(&there, region([-100, -64, -100], 8)).area()
        );
        assert_eq!(
            mesh_region(&here, region([0, 0, 0], 8)).len(),
            mesh_region(&there, region([-100, -64, -100], 8)).len()
        );
    }

    // --- determinism --------------------------------------------------------

    #[test]
    fn the_same_voxels_always_produce_the_same_geometry() {
        let view = Hollow {
            min: [0, 0, 0],
            max: [11, 11, 11],
        };
        let extent = region([0, 0, 0], 12);
        let first = mesh_region(&view, extent);
        let second = mesh_region(&view, extent);
        assert_eq!(first, second, "meshing is not reproducible");
    }

    // --- quad geometry ------------------------------------------------------

    #[test]
    fn every_quad_has_a_real_extent_and_belongs_to_a_filled_cell() {
        let view = Hollow {
            min: [0, 0, 0],
            max: [5, 5, 5],
        };
        let extent = region([0, 0, 0], 6);
        let mesh = mesh_region(&view, extent);
        assert!(!mesh.is_empty());

        for quad in &mesh.quads {
            assert!(quad.width > 0 && quad.height > 0, "degenerate quad");
            let owner = BlockPos::new(quad.origin[0], quad.origin[1], quad.origin[2]);
            assert!(
                view.surface_at(owner).is_some(),
                "a face was attributed to an empty cell at {owner:?}"
            );
        }
    }

    #[test]
    fn vertex_count_reports_four_per_rectangle() {
        let view = solid([0, 0, 0], [15, 15, 15]);
        let mesh = mesh_region(&view, region([0, 0, 0], 16));
        assert_eq!(mesh.vertex_count(), mesh.len() as u64 * 4);
        assert_eq!(mesh.vertex_count(), 24, "six rectangles, four corners each");
    }
}
