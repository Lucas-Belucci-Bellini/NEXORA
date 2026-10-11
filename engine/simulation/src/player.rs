//! The player: a body the simulation steers (ADR-0035).
//!
//! `PLAYER SYSTEM.md` draws the chain this module is: *Player → Movement
//! Intent → Character Controller → Physics → Final Position*, and says the
//! player system does not implement collision. So the player here is three
//! things and nothing else:
//!
//! * **a character body** in a physics world of its own, which owns where the
//!   player is, how fast it moves and whether it stands on something — the
//!   solver's answer, and the only copy of it;
//! * **the existing [`CharacterController`]**, which turns a heading into
//!   velocity; this module adds no physics;
//! * **a facing**, yaw and pitch, because physics has no rotation and the
//!   camera must not own gameplay state (`CAMERA SYSTEM.md`).
//!
//! It is fed a [`Walk`] — what the player asks for, in `[-1, 1]` per axis and
//! a jump — never a position or a velocity (`INPUT SYSTEM.md`: input produces
//! intent and never changes the world). No input is [`Walk::default`], which
//! stands still; it never means "repeat the last move".
//!
//! ## One call, one world tick
//!
//! [`Player::tick`] is exactly one world tick: one controller apply, then
//! `advance(WorldDuration::from_ticks(1))`, which at 20 ticks per second is
//! three substeps of 1/60 s. A frame that owes several ticks calls it several
//! times. Handing the physics accumulator four ticks at once would earn twelve
//! substeps against a cap of eight, and four would be dropped in silence.
//!
//! ## Where the trigonometry is
//!
//! In the private `aim`, which turns the facing and the walk into a unit
//! heading (`sin_cos` of the yaw), and in [`Eye::forward`], which the
//! interaction ray is cast along (ADR-0036). The step below `aim` is
//! trig-free — the physics crate uses no transcendental function but `sqrt` —
//! so the heading is the unit a future command log or network message
//! carries: the part that a platform's `libm` could round differently is
//! decided once, by whoever owns the player, and the rest replays bit for bit
//! on any target. The interaction ray ends the same way: what it decides is a
//! cell, and the command carries the cell's integers, not the angle.
//!
//! ## Rest is a fixed point, not sleep
//!
//! The controller writes velocity every tick, and writing velocity wakes a
//! body, so the player never sleeps. It does not need to: [`Player::spawn`]
//! puts the feet on the integer plane at a column's centre and refuses to
//! return until a still tick leaves the state bit-identical, and a resting
//! character is a fixed point of the solver from there on (pinned by
//! thousand-tick tests here and on generated terrain). That is what lets the
//! client require *exactly* zero movement when no key is pressed.

use std::f64::consts::{FRAC_PI_2, PI};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{BlockPos, WorldPosition};
use nexora_foundation::time::WorldDuration;
use nexora_physics::body::{BodyDescriptor, RigidBody};
use nexora_physics::character::{CharacterController, MoveIntent};
use nexora_physics::collision::overlaps_solid;
use nexora_physics::math::{Axis, Vec3};
use nexora_physics::voxel::VoxelSource;
use nexora_physics::world::PhysicsWorld;

use crate::spawn::Run;

/// How far above the feet the eye is, in metres.
///
/// The character box is 1.8 m tall, so the eye sits 0.72 above the body's
/// centre and 0.18 below the top of the box, and at least 0.3 from any wall
/// the box touches. Both are more than the 0.161 m from the eye to the corner
/// of a 70° near plane at 0.1, so no face the body can touch is ever clipped
/// by the near plane.
pub const EYE_HEIGHT: f64 = 1.62;

/// How fast the player turns and looks, in radians per second of world time.
pub const TURN_SPEED: f64 = FRAC_PI_2;

/// The steepest the player may look up or down, in radians: 89.9°.
///
/// The same value as `nexora_camera::MAX_PITCH`, written again here because
/// the simulation does not depend on the camera; the client checks the two
/// are equal to the bit, so a camera made from the eye never clamps what the
/// player decided.
pub const MAX_PITCH: f64 = FRAC_PI_2 - 0.1 * PI / 180.0;

/// The most still ticks [`Player::spawn`] runs before it gives up on the body
/// coming to rest.
///
/// A body placed with its feet on an integer plane is at rest after the first
/// tick that finds the floor; four is that, with room for the solver to snap
/// a box that began a fraction of a unit in the last place inside the floor.
pub const SETTLE_TICKS: u32 = 4;

