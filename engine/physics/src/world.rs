//! The physics world (PHY-0) and the solver step (PHY-6).
//!
//! `PHYSICS.md` §1 keeps the physical state in its own container rather than
//! inside the world runtime, and forbids physics from reaching into gameplay.
//! [`PhysicsWorld`] owns bodies, materials and the timestep; terrain arrives
//! through a [`VoxelSource`] the caller supplies, so nothing here knows what a
//! block is called or what it is for.
//!
//! ## Determinism
//!
//! Bodies are stored in a slot vector and stepped in ascending slot order.
//! Nothing iterates a hash map, nothing reads a wall clock, and the timestep is
//! integer arithmetic over world ticks. The same world stepped with the same
//! elapsed time and the same voxel source produces the same state, which is
//! what `NEXORA REPLAY AND DETERMINISM.md` requires and what the headless
//! slice's byte-identical save check actually verifies.
//!
//! ## What this solver does not do yet
//!
//! **Bodies do not collide with each other.** Every contact in this phase is
//! between a body and the voxel grid. That is enough for the vertical slice
//! `PHYSICS.md` describes — falling, walking, jumping, landing — and it is not
//! enough for vehicles, stacked falling blocks or moving platforms carrying a
//! passenger. The gap is recorded in the technical debt register with the
//! trigger that closes it; it is stated here so that nobody reads a silent
//! pass-through as a bug in the sweep.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::time::WorldDuration;

use crate::body::{BodyDescriptor, BodyId, BodyType, RigidBody, SleepState};
use crate::collision::{resolve, sweep_axis, Contact, Resolution};
use crate::gravity::GravityField;
use crate::material::{MaterialId, MaterialTable, PhysicsMaterial};
use crate::math::{Aabb, Axis, Vec3};
use crate::step::{FixedStep, StepPlan};
use crate::voxel::VoxelSource;

/// The most bodies one physics world will hold.
///
/// Bounded because the body count is reachable from content and from mods, and
/// `NEXORA SECURITY THREAT MODEL.md` treats both as untrusted. Refusing the
/// spawn is recoverable; running out of memory is not.
pub const MAX_BODIES: usize = 1 << 20;

/// How far below a body the solver looks for ground, in metres.
pub const GROUND_PROBE: f64 = 1.0e-3;

/// Impact speed below which a contact stops the body instead of bouncing it,
/// in metres per second.
///
/// Without a floor like this, a bouncy body converges on an infinite series of
/// ever-smaller bounces and never sleeps.
pub const BOUNCE_THRESHOLD: f64 = 1.0;

/// What one substep did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StepStats {
    /// Dynamic bodies that were awake and integrated.
    pub simulated: u32,
    /// Kinematic bodies that were moved.
    pub kinematic: u32,
    /// Contacts resolved against the voxel grid.
    pub contacts: u32,
    /// Bodies that fell asleep during this substep.
    pub slept: u32,
    /// Bodies that had to be pushed out of terrain they were already inside.
    pub depenetrated: u32,
    /// Bodies buried too deep to be pushed out. Non-zero means something above
    /// physics has to intervene; the solver will not dig them out on its own.
    pub stuck: u32,
}

impl StepStats {
    fn merge(&mut self, other: Self) {
        self.simulated += other.simulated;
        self.kinematic += other.kinematic;
        self.contacts += other.contacts;
        self.slept += other.slept;
        self.depenetrated += other.depenetrated;
        self.stuck += other.stuck;
    }
}

/// What a call to [`PhysicsWorld::advance`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StepReport {
    /// Substeps actually run.
    pub substeps: u32,
    /// Substeps the timestep cap refused, meaning physics is behind world time.
    pub dropped_substeps: u32,
    /// Totals across every substep.
    pub stats: StepStats,
}

impl StepReport {
    /// Whether the simulation failed to keep up with world time.
    #[must_use]
    pub const fn fell_behind(&self) -> bool {
        self.dropped_substeps > 0
    }
}

#[derive(Debug, Clone)]
struct Slot {
    generation: u32,
    body: Option<RigidBody>,
}

/// Bodies, materials, gravity and the timestep that drives them.
#[derive(Debug, Clone)]
pub struct PhysicsWorld {
    gravity: GravityField,
    materials: MaterialTable,
    step: FixedStep,
    slots: Vec<Slot>,
    free: Vec<u32>,
    live: usize,
    retired_slots: usize,
}

impl PhysicsWorld {
    /// Build a world.
    #[must_use]
    pub fn new(gravity: GravityField, step: FixedStep) -> Self {
        Self {
            gravity,
            materials: MaterialTable::new(),
            step,
            slots: Vec::new(),
            free: Vec::new(),
            live: 0,
            retired_slots: 0,
        }
    }

