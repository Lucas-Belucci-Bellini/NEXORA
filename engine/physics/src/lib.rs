//! # NEXORA physics
//!
//! Fixed-step rigid bodies, swept voxel collision, character control and
//! spatial queries. Implements the first stage of `PHYSICS.md` — items PHY-0
//! through PHY-14 and PHY-22, in that document's own order (§47).
//!
//! ```text
//! world time
//!   -> fixed timestep accumulator
//!   -> gravity and accumulated forces
//!   -> integrate velocity, then position
//!   -> sweep the box against the voxel grid, one axis at a time
//!   -> step up over a low obstacle, if the body walks
//!   -> stop or bounce, detect ground, shed speed to friction
//!   -> settle and sleep
//! ```
//!
//! ## What physics is allowed to know
//!
//! `PHYSICS.md` closes with the rule this crate is built around: physics knows
//! **mass, position, velocity, collision and forces**, and never *"this is a
//! sword"* or *"this is a villager"*. That is why `nexora-physics` depends on
//! `nexora-foundation` alone. Terrain arrives through the [`voxel::VoxelSource`]
//! trait rather than through `nexora-world`, so the same solver runs against
//! the real world, a flat test floor and a synthetic benchmark terrain without
//! any of the three knowing about the others — and Cargo enforces it, because
//! the dependency simply is not there to reach through.
//!
//! ## Determinism
//!
//! The accumulator is integer arithmetic over world ticks, bodies are stepped
//! in slot order, no container iterated here is unordered, and nothing reads a
//! wall clock. `NEXORA REPLAY AND DETERMINISM.md` asks for the same inputs to
//! produce the same state; [`world::PhysicsWorld`] carries a test that steps
//! two identically-built worlds and compares every body.
//!
//! ## Bounds on untrusted input
//!
//! `NEXORA SECURITY THREAT MODEL.md` treats mods, scripts and save files as
//! untrusted, and a velocity, an extent or a ray length is exactly the kind of
//! number that arrives from them. Every one of them is bounded, and the bound
//! is a named constant with the reason attached:
//! [`collision::MAX_SWEEP_LAYERS`], [`query::MAX_RAY_CELLS`],
//! [`body::MAX_HALF_EXTENT`], [`body::MAX_SPEED_LIMIT`],
//! [`gravity::MAX_GRAVITY_MAGNITUDE`], [`world::MAX_BODIES`] and the substep
//! cap in [`step::FixedStep`]. Non-finite values are refused at the boundary
//! rather than integrated into a position that then disappears.
//!
//! ## What is deliberately not here
//!
//! Reading this list is faster than discovering the gaps by experiment, and
//! each one is in the technical debt register with the trigger that closes it:
//!
//! * **Body-versus-body collision.** Every contact is body-against-voxel.
//!   Vehicles, stacked falling blocks and a platform carrying a passenger all
//!   need the pair solver, and none of them work yet.
//! * **Rotation.** No angular velocity, no inertia tensor. Boxes against cubes
//!   give a rotation nothing to act on, and carrying unread state would be a
//!   mock in the sense the startup brief §43 forbids.
//! * **Non-cube shapes.** Slabs, ramps, wedges and stairs are `PHYSICS.md`
//!   §13; [`voxel::VoxelShape`] is `non_exhaustive` because they are variants
//!   of it rather than a redesign.
//! * **Buoyancy, vehicles, structural analysis and destruction.** All named in
//!   `PHYSICS.md`, all later items in its own build order.

pub mod body;
pub mod character;
pub mod collision;
pub mod gravity;
pub mod material;
pub mod math;
pub mod query;
pub mod step;
pub mod voxel;
pub mod world;

pub use body::{BodyDescriptor, BodyId, BodyType, RigidBody, SleepState};
pub use character::{CharacterController, GroundState, MoveIntent};
pub use collision::{resolve, sweep_axis, AxisSweep, Contact, Resolution};
pub use gravity::GravityField;
pub use material::{MaterialId, MaterialTable, PhysicsMaterial};
pub use math::{Aabb, Axis, Vec3};
pub use query::{raycast, RayHit};
pub use step::{FixedStep, StepPlan};
pub use voxel::{VoxelShape, VoxelSource};
pub use world::{PhysicsWorld, StepReport, StepStats};
