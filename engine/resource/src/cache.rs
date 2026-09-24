//! A resource cache bounded in bytes.
//!
//! `NEXORA MEMORY AND RESOURCE OWNERSHIP.md`, *Asset cache*: caches must have
//! `capacity · priority · last-use · pinning · eviction policy · telemetry`.
//! Each of those is a field or a counter here, and each has a test.
//!
//! # Eviction
//!
//! When an insert would exceed the capacity, unpinned entries are evicted
//! lowest priority first and, within a priority, least recently used first.
//! "Least recently used" is a counter of cache operations rather than a clock,
//! so the order is deterministic and a test can assert it.
//!
//! # What eviction does not do
//!
//! It does not take a value away from anyone. Values are held as `Arc`; the
//! cache drops *its* reference, and a caller still holding one keeps a valid
//! value. That is what `RESOURCE AND ASSET SYSTEM.md` means by *"cache data is
//! disposable"* — disposable to the cache, not to its users.
//!
//! # When pinned entries fill the budget
//!
//! The insert is refused with [`Recovery::Retry`]. The alternative — letting
//! the cache grow past its capacity — would make "bounded" a suggestion, and a
//! budget that can be exceeded silently is not a budget.

use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;

use nexora_foundation::diagnostics::Counters;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;

/// How much a resource is worth keeping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Priority {
    /// First to go.
    Low,
    /// The default.
    #[default]
    Normal,
    /// Last to go among unpinned entries.
    High,
}

/// A cached value, type-erased.
pub type Shared = Arc<dyn Any + Send + Sync>;

#[derive(Debug)]
struct Slot {
    value: Shared,
    cost: u64,
    priority: Priority,
    last_use: u64,
    pinned: bool,
}

/// What the cache has done, for telemetry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    /// Lookups that found a value.
    pub hits: u64,
    /// Lookups that did not.
    pub misses: u64,
    /// Values inserted.
    pub inserts: u64,
    /// Values evicted to make room.
    pub evictions: u64,
    /// Inserts refused because pinned values filled the budget.
    pub refused: u64,
    /// Bytes held now.
    pub bytes: u64,
    /// The most bytes ever held at once.
    pub high_water: u64,
    /// Values held now.
    pub entries: usize,
}

/// Holds loaded resources within a byte budget.
#[derive(Debug)]
pub struct ResourceCache {
    capacity: u64,
    clock: u64,
    slots: BTreeMap<Identifier, Slot>,
    stats: CacheStats,
}

impl ResourceCache {
    /// A cache that holds at most `capacity` bytes.
    #[must_use]
    pub fn new(capacity: u64) -> Self {
        Self {
            capacity,
            clock: 0,
            slots: BTreeMap::new(),
            stats: CacheStats::default(),
        }
    }

    /// The budget, in bytes.
    #[must_use]
    pub const fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Look a value up, marking it used.
    pub fn get(&mut self, id: &Identifier) -> Option<Shared> {
        self.clock += 1;
        let now = self.clock;
        match self.slots.get_mut(id) {
            Some(slot) => {
                slot.last_use = now;
                self.stats.hits += 1;
                Some(Arc::clone(&slot.value))
            }
            None => {
                self.stats.misses += 1;
                None
            }
        }
    }

    /// Whether a value is held, without marking it used.
    #[must_use]
    pub fn contains(&self, id: &Identifier) -> bool {
        self.slots.contains_key(id)
    }

