//! Swept collision of an axis-aligned box against voxels (PHY-4, PHY-5, PHY-10).
//!
//! ## Why one axis at a time
//!
//! A box moving diagonally into a corner has no single correct "time of impact":
//! stopping at the first contact would freeze it against a wall it is sliding
//! along. Resolving each axis separately, against the box as the previous axes
//! left it, is what produces sliding — and it is exact for boxes against cubes,
//! because every contact plane is axis-aligned.
//!
//! The order is fixed at [`Axis::RESOLUTION_ORDER`] — **vertical first**. That
//! is not arbitrary: landing before moving horizontally is what lets a body
//! walk along the surface it just landed on within the same step. A different
//! order is defensible; a *varying* order is not, because
//! `NEXORA REPLAY AND DETERMINISM.md` requires the same inputs to produce the
//! same state, and axis order changes the answer in a corner.
//!
//! ## Bounds
//!
//! Every sweep is capped at [`MAX_SWEEP_LAYERS`] cells. `NEXORA SECURITY THREAT
//! MODEL.md` treats mod and script input as untrusted, and a velocity is
//! exactly that kind of input: without the cap, one absurd number turns a step
//! into an unbounded loop. Hitting the cap reports a block rather than
//! pretending the move succeeded.

use nexora_foundation::spatial::BlockPos;

use crate::material::MaterialId;
use crate::math::{snap_to_plane, Aabb, Axis, Vec3};
use crate::voxel::VoxelSource;

/// The furthest a single axis sweep will examine, in cells.
///
/// A body crossing four thousand blocks in one step is already outside any
/// regime this solver models; the cap exists so that it fails predictably
/// instead of hanging the tick.
pub const MAX_SWEEP_LAYERS: i64 = 4_096;

/// The furthest a body will be pushed out of terrain it is already inside, in
/// metres.
///
/// A body deeper than this is not recoverable by a local push - it is buried,
/// and something above physics has to decide what that means. Reporting it
/// beats shoving it an unbounded distance to a place nobody asked for.
pub const MAX_DEPENETRATION: f64 = 4.0;

/// What stopped a sweep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Contact {
    /// The axis the motion was blocked on.
    pub axis: Axis,
    /// `true` when the body was moving in the positive direction on that axis.
    pub positive: bool,
    /// The cell that blocked it.
    pub cell: BlockPos,
    /// The surface that was hit.
    pub material: MaterialId,
}

impl Contact {
    /// The outward normal of the surface, pointing back at the body.
    #[must_use]
    pub fn normal(self) -> Vec3 {
        Vec3::along(self.axis, if self.positive { -1.0 } else { 1.0 })
    }
}

/// The result of moving a box along one axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisSweep {
    /// How far the box may actually move.
    pub allowed: f64,
    /// What stopped it, if anything did.
    pub contact: Option<Contact>,
}

impl AxisSweep {
    /// A sweep that completed with nothing in the way.
    #[must_use]
    pub const fn clear(distance: f64) -> Self {
        Self {
            allowed: distance,
            contact: None,
        }
    }

    /// Whether something stopped the sweep.
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        self.contact.is_some()
    }
}

/// The result of moving a box through a full motion vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolution {
    /// Where the box ended up.
    pub aabb: Aabb,
    /// The motion that was actually applied.
    pub applied: Vec3,
    /// At most one contact per axis, indexed by [`Axis::index`].
    pub contacts: [Option<Contact>; 3],
    /// The push that freed the box from terrain it began inside, if any.
    pub depenetration: Vec3,
    /// Whether the box began inside terrain and could not be freed.
    pub stuck: bool,
}

impl Resolution {
    /// The contact on one axis, if the motion was blocked there.
    #[must_use]
    pub const fn contact(&self, axis: Axis) -> Option<Contact> {
        self.contacts[axis.index()]
    }

    /// Whether motion on this axis was stopped.
    #[must_use]
    pub const fn is_blocked(&self, axis: Axis) -> bool {
        self.contacts[axis.index()].is_some()
    }

