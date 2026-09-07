//! Base components.
//!
//! Implements `Entity System.md` §14 (ENTITY-13 base components), §15-§17
//! (transform, position, rotation), §6 (ENTITY-5 lifecycle), §9 (ENTITY-8
//! persistence policy) and §21 (ENTITY-20 tags).
//!
//! Composition, not inheritance (§13): an entity does not inherit thirty
//! classes, it carries the components it needs.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{BlockPos, WorldPosition};

/// Where an entity is and which way it faces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Continuous world position.
    pub position: WorldPosition,
    /// Horizontal facing, in degrees.
    pub yaw: f32,
    /// Vertical facing, in degrees.
    pub pitch: f32,
}

impl Transform {
    /// A transform at a position, facing north.
    #[must_use]
    pub const fn at(position: WorldPosition) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    /// The block this entity currently occupies.
    #[must_use]
    pub fn block(self) -> BlockPos {
        self.position.to_block_pos()
    }

    /// Reject a transform that is not finite or points nowhere real.
    ///
    /// # Errors
    ///
    /// Returns an error when the position is non-finite or the angles are NaN.
    pub fn validate(self) -> Result<Self> {
        self.position.require_finite()?;
        if !self.yaw.is_finite() || !self.pitch.is_finite() {
            return Err(
                Error::new(Domain::World, "transform", "rotation is not finite")
                    .with_recovery(Recovery::Reject),
            );
        }
        Ok(self)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::at(WorldPosition::ORIGIN)
    }
}

/// How fast an entity is moving, in blocks per second.
///
/// Integrated by [`crate::store::EntityStore::step`] using the **world clock**,
/// never wall-clock time, so a simulation replays identically
/// (`NEXORA REPLAY AND DETERMINISM.md`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity {
    /// Blocks per second on the east-west axis.
    pub dx: f64,
    /// Blocks per second on the vertical axis.
    pub dy: f64,
    /// Blocks per second on the north-south axis.
    pub dz: f64,
}

impl Velocity {
    /// Not moving.
    pub const STILL: Self = Self {
        dx: 0.0,
        dy: 0.0,
        dz: 0.0,
    };

    /// Build a velocity.
    ///
    /// # Errors
    ///
    /// Returns an error when any component is non-finite. A NaN velocity
    /// silently teleports an entity to nowhere on the next step.
    pub fn new(dx: f64, dy: f64, dz: f64) -> Result<Self> {
        if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() {
            return Err(
                Error::new(Domain::World, "velocity", "velocity is not finite")
                    .with_recovery(Recovery::Reject),
            );
        }
        Ok(Self { dx, dy, dz })
    }

    /// Whether the entity is stationary.
    #[must_use]
    pub fn is_still(self) -> bool {
        self.dx == 0.0 && self.dy == 0.0 && self.dz == 0.0
    }

    /// The displacement over `seconds`.
    #[must_use]
    pub fn displacement(self, seconds: f64) -> (f64, f64, f64) {
        (self.dx * seconds, self.dy * seconds, self.dz * seconds)
    }
}

/// An axis-aligned box around an entity's origin.
///
/// This is a *boundary*, not physics: it answers "do these two occupy the same
/// space", which is what spawning, queries and interaction need. Collision
/// response belongs to `PHYSICS.md` and is not implemented.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// Half-width on the east-west axis.
    pub half_x: f64,
    /// Half-height on the vertical axis.
    pub half_y: f64,
    /// Half-depth on the north-south axis.
    pub half_z: f64,
}

impl Bounds {
    /// A box with no extent, for entities that occupy a point.
    pub const POINT: Self = Self {
        half_x: 0.0,
        half_y: 0.0,
        half_z: 0.0,
    };

    /// Build a box from half-extents.
    ///
    /// # Errors
    ///
    /// Returns an error when any half-extent is negative or non-finite.
    pub fn new(half_x: f64, half_y: f64, half_z: f64) -> Result<Self> {
        for (axis, value) in [("x", half_x), ("y", half_y), ("z", half_z)] {
            if !value.is_finite() || value < 0.0 {
                return Err(Error::new(
                    Domain::World,
                    "bounds",
                    "half-extent must be finite and non-negative",
                )
                .with_recovery(Recovery::Reject)
                .with_context("axis", axis)
                .with_context("value", value.to_string()));
            }
        }
        Ok(Self {
            half_x,
            half_y,
            half_z,
        })
    }

