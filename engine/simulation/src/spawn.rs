//! Where a player can stand and walk: a run of columns found, never invented.
//!
//! Generator version 2's fixed-point value noise keeps neighboring generated
//! columns within the character's one-block step height. The search still
//! matters for edited terrain, unloaded columns, and bounded areas: a caller
//! that needs a safe path asks for a **run** of consecutive columns along one
//! cardinal direction, each at most one block above the last, with room above
//! it to stand in, ending at a wall or at the edge of the area searched.
//!
//! The search reads the terrain through the same [`VoxelSource`] the solver
//! collides against, so an edit the world has made is respected and an
//! unloaded column is the wall it is to physics. It scans a fixed spiral from
//! the centre of the area outward, so the same terrain always gives the same
//! run, and when no run qualifies it refuses rather than placing the player
//! somewhere that merely looks plausible.

use std::f64::consts::{FRAC_PI_2, PI};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::BlockPos;
use nexora_physics::voxel::VoxelSource;

/// Free cells a column needs above its top for a body to stand and step on
/// it: the 1.8 m character, lifted by its 1.0 m step.
pub const HEADROOM: i64 = 3;

/// A box of whole columns to search, and the vertical span to read them in.
///
/// The ceiling is part of the question, not a detail: a column's top is the
/// first solid cell **below the ceiling**, so a block hanging far above the
/// terrain — an edit in the sky, the world's top layer — is not mistaken for
/// the ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnArea {
    /// Lowest column x, inclusive.
    pub min_x: i64,
    /// Lowest column z, inclusive.
    pub min_z: i64,
    /// Highest column x, exclusive.
    pub max_x: i64,
    /// Highest column z, exclusive.
    pub max_z: i64,
    /// The lowest cell read, inclusive.
    pub floor_y: i64,
    /// The highest cell read, inclusive. Every cell between a column's top
    /// and this one is known to be empty.
    pub ceiling_y: i64,
}

impl ColumnArea {
    /// Whether the column at `x`, `z` is inside the area.
    #[must_use]
    pub const fn contains(&self, x: i64, z: i64) -> bool {
        x >= self.min_x && x < self.max_x && z >= self.min_z && z < self.max_z
    }
}

/// One of the four horizontal directions a run can face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinal {
    /// Towards `-Z`: yaw zero.
    NegZ,
    /// Towards `-X`: a quarter turn left.
    NegX,
    /// Towards `+Z`: a half turn.
    PosZ,
    /// Towards `+X`: a quarter turn right.
    PosX,
}

impl Cardinal {
    /// The order the search tries them in, from each column.
    pub const ORDER: [Self; 4] = [Self::NegZ, Self::NegX, Self::PosZ, Self::PosX];

    /// The yaw that looks this way, in `(-π, π]` (ADR-0029: zero looks down
    /// `-Z`, a quarter turn counter-clockwise from above looks down `-X`).
    #[must_use]
    pub fn yaw(self) -> f64 {
        match self {
            Self::NegZ => 0.0,
            Self::NegX => FRAC_PI_2,
            Self::PosZ => PI,
            Self::PosX => -FRAC_PI_2,
        }
    }

    /// One column this way, as `(dx, dz)`.
    #[must_use]
    pub const fn step(self) -> (i64, i64) {
        match self {
            Self::NegZ => (0, -1),
            Self::NegX => (-1, 0),
            Self::PosZ => (0, 1),
            Self::PosX => (1, 0),
        }
    }
}

/// A walkable run of columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    /// The first column's x, where a player spawns.
    pub x: i64,
    /// The first column's z.
    pub z: i64,
    /// Which way the run goes.
    pub facing: Cardinal,
    /// The first air cell above the first column: where the feet stand.
    pub feet_y: i64,
    /// Columns in the run, the first included.
    pub length: u32,
    /// Where the run ends, as the coordinate of the plane — `z` for a run
    /// along Z, `x` along X — that a body walking it meets with its leading
    /// face: the face of a column two or more blocks above the run's highest
    /// column, or the edge of the area searched.
    pub end_plane: i64,
}

impl Run {
    /// The run's columns, first to last.
    pub fn columns(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        let (dx, dz) = self.facing.step();
        (0..i64::from(self.length)).map(move |k| (self.x + dx * k, self.z + dz * k))
    }
}

