//! The registered set of command definitions (§30).
//!
//! Built on the runtime's [`Registry`], not a private map, so commands get the
//! same registration, freezing, runtime-id assignment and fingerprinting as
//! every other engine definition — and so a mod can register
//! `example:activate_machine` without the core changing (§74).

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_runtime::registry::{Registry, RegistryEntry, RuntimeId};

use crate::definition::CommandDefinition;
use crate::identity::CommandId;

/// Every command the engine knows how to accept.
#[derive(Debug)]
pub struct CommandRegistry {
    inner: Registry<CommandDefinition>,
}

impl CommandRegistry {
    /// An empty registry.
    ///
    /// # Errors
    ///
    /// Returns an error when the registry's own name is invalid, which can only
    /// happen if the constant below is edited wrongly.
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Registry::new(Identifier::nexora("command")?),
        })
    }

    /// Register a definition.
    ///
    /// # Errors
    ///
    /// Returns an error when the definition could never accept anything
    /// (see [`CommandDefinition::validate`]), when the id is already taken, or
    /// when the registry is frozen.
    pub fn register(&mut self, definition: CommandDefinition) -> Result<RuntimeId> {
        // Validate before inserting: a definition nobody can send is a mistake,
        // and it should be loud here rather than silent at the first rejection.
        definition.validate()?;
        let id = definition.id.identifier().clone();
        self.inner.register(id, definition)
    }

    /// Stop accepting registrations.
    ///
    /// Called once module loading is done, so a late registration cannot
    /// change the id assignment a save or a peer already agreed on.
    pub fn freeze(&mut self) {
        self.inner.freeze();
    }

    /// Look up a definition.
    #[must_use]
    pub fn get(&self, id: &CommandId) -> Option<&CommandDefinition> {
        self.inner.get(id.identifier()).map(RegistryEntry::value)
    }

    /// Look up a definition by its runtime id, for the hot path (§29).
    #[must_use]
    pub fn get_by_runtime_id(&self, runtime_id: RuntimeId) -> Option<&CommandDefinition> {
        self.inner
            .get_by_runtime_id(runtime_id)
            .map(RegistryEntry::value)
    }

    /// The runtime id assigned to a command.
    #[must_use]
    pub fn runtime_id_of(&self, id: &CommandId) -> Option<RuntimeId> {
        self.inner.runtime_id_of(id.identifier())
    }

    /// Look up a definition, failing when it is absent.
    ///
    /// # Errors
    ///
    /// Returns an error naming the command when nothing is registered under it.
    pub fn require(&self, id: &CommandId) -> Result<&CommandDefinition> {
        self.get(id).ok_or_else(|| {
            Error::new(
                Domain::Command,
                "command-registry",
                format!("no command is registered as `{id}`"),
            )
            .with_recovery(Recovery::Reject)
        })
    }

    /// How many commands are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether nothing is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// A fingerprint of the registered set, for agreeing with a peer or a save.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        self.inner.fingerprint()
    }

    /// Every registered command id.
    #[must_use]
    pub fn ids(&self) -> Vec<Identifier> {
        self.inner.identifiers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::version::CommandVersion;

    use crate::definition::SourcePolicy;
    use crate::identity::{ActorKind, Source};

    fn usable(id: &str) -> CommandDefinition {
        CommandDefinition::new(id, CommandVersion(1))
            .expect("valid id")
            .with_source(
                SourcePolicy::closed()
                    .allow_actor(ActorKind::Player)
                    .allow_source(Source::Network),
            )
    }

    #[test]
    fn a_registered_command_can_be_found_both_ways() {
        let mut registry = CommandRegistry::new().expect("valid registry");
        let runtime_id = registry
            .register(usable("nexora:break_block"))
            .expect("registers");
        let id = CommandId::nexora("break_block").expect("valid");

        assert_eq!(registry.get(&id).map(|d| d.id.clone()), Some(id.clone()));
        assert_eq!(registry.runtime_id_of(&id), Some(runtime_id));
        assert!(registry.get_by_runtime_id(runtime_id).is_some());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn an_unusable_definition_is_refused_at_registration() {
        // Loud here, rather than silent at the first rejection.
        let mut registry = CommandRegistry::new().expect("valid registry");
        let closed =
            CommandDefinition::new("nexora:break_block", CommandVersion(1)).expect("valid id");
        assert!(registry.register(closed).is_err());
        assert!(registry.is_empty());
    }

    #[test]
    fn a_mod_registers_into_its_own_namespace() {
        let mut registry = CommandRegistry::new().expect("valid registry");
        registry
            .register(usable("example:activate_machine"))
            .expect("registers");
        let id = CommandId::parse("example:activate_machine").expect("valid");
        assert!(registry.get(&id).is_some());
    }

    #[test]
    fn requiring_an_unregistered_command_names_it() {
        let registry = CommandRegistry::new().expect("valid registry");
        let id = CommandId::nexora("break_block").expect("valid");
        let error = registry.require(&id).expect_err("absent");
        assert!(error.to_string().contains("nexora:break_block"));
    }

    #[test]
    fn a_frozen_registry_refuses_late_registration() {
        let mut registry = CommandRegistry::new().expect("valid registry");
        registry
            .register(usable("nexora:break_block"))
            .expect("registers");
        registry.freeze();
        assert!(registry.register(usable("nexora:place_block")).is_err());
    }

    #[test]
    fn the_fingerprint_distinguishes_different_registered_sets() {
        let mut one = CommandRegistry::new().expect("valid registry");
        one.register(usable("nexora:break_block"))
            .expect("registers");

        let mut two = CommandRegistry::new().expect("valid registry");
        two.register(usable("nexora:break_block"))
            .expect("registers");
        two.register(usable("nexora:place_block"))
            .expect("registers");

        assert_ne!(one.fingerprint(), two.fingerprint());
    }
}
