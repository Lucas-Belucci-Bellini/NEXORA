//! The floating render origin (RENDER-6).
//!
//! The world is addressed in `i64` blocks and `f64` positions, out to 2^40
//! blocks (`MAX_BLOCK_COORD`). A GPU works in `f32`, whose 24-bit mantissa
//! cannot tell two neighbouring millimetres apart beyond 16 km. So nothing is
//! sent to the GPU in world coordinates. Everything is sent relative to a
//! **render origin**: an integer block, kept near the camera.
//!
//! Two properties make that safe, and both are tested:
//!
//! - **Block offsets are exact.** An offset between two blocks is an integer
//!   difference, and every integer below 2^24 is an `f32`. A mesh built in a
//!   section's local space and placed by [`RenderOrigin::offset_of`] lands on
//!   the same texel at any distance from the world origin.
//! - **Positions lose only what `f32` must.** [`RenderOrigin::to_render`]
//!   subtracts in `f64`, where subtracting an integer that close to a value is
//!   exact, and rounds to `f32` once. Within [`REBASE_DISTANCE`] of the origin
//!   the error is below half a millimetre, at 2^40 as at 0.
//!
//! Rebasing moves the origin, never the world: `SPATIAL AND COORDINATE
//! SYSTEM.md` requires that rebasing never changes a logical coordinate, and
//! nothing here stores one.

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{BlockPos, WorldPosition};

/// How far, in blocks along any axis, the camera may drift from the render
/// origin before the origin should move to it.
///
/// At 4,096 an `f32` step is 2^-11 of a block, under half a millimetre at a
/// metre a block. The renderer rebases before that is exceeded, so everything
/// near the camera, where precision shows, stays below it.
pub const REBASE_DISTANCE: i64 = 4096;

/// Offsets at or beyond this magnitude are not exact in `f32`.
pub const EXACT_OFFSET: i64 = 1 << 24;

/// A block the renderer treats as `(0, 0, 0)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderOrigin {
    block: BlockPos,
}

impl RenderOrigin {
    /// An origin at `block`.
    ///
    /// # Errors
    ///
    /// The block is beyond the world's coordinate limits.
    pub fn new(block: BlockPos) -> Result<Self> {
        Ok(Self {
            block: block.require_within_limits()?,
        })
    }

    /// An origin at the block containing `position`.
    ///
    /// # Errors
    ///
    /// The position is not finite, or beyond the world's limits.
    pub fn containing(position: WorldPosition) -> Result<Self> {
        Self::new(position.require_finite()?.to_block_pos())
    }

    /// The origin block.
    #[must_use]
    pub const fn block(&self) -> BlockPos {
        self.block
    }

    /// Where `block`'s minimum corner is in render space. Exact.
    ///
    /// # Errors
    ///
    /// The block is so far from the origin that the offset is not exact in
    /// `f32`: the caller is placing something it should not be drawing, or
    /// the origin has not been rebased.
    pub fn offset_of(&self, block: BlockPos) -> Result<[f32; 3]> {
        let deltas = [
            block.x - self.block.x,
            block.y - self.block.y,
            block.z - self.block.z,
        ];
        if deltas.iter().any(|delta| delta.abs() >= EXACT_OFFSET) {
            return Err(Error::new(
                Domain::Spatial,
                "render-origin",
                "a block is too far from the render origin to place exactly",
            )
            .with_recovery(Recovery::Reject)
            .with_context("origin", format!("{:?}", self.block))
            .with_context("block", format!("{block:?}")));
        }
        Ok(deltas.map(|delta| delta as f32))
    }

    /// `position` in render space.
    ///
    /// # Errors
    ///
    /// The position is not finite.
    pub fn to_render(&self, position: WorldPosition) -> Result<[f32; 3]> {
        let position = position.require_finite()?;
        Ok(self.relative(position).map(|value| value as f32))
    }

    /// `position` relative to the origin, before rounding to `f32`.
    pub(crate) fn relative(&self, position: WorldPosition) -> [f64; 3] {
        [
            position.x - self.block.x as f64,
            position.y - self.block.y as f64,
            position.z - self.block.z as f64,
        ]
    }

    /// A render-space point back in the world.
    #[must_use]
    pub fn to_world(&self, local: [f32; 3]) -> WorldPosition {
        WorldPosition::new(
            self.block.x as f64 + f64::from(local[0]),
            self.block.y as f64 + f64::from(local[1]),
            self.block.z as f64 + f64::from(local[2]),
        )
    }

    /// Whether `position` has drifted more than [`REBASE_DISTANCE`] blocks
    /// from the origin along any axis.
    #[must_use]
    pub fn needs_rebase(&self, position: WorldPosition) -> bool {
        self.relative(position)
            .iter()
            .any(|delta| !delta.is_finite() || delta.abs() > REBASE_DISTANCE as f64)
    }

