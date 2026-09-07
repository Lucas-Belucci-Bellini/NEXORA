//! # NEXORA Entity Foundation
//!
//! Identity, lifecycle, components, queries and persistence for the things that
//! live in a world. Implements `Entity System.md`.
//!
//! ## What this layer is
//!
//! `Entity System.md` §1: *"Entity System define quem existe no mundo, onde
//! está, qual é seu ciclo de vida e quais componentes possui"* — and explicitly
//! **not** combat, AI, physics, inventory or rendering. Those consume this; they
//! do not live here.
//!
//! ## Identity, twice over
//!
//! [`EntityId`] is a session handle: a slot plus a generation, cheap to copy,
//! and stale the moment the entity dies. [`PersistentEntityId`] is what a save
//! stores. The separation is the same one the block registry makes between a
//! runtime id and a namespaced identifier, for the same reason — a slot index
//! means nothing outside the process that assigned it.
//!
//! ## Not an ECS
//!
//! Components live in dense parallel columns, which is cache-friendly to
//! iterate, but there are no archetypes, no query planner and no system
//! scheduler. `ECS AND DATA ORIENTED RUNTIME.md` says the data-oriented runtime
//! must *complement* the public entity API rather than replace it, and that
//! storage layout is an implementation detail. See
//! `docs/adr/ADR-0006-entity-identity-and-storage.md`.
//!
//! ## Layer
//!
//! Depends on Foundation, Runtime and Persistence — **not** on the world crate.
//! Entities and the voxel world are siblings composed by the layer above, and
//! Cargo enforces that.

pub mod components;
pub mod events;
pub mod id;
pub mod persist;
pub mod query;
pub mod store;

pub use components::{Bounds, Lifecycle, PersistencePolicy, TagSet, Transform, Velocity};
pub use events::{EntityDespawned, EntitySpawned};
pub use id::{EntityId, EntityTypeId, PersistentEntityId};
pub use query::{EntityFilter, Nearby, Query};
pub use store::{EntityStore, SpawnContext, SpawnReason};
