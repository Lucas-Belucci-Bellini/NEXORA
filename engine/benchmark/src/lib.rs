//! # NEXORA measurement harness
//!
//! Implements the measurement side of `NEXORA TECHNOLOGY BENCHMARK PLAN.md`.
//!
//! ## What this can and cannot decide
//!
//! The benchmark plan's vertical slice runs
//! `window → input → RHI → camera → 16³ chunk → mesh → 1,000 entities →
//! physics → jobs → streaming → save/load → headless server → mod boundary`.
//! Phase 0 implements four of those thirteen stages
//! ([ADR-0005](../../../docs/adr/ADR-0005-phase-0-scope-boundary.md)), so this
//! harness measures four of them and **declares the rest unmeasured rather than
//! omitting them**. A metric that is silently absent reads as a metric that
//! passed.
//!
//! It therefore **cannot close the language gate**. Rule 5 of the plan is
//! explicit: do not choose from a single microcase. What it produces is the
//! reference baseline a competing stack has to be measured against — the thing
//! the gate was missing.
//!
//! ## Method
//!
//! Rules 3 and 4 of the plan require multiple runs and a recorded methodology:
//!
//! * every measurement warms up before it counts anything, so the first-touch
//!   page faults and branch predictor state of a cold run are not reported as
//!   the steady state;
//! * each sample times a batch of iterations and divides, which keeps the clock
//!   read out of the measurement for operations that cost nanoseconds;
//! * results report **median and p95**, not mean alone — a mean hides the tail
//!   that a frame budget actually cares about;
//! * every timed value passes through [`std::hint::black_box`] so the optimizer
//!   cannot delete work whose result is unused.

use std::fmt::Write as _;
use std::hint::black_box;
use std::time::{Duration, Instant};

pub mod suites;

/// What a measurement counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Wall time for one operation.
    TimePerOp,
    /// Bytes processed per second.
    Throughput,
    /// A size in bytes.
    Bytes,
    /// A plain count of things, which is not a size.
    Quantity,
}

/// One benchmarked operation.
#[derive(Debug, Clone)]
pub struct Measurement {
    /// Dotted name, e.g. `worldgen.chunk_32`.
    pub name: &'static str,
    /// One line saying what the number means and why it matters.
    pub note: &'static str,
    /// What the samples count.
    pub unit: Unit,
    /// Nanoseconds per operation, one entry per sample.
    pub samples: Vec<f64>,
    /// Bytes touched per operation, when throughput is meaningful.
    pub bytes_per_op: Option<u64>,
}

impl Measurement {
    fn sorted(&self) -> Vec<f64> {
        let mut values = self.samples.clone();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        values
    }

    /// Fastest observed sample.
    #[must_use]
    pub fn min(&self) -> f64 {
        self.sorted().first().copied().unwrap_or(0.0)
    }

    /// Median sample. The headline number: robust to a single scheduling hiccup.
    #[must_use]
    pub fn median(&self) -> f64 {
        let values = self.sorted();
        if values.is_empty() {
            return 0.0;
        }
        let mid = values.len() / 2;
        if values.len() % 2 == 0 {
            (values[mid - 1] + values[mid]) / 2.0
        } else {
            values[mid]
        }
    }

    /// 95th percentile. The tail a budget has to survive.
    #[must_use]
    pub fn p95(&self) -> f64 {
        let values = self.sorted();
        if values.is_empty() {
            return 0.0;
        }
        // Nearest-rank, clamped so a small sample count cannot index past the end.
        let rank = ((values.len() as f64) * 0.95).ceil() as usize;
        values[rank.saturating_sub(1).min(values.len() - 1)]
    }

    /// Arithmetic mean.
    #[must_use]
    pub fn mean(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }

    /// Relative standard deviation, as a percentage of the mean.
    ///
    /// A high value means the environment was noisy and the numbers should be
    /// treated as indicative rather than precise.
    #[must_use]
    pub fn relative_stddev(&self) -> f64 {
        let mean = self.mean();
        if self.samples.len() < 2 || mean <= 0.0 {
            return 0.0;
        }
        let variance = self
            .samples
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (self.samples.len() - 1) as f64;
        (variance.sqrt() / mean) * 100.0
    }

