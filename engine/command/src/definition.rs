//! What a command *is*, as opposed to one request to perform it.
//!
//! `Command System.md` §7 separates the definition (the type) from the instance
//! (§8, the real operation). The definition carries the policies — who may send
//! it, who may authorize it, how long it stays valid — so those answers live in
//! one registered place instead of being re-derived at each call site.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::version::CommandVersion;

use crate::identity::{ActorKind, CommandId, Source};

/// Which origins may submit this command (§7 `source`).
///
/// Deny-by-default: an empty policy accepts nothing. A command whose author
/// forgot to say who may send it is unreachable, which is recoverable; one that
/// defaults to "anyone" is a hole that nobody notices until it is used.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourcePolicy {
    allowed_actors: Vec<ActorKind>,
    allowed_sources: Vec<Source>,
}

impl SourcePolicy {
    /// A policy that accepts nothing.
    #[must_use]
    pub const fn closed() -> Self {
        Self {
            allowed_actors: Vec::new(),
            allowed_sources: Vec::new(),
        }
    }

    /// Allow an actor kind.
    #[must_use]
    pub fn allow_actor(mut self, kind: ActorKind) -> Self {
        if !self.allowed_actors.contains(&kind) {
            self.allowed_actors.push(kind);
        }
        self
    }

    /// Allow a technical origin.
    #[must_use]
    pub fn allow_source(mut self, source: Source) -> Self {
        if !self.allowed_sources.contains(&source) {
            self.allowed_sources.push(source);
        }
        self
    }

    /// Whether this actor kind may submit the command.
    #[must_use]
    pub fn permits_actor(&self, kind: ActorKind) -> bool {
        self.allowed_actors.contains(&kind)
    }

    /// Whether this origin may submit the command.
    #[must_use]
    pub fn permits_source(&self, source: Source) -> bool {
        self.allowed_sources.contains(&source)
    }
}

/// Who has to be authoritative for this command to execute (§7 `authority`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum AuthorityPolicy {
    /// The server must be authoritative. The default, and the safe answer:
    /// §43 makes the server the authority and §72 refuses to trust a client.
    #[default]
    ServerRequired,
    /// May execute wherever it is received. Reserved for commands that change
    /// nothing authoritative; nothing in Phase 0 uses it.
    Anywhere,
}

/// How long a request stays meaningful (§34).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExpiryPolicy {
    /// Ticks after submission beyond which the command is [`Expired`].
    ///
    /// `None` means it never expires. An `AttackCommand` arriving seconds late
    /// is §34's own example of why some commands must.
    ///
    /// [`Expired`]: crate::result::CommandStatus::Expired
    pub max_age_ticks: Option<u64>,
}

/// How many of these one actor may submit (§62).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RatePolicy {
    /// Maximum accepted per tick, per quota bucket.
    ///
    /// `None` means unlimited, which is only correct for commands the server
    /// issues to itself. §63's example is 100,000 `BreakBlockCommand`s: the
    /// queue must not be the thing that discovers the problem.
    pub max_per_tick: Option<u32>,
}

impl Default for RatePolicy {
    fn default() -> Self {
        // Deliberately finite. A definition that never thought about flooding
        // gets a limit that is generous for a human and ruinous for a script.
        Self {
            max_per_tick: Some(64),
        }
    }
}

/// The registered description of one command type (§7).
#[derive(Debug, Clone)]
pub struct CommandDefinition {
    /// `namespace:path`.
    pub id: CommandId,
    /// Schema version, so a stored or replayed instance can be read back (§78).
    pub version: CommandVersion,
    /// Who may submit it.
    pub source: SourcePolicy,
    /// Who must be authoritative.
    pub authority: AuthorityPolicy,
    /// When it stops being meaningful.
    pub expiry: ExpiryPolicy,
    /// How often one actor may send it.
    pub rate: RatePolicy,
}

impl CommandDefinition {
    /// A definition with deny-by-default policies.
    ///
    /// # Errors
    ///
    /// Returns an error when the id is not a valid identifier.
    pub fn new(id: &str, version: CommandVersion) -> Result<Self> {
        Ok(Self {
            id: CommandId::parse(id)?,
            version,
            source: SourcePolicy::closed(),
            authority: AuthorityPolicy::ServerRequired,
            expiry: ExpiryPolicy::default(),
            rate: RatePolicy::default(),
        })
    }

    /// Replace the source policy.
    #[must_use]
    pub fn with_source(mut self, source: SourcePolicy) -> Self {
        self.source = source;
        self
    }

    /// Replace the expiry policy.
    #[must_use]
    pub const fn with_expiry(mut self, expiry: ExpiryPolicy) -> Self {
        self.expiry = expiry;
        self
    }

    /// Replace the rate policy.
    #[must_use]
    pub const fn with_rate(mut self, rate: RatePolicy) -> Self {
        self.rate = rate;
        self
    }

    /// Fail if this definition could never accept anything.
    ///
    /// # Errors
    ///
    /// Returns an error when no actor kind or no source is allowed. Startup
    /// brief §45: a command nobody can ever send is a mistake, and it should
    /// be loud at registration rather than silent at the first rejection.
    pub fn validate(&self) -> Result<()> {
        if self.source.allowed_actors.is_empty() {
            return Err(refuse(format!(
                "command `{}` allows no actor kind and can never be sent",
                self.id
            )));
        }
        if self.source.allowed_sources.is_empty() {
            return Err(refuse(format!(
                "command `{}` allows no source and can never be sent",
                self.id
            )));
        }
        Ok(())
    }
}

/// A definition that can never be used is a caller mistake, not a fatal one.
fn refuse(message: String) -> Error {
    Error::new(Domain::Command, "command-definition", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version() -> CommandVersion {
        CommandVersion(1)
    }

    #[test]
    fn a_fresh_policy_permits_nothing() {
        let policy = SourcePolicy::closed();
        assert!(!policy.permits_actor(ActorKind::Player));
        assert!(!policy.permits_source(Source::Network));
        assert!(!policy.permits_actor(ActorKind::Admin));
    }

    #[test]
    fn allowing_is_additive_and_idempotent() {
        let policy = SourcePolicy::closed()
            .allow_actor(ActorKind::Player)
            .allow_actor(ActorKind::Player)
            .allow_source(Source::Network);
        assert!(policy.permits_actor(ActorKind::Player));
        assert!(policy.permits_source(Source::Network));
        assert!(!policy.permits_actor(ActorKind::Admin));
        assert_eq!(policy.allowed_actors.len(), 1);
    }

    #[test]
    fn the_default_authority_is_the_server() {
        let definition = CommandDefinition::new("nexora:break_block", version()).expect("valid");
        assert_eq!(definition.authority, AuthorityPolicy::ServerRequired);
    }

    #[test]
    fn the_default_rate_is_finite() {
        // A definition that never considered flooding still gets a ceiling.
        let definition = CommandDefinition::new("nexora:break_block", version()).expect("valid");
        assert!(definition.rate.max_per_tick.is_some());
    }

    #[test]
    fn a_definition_nobody_can_send_is_refused() {
        let definition = CommandDefinition::new("nexora:break_block", version()).expect("valid");
        assert!(definition.validate().is_err(), "no actor allowed");

        let actor_only = definition
            .clone()
            .with_source(SourcePolicy::closed().allow_actor(ActorKind::Player));
        assert!(actor_only.validate().is_err(), "no source allowed");

        let usable = definition.with_source(
            SourcePolicy::closed()
                .allow_actor(ActorKind::Player)
                .allow_source(Source::Network),
        );
        assert!(usable.validate().is_ok());
    }
}
