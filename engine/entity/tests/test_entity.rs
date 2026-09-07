//! The `TestEntity` proof.
//!
//! The startup brief §38 is explicit: before any mob, before any AI, a single
//! test entity must prove ten things. *"Não começar pela aranha."*
//!
//! ```text
//! exists · loads · saves · updates · receives events
//! moves through the world · can be queried
//! identity · transform · destruction
//! ```
//!
//! One test per item, named after it, so a failure says which guarantee broke.

use std::sync::{Arc, Mutex};

use nexora_entity::components::{
    Bounds, Lifecycle, PersistencePolicy, TagSet, Transform, Velocity,
};
use nexora_entity::events::{EntityDespawned, EntitySpawned};
use nexora_entity::id::EntityTypeId;
use nexora_entity::persist;
use nexora_entity::query::{EntityFilter, Query};
use nexora_entity::store::{EntityStore, SpawnContext, SpawnReason};
use nexora_foundation::ident::{Identifier, WorldId};
use nexora_foundation::spatial::{ChunkShape, WorldPosition};
use nexora_foundation::time::WorldDuration;
use nexora_persistence::SaveContainer;
use nexora_runtime::events::{EventBus, Priority};

const TICKS_PER_SECOND: u32 = 20;

fn world() -> WorldId {
    WorldId::derive("test-entity-world", 4242)
}

fn test_type() -> EntityTypeId {
    EntityTypeId::parse("nexora:entity/test").expect("valid entity type")
}

fn tag(raw: &str) -> Identifier {
    Identifier::parse(raw).expect("valid tag")
}

fn store() -> EntityStore {
    EntityStore::new(world())
}

fn spawn_test_entity(store: &mut EntityStore, at: WorldPosition) -> nexora_entity::EntityId {
    store
        .spawn(
            SpawnContext::new(test_type(), at, SpawnReason::Player)
                .with_bounds(Bounds::new(0.4, 0.9, 0.4).expect("valid bounds"))
                .with_tags(TagSet::from_iter_sorted([tag("nexora:tag/test")])),
        )
        .expect("spawn succeeds")
}

// 1 ------------------------------------------------------------------ exists
#[test]
fn it_exists() {
    let mut store = store();
    assert!(store.is_empty());

    let entity = spawn_test_entity(&mut store, WorldPosition::new(1.0, 64.0, 2.0));

    assert_eq!(store.len(), 1);
    assert!(store.contains(entity));
    assert_eq!(store.entity_type(entity).unwrap(), test_type());
    assert_eq!(store.lifecycle(entity).unwrap(), Lifecycle::Active);
}

// 2 ---------------------------------------------------------------- identity
#[test]
fn it_has_a_stable_identity_that_a_stale_handle_cannot_impersonate() {
    let mut store = store();
    let first = spawn_test_entity(&mut store, WorldPosition::ORIGIN);
    let first_persistent = store.persistent_id(first).unwrap();

    store.despawn(first).expect("despawn");

    // The slot is free, so the next spawn reuses it - and the old handle must
    // not resolve to the new occupant. This is the invariant from
    // NEXORA DATA VALIDATION AND INVARIANTS.md: a destroyed object cannot
    // remain addressable as active state.
    let second = spawn_test_entity(&mut store, WorldPosition::ORIGIN);
    assert_eq!(
        second.index(),
        first.index(),
        "the test is weak unless the slot is reused"
    );
    assert_ne!(second.generation(), first.generation());

    assert!(!store.contains(first));
    assert!(store.contains(second));

    let err = store
        .transform(first)
        .expect_err("a stale handle must not resolve");
    assert!(
        err.to_string().contains("reused by a newer entity"),
        "{err}"
    );

    // Persistent ids are never recycled either.
    assert_ne!(store.persistent_id(second).unwrap(), first_persistent);
}

#[test]
fn a_handle_from_another_world_is_refused() {
    let mut store = store();
    let entity = spawn_test_entity(&mut store, WorldPosition::ORIGIN);

    let mut other = EntityStore::new(WorldId::derive("a-different-world", 1));
    let err = other
        .transform(entity)
        .expect_err("cross-world handles must not resolve");
    assert!(err.to_string().contains("different world"), "{err}");
    assert!(other.despawn(entity).is_err());
}

