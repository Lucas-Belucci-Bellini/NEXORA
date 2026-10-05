//! The player's hands: an interaction ray from the eye, the block it meets,
//! and the two things a player can do to a block today (ADR-0036).
//!
//! `PLAYER SYSTEM.md` PLAYER-13 draws the chain — *Player → Interaction Ray →
//! Target → Interaction API* — PLAYER-16 narrows it to blocks, and PLAYER-14
//! says what an interaction carries: the player, the target, the position, the
//! face. This module is that chain, ending where the world's authority begins:
//!
//! ```text
//! eye + facing -> look_direction -> raycast (nexora-physics) -> Target
//!   -> BREAK_BLOCK / PLACE_BLOCK -> validation -> handler -> World::set_block
//! ```
//!
//! The commands, their handlers and the reach the authority enforces already
//! existed (`commands`); nothing here writes to the world except through them.
//!
//! # Two halves, in one process today
//!
//! [`target`] is the half a client runs: it reads terrain and says what the
//! player is looking at. The command it becomes is the only thing that crosses
//! to the authority, which checks it against the [`Stance`] it reads from the
//! player's body — never against anything the request claims (`Command
//! System.md` §72: a client command is untrusted). In one process both halves
//! are [`Interaction::act`]; over a network the target travels and the stance
//! does not.
//!
//! # Why the ray is shorter than the reach
//!
//! The authority measures reach from the eye to the **centre** of the target
//! block ([`MAX_REACH_BLOCKS`]); a ray meets a **face**. No point of a unit
//! cell is further than half its diagonal, √3/2, from its centre, so a ray no
//! longer than [`INTERACTION_REACH`] = 6 − √3/2 only ever offers blocks the
//! authority accepts. A longer ray would show the player targets that are then
//! refused. The place cell is the one in front of the face, whose centre is
//! further than the block's by up to one block, so a placement can still be
//! refused for reach — by the authority, which is where that rule lives.
//!
//! # A block is not placed inside a body
//!
//! `Build & Destruction Engine.md` BUILD-1 lists collision among what a
//! placement must check. A block placed into the space a body fills would bury
//! it, and the player treats being buried as a defect, not a state to carry on
//! from (ADR-0035). So the authority refuses it ([`StanceValidator`]). A block
//! under the feet in mid-jump is accepted: touching is not sharing volume.
//!
//! # Where the trigonometry is
//!
//! [`look_direction`] is the camera's `forward` formula, written again here
//! because the simulation does not depend on the camera crate; the client,
//! which sees both, pins them equal to the bit. It is `sin_cos`, so a target
//! computed on two platforms can differ in the last place (DEBT-0051): over a
//! network the target block is what travels, not the ray.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use nexora_command::dispatcher::Dispatcher;
use nexora_command::identity::{Actor, CommandId, InstanceIdSource, Source};
use nexora_command::instance::{CommandContext, CommandInstance, Target as CommandTarget};
use nexora_command::registry::CommandRegistry;
use nexora_command::result::FailureReason;
use nexora_command::validation::{ValidationPipeline, ValidationRequest, Validator};
use nexora_foundation::diagnostics::CorrelationId;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::BlockPos;
use nexora_foundation::time::WorldTime;
use nexora_physics::math::Vec3;
use nexora_physics::query::raycast;
use nexora_physics::voxel::VoxelSource;
use nexora_world::voxel::BlockStateId;
use nexora_world::world::World;

use crate::commands::{
    register_block_commands, ActorPosition, BreakBlockHandler, PlaceBlockHandler, BREAK_BLOCK,
    MAX_REACH_BLOCKS, PLACE_BLOCK,
};
use crate::player::{Eye, Player};
use crate::terrain::WorldVoxels;

/// Half the diagonal of a unit cell, √3/2: the furthest any point of a block
/// is from its centre.
pub const HALF_BLOCK_DIAGONAL: f64 = 0.866_025_403_784_438_6;

/// How far the interaction ray reaches from the eye, in metres: the
/// authority's reach less [`HALF_BLOCK_DIAGONAL`], so every block the ray
/// meets is one the authority accepts as within reach.
pub const INTERACTION_REACH: f64 = MAX_REACH_BLOCKS - HALF_BLOCK_DIAGONAL;

