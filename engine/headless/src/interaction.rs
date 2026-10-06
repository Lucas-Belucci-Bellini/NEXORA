//! The slice's last loop: a player changes the world, the world and the
//! player are saved, reloaded, and go on as if nothing had happened
//! (ADR-0036).
//!
//! ```text
//! WORLD -> PLAYER -> INTERACTION -> STATE CHANGE -> SAVE -> RELOAD -> WORLD CONTINUES
//! ```
//!
//! The stage has a world of its own, generated from the slice's seed, so the
//! main world's probes, journal and byte-identical determinism check never
//! see an edit they did not make. Everything the player does goes in the way
//! the client's window puts it in: a frame of HID usages — keys on the
//! keyboard, buttons on the mouse — sampled through the client's own table,
//! then one world tick, the edits as commands through the player's
//! [`BlockEditor`], then the player's physics. Nothing in this module writes a
//! block.

use std::cell::RefCell;
use std::rc::Rc;

use nexora_command::result::FailureReason;
use nexora_foundation::diagnostics::{Category, Diagnostics, Level};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{BlockPos, ChunkCoord};
use nexora_foundation::time::{CalendarConfig, TimeScale};
use nexora_persistence::SaveContainer;
use nexora_runtime::input::{ButtonCode, DeviceId, DeviceKind, InputFrame, Signal};
use nexora_simulation::interaction::Refusal;
use nexora_simulation::{
    aim, find_walkable_run, load_player, save_player, BlockEditor, ColumnArea, Controls, Edit,
    EditKind, Player, WorldVoxels,
};
use nexora_world::persist;
use nexora_world::voxel::{BlockStateId, AIR};
use nexora_world::world::{World, WorldDescriptor};

/// The HID usages the script presses: the client's table.
const KEY_UP: u16 = 0x52;
const KEY_DOWN: u16 = 0x51;
const KEY_SPACE: u16 = 0x2C;
/// The mouse's primary (mine) and secondary (build) buttons.
const PRIMARY: u16 = 1;
const SECONDARY: u16 = 2;

const KEYBOARD: DeviceId = DeviceId::new(DeviceKind::Keyboard, 0);
const MOUSE: DeviceId = DeviceId::new(DeviceKind::Mouse, 0);

/// How far from a block's top a body at rest on it may be, in blocks.
const LANDING_TOLERANCE: f64 = 1e-9;

/// Ticks of an arrow that reach the pitch limit from anywhere.
const LOOK_ALL_THE_WAY: u32 = 45;

/// What the stage observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InteractionOutcome {
    /// Blocks broken before the save.
    pub mined: u64,
    /// Blocks placed before the save.
    pub built: u64,
    /// Edits the pipeline refused before the save.
    pub refused: u64,
    /// Edits with nothing to aim at before the save.
    pub unsent: u64,
    /// How far the player fell after mining under its feet, in centimetres.
    pub fell_cm: i64,
    /// The save of the world and the player, in bytes.
    pub save_bytes: usize,
    /// Ticks the reloaded world ran, identical to the world never saved.
    pub continued_ticks: u64,
    /// Edits those ticks changed the world with, on each side.
    pub continued_edits: u64,
}

/// One scripted frame: keys, then mouse buttons, each going down (`true`)
/// or up (`false`).
type ScriptFrame = (Vec<(u16, bool)>, Vec<(u16, bool)>);

/// A world, its player, the player's authority and its controls: one frame
/// of input, one world tick.
struct Session {
    world: Rc<RefCell<World>>,
    player: Player,
    editor: BlockEditor,
    controls: Controls,
    attached: bool,
}

impl Session {
    fn new(world: Rc<RefCell<World>>, player: Player) -> Result<Self> {
        let stone = world
            .borrow()
            .block_id(&Identifier::parse("nexora:block/stone")?)?;
        let editor = BlockEditor::new(Rc::clone(&world), stone, 1)?;
        Ok(Self {
            world,
            player,
            editor,
            controls: Controls::new()?,
            attached: false,
        })
    }

