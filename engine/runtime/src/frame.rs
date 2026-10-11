//! The frame loop — `CORE.md` §16, ENGINE-0.
//!
//! Until now nothing in the engine was a *frame*. Streaming ticked when a test
//! called `tick`, physics stepped when the slice asked it to, and the numbers in
//! `docs/benchmarks/PHASE-0-BASELINE.md` — 2.72 ms to generate a chunk, 26.12 ms
//! to mesh a 32³ region, 4.70 µs for an idle streaming tick — had nothing inside
//! the engine to be compared against. They were compared against a frame in
//! prose, in the debt register, by hand.
//!
//! This module is that frame. It owns three things and deliberately nothing
//! else:
//!
//! 1. **A fixed timestep.** Real time arrives in irregular lumps; simulation
//!    wants equal ones. [`FrameSchedule`] accumulates the lumps and hands back
//!    whole steps.
//! 2. **A bound on catching up.** A frame that arrives half a second late owes
//!    ten steps at 20 Hz. Running all ten makes the *next* frame later still —
//!    the spiral. [`FrameSchedule`] runs at most `max_steps` and **discards the
//!    rest, reporting how many**, rather than carrying a debt that grows.
//! 3. **Attribution.** `NEXORA PERFORMANCE BUDGETS.md` says *"a subsystem must
//!    not consume another subsystem's budget invisibly"*. [`FrameRun::record`]
//!    charges elapsed time to the stage that spent it, refuses to charge one
//!    stage twice, refuses to charge them out of order — and
//!    [`FrameReport::unattributed`] reports the time inside the frame that no
//!    stage claimed. Without that last number the sum of the stages is a
//!    tautology: it equals itself.
//!
//! A frame presented to a window also **waits**: handing it over blocks until
//! a surface image comes free, which takes the display releasing one (with
//! FIFO, at its next refresh) and the GPU finishing the frame drawn into it.
//! On the operator's desktop a frame that works for 0.3 ms lasts the
//! monitor's whole period (baseline, Finding 33). That wait is declared with
//! [`FrameRun::waited_for_presentation`], inside the stage that spent it, and
//! the budget classifies the frame's **work**: its wall time without the wait,
//! which is the CPU's side of the frame. A budget that classified the wait
//! would be the monitor's, and the GPU's side needs its own measurement
//! (`NEXORA PERFORMANCE BUDGETS.md`: CPU and GPU work are profiled
//! separately).
//!
//! ## No clock lives here
//!
//! `nexora_foundation::time` opens with the rule that shapes it: *a system that
//! samples the host clock cannot be replayed*. Nothing in this module calls
//! `Instant::now`. The host measures and hands the durations in, exactly as
//! `WorldClock::advance` takes a delta rather than reading a clock. Feed the
//! same sequence of deltas and you get the same sequence of frames, on any
//! machine — which is what makes the tests below deterministic and what would
//! let a replay drive this loop.
//!
//! ## What is deliberately absent
//!
//! No interpolation of its own. [`StepPlan::alpha`] says how far real time
//! has run into the next step, and what to do with it is the presentation's:
//! the client draws its camera that far between the player's last two ticks
//! (ADR-0040), and the simulation never reads it. No thread of its own: the loop
//! is a value the host drives, not a `while` loop that owns the process — a
//! loop that owned the process could not be tested without a clock.
//!
//! ```
//! use std::time::Duration;
//! use nexora_runtime::frame::{FrameBudget, FrameLoop, FrameSchedule, FrameStage};
//!
//! let step = Duration::from_millis(50); // 20 Hz, the world clock's tick rate
//! let mut engine = FrameLoop::new(
//!     FrameSchedule::new(step, 4)?,
//!     FrameBudget::doubling_from(step),
//! );
//!
//! // A frame that arrived 120 ms after the last one owes two whole steps,
//! // and carries 20 ms towards the next.
//! let mut frame = engine.begin(Duration::from_millis(120));
//! assert_eq!(frame.plan().steps, 2);
//! frame.record(FrameStage::World, Duration::from_millis(9))?;
//! let report = frame.finish(Duration::from_millis(12));
//!
//! // Three of the twelve milliseconds were not claimed by any stage.
//! assert_eq!(report.unattributed(), Duration::from_millis(3));
//! assert!(report.class().is_within_target());
//! # Ok::<(), nexora_foundation::error::Error>(())
//! ```

