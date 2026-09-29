//! The camera against what its conventions promise (ADR-0029), checked by
//! projecting points, never by comparing a matrix with one typed by hand.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use nexora_camera::{Camera, CameraState, Projection, RenderOrigin, MAX_PITCH, REBASE_DISTANCE};
use nexora_foundation::spatial::{BlockPos, WorldPosition, MAX_BLOCK_COORD};

const NEAR: f64 = 0.1;
const FAR: f64 = 512.0;

fn perspective() -> Projection {
    Projection::perspective(70f64.to_radians(), NEAR, FAR).unwrap()
}

fn state_at(position: WorldPosition, yaw: f64, pitch: f64) -> CameraState {
    let mut camera = Camera::new(position, perspective()).unwrap();
    camera.set_orientation(yaw, pitch).unwrap();
    let origin = RenderOrigin::containing(position).unwrap();
    camera.sample(origin, 1600, 900).unwrap()
}

/// Project a render-space point to normalised device coordinates.
fn ndc(state: &CameraState, p: [f32; 3]) -> [f32; 3] {
    let c = state.view_projection.transform([p[0], p[1], p[2], 1.0]);
    [c[0] / c[3], c[1] / c[3], c[2] / c[3]]
}

fn ahead(state: &CameraState, distance: f32) -> [f32; 3] {
    [0, 1, 2].map(|axis| state.eye[axis] + state.forward[axis] * distance)
}

#[test]
fn reverse_z_puts_the_near_plane_at_one_and_the_far_plane_at_zero() {
    let state = state_at(WorldPosition::new(0.5, 70.0, 0.5), 0.3, -0.2);
    let near = ndc(&state, ahead(&state, NEAR as f32));
    let far = ndc(&state, ahead(&state, FAR as f32));
    assert!((near[2] - 1.0).abs() < 1e-5, "near depth {}", near[2]);
    assert!(far[2].abs() < 1e-5, "far depth {}", far[2]);
    // Straight ahead is the centre of the screen.
    for p in [near, far] {
        assert!(p[0].abs() < 1e-4 && p[1].abs() < 1e-4, "{p:?}");
    }
}

#[test]
fn depth_falls_as_distance_grows() {
    let state = state_at(WorldPosition::new(0.5, 70.0, 0.5), 1.0, 0.1);
    let mut previous = f32::INFINITY;
    // Just inside the near plane to just inside the far one: exactly on a
    // plane, f32 rounding may land a hair outside [0, 1].
    for distance in [0.11f32, 0.5, 1.0, 4.0, 16.0, 64.0, 256.0, 511.0] {
        let depth = ndc(&state, ahead(&state, distance))[2];
        assert!(depth < previous, "{distance}: {depth} not below {previous}");
        assert!((0.0..=1.0).contains(&depth), "{distance}: depth {depth}");
        previous = depth;
    }
}

#[test]
fn right_is_plus_x_and_up_is_plus_y_on_screen() {
    // Looking down -Z from the origin: +X in the world is right, +Y is up.
    let state = state_at(WorldPosition::new(0.0, 0.0, 0.0), 0.0, 0.0);
    let right = ndc(&state, [1.0, 0.0, -10.0]);
    let up = ndc(&state, [0.0, 1.0, -10.0]);
    assert!(right[0] > 0.0 && right[1].abs() < 1e-6);
    assert!(up[1] > 0.0 && up[0].abs() < 1e-6);
    // Behind the camera is not in front of it.
    let behind = state.view_projection.transform([0.0, 0.0, 10.0, 1.0]);
    assert!(behind[3] < 0.0);
}

#[test]
fn yaw_turns_counter_clockwise_seen_from_above_and_pitch_looks_up() {
    let mut camera = Camera::new(WorldPosition::ORIGIN, perspective()).unwrap();
    let close = |a: [f64; 3], b: [f64; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-12);
    assert!(close(camera.forward(), [0.0, 0.0, -1.0]));
    camera.set_orientation(FRAC_PI_2, 0.0).unwrap();
    assert!(close(camera.forward(), [-1.0, 0.0, 0.0]));
    camera.set_orientation(PI, 0.0).unwrap();
    assert!(close(camera.forward(), [0.0, 0.0, 1.0]));
    camera.set_orientation(0.0, FRAC_PI_4).unwrap();
    let f = camera.forward();
    assert!(f[1] > 0.7 && f[2] < -0.7);
}