    /// An earthlike world at the default physics rate.
    ///
    /// # Errors
    ///
    /// Returns an error when `ticks_per_second` is zero.
    pub fn earthlike(ticks_per_second: u32) -> Result<Self> {
        Ok(Self::new(
            GravityField::earthlike(),
            FixedStep::per_second(ticks_per_second)?,
        ))
    }

    /// The gravity field.
    #[must_use]
    pub const fn gravity(&self) -> GravityField {
        self.gravity
    }

    /// Replace the gravity field, waking every body so the change takes effect.
    ///
    /// Waking matters: a sleeping body under a field that just reversed would
    /// otherwise stay asleep on a ceiling.
    pub fn set_gravity(&mut self, gravity: GravityField) {
        self.gravity = gravity;
        for slot in &mut self.slots {
            if let Some(body) = slot.body.as_mut() {
                body.wake();
            }
        }
    }

    /// The material table.
    #[must_use]
    pub const fn materials(&self) -> &MaterialTable {
        &self.materials
    }

    /// The material table, for registration.
    pub fn materials_mut(&mut self) -> &mut MaterialTable {
        &mut self.materials
    }

    /// The timestep.
    #[must_use]
    pub const fn step(&self) -> &FixedStep {
        &self.step
    }

    /// How many bodies exist.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.live
    }

    /// Whether the world holds no bodies.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// Slots retired because their generation counter was exhausted.
    #[must_use]
    pub const fn retired_slots(&self) -> usize {
        self.retired_slots
    }

    /// How many bodies are awake.
    #[must_use]
    pub fn awake_count(&self) -> usize {
        self.iter()
            .filter(|(_, body)| matches!(body.sleep, SleepState::Awake))
            .count()
    }

    /// Create a body.
    ///
    /// # Errors
    ///
    /// Returns an error when the descriptor is invalid or the world is full.
    pub fn spawn(&mut self, descriptor: BodyDescriptor) -> Result<BodyId> {
        let body = RigidBody::from_descriptor(descriptor)?;
        if self.live >= MAX_BODIES {
            return Err(Error::new(
                Domain::Physics,
                "physics-world",
                "the physics world is full",
            )
            .with_recovery(Recovery::Reject)
            .with_context("capacity", MAX_BODIES.to_string()));
        }

        let index = if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.body = Some(body);
            index
        } else {
            let index = u32::try_from(self.slots.len()).map_err(|_| {
                Error::new(
                    Domain::Physics,
                    "physics-world",
                    "slot index space is exhausted",
                )
                .with_recovery(Recovery::Reject)
            })?;
            self.slots.push(Slot {
                generation: 0,
                body: Some(body),
            });
            index
        };
        self.live += 1;
        Ok(BodyId::new(index, self.slots[index as usize].generation))
    }

    /// Destroy a body. Returns whether the handle addressed a live body.
    pub fn despawn(&mut self, id: BodyId) -> bool {
        let Some(slot) = self.slots.get_mut(id.index() as usize) else {
            return false;
        };
        if slot.generation != id.generation() || slot.body.is_none() {
            return false;
        }
        slot.body = None;
        self.live -= 1;

        // Advancing the generation is what makes every outstanding handle
        // stale. When the counter is exhausted the slot is retired rather than
        // reused, so a very old handle can never alias a new body.
        match slot.generation.checked_add(1) {
            Some(next) => {
                slot.generation = next;
                self.free.push(id.index());
            }
            None => self.retired_slots += 1,
        }
        true
    }

    /// Look a body up.
    #[must_use]
    pub fn body(&self, id: BodyId) -> Option<&RigidBody> {
        let slot = self.slots.get(id.index() as usize)?;
        if slot.generation != id.generation() {
            return None;
        }
        slot.body.as_ref()
    }

    /// Look a body up for modification.
    pub fn body_mut(&mut self, id: BodyId) -> Option<&mut RigidBody> {
        let slot = self.slots.get_mut(id.index() as usize)?;
        if slot.generation != id.generation() {
            return None;
        }
        slot.body.as_mut()
    }

    /// Look a body up, refusing a stale handle.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not address a live body.
    pub fn require_body(&self, id: BodyId) -> Result<&RigidBody> {
        self.body(id).ok_or_else(|| stale_handle(id))
    }

    /// Look a body up for modification, refusing a stale handle.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not address a live body.
    pub fn require_body_mut(&mut self, id: BodyId) -> Result<&mut RigidBody> {
        if self.body(id).is_none() {
            return Err(stale_handle(id));
        }
        self.body_mut(id).ok_or_else(|| stale_handle(id))
    }

    /// Every live body with its handle, in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (BodyId, &RigidBody)> + '_ {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            let body = slot.body.as_ref()?;
            let id = BodyId::new(u32::try_from(index).unwrap_or(u32::MAX), slot.generation);
            Some((id, body))
        })
    }

    /// Wake every sleeping body overlapping a region.
    ///
    /// The world calls this when blocks change: a body asleep on a platform
    /// that was just mined has to fall, and nothing else would tell it to.
    /// Returns how many bodies were woken.
    pub fn wake_in(&mut self, region: Aabb) -> usize {
        let mut woken = 0;
        for slot in &mut self.slots {
            let Some(body) = slot.body.as_mut() else {
                continue;
            };
            if matches!(body.sleep, SleepState::Sleeping) && body.aabb().overlaps(region) {
                body.wake();
                woken += 1;
            }
        }
        woken
    }

    /// Fold elapsed world time in and run the substeps it earns.
    pub fn advance<S: VoxelSource + ?Sized>(
        &mut self,
        source: &S,
        elapsed: WorldDuration,
    ) -> StepReport {
        let StepPlan { substeps, dropped } = self.step.accumulate(elapsed);
        let seconds = self.step.seconds_per_step();
        let mut stats = StepStats::default();
        for _ in 0..substeps {
            stats.merge(self.step_once(source, seconds));
        }
        StepReport {
            substeps,
            dropped_substeps: dropped,
            stats,
        }
    }

    /// Run exactly one substep of `seconds`.
    ///
    /// Exposed for tests and for callers that drive their own accumulator. A
    /// non-finite or non-positive duration does nothing.
    pub fn step_once<S: VoxelSource + ?Sized>(&mut self, source: &S, seconds: f64) -> StepStats {
        let mut stats = StepStats::default();
        if !seconds.is_finite() || seconds <= 0.0 {
            return stats;
        }
        let gravity = self.gravity.acceleration();
        let gravity_magnitude = self.gravity.magnitude();
        // Cloned so the per-body loop can borrow the world mutably. The table
        // is small and this is once per substep, not once per body.
        let materials = self.materials.clone();

        for index in 0..self.slots.len() {
            let Some(body) = self.slots[index].body.as_mut() else {
                continue;
            };
            match body.body_type {
                BodyType::Static => {}
                BodyType::Kinematic => {
                    if body.velocity != Vec3::ZERO {
                        // Kinematic bodies are driven from outside physics and
                        // are not stopped by terrain: whatever moves them owns
                        // the decision about where they may go.
                        body.center += body.velocity.scaled(seconds);
                        stats.kinematic += 1;
                    }
                }
                BodyType::Dynamic => {
                    if !body.is_active() {
                        continue;
                    }
                    let outcome = integrate_dynamic(
                        body,
                        source,
                        &materials,
                        gravity,
                        gravity_magnitude,
                        seconds,
                    );
                    stats.contacts += outcome.contacts;
                    stats.depenetrated += u32::from(outcome.depenetrated);
                    stats.stuck += u32::from(outcome.stuck);
                    stats.simulated += 1;
                    let was_awake = matches!(body.sleep, SleepState::Awake);
                    body.update_sleep();
                    if was_awake && matches!(body.sleep, SleepState::Sleeping) {
                        stats.slept += 1;
                    }
                }
            }
        }
        stats
    }
}

