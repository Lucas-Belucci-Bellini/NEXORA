//! Budgets and the report a tick produces.
//!
//! `STREAMING SYSTEM.md` asks for **separate** memory, IO, generation,
//! activation and network budgets, with backpressure. The separation is the
//! point: a tick that is allowed to load as much as it likes will, on the first
//! frame after a teleport, generate every chunk in the new region at once and
//! stall for a second.
//!
//! A budget this tick refused is **deferred**, not dropped. The work stays on
//! the list and is attempted again next tick, in priority order — so a bounded
//! budget slows streaming down instead of losing chunks.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// The most work one tick will schedule, whatever the budget asks for.
///
/// Budgets are configuration, and configuration is untrusted input.
pub const MAX_OPERATIONS_PER_TICK: u32 = 4_096;

/// How much work one tick may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamingBudget {
    /// Targets that may become resident this tick.
    pub activations: u32,
    /// Targets that may stop being resident this tick.
    pub evictions: u32,
    /// The most targets that may be resident at once.
    pub max_resident: u32,
    /// The most bytes resident targets may occupy, as reported by the backend.
    pub max_bytes: u64,
}

impl StreamingBudget {
    /// A conservative budget suited to a single-threaded tick.
    pub const MODEST: Self = Self {
        activations: 4,
        evictions: 8,
        max_resident: 256,
        max_bytes: 512 * 1024 * 1024,
    };

    /// A budget that will not stand in the way, for tests and for tools.
    pub const UNLIMITED: Self = Self {
        activations: MAX_OPERATIONS_PER_TICK,
        evictions: MAX_OPERATIONS_PER_TICK,
        max_resident: u32::MAX,
        max_bytes: u64::MAX,
    };

    /// Validate the budget.
    ///
    /// # Errors
    ///
    /// Returns an error when the budget could never make progress: zero
    /// activations means nothing ever loads, and zero permitted residents means
    /// anything loaded is evicted on the same tick.
    pub fn validate(self) -> Result<Self> {
        if self.activations == 0 {
            return Err(reject(
                "a budget with no activations can never load anything",
            ));
        }
        if self.max_resident == 0 {
            return Err(reject(
                "a budget permitting no residents would evict every load immediately",
            ));
        }
        if self.activations > MAX_OPERATIONS_PER_TICK || self.evictions > MAX_OPERATIONS_PER_TICK {
            return Err(
                reject("per-tick operation counts exceed the permitted maximum")
                    .with_context("max", MAX_OPERATIONS_PER_TICK.to_string()),
            );
        }
        Ok(self)
    }
}

impl Default for StreamingBudget {
    fn default() -> Self {
        Self::MODEST
    }
}

/// What one tick did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StreamingReport {
    /// Targets that became resident.
    pub activated: u32,
    /// Targets that stopped being resident.
    pub evicted: u32,
    /// Tier changes towards more detail, activations included.
    pub promoted: u32,
    /// Tier changes towards less detail, evictions included.
    pub demoted: u32,
    /// Targets whose state was written out before they were dropped.
    pub persisted: u32,
    /// Work the budget refused this tick; it stays queued.
    pub deferred: u32,
    /// Targets evicted by memory or count pressure **despite** interest in them.
    /// Non-zero means the budget is smaller than what the observer wants.
    pub shed: u32,
    /// Backend refusals. The errors are kept, not swallowed — see
    /// [`crate::system::StreamingSystem::take_failures`].
    pub failed: u32,
    /// Targets resident after the tick.
    pub resident: u32,
    /// Targets the system is tracking at any tier.
    pub tracked: u32,
    /// Bytes the backend says the resident set occupies.
    pub bytes: u64,
}

impl StreamingReport {
    /// Whether the tick changed anything at all.
    #[must_use]
    pub const fn is_quiet(&self) -> bool {
        self.activated == 0 && self.evicted == 0 && self.promoted == 0 && self.demoted == 0
    }

    /// Whether the budget could not keep up with what interest asked for.
    #[must_use]
    pub const fn is_saturated(&self) -> bool {
        self.deferred > 0 || self.shed > 0
    }
}

fn reject(message: &'static str) -> Error {
    Error::new(Domain::World, "streaming-budget", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budgets_that_could_never_progress_are_refused() {
        assert!(StreamingBudget {
            activations: 0,
            ..StreamingBudget::MODEST
        }
        .validate()
        .is_err());
        assert!(StreamingBudget {
            max_resident: 0,
            ..StreamingBudget::MODEST
        }
        .validate()
        .is_err());
    }

    #[test]
    fn an_absurd_per_tick_count_is_refused() {
        assert!(StreamingBudget {
            activations: MAX_OPERATIONS_PER_TICK + 1,
            ..StreamingBudget::MODEST
        }
        .validate()
        .is_err());
        assert!(StreamingBudget {
            evictions: MAX_OPERATIONS_PER_TICK + 1,
            ..StreamingBudget::MODEST
        }
        .validate()
        .is_err());
    }

    #[test]
    fn the_supplied_budgets_are_themselves_valid() {
        assert!(StreamingBudget::MODEST.validate().is_ok());
        assert!(StreamingBudget::UNLIMITED.validate().is_ok());
        assert!(StreamingBudget::default().validate().is_ok());
    }

    #[test]
    fn zero_evictions_is_legal_because_it_only_delays_release() {
        // Unlike zero activations, it cannot deadlock: the work stays queued.
        assert!(StreamingBudget {
            evictions: 0,
            ..StreamingBudget::MODEST
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn a_default_report_is_quiet_and_unsaturated() {
        let report = StreamingReport::default();
        assert!(report.is_quiet());
        assert!(!report.is_saturated());
    }

    #[test]
    fn deferral_and_shedding_both_mean_saturated() {
        assert!(StreamingReport {
            deferred: 1,
            ..StreamingReport::default()
        }
        .is_saturated());
        assert!(StreamingReport {
            shed: 1,
            ..StreamingReport::default()
        }
        .is_saturated());
    }
}
