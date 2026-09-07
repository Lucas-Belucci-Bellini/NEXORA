//! Runtime lifecycle.
//!
//! Implements `NEXORA RUNTIME LIFECYCLE.md`. That document exists to *prevent
//! hidden initialization order dependencies*, so this module makes the order a
//! checked state machine rather than a convention: a runtime cannot reach
//! `SimulationRunning` without having passed every required phase before it,
//! and it cannot report a partial initialization as a running state.

use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// Which build of the runtime is executing.
///
/// Modes select different module sets but share the same lifecycle contracts
/// (`NEXORA RUNTIME LIFECYCLE.md`, "Rules").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RuntimeMode {
    /// Full client with presentation.
    Client,
    /// Dedicated server, no presentation.
    DedicatedServer,
    /// Client that also hosts the authoritative simulation.
    ListenServer,
    /// Simulation only, no window and no renderer.
    Headless,
    /// Authoring tools attached to a runtime.
    Editor,
    /// Automated tests.
    Test,
    /// Benchmark harness.
    Benchmark,
    /// Replay playback.
    Replay,
}

impl RuntimeMode {
    /// Whether this mode drives a presentation layer.
    #[must_use]
    pub const fn has_presentation(self) -> bool {
        matches!(self, Self::Client | Self::ListenServer | Self::Editor)
    }

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::DedicatedServer => "dedicated-server",
            Self::ListenServer => "listen-server",
            Self::Headless => "headless",
            Self::Editor => "editor",
            Self::Test => "test",
            Self::Benchmark => "benchmark",
            Self::Replay => "replay",
        }
    }
}

/// The ordered phases of a runtime process.
///
/// The sequence is taken verbatim from `NEXORA RUNTIME LIFECYCLE.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// The process has started; nothing is initialized.
    ProcessStart,
    /// Platform abstractions are available.
    PlatformInit,
    /// Foundation services are available.
    FoundationInit,
    /// Engine modules have been discovered.
    ModuleDiscovery,
    /// Module dependencies have been resolved and ordered.
    ModuleResolution,
    /// Bootstrap resources are loaded.
    ResourceBootstrap,
    /// Runtime services are initialized.
    RuntimeInit,
    /// A world has been attached to the runtime.
    WorldAttach,
    /// The simulation is advancing.
    SimulationRunning,
    /// The presentation layer is running.
    PresentationRunning,
    /// A shutdown has been requested.
    ShutdownRequested,
    /// The simulation has stopped.
    SimulationStop,
    /// World state has been flushed to storage.
    WorldFlush,
    /// Modules have stopped, in reverse dependency order.
    ModuleStop,
    /// Resources have been released.
    ResourceRelease,
    /// Platform abstractions have been torn down.
    PlatformShutdown,
    /// The process is ready to exit.
    ProcessExit,
}

impl Phase {
    /// Every phase, in order.
    pub const SEQUENCE: [Self; 17] = [
        Self::ProcessStart,
        Self::PlatformInit,
        Self::FoundationInit,
        Self::ModuleDiscovery,
        Self::ModuleResolution,
        Self::ResourceBootstrap,
        Self::RuntimeInit,
        Self::WorldAttach,
        Self::SimulationRunning,
        Self::PresentationRunning,
        Self::ShutdownRequested,
        Self::SimulationStop,
        Self::WorldFlush,
        Self::ModuleStop,
        Self::ResourceRelease,
        Self::PlatformShutdown,
        Self::ProcessExit,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProcessStart => "process-start",
            Self::PlatformInit => "platform-init",
            Self::FoundationInit => "foundation-init",
            Self::ModuleDiscovery => "module-discovery",
            Self::ModuleResolution => "module-resolution",
            Self::ResourceBootstrap => "resource-bootstrap",
            Self::RuntimeInit => "runtime-init",
            Self::WorldAttach => "world-attach",
            Self::SimulationRunning => "simulation-running",
            Self::PresentationRunning => "presentation-running",
            Self::ShutdownRequested => "shutdown-requested",
            Self::SimulationStop => "simulation-stop",
            Self::WorldFlush => "world-flush",
            Self::ModuleStop => "module-stop",
            Self::ResourceRelease => "resource-release",
            Self::PlatformShutdown => "platform-shutdown",
            Self::ProcessExit => "process-exit",
        }
    }

    /// Whether a mode must pass through this phase.
    ///
    /// Only presentation is optional today: a headless or dedicated-server
    /// process has no renderer to start, which is exactly the "different module
    /// selections, same lifecycle contracts" rule.
    #[must_use]
    pub const fn required_for(self, mode: RuntimeMode) -> bool {
        match self {
            Self::PresentationRunning => mode.has_presentation(),
            _ => true,
        }
    }

    /// Position in [`Phase::SEQUENCE`].
    #[must_use]
    pub fn index(self) -> usize {
        Self::SEQUENCE
            .iter()
            .position(|phase| *phase == self)
            .unwrap_or(0)
    }
}