    /// Throughput in MiB/s, when the operation has a byte count.
    #[must_use]
    pub fn throughput_mib_s(&self) -> Option<f64> {
        let bytes = self.bytes_per_op?;
        let nanos = self.median();
        if nanos <= 0.0 {
            return None;
        }
        Some((bytes as f64 / (nanos / 1e9)) / (1024.0 * 1024.0))
    }
}

/// How hard to run each measurement.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    /// Iterations run before sampling starts.
    pub warmup_iterations: u32,
    /// How many samples to collect.
    pub samples: u32,
    /// Iterations timed together in one sample.
    pub iterations_per_sample: u32,
}

impl Budget {
    /// The default: enough samples for a median and a p95 to mean something.
    #[must_use]
    pub const fn standard(iterations_per_sample: u32) -> Self {
        Self {
            warmup_iterations: 3,
            samples: 25,
            iterations_per_sample,
        }
    }

    /// A single pass, for expensive operations and for CI smoke runs.
    #[must_use]
    pub const fn coarse(iterations_per_sample: u32) -> Self {
        Self {
            warmup_iterations: 1,
            samples: 7,
            iterations_per_sample,
        }
    }

    /// Scale the work down so a smoke run finishes quickly.
    #[must_use]
    pub const fn smoke(self) -> Self {
        Self {
            warmup_iterations: 1,
            samples: 3,
            iterations_per_sample: self.iterations_per_sample,
        }
    }
}

/// Time an operation and return its measurement.
///
/// `operation` is run `warmup_iterations` times first, then
/// `samples` batches of `iterations_per_sample` are timed.
pub fn measure<F>(
    name: &'static str,
    note: &'static str,
    budget: Budget,
    mut operation: F,
) -> Measurement
where
    F: FnMut(),
{
    for _ in 0..budget.warmup_iterations {
        operation();
    }

    let mut samples = Vec::with_capacity(budget.samples as usize);
    for _ in 0..budget.samples {
        let started = Instant::now();
        for _ in 0..budget.iterations_per_sample {
            operation();
        }
        let elapsed = started.elapsed();
        samples.push(nanos_per_op(elapsed, budget.iterations_per_sample));
    }

    Measurement {
        name,
        note,
        unit: Unit::TimePerOp,
        samples,
        bytes_per_op: None,
    }
}

/// Time an operation that processes a known number of bytes.
pub fn measure_throughput<F>(
    name: &'static str,
    note: &'static str,
    budget: Budget,
    bytes_per_op: u64,
    mut operation: F,
) -> Measurement
where
    F: FnMut(),
{
    let mut measurement = measure(name, note, budget, &mut operation);
    measurement.unit = Unit::Throughput;
    measurement.bytes_per_op = Some(bytes_per_op);
    measurement
}

/// Record a size in bytes, e.g. a save file or peak memory.
#[must_use]
pub fn record_bytes(name: &'static str, note: &'static str, value: u64) -> Measurement {
    Measurement {
        name,
        note,
        unit: Unit::Bytes,
        samples: vec![value as f64],
        bytes_per_op: None,
    }
}

/// Record a plain count of things.
///
/// Kept distinct from [`record_bytes`] because rendering a count of 18 million
/// blocks as "17.4 MiB" is not a formatting quirk, it is a false statement.
#[must_use]
pub fn record_quantity(name: &'static str, note: &'static str, value: u64) -> Measurement {
    Measurement {
        name,
        note,
        unit: Unit::Quantity,
        samples: vec![value as f64],
        bytes_per_op: None,
    }
}

fn nanos_per_op(elapsed: Duration, iterations: u32) -> f64 {
    let iterations = iterations.max(1) as f64;
    elapsed.as_nanos() as f64 / iterations
}

