//! Memory budgets, one per pool, in one ledger.
//!
//! `NEXORA MEMORY AND RESOURCE OWNERSHIP.md` asks for three things this module
//! turns into types:
//!
//! * **Budgets per subsystem**, in the classes the document names
//!   ([`MemoryClass`]).
//! * **Debug**: *"cada grande pool deve expor contadores, high-water marks,
//!   leaks suspeitos e pressão de orçamento"* — [`PoolReport`] carries each of
//!   those four.
//! * `NEXORA PERFORMANCE BUDGETS.md`: every major system publishes
//!   `TARGET / WARNING / CRITICAL / EMERGENCY` ([`MemoryBudget`]), and *"a
//!   subsystem must not consume another subsystem's budget invisibly"* — which
//!   is what a single [`MemoryLedger`] with one named pool per owner is for.
//!
//! # What a pool is, and what it is not
//!
//! A pool is an **account**, not an allocator. The owner measures what it holds
//! and [`MemoryPool::record`]s the total; the pool keeps the high-water mark,
//! the worst pressure it has seen, and how often the owner refused work to stay
//! inside it. Routing every allocation through a pool would put an atomic on
//! every hot-path allocation to learn a number the owner already knows.
//!
//! Because the owner records a total rather than deltas, an account cannot
//! drift: the next record overwrites whatever the last one said.
//!
//! # One owner per byte
//!
//! Chunk memory is owned by the world, even though streaming decides which
//! chunks are resident. Streaming therefore **enforces** the world pool's
//! ceiling and does not open a pool of its own: two pools counting the same
//! bytes would make the ledger's total a lie. Register a pool for memory a
//! subsystem *holds*, not for memory it *influences*.
//!
//! # Thresholds
//!
//! A pool's pressure is the highest threshold its usage is **above**:
//!
//! ```text
//! usage <= TARGET              Nominal
//! TARGET    < usage <= WARNING  Elevated
//! WARNING   < usage <= CRITICAL Warning
//! CRITICAL  < usage <= EMERGENCY Critical
//! EMERGENCY < usage            Emergency — the budget was broken
//! ```
//!
//! `EMERGENCY` is the ceiling: an owner that is doing its job never records
//! above it, so [`Pressure::Emergency`] is always a defect and never a state
//! to operate in. A cache, whose job is to be full, publishes all four at its
//! capacity: full is nominal, and more than full is the one thing it may not be.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};

use crate::error::{Domain, Error, Recovery, Result};

/// The memory classes of `NEXORA MEMORY AND RESOURCE OWNERSHIP.md`, *Classes*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryClass {
    /// `ENGINE / LONG-LIVED`: registries, module state, anything that lives as
    /// long as the process.
    Engine,
    /// `WORLD / PERSISTENT`: authoritative world state — chunks, and chunks
    /// held outside the world until they can be written.
    World,
    /// `FRAME / TRANSIENT`: scratch that must be empty again at a frame
    /// boundary.
    Frame,
    /// `STREAMING`: loaded on demand and disposable — asset caches, staging.
    Streaming,
    /// `GPU`: owned by the renderer/RHI alone.
    Gpu,
    /// `NETWORK`: per-connection, per-session and per-queue buffers.
    Network,
    /// `SCRIPT / MOD`: quotas for code the engine does not trust.
    Script,
    /// `EDITOR`: tooling that does not ship in a game build.
    Editor,
}

impl MemoryClass {
    /// Every class, in the document's order.
    pub const ALL: [Self; 8] = [
        Self::Engine,
        Self::World,
        Self::Frame,
        Self::Streaming,
        Self::Gpu,
        Self::Network,
        Self::Script,
        Self::Editor,
    ];

    /// Stable lowercase name, safe for logs and reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Engine => "engine",
            Self::World => "world",
            Self::Frame => "frame",
            Self::Streaming => "streaming",
            Self::Gpu => "gpu",
            Self::Network => "network",
            Self::Script => "script",
            Self::Editor => "editor",
        }
    }

    /// Whether memory of this class must be gone at rest by its nature.
    ///
    /// Frame scratch that survives the frame is a leak by definition; every
    /// other class may legitimately hold memory at rest, and an owner that
    /// knows better says so with [`PoolSpec::drains_at_rest`].
    #[must_use]
    pub const fn drains_at_rest(self) -> bool {
        matches!(self, Self::Frame)
    }
}

