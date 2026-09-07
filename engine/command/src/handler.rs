//! The one authority that executes a command (§27, §28, §110).
//!
//! §110 is the distinction the whole crate turns on:
//!
//! ```text
//! COMMAND -> ONE AUTHORITY / HANDLER
//! EVENT   -> MANY SUBSCRIBERS
//! ```
//!
//! So a handler is registered *singly*, and registering a second one for the
//! same command is an error rather than a second subscriber.
//!
//! §28 draws the other line: a handler **adapts**, a specialized system
//! **decides**. `BreakBlockHandler` calls `BuildAndDestruction::break(...)`; it
//! does not itself know what breaking a block means. That is also why the rules
//! cannot live in this crate at all — §134, enforced by the dependency graph.

use crate::instance::CommandInstance;
use crate::result::FailureReason;

/// What a handler produced.
#[derive(Debug, Clone, Default)]
pub struct Execution {
    /// Names of the events the world should announce (§55).
    pub events: Vec<&'static str>,
}

impl Execution {
    /// An execution that changed state and announces nothing.
    #[must_use]
    pub const fn silent() -> Self {
        Self { events: Vec::new() }
    }

    /// An execution announcing one event.
    #[must_use]
    pub fn announcing(event: &'static str) -> Self {
        Self {
            events: vec![event],
        }
    }
}

/// Executes one kind of command.
///
/// The `&mut self` receiver is deliberate: a handler owns or borrows the
/// authority it adapts to, and dispatch is single-threaded per authority. It
/// keeps the "exactly one authority" rule true at the type level rather than
/// by convention.
pub trait CommandHandler {
    /// Perform the operation.
    ///
    /// Called only after every validation layer has passed, so a handler may
    /// assume the request is well formed and permitted — but not that the
    /// world still agrees, which is why it can still fail.
    ///
    /// # Errors
    ///
    /// Returns the reason the operation could not be completed.
    fn execute(&mut self, instance: &CommandInstance) -> Result<Execution, FailureReason>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::diagnostics::CorrelationId;
    use nexora_foundation::time::WorldTime;

    use crate::identity::{Actor, CommandId, CommandInstanceId, Source};
    use crate::instance::{CommandContext, Target};

    struct Counter {
        calls: u32,
    }

    impl CommandHandler for Counter {
        fn execute(&mut self, _: &CommandInstance) -> Result<Execution, FailureReason> {
            self.calls += 1;
            Ok(Execution::announcing("nexora:counted"))
        }
    }

    fn instance() -> CommandInstance {
        CommandInstance::new(
            CommandId::nexora("count").expect("valid"),
            CommandInstanceId(1),
            Target::None,
            Vec::new(),
            CommandContext::new(
                Actor::player(1),
                Source::Network,
                WorldTime(1),
                CorrelationId(1),
            ),
        )
    }

    #[test]
    fn a_handler_reports_the_events_it_produced() {
        let mut handler = Counter { calls: 0 };
        let execution = handler.execute(&instance()).expect("succeeds");
        assert_eq!(execution.events, vec!["nexora:counted"]);
        assert_eq!(handler.calls, 1);
    }

    #[test]
    fn a_handler_may_refuse_with_a_structured_reason() {
        struct Refuses;
        impl CommandHandler for Refuses {
            fn execute(&mut self, _: &CommandInstance) -> Result<Execution, FailureReason> {
                Err(FailureReason::InvalidState)
            }
        }
        assert_eq!(
            Refuses.execute(&instance()).expect_err("refuses"),
            FailureReason::InvalidState
        );
    }
}
