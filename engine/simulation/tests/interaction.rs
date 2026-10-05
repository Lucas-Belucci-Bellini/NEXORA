//! The player's hands against real generated worlds (ADR-0036).
//!
//! What only a generated world and a real body can show: that the block under
//! the centre of the view is the one that breaks, that a placement against a
//! face lands in front of it, that the authority refuses a block placed into
//! the body that asked for it, and accepts one placed under its feet in
//! mid-jump — after which the body stands on it rather than inside it.

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use nexora_command::identity::{Actor, Source};
use nexora_command::result::FailureReason;
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{BlockPos, ChunkCoord};
use nexora_foundation::time::{CalendarConfig, WorldTime, DEFAULT_TICKS_PER_SECOND};
use nexora_simulation::commands::MAX_REACH_BLOCKS;
use nexora_simulation::interaction::{look_direction, target, INTERACTION_REACH};
use nexora_simulation::player::MAX_PITCH;
use nexora_simulation::{
    find_walkable_run, Action, ColumnArea, Interaction, Player, Run, Walk, WorldVoxels,
};
use nexora_world::voxel::{BlockStateId, AIR};
use nexora_world::world::{World, WorldDescriptor};

/// The client's default seed.
const CLIENT_SEED: u64 = 0x4E58_4F52;

/// Chunk columns `-radius..=radius` on both axes, generated.
fn world(seed: u64, radius: i64) -> World {
    let mut world = World::create(
        WorldDescriptor::new("interaction-test", seed).expect("a descriptor"),
        CalendarConfig::earthlike(),
    )
    .expect("a world");
    world.bring_online().expect("online");
    for x in -radius..=radius {
        for z in -radius..=radius {
            world
                .load_or_generate(ChunkCoord::new(x, z))
                .expect("generated");
        }
    }
    world
}

/// The columns of chunk `0, 0`, in the band the client draws.
fn area(world: &World) -> ColumnArea {
    let size = i64::from(world.descriptor().shape.size_x());
    let (mut low, mut high) = (i64::MAX, i64::MIN);
    for x in 0..size {
        for z in 0..size {
            let height = world.surface_height(x, z);
            low = low.min(height);
            high = high.max(height);
        }
    }
    ColumnArea {
        min_x: 0,
        min_z: 0,
        max_x: size,
        max_z: size,
        floor_y: low - 8,
        ceiling_y: high + 7,
    }
}

fn stone(world: &World) -> BlockStateId {
    world
        .block_id(&Identifier::parse("nexora:block/stone").expect("valid"))
        .expect("registered")
}

/// A generated world shared with a player's hands, and the player at rest on
/// a run of three columns, looking `pitch` below level.
fn scene(pitch: f64) -> (Rc<RefCell<World>>, Interaction, Player, Run) {
    let world = world(CLIENT_SEED, 1);
    let run = find_walkable_run(&WorldVoxels::new(&world), &area(&world), 3).expect("a run");
    let player = Player::spawn(
        &WorldVoxels::new(&world),
        &run,
        pitch,
        DEFAULT_TICKS_PER_SECOND,
    )
    .expect("spawned");
    let place = stone(&world);
    let world = Rc::new(RefCell::new(world));
    let hands =
        Interaction::new(Rc::clone(&world), place, Actor::player(1), Source::Local).expect("hands");
    (world, hands, player, run)
}

fn tick(world: &Rc<RefCell<World>>, player: &mut Player, walk: Walk) {
    let world = world.borrow();
    let outcome = player
        .tick(&WorldVoxels::new(&world), walk)
        .expect("a tick");
    assert_eq!(outcome.depenetrated, 0, "the body was never inside a block");
}

#[test]
fn the_look_direction_is_a_unit_vector_and_yaw_zero_looks_down_negative_z() {
    assert_eq!(look_direction(0.0, 0.0).map(f64::abs), [0.0, 0.0, 1.0]);
    assert!(look_direction(0.0, 0.0)[2] < 0.0);
    let up = look_direction(0.3, MAX_PITCH);
    assert!(up[1] > 0.99);
    for step in 0..64 {
        let yaw = -PI + f64::from(step) * PI / 32.0;
        let pitch = (f64::from(step) / 32.0 - 1.0) * MAX_PITCH;
        let d = look_direction(yaw, pitch);
        let length = d.iter().map(|c| c * c).sum::<f64>().sqrt();
        assert!((length - 1.0).abs() < 1e-12, "{yaw} {pitch}: {length}");
    }
}

/// The ray stops where the authority's reach would refuse: over a fan of
/// directions from a real eye, every block offered has its centre within
/// [`MAX_REACH_BLOCKS`] of the eye, and some are near the edge of it.
#[test]
fn the_ray_offers_only_blocks_the_authority_accepts_as_within_reach() {
    let (world, _hands, player, _) = scene(0.0);
    let world = world.borrow();
    let terrain = WorldVoxels::new(&world);
    let eye = player.eye();
    let (mut offered, mut furthest) = (0, 0.0f64);
    for yaw_step in 0..96 {
        for pitch_step in 0..32 {
            let mut aimed = eye;
            aimed.yaw = -PI + f64::from(yaw_step) * PI / 48.0;
            aimed.pitch = -MAX_PITCH + f64::from(pitch_step) * MAX_PITCH / 16.0;
            let Some(hit) = target(&terrain, aimed) else {
                continue;
            };
            offered += 1;
            assert!(hit.distance <= INTERACTION_REACH);
            assert_eq!(
                hit.face.iter().map(|c| c.abs()).sum::<i64>(),
                1,
                "one face, one axis: {hit:?}"
            );
            let centre = [hit.block.x, hit.block.y, hit.block.z].map(|c| c as f64 + 0.5);
            let p = eye.position;
            let reach =
                ((centre[0] - p.x).powi(2) + (centre[1] - p.y).powi(2) + (centre[2] - p.z).powi(2))
                    .sqrt();
            assert!(reach <= MAX_REACH_BLOCKS, "{hit:?} is {reach} from the eye");
            furthest = furthest.max(reach);
        }
    }
    assert!(offered > 1000, "the fan met the ground: {offered}");
    assert!(
        furthest > INTERACTION_REACH,
        "and reached its edge: {furthest}"
    );
}