use std::time::Duration;

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// The stages one frame runs, in the order `CORE.md` §16 declares them.
///
/// One of the seven has no system behind it today: there is no audio mixer
/// in this repository, and `NEXORA DEFINITION OF DONE.md` forbids claiming
/// otherwise. It is named anyway because the *order* is the decision — a
/// stage that arrives later slots into a sequence that already exists,
/// instead of being appended wherever it happened to be written. Input
/// reached a real device with ADR-0031, and render prep and render a real
/// pass with ADR-0030; `nexora-client` runs all six (ADR-0032).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrameStage {
    /// Reading the outside world: the window host's devices (ADR-0031).
    Input,
    /// Game rules, commands, the world clock.
    Simulation,
    /// Residency, streaming, generation.
    World,
    /// Bodies, collision, the character controller.
    Physics,
    /// Turning world state into something drawable: the camera and the
    /// pass's recording (ADR-0029, ADR-0030).
    RenderPrep,
    /// Submitting it, and presenting it (ADR-0027).
    Render,
    /// Mixing. No system yet.
    Audio,
}

impl FrameStage {
    /// Every stage, in the order a frame runs them.
    pub const SEQUENCE: [Self; 7] = [
        Self::Input,
        Self::Simulation,
        Self::World,
        Self::Physics,
        Self::RenderPrep,
        Self::Render,
        Self::Audio,
    ];

    /// How many stages there are.
    pub const COUNT: usize = Self::SEQUENCE.len();

    /// The stage's position in [`FrameStage::SEQUENCE`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Input => 0,
            Self::Simulation => 1,
            Self::World => 2,
            Self::Physics => 3,
            Self::RenderPrep => 4,
            Self::Render => 5,
            Self::Audio => 6,
        }
    }

    /// A stable name for diagnostics and reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Simulation => "simulation",
            Self::World => "world",
            Self::Physics => "physics",
            Self::RenderPrep => "render-prep",
            Self::Render => "render",
            Self::Audio => "audio",
        }
    }

    /// Whether a system in this repository can run in this stage today.
    ///
    /// Reported rather than hidden: a frame report that shows four silent
    /// stages should say which of them are silent because nothing ran and which
    /// are silent because nothing *exists*.
    #[must_use]
    pub const fn has_system(self) -> bool {
        !matches!(self, Self::Audio)
    }
}

/// What one frame's worth of elapsed time buys in fixed steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepPlan {
    /// Fixed steps this frame runs.
    pub steps: u32,
    /// Steps the frame owed and will never run, because [`FrameSchedule`]
    /// refused to run more than its cap.
    ///
    /// Non-zero means simulated time fell behind real time and the gap was
    /// **dropped**, not deferred. Saturates at [`u64::MAX`], which needs a
    /// delta of geological size to reach.
    pub discarded: u64,
    /// Time left over towards the next frame, always shorter than one step.
    pub carried: Duration,
    /// The schedule's fixed step, so that `carried` can be read as a fraction
    /// of it without the schedule at hand.
    pub step: Duration,
}

impl StepPlan {
    /// How far real time has run into the next step: `carried / step`, in
    /// `[0, 1)`.
    ///
    /// What a presentation draws between the last two steps by (ADR-0040): at
    /// 0 it shows the state before the last step, and it never reaches the
    /// state after it, so what is drawn is never ahead of what was simulated.
    #[must_use]
    pub fn alpha(&self) -> f64 {
        // `carried < step` is `advance`'s invariant, so the quotient is below
        // one; both are whole nanoseconds, and a millisecond step keeps them
        // far inside the 2^53 an `f64` holds exactly.
        self.carried.as_nanos() as f64 / self.step.as_nanos() as f64
    }
}

/// A fixed timestep, an accumulator, and a cap on catching up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSchedule {
    step: Duration,
    max_steps: u32,
    carried: Duration,
}

impl FrameSchedule {
    /// Build a schedule.
    ///
    /// # Errors
    ///
    /// Returns an error when the step is zero — an accumulator divided by zero
    /// owes infinitely many steps — or when the cap is zero, which is a loop
    /// that can never simulate anything.
    pub fn new(step: Duration, max_steps: u32) -> Result<Self> {
        if step.is_zero() {
            return Err(reject("a frame schedule needs a step longer than zero"));
        }
        if max_steps == 0 {
            return Err(reject(
                "a frame schedule that runs no steps never simulates",
            ));
        }
        Ok(Self {
            step,
            max_steps,
            carried: Duration::ZERO,
        })
    }

    /// The fixed step.
    #[must_use]
    pub const fn step(&self) -> Duration {
        self.step
    }

    /// The most steps one frame will run.
    #[must_use]
    pub const fn max_steps(&self) -> u32 {
        self.max_steps
    }

    /// Time carried towards the next frame.
    #[must_use]
    pub const fn carried(&self) -> Duration {
        self.carried
    }