fn stale_handle(id: BodyId) -> Error {
    Error::new(
        Domain::Physics,
        "physics-world",
        "body handle does not address a live body",
    )
    .with_recovery(Recovery::Reject)
    .with_context("index", id.index().to_string())
    .with_context("generation", id.generation().to_string())
}

/// What integrating one body produced.
struct BodyOutcome {
    contacts: u32,
    depenetrated: bool,
    stuck: bool,
}

/// Integrate one dynamic body and resolve it against the voxels.
fn integrate_dynamic<S: VoxelSource + ?Sized>(
    body: &mut RigidBody,
    source: &S,
    materials: &MaterialTable,
    gravity: Vec3,
    gravity_magnitude: f64,
    seconds: f64,
) -> BodyOutcome {
    let force = body.take_force();
    let acceleration = gravity.scaled(body.gravity_scale) + force.scaled(body.inverse_mass());
    body.velocity += acceleration.scaled(seconds);

    if body.linear_damping > 0.0 {
        let retained = 1.0 - body.linear_damping * seconds;
        body.velocity = body.velocity.scaled(retained.clamp(0.0, 1.0));
    }
    body.clamp_speed();

    let motion = body.velocity.scaled(seconds);
    let resolution = resolve_with_step_up(source, body.aabb(), motion, body.step_height);
    body.center = resolution.aabb.center();

    let own = materials.get(body.material);
    let mut contacts = 0;
    let mut grounded = false;
    let mut ground_material = None;

    // A push out of terrain is a contact in every sense that matters: velocity
    // into the surface the body was inside has to stop, or the next step drives
    // it straight back in and the two fight forever.
    for axis in Axis::ALL {
        let push = resolution.depenetration.axis(axis);
        if push != 0.0 && body.velocity.axis(axis) * push < 0.0 {
            body.velocity = body.velocity.with_axis(axis, 0.0);
        }
        if push > 0.0 && axis == Axis::Y {
            grounded = true;
        }
    }

    for axis in Axis::ALL {
        let Some(contact) = resolution.contact(axis) else {
            continue;
        };
        contacts += 1;
        let surface = materials.get(contact.material);
        apply_contact_velocity(body, axis, own, surface);
        if axis == Axis::Y && !contact.positive {
            grounded = true;
            ground_material = Some(contact.material);
        }
    }

    // A body that is neither falling nor already resting still has to know
    // whether there is floor under it — otherwise nothing standing still would
    // ever report as grounded.
    if !grounded && resolution.applied.y >= 0.0 {
        let probe = sweep_axis(source, body.aabb(), Axis::Y, -GROUND_PROBE);
        if let Some(contact) = probe.contact {
            grounded = true;
            ground_material = Some(contact.material);
        }
    }

    body.grounded = grounded;
    body.ground_material = ground_material;

    if grounded {
        apply_ground_friction(
            body,
            own,
            materials,
            ground_material,
            gravity_magnitude,
            seconds,
        );
    }

    BodyOutcome {
        contacts,
        depenetrated: resolution.was_depenetrated(),
        stuck: resolution.stuck,
    }
}

