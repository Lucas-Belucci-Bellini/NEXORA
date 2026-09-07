//! The entity store.
//!
//! Implements `Entity System.md` §1 (ENTITY-0 core), §6-§8 (lifecycle, spawn,
//! despawn) and §13 (component architecture).
//!
//! ## Storage layout is an implementation detail
//!
//! `ECS AND DATA ORIENTED RUNTIME.md` states that as an invariant, and that the
//! data-oriented runtime *complements* the public entity API rather than
//! replacing it. So this store keeps components in dense parallel columns
//! indexed by slot — cache-friendly to iterate in batch — while callers only
//! ever see an [`EntityId`]. Swapping in archetypes later changes nothing a
//! caller can observe.
//!
//! It is **not** a full ECS. There are no archetypes, no query planner and no
//! system scheduler. Those are deferred until something measures a need for
//! them; see `docs/adr/ADR-0006-entity-identity-and-storage.md`.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::{Identifier, WorldId};
use nexora_foundation::spatial::WorldPosition;
use nexora_foundation::time::WorldDuration;
use nexora_runtime::events::EventBus;

use crate::components::{Bounds, Lifecycle, PersistencePolicy, TagSet, Transform, Velocity};
use crate::events::{EntityDespawned, EntitySpawned};
use crate::id::{stale_handle, EntityId, EntityTypeId, PersistentEntityId};

/// Why an entity is being created (`Entity System.md` §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SpawnReason {
    /// Placed during world generation.
    WorldGen,
    /// Created by a player action.
    Player,
    /// Produced by a spawner.
    Spawner,
    /// Born from existing entities.
    Breeding,
    /// Required by a quest.
    Quest,
    /// Part of a generated structure.
    Structure,
    /// Created by a mod.
    Mod,
    /// Restored from a save rather than newly created.
    Load,
}

impl SpawnReason {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorldGen => "worldgen",
            Self::Player => "player",
            Self::Spawner => "spawner",
            Self::Breeding => "breeding",
            Self::Quest => "quest",
            Self::Structure => "structure",
            Self::Mod => "mod",
            Self::Load => "load",
        }
    }
}

/// Everything needed to place an entity in the world.
#[derive(Debug, Clone)]
pub struct SpawnContext {
    /// What kind of entity this is.
    pub entity_type: EntityTypeId,
    /// Where it goes and which way it faces.
    pub transform: Transform,
    /// How fast it is moving.
    pub velocity: Velocity,
    /// Its boundary box.
    pub bounds: Bounds,
    /// Whether it survives a save.
    pub policy: PersistencePolicy,
    /// Tags for querying.
    pub tags: TagSet,
    /// Why it is being created.
    pub reason: SpawnReason,
}

impl SpawnContext {
    /// A minimal context: a point entity, still, persistent, untagged.
    #[must_use]
    pub fn new(entity_type: EntityTypeId, position: WorldPosition, reason: SpawnReason) -> Self {
        Self {
            entity_type,
            transform: Transform::at(position),
            velocity: Velocity::STILL,
            bounds: Bounds::POINT,
            policy: PersistencePolicy::Persistent,
            tags: TagSet::new(),
            reason,
        }
    }

    /// Set the boundary box.
    #[must_use]
    pub const fn with_bounds(mut self, bounds: Bounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// Set the persistence policy.
    #[must_use]
    pub const fn with_policy(mut self, policy: PersistencePolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Set the velocity.
    #[must_use]
    pub const fn with_velocity(mut self, velocity: Velocity) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set the tags.
    #[must_use]
    pub fn with_tags(mut self, tags: TagSet) -> Self {
        self.tags = tags;
        self
    }
}

/// Dense per-slot component columns.
#[derive(Debug, Default)]
struct Columns {
    generation: Vec<u32>,
    alive: Vec<bool>,
    persistent: Vec<PersistentEntityId>,
    entity_type: Vec<EntityTypeId>,
    transform: Vec<Transform>,
    velocity: Vec<Velocity>,
    bounds: Vec<Bounds>,
    lifecycle: Vec<Lifecycle>,
    policy: Vec<PersistencePolicy>,
    tags: Vec<TagSet>,
}

impl Columns {
    fn len(&self) -> usize {
        self.generation.len()
    }
}

/// Owns every entity in one world.
pub struct EntityStore {
    world: WorldId,
    columns: Columns,
    free: Vec<u32>,
    retired_slots: usize,
    live: usize,
    next_persistent: u64,
    by_persistent: BTreeMap<PersistentEntityId, u32>,
    bus: Option<EventBus>,
}

impl std::fmt::Debug for EntityStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityStore")
            .field("world", &self.world)
            .field("live", &self.live)
            .field("slots", &self.columns.len())
            .field("free", &self.free.len())
            .finish()
    }
}

