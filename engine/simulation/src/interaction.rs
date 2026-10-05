//! Interaction: the ray a player looks along, what it finds, and the
//! authority that turns a mining or building intent into a changed world
//! (ADR-0036).
//!
//! `PLAYER SYSTEM.md` draws it as *Player → Interaction Ray → Target →
//! Interaction API* (PLAYER-13), with a context of *player, target, position,
//! face* (PLAYER-14), and says twice that the player does not change a voxel
//! itself: *Build Intent → Build Engine* (PLAYER-21) and *Mining Intent →
//! Build/Destruction Engine* (PLAYER-22). The Build Engine does not exist yet;
//! its stand-ins are the block commands (ADR-0010), whose handlers are the
//! only code that writes a block for a player. So this module is three pieces
//! and none of them writes the world:
//!
//! * [`aim`] casts the interaction ray from an [`Eye`] along
//!   [`Eye::forward`], at most [`INTERACTION_REACH`] blocks, through the
//!   physics crate's grid walk (`PHYSICS.md` §23 names *"what block is the
//!   player looking at"* as one of its queries). It answers a [`Target`]: the
//!   first solid block, and the cell on the near side of the face the ray
//!   entered — where a block built against that face goes.
//! * [`PlayerTargetValidator`] is a domain layer of the command pipeline. It
//!   reads what the **authority** knows about the actor — never what the
//!   request claims (`Command System.md` §72) — and refuses a target out of
//!   reach (delegating to [`BlockTargetValidator`], so the reach rule exists
//!   once) and a block built where the actor's body is (BUILD-1's *colisão*).
//! * [`BlockEditor`] is the authority for one player: a registry with the
//!   block commands, a dispatcher with the standard pipeline plus that layer,
//!   and the handlers over the shared world. [`BlockEditor::apply`] casts the
//!   ray from its own copy of the player, sends `nexora:break_block` or
//!   `nexora:place_block` naming the cell's integers, and reports each
//!   [`Edit`] with the pipeline's answer.
//!
//! # What the command carries is a cell, not an angle
//!
//! The ray is the only trigonometry (`Eye::forward`), and a platform's `libm`
//! could round it differently (DEBT-0051). It ends in a cell, and the command
//! names that cell's integers: replaying the command changes the same block
//! on any machine, whatever the ray would have said there.
//!
//! # Reach is checked twice, on purpose
//!
//! The ray stops at [`INTERACTION_REACH`]; the server's rule is
//! [`MAX_REACH_BLOCKS`] from the eye to the block's centre. A cell the ray can
//! name is never further than the reach plus half a cell's diagonal, and the
//! two are held apart at compile time, so a target this client can aim at is
//! never refused for range by its own authority — while a request from
//! anywhere else still is.

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
    register_block_commands, ActorPosition, BlockTargetValidator, BreakBlockHandler,
    PlaceBlockHandler, BREAK_BLOCK, MAX_REACH_BLOCKS, PLACE_BLOCK,
};
use crate::controls::BlockIntent;
use crate::player::{Eye, Player};
use crate::terrain::WorldVoxels;

/// How far the interaction ray reaches from the eye, in blocks.
pub const INTERACTION_REACH: f64 = 5.0;

/// Half a cell's diagonal, `√3 / 2`: the furthest a cell's centre is from a
/// point on its surface.
const HALF_CELL_DIAGONAL: f64 = 0.866_025_403_784_438_6;

// A cell the ray names — the block hit, or the cell in front of the face it
// entered, both of which touch the hit point — has its centre within the reach
// plus half a diagonal of the eye. The authority's own limit must be further,
// or a player would be refused for a block it is plainly allowed to aim at.
const _: () = assert!(INTERACTION_REACH + HALF_CELL_DIAGONAL < MAX_REACH_BLOCKS);

/// What the interaction ray found (PLAYER-14's target, position and face).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    /// The first solid block along the ray.
    pub block: BlockPos,
    /// The cell on the eye's side of the face the ray entered: where a block
    /// built against that face goes. `None` when the eye is inside a solid
    /// cell, where no face was crossed.
    pub against: Option<BlockPos>,
    /// From the eye to the face, in blocks.
    pub distance: f64,
}

