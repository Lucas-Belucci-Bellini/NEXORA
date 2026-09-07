//! Rigid bodies (PHY-2) and sleeping (PHY-34).
//!
//! A body is **mass, extent, velocity and material** — nothing else.
//! `PHYSICS.md`'s closing rule is explicit that physics must not know that a
//! body is a sword, a villager or an ore, and the way to guarantee that is for
//! the type to have nowhere to put such a fact.
//!
//! ## What is missing, on purpose
//!
//! There is no rotation, no angular velocity and no inertia tensor. Every
//! collider in this phase is an axis-aligned box against axis-aligned voxels,
//! where a rotation has nothing to act on. Adding the fields now would mean
//! carrying state that no code reads and no test can check — the startup brief
//! §43 calls that a mock, and §52 calls it overengineering. The register
//! records it as debt with the trigger that brings it back.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::WorldPosition;

use crate::material::MaterialId;
use crate::math::{Aabb, Vec3};

/// The largest half-extent a body may declare, in metres.
///
/// Collision cost is proportional to the box's cross-section in cells, so an
/// unbounded extent is an unbounded per-step cost. Structures larger than this
/// belong in the world, not in a single body.
pub const MAX_HALF_EXTENT: f64 = 64.0;

/// The smallest mass a dynamic body may have, in kilograms.
pub const MIN_MASS: f64 = 1.0e-6;

/// The largest mass a dynamic body may have, in kilograms.
pub const MAX_MASS: f64 = 1.0e12;

/// The default speed clamp, in metres per second.
pub const DEFAULT_MAX_SPEED: f64 = 200.0;

/// The largest speed clamp a body may declare, in metres per second.
///
/// Keeps one step's motion well inside [`crate::collision::MAX_SWEEP_LAYERS`],
/// so the solver never has to report a capped sweep during normal simulation.
pub const MAX_SPEED_LIMIT: f64 = 1_000.0;

/// Speed below which a dynamic body is a candidate for sleeping, in metres per
/// second.
pub const SLEEP_SPEED_THRESHOLD: f64 = 0.02;

/// Consecutive slow steps before a body sleeps.
pub const SLEEP_STEPS: u32 = 30;

/// How a body participates in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyType {
    /// Never moves and is never integrated. Terrain fixtures and fixed
    /// structures.
    Static,
    /// Moved by something outside physics; collides but is not pushed.
    /// Platforms, doors and machine parts.
    Kinematic,
    /// Integrated under gravity and forces.
    Dynamic,
}

impl BodyType {
    /// Stable lowercase name, safe to emit in diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Kinematic => "kinematic",
            Self::Dynamic => "dynamic",
        }
    }

    /// Whether the solver integrates this body under forces.
    #[must_use]
    pub const fn is_simulated(self) -> bool {
        matches!(self, Self::Dynamic)
    }

    /// Whether the body may move at all.
    #[must_use]
    pub const fn is_movable(self) -> bool {
        !matches!(self, Self::Static)
    }
}

/// Whether a body is being simulated or has settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SleepState {
    /// Simulated every step.
    Awake,
    /// Settled, and skipped until something wakes it.
    Sleeping,
}

impl SleepState {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Awake => "awake",
            Self::Sleeping => "sleeping",
        }
    }
}

/// A session handle to a body in a [`crate::world::PhysicsWorld`].
///
/// Generational, for the same reason `nexora_entity::EntityId` is: a slot that
/// gets reused must not let an old handle address the body that replaced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BodyId {
    index: u32,
    generation: u32,
}

impl BodyId {
    /// Build a handle. Normally produced by the world, not by callers.
    #[must_use]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Slot index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// Generation of the slot when the handle was issued.
    #[must_use]
    pub const fn generation(self) -> u32 {
        self.generation
    }
}

/// Everything needed to create a body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyDescriptor {
    /// How the body participates.
    pub body_type: BodyType,
    /// Centre of the collision box.
    pub center: Vec3,
    /// Half the size of the collision box on each axis.
    pub half_extents: Vec3,
    /// Mass in kilograms. Ignored for static and kinematic bodies.
    pub mass: f64,
    /// Multiplier on the world's gravity field.
    pub gravity_scale: f64,
    /// Fraction of velocity shed per second while moving.
    pub linear_damping: f64,
    /// The body's own surface, combined with whatever it touches.
    pub material: MaterialId,
    /// Speed clamp in metres per second.
    pub max_speed: f64,
    /// How high the body can climb without jumping, in metres. Zero means it
    /// cannot step up at all, which is right for anything that is not a
    /// walking character.
    pub step_height: f64,
}

