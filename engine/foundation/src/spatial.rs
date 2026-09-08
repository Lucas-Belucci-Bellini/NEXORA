//! Coordinate spaces and conversions.
//!
//! Implements `SPATIAL AND COORDINATE SYSTEM.md` and `CHUNK & VOXEL ENGINE.md`
//! §2-§4. Two rules drive every decision here:
//!
//! 1. **Block identity is exact and stable.** Block coordinates are integers,
//!    never floats, so a position means the same voxel forever.
//! 2. **Negative coordinates are not a special case.** Truncating division
//!    (`-1 / 32 == 0`) puts block `-1` in chunk `0` alongside block `+1`, which
//!    silently mirrors the world across the origin. Every conversion below uses
//!    Euclidean division so that block `-1` lands in chunk `-1` with local
//!    offset `31`.

use crate::error::{Domain, Error, Recovery, Result};

/// Largest absolute block coordinate the engine accepts on any axis.
///
/// This is not a design limit on world size so much as an overflow guard: it
/// keeps `section * size + local` far inside `i64` for any legal chunk shape,
/// so conversions cannot wrap around into a different part of the world.
pub const MAX_BLOCK_COORD: i64 = 1 << 40;

/// Largest accepted extent of a chunk section on one axis.
pub const MAX_SECTION_EXTENT: u32 = 256;

/// One of the three coordinate axes.
///
/// Shared vocabulary rather than each system's own: physics resolves collision
/// one axis at a time, the mesher sweeps one axis at a time, and a third copy
/// of the same three-variant enum would mean three places to disagree about
/// what "the other two axes" means.
///
/// Policies that happen to be *about* axes stay with the system that owns them
/// — the order collision resolves in is a physics decision and lives there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Axis {
    /// East-west.
    X,
    /// Up-down.
    Y,
    /// North-south.
    Z,
}

impl Axis {
    /// All three axes in coordinate order.
    pub const ALL: [Self; 3] = [Self::X, Self::Y, Self::Z];

    /// Index into a three-component array.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }

    /// The other two axes, in coordinate order.
    #[must_use]
    pub const fn others(self) -> [Self; 2] {
        match self {
            Self::X => [Self::Y, Self::Z],
            Self::Y => [Self::X, Self::Z],
            Self::Z => [Self::X, Self::Y],
        }
    }

    /// Stable lowercase name, safe to emit in diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
        }
    }
}

/// A canonical integer block position in a dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockPos {
    /// East-west axis.
    pub x: i64,
    /// Vertical axis.
    pub y: i64,
    /// North-south axis.
    pub z: i64,
}

impl BlockPos {
    /// Construct a block position.
    #[must_use]
    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// The origin block.
    pub const ORIGIN: Self = Self::new(0, 0, 0);

    /// Whether every axis is inside [`MAX_BLOCK_COORD`].
    ///
    /// Uses `unsigned_abs` rather than `abs`: `i64::MIN.abs()` overflows, so the
    /// obvious spelling would panic on exactly the input this guard exists for.
    #[must_use]
    pub const fn is_within_limits(self) -> bool {
        const LIMIT: u64 = MAX_BLOCK_COORD as u64;
        self.x.unsigned_abs() <= LIMIT
            && self.y.unsigned_abs() <= LIMIT
            && self.z.unsigned_abs() <= LIMIT
    }

    /// Reject a position outside the engine's coordinate limits.
    ///
    /// # Errors
    ///
    /// Returns an error when any axis exceeds [`MAX_BLOCK_COORD`].
    pub fn require_within_limits(self) -> Result<Self> {
        if self.is_within_limits() {
            Ok(self)
        } else {
            Err(Error::new(
                Domain::Spatial,
                "block-pos",
                "block position exceeds world limits",
            )
            .with_recovery(Recovery::Reject)
            .with_context("position", format!("{},{},{}", self.x, self.y, self.z))
            .with_context("limit", MAX_BLOCK_COORD.to_string()))
        }
    }
}

/// A continuous position in world space.
///
/// `SPATIAL AND COORDINATE SYSTEM.md` warns against assuming one numeric type
/// suffices for every subsystem: voxel identity is exact and integral, but an
/// entity standing between two blocks is not. Blocks use [`BlockPos`]; anything
/// that moves continuously uses this.
///
/// `f64` rather than `f32` because the coordinate range is large: at the
/// engine's limits an `f32` mantissa cannot resolve single blocks, which is the
/// precision collapse the document's "high precision world transforms"
/// requirement exists to avoid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldPosition {
    /// East-west axis.
    pub x: f64,
    /// Vertical axis.
    pub y: f64,
    /// North-south axis.
    pub z: f64,
}

