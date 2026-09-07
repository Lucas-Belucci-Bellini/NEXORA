//! Validation in layers (§18–§26), in the pipeline's own order (§17).
//!
//! §18 opens by rejecting the obvious design: *"Não queremos uma única função
//! `validateCommand()` gigantesca."* So each concern is its own trait
//! implementation, and [`ValidationPipeline`] runs them in a fixed order,
//! cheapest and most universal first — a malformed request is rejected before
//! anything asks the world a question.
//!
//! # The order is a security property, not a performance one
//!
//! Structure, then identity, then authorization, then quota, and only then
//! anything that touches world state. Reordering would let an unauthenticated
//! caller make the server do work — the classic way a validation pipeline
//! becomes an amplification vector.
//!
//! # There is no trusted bypass
//!
//! §72: *"Nunca assumir internal = always valid, porque bugs internos também
//! existem."* Every command runs the same layers whatever its [`Source`]. The
//! source changes which *policies* apply, never whether they are checked.
//!
//! [`Source`]: crate::identity::Source

use std::collections::HashMap;

use nexora_foundation::time::WorldTime;

use crate::definition::CommandDefinition;
use crate::identity::QuotaKey;
use crate::instance::{CommandInstance, Parameter, MAX_PARAMETER_TEXT_BYTES};
use crate::result::FailureReason;

/// One layer of validation.
///
/// Returning `None` means "this layer has no objection", not "this command is
/// good" — only the whole pipeline can say that.
pub trait Validator {
    /// A short stable name, so a trace can say which layer refused (§91).
    fn name(&self) -> &'static str;

    /// Inspect the request.
    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason>;
}

/// Everything a layer is allowed to look at.
///
/// A borrowed view rather than owned state: a validator that could mutate the
/// world would be a validator that can change what a later layer sees.
#[derive(Debug)]
pub struct ValidationRequest<'a> {
    /// The request under inspection.
    pub instance: &'a CommandInstance,
    /// Its registered definition.
    pub definition: &'a CommandDefinition,
    /// The tick validation is running at.
    pub now: WorldTime,
    /// Whether this process is authoritative (§43).
    pub authoritative: bool,
}

/// §19 — shape and bounds, before anything touches the world.
#[derive(Debug, Default)]
pub struct StructuralValidator;

impl Validator for StructuralValidator {
    fn name(&self) -> &'static str {
        "structural"
    }

    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason> {
        for parameter in &request.instance.parameters {
            let too_long = match parameter {
                Parameter::Text(text) | Parameter::Ident(text) => {
                    text.len() > MAX_PARAMETER_TEXT_BYTES
                }
                // §19's own examples: a negative quantity and an invalid
                // number are structural faults, not gameplay ones.
                Parameter::Real(value) => {
                    if !value.is_finite() {
                        return Some(FailureReason::MalformedRequest);
                    }
                    false
                }
                Parameter::Integer(_) | Parameter::Count(_) | Parameter::Flag(_) => false,
            };
            if too_long {
                return Some(FailureReason::MalformedRequest);
            }
        }
        None
    }
}

/// §20 — is the actor who the request claims?
///
/// The engine has no session table yet, so this checks the invariant that can
/// be checked: an actor kind that must carry an identity has to carry one. A
/// player with no player id is a request the server cannot attribute, and
/// attributing it to nobody is how an unauthenticated action gets performed.
#[derive(Debug, Default)]
pub struct IdentityValidator;

impl Validator for IdentityValidator {
    fn name(&self) -> &'static str {
        "identity"
    }

    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason> {
        use crate::identity::ActorKind;

        let actor = &request.instance.context.actor;
        let needs_identity = matches!(
            actor.kind,
            ActorKind::Player | ActorKind::Npc | ActorKind::Machine
        );
        let has_identity =
            actor.player.is_some() || actor.account.is_some() || actor.entity.is_some();

        if needs_identity && !has_identity {
            return Some(FailureReason::IdentityMismatch);
        }
        None
    }
}

/// §21 and §7 — may this actor, arriving this way, send this command?
#[derive(Debug, Default)]
pub struct AuthorizationValidator;