/// Cast the interaction ray from `eye` along where it looks, at most
/// [`INTERACTION_REACH`] blocks, against `terrain`.
///
/// Returns `None` when nothing solid is within reach. A cell the source
/// reports solid is solid here, so terrain that has not been generated —
/// which a world view reports solid — is a target like any other; the
/// authority refuses it ([`FailureReason::WorldNotLoaded`]).
#[must_use]
pub fn aim<S: VoxelSource + ?Sized>(terrain: &S, eye: &Eye) -> Option<Target> {
    let origin = Vec3::new(eye.position.x, eye.position.y, eye.position.z);
    let [x, y, z] = eye.forward();
    let hit = raycast(terrain, origin, Vec3::new(x, y, z), INTERACTION_REACH)?;
    let against = if hit.started_inside {
        None
    } else {
        // The normal points back along the ray, out of the face entered: one
        // unit along one axis, so rounding it is exact.
        let step = |component: f64| component.round() as i64;
        Some(BlockPos::new(
            hit.cell.x + step(hit.normal.x),
            hit.cell.y + step(hit.normal.y),
            hit.cell.z + step(hit.normal.z),
        ))
    };
    Some(Target {
        block: hit.cell,
        against,
        distance: hit.distance,
    })
}

/// What the authority knows about the acting player, read from its own copy
/// of the player when a command is sent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActorBody {
    /// The eye, where reach is measured from.
    pub eye: ActorPosition,
    /// The cells the body occupies, lowest and highest corner inclusive
    /// ([`Player::occupied_cells`]).
    pub cells: (BlockPos, BlockPos),
}

impl ActorBody {
    /// The actor as `player` is now.
    #[must_use]
    pub fn of(player: &Player) -> Self {
        let eye = player.eye().position;
        Self {
            eye: ActorPosition {
                x: eye.x,
                y: eye.y,
                z: eye.z,
            },
            cells: player.occupied_cells(),
        }
    }

    /// Whether the body occupies `cell`.
    #[must_use]
    pub const fn occupies(&self, cell: BlockPos) -> bool {
        let (low, high) = self.cells;
        cell.x >= low.x
            && cell.x <= high.x
            && cell.y >= low.y
            && cell.y <= high.y
            && cell.z >= low.z
            && cell.z <= high.z
    }
}

/// §23 target validation for a player's block commands, against the
/// authority's own copy of the player.
///
/// Reach is [`BlockTargetValidator`]'s rule, called rather than repeated.
/// A place whose cell the body occupies is refused as
/// [`FailureReason::InvalidState`] — the same answer as placing into a cell a
/// block already holds: the space is taken. With no actor recorded, nothing
/// is known to be out of reach or occupied, as with `BlockTargetValidator`.
#[derive(Debug, Clone, Default)]
pub struct PlayerTargetValidator {
    /// The actor, as the authority last read it.
    pub actor: Rc<Cell<Option<ActorBody>>>,
}

impl Validator for PlayerTargetValidator {
    fn name(&self) -> &'static str {
        "player-target"
    }

    fn check(&self, request: &ValidationRequest<'_>) -> Option<FailureReason> {
        let actor = self.actor.get();
        let reach = BlockTargetValidator {
            actor_position: actor.map(|body| body.eye),
        };
        if let Some(reason) = reach.check(request) {
            return Some(reason);
        }
        let (Some(body), CommandTarget::Block { x, y, z }) = (actor, &request.instance.target)
        else {
            return None;
        };
        let placing = request.instance.id.identifier().to_string() == PLACE_BLOCK;
        (placing && body.occupies(BlockPos::new(*x, *y, *z))).then_some(FailureReason::InvalidState)
    }
}

/// Which edit a player asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    /// Break the block looked at (`nexora:break_block`).
    Mine,
    /// Place a block against the face looked at (`nexora:place_block`).
    Build,
}

impl EditKind {
    /// A stable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mine => "mine",
            Self::Build => "build",
        }
    }
}

/// One edit a player asked for, and what became of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edit {
    /// What was asked.
    pub kind: EditKind,
    /// The cell the command named; `None` when the ray found nothing to
    /// name, so no command was sent.
    pub cell: Option<BlockPos>,
    /// `None` when the world changed; otherwise why not.
    pub refused: Option<Refusal>,
}

impl Edit {
    /// Whether the world changed.
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.refused.is_none()
    }
}

/// Why an edit did not change the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Nothing solid within reach, or (to build) no face crossed: there was
    /// no cell to name, and no command was sent.
    NoTarget,
    /// The cell lies outside the area this editor may change; no command was
    /// sent.
    OutsideArea,
    /// The command pipeline refused it.
    Command(FailureReason),
}