    /// The origin to use for a camera at `position`: this one while it is
    /// close enough, otherwise the block the camera is in.
    ///
    /// # Errors
    ///
    /// The position is not finite, or beyond the world's limits.
    pub fn follow(self, position: WorldPosition) -> Result<Self> {
        if self.needs_rebase(position) {
            Self::containing(position)
        } else {
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::spatial::MAX_BLOCK_COORD;

    /// Far out: every test here runs at the origin and at the world's edge.
    const FAR: i64 = MAX_BLOCK_COORD - 10_000;

    fn origin(at: i64) -> RenderOrigin {
        RenderOrigin::new(BlockPos::new(at, -at / 2, at)).expect("origin")
    }

    #[test]
    fn block_offsets_are_exact_at_any_distance_from_the_world_origin() {
        for base in [0, -1_000_000, FAR, -FAR] {
            let origin = origin(base);
            let block = BlockPos::new(base + 4095, -base / 2 - 17, base - 1);
            assert_eq!(
                origin.offset_of(block).unwrap(),
                [4095.0, -17.0, -1.0],
                "at {base}"
            );
        }
    }

    #[test]
    fn an_offset_that_f32_cannot_hold_exactly_is_refused() {
        let origin = origin(0);
        assert!(origin
            .offset_of(BlockPos::new(EXACT_OFFSET - 1, 0, 0))
            .is_ok());
        assert!(origin.offset_of(BlockPos::new(EXACT_OFFSET, 0, 0)).is_err());
        assert!(origin
            .offset_of(BlockPos::new(0, 0, -EXACT_OFFSET))
            .is_err());
    }

    #[test]
    fn positions_keep_sub_millimetre_precision_at_the_edge_of_the_world() {
        // A position 2^40 blocks out, a fraction into a block, within the
        // rebase distance of the origin. f64 holds it to 2^-12; render space
        // must not lose more than f32's own step at 4,096 (2^-11).
        for base in [0, FAR, -FAR] {
            let origin = origin(base);
            let world = WorldPosition::new(
                base as f64 + 4000.3125,
                (-base / 2) as f64 - 12.75,
                base as f64 + 0.5,
            );
            let render = origin.to_render(world).unwrap();
            let expected = [4000.3125f32, -12.75, 0.5];
            for axis in 0..3 {
                assert!(
                    (render[axis] - expected[axis]).abs() <= 2f32.powi(-11),
                    "axis {axis} at {base}: {} against {}",
                    render[axis],
                    expected[axis]
                );
            }
            // And back: the round trip lands where it started, to the same step.
            let back = origin.to_world(render);
            assert!((back.x - world.x).abs() <= 2f64.powi(-11));
            assert!((back.y - world.y).abs() <= 2f64.powi(-11));
            assert!((back.z - world.z).abs() <= 2f64.powi(-11));
        }
    }

    #[test]
    fn what_f32_alone_would_do_at_the_edge_of_the_world_is_why_the_origin_exists() {
        // Not a test of this module: the reason for it. Rounding the same
        // world coordinate straight to f32 loses whole kilometres of blocks.
        let world = FAR as f64 + 4000.3125;
        let error = (f64::from(world as f32) - world).abs();
        assert!(error > 1_000.0, "f32 at 2^40 is off by {error} blocks");
    }

    #[test]
    fn rebasing_moves_the_origin_and_never_the_world() {
        let origin = origin(FAR);
        let near = WorldPosition::new(FAR as f64 + 100.5, (-FAR / 2) as f64, FAR as f64);
        assert!(!origin.needs_rebase(near));
        assert_eq!(origin.follow(near).unwrap(), origin);

        let far = WorldPosition::new(FAR as f64 + 5000.25, (-FAR / 2) as f64 + 3.0, FAR as f64);
        assert!(origin.needs_rebase(far));
        let moved = origin.follow(far).unwrap();
        assert_eq!(moved.block(), BlockPos::new(FAR + 5000, -FAR / 2 + 3, FAR));
        // The same world point, seen from both origins, is the same world point.
        let point = WorldPosition::new(FAR as f64 + 4500.0, (-FAR / 2) as f64, FAR as f64 + 1.0);
        let through_old = origin.to_world(origin.to_render(point).unwrap());
        let through_new = moved.to_world(moved.to_render(point).unwrap());
        assert_eq!(through_old, point);
        assert_eq!(through_new, point);
    }

    #[test]
    fn a_position_that_is_not_finite_is_refused_and_forces_a_rebase_check() {
        let origin = origin(0);
        let bad = WorldPosition::new(f64::NAN, 0.0, 0.0);
        assert!(origin.to_render(bad).is_err());
        assert!(origin.needs_rebase(bad));
        assert!(origin.follow(bad).is_err());
    }

    #[test]
    fn an_origin_beyond_the_world_is_refused() {
        assert!(RenderOrigin::new(BlockPos::new(MAX_BLOCK_COORD + 1, 0, 0)).is_err());
    }
}
