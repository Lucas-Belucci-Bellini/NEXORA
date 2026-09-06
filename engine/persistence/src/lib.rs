//! # NEXORA Persistence
//!
//! The versioned, checksummed, atomically written save container described by
//! `NEXORA SAVE FORMAT AND COMPATIBILITY.md`.
//!
//! This crate knows nothing about chunks, entities or civilizations. It moves
//! named, verified blobs to and from disk; the systems that own that state
//! decide what goes inside. That separation is what keeps the dependency
//! direction in `NEXORA DEPENDENCY MATRIX.md` intact - persistence sits beside
//! the world rather than above it.

pub mod codec;
pub mod container;

pub use codec::{Reader, Writer};
pub use container::{quarantine, SaveContainer};