    /// Take one frame's elapsed real time and say what it buys.
    ///
    /// The cap is the whole point. Ten owed steps run as four, and the other
    /// six are thrown away rather than carried: carrying them would make the
    /// next frame owe them *plus* whatever it earns, which is the spiral this
    /// guard exists to stop. `StepPlan::discarded` is how the caller finds out
    /// — silently dropping simulated time is the failure mode `DEBT-0004` was.
    pub fn advance(&mut self, elapsed: Duration) -> StepPlan {
        self.carried = self.carried.saturating_add(elapsed);

        let owed = self.carried.as_nanos() / self.step.as_nanos();
        let steps = u32::try_from(owed).unwrap_or(u32::MAX).min(self.max_steps);
        let discarded = u64::try_from(owed - u128::from(steps)).unwrap_or(u64::MAX);

        let ran = self.step.checked_mul(steps).unwrap_or(Duration::MAX);
        self.carried = self.carried.saturating_sub(ran);
        if discarded > 0 {
            // Everything but the remainder goes. `carried` is below one step
            // afterwards, which is the invariant the next frame relies on.
            let dropped = self
                .step
                .checked_mul(u32::try_from(discarded).unwrap_or(u32::MAX))
                .unwrap_or(Duration::MAX);
            self.carried = self.carried.saturating_sub(dropped);
        }

        StepPlan {
            steps,
            discarded,
            carried: self.carried,
            step: self.step,
        }
    }
}

/// Where a measured duration falls against a budget.
///
/// The four names are `NEXORA PERFORMANCE BUDGETS.md`'s, not this module's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BudgetClass {
    /// Within the budget.
    Target,
    /// Past the target, below the warning line.
    Warning,
    /// Past the warning line.
    Critical,
    /// Past the critical line: the frame is a visible stall.
    Emergency,
}

impl BudgetClass {
    /// A stable name for diagnostics and reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Target => "target",
            Self::Warning => "warning",
            Self::Critical => "critical",
            Self::Emergency => "emergency",
        }
    }

    /// Whether the frame fit its budget.
    #[must_use]
    pub const fn is_within_target(self) -> bool {
        matches!(self, Self::Target)
    }
}

/// The four thresholds `NEXORA PERFORMANCE BUDGETS.md` asks every major system
/// to publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameBudget {
    target: Duration,
    warning: Duration,
    critical: Duration,
    emergency: Duration,
}

impl FrameBudget {
    /// Publish four measured thresholds.
    ///
    /// # Errors
    ///
    /// Returns an error when the target is zero, or when the thresholds do not
    /// rise: a warning line below the target would classify a frame as
    /// `Critical` that the target already called acceptable.
    pub fn new(
        target: Duration,
        warning: Duration,
        critical: Duration,
        emergency: Duration,
    ) -> Result<Self> {
        if target.is_zero() {
            return Err(reject("a frame budget needs a target longer than zero"));
        }
        if warning < target || critical < warning || emergency < critical {
            return Err(reject(
                "frame budget thresholds must not decrease from target to emergency",
            ));
        }
        Ok(Self {
            target,
            warning,
            critical,
            emergency,
        })
    }

    /// Thresholds derived from a frame period by doubling.
    ///
    /// **These are ratios, not measurements.** The target is the period itself,
    /// which is arithmetic — one frame at 20 Hz is 50 ms — and the other three
    /// are 2×, 4× and 8× it, which is a convention this constructor's name
    /// carries so that nobody reads them as measured. The only honest thresholds
    /// are the ones a system publishes from its own numbers. The first is
    /// physics' substep budget (`nexora_physics::budget`, DEBT-0013), measured
    /// on two machines; the frame as a whole has none yet, because no process
    /// runs frames against a real clock (DEBT-0041).
    #[must_use]
    pub fn doubling_from(period: Duration) -> Self {
        let scale = |factor: u32| period.checked_mul(factor).unwrap_or(Duration::MAX);
        Self {
            target: period,
            warning: scale(2),
            critical: scale(4),
            emergency: scale(8),
        }
    }

    /// The target threshold.
    #[must_use]
    pub const fn target(&self) -> Duration {
        self.target
    }

    /// The warning threshold.
    #[must_use]
    pub const fn warning(&self) -> Duration {
        self.warning
    }

    /// The critical threshold.
    #[must_use]
    pub const fn critical(&self) -> Duration {
        self.critical
    }

    /// The emergency threshold.
    #[must_use]
    pub const fn emergency(&self) -> Duration {
        self.emergency
    }

