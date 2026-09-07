//! The fixed timestep (PHY-1).
//!
//! `PHYSICS.md` §2 requires physics time to be independent of frame time: a
//! simulation whose step size follows the frame rate gives different results on
//! different machines, which `NEXORA REPLAY AND DETERMINISM.md` forbids outright.
//!
//! ## Why the accumulator is integer arithmetic
//!
//! The obvious accumulator adds `f64` seconds and subtracts a step at a time.
//! It drifts: `1.0 / 60.0` has no exact binary representation, so after enough
//! frames the number of steps taken depends on the order the additions happened
//! in. This one counts in **world ticks scaled by the step rate**, so both the
//! addition and the subtraction are exact integers and the same elapsed time
//! always yields the same number of steps.
//!
//! ## Why there is a cap
//!
//! Without one, a long stall hands the accumulator a large elapsed time, which
//! produces many steps, which takes longer, which produces more steps. That
//! spiral is a denial of service the engine inflicts on itself. Reaching the cap
//! reports the substeps it had to drop rather than pretending the time was
//! simulated.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::time::WorldDuration;

/// The default physics rate, in steps per second of world time.
pub const DEFAULT_STEPS_PER_SECOND: u32 = 60;

/// The most substeps one [`FixedStep::accumulate`] will schedule.
pub const DEFAULT_MAX_SUBSTEPS: u32 = 8;

/// How many steps an elapsed duration is worth, and what had to be dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StepPlan {
    /// Substeps to run now.
    pub substeps: u32,
    /// Substeps the cap refused to schedule. Non-zero means the simulation is
    /// falling behind world time, which is a fact worth reporting rather than
    /// absorbing.
    pub dropped: u32,
}

impl StepPlan {
    /// Whether the cap discarded simulation time.
    #[must_use]
    pub const fn fell_behind(&self) -> bool {
        self.dropped > 0
    }
}

/// A fixed-rate accumulator over world time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedStep {
    steps_per_second: u32,
    ticks_per_second: u32,
    max_substeps: u32,
    /// Elapsed ticks multiplied by `steps_per_second`; one step costs
    /// `ticks_per_second` of it.
    accumulator: u64,
}

impl FixedStep {
    /// Build an accumulator.
    ///
    /// # Errors
    ///
    /// Returns an error when either rate is zero, or when `max_substeps` is
    /// zero — a cap of zero would mean physics never advances, which is a
    /// configuration mistake rather than a valid choice.
    pub fn new(steps_per_second: u32, ticks_per_second: u32, max_substeps: u32) -> Result<Self> {
        let checks = [
            ("steps_per_second", steps_per_second),
            ("ticks_per_second", ticks_per_second),
            ("max_substeps", max_substeps),
        ];
        for (name, value) in checks {
            if value == 0 {
                return Err(
                    Error::new(Domain::Physics, "fixed-step", "rate must be positive")
                        .with_recovery(Recovery::Reject)
                        .with_context("field", name),
                );
            }
        }
        Ok(Self {
            steps_per_second,
            ticks_per_second,
            max_substeps,
            accumulator: 0,
        })
    }

    /// An accumulator at [`DEFAULT_STEPS_PER_SECOND`] over a tick rate.
    ///
    /// # Errors
    ///
    /// Returns an error when `ticks_per_second` is zero.
    pub fn per_second(ticks_per_second: u32) -> Result<Self> {
        Self::new(
            DEFAULT_STEPS_PER_SECOND,
            ticks_per_second,
            DEFAULT_MAX_SUBSTEPS,
        )
    }

    /// Steps per second of world time.
    #[must_use]
    pub const fn steps_per_second(&self) -> u32 {
        self.steps_per_second
    }

    /// The duration of one substep, in seconds.
    #[must_use]
    pub fn seconds_per_step(&self) -> f64 {
        1.0 / f64::from(self.steps_per_second)
    }

    /// The most substeps a single call will schedule.
    #[must_use]
    pub const fn max_substeps(&self) -> u32 {
        self.max_substeps
    }

    /// Unsimulated world time held over for the next call, in seconds.
    #[must_use]
    pub fn pending_seconds(&self) -> f64 {
        self.accumulator as f64
            / (f64::from(self.steps_per_second) * f64::from(self.ticks_per_second))
    }