    /// Hold a value, evicting others if needed.
    ///
    /// # Errors
    ///
    /// Returns an error when the value alone is larger than the capacity, or
    /// when pinned values leave no room for it. Nothing is evicted in either
    /// case: a refused insert must not have cost the cache anything.
    pub fn insert(
        &mut self,
        id: Identifier,
        value: Shared,
        cost: u64,
        priority: Priority,
    ) -> Result<()> {
        if cost > self.capacity {
            self.stats.refused += 1;
            return Err(full("the resource is larger than the whole cache budget")
                .with_context("resource", id.to_string())
                .with_context("cost", cost.to_string())
                .with_context("capacity", self.capacity.to_string()));
        }
        // Replacing a value frees its own cost first.
        let previous = self.slots.get(&id).map_or(0, |slot| slot.cost);
        let needed = (self.stats.bytes - previous + cost).saturating_sub(self.capacity);

        // Choose victims before touching anything, so a refusal evicts nothing.
        let mut candidates: Vec<(&Identifier, &Slot)> = self
            .slots
            .iter()
            .filter(|(key, slot)| !slot.pinned && *key != &id)
            .collect();
        candidates.sort_by_key(|(_, slot)| (slot.priority, slot.last_use));
        let mut freed = 0;
        let mut victims = Vec::new();
        for (key, slot) in candidates {
            if freed >= needed {
                break;
            }
            freed += slot.cost;
            victims.push(key.clone());
        }
        if freed < needed {
            self.stats.refused += 1;
            return Err(full("pinned resources leave no room in the cache budget")
                .with_context("resource", id.to_string())
                .with_context("needed", needed.to_string())
                .with_context("evictable", freed.to_string()));
        }

        for victim in victims {
            if let Some(slot) = self.slots.remove(&victim) {
                self.stats.bytes -= slot.cost;
                self.stats.evictions += 1;
            }
        }
        self.clock += 1;
        let pinned = self.slots.get(&id).is_some_and(|slot| slot.pinned);
        if let Some(old) = self.slots.insert(
            id,
            Slot {
                value,
                cost,
                priority,
                last_use: self.clock,
                pinned,
            },
        ) {
            self.stats.bytes -= old.cost;
        }
        self.stats.bytes += cost;
        self.stats.inserts += 1;
        self.stats.high_water = self.stats.high_water.max(self.stats.bytes);
        self.stats.entries = self.slots.len();
        Ok(())
    }

    /// Keep a value regardless of pressure. Returns whether it was held.
    pub fn pin(&mut self, id: &Identifier) -> bool {
        self.set_pinned(id, true)
    }

    /// Let a pinned value be evicted again. Returns whether it was held.
    pub fn unpin(&mut self, id: &Identifier) -> bool {
        self.set_pinned(id, false)
    }

    fn set_pinned(&mut self, id: &Identifier, pinned: bool) -> bool {
        self.slots.get_mut(id).is_some_and(|slot| {
            slot.pinned = pinned;
            true
        })
    }

    /// Drop a value, pinned or not. Returns whether it was held.
    pub fn remove(&mut self, id: &Identifier) -> bool {
        match self.slots.remove(id) {
            Some(slot) => {
                self.stats.bytes -= slot.cost;
                self.stats.entries = self.slots.len();
                true
            }
            None => false,
        }
    }

    /// What the cache has done so far.
    #[must_use]
    pub const fn stats(&self) -> CacheStats {
        self.stats
    }

    /// Publish the counters under `resource.cache.*`.
    ///
    /// `NEXORA MEMORY AND RESOURCE OWNERSHIP.md` *Debug*: every large pool
    /// exposes counters, high-water marks and budget pressure. Counters are
    /// monotonic, so this adds what happened since the last publish.
    pub fn publish(&self, counters: &Counters, since: CacheStats) {
        let now = self.stats;
        counters.add("resource.cache.hits", now.hits - since.hits);
        counters.add("resource.cache.misses", now.misses - since.misses);
        counters.add("resource.cache.inserts", now.inserts - since.inserts);
        counters.add("resource.cache.evictions", now.evictions - since.evictions);
        counters.add("resource.cache.refused", now.refused - since.refused);
    }
}