impl WorldPosition {
    /// The world origin.
    pub const ORIGIN: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Construct a continuous world position.
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// The block that contains this position.
    ///
    /// Floors rather than truncates: `-0.5` is inside block `-1`, not block `0`.
    /// Truncation here would place everything in the negative half-space one
    /// block too high, which is the same defect as truncating chunk division.
    #[must_use]
    pub fn to_block_pos(self) -> BlockPos {
        BlockPos::new(
            self.x.floor() as i64,
            self.y.floor() as i64,
            self.z.floor() as i64,
        )
    }

    /// The centre of a block.
    #[must_use]
    pub fn from_block_center(block: BlockPos) -> Self {
        Self::new(
            block.x as f64 + 0.5,
            block.y as f64 + 0.5,
            block.z as f64 + 0.5,
        )
    }

    /// Offset by a delta.
    #[must_use]
    pub const fn offset(self, dx: f64, dy: f64, dz: f64) -> Self {
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }

    /// Squared distance to another position.
    ///
    /// Squared, so a radius comparison needs no square root.
    #[must_use]
    pub fn distance_squared(self, other: Self) -> f64 {
        let (dx, dy, dz) = (self.x - other.x, self.y - other.y, self.z - other.z);
        dx * dx + dy * dy + dz * dz
    }

    /// Distance to another position.
    #[must_use]
    pub fn distance(self, other: Self) -> f64 {
        self.distance_squared(other).sqrt()
    }

    /// Whether every component is finite.
    ///
    /// A NaN position would silently poison every distance comparison it
    /// touches, so callers validate at the boundary instead.
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Reject a non-finite position.
    ///
    /// # Errors
    ///
    /// Returns an error when any component is NaN or infinite.
    pub fn require_finite(self) -> Result<Self> {
        if self.is_finite() {
            return Ok(self);
        }
        Err(
            Error::new(Domain::Spatial, "world-position", "position is not finite")
                .with_recovery(Recovery::Reject)
                .with_context("position", format!("{},{},{}", self.x, self.y, self.z)),
        )
    }
}

/// A chunk column address: the `(x, z)` footprint shared by a stack of sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkCoord {
    /// Column index on the east-west axis.
    pub x: i64,
    /// Column index on the north-south axis.
    pub z: i64,
}

impl ChunkCoord {
    /// Construct a chunk column address.
    #[must_use]
    pub const fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }
}

/// A cubic section address within a dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionCoord {
    /// Section index on the east-west axis.
    pub x: i64,
    /// Section index on the vertical axis.
    pub y: i64,
    /// Section index on the north-south axis.
    pub z: i64,
}

impl SectionCoord {
    /// Construct a section address.
    #[must_use]
    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    /// The chunk column this section belongs to.
    #[must_use]
    pub const fn column(self) -> ChunkCoord {
        ChunkCoord::new(self.x, self.z)
    }
}

/// A region address: the storage grouping of chunk columns into one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionCoord {
    /// Region index on the east-west axis.
    pub x: i64,
    /// Region index on the north-south axis.
    pub z: i64,
}

impl RegionCoord {
    /// Construct a region address.
    #[must_use]
    pub const fn new(x: i64, z: i64) -> Self {
        Self { x, z }
    }
}

/// A position inside one section, in section-local voxel units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalPos {
    /// Local east-west offset.
    pub x: u32,
    /// Local vertical offset.
    pub y: u32,
    /// Local north-south offset.
    pub z: u32,
}

impl LocalPos {
    /// Construct a section-local position.
    #[must_use]
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
}

/// The voxel extent of one chunk section.
///
/// `CHUNK & VOXEL ENGINE.md` §3 is explicit that the engine must not assume a
/// fixed chunk size anywhere, so the shape is a runtime value carried alongside
/// the data rather than a compile-time constant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkShape {
    size_x: u32,
    size_y: u32,
    size_z: u32,
}

/// Default section extent on every axis.
pub const DEFAULT_SECTION_SIZE: u32 = 32;