    /// One frame: `keys` and `buttons` go down (`true`) or up (`false`),
    /// then one world tick, the edits asked for, and the player's physics.
    fn frame(&mut self, keys: &[(u16, bool)], buttons: &[(u16, bool)]) -> Result<Vec<Edit>> {
        let mut signals = InputFrame::new();
        if !self.attached {
            signals.push(Signal::Attached(KEYBOARD));
            signals.push(Signal::Attached(MOUSE));
            self.attached = true;
        }
        for &(usage, pressed) in keys {
            signals.push(Signal::button(KEYBOARD, ButtonCode(usage), pressed));
        }
        for &(usage, pressed) in buttons {
            signals.push(Signal::button(MOUSE, ButtonCode(usage), pressed));
        }
        let intent = self.controls.sample(&signals);
        self.world
            .borrow_mut()
            .clock_mut()
            .advance_by(TimeScale::Tick, 1)?;
        let now = self.world.borrow().clock().now();
        let edits = self.editor.apply(&self.player, intent.block(), now)?;
        let world = self.world.borrow();
        let outcome = self.player.tick(&WorldVoxels::new(&world), intent.walk())?;
        if outcome.depenetrated > 0 {
            return Err(failure(
                "the player was pushed out of terrain; no edit may bury it",
            ));
        }
        Ok(edits)
    }

    /// Hold `key` for `ticks` frames, then let it go.
    fn hold(&mut self, key: u16, ticks: u32) -> Result<()> {
        self.frame(&[(key, true)], &[])?;
        for _ in 1..ticks {
            self.frame(&[], &[])?;
        }
        self.frame(&[(key, false)], &[])?;
        Ok(())
    }

    /// Click a mouse button: down for one frame, up on the next. Returns the
    /// edits of the frame it went down in.
    fn click(&mut self, button: u16) -> Result<Vec<Edit>> {
        let edits = self.frame(&[], &[(button, true)])?;
        let after = self.frame(&[], &[(button, false)])?;
        if !after.is_empty() {
            return Err(failure("a button held down asked for a second edit"));
        }
        Ok(edits)
    }

    /// What the player's ray names now.
    fn target(&self) -> Option<nexora_simulation::Target> {
        let world = self.world.borrow();
        aim(&WorldVoxels::new(&world), &self.player.eye())
    }

    fn block(&self, cell: BlockPos) -> Result<BlockStateId> {
        self.world.borrow().get_block(cell)
    }

    /// The world and the player, as one save.
    fn save(&self) -> Result<SaveContainer> {
        let mut container = persist::save(&self.world.borrow())?;
        save_player(&mut container, &self.player)?;
        Ok(container)
    }
}

/// The one edit a click must have made, changing the world at `cell`.
fn expect_changed(edits: &[Edit], kind: EditKind, cell: BlockPos) -> Result<()> {
    match edits {
        [edit] if edit.kind == kind && edit.cell == Some(cell) && edit.changed() => Ok(()),
        _ => Err(failure("a click did not make the one edit the ray named")
            .with_context("kind", kind.as_str())
            .with_context("cell", format!("{cell:?}"))
            .with_context("edits", format!("{edits:?}"))),
    }
}

/// Frames with no input until the player stands still on something, its
/// feet below `under`; returns the height of the feet.
fn settle_lower(session: &mut Session, under: f64) -> Result<f64> {
    let mut before = session.player.state();
    for _ in 0..60 {
        session.frame(&[], &[])?;
        let now = session.player.state();
        if now.grounded && now.feet.y < under && now.is_bit_identical(&before) {
            return Ok(now.feet.y);
        }
        before = now;
    }
    Err(failure("the player did not come to rest"))
}

/// Generate the stage's world: columns `-radius..=radius`, from the slice's
/// seed, under a name of its own.
fn generate(seed: u64, radius: i64) -> Result<(World, ColumnArea)> {
    let mut world = World::create(
        WorldDescriptor::new("nexora-slice-interaction", seed)?,
        CalendarConfig::earthlike(),
    )?;
    world.bring_online()?;
    for x in -radius..=radius {
        for z in -radius..=radius {
            world.load_or_generate(ChunkCoord::new(x, z))?;
        }
    }
    let shape = world.descriptor().shape;
    let (sx, sz) = (i64::from(shape.size_x()), i64::from(shape.size_z()));
    let (min_x, max_x) = (-radius * sx, (radius + 1) * sx);
    let (min_z, max_z) = (-radius * sz, (radius + 1) * sz);
    let mut high = i64::MIN;
    for x in min_x..max_x {
        for z in min_z..max_z {
            high = high.max(world.surface_height(x, z));
        }
    }
    let area = ColumnArea {
        min_x,
        min_z,
        max_x,
        max_z,
        floor_y: world.descriptor().bounds.min_y,
        ceiling_y: high + 8,
    };
    Ok((world, area))
}

