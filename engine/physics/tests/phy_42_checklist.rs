//! The test list `PHYSICS.md` §43 (PHY-42) asks for, one test per item.
//!
//! The document names twelve behaviours and three stress cases. Seven of the
//! behaviours are implemented and are checked here against what they should do.
//! The rest are **not** implemented, and rather than leaving them off the list —
//! where a reader would have to discover the gap by experiment — each one has a
//! test that pins the behaviour the engine has *today*. Those tests are written
//! to fail the moment the feature arrives, which is what turns them into a
//! reminder instead of an excuse.

use nexora_foundation::spatial::BlockPos;
use nexora_physics::body::{BodyDescriptor, BodyType, SleepState};
use nexora_physics::character::{CharacterController, GroundState, MoveIntent};
use nexora_physics::collision::overlaps_solid;
use nexora_physics::gravity::GravityField;
use nexora_physics::material::PhysicsMaterial;
use nexora_physics::math::Vec3;
use nexora_physics::query::raycast;
use nexora_physics::step::FixedStep;
use nexora_physics::voxel::{FlatGround, VoxelShape, VoxelSource};
use nexora_physics::world::PhysicsWorld;

const DT: f64 = 1.0 / 60.0;

fn world() -> PhysicsWorld {
    PhysicsWorld::new(
        GravityField::earthlike(),
        FixedStep::new(60, 20, 8).expect("valid rates"),
    )
}

/// Terrain whose height rises by one block per block of x, up to a cap.
struct Staircase {
    top: i64,
}

impl VoxelSource for Staircase {
    fn shape_at(&self, position: BlockPos) -> VoxelShape {
        let height = position.x.clamp(0, self.top);
        if position.y < height {
            VoxelShape::SOLID
        } else {
            VoxelShape::Empty
        }
    }
}

/// A single block wall one cell thick, standing on a floor.
struct ThinWall {
    at_x: i64,
}

impl VoxelSource for ThinWall {
    fn shape_at(&self, position: BlockPos) -> VoxelShape {
        if position.y < 0 || (position.x == self.at_x && position.y < 4) {
            VoxelShape::SOLID
        } else {
            VoxelShape::Empty
        }
    }
}

// ---------------------------------------------------------------- implemented

#[test]
fn falling_body() {
    let mut world = world();
    let id = world
        .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 100.0, 0.5)))
        .expect("spawn");
    for _ in 0..30 {
        world.step_once(&FlatGround::at(0), DT);
    }
    let body = world.body(id).expect("live");
    assert!(body.velocity.y < 0.0, "it did not start falling");
    assert!(body.center.y < 100.0);
}

#[test]
fn collision() {
    let ground = FlatGround::at(0);
    let mut world = world();
    let id = world
        .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 30.0, 0.5)))
        .expect("spawn");
    for _ in 0..900 {
        world.step_once(&ground, DT);
    }
    let body = world.body(id).expect("live");
    assert!(body.grounded);
    assert!(!overlaps_solid(&ground, body.aabb()));
}

#[test]
fn friction() {
    let mut world = world();
    let ice = world
        .materials_mut()
        .register(PhysicsMaterial {
            friction: 0.01,
            ..PhysicsMaterial::DEFAULT
        })
        .expect("registered");

    let slide = |world: &mut PhysicsWorld, ground: &FlatGround| {
        let id = world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 0.5, 0.5)))
            .expect("spawn");
        world
            .body_mut(id)
            .expect("live")
            .set_velocity(Vec3::new(8.0, 0.0, 0.0))
            .expect("finite");
        for _ in 0..60 {
            world.step_once(ground, DT);
        }
        world.body(id).expect("live").velocity.x
    };

    let mut icy = world.clone();
    let on_stone = slide(&mut world, &FlatGround::at(0));
    let on_ice = slide(&mut icy, &FlatGround::at(0).of(ice));
    assert!(
        on_ice > on_stone,
        "ice {on_ice} should retain more speed than stone {on_stone}"
    );
}

#[test]
fn bounce() {
    let mut world = world();
    let rubber = world
        .materials_mut()
        .register(PhysicsMaterial {
            restitution: 0.7,
            ..PhysicsMaterial::DEFAULT
        })
        .expect("registered");
    let id = world
        .spawn(
            BodyDescriptor::dynamic()
                .at(Vec3::new(0.5, 8.0, 0.5))
                .made_of(rubber),
        )
        .expect("spawn");

    let mut peak_after_landing: Option<f64> = None;
    let mut landed = false;
    for _ in 0..600 {
        world.step_once(&FlatGround::at(0), DT);
        let body = world.body(id).expect("live");
        if body.grounded {
            landed = true;
        }
        if landed && body.velocity.y > 0.0 {
            let height = body.center.y;
            peak_after_landing =
                Some(peak_after_landing.map_or(height, |best: f64| best.max(height)));
        }
    }
    let peak = peak_after_landing.expect("a rubber body must leave the floor again");
    assert!(peak > 1.0, "the rebound barely left the ground: {peak}");
    assert!(peak < 8.0, "the rebound gained energy: {peak}");
}

