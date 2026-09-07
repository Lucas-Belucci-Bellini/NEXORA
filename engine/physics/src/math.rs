//! Vectors, axes and axis-aligned boxes.
//!
//! Physics needs a vector that is *not* a position: a force, a velocity and a
//! contact normal are not points in the world, and giving them the same type as
//! [`WorldPosition`] invites adding a position to a position. [`Vec3`] is that
//! type, and it converts explicitly at the boundary.
//!
//! Everything here is `f64`. `NEXORA REPLAY AND DETERMINISM.md` requires the
//! same inputs to produce the same state, and the accumulator in
//! [`crate::step`] is what makes that possible — but only if no value silently
//! loses precision on the way through. Nothing rounds to `f32`.

use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{BlockPos, WorldPosition};

/// How close a face must be to a voxel plane to count as lying on it.
///
/// Not a fudge factor. `1.9 - 0.9` is `0.9999999999999999` in binary floating
/// point, so a body standing exactly on a block is, as a bit pattern, a
/// fraction of a unit in the last place *inside* it. Without a tolerance the
/// solver reads that as "already past the surface", looks only at the cells
/// below, and lets the body fall through the floor it is standing on.
///
/// The tolerance is scaled by the magnitude of the coordinate, because
/// `NEXORA SPATIAL AND COORDINATE SYSTEM.md` allows positions out to 2^40
/// where a `f64` step is already larger than this constant.
pub const CONTACT_EPSILON: f64 = 1.0e-9;

/// Round a coordinate to a voxel plane when it is within [`CONTACT_EPSILON`].
///
/// Used to decide *which cells* a box occupies. Never used to decide how far a
/// body may move: the distance is always measured from the real coordinate, so
/// snapping corrects a position rather than teleporting it.
#[must_use]
pub fn snap_to_plane(value: f64) -> f64 {
    let nearest = value.round();
    let tolerance = CONTACT_EPSILON.max(value.abs() * f64::EPSILON * 8.0);
    if (value - nearest).abs() <= tolerance {
        nearest
    } else {
        value
    }
}

/// One of the three world axes.
///
/// Collision is resolved one axis at a time, so the axis is a value the solver
/// passes around rather than three copies of the same code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    /// East-west.
    X,
    /// Up-down.
    Y,
    /// North-south.
    Z,
}

impl Axis {
    /// The three axes in resolution order: vertical first.
    ///
    /// See [`crate::collision`] for why the order is fixed and why it is this
    /// one.
    pub const RESOLUTION_ORDER: [Self; 3] = [Self::Y, Self::X, Self::Z];

    /// All three axes in coordinate order.
    pub const ALL: [Self; 3] = [Self::X, Self::Y, Self::Z];

    /// Index into a three-component array.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }

    /// The other two axes, in coordinate order.
    #[must_use]
    pub const fn others(self) -> [Self; 2] {
        match self {
            Self::X => [Self::Y, Self::Z],
            Self::Y => [Self::X, Self::Z],
            Self::Z => [Self::X, Self::Y],
        }
    }

    /// Stable lowercase name, safe to emit in diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
        }
    }
}

/// A three-component vector: velocity, force, half-extent or normal.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    /// East-west component.
    pub x: f64,
    /// Up-down component.
    pub y: f64,
    /// North-south component.
    pub z: f64,
}

impl Vec3 {
    /// The zero vector.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Build a vector.
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Build a vector with the same value on every axis.
    #[must_use]
    pub const fn splat(value: f64) -> Self {
        Self::new(value, value, value)
    }

    /// Build a vector along one axis.
    #[must_use]
    pub const fn along(axis: Axis, value: f64) -> Self {
        match axis {
            Axis::X => Self::new(value, 0.0, 0.0),
            Axis::Y => Self::new(0.0, value, 0.0),
            Axis::Z => Self::new(0.0, 0.0, value),
        }
    }