/// Coarse runtime health, from `NEXORA RUNTIME LIFECYCLE.md`, "Safe states".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// Initializing; not yet safe to use.
    Booting,
    /// Initialized and idle.
    Ready,
    /// Simulating.
    Running,
    /// Transitioning to paused.
    Pausing,
    /// Paused; state is intact but not advancing.
    Paused,
    /// Shutting down.
    Stopping,
    /// Initialization or operation failed; not a usable runtime.
    Failed,
}

/// Tracks the phase and state of one runtime process.
#[derive(Debug)]
pub struct Lifecycle {
    mode: RuntimeMode,
    phase: Phase,
    state: State,
    failure: Option<Error>,
    history: Vec<Phase>,
}

impl Lifecycle {
    /// Start a lifecycle at [`Phase::ProcessStart`].
    #[must_use]
    pub fn new(mode: RuntimeMode) -> Self {
        Self {
            mode,
            phase: Phase::ProcessStart,
            state: State::Booting,
            failure: None,
            history: vec![Phase::ProcessStart],
        }
    }

    /// The runtime mode.
    #[must_use]
    pub const fn mode(&self) -> RuntimeMode {
        self.mode
    }

    /// The current phase.
    #[must_use]
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    /// The current state.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// The recorded failure, if the runtime has failed.
    #[must_use]
    pub const fn failure(&self) -> Option<&Error> {
        self.failure.as_ref()
    }

    /// Every phase entered so far, in order.
    #[must_use]
    pub fn history(&self) -> &[Phase] {
        &self.history
    }

    /// The next phase this mode must enter, if any.
    #[must_use]
    pub fn next_phase(&self) -> Option<Phase> {
        Phase::SEQUENCE
            .iter()
            .skip(self.phase.index() + 1)
            .copied()
            .find(|phase| phase.required_for(self.mode))
    }

    /// Enter the next phase.
    ///
    /// # Errors
    ///
    /// Returns an error when `target` is not the next required phase, or when
    /// the runtime has already failed. Skipping a phase is rejected rather than
    /// tolerated: a skipped phase is precisely the hidden ordering dependency
    /// this state machine exists to catch.
    pub fn advance_to(&mut self, target: Phase) -> Result<()> {
        if self.state == State::Failed {
            return Err(self
                .error("cannot advance a failed runtime")
                .with_context("attempted", target.as_str()));
        }

        let expected = self.next_phase().ok_or_else(|| {
            self.error("the runtime has already reached its final phase")
                .with_context("attempted", target.as_str())
        })?;

        if target != expected {
            return Err(self
                .error("lifecycle phases must be entered in order")
                .with_context("attempted", target.as_str())
                .with_context("expected", expected.as_str())
                .with_context("mode", self.mode.as_str()));
        }

        self.phase = target;
        self.history.push(target);
        self.state = match target {
            Phase::SimulationRunning | Phase::PresentationRunning => State::Running,
            Phase::WorldAttach => State::Ready,
            Phase::ShutdownRequested
            | Phase::SimulationStop
            | Phase::WorldFlush
            | Phase::ModuleStop
            | Phase::ResourceRelease
            | Phase::PlatformShutdown
            | Phase::ProcessExit => State::Stopping,
            _ => State::Booting,
        };
        Ok(())
    }

    /// Advance through every phase up to and including `target`.
    ///
    /// # Errors
    ///
    /// Returns an error when `target` is behind the current phase or is not
    /// required for this mode.
    pub fn advance_through(&mut self, target: Phase) -> Result<()> {
        if !target.required_for(self.mode) {
            return Err(self
                .error("phase is not part of this runtime mode")
                .with_context("attempted", target.as_str())
                .with_context("mode", self.mode.as_str()));
        }
        if target.index() < self.phase.index() {
            return Err(self
                .error("the lifecycle cannot move backwards")
                .with_context("attempted", target.as_str())
                .with_context("current", self.phase.as_str()));
        }
        while self.phase != target {
            let next = self
                .next_phase()
                .ok_or_else(|| self.error("ran out of phases before reaching the target"))?;
            self.advance_to(next)?;
        }
        Ok(())
    }

    /// Pause a running runtime.
    ///
    /// # Errors
    ///
    /// Returns an error unless the runtime is currently running.
    pub fn pause(&mut self) -> Result<()> {
        if self.state != State::Running {
            return Err(self
                .error("only a running runtime can be paused")
                .with_context("state", format!("{:?}", self.state)));
        }
        self.state = State::Paused;
        Ok(())
    }

    /// Resume a paused runtime.
    ///
    /// # Errors
    ///
    /// Returns an error unless the runtime is currently paused.
    pub fn resume(&mut self) -> Result<()> {
        if self.state != State::Paused {
            return Err(self
                .error("only a paused runtime can be resumed")
                .with_context("state", format!("{:?}", self.state)));
        }
        self.state = State::Running;
        Ok(())
    }