/// Run the loop on a world of its own, from `seed`.
///
/// Before the save, through the arrow keys, Space and the mouse only, and in
/// an order whose outcome the terrain cannot change — every step aims
/// straight down at the block the player stands on, or straight up at open
/// sky (the generator makes no overhangs):
///
/// 1. **look straight down** and **build**: the cell in front of the face the
///    ray enters is the one the feet are in, and the authority refuses it;
/// 2. **mine** the block under the feet: the player falls at least a block,
///    and lands without ever being pushed out of terrain;
/// 3. **jump**, and at the top of the jump **build** under the feet: the cell
///    is free now, the block goes in, and the player lands on it;
/// 4. **look straight up** and **mine**: nothing in reach, no command.
///
/// Then the world and the player are saved into one container, encoded,
/// decoded, loaded and resumed. The resumed player must be the saved one to
/// the bit, every edited cell must hold what it held, and then **the world
/// continues**: the reloaded world and the one never saved run the same
/// frames — the player looks down, mines, jumps and builds — side by side,
/// and must stay identical tick by tick and save the same bytes at the end.
pub(crate) fn interact_and_resume(
    seed: u64,
    radius: i64,
    diagnostics: &Diagnostics,
) -> Result<InteractionOutcome> {
    let (world, area) = generate(seed, radius)?;
    let ticks_per_second = world.clock().calendar().ticks_per_second();
    let player = {
        let terrain = WorldVoxels::new(&world);
        let run = find_walkable_run(&terrain, &area, 3)?;
        Player::spawn(&terrain, &run, 0.0, ticks_per_second)?
    };
    let mut session = Session::new(Rc::new(RefCell::new(world)), player)?;
    let stone = session
        .world
        .borrow()
        .block_id(&Identifier::parse("nexora:block/stone")?)?;
    let mut edited = Vec::new();

    // 1. Straight down, building into the feet is refused.
    session.hold(KEY_DOWN, LOOK_ALL_THE_WAY)?;
    let floor = session
        .target()
        .ok_or_else(|| failure("looking straight down, there is no floor"))?;
    let feet = floor
        .against
        .ok_or_else(|| failure("the floor's top face was not crossed"))?;
    let refused = session.click(SECONDARY)?;
    match refused.as_slice() {
        [edit]
            if edit.refused == Some(Refusal::Command(FailureReason::InvalidState))
                && edit.cell == Some(feet) => {}
        _ => {
            return Err(
                failure("building into the player's own body was not refused")
                    .with_context("edits", format!("{refused:?}")),
            )
        }
    }
    if session.block(feet)? != AIR {
        return Err(failure("a refused build changed the world"));
    }

    // 2. Mine under the feet: the player falls, and lands.
    let standing = session.player.state();
    expect_changed(&session.click(PRIMARY)?, EditKind::Mine, floor.block)?;
    if session.block(floor.block)? != AIR {
        return Err(failure("a mined block is not air"));
    }
    edited.push(floor.block);
    let landed = settle_lower(&mut session, standing.feet.y)?;
    // Within a nanometre: a body comes to rest on a plane to the solver's
    // snapping tolerance, not always exactly on it.
    let fell = standing.feet.y - landed;
    if fell < 1.0 - LANDING_TOLERANCE {
        return Err(failure("the player fell less than the block it mined")
            .with_context("fell", format!("{fell:e}")));
    }

    // 3. Jump, and at the top of the jump build under the feet.
    let below = session
        .target()
        .ok_or_else(|| failure("in the hole, there is no floor"))?;
    let under = below
        .against
        .ok_or_else(|| failure("the hole's floor face was not crossed"))?;
    session.frame(&[(KEY_SPACE, true)], &[])?;
    session.frame(&[(KEY_SPACE, false)], &[])?;
    let mut risen = false;
    for _ in 0..20 {
        // Clear of the cell under the feet once the feet are above its top.
        if session.player.state().feet.y > landed + 1.0 {
            risen = true;
            break;
        }
        session.frame(&[], &[])?;
    }
    if !risen {
        return Err(failure("the jump never lifted the feet a block"));
    }
    expect_changed(&session.click(SECONDARY)?, EditKind::Build, under)?;
    if session.block(under)? != stone {
        return Err(failure("a built block is not the block built"));
    }
    edited.push(under);
    let on_top = settle_lower(&mut session, f64::INFINITY)?;
    if (on_top - (landed + 1.0)).abs() > LANDING_TOLERANCE {
        return Err(failure("the player did not land on the block it built")
            .with_context("feet", format!("{on_top:e}"))
            .with_context("expected", format!("{:e}", landed + 1.0)));
    }

    // 4. Straight up: nothing in reach, no command.
    session.hold(KEY_UP, 2 * LOOK_ALL_THE_WAY)?;
    match session.click(PRIMARY)?.as_slice() {
        [edit] if edit.refused == Some(Refusal::NoTarget) => {}
        other => {
            return Err(failure("mining the sky sent something")
                .with_context("edits", format!("{other:?}")))
        }
    }
    let tally = session.editor.tally();

    // --- save, reload, resume ----------------------------------------------
    let container = session.save()?;
    let bytes = container.encode();
    let reopened = SaveContainer::decode(&bytes)?;
    let mut loaded = persist::load(&reopened)?;
    loaded.bring_online()?;
    if loaded.clock().now() != session.world.borrow().clock().now() {
        return Err(failure("the world's time did not survive the save"));
    }
    for cell in &edited {
        if loaded.get_block(*cell)? != session.block(*cell)? {
            return Err(failure("an edit did not survive the save")
                .with_context("cell", format!("{cell:?}")));
        }
    }
    let saved = load_player(&reopened)?.ok_or_else(|| failure("the save holds no player"))?;
    let resumed = Player::resume(&WorldVoxels::new(&loaded), &saved, ticks_per_second)?;
    if !resumed.state().is_bit_identical(&session.player.state()) {
        return Err(failure("the resumed player is not the saved one")
            .with_context("saved", format!("{:?}", session.player.state()))
            .with_context("resumed", format!("{:?}", resumed.state())));
    }
    let mut again = Session::new(Rc::new(RefCell::new(loaded)), resumed)?;

    // --- the world continues -----------------------------------------------
    // The same frames for both: look back down, mine, jump and build at a
    // fixed tick. After every one the two must agree on the edits and on the
    // player, to the bit; at the end, on every byte of the save.
    let mut script: Vec<ScriptFrame> = Vec::new();
    script.push((vec![(KEY_DOWN, true)], vec![]));
    script.extend((1..2 * LOOK_ALL_THE_WAY).map(|_| (vec![], vec![])));
    script.push((vec![(KEY_DOWN, false)], vec![(PRIMARY, true)]));
    script.push((vec![], vec![(PRIMARY, false)]));
    script.extend((0..20).map(|_| (vec![], vec![])));
    script.push((vec![(KEY_SPACE, true)], vec![]));
    script.push((vec![(KEY_SPACE, false)], vec![]));
    script.extend((0..4).map(|_| (vec![], vec![])));
    script.push((vec![], vec![(SECONDARY, true)]));
    script.push((vec![], vec![(SECONDARY, false)]));
    script.extend((0..20).map(|_| (vec![], vec![])));
    let mut continued_ticks = 0;
    let mut continued_edits = 0;
    for (keys, buttons) in &script {
        let ours = session.frame(keys, buttons)?;
        let theirs = again.frame(keys, buttons)?;
        continued_ticks += 1;
        if ours != theirs
            || !again
                .player
                .state()
                .is_bit_identical(&session.player.state())
        {
            return Err(
                failure("the reloaded world went another way than the one never saved")
                    .with_context("tick", continued_ticks.to_string())
                    .with_context("edits", format!("{ours:?} against {theirs:?}")),
            );
        }
        continued_edits += ours.iter().filter(|edit| edit.changed()).count() as u64;
    }
    if continued_edits == 0 {
        return Err(failure("the continuation changed nothing to compare")
            .with_context("ticks", continued_ticks.to_string()));
    }
    if session.save()?.encode() != again.save()?.encode() {
        return Err(failure(
            "the reloaded world and the one never saved saved different bytes",
        ));
    }

    diagnostics.log(
        Level::Debug,
        Category::World,
        "slice/interaction",
        "the player changed the world, was saved with it, and both went on",
    );
    Ok(InteractionOutcome {
        mined: tally.mined,
        built: tally.built,
        refused: tally.refused,
        unsent: tally.unsent,
        fell_cm: (fell * 100.0).round() as i64,
        save_bytes: bytes.len(),
        continued_ticks,
        continued_edits,
    })
}

fn failure(message: &'static str) -> Error {
    Error::new(Domain::World, "slice/interaction", message).with_recovery(Recovery::Manual)
}