    /// Whether anything was hit at all.
    #[must_use]
    pub fn hit_anything(&self) -> bool {
        Axis::ALL.iter().any(|&axis| self.is_blocked(axis))
    }

    /// Whether the box had to be pushed out of terrain before it could move.
    #[must_use]
    pub fn was_depenetrated(&self) -> bool {
        self.depenetration != Vec3::ZERO
    }
}

/// Whether any cell the box overlaps is solid.
///
/// Uses the touching convention from [`Aabb`]: a box resting exactly on a floor
/// does not overlap it.
pub fn overlaps_solid<S: VoxelSource + ?Sized>(source: &S, aabb: Aabb) -> bool {
    solid_cell_in(source, aabb).is_some()
}

/// The first solid cell the box overlaps, scanned in ascending x, y, z order.
///
/// The scan order is part of the contract: it decides which material a contact
/// reports when a body straddles two different surfaces, and an unspecified
/// order would make that answer vary between runs.
pub fn solid_cell_in<S: VoxelSource + ?Sized>(
    source: &S,
    aabb: Aabb,
) -> Option<(BlockPos, MaterialId)> {
    let (x0, x1) = aabb.voxel_span(Axis::X);
    let (y0, y1) = aabb.voxel_span(Axis::Y);
    let (z0, z1) = aabb.voxel_span(Axis::Z);
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                let cell = BlockPos::new(x, y, z);
                if let Some(material) = source.shape_at(cell).material() {
                    return Some((cell, material));
                }
            }
        }
    }
    None
}

/// The extent of the solid cells a box overlaps, per axis.
///
/// Returns `None` when the box overlaps nothing solid.
fn solid_extent_in<S: VoxelSource + ?Sized>(
    source: &S,
    aabb: Aabb,
) -> Option<([i64; 3], [i64; 3])> {
    let (x0, x1) = aabb.voxel_span(Axis::X);
    let (y0, y1) = aabb.voxel_span(Axis::Y);
    let (z0, z1) = aabb.voxel_span(Axis::Z);
    let mut low = [i64::MAX; 3];
    let mut high = [i64::MIN; 3];
    let mut found = false;
    for x in x0..=x1 {
        for y in y0..=y1 {
            for z in z0..=z1 {
                if !source.shape_at(BlockPos::new(x, y, z)).is_solid() {
                    continue;
                }
                found = true;
                let cell = [x, y, z];
                for axis in 0..3 {
                    low[axis] = low[axis].min(cell[axis]);
                    high[axis] = high[axis].max(cell[axis] + 1);
                }
            }
        }
    }
    found.then_some((low, high))
}

/// The smallest push that frees a box from terrain it is inside.
///
/// A body can end up inside terrain without the solver ever having moved it
/// there: a block placed where it stands, a chunk generating around it, or a
/// save restoring a position the world no longer agrees with. The sweep alone
/// cannot recover from that state - it measures motion *into* new cells, and a
/// box already inside one has no new cell to enter - so the recovery is a
/// separate, explicit step.
///
/// Returns [`Vec3::ZERO`] when the box is already clear, and `None` when no
/// push within [`MAX_DEPENETRATION`] frees it.
#[must_use]
pub fn depenetrate<S: VoxelSource + ?Sized>(source: &S, aabb: Aabb) -> Option<Vec3> {
    let Some((low, high)) = solid_extent_in(source, aabb) else {
        return Some(Vec3::ZERO);
    };

    // Six ways out: to either side along each axis. Ordered by distance, then
    // by axis and direction, so the choice never depends on iteration luck.
    let mut candidates: [(f64, Axis, bool); 6] = [(0.0, Axis::X, true); 6];
    for (slot, axis) in Axis::ALL.iter().enumerate() {
        let index = axis.index();
        candidates[slot * 2] = (high[index] as f64 - aabb.min.axis(*axis), *axis, true);
        candidates[slot * 2 + 1] = (low[index] as f64 - aabb.max.axis(*axis), *axis, false);
    }
    candidates.sort_by(|a, b| {
        a.0.abs()
            .partial_cmp(&b.0.abs())
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| a.1.index().cmp(&b.1.index()))
            .then_with(|| a.2.cmp(&b.2))
    });

    for (distance, axis, _) in candidates {
        if !distance.is_finite() || distance.abs() > MAX_DEPENETRATION {
            continue;
        }
        let push = Vec3::along(axis, distance);
        if !overlaps_solid(source, aabb.translated(push)) {
            return Some(push);
        }
    }
    None
}