impl Validator for AuthorizationValidator {
    fn name(&self) -> &'static str {
        "authorization"
    }

    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason> {
        use crate::definition::AuthorityPolicy;

        let context = &request.instance.context;
        if !request.definition.source.permits_actor(context.actor.kind)
            || !request.definition.source.permits_source(context.source)
        {
            return Some(FailureReason::SourceNotPermitted);
        }
        if request.definition.authority == AuthorityPolicy::ServerRequired && !request.authoritative
        {
            return Some(FailureReason::NotAuthoritative);
        }
        None
    }
}

/// §34 — is this still worth acting on?
#[derive(Debug, Default)]
pub struct ExpiryValidator;

impl Validator for ExpiryValidator {
    fn name(&self) -> &'static str {
        "expiry"
    }

    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason> {
        let limit = request.definition.expiry.max_age_ticks?;
        (request.instance.age_ticks(request.now) > limit).then_some(FailureReason::Expired)
    }
}

/// §41 — has something asked for this to be abandoned?
#[derive(Debug, Default)]
pub struct CancellationValidator;

impl Validator for CancellationValidator {
    fn name(&self) -> &'static str {
        "cancellation"
    }

    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason> {
        request
            .instance
            .context
            .cancelled
            .then_some(FailureReason::Cancelled)
    }
}

/// §62–§63 — per-actor, per-command-type quotas.
///
/// Counts are per tick and per bucket. §63's example is 100,000
/// `BreakBlockCommand`s: the queue must not be what discovers that.
#[derive(Debug, Default)]
pub struct RateLimiter {
    counts: HashMap<(QuotaKey, String), u32>,
    tick: u64,
}

impl RateLimiter {
    /// An empty limiter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            tick: 0,
        }
    }

    /// Count one accepted command, and say whether it fits the quota.
    ///
    /// Call this **once** per command that reaches the quota layer, and only
    /// after the cheaper layers have passed: counting rejected garbage would
    /// let a flood of malformed requests exhaust a legitimate actor's budget.
    pub fn admit(
        &mut self,
        now: WorldTime,
        key: QuotaKey,
        definition: &CommandDefinition,
    ) -> Option<FailureReason> {
        if now.0 != self.tick {
            self.counts.clear();
            self.tick = now.0;
        }
        let limit = definition.rate.max_per_tick?;
        let entry = self
            .counts
            .entry((key, definition.id.to_string()))
            .or_insert(0);
        if *entry >= limit {
            return Some(FailureReason::RateLimited);
        }
        *entry += 1;
        None
    }

    /// How many have been admitted this tick for a bucket and command.
    #[must_use]
    pub fn admitted(&self, key: QuotaKey, command: &str) -> u32 {
        self.counts
            .get(&(key, command.to_owned()))
            .copied()
            .unwrap_or(0)
    }
}

/// The layers, run in order.
///
/// Holds the universal ones itself and accepts domain-specific layers (§22–§26)
/// afterwards, because target, state and resource checks need knowledge this
/// crate is forbidden to have (§134).
pub struct ValidationPipeline {
    universal: Vec<Box<dyn Validator>>,
    domain: Vec<Box<dyn Validator>>,
}

impl core::fmt::Debug for ValidationPipeline {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ValidationPipeline")
            .field("universal", &self.universal.len())
            .field("domain", &self.domain.len())
            .finish()
    }
}

impl ValidationPipeline {
    /// The universal layers, in §17's order.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            universal: vec![
                Box::new(StructuralValidator),
                Box::new(IdentityValidator),
                Box::new(AuthorizationValidator),
                Box::new(CancellationValidator),
                Box::new(ExpiryValidator),
            ],
            domain: Vec::new(),
        }
    }

    /// Add a domain layer, which runs after every universal one.
    #[must_use]
    pub fn with_domain(mut self, validator: Box<dyn Validator>) -> Self {
        self.domain.push(validator);
        self
    }

    /// Run every layer, stopping at the first objection.
    ///
    /// Returns the layer's name alongside the reason so a trace can say *where*
    /// it stopped, not merely that it did.
    #[must_use]
    pub fn run(&self, request: &ValidationRequest<'_>) -> Option<(&'static str, FailureReason)> {
        self.universal
            .iter()
            .chain(self.domain.iter())
            .find_map(|validator| {
                validator
                    .check(request)
                    .map(|reason| (validator.name(), reason))
            })
    }

    /// The layer names, in the order they run.
    #[must_use]
    pub fn layer_names(&self) -> Vec<&'static str> {
        self.universal
            .iter()
            .chain(self.domain.iter())
            .map(|validator| validator.name())
            .collect()
    }
}