    /// Which class a measured duration falls in.
    ///
    /// A duration exactly on a threshold is *within* it: a 50 ms frame against
    /// a 50 ms target met the target rather than missing it.
    #[must_use]
    pub fn classify(&self, elapsed: Duration) -> BudgetClass {
        if elapsed <= self.target {
            BudgetClass::Target
        } else if elapsed <= self.warning {
            BudgetClass::Warning
        } else if elapsed <= self.critical {
            BudgetClass::Critical
        } else {
            BudgetClass::Emergency
        }
    }
}

/// The frame loop: a schedule, a budget, and a count of frames run.
#[derive(Debug, Clone)]
pub struct FrameLoop {
    schedule: FrameSchedule,
    budget: FrameBudget,
    frames: u64,
}

impl FrameLoop {
    /// Build a loop.
    #[must_use]
    pub const fn new(schedule: FrameSchedule, budget: FrameBudget) -> Self {
        Self {
            schedule,
            budget,
            frames: 0,
        }
    }

    /// Frames begun so far.
    #[must_use]
    pub const fn frames(&self) -> u64 {
        self.frames
    }

    /// The schedule.
    #[must_use]
    pub const fn schedule(&self) -> &FrameSchedule {
        &self.schedule
    }

    /// The budget.
    #[must_use]
    pub const fn budget(&self) -> &FrameBudget {
        &self.budget
    }

    /// Open a frame, given the real time that passed since the last one.
    ///
    /// The returned [`FrameRun`] borrows the loop for the length of the frame,
    /// so a second frame cannot be opened while one is in progress.
    pub fn begin(&mut self, elapsed: Duration) -> FrameRun<'_> {
        self.frames = self.frames.wrapping_add(1);
        let number = self.frames;
        let plan = self.schedule.advance(elapsed);
        FrameRun {
            number,
            plan,
            budget: &self.budget,
            spans: [Duration::ZERO; FrameStage::COUNT],
            charged: [false; FrameStage::COUNT],
            last_recorded: None,
            presentation_wait: None,
        }
    }
}

/// A frame in progress.
#[derive(Debug)]
pub struct FrameRun<'a> {
    number: u64,
    plan: StepPlan,
    budget: &'a FrameBudget,
    spans: [Duration; FrameStage::COUNT],
    charged: [bool; FrameStage::COUNT],
    last_recorded: Option<usize>,
    presentation_wait: Option<Duration>,
}

impl FrameRun<'_> {
    /// Which frame this is, counting from one.
    #[must_use]
    pub const fn number(&self) -> u64 {
        self.number
    }

    /// What the elapsed time bought.
    #[must_use]
    pub const fn plan(&self) -> StepPlan {
        self.plan
    }

    /// Charge elapsed time to the stage that spent it.
    ///
    /// # Errors
    ///
    /// Returns an error when a stage is charged twice, or charged after a later
    /// stage already was. Both are refused rather than merged, because both are
    /// ways for one subsystem's cost to end up on another's line — which is the
    /// invisibility `NEXORA PERFORMANCE BUDGETS.md` forbids. Skipping a stage is
    /// fine: four of the seven have nothing to run.
    pub fn record(&mut self, stage: FrameStage, spent: Duration) -> Result<()> {
        let index = stage.index();
        if let Some(last) = self.last_recorded {
            if index == last {
                return Err(
                    reject("a frame stage was charged twice").with_context("stage", stage.as_str())
                );
            }
            if index < last {
                return Err(reject("a frame stage was charged after a later stage")
                    .with_context("stage", stage.as_str())
                    .with_context("after", FrameStage::SEQUENCE[last].as_str()));
            }
        }
        self.spans[index] = spent;
        self.charged[index] = true;
        self.last_recorded = Some(index);
        Ok(())
    }

    /// Declare how much of a charged stage was spent waiting on presentation:
    /// blocked until the window system took the frame, which needs a free
    /// surface image (the display's refresh, the GPU's earlier frames).
    ///
    /// The wait stays in the stage's line, and in the frame's wall time, where
    /// it happened. What changes is what the budget classifies: the frame's
    /// work, its wall time less this wait ([`FrameReport::work`]).
    ///
    /// # Errors
    ///
    /// Returns an error when `stage` has not been charged, when the wait is
    /// longer than the stage that contains it, or when a wait was already
    /// declared this frame. Each of those would let presentation absorb time
    /// some subsystem spent, which is the invisibility this module refuses.
    pub fn waited_for_presentation(&mut self, stage: FrameStage, waited: Duration) -> Result<()> {
        let index = stage.index();
        if !self.charged[index] {
            return Err(
                reject("a presentation wait names a stage that was not charged")
                    .with_context("stage", stage.as_str()),
            );
        }
        if waited > self.spans[index] {
            return Err(
                reject("a presentation wait is longer than the stage that contains it")
                    .with_context("stage", stage.as_str())
                    .with_context("waited_us", waited.as_micros().to_string())
                    .with_context("stage_us", self.spans[index].as_micros().to_string()),
            );
        }
        if self.presentation_wait.is_some() {
            return Err(reject(
                "a presentation wait was declared twice in one frame",
            ));
        }
        self.presentation_wait = Some(waited);
        Ok(())
    }

    /// Close the frame, given how long the whole frame took.
    ///
    /// `wall` is measured by the host around everything the frame did, stages
    /// included. Less any presentation wait, it is what the budget
    /// classifies, and the difference between it and the sum of the stages
    /// is [`FrameReport::unattributed`].
    #[must_use]
    pub fn finish(self, wall: Duration) -> FrameReport {
        let attributed = self
            .spans
            .iter()
            .fold(Duration::ZERO, |total, span| total.saturating_add(*span));
        let presentation_wait = self.presentation_wait.unwrap_or_default();
        FrameReport {
            number: self.number,
            plan: self.plan,
            spans: self.spans,
            wall,
            attributed,
            presentation_wait,
            class: self.budget.classify(wall.saturating_sub(presentation_wait)),
        }
    }
}