/// Move a box along one axis until something stops it.
///
/// Returns how far it may travel and what blocked it. A zero or non-finite
/// `delta` moves nothing and reports no contact.
pub fn sweep_axis<S: VoxelSource + ?Sized>(
    source: &S,
    aabb: Aabb,
    axis: Axis,
    delta: f64,
) -> AxisSweep {
    if !delta.is_finite() || delta == 0.0 {
        return AxisSweep::clear(0.0);
    }

    let positive = delta > 0.0;
    // The leading face, and where it wants to end up.
    let start = if positive {
        aabb.max.axis(axis)
    } else {
        aabb.min.axis(axis)
    };
    let end = start + delta;
    if !end.is_finite() {
        return AxisSweep::clear(0.0);
    }

    // The cells the box already occupies on the two axes it is not moving
    // along. They cannot change during a single-axis move.
    let [first_other, second_other] = axis.others();
    let (a0, a1) = aabb.voxel_span(first_other);
    let (b0, b1) = aabb.voxel_span(second_other);

    // Which layers the leading face newly enters. The face is snapped to the
    // grid first: a face a fraction of a unit in the last place past a plane
    // has not really entered the cell beyond it, and treating it as though it
    // had is what lets a body fall through the floor it is standing on. The
    // permitted distance below is still measured from the real face, so a body
    // that had drifted inside is nudged back out rather than teleported.
    let snapped = snap_to_plane(start);
    let (nearest, furthest) = if positive {
        (snapped.ceil() as i64, (end.ceil() as i64) - 1)
    } else {
        ((snapped.floor() as i64) - 1, end.floor() as i64)
    };

    let layers = if positive {
        furthest - nearest
    } else {
        nearest - furthest
    };
    // A move that enters no new layer cannot be obstructed.
    if layers < 0 {
        return AxisSweep::clear(delta);
    }
    let capped = layers.min(MAX_SWEEP_LAYERS - 1);
    let hit_cap = capped < layers;

    for offset in 0..=capped {
        let layer = if positive {
            nearest + offset
        } else {
            nearest - offset
        };
        for a in a0..=a1 {
            for b in b0..=b1 {
                let cell = cell_at(axis, layer, first_other, a, second_other, b);
                let Some(material) = source.shape_at(cell).material() else {
                    continue;
                };
                // Stop with the leading face flush against the blocking plane.
                let plane = if positive {
                    layer as f64
                } else {
                    (layer + 1) as f64
                };
                return AxisSweep {
                    allowed: plane - start,
                    contact: Some(Contact {
                        axis,
                        positive,
                        cell,
                        material,
                    }),
                };
            }
        }
    }

    if hit_cap {
        // The sweep ran out of budget before it ran out of distance. Reporting
        // the distance actually examined, blocked, is the honest answer: the
        // cells beyond it were never looked at, so claiming the move succeeded
        // would be a statement the solver cannot support.
        let examined = if positive {
            (nearest + capped) as f64
        } else {
            (nearest - capped + 1) as f64
        };
        return AxisSweep {
            allowed: examined - start,
            contact: Some(Contact {
                axis,
                positive,
                cell: cell_at(
                    axis,
                    if positive {
                        nearest + capped
                    } else {
                        nearest - capped
                    },
                    first_other,
                    a0,
                    second_other,
                    b0,
                ),
                material: MaterialId::DEFAULT,
            }),
        };
    }

    AxisSweep::clear(delta)
}

