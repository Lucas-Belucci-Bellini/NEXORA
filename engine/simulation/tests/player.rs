//! The player against real generated worlds, through `WorldVoxels` (ADR-0035).
//!
//! The unit tests in `player.rs` pin the arithmetic on fixtures; these pin
//! what only the generator can show: that a run found in generated terrain
//! is really walkable, that the terrain can be walked across at all
//! (ADR-0039), that a resting player on it is a fixed point of the
//! solver to the bit, that an unloaded column is the wall the policy says it
//! is, and that the player reads the world and never writes it.

use nexora_foundation::spatial::ChunkCoord;
use nexora_foundation::time::{CalendarConfig, DEFAULT_TICKS_PER_SECOND};
use nexora_runtime::input::{ButtonCode, DeviceId, DeviceKind, InputFrame, Signal};
use nexora_simulation::{
    find_walkable_run, Cardinal, ColumnArea, Controls, Player, Run, Walk, WorldVoxels,
};
use nexora_world::world::{World, WorldDescriptor};

/// The client's default seed.
const CLIENT_SEED: u64 = 0x4E58_4F52;

/// The client's run: one second of W never reaches its end.
const CLIENT_RUN: u32 = 5;

/// Chunk columns `-radius..=radius` on both axes, generated.
fn world(seed: u64, radius: i64) -> World {
    let mut world = World::create(
        WorldDescriptor::new("player-test", seed).expect("a descriptor"),
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

/// The columns of chunks `-radius..=radius`, read in the band the client
/// draws: eight blocks below the lowest surface to eight above the highest.
fn area(world: &World, radius: i64) -> ColumnArea {
    let size = i64::from(world.descriptor().shape.size_x());
    let (min, max) = (-radius * size, (radius + 1) * size);
    let (mut low, mut high) = (i64::MAX, i64::MIN);
    for x in min..max {
        for z in min..max {
            let height = world.surface_height(x, z);
            low = low.min(height);
            high = high.max(height);
        }
    }
    ColumnArea {
        min_x: min,
        min_z: min,
        max_x: max,
        max_z: max,
        floor_y: low - 8,
        ceiling_y: high + 7,
    }
}

fn spawn(world: &World, run: &Run) -> Player {
    Player::spawn(&WorldVoxels::new(world), run, 0.0, DEFAULT_TICKS_PER_SECOND)
        .unwrap_or_else(|error| panic!("spawn on {run:?}: {error}"))
}

const KEYBOARD: DeviceId = DeviceId::new(DeviceKind::Keyboard, 0);

/// Two hundred frames of a player's keys: attach, W, then a left turn, a
/// jump and a look up layered on and taken off at their own times.
fn scripted_frames() -> Vec<InputFrame> {
    let key = |usage: u16, pressed: bool| Signal::button(KEYBOARD, ButtonCode(usage), pressed);
    (0..200)
        .map(|frame| {
            let mut signals = InputFrame::new();
            match frame {
                0 => {
                    signals.push(Signal::Attached(KEYBOARD));
                    signals.push(key(0x1A, true)); // W
                }
                30 => signals.push(key(0x50, true)), // Left arrow
                50 => signals.push(key(0x50, false)),
                60 => signals.push(key(0x2C, true)), // Space, tapped
                61 => signals.push(key(0x2C, false)),
                90 => signals.push(key(0x52, true)), // Up arrow
                110 => signals.push(key(0x52, false)),
                150 => signals.push(key(0x1A, false)),
                _ => {}
            }
            signals
        })
        .collect()
}

/// What `tests/client.rs`'s exact "no key, no movement" rests on.
#[test]
fn a_player_on_generated_terrain_stands_still_without_intent() {
    for seed in 1..=8 {
        let world = world(seed, 1);
        let terrain = WorldVoxels::new(&world);
        let run = find_walkable_run(&terrain, &area(&world, 1), 4)
            .unwrap_or_else(|error| panic!("seed {seed}: {error}"));
        let mut player = spawn(&world, &run);
        let (state, eye) = (player.state(), player.eye());
        assert!(state.grounded, "seed {seed}");
        assert_eq!(state.velocity, [0.0; 3], "seed {seed}");
        for tick in 0..1_000 {
            let outcome = player
                .tick(&terrain, Walk::default())
                .unwrap_or_else(|error| panic!("seed {seed}, tick {tick}: {error}"));
            assert_eq!(outcome.depenetrated, 0);
            assert!(
                player.state().is_bit_identical(&state),
                "seed {seed}, tick {tick}: {:?} after {state:?}",
                player.state()
            );
        }
        assert_eq!(player.eye(), eye, "seed {seed}");
        assert_eq!(player.eye().position.y.to_bits(), eye.position.y.to_bits());
    }
}

/// The unloaded-is-solid policy (`terrain.rs`), through the real adapter: a
/// player walking and jumping into the edge of the only loaded chunk stays in
/// it, is never buried and never ends a tick inside terrain.
#[test]
fn the_player_cannot_leave_a_lone_generated_column() {
    let world = world(0x5EED, 0);
    let terrain = WorldVoxels::new(&world);
    let size = i64::from(world.descriptor().shape.size_x());
    let middle = size / 2;
    let edges = [
        (Cardinal::NegZ, middle, 0, 0),
        (Cardinal::NegX, 0, middle, 0),
        (Cardinal::PosZ, middle, size - 1, size),
        (Cardinal::PosX, size - 1, middle, size),
    ];
    let run_and_jump = Walk {
        forward: 1.0,
        jump: true,
        ..Walk::default()
    };
    for (facing, x, z, end_plane) in edges {
        let run = Run {
            x,
            z,
            facing,
            feet_y: world.surface_height(x, z) + 1,
            length: 1,
            end_plane,
        };
        let mut player = spawn(&world, &run);
        for tick in 0..200 {
            player
                .tick(&terrain, run_and_jump)
                .unwrap_or_else(|error| panic!("{facing:?}, tick {tick}: {error}"));
            let feet = player.state().feet;
            for (axis, centre) in [("x", feet.x), ("z", feet.z)] {
                assert!(
                    centre - 0.3 >= -1e-9 && centre + 0.3 <= size as f64 + 1e-9,
                    "{facing:?}, tick {tick}: the box left the chunk on {axis} at {centre}"
                );
            }
        }
    }
}

#[test]
fn the_player_reads_the_world_and_never_writes_it() {
    let world = world(42, 1);
    let revision = world.revision();
    let terrain = WorldVoxels::new(&world);
    let run = find_walkable_run(&terrain, &area(&world, 1), 4).expect("a run");
    let mut player = spawn(&world, &run);
    let mut controls = Controls::new().expect("controls");
    for frame in scripted_frames() {
        player
            .tick(&terrain, controls.sample(&frame).walk())
            .expect("a tick");
    }
    assert_eq!(world.revision(), revision);
}

/// The client's search, on twenty seeds: the same run every time, inside
/// the drawn columns, obeying its own rules — and walked, not just looked at.
#[test]
fn the_walkable_run_is_deterministic_and_really_walkable() {
    for seed in 1..=20 {
        // The drawn columns and a ring, as the client generates them.
        let world = world(seed, 2);
        let terrain = WorldVoxels::new(&world);
        let drawn = area(&world, 1);
        let run = find_walkable_run(&terrain, &drawn, CLIENT_RUN)
            .unwrap_or_else(|error| panic!("seed {seed}: {error}"));
        assert_eq!(
            find_walkable_run(&terrain, &drawn, CLIENT_RUN).expect("again"),
            run,
            "seed {seed}"
        );
        assert!(run.length >= CLIENT_RUN, "seed {seed}");
        let mut previous: Option<i64> = None;
        for (x, z) in run.columns() {
            assert!(drawn.contains(x, z), "seed {seed}: ({x}, {z}) is not drawn");
            let top = world.surface_height(x, z);
            assert!(drawn.ceiling_y - top >= 3, "seed {seed}");
            if let Some(before) = previous {
                assert!(top <= before + 1, "seed {seed}: {top} after {before}");
            }
            previous = Some(top);
        }
        assert_eq!(run.feet_y, world.surface_height(run.x, run.z) + 1);

        // Walk it: the player reaches the last column.
        let mut player = spawn(&world, &run);
        let start = player.state().feet;
        let (dx, dz) = run.facing.step();
        let ticks = (f64::from(run.length) / 0.19).ceil() as u32 + 20;
        let forward = Walk {
            forward: 1.0,
            ..Walk::default()
        };
        for tick in 0..ticks {
            player
                .tick(&terrain, forward)
                .unwrap_or_else(|error| panic!("seed {seed}, tick {tick}: {error}"));
        }
        let end = player.state().feet;
        let walked = (end.x - start.x) * dx as f64 + (end.z - start.z) * dz as f64;
        assert!(
            walked >= f64::from(run.length - 1),
            "seed {seed}: walked {walked} of a run of {}",
            run.length
        );
    }
}

/// The client refuses to start without a run of five in the drawn columns;
/// on these seeds it never has to.
#[test]
fn a_run_of_five_exists_on_every_calibration_seed() {
    for seed in std::iter::once(CLIENT_SEED).chain(1..=20) {
        let world = world(seed, 1);
        let terrain = WorldVoxels::new(&world);
        let run = find_walkable_run(&terrain, &area(&world, 1), CLIENT_RUN);
        assert!(run.is_ok(), "seed {seed}: {run:?}");
    }
}

/// Replay from raw input, on one build and target: two worlds from one seed
/// and the same frames of HID usages give the same player to the bit.
#[test]
fn the_same_input_frames_give_the_same_player_bit_for_bit() {
    let (first_world, second_world) = (world(7, 1), world(7, 1));
    let (first_terrain, second_terrain) = (
        WorldVoxels::new(&first_world),
        WorldVoxels::new(&second_world),
    );
    let run = find_walkable_run(&first_terrain, &area(&first_world, 1), 4).expect("a run");
    assert_eq!(
        find_walkable_run(&second_terrain, &area(&second_world, 1), 4).expect("a run"),
        run
    );
    let (mut first, mut second) = (spawn(&first_world, &run), spawn(&second_world, &run));
    let (mut first_keys, mut second_keys) = (
        Controls::new().expect("controls"),
        Controls::new().expect("controls"),
    );
    let spawned = first.state().feet;
    let mut moved = false;
    for (tick, frame) in scripted_frames().iter().enumerate() {
        first
            .tick(&first_terrain, first_keys.sample(frame).walk())
            .expect("a tick");
        second
            .tick(&second_terrain, second_keys.sample(frame).walk())
            .expect("a tick");
        assert!(
            first.state().is_bit_identical(&second.state()),
            "tick {tick}: {:?} against {:?}",
            first.state(),
            second.state()
        );
        moved |= first.state().feet != spawned;
    }
    assert!(moved, "the script must actually move the player");
}

/// DEBT-0054, the Phase 2 exit's *navegável* (ADR-0039): on generator
/// version 2 a player walks across generated terrain in a straight line, in
/// every direction, through chunk borders, up and down its slopes, and
/// nothing stops it short of the end of what is loaded. On version 1's
/// pillars the same walk stopped within a block or two.
#[test]
fn a_player_walks_across_generated_terrain_in_every_direction() {
    const TICKS: u32 = 200;
    for seed in [CLIENT_SEED, 1, 28, 987_654_321] {
        // Columns -64..96 on both axes are generated; the walk starts at 16,
        // so 200 ticks (about 42 blocks) stay inside them every way.
        let world = world(seed, 2);
        let terrain = WorldVoxels::new(&world);
        for facing in [
            Cardinal::NegZ,
            Cardinal::PosX,
            Cardinal::PosZ,
            Cardinal::NegX,
        ] {
            let run = Run {
                x: 16,
                z: 16,
                facing,
                feet_y: world.surface_height(16, 16) + 1,
                length: 1,
                end_plane: 16,
            };
            let mut player = spawn(&world, &run);
            let start = player.state();
            let mut climbed = 0.0f64;
            let mut last = start;
            for tick in 0..TICKS {
                let outcome = player
                    .tick(
                        &terrain,
                        Walk {
                            forward: 1.0,
                            ..Walk::default()
                        },
                    )
                    .unwrap_or_else(|error| panic!("seed {seed} {facing:?} tick {tick}: {error}"));
                assert_eq!(
                    outcome.depenetrated, 0,
                    "seed {seed} {facing:?} tick {tick}"
                );
                let now = player.state();
                climbed += (now.feet.y - last.feet.y).abs();
                last = now;
            }
            let (dx, dz) = facing.step();
            let forward =
                (last.feet.x - start.feet.x) * dx as f64 + (last.feet.z - start.feet.z) * dz as f64;
            assert!(
                forward >= 35.0,
                "seed {seed} {facing:?}: walked only {forward:.2} blocks in {TICKS} ticks"
            );
            assert!(
                climbed >= 1.0,
                "seed {seed} {facing:?}: the ground was flat"
            );
        }
    }
}
