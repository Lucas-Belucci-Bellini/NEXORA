//! # NEXORA World Runtime
//!
//! Voxel storage, chunk lifecycle, world identity and deterministic generation.
//!
//! | Module | Contract document |
//! |---|---|
//! | [`voxel`] | `CHUNK & VOXEL ENGINE.md` §5-§10 - palette-compressed section storage |
//! | [`chunk`] | `CHUNK & VOXEL ENGINE.md` §13-§14, §33-§34 - lifecycle, dirty tracking, journal |
//! | [`world`] | `NEXORA WORLD STATE LIFECYCLE.md`, seed reproducibility |
//! | [`persist`] | `NEXORA SAVE FORMAT AND COMPATIBILITY.md` - identifier-based world saves |
//!
//! The world owns authoritative voxel state. Presentation reads it; nothing
//! outside this crate mutates it, per `NEXORA DEPENDENCY MATRIX.md`.

pub mod chunk;
pub mod persist;
pub mod voxel;
pub mod world;

pub use chunk::{Chunk, ChunkState, VoxelChange};
pub use voxel::{BlockStateId, Section, AIR};
pub use world::{World, WorldDescriptor, WorldId, WorldPhase};