impl ChunkShape {
    /// Validate and construct a chunk shape.
    ///
    /// # Errors
    ///
    /// Returns an error when any axis is zero or larger than
    /// [`MAX_SECTION_EXTENT`].
    pub fn new(size_x: u32, size_y: u32, size_z: u32) -> Result<Self> {
        for (axis, size) in [("x", size_x), ("y", size_y), ("z", size_z)] {
            if size == 0 || size > MAX_SECTION_EXTENT {
                return Err(Error::new(
                    Domain::Spatial,
                    "chunk-shape",
                    "section extent must be between 1 and the maximum section extent",
                )
                .with_recovery(Recovery::Reject)
                .with_context("axis", axis)
                .with_context("size", size.to_string())
                .with_context("max", MAX_SECTION_EXTENT.to_string()));
            }
        }
        Ok(Self {
            size_x,
            size_y,
            size_z,
        })
    }

    /// The engine default, `32 x 32 x 32`.
    #[must_use]
    pub const fn cubic_default() -> Self {
        Self {
            size_x: DEFAULT_SECTION_SIZE,
            size_y: DEFAULT_SECTION_SIZE,
            size_z: DEFAULT_SECTION_SIZE,
        }
    }

    /// Extent on the east-west axis.
    #[must_use]
    pub const fn size_x(self) -> u32 {
        self.size_x
    }

    /// Extent on the vertical axis.
    #[must_use]
    pub const fn size_y(self) -> u32 {
        self.size_y
    }

    /// Extent on the north-south axis.
    #[must_use]
    pub const fn size_z(self) -> u32 {
        self.size_z
    }

    /// Number of voxels in one section of this shape.
    #[must_use]
    pub const fn volume(self) -> usize {
        (self.size_x as usize) * (self.size_y as usize) * (self.size_z as usize)
    }

    /// The section that contains a block position.
    #[must_use]
    pub const fn section_of(self, pos: BlockPos) -> SectionCoord {
        SectionCoord::new(
            pos.x.div_euclid(self.size_x as i64),
            pos.y.div_euclid(self.size_y as i64),
            pos.z.div_euclid(self.size_z as i64),
        )
    }

    /// The section-local offset of a block position.
    ///
    /// Always in `0..size` on each axis, including for negative world
    /// coordinates.
    #[must_use]
    pub const fn local_of(self, pos: BlockPos) -> LocalPos {
        LocalPos::new(
            pos.x.rem_euclid(self.size_x as i64) as u32,
            pos.y.rem_euclid(self.size_y as i64) as u32,
            pos.z.rem_euclid(self.size_z as i64) as u32,
        )
    }