/// The four thresholds of `NEXORA PERFORMANCE BUDGETS.md`, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryBudget {
    target: u64,
    warning: u64,
    critical: u64,
    emergency: u64,
}

impl MemoryBudget {
    /// A budget with all four thresholds stated.
    ///
    /// # Errors
    ///
    /// Returns an error unless `0 < emergency` and
    /// `target <= warning <= critical <= emergency`. Out of order, a pool
    /// could be "critical" below its warning; a zero ceiling admits nothing.
    pub fn new(target: u64, warning: u64, critical: u64, emergency: u64) -> Result<Self> {
        if emergency == 0 {
            return Err(invalid(
                "a memory budget with a zero ceiling admits nothing",
            ));
        }
        if !(target <= warning && warning <= critical && critical <= emergency) {
            return Err(invalid(
                "memory thresholds must satisfy target <= warning <= critical <= emergency",
            )
            .with_context("target", target.to_string())
            .with_context("warning", warning.to_string())
            .with_context("critical", critical.to_string())
            .with_context("emergency", emergency.to_string()));
        }
        Ok(Self {
            target,
            warning,
            critical,
            emergency,
        })
    }

    /// The budget of something whose job is to be full: a cache.
    ///
    /// All four thresholds sit at the capacity, so a full cache is nominal and
    /// only an over-full one registers — as [`Pressure::Emergency`]. A cache's
    /// real pressure signal is how often it had to refuse, which the pool
    /// counts separately ([`MemoryPool::refuse`]).
    ///
    /// # Errors
    ///
    /// Returns an error for a zero capacity.
    pub fn capacity(bytes: u64) -> Result<Self> {
        Self::new(bytes, bytes, bytes, bytes)
    }

    /// Bytes at or below which usage is nominal.
    #[must_use]
    pub const fn target(&self) -> u64 {
        self.target
    }

    /// Bytes above which usage is a warning.
    #[must_use]
    pub const fn warning(&self) -> u64 {
        self.warning
    }

    /// Bytes above which usage is critical.
    #[must_use]
    pub const fn critical(&self) -> u64 {
        self.critical
    }

    /// The ceiling. Recording above it is a broken budget.
    #[must_use]
    pub const fn emergency(&self) -> u64 {
        self.emergency
    }

    /// The pressure `bytes` of usage puts on this budget.
    #[must_use]
    pub const fn pressure_at(&self, bytes: u64) -> Pressure {
        if bytes > self.emergency {
            Pressure::Emergency
        } else if bytes > self.critical {
            Pressure::Critical
        } else if bytes > self.warning {
            Pressure::Warning
        } else if bytes > self.target {
            Pressure::Elevated
        } else {
            Pressure::Nominal
        }
    }
}

/// How close a pool is to its ceiling. Ordered: a later variant is worse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pressure {
    /// At or under target.
    Nominal,
    /// Over target, not yet a warning.
    Elevated,
    /// Over the warning threshold: tell someone.
    Warning,
    /// Over the critical threshold: the owner should be shedding.
    Critical,
    /// Over the ceiling: the budget was broken.
    Emergency,
}

impl Pressure {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::Elevated => "elevated",
            Self::Warning => "warning",
            Self::Critical => "critical",
            Self::Emergency => "emergency",
        }
    }

    const fn to_u8(self) -> u8 {
        self as u8
    }

    const fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Nominal,
            1 => Self::Elevated,
            2 => Self::Warning,
            3 => Self::Critical,
            _ => Self::Emergency,
        }
    }
}

/// Everything needed to open a pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolSpec {
    /// Dotted, like counter names: `world.chunks`, `resource.textures`.
    pub name: &'static str,
    /// Which class of memory it is.
    pub class: MemoryClass,
    /// Its four thresholds.
    pub budget: MemoryBudget,
    /// Whether it must hold nothing at rest. See [`MemoryLedger::suspected_leaks`].
    pub drains_at_rest: bool,
}

impl PoolSpec {
    /// A pool that drains at rest only if its class does.
    #[must_use]
    pub const fn new(name: &'static str, class: MemoryClass, budget: MemoryBudget) -> Self {
        Self {
            name,
            class,
            budget,
            drains_at_rest: class.drains_at_rest(),
        }
    }

    /// Declare that the owner holds nothing in this pool once it is at rest.
    #[must_use]
    pub const fn drains_at_rest(mut self) -> Self {
        self.drains_at_rest = true;
        self
    }
}