    /// Whether two placed boxes share any space.
    ///
    /// Touching faces do **not** count as overlapping: two entities standing
    /// exactly side by side are adjacent, not colliding.
    #[must_use]
    pub fn overlaps(self, at: WorldPosition, other: Self, other_at: WorldPosition) -> bool {
        (at.x - other_at.x).abs() < self.half_x + other.half_x
            && (at.y - other_at.y).abs() < self.half_y + other.half_y
            && (at.z - other_at.z).abs() < self.half_z + other.half_z
    }

    /// Whether a point is inside this box placed at `at`.
    #[must_use]
    pub fn contains(self, at: WorldPosition, point: WorldPosition) -> bool {
        (point.x - at.x).abs() <= self.half_x
            && (point.y - at.y).abs() <= self.half_y
            && (point.z - at.z).abs() <= self.half_z
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self::POINT
    }
}

/// Entity lifecycle states (`Entity System.md` §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lifecycle {
    /// Allocated but not yet placed in the world.
    Created,
    /// Being placed.
    Spawning,
    /// Present and simulated.
    Active,
    /// Present but not simulated - asleep, or outside the simulation distance.
    Inactive,
    /// Being taken out of the world.
    Despawning,
    /// Being deleted.
    Removing,
    /// Gone.
    Removed,
}

impl Lifecycle {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Spawning => "spawning",
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Despawning => "despawning",
            Self::Removing => "removing",
            Self::Removed => "removed",
        }
    }

    /// Whether the entity is present in the world at all.
    ///
    /// `Entity System.md` §8 is explicit that despawn does not necessarily mean
    /// permanent destruction, so "present" and "simulated" are different
    /// questions.
    #[must_use]
    pub const fn is_present(self) -> bool {
        matches!(self, Self::Active | Self::Inactive)
    }

    /// Whether the entity is being simulated right now.
    #[must_use]
    pub const fn is_simulated(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Whether `next` is a legal transition from this state.
    #[must_use]
    pub const fn can_transition_to(self, next: Self) -> bool {
        match self {
            Self::Created => matches!(next, Self::Spawning | Self::Removing),
            Self::Spawning => matches!(next, Self::Active | Self::Removing),
            Self::Active => matches!(next, Self::Inactive | Self::Despawning | Self::Removing),
            Self::Inactive => matches!(next, Self::Active | Self::Despawning | Self::Removing),
            Self::Despawning => matches!(next, Self::Inactive | Self::Removing),
            Self::Removing => matches!(next, Self::Removed),
            Self::Removed => false,
        }
    }
}

/// What happens to an entity when the world is saved (`Entity System.md` §9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistencePolicy {
    /// Always written to the save: players, vehicles, named creatures.
    Persistent,
    /// Never written: particles, short-lived projectiles.
    Temporary,
    /// Written as part of its region's state.
    Regional,
    /// May be collapsed into an aggregate population rather than stored one by one.
    Abstractable,
}

impl PersistencePolicy {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Persistent => "persistent",
            Self::Temporary => "temporary",
            Self::Regional => "regional",
            Self::Abstractable => "abstractable",
        }
    }

    /// Whether an entity with this policy is written to a save.
    #[must_use]
    pub const fn is_saved(self) -> bool {
        !matches!(self, Self::Temporary)
    }

    /// Encode for serialization.
    #[must_use]
    pub const fn to_tag(self) -> u8 {
        match self {
            Self::Persistent => 0,
            Self::Temporary => 1,
            Self::Regional => 2,
            Self::Abstractable => 3,
        }
    }

    /// Decode from serialization.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown tag rather than defaulting, because
    /// guessing here would silently change whether an entity survives a save.
    pub fn from_tag(tag: u8) -> Result<Self> {
        match tag {
            0 => Ok(Self::Persistent),
            1 => Ok(Self::Temporary),
            2 => Ok(Self::Regional),
            3 => Ok(Self::Abstractable),
            other => Err(Error::new(
                Domain::Save,
                "persistence-policy",
                "unknown persistence policy tag",
            )
            .with_recovery(Recovery::Quarantine)
            .with_context("tag", other.to_string())),
        }
    }
}

