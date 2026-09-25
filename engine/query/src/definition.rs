//! The registered description of one query.

use nexora_command::definition::SourcePolicy;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::QueryVersion;

/// The result budget a definition gets unless it asks for another.
///
/// Small on purpose. A query that needs more is a query whose author should
/// have to say so, in the definition, where a reviewer sees it.
pub const DEFAULT_MAX_RESULTS: usize = 256;

/// The contract for one query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryDefinition {
    /// `namespace:path`.
    pub id: Identifier,
    /// The shape of its input and answer.
    pub version: QueryVersion,
    /// The system that owns the data it reads (`NEXORA DATA OWNERSHIP AND
    /// SOURCE OF TRUTH.md`: every authoritative datum has exactly one owner).
    /// Recorded so that "who answers this?" has an answer in the registry,
    /// not only in the handler's source.
    pub owner: Identifier,
    /// Who may ask, through which door. Closed until opened.
    pub access: SourcePolicy,
    /// The most results one answer may carry.
    pub max_results: usize,
}

impl QueryDefinition {
    /// A definition nobody may ask yet, with the default budget.
    ///
    /// # Errors
    ///
    /// Returns an error when the id or the owner is not a valid identifier.
    pub fn new(id: &str, version: QueryVersion, owner: &str) -> Result<Self> {
        Ok(Self {
            id: Identifier::parse(id)?,
            version,
            owner: Identifier::parse(owner)?,
            access: SourcePolicy::closed(),
            max_results: DEFAULT_MAX_RESULTS,
        })
    }

    /// Replace the access policy.
    #[must_use]
    pub fn with_access(mut self, access: SourcePolicy) -> Self {
        self.access = access;
        self
    }

    /// Replace the result budget.
    #[must_use]
    pub const fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = max_results;
        self
    }

    /// Fail if this definition could never answer anyone.
    ///
    /// # Errors
    ///
    /// Returns an error when the policy admits nobody or the budget is zero.
    /// Both are definitions that register cleanly and then refuse every
    /// request, which is a bug best found at startup.
    pub fn validate(&self) -> Result<()> {
        if !self.access.admits_anyone() {
            return Err(
                unusable("the query admits no actor or no source and can never be asked")
                    .with_context("query", self.id.to_string()),
            );
        }
        if self.max_results == 0 {
            return Err(unusable("the query has a result budget of zero")
                .with_context("query", self.id.to_string()));
        }
        Ok(())
    }
}

fn unusable(message: &'static str) -> Error {
    Error::new(Domain::Query, "query-definition", message).with_recovery(Recovery::Reject)
}
