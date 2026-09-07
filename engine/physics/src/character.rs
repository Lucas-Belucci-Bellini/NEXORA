//! Character control (PHY-13) and ground state (PHY-14).
//!
//! `PHYSICS.md` §14 is explicit that a player or NPC should not be a generic
//! rigid body pushed around by forces: a character that accelerates like a
//! crate feels wrong and, worse, is not controllable. So the controller writes
//! **velocity** rather than forces, and everything below it — gravity,
//! collision, step-up, friction — is the ordinary solver.
//!
//! ## Why authority differs on the ground and in the air
//!
//! §15 asks other systems to be able to ask `player.isGrounded()` without
//! knowing how physics decides it. That answer is also what decides how much of
//! the character's intent is honoured: full control while standing on
//! something, a configurable fraction while falling. Air control of `1.0` would
//! let a character change direction in mid-air as freely as on the ground;
//! `0.0` would make a jump uncorrectable.

use nexora_foundation::error::Result;

use crate::body::BodyId;
use crate::math::Vec3;
use crate::world::PhysicsWorld;

/// Whether a character is standing on something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroundState {
    /// Resting on a surface.
    Grounded,
    /// Falling, rising or otherwise unsupported.
    Airborne,
}

impl GroundState {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grounded => "grounded",
            Self::Airborne => "airborne",
        }
    }

    /// Whether the character is supported.
    #[must_use]
    pub const fn is_grounded(self) -> bool {
        matches!(self, Self::Grounded)
    }
}

/// What a character is trying to do this tick.
///
/// The direction is a **horizontal intent**, not a velocity: its length is
/// ignored beyond zero-or-not, so an analogue stick at half deflection and a
/// keyboard both produce the same walk. A caller that wants to walk slowly
/// should say so through [`CharacterController::walk_speed`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MoveIntent {
    /// Desired direction of travel. The vertical component is ignored.
    pub direction: Vec3,
    /// Whether to jump, honoured only when grounded.
    pub jump: bool,
}

impl MoveIntent {
    /// Stand still.
    pub const STILL: Self = Self {
        direction: Vec3::ZERO,
        jump: false,
    };

    /// Walk in a direction.
    #[must_use]
    pub const fn walking(direction: Vec3) -> Self {
        Self {
            direction,
            jump: false,
        }
    }

    /// Jump without walking.
    #[must_use]
    pub const fn jumping() -> Self {
        Self {
            direction: Vec3::ZERO,
            jump: true,
        }
    }

    /// The same intent with the jump requested.
    #[must_use]
    pub const fn and_jump(mut self) -> Self {
        self.jump = true;
        self
    }
}

/// Turns intent into velocity for one body.
///
/// Holds tuning, not state. The character's actual state lives in the body, so
/// a controller can be rebuilt from configuration at any time without losing
/// where the character is or how fast it is moving.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CharacterController {
    /// The body this controller drives.
    pub body: BodyId,
    /// Ground speed, in metres per second.
    pub walk_speed: f64,
    /// Upward speed imparted by a jump, in metres per second.
    pub jump_speed: f64,
    /// Fraction of walking authority retained while airborne, `0.0` to `1.0`.
    pub air_control: f64,
}

impl CharacterController {
    /// A controller with humanlike defaults: 4.3 m/s walk, a jump that clears
    /// slightly over one block.
    #[must_use]
    pub const fn new(body: BodyId) -> Self {
        Self {
            body,
            walk_speed: 4.3,
            jump_speed: 5.0,
            air_control: 0.2,
        }
    }

    /// Set the walking speed.
    #[must_use]
    pub const fn walking_at(mut self, speed: f64) -> Self {
        self.walk_speed = speed;
        self
    }

    /// Set the jump speed.
    #[must_use]
    pub const fn jumping_at(mut self, speed: f64) -> Self {
        self.jump_speed = speed;
        self
    }