// 3 --------------------------------------------------------------- transform
#[test]
fn it_has_a_transform() {
    let mut store = store();
    let at = WorldPosition::new(-3.5, 70.25, 8.75);
    let entity = spawn_test_entity(&mut store, at);

    let transform = store.transform(entity).unwrap();
    assert_eq!(transform.position, at);
    // Floors into the block below, not toward zero.
    assert_eq!(
        transform.block(),
        nexora_foundation::spatial::BlockPos::new(-4, 70, 8)
    );

    store
        .set_transform(
            entity,
            Transform {
                position: at,
                yaw: 90.0,
                pitch: -15.0,
            },
        )
        .expect("set transform");
    assert_eq!(store.transform(entity).unwrap().yaw, 90.0);

    // Garbage in is refused rather than stored.
    let bad = Transform::at(WorldPosition::new(f64::NAN, 0.0, 0.0));
    assert!(store.set_transform(entity, bad).is_err());
    assert!(store.transform(entity).unwrap().position.is_finite());
}

// 4 ------------------------------------------------------------------- spawn
#[test]
fn it_spawns_with_a_reason_and_a_policy() {
    let mut store = store();

    let permanent = store
        .spawn(SpawnContext::new(
            test_type(),
            WorldPosition::ORIGIN,
            SpawnReason::WorldGen,
        ))
        .unwrap();
    let ephemeral = store
        .spawn(
            SpawnContext::new(test_type(), WorldPosition::ORIGIN, SpawnReason::Spawner)
                .with_policy(PersistencePolicy::Temporary),
        )
        .unwrap();

    assert_eq!(
        store.policy(permanent).unwrap(),
        PersistencePolicy::Persistent
    );
    assert_eq!(
        store.policy(ephemeral).unwrap(),
        PersistencePolicy::Temporary
    );
    assert_eq!(store.len(), 2);
}

// 5 ------------------------------------------------------ updates / movement
#[test]
fn it_updates_and_moves_through_the_world() {
    let mut store = store();
    let start = WorldPosition::new(0.0, 64.0, 0.0);
    let entity = store
        .spawn(
            SpawnContext::new(test_type(), start, SpawnReason::Player)
                .with_velocity(Velocity::new(2.0, 0.0, -1.0).unwrap()),
        )
        .unwrap();

    // One in-world second at 20 ticks/s.
    let moved = store
        .step(WorldDuration::from_ticks(20), TICKS_PER_SECOND)
        .unwrap();
    assert_eq!(moved, 1);

    let position = store.transform(entity).unwrap().position;
    assert!((position.x - 2.0).abs() < 1e-9, "x was {}", position.x);
    assert!((position.y - 64.0).abs() < 1e-9);
    assert!((position.z + 1.0).abs() < 1e-9, "z was {}", position.z);

    // Half a second more.
    store
        .step(WorldDuration::from_ticks(10), TICKS_PER_SECOND)
        .unwrap();
    assert!((store.transform(entity).unwrap().position.x - 3.0).abs() < 1e-9);
}

#[test]
fn movement_is_driven_by_world_time_not_wall_clock() {
    // The same tick count must always produce the same displacement, no matter
    // how long the call actually took. Otherwise a replay diverges.
    let run = || {
        let mut store = store();
        let entity = store
            .spawn(
                SpawnContext::new(test_type(), WorldPosition::ORIGIN, SpawnReason::Player)
                    .with_velocity(Velocity::new(1.5, -0.25, 3.0).unwrap()),
            )
            .unwrap();
        for _ in 0..100 {
            store
                .step(WorldDuration::from_ticks(7), TICKS_PER_SECOND)
                .unwrap();
        }
        store.transform(entity).unwrap().position
    };
    assert_eq!(run(), run());
}

#[test]
fn a_sleeping_entity_does_not_move() {
    let mut store = store();
    let entity = store
        .spawn(
            SpawnContext::new(test_type(), WorldPosition::ORIGIN, SpawnReason::Player)
                .with_velocity(Velocity::new(5.0, 0.0, 0.0).unwrap()),
        )
        .unwrap();

    // Despawn is not death: an inactive entity is still present, just not ticked.
    store.set_lifecycle(entity, Lifecycle::Inactive).unwrap();
    let moved = store
        .step(WorldDuration::from_ticks(100), TICKS_PER_SECOND)
        .unwrap();

    assert_eq!(moved, 0);
    assert_eq!(
        store.transform(entity).unwrap().position,
        WorldPosition::ORIGIN
    );
    assert!(store.lifecycle(entity).unwrap().is_present());

    // Waking it up resumes movement.
    store.set_lifecycle(entity, Lifecycle::Active).unwrap();
    assert_eq!(
        store
            .step(WorldDuration::from_ticks(20), TICKS_PER_SECOND)
            .unwrap(),
        1
    );
}