#[test]
fn look_at_points_the_camera_at_its_target() {
    let origin = WorldPosition::new(10.5, 64.0, -3.25);
    let mut camera = Camera::new(origin, perspective()).unwrap();
    for target in [
        WorldPosition::new(40.0, 70.0, 12.0),
        WorldPosition::new(-5.0, 10.0, -90.0),
        WorldPosition::new(10.5, 64.0, 100.0),
    ] {
        camera.look_at(target).unwrap();
        let state = camera
            .sample(RenderOrigin::containing(origin).unwrap(), 800, 600)
            .unwrap();
        let p = state.origin.to_render(target).unwrap();
        let screen = ndc(&state, p);
        assert!(
            screen[0].abs() < 1e-4 && screen[1].abs() < 1e-4,
            "{target:?} lands at {screen:?}"
        );
    }
    assert!(camera.look_at(origin).is_err());
}

#[test]
fn pitch_is_clamped_and_yaw_wraps() {
    let mut camera = Camera::new(WorldPosition::ORIGIN, perspective()).unwrap();
    camera.set_orientation(3.0 * PI, 10.0).unwrap();
    assert!((camera.yaw() - PI).abs() < 1e-12);
    assert_eq!(camera.pitch(), MAX_PITCH);
    camera.set_orientation(-PI / 2.0 - 2.0 * PI, -10.0).unwrap();
    assert!((camera.yaw() + PI / 2.0).abs() < 1e-12);
    assert_eq!(camera.pitch(), -MAX_PITCH);
    // Straight up still has a right-hand direction: the state is finite.
    let state = camera
        .sample(
            RenderOrigin::containing(WorldPosition::ORIGIN).unwrap(),
            8,
            8,
        )
        .unwrap();
    assert!(state
        .view_projection
        .cols
        .iter()
        .flatten()
        .all(|v| v.is_finite()));
    assert!(camera.set_orientation(f64::NAN, 0.0).is_err());
}

#[test]
fn the_view_is_the_same_at_the_edge_of_the_world_as_at_its_centre() {
    // What floating origin buys: the matrices depend only on where the camera
    // is relative to the origin, so the same scene 2^40 blocks out produces
    // the same bits.
    let far = (MAX_BLOCK_COORD - 100_000) as f64;
    let offset = [0.375, 0.5, -0.625];
    let centre = WorldPosition::new(offset[0], 70.0 + offset[1], offset[2]);
    let edge = WorldPosition::new(far + offset[0], 70.0 + offset[1], -far + offset[2]);
    let a = state_at(centre, 0.7, -0.3);
    let b = state_at(edge, 0.7, -0.3);
    assert_eq!(a.view_projection, b.view_projection);
    assert_eq!(a.eye, b.eye);
    // And a block ten ahead of each lands on the same pixel.
    let block_a = BlockPos::new(3, 60, -8);
    let block_b = BlockPos::new(block_a.x + far as i64, block_a.y, block_a.z - far as i64);
    let pa = a.origin.offset_of(block_a).unwrap();
    let pb = b.origin.offset_of(block_b).unwrap();
    assert_eq!(ndc(&a, pa), ndc(&b, pb));
}

#[test]
fn a_camera_too_far_from_its_origin_is_refused_until_the_origin_follows() {
    let position = WorldPosition::new((REBASE_DISTANCE + 10) as f64, 0.0, 0.0);
    let camera = Camera::new(position, perspective()).unwrap();
    let stale = RenderOrigin::new(BlockPos::ORIGIN).unwrap();
    assert!(camera.sample(stale, 64, 64).is_err());
    let followed = stale.follow(position).unwrap();
    assert!(camera.sample(followed, 64, 64).is_ok());
}