/// What the player asks for in one tick, in the player's own frame.
///
/// Action-level intent: how hard to walk forward and right, turn and look,
/// each in `[-1, 1]`, and whether to jump. It carries no position and no
/// velocity, so nothing that produces one can move the player anywhere the
/// solver would not.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Walk {
    /// `+1` forward, `-1` back.
    pub forward: f64,
    /// `+1` right, `-1` left.
    pub right: f64,
    /// `+1` turn left (counter-clockwise seen from above), `-1` right.
    pub turn: f64,
    /// `+1` look up, `-1` down.
    pub look: f64,
    /// Jump; honoured only while standing on something.
    pub jump: bool,
}

impl Walk {
    /// The same walk with every axis inside `[-1, 1]`, and a non-finite axis
    /// read as `0`.
    ///
    /// The boundary of the player is where an untrusted value stops: a
    /// controller, a script or a network message that asks for fifty times
    /// full deflection gets full deflection, and one that sends a NaN asks
    /// for nothing.
    #[must_use]
    pub fn clamped(self) -> Self {
        let axis = |value: f64| {
            if value.is_finite() {
                value.clamp(-1.0, 1.0)
            } else {
                0.0
            }
        };
        Self {
            forward: axis(self.forward),
            right: axis(self.right),
            turn: axis(self.turn),
            look: axis(self.look),
            jump: self.jump,
        }
    }
}

/// Where the player sees from, in foundation types.
///
/// A camera is made from this every frame and never read back: the camera
/// owns no gameplay state (`CAMERA SYSTEM.md`), and the simulation does not
/// depend on the camera crate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eye {
    /// The eye, [`EYE_HEIGHT`] above the feet at the centre of the body.
    pub position: WorldPosition,
    /// Rotation about `+Y`, in radians, in `(-π, π]`; zero looks down `-Z`.
    pub yaw: f64,
    /// Elevation, in radians, within `±MAX_PITCH`; positive looks up.
    pub pitch: f64,
}

impl Eye {
    /// The unit direction the eye looks along: what the centre of the
    /// screen shows.
    ///
    /// The camera's own formula (`nexora_camera::Camera::forward`), written
    /// again because the simulation does not depend on the camera; the
    /// client checks the two agree to the bit, so the block a player aims at
    /// is the block under the centre of a frame drawn at this eye. Between
    /// ticks the client draws a little behind it (ADR-0040), never ahead.
    #[must_use]
    pub fn forward(&self) -> [f64; 3] {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        [-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch]
    }
}

/// Everything the player is, read from the body and the facing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerState {
    /// The bottom of the body, at its horizontal centre.
    pub feet: WorldPosition,
    /// The body's velocity, in metres per second.
    pub velocity: [f64; 3],
    /// Whether the solver left the body standing on something.
    pub grounded: bool,
    /// Rotation about `+Y`, in radians, in `(-π, π]`.
    pub yaw: f64,
    /// Elevation, in radians, within `±MAX_PITCH`.
    pub pitch: f64,
}

impl PlayerState {
    /// Whether two states are the same to the bit.
    ///
    /// Stricter than `==`, which calls `0.0` and `-0.0` equal: a replay that
    /// differs only in the sign of a zero has still diverged, and this is the
    /// comparison the determinism checks are written in.
    #[must_use]
    pub fn is_bit_identical(&self, other: &Self) -> bool {
        self.bits() == other.bits()
    }

    fn bits(&self) -> ([u64; 8], bool) {
        (
            [
                self.feet.x.to_bits(),
                self.feet.y.to_bits(),
                self.feet.z.to_bits(),
                self.velocity[0].to_bits(),
                self.velocity[1].to_bits(),
                self.velocity[2].to_bits(),
                self.yaw.to_bits(),
                self.pitch.to_bits(),
            ],
            self.grounded,
        )
    }
}

/// What one tick of the player did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TickOutcome {
    /// Physics substeps the tick ran.
    pub substeps: u32,
    /// Contacts resolved against the terrain.
    pub contacts: u32,
    /// Substeps that had to push the body out of terrain it was inside.
    ///
    /// Zero whenever the terrain did not change around the body. A caller
    /// whose world nothing edits may treat non-zero as a defect.
    pub depenetrated: u32,
}

