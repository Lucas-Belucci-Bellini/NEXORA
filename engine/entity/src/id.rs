//! Entity identity.
//!
//! Implements `Entity System.md` §2 (ENTITY-1) and §3 (ENTITY-2).
//!
//! Two identities, deliberately:
//!
//! * [`EntityId`] is a **session handle** — a slot index plus a generation. It
//!   is cheap to copy and compare, and it goes stale the instant the entity is
//!   destroyed.
//! * [`PersistentEntityId`] is what a **save** stores.
//!
//! This is the same separation the block registry already makes between a
//! runtime id and a namespaced identifier, and it exists for the same reason.
//! A slot index is meaningful only inside the process that assigned it; writing
//! one to disk means a reload can hand entity A's history to entity B.

use core::fmt;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::{Identifier, WorldId};

/// A session-local handle to a live entity.
///
/// The `generation` is what makes a stale handle detectable. When an entity is
/// destroyed its slot's generation advances, so every handle issued before that
/// point stops resolving — satisfying the invariant in
/// `NEXORA DATA VALIDATION AND INVARIANTS.md` that *a destroyed object cannot
/// remain addressable as active state*.
///
/// Never serialize one. Use [`PersistentEntityId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId {
    world: WorldId,
    index: u32,
    generation: u32,
}

impl EntityId {
    /// Construct a handle. Normally produced by the store, not by callers.
    #[must_use]
    pub const fn new(world: WorldId, index: u32, generation: u32) -> Self {
        Self {
            world,
            index,
            generation,
        }
    }

    /// The world this entity belongs to.
    #[must_use]
    pub const fn world(self) -> WorldId {
        self.world
    }

    /// Slot index within the store.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// Generation of the slot when this handle was issued.
    #[must_use]
    pub const fn generation(self) -> u32 {
        self.generation
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "entity:{:016x}/{}#{}",
            self.world.0, self.index, self.generation
        )
    }
}

/// The identity a save stores.
///
/// Allocated from a per-world counter that is itself persisted, so ids stay
/// unique across sessions and are reproducible under replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersistentEntityId(pub u64);

impl fmt::Display for PersistentEntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pid:{:016x}", self.0)
    }
}

/// What kind of thing an entity is, e.g. `nexora:entity/test`.
///
/// `Entity System.md` §3 separates the *type* from the *instance*: `nexora:deer`
/// is a type; the animal standing in a forest is an instance of it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityTypeId(Identifier);

impl EntityTypeId {
    /// Wrap an identifier as an entity type.
    #[must_use]
    pub const fn new(id: Identifier) -> Self {
        Self(id)
    }

    /// Parse the canonical `namespace:path` form.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier is malformed.
    pub fn parse(raw: &str) -> Result<Self> {
        Ok(Self(Identifier::parse(raw)?))
    }

    /// The underlying identifier.
    #[must_use]
    pub const fn identifier(&self) -> &Identifier {
        &self.0
    }
}

impl fmt::Display for EntityTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Build the error returned when a handle does not resolve.
pub(crate) fn stale_handle(id: EntityId, reason: &'static str) -> Error {
    Error::new(
        Domain::World,
        "entity-store",
        "entity handle does not resolve",
    )
    .with_recovery(Recovery::Reject)
    .with_context("entity", id.to_string())
    .with_context("reason", reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_carry_world_slot_and_generation() {
        let world = WorldId::derive("test", 1);
        let id = EntityId::new(world, 7, 3);
        assert_eq!(id.world(), world);
        assert_eq!(id.index(), 7);
        assert_eq!(id.generation(), 3);
        assert!(id.to_string().contains("#3"));
    }

    #[test]
    fn handles_from_different_generations_are_distinct() {
        let world = WorldId::derive("test", 1);
        assert_ne!(EntityId::new(world, 7, 1), EntityId::new(world, 7, 2));
    }

    #[test]
    fn handles_from_different_worlds_are_distinct() {
        let a = EntityId::new(WorldId::derive("a", 1), 0, 0);
        let b = EntityId::new(WorldId::derive("b", 1), 0, 0);
        assert_ne!(a, b);
    }

    #[test]
    fn entity_types_are_namespaced_identifiers() {
        let deer = EntityTypeId::parse("nexora:entity/deer").expect("valid");
        assert_eq!(deer.to_string(), "nexora:entity/deer");
        assert!(deer.identifier().namespace().is_first_party());
        assert!(EntityTypeId::parse("not a type").is_err());
    }
}
