//! What a command *ended as* (§50–§54).
//!
//! §52 is explicit that the answer must not be a bare `false`. A structured
//! reason is what lets the UI say something useful, the log say something
//! searchable, and a script branch on something stable.
//!
//! §56 draws the other line: a result answers *"how did the operation I asked
//! for end?"*, while an event answers *"what happened in the world?"*. They are
//! not the same statement and can legitimately differ — a command may succeed
//! having changed nothing, and one command may produce several events.

use crate::identity::{CommandId, CommandInstanceId};
use crate::lifecycle::Phase;

/// How a command ended (§51).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandStatus {
    /// Executed and committed.
    Success,
    /// Refused before validation: malformed, unknown, or over quota.
    Rejected,
    /// The actor was not allowed to do this.
    Denied,
    /// Structurally wrong.
    Invalid,
    /// Accepted and attempted, but the handler could not complete it.
    Failed,
    /// Abandoned before execution (§41).
    Cancelled,
    /// Arrived, or waited, too long to still be meaningful (§34).
    Expired,
}

impl CommandStatus {
    /// Whether the world may have changed.
    ///
    /// Only [`Success`] may have changed anything: every other status is
    /// returned before a handler commits, and [`Failed`] means the handler
    /// reported it could not complete.
    ///
    /// [`Success`]: CommandStatus::Success
    /// [`Failed`]: CommandStatus::Failed
    #[must_use]
    pub const fn changed_state(self) -> bool {
        matches!(self, Self::Success)
    }

    /// Stable lowercase name for logs and metrics (§92).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Rejected => "rejected",
            Self::Denied => "denied",
            Self::Invalid => "invalid",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        }
    }
}

/// Why a command did not succeed (§52).
///
/// A closed, stable set (§53) rather than a free-text string: UI, logs, scripts
/// and tests all branch on these, and text that can be reworded is text nothing
/// can depend on. `#[non_exhaustive]` so adding a reason is not a break.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FailureReason {
    /// No definition is registered under this id.
    UnknownCommand,
    /// A parameter or target was missing, malformed or out of range.
    MalformedRequest,
    /// The actor is not who the request claims (§20).
    IdentityMismatch,
    /// This actor kind or origin may not send this command (§7 source policy).
    SourceNotPermitted,
    /// The actor lacks permission (§21).
    NoPermission,
    /// This process is not authoritative for the command (§43).
    NotAuthoritative,
    /// The actor exceeded its quota (§62).
    RateLimited,
    /// The target does not exist, or is not where the request says (§23).
    TargetNotFound,
    /// The target is out of the actor's reach (§23).
    TargetOutOfRange,
    /// The region holding the target is not resident (§23).
    WorldNotLoaded,
    /// The world is in a state that forbids this (§24).
    InvalidState,
    /// A required resource was missing (§25).
    MissingResource,
    /// No handler is registered for a command that is otherwise valid.
    NoHandler,
    /// The handler ran and reported failure.
    HandlerFailed,
    /// Abandoned before execution.
    Cancelled,
    /// Too old to act on.
    Expired,
    /// Already applied; this is a duplicate (§35).
    AlreadyApplied,
}

impl FailureReason {
    /// Stable lowercase code (§53).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnknownCommand => "unknown_command",
            Self::MalformedRequest => "malformed_request",
            Self::IdentityMismatch => "identity_mismatch",
            Self::SourceNotPermitted => "source_not_permitted",
            Self::NoPermission => "no_permission",
            Self::NotAuthoritative => "not_authoritative",
            Self::RateLimited => "rate_limited",
            Self::TargetNotFound => "target_not_found",
            Self::TargetOutOfRange => "target_out_of_range",
            Self::WorldNotLoaded => "world_not_loaded",
            Self::InvalidState => "invalid_state",
            Self::MissingResource => "missing_resource",
            Self::NoHandler => "no_handler",
            Self::HandlerFailed => "handler_failed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
            Self::AlreadyApplied => "already_applied",
        }
    }

    /// The status this reason implies.
    ///
    /// Keeping the mapping here stops two call sites reporting the same reason
    /// under different statuses, which would make the metrics in §92 lie.
    #[must_use]
    pub const fn status(self) -> CommandStatus {
        match self {
            Self::MalformedRequest => CommandStatus::Invalid,
            Self::IdentityMismatch
            | Self::SourceNotPermitted
            | Self::NoPermission
            | Self::NotAuthoritative => CommandStatus::Denied,
            Self::UnknownCommand
            | Self::RateLimited
            | Self::TargetNotFound
            | Self::TargetOutOfRange
            | Self::WorldNotLoaded
            | Self::InvalidState
            | Self::MissingResource
            | Self::NoHandler
            | Self::AlreadyApplied => CommandStatus::Rejected,
            Self::HandlerFailed => CommandStatus::Failed,
            Self::Cancelled => CommandStatus::Cancelled,
            Self::Expired => CommandStatus::Expired,
        }
    }
}