    /// Read one component.
    #[must_use]
    pub const fn axis(self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    /// Replace one component.
    #[must_use]
    pub const fn with_axis(mut self, axis: Axis, value: f64) -> Self {
        match axis {
            Axis::X => self.x = value,
            Axis::Y => self.y = value,
            Axis::Z => self.z = value,
        }
        self
    }

    /// Scale every component.
    #[must_use]
    pub fn scaled(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    /// Dot product.
    #[must_use]
    pub fn dot(self, other: Self) -> f64 {
        self.x
            .mul_add(other.x, self.y.mul_add(other.y, self.z * other.z))
    }

    /// Squared length. Prefer this to [`Self::length`] when comparing.
    #[must_use]
    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    /// Length.
    #[must_use]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Whether every component is finite.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Reject a vector carrying a NaN or an infinity.
    ///
    /// A single non-finite component poisons every position it is integrated
    /// into, and it does so silently: the body simply disappears. Rejecting it
    /// at the boundary is the "fail loudly" rule applied to arithmetic.
    ///
    /// # Errors
    ///
    /// Returns an error when any component is not finite.
    pub fn require_finite(self, owner: &'static str) -> Result<Self> {
        if self.is_finite() {
            return Ok(self);
        }
        Err(
            Error::new(Domain::Physics, owner, "vector component is not finite")
                .with_recovery(Recovery::Reject)
                .with_context("x", self.x.to_string())
                .with_context("y", self.y.to_string())
                .with_context("z", self.z.to_string()),
        )
    }

    /// Convert to a world position, treating the vector as an offset from the
    /// world origin.
    #[must_use]
    pub const fn to_position(self) -> WorldPosition {
        WorldPosition::new(self.x, self.y, self.z)
    }

    /// Read a world position as a vector from the origin.
    #[must_use]
    pub const fn from_position(position: WorldPosition) -> Self {
        Self::new(position.x, position.y, position.z)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        self.scaled(rhs)
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// An axis-aligned bounding box in world space.
///
/// **Touching is not overlapping.** A box whose face sits exactly on a voxel
/// plane is outside that voxel, not inside it. This is the same convention
/// `nexora_entity::Bounds` uses, and the two disagreeing would mean an entity
/// query and a physics query could give opposite answers about the same pair.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    /// Lower corner.
    pub min: Vec3,
    /// Upper corner.
    pub max: Vec3,
}

impl Aabb {
    /// Build a box from its corners.
    ///
    /// # Errors
    ///
    /// Returns an error when a corner is not finite or `min` exceeds `max` on
    /// any axis.
    pub fn new(min: Vec3, max: Vec3) -> Result<Self> {
        min.require_finite("aabb")?;
        max.require_finite("aabb")?;
        for axis in Axis::ALL {
            if min.axis(axis) > max.axis(axis) {
                return Err(Error::new(
                    Domain::Physics,
                    "aabb",
                    "the lower corner is above the upper corner",
                )
                .with_recovery(Recovery::Reject)
                .with_context("axis", axis.as_str())
                .with_context("min", min.axis(axis).to_string())
                .with_context("max", max.axis(axis).to_string()));
            }
        }
        Ok(Self { min, max })
    }

    /// Build a box centred on a point.
    ///
    /// # Errors
    ///
    /// Returns an error when the centre is not finite or a half-extent is
    /// negative or not finite.
    pub fn from_center(center: Vec3, half_extents: Vec3) -> Result<Self> {
        center.require_finite("aabb")?;
        half_extents.require_finite("aabb")?;
        for axis in Axis::ALL {
            if half_extents.axis(axis) < 0.0 {
                return Err(Error::new(
                    Domain::Physics,
                    "aabb",
                    "a half-extent cannot be negative",
                )
                .with_recovery(Recovery::Reject)
                .with_context("axis", axis.as_str())
                .with_context("value", half_extents.axis(axis).to_string()));
            }
        }
        Ok(Self {
            min: center - half_extents,
            max: center + half_extents,
        })
    }

    /// The centre of the box.
    #[must_use]
    pub fn center(self) -> Vec3 {
        (self.min + self.max).scaled(0.5)
    }

    /// Half the size on each axis.
    #[must_use]
    pub fn half_extents(self) -> Vec3 {
        (self.max - self.min).scaled(0.5)
    }

    /// The box translated by an offset.
    #[must_use]
    pub fn translated(self, offset: Vec3) -> Self {
        Self {
            min: self.min + offset,
            max: self.max + offset,
        }
    }

    /// The box moved along one axis.
    #[must_use]
    pub fn moved(self, axis: Axis, distance: f64) -> Self {
        self.translated(Vec3::along(axis, distance))
    }

    /// The box grown by a margin on every axis.
    #[must_use]
    pub fn expanded(self, margin: f64) -> Self {
        Self {
            min: self.min - Vec3::splat(margin),
            max: self.max + Vec3::splat(margin),
        }
    }

    /// The box grown to also contain its own swept motion.
    #[must_use]
    pub fn swept(self, motion: Vec3) -> Self {
        let moved = self.translated(motion);
        Self {
            min: Vec3::new(
                self.min.x.min(moved.min.x),
                self.min.y.min(moved.min.y),
                self.min.z.min(moved.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(moved.max.x),
                self.max.y.max(moved.max.y),
                self.max.z.max(moved.max.z),
            ),
        }
    }

    /// Whether two boxes share volume. Touching faces do not.
    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        Axis::ALL.iter().all(|&axis| {
            self.min.axis(axis) < other.max.axis(axis) && self.max.axis(axis) > other.min.axis(axis)
        })
    }

    /// The inclusive range of voxel indices the box overlaps on one axis.
    ///
    /// Follows the touching convention: a box spanning exactly `[2.0, 3.0]`
    /// overlaps voxel 2 only, because voxel 1 ends where the box begins and
    /// voxel 3 begins where the box ends.
    #[must_use]
    pub fn voxel_span(self, axis: Axis) -> (i64, i64) {
        let low = snap_to_plane(self.min.axis(axis)).floor() as i64;
        let high = (snap_to_plane(self.max.axis(axis)).ceil() as i64) - 1;
        (low, high.max(low))
    }

    /// The lowest voxel corner the box touches.
    #[must_use]
    pub fn min_voxel(self) -> BlockPos {
        BlockPos::new(
            self.voxel_span(Axis::X).0,
            self.voxel_span(Axis::Y).0,
            self.voxel_span(Axis::Z).0,
        )
    }

    /// The highest voxel corner the box touches.
    #[must_use]
    pub fn max_voxel(self) -> BlockPos {
        BlockPos::new(
            self.voxel_span(Axis::X).1,
            self.voxel_span(Axis::Y).1,
            self.voxel_span(Axis::Z).1,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_index_and_complement_consistently() {
        assert_eq!(Axis::X.index(), 0);
        assert_eq!(Axis::Y.index(), 1);
        assert_eq!(Axis::Z.index(), 2);
        for axis in Axis::ALL {
            let others = axis.others();
            assert_ne!(others[0], axis);
            assert_ne!(others[1], axis);
            assert_ne!(others[0], others[1]);
        }
    }

    #[test]
    fn vertical_is_resolved_first() {
        assert_eq!(Axis::RESOLUTION_ORDER[0], Axis::Y);
    }

    #[test]
    fn vector_components_round_trip_through_axis_access() {
        let v = Vec3::new(1.5, -2.5, 3.25);
        for axis in Axis::ALL {
            assert_eq!(v.with_axis(axis, 9.0).axis(axis), 9.0);
        }
        assert_eq!(v.axis(Axis::X), 1.5);
        assert_eq!(v.axis(Axis::Y), -2.5);
        assert_eq!(v.axis(Axis::Z), 3.25);
    }

    #[test]
    fn non_finite_vectors_are_refused() {
        assert!(Vec3::new(f64::NAN, 0.0, 0.0)
            .require_finite("test")
            .is_err());
        assert!(Vec3::new(0.0, f64::INFINITY, 0.0)
            .require_finite("test")
            .is_err());
        assert!(Vec3::new(1.0, 2.0, 3.0).require_finite("test").is_ok());
    }

    #[test]
    fn a_box_touching_a_voxel_plane_does_not_overlap_the_next_voxel() {
        // Exactly [2.0, 3.0] on every axis: voxel 2 only.
        let aabb = Aabb::new(Vec3::splat(2.0), Vec3::splat(3.0)).expect("valid");
        for axis in Axis::ALL {
            assert_eq!(aabb.voxel_span(axis), (2, 2), "axis {}", axis.as_str());
        }
    }

    #[test]
    fn voxel_spans_cover_partial_cells_at_both_ends() {
        let aabb = Aabb::new(Vec3::new(2.3, -0.5, -3.0), Vec3::new(3.7, 0.5, -2.4)).expect("valid");
        assert_eq!(aabb.voxel_span(Axis::X), (2, 3));
        // -0.5 floors to -1, 0.5 ceils to 1, so cells -1 and 0.
        assert_eq!(aabb.voxel_span(Axis::Y), (-1, 0));
        // -3.0 floors to -3, -2.4 ceils to -2, so cell -3 only.
        assert_eq!(aabb.voxel_span(Axis::Z), (-3, -3));
    }

    #[test]
    fn a_face_a_fraction_of_a_ulp_inside_a_plane_is_treated_as_on_it() {
        // The exact arithmetic that produced the defect: a 1.8 m tall body
        // whose centre is at 1.9 has its feet at 0.9999999999999999.
        let feet = 1.9 - 0.9;
        assert_ne!(feet, 1.0, "the premise of this test is a rounding error");
        let aabb =
            Aabb::new(Vec3::new(0.2, feet, 0.2), Vec3::new(0.8, feet + 1.8, 0.8)).expect("valid");
        assert_eq!(
            aabb.voxel_span(Axis::Y).0,
            1,
            "a body standing on a block must not occupy the block"
        );
    }

    #[test]
    fn snapping_leaves_a_real_gap_alone() {
        // A millimetre is a real distance, not a rounding error.
        assert!((snap_to_plane(1.001) - 1.001).abs() < f64::EPSILON);
        assert!((snap_to_plane(0.999) - 0.999).abs() < f64::EPSILON);
        assert_eq!(snap_to_plane(1.0), 1.0);
        assert_eq!(snap_to_plane(-3.0), -3.0);
    }

    #[test]
    fn snapping_scales_with_the_coordinate() {
        // Far from the origin a f64 step is larger than CONTACT_EPSILON, so a
        // fixed tolerance would stop working exactly where the world is biggest.
        let far = 1.0e11_f64;
        let nudged = far + far * f64::EPSILON * 2.0;
        assert_ne!(nudged, far);
        assert_eq!(snap_to_plane(nudged), far);
    }

    #[test]
    fn a_degenerate_box_still_reports_one_cell() {
        // A zero-size box is legal (a point probe) and must not produce an
        // empty span, which would make every query answer "nothing here".
        let aabb = Aabb::new(Vec3::splat(4.0), Vec3::splat(4.0)).expect("valid");
        assert_eq!(aabb.voxel_span(Axis::X), (4, 4));
    }

    #[test]
    fn boxes_that_only_touch_do_not_overlap() {
        let a = Aabb::new(Vec3::ZERO, Vec3::splat(1.0)).expect("valid");
        let b = Aabb::new(Vec3::splat(1.0), Vec3::splat(2.0)).expect("valid");
        assert!(!a.overlaps(b));
        assert!(a.overlaps(b.translated(Vec3::splat(-0.5))));
    }

    #[test]
    fn inverted_corners_are_refused() {
        assert!(Aabb::new(Vec3::splat(1.0), Vec3::splat(0.0)).is_err());
        assert!(Aabb::from_center(Vec3::ZERO, Vec3::new(1.0, -1.0, 1.0)).is_err());
    }

    #[test]
    fn a_swept_box_contains_both_ends() {
        let start = Aabb::new(Vec3::ZERO, Vec3::splat(1.0)).expect("valid");
        let motion = Vec3::new(2.0, -3.0, 0.0);
        let swept = start.swept(motion);
        assert!(swept.min.x <= start.min.x && swept.max.x >= start.max.x + 2.0);
        assert!(swept.min.y <= start.min.y - 3.0 && swept.max.y >= start.max.y);
    }

    #[test]
    fn centre_and_half_extents_round_trip() {
        let center = Vec3::new(1.0, 2.0, 3.0);
        let half = Vec3::new(0.3, 0.9, 0.3);
        let aabb = Aabb::from_center(center, half).expect("valid");
        assert!((aabb.center() - center).length() < 1e-12);
        assert!((aabb.half_extents() - half).length() < 1e-12);
    }
}