/// What one frame did and what it cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameReport {
    number: u64,
    plan: StepPlan,
    spans: [Duration; FrameStage::COUNT],
    wall: Duration,
    attributed: Duration,
    presentation_wait: Duration,
    class: BudgetClass,
}

impl FrameReport {
    /// Which frame this was, counting from one.
    #[must_use]
    pub const fn number(&self) -> u64 {
        self.number
    }

    /// What the elapsed time bought.
    #[must_use]
    pub const fn plan(&self) -> StepPlan {
        self.plan
    }

    /// How long the whole frame took, as the host measured it.
    #[must_use]
    pub const fn wall(&self) -> Duration {
        self.wall
    }

    /// How much of the frame was spent waiting on presentation
    /// ([`FrameRun::waited_for_presentation`]); zero when none was declared.
    #[must_use]
    pub const fn presentation_wait(&self) -> Duration {
        self.presentation_wait
    }

    /// The frame's work: its wall time less the presentation wait. This is what
    /// the budget classifies.
    #[must_use]
    pub fn work(&self) -> Duration {
        self.wall.saturating_sub(self.presentation_wait)
    }

    /// What one stage was charged.
    #[must_use]
    pub const fn stage(&self, stage: FrameStage) -> Duration {
        self.spans[stage.index()]
    }

    /// The sum of every stage.
    #[must_use]
    pub const fn attributed(&self) -> Duration {
        self.attributed
    }

    /// Time inside the frame that no stage claimed.
    ///
    /// This is the number that makes attribution mean something. The sum of the
    /// stages equals itself no matter what the host reports; the gap between
    /// that sum and the frame's own wall time is work nobody owned — a lock
    /// waited on, an allocation, a stage the host forgot to charge.
    #[must_use]
    pub fn unattributed(&self) -> Duration {
        self.wall.saturating_sub(self.attributed)
    }

    /// Time the stages claimed beyond the frame's own length.
    ///
    /// Non-zero means the host's own accounting is wrong — stages cannot
    /// together outlast the frame that contains them — so it is surfaced rather
    /// than clamped to zero and forgotten.
    #[must_use]
    pub fn overattributed(&self) -> Duration {
        self.attributed.saturating_sub(self.wall)
    }

    /// Which budget class the frame's work fell in.
    #[must_use]
    pub const fn class(&self) -> BudgetClass {
        self.class
    }

    /// Whether the frame missed its target.
    #[must_use]
    pub const fn is_over_budget(&self) -> bool {
        !self.class.is_within_target()
    }
}