/// Keep a value from being optimized away.
///
/// Re-exported so suites do not each import `std::hint`.
pub fn consume<T>(value: T) -> T {
    black_box(value)
}

/// A stage of the benchmark plan's vertical slice that has no implementation
/// yet, and therefore no number.
#[derive(Debug, Clone, Copy)]
pub struct Unmeasured {
    /// The stage or metric named by the plan.
    pub name: &'static str,
    /// Why it cannot be measured today.
    pub reason: &'static str,
}

/// Peak resident set size of this process, in bytes.
///
/// Reads `VmHWM` from `/proc/self/status`, which is the high-water mark rather
/// than the current usage — the number a memory budget is written against.
/// Returns `None` off Linux or when the file is unreadable.
#[must_use]
pub fn peak_resident_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            let kilobytes: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(kilobytes * 1024);
        }
    }
    None
}

/// Size of this executable on disk, in bytes.
#[must_use]
pub fn executable_size_bytes() -> Option<u64> {
    let path = std::env::current_exe().ok()?;
    std::fs::metadata(path).ok().map(|meta| meta.len())
}

/// A complete benchmark run.
#[derive(Debug, Clone)]
pub struct Report {
    /// Measurements, in the order they ran.
    pub measurements: Vec<Measurement>,
    /// Stages the plan asks for that Phase 0 cannot provide.
    pub unmeasured: Vec<Unmeasured>,
    /// Description of the machine the numbers came from.
    pub environment: Environment,
}

/// Where the numbers came from.
///
/// Rule 4 of the benchmark plan: record methodology and hardware. A benchmark
/// number with no machine attached cannot be compared to anything.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Logical CPUs available.
    pub cpus: usize,
    /// Build profile, debug or release.
    pub profile: &'static str,
    /// Target triple.
    pub target: &'static str,
    /// Peak resident memory at the end of the run, in bytes.
    pub peak_resident_bytes: Option<u64>,
    /// Benchmark binary size, in bytes.
    pub executable_bytes: Option<u64>,
}

impl Environment {
    /// Capture the current environment.
    #[must_use]
    pub fn capture() -> Self {
        Self {
            cpus: std::thread::available_parallelism()
                .map(std::num::NonZeroUsize::get)
                .unwrap_or(1),
            profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            target: std::env::consts::ARCH,
            peak_resident_bytes: peak_resident_bytes(),
            executable_bytes: executable_size_bytes(),
        }
    }
}

/// Render a report as a Markdown document.
#[must_use]
pub fn format_markdown(report: &Report) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "| measurement | median | p95 | rel. σ | note |");
    let _ = writeln!(out, "| --- | ---: | ---: | ---: | --- |");
    for measurement in &report.measurements {
        let (median, p95) = match measurement.unit {
            Unit::Bytes => (format_bytes(measurement.median()), String::from("—")),
            Unit::Quantity => (format_quantity(measurement.median()), String::from("—")),
            Unit::Throughput => (
                measurement
                    .throughput_mib_s()
                    .map_or_else(|| String::from("—"), |value| format!("{value:.0} MiB/s")),
                format_time(measurement.p95()),
            ),
            Unit::TimePerOp => (
                format_time(measurement.median()),
                format_time(measurement.p95()),
            ),
        };
        let spread = if measurement.samples.len() > 1 {
            format!("{:.1}%", measurement.relative_stddev())
        } else {
            String::from("—")
        };
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} |",
            measurement.name, median, p95, spread, measurement.note
        );
    }

    if !report.unmeasured.is_empty() {
        let _ = writeln!(out, "\n### Not measured\n");
        let _ = writeln!(out, "| stage the plan asks for | why there is no number |");
        let _ = writeln!(out, "| --- | --- |");
        for entry in &report.unmeasured {
            let _ = writeln!(out, "| {} | {} |", entry.name, entry.reason);
        }
    }

    out
}

