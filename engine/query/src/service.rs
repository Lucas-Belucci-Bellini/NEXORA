//! Asking: the one path from a request to an answer.
//!
//! ```text
//! request -> definition? -> version -> actor -> source -> answer(budget)
//!         -> re-count -> answer | refusal
//! ```
//!
//! The order matches the command pipeline's reasoning: nothing reaches a
//! handler — the only step that does real work — until the cheap, universal
//! checks have passed.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use nexora_command::identity::{Actor, Source};
use nexora_foundation::diagnostics::Counters;
use nexora_foundation::error::Result;
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::QueryVersion;
use nexora_runtime::registry::Registry;

use crate::definition::QueryDefinition;

/// How much one answer may carry. Handed to the handler so it can stop early.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    /// The most results the answer may hold.
    pub max_results: usize,
}

/// Answers one kind of question about `S`, from a shared reference.
pub trait Query<S: ?Sized> {
    /// What the caller supplies.
    type Input;
    /// What the caller gets back.
    type Output;

    /// The identifier its definition is registered under.
    fn id(&self) -> &Identifier;

    /// Answer from `state`, within `budget`.
    ///
    /// # Errors
    ///
    /// Returns why no answer exists: the input is invalid, the data is not
    /// available, or answering would exceed the budget.
    fn answer(
        &self,
        state: &S,
        input: &Self::Input,
        budget: Budget,
    ) -> std::result::Result<Self::Output, QueryFailure>;

    /// How many results an answer holds, for the budget check.
    fn result_count(output: &Self::Output) -> usize;
}

/// Why a query was not answered.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryFailure {
    /// No definition is registered under this id.
    UnknownQuery,
    /// The request was built against another version of the contract.
    VersionMismatch {
        /// What the definition is.
        registered: QueryVersion,
        /// What the request named.
        requested: QueryVersion,
    },
    /// This kind of actor may not ask.
    ActorNotPermitted,
    /// Requests arriving through this door may not ask.
    SourceNotPermitted,
    /// The answer would exceed the definition's budget.
    OverBudget {
        /// The budget.
        limit: usize,
    },
    /// The input is well formed but asks for something the query refuses.
    Invalid(String),
    /// The data exists in principle but cannot be read now (not resident, not
    /// loaded). Distinct from "there is nothing there".
    Unavailable(String),
}

impl QueryFailure {
    /// A stable short name, for diagnostics and counters.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::UnknownQuery => "unknown-query",
            Self::VersionMismatch { .. } => "version-mismatch",
            Self::ActorNotPermitted => "actor-not-permitted",
            Self::SourceNotPermitted => "source-not-permitted",
            Self::OverBudget { .. } => "over-budget",
            Self::Invalid(_) => "invalid",
            Self::Unavailable(_) => "unavailable",
        }
    }
}

impl fmt::Display for QueryFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionMismatch {
                registered,
                requested,
            } => write!(
                f,
                "version-mismatch (registered {registered}, requested {requested})"
            ),
            Self::OverBudget { limit } => write!(f, "over-budget (limit {limit})"),
            Self::Invalid(reason) => write!(f, "invalid: {reason}"),
            Self::Unavailable(reason) => write!(f, "unavailable: {reason}"),
            other => f.write_str(other.as_str()),
        }
    }
}

/// Who is asking, through which door, against which version.
#[derive(Debug, Clone)]
pub struct QueryRequest<'a, I> {
    /// Who.
    pub actor: &'a Actor,
    /// Through which door.
    pub source: Source,
    /// The contract version the caller was built against.
    pub version: QueryVersion,
    /// The question.
    pub input: I,
}

/// What the service has done.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueryStats {
    /// Questions answered.
    pub answered: u64,
    /// Questions refused, for any reason.
    pub refused: u64,
    /// Answers a handler produced past its budget. Always a handler bug.
    pub overruns: u64,
}