impl BodyDescriptor {
    /// A one-cubic-metre dynamic body at the origin.
    #[must_use]
    pub fn dynamic() -> Self {
        Self {
            body_type: BodyType::Dynamic,
            center: Vec3::ZERO,
            half_extents: Vec3::splat(0.5),
            mass: 1.0,
            gravity_scale: 1.0,
            linear_damping: 0.0,
            material: MaterialId::DEFAULT,
            max_speed: DEFAULT_MAX_SPEED,
            step_height: 0.0,
        }
    }

    /// A humanoid character: 0.6 m across, 1.8 m tall, able to walk up a step
    /// one block high.
    ///
    /// The step height is exactly `1.0` and that is deliberate. Every shape in
    /// this phase is a whole cube, so the shortest feature that can exist is
    /// one metre tall; a step height of 0.6 - the value that is conventional
    /// where half-slabs exist - would be a setting that can never climb
    /// anything, which is a feature that does nothing. Sub-block values become
    /// meaningful when `PHYSICS.md` §13 (PHY-12) adds slabs and ramps.
    #[must_use]
    pub fn character() -> Self {
        Self {
            half_extents: Vec3::new(0.3, 0.9, 0.3),
            mass: 70.0,
            step_height: 1.0,
            ..Self::dynamic()
        }
    }

    /// A body that collides but is never integrated.
    #[must_use]
    pub fn kinematic() -> Self {
        Self {
            body_type: BodyType::Kinematic,
            ..Self::dynamic()
        }
    }

    /// Place the body.
    #[must_use]
    pub const fn at(mut self, center: Vec3) -> Self {
        self.center = center;
        self
    }

    /// Set the collision box half-extents.
    #[must_use]
    pub const fn sized(mut self, half_extents: Vec3) -> Self {
        self.half_extents = half_extents;
        self
    }

    /// Set the mass.
    #[must_use]
    pub const fn weighing(mut self, mass: f64) -> Self {
        self.mass = mass;
        self
    }

    /// Set the surface material.
    #[must_use]
    pub const fn made_of(mut self, material: MaterialId) -> Self {
        self.material = material;
        self
    }

    /// Validate every field.
    ///
    /// # Errors
    ///
    /// Returns an error when a value is not finite or lies outside the range
    /// its constant documents.
    pub fn validate(self) -> Result<Self> {
        self.center.require_finite("body")?;
        self.half_extents.require_finite("body")?;
        for axis in crate::math::Axis::ALL {
            let extent = self.half_extents.axis(axis);
            if !(0.0..=MAX_HALF_EXTENT).contains(&extent) {
                return Err(reject("half-extent outside its permitted range")
                    .with_context("axis", axis.as_str())
                    .with_context("value", extent.to_string())
                    .with_context("max", MAX_HALF_EXTENT.to_string()));
            }
        }
        if self.body_type.is_simulated() && !(MIN_MASS..=MAX_MASS).contains(&self.mass) {
            return Err(reject("dynamic body mass outside its permitted range")
                .with_context("mass", self.mass.to_string())
                .with_context("min", MIN_MASS.to_string())
                .with_context("max", MAX_MASS.to_string()));
        }
        if !self.gravity_scale.is_finite() {
            return Err(reject("gravity scale is not finite"));
        }
        if !(0.0..=1.0).contains(&self.linear_damping) {
            return Err(reject("linear damping must be a fraction")
                .with_context("value", self.linear_damping.to_string()));
        }
        if !(0.0..=MAX_SPEED_LIMIT).contains(&self.max_speed) {
            return Err(reject("speed clamp outside its permitted range")
                .with_context("value", self.max_speed.to_string())
                .with_context("max", MAX_SPEED_LIMIT.to_string()));
        }
        if !self.step_height.is_finite() || self.step_height < 0.0 {
            return Err(reject("step height must be zero or positive")
                .with_context("value", self.step_height.to_string()));
        }
        Ok(self)
    }
}