/// One owner's account. Shared as `Arc`; every method takes `&self`.
#[derive(Debug)]
pub struct MemoryPool {
    spec: PoolSpec,
    current: AtomicU64,
    high_water: AtomicU64,
    records: AtomicU64,
    refusals: AtomicU64,
    worst: AtomicU8,
}

impl MemoryPool {
    fn new(spec: PoolSpec) -> Self {
        Self {
            spec,
            current: AtomicU64::new(0),
            high_water: AtomicU64::new(0),
            records: AtomicU64::new(0),
            refusals: AtomicU64::new(0),
            worst: AtomicU8::new(Pressure::Nominal.to_u8()),
        }
    }

    /// The pool's name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.spec.name
    }

    /// The pool's class.
    #[must_use]
    pub const fn class(&self) -> MemoryClass {
        self.spec.class
    }

    /// The pool's budget.
    #[must_use]
    pub const fn budget(&self) -> MemoryBudget {
        self.spec.budget
    }

    /// Record what the owner holds now, in bytes. Returns the pressure.
    pub fn record(&self, bytes: u64) -> Pressure {
        self.current.store(bytes, Ordering::Relaxed);
        self.high_water.fetch_max(bytes, Ordering::Relaxed);
        self.records.fetch_add(1, Ordering::Relaxed);
        let pressure = self.spec.budget.pressure_at(bytes);
        self.worst.fetch_max(pressure.to_u8(), Ordering::Relaxed);
        pressure
    }

    /// Count a piece of work the owner turned down to stay in budget.
    pub fn refuse(&self) {
        self.refusals.fetch_add(1, Ordering::Relaxed);
    }

    /// Whether `extra` more bytes would still be at or under the ceiling.
    ///
    /// Advisory: the owner is the one that holds the memory, and the one that
    /// decides. A pool that refused for it would be an allocator.
    #[must_use]
    pub fn admits(&self, extra: u64) -> bool {
        self.current()
            .checked_add(extra)
            .is_some_and(|total| total <= self.spec.budget.emergency)
    }

    /// Bytes held at the last record.
    #[must_use]
    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    /// The most bytes ever recorded.
    #[must_use]
    pub fn high_water(&self) -> u64 {
        self.high_water.load(Ordering::Relaxed)
    }

    /// The pressure at the last record.
    #[must_use]
    pub fn pressure(&self) -> Pressure {
        self.spec.budget.pressure_at(self.current())
    }

    /// The worst pressure any record has shown.
    #[must_use]
    pub fn worst_pressure(&self) -> Pressure {
        Pressure::from_u8(self.worst.load(Ordering::Relaxed))
    }

    /// A consistent-enough picture of the pool for a report.
    #[must_use]
    pub fn report(&self) -> PoolReport {
        PoolReport {
            name: self.spec.name,
            class: self.spec.class,
            budget: self.spec.budget,
            current: self.current(),
            high_water: self.high_water(),
            records: self.records.load(Ordering::Relaxed),
            refusals: self.refusals.load(Ordering::Relaxed),
            pressure: self.pressure(),
            worst: self.worst_pressure(),
            drains_at_rest: self.spec.drains_at_rest,
        }
    }
}

/// One pool, as a report sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolReport {
    /// The pool's name.
    pub name: &'static str,
    /// Its class.
    pub class: MemoryClass,
    /// Its budget.
    pub budget: MemoryBudget,
    /// Bytes held at the last record.
    pub current: u64,
    /// The most bytes ever recorded.
    pub high_water: u64,
    /// How many times the owner recorded. Zero means the pool was opened and
    /// never used, which is a wiring mistake worth seeing.
    pub records: u64,
    /// Work the owner refused to stay in budget.
    pub refusals: u64,
    /// Pressure now.
    pub pressure: Pressure,
    /// Worst pressure ever recorded.
    pub worst: Pressure,
    /// Whether the pool must be empty at rest.
    pub drains_at_rest: bool,
}

impl PoolReport {
    /// Whether this pool is a suspected leak: it must drain at rest, and does
    /// not hold nothing.
    #[must_use]
    pub const fn is_suspected_leak(&self) -> bool {
        self.drains_at_rest && self.current > 0
    }
}

/// Every pool in the process, by name.
#[derive(Debug, Default)]
pub struct MemoryLedger {
    pools: RwLock<BTreeMap<&'static str, Arc<MemoryPool>>>,
}