/// The top of a column: the first solid cell scanning down from the area's
/// ceiling to its floor, or `None` when there is none.
pub(crate) fn column_top<S: VoxelSource + ?Sized>(
    terrain: &S,
    x: i64,
    z: i64,
    area: &ColumnArea,
) -> Option<i64> {
    (area.floor_y..=area.ceiling_y)
        .rev()
        .find(|&y| terrain.is_solid(BlockPos::new(x, y, z)))
}

/// Find the first run of at least `min_len` columns in `area`.
///
/// Columns are tried on Chebyshev rings from the centre of the area outward,
/// by ascending x and then z within a ring, and from each column the facings
/// in [`Cardinal::ORDER`]. A run extends while the next column is inside the
/// area, its top is at most one above the current one, and it has
/// [`HEADROOM`] free cells. It ends at the area's edge, or at a wall: a
/// column at least two above the **highest** column of the run so far, not
/// merely the last. A body that walks off a high column flies — air control
/// keeps its walking speed and nothing in the air takes it away — so after a
/// drop it can sail over a column two above the one it is crossing; only a
/// column two above anywhere it can be flying from is a wall it must meet.
/// A chain that breaks any other way — a column with no top in the span, too
/// little room, or a rise that is neither a step nor such a wall — is not a
/// run at all, so a run's [`Run::end_plane`] is always a wall or the edge.
///
/// # Errors
///
/// [`Recovery::Reject`] when the area is empty, `min_len` is zero, or no run
/// that long exists in it.
pub fn find_walkable_run<S: VoxelSource + ?Sized>(
    terrain: &S,
    area: &ColumnArea,
    min_len: u32,
) -> Result<Run> {
    if area.min_x >= area.max_x || area.min_z >= area.max_z || area.floor_y > area.ceiling_y {
        return Err(
            refused("the area to search holds no column").with_context("area", format!("{area:?}"))
        );
    }
    if min_len == 0 {
        return Err(refused("a run must be at least one column long"));
    }
    let centre_x = (area.min_x + area.max_x - 1).div_euclid(2);
    let centre_z = (area.min_z + area.max_z - 1).div_euclid(2);
    let reach = (centre_x - area.min_x)
        .max(area.max_x - 1 - centre_x)
        .max(centre_z - area.min_z)
        .max(area.max_z - 1 - centre_z);
    for radius in 0..=reach {
        for (x, z) in ring((centre_x, centre_z), radius) {
            if !area.contains(x, z) {
                continue;
            }
            let Some(top) = standable(terrain, x, z, area) else {
                continue;
            };
            for facing in Cardinal::ORDER {
                match walk_out(terrain, area, (x, z), top, facing) {
                    Some(run) if run.length >= min_len => return Ok(run),
                    _ => {}
                }
            }
        }
    }
    Err(refused("no walkable run that long exists in the area")
        .with_context("min_len", min_len.to_string())
        .with_context("area", format!("{area:?}")))
}

/// The columns at Chebyshev distance `radius` from `centre`, by ascending x
/// and then z.
///
/// On the ring's two x edges every z belongs to it; between them, only its
/// two z edges do, which is what stepping by `2 · radius` visits.
fn ring((centre_x, centre_z): (i64, i64), radius: i64) -> impl Iterator<Item = (i64, i64)> {
    (centre_x - radius..=centre_x + radius).flat_map(move |x| {
        let stride = if (x - centre_x).abs() == radius {
            1
        } else {
            2 * radius
        };
        (centre_z - radius..=centre_z + radius)
            .step_by(usize::try_from(stride).unwrap_or(1))
            .map(move |z| (x, z))
    })
}

/// A column's top, when a body can stand on it.
fn standable<S: VoxelSource + ?Sized>(
    terrain: &S,
    x: i64,
    z: i64,
    area: &ColumnArea,
) -> Option<i64> {
    column_top(terrain, x, z, area).filter(|top| area.ceiling_y - top >= HEADROOM)
}