// 6 ----------------------------------------------------- collision boundary
#[test]
fn it_has_a_collision_boundary() {
    let mut store = store();
    let bounds = Bounds::new(0.5, 1.0, 0.5).unwrap();

    let standing = store
        .spawn(
            SpawnContext::new(
                test_type(),
                WorldPosition::new(0.0, 64.0, 0.0),
                SpawnReason::Player,
            )
            .with_bounds(bounds),
        )
        .unwrap();
    assert_eq!(store.bounds(standing).unwrap(), bounds);

    // Overlapping.
    let overlapping = Query::overlapping(
        &store,
        WorldPosition::new(0.5, 64.0, 0.0),
        bounds,
        &EntityFilter::new(),
    );
    assert_eq!(overlapping, vec![standing]);

    // Exactly touching is adjacency, not collision.
    let touching = Query::overlapping(
        &store,
        WorldPosition::new(1.0, 64.0, 0.0),
        bounds,
        &EntityFilter::new(),
    );
    assert!(touching.is_empty());

    // Far away.
    let apart = Query::overlapping(
        &store,
        WorldPosition::new(50.0, 64.0, 0.0),
        bounds,
        &EntityFilter::new(),
    );
    assert!(apart.is_empty());
}

// 7 ------------------------------------------------------------------ events
#[test]
fn it_publishes_facts_when_it_appears_and_when_it_goes() {
    let bus = EventBus::new();
    let spawned = Arc::new(Mutex::new(Vec::new()));
    let despawned = Arc::new(Mutex::new(Vec::new()));

    let sink = spawned.clone();
    bus.subscribe::<EntitySpawned, _>(
        tag("nexora:system/observer"),
        Priority::Normal,
        move |event, _| sink.lock().unwrap().push((event.entity, event.reason)),
    );
    let sink = despawned.clone();
    bus.subscribe::<EntityDespawned, _>(
        tag("nexora:system/observer"),
        Priority::Normal,
        move |event, _| sink.lock().unwrap().push(event.entity),
    );

    let mut store = EntityStore::new(world()).with_event_bus(bus);
    let entity = spawn_test_entity(&mut store, WorldPosition::new(5.0, 64.0, 5.0));

    assert_eq!(*spawned.lock().unwrap(), [(entity, SpawnReason::Player)]);
    assert!(despawned.lock().unwrap().is_empty());

    store.despawn(entity).unwrap();
    assert_eq!(*despawned.lock().unwrap(), [entity]);
}