/// Move a box through a full motion vector, resolving one axis at a time.
///
/// A box that begins inside terrain is pushed out first; see [`depenetrate`].
pub fn resolve<S: VoxelSource + ?Sized>(source: &S, aabb: Aabb, motion: Vec3) -> Resolution {
    let freeing = depenetrate(source, aabb);
    let depenetration = freeing.unwrap_or(Vec3::ZERO);
    let mut current = aabb.translated(depenetration);
    let mut applied = Vec3::ZERO;
    let mut contacts: [Option<Contact>; 3] = [None; 3];

    for axis in Axis::RESOLUTION_ORDER {
        let sweep = sweep_axis(source, current, axis, motion.axis(axis));
        current = current.moved(axis, sweep.allowed);
        applied = applied.with_axis(axis, sweep.allowed);
        contacts[axis.index()] = sweep.contact;
    }

    Resolution {
        aabb: current,
        applied,
        contacts,
        depenetration,
        stuck: freeing.is_none(),
    }
}

fn cell_at(
    axis: Axis,
    along: i64,
    first_other: Axis,
    first: i64,
    second_other: Axis,
    second: i64,
) -> BlockPos {
    let mut coords = [0i64; 3];
    coords[axis.index()] = along;
    coords[first_other.index()] = first;
    coords[second_other.index()] = second;
    BlockPos::new(coords[0], coords[1], coords[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::{EmptySpace, FlatGround, VoxelShape};
    use std::collections::BTreeSet;

    /// A source built from an explicit set of solid cells.
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

    fn unit_box(x: f64, y: f64, z: f64) -> Aabb {
        Aabb::new(Vec3::new(x, y, z), Vec3::new(x + 1.0, y + 1.0, z + 1.0)).expect("valid")
    }

    #[test]
    fn a_clear_sweep_travels_the_whole_distance() {
        let sweep = sweep_axis(&EmptySpace, unit_box(0.0, 10.0, 0.0), Axis::Y, -5.0);
        assert!((sweep.allowed + 5.0).abs() < 1e-12);
        assert!(!sweep.is_blocked());
    }

    #[test]
    fn a_falling_box_lands_exactly_on_the_surface() {
        let ground = FlatGround::at(0);
        let sweep = sweep_axis(&ground, unit_box(0.0, 10.0, 0.0), Axis::Y, -20.0);
        // From y = 10 down to y = 0: ten metres, not twenty.
        assert!(
            (sweep.allowed + 10.0).abs() < 1e-12,
            "allowed {}",
            sweep.allowed
        );
        let contact = sweep.contact.expect("the floor blocks it");
        assert_eq!(contact.axis, Axis::Y);
        assert!(!contact.positive);
        assert_eq!(contact.cell.y, -1);
    }

    #[test]
    fn a_box_already_resting_on_the_floor_cannot_sink() {
        let ground = FlatGround::at(0);
        let sweep = sweep_axis(&ground, unit_box(0.0, 0.0, 0.0), Axis::Y, -1.0);
        assert!(sweep.allowed.abs() < 1e-12, "allowed {}", sweep.allowed);
        assert!(sweep.is_blocked());
    }

    #[test]
    fn resting_on_the_floor_is_not_overlapping_it() {
        // The touching convention has to hold here too, or a body would be
        // reported as inside the ground it is standing on.
        let ground = FlatGround::at(0);
        assert!(!overlaps_solid(&ground, unit_box(0.0, 0.0, 0.0)));
        assert!(overlaps_solid(&ground, unit_box(0.0, -0.001, 0.0)));
    }

    #[test]
    fn upward_motion_stops_under_a_ceiling() {
        let cells = Cells::from(&[(0, 5, 0)]);
        let sweep = sweep_axis(&cells, unit_box(0.0, 0.0, 0.0), Axis::Y, 10.0);
        // The box top starts at 1.0 and must stop at 5.0.
        assert!(
            (sweep.allowed - 4.0).abs() < 1e-12,
            "allowed {}",
            sweep.allowed
        );
        assert_eq!(sweep.contact.expect("ceiling").cell.y, 5);
    }

    #[test]
    fn a_wall_stops_horizontal_motion_at_its_face() {
        let cells = Cells::from(&[(3, 0, 0)]);
        let sweep = sweep_axis(&cells, unit_box(0.0, 0.0, 0.0), Axis::X, 10.0);
        assert!(
            (sweep.allowed - 2.0).abs() < 1e-12,
            "allowed {}",
            sweep.allowed
        );
        let contact = sweep.contact.expect("wall");
        assert!(contact.positive);
        assert_eq!(contact.normal(), Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn a_wall_stops_negative_motion_at_its_far_face() {
        let cells = Cells::from(&[(-3, 0, 0)]);
        let sweep = sweep_axis(&cells, unit_box(0.0, 0.0, 0.0), Axis::X, -10.0);
        // Box min starts at 0.0; the wall spans [-3, -2), so it stops at -2.0.
        assert!(
            (sweep.allowed + 2.0).abs() < 1e-12,
            "allowed {}",
            sweep.allowed
        );
        assert_eq!(
            sweep.contact.expect("wall").normal(),
            Vec3::new(1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn a_box_flush_against_a_wall_cannot_advance_but_can_retreat() {
        let cells = Cells::from(&[(1, 0, 0)]);
        let flush = unit_box(0.0, 0.0, 0.0);
        assert!(sweep_axis(&cells, flush, Axis::X, 1.0).allowed.abs() < 1e-12);
        assert!((sweep_axis(&cells, flush, Axis::X, -1.0).allowed + 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_gap_exactly_the_width_of_the_box_is_passable() {
        // Walls at x = -1 and x = 1, a one-block gap at x = 0.
        let cells = Cells::from(&[(-1, 0, 0), (1, 0, 0)]);
        let sweep = sweep_axis(&cells, unit_box(0.0, 0.0, 0.0), Axis::Z, 5.0);
        assert!(
            (sweep.allowed - 5.0).abs() < 1e-12,
            "allowed {}",
            sweep.allowed
        );
    }

    #[test]
    fn a_diagonal_move_into_a_wall_slides_along_it() {
        let cells = Cells::from(&[(3, 0, 0)]);
        let resolution = resolve(&cells, unit_box(0.0, 0.0, 0.0), Vec3::new(5.0, 0.0, 4.0));
        assert!(resolution.is_blocked(Axis::X));
        assert!(!resolution.is_blocked(Axis::Z));
        // Blocked on x, but the z component still went the whole way. That is
        // the entire reason axes are resolved separately.
        assert!((resolution.applied.x - 2.0).abs() < 1e-12);
        assert!((resolution.applied.z - 4.0).abs() < 1e-12);
    }

    #[test]
    fn landing_and_walking_happen_in_the_same_step() {
        // Vertical first: the box lands on the floor, and the horizontal move
        // is then resolved against the floor rather than through it.
        let ground = FlatGround::at(0);
        let resolution = resolve(&ground, unit_box(0.0, 4.0, 0.0), Vec3::new(2.0, -10.0, 0.0));
        assert!(
            (resolution.aabb.min.y).abs() < 1e-12,
            "y {}",
            resolution.aabb.min.y
        );
        assert!((resolution.applied.x - 2.0).abs() < 1e-12);
    }

    #[test]
    fn a_zero_motion_resolves_to_no_movement_and_no_contact() {
        let ground = FlatGround::at(0);
        let resolution = resolve(&ground, unit_box(0.0, 0.0, 0.0), Vec3::ZERO);
        assert_eq!(resolution.applied, Vec3::ZERO);
        assert!(!resolution.hit_anything());
    }

    #[test]
    fn a_non_finite_motion_moves_nothing_instead_of_producing_a_non_finite_box() {
        let ground = FlatGround::at(0);
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let resolution = resolve(&ground, unit_box(0.0, 5.0, 0.0), Vec3::new(bad, bad, bad));
            assert_eq!(resolution.applied, Vec3::ZERO, "value {bad}");
            assert!(resolution.aabb.min.is_finite());
        }
    }

    #[test]
    fn an_absurd_velocity_is_capped_rather_than_looping() {
        // Nothing is in the way for millions of cells; without the cap this
        // would examine every one of them.
        let resolution = resolve(
            &EmptySpace,
            unit_box(0.0, 0.0, 0.0),
            Vec3::new(1.0e9, 0.0, 0.0),
        );
        assert!(
            resolution.is_blocked(Axis::X),
            "the cap must report a block"
        );
        assert!(
            resolution.applied.x <= MAX_SWEEP_LAYERS as f64,
            "moved {} cells",
            resolution.applied.x
        );
        assert!(resolution.applied.x > 0.0);
    }

    #[test]
    fn a_sweep_shorter_than_one_cell_still_detects_the_adjacent_wall() {
        let cells = Cells::from(&[(1, 0, 0)]);
        let sweep = sweep_axis(&cells, unit_box(0.0, 0.0, 0.0), Axis::X, 0.01);
        assert!(sweep.is_blocked());
        assert!(sweep.allowed.abs() < 1e-12);
    }

    #[test]
    fn the_reported_material_comes_from_the_cell_that_blocked() {
        struct Striped;
        impl VoxelSource for Striped {
            fn shape_at(&self, position: BlockPos) -> VoxelShape {
                if position.y < 0 {
                    VoxelShape::Cube(MaterialId(if position.x >= 0 { 2 } else { 3 }))
                } else {
                    VoxelShape::Empty
                }
            }
        }
        let sweep = sweep_axis(&Striped, unit_box(5.0, 4.0, 0.0), Axis::Y, -10.0);
        assert_eq!(sweep.contact.expect("floor").material, MaterialId(2));
        let sweep = sweep_axis(&Striped, unit_box(-5.0, 4.0, 0.0), Axis::Y, -10.0);
        assert_eq!(sweep.contact.expect("floor").material, MaterialId(3));
    }

    #[test]
    fn resolution_is_identical_when_repeated() {
        // The same inputs must give the same answer; the solver holds no state
        // between calls and the scan order is fixed.
        let cells = Cells::from(&[(3, 0, 0), (0, -1, 0), (0, 0, 4)]);
        let start = unit_box(0.0, 0.0, 0.0);
        let motion = Vec3::new(5.0, -2.0, 6.0);
        let first = resolve(&cells, start, motion);
        for _ in 0..8 {
            assert_eq!(resolve(&cells, start, motion), first);
        }
    }

    #[test]
    fn a_body_resting_a_ulp_inside_the_floor_does_not_fall_through_it() {
        // The defect this guards against, exactly as it was found: a 1.8 m body
        // whose centre is 1.9 has its feet at 0.9999999999999999, which reads as
        // "already below the plane". The sweep then looked only at the cells
        // underneath and let it fall through the block it was standing on.
        let cells = Cells::from(&[(0, 0, 0)]);
        let feet = 1.9 - 0.9;
        assert_ne!(feet, 1.0, "the premise of this test is a rounding error");
        let standing =
            Aabb::new(Vec3::new(0.2, feet, 0.2), Vec3::new(0.8, feet + 1.8, 0.8)).expect("valid");

        let sweep = sweep_axis(&cells, standing, Axis::Y, -0.0027);
        assert!(sweep.is_blocked(), "it fell through the floor");
        assert!(sweep.allowed >= 0.0, "it sank by {}", sweep.allowed);

        // And over many steps it neither sinks nor drifts upwards.
        let mut current = standing;
        for _ in 0..600 {
            current = resolve(&cells, current, Vec3::new(0.0, -0.0027, 0.0)).aabb;
        }
        assert!(
            (current.min.y - 1.0).abs() < 1.0e-9,
            "settled at {} instead of on the surface",
            current.min.y
        );
    }

    #[test]
    fn a_box_that_starts_inside_terrain_is_pushed_out_the_nearest_way() {
        let cells = Cells::from(&[(0, 0, 0)]);
        // Overlapping the cell by 0.2 from above: the shortest way out is up.
        let sunk = Aabb::new(Vec3::new(0.2, 0.8, 0.2), Vec3::new(0.8, 1.8, 0.8)).expect("valid");
        assert!(overlaps_solid(&cells, sunk));
        let push = depenetrate(&cells, sunk).expect("freeable");
        assert!((push.y - 0.2).abs() < 1e-12, "pushed {push:?}");
        assert!(!overlaps_solid(&cells, sunk.translated(push)));
    }

    #[test]
    fn depenetration_prefers_the_shortest_axis() {
        let cells = Cells::from(&[(0, 0, 0)]);
        // Deep vertically, shallow horizontally: out sideways, not upwards.
        let wedged = Aabb::new(Vec3::new(0.9, 0.1, 0.2), Vec3::new(1.5, 0.9, 0.8)).expect("valid");
        let push = depenetrate(&cells, wedged).expect("freeable");
        assert!((push.x - 0.1).abs() < 1e-12, "pushed {push:?}");
        assert!(push.y.abs() < 1e-12);
    }

    #[test]
    fn a_box_already_clear_is_not_pushed_at_all() {
        let ground = FlatGround::at(0);
        let resting = unit_box(0.0, 0.0, 0.0);
        assert_eq!(depenetrate(&ground, resting), Some(Vec3::ZERO));
        assert!(!resolve(&ground, resting, Vec3::ZERO).was_depenetrated());
    }

    #[test]
    fn a_buried_box_reports_that_it_is_stuck_instead_of_being_flung() {
        struct Everywhere;
        impl VoxelSource for Everywhere {
            fn shape_at(&self, _position: BlockPos) -> VoxelShape {
                VoxelShape::SOLID
            }
        }
        let buried = unit_box(0.0, 0.0, 0.0);
        assert_eq!(depenetrate(&Everywhere, buried), None);
        let resolution = resolve(&Everywhere, buried, Vec3::new(1.0, 1.0, 1.0));
        assert!(resolution.stuck);
        assert_eq!(resolution.depenetration, Vec3::ZERO);
        assert!(
            resolution.aabb.min.is_finite(),
            "a stuck body must stay where it is, not be moved somewhere arbitrary"
        );
    }

    #[test]
    fn depenetration_is_the_same_answer_every_time() {
        // Six candidate directions with ties broken by axis and sign, so the
        // choice cannot depend on iteration order.
        let cells = Cells::from(&[(0, 0, 0)]);
        let centred = Aabb::from_center(Vec3::splat(0.5), Vec3::splat(0.5)).expect("valid");
        let first = depenetrate(&cells, centred);
        for _ in 0..16 {
            assert_eq!(depenetrate(&cells, centred), first);
        }
    }

    #[test]
    fn a_body_never_ends_a_resolution_inside_a_wall() {
        // The property that matters more than any single case: whatever the
        // motion, the resolved box does not overlap solid space.
        let cells = Cells::from(&[
            (1, 0, 0),
            (-1, 0, 0),
            (0, 0, 1),
            (0, 0, -1),
            (0, -1, 0),
            (0, 2, 0),
        ]);
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        for _ in 0..2_000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let component = |shift: u32| {
                let raw = ((seed >> shift) & 0xFFFF) as f64 / 65_535.0;
                raw.mul_add(8.0, -4.0)
            };
            let motion = Vec3::new(component(0), component(16), component(32));
            // Start from a scatter of positions, including ones that begin
            // inside the surrounding blocks: a body put there by a block
            // placement has to be recoverable, not permanently sunk.
            let start = Aabb::from_center(
                Vec3::new(
                    0.5 + component(48) * 0.25,
                    0.9 + component(8) * 0.25,
                    0.5 + component(24) * 0.25,
                ),
                Vec3::new(0.3, 0.9, 0.3),
            )
            .expect("valid");
            let resolved = resolve(&cells, start, motion);
            if resolved.stuck {
                continue;
            }
            assert!(
                !overlaps_solid(&cells, resolved.aabb),
                "motion {motion:?} from {start:?} ended inside a wall at {:?}",
                resolved.aabb
            );
        }
    }
}