impl Refusal {
    /// A stable code: `no_target`, `outside_area`, or the pipeline's own.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NoTarget => "no_target",
            Self::OutsideArea => "outside_area",
            Self::Command(reason) => reason.code(),
        }
    }
}

/// What a [`BlockEditor`] has done, summed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditTally {
    /// Blocks broken.
    pub mined: u64,
    /// Blocks placed.
    pub built: u64,
    /// Asked for with nothing to name, or outside the area: no command sent.
    pub unsent: u64,
    /// Sent, and refused by the pipeline.
    pub refused: u64,
}

impl EditTally {
    fn count(&mut self, edit: &Edit) {
        match (edit.kind, edit.refused) {
            (EditKind::Mine, None) => self.mined += 1,
            (EditKind::Build, None) => self.built += 1,
            (_, Some(Refusal::Command(_))) => self.refused += 1,
            (_, Some(_)) => self.unsent += 1,
        }
    }
}

/// The cells an editor may change: lowest corner inclusive, highest
/// exclusive, on every axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditArea {
    /// The lowest corner, inclusive.
    pub min: BlockPos,
    /// The highest corner, exclusive.
    pub max: BlockPos,
}

impl EditArea {
    /// Whether `cell` lies inside.
    #[must_use]
    pub const fn contains(&self, cell: BlockPos) -> bool {
        cell.x >= self.min.x
            && cell.x < self.max.x
            && cell.y >= self.min.y
            && cell.y < self.max.y
            && cell.z >= self.min.z
            && cell.z < self.max.z
    }
}

/// The authority for one player's block edits (ADR-0036).
///
/// Every edit goes through the command pipeline: the standard layers, then
/// [`PlayerTargetValidator`], then the one handler for the command, which is
/// the only code here that writes the world.
pub struct BlockEditor {
    world: Rc<RefCell<World>>,
    registry: CommandRegistry,
    dispatcher: Dispatcher,
    ids: InstanceIdSource,
    actor: Rc<Cell<Option<ActorBody>>>,
    who: Actor,
    source: Source,
    area: Option<EditArea>,
    correlation: u64,
    tally: EditTally,
}