    /// The block position of a section's minimum corner.
    ///
    /// # Errors
    ///
    /// Returns an error when the section address is so far out that the origin
    /// would overflow `i64`.
    pub fn section_origin(self, section: SectionCoord) -> Result<BlockPos> {
        let origin = |index: i64, size: u32, axis: &'static str| -> Result<i64> {
            index.checked_mul(size as i64).ok_or_else(|| {
                Error::new(
                    Domain::Spatial,
                    "chunk-shape",
                    "section origin overflows i64",
                )
                .with_recovery(Recovery::Reject)
                .with_context("axis", axis)
                .with_context("section", index.to_string())
            })
        };
        Ok(BlockPos::new(
            origin(section.x, self.size_x, "x")?,
            origin(section.y, self.size_y, "y")?,
            origin(section.z, self.size_z, "z")?,
        ))
    }

    /// The absolute block position of a section-local offset.
    ///
    /// # Errors
    ///
    /// Returns an error when the local offset is outside the shape, or when the
    /// resulting position overflows `i64`.
    pub fn to_block_pos(self, section: SectionCoord, local: LocalPos) -> Result<BlockPos> {
        self.require_in_bounds(local)?;
        let origin = self.section_origin(section)?;
        let add = |base: i64, offset: u32, axis: &'static str| -> Result<i64> {
            base.checked_add(offset as i64).ok_or_else(|| {
                Error::new(
                    Domain::Spatial,
                    "chunk-shape",
                    "block position overflows i64",
                )
                .with_recovery(Recovery::Reject)
                .with_context("axis", axis)
            })
        };
        Ok(BlockPos::new(
            add(origin.x, local.x, "x")?,
            add(origin.y, local.y, "y")?,
            add(origin.z, local.z, "z")?,
        ))
    }

    /// Whether a local position lies inside this shape.
    #[must_use]
    pub const fn contains(self, local: LocalPos) -> bool {
        local.x < self.size_x && local.y < self.size_y && local.z < self.size_z
    }

    /// Reject a local position that lies outside this shape.
    ///
    /// # Errors
    ///
    /// Returns an error when the position is out of bounds.
    pub fn require_in_bounds(self, local: LocalPos) -> Result<()> {
        if self.contains(local) {
            return Ok(());
        }
        Err(Error::new(
            Domain::Spatial,
            "chunk-shape",
            "local position is outside the section",
        )
        .with_recovery(Recovery::Reject)
        .with_context("local", format!("{},{},{}", local.x, local.y, local.z))
        .with_context(
            "shape",
            format!("{}x{}x{}", self.size_x, self.size_y, self.size_z),
        ))
    }

    /// Flatten a local position into a storage index.
    ///
    /// Layout is x-major within a row, then z, then y, so that iterating a
    /// horizontal slice walks contiguous memory.
    ///
    /// # Errors
    ///
    /// Returns an error when the position is outside the shape.
    pub fn index_of(self, local: LocalPos) -> Result<usize> {
        self.require_in_bounds(local)?;
        Ok(self.index_of_unchecked(local))
    }

    /// Flatten a local position without bounds checking.
    ///
    /// Callers must have validated the position; this exists for hot loops that
    /// already iterate within the shape.
    #[must_use]
    pub const fn index_of_unchecked(self, local: LocalPos) -> usize {
        ((local.y as usize) * (self.size_z as usize) + (local.z as usize)) * (self.size_x as usize)
            + (local.x as usize)
    }

    /// Recover a local position from a storage index.
    ///
    /// # Errors
    ///
    /// Returns an error when the index is outside the section volume.
    pub fn local_from_index(self, index: usize) -> Result<LocalPos> {
        if index >= self.volume() {
            return Err(Error::new(
                Domain::Spatial,
                "chunk-shape",
                "storage index is outside the section volume",
            )
            .with_recovery(Recovery::Reject)
            .with_context("index", index.to_string())
            .with_context("volume", self.volume().to_string()));
        }
        let size_x = self.size_x as usize;
        let size_z = self.size_z as usize;
        let x = index % size_x;
        let rest = index / size_x;
        let z = rest % size_z;
        let y = rest / size_z;
        Ok(LocalPos::new(x as u32, y as u32, z as u32))
    }
}

/// How many chunk columns a region covers on each axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionShape {
    columns_per_axis: u32,
}

/// Default region extent, in chunk columns per axis.
pub const DEFAULT_REGION_COLUMNS: u32 = 32;

impl RegionShape {
    /// Validate and construct a region shape.
    ///
    /// # Errors
    ///
    /// Returns an error when the extent is zero.
    pub fn new(columns_per_axis: u32) -> Result<Self> {
        if columns_per_axis == 0 {
            return Err(Error::new(
                Domain::Spatial,
                "region-shape",
                "a region must span at least one chunk column",
            )
            .with_recovery(Recovery::Reject));
        }
        Ok(Self { columns_per_axis })
    }

    /// The engine default region extent.
    #[must_use]
    pub const fn default_shape() -> Self {
        Self {
            columns_per_axis: DEFAULT_REGION_COLUMNS,
        }
    }

    /// Chunk columns spanned per axis.
    #[must_use]
    pub const fn columns_per_axis(self) -> u32 {
        self.columns_per_axis
    }

