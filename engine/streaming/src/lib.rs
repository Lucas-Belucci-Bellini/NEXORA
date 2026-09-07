//! # NEXORA streaming
//!
//! Interest, priority, budgets, level-of-detail tiers and eviction. Implements
//! `STREAMING SYSTEM.md`'s pipeline:
//!
//! ```text
//! Interest → Priority → Request → Load/Generate → Validate
//!          → Activate → Simulate → Deactivate → Persist → Evict
//! ```
//!
//! ## What streaming is not
//!
//! The document's own first line draws the boundary:
//!
//! > Streaming controls which world, entity, resource and simulation data is
//! > resident. **It does not own generation or gameplay rules.**
//!
//! So `nexora-streaming` depends on `nexora-foundation` and nothing else.
//! Making something resident happens through the [`backend::ResidencyBackend`]
//! trait, which means the manager *cannot* generate a chunk — the dependency
//! is not there to reach through. The same boundary `nexora-physics` has, for
//! the same reason, and Cargo enforces both.
//!
//! ## The invariant that outranks the rest
//!
//! `WORLD CONTINUITY AND PLAYER INDEPENDENCE.md` §18: **logical identity and
//! persistent state survive eviction.** A tier says how much of a thing is
//! loaded, never whether it still exists. Concretely: the manager persists a
//! dirty target before dropping it, and if that write fails it **does not
//! evict** — holding memory is recoverable, losing an edit is not.
//!
//! ## Two ways the budget can bite
//!
//! Reported separately, because they mean different things:
//!
//! * **Deferred** — the per-tick activation or eviction budget ran out. The
//!   work is still queued and happens next tick, nearest-first.
//! * **Shed** — memory or resident-count pressure evicted something interest
//!   actively wanted. Non-zero `shed` means the budget is smaller than what the
//!   observer is asking for, which is a fact worth surfacing rather than
//!   absorbing.
//!
//! ## Determinism
//!
//! Every container is a `BTreeMap` or `BTreeSet`, every sort carries an
//! explicit tie-break, and nothing reads a clock. The same interest, budget and
//! backend produce the same decisions in the same order.
//!
//! ## What is deliberately not here
//!
//! Only chunks stream. `STREAMING SYSTEM.md` also lists regions, entities,
//! structures, resources, GPU assets, dimensions and simulation contexts;
//! [`target::StreamTarget`] is `#[non_exhaustive]` because those are variants
//! of it rather than a different mechanism. The `Regional` and `Abstract` tiers
//! are real states of the manager but hold no distinct data yet — the regional
//! simulation that would fill them is Phase 4. Both gaps are in the technical
//! debt register with the trigger that closes them.

pub mod backend;
pub mod budget;
pub mod interest;
pub mod lod;
pub mod system;
pub mod target;

pub use backend::{MemoryBackend, ResidencyBackend};
pub use budget::{StreamingBudget, StreamingReport};
pub use interest::{InterestId, InterestSource, LodRadii};
pub use lod::Lod;
pub use system::{StreamingSystem, TargetState};
pub use target::{StreamHandle, StreamReason, StreamTarget};
