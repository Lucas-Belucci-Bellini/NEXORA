//! Physics' published time budget (`NEXORA PERFORMANCE BUDGETS.md`).
//!
//! Every major system publishes `TARGET`, `WARNING`, `CRITICAL` and
//! `EMERGENCY`. Physics waited to publish its thresholds until it had been
//! measured on more than one machine (DEBT-0013). Thresholds taken from one
//! shared container would claim precision nobody had earned.
//!
//! # What is budgeted
//!
//! One substep of [`CROWD`] awake dynamic bodies against generated terrain:
//! the benchmark plan's own stress figure, and the `physics.thousand_bodies_step`
//! measurement. Awake is the case that costs. Asleep, the same crowd costs
//! about a hundredth of it, and a budget written against sleepers would pass
//! a world where nothing ever settles.
//!
//! # Where the numbers come from
//!
//! | machine | median | p95 |
//! | --- | ---: | ---: |
//! | AMD Ryzen 5 5500, Windows 10 (local validation, commit `0b7bcec`) | 84.6 µs | 119.4 µs |
//! | 4-CPU shared container, Linux (two runs, same code) | 113.9–116.4 µs | 165.4–168.0 µs |
//!
//! The two machines differ by a factor, not in shape: every physics row is
//! 1.27–1.58× slower in the container, in the same order. `docs/benchmarks/
//! PHASE-0-BASELINE.md`, Appendix I, has the rows.
//!
//! * **TARGET, 250 µs** — measured: the slower machine's p95 with half again
//!   to spare. At one substep per 60 Hz frame it is 1.5% of the frame.
//! * **EMERGENCY, 2 ms** — arithmetic, not measurement. [`FixedStep`] catches
//!   up at most [`DEFAULT_MAX_SUBSTEPS`] substeps in one frame. Eight substeps
//!   of 2 ms are 16 ms, a whole 60 Hz frame spent in physics alone. Past this
//!   line a frame that falls behind cannot catch up, and falls further behind.
//! * **WARNING, 500 µs** and **CRITICAL, 1 ms** halve the way down from
//!   the emergency line. A catch-up frame at `CRITICAL` spends half the frame
//!   in physics.
//!
//! Nothing here reads a clock. The thresholds are data. The benchmark
//! classifies its measurement against them, and a future runtime stage can
//! classify a live substep the same way.
//!
//! [`FixedStep`]: crate::step::FixedStep
//! [`DEFAULT_MAX_SUBSTEPS`]: crate::step::DEFAULT_MAX_SUBSTEPS

use core::time::Duration;

/// Awake dynamic bodies in the budgeted substep.
pub const CROWD: usize = 1_000;

/// The four thresholds for one substep, lowest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubstepBudget {
    /// Within this, physics is where it was measured to be.
    pub target: Duration,
    /// Twice or more what was measured: a regression, or a bigger crowd.
    pub warning: Duration,
    /// A catch-up frame spends half of a 60 Hz frame in physics.
    pub critical: Duration,
    /// A catch-up frame spends all of it: the frame loop cannot recover.
    pub emergency: Duration,
}

impl SubstepBudget {
    /// The thresholds as `[target, warning, critical, emergency]`.
    #[must_use]
    pub const fn thresholds(&self) -> [Duration; 4] {
        [self.target, self.warning, self.critical, self.emergency]
    }
}

/// The published budget for one substep of [`CROWD`] awake bodies.
pub const CROWD_SUBSTEP: SubstepBudget = SubstepBudget {
    target: Duration::from_micros(250),
    warning: Duration::from_micros(500),
    critical: Duration::from_millis(1),
    emergency: Duration::from_millis(2),
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::{DEFAULT_MAX_SUBSTEPS, DEFAULT_STEPS_PER_SECOND};

    #[test]
    fn the_thresholds_rise() {
        let [target, warning, critical, emergency] = CROWD_SUBSTEP.thresholds();
        assert!(Duration::ZERO < target);
        assert!(target < warning && warning < critical && critical < emergency);
    }

    #[test]
    fn the_emergency_line_is_the_frame_a_catch_up_cannot_survive() {
        // The derivation, held to the constants it came from: if the step
        // rate or the catch-up cap changes, this budget has to be re-derived.
        let frame = Duration::from_secs(1) / DEFAULT_STEPS_PER_SECOND;
        let worst_catch_up = CROWD_SUBSTEP.emergency * DEFAULT_MAX_SUBSTEPS;
        assert!(worst_catch_up <= frame, "{worst_catch_up:?} > {frame:?}");
        assert!(
            (CROWD_SUBSTEP.emergency * 2) * DEFAULT_MAX_SUBSTEPS > frame,
            "the line is where a frame is lost, not far below it"
        );
        assert_eq!(
            CROWD_SUBSTEP.critical * DEFAULT_MAX_SUBSTEPS * 2,
            worst_catch_up
        );
    }

    #[test]
    fn the_target_keeps_a_steady_crowd_under_two_percent_of_a_core() {
        let per_second = CROWD_SUBSTEP.target * DEFAULT_STEPS_PER_SECOND;
        assert!(per_second <= Duration::from_millis(20), "{per_second:?}");
    }
}