    /// Set how much control the character keeps in the air.
    #[must_use]
    pub const fn with_air_control(mut self, air_control: f64) -> Self {
        self.air_control = air_control;
        self
    }

    /// Whether the body is currently supported.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle no longer addresses a live body.
    pub fn ground_state(&self, world: &PhysicsWorld) -> Result<GroundState> {
        let body = world.require_body(self.body)?;
        Ok(if body.grounded {
            GroundState::Grounded
        } else {
            GroundState::Airborne
        })
    }

    /// Apply one tick of intent, returning the state it was applied in.
    ///
    /// Writes horizontal velocity directly rather than accumulating force: see
    /// the module documentation for why. The vertical component is left to
    /// gravity, except for the jump.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle no longer addresses a live body, or
    /// when the resulting velocity would not be finite.
    pub fn apply(&self, world: &mut PhysicsWorld, intent: MoveIntent) -> Result<GroundState> {
        let (grounded, current) = {
            let body = world.require_body(self.body)?;
            (body.grounded, body.velocity)
        };
        let state = if grounded {
            GroundState::Grounded
        } else {
            GroundState::Airborne
        };

        let authority = if grounded {
            1.0
        } else {
            self.air_control.clamp(0.0, 1.0)
        };

        let flat = Vec3::new(intent.direction.x, 0.0, intent.direction.z);
        let length = flat.length();
        let desired = if length > 0.0 && self.walk_speed > 0.0 {
            flat.scaled(self.walk_speed / length)
        } else {
            Vec3::ZERO
        };

        // Blend towards the desired horizontal velocity. On the ground the
        // blend is total, which is what makes a character stop when the input
        // stops instead of sliding on for a second.
        let blended = Vec3::new(
            current.x + (desired.x - current.x) * authority,
            current.y,
            current.z + (desired.z - current.z) * authority,
        );

        let velocity = if intent.jump && grounded {
            blended.with_axis(crate::math::Axis::Y, self.jump_speed)
        } else {
            blended
        };

        let body = world.require_body_mut(self.body)?;
        body.set_velocity(velocity)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::BodyDescriptor;
    use crate::gravity::GravityField;
    use crate::step::FixedStep;
    use crate::voxel::FlatGround;

    fn world_with_character() -> (PhysicsWorld, CharacterController) {
        let mut world = PhysicsWorld::new(
            GravityField::earthlike(),
            FixedStep::new(60, 20, 8).expect("valid"),
        );
        let body = world
            .spawn(BodyDescriptor::character().at(Vec3::new(0.5, 0.9, 0.5)))
            .expect("spawn");
        (world, CharacterController::new(body))
    }

    #[test]
    fn walking_on_the_ground_sets_the_full_walk_speed() {
        let (mut world, controller) = world_with_character();
        world.body_mut(controller.body).expect("live").grounded = true;
        let state = controller
            .apply(&mut world, MoveIntent::walking(Vec3::new(1.0, 0.0, 0.0)))
            .expect("applied");
        assert_eq!(state, GroundState::Grounded);
        let velocity = world.body(controller.body).expect("live").velocity;
        assert!((velocity.x - controller.walk_speed).abs() < 1e-12);
        assert!(velocity.z.abs() < 1e-12);
    }

    #[test]
    fn intent_length_does_not_change_the_speed() {
        let (mut world, controller) = world_with_character();
        world.body_mut(controller.body).expect("live").grounded = true;
        controller
            .apply(&mut world, MoveIntent::walking(Vec3::new(100.0, 0.0, 0.0)))
            .expect("applied");
        let fast = world.body(controller.body).expect("live").velocity.x;
        controller
            .apply(&mut world, MoveIntent::walking(Vec3::new(0.001, 0.0, 0.0)))
            .expect("applied");
        let slow = world.body(controller.body).expect("live").velocity.x;
        assert!((fast - slow).abs() < 1e-12);
    }

    #[test]
    fn a_vertical_intent_component_is_ignored() {
        let (mut world, controller) = world_with_character();
        world.body_mut(controller.body).expect("live").grounded = true;
        controller
            .apply(&mut world, MoveIntent::walking(Vec3::new(0.0, 10.0, 0.0)))
            .expect("applied");
        // No horizontal component means no walk, and the vertical part of the
        // intent must not become a jump.
        let velocity = world.body(controller.body).expect("live").velocity;
        assert!(velocity.length() < 1e-12);
    }

    #[test]
    fn releasing_the_input_stops_a_grounded_character() {
        let (mut world, controller) = world_with_character();
        world.body_mut(controller.body).expect("live").grounded = true;
        controller
            .apply(&mut world, MoveIntent::walking(Vec3::new(1.0, 0.0, 0.0)))
            .expect("applied");
        controller
            .apply(&mut world, MoveIntent::STILL)
            .expect("applied");
        let velocity = world.body(controller.body).expect("live").velocity;
        assert!(velocity.x.abs() < 1e-12, "still sliding at {}", velocity.x);
    }

    #[test]
    fn air_control_limits_how_much_intent_is_honoured() {
        let (mut world, controller) = world_with_character();
        let controller = controller.with_air_control(0.25);
        {
            let body = world.body_mut(controller.body).expect("live");
            body.grounded = false;
            body.set_velocity(Vec3::ZERO).expect("finite");
        }
        controller
            .apply(&mut world, MoveIntent::walking(Vec3::new(1.0, 0.0, 0.0)))
            .expect("applied");
        let velocity = world.body(controller.body).expect("live").velocity;
        assert!((velocity.x - controller.walk_speed * 0.25).abs() < 1e-12);
    }

    #[test]
    fn a_jump_is_refused_in_mid_air() {
        let (mut world, controller) = world_with_character();
        world.body_mut(controller.body).expect("live").grounded = false;
        let state = controller
            .apply(&mut world, MoveIntent::jumping())
            .expect("applied");
        assert_eq!(state, GroundState::Airborne);
        assert!(world.body(controller.body).expect("live").velocity.y.abs() < 1e-12);
    }

    #[test]
    fn a_jump_from_the_ground_imparts_the_jump_speed() {
        let (mut world, controller) = world_with_character();
        world.body_mut(controller.body).expect("live").grounded = true;
        controller
            .apply(&mut world, MoveIntent::jumping())
            .expect("applied");
        let velocity = world.body(controller.body).expect("live").velocity;
        assert!((velocity.y - controller.jump_speed).abs() < 1e-12);
    }

    #[test]
    fn walking_and_jumping_compose() {
        let (mut world, controller) = world_with_character();
        world.body_mut(controller.body).expect("live").grounded = true;
        controller
            .apply(
                &mut world,
                MoveIntent::walking(Vec3::new(0.0, 0.0, 1.0)).and_jump(),
            )
            .expect("applied");
        let velocity = world.body(controller.body).expect("live").velocity;
        assert!((velocity.z - controller.walk_speed).abs() < 1e-12);
        assert!((velocity.y - controller.jump_speed).abs() < 1e-12);
    }

    #[test]
    fn a_controller_on_a_dead_body_reports_the_stale_handle() {
        let (mut world, controller) = world_with_character();
        assert!(world.despawn(controller.body));
        assert!(controller.ground_state(&world).is_err());
        assert!(controller.apply(&mut world, MoveIntent::STILL).is_err());
    }

    #[test]
    fn the_ground_state_follows_what_the_solver_decided() {
        let ground = FlatGround::at(0);
        let (mut world, controller) = world_with_character();
        assert_eq!(
            controller.ground_state(&world).expect("live"),
            GroundState::Airborne
        );
        // One step is enough for the solver to notice the floor underfoot.
        world.step_once(&ground, 1.0 / 60.0);
        assert_eq!(
            controller.ground_state(&world).expect("live"),
            GroundState::Grounded
        );
    }
}