/// The structured outcome of one command (§50).
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// How it ended.
    pub status: CommandStatus,
    /// Which command it was.
    pub id: CommandId,
    /// Which execution it was.
    pub instance: CommandInstanceId,
    /// Why, when it did not succeed.
    pub reason: Option<FailureReason>,
    /// How far it got before ending. Useful in a trace (§91): "denied at
    /// authorization" and "denied at target validation" are different bugs.
    pub reached: Phase,
    /// Identifiers of the events the handler emitted (§55).
    ///
    /// The ids, not the events: the result reports *that* the world announced
    /// something, and the event bus carries *what* (§56).
    pub events: Vec<&'static str>,
}

impl CommandResult {
    /// A successful outcome.
    #[must_use]
    pub const fn success(id: CommandId, instance: CommandInstanceId) -> Self {
        Self {
            status: CommandStatus::Success,
            id,
            instance,
            reason: None,
            reached: Phase::Completed,
            events: Vec::new(),
        }
    }

    /// A failure, with its status derived from the reason.
    #[must_use]
    pub const fn refused(
        id: CommandId,
        instance: CommandInstanceId,
        reason: FailureReason,
        reached: Phase,
    ) -> Self {
        Self {
            status: reason.status(),
            id,
            instance,
            reason: Some(reason),
            reached,
            events: Vec::new(),
        }
    }

    /// Note an event the handler emitted.
    #[must_use]
    pub fn with_event(mut self, event: &'static str) -> Self {
        self.events.push(event);
        self
    }

    /// Whether this succeeded.
    #[must_use]
    pub const fn succeeded(&self) -> bool {
        matches!(self.status, CommandStatus::Success)
    }

    /// The localization key a UI can render (§54).
    ///
    /// Shaped `command.<namespace>.<path>.<reason>`, so the engine never has to
    /// know a language. Returns `None` on success, which has nothing to explain.
    #[must_use]
    pub fn localized_reason_key(&self) -> Option<String> {
        self.reason.map(|reason| {
            format!(
                "command.{}.{}.{}",
                self.id.identifier().namespace().as_str(),
                self.id.identifier().path(),
                reason.code()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> CommandId {
        CommandId::nexora("break_block").expect("valid")
    }

    #[test]
    fn only_success_claims_the_world_changed() {
        assert!(CommandStatus::Success.changed_state());
        for status in [
            CommandStatus::Rejected,
            CommandStatus::Denied,
            CommandStatus::Invalid,
            CommandStatus::Failed,
            CommandStatus::Cancelled,
            CommandStatus::Expired,
        ] {
            assert!(
                !status.changed_state(),
                "{status:?} must not claim a change"
            );
        }
    }

    #[test]
    fn a_reason_maps_to_exactly_one_status() {
        assert_eq!(FailureReason::NoPermission.status(), CommandStatus::Denied);
        assert_eq!(
            FailureReason::MalformedRequest.status(),
            CommandStatus::Invalid
        );
        assert_eq!(FailureReason::Expired.status(), CommandStatus::Expired);
        assert_eq!(FailureReason::HandlerFailed.status(), CommandStatus::Failed);
    }

    #[test]
    fn a_refusal_carries_how_far_it_got() {
        // "denied at authorization" and "denied at target validation" are
        // different bugs, so the phase is part of the answer.
        let result = CommandResult::refused(
            id(),
            CommandInstanceId(3),
            FailureReason::NoPermission,
            Phase::StructurallyValidated,
        );
        assert_eq!(result.status, CommandStatus::Denied);
        assert_eq!(result.reached, Phase::StructurallyValidated);
        assert!(!result.succeeded());
    }

    #[test]
    fn the_localization_key_is_built_from_stable_parts() {
        let result = CommandResult::refused(
            id(),
            CommandInstanceId(3),
            FailureReason::TargetOutOfRange,
            Phase::Validated,
        );
        assert_eq!(
            result.localized_reason_key().as_deref(),
            Some("command.nexora.break_block.target_out_of_range")
        );
    }

    #[test]
    fn success_has_nothing_to_localize() {
        let result = CommandResult::success(id(), CommandInstanceId(1));
        assert!(result.localized_reason_key().is_none());
        assert!(result.succeeded());
    }

    #[test]
    fn every_failure_code_is_distinct() {
        // The codes are a stable API (§53); two reasons sharing one would make
        // them indistinguishable to anything that branches on the text.
        let reasons = [
            FailureReason::UnknownCommand,
            FailureReason::MalformedRequest,
            FailureReason::IdentityMismatch,
            FailureReason::SourceNotPermitted,
            FailureReason::NoPermission,
            FailureReason::NotAuthoritative,
            FailureReason::RateLimited,
            FailureReason::TargetNotFound,
            FailureReason::TargetOutOfRange,
            FailureReason::WorldNotLoaded,
            FailureReason::InvalidState,
            FailureReason::MissingResource,
            FailureReason::NoHandler,
            FailureReason::HandlerFailed,
            FailureReason::Cancelled,
            FailureReason::Expired,
            FailureReason::AlreadyApplied,
        ];
        let mut codes: Vec<&str> = reasons.iter().map(|reason| reason.code()).collect();
        let total = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), total, "duplicate failure code");
    }
}