/// Follow one facing from a standable column for as long as it is walkable.
fn walk_out<S: VoxelSource + ?Sized>(
    terrain: &S,
    area: &ColumnArea,
    (x, z): (i64, i64),
    top: i64,
    facing: Cardinal,
) -> Option<Run> {
    let (dx, dz) = facing.step();
    let (mut last_x, mut last_z, mut current, mut length) = (x, z, top, 1_u32);
    // The highest top of the run so far: the highest a body walking it can
    // be, falling, when it reaches the next column.
    let mut highest = top;
    loop {
        let (next_x, next_z) = (last_x + dx, last_z + dz);
        if !area.contains(next_x, next_z) {
            break;
        }
        match column_top(terrain, next_x, next_z, area) {
            // Two above anything the body can be flying from, lifted by its
            // whole step height: a wall it meets.
            Some(next) if next >= highest + 2 => break,
            // A step, or any drop, with room above it.
            Some(next) if next <= current + 1 && area.ceiling_y - next >= HEADROOM => {
                (last_x, last_z, current) = (next_x, next_z, next);
                highest = highest.max(next);
                length += 1;
            }
            _ => return None,
        }
    }
    let end_plane = match facing {
        Cardinal::NegZ => last_z,
        Cardinal::NegX => last_x,
        Cardinal::PosZ => last_z + 1,
        Cardinal::PosX => last_x + 1,
    };
    Some(Run {
        x,
        z,
        facing,
        feet_y: top + 1,
        length,
        end_plane,
    })
}