    /// The region that stores a chunk column.
    #[must_use]
    pub const fn region_of(self, chunk: ChunkCoord) -> RegionCoord {
        RegionCoord::new(
            chunk.x.div_euclid(self.columns_per_axis as i64),
            chunk.z.div_euclid(self.columns_per_axis as i64),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic coordinate generator: a SplitMix64 step, used so the sweep
    /// is reproducible rather than randomly flaky.
    fn spread(seed: u64) -> i64 {
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        // Keep well inside the coordinate limit so conversions cannot overflow.
        ((z ^ (z >> 31)) as i64) % (MAX_BLOCK_COORD / 2)
    }

    #[test]
    fn negative_coordinates_floor_instead_of_truncating() {
        let shape = ChunkShape::cubic_default();

        // The bug this guards: truncating division puts -1 and +1 in the same
        // chunk, mirroring the world across the origin.
        assert_eq!(
            shape.section_of(BlockPos::new(-1, -1, -1)),
            SectionCoord::new(-1, -1, -1)
        );
        assert_eq!(
            shape.local_of(BlockPos::new(-1, -1, -1)),
            LocalPos::new(31, 31, 31)
        );

        assert_eq!(
            shape.section_of(BlockPos::new(0, 0, 0)),
            SectionCoord::new(0, 0, 0)
        );
        assert_eq!(
            shape.local_of(BlockPos::new(0, 0, 0)),
            LocalPos::new(0, 0, 0)
        );

        assert_eq!(
            shape.section_of(BlockPos::new(-32, -32, -32)),
            SectionCoord::new(-1, -1, -1)
        );
        assert_eq!(
            shape.local_of(BlockPos::new(-32, -32, -32)),
            LocalPos::new(0, 0, 0)
        );

        assert_eq!(
            shape.section_of(BlockPos::new(-33, -33, -33)),
            SectionCoord::new(-2, -2, -2)
        );
        assert_eq!(
            shape.local_of(BlockPos::new(-33, -33, -33)),
            LocalPos::new(31, 31, 31)
        );
    }

    #[test]
    fn block_to_section_and_back_is_lossless() {
        let shapes = [
            ChunkShape::cubic_default(),
            ChunkShape::new(16, 16, 16).unwrap(),
            // A deliberately non-cubic, non-power-of-two shape: the conversions
            // must not secretly depend on 32 or on powers of two.
            ChunkShape::new(7, 5, 13).unwrap(),
        ];

        for shape in shapes {
            for seed in 0..2_000u64 {
                let pos = BlockPos::new(spread(seed), spread(seed ^ 0xAAAA), spread(seed ^ 0x5555));
                let section = shape.section_of(pos);
                let local = shape.local_of(pos);

                assert!(
                    shape.contains(local),
                    "local {local:?} escaped shape {shape:?}"
                );

                let rebuilt = shape
                    .to_block_pos(section, local)
                    .expect("round trip in range");
                assert_eq!(rebuilt, pos, "shape {shape:?} lost position {pos:?}");
            }
        }
    }

    #[test]
    fn storage_index_round_trips_over_the_whole_volume() {
        let shape = ChunkShape::new(7, 5, 13).unwrap();
        let mut seen = vec![false; shape.volume()];

        for y in 0..shape.size_y() {
            for z in 0..shape.size_z() {
                for x in 0..shape.size_x() {
                    let local = LocalPos::new(x, y, z);
                    let index = shape.index_of(local).expect("in bounds");
                    assert!(!seen[index], "index {index} produced twice");
                    seen[index] = true;
                    assert_eq!(shape.local_from_index(index).unwrap(), local);
                }
            }
        }

        assert!(
            seen.into_iter().all(|hit| hit),
            "index mapping did not cover the volume"
        );
    }

    #[test]
    fn out_of_bounds_access_is_rejected_not_wrapped() {
        let shape = ChunkShape::cubic_default();
        assert!(shape.index_of(LocalPos::new(32, 0, 0)).is_err());
        assert!(shape.index_of(LocalPos::new(0, 32, 0)).is_err());
        assert!(shape.index_of(LocalPos::new(0, 0, 32)).is_err());
        assert!(shape.local_from_index(shape.volume()).is_err());
    }

    #[test]
    fn invalid_shapes_are_rejected() {
        assert!(ChunkShape::new(0, 32, 32).is_err());
        assert!(ChunkShape::new(32, 0, 32).is_err());
        assert!(ChunkShape::new(32, 32, 0).is_err());
        assert!(ChunkShape::new(MAX_SECTION_EXTENT + 1, 32, 32).is_err());
        assert!(ChunkShape::new(MAX_SECTION_EXTENT, 1, 1).is_ok());
    }

    #[test]
    fn extreme_coordinates_are_bounded_not_wrapped() {
        let shape = ChunkShape::cubic_default();
        let far = BlockPos::new(MAX_BLOCK_COORD, -MAX_BLOCK_COORD, MAX_BLOCK_COORD);
        assert!(far.is_within_limits());
        let section = shape.section_of(far);
        let local = shape.local_of(far);
        assert_eq!(shape.to_block_pos(section, local).unwrap(), far);

        let beyond = BlockPos::new(MAX_BLOCK_COORD + 1, 0, 0);
        assert!(!beyond.is_within_limits());
        assert!(beyond.require_within_limits().is_err());

        // A section address near i64::MAX must report overflow instead of wrapping.
        assert!(shape
            .section_origin(SectionCoord::new(i64::MAX, 0, 0))
            .is_err());
    }

    #[test]
    fn regions_group_columns_with_floor_semantics() {
        let region = RegionShape::default_shape();
        assert_eq!(
            region.region_of(ChunkCoord::new(0, 0)),
            RegionCoord::new(0, 0)
        );
        assert_eq!(
            region.region_of(ChunkCoord::new(31, 31)),
            RegionCoord::new(0, 0)
        );
        assert_eq!(
            region.region_of(ChunkCoord::new(32, 32)),
            RegionCoord::new(1, 1)
        );
        assert_eq!(
            region.region_of(ChunkCoord::new(-1, -1)),
            RegionCoord::new(-1, -1)
        );
        assert_eq!(
            region.region_of(ChunkCoord::new(-32, -32)),
            RegionCoord::new(-1, -1)
        );
        assert_eq!(
            region.region_of(ChunkCoord::new(-33, -33)),
            RegionCoord::new(-2, -2)
        );
        assert!(RegionShape::new(0).is_err());
    }

    #[test]
    fn continuous_positions_floor_into_the_right_block() {
        // The same defect class as truncating chunk division: truncation puts
        // -0.5 in block 0, one block too high across the whole negative side.
        assert_eq!(
            WorldPosition::new(0.0, 0.0, 0.0).to_block_pos(),
            BlockPos::new(0, 0, 0)
        );
        assert_eq!(
            WorldPosition::new(0.9, 0.9, 0.9).to_block_pos(),
            BlockPos::new(0, 0, 0)
        );
        assert_eq!(
            WorldPosition::new(-0.5, -0.5, -0.5).to_block_pos(),
            BlockPos::new(-1, -1, -1)
        );
        assert_eq!(
            WorldPosition::new(-1.0, -1.0, -1.0).to_block_pos(),
            BlockPos::new(-1, -1, -1)
        );
        assert_eq!(
            WorldPosition::new(-1.1, -1.1, -1.1).to_block_pos(),
            BlockPos::new(-2, -2, -2)
        );
    }

    #[test]
    fn block_centres_round_trip_to_their_block() {
        for block in [
            BlockPos::new(0, 0, 0),
            BlockPos::new(-1, -1, -1),
            BlockPos::new(37, -412, 9_001),
            BlockPos::new(-1_000_000, 64, -7),
        ] {
            assert_eq!(
                WorldPosition::from_block_center(block).to_block_pos(),
                block
            );
        }
    }

    #[test]
    fn distances_are_euclidean_and_squared_avoids_the_root() {
        let a = WorldPosition::new(0.0, 0.0, 0.0);
        let b = WorldPosition::new(3.0, 4.0, 0.0);
        assert!((a.distance(b) - 5.0).abs() < 1e-12);
        assert!((a.distance_squared(b) - 25.0).abs() < 1e-12);
        assert!((a.distance(a)).abs() < 1e-12);
    }

    #[test]
    fn non_finite_positions_are_rejected() {
        assert!(WorldPosition::new(f64::NAN, 0.0, 0.0)
            .require_finite()
            .is_err());
        assert!(WorldPosition::new(0.0, f64::INFINITY, 0.0)
            .require_finite()
            .is_err());
        assert!(WorldPosition::new(0.0, 0.0, f64::NEG_INFINITY)
            .require_finite()
            .is_err());
        assert!(WorldPosition::new(1.0, 2.0, 3.0).require_finite().is_ok());
    }

    #[test]
    fn offsets_accumulate() {
        let moved = WorldPosition::ORIGIN
            .offset(1.5, -2.0, 0.25)
            .offset(0.5, 2.0, 0.75);
        assert_eq!(moved, WorldPosition::new(2.0, 0.0, 1.0));
    }

    #[test]
    fn section_knows_its_column() {
        assert_eq!(SectionCoord::new(4, -7, 9).column(), ChunkCoord::new(4, 9));
    }
}