/// A player: one character body the simulation steers, and where it faces.
///
/// The body lives in a physics world of its own (DEBT-0050): the first NPC or
/// crate that shares the frame with the player will need one world for both,
/// and the public surface — [`Player::spawn`], [`Player::tick`],
/// [`Player::eye`], [`Player::state`] — is what stays when that happens.
#[derive(Debug, Clone)]
pub struct Player {
    physics: PhysicsWorld,
    controller: CharacterController,
    yaw: f64,
    pitch: f64,
    /// One world tick, in seconds: what a turn is scaled by.
    tick_seconds: f64,
    /// The substeps one world tick must run; anything else is a defect.
    substeps_per_tick: u32,
}

impl Player {
    /// Put a character body on the first column of `run`, facing along it,
    /// and wait for it to come to rest.
    ///
    /// The feet are placed on the integer plane at the centre of the column,
    /// which is where a resting box is a fixed point of the solver. Up to
    /// [`SETTLE_TICKS`] still ticks run, against `terrain`, until one leaves
    /// the state grounded and identical to the bit; this does not advance any
    /// world clock, because the player owns its physics world.
    ///
    /// # Errors
    ///
    /// Refused ([`Recovery::Reject`]) when the tick rate is not a whole
    /// number of physics substeps within the substep cap, the pitch is not
    /// finite, the body would begin inside terrain, or it does not come to
    /// rest; and any error a tick returns.
    pub fn spawn<S: VoxelSource + ?Sized>(
        terrain: &S,
        run: &Run,
        pitch: f64,
        ticks_per_second: u32,
    ) -> Result<Self> {
        if !pitch.is_finite() {
            return Err(
                refused("the spawn pitch is not finite").with_context("pitch", pitch.to_string())
            );
        }
        let half_height = BodyDescriptor::character().half_extents.y;
        let centre = Vec3::new(
            run.x as f64 + 0.5,
            run.feet_y as f64 + half_height,
            run.z as f64 + 0.5,
        );
        let mut player = Self::placed(
            centre,
            run.facing.yaw(),
            pitch.clamp(-MAX_PITCH, MAX_PITCH),
            ticks_per_second,
        )?;
        if overlaps_solid(terrain, player.body().aabb()) {
            return Err(refused("the spawn is inside terrain")
                .with_context("column", format!("{},{}", run.x, run.z))
                .with_context("feet", run.feet_y.to_string()));
        }

        let mut before = player.state();
        for _ in 0..SETTLE_TICKS {
            player.tick(terrain, Walk::default())?;
            let after = player.state();
            if after.grounded && after.is_bit_identical(&before) {
                return Ok(player);
            }
            before = after;
        }
        Err(refused("the player did not come to rest at its spawn")
            .with_context("column", format!("{},{}", run.x, run.z))
            .with_context("ticks", SETTLE_TICKS.to_string()))
    }

    /// Put the player back exactly as `state` describes it: the feet, the
    /// velocity, whether it stood on something, and where it faced — the
    /// player a save kept (PLAYER-53, ADR-0036).
    ///
    /// Nothing is run: a player saved mid-jump resumes mid-jump, and from
    /// here every tick is the tick the saved player would have run next, to
    /// the bit (pinned by a test here and by the headless slice).
    ///
    /// # Errors
    ///
    /// Refused ([`Recovery::Reject`]) when the tick rate is not a whole
    /// number of physics substeps within the substep cap, a value is not
    /// finite, the facing is outside `(-π, π]` and `±MAX_PITCH`, or the body
    /// would begin inside terrain — terrain the save does not describe, or
    /// one edited since. A caller with a fallback spawns instead.
    pub fn resume<S: VoxelSource + ?Sized>(
        terrain: &S,
        state: &PlayerState,
        ticks_per_second: u32,
    ) -> Result<Self> {
        let finite = [
            state.feet.x,
            state.feet.y,
            state.feet.z,
            state.velocity[0],
            state.velocity[1],
            state.velocity[2],
            state.yaw,
            state.pitch,
        ]
        .iter()
        .all(|value| value.is_finite());
        let facing = state.yaw > -PI && state.yaw <= PI && state.pitch.abs() <= MAX_PITCH;
        if !(finite && facing) {
            return Err(
                refused("a resumed player is not a state a player can be in")
                    .with_context("state", format!("{state:?}")),
            );
        }
        let half_height = BodyDescriptor::character().half_extents.y;
        let centre = Vec3::new(state.feet.x, state.feet.y + half_height, state.feet.z);
        let mut player = Self::placed(centre, state.yaw, state.pitch, ticks_per_second)?;
        let [vx, vy, vz] = state.velocity;
        let handle = player.controller.body;
        let body = player
            .physics
            .body_mut(handle)
            .expect("the player's body lives as long as the player");
        body.set_velocity(Vec3::new(vx, vy, vz))?;
        body.grounded = state.grounded;
        if overlaps_solid(terrain, player.body().aabb()) {
            return Err(refused("the resumed player is inside terrain")
                .with_context("feet", format!("{:?}", state.feet)));
        }
        Ok(player)
    }