fn full(message: &'static str) -> Error {
    Error::new(Domain::Content, "resource-cache", message).with_recovery(Recovery::Retry)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).unwrap()
    }

    fn value(n: u32) -> Shared {
        Arc::new(n)
    }

    #[test]
    fn the_budget_holds_and_the_least_valuable_go_first() {
        let mut cache = ResourceCache::new(30);
        cache
            .insert(id("nexora:data/a"), value(1), 10, Priority::Normal)
            .unwrap();
        cache
            .insert(id("nexora:data/b"), value(2), 10, Priority::High)
            .unwrap();
        cache
            .insert(id("nexora:data/c"), value(3), 10, Priority::Normal)
            .unwrap();
        // Touch a: now c is the least recently used among the normal ones.
        assert!(cache.get(&id("nexora:data/a")).is_some());

        cache
            .insert(id("nexora:data/d"), value(4), 10, Priority::Normal)
            .unwrap();
        assert!(
            !cache.contains(&id("nexora:data/c")),
            "least recently used goes"
        );
        assert!(cache.contains(&id("nexora:data/a")));
        assert!(cache.contains(&id("nexora:data/b")));

        cache
            .insert(id("nexora:data/e"), value(5), 20, Priority::Low)
            .unwrap();
        assert!(
            cache.contains(&id("nexora:data/b")),
            "high priority stays longest"
        );
        assert_eq!(cache.stats().bytes, 30);
        assert!(cache.stats().bytes <= cache.capacity());
        assert_eq!(cache.stats().evictions, 3);
        assert_eq!(cache.stats().high_water, 30);
    }

    #[test]
    fn pinned_values_are_never_evicted_and_a_full_pin_set_refuses_cleanly() {
        let mut cache = ResourceCache::new(20);
        cache
            .insert(id("nexora:data/a"), value(1), 10, Priority::Low)
            .unwrap();
        cache
            .insert(id("nexora:data/b"), value(2), 10, Priority::Low)
            .unwrap();
        assert!(cache.pin(&id("nexora:data/a")));
        assert!(cache.pin(&id("nexora:data/b")));

        let err = cache
            .insert(id("nexora:data/c"), value(3), 5, Priority::High)
            .expect_err("no room");
        assert_eq!(err.recovery(), Recovery::Retry);
        assert!(cache.contains(&id("nexora:data/a")) && cache.contains(&id("nexora:data/b")));
        assert_eq!(cache.stats().refused, 1);
        assert_eq!(cache.stats().evictions, 0, "a refusal evicts nothing");

        cache.unpin(&id("nexora:data/a"));
        cache
            .insert(id("nexora:data/c"), value(3), 5, Priority::High)
            .unwrap();
        assert!(!cache.contains(&id("nexora:data/a")));
    }

    #[test]
    fn eviction_never_takes_a_value_away_from_its_holder() {
        let mut cache = ResourceCache::new(10);
        cache
            .insert(id("nexora:data/a"), value(7), 10, Priority::Normal)
            .unwrap();
        let held = cache.get(&id("nexora:data/a")).unwrap();
        cache
            .insert(id("nexora:data/b"), value(8), 10, Priority::Normal)
            .unwrap();
        assert!(!cache.contains(&id("nexora:data/a")));
        assert_eq!(held.downcast_ref::<u32>(), Some(&7));
    }

    #[test]
    fn replacing_a_value_counts_its_bytes_once_and_keeps_its_pin() {
        let mut cache = ResourceCache::new(10);
        cache
            .insert(id("nexora:data/a"), value(1), 6, Priority::Normal)
            .unwrap();
        cache.pin(&id("nexora:data/a"));
        cache
            .insert(id("nexora:data/a"), value(2), 8, Priority::Normal)
            .unwrap();
        assert_eq!(cache.stats().bytes, 8);
        assert_eq!(cache.stats().entries, 1);
        let err = cache
            .insert(id("nexora:data/b"), value(3), 5, Priority::Normal)
            .expect_err("a is still pinned");
        assert!(err.to_string().contains("pinned"), "{err}");

        let too_big = cache.insert(id("nexora:data/c"), value(4), 11, Priority::High);
        assert!(too_big.is_err());
    }

    #[test]
    fn telemetry_reaches_the_counters() {
        let counters = Counters::new();
        let mut cache = ResourceCache::new(10);
        let start = cache.stats();
        cache
            .insert(id("nexora:data/a"), value(1), 10, Priority::Normal)
            .unwrap();
        cache.get(&id("nexora:data/a"));
        cache.get(&id("nexora:data/b"));
        cache.publish(&counters, start);
        assert_eq!(counters.get("resource.cache.hits"), 1);
        assert_eq!(counters.get("resource.cache.misses"), 1);
        assert_eq!(counters.get("resource.cache.inserts"), 1);
    }
}
