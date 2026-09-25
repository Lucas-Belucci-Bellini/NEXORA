//! Entity queries.
//!
//! Implements `Entity System.md` §35 (ENTITY-34), §36 (nearby entities) and
//! §37 (query filters).
//!
//! ## Which queries use the index, and why the others do not
//!
//! The spatial queries — [`Query::within_radius`], [`Query::in_chunk`] and
//! [`Query::overlapping`] — go through the loose grid of [`crate::index`],
//! because measurement showed a radius query over 100,000 entities examining
//! all of them to return six.
//!
//! [`Query::matching`], [`Query::count`], [`Query::by_type`] and
//! [`Query::by_tag`] are still linear scans, and that is not an omission. They
//! ask a question with no position in it, so every entity is a candidate; the
//! same measurement had `by_type` matching the entire population. An index
//! cannot narrow a result that is already everything.
//!
//! A spatial query still falls back to the scan when the region asked about
//! covers more cells than the store holds entities — see
//! `EntityStore::prefers_index`. The index is a way to ask about a small part of
//! a large world, not a faster way to ask about all of it.

use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{ChunkCoord, ChunkShape, WorldPosition};

use crate::components::{Lifecycle, PersistencePolicy};
use crate::id::{EntityId, EntityTypeId};
use crate::index::CellRect;
use crate::store::{EntityStore, SlotView};

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

        let mut found = Vec::new();
        let mut consider = |view: SlotView<'_>| {
            if !filter.matches(&view) {
                return;
            }
            let squared = view.transform.position.distance_squared(centre);
            if squared <= radius_squared {
                found.push(Nearby {
                    entity: view.id,
                    distance: squared.sqrt(),
                });
            }
        };

        Self::visit(store, Self::radius_rect(centre, radius), &mut consider);

        // Ties break by entity index so the order is deterministic; a bare
        // float sort would leave equidistant entities in slot order, which is
        // stable under a scan and not under the index, where cell order and
        // swap-remove decide it.
        found.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.entity.index().cmp(&b.entity.index()))
        });
        found
    }

    /// The cells a radius query has to look in.
    ///
    /// The sphere's horizontal footprint. Vertical extent is not narrowed — the
    /// grid has no vertical subdivision — so the distance test is what rejects
    /// an entity directly above the radius.
    fn radius_rect(centre: WorldPosition, radius: f64) -> CellRect {
        CellRect::covering(
            centre.x - radius,
            centre.z - radius,
            centre.x + radius,
            centre.z + radius,
        )
    }

    /// How many entities a radius query would examine to answer.
    ///
    /// A diagnostic, and the honest measure of what the index bought: before it
    /// existed this was the whole population, every time. It is a count, so it
    /// is the same on every machine and in every build — unlike the time the
    /// difference takes, which is not.
    ///
    /// Goes through the same rectangle and the same fallback as the query
    /// itself, so it cannot report a narrowing the query does not get.
    #[must_use]
    pub fn radius_candidates(store: &EntityStore, centre: WorldPosition, radius: f64) -> usize {
        if !radius.is_finite() || radius < 0.0 {
            return 0;
        }
        let mut seen = 0usize;
        Self::visit(store, Self::radius_rect(centre, radius), &mut |_| seen += 1);
        seen
    }

    /// Entities whose position falls inside a chunk column.
    #[must_use]
    pub fn in_chunk(
        store: &EntityStore,
        shape: ChunkShape,
        chunk: ChunkCoord,
        filter: &EntityFilter,
    ) -> Vec<EntityId> {
        let mut found = Vec::new();
        let mut consider = |view: SlotView<'_>| {
            if filter.matches(&view) && shape.section_of(view.transform.block()).column() == chunk {
                found.push(view.id);
            }
        };

        // The column's own corners, in blocks. Saturating because a chunk
        // coordinate near `i64::MAX` multiplied by an extent is not a position
        // any entity holds, and wrapping it would name a rectangle elsewhere.
        let size_x = i64::from(shape.size_x());
        let size_z = i64::from(shape.size_z());
        let min_x = chunk.x.saturating_mul(size_x);
        let min_z = chunk.z.saturating_mul(size_z);
        let rect = CellRect::covering_blocks(
            min_x,
            min_z,
            min_x.saturating_add(size_x - 1),
            min_z.saturating_add(size_z - 1),
        );
        Self::visit(store, rect, &mut consider);
        // Slot order, which is what a scan produced and what a walk over cells
        // does not. Determinism here is not cosmetic: `NEXORA REPLAY AND
        // DETERMINISM.md` requires the same inputs to yield the same sequence,
        // and an index reorganising itself on a despawn would otherwise change
        // the answer to an unchanged question.
        found.sort_unstable_by_key(|id| id.index());
        found
    }

    /// Entities whose boundary box overlaps the given box.
    #[must_use]
    pub fn overlapping(
        store: &EntityStore,
        at: WorldPosition,
        bounds: crate::components::Bounds,
        filter: &EntityFilter,
    ) -> Vec<EntityId> {
        let mut found = Vec::new();
        let mut consider = |view: SlotView<'_>| {
            if filter.matches(&view) && bounds.overlaps(at, view.bounds, view.transform.position) {
                found.push(view.id);
            }
        };

        // An entity is filed by its centre, so one whose centre sits outside the
        // query box can still reach into it. Widening by the largest half-extent
        // the store has ever held makes the candidate set a superset; the
        // `overlaps` test above is still what decides.
        let reach = bounds.widest_horizontal_half() + store.spatial_index().widest_half_extent();
        let rect = CellRect::covering(at.x - reach, at.z - reach, at.x + reach, at.z + reach);
        Self::visit(store, rect, &mut consider);
        found.sort_unstable_by_key(|id| id.index());
        found
    }

    /// Offer every candidate in a rectangle of cells to `consider`, or every
    /// live entity when the rectangle is the more expensive of the two.
    ///
    /// One place decides, so the three spatial queries cannot drift into three
    /// different answers about when the index is worth using.
    fn visit<F: FnMut(SlotView<'_>)>(store: &EntityStore, rect: CellRect, consider: &mut F) {
        if store.prefers_index(rect) {
            store.spatial_index().for_each_in(rect, |slot| {
                if let Some(view) = store.slot_view(slot as usize) {
                    consider(view);
                }
            });
        } else {
            for slot in store.live_slots() {
                if let Some(view) = store.slot_view(slot) {
                    consider(view);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Bounds, TagSet, Velocity};
    use crate::store::{SpawnContext, SpawnReason};
    use nexora_foundation::ident::WorldId;
    use nexora_foundation::rng::Rng;
    use nexora_foundation::spatial::ChunkShape;
    use nexora_foundation::time::WorldDuration;

    fn kind() -> EntityTypeId {
        EntityTypeId::parse("nexora:entity/test").expect("valid")
    }

    fn at(store: &mut EntityStore, x: f64, y: f64, z: f64) -> EntityId {
        store
            .spawn(SpawnContext::new(
                kind(),
                WorldPosition::new(x, y, z),
                SpawnReason::Player,
            ))
            .expect("spawn")
    }

    /// The answer computed without the index: every live entity, tested
    /// directly. Anything the index returns has to match this exactly, so the
    /// reference deliberately shares no code with the thing it checks.
    fn brute_force_radius(
        store: &EntityStore,
        centre: WorldPosition,
        radius: f64,
    ) -> Vec<EntityId> {
        let mut found: Vec<EntityId> = store
            .iter()
            .filter(|&id| {
                store
                    .transform(id)
                    .expect("live")
                    .position
                    .distance_squared(centre)
                    <= radius * radius
            })
            .collect();
        found.sort_unstable_by_key(|id| id.index());
        found
    }

    /// Entities parked far from anywhere a test probes.
    ///
    /// `EntityStore::prefers_index` falls back to the scan when the query's
    /// rectangle covers more cells than the store holds entities, which a store
    /// with three entities in it always does. A test written without this
    /// exercises the scan, agrees with itself, and proves nothing about the
    /// index — which is how the first draft of the three tests below passed
    /// with every one of the store's index hooks deleted.
    fn crowd(store: &mut EntityStore, count: usize) {
        for step in 0..count {
            at(store, 1_000_000.0 + step as f64, 64.0, 1_000_000.0);
        }
    }

    fn populated(count: usize, spread: f64) -> EntityStore {
        let mut rng = Rng::from_seed(0x0010_0010);
        let mut store = EntityStore::new(WorldId::derive("query-tests", 3));
        for _ in 0..count {
            let position = WorldPosition::new(
                rng.next_f64().mul_add(spread, -spread / 2.0),
                rng.next_f64() * 128.0,
                rng.next_f64().mul_add(spread, -spread / 2.0),
            );
            store
                .spawn(SpawnContext::new(kind(), position, SpawnReason::WorldGen))
                .expect("spawn");
        }
        store
    }

    #[test]
    fn the_index_answers_a_radius_query_exactly_as_a_full_scan_would() {
        let store = populated(2_000, 512.0);
        // Radii on both sides of the fallback threshold, and one that spans the
        // whole population so the scan branch is exercised too.
        for radius in [0.0, 1.0, 7.5, 16.0, 17.0, 64.0, 1_024.0] {
            for centre in [
                WorldPosition::new(0.0, 64.0, 0.0),
                WorldPosition::new(-255.0, 0.0, 255.0),
                WorldPosition::new(37.5, 200.0, -12.25),
            ] {
                let mut viaindex: Vec<EntityId> =
                    Query::within_radius(&store, centre, radius, &EntityFilter::new())
                        .into_iter()
                        .map(|near| near.entity)
                        .collect();
                viaindex.sort_unstable_by_key(|id| id.index());
                assert_eq!(
                    viaindex,
                    brute_force_radius(&store, centre, radius),
                    "radius {radius} around {centre:?}"
                );
            }
        }
    }

    #[test]
    fn a_radius_query_is_still_sorted_nearest_first() {
        let mut store = EntityStore::new(WorldId::derive("query-tests", 4));
        // Spawned far-to-near, and in three different cells, so neither slot
        // order nor cell order is already the answer.
        let far = at(&mut store, 30.0, 0.0, 0.0);
        let near = at(&mut store, 1.0, 0.0, 0.0);
        let middle = at(&mut store, 17.0, 0.0, 0.0);

        let found = Query::within_radius(&store, WorldPosition::ORIGIN, 64.0, &EntityFilter::new());
        assert_eq!(
            found.iter().map(|n| n.entity).collect::<Vec<_>>(),
            vec![near, middle, far]
        );
    }

    #[test]
    fn equidistant_results_keep_a_deterministic_order_across_cells() {
        // Far enough out that the populated background contributes nothing, and
        // populous enough that the index, not the scan, chooses the candidates.
        let mut store = populated(400, 100_000.0);
        let centre = WorldPosition::new(0.0, 0.0, 0.0);
        // Same distance, opposite sides, different cells.
        let east = at(&mut store, 20.0, 0.0, 0.0);
        let west = at(&mut store, -20.0, 0.0, 0.0);
        assert!(store.prefers_index(CellRect::covering(-32.0, -32.0, 32.0, 32.0)));

        let found = Query::within_radius(&store, centre, 32.0, &EntityFilter::new());
        assert_eq!(
            found.iter().map(|n| n.entity).collect::<Vec<_>>(),
            vec![east, west],
            "the tie must break by slot index, not by which cell was walked first"
        );
    }

    #[test]
    fn moving_an_entity_takes_it_out_of_the_cell_it_left() {
        let mut store = EntityStore::new(WorldId::derive("query-tests", 6));
        crowd(&mut store, 64);
        let id = at(&mut store, 4.0, 64.0, 4.0);
        let origin = WorldPosition::new(4.0, 64.0, 4.0);
        let elsewhere = WorldPosition::new(4_000.0, 64.0, 4_000.0);
        assert!(store.prefers_index(CellRect::covering(2.0, 2.0, 6.0, 6.0)));

        assert_eq!(
            Query::within_radius(&store, origin, 2.0, &EntityFilter::new()).len(),
            1
        );

        store
            .set_transform(id, crate::components::Transform::at(elsewhere))
            .expect("move");

        assert!(
            Query::within_radius(&store, origin, 2.0, &EntityFilter::new()).is_empty(),
            "an index that kept the old cell would still answer here"
        );
        assert_eq!(
            Query::within_radius(&store, elsewhere, 2.0, &EntityFilter::new()).len(),
            1
        );
    }

    #[test]
    fn stepping_across_a_cell_boundary_refiles_the_entity() {
        let mut store = EntityStore::new(WorldId::derive("query-tests", 7));
        crowd(&mut store, 64);
        // One block short of the boundary at x = 16, moving east fast enough to
        // cross it in a single tick.
        let start = WorldPosition::new(15.0, 64.0, 0.0);
        let id = store
            .spawn(
                SpawnContext::new(kind(), start, SpawnReason::Player)
                    .with_velocity(Velocity::new(80.0, 0.0, 0.0).expect("finite")),
            )
            .expect("spawn");

        store
            .step(WorldDuration::from_ticks(1), 20)
            .expect("one tick");

        let now = store.transform(id).expect("live").position;
        assert!(now.x >= 16.0, "the test needs the entity to have crossed");
        assert!(store.prefers_index(CellRect::covering(
            now.x - 0.5,
            now.z - 0.5,
            now.x + 0.5,
            now.z + 0.5
        )));
        assert_eq!(
            Query::within_radius(&store, now, 0.5, &EntityFilter::new()).len(),
            1,
            "step must refile an entity whose cell changed"
        );
        assert!(
            Query::within_radius(&store, start, 0.5, &EntityFilter::new()).is_empty(),
            "and must not leave it behind in the cell it left"
        );
    }

    #[test]
    fn a_despawned_entity_leaves_no_trace_in_its_cell() {
        let mut store = EntityStore::new(WorldId::derive("query-tests", 8));
        crowd(&mut store, 256);
        let here = WorldPosition::new(100.0, 64.0, 100.0);
        let doomed = at(&mut store, 100.0, 64.0, 100.0);
        let survivor = at(&mut store, 101.0, 64.0, 100.0);
        // The rectangle of the *widest* query below, not of the first one: a
        // radius of 64 covers 81 cells, and asserting on the narrow query's
        // rectangle let the wide one fall back to the scan unnoticed.
        assert!(store.prefers_index(CellRect::covering(36.0, 36.0, 164.0, 164.0)));

        store.despawn(doomed).expect("despawn");
        assert_eq!(
            Query::within_radius(&store, here, 8.0, &EntityFilter::new())
                .iter()
                .map(|near| near.entity)
                .collect::<Vec<_>>(),
            vec![survivor]
        );

        // The freed slot comes back in a *different* cell, and a query wide
        // enough to cover both must still answer once. An index that kept the
        // dead entry files the slot twice, and every such query returns the same
        // entity twice — `slot_view` rejects a dead slot, so nothing else here
        // would notice.
        let reused = at(&mut store, 140.0, 64.0, 100.0);
        assert_eq!(reused.index(), doomed.index(), "the test needs slot reuse");

        let found = Query::within_radius(&store, here, 64.0, &EntityFilter::new());
        let mut ids: Vec<EntityId> = found.iter().map(|near| near.entity).collect();
        ids.sort_unstable_by_key(|id| id.index());
        let mut unique = ids.clone();
        unique.dedup();
        assert_eq!(ids, unique, "an entity was returned more than once");
        assert_eq!(unique.len(), 2);
        assert!(unique.contains(&reused) && unique.contains(&survivor));
    }

    #[test]
    fn emptying_a_store_empties_the_grid_rather_than_leaking_its_cells() {
        let mut store = EntityStore::new(WorldId::derive("query-tests", 13));
        let mut spawned = Vec::new();
        for step in 0..256i64 {
            // One per cell, so a leak is a cell that outlives its entity.
            spawned.push(at(&mut store, (step * 16) as f64, 64.0, (step * 16) as f64));
        }
        assert_eq!(store.occupied_cells(), 256);

        for id in spawned {
            store.despawn(id).expect("despawn");
        }
        assert!(store.is_empty());
        assert_eq!(
            store.occupied_cells(),
            0,
            "every cell an entity ever occupied is still filed"
        );
    }

    #[test]
    fn in_chunk_agrees_with_a_scan_for_shapes_both_smaller_and_larger_than_a_cell() {
        let store = populated(1_500, 256.0);
        for shape in [
            ChunkShape::new(8, 8, 8).expect("valid"),
            ChunkShape::new(16, 16, 16).expect("valid"),
            ChunkShape::cubic_default(),
            ChunkShape::new(64, 64, 64).expect("valid"),
        ] {
            for column in [
                ChunkCoord::new(0, 0),
                ChunkCoord::new(-1, -1),
                ChunkCoord::new(2, -3),
            ] {
                let mut expected: Vec<EntityId> = store
                    .iter()
                    .filter(|&id| {
                        shape
                            .section_of(store.transform(id).expect("live").block())
                            .column()
                            == column
                    })
                    .collect();
                expected.sort_unstable_by_key(|id| id.index());
                assert_eq!(
                    Query::in_chunk(&store, shape, column, &EntityFilter::new()),
                    expected,
                    "shape {shape:?} column {column:?}"
                );
            }
        }
    }

    #[test]
    fn a_box_query_finds_an_entity_whose_centre_lies_outside_the_box() {
        // Enough population that the index branch is the one taken: with a
        // handful of entities the rectangle covers more cells than the store
        // holds entities and every query falls back to the scan, which would
        // pass this test without the widening it exists to check.
        let mut store = populated(400, 4_096.0);
        // Two cells east of the probe, and wide enough to reach back into it:
        // spans x in [-2, 38] while the probe spans [-1, 1].
        let wide = store
            .spawn(
                SpawnContext::new(
                    kind(),
                    WorldPosition::new(18.0, 64.0, 0.0),
                    SpawnReason::Player,
                )
                .with_bounds(Bounds::new(20.0, 64.0, 20.0).expect("valid")),
            )
            .expect("spawn");

        let probe = Bounds::new(1.0, 1.0, 1.0).expect("valid");
        let at = WorldPosition::new(0.0, 64.0, 0.0);
        let reach = probe.widest_horizontal_half() + store.spatial_index().widest_half_extent();
        assert!(
            store.prefers_index(CellRect::covering(
                at.x - reach,
                at.z - reach,
                at.x + reach,
                at.z + reach
            )),
            "the test only means something if the index is the path taken"
        );

        assert!(
            Query::overlapping(&store, at, probe, &EntityFilter::new()).contains(&wide),
            "the candidate rectangle must widen by the widest entity in the store"
        );
    }

    #[test]
    fn overlapping_agrees_with_a_scan_over_a_mixed_population() {
        let mut store = EntityStore::new(WorldId::derive("query-tests", 10));
        let mut rng = Rng::from_seed(0xBEEF_0010);
        for index in 0..800usize {
            let half = if index % 50 == 0 { 9.0 } else { 0.4 };
            store
                .spawn(
                    SpawnContext::new(
                        kind(),
                        WorldPosition::new(
                            rng.next_f64().mul_add(256.0, -128.0),
                            rng.next_f64() * 64.0,
                            rng.next_f64().mul_add(256.0, -128.0),
                        ),
                        SpawnReason::WorldGen,
                    )
                    .with_bounds(Bounds::new(half, 1.8, half).expect("valid"))
                    .with_tags(TagSet::new()),
                )
                .expect("spawn");
        }

        let probe = Bounds::new(3.0, 3.0, 3.0).expect("valid");
        for at in [
            WorldPosition::new(0.0, 32.0, 0.0),
            WorldPosition::new(-120.0, 10.0, 119.0),
            WorldPosition::new(64.5, 50.0, -64.5),
        ] {
            let mut expected: Vec<EntityId> = store
                .iter()
                .filter(|&id| {
                    probe.overlaps(
                        at,
                        store.bounds(id).expect("live"),
                        store.transform(id).expect("live").position,
                    )
                })
                .collect();
            expected.sort_unstable_by_key(|id| id.index());
            assert_eq!(
                Query::overlapping(&store, at, probe, &EntityFilter::new()),
                expected,
                "box at {at:?}"
            );
        }
    }

    #[test]
    fn a_filter_still_applies_when_the_index_chose_the_candidates() {
        let mut store = EntityStore::new(WorldId::derive("query-tests", 11));
        let other = EntityTypeId::parse("nexora:entity/other").expect("valid");
        let wanted = at(&mut store, 1.0, 0.0, 1.0);
        store
            .spawn(SpawnContext::new(
                other,
                WorldPosition::new(2.0, 0.0, 2.0),
                SpawnReason::Player,
            ))
            .expect("spawn");

        let found = Query::within_radius(
            &store,
            WorldPosition::ORIGIN,
            8.0,
            &EntityFilter::new().of_type(kind()),
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].entity, wanted);
    }

    #[test]
    fn an_empty_store_answers_every_spatial_query_without_panicking() {
        let store = EntityStore::new(WorldId::derive("query-tests", 12));
        assert!(
            Query::within_radius(&store, WorldPosition::ORIGIN, 1_000.0, &EntityFilter::new())
                .is_empty()
        );
        assert!(Query::in_chunk(
            &store,
            ChunkShape::cubic_default(),
            ChunkCoord::new(0, 0),
            &EntityFilter::new()
        )
        .is_empty());
        assert!(Query::overlapping(
            &store,
            WorldPosition::ORIGIN,
            Bounds::POINT,
            &EntityFilter::new()
        )
        .is_empty());
    }

    #[test]
    fn a_store_restored_from_a_save_comes_back_indexed() {
        // ADR-0015 claims no migration is needed because `persist::load` spawns,
        // and spawning files. Nothing else asserts that, and a save that loaded
        // into an unindexed store would answer every spatial query with nothing.
        let mut original = EntityStore::new(WorldId::derive("query-tests", 14));
        crowd(&mut original, 256);
        let here = WorldPosition::new(700.0, 64.0, -700.0);
        at(&mut original, 700.0, 64.0, -700.0);
        at(&mut original, 703.0, 64.0, -698.0);

        let mut container = nexora_persistence::SaveContainer::new();
        crate::persist::save(&original, &mut container).expect("save");

        let mut restored = EntityStore::new(original.world());
        crate::persist::load(&mut restored, &container).expect("load");
        assert_eq!(restored.len(), original.len());
        assert_eq!(restored.occupied_cells(), original.occupied_cells());
        assert!(restored.prefers_index(CellRect::covering(684.0, -716.0, 716.0, -684.0)));
        assert_eq!(
            Query::within_radius(&restored, here, 8.0, &EntityFilter::new()).len(),
            2
        );
    }

    #[test]
    fn the_candidate_count_narrows_with_the_index_and_falls_back_to_everything() {
        let store = populated(20_000, 1_131.0);
        let centre = WorldPosition::new(565.0, 96.0, 565.0);

        let narrow = Query::radius_candidates(&store, centre, 16.0);
        let hits = Query::within_radius(&store, centre, 16.0, &EntityFilter::new()).len();
        assert!(hits <= narrow, "a hit has to have been a candidate");
        assert!(
            narrow < store.len() / 50,
            "examined {narrow} of {} — the index narrowed nothing",
            store.len()
        );

        // Wide enough to lose: the fallback examines the population, which is
        // the most any query can, and never more.
        assert_eq!(
            Query::radius_candidates(&store, centre, 1e300),
            store.len(),
            "the fallback examines each entity once"
        );
    }

    #[test]
    fn a_radius_that_is_not_a_radius_returns_nothing_rather_than_everything() {
        let store = populated(50, 64.0);
        // Infinity among them: "within an infinite distance" is every entity in
        // the world, which is not a spatial question, and the pre-index contract
        // has always refused it rather than answering it.
        for radius in [-1.0, f64::NAN, f64::NEG_INFINITY, f64::INFINITY] {
            assert!(
                Query::within_radius(&store, WorldPosition::ORIGIN, radius, &EntityFilter::new())
                    .is_empty(),
                "radius {radius}"
            );
        }
    }

    #[test]
    fn a_radius_too_large_for_the_grid_falls_back_instead_of_walking_the_coordinate_space() {
        let store = populated(50, 64.0);
        // 1e300 is finite, so it is a legal radius, and its rectangle spans
        // about 1.3e36 cells. The fallback is not a speed choice here: walking
        // that rectangle does not finish, so the guard is what makes the query
        // answerable at all.
        //
        // The decision is asserted rather than only the answer. Deleting the
        // guard makes this query hang, and a test that hangs reports nothing —
        // it has to be caught by asking what was decided, which returns.
        let rect = CellRect::covering(-1e300, -1e300, 1e300, 1e300);
        assert!(
            rect.cell_count() > 1e36 as i128,
            "the rectangle really is that large"
        );
        assert!(
            !store.prefers_index(rect),
            "a rectangle this size must fall back to the scan"
        );
        let found =
            Query::within_radius(&store, WorldPosition::ORIGIN, 1e300, &EntityFilter::new());
        assert_eq!(found.len(), 50);
    }
}