    /// A body whose centre is `centre`, facing `yaw` and `pitch`, in a
    /// physics world of its own at `ticks_per_second`.
    fn placed(centre: Vec3, yaw: f64, pitch: f64, ticks_per_second: u32) -> Result<Self> {
        let mut physics = PhysicsWorld::earthlike(ticks_per_second)?;
        let rate = physics.step().steps_per_second();
        if rate % ticks_per_second != 0 || rate / ticks_per_second > physics.step().max_substeps() {
            return Err(
                refused("a world tick is not a whole number of physics substeps")
                    .with_context("ticks_per_second", ticks_per_second.to_string())
                    .with_context("substeps_per_second", rate.to_string()),
            );
        }
        let body = physics.spawn(BodyDescriptor::character().at(centre))?;
        Ok(Self {
            physics,
            controller: CharacterController::new(body),
            yaw,
            pitch,
            tick_seconds: 1.0 / f64::from(ticks_per_second),
            substeps_per_tick: rate / ticks_per_second,
        })
    }

    /// Run exactly one world tick of `walk` against `terrain`.
    ///
    /// The walk is clamped first ([`Walk::clamped`]). The facing turns, then
    /// the body moves in the direction the new facing gives: one controller
    /// apply, then one tick of physics. The terrain is only read.
    ///
    /// # Errors
    ///
    /// [`Recovery::Manual`] when the body is buried and cannot be freed, the
    /// tick did not run as exactly one tick of physics substeps, or the body
    /// ends the tick inside terrain. None of them can happen against terrain
    /// that does not change around the body; each is a defect worth stopping
    /// for, not a state to carry on from.
    pub fn tick<S: VoxelSource + ?Sized>(
        &mut self,
        terrain: &S,
        walk: Walk,
    ) -> Result<TickOutcome> {
        let heading = self.aim(walk.clamped());
        self.advance(terrain, heading)
    }

    /// Where the player sees from.
    #[must_use]
    pub fn eye(&self) -> Eye {
        let body = self.body();
        let centre = body.center;
        Eye {
            position: WorldPosition::new(
                centre.x,
                centre.y + (EYE_HEIGHT - body.half_extents.y),
                centre.z,
            ),
            yaw: self.yaw,
            pitch: self.pitch,
        }
    }

    /// Everything the player is, now.
    #[must_use]
    pub fn state(&self) -> PlayerState {
        let body = self.body();
        let (centre, velocity) = (body.center, body.velocity);
        PlayerState {
            feet: WorldPosition::new(centre.x, centre.y - body.half_extents.y, centre.z),
            velocity: [velocity.x, velocity.y, velocity.z],
            grounded: body.grounded,
            yaw: self.yaw,
            pitch: self.pitch,
        }
    }

    /// The cells the body occupies, lowest and highest corner inclusive.
    ///
    /// Read with physics' own touching convention (`Aabb::voxel_span`): a
    /// body resting exactly on a floor does not occupy the floor's cell, and
    /// one flush against a wall does not occupy the wall's. A block placed in
    /// any of these cells would bury the body; one placed outside them
    /// cannot (BUILD-1's *colisão*).
    #[must_use]
    pub fn occupied_cells(&self) -> (BlockPos, BlockPos) {
        let aabb = self.body().aabb();
        let [x, y, z] = [Axis::X, Axis::Y, Axis::Z].map(|axis| aabb.voxel_span(axis));
        (BlockPos::new(x.0, y.0, z.0), BlockPos::new(x.1, y.1, z.1))
    }