impl EntityStore {
    /// Create an empty store for a world.
    #[must_use]
    pub fn new(world: WorldId) -> Self {
        Self {
            world,
            columns: Columns::default(),
            free: Vec::new(),
            retired_slots: 0,
            live: 0,
            next_persistent: 1,
            by_persistent: BTreeMap::new(),
            bus: None,
        }
    }

    /// Attach an event bus, so spawn and despawn publish facts.
    #[must_use]
    pub fn with_event_bus(mut self, bus: EventBus) -> Self {
        self.bus = Some(bus);
        self
    }

    /// The world these entities belong to.
    #[must_use]
    pub const fn world(&self) -> WorldId {
        self.world
    }

    /// How many entities are alive.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.live
    }

    /// Whether no entities are alive.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// Slots allocated, including free ones. Storage detail, exposed for
    /// diagnostics only.
    #[must_use]
    pub fn slot_count(&self) -> usize {
        self.columns.len()
    }

    /// Slots permanently retired because their generation counter was exhausted.
    #[must_use]
    pub const fn retired_slots(&self) -> usize {
        self.retired_slots
    }

    /// The next persistent id that will be handed out.
    #[must_use]
    pub const fn next_persistent_id(&self) -> u64 {
        self.next_persistent
    }

    /// Restore the persistent id counter when loading a save.
    ///
    /// Never lowers it: reusing an id that a saved entity already holds would
    /// make two different entities indistinguishable in the world's history.
    pub fn restore_persistent_counter(&mut self, next: u64) {
        self.next_persistent = self.next_persistent.max(next);
    }

    /// Create an entity.
    ///
    /// # Errors
    ///
    /// Returns an error when the transform is invalid, or when the store has
    /// exhausted its slot space.
    pub fn spawn(&mut self, context: SpawnContext) -> Result<EntityId> {
        self.spawn_with_persistent_id(context, None)
    }

    /// Create an entity, optionally reusing a persistent id from a save.
    ///
    /// # Errors
    ///
    /// Returns an error when the transform is invalid, the persistent id is
    /// already taken, or the store is full.
    pub fn spawn_with_persistent_id(
        &mut self,
        context: SpawnContext,
        persistent: Option<PersistentEntityId>,
    ) -> Result<EntityId> {
        context.transform.validate()?;

        let persistent = match persistent {
            Some(id) => {
                if self.by_persistent.contains_key(&id) {
                    return Err(Error::new(
                        Domain::World,
                        "entity-store",
                        "a live entity already holds this persistent id",
                    )
                    .with_recovery(Recovery::Reject)
                    .with_context("persistent_id", id.to_string()));
                }
                // Keep the counter ahead of anything restored from disk.
                self.next_persistent = self.next_persistent.max(id.0 + 1);
                id
            }
            None => {
                let id = PersistentEntityId(self.next_persistent);
                self.next_persistent += 1;
                id
            }
        };

        let index = match self.free.pop() {
            Some(index) => {
                let slot = index as usize;
                self.columns.alive[slot] = true;
                self.columns.persistent[slot] = persistent;
                self.columns.entity_type[slot] = context.entity_type;
                self.columns.transform[slot] = context.transform;
                self.columns.velocity[slot] = context.velocity;
                self.columns.bounds[slot] = context.bounds;
                self.columns.lifecycle[slot] = Lifecycle::Active;
                self.columns.policy[slot] = context.policy;
                self.columns.tags[slot] = context.tags;
                index
            }
            None => {
                let index = u32::try_from(self.columns.len()).map_err(|_| {
                    Error::new(Domain::World, "entity-store", "entity slot space exhausted").fatal()
                })?;
                self.columns.generation.push(0);
                self.columns.alive.push(true);
                self.columns.persistent.push(persistent);
                self.columns.entity_type.push(context.entity_type);
                self.columns.transform.push(context.transform);
                self.columns.velocity.push(context.velocity);
                self.columns.bounds.push(context.bounds);
                self.columns.lifecycle.push(Lifecycle::Active);
                self.columns.policy.push(context.policy);
                self.columns.tags.push(context.tags);
                index
            }
        };

        self.live += 1;
        self.by_persistent.insert(persistent, index);

        let id = EntityId::new(self.world, index, self.columns.generation[index as usize]);
        if let Some(bus) = &self.bus {
            let _ = bus.publish(&EntitySpawned {
                entity: id,
                entity_type: self.columns.entity_type[index as usize].clone(),
                position: self.columns.transform[index as usize].position,
                reason: context.reason,
            });
        }
        Ok(id)
    }

    /// Destroy an entity and invalidate every handle to it.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn despawn(&mut self, id: EntityId) -> Result<()> {
        let index = self.resolve(id)?;
        let slot = index as usize;

        let entity_type = self.columns.entity_type[slot].clone();
        let position = self.columns.transform[slot].position;

        self.columns.alive[slot] = false;
        self.columns.lifecycle[slot] = Lifecycle::Removed;
        self.columns.tags[slot] = TagSet::new();
        self.by_persistent.remove(&self.columns.persistent[slot]);
        self.live -= 1;

        // Advancing the generation is what makes every outstanding handle stale.
        match self.columns.generation[slot].checked_add(1) {
            Some(next) => {
                self.columns.generation[slot] = next;
                self.free.push(index);
            }
            None => {
                // The counter is exhausted. Reusing the slot now would let a very
                // old handle alias a new entity, so the slot is retired instead.
                self.retired_slots += 1;
            }
        }

        if let Some(bus) = &self.bus {
            let _ = bus.publish(&EntityDespawned {
                entity: id,
                entity_type,
                position,
            });
        }
        Ok(())
    }

    /// Whether a handle resolves to a live entity.
    #[must_use]
    pub fn contains(&self, id: EntityId) -> bool {
        self.resolve(id).is_ok()
    }

    /// Resolve a handle to its slot index.
    ///
    /// # Errors
    ///
    /// Returns an error naming why the handle failed: wrong world, out of
    /// range, stale generation, or already destroyed.
    pub fn resolve(&self, id: EntityId) -> Result<u32> {
        if id.world() != self.world {
            return Err(stale_handle(id, "handle belongs to a different world"));
        }
        let slot = id.index() as usize;
        if slot >= self.columns.len() {
            return Err(stale_handle(id, "slot index is out of range"));
        }
        if self.columns.generation[slot] != id.generation() {
            return Err(stale_handle(id, "slot was reused by a newer entity"));
        }
        if !self.columns.alive[slot] {
            return Err(stale_handle(id, "entity was destroyed"));
        }
        Ok(id.index())
    }

    /// The handle for a persistent id, if that entity is loaded.
    #[must_use]
    pub fn resolve_persistent(&self, persistent: PersistentEntityId) -> Option<EntityId> {
        let index = *self.by_persistent.get(&persistent)?;
        Some(EntityId::new(
            self.world,
            index,
            self.columns.generation[index as usize],
        ))
    }

    /// Every live entity, in slot order.
    ///
    /// Slot order is stable within a session and is **not** a simulation
    /// ordering guarantee; nothing authoritative may depend on it.
    pub fn iter(&self) -> impl Iterator<Item = EntityId> + '_ {
        (0..self.columns.len())
            .filter(move |&slot| self.columns.alive[slot])
            .map(move |slot| EntityId::new(self.world, slot as u32, self.columns.generation[slot]))
    }

    /// Advance every active entity by its velocity.
    ///
    /// Uses world time, never wall-clock time, so a replay produces the same
    /// positions. Returns how many entities moved.
    ///
    /// # Errors
    ///
    /// Returns an error when the calendar cannot express the duration.
    pub fn step(&mut self, elapsed: WorldDuration, ticks_per_second: u32) -> Result<usize> {
        if ticks_per_second == 0 {
            return Err(
                Error::new(Domain::Time, "entity-store", "tick rate must be positive")
                    .with_recovery(Recovery::Reject),
            );
        }
        let seconds = elapsed.ticks() as f64 / f64::from(ticks_per_second);
        let mut moved = 0usize;

        // A straight walk over dense columns: this is the batch iteration the
        // data-oriented layout exists for.
        for slot in 0..self.columns.len() {
            if !self.columns.alive[slot] || !self.columns.lifecycle[slot].is_simulated() {
                continue;
            }
            let velocity = self.columns.velocity[slot];
            if velocity.is_still() {
                continue;
            }
            let (dx, dy, dz) = velocity.displacement(seconds);
            let transform = &mut self.columns.transform[slot];
            transform.position = transform.position.offset(dx, dy, dz);
            moved += 1;
        }
        Ok(moved)
    }

    /// Read an entity's transform.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn transform(&self, id: EntityId) -> Result<Transform> {
        Ok(self.columns.transform[self.resolve(id)? as usize])
    }

    /// Replace an entity's transform.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve or the transform is
    /// invalid.
    pub fn set_transform(&mut self, id: EntityId, transform: Transform) -> Result<()> {
        let index = self.resolve(id)?;
        self.columns.transform[index as usize] = transform.validate()?;
        Ok(())
    }

    /// Read an entity's velocity.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn velocity(&self, id: EntityId) -> Result<Velocity> {
        Ok(self.columns.velocity[self.resolve(id)? as usize])
    }

    /// Set an entity's velocity.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn set_velocity(&mut self, id: EntityId, velocity: Velocity) -> Result<()> {
        let index = self.resolve(id)?;
        self.columns.velocity[index as usize] = velocity;
        Ok(())
    }

    /// Read an entity's boundary box.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn bounds(&self, id: EntityId) -> Result<Bounds> {
        Ok(self.columns.bounds[self.resolve(id)? as usize])
    }

    /// Read an entity's type.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn entity_type(&self, id: EntityId) -> Result<EntityTypeId> {
        Ok(self.columns.entity_type[self.resolve(id)? as usize].clone())
    }

    /// Read an entity's persistent id.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn persistent_id(&self, id: EntityId) -> Result<PersistentEntityId> {
        Ok(self.columns.persistent[self.resolve(id)? as usize])
    }

    /// Read an entity's persistence policy.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn policy(&self, id: EntityId) -> Result<PersistencePolicy> {
        Ok(self.columns.policy[self.resolve(id)? as usize])
    }

    /// Read an entity's lifecycle state.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn lifecycle(&self, id: EntityId) -> Result<Lifecycle> {
        Ok(self.columns.lifecycle[self.resolve(id)? as usize])
    }

    /// Move an entity to a new lifecycle state.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve or the transition is
    /// not legal.
    pub fn set_lifecycle(&mut self, id: EntityId, next: Lifecycle) -> Result<()> {
        let index = self.resolve(id)?;
        let current = self.columns.lifecycle[index as usize];
        if !current.can_transition_to(next) {
            return Err(Error::new(
                Domain::World,
                "entity-store",
                "illegal entity lifecycle transition",
            )
            .with_recovery(Recovery::Manual)
            .with_context("entity", id.to_string())
            .with_context("from", current.as_str())
            .with_context("to", next.as_str()));
        }
        self.columns.lifecycle[index as usize] = next;
        Ok(())
    }

    /// Read an entity's tags.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn tags(&self, id: EntityId) -> Result<&TagSet> {
        Ok(&self.columns.tags[self.resolve(id)? as usize])
    }

    /// Add a tag to an entity. Returns whether it was newly added.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn add_tag(&mut self, id: EntityId, tag: Identifier) -> Result<bool> {
        let index = self.resolve(id)?;
        Ok(self.columns.tags[index as usize].insert(tag))
    }

    /// Remove a tag from an entity. Returns whether it was present.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle does not resolve.
    pub fn remove_tag(&mut self, id: EntityId, tag: &Identifier) -> Result<bool> {
        let index = self.resolve(id)?;
        Ok(self.columns.tags[index as usize].remove(tag))
    }

    /// Read the column data for a live slot, for the query and persistence
    /// layers inside this crate.
    pub(crate) fn slot_view(&self, slot: usize) -> Option<SlotView<'_>> {
        if slot >= self.columns.len() || !self.columns.alive[slot] {
            return None;
        }
        Some(SlotView {
            id: EntityId::new(self.world, slot as u32, self.columns.generation[slot]),
            persistent: self.columns.persistent[slot],
            entity_type: &self.columns.entity_type[slot],
            transform: self.columns.transform[slot],
            velocity: self.columns.velocity[slot],
            bounds: self.columns.bounds[slot],
            lifecycle: self.columns.lifecycle[slot],
            policy: self.columns.policy[slot],
            tags: &self.columns.tags[slot],
        })
    }

    /// Every live slot index, for internal iteration.
    pub(crate) fn live_slots(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.columns.len()).filter(move |slot| self.columns.alive[*slot])
    }
}

