//! Routes a validated command to its one authority (§29), and runs the pipeline
//! that gets it there (§17).
//!
//! Handler lookup is by [`RuntimeId`], per §29's *"lookup deve usar
//! handles/runtime IDs para o hot path"* — the registry assigns one at
//! registration, so dispatch indexes a vector instead of hashing a string.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::time::WorldTime;
use nexora_runtime::registry::RuntimeId;

use crate::handler::CommandHandler;
use crate::instance::CommandInstance;
use crate::lifecycle::Phase;
use crate::queue::{CommandQueue, DrainedBatch, EnqueueRejection, Queued};
use crate::registry::CommandRegistry;
use crate::result::{CommandResult, CommandStatus, FailureReason};
use crate::validation::{RateLimiter, ValidationPipeline, ValidationRequest};

/// Runs commands: validate, then hand to exactly one authority.
///
/// Owns the handlers because §110 gives a command exactly one authority. A
/// second registration for the same command is an error, not a subscription.
pub struct Dispatcher {
    handlers: Vec<Option<Box<dyn CommandHandler>>>,
    pipeline: ValidationPipeline,
    limiter: RateLimiter,
    authoritative: bool,
    executed: u64,
    refused: u64,
}

impl core::fmt::Debug for Dispatcher {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("Dispatcher")
            .field("handlers", &self.handlers.iter().flatten().count())
            .field("authoritative", &self.authoritative)
            .field("executed", &self.executed)
            .field("refused", &self.refused)
            .finish()
    }
}

