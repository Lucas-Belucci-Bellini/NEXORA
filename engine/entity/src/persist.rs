//! Entity persistence.
//!
//! Implements `Entity System.md` §9-§11 (persistence policy) against the save
//! container.
//!
//! Two rules from the architecture drive the format:
//!
//! * `ECS AND DATA ORIENTED RUNTIME.md`: **"Persistence serializes logical
//!   state, not storage layout internals."** Nothing about slots, generations or
//!   column order reaches the disk.
//! * `Entity System.md` §9: a `TEMPORARY` entity is not written at all. A
//!   particle must not come back after a reload.
//!
//! Entities are restored with **new** session handles and their **original**
//! persistent ids, exactly as chunk palettes are remapped through identifiers.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::WorldPosition;
use nexora_persistence::codec::{Reader, Writer};
use nexora_persistence::SaveContainer;

use crate::components::{Bounds, PersistencePolicy, TagSet, Transform, Velocity};
use crate::id::{EntityTypeId, PersistentEntityId};
use crate::store::{EntityStore, SpawnContext, SpawnReason};

/// Save section holding entity state.
pub const SECTION_ENTITIES: &str = "nexora:save/entities";

/// Encode a store's saveable entities into a save section.
///
/// # Errors
///
/// Returns an error when the section identifier cannot be built.
pub fn save(store: &EntityStore, container: &mut SaveContainer) -> Result<usize> {
    let mut body = Writer::new();

    // Written first so a reload cannot hand out an id a saved entity already
    // holds.
    body.u64(store.next_persistent_id());

    let saveable: Vec<_> = store
        .live_slots()
        .filter_map(|slot| store.slot_view(slot))
        .filter(|view| view.policy.is_saved())
        .collect();

    body.u32(saveable.len() as u32);
    for view in &saveable {
        body.u64(view.persistent.0);
        body.string(&view.entity_type.to_string());
        body.f64(view.transform.position.x);
        body.f64(view.transform.position.y);
        body.f64(view.transform.position.z);
        body.f32(view.transform.yaw);
        body.f32(view.transform.pitch);
        body.f64(view.velocity.dx);
        body.f64(view.velocity.dy);
        body.f64(view.velocity.dz);
        body.f64(view.bounds.half_x);
        body.f64(view.bounds.half_y);
        body.f64(view.bounds.half_z);
        body.u8(view.policy.to_tag());
        body.u32(view.tags.len() as u32);
        for tag in view.tags.as_slice() {
            body.string(&tag.to_string());
        }
    }

    let count = saveable.len();
    container.put(Identifier::parse(SECTION_ENTITIES)?, body.finish());
    Ok(count)
}

/// Restore entities from a save section into a store.
///
/// A container with no entity section loads as an empty population rather than
/// an error: a world saved before entities existed is still a valid world.
///
/// # Errors
///
/// Returns an error when the section is malformed or names invalid content.
pub fn load(store: &mut EntityStore, container: &SaveContainer) -> Result<usize> {
    let section = Identifier::parse(SECTION_ENTITIES)?;
    let Some(bytes) = container.get(&section) else {
        return Ok(0);
    };

    let mut reader = Reader::new(bytes);
    store.restore_persistent_counter(reader.u64()?);

    let count = reader.u32()?;
    let mut restored = 0usize;

    for index in 0..count {
        let persistent = PersistentEntityId(reader.u64()?);
        let entity_type = EntityTypeId::parse(reader.string()?).map_err(|cause| {
            malformed("saved entity names an invalid entity type")
                .with_context("index", index.to_string())
                .with_source(cause)
        })?;

        let position = WorldPosition::new(reader.f64()?, reader.f64()?, reader.f64()?);
        let transform = Transform {
            position,
            yaw: reader.f32()?,
            pitch: reader.f32()?,
        }
        .validate()
        .map_err(|cause| {
            malformed("saved entity has an invalid transform")
                .with_context("persistent_id", persistent.to_string())
                .with_source(cause)
        })?;

        let velocity =
            Velocity::new(reader.f64()?, reader.f64()?, reader.f64()?).map_err(|cause| {
                malformed("saved entity has an invalid velocity")
                    .with_context("persistent_id", persistent.to_string())
                    .with_source(cause)
            })?;

        let bounds = Bounds::new(reader.f64()?, reader.f64()?, reader.f64()?).map_err(|cause| {
            malformed("saved entity has invalid bounds")
                .with_context("persistent_id", persistent.to_string())
                .with_source(cause)
        })?;

        let policy = PersistencePolicy::from_tag(reader.u8()?)?;
        if !policy.is_saved() {
            // A temporary entity is never written, so finding one means the file
            // disagrees with the rules that produced it.
            return Err(
                malformed("save contains an entity whose policy forbids saving")
                    .with_context("persistent_id", persistent.to_string())
                    .with_context("policy", policy.as_str()),
            );
        }

        let tag_count = reader.u32()?;
        let mut tags = TagSet::new();
        for _ in 0..tag_count {
            tags.insert(Identifier::parse(reader.string()?)?);
        }

        let context = SpawnContext {
            entity_type,
            transform,
            velocity,
            bounds,
            policy,
            tags,
            reason: SpawnReason::Load,
        };
        store.spawn_with_persistent_id(context, Some(persistent))?;
        restored += 1;
    }

    reader.expect_exhausted()?;
    Ok(restored)
}

fn malformed(message: &'static str) -> Error {
    Error::new(Domain::Save, "entity-persist", message).with_recovery(Recovery::Quarantine)
}
