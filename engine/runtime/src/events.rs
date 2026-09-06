//! Event bus.
//!
//! Implements `Event Bus.md`. Its §3 distinction is the one that shapes this
//! module: an **event is a fact that already happened**, not a request. Facts
//! are therefore not cancelable here, and handlers observe rather than veto.
//! Request/response events (§44-§45) are a separate mechanism and are
//! deliberately not implemented yet - see the Phase 0 scope note in
//! `docs/adr/ADR-0005-phase-0-scope.md`.
//!
//! Two protections come from the same document:
//!
//! * **Correlation and causation ids** (§9-§10) so a chain of consequences can
//!   be reconstructed after the fact.
//! * **A depth limit** (§54-§55) so that an event which re-triggers itself
//!   fails loudly instead of recursing until the stack dies.

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use nexora_foundation::diagnostics::CorrelationId;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::time::WorldTime;

/// Maximum nesting of event dispatch before the bus refuses to go deeper.
pub const MAX_EVENT_DEPTH: u32 = 32;

thread_local! {
    /// Dispatch depth for the current thread.
    ///
    /// Handlers run synchronously on the publishing thread, so depth is a
    /// per-thread property; a shared counter would make one thread's nesting
    /// throttle another's.
    static DISPATCH_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Unique id of one published event instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(pub u64);

/// Handler ordering within one event type.
///
/// `Event Bus.md` §40 warns that priority must not become an invisible
/// dependency: if a handler only works because another ran first, that is a
/// missing explicit relationship, not a tuning problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Runs before everything else. Reserved for engine bookkeeping.
    First,
    /// Runs early.
    High,
    /// The default.
    Normal,
    /// Runs late.
    Low,
    /// Runs after everything else. Reserved for observers and telemetry.
    Last,
}

/// A handle used to unsubscribe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubscriptionHandle(pub u64);

/// Context carried alongside a published event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventContext {
    /// Identity of this event instance.
    pub event_id: EventId,
    /// The chain this event belongs to.
    pub correlation: CorrelationId,
    /// The event that caused this one, if any.
    pub causation: Option<EventId>,
    /// Nesting depth at dispatch time; zero for a top-level publish.
    pub depth: u32,
    /// World time at dispatch, when a world is attached.
    pub world_time: Option<WorldTime>,
}

/// A fact that has already happened.
pub trait Event: Any + Send + Sync + 'static {
    /// The event's stable type identifier, e.g. `nexora:event/chunk_loaded`.
    fn event_type(&self) -> Identifier;
}

/// What one publish call did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReport {
    /// Identity assigned to the published event.
    pub event_id: EventId,
    /// How many handlers were invoked.
    pub delivered: usize,
}

type Callback = Arc<dyn Fn(&dyn Any, &EventContext) + Send + Sync>;

#[derive(Clone)]
struct Subscriber {
    handle: SubscriptionHandle,
    owner: Identifier,
    priority: Priority,
    sequence: u64,
    callback: Callback,
}

/// A synchronous, typed publish/subscribe bus.
///
/// Cloning shares the same subscriber set, so systems can each hold a handle
/// without a global.
#[derive(Clone)]
pub struct EventBus {
    inner: Arc<Inner>,
}

