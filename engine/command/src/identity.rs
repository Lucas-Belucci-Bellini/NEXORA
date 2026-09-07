//! Who asked, where it came in from, and which operation it is.
//!
//! `Command System.md` §12 insists Actor and Source are different questions:
//! the actor may be a *player* while the technical origin is the *network*, or
//! the actor a *script* while the origin is the *mod runtime*. Collapsing them
//! loses exactly the distinction a diagnosis needs.

use nexora_foundation::error::{Domain, Error, Result};
use nexora_foundation::ident::Identifier;

/// Which operation this is: `namespace:path`, per §9.
///
/// A newtype over [`Identifier`] rather than a bare one, so a command id cannot
/// be passed where a block id is expected. Mods register into their own
/// namespace without touching the core (§74).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandId(Identifier);

impl CommandId {
    /// Wrap an identifier as a command id.
    #[must_use]
    pub const fn new(id: Identifier) -> Self {
        Self(id)
    }

    /// Parse `namespace:path`.
    ///
    /// # Errors
    ///
    /// Returns an error when the text is not a valid identifier.
    pub fn parse(raw: &str) -> Result<Self> {
        Identifier::parse(raw).map(Self)
    }

    /// A command id in the engine's own namespace.
    ///
    /// # Errors
    ///
    /// Returns an error when `path` is not a valid identifier path.
    pub fn nexora(path: &str) -> Result<Self> {
        Identifier::nexora(path).map(Self)
    }

    /// The underlying identifier.
    #[must_use]
    pub const fn identifier(&self) -> &Identifier {
        &self.0
    }
}

impl core::fmt::Display for CommandId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// One execution of a command, unique for the life of the session (§10).
///
/// This is what makes deduplication, tracing, audit and idempotency possible,
/// so it is minted in exactly one place — [`InstanceIdSource`] — rather than by
/// whoever happens to build the instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandInstanceId(pub u64);

/// Mints [`CommandInstanceId`]s.
///
/// Monotonic and never reused: a repeated id would make two different
/// operations look like one retry of the same operation, which is precisely the
/// mistake the id exists to prevent.
#[derive(Debug)]
pub struct InstanceIdSource {
    next: u64,
}

impl InstanceIdSource {
    /// A source starting from the first valid id.
    #[must_use]
    pub const fn new() -> Self {
        // Starts at 1 so that zero is available as "no instance" in traces and
        // never collides with a real one.
        Self { next: 1 }
    }

    /// Mint the next id.
    ///
    /// # Errors
    ///
    /// Returns an error when the counter is exhausted. At one command per
    /// nanosecond this takes 584 years, but exhaustion silently wrapping to an
    /// id already used would break deduplication, so it fails loudly instead.
    ///
    /// The counter advances before the id is returned, so `u64::MAX` is never
    /// issued — the last usable id is `u64::MAX - 1`. Handing out that final id
    /// would need a separate "exhausted" flag, and one wasted id out of 2^64 is
    /// a better trade than extra state on the path that must never be wrong.
    pub fn mint(&mut self) -> Result<CommandInstanceId> {
        let id = self.next;
        self.next = self.next.checked_add(1).ok_or_else(|| {
            Error::new(
                Domain::Command,
                "command-instance-id",
                "command instance ids are exhausted; the counter cannot wrap \
                 without breaking deduplication",
            )
            .fatal()
        })?;
        Ok(CommandInstanceId(id))
    }
}

impl Default for InstanceIdSource {
    fn default() -> Self {
        Self::new()
    }
}

/// Who requested the operation (§11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ActorKind {
    /// A human player.
    Player,
    /// A non-player character acting on its own behalf.
    Npc,
    /// An AI decision system.
    Ai,
    /// A script, run by the mod runtime.
    Script,
    /// An automated in-world machine.
    Machine,
    /// A world event, such as weather or a disaster.
    WorldEvent,
    /// An operator with elevated permissions.
    Admin,
    /// The server itself.
    ServerSystem,
    /// A mod, acting as itself rather than through a script.
    Mod,
}

/// The technical origin of the request (§12).
///
/// Deliberately separate from [`ActorKind`]: a player acting over the network
/// and a player acting from a local session are the same actor arriving through
/// different doors, and only one of those doors is remote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Source {
    /// Arrived over the network from a connected client.
    Network,
    /// Originated inside this process, from the server's own systems.
    Local,
    /// Came from the mod runtime.
    ModRuntime,
    /// Typed at an operator console.
    Console,
    /// Replayed from a recorded log.
    Replay,
}

