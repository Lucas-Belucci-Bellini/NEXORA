//! Structured diagnostics.
//!
//! Implements `DIAGNOSTICS AND OBSERVABILITY.md` and the field list in
//! `NEXORA OBSERVABILITY DATA MODEL.md`. The point of this module is stated in
//! that document: behaviour must be inspectable *without ad-hoc debug prints*.
//! Records therefore carry typed fields rather than pre-formatted text, so a
//! sink can index them.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::time::WorldTime;

/// Severity of a diagnostic record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// Very fine-grained tracing.
    Trace,
    /// Developer-facing detail.
    Debug,
    /// Normal operational milestones.
    Info,
    /// Something unexpected that did not stop the operation.
    Warn,
    /// An operation failed.
    Error,
    /// The runtime cannot continue.
    Fatal,
}

impl Level {
    /// Stable uppercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }
}

/// Log categories from `CORE.md` §12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Category {
    /// Core infrastructure.
    Core,
    /// Engine runtime.
    Engine,
    /// World and chunks.
    World,
    /// Rendering.
    Render,
    /// Physics.
    Physics,
    /// Audio.
    Audio,
    /// Networking.
    Network,
    /// Mods and scripting.
    Mod,
    /// Agent decision-making.
    Ai,
    /// Persistence.
    Save,
}

impl Category {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Engine => "engine",
            Self::World => "world",
            Self::Render => "render",
            Self::Physics => "physics",
            Self::Audio => "audio",
            Self::Network => "network",
            Self::Mod => "mod",
            Self::Ai => "ai",
            Self::Save => "save",
        }
    }
}

/// A correlation identifier tying related records together.
///
/// The observability model requires commands, events, saves and network
/// requests to share one of these so a causal chain can be reassembled from
/// logs alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CorrelationId(pub u64);

/// One structured diagnostic record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Severity.
    pub level: Level,
    /// Owning category.
    pub category: Category,
    /// The specific system that emitted this.
    pub system: &'static str,
    /// Human-readable summary. Should not interpolate the structured fields.
    pub message: String,
    /// Indexed key/value pairs.
    pub fields: Vec<(&'static str, String)>,
    /// World time when emitted, when a world is attached.
    pub world_time: Option<WorldTime>,
    /// Chain identifier, when this record belongs to a traced operation.
    pub correlation: Option<CorrelationId>,
}

impl Record {
    /// Start a record.
    #[must_use]
    pub fn new(
        level: Level,
        category: Category,
        system: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level,
            category,
            system,
            message: message.into(),
            fields: Vec::new(),
            world_time: None,
            correlation: None,
        }
    }

    /// Attach a structured field.
    #[must_use]
    pub fn with_field(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.fields.push((key, value.into()));
        self
    }

    /// Attach the world time.
    #[must_use]
    pub const fn at(mut self, time: WorldTime) -> Self {
        self.world_time = Some(time);
        self
    }

    /// Attach a correlation identifier.
    #[must_use]
    pub const fn correlated(mut self, id: CorrelationId) -> Self {
        self.correlation = Some(id);
        self
    }

    /// Render as a single stable line.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = format!(
            "{level} [{category}/{system}] {message}",
            level = self.level.as_str(),
            category = self.category.as_str(),
            system = self.system,
            message = self.message
        );
        if let Some(time) = self.world_time {
            out.push_str(&format!(" tick={}", time.ticks()));
        }
        if let Some(id) = self.correlation {
            out.push_str(&format!(" correlation={}", id.0));
        }
        for (key, value) in &self.fields {
            out.push_str(&format!(" {key}={value}"));
        }
        out
    }
}

/// A destination for diagnostic records.
pub trait Sink: Send + Sync {
    /// Consume one record.
    fn record(&self, record: &Record);
}

/// A sink that writes rendered lines to standard error.
#[derive(Debug, Default)]
pub struct StderrSink;

impl Sink for StderrSink {
    fn record(&self, record: &Record) {
        eprintln!("{}", record.render());
    }
}

/// A sink that keeps records in memory.
///
/// Tests assert on emitted diagnostics through this rather than by capturing
/// stdout, which keeps observability itself testable.
#[derive(Debug, Default)]
pub struct CollectingSink {
    records: Mutex<Vec<Record>>,
}