fn refused(message: &'static str) -> Error {
    Error::new(Domain::World, "player-spawn", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_physics::voxel::VoxelShape;

    /// Terrain whose column tops are given by a function.
    struct Heights<F: Fn(i64, i64) -> i64>(F);

    impl<F: Fn(i64, i64) -> i64> VoxelSource for Heights<F> {
        fn shape_at(&self, position: BlockPos) -> VoxelShape {
            if position.y <= (self.0)(position.x, position.z) {
                VoxelShape::SOLID
            } else {
                VoxelShape::Empty
            }
        }
    }

    fn area(min: i64, max: i64) -> ColumnArea {
        ColumnArea {
            min_x: min,
            min_z: min,
            max_x: max,
            max_z: max,
            floor_y: -16,
            ceiling_y: 32,
        }
    }

    /// A checkerboard of tops 0 and 2: from a low column every neighbour is
    /// a wall, and from a high one every neighbour is a drop into a low
    /// column walled in on every side. No run is longer than two.
    #[test]
    fn find_walkable_run_refuses_an_area_with_no_run() {
        let terrain = Heights(|x, z| 2 * (x + z).rem_euclid(2));
        let error = find_walkable_run(&terrain, &area(-8, 8), 3).expect_err("no run of three");
        assert_eq!(error.recovery(), Recovery::Reject);
        let two = find_walkable_run(&terrain, &area(-8, 8), 2).expect("a drop is walkable");
        assert_eq!(two.length, 2);
        assert!(
            find_walkable_run(&terrain, &area(0, 0), 1).is_err(),
            "an empty area"
        );
        assert!(find_walkable_run(&terrain, &area(-8, 8), 0).is_err());
    }

    /// Stairs climbing towards -Z one block a column, then a wall three
    /// blocks higher: every rule a run is held to is visible in one run.
    #[test]
    fn the_run_obeys_its_own_rules() {
        let top = |_x: i64, z: i64| match z {
            ..=-1 => 9,
            0..=6 => 6 - z,
            _ => 0,
        };
        let terrain = Heights(top);
        let search = ColumnArea {
            min_x: 0,
            min_z: -2,
            max_x: 1,
            max_z: 12,
            floor_y: -16,
            ceiling_y: 20,
        };
        let run = find_walkable_run(&terrain, &search, 4).expect("the stairs are a run");
        assert_eq!((run.x, run.z), (0, 4), "the centre of the strip");
        assert_eq!(run.facing, Cardinal::NegZ);
        assert_eq!(run.length, 5);
        let columns: Vec<(i64, i64)> = run.columns().collect();
        assert_eq!(columns.len(), run.length as usize);
        let mut previous = None;
        for &(x, z) in &columns {
            assert!(search.contains(x, z));
            let here = column_top(&terrain, x, z, &search).expect("a top");
            assert!(search.ceiling_y - here >= HEADROOM);
            if let Some(before) = previous {
                assert!(here <= before + 1, "{here} after {before}");
            }
            previous = Some(here);
        }
        // The run ends where the stairs jump by three, at that column's face.
        let (last_x, last_z) = *columns.last().expect("a column");
        assert_eq!(
            run.end_plane, last_z,
            "a -Z run ends on its last column's -Z face"
        );
        let beyond = column_top(&terrain, last_x, last_z - 1, &search).expect("a top");
        assert!(
            beyond >= previous.expect("a top") + 2,
            "a wall ends the run"
        );
        assert_eq!(run.feet_y, top(run.x, run.z) + 1);
        assert_eq!(find_walkable_run(&terrain, &search, 4).expect("again"), run);
    }

    /// After a drop, a column two above the one before it is not a wall: a
    /// body that walked off the high column is flying over it. Only a column
    /// two above the run's highest is. (`player.rs` shows the solver doing
    /// exactly that on the same shape.)
    #[test]
    fn after_a_drop_only_a_column_two_above_the_highest_is_a_wall() {
        // A one-row strip searched from x = 1. Column 0 reaches nearly to the
        // ceiling: a wall to anything beside it, and no place to stand.
        let strip = ColumnArea {
            min_x: 0,
            min_z: 0,
            max_x: 4,
            max_z: 1,
            floor_y: -16,
            ceiling_y: 40,
        };
        // From x = 1 along +X: 10, a drop to 2, then 4 — two above the drop,
        // eight below where a body walking off 10 is flying.
        let low_wall = Heights(|x, _| match x {
            ..=0 => 39,
            1 => 10,
            2 => 2,
            _ => 4,
        });
        let run = find_walkable_run(&low_wall, &strip, 2).expect("the far side is a run");
        assert_ne!(
            (run.x, run.facing),
            (1, Cardinal::PosX),
            "10, 2, then 4 is flown over, not met"
        );
        // From x = 3 along -X: 4, a drop to 2, then 10, which is two above
        // the highest (4): a wall, met on column 2's -X face.
        assert_eq!(
            (run.x, run.facing, run.length, run.end_plane),
            (3, Cardinal::NegX, 2, 2)
        );

        // The same drop, ended by 12: two above the highest, a wall.
        let high_wall = Heights(|x, _| match x {
            ..=0 => 39,
            1 => 10,
            2 => 2,
            _ => 12,
        });
        let run = find_walkable_run(&high_wall, &strip, 2).expect("10, 2, then 12 is a wall");
        assert_eq!(
            (run.x, run.facing, run.length, run.end_plane),
            (1, Cardinal::PosX, 2, 3)
        );
    }

    /// The same flat floor ends at the edge of the area when nothing walls it.
    #[test]
    fn a_run_on_a_flat_floor_ends_at_the_edge_of_the_area() {
        let terrain = Heights(|_, _| 0);
        let run = find_walkable_run(&terrain, &area(-4, 4), 1).expect("a run");
        assert_eq!((run.x, run.z), (-1, -1), "the centre of [-4, 4)");
        assert_eq!(run.facing, Cardinal::NegZ, "the first facing tried");
        assert_eq!(run.end_plane, -4, "the area's -Z edge");
        assert_eq!(run.length, 4);
    }

    /// A column whose top is too close to the ceiling is neither a step nor
    /// a wall, and the search will not call the chain through it a run.
    #[test]
    fn a_column_without_headroom_breaks_no_run_into_a_wall() {
        // A one-wide strip, so a run can only go along z; two columns one
        // block up, which are steps when there is room above them.
        let terrain = Heights(|_, z| if z == -3 || z == 2 { 30 } else { 29 });
        let strip = |ceiling_y| ColumnArea {
            min_x: 0,
            min_z: -4,
            max_x: 1,
            max_z: 4,
            floor_y: 0,
            ceiling_y,
        };
        assert!(
            find_walkable_run(&terrain, &strip(32), 3).is_err(),
            "two cells above the steps: no run through them"
        );
        let roomy = find_walkable_run(&terrain, &strip(33), 3).expect("three cells: steps");
        assert!(roomy.length >= 3);
    }

    #[test]
    fn the_four_facings_turn_the_way_their_yaws_say() {
        for facing in Cardinal::ORDER {
            let (dx, dz) = facing.step();
            let (sin, cos) = facing.yaw().sin_cos();
            // Forward at a yaw is (-sin, -cos).
            assert!((-sin - dx as f64).abs() < 1e-12, "{facing:?}");
            assert!((-cos - dz as f64).abs() < 1e-12, "{facing:?}");
        }
    }
}