struct Inner {
    subscribers: RwLock<BTreeMap<TypeId, Vec<Subscriber>>>,
    next_handle: AtomicU64,
    next_event_id: AtomicU64,
    next_correlation: AtomicU64,
    world_time: RwLock<Option<WorldTime>>,
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let types = self
            .inner
            .subscribers
            .read()
            .map(|map| map.len())
            .unwrap_or(0);
        f.debug_struct("EventBus")
            .field("event_types", &types)
            .finish()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Create an empty bus.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                subscribers: RwLock::new(BTreeMap::new()),
                next_handle: AtomicU64::new(1),
                next_event_id: AtomicU64::new(1),
                next_correlation: AtomicU64::new(1),
                world_time: RwLock::new(None),
            }),
        }
    }

    /// Set the world time stamped onto subsequently published events.
    pub fn set_world_time(&self, time: Option<WorldTime>) {
        if let Ok(mut slot) = self.inner.world_time.write() {
            *slot = time;
        }
    }

    /// Subscribe to one event type.
    ///
    /// `owner` identifies the subscribing system, so a leaked subscription can
    /// be traced back to whoever registered it.
    pub fn subscribe<E, F>(
        &self,
        owner: Identifier,
        priority: Priority,
        handler: F,
    ) -> SubscriptionHandle
    where
        E: Event,
        F: Fn(&E, &EventContext) + Send + Sync + 'static,
    {
        let handle = SubscriptionHandle(self.inner.next_handle.fetch_add(1, Ordering::Relaxed));
        let sequence = handle.0;

        let callback: Callback = Arc::new(move |any: &dyn Any, context: &EventContext| {
            // The map is keyed by TypeId, so this downcast cannot legitimately
            // fail; ignoring a mismatch is safer than panicking mid-dispatch.
            if let Some(event) = any.downcast_ref::<E>() {
                handler(event, context);
            }
        });

        let subscriber = Subscriber {
            handle,
            owner,
            priority,
            sequence,
            callback,
        };

        if let Ok(mut subscribers) = self.inner.subscribers.write() {
            let list = subscribers.entry(TypeId::of::<E>()).or_default();
            list.push(subscriber);
            // Deterministic dispatch order: priority first, then subscription order.
            list.sort_by(|a, b| {
                a.priority
                    .cmp(&b.priority)
                    .then(a.sequence.cmp(&b.sequence))
            });
        }
        handle
    }

    /// Remove a subscription. Returns whether it existed.
    pub fn unsubscribe(&self, handle: SubscriptionHandle) -> bool {
        let Ok(mut subscribers) = self.inner.subscribers.write() else {
            return false;
        };
        for list in subscribers.values_mut() {
            if let Some(position) = list.iter().position(|s| s.handle == handle) {
                list.remove(position);
                return true;
            }
        }
        false
    }

    /// How many handlers are subscribed to an event type.
    #[must_use]
    pub fn subscriber_count<E: Event>(&self) -> usize {
        self.inner
            .subscribers
            .read()
            .ok()
            .and_then(|map| map.get(&TypeId::of::<E>()).map(Vec::len))
            .unwrap_or(0)
    }

    /// The owners currently subscribed to an event type, for diagnostics.
    #[must_use]
    pub fn subscribers_of<E: Event>(&self) -> Vec<Identifier> {
        self.inner
            .subscribers
            .read()
            .ok()
            .and_then(|map| {
                map.get(&TypeId::of::<E>())
                    .map(|list| list.iter().map(|s| s.owner.clone()).collect())
            })
            .unwrap_or_default()
    }

    /// Allocate a fresh correlation id for a new chain of consequences.
    #[must_use]
    pub fn new_correlation(&self) -> CorrelationId {
        CorrelationId(self.inner.next_correlation.fetch_add(1, Ordering::Relaxed))
    }

    /// Publish an event as the start of a new chain.
    ///
    /// # Errors
    ///
    /// Returns an error when the dispatch depth limit is exceeded.
    pub fn publish<E: Event>(&self, event: &E) -> Result<PublishReport> {
        let correlation = self.new_correlation();
        self.dispatch(event, correlation, None)
    }

    /// Publish an event caused by another.
    ///
    /// The new event inherits the cause's correlation id and records it as its
    /// causation, which is what makes a consequence chain reconstructable.
    ///
    /// # Errors
    ///
    /// Returns an error when the dispatch depth limit is exceeded.
    pub fn publish_caused_by<E: Event>(
        &self,
        event: &E,
        cause: &EventContext,
    ) -> Result<PublishReport> {
        self.dispatch(event, cause.correlation, Some(cause.event_id))
    }

    fn dispatch<E: Event>(
        &self,
        event: &E,
        correlation: CorrelationId,
        causation: Option<EventId>,
    ) -> Result<PublishReport> {
        let depth = DISPATCH_DEPTH.with(Cell::get);
        if depth >= MAX_EVENT_DEPTH {
            return Err(Error::new(
                Domain::Event,
                "event-bus",
                "event dispatch exceeded the maximum depth; this is an event loop",
            )
            .with_recovery(Recovery::Reject)
            .with_context("event_type", event.event_type().to_string())
            .with_context("depth", depth.to_string())
            .with_context("limit", MAX_EVENT_DEPTH.to_string()));
        }

        let context = EventContext {
            event_id: EventId(self.inner.next_event_id.fetch_add(1, Ordering::Relaxed)),
            correlation,
            causation,
            depth,
            world_time: self.inner.world_time.read().ok().and_then(|slot| *slot),
        };

        // Snapshot the handler list and release the lock before invoking any of
        // them: a handler is allowed to publish, subscribe or unsubscribe, and
        // holding the lock across the call would deadlock the moment one does.
        let handlers: Vec<Callback> = self
            .inner
            .subscribers
            .read()
            .ok()
            .and_then(|map| {
                map.get(&TypeId::of::<E>())
                    .map(|list| list.iter().map(|s| s.callback.clone()).collect())
            })
            .unwrap_or_default();

        DISPATCH_DEPTH.with(|cell| cell.set(depth + 1));
        for handler in &handlers {
            handler(event as &dyn Any, &context);
        }
        DISPATCH_DEPTH.with(|cell| cell.set(depth));

        Ok(PublishReport {
            event_id: context.event_id,
            delivered: handlers.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Debug)]
    struct ChunkLoaded {
        x: i64,
        z: i64,
    }

    impl Event for ChunkLoaded {
        fn event_type(&self) -> Identifier {
            Identifier::parse("nexora:event/chunk_loaded").expect("valid")
        }
    }

    #[derive(Debug)]
    struct BlockPlaced;

    impl Event for BlockPlaced {
        fn event_type(&self) -> Identifier {
            Identifier::parse("nexora:event/block_placed").expect("valid")
        }
    }

    fn owner(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("valid owner")
    }

    #[test]
    fn handlers_receive_the_typed_event() {
        let bus = EventBus::new();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();

        bus.subscribe::<ChunkLoaded, _>(
            owner("nexora:system/lighting"),
            Priority::Normal,
            move |event, _| {
                sink.lock().expect("lock").push((event.x, event.z));
            },
        );

        let report = bus.publish(&ChunkLoaded { x: 4, z: -7 }).unwrap();
        assert_eq!(report.delivered, 1);
        assert_eq!(*seen.lock().unwrap(), [(4, -7)]);
    }

    #[test]
    fn events_are_routed_only_to_their_own_type() {
        let bus = EventBus::new();
        let chunk_hits = Arc::new(Mutex::new(0usize));
        let block_hits = Arc::new(Mutex::new(0usize));

        let sink = chunk_hits.clone();
        bus.subscribe::<ChunkLoaded, _>(owner("nexora:system/a"), Priority::Normal, move |_, _| {
            *sink.lock().unwrap() += 1;
        });
        let sink = block_hits.clone();
        bus.subscribe::<BlockPlaced, _>(owner("nexora:system/b"), Priority::Normal, move |_, _| {
            *sink.lock().unwrap() += 1;
        });

        bus.publish(&ChunkLoaded { x: 0, z: 0 }).unwrap();
        assert_eq!(*chunk_hits.lock().unwrap(), 1);
        assert_eq!(*block_hits.lock().unwrap(), 0);
    }

    #[test]
    fn dispatch_order_follows_priority_then_subscription_order() {
        let bus = EventBus::new();
        let order = Arc::new(Mutex::new(Vec::new()));

        for (label, priority) in [
            ("late", Priority::Last),
            ("normal-first", Priority::Normal),
            ("early", Priority::First),
            ("normal-second", Priority::Normal),
        ] {
            let sink = order.clone();
            bus.subscribe::<ChunkLoaded, _>(owner("nexora:system/x"), priority, move |_, _| {
                sink.lock().unwrap().push(label);
            });
        }

        bus.publish(&ChunkLoaded { x: 0, z: 0 }).unwrap();
        assert_eq!(
            *order.lock().unwrap(),
            ["early", "normal-first", "normal-second", "late"]
        );
    }

    #[test]
    fn causation_chains_share_a_correlation_id() {
        let bus = EventBus::new();
        let observed = Arc::new(Mutex::new(Vec::new()));

        // A handler that reacts to one fact by publishing another.
        let inner_bus = bus.clone();
        bus.subscribe::<ChunkLoaded, _>(
            owner("nexora:system/spawner"),
            Priority::Normal,
            move |_, context| {
                inner_bus
                    .publish_caused_by(&BlockPlaced, context)
                    .expect("nested publish");
            },
        );

        let sink = observed.clone();
        bus.subscribe::<BlockPlaced, _>(
            owner("nexora:system/audit"),
            Priority::Normal,
            move |_, context| {
                sink.lock()
                    .unwrap()
                    .push((context.correlation, context.causation, context.depth));
            },
        );

        let report = bus.publish(&ChunkLoaded { x: 1, z: 1 }).unwrap();
        let recorded = observed.lock().unwrap().clone();
        assert_eq!(recorded.len(), 1);

        let (correlation, causation, depth) = recorded[0];
        assert_eq!(
            causation,
            Some(report.event_id),
            "the consequence must name its cause"
        );
        assert_eq!(depth, 1, "the nested event is one level deep");
        // Both events belong to the same chain.
        assert_eq!(correlation.0, 1);
    }

    #[test]
    fn an_event_loop_is_refused_instead_of_overflowing_the_stack() {
        let bus = EventBus::new();
        let failures = Arc::new(Mutex::new(0usize));

        let inner_bus = bus.clone();
        let counter = failures.clone();
        // A handler that republishes its own event type: an infinite loop.
        bus.subscribe::<ChunkLoaded, _>(
            owner("nexora:system/loop"),
            Priority::Normal,
            move |event, context| {
                if inner_bus
                    .publish_caused_by(
                        &ChunkLoaded {
                            x: event.x,
                            z: event.z,
                        },
                        context,
                    )
                    .is_err()
                {
                    *counter.lock().unwrap() += 1;
                }
            },
        );

        bus.publish(&ChunkLoaded { x: 0, z: 0 })
            .expect("the outermost publish succeeds");
        // The recursion stopped at the limit rather than exhausting the stack.
        assert_eq!(*failures.lock().unwrap(), 1);
    }

    #[test]
    fn unsubscribing_stops_delivery() {
        let bus = EventBus::new();
        let hits = Arc::new(Mutex::new(0usize));
        let sink = hits.clone();

        let handle = bus.subscribe::<ChunkLoaded, _>(
            owner("nexora:system/temp"),
            Priority::Normal,
            move |_, _| {
                *sink.lock().unwrap() += 1;
            },
        );

        bus.publish(&ChunkLoaded { x: 0, z: 0 }).unwrap();
        assert_eq!(bus.subscriber_count::<ChunkLoaded>(), 1);

        assert!(bus.unsubscribe(handle));
        assert!(!bus.unsubscribe(handle), "unsubscribing twice is a no-op");
        assert_eq!(bus.subscriber_count::<ChunkLoaded>(), 0);

        bus.publish(&ChunkLoaded { x: 0, z: 0 }).unwrap();
        assert_eq!(*hits.lock().unwrap(), 1, "no delivery after unsubscribe");
    }

    #[test]
    fn publishing_with_no_subscribers_is_not_an_error() {
        let bus = EventBus::new();
        let report = bus.publish(&ChunkLoaded { x: 0, z: 0 }).unwrap();
        assert_eq!(report.delivered, 0);
    }

    #[test]
    fn world_time_is_stamped_onto_events() {
        let bus = EventBus::new();
        let stamped = Arc::new(Mutex::new(None));
        let sink = stamped.clone();

        bus.subscribe::<ChunkLoaded, _>(
            owner("nexora:system/clock"),
            Priority::Normal,
            move |_, context| {
                *sink.lock().unwrap() = context.world_time;
            },
        );

        bus.set_world_time(Some(WorldTime(4_242)));
        bus.publish(&ChunkLoaded { x: 0, z: 0 }).unwrap();
        assert_eq!(*stamped.lock().unwrap(), Some(WorldTime(4_242)));
    }

    #[test]
    fn subscribers_can_be_listed_by_owner() {
        let bus = EventBus::new();
        bus.subscribe::<ChunkLoaded, _>(
            owner("nexora:system/lighting"),
            Priority::Normal,
            |_, _| {},
        );
        bus.subscribe::<ChunkLoaded, _>(owner("example:system/minimap"), Priority::Low, |_, _| {});

        let owners: Vec<String> = bus
            .subscribers_of::<ChunkLoaded>()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(owners, ["nexora:system/lighting", "example:system/minimap"]);
    }

    #[test]
    fn a_handler_may_subscribe_during_dispatch_without_deadlocking() {
        let bus = EventBus::new();
        let inner_bus = bus.clone();

        bus.subscribe::<ChunkLoaded, _>(
            owner("nexora:system/late-binder"),
            Priority::Normal,
            move |_, _| {
                inner_bus.subscribe::<BlockPlaced, _>(
                    owner("nexora:system/added-later"),
                    Priority::Normal,
                    |_, _| {},
                );
            },
        );

        bus.publish(&ChunkLoaded { x: 0, z: 0 }).unwrap();
        assert_eq!(bus.subscriber_count::<BlockPlaced>(), 1);
    }
}
