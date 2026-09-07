//! Block commands: the first real pipeline from intent to a changed world.
//!
//! `Command System.md` §115 defines the first vertical slice as
//! `CLIENT → Networking → BreakBlockCommand → Validation → Server →
//! Build & Destruction → Block System → BlockBrokenEvent → Loot → Item →
//! Persistence → Networking`. Networking, Build & Destruction, Loot and Item do
//! not exist yet, so this is the reachable part of it:
//!
//! ```text
//! BreakBlockCommand -> validation -> handler -> World::set_block -> event name
//! ```
//!
//! # Why these live here and not in `nexora-command`
//!
//! §134 forbids the command system from containing block rules, and §28
//! separates the *handler* (which adapts) from the *system* (which decides).
//! `nexora-command` cannot reach `nexora-world` — the dependency is not
//! declared — so the rule is enforced by the build. `nexora-simulation` is the
//! one crate allowed to see both, exactly as it is for physics (ADR-0007) and
//! residency (ADR-0008).
//!
//! # The reach check is a server-side rule, deliberately
//!
//! §23 lists *"distance valid?"* among target validation, and §72 says a client
//! command is untrusted. A client that asks to break a block a kilometre away
//! is refused here, on the authority's side, rather than trusted to have
//! checked its own reach.

use std::cell::RefCell;
use std::rc::Rc;

use nexora_command::definition::{CommandDefinition, SourcePolicy};
use nexora_command::handler::{CommandHandler, Execution};
use nexora_command::identity::{ActorKind, Source};
use nexora_command::instance::{CommandInstance, Parameter, Target};
use nexora_command::registry::CommandRegistry;
use nexora_command::result::FailureReason;
use nexora_command::validation::{ValidationRequest, Validator};
use nexora_foundation::error::Result;
use nexora_foundation::spatial::BlockPos;
use nexora_foundation::version::CommandVersion;
use nexora_runtime::registry::RuntimeId;
use nexora_world::voxel::{BlockStateId, AIR};
use nexora_world::world::World;

/// How far an actor may reach to change a block, in blocks.
///
/// A single published constant rather than a number inline in the check, so the
/// value the server enforces is the value a client can be told (§44 startup
/// brief: no magic numbers).
pub const MAX_REACH_BLOCKS: f64 = 6.0;

/// The command id for breaking a block.
pub const BREAK_BLOCK: &str = "nexora:break_block";

/// The command id for placing a block.
pub const PLACE_BLOCK: &str = "nexora:place_block";

/// The event name announced after a successful break (§55).
pub const BLOCK_BROKEN_EVENT: &str = "nexora:block_broken";

/// The event name announced after a successful place.
pub const BLOCK_PLACED_EVENT: &str = "nexora:block_placed";

/// Register the block commands.
///
/// # Errors
///
/// Returns an error when a definition is invalid or already registered.
pub fn register_block_commands(registry: &mut CommandRegistry) -> Result<(RuntimeId, RuntimeId)> {
    let sources = || {
        SourcePolicy::closed()
            .allow_actor(ActorKind::Player)
            .allow_actor(ActorKind::Admin)
            .allow_source(Source::Network)
            .allow_source(Source::Local)
    };

    let break_id = registry
        .register(CommandDefinition::new(BREAK_BLOCK, CommandVersion(1))?.with_source(sources()))?;
    let place_id = registry
        .register(CommandDefinition::new(PLACE_BLOCK, CommandVersion(1))?.with_source(sources()))?;
    Ok((break_id, place_id))
}

/// Where the actor is standing, for the reach check.
///
/// A separate input rather than something read from the instance, because the
/// server must not take the actor's word for its own position — that is the
/// value the check exists to test against.
#[derive(Debug, Clone, Copy, Default)]
pub struct ActorPosition {
    /// World X.
    pub x: f64,
    /// World Y.
    pub y: f64,
    /// World Z.
    pub z: f64,
}