/// The tags an entity carries, e.g. `nexora:tag/living`.
///
/// Kept sorted and deduplicated so membership is a binary search and two
/// equivalent tag sets compare equal regardless of insertion order.
///
/// Stored as a `Vec` rather than an interned bitset. A bitset would query
/// faster, but inventing a fixed tag ceiling before anything has measured the
/// cost would be the kind of unmeasured optimization `DEBT-0005` already caught
/// once.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagSet {
    tags: Vec<Identifier>,
}

impl TagSet {
    /// An empty tag set.
    #[must_use]
    pub const fn new() -> Self {
        Self { tags: Vec::new() }
    }

    /// Build from a list, sorting and deduplicating.
    #[must_use]
    pub fn from_iter_sorted(tags: impl IntoIterator<Item = Identifier>) -> Self {
        let mut tags: Vec<Identifier> = tags.into_iter().collect();
        tags.sort();
        tags.dedup();
        Self { tags }
    }

    /// Add a tag. Returns whether it was newly added.
    pub fn insert(&mut self, tag: Identifier) -> bool {
        match self.tags.binary_search(&tag) {
            Ok(_) => false,
            Err(position) => {
                self.tags.insert(position, tag);
                true
            }
        }
    }

    /// Remove a tag. Returns whether it was present.
    pub fn remove(&mut self, tag: &Identifier) -> bool {
        match self.tags.binary_search(tag) {
            Ok(position) => {
                self.tags.remove(position);
                true
            }
            Err(_) => false,
        }
    }

    /// Whether the tag is present.
    #[must_use]
    pub fn contains(&self, tag: &Identifier) -> bool {
        self.tags.binary_search(tag).is_ok()
    }

    /// How many tags are carried.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    /// Whether no tags are carried.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// The tags, in sorted order.
    #[must_use]
    pub fn as_slice(&self) -> &[Identifier] {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("valid tag")
    }

    #[test]
    fn a_transform_reports_the_block_it_stands_in() {
        let transform = Transform::at(WorldPosition::new(-0.5, 64.9, 3.2));
        assert_eq!(transform.block(), BlockPos::new(-1, 64, 3));
    }

    #[test]
    fn invalid_transforms_are_rejected() {
        assert!(Transform::at(WorldPosition::new(f64::NAN, 0.0, 0.0))
            .validate()
            .is_err());
        let bad = Transform {
            yaw: f32::NAN,
            ..Transform::default()
        };
        assert!(bad.validate().is_err());
        assert!(Transform::default().validate().is_ok());
    }

    #[test]
    fn velocities_reject_non_finite_components() {
        assert!(Velocity::new(f64::NAN, 0.0, 0.0).is_err());
        assert!(Velocity::new(0.0, f64::INFINITY, 0.0).is_err());
        assert!(Velocity::new(1.0, -2.0, 0.5).is_ok());
        assert!(Velocity::STILL.is_still());
        assert!(!Velocity::new(0.0, 0.0, 0.1).unwrap().is_still());
    }

    #[test]
    fn displacement_scales_with_time() {
        let velocity = Velocity::new(2.0, -1.0, 0.5).unwrap();
        let (dx, dy, dz) = velocity.displacement(4.0);
        assert!((dx - 8.0).abs() < 1e-12);
        assert!((dy + 4.0).abs() < 1e-12);
        assert!((dz - 2.0).abs() < 1e-12);
    }

    #[test]
    fn bounds_reject_negative_or_non_finite_extents() {
        assert!(Bounds::new(-1.0, 1.0, 1.0).is_err());
        assert!(Bounds::new(1.0, f64::NAN, 1.0).is_err());
        assert!(Bounds::new(0.0, 0.0, 0.0).is_ok());
        assert!(Bounds::new(0.5, 1.0, 0.5).is_ok());
    }

    #[test]
    fn overlapping_boxes_are_detected_and_touching_ones_are_not() {
        let box_a = Bounds::new(0.5, 1.0, 0.5).unwrap();
        let at_a = WorldPosition::new(0.0, 0.0, 0.0);

        // Clearly inside.
        assert!(box_a.overlaps(at_a, box_a, WorldPosition::new(0.5, 0.0, 0.0)));
        // Exactly touching faces: adjacent, not colliding.
        assert!(!box_a.overlaps(at_a, box_a, WorldPosition::new(1.0, 0.0, 0.0)));
        // Clearly apart.
        assert!(!box_a.overlaps(at_a, box_a, WorldPosition::new(5.0, 0.0, 0.0)));
        // Separated on one axis only is still not an overlap.
        assert!(!box_a.overlaps(at_a, box_a, WorldPosition::new(0.2, 9.0, 0.2)));
    }