impl CollectingSink {
    /// Create an empty collecting sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot the records collected so far.
    ///
    /// # Panics
    ///
    /// Panics if a previous caller panicked while holding the lock.
    #[must_use]
    pub fn records(&self) -> Vec<Record> {
        self.records
            .lock()
            .expect("diagnostics sink lock poisoned")
            .clone()
    }

    /// Number of records at or above a level.
    #[must_use]
    pub fn count_at_least(&self, level: Level) -> usize {
        self.records()
            .iter()
            .filter(|record| record.level >= level)
            .count()
    }
}

impl Sink for CollectingSink {
    fn record(&self, record: &Record) {
        if let Ok(mut records) = self.records.lock() {
            records.push(record.clone());
        }
    }
}

/// Named monotonic counters.
///
/// These are the `METRIC` telemetry class: cheap, always-on numbers such as
/// chunk loads or jobs completed.
#[derive(Debug, Default)]
pub struct Counters {
    values: RwLock<BTreeMap<&'static str, AtomicU64>>,
}

impl Counters {
    /// Create an empty counter set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add to a counter, creating it if absent.
    pub fn add(&self, name: &'static str, amount: u64) {
        // Fast path: the counter already exists, so a read lock is enough.
        if let Ok(values) = self.values.read() {
            if let Some(counter) = values.get(name) {
                counter.fetch_add(amount, Ordering::Relaxed);
                return;
            }
        }
        if let Ok(mut values) = self.values.write() {
            values
                .entry(name)
                .or_default()
                .fetch_add(amount, Ordering::Relaxed);
        }
    }

    /// Increment a counter by one.
    pub fn increment(&self, name: &'static str) {
        self.add(name, 1);
    }

    /// Read a counter; absent counters read as zero.
    #[must_use]
    pub fn get(&self, name: &str) -> u64 {
        self.values
            .read()
            .ok()
            .and_then(|values| values.get(name).map(|c| c.load(Ordering::Relaxed)))
            .unwrap_or(0)
    }

    /// Snapshot every counter, sorted by name.
    #[must_use]
    pub fn snapshot(&self) -> Vec<(&'static str, u64)> {
        self.values
            .read()
            .map(|values| {
                values
                    .iter()
                    .map(|(name, c)| (*name, c.load(Ordering::Relaxed)))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Health of a subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    /// Operating normally.
    Ready,
    /// Working, but outside its intended budget or with reduced capability.
    Degraded,
    /// Not usable.
    Failed,
}

/// The diagnostics facade a runtime hands to its systems.
///
/// Cloning shares the same sinks and counters, so subsystems can hold their own
/// handle without a global.
#[derive(Clone)]
pub struct Diagnostics {
    inner: Arc<DiagnosticsInner>,
}

struct DiagnosticsInner {
    sinks: Vec<Box<dyn Sink>>,
    min_level: Level,
    counters: Counters,
    next_correlation: AtomicU64,
}

impl std::fmt::Debug for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Diagnostics")
            .field("sinks", &self.inner.sinks.len())
            .field("min_level", &self.inner.min_level)
            .finish()
    }
}

impl Diagnostics {
    /// Build a diagnostics facade over a set of sinks.
    #[must_use]
    pub fn new(sinks: Vec<Box<dyn Sink>>, min_level: Level) -> Self {
        Self {
            inner: Arc::new(DiagnosticsInner {
                sinks,
                min_level,
                counters: Counters::new(),
                next_correlation: AtomicU64::new(1),
            }),
        }
    }

    /// A facade that discards everything, for tests and headless tools.
    #[must_use]
    pub fn silent() -> Self {
        Self::new(Vec::new(), Level::Fatal)
    }

    /// Emit a record if it meets the minimum level.
    pub fn record(&self, record: Record) {
        if record.level < self.inner.min_level {
            return;
        }
        for sink in &self.inner.sinks {
            sink.record(&record);
        }
    }

    /// Emit a simple message.
    pub fn log(
        &self,
        level: Level,
        category: Category,
        system: &'static str,
        message: impl Into<String>,
    ) {
        self.record(Record::new(level, category, system, message));
    }