// 8 ----------------------------------------------------------------- queried
#[test]
fn it_can_be_queried() {
    let mut store = store();
    let other_type = EntityTypeId::parse("nexora:entity/other").unwrap();

    let a = spawn_test_entity(&mut store, WorldPosition::new(0.0, 64.0, 0.0));
    let b = spawn_test_entity(&mut store, WorldPosition::new(3.0, 64.0, 0.0));
    let far = spawn_test_entity(&mut store, WorldPosition::new(100.0, 64.0, 0.0));
    let different = store
        .spawn(SpawnContext::new(
            other_type.clone(),
            WorldPosition::ORIGIN,
            SpawnReason::Mod,
        ))
        .unwrap();

    // By type.
    let mut by_type = Query::by_type(&store, test_type());
    by_type.sort();
    let mut expected = vec![a, b, far];
    expected.sort();
    assert_eq!(by_type, expected);
    assert_eq!(Query::by_type(&store, other_type), vec![different]);

    // By tag.
    assert_eq!(Query::by_tag(&store, tag("nexora:tag/test")).len(), 3);
    assert!(Query::by_tag(&store, tag("nexora:tag/absent")).is_empty());

    // By radius, nearest first.
    let nearby = Query::within_radius(
        &store,
        WorldPosition::new(0.0, 64.0, 0.0),
        5.0,
        &EntityFilter::new(),
    );
    let order: Vec<_> = nearby.iter().map(|hit| hit.entity).collect();
    // `a` sits exactly on the query point, so it must come first.
    assert_eq!(order.first(), Some(&a));
    assert!(order.contains(&b), "b is 3 blocks away, inside the radius");
    assert!(
        !order.contains(&far),
        "the distant entity is outside the radius"
    );
    // `different` is at the world origin, 64 blocks below the query point.
    assert!(
        !order.contains(&different),
        "an entity outside the radius must not be returned"
    );
    assert!(nearby[0].distance.abs() < 1e-12);
    // Distances ascend.
    for pair in nearby.windows(2) {
        assert!(pair[0].distance <= pair[1].distance);
    }

    // By chunk.
    let shape = ChunkShape::cubic_default();
    let origin_chunk = Query::in_chunk(
        &store,
        shape,
        nexora_foundation::spatial::ChunkCoord::new(0, 0),
        &EntityFilter::new(),
    );
    assert!(origin_chunk.contains(&a) && !origin_chunk.contains(&far));

    // Composed filters.
    assert_eq!(
        Query::count(
            &store,
            &EntityFilter::new()
                .of_type(test_type())
                .with_tag(tag("nexora:tag/test"))
        ),
        3
    );
}

// 9 -------------------------------------------------------------- save/load
#[test]
fn it_saves_and_loads_keeping_its_persistent_identity() {
    let mut original = store();
    let alpha = store_entity(
        &mut original,
        WorldPosition::new(1.5, 64.0, -2.5),
        "nexora:tag/alpha",
    );
    let beta = store_entity(
        &mut original,
        WorldPosition::new(-40.0, 12.0, 900.0),
        "nexora:tag/beta",
    );
    original
        .set_velocity(alpha, Velocity::new(0.5, 0.0, -0.25).unwrap())
        .unwrap();

    let alpha_pid = original.persistent_id(alpha).unwrap();
    let beta_pid = original.persistent_id(beta).unwrap();

    let mut container = SaveContainer::new();
    let saved = persist::save(&original, &mut container).unwrap();
    assert_eq!(saved, 2);

    // Through real bytes, not just the in-memory container.
    let encoded = container.encode();
    let reopened = SaveContainer::decode(&encoded).unwrap();

    let mut restored = store();
    let count = persist::load(&mut restored, &reopened).unwrap();
    assert_eq!(count, 2);
    assert_eq!(restored.len(), 2);

    // Persistent ids survived; session handles are new.
    let alpha_again = restored
        .resolve_persistent(alpha_pid)
        .expect("alpha came back");
    let beta_again = restored
        .resolve_persistent(beta_pid)
        .expect("beta came back");

    assert_eq!(
        restored.transform(alpha_again).unwrap().position,
        WorldPosition::new(1.5, 64.0, -2.5)
    );
    assert_eq!(
        restored.transform(beta_again).unwrap().position,
        WorldPosition::new(-40.0, 12.0, 900.0)
    );
    assert_eq!(
        restored.velocity(alpha_again).unwrap(),
        Velocity::new(0.5, 0.0, -0.25).unwrap()
    );
    assert!(restored
        .tags(alpha_again)
        .unwrap()
        .contains(&tag("nexora:tag/alpha")));
    assert_eq!(restored.entity_type(beta_again).unwrap(), test_type());

    // A new spawn cannot collide with a restored id.
    let fresh = spawn_test_entity(&mut restored, WorldPosition::ORIGIN);
    assert!(restored.persistent_id(fresh).unwrap().0 > alpha_pid.0.max(beta_pid.0));
}

#[test]
fn temporary_entities_do_not_come_back() {
    let mut original = store();
    let permanent = spawn_test_entity(&mut original, WorldPosition::new(1.0, 1.0, 1.0));
    original
        .spawn(
            SpawnContext::new(
                test_type(),
                WorldPosition::new(2.0, 2.0, 2.0),
                SpawnReason::Spawner,
            )
            .with_policy(PersistencePolicy::Temporary),
        )
        .unwrap();
    assert_eq!(original.len(), 2);

    let mut container = SaveContainer::new();
    let saved = persist::save(&original, &mut container).unwrap();
    assert_eq!(saved, 1, "only the persistent entity is written");

    let mut restored = store();
    persist::load(&mut restored, &container).unwrap();
    assert_eq!(restored.len(), 1);

    let survivor = restored.resolve_persistent(original.persistent_id(permanent).unwrap());
    assert!(survivor.is_some(), "the persistent entity survived");
}