impl BlockEditor {
    /// An authority writing into `world` for local player `player`,
    /// building `place` blocks.
    ///
    /// Its commands arrive from [`Source::Local`]: the process that owns the
    /// world is the one the player sits at. There is no session to tie the
    /// player to yet (DEBT-0022), and no network for a command to cross; a
    /// server's editor is the same type with another source, and Phase 6's.
    ///
    /// # Errors
    ///
    /// The block commands could not be registered or their handlers attached.
    pub fn new(world: Rc<RefCell<World>>, place: BlockStateId, player: u64) -> Result<Self> {
        let mut registry = CommandRegistry::new()?;
        let (break_id, place_id) = register_block_commands(&mut registry)?;
        registry.freeze();
        let actor = Rc::new(Cell::new(None));
        let mut dispatcher = Dispatcher::new().with_pipeline(
            ValidationPipeline::standard().with_domain(Box::new(PlayerTargetValidator {
                actor: Rc::clone(&actor),
            })),
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
            ids: InstanceIdSource::new(),
            actor,
            who: Actor::player(player),
            source: Source::Local,
            area: None,
            correlation: 0,
            tally: EditTally::default(),
        })
    }

    /// Limit edits to `area`: a cell outside it is not sent.
    ///
    /// The client sets the region it draws (ADR-0036): an edit it could not
    /// show is not one it should make.
    #[must_use]
    pub const fn within(mut self, area: EditArea) -> Self {
        self.area = Some(area);
        self
    }

    /// What this editor has done so far.
    #[must_use]
    pub const fn tally(&self) -> EditTally {
        self.tally
    }

    /// Carry out what `intent` asks of `player` at world time `now`: mine
    /// first, then build, each with its own ray, so a build in the same tick
    /// as a mine aims past the block just broken.
    ///
    /// The actor the pipeline judges is read from `player` here, not taken
    /// from anything the request carries.
    ///
    /// # Errors
    ///
    /// A command id or an instance id could not be made; a refusal is not an
    /// error, it is an [`Edit`] that says why.
    pub fn apply(
        &mut self,
        player: &Player,
        intent: BlockIntent,
        now: WorldTime,
    ) -> Result<Vec<Edit>> {
        let mut edits = Vec::new();
        for (asked, kind) in [
            (intent.mine, EditKind::Mine),
            (intent.build, EditKind::Build),
        ] {
            if asked {
                let edit = self.one(player, kind, now)?;
                self.tally.count(&edit);
                edits.push(edit);
            }
        }
        Ok(edits)
    }

    fn one(&mut self, player: &Player, kind: EditKind, now: WorldTime) -> Result<Edit> {
        let eye = player.eye();
        let target = {
            let world = self.world.borrow();
            aim(&WorldVoxels::new(&world), &eye)
        };
        let cell = match (kind, target) {
            (EditKind::Mine, Some(target)) => Some(target.block),
            (EditKind::Build, Some(target)) => target.against,
            (_, None) => None,
        };
        let Some(cell) = cell else {
            return Ok(Edit {
                kind,
                cell: None,
                refused: Some(Refusal::NoTarget),
            });
        };
        if self.area.is_some_and(|area| !area.contains(cell)) {
            return Ok(Edit {
                kind,
                cell: Some(cell),
                refused: Some(Refusal::OutsideArea),
            });
        }
        self.actor.set(Some(ActorBody::of(player)));
        self.correlation += 1;
        let command = match kind {
            EditKind::Mine => BREAK_BLOCK,
            EditKind::Build => PLACE_BLOCK,
        };
        let instance = CommandInstance::new(
            CommandId::parse(command)?,
            self.ids.mint()?,
            CommandTarget::Block {
                x: cell.x,
                y: cell.y,
                z: cell.z,
            },
            Vec::new(),
            CommandContext::new(
                self.who.clone(),
                self.source,
                now,
                CorrelationId(self.correlation),
            ),
        );
        let result = self.dispatcher.dispatch(&self.registry, &instance, now);
        Ok(Edit {
            kind,
            cell: Some(cell),
            refused: if result.succeeded() {
                None
            } else {
                Some(Refusal::Command(result.reason.ok_or_else(|| {
                    Error::new(
                        Domain::Command,
                        "block-editor",
                        "a refused command gave no reason",
                    )
                    .with_recovery(Recovery::Manual)
                })?))
            },
        })
    }
}