/// Holds query definitions and answers through them.
#[derive(Debug)]
pub struct QueryService {
    definitions: Registry<QueryDefinition>,
    answered: AtomicU64,
    refused: AtomicU64,
    overruns: AtomicU64,
}

impl QueryService {
    /// An empty service.
    ///
    /// # Errors
    ///
    /// Returns an error only if the registry's own name stops being a valid
    /// identifier, which would be a build-time mistake.
    pub fn new() -> Result<Self> {
        Ok(Self {
            definitions: Registry::new(Identifier::parse("nexora:registry/query")?),
            answered: AtomicU64::new(0),
            refused: AtomicU64::new(0),
            overruns: AtomicU64::new(0),
        })
    }

    /// Register a definition.
    ///
    /// # Errors
    ///
    /// Returns an error when the definition could never answer anyone, when
    /// the id is already registered — a query has one owner, and a second
    /// registration silently winning would mean whichever module loaded last
    /// decides what the world says — or when the service is frozen.
    pub fn register(&mut self, definition: QueryDefinition) -> Result<()> {
        definition.validate()?;
        self.definitions
            .register(definition.id.clone(), definition)?;
        Ok(())
    }

    /// Refuse further registration.
    pub fn freeze(&mut self) {
        self.definitions.freeze();
    }

    /// The definition registered under an id.
    #[must_use]
    pub fn definition(&self, id: &Identifier) -> Option<&QueryDefinition> {
        self.definitions.get(id).map(|entry| entry.value())
    }

    /// Ask a query.
    ///
    /// # Errors
    ///
    /// Returns the reason the question was refused or could not be answered.
    pub fn ask<S: ?Sized, Q: Query<S>>(
        &self,
        query: &Q,
        state: &S,
        request: &QueryRequest<'_, Q::Input>,
    ) -> std::result::Result<Q::Output, QueryFailure> {
        let outcome = self.check(query.id(), request).and_then(|definition| {
            let budget = Budget {
                max_results: definition.max_results,
            };
            let output = query.answer(state, &request.input, budget)?;
            if Q::result_count(&output) > budget.max_results {
                // The handler was told its budget and ignored it. Refuse the
                // answer rather than trust the handler: this is the check that
                // makes the budget a property of the service, not a courtesy.
                self.overruns.fetch_add(1, Ordering::Relaxed);
                return Err(QueryFailure::OverBudget {
                    limit: budget.max_results,
                });
            }
            Ok(output)
        });
        let counter = if outcome.is_ok() {
            &self.answered
        } else {
            &self.refused
        };
        counter.fetch_add(1, Ordering::Relaxed);
        outcome
    }

    fn check<I>(
        &self,
        id: &Identifier,
        request: &QueryRequest<'_, I>,
    ) -> std::result::Result<&QueryDefinition, QueryFailure> {
        let definition = self.definition(id).ok_or(QueryFailure::UnknownQuery)?;
        if definition.version != request.version {
            return Err(QueryFailure::VersionMismatch {
                registered: definition.version,
                requested: request.version,
            });
        }
        if !definition.access.permits_actor(request.actor.kind) {
            return Err(QueryFailure::ActorNotPermitted);
        }
        if !definition.access.permits_source(request.source) {
            return Err(QueryFailure::SourceNotPermitted);
        }
        Ok(definition)
    }

    /// What the service has done so far.
    #[must_use]
    pub fn stats(&self) -> QueryStats {
        QueryStats {
            answered: self.answered.load(Ordering::Relaxed),
            refused: self.refused.load(Ordering::Relaxed),
            overruns: self.overruns.load(Ordering::Relaxed),
        }
    }