#[test]
fn a_world_saved_without_entities_still_loads() {
    // A save written before this system existed has no entity section at all.
    let container = SaveContainer::new();
    let mut restored = store();
    assert_eq!(persist::load(&mut restored, &container).unwrap(), 0);
    assert!(restored.is_empty());
}

#[test]
fn a_corrupt_entity_section_is_refused_rather_than_partially_loaded() {
    let mut original = store();
    spawn_test_entity(&mut original, WorldPosition::new(1.0, 1.0, 1.0));
    let mut container = SaveContainer::new();
    persist::save(&original, &mut container).unwrap();

    let section = Identifier::parse(persist::SECTION_ENTITIES).unwrap();
    let good = container.get(&section).unwrap().to_vec();

    // Every truncation of the section must be refused.
    for cut in 0..good.len() {
        let mut damaged = SaveContainer::new();
        damaged.put(section.clone(), good[..cut].to_vec());
        let mut restored = store();
        assert!(
            persist::load(&mut restored, &damaged).is_err(),
            "a section truncated to {cut} bytes loaded anyway"
        );
    }
}

fn store_entity(
    store: &mut EntityStore,
    at: WorldPosition,
    tag_name: &str,
) -> nexora_entity::EntityId {
    store
        .spawn(
            SpawnContext::new(test_type(), at, SpawnReason::WorldGen)
                .with_bounds(Bounds::new(0.3, 0.8, 0.3).unwrap())
                .with_tags(TagSet::from_iter_sorted([tag(tag_name)])),
        )
        .expect("spawn")
}

// 10 ------------------------------------------------------------ destruction
#[test]
fn it_can_be_destroyed() {
    let mut store = store();
    let entity = spawn_test_entity(&mut store, WorldPosition::ORIGIN);
    let persistent = store.persistent_id(entity).unwrap();

    store.despawn(entity).expect("despawn");

    assert!(store.is_empty());
    assert!(!store.contains(entity));
    assert!(store.resolve_persistent(persistent).is_none());
    assert!(store.transform(entity).is_err());
    assert!(store.despawn(entity).is_err(), "despawning twice must fail");
    assert!(Query::matching(&store, &EntityFilter::new()).is_empty());
}

// ------------------------------------------------------------ full lifecycle
#[test]
fn the_whole_cycle_holds_together() {
    let bus = EventBus::new();
    let seen = Arc::new(Mutex::new(0usize));
    let sink = seen.clone();
    bus.subscribe::<EntitySpawned, _>(tag("nexora:system/audit"), Priority::Normal, move |_, _| {
        *sink.lock().unwrap() += 1;
    });

    let mut store = EntityStore::new(world()).with_event_bus(bus);

    // Populate, move, query, save, wipe, reload, verify.
    for index in 0..64u32 {
        let at = WorldPosition::new(f64::from(index), 64.0, f64::from(index % 8));
        let entity = spawn_test_entity(&mut store, at);
        store
            .set_velocity(entity, Velocity::new(0.1, 0.0, 0.0).unwrap())
            .unwrap();
    }
    assert_eq!(*seen.lock().unwrap(), 64);

    assert_eq!(
        store
            .step(WorldDuration::from_ticks(20), TICKS_PER_SECOND)
            .unwrap(),
        64
    );

    let mut container = SaveContainer::new();
    assert_eq!(persist::save(&store, &mut container).unwrap(), 64);

    let mut restored = self::store();
    persist::load(
        &mut restored,
        &SaveContainer::decode(&container.encode()).unwrap(),
    )
    .unwrap();
    assert_eq!(restored.len(), 64);

    // The entity that started at x=0 moved to x=0.1 before saving.
    let nearest = Query::within_radius(
        &restored,
        WorldPosition::new(0.1, 64.0, 0.0),
        0.001,
        &EntityFilter::new(),
    );
    assert_eq!(
        nearest.len(),
        1,
        "the moved position survived the round trip"
    );
}
