//! Spatial queries (PHY-22).
//!
//! `PHYSICS.md` §23 lists the queries other systems need: what block is the
//! player looking at, is there an obstacle ahead of the AI, what is under the
//! vehicle. All of them are the same question — *where does this ray first meet
//! something solid* — and answering it by stepping the voxel grid cell by cell
//! is both exact and cheap, which is why it is not built on the box sweep.
//!
//! The traversal is the standard grid walk: advance along whichever axis
//! reaches its next cell boundary first. It visits every cell the ray passes
//! through and no others, so a ray cannot slip diagonally between two blocks
//! that share an edge.

use nexora_foundation::spatial::BlockPos;

use crate::material::MaterialId;
use crate::math::{Aabb, Axis, Vec3};
use crate::voxel::VoxelSource;

/// The most cells a single ray will visit.
///
/// Rays come from gameplay and from mods, and an unbounded range is an
/// unbounded loop. A caller that needs to see further should say so by
/// splitting the query, not by handing the solver a larger number.
pub const MAX_RAY_CELLS: u32 = 4_096;

/// Where a ray first met something solid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// The cell that was hit.
    pub cell: BlockPos,
    /// Distance from the ray origin, in metres.
    pub distance: f64,
    /// The face that was entered, pointing back along the ray. Zero when the
    /// ray began inside solid space, where no face was crossed.
    pub normal: Vec3,
    /// The surface that was hit.
    pub material: MaterialId,
    /// Whether the ray origin was already inside a solid cell.
    pub started_inside: bool,
}

impl RayHit {
    /// The point where the ray met the surface.
    #[must_use]
    pub fn point(&self, origin: Vec3, direction: Vec3) -> Vec3 {
        origin + direction.scaled(self.distance)
    }
}

/// Walk a ray through the voxel grid until it meets something solid.
///
/// `direction` need not be a unit vector; it is normalised, and `distance` is
/// reported in metres either way. Returns `None` when nothing was hit inside
/// `max_distance`, when the direction has no length, or when either input is
/// not finite.
#[must_use]
pub fn raycast<S: VoxelSource + ?Sized>(
    source: &S,
    origin: Vec3,
    direction: Vec3,
    max_distance: f64,
) -> Option<RayHit> {
    if !origin.is_finite() || !direction.is_finite() || !max_distance.is_finite() {
        return None;
    }
    if max_distance <= 0.0 {
        return None;
    }
    let length = direction.length();
    if length <= 0.0 {
        return None;
    }
    let direction = direction.scaled(1.0 / length);

    let mut cell = [
        origin.x.floor() as i64,
        origin.y.floor() as i64,
        origin.z.floor() as i64,
    ];

    if let Some(material) = source
        .shape_at(BlockPos::new(cell[0], cell[1], cell[2]))
        .material()
    {
        return Some(RayHit {
            cell: BlockPos::new(cell[0], cell[1], cell[2]),
            distance: 0.0,
            normal: Vec3::ZERO,
            material,
            started_inside: true,
        });
    }

    // Per axis: which way we step, how far to the next boundary, and how far
    // between boundaries.
    let mut step = [0i64; 3];
    let mut next_boundary = [f64::INFINITY; 3];
    let mut per_cell = [f64::INFINITY; 3];
    for axis in Axis::ALL {
        let index = axis.index();
        let component = direction.axis(axis);
        if component > 0.0 {
            step[index] = 1;
            next_boundary[index] = ((cell[index] + 1) as f64 - origin.axis(axis)) / component;
            per_cell[index] = 1.0 / component;
        } else if component < 0.0 {
            step[index] = -1;
            next_boundary[index] = (cell[index] as f64 - origin.axis(axis)) / component;
            per_cell[index] = -1.0 / component;
        }
    }

    for _ in 0..MAX_RAY_CELLS {
        // Cross whichever boundary comes first. Ties resolve to the lowest axis
        // index, so a ray along a cell edge behaves the same way every run.
        let mut axis = 0usize;
        for candidate in 1..3 {
            if next_boundary[candidate] < next_boundary[axis] {
                axis = candidate;
            }
        }
        let distance = next_boundary[axis];
        if !distance.is_finite() || distance > max_distance {
            return None;
        }

        cell[axis] += step[axis];
        next_boundary[axis] += per_cell[axis];

        let position = BlockPos::new(cell[0], cell[1], cell[2]);
        if let Some(material) = source.shape_at(position).material() {
            let normal = Vec3::along(Axis::ALL[axis], if step[axis] > 0 { -1.0 } else { 1.0 });
            return Some(RayHit {
                cell: position,
                distance,
                normal,
                material,
                started_inside: false,
            });
        }
    }

    None
}