impl MemoryLedger {
    /// An empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a pool.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is empty or already taken: two owners
    /// recording into one name would overwrite each other's totals, which is
    /// the invisible sharing the ledger exists to prevent.
    pub fn register(&self, spec: PoolSpec) -> Result<Arc<MemoryPool>> {
        if spec.name.is_empty() {
            return Err(invalid("a memory pool needs a name"));
        }
        let mut pools = self.pools.write().map_err(|_| poisoned())?;
        if pools.contains_key(spec.name) {
            return Err(invalid("a memory pool with this name is already open")
                .with_context("pool", spec.name));
        }
        let pool = Arc::new(MemoryPool::new(spec));
        pools.insert(spec.name, Arc::clone(&pool));
        Ok(pool)
    }

    /// A pool by name.
    #[must_use]
    pub fn pool(&self, name: &str) -> Option<Arc<MemoryPool>> {
        self.pools.read().ok()?.get(name).cloned()
    }

    /// Every pool, by name.
    #[must_use]
    pub fn report(&self) -> MemoryReport {
        let pools = self
            .pools
            .read()
            .map(|pools| pools.values().map(|pool| pool.report()).collect())
            .unwrap_or_default();
        MemoryReport { pools }
    }

    /// Pools that must be empty at rest and are not.
    ///
    /// Only meaningful when the caller *is* at rest — between frames, after a
    /// stage has finished — which is why it is a question the caller asks
    /// rather than a check the ledger runs on every record.
    #[must_use]
    pub fn suspected_leaks(&self) -> Vec<&'static str> {
        self.report()
            .pools
            .iter()
            .filter(|pool| pool.is_suspected_leak())
            .map(|pool| pool.name)
            .collect()
    }
}

/// Every pool at one moment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MemoryReport {
    /// The pools, by name.
    pub pools: Vec<PoolReport>,
}

impl MemoryReport {
    /// Bytes held now across every pool of `class`.
    #[must_use]
    pub fn current_in(&self, class: MemoryClass) -> u64 {
        self.pools
            .iter()
            .filter(|pool| pool.class == class)
            .map(|pool| pool.current)
            .sum()
    }

    /// The worst pressure any pool has ever recorded.
    #[must_use]
    pub fn worst(&self) -> Pressure {
        self.pools
            .iter()
            .map(|pool| pool.worst)
            .max()
            .unwrap_or(Pressure::Nominal)
    }

    /// Pools that are suspected leaks. See [`MemoryLedger::suspected_leaks`].
    #[must_use]
    pub fn suspected_leaks(&self) -> usize {
        self.pools
            .iter()
            .filter(|pool| pool.is_suspected_leak())
            .count()
    }
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Core, "memory-ledger", message).with_recovery(Recovery::Reject)
}

