//! # NEXORA Runtime Services
//!
//! The layer above Foundation and below World, per `NEXORA DEPENDENCY MATRIX.md`.
//! It provides the machinery that lets systems exist and talk to each other,
//! and contains no gameplay of its own.
//!
//! | Module | Contract document |
//! |---|---|
//! | [`lifecycle`] | `NEXORA RUNTIME LIFECYCLE.md` - phase ordering as a checked state machine |
//! | [`module`] | `ENGINE MODULE SYSTEM.md` - trusted engine modules and their graph |
//! | [`registry`] | `Registry System.md` - namespaced content and fingerprints |
//! | [`events`] | `Event Bus.md` - facts, correlation, causation, loop protection |
//! | [`jobs`] | `JOB SYSTEM.md` - a bounded worker pool with priorities |
//!
//! `NEXORA ARCHITECTURE RULES.md` §4 governs the vocabulary used here:
//! a command is an intention, an event is a fact that already happened, and a
//! query is a read. [`events`] carries facts only.

pub mod events;
pub mod jobs;
pub mod lifecycle;
pub mod module;
pub mod registry;

pub use events::{Event, EventBus, EventContext, EventId};
pub use jobs::{JobHandle, JobOutcome, JobSystem};
pub use lifecycle::{Lifecycle, Phase, RuntimeMode, State};
pub use module::{EngineModule, ModuleContext, ModuleId, ModuleManager};
pub use registry::{Registry, RegistryEntry, RuntimeId};