    /// Turn the facing by the walk, and derive the heading the body is to
    /// move in. The only trigonometry a tick does.
    ///
    /// The facing changes only when a turn or a look was asked for, so a
    /// player standing still keeps its yaw to the bit — wrapping a yaw that
    /// is already in range can move a negative one by one unit in the last
    /// place.
    fn aim(&mut self, walk: Walk) -> MoveIntent {
        if walk.turn != 0.0 || walk.look != 0.0 {
            self.yaw = wrap_yaw(self.yaw + walk.turn * TURN_SPEED * self.tick_seconds);
            self.pitch = (self.pitch + walk.look * TURN_SPEED * self.tick_seconds)
                .clamp(-MAX_PITCH, MAX_PITCH);
        }
        // Forward in the horizontal plane is (-sin, -cos) and right of it is
        // (cos, -sin): yaw zero looks down -Z, so right is +X (ADR-0029). The
        // pitch plays no part, so looking down does not slow walking, and the
        // controller normalises the length, so a diagonal is no faster.
        let direction = if walk.forward == 0.0 && walk.right == 0.0 {
            Vec3::ZERO
        } else {
            let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
            Vec3::new(
                -sin_yaw * walk.forward + cos_yaw * walk.right,
                0.0,
                -cos_yaw * walk.forward - sin_yaw * walk.right,
            )
        };
        MoveIntent {
            direction,
            jump: walk.jump,
        }
    }

    /// One controller apply and one world tick of physics, in one session
    /// against `terrain`. No trigonometry: what it is given is a heading.
    fn advance<S: VoxelSource + ?Sized>(
        &mut self,
        terrain: &S,
        heading: MoveIntent,
    ) -> Result<TickOutcome> {
        let report = {
            let mut session = self.physics.against(terrain);
            self.controller.apply(session.world_mut(), heading)?;
            session.advance(WorldDuration::from_ticks(1))
        };
        if report.stats.stuck > 0 {
            return Err(defect(
                "the player is buried in terrain and could not be freed",
            ));
        }
        if report.dropped_substeps > 0 || report.substeps != self.substeps_per_tick {
            return Err(
                defect("one world tick did not run as one tick of physics substeps")
                    .with_context("substeps", report.substeps.to_string())
                    .with_context("dropped", report.dropped_substeps.to_string())
                    .with_context("expected", self.substeps_per_tick.to_string()),
            );
        }
        let body = self.body();
        if overlaps_solid(terrain, body.aabb()) {
            let centre = body.center;
            return Err(
                defect("the player ended a tick inside terrain").with_context(
                    "centre",
                    format!("{:.6},{:.6},{:.6}", centre.x, centre.y, centre.z),
                ),
            );
        }
        Ok(TickOutcome {
            substeps: report.substeps,
            contacts: report.stats.contacts,
            depenetrated: report.stats.depenetrated,
        })
    }

    /// The player's body.
    fn body(&self) -> &RigidBody {
        // The physics world is private and nothing in it is ever despawned,
        // so the handle the player was built with addresses its body for as
        // long as the player exists.
        self.physics
            .body(self.controller.body)
            .expect("the player's body lives as long as the player")
    }
}

/// Wrap a yaw into `(-π, π]`, the way the camera does.
fn wrap_yaw(yaw: f64) -> f64 {
    let wrapped = yaw.rem_euclid(2.0 * PI);
    if wrapped > PI {
        wrapped - 2.0 * PI
    } else {
        wrapped
    }
}

fn refused(message: &'static str) -> Error {
    Error::new(Domain::Physics, "player", message).with_recovery(Recovery::Reject)
}