fn poisoned() -> Error {
    Error::new(
        Domain::Core,
        "memory-ledger",
        "the memory ledger's lock was poisoned by a panicking owner",
    )
    .fatal()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> MemoryBudget {
        MemoryBudget::new(100, 200, 300, 400).unwrap()
    }

    #[test]
    fn thresholds_out_of_order_or_a_zero_ceiling_are_refused() {
        assert!(MemoryBudget::new(0, 0, 0, 0).is_err());
        assert!(MemoryBudget::new(200, 100, 300, 400).is_err());
        assert!(MemoryBudget::new(100, 200, 400, 300).is_err());
        assert!(MemoryBudget::capacity(0).is_err());
        assert!(MemoryBudget::new(0, 0, 0, 1).is_ok());
    }

    #[test]
    fn pressure_is_the_highest_threshold_usage_is_above() {
        let budget = budget();
        assert_eq!(budget.pressure_at(0), Pressure::Nominal);
        assert_eq!(budget.pressure_at(100), Pressure::Nominal);
        assert_eq!(budget.pressure_at(101), Pressure::Elevated);
        assert_eq!(budget.pressure_at(201), Pressure::Warning);
        assert_eq!(budget.pressure_at(301), Pressure::Critical);
        assert_eq!(budget.pressure_at(400), Pressure::Critical);
        assert_eq!(budget.pressure_at(401), Pressure::Emergency);
    }

    #[test]
    fn a_full_cache_is_nominal_and_an_overfull_one_is_an_emergency() {
        let cache = MemoryBudget::capacity(1024).unwrap();
        assert_eq!(cache.pressure_at(1024), Pressure::Nominal);
        assert_eq!(cache.pressure_at(1025), Pressure::Emergency);
    }

    #[test]
    fn a_pool_keeps_its_high_water_mark_and_worst_pressure_after_usage_falls() {
        let ledger = MemoryLedger::new();
        let pool = ledger
            .register(PoolSpec::new("world.chunks", MemoryClass::World, budget()))
            .unwrap();
        assert_eq!(pool.record(250), Pressure::Warning);
        assert_eq!(pool.record(50), Pressure::Nominal);
        let report = pool.report();
        assert_eq!(report.current, 50);
        assert_eq!(report.high_water, 250);
        assert_eq!(report.pressure, Pressure::Nominal);
        assert_eq!(
            report.worst,
            Pressure::Warning,
            "a spike that passed is still reported"
        );
        assert_eq!(report.records, 2);
    }

    #[test]
    fn admission_is_measured_against_the_ceiling() {
        let ledger = MemoryLedger::new();
        let pool = ledger
            .register(PoolSpec::new("a", MemoryClass::Engine, budget()))
            .unwrap();
        pool.record(350);
        assert!(pool.admits(50));
        assert!(!pool.admits(51));
        assert!(!pool.admits(u64::MAX), "no overflow into admission");
        pool.refuse();
        assert_eq!(pool.report().refusals, 1);
    }

    #[test]
    fn two_owners_may_not_share_a_name() {
        let ledger = MemoryLedger::new();
        ledger
            .register(PoolSpec::new("world.chunks", MemoryClass::World, budget()))
            .unwrap();
        let err = ledger
            .register(PoolSpec::new(
                "world.chunks",
                MemoryClass::Streaming,
                budget(),
            ))
            .unwrap_err();
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(ledger
            .register(PoolSpec::new("", MemoryClass::World, budget()))
            .is_err());
    }

    #[test]
    fn only_pools_that_must_drain_are_suspected_of_leaking() {
        let ledger = MemoryLedger::new();
        let world = ledger
            .register(PoolSpec::new("world.chunks", MemoryClass::World, budget()))
            .unwrap();
        let frame = ledger
            .register(PoolSpec::new("frame.scratch", MemoryClass::Frame, budget()))
            .unwrap();
        let retained = ledger
            .register(
                PoolSpec::new("world.retained", MemoryClass::World, budget()).drains_at_rest(),
            )
            .unwrap();
        world.record(300);
        frame.record(10);
        retained.record(20);
        assert_eq!(
            ledger.suspected_leaks(),
            ["frame.scratch", "world.retained"],
            "a world holding chunks at rest is not a leak; frame scratch is"
        );
        frame.record(0);
        retained.record(0);
        assert!(ledger.suspected_leaks().is_empty());
        assert_eq!(ledger.report().suspected_leaks(), 0);
    }

    #[test]
    fn the_report_totals_by_class_and_names_the_worst_pressure_seen() {
        let ledger = MemoryLedger::new();
        let a = ledger
            .register(PoolSpec::new("b.pool", MemoryClass::Streaming, budget()))
            .unwrap();
        let b = ledger
            .register(PoolSpec::new("a.pool", MemoryClass::Streaming, budget()))
            .unwrap();
        a.record(120);
        b.record(30);
        let report = ledger.report();
        assert_eq!(
            report.pools.iter().map(|p| p.name).collect::<Vec<_>>(),
            ["a.pool", "b.pool"],
            "by name, so reports are stable"
        );
        assert_eq!(report.current_in(MemoryClass::Streaming), 150);
        assert_eq!(report.current_in(MemoryClass::World), 0);
        assert_eq!(report.worst(), Pressure::Elevated);
        assert_eq!(MemoryLedger::new().report().worst(), Pressure::Nominal);
    }

    #[test]
    fn pools_are_shared_across_threads() {
        let ledger = Arc::new(MemoryLedger::new());
        let pool = ledger
            .register(PoolSpec::new("shared", MemoryClass::Engine, budget()))
            .unwrap();
        let handles: Vec<_> = (1..=8)
            .map(|n| {
                let pool = Arc::clone(&pool);
                std::thread::spawn(move || {
                    pool.record(n * 10);
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(pool.high_water(), 80);
        assert_eq!(pool.report().records, 8);
        assert!(Arc::ptr_eq(&ledger.pool("shared").unwrap(), &pool));
    }

    #[test]
    fn every_class_has_a_distinct_name() {
        let names: std::collections::BTreeSet<_> = MemoryClass::ALL
            .iter()
            .map(|class| class.as_str())
            .collect();
        assert_eq!(names.len(), MemoryClass::ALL.len());
    }
}