    /// Publish the counters under `query.*`, as the difference from `since`.
    pub fn publish(&self, counters: &Counters, since: QueryStats) {
        let now = self.stats();
        counters.add("query.answered", now.answered - since.answered);
        counters.add("query.refused", now.refused - since.refused);
        counters.add("query.overruns", now.overruns - since.overruns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_command::definition::SourcePolicy;
    use nexora_command::identity::ActorKind;

    /// A tiny state and two queries over it.
    struct Ledger {
        entries: Vec<u32>,
    }

    struct Entry {
        id: Identifier,
    }
    impl Query<Ledger> for Entry {
        type Input = usize;
        type Output = u32;
        fn id(&self) -> &Identifier {
            &self.id
        }
        fn answer(
            &self,
            state: &Ledger,
            index: &usize,
            _: Budget,
        ) -> std::result::Result<u32, QueryFailure> {
            state
                .entries
                .get(*index)
                .copied()
                .ok_or_else(|| QueryFailure::Invalid(format!("no entry {index}")))
        }
        fn result_count(_: &u32) -> usize {
            1
        }
    }

    /// Returns everything, and — when `honest` is false — ignores its budget.
    struct All {
        id: Identifier,
        honest: bool,
    }
    impl Query<Ledger> for All {
        type Input = ();
        type Output = Vec<u32>;
        fn id(&self) -> &Identifier {
            &self.id
        }
        fn answer(
            &self,
            state: &Ledger,
            _: &(),
            budget: Budget,
        ) -> std::result::Result<Vec<u32>, QueryFailure> {
            if self.honest && state.entries.len() > budget.max_results {
                return Err(QueryFailure::OverBudget {
                    limit: budget.max_results,
                });
            }
            Ok(state.entries.clone())
        }
        fn result_count(output: &Vec<u32>) -> usize {
            output.len()
        }
    }

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).unwrap()
    }

    fn open() -> SourcePolicy {
        SourcePolicy::closed()
            .allow_actor(ActorKind::Player)
            .allow_actor(ActorKind::ServerSystem)
            .allow_source(Source::Local)
            .allow_source(Source::Network)
    }

    fn service() -> QueryService {
        let mut service = QueryService::new().unwrap();
        service
            .register(
                QueryDefinition::new("test:entry", QueryVersion(1), "test:ledger")
                    .unwrap()
                    .with_access(open()),
            )
            .unwrap();
        service
            .register(
                QueryDefinition::new("test:all", QueryVersion(1), "test:ledger")
                    .unwrap()
                    .with_access(open())
                    .with_max_results(3),
            )
            .unwrap();
        service.freeze();
        service
    }

    fn ask_as<Q: Query<Ledger>>(
        service: &QueryService,
        query: &Q,
        state: &Ledger,
        actor: &Actor,
        source: Source,
        input: Q::Input,
    ) -> std::result::Result<Q::Output, QueryFailure> {
        service.ask(
            query,
            state,
            &QueryRequest {
                actor,
                source,
                version: QueryVersion(1),
                input,
            },
        )
    }

    #[test]
    fn a_permitted_question_is_answered_and_counted() {
        let service = service();
        let ledger = Ledger {
            entries: vec![7, 8],
        };
        let entry = Entry {
            id: id("test:entry"),
        };
        assert_eq!(
            ask_as(
                &service,
                &entry,
                &ledger,
                &Actor::player(1),
                Source::Network,
                1
            ),
            Ok(8)
        );
        assert_eq!(
            ask_as(
                &service,
                &entry,
                &ledger,
                &Actor::server(),
                Source::Local,
                5
            ),
            Err(QueryFailure::Invalid("no entry 5".to_owned()))
        );
        assert_eq!(
            service.stats(),
            QueryStats {
                answered: 1,
                refused: 1,
                overruns: 0
            }
        );
    }