/// The unit vector a facing looks along: yaw zero looks down `-Z`, a
/// positive pitch looks up (ADR-0029).
///
/// The same formula as the camera's `forward`, so the block under the centre
/// of the screen is the block the ray meets.
#[must_use]
pub fn look_direction(yaw: f64, pitch: f64) -> [f64; 3] {
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    [-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch]
}

/// What a player can do to a block today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Break the block the ray meets.
    Break,
    /// Place a block against the face the ray meets.
    Place,
}

impl Action {
    /// The command this action is sent as.
    #[must_use]
    pub const fn command(self) -> &'static str {
        match self {
            Self::Break => BREAK_BLOCK,
            Self::Place => PLACE_BLOCK,
        }
    }
}

/// The block the interaction ray met, and the face it met it by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// The solid cell the ray met.
    pub block: BlockPos,
    /// The face it entered by, as the unit step out of `block` through that
    /// face: exactly one component is `±1`.
    pub face: [i64; 3],
    /// Metres from the eye to where the ray met the face.
    pub distance: f64,
}

impl Target {
    /// The cell in front of the face: where a block placed against it goes.
    #[must_use]
    pub const fn in_front(&self) -> BlockPos {
        BlockPos::new(
            self.block.x + self.face[0],
            self.block.y + self.face[1],
            self.block.z + self.face[2],
        )
    }

    /// The cell `action` changes: the block itself to break, the cell in
    /// front of its face to place.
    #[must_use]
    pub const fn cell_for(&self, action: Action) -> BlockPos {
        match action {
            Action::Break => self.block,
            Action::Place => self.in_front(),
        }
    }
}

/// What the player is looking at: the first solid block along the look
/// direction from the eye, within [`INTERACTION_REACH`].
///
/// `None` when nothing solid is within reach, or when the eye is itself inside
/// a solid cell — there is no face to act on from inside a block.
#[must_use]
pub fn target<S: VoxelSource + ?Sized>(terrain: &S, eye: Eye) -> Option<Target> {
    let origin = Vec3::new(eye.position.x, eye.position.y, eye.position.z);
    let [x, y, z] = look_direction(eye.yaw, eye.pitch);
    let hit = raycast(terrain, origin, Vec3::new(x, y, z), INTERACTION_REACH)?;
    if hit.started_inside {
        return None;
    }
    // The solver's normal is exactly one axis at ±1, so these are exact.
    let face = [hit.normal.x, hit.normal.y, hit.normal.z].map(|component| component as i64);
    Some(Target {
        block: hit.cell,
        face,
        distance: hit.distance,
    })
}

/// What the authority knows of an actor when it acts: where the eye is, and
/// the box the body fills, in world metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stance {
    /// The eye: what reach is measured from.
    pub eye: ActorPosition,
    /// The body's lower corner.
    pub body_min: [f64; 3],
    /// The body's upper corner.
    pub body_max: [f64; 3],
}

impl Stance {
    /// Whether a block in `cell` would share volume with the body. Touching
    /// faces do not, the convention physics collides by.
    #[must_use]
    pub fn overlaps_block(&self, cell: BlockPos) -> bool {
        let low = [cell.x, cell.y, cell.z].map(|value| value as f64);
        (0..3).all(|axis| low[axis] < self.body_max[axis] && low[axis] + 1.0 > self.body_min[axis])
    }
}

/// The authority's check of a block command against the acting player's
/// [`Stance`]: within reach of the eye, and, for a placement, not into the
/// body.
///
/// The stance is set by [`Interaction::act`] for the one dispatch it makes,
/// from the player's body. With no stance set the request is refused: an
/// authority that does not know where the actor is cannot say it can reach.
#[derive(Debug)]
pub struct StanceValidator {
    stance: Rc<Cell<Option<Stance>>>,
    place: CommandId,
}

impl StanceValidator {
    /// A validator reading the stance from `stance`.
    ///
    /// # Errors
    ///
    /// The place command's id did not parse (a defect in this crate).
    pub fn new(stance: Rc<Cell<Option<Stance>>>) -> Result<Self> {
        Ok(Self {
            stance,
            place: CommandId::parse(PLACE_BLOCK)?,
        })
    }
}