#[test]
fn the_frustum_agrees_with_the_projection_point_by_point() {
    let state = state_at(WorldPosition::new(3.5, 80.0, -7.25), 2.2, -0.4);
    // A deterministic spray of points around the camera.
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    let mut inside = 0;
    for _ in 0..20_000 {
        let p = [
            f64::from(state.eye[0]) + (next() - 0.5) * 1200.0,
            f64::from(state.eye[1]) + (next() - 0.5) * 1200.0,
            f64::from(state.eye[2]) + (next() - 0.5) * 1200.0,
        ];
        let c = state
            .view_projection
            .transform([p[0] as f32, p[1] as f32, p[2] as f32, 1.0]);
        let w = f64::from(c[3]);
        let margins = [
            w + f64::from(c[0]),
            w - f64::from(c[0]),
            w + f64::from(c[1]),
            w - f64::from(c[1]),
            w - f64::from(c[2]),
            f64::from(c[2]),
        ];
        // Points within a hundredth of a block of a plane could go either
        // way under f32 rounding. Measured in blocks: clip-space margins
        // have no common scale (reverse-Z makes the far one tiny).
        if state
            .frustum
            .planes
            .iter()
            .any(|plane| plane.distance(p).abs() < 1e-2)
        {
            continue;
        }
        let clipped = margins.iter().all(|m| *m > 0.0);
        assert_eq!(state.frustum.contains(p), clipped, "at {p:?}");
        inside += usize::from(clipped);
    }
    // The spray actually tested both answers.
    assert!(inside > 100, "only {inside} points inside");
}

#[test]
fn culling_never_drops_a_box_with_a_visible_point() {
    // Brute force: a box is visible if any of its sampled points is inside.
    // The culler may keep boxes that are not; it may never drop one that is.
    let position = WorldPosition::new(0.5, 70.0, 0.5);
    let state = state_at(position, 0.9, -0.25);
    let (mut kept, mut dropped) = (0, 0);
    for cx in -12i64..=12 {
        for cz in -12i64..=12 {
            let min = BlockPos::new(cx * 16, 0, cz * 16);
            let max = BlockPos::new(cx * 16 + 16, 128, cz * 16 + 16);
            let sees = state.sees_blocks(min, max);
            let base = state.origin.block();
            let mut any = false;
            for i in 0..=8 {
                for j in 0..=8 {
                    for k in 0..=8 {
                        let p = [
                            (min.x - base.x) as f64 + 2.0 * f64::from(i),
                            (min.y - base.y) as f64 + 16.0 * f64::from(j),
                            (min.z - base.z) as f64 + 2.0 * f64::from(k),
                        ];
                        any |= state.frustum.contains(p);
                    }
                }
            }
            if any {
                assert!(
                    sees,
                    "column ({cx}, {cz}) has a visible point and was culled"
                );
            }
            if sees {
                kept += 1;
            } else {
                dropped += 1;
            }
        }
    }
    // A 70° view keeps a wedge of the 625 columns, not all and not none.
    assert!(kept > 20 && dropped > 300, "kept {kept}, dropped {dropped}");
}

#[test]
fn orthographic_maps_near_and_far_to_one_and_zero_without_perspective() {
    let projection = Projection::orthographic(32.0, 0.0, 100.0).unwrap();
    let camera = Camera::new(WorldPosition::ORIGIN, projection).unwrap();
    let state = camera
        .sample(RenderOrigin::new(BlockPos::ORIGIN).unwrap(), 200, 100)
        .unwrap();
    assert_eq!(ndc(&state, [0.0, 0.0, 0.0])[2], 1.0);
    assert!(ndc(&state, [0.0, 0.0, -100.0])[2].abs() < 1e-6);
    // Size does not change with distance: the top edge is 16 blocks up at
    // every depth.
    for z in [-1.0f32, -50.0, -99.0] {
        assert!((ndc(&state, [0.0, 16.0, z])[1] - 1.0).abs() < 1e-6);
    }
}

#[test]
fn projections_out_of_range_are_refused() {
    for (fov, near, far) in [
        (0.0, 0.1, 10.0),
        (PI, 0.1, 10.0),
        (1.0, 0.0, 10.0),
        (1.0, 0.1, 0.1),
        (1.0, f64::NAN, 10.0),
        (1.0, 0.1, f64::INFINITY),
    ] {
        assert!(
            Projection::perspective(fov, near, far).is_err(),
            "{fov} {near} {far}"
        );
    }
    assert!(Projection::orthographic(0.0, 0.0, 1.0).is_err());
    assert!(Projection::orthographic(1.0, -1.0, 1.0).is_err());
    assert!(Camera::new(WorldPosition::new(f64::NAN, 0.0, 0.0), perspective()).is_err());
    let camera = Camera::new(WorldPosition::ORIGIN, perspective()).unwrap();
    let origin = RenderOrigin::new(BlockPos::ORIGIN).unwrap();
    assert!(camera.sample(origin, 0, 10).is_err());
}