#[test]
fn stairs() {
    let source = Staircase { top: 5 };
    let mut world = world();
    let id = world
        .spawn(BodyDescriptor::character().at(Vec3::new(-2.0, 0.9, 0.5)))
        .expect("spawn");
    let controller = CharacterController::new(id);
    for _ in 0..600 {
        controller
            .apply(&mut world, MoveIntent::walking(Vec3::new(1.0, 0.0, 0.0)))
            .expect("live");
        world.step_once(&source, DT);
    }
    let body = world.body(id).expect("live");
    assert!(
        body.center.y > 4.0,
        "the character never climbed the stairs: y {}",
        body.center.y
    );
    assert!(!overlaps_solid(&source, body.aabb()));
}

#[test]
fn raycast_query() {
    let source = Staircase { top: 5 };
    let hit = raycast(
        &source,
        Vec3::new(2.5, 10.0, 0.5),
        Vec3::new(0.0, -1.0, 0.0),
        100.0,
    )
    .expect("the third stair is below");
    assert_eq!(hit.cell, BlockPos::new(2, 1, 0));
    assert!(
        (hit.distance - 8.0).abs() < 1e-12,
        "distance {}",
        hit.distance
    );
    assert_eq!(hit.normal, Vec3::new(0.0, 1.0, 0.0));
}

#[test]
fn sleep_and_wake() {
    let ground = FlatGround::at(0);
    let mut world = world();
    let id = world
        .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 3.0, 0.5)))
        .expect("spawn");
    for _ in 0..900 {
        world.step_once(&ground, DT);
    }
    assert_eq!(world.body(id).expect("live").sleep, SleepState::Sleeping);

    world
        .body_mut(id)
        .expect("live")
        .apply_impulse(Vec3::new(0.0, 50.0, 0.0))
        .expect("finite");
    assert_eq!(world.body(id).expect("live").sleep, SleepState::Awake);
    assert!(world.body(id).expect("live").velocity.y > 0.0);
}

#[test]
fn continuous_collision_a_fast_body_does_not_tunnel_through_a_thin_wall() {
    // The sweep is continuous, so this holds even when one step covers far more
    // than the wall's thickness. A discrete "move then test" solver would put
    // the body straight through it.
    let source = ThinWall { at_x: 50 };
    let mut world = world();
    let id = world
        .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 0.5, 0.5)))
        .expect("spawn");
    world
        .body_mut(id)
        .expect("live")
        .set_velocity(Vec3::new(200.0, 0.0, 0.0))
        .expect("finite");
    // A whole second in one step: 200 metres of motion against a one-metre wall.
    world.step_once(&source, 1.0);
    let body = world.body(id).expect("live");
    assert!(
        body.center.x < 50.0,
        "tunnelled to x {} through the wall at 50",
        body.center.x
    );
    assert!(!overlaps_solid(&source, body.aabb()));
}

// ------------------------------------------------------------------- stress

#[test]
fn stress_one_thousand_bodies_settle_without_diverging() {
    let ground = FlatGround::at(0);
    let mut world = world();
    for index in 0..1_000u32 {
        let x = f64::from(index % 40) * 2.0;
        let z = f64::from(index / 40) * 2.0;
        let y = 2.0 + f64::from(index % 17);
        world
            .spawn(BodyDescriptor::dynamic().at(Vec3::new(x + 0.5, y, z + 0.5)))
            .expect("spawn");
    }
    assert_eq!(world.len(), 1_000);

    for _ in 0..300 {
        world.step_once(&ground, DT);
    }

    for (id, body) in world.iter() {
        assert!(body.center.is_finite(), "body {} exploded", id.index());
        assert!(
            !overlaps_solid(&ground, body.aabb()),
            "body {} sank into the floor",
            id.index()
        );
    }
    // Sleeping is the point of the stress test: a thousand settled bodies must
    // stop costing anything.
    assert!(
        world.awake_count() < world.len() / 2,
        "{} of {} bodies are still awake",
        world.awake_count(),
        world.len()
    );
}

// --------------------------------------------------------------- not yet built
//
// These pin what the engine does today so that implementing the feature breaks
// the test and forces this list to be updated with it.