fn defect(message: &'static str) -> Error {
    Error::new(Domain::Physics, "player", message).with_recovery(Recovery::Manual)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spawn::{find_walkable_run, Cardinal, ColumnArea};
    use nexora_foundation::spatial::BlockPos;
    use nexora_physics::voxel::{FlatGround, VoxelShape};

    const TPS: u32 = 20;

    /// A floor whose top cell is `y = -1` everywhere, so standing feet are at
    /// `y = 0`, and a search area around the origin.
    fn flat() -> (FlatGround, ColumnArea) {
        (
            FlatGround::at(0),
            ColumnArea {
                min_x: -16,
                min_z: -16,
                max_x: 16,
                max_z: 16,
                floor_y: -8,
                ceiling_y: 8,
            },
        )
    }

    fn on_flat_ground() -> (FlatGround, Player, Run) {
        let (ground, area) = flat();
        let run = find_walkable_run(&ground, &area, 4).expect("a flat area is all run");
        let player = Player::spawn(&ground, &run, 0.0, TPS).expect("spawns on flat ground");
        (ground, player, run)
    }

    const FORWARD: Walk = Walk {
        forward: 1.0,
        right: 0.0,
        turn: 0.0,
        look: 0.0,
        jump: false,
    };

    /// Terrain along `-Z` from a column at `z = 0`: feet stand at `y = 0`
    /// down to `z = -3`, one block higher to `z = -6`, and two blocks higher
    /// than that beyond it — a step the character climbs, then a wall it
    /// cannot, whose face is the plane `z = -6`.
    struct Stepped;

    impl Stepped {
        /// The top solid cell of a column.
        const fn top(z: i64) -> i64 {
            if z >= -3 {
                -1
            } else if z >= -6 {
                0
            } else {
                2
            }
        }
    }

    impl VoxelSource for Stepped {
        fn shape_at(&self, position: BlockPos) -> VoxelShape {
            if position.y <= Self::top(position.z) {
                VoxelShape::SOLID
            } else {
                VoxelShape::Empty
            }
        }
    }

    #[test]
    fn a_spawned_player_rests_exactly_on_the_floor() {
        let (_, player, run) = on_flat_ground();
        let state = player.state();
        assert_eq!(state.feet.y.to_bits(), (run.feet_y as f64).to_bits());
        assert_eq!(run.feet_y, 0);
        assert!(state.grounded);
        assert_eq!(state.velocity, [0.0; 3]);
        assert_eq!(state.yaw, run.facing.yaw());
    }

    #[test]
    fn a_player_given_no_intent_does_not_move_bit_for_bit() {
        let (ground, mut player, _) = on_flat_ground();
        let spawned = player.state();
        for tick in 0..1_000 {
            player.tick(&ground, Walk::default()).expect("a still tick");
            let now = player.state();
            assert!(now.grounded, "tick {tick}");
            assert!(now.is_bit_identical(&spawned), "tick {tick}: {now:?}");
        }
    }

    /// Friction acts after each substep's motion, so a grounded tick at
    /// 4.3 m/s covers the three substeps at 4.3, 4.3 - μg/60 and
    /// 4.3 - 2μg/60, for μ = sqrt(0.6 · 0.6) and g = 9.80665: 0.210096675 m.
    #[test]
    fn walking_forward_on_flat_ground_matches_the_closed_form() {
        let (ground, mut player, run) = on_flat_ground();
        assert_eq!(run.facing, Cardinal::NegZ, "yaw zero looks down -Z");
        let start = player.state();
        for tick in 0..20 {
            let outcome = player.tick(&ground, FORWARD).expect("a tick");
            assert_eq!(outcome.substeps, 3, "tick {tick}");
            assert!(player.state().grounded, "tick {tick}");
        }
        let end = player.state();
        let mu_g = 0.6_f64 * 9.806_65;
        let per_tick = (4.3 + (4.3 - mu_g / 60.0) + (4.3 - 2.0 * mu_g / 60.0)) / 60.0;
        let walked = start.feet.z - end.feet.z;
        assert!(
            (walked - 20.0 * per_tick).abs() < 1e-9,
            "{walked} against {}",
            20.0 * per_tick
        );
        assert!((walked - 4.201_933_5).abs() < 1e-6, "{walked}");
        assert_eq!(end.feet.x.to_bits(), start.feet.x.to_bits());
        assert_eq!(end.feet.y - start.feet.y, 0.0);
    }

    #[test]
    fn diagonal_walking_is_no_faster() {
        let (ground, mut straight, _) = on_flat_ground();
        let mut diagonal = straight.clone();
        let (a, b) = (straight.state(), diagonal.state());
        let both = Walk {
            right: 1.0,
            ..FORWARD
        };
        for _ in 0..10 {
            straight.tick(&ground, FORWARD).expect("a tick");
            diagonal.tick(&ground, both).expect("a tick");
        }
        let distance = |from: PlayerState, to: PlayerState| {
            (to.feet.x - from.feet.x).hypot(to.feet.z - from.feet.z)
        };
        let (s, d) = (distance(a, straight.state()), distance(b, diagonal.state()));
        assert!((s - d).abs() < 1e-9, "straight {s}, diagonal {d}");
        assert!(s > 2.0, "{s}");
    }

    #[test]
    fn releasing_the_keys_stops_within_a_tick_and_stays_still() {
        let (ground, mut player, _) = on_flat_ground();
        for _ in 0..5 {
            player.tick(&ground, FORWARD).expect("a tick");
        }
        let walking = player.state();
        player.tick(&ground, Walk::default()).expect("a tick");
        let released = player.state();
        let slid = (released.feet.x - walking.feet.x).hypot(released.feet.z - walking.feet.z);
        assert!(slid < 1e-12, "slid {slid} after release");
        for tick in 0..1_000 {
            player.tick(&ground, Walk::default()).expect("a tick");
            assert!(
                player.state().is_bit_identical(&released),
                "tick {tick}: {:?}",
                player.state()
            );
        }
    }

    #[test]
    fn turning_is_about_plus_y_and_forward_follows_yaw() {
        let (ground, mut player, _) = on_flat_ground();
        let start = player.state();
        let turn = Walk {
            turn: 1.0,
            ..Walk::default()
        };
        for _ in 0..20 {
            player.tick(&ground, turn).expect("a tick");
            let now = player.state();
            assert_eq!(now.feet.x.to_bits(), start.feet.x.to_bits());
            assert_eq!(now.feet.y.to_bits(), start.feet.y.to_bits());
            assert_eq!(now.feet.z.to_bits(), start.feet.z.to_bits());
        }
        let turned = player.state();
        assert!((turned.yaw - FRAC_PI_2).abs() < 1e-9, "{}", turned.yaw);
        player.tick(&ground, FORWARD).expect("a tick");
        let moved = player.state();
        assert!(
            moved.feet.x - turned.feet.x < 0.0,
            "a quarter turn left looks down -X"
        );
        assert!((moved.feet.z - turned.feet.z).abs() < 1e-12);
    }

    #[test]
    fn pitch_is_clamped_and_does_not_change_walking() {
        let (ground, level, _) = on_flat_ground();
        let mut looking = level.clone();
        let up = Walk {
            look: 1.0,
            ..Walk::default()
        };
        for _ in 0..100 {
            looking.tick(&ground, up).expect("a tick");
        }
        assert_eq!(looking.state().pitch.to_bits(), MAX_PITCH.to_bits());
        assert_eq!(looking.eye().pitch.to_bits(), MAX_PITCH.to_bits());

        let mut level = level;
        let (a, b) = (level.state(), looking.state());
        level.tick(&ground, FORWARD).expect("a tick");
        looking.tick(&ground, FORWARD).expect("a tick");
        let (da, db) = (
            level.state().feet.z - a.feet.z,
            looking.state().feet.z - b.feet.z,
        );
        assert_eq!(da.to_bits(), db.to_bits(), "{da} level, {db} looking up");
    }

    #[test]
    fn a_one_block_rise_is_climbed_and_a_two_block_rise_stops_the_walk() {
        let terrain = Stepped;
        let run = Run {
            x: 0,
            z: 0,
            facing: Cardinal::NegZ,
            feet_y: 0,
            length: 6,
            end_plane: -6,
        };
        let mut player = Player::spawn(&terrain, &run, 0.0, TPS).expect("spawns");
        let start = player.state();
        let mut highest = start.feet.y;
        for tick in 0..80 {
            player
                .tick(&terrain, FORWARD)
                .unwrap_or_else(|error| panic!("tick {tick}: {error}"));
            let state = player.state();
            let leading_face = state.feet.z - 0.3;
            assert!(
                leading_face >= -6.0 - 1e-9,
                "tick {tick}: the leading face passed the wall at {leading_face}"
            );
            highest = highest.max(state.feet.y);
        }
        let end = player.state();
        assert!(end.grounded);
        // One block, to the representation: the solver keeps the body's
        // centre, 1.9 on the step, and 1.9 - 0.9 is 0.9999999999999999 in
        // binary — the case `nexora_physics::math::CONTACT_EPSILON` exists
        // for. The feet are reported as the solver holds them, not snapped.
        let climbed = end.feet.y - start.feet.y;
        assert!((climbed - 1.0).abs() < 1e-12, "climbed {climbed}");
        assert_eq!(highest, end.feet.y, "never above the step it climbed");
        assert!(
            (end.feet.z - 0.3 - (-6.0)).abs() < 1e-9,
            "flush with the wall: {}",
            end.feet.z - 0.3
        );
    }

    #[test]
    fn a_jump_rises_about_one_block_only_from_the_ground() {
        let (ground, mut player, _) = on_flat_ground();
        let start = player.state();
        let jump = Walk {
            jump: true,
            ..Walk::default()
        };
        player.tick(&ground, jump).expect("the jump");
        let mut apex = player.state().feet.y;
        let mut landed = None;
        let mut previous = player.state();
        for tick in 0..60 {
            // Still holding jump: a body in the air must not be lifted again.
            player.tick(&ground, jump).expect("a tick");
            let now = player.state();
            if !previous.grounded && !now.grounded {
                assert!(
                    now.velocity[1] <= previous.velocity[1],
                    "tick {tick}: rose from {} to {} in the air",
                    previous.velocity[1],
                    now.velocity[1]
                );
            }
            apex = apex.max(now.feet.y);
            if now.grounded {
                landed = Some(now);
                break;
            }
            previous = now;
        }
        let rise = apex - start.feet.y;
        assert!((1.10..=1.35).contains(&rise), "rose {rise}");
        let landed = landed.expect("the player came back down");
        assert_eq!(landed.feet.y.to_bits(), start.feet.y.to_bits());
        assert_eq!(landed.feet.x.to_bits(), start.feet.x.to_bits());
        assert_eq!(landed.feet.z.to_bits(), start.feet.z.to_bits());
    }

    #[test]
    fn an_out_of_range_walk_asks_for_no_more_than_full_deflection() {
        let (ground, player, _) = on_flat_ground();
        let script = |walk: Walk| {
            let mut player = player.clone();
            for _ in 0..10 {
                player.tick(&ground, walk).expect("a tick");
            }
            player.state()
        };
        let full = script(Walk {
            turn: 1.0,
            forward: 1.0,
            ..Walk::default()
        });
        let fifty = script(Walk {
            turn: 50.0,
            forward: 1.0e300,
            ..Walk::default()
        });
        assert!(fifty.is_bit_identical(&full));
        let nothing = script(Walk::default());
        let not_a_number = script(Walk {
            forward: f64::NAN,
            right: f64::INFINITY,
            turn: f64::NEG_INFINITY,
            look: f64::NAN,
            jump: false,
        });
        assert!(not_a_number.is_bit_identical(&nothing));
    }

    #[test]
    fn the_same_walks_give_the_same_player_bit_for_bit() {
        let (ground, first, _) = on_flat_ground();
        let mut second = first.clone();
        let mut first = first;
        for tick in 0..200_u32 {
            let walk = Walk {
                forward: f64::from(tick % 3) - 1.0,
                right: if tick % 7 < 3 { 1.0 } else { 0.0 },
                turn: if tick % 11 < 4 { 1.0 } else { -0.5 },
                look: if tick % 13 < 6 { 1.0 } else { -1.0 },
                jump: tick % 17 == 0,
            };
            first.tick(&ground, walk).expect("a tick");
            second.tick(&ground, walk).expect("a tick");
            assert!(
                first.state().is_bit_identical(&second.state()),
                "tick {tick}"
            );
            assert_eq!(first.eye(), second.eye());
        }
    }

    #[test]
    fn the_eye_is_eye_height_above_the_feet() {
        let (_, player, _) = on_flat_ground();
        let (eye, state) = (player.eye(), player.state());
        assert!((eye.position.y - state.feet.y - EYE_HEIGHT).abs() < 1e-12);
        assert_eq!(eye.position.x, state.feet.x);
        assert_eq!(eye.position.z, state.feet.z);
    }

    #[test]
    fn a_spawn_inside_terrain_or_at_a_tick_rate_physics_cannot_keep_is_refused() {
        let (ground, area) = flat();
        let run = find_walkable_run(&ground, &area, 1).expect("a run");
        let buried = Run { feet_y: -1, ..run };
        let error = Player::spawn(&ground, &buried, 0.0, TPS).expect_err("inside the floor");
        assert_eq!(error.recovery(), Recovery::Reject);
        // 7 ticks a second is not a whole number of 60 Hz substeps.
        assert!(Player::spawn(&ground, &run, 0.0, 7).is_err());
        // One tick a second would be 60 substeps against a cap of 8.
        assert!(Player::spawn(&ground, &run, 0.0, 1).is_err());
        assert!(Player::spawn(&ground, &run, f64::NAN, TPS).is_err());
    }
}