fn reject(message: &'static str) -> Error {
    Error::new(Domain::Core, "frame", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: Duration = Duration::from_millis(50);

    fn schedule(max_steps: u32) -> FrameSchedule {
        FrameSchedule::new(STEP, max_steps).expect("a 50 ms step is valid")
    }

    fn engine(max_steps: u32) -> FrameLoop {
        FrameLoop::new(schedule(max_steps), FrameBudget::doubling_from(STEP))
    }

    #[test]
    fn the_stage_order_is_the_one_core_declares() {
        // `CORE.md` §16: Input -> Simulation -> World -> Physics -> Render
        // Preparation -> Render -> Audio. Written out so reordering the enum
        // has to be a deliberate edit to this list as well.
        let names: Vec<&str> = FrameStage::SEQUENCE
            .iter()
            .map(|stage| stage.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "input",
                "simulation",
                "world",
                "physics",
                "render-prep",
                "render",
                "audio"
            ]
        );
        for (position, stage) in FrameStage::SEQUENCE.iter().enumerate() {
            assert_eq!(
                stage.index(),
                position,
                "{} is out of place",
                stage.as_str()
            );
        }
    }

    #[test]
    fn only_the_stages_with_a_system_say_they_have_one() {
        let staffed: Vec<&str> = FrameStage::SEQUENCE
            .iter()
            .filter(|stage| stage.has_system())
            .map(|stage| stage.as_str())
            .collect();
        assert_eq!(
            staffed,
            [
                "input",
                "simulation",
                "world",
                "physics",
                "render-prep",
                "render"
            ]
        );
    }

    #[test]
    fn a_schedule_needs_a_step_and_a_cap() {
        assert!(FrameSchedule::new(Duration::ZERO, 4).is_err());
        assert!(FrameSchedule::new(STEP, 0).is_err());
        assert!(FrameSchedule::new(STEP, 1).is_ok());
    }

    #[test]
    fn short_frames_carry_until_they_add_up_to_a_step() {
        let mut plan = schedule(4);
        let first = plan.advance(Duration::from_millis(30));
        assert_eq!(first.steps, 0);
        assert_eq!(first.carried, Duration::from_millis(30));

        let second = plan.advance(Duration::from_millis(30));
        assert_eq!(second.steps, 1, "60 ms owes one 50 ms step");
        assert_eq!(second.carried, Duration::from_millis(10));
    }

    #[test]
    fn alpha_is_how_far_into_the_next_step_real_time_has_run() {
        let mut plan = schedule(8);
        // Nothing has run yet: the state shown is the state there is.
        assert_eq!(
            plan.advance(Duration::ZERO).alpha().to_bits(),
            0.0f64.to_bits()
        );
        let frame = plan.advance(Duration::from_millis(120));
        assert_eq!(frame.steps, 2);
        assert_eq!(frame.alpha(), 0.4, "20 ms into a 50 ms step");
        // Frames shorter than a step move alpha on without running a step.
        let next = plan.advance(Duration::from_millis(15));
        assert_eq!(next.steps, 0);
        assert_eq!(next.alpha(), 0.7);
        // And a step lands it back near zero, never on one.
        let wrapped = plan.advance(Duration::from_millis(15));
        assert_eq!(wrapped.steps, 1);
        assert_eq!(wrapped.alpha(), 0.0);
        // After a hitch the backlog is gone, and alpha is still below one.
        let hitch = plan.advance(Duration::from_millis(10_049));
        assert!(hitch.discarded > 0);
        assert!((0.0..1.0).contains(&hitch.alpha()), "{}", hitch.alpha());
    }

    #[test]
    fn a_long_frame_runs_whole_steps_and_keeps_the_remainder() {
        let mut plan = schedule(8);
        let frame = plan.advance(Duration::from_millis(120));
        assert_eq!(frame.steps, 2);
        assert_eq!(frame.discarded, 0);
        assert_eq!(frame.carried, Duration::from_millis(20));
    }

    #[test]
    fn a_hitch_is_capped_and_says_how_many_steps_it_dropped() {
        let mut plan = schedule(4);
        let frame = plan.advance(Duration::from_millis(500));
        assert_eq!(frame.steps, 4, "the cap holds");
        assert_eq!(frame.discarded, 6, "ten owed, four run");
        assert!(
            frame.carried < STEP,
            "the accumulator must not hold a whole step after a capped frame"
        );
    }

    /// The test the cap exists for: what it dropped must be **gone**.
    ///
    /// Remove the `discarded` branch in `advance` and this fails — the frame
    /// after a hitch would run the cap again, and again, working off a backlog
    /// instead of returning to real time.
    #[test]
    fn the_backlog_a_hitch_leaves_is_discarded_not_carried() {
        let mut plan = schedule(4);
        plan.advance(Duration::from_millis(500));

        let after = plan.advance(STEP);
        assert_eq!(
            after.steps, 1,
            "the frame after a hitch owes one step, not the six the hitch dropped"
        );
        assert_eq!(after.discarded, 0);
    }

    #[test]
    fn a_frame_that_owes_exactly_the_cap_discards_nothing() {
        let mut plan = schedule(4);
        let frame = plan.advance(Duration::from_millis(200));
        assert_eq!(frame.steps, 4);
        assert_eq!(frame.discarded, 0, "four owed against a cap of four");
    }

    #[test]
    fn the_same_deltas_produce_the_same_frames() {
        let deltas = [17, 50, 3, 900, 50, 50, 31, 0, 120];
        let run = |_: u8| {
            let mut plan = schedule(4);
            deltas
                .iter()
                .map(|ms| plan.advance(Duration::from_millis(*ms)))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(0), run(1), "the schedule reads no clock of its own");
    }

    #[test]
    fn budget_thresholds_have_to_rise() {
        let ms = Duration::from_millis;
        assert!(FrameBudget::new(ms(10), ms(20), ms(40), ms(80)).is_ok());
        assert!(FrameBudget::new(Duration::ZERO, ms(20), ms(40), ms(80)).is_err());
        assert!(
            FrameBudget::new(ms(10), ms(5), ms(40), ms(80)).is_err(),
            "a warning line below the target classifies an acceptable frame as critical"
        );
        assert!(FrameBudget::new(ms(10), ms(20), ms(15), ms(80)).is_err());
        assert!(FrameBudget::new(ms(10), ms(20), ms(40), ms(30)).is_err());
    }

    #[test]
    fn a_frame_exactly_on_a_threshold_is_inside_it() {
        let budget = FrameBudget::doubling_from(STEP);
        assert_eq!(budget.classify(STEP), BudgetClass::Target);
        assert_eq!(
            budget.classify(Duration::from_millis(100)),
            BudgetClass::Warning
        );
        assert_eq!(
            budget.classify(Duration::from_millis(200)),
            BudgetClass::Critical
        );
        assert_eq!(
            budget.classify(Duration::from_millis(201)),
            BudgetClass::Emergency
        );
    }

    #[test]
    fn a_stage_cannot_be_charged_twice() {
        let mut engine = engine(4);
        let mut frame = engine.begin(STEP);
        frame
            .record(FrameStage::World, Duration::from_millis(1))
            .expect("first charge");
        assert!(
            frame
                .record(FrameStage::World, Duration::from_millis(1))
                .is_err(),
            "charging a stage twice would double its line and hide where the time went"
        );
    }

    #[test]
    fn a_stage_cannot_be_charged_after_a_later_one() {
        let mut engine = engine(4);
        let mut frame = engine.begin(STEP);
        frame
            .record(FrameStage::Physics, Duration::from_millis(1))
            .expect("physics");
        assert!(
            frame
                .record(FrameStage::World, Duration::from_millis(1))
                .is_err(),
            "the sequence is the contract; charging backwards rewrites it after the fact"
        );
    }

    #[test]
    fn stages_with_nothing_to_run_can_be_skipped() {
        let mut engine = engine(4);
        let mut frame = engine.begin(STEP);
        frame
            .record(FrameStage::Simulation, Duration::from_millis(1))
            .expect("simulation");
        frame
            .record(FrameStage::Physics, Duration::from_millis(2))
            .expect("physics, with world skipped");
        let report = frame.finish(Duration::from_millis(3));
        assert_eq!(report.stage(FrameStage::World), Duration::ZERO);
        assert_eq!(report.attributed(), Duration::from_millis(3));
    }

    /// The property the whole module exists for: time nobody claimed is visible.
    #[test]
    fn time_no_stage_claimed_is_reported_rather_than_absorbed() {
        let mut engine = engine(4);
        let mut frame = engine.begin(STEP);
        frame
            .record(FrameStage::Simulation, Duration::from_millis(2))
            .expect("simulation");
        frame
            .record(FrameStage::World, Duration::from_millis(1))
            .expect("world");
        let report = frame.finish(Duration::from_millis(10));

        assert_eq!(report.attributed(), Duration::from_millis(3));
        assert_eq!(
            report.unattributed(),
            Duration::from_millis(7),
            "seven of the ten milliseconds belong to no subsystem's budget"
        );
        assert_eq!(report.overattributed(), Duration::ZERO);
    }

    #[test]
    fn a_host_that_charges_more_than_the_frame_lasted_is_caught() {
        let mut engine = engine(4);
        let mut frame = engine.begin(STEP);
        frame
            .record(FrameStage::World, Duration::from_millis(30))
            .expect("world");
        let report = frame.finish(Duration::from_millis(10));
        assert_eq!(report.unattributed(), Duration::ZERO);
        assert_eq!(
            report.overattributed(),
            Duration::from_millis(20),
            "stages cannot together outlast the frame containing them"
        );
    }

    #[test]
    fn the_budget_classifies_the_frame_not_the_sum_of_its_stages() {
        let mut engine = engine(4);
        let mut frame = engine.begin(STEP);
        // One millisecond of stage work inside a frame that took a quarter of a
        // second: the stall was real even though no stage owned it.
        frame
            .record(FrameStage::World, Duration::from_millis(1))
            .expect("world");
        let report = frame.finish(Duration::from_millis(250));
        assert_eq!(report.class(), BudgetClass::Emergency);
        assert!(report.is_over_budget());
    }

    /// Finding 33, replayed: a frame that works for 1.5 ms and then waits
    /// 8.5 ms for a 100 Hz monitor is a 1.5 ms frame to the budget. Its wall
    /// time and its render line still show the ten.
    #[test]
    fn the_budget_classifies_work_not_the_wait_for_presentation() {
        let ms = Duration::from_millis;
        let budget = FrameBudget::new(ms(4), ms(8), ms(16), ms(32)).expect("rising");
        let mut engine = FrameLoop::new(schedule(4), budget);
        let mut frame = engine.begin(STEP);
        frame.record(FrameStage::RenderPrep, ms(1)).expect("prep");
        frame.record(FrameStage::Render, ms(9)).expect("render");
        frame
            .waited_for_presentation(FrameStage::Render, Duration::from_micros(8_500))
            .expect("the wait is inside render");
        let report = frame.finish(ms(10));

        assert_eq!(report.wall(), ms(10));
        assert_eq!(report.stage(FrameStage::Render), ms(9));
        assert_eq!(report.presentation_wait(), Duration::from_micros(8_500));
        assert_eq!(report.work(), Duration::from_micros(1_500));
        assert_eq!(report.class(), BudgetClass::Target);
        assert_eq!(
            report.unattributed(),
            Duration::ZERO,
            "the wait is attributed to render"
        );
    }

    /// Without the wait the same frame is critical: the monitor, not the
    /// engine, would be what the budget judged.
    #[test]
    fn a_frame_that_declares_no_wait_is_classified_by_its_wall_time() {
        let ms = Duration::from_millis;
        let budget = FrameBudget::new(ms(4), ms(8), ms(16), ms(32)).expect("rising");
        let mut engine = FrameLoop::new(schedule(4), budget);
        let mut frame = engine.begin(STEP);
        frame.record(FrameStage::Render, ms(9)).expect("render");
        let report = frame.finish(ms(10));
        assert_eq!(report.presentation_wait(), Duration::ZERO);
        assert_eq!(report.work(), ms(10));
        assert_eq!(report.class(), BudgetClass::Critical);
    }

    #[test]
    fn a_presentation_wait_must_sit_inside_a_charged_stage() {
        let ms = Duration::from_millis;
        let mut engine = engine(4);
        let mut frame = engine.begin(STEP);
        assert!(
            frame
                .waited_for_presentation(FrameStage::Render, ms(1))
                .is_err(),
            "render was never charged, so nothing contains the wait"
        );
        frame.record(FrameStage::Render, ms(2)).expect("render");
        assert!(
            frame
                .waited_for_presentation(FrameStage::Render, ms(3))
                .is_err(),
            "a 3 ms wait cannot hide inside a 2 ms stage"
        );
        frame
            .waited_for_presentation(FrameStage::Render, ms(2))
            .expect("a stage that was all waiting");
        assert!(
            frame
                .waited_for_presentation(FrameStage::Render, ms(1))
                .is_err(),
            "a second wait would count presentation twice"
        );
    }

    /// A stage charged with zero time is still charged: a wait of zero may
    /// name it, a longer one may not.
    #[test]
    fn a_stage_charged_with_nothing_holds_only_a_zero_wait() {
        let mut engine = engine(4);
        let mut frame = engine.begin(STEP);
        frame
            .record(FrameStage::Render, Duration::ZERO)
            .expect("render");
        assert!(frame
            .waited_for_presentation(FrameStage::Render, Duration::from_nanos(1))
            .is_err());
        frame
            .waited_for_presentation(FrameStage::Render, Duration::ZERO)
            .expect("zero fits");
    }

    #[test]
    fn frames_are_numbered_from_one_and_keep_counting() {
        let mut engine = engine(4);
        assert_eq!(engine.frames(), 0);
        assert_eq!(engine.begin(STEP).number(), 1);
        assert_eq!(engine.begin(STEP).number(), 2);
        assert_eq!(engine.frames(), 2);
    }

    #[test]
    fn a_frame_carries_the_plan_through_to_the_report() {
        let mut engine = engine(4);
        let frame = engine.begin(Duration::from_millis(500));
        let plan = frame.plan();
        let report = frame.finish(Duration::from_millis(500));
        assert_eq!(report.plan(), plan);
        assert_eq!(report.plan().steps, 4);
        assert_eq!(report.plan().discarded, 6);
    }
}