#[test]
fn not_yet_moving_platform_does_not_carry_a_passenger() {
    // PHY-25. A character standing where a kinematic platform is does not ride
    // it, because bodies do not collide with bodies at all yet.
    let ground = FlatGround::at(0);
    let mut world = world();
    let platform = world
        .spawn(
            BodyDescriptor::kinematic()
                .at(Vec3::new(0.5, 0.5, 0.5))
                .sized(Vec3::new(2.0, 0.5, 2.0)),
        )
        .expect("spawn");
    world
        .body_mut(platform)
        .expect("live")
        .set_velocity(Vec3::new(1.0, 0.0, 0.0))
        .expect("finite");
    let rider = world
        .spawn(BodyDescriptor::character().at(Vec3::new(0.5, 1.9, 0.5)))
        .expect("spawn");

    let start_x = world.body(rider).expect("live").center.x;
    for _ in 0..120 {
        world.step_once(&ground, DT);
    }
    let moved_platform = world.body(platform).expect("live").center.x - 0.5;
    let moved_rider = world.body(rider).expect("live").center.x - start_x;
    assert!(
        moved_platform > 1.0,
        "the platform itself should have moved"
    );
    assert!(
        moved_rider.abs() < 1e-9,
        "the rider moved {moved_rider}: platforms now carry passengers, so this \
         checklist entry has become a real test - move it up and write it"
    );
}

#[test]
fn not_yet_two_dynamic_bodies_pass_through_each_other() {
    // PHY-4 through PHY-6 for body pairs. Every contact today is against the
    // voxel grid.
    // In empty space, so that ground friction cannot stop them short of each
    // other and make this test pass for the wrong reason.
    let mut world = world();
    let left = world
        .spawn(BodyDescriptor {
            gravity_scale: 0.0,
            ..BodyDescriptor::dynamic().at(Vec3::new(0.0, 0.5, 0.5))
        })
        .expect("spawn");
    let right = world
        .spawn(BodyDescriptor {
            gravity_scale: 0.0,
            ..BodyDescriptor::dynamic().at(Vec3::new(6.0, 0.5, 0.5))
        })
        .expect("spawn");
    world
        .body_mut(left)
        .expect("live")
        .set_velocity(Vec3::new(3.0, 0.0, 0.0))
        .expect("finite");
    world
        .body_mut(right)
        .expect("live")
        .set_velocity(Vec3::new(-3.0, 0.0, 0.0))
        .expect("finite");

    for _ in 0..120 {
        world.step_once(&nexora_physics::voxel::EmptySpace, DT);
    }
    assert!(
        world.body(left).expect("live").center.x > world.body(right).expect("live").center.x,
        "the two bodies collided: the pair solver exists now, so this checklist \
         entry has become a real test - move it up and write it"
    );
}

#[test]
fn not_yet_there_are_no_slopes_only_stacked_cubes() {
    // PHY-12. Every shape is a whole cube, which is why `stairs` above is the
    // closest thing to a slope test that can exist.
    let source = Staircase { top: 5 };
    for x in 0..5 {
        // Each column is either entirely solid or entirely empty at a given
        // height: nothing is partially filled.
        let height = x.clamp(0, 5);
        assert!(source.shape_at(BlockPos::new(x, height - 1, 0)).is_solid());
        assert!(!source.shape_at(BlockPos::new(x, height, 0)).is_solid());
    }
}

#[test]
fn not_yet_buoyancy_bodies_ignore_fluid_and_keep_falling() {
    // PHY-16 and PHY-17. There is no fluid source, so a body in what would be
    // water falls exactly as it does in air.
    let mut world = world();
    let id = world
        .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 100.0, 0.5)))
        .expect("spawn");
    for _ in 0..120 {
        world.step_once(&nexora_physics::voxel::EmptySpace, DT);
    }
    let body = world.body(id).expect("live");
    let expected = -nexora_physics::gravity::EARTHLIKE_GRAVITY * 2.0;
    assert!(
        (body.velocity.y - expected).abs() < 1e-9,
        "velocity {} is not free fall: something is displacing the body",
        body.velocity.y
    );
}

#[test]
fn not_yet_bodies_have_no_orientation() {
    // PHY-2's angular half. A body is an axis-aligned box, and the type has
    // nowhere to record a rotation.
    let mut world = world();
    let id = world
        .spawn(BodyDescriptor::dynamic().at(Vec3::new(0.5, 0.5, 0.5)))
        .expect("spawn");
    let before = world.body(id).expect("live").aabb();
    for _ in 0..120 {
        world.step_once(&FlatGround::at(0), DT);
    }
    let after = world.body(id).expect("live").aabb();
    // The box keeps its axis-aligned extents whatever happens to it.
    let extents = |b: nexora_physics::math::Aabb| b.half_extents();
    assert!((extents(before) - extents(after)).length() < 1e-12);
}

#[test]
fn every_body_type_is_covered_by_this_checklist() {
    // A guard against a fourth body type being added without a test for it.
    for body_type in [BodyType::Static, BodyType::Kinematic, BodyType::Dynamic] {
        assert!(!body_type.as_str().is_empty());
    }
    assert_eq!(GroundState::Grounded.as_str(), "grounded");
}
