//! Structured error taxonomy.
//!
//! Implements `CORE.md` §14 (CORE-11 Error System) and the "fail loudly" rule:
//! a failure carries *who owns it* and *how it can be recovered*, never just a
//! string. `NEXORA DATA VALIDATION AND INVARIANTS.md` requires that internal
//! invariant failures identify the owner and the recovery path, so those two
//! fields are mandatory rather than optional.

use core::fmt;

/// Which architectural area owns a failure.
///
/// The domain is not a category for pretty-printing: it names the system that
/// is responsible for the failure and therefore for fixing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Domain {
    /// Engine bootstrap and lifecycle sequencing.
    Core,
    /// Engine module discovery, resolution and lifecycle.
    Module,
    /// Configuration parsing, validation and migration.
    Config,
    /// Registry registration, freezing and resolution.
    Registry,
    /// Event bus publication and dispatch.
    Event,
    /// Job scheduling and worker pools.
    Job,
    /// World lifecycle and world-level state.
    World,
    /// Chunk, section and voxel storage.
    Chunk,
    /// Save container reading, writing and migration.
    Save,
    /// Networking and replication.
    Network,
    /// Content, assets and resource packs.
    Content,
    /// Authority, sandboxing and trust boundaries.
    Security,
    /// World clock and calendar.
    Time,
    /// Coordinate spaces and conversions.
    Spatial,
    /// Rigid bodies, collision and physical queries.
    Physics,
}

impl Domain {
    /// Stable lowercase name, safe to emit in logs and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Module => "module",
            Self::Config => "config",
            Self::Registry => "registry",
            Self::Event => "event",
            Self::Job => "job",
            Self::World => "world",
            Self::Chunk => "chunk",
            Self::Save => "save",
            Self::Network => "network",
            Self::Content => "content",
            Self::Security => "security",
            Self::Time => "time",
            Self::Spatial => "spatial",
            Self::Physics => "physics",
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether the runtime may continue after this failure.
///
/// `CORE.md` §14 is explicit that an error must not simply crash the process.
/// A recoverable error may disable a subsystem and keep running; a fatal error
/// means the runtime cannot honour its own contracts any more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// The owning subsystem can be isolated, quarantined or retried.
    Recoverable,
    /// Continuing would mean running with corrupted or undefined state.
    Fatal,
}

/// The action that can repair, or safely contain, a failure.
///
/// Recording the recovery path at the point of failure is what makes
/// quarantine-instead-of-corrupt possible: the caller does not have to guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Recovery {
    /// The operation is safe to attempt again unchanged.
    Retry,
    /// Move the offending data aside and continue without it.
    Quarantine,
    /// A schema migration exists or must be written.
    Migrate,
    /// Reject the input; it came from outside the trust boundary.
    Reject,
    /// Turn off the owning subsystem and keep the rest of the runtime alive.
    DisableSubsystem,
    /// No automatic recovery: an operator or a code change is required.
    Manual,
}

impl Recovery {
    /// Stable lowercase name, safe to emit in logs and telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Retry => "retry",
            Self::Quarantine => "quarantine",
            Self::Migrate => "migrate",
            Self::Reject => "reject",
            Self::DisableSubsystem => "disable-subsystem",
            Self::Manual => "manual",
        }
    }
}

/// A NEXORA failure.
///
/// Errors carry structured context pairs instead of interpolating everything
/// into one message string, so that diagnostics and telemetry can index them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    domain: Domain,
    severity: Severity,
    recovery: Recovery,
    owner: &'static str,
    message: String,
    context: Vec<(&'static str, String)>,
    source: Option<Box<Error>>,
}

impl Error {
    /// Create a recoverable error whose recovery path is [`Recovery::Manual`].
    ///
    /// `owner` names the system that must fix this, e.g. `"chunk-manager"`.
    #[must_use]
    pub fn new(domain: Domain, owner: &'static str, message: impl Into<String>) -> Self {
        Self {
            domain,
            severity: Severity::Recoverable,
            recovery: Recovery::Manual,
            owner,
            message: message.into(),
            context: Vec::new(),
            source: None,
        }
    }

    /// Mark this failure as fatal to the runtime.
    #[must_use]
    pub fn fatal(mut self) -> Self {
        self.severity = Severity::Fatal;
        self
    }

    /// Declare how this failure can be recovered from.
    #[must_use]
    pub fn with_recovery(mut self, recovery: Recovery) -> Self {
        self.recovery = recovery;
        self
    }

    /// Attach one structured context pair.
    #[must_use]
    pub fn with_context(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.context.push((key, value.into()));
        self
    }

    /// Wrap a lower-level cause, preserving the original diagnosis.
    #[must_use]
    pub fn with_source(mut self, source: Error) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// The area that owns this failure.
    #[must_use]
    pub const fn domain(&self) -> Domain {
        self.domain
    }

    /// Whether the runtime may continue.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// The declared recovery path.
    #[must_use]
    pub const fn recovery(&self) -> Recovery {
        self.recovery
    }

    /// The specific system responsible.
    #[must_use]
    pub const fn owner(&self) -> &'static str {
        self.owner
    }

    /// The human-readable summary.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The structured context pairs, in the order they were attached.
    #[must_use]
    pub fn context(&self) -> &[(&'static str, String)] {
        &self.context
    }

    /// The wrapped cause, if any.
    #[must_use]
    pub fn source_error(&self) -> Option<&Error> {
        self.source.as_deref()
    }

    /// Whether continuing the runtime after this failure is defensible.
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(self.severity, Severity::Fatal)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}/{}] {} (recovery: {})",
            self.domain.as_str(),
            self.owner,
            self.message,
            self.recovery.as_str()
        )?;
        for (key, value) in &self.context {
            write!(f, " {key}={value}")?;
        }
        if let Some(source) = &self.source {
            write!(f, " <- {source}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

/// Convenience alias for fallible NEXORA operations.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_reports_owner_and_recovery() {
        let err = Error::new(Domain::Save, "save-container", "checksum mismatch")
            .with_recovery(Recovery::Quarantine)
            .with_context("section", "chunk-data")
            .with_context("expected", "0xdeadbeef");

        assert_eq!(err.domain(), Domain::Save);
        assert_eq!(err.owner(), "save-container");
        assert_eq!(err.recovery(), Recovery::Quarantine);
        assert!(!err.is_fatal());

        let rendered = err.to_string();
        assert!(rendered.contains("save/save-container"), "{rendered}");
        assert!(rendered.contains("recovery: quarantine"), "{rendered}");
        assert!(rendered.contains("section=chunk-data"), "{rendered}");
    }

    #[test]
    fn errors_chain_without_losing_the_root_cause() {
        let root = Error::new(Domain::Chunk, "section-storage", "palette overflow");
        let wrapped = Error::new(Domain::World, "chunk-manager", "chunk load failed")
            .with_source(root.clone())
            .fatal();

        assert!(wrapped.is_fatal());
        assert_eq!(wrapped.source_error(), Some(&root));
        assert!(wrapped.to_string().contains("palette overflow"));
    }

    #[test]
    fn default_severity_is_recoverable() {
        let err = Error::new(Domain::Module, "module-graph", "missing dependency");
        assert_eq!(err.severity(), Severity::Recoverable);
        assert_eq!(err.recovery(), Recovery::Manual);
    }
}