/// A borrowed view of one entity's components.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SlotView<'a> {
    pub(crate) id: EntityId,
    pub(crate) persistent: PersistentEntityId,
    pub(crate) entity_type: &'a EntityTypeId,
    pub(crate) transform: Transform,
    pub(crate) velocity: Velocity,
    pub(crate) bounds: Bounds,
    pub(crate) lifecycle: Lifecycle,
    pub(crate) policy: PersistencePolicy,
    pub(crate) tags: &'a TagSet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::spatial::WorldPosition;

    fn store() -> EntityStore {
        EntityStore::new(WorldId::derive("store-tests", 1))
    }

    fn context() -> SpawnContext {
        SpawnContext::new(
            EntityTypeId::parse("nexora:entity/test").expect("valid"),
            WorldPosition::ORIGIN,
            SpawnReason::Player,
        )
    }

    #[test]
    fn freed_slots_are_reused_rather_than_growing_the_columns() {
        let mut store = store();
        let first = store.spawn(context()).unwrap();
        assert_eq!(store.slot_count(), 1);

        store.despawn(first).unwrap();
        let second = store.spawn(context()).unwrap();

        assert_eq!(store.slot_count(), 1, "the free slot must be reused");
        assert_eq!(second.index(), first.index());
        assert_eq!(second.generation(), first.generation() + 1);
    }

    #[test]
    fn an_exhausted_generation_retires_the_slot_instead_of_aliasing() {
        // A slot whose generation counter has wrapped could hand a very old
        // handle access to a new entity. It is retired instead.
        let mut store = store();
        let entity = store.spawn(context()).unwrap();
        store.columns.generation[entity.index() as usize] = u32::MAX;

        let doomed = EntityId::new(store.world(), entity.index(), u32::MAX);
        store.despawn(doomed).unwrap();

        assert_eq!(store.retired_slots(), 1);
        assert!(
            store.free.is_empty(),
            "an exhausted slot must not return to the free list"
        );

        // The next spawn takes a fresh slot.
        let next = store.spawn(context()).unwrap();
        assert_ne!(next.index(), entity.index());
    }

    #[test]
    fn persistent_ids_are_never_reused_within_a_session() {
        let mut store = store();
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..500 {
            let entity = store.spawn(context()).unwrap();
            assert!(
                seen.insert(store.persistent_id(entity).unwrap()),
                "id handed out twice"
            );
            store.despawn(entity).unwrap();
        }
        assert_eq!(seen.len(), 500);
    }

    #[test]
    fn restoring_the_counter_never_lowers_it() {
        let mut store = store();
        store.spawn(context()).unwrap();
        let high = store.next_persistent_id();

        store.restore_persistent_counter(1);
        assert_eq!(
            store.next_persistent_id(),
            high,
            "a lower value must not roll the counter back"
        );

        store.restore_persistent_counter(high + 1_000);
        assert_eq!(store.next_persistent_id(), high + 1_000);
    }

    #[test]
    fn a_persistent_id_cannot_be_claimed_twice() {
        let mut store = store();
        let taken = PersistentEntityId(42);
        store
            .spawn_with_persistent_id(context(), Some(taken))
            .unwrap();

        let err = store
            .spawn_with_persistent_id(context(), Some(taken))
            .expect_err("a duplicate persistent id must be refused");
        assert!(err.to_string().contains("already holds"), "{err}");
    }

    #[test]
    fn iteration_visits_exactly_the_live_entities() {
        let mut store = store();
        let a = store.spawn(context()).unwrap();
        let b = store.spawn(context()).unwrap();
        let c = store.spawn(context()).unwrap();
        store.despawn(b).unwrap();

        let mut live: Vec<_> = store.iter().collect();
        live.sort();
        let mut expected = vec![a, c];
        expected.sort();
        assert_eq!(live, expected);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn a_zero_tick_rate_is_refused_rather_than_dividing_by_zero() {
        let mut store = store();
        store.spawn(context()).unwrap();
        assert!(store.step(WorldDuration::from_ticks(20), 0).is_err());
    }

    #[test]
    fn illegal_lifecycle_transitions_are_refused_with_both_states() {
        let mut store = store();
        let entity = store.spawn(context()).unwrap();
        store.set_lifecycle(entity, Lifecycle::Despawning).unwrap();

        let err = store
            .set_lifecycle(entity, Lifecycle::Active)
            .expect_err("despawning cannot go straight back to active");
        assert!(err.to_string().contains("from=despawning"), "{err}");
        assert!(err.to_string().contains("to=active"), "{err}");
    }
}