impl Validator for StanceValidator {
    fn name(&self) -> &'static str {
        "player-stance"
    }

    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason> {
        let CommandTarget::Block { x, y, z } = request.instance.target else {
            return Some(FailureReason::MalformedRequest);
        };
        let Some(stance) = self.stance.get() else {
            return Some(FailureReason::InvalidState);
        };
        let block = BlockPos::new(x, y, z);
        if stance.eye.distance_to(block) > MAX_REACH_BLOCKS {
            return Some(FailureReason::TargetOutOfRange);
        }
        if request.instance.id == self.place && stance.overlaps_block(block) {
            // BUILD-1's collision check: the cell is taken, by a body.
            return Some(FailureReason::InvalidState);
        }
        None
    }
}

/// A change one action made to the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edit {
    /// The cell that changed.
    pub position: BlockPos,
    /// What was there.
    pub before: BlockStateId,
    /// What is there now.
    pub after: BlockStateId,
}

/// What one action came to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attempt {
    /// What was asked for.
    pub action: Action,
    /// What the ray met; `None` when nothing was within reach, and then no
    /// command was sent.
    pub target: Option<Target>,
    /// Why the authority refused, when it did.
    pub refused: Option<FailureReason>,
    /// The change, when it was accepted.
    pub edit: Option<Edit>,
}

/// One player's hands on one world: the block commands, their validation —
/// the universal layers and the [`StanceValidator`] — and their handlers.
///
/// The world is shared with the handlers, the same ownership the command
/// stage of the slice uses; the caller keeps its own handle and reads and
/// ticks against the world between actions.
#[derive(Debug)]
pub struct Interaction {
    world: Rc<RefCell<World>>,
    registry: CommandRegistry,
    dispatcher: Dispatcher,
    stance: Rc<Cell<Option<Stance>>>,
    ids: InstanceIdSource,
    actor: Actor,
    source: Source,
    correlation: u64,
}

impl Interaction {
    /// Hands for `actor`, arriving through `source`, placing `place` when a
    /// block is placed.
    ///
    /// # Errors
    ///
    /// The commands could not be registered or their handlers attached.
    pub fn new(
        world: Rc<RefCell<World>>,
        place: BlockStateId,
        actor: Actor,
        source: Source,
    ) -> Result<Self> {
        let mut registry = CommandRegistry::new()?;
        let (break_id, place_id) = register_block_commands(&mut registry)?;
        registry.freeze();
        let stance = Rc::new(Cell::new(None));
        let mut dispatcher = Dispatcher::new().with_pipeline(
            ValidationPipeline::standard()
                .with_domain(Box::new(StanceValidator::new(Rc::clone(&stance))?)),
        );
        dispatcher.attach(
            break_id,
            Box::new(BreakBlockHandler::new(Rc::clone(&world))),
        )?;
        dispatcher.attach(
            place_id,
            Box::new(PlaceBlockHandler::new(Rc::clone(&world), place)),
        )?;
        Ok(Self {
            world,
            registry,
            dispatcher,
            stance,
            ids: InstanceIdSource::new(),
            actor,
            source,
            correlation: 0,
        })
    }

    /// Hands for the one local player of a single-process client: player 1,
    /// arriving locally. The client names no command type to build them.
    ///
    /// # Errors
    ///
    /// As [`Interaction::new`].
    pub fn local_player(world: Rc<RefCell<World>>, place: BlockStateId) -> Result<Self> {
        Self::new(world, place, Actor::player(1), Source::Local)
    }

