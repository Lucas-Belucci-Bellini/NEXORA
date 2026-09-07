//! What can be streamed, why, and the handle that pins it.
//!
//! `STREAMING SYSTEM.md` lists the domains: chunks, regions, entities,
//! structures, resources, GPU assets, dimensions and simulation contexts. Only
//! chunks exist to stream in this phase, so [`StreamTarget`] carries one
//! variant — and is `#[non_exhaustive]`, because the rest are variants of it
//! rather than a different mechanism.

use core::fmt;

use nexora_foundation::spatial::ChunkCoord;

/// Something whose residency the manager controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StreamTarget {
    /// One chunk column.
    Chunk(ChunkCoord),
}

impl StreamTarget {
    /// The column this target sits in, for distance and priority.
    #[must_use]
    pub const fn column(self) -> ChunkCoord {
        match self {
            Self::Chunk(coord) => coord,
        }
    }

    /// Chebyshev distance to a column, in chunks.
    ///
    /// Chebyshev rather than Euclidean because interest radii describe the
    /// square region a world generates around an observer; a Euclidean radius
    /// would leave the corners of that square unloaded and visible.
    ///
    /// Returns `u64`, not `i64`. The span between the two ends of the
    /// coordinate range does not fit in a signed 64-bit integer, and a wrapped
    /// distance reads as *adjacent* — which would load the far side of the
    /// world and evict what the observer is standing on.
    #[must_use]
    pub fn chunk_distance(self, from: ChunkCoord) -> u64 {
        let here = self.column();
        here.x.abs_diff(from.x).max(here.z.abs_diff(from.z))
    }
}

impl fmt::Display for StreamTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chunk(coord) => write!(f, "chunk({},{})", coord.x, coord.z),
        }
    }
}

/// Why a target is being streamed.
///
/// Recorded rather than inferred: when a region is resident and nobody can say
/// why, the only safe action is to keep it, and a manager that can never let go
/// is not a streaming system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum StreamReason {
    /// A player or camera is near it.
    Interest,
    /// Loaded ahead of predicted movement.
    Prefetch,
    /// A world event needs it resident.
    WorldEvent,
    /// Something asked for it by name and holds a handle.
    Explicit,
}

impl StreamReason {
    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Interest => "interest",
            Self::Prefetch => "prefetch",
            Self::WorldEvent => "world-event",
            Self::Explicit => "explicit",
        }
    }
}

/// A pin held on a target, keeping it resident regardless of interest.
///
/// Generational for the same reason entity and body handles are: a handle that
/// outlives its request must stop resolving rather than release someone else's
/// pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamHandle {
    index: u32,
    generation: u32,
}

impl StreamHandle {
    /// Build a handle. Normally produced by the system, not by callers.
    #[must_use]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Slot index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// Generation of the slot when the handle was issued.
    #[must_use]
    pub const fn generation(self) -> u32 {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_is_chebyshev_so_a_radius_describes_a_square() {
        let target = StreamTarget::Chunk(ChunkCoord::new(3, 4));
        assert_eq!(target.chunk_distance(ChunkCoord::new(0, 0)), 4);
        // A corner of the square is the same distance as an edge.
        let corner = StreamTarget::Chunk(ChunkCoord::new(4, 4));
        let edge = StreamTarget::Chunk(ChunkCoord::new(4, 0));
        assert_eq!(
            corner.chunk_distance(ChunkCoord::new(0, 0)),
            edge.chunk_distance(ChunkCoord::new(0, 0))
        );
    }

    #[test]
    fn distance_is_symmetric_and_zero_at_home() {
        let origin = ChunkCoord::new(-7, 12);
        assert_eq!(StreamTarget::Chunk(origin).chunk_distance(origin), 0);
        let a = ChunkCoord::new(-7, 12);
        let b = ChunkCoord::new(3, -5);
        assert_eq!(
            StreamTarget::Chunk(a).chunk_distance(b),
            StreamTarget::Chunk(b).chunk_distance(a)
        );
    }

    #[test]
    fn distance_does_not_overflow_at_the_edge_of_the_world() {
        // abs_diff rather than subtraction: i64::MIN - i64::MAX would wrap, and
        // a wrapped distance reads as "very close" and loads the far side of
        // the world.
        let far = StreamTarget::Chunk(ChunkCoord::new(i64::MAX, i64::MAX));
        let near = ChunkCoord::new(i64::MIN, i64::MIN);
        assert_eq!(far.chunk_distance(near), u64::MAX);
    }

    #[test]
    fn targets_order_deterministically_for_tie_breaking() {
        let mut targets = [
            StreamTarget::Chunk(ChunkCoord::new(1, 0)),
            StreamTarget::Chunk(ChunkCoord::new(0, 1)),
            StreamTarget::Chunk(ChunkCoord::new(0, 0)),
        ];
        targets.sort();
        assert_eq!(targets[0], StreamTarget::Chunk(ChunkCoord::new(0, 0)));
        assert_eq!(targets[1], StreamTarget::Chunk(ChunkCoord::new(0, 1)));
    }

    #[test]
    fn a_target_renders_readably() {
        assert_eq!(
            StreamTarget::Chunk(ChunkCoord::new(-2, 5)).to_string(),
            "chunk(-2,5)"
        );
    }

    #[test]
    fn handles_carry_their_generation() {
        let handle = StreamHandle::new(2, 9);
        assert_eq!(handle.index(), 2);
        assert_eq!(handle.generation(), 9);
        assert_ne!(handle, StreamHandle::new(2, 10));
    }

    #[test]
    fn every_reason_has_a_name() {
        for reason in [
            StreamReason::Interest,
            StreamReason::Prefetch,
            StreamReason::WorldEvent,
            StreamReason::Explicit,
        ] {
            assert!(!reason.as_str().is_empty());
        }
    }
}
