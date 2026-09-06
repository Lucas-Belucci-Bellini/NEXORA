//! Entity queries.
//!
//! Implements `Entity System.md` §35 (ENTITY-34), §36 (nearby entities) and
//! §37 (query filters).
//!
//! Every query here is a **linear scan** over the dense component columns. That
//! is deliberate for Phase 0: the spatial index of §34 (ENTITY-33) is not built
//! yet, and building one before anything has measured the scan would repeat the
//! mistake `DEBT-0005` caught. The benchmark suite measures these directly, so
//! the decision to index will have a number behind it.

use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{ChunkCoord, ChunkShape, WorldPosition};

use crate::components::{Lifecycle, PersistencePolicy};
use crate::id::{EntityId, EntityTypeId};
use crate::store::EntityStore;

/// A composable set of conditions an entity must meet.
///
/// An empty filter matches every live entity.
#[derive(Debug, Clone, Default)]
pub struct EntityFilter {
    entity_type: Option<EntityTypeId>,
    tag: Option<Identifier>,
    lifecycle: Option<Lifecycle>,
    policy: Option<PersistencePolicy>,
    simulated_only: bool,
}

impl EntityFilter {
    /// Match everything.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only entities of this type.
    #[must_use]
    pub fn of_type(mut self, entity_type: EntityTypeId) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    /// Only entities carrying this tag.
    #[must_use]
    pub fn with_tag(mut self, tag: Identifier) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Only entities in this lifecycle state.
    #[must_use]
    pub const fn in_lifecycle(mut self, lifecycle: Lifecycle) -> Self {
        self.lifecycle = Some(lifecycle);
        self
    }

    /// Only entities with this persistence policy.
    #[must_use]
    pub const fn with_policy(mut self, policy: PersistencePolicy) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Only entities currently being simulated.
    #[must_use]
    pub const fn simulated(mut self) -> Self {
        self.simulated_only = true;
        self
    }

    fn matches(&self, view: &crate::store::SlotView<'_>) -> bool {
        if let Some(entity_type) = &self.entity_type {
            if view.entity_type != entity_type {
                return false;
            }
        }
        if let Some(tag) = &self.tag {
            if !view.tags.contains(tag) {
                return false;
            }
        }
        if let Some(lifecycle) = self.lifecycle {
            if view.lifecycle != lifecycle {
                return false;
            }
        }
        if let Some(policy) = self.policy {
            if view.policy != policy {
                return false;
            }
        }
        if self.simulated_only && !view.lifecycle.is_simulated() {
            return false;
        }
        true
    }
}

/// An entity found by a spatial query, with how far away it was.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nearby {
    /// The entity.
    pub entity: EntityId,
    /// Distance from the query point, in blocks.
    pub distance: f64,
}

/// Query helpers over a store.
///
/// Free functions rather than methods so the store stays focused on storage and
/// lifecycle; query strategy is expected to change once an index exists.
pub struct Query;

impl Query {
    /// Every entity matching a filter.
    #[must_use]
    pub fn matching(store: &EntityStore, filter: &EntityFilter) -> Vec<EntityId> {
        store
            .live_slots()
            .filter_map(|slot| store.slot_view(slot))
            .filter(|view| filter.matches(view))
            .map(|view| view.id)
            .collect()
    }

    /// How many entities match a filter, without building a list.
    #[must_use]
    pub fn count(store: &EntityStore, filter: &EntityFilter) -> usize {
        store
            .live_slots()
            .filter_map(|slot| store.slot_view(slot))
            .filter(|view| filter.matches(view))
            .count()
    }

    /// Entities of a type.
    #[must_use]
    pub fn by_type(store: &EntityStore, entity_type: EntityTypeId) -> Vec<EntityId> {
        Self::matching(store, &EntityFilter::new().of_type(entity_type))
    }

    /// Entities carrying a tag.
    #[must_use]
    pub fn by_tag(store: &EntityStore, tag: Identifier) -> Vec<EntityId> {
        Self::matching(store, &EntityFilter::new().with_tag(tag))
    }

    /// Entities within `radius` blocks of a point, nearest first.
    ///
    /// Compares squared distances while filtering and takes the square root
    /// only for the results that survive.
    #[must_use]
    pub fn within_radius(
        store: &EntityStore,
        centre: WorldPosition,
        radius: f64,
        filter: &EntityFilter,
    ) -> Vec<Nearby> {
        if !radius.is_finite() || radius < 0.0 {
            return Vec::new();
        }
        let radius_squared = radius * radius;

        let mut found: Vec<Nearby> = store
            .live_slots()
            .filter_map(|slot| store.slot_view(slot))
            .filter(|view| filter.matches(view))
            .filter_map(|view| {
                let squared = view.transform.position.distance_squared(centre);
                (squared <= radius_squared).then(|| Nearby {
                    entity: view.id,
                    distance: squared.sqrt(),
                })
            })
            .collect();

        // Ties break by entity index so the order is deterministic; a bare
        // float sort would leave equidistant entities in slot order, which is
        // stable in practice but not guaranteed by the comparator.
        found.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.entity.index().cmp(&b.entity.index()))
        });
        found
    }

    /// Entities whose position falls inside a chunk column.
    #[must_use]
    pub fn in_chunk(
        store: &EntityStore,
        shape: ChunkShape,
        chunk: ChunkCoord,
        filter: &EntityFilter,
    ) -> Vec<EntityId> {
        store
            .live_slots()
            .filter_map(|slot| store.slot_view(slot))
            .filter(|view| filter.matches(view))
            .filter(|view| shape.section_of(view.transform.block()).column() == chunk)
            .map(|view| view.id)
            .collect()
    }

    /// Entities whose boundary box overlaps the given box.
    #[must_use]
    pub fn overlapping(
        store: &EntityStore,
        at: WorldPosition,
        bounds: crate::components::Bounds,
        filter: &EntityFilter,
    ) -> Vec<EntityId> {
        store
            .live_slots()
            .filter_map(|slot| store.slot_view(slot))
            .filter(|view| filter.matches(view))
            .filter(|view| bounds.overlaps(at, view.bounds, view.transform.position))
            .map(|view| view.id)
            .collect()
    }
}