impl core::fmt::Debug for BlockEditor {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("BlockEditor")
            .field("area", &self.area)
            .field("tally", &self.tally)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Walk, MAX_PITCH};
    use crate::spawn::{find_walkable_run, ColumnArea};
    use nexora_command::definition::CommandDefinition;
    use nexora_command::identity::InstanceIdSource;
    use nexora_foundation::ident::Identifier;
    use nexora_foundation::spatial::{ChunkCoord, WorldPosition};
    use nexora_foundation::time::{CalendarConfig, DEFAULT_TICKS_PER_SECOND};
    use nexora_foundation::version::CommandVersion;
    use nexora_physics::voxel::{FlatGround, VoxelShape};
    use nexora_world::voxel::AIR;
    use nexora_world::world::WorldDescriptor;

    const SEED: u64 = 0x4E58_4F52;

    /// A generated world, columns `-1..=1` and a ring, and the area of the
    /// inner columns read down from above their highest surface.
    fn world() -> (World, ColumnArea) {
        let mut world = World::create(
            WorldDescriptor::new("interaction", SEED).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        world.bring_online().unwrap();
        for x in -2..=2 {
            for z in -2..=2 {
                world.load_or_generate(ChunkCoord::new(x, z)).unwrap();
            }
        }
        let size = i64::from(world.descriptor().shape.size_x());
        let (min, max) = (-size, 2 * size);
        let mut high = i64::MIN;
        for x in min..max {
            for z in min..max {
                high = high.max(world.surface_height(x, z));
            }
        }
        let area = ColumnArea {
            min_x: min,
            min_z: min,
            max_x: max,
            max_z: max,
            floor_y: world.descriptor().bounds.min_y,
            ceiling_y: high + 8,
        };
        (world, area)
    }

    /// The world shared the way the editor needs it, a player standing in it
    /// looking `pitch` below level, and an editor building stone.
    fn standing(pitch: f64) -> (Rc<RefCell<World>>, Player, BlockEditor) {
        let (world, area) = world();
        let player = {
            let terrain = WorldVoxels::new(&world);
            let run = find_walkable_run(&terrain, &area, 3).unwrap();
            Player::spawn(&terrain, &run, pitch, DEFAULT_TICKS_PER_SECOND).unwrap()
        };
        let stone = world
            .block_id(&Identifier::parse("nexora:block/stone").unwrap())
            .unwrap();
        let world = Rc::new(RefCell::new(world));
        let editor = BlockEditor::new(Rc::clone(&world), stone, 1).unwrap();
        (world, player, editor)
    }

    /// Every cell within two blocks of `centre`, and what it holds.
    fn cells_around(world: &World, centre: BlockPos) -> Vec<(BlockPos, BlockStateId)> {
        let mut cells = Vec::new();
        for dx in -2..=2 {
            for dy in -2..=2 {
                for dz in -2..=2 {
                    let cell = BlockPos::new(centre.x + dx, centre.y + dy, centre.z + dz);
                    cells.push((cell, world.get_block(cell).unwrap()));
                }
            }
        }
        cells
    }

    fn target_of(world: &Rc<RefCell<World>>, player: &Player) -> Option<Target> {
        let world = world.borrow();
        aim(&WorldVoxels::new(&world), &player.eye())
    }

    const MINE: BlockIntent = BlockIntent {
        mine: true,
        build: false,
    };
    const BUILD: BlockIntent = BlockIntent {
        mine: false,
        build: true,
    };

    /// Looking straight down from a body standing on flat ground: the floor
    /// cell under the eye, entered through its top face, and the cell above
    /// it — where the feet are.
    #[test]
    fn the_ray_names_the_block_below_and_the_cell_in_front_of_its_face() {
        let eye = Eye {
            position: WorldPosition::new(0.5, 1.62, 0.5),
            yaw: 0.0,
            pitch: -MAX_PITCH,
        };
        let target = aim(&FlatGround::at(0), &eye).expect("the floor is in reach");
        assert_eq!(target.block, BlockPos::new(0, -1, 0));
        assert_eq!(target.against, Some(BlockPos::new(0, 0, 0)));
        assert!((target.distance - 1.62).abs() < 1e-3, "{}", target.distance);
    }

    /// Level on flat ground, nothing is within reach; and the reach is a
    /// hard edge: a wall at 4.9 blocks is a target, one at 5.1 is not.
    #[test]
    fn nothing_beyond_the_reach_is_a_target() {
        let level = Eye {
            position: WorldPosition::new(0.5, 1.62, 0.5),
            yaw: 0.0,
            pitch: 0.0,
        };
        assert_eq!(aim(&FlatGround::at(0), &level), None);

        /// A wall whose face, towards +Z, is the plane `z = face`.
        struct Wall {
            face: i64,
        }
        impl VoxelSource for Wall {
            fn shape_at(&self, position: BlockPos) -> VoxelShape {
                if position.z < self.face {
                    VoxelShape::SOLID
                } else {
                    VoxelShape::Empty
                }
            }
        }
        // The eye at z = 0.5 looks down -Z: a face at z = -4 is 4.5 away.
        let near = aim(&Wall { face: -4 }, &level).expect("4.5 blocks away");
        assert_eq!(near.block, BlockPos::new(0, 1, -5));
        assert_eq!(near.against, Some(BlockPos::new(0, 1, -4)));
        assert_eq!(aim(&Wall { face: -5 }, &level), None, "5.5 blocks away");
    }

    /// An eye inside a solid cell crossed no face: there is a block to mine
    /// and nowhere to build.
    #[test]
    fn an_eye_inside_a_block_has_nothing_to_build_against() {
        let buried = Eye {
            position: WorldPosition::new(0.5, -0.5, 0.5),
            yaw: 0.0,
            pitch: 0.0,
        };
        let target = aim(&FlatGround::at(0), &buried).expect("inside");
        assert_eq!(target.block, BlockPos::new(0, -1, 0));
        assert_eq!(target.against, None);
    }

    /// The same direction as the camera's, written out again here: a ray
    /// cast straight ahead at yaw π/2 runs down -X.
    #[test]
    fn the_eye_looks_where_the_camera_would() {
        let eye = Eye {
            position: WorldPosition::new(0.0, 0.0, 0.0),
            yaw: std::f64::consts::FRAC_PI_2,
            pitch: 0.0,
        };
        let [x, y, z] = eye.forward();
        assert!((x + 1.0).abs() < 1e-12 && y == 0.0 && z.abs() < 1e-12);
    }

    #[test]
    fn mining_breaks_the_block_the_ray_names_and_nothing_else() {
        let (world, player, mut editor) = standing(-60f64.to_radians());
        let target = target_of(&world, &player).expect("the ground ahead is in reach");
        let before = cells_around(&world.borrow(), target.block);
        let edits = editor.apply(&player, MINE, WorldTime(1)).unwrap();
        assert_eq!(
            edits,
            vec![Edit {
                kind: EditKind::Mine,
                cell: Some(target.block),
                refused: None,
            }]
        );
        let after = cells_around(&world.borrow(), target.block);
        let changed: Vec<_> = before
            .iter()
            .zip(&after)
            .filter(|(a, b)| a != b)
            .map(|(a, _)| a.0)
            .collect();
        assert_eq!(changed, vec![target.block], "exactly one cell changed");
        assert_eq!(world.borrow().get_block(target.block).unwrap(), AIR);
        assert_eq!(editor.tally().mined, 1);
    }

    #[test]
    fn building_places_a_block_against_the_face_the_ray_entered() {
        let (world, player, mut editor) = standing(-60f64.to_radians());
        let target = target_of(&world, &player).expect("in reach");
        let cell = target.against.expect("a face was crossed");
        assert_eq!(world.borrow().get_block(cell).unwrap(), AIR);
        let edits = editor.apply(&player, BUILD, WorldTime(1)).unwrap();
        assert!(edits[0].changed(), "{edits:?}");
        assert_eq!(edits[0].cell, Some(cell));
        let stone = world
            .borrow()
            .block_id(&Identifier::parse("nexora:block/stone").unwrap())
            .unwrap();
        assert_eq!(world.borrow().get_block(cell).unwrap(), stone);
        assert_eq!(editor.tally().built, 1);
    }

    /// Looking straight down, the face crossed is the top of the block the
    /// feet stand on, and the cell in front of it is the one the feet are
    /// in: a block there would bury the body, and the authority refuses it
    /// as it refuses building into a cell a block already holds.
    #[test]
    fn building_where_the_body_is_is_refused() {
        let (world, player, mut editor) = standing(-MAX_PITCH);
        let target = target_of(&world, &player).expect("the floor");
        let cell = target.against.expect("its top face");
        assert!(ActorBody::of(&player).occupies(cell));
        let edits = editor.apply(&player, BUILD, WorldTime(1)).unwrap();
        assert_eq!(
            edits[0].refused,
            Some(Refusal::Command(FailureReason::InvalidState))
        );
        assert_eq!(world.borrow().get_block(cell).unwrap(), AIR);
        assert_eq!(editor.tally().refused, 1);
    }

    /// The edit reaches physics: mining the block under the feet drops the
    /// body one block onto the next, without ever pushing it out of terrain.
    #[test]
    fn mining_under_the_feet_drops_the_player_a_block() {
        let (world, mut player, mut editor) = standing(-MAX_PITCH);
        let start = player.state();
        let edits = editor.apply(&player, MINE, WorldTime(1)).unwrap();
        assert!(edits[0].changed(), "{edits:?}");
        let world = world.borrow();
        let terrain = WorldVoxels::new(&world);
        let mut landed = None;
        for _ in 0..40 {
            let outcome = player.tick(&terrain, Walk::default()).unwrap();
            assert_eq!(outcome.depenetrated, 0);
            let now = player.state();
            if now.grounded && now.feet.y < start.feet.y {
                landed = Some(now);
                break;
            }
        }
        let landed = landed.expect("the player fell and landed");
        // The floor below the mined block may itself be lower still: the
        // body drops at least the block it stood on.
        assert!(landed.feet.y <= start.feet.y - 1.0, "{landed:?}");
    }

    #[test]
    fn looking_at_the_sky_sends_no_command() {
        let (world, player, mut editor) = standing(MAX_PITCH);
        assert_eq!(target_of(&world, &player), None);
        let edits = editor.apply(&player, MINE.or(BUILD), WorldTime(1)).unwrap();
        assert_eq!(edits.len(), 2);
        assert!(edits
            .iter()
            .all(|edit| edit.refused == Some(Refusal::NoTarget) && edit.cell.is_none()));
        assert_eq!(
            editor.tally(),
            EditTally {
                unsent: 2,
                ..EditTally::default()
            }
        );
    }

    #[test]
    fn an_area_keeps_edits_inside_it() {
        let (world, player, editor) = standing(-60f64.to_radians());
        let target = target_of(&world, &player).expect("in reach");
        let mut editor = editor.within(EditArea {
            min: BlockPos::new(target.block.x + 1, i64::MIN / 2, i64::MIN / 2),
            max: BlockPos::new(i64::MAX / 2, i64::MAX / 2, i64::MAX / 2),
        });
        let edits = editor.apply(&player, MINE, WorldTime(1)).unwrap();
        assert_eq!(edits[0].refused, Some(Refusal::OutsideArea));
        assert_ne!(world.borrow().get_block(target.block).unwrap(), AIR);
    }

    /// Mine and build in one tick: the build aims past the block just
    /// broken, so the two never name the same cell for the same reason.
    #[test]
    fn mining_then_building_in_one_tick_is_mine_first() {
        let (world, player, mut editor) = standing(-60f64.to_radians());
        let first = target_of(&world, &player).expect("in reach");
        let edits = editor.apply(&player, MINE.or(BUILD), WorldTime(1)).unwrap();
        assert_eq!(edits[0].kind, EditKind::Mine);
        assert_eq!(edits[0].cell, Some(first.block));
        assert_eq!(edits[1].kind, EditKind::Build);
        assert!(edits.iter().all(Edit::changed), "{edits:?}");
        assert_ne!(edits[1].cell, first.against, "the second ray went further");
    }

    /// The authority judges the actor it holds, not one the request brings:
    /// the same cell is in reach for a player standing by it and out of
    /// reach once the recorded actor is elsewhere.
    #[test]
    fn the_validator_judges_the_actor_the_authority_recorded() {
        let actor = Rc::new(Cell::new(None));
        let validator = PlayerTargetValidator {
            actor: Rc::clone(&actor),
        };
        let definition = CommandDefinition::new(PLACE_BLOCK, CommandVersion(1)).unwrap();
        let mut ids = InstanceIdSource::new();
        let mut request = |command: &str, cell: BlockPos| {
            CommandInstance::new(
                CommandId::parse(command).unwrap(),
                ids.mint().unwrap(),
                CommandTarget::Block {
                    x: cell.x,
                    y: cell.y,
                    z: cell.z,
                },
                Vec::new(),
                CommandContext::new(
                    Actor::player(1),
                    Source::Local,
                    WorldTime(1),
                    CorrelationId(1),
                ),
            )
        };
        let check = |instance: &CommandInstance| {
            validator.check(&ValidationRequest {
                instance,
                definition: &definition,
                now: WorldTime(1),
                authoritative: true,
            })
        };
        let body = ActorBody {
            eye: ActorPosition {
                x: 0.5,
                y: 1.62,
                z: 0.5,
            },
            cells: (BlockPos::new(0, 0, 0), BlockPos::new(0, 1, 0)),
        };
        let beside = BlockPos::new(1, 0, 0);
        let feet = BlockPos::new(0, 0, 0);
        assert_eq!(check(&request(PLACE_BLOCK, beside)), None, "no actor yet");
        actor.set(Some(body));
        assert_eq!(check(&request(PLACE_BLOCK, beside)), None);
        assert_eq!(
            check(&request(PLACE_BLOCK, feet)),
            Some(FailureReason::InvalidState)
        );
        assert_eq!(
            check(&request(BREAK_BLOCK, feet)),
            None,
            "the body is no reason to refuse a break"
        );
        actor.set(Some(ActorBody {
            eye: ActorPosition {
                x: 40.5,
                ..body.eye
            },
            ..body
        }));
        assert_eq!(
            check(&request(PLACE_BLOCK, beside)),
            Some(FailureReason::TargetOutOfRange)
        );
    }

    /// The same intents against the same world change it the same way: the
    /// commands carry cells, so two runs save the same bytes.
    #[test]
    fn the_same_intents_make_the_same_world() {
        let run = || {
            let (world, player, mut editor) = standing(-60f64.to_radians());
            for tick in 1..=3 {
                editor
                    .apply(&player, MINE.or(BUILD), WorldTime(tick))
                    .unwrap();
            }
            drop(editor);
            let world = Rc::try_unwrap(world).unwrap().into_inner();
            nexora_world::persist::save(&world).unwrap().encode()
        };
        assert_eq!(run(), run());
    }
}