/// A body in the simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidBody {
    /// How the body participates.
    pub body_type: BodyType,
    /// Centre of the collision box.
    pub center: Vec3,
    /// Half the size of the collision box on each axis.
    pub half_extents: Vec3,
    /// Current velocity, in metres per second.
    pub velocity: Vec3,
    /// Mass in kilograms.
    pub mass: f64,
    /// Multiplier on the world gravity field.
    pub gravity_scale: f64,
    /// Fraction of velocity shed per second.
    pub linear_damping: f64,
    /// The body's surface.
    pub material: MaterialId,
    /// Speed clamp.
    pub max_speed: f64,
    /// Maximum climbable height without jumping.
    pub step_height: f64,
    /// Whether the body is currently simulated.
    pub sleep: SleepState,
    /// Whether the last step left the body resting on something.
    pub grounded: bool,
    /// The surface the body is resting on, when it is grounded.
    pub ground_material: Option<MaterialId>,
    /// Force accumulated for the next step, in newtons.
    pending_force: Vec3,
    /// Consecutive slow steps.
    still_steps: u32,
}

impl RigidBody {
    /// Build a body from a validated descriptor.
    ///
    /// # Errors
    ///
    /// Returns an error when the descriptor is invalid.
    pub fn from_descriptor(descriptor: BodyDescriptor) -> Result<Self> {
        let descriptor = descriptor.validate()?;
        Ok(Self {
            body_type: descriptor.body_type,
            center: descriptor.center,
            half_extents: descriptor.half_extents,
            velocity: Vec3::ZERO,
            mass: descriptor.mass,
            gravity_scale: descriptor.gravity_scale,
            linear_damping: descriptor.linear_damping,
            material: descriptor.material,
            max_speed: descriptor.max_speed,
            step_height: descriptor.step_height,
            sleep: SleepState::Awake,
            grounded: false,
            ground_material: None,
            pending_force: Vec3::ZERO,
            still_steps: 0,
        })
    }

    /// The collision box in world space.
    #[must_use]
    pub fn aabb(&self) -> Aabb {
        Aabb {
            min: self.center - self.half_extents,
            max: self.center + self.half_extents,
        }
    }

    /// The centre as a world position, for callers outside physics.
    #[must_use]
    pub const fn position(&self) -> WorldPosition {
        self.center.to_position()
    }

    /// Reciprocal mass, zero for bodies that forces cannot move.
    #[must_use]
    pub fn inverse_mass(&self) -> f64 {
        if self.body_type.is_simulated() && self.mass > 0.0 {
            1.0 / self.mass
        } else {
            0.0
        }
    }

    /// Whether the solver should integrate this body this step.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.body_type.is_simulated() && matches!(self.sleep, SleepState::Awake)
    }

    /// Add a force for the next step, in newtons (PHY-9).
    ///
    /// Wakes the body: a force applied to something asleep that stayed asleep
    /// would be silently discarded.
    ///
    /// # Errors
    ///
    /// Returns an error when the force is not finite.
    pub fn apply_force(&mut self, force: Vec3) -> Result<()> {
        self.pending_force += force.require_finite("body-force")?;
        self.wake();
        Ok(())
    }

    /// Change velocity immediately by an impulse, in newton-seconds (PHY-9).
    ///
    /// # Errors
    ///
    /// Returns an error when the impulse is not finite.
    pub fn apply_impulse(&mut self, impulse: Vec3) -> Result<()> {
        let impulse = impulse.require_finite("body-impulse")?;
        let inverse_mass = self.inverse_mass();
        if inverse_mass > 0.0 {
            self.velocity += impulse.scaled(inverse_mass);
            self.clamp_speed();
        }
        self.wake();
        Ok(())
    }

    /// Set velocity directly, for kinematic motion and for jumps.
    ///
    /// # Errors
    ///
    /// Returns an error when the velocity is not finite.
    pub fn set_velocity(&mut self, velocity: Vec3) -> Result<()> {
        self.velocity = velocity.require_finite("body-velocity")?;
        self.clamp_speed();
        self.wake();
        Ok(())
    }

    /// Move the body without integrating, for teleports and spawn placement.
    ///
    /// # Errors
    ///
    /// Returns an error when the centre is not finite.
    pub fn teleport(&mut self, center: Vec3) -> Result<()> {
        self.center = center.require_finite("body-teleport")?;
        self.grounded = false;
        self.ground_material = None;
        self.wake();
        Ok(())
    }

    /// Wake the body and restart its settling count.
    pub fn wake(&mut self) {
        self.sleep = SleepState::Awake;
        self.still_steps = 0;
    }

    /// Force the body to sleep immediately.
    pub fn sleep_now(&mut self) {
        self.sleep = SleepState::Sleeping;
        self.velocity = Vec3::ZERO;
        self.pending_force = Vec3::ZERO;
    }

    /// Consecutive steps the body has been below the sleep threshold.
    #[must_use]
    pub const fn still_steps(&self) -> u32 {
        self.still_steps
    }

    /// Forces waiting to be applied on the next step.
    #[must_use]
    pub const fn pending_force(&self) -> Vec3 {
        self.pending_force
    }

    /// Take the accumulated force, leaving none behind.
    pub(crate) fn take_force(&mut self) -> Vec3 {
        core::mem::replace(&mut self.pending_force, Vec3::ZERO)
    }

    /// Clamp velocity to the body's speed limit, preserving direction.
    pub(crate) fn clamp_speed(&mut self) {
        let speed = self.velocity.length();
        if speed > self.max_speed && speed > 0.0 {
            self.velocity = self.velocity.scaled(self.max_speed / speed);
        }
    }

    /// Advance the settling count and put the body to sleep once it has been
    /// slow for long enough.
    ///
    /// A body that is not resting on anything never sleeps: something falling
    /// slowly is not something that has settled.
    pub(crate) fn update_sleep(&mut self) {
        if !self.body_type.is_simulated() {
            return;
        }
        let slow = self.velocity.length() <= SLEEP_SPEED_THRESHOLD;
        if slow && self.grounded {
            self.still_steps = self.still_steps.saturating_add(1);
            if self.still_steps >= SLEEP_STEPS {
                self.sleep_now();
            }
        } else {
            self.still_steps = 0;
        }
    }
}