    /// Fold an elapsed world duration in and report the substeps it earns.
    ///
    /// Time beyond the cap is discarded, not carried: carrying it would only
    /// move the spiral to the next call.
    #[must_use]
    pub fn accumulate(&mut self, elapsed: WorldDuration) -> StepPlan {
        let scaled = elapsed
            .ticks()
            .saturating_mul(u64::from(self.steps_per_second));
        self.accumulator = self.accumulator.saturating_add(scaled);

        let cost = u64::from(self.ticks_per_second);
        let earned = self.accumulator / cost;
        let capped = earned.min(u64::from(self.max_substeps));
        self.accumulator -= capped * cost;

        let dropped = earned - capped;
        if dropped > 0 {
            // Drop the remainder too: a partial step of abandoned time would
            // reappear as a phantom step later.
            self.accumulator = 0;
        }

        StepPlan {
            substeps: u32::try_from(capped).unwrap_or(self.max_substeps),
            dropped: u32::try_from(dropped).unwrap_or(u32::MAX),
        }
    }

    /// Discard pending time, for example after a world reload.
    pub fn reset(&mut self) {
        self.accumulator = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_rates_are_refused() {
        assert!(FixedStep::new(0, 20, 8).is_err());
        assert!(FixedStep::new(60, 0, 8).is_err());
        assert!(FixedStep::new(60, 20, 0).is_err());
    }

    #[test]
    fn one_world_second_is_exactly_the_step_rate() {
        let mut step = FixedStep::new(60, 20, 1_000).expect("valid");
        let plan = step.accumulate(WorldDuration::from_ticks(20));
        assert_eq!(plan.substeps, 60);
        assert_eq!(plan.dropped, 0);
        assert!(step.pending_seconds().abs() < 1e-12);
    }

    #[test]
    fn a_tick_that_is_not_a_whole_step_carries_the_remainder_exactly() {
        // 20 ticks/s and 60 steps/s: one tick is exactly three steps, so use a
        // rate where it is not. 7 steps/s over 20 ticks/s: one tick earns
        // 7/20 of a step.
        let mut step = FixedStep::new(7, 20, 1_000).expect("valid");
        let mut total = 0u32;
        for _ in 0..20 {
            total += step.accumulate(WorldDuration::from_ticks(1)).substeps;
        }
        // Twenty ticks is one world second, which must be exactly seven steps.
        assert_eq!(total, 7);
        assert!(step.pending_seconds().abs() < 1e-12);
    }

    #[test]
    fn the_step_count_does_not_drift_over_a_long_run() {
        // The whole reason the accumulator is integer arithmetic. A thousand
        // world seconds must be exactly a thousand seconds of steps, however
        // the elapsed time was chopped up.
        let mut one_at_a_time = FixedStep::new(60, 20, 10_000).expect("valid");
        let mut total = 0u64;
        for _ in 0..20_000 {
            total += u64::from(
                one_at_a_time
                    .accumulate(WorldDuration::from_ticks(1))
                    .substeps,
            );
        }
        assert_eq!(total, 60_000);

        // The bulk accumulator needs a cap above the whole run, or it would be
        // measuring the cap rather than the arithmetic.
        let mut in_bulk = FixedStep::new(60, 20, 100_000).expect("valid");
        let bulk = in_bulk.accumulate(WorldDuration::from_ticks(20_000));
        assert_eq!(u64::from(bulk.substeps), total);
        assert_eq!(bulk.dropped, 0);
    }

    #[test]
    fn the_cap_reports_what_it_dropped_instead_of_hiding_it() {
        let mut step = FixedStep::new(60, 20, 8).expect("valid");
        // Ten world seconds arrive at once: 600 steps' worth.
        let plan = step.accumulate(WorldDuration::from_ticks(200));
        assert_eq!(plan.substeps, 8);
        assert_eq!(plan.dropped, 592);
        assert!(plan.fell_behind());
        // And the abandoned time does not come back later.
        assert!(step.pending_seconds().abs() < 1e-12);
        assert_eq!(step.accumulate(WorldDuration::ZERO).substeps, 0);
    }

    #[test]
    fn zero_elapsed_time_earns_no_step() {
        let mut step = FixedStep::per_second(20).expect("valid");
        assert_eq!(step.accumulate(WorldDuration::ZERO), StepPlan::default());
    }

    #[test]
    fn an_absurd_elapsed_duration_saturates_instead_of_overflowing() {
        let mut step = FixedStep::new(60, 20, 8).expect("valid");
        let plan = step.accumulate(WorldDuration::from_ticks(u64::MAX));
        assert_eq!(plan.substeps, 8);
        assert!(plan.fell_behind());
    }

    #[test]
    fn reset_clears_pending_time() {
        let mut step = FixedStep::new(60, 20, 1_000).expect("valid");
        let _ = step.accumulate(WorldDuration::from_ticks(1));
        step.reset();
        assert!(step.pending_seconds().abs() < 1e-12);
    }

    #[test]
    fn the_substep_duration_matches_the_declared_rate() {
        let step = FixedStep::new(50, 20, 8).expect("valid");
        assert!((step.seconds_per_step() - 0.02).abs() < 1e-12);
        assert_eq!(step.steps_per_second(), 50);
        assert_eq!(step.max_substeps(), 8);
    }
}
