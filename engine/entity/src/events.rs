//! Entity facts published on the event bus.
//!
//! Facts, not commands (`NEXORA ARCHITECTURE RULES.md` §4): these say what
//! already happened. Nothing here can be vetoed by a listener.

use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::WorldPosition;
use nexora_runtime::events::Event;

use crate::id::{EntityId, EntityTypeId};
use crate::store::SpawnReason;

/// An entity now exists in the world.
#[derive(Debug, Clone)]
pub struct EntitySpawned {
    /// The new entity.
    pub entity: EntityId,
    /// What kind of entity it is.
    pub entity_type: EntityTypeId,
    /// Where it appeared.
    pub position: WorldPosition,
    /// Why it was created.
    pub reason: SpawnReason,
}

impl Event for EntitySpawned {
    fn event_type(&self) -> Identifier {
        Identifier::parse("nexora:event/entity_spawned").expect("static identifier")
    }
}

/// An entity has been removed from the world.
#[derive(Debug, Clone)]
pub struct EntityDespawned {
    /// The handle, now stale.
    pub entity: EntityId,
    /// What kind of entity it was.
    pub entity_type: EntityTypeId,
    /// Where it was when it went.
    pub position: WorldPosition,
}

impl Event for EntityDespawned {
    fn event_type(&self) -> Identifier {
        Identifier::parse("nexora:event/entity_despawned").expect("static identifier")
    }
}