    /// Aim from `player`'s eye and ask the authority to do `action` to what
    /// the ray meets, at world time `now`.
    ///
    /// Nothing is sent when nothing is within reach. A refusal is an answer,
    /// not an error: it comes back in [`Attempt::refused`].
    ///
    /// # Errors
    ///
    /// A command id or instance id could not be made, or the world could not
    /// say what an accepted command left in the cell — defects, not refusals.
    pub fn act(&mut self, player: &Player, action: Action, now: WorldTime) -> Result<Attempt> {
        let aimed = {
            let world = self.world.borrow();
            target(&WorldVoxels::new(&world), player.eye())
        };
        let Some(aimed) = aimed else {
            return Ok(Attempt {
                action,
                target: None,
                refused: None,
                edit: None,
            });
        };
        let cell = aimed.cell_for(action);
        let before = self.world.borrow().get_block(cell).ok();
        self.correlation += 1;
        let instance = CommandInstance::new(
            CommandId::parse(action.command())?,
            self.ids.mint()?,
            CommandTarget::Block {
                x: cell.x,
                y: cell.y,
                z: cell.z,
            },
            Vec::new(),
            CommandContext::new(
                self.actor.clone(),
                self.source,
                now,
                CorrelationId(self.correlation),
            ),
        );

        // The stance is the authority's, read from the body for exactly this
        // dispatch, and gone again before anything else can be validated.
        self.stance.set(Some(player.stance()));
        let result = self.dispatcher.dispatch(&self.registry, &instance, now);
        self.stance.set(None);

        if !result.succeeded() {
            return Ok(Attempt {
                action,
                target: Some(aimed),
                refused: Some(result.reason.unwrap_or(FailureReason::HandlerFailed)),
                edit: None,
            });
        }
        let after = self.world.borrow().get_block(cell)?;
        let before = before.ok_or_else(|| {
            Error::new(
                Domain::World,
                "interaction",
                "a command changed a cell the world could not read before",
            )
            .with_recovery(Recovery::Manual)
            .with_context("cell", format!("{cell:?}"))
        })?;
        Ok(Attempt {
            action,
            target: Some(aimed),
            refused: None,
            edit: Some(Edit {
                position: cell,
                before,
                after,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_command::definition::CommandDefinition;
    use nexora_foundation::version::CommandVersion;

    /// A body standing in column `0, 0` with its feet on `y = 0`.
    fn standing() -> Stance {
        Stance {
            eye: ActorPosition {
                x: 0.5,
                y: 1.62,
                z: 0.5,
            },
            body_min: [0.2, 0.0, 0.2],
            body_max: [0.8, 1.8, 0.8],
        }
    }

    #[test]
    fn touching_is_not_sharing_volume() {
        let body = standing();
        assert!(
            !body.overlaps_block(BlockPos::new(0, -1, 0)),
            "under the feet"
        );
        assert!(
            !body.overlaps_block(BlockPos::new(0, 2, 0)),
            "over the head"
        );
        assert!(!body.overlaps_block(BlockPos::new(1, 0, 0)), "beside");
        assert!(
            body.overlaps_block(BlockPos::new(0, 0, 0)),
            "the feet's cell"
        );
        assert!(
            body.overlaps_block(BlockPos::new(0, 1, 0)),
            "the head's cell"
        );

        // Over a ledge, straddling the edge between two columns: the gap
        // under the overhang touches the feet and can be filled; the cell
        // beside the legs cannot.
        let straddling = Stance {
            body_min: [0.7, 3.0, 0.2],
            body_max: [1.3, 4.8, 0.8],
            ..body
        };
        assert!(!straddling.overlaps_block(BlockPos::new(1, 2, 0)));
        assert!(straddling.overlaps_block(BlockPos::new(1, 3, 0)));
    }

    #[test]
    fn the_validator_needs_a_stance_and_holds_reach_and_the_body() {
        let stance = Rc::new(Cell::new(None));
        let validator = StanceValidator::new(Rc::clone(&stance)).unwrap();
        let mut ids = InstanceIdSource::new();
        let mut check = |command: &str, at: BlockPos| {
            let definition = CommandDefinition::new(command, CommandVersion(1)).unwrap();
            let instance = CommandInstance::new(
                CommandId::parse(command).unwrap(),
                ids.mint().unwrap(),
                CommandTarget::Block {
                    x: at.x,
                    y: at.y,
                    z: at.z,
                },
                Vec::new(),
                CommandContext::new(
                    Actor::player(1),
                    Source::Local,
                    WorldTime(1),
                    CorrelationId(1),
                ),
            );
            validator.check(&ValidationRequest {
                instance: &instance,
                definition: &definition,
                now: WorldTime(1),
                authoritative: true,
            })
        };
        let near = BlockPos::new(0, -1, 2);
        let far = BlockPos::new(0, -1, 7);
        let feet = BlockPos::new(0, 0, 0);

        assert_eq!(
            check(BREAK_BLOCK, near),
            Some(FailureReason::InvalidState),
            "no stance, no reach"
        );
        stance.set(Some(standing()));
        assert_eq!(check(BREAK_BLOCK, near), None);
        assert_eq!(
            check(BREAK_BLOCK, far),
            Some(FailureReason::TargetOutOfRange)
        );
        assert_eq!(check(PLACE_BLOCK, near), None);
        assert_eq!(check(PLACE_BLOCK, feet), Some(FailureReason::InvalidState));
        // Breaking is not placing: the body does not stop a break.
        assert_eq!(check(BREAK_BLOCK, feet), None);
    }
}
