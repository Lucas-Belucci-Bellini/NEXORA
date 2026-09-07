//! The phases a command passes through (§16).

/// Where a command has reached.
///
/// §16 lists the happy path and the failure states separately, and they are
/// modelled separately here: [`Phase`] is a position in the pipeline, while a
/// failure is an *outcome* and lives in [`CommandStatus`]. Mixing them would
/// make "how far did it get" and "how did it end" the same question, and the
/// audit trail needs both.
///
/// [`CommandStatus`]: crate::result::CommandStatus
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// Built, not yet submitted.
    Created,
    /// Submitted and accepted for processing.
    Received,
    /// Shape and bounds check passed (§19).
    StructurallyValidated,
    /// The actor is who they claim and may send this (§20, §21).
    Authorized,
    /// Waiting in the queue (§31).
    Queued,
    /// World, target, state and resource checks passed (§22-26).
    Validated,
    /// A handler is running.
    Executing,
    /// The state change is applied.
    Committed,
    /// Finished, with events emitted.
    Completed,
}

impl Phase {
    /// Stable lowercase name for logs and traces (§91).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Received => "received",
            Self::StructurallyValidated => "structurally-validated",
            Self::Authorized => "authorized",
            Self::Queued => "queued",
            Self::Validated => "validated",
            Self::Executing => "executing",
            Self::Committed => "committed",
            Self::Completed => "completed",
        }
    }

    /// The phase that follows, or `None` at the end.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Created => Some(Self::Received),
            Self::Received => Some(Self::StructurallyValidated),
            Self::StructurallyValidated => Some(Self::Authorized),
            Self::Authorized => Some(Self::Queued),
            Self::Queued => Some(Self::Validated),
            Self::Validated => Some(Self::Executing),
            Self::Executing => Some(Self::Committed),
            Self::Committed => Some(Self::Completed),
            Self::Completed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_phases_run_in_the_documented_order() {
        // §16, in order. Written out rather than generated, so that reordering
        // the enum silently is not enough to make this pass.
        let expected = [
            Phase::Created,
            Phase::Received,
            Phase::StructurallyValidated,
            Phase::Authorized,
            Phase::Queued,
            Phase::Validated,
            Phase::Executing,
            Phase::Committed,
            Phase::Completed,
        ];
        let mut walked = vec![Phase::Created];
        while let Some(next) = walked.last().copied().and_then(Phase::next) {
            walked.push(next);
        }
        assert_eq!(walked, expected);
    }

    #[test]
    fn ordering_matches_progression() {
        assert!(Phase::Created < Phase::Queued);
        assert!(Phase::Queued < Phase::Committed);
        assert_eq!(Phase::Completed.next(), None);
    }

    #[test]
    fn every_phase_has_a_distinct_name() {
        let mut names: Vec<&str> = Vec::new();
        let mut phase = Some(Phase::Created);
        while let Some(current) = phase {
            names.push(current.as_str());
            phase = current.next();
        }
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "duplicate phase name");
    }
}