/// Whether a box would be clear at this position.
///
/// The check other systems need before placing something: spawn points, block
/// placement and teleport destinations all have to know whether the volume is
/// free.
#[must_use]
pub fn is_clear<S: VoxelSource + ?Sized>(source: &S, aabb: Aabb) -> bool {
    !crate::collision::overlaps_solid(source, aabb)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::{EmptySpace, FlatGround, VoxelShape};
    use std::collections::BTreeSet;

    struct Cells {
        solid: BTreeSet<(i64, i64, i64)>,
    }

    impl Cells {
        fn from(cells: &[(i64, i64, i64)]) -> Self {
            Self {
                solid: cells.iter().copied().collect(),
            }
        }
    }

    impl VoxelSource for Cells {
        fn shape_at(&self, position: BlockPos) -> VoxelShape {
            if self.solid.contains(&(position.x, position.y, position.z)) {
                VoxelShape::SOLID
            } else {
                VoxelShape::Empty
            }
        }
    }

    #[test]
    fn a_ray_through_empty_space_hits_nothing() {
        assert!(raycast(&EmptySpace, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 1_000.0).is_none());
    }

    #[test]
    fn a_downward_ray_hits_the_floor_at_the_right_distance() {
        let ground = FlatGround::at(0);
        let hit = raycast(
            &ground,
            Vec3::new(0.5, 10.0, 0.5),
            Vec3::new(0.0, -1.0, 0.0),
            100.0,
        )
        .expect("the floor is there");
        assert!(
            (hit.distance - 10.0).abs() < 1e-12,
            "distance {}",
            hit.distance
        );
        assert_eq!(hit.cell.y, -1);
        assert_eq!(hit.normal, Vec3::new(0.0, 1.0, 0.0));
        assert!(!hit.started_inside);
    }

    #[test]
    fn the_hit_point_lands_on_the_surface() {
        let ground = FlatGround::at(0);
        let origin = Vec3::new(0.5, 10.0, 0.5);
        let direction = Vec3::new(0.0, -1.0, 0.0);
        let hit = raycast(&ground, origin, direction, 100.0).expect("hit");
        assert!(hit.point(origin, direction).y.abs() < 1e-12);
    }

    #[test]
    fn a_direction_need_not_be_normalised() {
        let ground = FlatGround::at(0);
        let long = raycast(
            &ground,
            Vec3::new(0.5, 10.0, 0.5),
            Vec3::new(0.0, -50.0, 0.0),
            100.0,
        )
        .expect("hit");
        let unit = raycast(
            &ground,
            Vec3::new(0.5, 10.0, 0.5),
            Vec3::new(0.0, -1.0, 0.0),
            100.0,
        )
        .expect("hit");
        assert!((long.distance - unit.distance).abs() < 1e-12);
    }

    #[test]
    fn a_ray_stops_at_its_maximum_distance() {
        let ground = FlatGround::at(0);
        assert!(raycast(
            &ground,
            Vec3::new(0.5, 10.0, 0.5),
            Vec3::new(0.0, -1.0, 0.0),
            5.0
        )
        .is_none());
        assert!(raycast(
            &ground,
            Vec3::new(0.5, 10.0, 0.5),
            Vec3::new(0.0, -1.0, 0.0),
            10.5
        )
        .is_some());
    }

    #[test]
    fn a_ray_starting_inside_solid_says_so() {
        let ground = FlatGround::at(0);
        let hit = raycast(
            &ground,
            Vec3::new(0.5, -5.0, 0.5),
            Vec3::new(0.0, 1.0, 0.0),
            100.0,
        )
        .expect("already inside");
        assert!(hit.started_inside);
        assert_eq!(hit.distance, 0.0);
        assert_eq!(hit.normal, Vec3::ZERO);
    }

    #[test]
    fn a_diagonal_ray_cannot_slip_between_two_blocks_sharing_an_edge() {
        // The classic grid-walk failure: a ray aimed exactly at the corner
        // between (1,1) and (2,2) must hit one of them, not pass through.
        let cells = Cells::from(&[(1, 1, 0), (2, 2, 0)]);
        let hit = raycast(
            &cells,
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(1.0, 1.0, 0.0),
            20.0,
        )
        .expect("must hit one of the pair");
        assert!(
            (hit.cell.x, hit.cell.y) == (1, 1) || (hit.cell.x, hit.cell.y) == (2, 2),
            "hit {:?}",
            hit.cell
        );
    }

    #[test]
    fn the_normal_names_the_face_that_was_entered() {
        let cells = Cells::from(&[(5, 0, 0)]);
        let hit = raycast(
            &cells,
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(1.0, 0.0, 0.0),
            20.0,
        )
        .expect("hit");
        assert_eq!(hit.normal, Vec3::new(-1.0, 0.0, 0.0));

        let cells = Cells::from(&[(-5, 0, 0)]);
        let hit = raycast(
            &cells,
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(-1.0, 0.0, 0.0),
            20.0,
        )
        .expect("hit");
        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn degenerate_rays_are_refused_rather_than_looping() {
        let ground = FlatGround::at(0);
        let origin = Vec3::new(0.5, 10.0, 0.5);
        assert!(raycast(&ground, origin, Vec3::ZERO, 100.0).is_none());
        assert!(raycast(&ground, origin, Vec3::new(0.0, -1.0, 0.0), 0.0).is_none());
        assert!(raycast(&ground, origin, Vec3::new(0.0, -1.0, 0.0), -5.0).is_none());
        assert!(raycast(&ground, origin, Vec3::new(f64::NAN, 0.0, 0.0), 10.0).is_none());
        assert!(raycast(
            &ground,
            Vec3::new(f64::NAN, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            10.0
        )
        .is_none());
    }

    #[test]
    fn a_ray_longer_than_the_cell_budget_gives_up_instead_of_hanging() {
        let far = f64::from(MAX_RAY_CELLS) * 10.0;
        assert!(raycast(&EmptySpace, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), far).is_none());
    }

    #[test]
    fn negative_coordinates_traverse_correctly() {
        // Truncation instead of floor would put the origin in the wrong cell
        // and make every hit on the negative side of the origin wrong.
        let cells = Cells::from(&[(-5, -5, -5)]);
        let hit = raycast(
            &cells,
            Vec3::new(-0.5, -4.5, -4.5),
            Vec3::new(-1.0, 0.0, 0.0),
            20.0,
        )
        .expect("hit");
        assert_eq!(hit.cell, BlockPos::new(-5, -5, -5));
        assert!(
            (hit.distance - 3.5).abs() < 1e-12,
            "distance {}",
            hit.distance
        );
    }

    #[test]
    fn clearance_matches_the_collision_convention() {
        let ground = FlatGround::at(0);
        let resting = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)).expect("valid");
        assert!(is_clear(&ground, resting));
        assert!(!is_clear(
            &ground,
            resting.translated(Vec3::new(0.0, -0.5, 0.0))
        ));
    }
}