/// Render a report as an aligned plain-text table.
#[must_use]
pub fn format_text(report: &Report) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "NEXORA Phase 0 benchmark");
    let _ = writeln!(out, "========================");
    let _ = writeln!(
        out,
        "{} logical CPUs · {} profile · {}",
        report.environment.cpus, report.environment.profile, report.environment.target
    );
    let _ = writeln!(out);

    let width = report
        .measurements
        .iter()
        .map(|m| m.name.len())
        .max()
        .unwrap_or(20)
        .max(20);
    let _ = writeln!(
        out,
        "{:<width$}  {:>12}  {:>12}  {:>7}",
        "measurement", "median", "p95", "rel σ"
    );
    let _ = writeln!(out, "{}", "-".repeat(width + 38));

    for measurement in &report.measurements {
        let median = match measurement.unit {
            Unit::Bytes => format_bytes(measurement.median()),
            Unit::Quantity => format_quantity(measurement.median()),
            Unit::Throughput => measurement
                .throughput_mib_s()
                .map_or_else(|| String::from("—"), |value| format!("{value:.0} MiB/s")),
            Unit::TimePerOp => format_time(measurement.median()),
        };
        let p95 = match measurement.unit {
            Unit::Bytes | Unit::Quantity => String::from("—"),
            _ => format_time(measurement.p95()),
        };
        let spread = if measurement.samples.len() > 1 {
            format!("{:.1}%", measurement.relative_stddev())
        } else {
            String::from("—")
        };
        let _ = writeln!(
            out,
            "{:<width$}  {median:>12}  {p95:>12}  {spread:>7}",
            measurement.name
        );
    }

    if !report.unmeasured.is_empty() {
        let _ = writeln!(out, "\nnot measured ({}):", report.unmeasured.len());
        for entry in &report.unmeasured {
            let _ = writeln!(out, "  {:<22} {}", entry.name, entry.reason);
        }
    }
    out
}

/// Format nanoseconds with a sensible unit.
#[must_use]
pub fn format_time(nanos: f64) -> String {
    if nanos < 1_000.0 {
        format!("{nanos:.1} ns")
    } else if nanos < 1_000_000.0 {
        format!("{:.2} µs", nanos / 1_000.0)
    } else if nanos < 1_000_000_000.0 {
        format!("{:.2} ms", nanos / 1_000_000.0)
    } else {
        format!("{:.2} s", nanos / 1_000_000_000.0)
    }
}

/// Format a plain count with thousands separators.
#[must_use]
pub fn format_quantity(value: f64) -> String {
    let digits = format!("{:.0}", value.max(0.0));
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}

