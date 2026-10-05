//! # NEXORA simulation layer
//!
//! The composition point. `NEXORA DEPENDENCY MATRIX.md` puts Simulation above
//! World and allows it to depend on it; `PHYSICS.md` requires physics to reach
//! terrain through a provider rather than through the world runtime. Both are
//! satisfied by putting the one legal bridge here:
//!
//! ```text
//! nexora-world  ──┐
//!                 ├──► nexora-simulation ──► the slice, the client, the benchmark, the server
//! nexora-physics ─┘
//! ```
//!
//! `nexora-physics` does not depend on `nexora-world`, and `nexora-world` does
//! not depend on `nexora-physics`. Neither can, because the dependency is not
//! declared — Cargo enforces the layering rather than a reviewer noticing it.
//!
//! The player lives here for the same reason: a body the simulation steers
//! (ADR-0035) is input intent, physics and the world at once, and this is the
//! one crate that sees all three. It hands out foundation types only, so the
//! client, the slice and the benchmark drive it without naming a physics type.
//! Its hands do too (ADR-0036): the interaction ray is a physics query and its
//! result a world command, and both meet here.

pub mod commands;
pub mod content;
pub mod controls;
pub mod interaction;
pub mod player;
pub mod queries;
pub mod residency;
pub mod spawn;
pub mod surfaces;
pub mod terrain;

pub use commands::{BreakBlockHandler, PlaceBlockHandler};
pub use content::{BlockContent, ContentBlock};
pub use controls::{Controls, Intent, DEFAULT_BUTTONS, DEFAULT_KEYS};
pub use interaction::{Action, Attempt, Edit, Interaction, Stance, Target};
pub use player::{Eye, Player, PlayerState, TickOutcome, Walk};
pub use queries::WorldQueries;
pub use residency::{FlushReport, RetainedChunks, WorldResidency};
pub use spawn::{find_walkable_run, Cardinal, ColumnArea, Run};
pub use surfaces::{SurfaceTable, SurfaceTableBuilder, WorldSurfaces, UNMAPPED_SURFACE};
pub use terrain::{PhysicsModule, WorldVoxels};