    #[test]
    fn bounds_contain_points_inclusively() {
        let bounds = Bounds::new(1.0, 1.0, 1.0).unwrap();
        let at = WorldPosition::new(10.0, 10.0, 10.0);
        assert!(bounds.contains(at, at));
        assert!(bounds.contains(at, WorldPosition::new(11.0, 10.0, 10.0)));
        assert!(!bounds.contains(at, WorldPosition::new(11.1, 10.0, 10.0)));
    }

    #[test]
    fn lifecycle_distinguishes_present_from_simulated() {
        assert!(Lifecycle::Active.is_present() && Lifecycle::Active.is_simulated());
        // Asleep: still in the world, not being ticked. Despawn is not death.
        assert!(Lifecycle::Inactive.is_present() && !Lifecycle::Inactive.is_simulated());
        assert!(!Lifecycle::Removed.is_present());
        assert!(!Lifecycle::Created.is_present());
    }

    #[test]
    fn lifecycle_transitions_follow_the_documented_flow() {
        let mut state = Lifecycle::Created;
        for next in [
            Lifecycle::Spawning,
            Lifecycle::Active,
            Lifecycle::Inactive,
            Lifecycle::Active,
            Lifecycle::Despawning,
            Lifecycle::Removing,
            Lifecycle::Removed,
        ] {
            assert!(
                state.can_transition_to(next),
                "{state:?} -> {next:?} was refused"
            );
            state = next;
        }
        // Removed is terminal.
        assert!(!Lifecycle::Removed.can_transition_to(Lifecycle::Active));
        // No resurrection by skipping.
        assert!(!Lifecycle::Created.can_transition_to(Lifecycle::Active));
    }

    #[test]
    fn only_temporary_entities_are_left_out_of_a_save() {
        assert!(PersistencePolicy::Persistent.is_saved());
        assert!(PersistencePolicy::Regional.is_saved());
        assert!(PersistencePolicy::Abstractable.is_saved());
        assert!(!PersistencePolicy::Temporary.is_saved());
    }

    #[test]
    fn policy_tags_round_trip_and_reject_the_unknown() {
        for policy in [
            PersistencePolicy::Persistent,
            PersistencePolicy::Temporary,
            PersistencePolicy::Regional,
            PersistencePolicy::Abstractable,
        ] {
            assert_eq!(
                PersistencePolicy::from_tag(policy.to_tag()).unwrap(),
                policy
            );
        }
        assert!(PersistencePolicy::from_tag(99).is_err());
    }

    #[test]
    fn tag_sets_stay_sorted_and_deduplicated() {
        let mut tags = TagSet::new();
        assert!(tags.is_empty());

        assert!(tags.insert(tag("nexora:tag/mob")));
        assert!(tags.insert(tag("nexora:tag/living")));
        assert!(
            !tags.insert(tag("nexora:tag/mob")),
            "a duplicate must not be added twice"
        );
        assert_eq!(tags.len(), 2);

        let rendered: Vec<String> = tags.as_slice().iter().map(ToString::to_string).collect();
        assert_eq!(rendered, ["nexora:tag/living", "nexora:tag/mob"]);

        assert!(tags.contains(&tag("nexora:tag/mob")));
        assert!(!tags.contains(&tag("nexora:tag/vehicle")));

        assert!(tags.remove(&tag("nexora:tag/mob")));
        assert!(!tags.remove(&tag("nexora:tag/mob")));
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn tag_sets_compare_equal_regardless_of_insertion_order() {
        let forward = TagSet::from_iter_sorted([
            tag("nexora:tag/a"),
            tag("nexora:tag/b"),
            tag("nexora:tag/a"),
        ]);
        let reverse = TagSet::from_iter_sorted([tag("nexora:tag/b"), tag("nexora:tag/a")]);
        assert_eq!(forward, reverse);
        assert_eq!(forward.len(), 2);
    }
}