/// Stop or bounce a body on the axis it was blocked on.
fn apply_contact_velocity(
    body: &mut RigidBody,
    axis: Axis,
    own: PhysicsMaterial,
    surface: PhysicsMaterial,
) {
    let incoming = body.velocity.axis(axis);
    let restitution = own.contact_restitution(surface);
    let outgoing = if restitution > 0.0 && incoming.abs() > BOUNCE_THRESHOLD {
        -incoming * restitution
    } else {
        0.0
    };
    body.velocity = body.velocity.with_axis(axis, outgoing);
}

/// Shed horizontal speed against the surface the body is standing on (PHY-15).
///
/// Coulomb friction: the retarding acceleration is the contact coefficient
/// times the normal acceleration, which for a flat floor is gravity. It cannot
/// reverse the motion — a body slowed to a stop stays stopped.
fn apply_ground_friction(
    body: &mut RigidBody,
    own: PhysicsMaterial,
    materials: &MaterialTable,
    ground_material: Option<MaterialId>,
    gravity_magnitude: f64,
    seconds: f64,
) {
    let Some(ground) = ground_material else {
        return;
    };
    let coefficient = own.contact_friction(materials.get(ground));
    if coefficient <= 0.0 || gravity_magnitude <= 0.0 {
        return;
    }
    let horizontal = Vec3::new(body.velocity.x, 0.0, body.velocity.z);
    let speed = horizontal.length();
    if speed <= 0.0 {
        return;
    }
    let loss = coefficient * gravity_magnitude * seconds;
    let remaining = (speed - loss).max(0.0);
    let scaled = horizontal.scaled(remaining / speed);
    body.velocity.x = scaled.x;
    body.velocity.z = scaled.z;
}