impl Dispatcher {
    /// A dispatcher for an authoritative process.
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            pipeline: ValidationPipeline::standard(),
            limiter: RateLimiter::new(),
            authoritative: true,
            executed: 0,
            refused: 0,
        }
    }

    /// Say whether this process is authoritative (§43).
    ///
    /// A non-authoritative dispatcher refuses every command whose definition
    /// requires server authority, which is most of them.
    #[must_use]
    pub const fn authoritative(mut self, authoritative: bool) -> Self {
        self.authoritative = authoritative;
        self
    }

    /// Replace the validation pipeline, e.g. to add domain layers (§22–§26).
    #[must_use]
    pub fn with_pipeline(mut self, pipeline: ValidationPipeline) -> Self {
        self.pipeline = pipeline;
        self
    }

    /// Attach the one authority for a command.
    ///
    /// # Errors
    ///
    /// Returns an error when a handler is already attached for `runtime_id`.
    /// §110: a command has one authority; a second one is a bug, and silently
    /// replacing the first would mean whichever module loaded last decides what
    /// breaking a block means.
    pub fn attach(
        &mut self,
        runtime_id: RuntimeId,
        handler: Box<dyn CommandHandler>,
    ) -> Result<()> {
        let index = runtime_id.0 as usize;
        if self.handlers.len() <= index {
            self.handlers.resize_with(index + 1, || None);
        }
        if self.handlers[index].is_some() {
            return Err(Error::new(
                Domain::Command,
                "command-dispatcher",
                format!(
                    "a handler is already attached for runtime id {}; a command has \
                     exactly one authority",
                    runtime_id.0
                ),
            )
            .with_recovery(Recovery::Reject));
        }
        self.handlers[index] = Some(handler);
        Ok(())
    }

    /// Validate and execute one command immediately.
    ///
    /// This is the whole pipeline for a single request: every universal layer,
    /// then the quota, then the domain layers, then the handler. There is no
    /// path through this function that reaches a handler without passing all
    /// of them — §72.
    pub fn dispatch(
        &mut self,
        registry: &CommandRegistry,
        instance: &CommandInstance,
        now: WorldTime,
    ) -> CommandResult {
        let id = instance.id.clone();
        let key = instance.instance;

        let Some(definition) = registry.get(&id) else {
            self.refused += 1;
            return CommandResult::refused(id, key, FailureReason::UnknownCommand, Phase::Received);
        };

        let request = ValidationRequest {
            instance,
            definition,
            now,
            authoritative: self.authoritative,
        };
        if let Some((_layer, reason)) = self.pipeline.run(&request) {
            self.refused += 1;
            return CommandResult::refused(id, key, reason, Phase::Received);
        }

        // The quota is charged only once the cheap layers have passed, so a
        // flood of malformed requests cannot exhaust a legitimate actor's
        // budget on its behalf.
        if let Some(reason) =
            self.limiter
                .admit(now, instance.context.actor.quota_key(), definition)
        {
            self.refused += 1;
            return CommandResult::refused(id, key, reason, Phase::Authorized);
        }

        let Some(runtime_id) = registry.runtime_id_of(&id) else {
            self.refused += 1;
            return CommandResult::refused(id, key, FailureReason::NoHandler, Phase::Validated);
        };
        let Some(Some(handler)) = self.handlers.get_mut(runtime_id.0 as usize) else {
            self.refused += 1;
            return CommandResult::refused(id, key, FailureReason::NoHandler, Phase::Validated);
        };

        match handler.execute(instance) {
            Ok(execution) => {
                self.executed += 1;
                let mut result = CommandResult::success(id, key);
                result.events = execution.events;
                result
            }
            Err(reason) => {
                self.refused += 1;
                CommandResult::refused(id, key, reason, Phase::Executing)
            }
        }
    }

    /// Queue a command for later execution.
    ///
    /// # Errors
    ///
    /// Returns the queue's own rejection when the command is a duplicate or
    /// the queue is full.
    pub fn submit(
        queue: &mut CommandQueue,
        entry: Queued,
    ) -> core::result::Result<(), EnqueueRejection> {
        queue.enqueue(entry)
    }

    /// Drain the queue and run everything that is ready.
    ///
    /// Returns one result per command attempted, in execution order.
    pub fn run_queued(
        &mut self,
        registry: &CommandRegistry,
        queue: &mut CommandQueue,
        now: WorldTime,
        limit: usize,
    ) -> Vec<CommandResult> {
        let DrainedBatch { ready, .. } = queue.drain_ready(now, limit);
        ready
            .iter()
            .map(|entry| self.dispatch(registry, &entry.instance, now))
            .collect()
    }

    /// How many commands have executed successfully (§92).
    #[must_use]
    pub const fn executed(&self) -> u64 {
        self.executed
    }

    /// How many have been refused, at any layer (§92).
    #[must_use]
    pub const fn refused(&self) -> u64 {
        self.refused
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// A one-line trace of an outcome (§91).
///
/// Deliberately built from the stable codes rather than prose, so a log can be
/// grepped and a metric can be keyed on it.
#[must_use]
pub fn trace_line(result: &CommandResult) -> String {
    match result.status {
        CommandStatus::Success => format!(
            "{} instance={} status=success events={}",
            result.id,
            result.instance.0,
            result.events.len()
        ),
        _ => format!(
            "{} instance={} status={} reason={} reached={}",
            result.id,
            result.instance.0,
            result.status.as_str(),
            result
                .reason
                .map_or("-", crate::result::FailureReason::code),
            result.reached.as_str()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::diagnostics::CorrelationId;
    use nexora_foundation::version::CommandVersion;

    use crate::definition::{AuthorityPolicy, CommandDefinition, RatePolicy, SourcePolicy};
    use crate::handler::Execution;
    use crate::identity::{Actor, ActorKind, CommandId, CommandInstanceId, Source};
    use crate::instance::{CommandContext, Target};

    struct Recorder {
        calls: std::rc::Rc<std::cell::RefCell<u32>>,
    }

    impl CommandHandler for Recorder {
        fn execute(
            &mut self,
            _: &CommandInstance,
        ) -> core::result::Result<Execution, FailureReason> {
            *self.calls.borrow_mut() += 1;
            Ok(Execution::announcing("nexora:block_broken"))
        }
    }

    fn definition() -> CommandDefinition {
        CommandDefinition::new("nexora:break_block", CommandVersion(1))
            .expect("valid")
            .with_source(
                SourcePolicy::closed()
                    .allow_actor(ActorKind::Player)
                    .allow_source(Source::Network),
            )
    }

    fn registry_with(definition: CommandDefinition) -> (CommandRegistry, RuntimeId) {
        let mut registry = CommandRegistry::new().expect("valid");
        let runtime_id = registry.register(definition).expect("registers");
        (registry, runtime_id)
    }

    fn instance(id: u64) -> CommandInstance {
        CommandInstance::new(
            CommandId::nexora("break_block").expect("valid"),
            CommandInstanceId(id),
            Target::Block { x: 1, y: 2, z: 3 },
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
    fn a_valid_command_reaches_its_handler_and_reports_the_event() {
        let (registry, runtime_id) = registry_with(definition());
        let calls = std::rc::Rc::new(std::cell::RefCell::new(0));
        let mut dispatcher = Dispatcher::new();
        dispatcher
            .attach(
                runtime_id,
                Box::new(Recorder {
                    calls: std::rc::Rc::clone(&calls),
                }),
            )
            .expect("first handler");

        let result = dispatcher.dispatch(&registry, &instance(1), WorldTime(1));
        assert!(result.succeeded());
        assert_eq!(result.events, vec!["nexora:block_broken"]);
        assert_eq!(*calls.borrow(), 1);
        assert_eq!(dispatcher.executed(), 1);
    }

    #[test]
    fn a_command_has_exactly_one_authority() {
        // §110. Silently replacing the first handler would mean whichever
        // module loaded last decides what breaking a block means.
        let (_registry, runtime_id) = registry_with(definition());
        let calls = std::rc::Rc::new(std::cell::RefCell::new(0));
        let mut dispatcher = Dispatcher::new();
        dispatcher
            .attach(
                runtime_id,
                Box::new(Recorder {
                    calls: std::rc::Rc::clone(&calls),
                }),
            )
            .expect("first handler");
        assert!(dispatcher
            .attach(runtime_id, Box::new(Recorder { calls }))
            .is_err());
    }

    #[test]
    fn an_unregistered_command_never_reaches_a_handler() {
        let registry = CommandRegistry::new().expect("valid");
        let mut dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(&registry, &instance(1), WorldTime(1));
        assert_eq!(result.reason, Some(FailureReason::UnknownCommand));
        assert_eq!(result.reached, Phase::Received);
    }

    #[test]
    fn a_valid_command_with_no_handler_is_refused_not_dropped() {
        let (registry, _) = registry_with(definition());
        let mut dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(&registry, &instance(1), WorldTime(1));
        assert_eq!(result.reason, Some(FailureReason::NoHandler));
        assert_eq!(result.status, CommandStatus::Rejected);
    }

    #[test]
    fn validation_runs_before_the_handler_does() {
        let (registry, runtime_id) = registry_with(definition());
        let calls = std::rc::Rc::new(std::cell::RefCell::new(0));
        let mut dispatcher = Dispatcher::new();
        dispatcher
            .attach(
                runtime_id,
                Box::new(Recorder {
                    calls: std::rc::Rc::clone(&calls),
                }),
            )
            .expect("handler");

        // Wrong origin for this definition.
        let mut denied = instance(1);
        denied.context.source = Source::Console;
        let result = dispatcher.dispatch(&registry, &denied, WorldTime(1));

        assert_eq!(result.status, CommandStatus::Denied);
        assert_eq!(*calls.borrow(), 0, "the handler must not have run");
    }

    #[test]
    fn a_non_authoritative_dispatcher_refuses_server_required_commands() {
        let (registry, runtime_id) = registry_with(definition());
        let calls = std::rc::Rc::new(std::cell::RefCell::new(0));
        let mut dispatcher = Dispatcher::new().authoritative(false);
        dispatcher
            .attach(
                runtime_id,
                Box::new(Recorder {
                    calls: std::rc::Rc::clone(&calls),
                }),
            )
            .expect("handler");

        let result = dispatcher.dispatch(&registry, &instance(1), WorldTime(1));
        assert_eq!(result.reason, Some(FailureReason::NotAuthoritative));
        assert_eq!(*calls.borrow(), 0);
    }

    #[test]
    fn a_handler_failure_is_reported_as_failed_not_denied() {
        struct Refuses;
        impl CommandHandler for Refuses {
            fn execute(
                &mut self,
                _: &CommandInstance,
            ) -> core::result::Result<Execution, FailureReason> {
                Err(FailureReason::TargetNotFound)
            }
        }

        let (registry, runtime_id) = registry_with(definition());
        let mut dispatcher = Dispatcher::new();
        dispatcher
            .attach(runtime_id, Box::new(Refuses))
            .expect("handler");

        let result = dispatcher.dispatch(&registry, &instance(1), WorldTime(1));
        assert_eq!(result.reason, Some(FailureReason::TargetNotFound));
        assert_eq!(result.reached, Phase::Executing);
    }

    #[test]
    fn the_quota_stops_a_flood_before_the_handler() {
        let (registry, runtime_id) = registry_with(definition().with_rate(RatePolicy {
            max_per_tick: Some(2),
        }));
        let calls = std::rc::Rc::new(std::cell::RefCell::new(0));
        let mut dispatcher = Dispatcher::new();
        dispatcher
            .attach(
                runtime_id,
                Box::new(Recorder {
                    calls: std::rc::Rc::clone(&calls),
                }),
            )
            .expect("handler");

        for id in 1..=5 {
            dispatcher.dispatch(&registry, &instance(id), WorldTime(1));
        }
        assert_eq!(
            *calls.borrow(),
            2,
            "only the quota's worth reached the handler"
        );
        assert_eq!(dispatcher.executed(), 2);
        assert_eq!(dispatcher.refused(), 3);
    }

    #[test]
    fn a_malformed_flood_does_not_consume_the_actors_quota() {
        // The quota is charged after the cheap layers, so garbage cannot spend
        // a legitimate actor's budget on their behalf.
        let (registry, runtime_id) = registry_with(definition().with_rate(RatePolicy {
            max_per_tick: Some(1),
        }));
        let calls = std::rc::Rc::new(std::cell::RefCell::new(0));
        let mut dispatcher = Dispatcher::new();
        dispatcher
            .attach(
                runtime_id,
                Box::new(Recorder {
                    calls: std::rc::Rc::clone(&calls),
                }),
            )
            .expect("handler");

        for id in 1..=10 {
            let mut malformed = instance(id);
            malformed.parameters = vec![crate::instance::Parameter::Real(f64::NAN)];
            dispatcher.dispatch(&registry, &malformed, WorldTime(1));
        }
        // The one good command still gets through.
        let result = dispatcher.dispatch(&registry, &instance(99), WorldTime(1));
        assert!(result.succeeded());
        assert_eq!(*calls.borrow(), 1);
    }

    #[test]
    fn queued_commands_run_in_priority_order() {
        use crate::queue::{Priority, Queued};

        let (registry, runtime_id) = registry_with(definition());
        let calls = std::rc::Rc::new(std::cell::RefCell::new(0));
        let mut dispatcher = Dispatcher::new();
        dispatcher
            .attach(
                runtime_id,
                Box::new(Recorder {
                    calls: std::rc::Rc::clone(&calls),
                }),
            )
            .expect("handler");

        let mut queue = CommandQueue::new();
        Dispatcher::submit(&mut queue, Queued::now(instance(1), WorldTime(1))).expect("queued");
        Dispatcher::submit(
            &mut queue,
            Queued::now(instance(2), WorldTime(1)).with_priority(Priority::Critical),
        )
        .expect("queued");

        let results = dispatcher.run_queued(&registry, &mut queue, WorldTime(1), 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].instance, CommandInstanceId(2));
        assert!(results.iter().all(CommandResult::succeeded));
    }

    #[test]
    fn the_trace_line_carries_the_stable_codes() {
        let (registry, _) = registry_with(definition());
        let mut dispatcher = Dispatcher::new();
        let result = dispatcher.dispatch(&registry, &instance(1), WorldTime(1));
        let line = trace_line(&result);
        assert!(line.contains("status=rejected"), "{line}");
        assert!(line.contains("reason=no_handler"), "{line}");
        assert!(line.contains("reached=validated"), "{line}");
    }

    #[test]
    fn the_default_authority_policy_is_what_the_document_says() {
        assert_eq!(definition().authority, AuthorityPolicy::ServerRequired);
    }
}