impl ActorPosition {
    /// Distance to a block's centre.
    #[must_use]
    pub fn distance_to(self, block: BlockPos) -> f64 {
        let dx = self.x - (block.x as f64 + 0.5);
        let dy = self.y - (block.y as f64 + 0.5);
        let dz = self.z - (block.z as f64 + 0.5);
        dx.mul_add(dx, dy.mul_add(dy, dz * dz)).sqrt()
    }
}

/// §23 — target validation for block commands.
///
/// Registered as a domain layer on the pipeline, so it runs after every
/// universal layer and before any handler.
#[derive(Debug, Default)]
pub struct BlockTargetValidator {
    /// Where the acting player is, when known.
    pub actor_position: Option<ActorPosition>,
}

impl Validator for BlockTargetValidator {
    fn name(&self) -> &'static str {
        "block-target"
    }

    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason> {
        let Target::Block { x, y, z } = request.instance.target else {
            // A block command aimed at something that is not a block is
            // malformed, not merely unlucky.
            return Some(FailureReason::MalformedRequest);
        };

        if let Some(position) = self.actor_position {
            let block = BlockPos::new(x, y, z);
            if position.distance_to(block) > MAX_REACH_BLOCKS {
                return Some(FailureReason::TargetOutOfRange);
            }
        }
        None
    }
}

/// Breaks a block (§28: adapts the command to the system that owns the rule).
pub struct BreakBlockHandler {
    world: Rc<RefCell<World>>,
}

impl BreakBlockHandler {
    /// A handler writing into `world`.
    #[must_use]
    pub const fn new(world: Rc<RefCell<World>>) -> Self {
        Self { world }
    }
}

impl CommandHandler for BreakBlockHandler {
    fn execute(
        &mut self,
        instance: &CommandInstance,
    ) -> core::result::Result<Execution, FailureReason> {
        let Target::Block { x, y, z } = instance.target else {
            return Err(FailureReason::MalformedRequest);
        };
        let position = BlockPos::new(x, y, z);
        let mut world = self.world.borrow_mut();

        // Breaking air is not a state change, and reporting success for it
        // would announce a `BlockBrokenEvent` for a block that never existed.
        match world.get_block(position) {
            Ok(state) if state == AIR => return Err(FailureReason::TargetNotFound),
            Ok(_) => {}
            // The world's own answer for a non-resident chunk. §23 lists
            // "chunk loaded?" as target validation, and this is the authority
            // that actually knows.
            Err(_) => return Err(FailureReason::WorldNotLoaded),
        }

        world
            .set_block(position, AIR)
            .map_err(|_| FailureReason::InvalidState)?;
        Ok(Execution::announcing(BLOCK_BROKEN_EVENT))
    }
}

/// Places a block.
pub struct PlaceBlockHandler {
    world: Rc<RefCell<World>>,
    /// The state placed when the request does not name one.
    default_state: BlockStateId,
}

impl PlaceBlockHandler {
    /// A handler writing `default_state` into `world`.
    #[must_use]
    pub const fn new(world: Rc<RefCell<World>>, default_state: BlockStateId) -> Self {
        Self {
            world,
            default_state,
        }
    }
}