/// Format a byte count with a sensible unit.
#[must_use]
pub fn format_bytes(bytes: f64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    if bytes < KIB {
        format!("{bytes:.0} B")
    } else if bytes < MIB {
        format!("{:.1} KiB", bytes / KIB)
    } else {
        format!("{:.1} MiB", bytes / MIB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measurement_from(samples: &[f64]) -> Measurement {
        Measurement {
            name: "test",
            note: "",
            unit: Unit::TimePerOp,
            samples: samples.to_vec(),
            bytes_per_op: None,
        }
    }

    #[test]
    fn statistics_are_computed_from_sorted_samples() {
        // Deliberately unsorted, including an outlier at the end.
        let measurement = measurement_from(&[30.0, 10.0, 20.0, 50.0, 40.0]);
        assert!((measurement.min() - 10.0).abs() < f64::EPSILON);
        assert!((measurement.median() - 30.0).abs() < f64::EPSILON);
        assert!((measurement.mean() - 30.0).abs() < f64::EPSILON);
        assert!((measurement.p95() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn median_averages_the_middle_pair_for_even_counts() {
        let measurement = measurement_from(&[10.0, 20.0, 30.0, 40.0]);
        assert!((measurement.median() - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn p95_never_indexes_past_the_end() {
        // The bug this guards: nearest-rank on a short sample list rounds up
        // past the last index and panics.
        for count in 1..40usize {
            let samples: Vec<f64> = (0..count).map(|index| index as f64).collect();
            let measurement = measurement_from(&samples);
            let p95 = measurement.p95();
            assert!(p95 >= measurement.median(), "p95 below median at n={count}");
            assert!(
                p95 <= measurement.min() + count as f64,
                "p95 out of range at n={count}"
            );
        }
    }

    #[test]
    fn a_single_sample_reports_no_spread() {
        let measurement = measurement_from(&[42.0]);
        assert!((measurement.relative_stddev()).abs() < f64::EPSILON);
        assert!((measurement.median() - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_measurements_do_not_panic() {
        let measurement = measurement_from(&[]);
        assert_eq!(measurement.median(), 0.0);
        assert_eq!(measurement.p95(), 0.0);
        assert_eq!(measurement.mean(), 0.0);
        assert_eq!(measurement.min(), 0.0);
    }

    #[test]
    fn identical_samples_have_zero_relative_deviation() {
        let measurement = measurement_from(&[100.0; 10]);
        assert!(measurement.relative_stddev() < f64::EPSILON);
    }

    #[test]
    fn throughput_is_derived_from_the_median() {
        let mut measurement = measurement_from(&[1_000_000.0]); // 1 ms per op
        measurement.unit = Unit::Throughput;
        measurement.bytes_per_op = Some(1024 * 1024); // 1 MiB per op
                                                      // 1 MiB in 1 ms is 1000 MiB/s.
        let throughput = measurement.throughput_mib_s().expect("throughput");
        assert!((throughput - 1000.0).abs() < 1.0, "got {throughput}");
    }

    #[test]
    fn measuring_actually_runs_the_operation() {
        let mut count = 0u32;
        let budget = Budget {
            warmup_iterations: 2,
            samples: 3,
            iterations_per_sample: 5,
        };
        let measurement = measure("test", "", budget, || count += 1);

        assert_eq!(count, 2 + 3 * 5, "warmup and samples must both run");
        assert_eq!(measurement.samples.len(), 3);
        assert!(measurement.median() > 0.0);
    }

    #[test]
    fn units_render_at_a_sensible_scale() {
        assert_eq!(format_time(500.0), "500.0 ns");
        assert_eq!(format_time(1_500.0), "1.50 µs");
        assert_eq!(format_time(2_500_000.0), "2.50 ms");
        assert_eq!(format_time(3_000_000_000.0), "3.00 s");
        assert_eq!(format_bytes(512.0), "512 B");
        assert_eq!(format_bytes(2048.0), "2.0 KiB");
        assert_eq!(format_bytes(3.0 * 1024.0 * 1024.0), "3.0 MiB");
    }

    #[test]
    fn counts_are_not_rendered_as_byte_sizes() {
        // The bug this pins: 18 million blocks reported as "17.4 MiB".
        assert_eq!(format_quantity(18_262_016.0), "18 262 016");
        assert_eq!(format_quantity(0.0), "0");
        assert_eq!(format_quantity(999.0), "999");
        assert_eq!(format_quantity(1_000.0), "1 000");

        let counted = record_quantity("blocks", "", 18_262_016);
        assert_eq!(counted.unit, Unit::Quantity);
        let sized = record_bytes("save", "", 155_760);
        assert_eq!(sized.unit, Unit::Bytes);
    }

    #[test]
    fn a_report_renders_both_measurements_and_gaps() {
        let report = Report {
            measurements: vec![measurement_from(&[1_000.0])],
            unmeasured: vec![Unmeasured {
                name: "frame time",
                reason: "no renderer",
            }],
            environment: Environment::capture(),
        };

        let markdown = format_markdown(&report);
        assert!(markdown.contains("`test`"), "{markdown}");
        assert!(markdown.contains("Not measured"), "{markdown}");
        assert!(markdown.contains("no renderer"), "{markdown}");

        let text = format_text(&report);
        assert!(text.contains("not measured (1)"), "{text}");
    }

    #[test]
    fn the_environment_is_recorded() {
        let environment = Environment::capture();
        assert!(environment.cpus >= 1);
        assert!(!environment.target.is_empty());
    }
}