/// Resolve a motion, retrying blocked horizontal movement over a low obstacle
/// (PHY-13).
///
/// The step-up is only accepted when it makes more horizontal progress *and*
/// the body comes back down onto something. Without the second condition a
/// character would climb into the air at the edge of a gap.
fn resolve_with_step_up<S: VoxelSource + ?Sized>(
    source: &S,
    aabb: Aabb,
    motion: Vec3,
    step_height: f64,
) -> Resolution {
    let plain = resolve(source, aabb, motion);
    if step_height <= 0.0 || !(plain.is_blocked(Axis::X) || plain.is_blocked(Axis::Z)) {
        return plain;
    }

    let after_vertical = aabb
        .translated(plain.depenetration)
        .moved(Axis::Y, plain.applied.y);
    let lift = sweep_axis(source, after_vertical, Axis::Y, step_height).allowed;
    if lift <= 0.0 {
        return plain;
    }

    let lifted = after_vertical.moved(Axis::Y, lift);
    let mut stepped = lifted;
    let mut applied = Vec3::ZERO;
    let mut contacts: [Option<Contact>; 3] = [None; 3];
    for axis in [Axis::X, Axis::Z] {
        let sweep = sweep_axis(source, stepped, axis, motion.axis(axis));
        stepped = stepped.moved(axis, sweep.allowed);
        applied = applied.with_axis(axis, sweep.allowed);
        contacts[axis.index()] = sweep.contact;
    }

    let freed = aabb.translated(plain.depenetration);
    let horizontal_gain = applied.x.abs() + applied.z.abs();
    let plain_gain = plain.applied.x.abs() + plain.applied.z.abs();
    if horizontal_gain <= plain_gain {
        return plain;
    }

    let drop = sweep_axis(source, stepped, Axis::Y, -lift);
    if !drop.is_blocked() {
        // Nothing to land on: this was a gap, not a step.
        return plain;
    }

    let landed = stepped.moved(Axis::Y, drop.allowed);
    contacts[Axis::Y.index()] = drop.contact;
    Resolution {
        aabb: landed,
        applied: Vec3::new(
            landed.min.x - freed.min.x,
            landed.min.y - freed.min.y,
            landed.min.z - freed.min.z,
        ),
        contacts,
        // The step-up path starts from the plain resolution, which already did
        // whatever freeing was needed.
        depenetration: plain.depenetration,
        stuck: plain.stuck,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{SLEEP_SPEED_THRESHOLD, SLEEP_STEPS};
    use crate::voxel::{EmptySpace, FlatGround};

    fn world() -> PhysicsWorld {
        PhysicsWorld::new(
            GravityField::earthlike(),
            FixedStep::new(60, 20, 8).expect("valid"),
        )
    }

    #[test]
    fn a_new_world_is_empty() {
        let world = world();
        assert!(world.is_empty());
        assert_eq!(world.len(), 0);
        assert_eq!(world.awake_count(), 0);
        assert_eq!(world.iter().count(), 0);
    }

    #[test]
    fn spawning_and_despawning_track_the_population() {
        let mut world = world();
        let a = world.spawn(BodyDescriptor::dynamic()).expect("spawn");
        let b = world.spawn(BodyDescriptor::dynamic()).expect("spawn");
        assert_eq!(world.len(), 2);
        assert!(world.despawn(a));
        assert_eq!(world.len(), 1);
        // Despawning twice is not an error, but it is not a second removal.
        assert!(!world.despawn(a));
        assert_eq!(world.len(), 1);
        assert!(world.body(b).is_some());
    }

    #[test]
    fn a_stale_handle_never_addresses_the_body_that_replaced_it() {
        let mut world = world();
        let first = world.spawn(BodyDescriptor::dynamic()).expect("spawn");
        assert!(world.despawn(first));
        let second = world.spawn(BodyDescriptor::dynamic()).expect("spawn");
        // The slot was reused, so the index matches but the generation does not.
        assert_eq!(first.index(), second.index());
        assert_ne!(first, second);
        assert!(world.body(first).is_none());
        assert!(world.require_body(first).is_err());
        assert!(world.body(second).is_some());
    }

    #[test]
    fn iteration_is_in_slot_order_and_skips_holes() {
        let mut world = world();
        let ids: Vec<_> = (0..4)
            .map(|_| world.spawn(BodyDescriptor::dynamic()).expect("spawn"))
            .collect();
        assert!(world.despawn(ids[1]));
        let seen: Vec<_> = world.iter().map(|(id, _)| id.index()).collect();
        assert_eq!(seen, vec![0, 2, 3]);
    }

    #[test]
    fn an_invalid_descriptor_does_not_consume_a_slot() {
        let mut world = world();
        assert!(world
            .spawn(BodyDescriptor::dynamic().weighing(0.0))
            .is_err());
        assert_eq!(world.len(), 0);
        assert_eq!(world.iter().count(), 0);
    }

    #[test]
    fn a_body_in_free_fall_accelerates_at_gravity() {
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 100.0, 0.5)))
            .expect("spawn");
        let seconds = 1.0 / 60.0;
        for _ in 0..60 {
            world.step_once(&EmptySpace, seconds);
        }
        let body = world.body(id).expect("live");
        // Semi-implicit Euler over one second of 60 steps: v = -g, and the
        // position lags the closed form by half a step, which is the known and
        // accepted behaviour of the integrator rather than an error.
        assert!(
            (body.velocity.y + crate::gravity::EARTHLIKE_GRAVITY).abs() < 1e-9,
            "velocity {}",
            body.velocity.y
        );
        let closed_form = 100.0 - 0.5 * crate::gravity::EARTHLIKE_GRAVITY;
        assert!(
            (body.center.y - closed_form).abs() < 0.1,
            "fell to {} instead of about {closed_form}",
            body.center.y
        );
    }

    #[test]
    fn a_falling_body_lands_and_stays_landed() {
        let ground = FlatGround::at(0);
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 20.0, 0.5)))
            .expect("spawn");
        for _ in 0..600 {
            world.step_once(&ground, 1.0 / 60.0);
        }
        let body = world.body(id).expect("live");
        assert!(body.grounded, "never landed, at y = {}", body.center.y);
        // Half-extent 0.5, so the centre rests at 0.5 above the surface.
        assert!(
            (body.center.y - 0.5).abs() < 1e-9,
            "resting at {}",
            body.center.y
        );
    }

    #[test]
    fn a_landed_body_falls_asleep_and_stops_being_simulated() {
        let ground = FlatGround::at(0);
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 2.0, 0.5)))
            .expect("spawn");
        for _ in 0..600 {
            world.step_once(&ground, 1.0 / 60.0);
        }
        assert_eq!(world.body(id).expect("live").sleep, SleepState::Sleeping);
        assert_eq!(world.awake_count(), 0);
        // And a sleeping body costs nothing.
        let stats = world.step_once(&ground, 1.0 / 60.0);
        assert_eq!(stats.simulated, 0);
    }

    #[test]
    fn mining_the_floor_wakes_what_was_sleeping_on_it() {
        let ground = FlatGround::at(0);
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 2.0, 0.5)))
            .expect("spawn");
        for _ in 0..600 {
            world.step_once(&ground, 1.0 / 60.0);
        }
        assert_eq!(world.body(id).expect("live").sleep, SleepState::Sleeping);

        let region =
            Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(2.0, 2.0, 2.0)).expect("valid");
        assert_eq!(world.wake_in(region), 1);
        assert_eq!(world.body(id).expect("live").sleep, SleepState::Awake);
        // A region nowhere near the body wakes nothing.
        let elsewhere = Aabb::new(Vec3::splat(500.0), Vec3::splat(501.0)).expect("valid");
        assert_eq!(world.wake_in(elsewhere), 0);
    }

    #[test]
    fn changing_gravity_wakes_everything() {
        let ground = FlatGround::at(0);
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 2.0, 0.5)))
            .expect("spawn");
        for _ in 0..600 {
            world.step_once(&ground, 1.0 / 60.0);
        }
        assert_eq!(world.body(id).expect("live").sleep, SleepState::Sleeping);
        world.set_gravity(GravityField::none());
        assert_eq!(world.body(id).expect("live").sleep, SleepState::Awake);
    }

    #[test]
    fn a_bouncy_body_bounces_and_a_dead_one_does_not() {
        let mut world = world();
        let bouncy = world
            .materials_mut()
            .register(PhysicsMaterial {
                restitution: 0.8,
                ..PhysicsMaterial::DEFAULT
            })
            .expect("registered");
        let ground = FlatGround::at(0);

        let dead_id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 10.0, 0.5)))
            .expect("spawn");
        let bouncy_id = world
            .spawn(
                BodyDescriptor::dynamic()
                    .at(Vec3::new(10.5, 10.0, 0.5))
                    .made_of(bouncy),
            )
            .expect("spawn");

        let mut bounced = false;
        for _ in 0..300 {
            world.step_once(&ground, 1.0 / 60.0);
            if world.body(bouncy_id).expect("live").velocity.y > 1.0 {
                bounced = true;
            }
            assert!(
                world.body(dead_id).expect("live").velocity.y <= 1e-9,
                "an unbouncy body bounced"
            );
        }
        assert!(bounced, "a restitution of 0.8 must produce a rebound");
    }

    #[test]
    fn friction_stops_a_slide_and_ice_lets_it_run() {
        // A material id means nothing outside the table that issued it, so
        // both worlds have to register the ice themselves.
        let icy_world_with_ice = || {
            let mut world = world();
            let ice = world
                .materials_mut()
                .register(PhysicsMaterial {
                    friction: 0.02,
                    ..PhysicsMaterial::DEFAULT
                })
                .expect("registered");
            (world, ice)
        };
        let (mut world, ice) = icy_world_with_ice();
        let stone = FlatGround::at(0);
        let slippery = FlatGround::at(0).of(ice);

        let launch = |world: &mut PhysicsWorld| {
            let id = world
                .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 0.5, 0.5)))
                .expect("spawn");
            world
                .body_mut(id)
                .expect("live")
                .set_velocity(Vec3::new(10.0, 0.0, 0.0))
                .expect("finite");
            id
        };

        let on_stone = launch(&mut world);
        for _ in 0..120 {
            world.step_once(&stone, 1.0 / 60.0);
        }
        let stone_speed = world.body(on_stone).expect("live").velocity.x;

        // A fresh world, so the two bodies never share a floor.
        let (mut icy_world, _) = icy_world_with_ice();
        let on_ice = launch(&mut icy_world);
        for _ in 0..120 {
            icy_world.step_once(&slippery, 1.0 / 60.0);
        }
        let ice_speed = icy_world.body(on_ice).expect("live").velocity.x;

        assert!(
            stone_speed < ice_speed,
            "stone {stone_speed} should slow more than ice {ice_speed}"
        );
        assert!(
            stone_speed.abs() < 1e-9,
            "stone never stopped it: {stone_speed}"
        );
        assert!(ice_speed > 5.0, "ice stopped it too fast: {ice_speed}");
    }

    #[test]
    fn a_character_climbs_a_one_block_step_but_not_a_wall() {
        // Floor at y = 0, and a raised ledge at x >= 4 whose top is y = 1.
        struct Ledge {
            height: i64,
        }
        impl VoxelSource for Ledge {
            fn shape_at(
                &self,
                position: nexora_foundation::spatial::BlockPos,
            ) -> crate::voxel::VoxelShape {
                let solid = if position.x >= 4 {
                    position.y < self.height
                } else {
                    position.y < 0
                };
                if solid {
                    crate::voxel::VoxelShape::SOLID
                } else {
                    crate::voxel::VoxelShape::Empty
                }
            }
        }

        let walk = |height: i64| {
            let mut world = PhysicsWorld::new(
                GravityField::earthlike(),
                FixedStep::new(60, 20, 8).expect("valid"),
            );
            let id = world
                .spawn(BodyDescriptor::character().at(Vec3::new(0.5, 0.9, 0.5)))
                .expect("spawn");
            let source = Ledge { height };
            for _ in 0..240 {
                // Drive the character forward at a constant walk, leaving the
                // vertical component to gravity and the solver.
                let falling = world.body(id).expect("live").velocity.y;
                world
                    .body_mut(id)
                    .expect("live")
                    .set_velocity(Vec3::new(4.0, falling, 0.0))
                    .expect("finite");
                world.step_once(&source, 1.0 / 60.0);
            }
            let body = world.body(id).expect("live");
            (body.center.x, body.center.y)
        };

        // A half-block step (step_height 0.6) is climbable.
        let (x_over_step, y_over_step) = walk(1);
        assert!(
            x_over_step > 5.0,
            "did not get onto the ledge: x {x_over_step}"
        );
        assert!(
            (y_over_step - 1.9).abs() < 1e-6,
            "should stand on top of the ledge, at y {y_over_step}"
        );

        // A three-block wall is not.
        let (x_at_wall, _) = walk(3);
        assert!(
            x_at_wall < 4.0,
            "walked through a wall it cannot step over: x {x_at_wall}"
        );
    }

    #[test]
    fn a_character_does_not_step_up_into_thin_air_at_a_ledge() {
        // A pit at x >= 4: stepping up must not let the character walk out over
        // it, because there is nothing to come back down onto.
        struct Pit;
        impl VoxelSource for Pit {
            fn shape_at(
                &self,
                position: nexora_foundation::spatial::BlockPos,
            ) -> crate::voxel::VoxelShape {
                if position.x < 4 && position.y < 0 {
                    crate::voxel::VoxelShape::SOLID
                } else {
                    crate::voxel::VoxelShape::Empty
                }
            }
        }
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::character().at(Vec3::new(0.5, 0.9, 0.5)))
            .expect("spawn");
        for _ in 0..120 {
            let current = world.body(id).expect("live").velocity.y;
            world
                .body_mut(id)
                .expect("live")
                .set_velocity(Vec3::new(4.0, current, 0.0))
                .expect("finite");
            world.step_once(&Pit, 1.0 / 60.0);
        }
        let body = world.body(id).expect("live");
        assert!(
            body.center.y < 0.0,
            "hovered over the pit at y {}",
            body.center.y
        );
    }

    #[test]
    fn kinematic_bodies_move_without_being_stopped_by_terrain() {
        let ground = FlatGround::at(0);
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::kinematic().at(Vec3::new(0.5, 10.0, 0.5)))
            .expect("spawn");
        world
            .body_mut(id)
            .expect("live")
            .set_velocity(Vec3::new(0.0, -1.0, 0.0))
            .expect("finite");
        for _ in 0..1_200 {
            world.step_once(&ground, 1.0 / 60.0);
        }
        // Twenty seconds at one metre per second, straight through the floor,
        // because something outside physics owns where a kinematic body goes.
        assert!(world.body(id).expect("live").center.y < -5.0);
    }

    #[test]
    fn static_bodies_never_move() {
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor {
                body_type: BodyType::Static,
                ..BodyDescriptor::dynamic().at(Vec3::new(0.5, 10.0, 0.5))
            })
            .expect("spawn");
        for _ in 0..600 {
            world.step_once(&EmptySpace, 1.0 / 60.0);
        }
        assert_eq!(
            world.body(id).expect("live").center,
            Vec3::new(0.5, 10.0, 0.5)
        );
    }

    #[test]
    fn advance_runs_the_substeps_the_elapsed_time_earns() {
        let ground = FlatGround::at(0);
        let mut world = world();
        let _ = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 50.0, 0.5)))
            .expect("spawn");
        // One world second at 20 ticks/s and 60 steps/s, but the cap is 8.
        let report = world.advance(&ground, WorldDuration::from_ticks(20));
        assert_eq!(report.substeps, 8);
        assert!(report.fell_behind());
        assert_eq!(report.stats.simulated, 8);
    }

    #[test]
    fn a_non_positive_substep_does_nothing() {
        let ground = FlatGround::at(0);
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 50.0, 0.5)))
            .expect("spawn");
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(world.step_once(&ground, bad), StepStats::default());
        }
        assert_eq!(world.body(id).expect("live").center.y, 50.0);
    }

    #[test]
    fn stepping_is_reproducible_from_the_same_starting_state() {
        // The determinism claim, checked rather than asserted: two worlds built
        // the same way and stepped the same way end in the same state.
        let ground = FlatGround::at(0);
        let build = || {
            let mut world = PhysicsWorld::new(
                GravityField::earthlike(),
                FixedStep::new(60, 20, 8).expect("valid"),
            );
            for index in 0..32 {
                let offset = f64::from(index);
                world
                    .spawn(BodyDescriptor::dynamic().at(Vec3::new(
                        offset * 1.5,
                        10.0 + offset * 0.25,
                        0.5,
                    )))
                    .expect("spawn");
            }
            world
        };
        let mut first = build();
        let mut second = build();
        for _ in 0..400 {
            first.step_once(&ground, 1.0 / 60.0);
            second.step_once(&ground, 1.0 / 60.0);
        }
        for ((id_a, a), (id_b, b)) in first.iter().zip(second.iter()) {
            assert_eq!(id_a, id_b);
            assert_eq!(a.center, b.center, "body {} diverged", id_a.index());
            assert_eq!(a.velocity, b.velocity);
            assert_eq!(a.sleep, b.sleep);
        }
    }

    #[test]
    fn no_body_ever_comes_to_rest_inside_terrain() {
        // The invariant that matters: whatever the starting state, a settled
        // body is outside solid space.
        let ground = FlatGround::at(0);
        let mut world = world();
        let mut seed = 0x5DEE_CE66_D125_1D4Du64;
        for _ in 0..64 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let spread = |shift: u32| (((seed >> shift) & 0xFFF) as f64) / 4_095.0;
            let id = world
                .spawn(BodyDescriptor::dynamic().at(Vec3::new(
                    spread(0) * 20.0 - 10.0,
                    spread(16) * 30.0 + 1.0,
                    spread(32) * 20.0 - 10.0,
                )))
                .expect("spawn");
            world
                .body_mut(id)
                .expect("live")
                .set_velocity(Vec3::new(
                    spread(8) * 20.0 - 10.0,
                    spread(24) * -20.0,
                    spread(40) * 20.0 - 10.0,
                ))
                .expect("finite");
        }
        for _ in 0..900 {
            world.step_once(&ground, 1.0 / 60.0);
        }
        for (id, body) in world.iter() {
            assert!(
                !crate::collision::overlaps_solid(&ground, body.aabb()),
                "body {} sank into the floor at {:?}",
                id.index(),
                body.center
            );
        }
    }

    #[test]
    fn a_block_placed_inside_a_body_pushes_it_out_rather_than_trapping_it() {
        // The routine case the depenetration pass exists for: the world changes
        // under a body that was standing somewhere legal a moment ago.
        struct Filled {
            up_to: i64,
        }
        impl VoxelSource for Filled {
            fn shape_at(
                &self,
                position: nexora_foundation::spatial::BlockPos,
            ) -> crate::voxel::VoxelShape {
                if position.y < self.up_to {
                    crate::voxel::VoxelShape::SOLID
                } else {
                    crate::voxel::VoxelShape::Empty
                }
            }
        }

        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::character().at(Vec3::new(0.5, 0.9, 0.5)))
            .expect("spawn");
        world.step_once(&Filled { up_to: 0 }, 1.0 / 60.0);
        assert!(world.body(id).expect("live").grounded);

        // The floor rises by a block, swallowing the character's legs.
        let raised = Filled { up_to: 1 };
        assert!(crate::collision::overlaps_solid(
            &raised,
            world.body(id).expect("live").aabb()
        ));
        let stats = world.step_once(&raised, 1.0 / 60.0);
        assert_eq!(stats.depenetrated, 1);
        assert_eq!(stats.stuck, 0);
        assert!(!crate::collision::overlaps_solid(
            &raised,
            world.body(id).expect("live").aabb()
        ));
        assert!(world.body(id).expect("live").center.y > 1.0);
    }

    #[test]
    fn a_body_buried_beyond_recovery_is_reported_not_flung() {
        struct Everywhere;
        impl VoxelSource for Everywhere {
            fn shape_at(
                &self,
                _position: nexora_foundation::spatial::BlockPos,
            ) -> crate::voxel::VoxelShape {
                crate::voxel::VoxelShape::SOLID
            }
        }
        let mut world = world();
        let id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 0.5, 0.5)))
            .expect("spawn");
        let before = world.body(id).expect("live").center;
        let stats = world.step_once(&Everywhere, 1.0 / 60.0);
        assert_eq!(stats.stuck, 1);
        assert_eq!(world.body(id).expect("live").center, before);
    }

    #[test]
    fn sleep_thresholds_are_reachable_within_a_reasonable_run() {
        // Guards the constants against being set to values that make sleeping
        // unreachable, which would silently turn the optimisation off. Checked
        // at compile time, so the guard cannot be skipped by not running a test.
        const {
            assert!(SLEEP_SPEED_THRESHOLD > 0.0);
            assert!(SLEEP_STEPS > 0 && SLEEP_STEPS < 600);
        }
    }
}