impl CommandHandler for PlaceBlockHandler {
    fn execute(
        &mut self,
        instance: &CommandInstance,
    ) -> core::result::Result<Execution, FailureReason> {
        let Target::Block { x, y, z } = instance.target else {
            return Err(FailureReason::MalformedRequest);
        };
        let position = BlockPos::new(x, y, z);

        // The state to place, if the request named one.
        let state = instance
            .parameters
            .iter()
            .find_map(|parameter| match parameter {
                Parameter::Count(raw) => u32::try_from(*raw).ok().map(BlockStateId),
                _ => None,
            })
            .unwrap_or(self.default_state);

        let mut world = self.world.borrow_mut();
        match world.get_block(position) {
            // Placing into occupied space is a state error, not a missing
            // target: the block the caller wants to occupy is already taken.
            Ok(existing) if existing != AIR => return Err(FailureReason::InvalidState),
            Ok(_) => {}
            Err(_) => return Err(FailureReason::WorldNotLoaded),
        }

        world
            .set_block(position, state)
            .map_err(|_| FailureReason::InvalidState)?;
        Ok(Execution::announcing(BLOCK_PLACED_EVENT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_command::dispatcher::Dispatcher;
    use nexora_command::identity::{Actor, CommandId, CommandInstanceId, InstanceIdSource};
    use nexora_command::instance::CommandContext;
    use nexora_command::result::CommandStatus;
    use nexora_command::validation::ValidationPipeline;
    use nexora_foundation::diagnostics::CorrelationId;
    use nexora_foundation::ident::Identifier;
    use nexora_foundation::time::{CalendarConfig, WorldTime};
    use nexora_world::world::WorldDescriptor;

    /// A world with one generated chunk around the origin.
    fn world() -> Rc<RefCell<World>> {
        let descriptor = WorldDescriptor::new("command-test", 0xC0FFEE).expect("valid");
        let mut world = World::create(descriptor, CalendarConfig::earthlike()).expect("created");
        world.bring_online().expect("online");
        world
            .load_or_generate(nexora_foundation::spatial::ChunkCoord::new(0, 0))
            .expect("generated");
        Rc::new(RefCell::new(world))
    }

    fn stone(world: &World) -> BlockStateId {
        world
            .block_id(&Identifier::parse("nexora:block/stone").expect("valid"))
            .expect("registered")
    }

    /// A solid block inside the generated chunk, found rather than assumed:
    /// hard-coding a height would make the test depend on the generator's
    /// current shape instead of on the command pipeline.
    fn a_solid_block(world: &World) -> BlockPos {
        for y in (0..120).rev() {
            let position = BlockPos::new(4, y, 4);
            if world.get_block(position).map(|state| state != AIR) == Ok(true) {
                return position;
            }
        }
        panic!("the generated chunk has no solid block in column (4, 4)");
    }

    struct Harness {
        registry: CommandRegistry,
        dispatcher: Dispatcher,
        ids: InstanceIdSource,
        world: Rc<RefCell<World>>,
    }

    fn harness(actor_position: Option<ActorPosition>) -> Harness {
        let world = world();
        let mut registry = CommandRegistry::new().expect("valid");
        let (break_id, place_id) = register_block_commands(&mut registry).expect("registers");

        let default_state = stone(&world.borrow());
        let mut dispatcher = Dispatcher::new().with_pipeline(
            ValidationPipeline::standard()
                .with_domain(Box::new(BlockTargetValidator { actor_position })),
        );
        dispatcher
            .attach(
                break_id,
                Box::new(BreakBlockHandler::new(Rc::clone(&world))),
            )
            .expect("break handler");
        dispatcher
            .attach(
                place_id,
                Box::new(PlaceBlockHandler::new(Rc::clone(&world), default_state)),
            )
            .expect("place handler");

        Harness {
            registry,
            dispatcher,
            ids: InstanceIdSource::new(),
            world,
        }
    }

    fn request(
        harness: &mut Harness,
        command: &str,
        target: Target,
        parameters: Vec<Parameter>,
    ) -> CommandInstance {
        CommandInstance::new(
            CommandId::parse(command).expect("valid"),
            harness.ids.mint().expect("id"),
            target,
            parameters,
            CommandContext::new(
                Actor::player(1),
                Source::Network,
                WorldTime(1),
                CorrelationId(1),
            ),
        )
    }

    #[test]
    fn breaking_a_block_changes_the_world_and_announces_it() {
        let mut harness = harness(None);
        let position = a_solid_block(&harness.world.borrow());
        let instance = request(
            &mut harness,
            BREAK_BLOCK,
            Target::Block {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            Vec::new(),
        );

        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));

        assert!(result.succeeded(), "{:?}", result.reason);
        assert_eq!(result.events, vec![BLOCK_BROKEN_EVENT]);
        assert_eq!(
            harness
                .world
                .borrow()
                .get_block(position)
                .expect("resident"),
            AIR,
            "the world actually changed"
        );
    }

    #[test]
    fn placing_a_block_fills_air_and_announces_it() {
        let mut harness = harness(None);
        let solid = a_solid_block(&harness.world.borrow());
        // Directly above the surface is air.
        let position = BlockPos::new(solid.x, solid.y + 1, solid.z);
        assert_eq!(
            harness
                .world
                .borrow()
                .get_block(position)
                .expect("resident"),
            AIR
        );

        let instance = request(
            &mut harness,
            PLACE_BLOCK,
            Target::Block {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            Vec::new(),
        );
        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));

        assert!(result.succeeded(), "{:?}", result.reason);
        assert_eq!(result.events, vec![BLOCK_PLACED_EVENT]);
        assert_ne!(
            harness
                .world
                .borrow()
                .get_block(position)
                .expect("resident"),
            AIR
        );
    }

    #[test]
    fn breaking_air_reports_no_target_rather_than_succeeding() {
        // Reporting success would announce a BlockBrokenEvent for a block that
        // never existed, and §56 says the event must describe what happened.
        let mut harness = harness(None);
        let solid = a_solid_block(&harness.world.borrow());
        let instance = request(
            &mut harness,
            BREAK_BLOCK,
            Target::Block {
                x: solid.x,
                y: solid.y + 10,
                z: solid.z,
            },
            Vec::new(),
        );
        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));