#[test]
fn breaking_removes_the_block_under_the_centre_of_the_view() {
    let (world, mut hands, player, _) = scene(-60f64.to_radians());
    let aimed = {
        let world = world.borrow();
        target(&WorldVoxels::new(&world), player.eye()).expect("the ground ahead")
    };
    assert_eq!(
        aimed.face,
        [0, 1, 0],
        "looking down, the ray meets a top face"
    );
    let attempt = hands
        .act(&player, Action::Break, WorldTime(1))
        .expect("an answer");
    assert_eq!(attempt.refused, None);
    let edit = attempt.edit.expect("an edit");
    assert_eq!(edit.position, aimed.block);
    assert_ne!(edit.before, AIR);
    assert_eq!(edit.after, AIR);
    assert_eq!(world.borrow().get_block(aimed.block), Ok(AIR));
}

#[test]
fn placing_lands_in_front_of_the_face_the_ray_met() {
    let (world, mut hands, player, _) = scene(-60f64.to_radians());
    let aimed = {
        let world = world.borrow();
        target(&WorldVoxels::new(&world), player.eye()).expect("the ground ahead")
    };
    let attempt = hands
        .act(&player, Action::Place, WorldTime(1))
        .expect("an answer");
    assert_eq!(attempt.refused, None, "{attempt:?}");
    let edit = attempt.edit.expect("an edit");
    assert_eq!(edit.position, aimed.in_front());
    assert_eq!(edit.before, AIR);
    let stone = stone(&world.borrow());
    assert_eq!(edit.after, stone);
    assert_eq!(world.borrow().get_block(aimed.in_front()), Ok(stone));
}

#[test]
fn a_block_is_not_placed_inside_the_body_that_asks_for_it() {
    // Straight down: the ray meets the top of the block under the feet, and
    // the cell in front of that face is the one the feet stand in.
    let (world, mut hands, player, run) = scene(-MAX_PITCH);
    let feet = BlockPos::new(run.x, run.feet_y, run.z);
    let attempt = hands
        .act(&player, Action::Place, WorldTime(1))
        .expect("an answer");
    assert_eq!(attempt.target.map(|t| t.in_front()), Some(feet));
    assert_eq!(attempt.refused, Some(FailureReason::InvalidState));
    assert_eq!(attempt.edit, None);
    assert_eq!(world.borrow().get_block(feet), Ok(AIR), "nothing changed");
}

#[test]
fn a_block_placed_under_the_feet_mid_jump_is_stood_on() {
    let (world, mut hands, mut player, run) = scene(-MAX_PITCH);
    let ground = f64::from(i32::try_from(run.feet_y).expect("small"));
    tick(
        &world,
        &mut player,
        Walk {
            jump: true,
            ..Walk::default()
        },
    );
    let mut rose = false;
    for _ in 0..20 {
        if player.state().feet.y >= ground + 1.0 {
            rose = true;
            break;
        }
        tick(&world, &mut player, Walk::default());
    }
    assert!(rose, "a jump clears one block: {:?}", player.state());

    // The feet are above the cell now, touching or clear of it: accepted.
    let attempt = hands
        .act(&player, Action::Place, WorldTime(2))
        .expect("an answer");
    assert_eq!(attempt.refused, None, "{attempt:?}");
    let placed = attempt.edit.expect("an edit").position;
    assert_eq!(placed, BlockPos::new(run.x, run.feet_y, run.z));

    // The body comes down on the new block, never inside it.
    for _ in 0..40 {
        tick(&world, &mut player, Walk::default());
    }
    let state = player.state();
    assert!(state.grounded);
    assert_eq!(state.feet.y, ground + 1.0, "standing one block higher");
}

#[test]
fn breaking_the_block_underfoot_drops_the_player_into_the_hole() {
    let (world, mut hands, mut player, run) = scene(-MAX_PITCH);
    let ground = f64::from(i32::try_from(run.feet_y).expect("small"));
    let attempt = hands
        .act(&player, Action::Break, WorldTime(1))
        .expect("an answer");
    assert_eq!(
        attempt.edit.map(|e| e.position),
        Some(BlockPos::new(run.x, run.feet_y - 1, run.z))
    );
    for _ in 0..40 {
        tick(&world, &mut player, Walk::default());
    }
    let state = player.state();
    assert!(state.grounded);
    assert!(state.feet.y < ground, "fell into the hole: {state:?}");
}

#[test]
fn the_same_actions_on_the_same_seed_make_the_same_edits() {
    let run = || {
        let (world, mut hands, mut player, _) = scene(-45f64.to_radians());
        let mut edits = Vec::new();
        for (tick_index, action) in [Action::Break, Action::Break, Action::Place]
            .into_iter()
            .enumerate()
        {
            let now = WorldTime(tick_index as u64 + 1);
            edits.push(hands.act(&player, action, now).expect("an answer").edit);
            tick(
                &world,
                &mut player,
                Walk {
                    forward: 1.0,
                    ..Walk::default()
                },
            );
        }
        (edits, player.state())
    };
    let (first, a) = run();
    let (second, b) = run();
    assert_eq!(first, second);
    assert!(a.is_bit_identical(&b));
    assert!(first.iter().all(Option::is_some), "{first:?}");
}