    #[test]
    fn nobody_is_answered_by_default_and_the_doors_are_checked_separately() {
        let service = service();
        let ledger = Ledger { entries: vec![1] };
        let entry = Entry {
            id: id("test:entry"),
        };
        let script = Actor::of(ActorKind::Script);
        assert_eq!(
            ask_as(&service, &entry, &ledger, &script, Source::Local, 0),
            Err(QueryFailure::ActorNotPermitted)
        );
        assert_eq!(
            ask_as(
                &service,
                &entry,
                &ledger,
                &Actor::player(1),
                Source::ModRuntime,
                0
            ),
            Err(QueryFailure::SourceNotPermitted)
        );
        let unknown = Entry {
            id: id("test:absent"),
        };
        assert_eq!(
            ask_as(
                &service,
                &unknown,
                &ledger,
                &Actor::server(),
                Source::Local,
                0
            ),
            Err(QueryFailure::UnknownQuery)
        );

        // A definition that admits nobody is refused at registration.
        let mut fresh = QueryService::new().unwrap();
        let closed = QueryDefinition::new("test:closed", QueryVersion(1), "test:ledger").unwrap();
        let err = fresh.register(closed).expect_err("admits nobody");
        assert!(err.to_string().contains("can never be asked"), "{err}");
        let zero = QueryDefinition::new("test:zero", QueryVersion(1), "test:ledger")
            .unwrap()
            .with_access(open())
            .with_max_results(0);
        assert!(fresh.register(zero).is_err());
    }

    #[test]
    fn a_request_built_against_another_version_is_refused_by_name() {
        let service = service();
        let ledger = Ledger { entries: vec![1] };
        let entry = Entry {
            id: id("test:entry"),
        };
        let answer = service.ask(
            &entry,
            &ledger,
            &QueryRequest {
                actor: &Actor::server(),
                source: Source::Local,
                version: QueryVersion(2),
                input: 0,
            },
        );
        assert_eq!(
            answer,
            Err(QueryFailure::VersionMismatch {
                registered: QueryVersion(1),
                requested: QueryVersion(2)
            })
        );
        assert!(answer.unwrap_err().to_string().contains("query-v2"));
    }

    #[test]
    fn the_budget_is_the_services_not_the_handlers() {
        let service = service();
        let small = Ledger {
            entries: vec![1, 2, 3],
        };
        let large = Ledger {
            entries: vec![1, 2, 3, 4],
        };
        let honest = All {
            id: id("test:all"),
            honest: true,
        };
        let careless = All {
            id: id("test:all"),
            honest: false,
        };
        let server = Actor::server();
        assert_eq!(
            ask_as(&service, &honest, &small, &server, Source::Local, ()),
            Ok(vec![1, 2, 3])
        );
        assert_eq!(
            ask_as(&service, &honest, &large, &server, Source::Local, ()),
            Err(QueryFailure::OverBudget { limit: 3 })
        );
        // The careless handler returns four; the service refuses the answer.
        assert_eq!(
            ask_as(&service, &careless, &large, &server, Source::Local, ()),
            Err(QueryFailure::OverBudget { limit: 3 })
        );
        assert_eq!(service.stats().overruns, 1);
    }

    #[test]
    fn a_second_owner_for_one_query_is_refused() {
        let mut service = QueryService::new().unwrap();
        let definition = QueryDefinition::new("test:entry", QueryVersion(1), "test:ledger")
            .unwrap()
            .with_access(open());
        service.register(definition.clone()).unwrap();
        let mut rival = definition;
        rival.owner = id("test:other");
        assert!(service.register(rival).is_err());
    }

    #[test]
    fn asking_needs_no_exclusive_access_so_threads_can_share_a_service() {
        let service = service();
        let ledger = Ledger {
            entries: (0..64).collect(),
        };
        let entry = Entry {
            id: id("test:entry"),
        };
        std::thread::scope(|scope| {
            for thread in 0..4 {
                let (service, ledger, entry) = (&service, &ledger, &entry);
                scope.spawn(move || {
                    for index in 0..16 {
                        let at = thread * 16 + index;
                        let answer =
                            ask_as(service, entry, ledger, &Actor::server(), Source::Local, at);
                        assert_eq!(answer, Ok(at as u32));
                    }
                });
            }
        });
        assert_eq!(service.stats().answered, 64);

        let counters = Counters::new();
        service.publish(&counters, QueryStats::default());
        assert_eq!(counters.get("query.answered"), 64);
    }
}