impl Default for ValidationPipeline {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::diagnostics::CorrelationId;
    use nexora_foundation::version::CommandVersion;

    use crate::definition::{ExpiryPolicy, SourcePolicy};
    use crate::identity::{Actor, ActorKind, CommandId, CommandInstanceId, Source};
    use crate::instance::{CommandContext, Target};

    fn definition() -> CommandDefinition {
        CommandDefinition::new("nexora:break_block", CommandVersion(1))
            .expect("valid")
            .with_source(
                SourcePolicy::closed()
                    .allow_actor(ActorKind::Player)
                    .allow_source(Source::Network),
            )
    }

    fn instance_from(actor: Actor, source: Source, parameters: Vec<Parameter>) -> CommandInstance {
        CommandInstance::new(
            CommandId::nexora("break_block").expect("valid"),
            CommandInstanceId(1),
            Target::None,
            parameters,
            CommandContext::new(actor, source, WorldTime(10), CorrelationId(1)),
        )
    }

    fn request<'a>(
        instance: &'a CommandInstance,
        definition: &'a CommandDefinition,
    ) -> ValidationRequest<'a> {
        ValidationRequest {
            instance,
            definition,
            now: WorldTime(10),
            authoritative: true,
        }
    }

    #[test]
    fn a_well_formed_player_command_passes_every_layer() {
        let definition = definition();
        let instance = instance_from(Actor::player(1), Source::Network, Vec::new());
        assert!(ValidationPipeline::standard()
            .run(&request(&instance, &definition))
            .is_none());
    }

    #[test]
    fn structure_is_checked_before_anything_asks_the_world() {
        // An oversized parameter must be refused by the first layer, so that a
        // malformed request never reaches a layer that does real work.
        let definition = definition();
        let instance = instance_from(
            Actor::player(1),
            Source::Network,
            vec![Parameter::Text("x".repeat(MAX_PARAMETER_TEXT_BYTES + 1))],
        );
        let (layer, reason) = ValidationPipeline::standard()
            .run(&request(&instance, &definition))
            .expect("refused");
        assert_eq!(layer, "structural");
        assert_eq!(reason, FailureReason::MalformedRequest);
    }

    #[test]
    fn a_non_finite_number_is_a_structural_fault() {
        let definition = definition();
        let instance = instance_from(
            Actor::player(1),
            Source::Network,
            vec![Parameter::Real(f64::NAN)],
        );
        assert_eq!(
            ValidationPipeline::standard()
                .run(&request(&instance, &definition))
                .map(|(_, reason)| reason),
            Some(FailureReason::MalformedRequest)
        );
    }

    #[test]
    fn a_player_with_no_identity_cannot_be_attributed() {
        let definition = definition();
        let instance = instance_from(Actor::of(ActorKind::Player), Source::Network, Vec::new());
        let (layer, reason) = ValidationPipeline::standard()
            .run(&request(&instance, &definition))
            .expect("refused");
        assert_eq!(layer, "identity");
        assert_eq!(reason, FailureReason::IdentityMismatch);
    }

    #[test]
    fn an_origin_the_definition_does_not_allow_is_denied() {
        let definition = definition();
        // Same actor, different door. The policy names the door too (§12).
        let instance = instance_from(Actor::player(1), Source::Console, Vec::new());
        assert_eq!(
            ValidationPipeline::standard()
                .run(&request(&instance, &definition))
                .map(|(_, reason)| reason),
            Some(FailureReason::SourceNotPermitted)
        );
    }

    #[test]
    fn a_non_authoritative_process_refuses_a_server_required_command() {
        let definition = definition();
        let instance = instance_from(Actor::player(1), Source::Network, Vec::new());
        let mut request = request(&instance, &definition);
        request.authoritative = false;
        assert_eq!(
            ValidationPipeline::standard()
                .run(&request)
                .map(|(_, reason)| reason),
            Some(FailureReason::NotAuthoritative)
        );
    }

    #[test]
    fn the_server_is_validated_like_everyone_else() {
        // §72: "nunca assumir internal = always valid". A server command that
        // the definition does not permit is refused exactly like a player's.
        let definition = definition();
        let instance = instance_from(Actor::server(), Source::Local, Vec::new());
        assert_eq!(
            ValidationPipeline::standard()
                .run(&request(&instance, &definition))
                .map(|(_, reason)| reason),
            Some(FailureReason::SourceNotPermitted)
        );
    }

    #[test]
    fn a_stale_command_expires() {
        let definition = definition().with_expiry(ExpiryPolicy {
            max_age_ticks: Some(5),
        });
        let instance = instance_from(Actor::player(1), Source::Network, Vec::new());
        let mut request = request(&instance, &definition);
        request.now = WorldTime(20);
        assert_eq!(
            ValidationPipeline::standard()
                .run(&request)
                .map(|(_, reason)| reason),
            Some(FailureReason::Expired)
        );
    }

    #[test]
    fn the_universal_layers_run_in_the_documented_order() {
        // §17: structure, identity, authorization, then the rest. If this
        // order changes, an unauthenticated caller can make the server work.
        assert_eq!(
            ValidationPipeline::standard().layer_names(),
            vec![
                "structural",
                "identity",
                "authorization",
                "cancellation",
                "expiry",
            ]
        );
    }

    #[test]
    fn domain_layers_run_after_every_universal_one() {
        struct AlwaysRefuses;
        impl Validator for AlwaysRefuses {
            fn name(&self) -> &'static str {
                "domain"
            }
            fn check(&self, _: &ValidationRequest<'_>) -> Option<FailureReason> {
                Some(FailureReason::InvalidState)
            }
        }

        let definition = definition();
        let pipeline = ValidationPipeline::standard().with_domain(Box::new(AlwaysRefuses));
        assert_eq!(pipeline.layer_names().last(), Some(&"domain"));

        // A structurally broken request stops at the first layer, so the
        // domain layer never sees it.
        let broken = instance_from(
            Actor::player(1),
            Source::Network,
            vec![Parameter::Real(f64::INFINITY)],
        );
        assert_eq!(
            pipeline.run(&request(&broken, &definition)).map(|(l, _)| l),
            Some("structural")
        );
    }

    #[test]
    fn the_quota_admits_up_to_the_limit_then_refuses() {
        let definition = definition().with_rate(crate::definition::RatePolicy {
            max_per_tick: Some(2),
        });
        let mut limiter = RateLimiter::new();
        let key = QuotaKey::Player(1);
        assert!(limiter.admit(WorldTime(1), key, &definition).is_none());
        assert!(limiter.admit(WorldTime(1), key, &definition).is_none());
        assert_eq!(
            limiter.admit(WorldTime(1), key, &definition),
            Some(FailureReason::RateLimited)
        );
        assert_eq!(limiter.admitted(key, "nexora:break_block"), 2);
    }

    #[test]
    fn quotas_reset_each_tick_and_do_not_bleed_between_actors() {
        let definition = definition().with_rate(crate::definition::RatePolicy {
            max_per_tick: Some(1),
        });
        let mut limiter = RateLimiter::new();
        assert!(limiter
            .admit(WorldTime(1), QuotaKey::Player(1), &definition)
            .is_none());
        // A different player has their own budget in the same tick.
        assert!(limiter
            .admit(WorldTime(1), QuotaKey::Player(2), &definition)
            .is_none());
        // And the next tick starts clean.
        assert!(limiter
            .admit(WorldTime(2), QuotaKey::Player(1), &definition)
            .is_none());
    }

    #[test]
    fn one_command_type_flooding_does_not_consume_anothers_budget() {
        // §63: a flood of one kind must not starve a different kind.
        let breaking = definition().with_rate(crate::definition::RatePolicy {
            max_per_tick: Some(1),
        });
        let placing = CommandDefinition::new("nexora:place_block", CommandVersion(1))
            .expect("valid")
            .with_rate(crate::definition::RatePolicy {
                max_per_tick: Some(1),
            });

        let mut limiter = RateLimiter::new();
        let key = QuotaKey::Player(1);
        assert!(limiter.admit(WorldTime(1), key, &breaking).is_none());
        assert_eq!(
            limiter.admit(WorldTime(1), key, &breaking),
            Some(FailureReason::RateLimited)
        );
        assert!(limiter.admit(WorldTime(1), key, &placing).is_none());
    }
}