    /// Record a failure.
    ///
    /// A failed runtime stays failed: partial initialization must never be
    /// mistaken for a valid running state.
    pub fn fail(&mut self, cause: Error) {
        self.state = State::Failed;
        self.failure = Some(cause);
    }

    /// Whether the simulation may advance right now.
    #[must_use]
    pub const fn is_simulating(&self) -> bool {
        matches!(self.state, State::Running)
    }

    fn error(&self, message: &'static str) -> Error {
        Error::new(Domain::Core, "lifecycle", message)
            .with_recovery(Recovery::Manual)
            .with_context("phase", self.phase.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_headless_runtime_skips_presentation() {
        let mut lifecycle = Lifecycle::new(RuntimeMode::Headless);
        lifecycle.advance_through(Phase::SimulationRunning).unwrap();

        // The next required phase jumps straight over presentation.
        assert_eq!(lifecycle.next_phase(), Some(Phase::ShutdownRequested));
        lifecycle.advance_to(Phase::ShutdownRequested).unwrap();
        assert!(!lifecycle.history().contains(&Phase::PresentationRunning));
    }

    #[test]
    fn a_client_runtime_must_start_presentation() {
        let mut lifecycle = Lifecycle::new(RuntimeMode::Client);
        lifecycle.advance_through(Phase::SimulationRunning).unwrap();
        assert_eq!(lifecycle.next_phase(), Some(Phase::PresentationRunning));

        let err = lifecycle
            .advance_to(Phase::ShutdownRequested)
            .expect_err("a client cannot skip presentation");
        assert!(err.to_string().contains("in order"), "{err}");
    }

    #[test]
    fn phases_cannot_be_skipped() {
        let mut lifecycle = Lifecycle::new(RuntimeMode::Headless);
        let err = lifecycle
            .advance_to(Phase::SimulationRunning)
            .expect_err("skipping initialization must be refused");
        assert!(err.to_string().contains("expected=platform-init"), "{err}");
        // The failed attempt did not move the lifecycle.
        assert_eq!(lifecycle.phase(), Phase::ProcessStart);
    }

    #[test]
    fn the_lifecycle_cannot_run_backwards() {
        let mut lifecycle = Lifecycle::new(RuntimeMode::Headless);
        lifecycle.advance_through(Phase::RuntimeInit).unwrap();
        assert!(lifecycle.advance_through(Phase::PlatformInit).is_err());
    }

    #[test]
    fn a_failed_runtime_never_reports_as_running() {
        let mut lifecycle = Lifecycle::new(RuntimeMode::Headless);
        lifecycle.advance_through(Phase::ResourceBootstrap).unwrap();
        lifecycle.fail(Error::new(Domain::Module, "renderer", "device not found"));

        assert_eq!(lifecycle.state(), State::Failed);
        assert!(!lifecycle.is_simulating());
        assert!(lifecycle.failure().is_some());
        // Partial initialization cannot be continued into a running state.
        assert!(lifecycle.advance_to(Phase::RuntimeInit).is_err());
    }

    #[test]
    fn pause_and_resume_require_the_right_state() {
        let mut lifecycle = Lifecycle::new(RuntimeMode::Headless);
        assert!(lifecycle.pause().is_err(), "cannot pause while booting");

        lifecycle.advance_through(Phase::SimulationRunning).unwrap();
        assert!(lifecycle.is_simulating());

        lifecycle.pause().unwrap();
        assert_eq!(lifecycle.state(), State::Paused);
        assert!(!lifecycle.is_simulating());
        assert!(lifecycle.pause().is_err(), "cannot pause twice");

        lifecycle.resume().unwrap();
        assert!(lifecycle.is_simulating());
        assert!(lifecycle.resume().is_err(), "cannot resume while running");
    }

    #[test]
    fn a_full_run_reaches_process_exit_in_order() {
        for mode in [
            RuntimeMode::Headless,
            RuntimeMode::Client,
            RuntimeMode::DedicatedServer,
        ] {
            let mut lifecycle = Lifecycle::new(mode);
            lifecycle.advance_through(Phase::ProcessExit).unwrap();

            assert_eq!(lifecycle.phase(), Phase::ProcessExit);
            assert_eq!(lifecycle.next_phase(), None);

            let history = lifecycle.history();
            let mut ascending = history.iter().map(|phase| phase.index());
            let mut previous = ascending.next().expect("history is never empty");
            for index in ascending {
                assert!(index > previous, "history went backwards in mode {mode:?}");
                previous = index;
            }
            assert_eq!(
                history.contains(&Phase::PresentationRunning),
                mode.has_presentation(),
                "presentation handling is wrong for {mode:?}"
            );
        }
    }
}
