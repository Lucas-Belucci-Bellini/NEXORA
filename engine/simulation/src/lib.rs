//! # NEXORA simulation layer
//!
//! The composition point. `NEXORA DEPENDENCY MATRIX.md` puts Simulation above
//! World and allows it to depend on it; `PHYSICS.md` requires physics to reach
//! terrain through a provider rather than through the world runtime. Both are
//! satisfied by putting the one legal bridge here:
//!
//! ```text
//! nexora-world  ──┐
//!                 ├──► nexora-simulation ──► the slice, the benchmark, the server
//! nexora-physics ─┘
//! ```
//!
//! `nexora-physics` does not depend on `nexora-world`, and `nexora-world` does
//! not depend on `nexora-physics`. Neither can, because the dependency is not
//! declared — Cargo enforces the layering rather than a reviewer noticing it.

pub mod terrain;

pub use terrain::{PhysicsModule, WorldVoxels};