impl Source {
    /// Whether input from this origin is controlled by someone other than the
    /// server.
    ///
    /// §72 is explicit that a client command is `UNTRUSTED` while a server one
    /// is only `TRUSTED-ish`, and that *"nunca assumir internal = always
    /// valid"*. So this answers "who controls this input", and **never**
    /// "may validation be skipped" — nothing in this crate branches on it to
    /// skip a check. It exists for quotas and for diagnostics.
    #[must_use]
    pub const fn is_externally_controlled(self) -> bool {
        match self {
            Self::Network | Self::ModRuntime | Self::Console => true,
            Self::Local | Self::Replay => false,
        }
    }
}

/// Who is asking, in enough detail to authorize them (§11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    /// What kind of thing this is.
    pub kind: ActorKind,
    /// Stable account identity, when there is one.
    pub account: Option<u64>,
    /// The player this actor is, or acts for.
    pub player: Option<u64>,
    /// The entity this actor drives, when embodied.
    pub entity: Option<u64>,
}

impl Actor {
    /// An actor of `kind` with no identity attached.
    #[must_use]
    pub const fn of(kind: ActorKind) -> Self {
        Self {
            kind,
            account: None,
            player: None,
            entity: None,
        }
    }

    /// A player actor.
    #[must_use]
    pub const fn player(player: u64) -> Self {
        Self {
            kind: ActorKind::Player,
            account: None,
            player: Some(player),
            entity: None,
        }
    }

    /// The server acting on its own behalf.
    #[must_use]
    pub const fn server() -> Self {
        Self::of(ActorKind::ServerSystem)
    }

    /// A key that groups this actor's commands for quota accounting.
    ///
    /// Two requests from the same player share a bucket even when they arrive
    /// on different connections; two anonymous actors of the same kind share
    /// one, which is the conservative direction — it can throttle unrelated
    /// callers, but it cannot let a flood through by splitting it.
    #[must_use]
    pub const fn quota_key(&self) -> QuotaKey {
        match (self.player, self.account, self.entity) {
            (Some(player), _, _) => QuotaKey::Player(player),
            (None, Some(account), _) => QuotaKey::Account(account),
            (None, None, Some(entity)) => QuotaKey::Entity(entity),
            (None, None, None) => QuotaKey::Kind(self.kind),
        }
    }
}

/// The bucket a command counts against for rate limiting (§62).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuotaKey {
    /// Per player.
    Player(u64),
    /// Per account, when no player is attached.
    Account(u64),
    /// Per entity, when neither is.
    Entity(u64),
    /// Per actor kind, for actors with no identity at all.
    Kind(ActorKind),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_id_round_trips_through_its_text() {
        let id = CommandId::parse("nexora:break_block").expect("valid");
        assert_eq!(id.to_string(), "nexora:break_block");
        assert_eq!(id, CommandId::nexora("break_block").expect("valid"));
    }

    #[test]
    fn a_mod_can_claim_its_own_namespace() {
        let id = CommandId::parse("example:activate_machine").expect("valid");
        assert_eq!(id.identifier().namespace().as_str(), "example");
    }

    #[test]
    fn instance_ids_are_unique_and_never_zero() {
        let mut source = InstanceIdSource::new();
        let first = source.mint().expect("fresh source");
        let second = source.mint().expect("fresh source");
        assert_ne!(first, second);
        assert_ne!(first.0, 0);
    }

    #[test]
    fn an_exhausted_source_fails_rather_than_reusing_an_id() {
        // A wrapped counter would make a new operation look like a retry of an
        // old one, and deduplication would then drop it.
        let mut source = InstanceIdSource { next: u64::MAX - 1 };
        assert_eq!(
            source.mint().expect("last usable id"),
            CommandInstanceId(u64::MAX - 1)
        );
        // `u64::MAX` itself is never issued: the counter advances first, so
        // exhaustion is detected one id early. Documented on `mint`.
        assert!(source.mint().is_err());
    }

    #[test]
    fn actor_and_source_are_independent() {
        // §12's example: the actor is a player, the origin is the network.
        let actor = Actor::player(17);
        assert_eq!(actor.kind, ActorKind::Player);
        assert!(Source::Network.is_externally_controlled());
        // And the same player arriving locally is the same actor.
        assert!(!Source::Local.is_externally_controlled());
    }

    #[test]
    fn quota_keys_prefer_the_most_specific_identity() {
        assert_eq!(Actor::player(7).quota_key(), QuotaKey::Player(7));

        let account_only = Actor {
            kind: ActorKind::Player,
            account: Some(3),
            player: None,
            entity: None,
        };
        assert_eq!(account_only.quota_key(), QuotaKey::Account(3));

        assert_eq!(
            Actor::server().quota_key(),
            QuotaKey::Kind(ActorKind::ServerSystem)
        );
    }
}
