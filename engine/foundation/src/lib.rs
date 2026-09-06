//! # NEXORA Foundation
//!
//! The lowest layer of the NEXORA engine. Per `NEXORA DEPENDENCY MATRIX.md` the
//! Foundation may depend on platform abstractions and nothing else, so this
//! crate has **no dependencies at all** - not the standard library's unstable
//! hashers, not third-party serialization, nothing whose representation could
//! drift underneath a save file. See
//! `docs/adr/ADR-0002-zero-dependency-foundation.md`.
//!
//! ## What lives here
//!
//! | Module | Contract document |
//! |---|---|
//! | [`error`] | `CORE.md` §14 - error taxonomy with owner and recovery path |
//! | [`version`] | `NEXORA SAVE FORMAT AND COMPATIBILITY.md` - versioned contracts |
//! | [`ident`] | `NEXORA NAMING AND TERMINOLOGY.md` - `namespace:path` identifiers |
//! | [`hashing`] | Stable digests for fingerprints and integrity |
//! | [`spatial`] | `SPATIAL AND COORDINATE SYSTEM.md` - coordinate spaces |
//! | [`time`] | `TIME AND CALENDAR SYSTEM.md` - the authoritative world clock |
//! | [`rng`] | `NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md` - determinism |
//! | [`diagnostics`] | `DIAGNOSTICS AND OBSERVABILITY.md` - structured telemetry |
//! | [`config`] | `CONFIGURATION AND SETTINGS SYSTEM.md` - layered settings |
//!
//! ## What does not live here
//!
//! No gameplay, no world content, no rendering. `NEXORA ARCHITECTURE RULES.md`
//! §1: the Core provides rules and contracts; content must not reach into it.

pub mod config;
pub mod diagnostics;
pub mod error;
pub mod hashing;
pub mod ident;
pub mod rng;
pub mod spatial;
pub mod time;
pub mod version;

pub use error::{Domain, Error, Recovery, Result, Severity};
pub use ident::{Identifier, Namespace, WorldId};
pub use spatial::{
    BlockPos, ChunkCoord, ChunkShape, LocalPos, RegionCoord, SectionCoord, WorldPosition,
};
pub use time::{CalendarConfig, WorldClock, WorldDuration, WorldTime};
pub use version::{EngineVersion, VersionSet, ENGINE_VERSION};
