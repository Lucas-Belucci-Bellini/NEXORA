//! One real request to perform a command (§8), and the context it carries (§13).

use nexora_foundation::diagnostics::CorrelationId;
use nexora_foundation::time::WorldTime;

use crate::identity::{Actor, CommandId, CommandInstanceId, Source};

/// What an operation is aimed at.
///
/// `#[non_exhaustive]` because §65 adds spatial targets, §103 entity targets
/// and §104 dimension targets later. Adding a variant must not be a breaking
/// change for callers that already match on the ones that exist.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Target {
    /// Aimed at nothing in particular.
    None,
    /// A block position in the current world.
    Block {
        /// World X.
        x: i64,
        /// World Y.
        y: i64,
        /// World Z.
        z: i64,
    },
    /// An entity, by its session handle.
    Entity(u64),
}

/// A parameter carried by a request.
///
/// A closed value type rather than an open `Any`, because §72 treats a command
/// as untrusted input: a fixed set of shapes can be bounds-checked by
/// [`structural`] validation, and an arbitrary payload cannot.
///
/// [`structural`]: crate::validation::StructuralValidator
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Parameter {
    /// A signed integer.
    Integer(i64),
    /// An unsigned count.
    Count(u64),
    /// A real number.
    Real(f64),
    /// A flag.
    Flag(bool),
    /// A short text value, length-bounded at construction.
    Text(String),
    /// A registry identifier, as text.
    Ident(String),
}

/// Longest accepted [`Parameter::Text`] or [`Parameter::Ident`], in bytes.
///
/// §72: a command is untrusted input, so every unbounded field is a memory
/// budget an attacker gets to choose. 256 bytes holds any identifier the engine
/// produces with room to spare.
pub const MAX_PARAMETER_TEXT_BYTES: usize = 256;

/// Everything the pipeline needs to know about the circumstances (§13).
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Who asked.
    pub actor: Actor,
    /// Where it technically came in from.
    pub source: Source,
    /// The tick at which it was received.
    pub tick: WorldTime,
    /// Ties this operation to the wider one it belongs to (§14).
    pub correlation: CorrelationId,
    /// The instance that caused this one, when it was caused by another (§15).
    pub causation: Option<CommandInstanceId>,
    /// Set when something has asked for this command to be abandoned (§41).
    pub cancelled: bool,
}

impl CommandContext {
    /// A context for an actor acting through `source` at `tick`.
    #[must_use]
    pub const fn new(
        actor: Actor,
        source: Source,
        tick: WorldTime,
        correlation: CorrelationId,
    ) -> Self {
        Self {
            actor,
            source,
            tick,
            correlation,
            causation: None,
            cancelled: false,
        }
    }

    /// Record that this command was caused by an earlier one (§15).
    #[must_use]
    pub const fn caused_by(mut self, cause: CommandInstanceId) -> Self {
        self.causation = Some(cause);
        self
    }
}

/// One request to perform a command (§8).
///
/// The instance is *intent*. Nothing here says the world changed — that is
/// what an event says (§2, §56), and only after a handler has executed.
#[derive(Debug, Clone)]
pub struct CommandInstance {
    /// Which command this is.
    pub id: CommandId,
    /// Unique for this execution (§10).
    pub instance: CommandInstanceId,
    /// What it is aimed at.
    pub target: Target,
    /// Its arguments.
    pub parameters: Vec<Parameter>,
    /// The circumstances.
    pub context: CommandContext,
}

impl CommandInstance {
    /// Build a request.
    #[must_use]
    pub const fn new(
        id: CommandId,
        instance: CommandInstanceId,
        target: Target,
        parameters: Vec<Parameter>,
        context: CommandContext,
    ) -> Self {
        Self {
            id,
            instance,
            target,
            parameters,
            context,
        }
    }

    /// Age in ticks at `now`.
    ///
    /// Saturating rather than wrapping: a clock that appears to have gone
    /// backwards yields zero age, which reads as "fresh" and lets expiry
    /// checks pass, instead of yielding a huge age that would expire every
    /// command in flight.
    #[must_use]
    pub const fn age_ticks(&self, now: WorldTime) -> u64 {
        now.0.saturating_sub(self.context.tick.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ActorKind;

    fn context() -> CommandContext {
        CommandContext::new(
            Actor::player(1),
            Source::Network,
            WorldTime(100),
            CorrelationId(7),
        )
    }

    #[test]
    fn an_instance_carries_intent_not_outcome() {
        let instance = CommandInstance::new(
            CommandId::nexora("break_block").expect("valid"),
            CommandInstanceId(1),
            Target::Block { x: 4, y: 70, z: -3 },
            vec![Parameter::Flag(true)],
            context(),
        );
        assert_eq!(instance.target, Target::Block { x: 4, y: 70, z: -3 });
        assert_eq!(instance.context.actor.kind, ActorKind::Player);
    }

    #[test]
    fn age_is_measured_from_the_receiving_tick() {
        let instance = CommandInstance::new(
            CommandId::nexora("attack").expect("valid"),
            CommandInstanceId(1),
            Target::None,
            Vec::new(),
            context(),
        );
        assert_eq!(instance.age_ticks(WorldTime(140)), 40);
    }

    #[test]
    fn a_clock_that_went_backwards_reads_as_fresh_not_ancient() {
        // Saturating the other way would expire everything in flight the moment
        // a clock correction landed.
        let instance = CommandInstance::new(
            CommandId::nexora("attack").expect("valid"),
            CommandInstanceId(1),
            Target::None,
            Vec::new(),
            context(),
        );
        assert_eq!(instance.age_ticks(WorldTime(1)), 0);
    }

    #[test]
    fn causation_is_recorded_when_one_command_begets_another() {
        let derived = context().caused_by(CommandInstanceId(42));
        assert_eq!(derived.causation, Some(CommandInstanceId(42)));
        // And correlation survives, so the whole chain stays traceable (§14).
        assert_eq!(derived.correlation, CorrelationId(7));
    }
}