fn reject(message: &'static str) -> Error {
    Error::new(Domain::Physics, "body", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(descriptor: BodyDescriptor) -> RigidBody {
        RigidBody::from_descriptor(descriptor).expect("valid descriptor")
    }

    #[test]
    fn a_body_reports_the_box_its_extents_describe() {
        let body = body(
            BodyDescriptor::dynamic()
                .at(Vec3::new(10.0, 20.0, 30.0))
                .sized(Vec3::new(0.5, 1.0, 0.25)),
        );
        let aabb = body.aabb();
        assert_eq!(aabb.min, Vec3::new(9.5, 19.0, 29.75));
        assert_eq!(aabb.max, Vec3::new(10.5, 21.0, 30.25));
    }

    #[test]
    fn only_dynamic_bodies_respond_to_forces() {
        for body_type in [BodyType::Static, BodyType::Kinematic] {
            let mut b = body(BodyDescriptor {
                body_type,
                ..BodyDescriptor::dynamic()
            });
            b.apply_impulse(Vec3::new(100.0, 0.0, 0.0)).expect("finite");
            assert_eq!(b.velocity, Vec3::ZERO, "{}", body_type.as_str());
            assert_eq!(b.inverse_mass(), 0.0);
        }
        let mut dynamic = body(BodyDescriptor::dynamic().weighing(2.0));
        dynamic
            .apply_impulse(Vec3::new(10.0, 0.0, 0.0))
            .expect("finite");
        assert!((dynamic.velocity.x - 5.0).abs() < 1e-12);
    }

    #[test]
    fn an_impulse_cannot_exceed_the_speed_clamp() {
        let mut b = body(BodyDescriptor::dynamic().weighing(1.0));
        b.apply_impulse(Vec3::new(1.0e6, 0.0, 0.0)).expect("finite");
        assert!((b.velocity.length() - DEFAULT_MAX_SPEED).abs() < 1e-9);
        // And the direction survives the clamp.
        assert!(b.velocity.x > 0.0);
        assert!(b.velocity.y.abs() < 1e-12);
    }

    #[test]
    fn non_finite_inputs_are_refused_at_every_entry_point() {
        let mut b = body(BodyDescriptor::dynamic());
        let bad = Vec3::new(f64::NAN, 0.0, 0.0);
        assert!(b.apply_force(bad).is_err());
        assert!(b.apply_impulse(bad).is_err());
        assert!(b.set_velocity(bad).is_err());
        assert!(b.teleport(bad).is_err());
        // None of the refusals left the body in a poisoned state.
        assert!(b.center.is_finite());
        assert!(b.velocity.is_finite());
    }

    #[test]
    fn invalid_descriptors_are_refused() {
        assert!(BodyDescriptor::dynamic()
            .sized(Vec3::splat(MAX_HALF_EXTENT * 2.0))
            .validate()
            .is_err());
        assert!(BodyDescriptor::dynamic().weighing(0.0).validate().is_err());
        assert!(BodyDescriptor::dynamic()
            .weighing(MAX_MASS * 10.0)
            .validate()
            .is_err());
        assert!(BodyDescriptor {
            linear_damping: 2.0,
            ..BodyDescriptor::dynamic()
        }
        .validate()
        .is_err());
        assert!(BodyDescriptor {
            max_speed: MAX_SPEED_LIMIT * 2.0,
            ..BodyDescriptor::dynamic()
        }
        .validate()
        .is_err());
        assert!(BodyDescriptor {
            step_height: -1.0,
            ..BodyDescriptor::dynamic()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn a_massless_static_body_is_still_a_valid_descriptor() {
        // Mass is meaningless for a body forces cannot move, so it must not be
        // a reason to refuse one.
        assert!(BodyDescriptor {
            body_type: BodyType::Static,
            mass: 0.0,
            ..BodyDescriptor::dynamic()
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn a_grounded_body_sleeps_only_after_staying_still() {
        let mut b = body(BodyDescriptor::dynamic());
        b.grounded = true;
        for step in 0..SLEEP_STEPS - 1 {
            b.update_sleep();
            assert_eq!(b.sleep, SleepState::Awake, "slept at step {step}");
        }
        b.update_sleep();
        assert_eq!(b.sleep, SleepState::Sleeping);
        assert!(!b.is_active());
    }

    #[test]
    fn a_body_in_free_fall_never_sleeps_however_slowly_it_moves() {
        let mut b = body(BodyDescriptor::dynamic());
        b.grounded = false;
        for _ in 0..SLEEP_STEPS * 10 {
            b.update_sleep();
        }
        assert_eq!(b.sleep, SleepState::Awake);
    }

    #[test]
    fn any_force_wakes_a_sleeping_body() {
        let mut b = body(BodyDescriptor::dynamic());
        b.sleep_now();
        assert_eq!(b.sleep, SleepState::Sleeping);
        b.apply_force(Vec3::new(1.0, 0.0, 0.0)).expect("finite");
        assert_eq!(b.sleep, SleepState::Awake);
        assert_eq!(b.still_steps(), 0);
    }

    #[test]
    fn a_teleport_forgets_the_ground_it_was_standing_on() {
        let mut b = body(BodyDescriptor::dynamic());
        b.grounded = true;
        b.ground_material = Some(MaterialId(4));
        b.teleport(Vec3::new(0.0, 100.0, 0.0)).expect("finite");
        assert!(!b.grounded);
        assert_eq!(b.ground_material, None);
    }

    #[test]
    fn forces_accumulate_and_are_taken_once() {
        let mut b = body(BodyDescriptor::dynamic());
        b.apply_force(Vec3::new(1.0, 0.0, 0.0)).expect("finite");
        b.apply_force(Vec3::new(2.0, 0.0, 0.0)).expect("finite");
        assert!((b.pending_force().x - 3.0).abs() < 1e-12);
        assert!((b.take_force().x - 3.0).abs() < 1e-12);
        assert_eq!(b.pending_force(), Vec3::ZERO);
    }

    #[test]
    fn the_character_preset_is_a_humanoid_that_can_step_up() {
        let character = BodyDescriptor::character().validate().expect("valid");
        assert!((character.half_extents.y * 2.0 - 1.8).abs() < 1e-12);
        // One whole cube: see the preset's own documentation for why a smaller
        // value would be unusable against cube-only terrain.
        assert!((character.step_height - 1.0).abs() < 1e-12);
        // The default dynamic body cannot step: only things that walk should.
        assert_eq!(BodyDescriptor::dynamic().step_height, 0.0);
    }

    #[test]
    fn handles_carry_their_generation() {
        let id = BodyId::new(3, 7);
        assert_eq!(id.index(), 3);
        assert_eq!(id.generation(), 7);
        assert_ne!(id, BodyId::new(3, 8));
    }
}