    /// The shared counter set.
    #[must_use]
    pub fn counters(&self) -> &Counters {
        &self.inner.counters
    }

    /// Allocate a fresh correlation identifier.
    #[must_use]
    pub fn new_correlation(&self) -> CorrelationId {
        CorrelationId(self.inner.next_correlation.fetch_add(1, Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collecting() -> (Diagnostics, Arc<CollectingSink>) {
        let sink = Arc::new(CollectingSink::new());
        let diagnostics = Diagnostics::new(vec![Box::new(SharedSink(sink.clone()))], Level::Trace);
        (diagnostics, sink)
    }

    /// Adapter so a test can hold a handle to the same sink the facade writes to.
    struct SharedSink(Arc<CollectingSink>);
    impl Sink for SharedSink {
        fn record(&self, record: &Record) {
            self.0.record(record);
        }
    }

    #[test]
    fn records_carry_structured_fields_not_interpolated_text() {
        let (diagnostics, sink) = collecting();
        diagnostics.record(
            Record::new(
                Level::Info,
                Category::World,
                "chunk-manager",
                "chunk loaded",
            )
            .with_field("chunk", "4,-7")
            .at(WorldTime(120)),
        );

        let records = sink.records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].message, "chunk loaded");
        assert_eq!(records[0].fields, vec![("chunk", "4,-7".to_string())]);
        assert_eq!(records[0].world_time, Some(WorldTime(120)));

        let rendered = records[0].render();
        assert!(
            rendered.contains("INFO [world/chunk-manager] chunk loaded"),
            "{rendered}"
        );
        assert!(rendered.contains("tick=120"), "{rendered}");
        assert!(rendered.contains("chunk=4,-7"), "{rendered}");
    }

    #[test]
    fn level_filter_drops_records_below_the_threshold() {
        let sink = Arc::new(CollectingSink::new());
        let diagnostics = Diagnostics::new(vec![Box::new(SharedSink(sink.clone()))], Level::Warn);

        diagnostics.log(Level::Debug, Category::Core, "boot", "not recorded");
        diagnostics.log(Level::Warn, Category::Core, "boot", "recorded");
        diagnostics.log(Level::Error, Category::Core, "boot", "recorded");

        assert_eq!(sink.records().len(), 2);
        assert_eq!(sink.count_at_least(Level::Error), 1);
    }

    #[test]
    fn counters_accumulate_and_snapshot_sorted() {
        let (diagnostics, _) = collecting();
        diagnostics.counters().increment("chunks.loaded");
        diagnostics.counters().add("chunks.loaded", 4);
        diagnostics.counters().increment("jobs.completed");

        assert_eq!(diagnostics.counters().get("chunks.loaded"), 5);
        assert_eq!(diagnostics.counters().get("jobs.completed"), 1);
        assert_eq!(diagnostics.counters().get("never.touched"), 0);
        assert_eq!(
            diagnostics.counters().snapshot(),
            vec![("chunks.loaded", 5), ("jobs.completed", 1)]
        );
    }

    #[test]
    fn correlation_ids_are_unique_and_propagate() {
        let (diagnostics, sink) = collecting();
        let first = diagnostics.new_correlation();
        let second = diagnostics.new_correlation();
        assert_ne!(first, second);

        diagnostics.record(
            Record::new(Level::Info, Category::Save, "container", "commit").correlated(first),
        );
        assert_eq!(sink.records()[0].correlation, Some(first));
    }

    #[test]
    fn silent_diagnostics_drop_everything_without_panicking() {
        let diagnostics = Diagnostics::silent();
        diagnostics.log(Level::Error, Category::Core, "boot", "ignored");
        diagnostics.counters().increment("still.counts");
        assert_eq!(diagnostics.counters().get("still.counts"), 1);
    }

    #[test]
    fn counters_are_safe_across_threads() {
        let (diagnostics, _) = collecting();
        std::thread::scope(|scope| {
            for _ in 0..8 {
                let diagnostics = diagnostics.clone();
                scope.spawn(move || {
                    for _ in 0..1_000 {
                        diagnostics.counters().increment("parallel.hits");
                    }
                });
            }
        });
        assert_eq!(diagnostics.counters().get("parallel.hits"), 8_000);
    }
}