        assert_eq!(result.reason, Some(FailureReason::TargetNotFound));
        assert!(
            result.events.is_empty(),
            "nothing happened, announce nothing"
        );
    }

    #[test]
    fn placing_into_occupied_space_is_refused() {
        let mut harness = harness(None);
        let solid = a_solid_block(&harness.world.borrow());
        let before = harness.world.borrow().get_block(solid).expect("resident");

        let instance = request(
            &mut harness,
            PLACE_BLOCK,
            Target::Block {
                x: solid.x,
                y: solid.y,
                z: solid.z,
            },
            Vec::new(),
        );
        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));

        assert_eq!(result.reason, Some(FailureReason::InvalidState));
        assert_eq!(
            harness.world.borrow().get_block(solid).expect("resident"),
            before,
            "a refused command must not change anything"
        );
    }

    #[test]
    fn a_block_outside_a_resident_chunk_reports_the_world_not_loaded() {
        let mut harness = harness(None);
        // Far outside the one generated chunk.
        let instance = request(
            &mut harness,
            BREAK_BLOCK,
            Target::Block {
                x: 100_000,
                y: 64,
                z: 100_000,
            },
            Vec::new(),
        );
        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));
        assert_eq!(result.reason, Some(FailureReason::WorldNotLoaded));
    }

    #[test]
    fn a_block_out_of_reach_is_refused_by_the_server() {
        // §72: the client's own reach check is not evidence. The authority
        // rejects it, and the world is untouched.
        let mut harness = harness(Some(ActorPosition {
            x: 0.0,
            y: 70.0,
            z: 0.0,
        }));
        let position = BlockPos::new(60, 70, 60);
        let instance = request(
            &mut harness,
            BREAK_BLOCK,
            Target::Block {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            Vec::new(),
        );
        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));

        assert_eq!(result.reason, Some(FailureReason::TargetOutOfRange));
        assert_eq!(result.status, CommandStatus::Rejected);
    }

    #[test]
    fn a_block_within_reach_passes_the_same_check() {
        let mut harness = harness(Some(ActorPosition {
            x: 4.5,
            y: 100.0,
            z: 4.5,
        }));
        let position = a_solid_block(&harness.world.borrow());
        // Stand next to the block rather than at a fixed height, so the test
        // does not depend on where the generator put the surface.
        let standing = ActorPosition {
            x: position.x as f64 + 0.5,
            y: position.y as f64 + 1.5,
            z: position.z as f64 + 0.5,
        };
        harness.dispatcher = Dispatcher::new().with_pipeline(
            ValidationPipeline::standard().with_domain(Box::new(BlockTargetValidator {
                actor_position: Some(standing),
            })),
        );
        let break_id = harness
            .registry
            .runtime_id_of(&CommandId::parse(BREAK_BLOCK).expect("valid"))
            .expect("registered");
        harness
            .dispatcher
            .attach(
                break_id,
                Box::new(BreakBlockHandler::new(Rc::clone(&harness.world))),
            )
            .expect("handler");

        let instance = request(
            &mut harness,
            BREAK_BLOCK,
            Target::Block {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            Vec::new(),
        );
        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));
        assert!(result.succeeded(), "{:?}", result.reason);
    }

    #[test]
    fn a_command_aimed_at_no_block_is_malformed() {
        let mut harness = harness(None);
        let instance = request(&mut harness, BREAK_BLOCK, Target::None, Vec::new());
        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));
        assert_eq!(result.reason, Some(FailureReason::MalformedRequest));
    }

    #[test]
    fn an_unregistered_actor_kind_cannot_break_blocks() {
        // The definitions allow Player and Admin. A script arriving through the
        // mod runtime is refused, and the world is untouched.
        let mut harness = harness(None);
        let position = a_solid_block(&harness.world.borrow());
        let mut instance = request(
            &mut harness,
            BREAK_BLOCK,
            Target::Block {
                x: position.x,
                y: position.y,
                z: position.z,
            },
            Vec::new(),
        );
        instance.context.actor = Actor::of(ActorKind::Script);
        instance.context.source = Source::ModRuntime;

        let result = harness
            .dispatcher
            .dispatch(&harness.registry, &instance, WorldTime(1));
        assert_eq!(result.status, CommandStatus::Denied);
        assert_ne!(
            harness
                .world
                .borrow()
                .get_block(position)
                .expect("resident"),
            AIR
        );
    }

    #[test]
    fn the_reach_limit_is_the_published_constant() {
        // The value the server enforces has to be the value a client can be
        // told, or the two disagree at exactly the boundary.
        let origin = ActorPosition {
            x: 0.5,
            y: 0.5,
            z: 0.5,
        };
        let just_inside = BlockPos::new(0, 0, MAX_REACH_BLOCKS as i64 - 1);
        let far_outside = BlockPos::new(0, 0, MAX_REACH_BLOCKS as i64 + 4);
        assert!(origin.distance_to(just_inside) <= MAX_REACH_BLOCKS);
        assert!(origin.distance_to(far_outside) > MAX_REACH_BLOCKS);
    }

    #[test]
    fn a_duplicate_instance_id_cannot_break_the_same_block_twice() {
        // §10/§35: the instance id exists so a replayed request is recognised.
        let mut harness = harness(None);
        let position = a_solid_block(&harness.world.borrow());
        let target = Target::Block {
            x: position.x,
            y: position.y,
            z: position.z,
        };

        let mut queue = nexora_command::queue::CommandQueue::new();
        let instance = request(&mut harness, BREAK_BLOCK, target, Vec::new());
        let replayed = CommandInstance::new(
            instance.id.clone(),
            instance.instance,
            instance.target.clone(),
            Vec::new(),
            instance.context.clone(),
        );

        assert!(queue
            .enqueue(nexora_command::queue::Queued::now(instance, WorldTime(1)))
            .is_ok());
        assert_eq!(
            queue.enqueue(nexora_command::queue::Queued::now(replayed, WorldTime(1))),
            Err(nexora_command::queue::EnqueueRejection::Duplicate)
        );

        let results =
            harness
                .dispatcher
                .run_queued(&harness.registry, &mut queue, WorldTime(1), 10);
        assert_eq!(results.len(), 1, "the duplicate never ran");
        assert!(results[0].succeeded());
        assert_eq!(CommandInstanceId(1), results[0].instance);
    }
}
