//! A world save spread over region files.
//!
//! `CHUNK & VOXEL ENGINE.md` §27 (CHUNK-25) asks for chunks grouped into
//! regions rather than a file per column. [`persist`] writes the other shape:
//! one container holding every resident chunk in one section. That is a
//! snapshot, and it costs the whole world on every save - encode, deflate,
//! write, read back - however little of it moved.
//!
//! A [`RegionStore`] is the same bytes arranged as a directory:
//!
//! ```text
//! <root>/world.nxsv               world header: identity, seed, bounds, clock
//! <root>/region/r.0.0.nxsv        one region: its palette and its columns
//! <root>/region/r.-1.0.nxsv
//! ```
//!
//! Every file is an ordinary [`SaveContainer`], so region files inherit the
//! container's framing, per-section checksums, compression and atomic write
//! with read-back verification. Nothing here is a new save format.
//!
//! Three decisions shape this module, recorded in ADR-0014:
//!
//! 1. **A region file is authoritative for its region; the resident set is not
//!    a delete list.** Writing never removes a region file for a region the
//!    world does not currently hold. That is the difference from the snapshot
//!    container, and it is the point: a region that scrolled out of the
//!    resident set is still part of the world.
//! 2. **Each region file carries its own identifier palette.** It costs the
//!    palette once per region and buys a file that can be read on its own,
//!    without the header and without its neighbours - which is what a
//!    streaming `activate` needs.
//! 3. **The header is written last.** A crash between the regions and the
//!    header leaves the regions ahead of the clock, and replaying the journal
//!    over them is a no-op because `SetBlock` is idempotent. Writing the header
//!    first would leave the clock ahead of the regions, and no replay repairs
//!    that.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{ChunkCoord, RegionCoord, RegionShape};
use nexora_persistence::codec::{Reader, Writer};
use nexora_persistence::SaveContainer;

use crate::chunk::Chunk;
use crate::persist::{self, SECTION_BLOCK_PALETTE, SECTION_CHUNKS, SECTION_WORLD_HEADER};
use crate::voxel::BlockStateId;
use crate::world::{BlockDefinition, World};

/// File name of the world header inside a region store.
pub const HEADER_FILE: &str = "world.nxsv";

/// Directory holding the region files, relative to the store root.
pub const REGION_DIR: &str = "region";

/// Extension shared by the header and every region file.
pub const SAVE_EXTENSION: &str = "nxsv";

/// Section in the header file recording how columns were grouped.
///
/// It lives only in a region store, never in a single-container save: which
/// region a column belongs to is a property of this layout, not of the world.
pub const SECTION_REGION_SHAPE: &str = "nexora:save/region_shape";

/// What one write touched, and what it was able to leave alone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WriteReport {
    /// Regions encoded and written, in ascending order.
    pub written: Vec<RegionCoord>,
    /// Regions left untouched because nothing in them had changed.
    pub skipped: Vec<RegionCoord>,
    /// Bytes written across the region files, excluding the header.
    pub bytes: u64,
}

impl WriteReport {
    /// How many regions the world spans, written and skipped together.
    #[must_use]
    pub fn regions(&self) -> usize {
        self.written.len() + self.skipped.len()
    }
}

/// A world save laid out as one file per region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionStore {
    root: PathBuf,
    shape: RegionShape,
}

impl RegionStore {
    /// Open a store at `root`, grouping columns by the engine default region
    /// extent.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::with_shape(root, RegionShape::default_shape())
    }

    /// Open a store at `root` with an explicit region extent.
    #[must_use]
    pub fn with_shape(root: impl Into<PathBuf>, shape: RegionShape) -> Self {
        Self {
            root: root.into(),
            shape,
        }
    }

    /// The directory this store owns.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// How many columns per axis one region covers here.
    #[must_use]
    pub const fn shape(&self) -> RegionShape {
        self.shape
    }

    /// Path of the world header file.
    #[must_use]
    pub fn header_path(&self) -> PathBuf {
        self.root.join(HEADER_FILE)
    }

    /// Path of one region's file, whether or not it exists.
    #[must_use]
    pub fn region_path(&self, region: RegionCoord) -> PathBuf {
        self.root
            .join(REGION_DIR)
            .join(format!("r.{}.{}.{SAVE_EXTENSION}", region.x, region.z))
    }

    /// Write every resident region, whether or not it changed.
    ///
    /// # Errors
    ///
    /// Returns an error when a resident block state has no registered
    /// identifier, or when any file cannot be written and verified.
    pub fn write_all(&self, world: &mut World) -> Result<WriteReport> {
        self.write(world, Scope::Everything)
    }

    /// Write only the regions holding a changed column, plus any region that
    /// has no file yet.
    ///
    /// # Errors
    ///
    /// Returns an error when a resident block state has no registered
    /// identifier, or when any file cannot be written and verified.
    pub fn write_dirty(&self, world: &mut World) -> Result<WriteReport> {
        self.write(world, Scope::Changed)
    }

    fn write(&self, world: &mut World, scope: Scope) -> Result<WriteReport> {
        // Before anything is written: a store grouped one way and reopened
        // another way would file columns under region addresses that already
        // hold different columns, and `write_dirty` would skip past the
        // mismatch. Refusing is the only answer that does not lose data.
        self.check_recorded_shape()?;

        let grouped = self.group(world);
        let mut report = WriteReport::default();

        for (region, columns) in grouped {
            let dirty = columns
                .iter()
                .any(|&coord| world.chunk(coord).is_some_and(Chunk::is_dirty));
            let path = self.region_path(region);
            if scope == Scope::Changed && !dirty && path.exists() {
                report.skipped.push(region);
                continue;
            }

            let mut container = SaveContainer::new();
            container.put(
                persist::section_id(SECTION_BLOCK_PALETTE)?,
                persist::encode_palette(world)?,
            );
            container.put(
                persist::section_id(SECTION_CHUNKS)?,
                persist::encode_chunks_of(world, &columns),
            );
            container.write_atomic(&path)?;
            // Measured from the file rather than by encoding a second time:
            // deflate is the expensive half of a write, and asking for a size
            // is not a reason to pay it twice.
            report.bytes += fs::metadata(&path)
                .map_err(|cause| io_error("size region", &path, &cause))?
                .len();

            // Only once the bytes are on disk and verified. Clearing first
            // would turn a failed write into a silently unsaved region.
            for coord in columns {
                world.mark_saved(coord);
            }
            report.written.push(region);
        }

        // Last, deliberately: see the module header. Regions ahead of the
        // clock are repairable by replay; a clock ahead of the regions is not.
        let mut header = SaveContainer::new();
        header.put(
            persist::section_id(SECTION_WORLD_HEADER)?,
            persist::encode_header(world),
        );
        header.put(
            persist::section_id(SECTION_REGION_SHAPE)?,
            self.encode_shape(),
        );
        header.write_atomic(&self.header_path())?;

        Ok(report)
    }

    /// Rebuild the world from the header and every region file on disk.
    ///
    /// # Errors
    ///
    /// Returns an error when the header is missing or malformed, when a region
    /// file fails verification, or when a region names block content this
    /// build does not register.
    pub fn read(&self) -> Result<World> {
        self.read_with(&[])
    }

    /// [`RegionStore::read`], with block content registered.
    ///
    /// The region-store twin of `persist::load_with` (ADR-0020): content
    /// enters through the same door on every load path, or a world that
    /// saves stones as regions could not read them back.
    ///
    /// # Errors
    ///
    /// See [`RegionStore::read`]; a block the regions name that neither the
    /// built-ins nor `content` register is refused by name.
    pub fn read_with(&self, content: &[(Identifier, BlockDefinition)]) -> Result<World> {
        let header = SaveContainer::read(&self.header_path())?;
        header.compatibility()?;
        self.compare_shape(&header)?;
        let (descriptor, calendar, now) =
            persist::decode_header(header.require(&persist::section_id(SECTION_WORLD_HEADER)?)?)?;
        let mut world = World::resumed_with(descriptor, calendar, now, content)?;

        for region in self.regions_on_disk()? {
            for chunk in self.read_region(&world, region)? {
                world.insert_chunk(chunk);
            }
        }
        Ok(world)
    }

    /// Read one region's columns, resolving its palette against `world`.
    ///
    /// A region file is self-contained, so this touches exactly one file. An
    /// absent region reads as no columns rather than as an error: a region that
    /// was never written is a region that was never edited.
    ///
    /// # Errors
    ///
    /// Returns an error when the file exists but fails verification, or when it
    /// names block content this build does not register.
    pub fn read_region(&self, world: &World, region: RegionCoord) -> Result<Vec<Chunk>> {
        let path = self.region_path(region);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let container = SaveContainer::read(&path)?;
        container.compatibility()?;

        let saved = persist::decode_palette(
            container.require(&persist::section_id(SECTION_BLOCK_PALETTE)?)?,
        )?;
        let remap: Vec<BlockStateId> = saved
            .iter()
            .map(|id| {
                world.block_id(id).map_err(|cause| {
                    Error::new(
                        Domain::Save,
                        "region-store",
                        "a region references block content that is not registered in this build",
                    )
                    .with_recovery(Recovery::Quarantine)
                    .with_context("block", id.to_string())
                    .with_context("region", format!("{},{}", region.x, region.z))
                    .with_source(cause)
                })
            })
            .collect::<Result<_>>()?;

        persist::decode_chunks(
            container.require(&persist::section_id(SECTION_CHUNKS)?)?,
            world.descriptor().shape,
            &remap,
        )
    }

    /// Read one column out of its region file, if the file holds it.
    ///
    /// `None` means the store has nothing for that column — either the region
    /// has no file, or the file has other columns but not this one.
    ///
    /// # Errors
    ///
    /// Returns an error when the region file exists but fails verification, or
    /// names block content this build does not register.
    pub fn read_column(&self, world: &World, coord: ChunkCoord) -> Result<Option<Chunk>> {
        let region = self.shape.region_of(coord);
        Ok(self
            .read_region(world, region)?
            .into_iter()
            .find(|chunk| chunk.coord() == coord))
    }

    /// Merge columns into their region files, keeping every stored column the
    /// call does not name.
    ///
    /// This is the write a streaming eviction needs: the column leaving memory
    /// is not in the world any more, and the region file it belongs to holds
    /// columns that are not in the world either. Writing the region from the
    /// resident set would drop them, so the file is read, overlaid and written
    /// back — which is also why this is one read-modify-write per region and
    /// not per column.
    ///
    /// # Errors
    ///
    /// Returns an error when a stored region cannot be read, when a block state
    /// has no registered identifier, or when a file cannot be written.
    pub fn store_columns(&self, world: &World, columns: Vec<Chunk>) -> Result<WriteReport> {
        self.check_recorded_shape()?;

        let mut by_region: BTreeMap<RegionCoord, BTreeMap<ChunkCoord, Chunk>> = BTreeMap::new();
        for chunk in columns {
            by_region
                .entry(self.shape.region_of(chunk.coord()))
                .or_default()
                .insert(chunk.coord(), chunk);
        }

        let mut report = WriteReport::default();
        for (region, mut merged) in by_region {
            // Existing first, incoming second: a column named by the caller
            // replaces the stored one rather than being discarded by it.
            for stored in self.read_region(world, region)? {
                merged.entry(stored.coord()).or_insert(stored);
            }

            let held: Vec<&Chunk> = merged.values().collect();
            let mut container = SaveContainer::new();
            container.put(
                persist::section_id(SECTION_BLOCK_PALETTE)?,
                persist::encode_palette(world)?,
            );
            container.put(
                persist::section_id(SECTION_CHUNKS)?,
                persist::encode_chunks_from(&held),
            );

            let path = self.region_path(region);
            container.write_atomic(&path)?;
            report.bytes += fs::metadata(&path)
                .map_err(|cause| io_error("size region", &path, &cause))?
                .len();
            report.written.push(region);
        }
        Ok(report)
    }

    /// Every region this store has a file for, in ascending order.
    ///
    /// # Errors
    ///
    /// Returns an error when the region directory cannot be listed, or when it
    /// holds a `.nxsv` file whose name is not a region address.
    pub fn regions_on_disk(&self) -> Result<Vec<RegionCoord>> {
        let dir = self.root.join(REGION_DIR);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&dir).map_err(|cause| io_error("list regions", &dir, &cause))?;
        let mut found = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|cause| io_error("list regions", &dir, &cause))?;
            let path = entry.path();
            // Quarantined copies and interrupted writes keep their own suffix
            // after `.nxsv`, so filtering on the extension leaves them out
            // without having to know their spellings here.
            if path.extension().and_then(|e| e.to_str()) != Some(SAVE_EXTENSION) {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_owned();
            found.push(parse_region_stem(&stem, &path)?);
        }
        found.sort_unstable();
        Ok(found)
    }

    fn encode_shape(&self) -> Vec<u8> {
        let mut out = Writer::new();
        out.u32(self.shape.columns_per_axis());
        out.finish()
    }

    /// Fail when the store on disk was written with a different region extent.
    ///
    /// A header without the section predates it and is accepted: the only
    /// stores that can exist without it were written by this same default.
    fn compare_shape(&self, header: &SaveContainer) -> Result<()> {
        let Some(bytes) = header.get(&persist::section_id(SECTION_REGION_SHAPE)?) else {
            return Ok(());
        };
        let mut reader = Reader::new(bytes);
        let recorded = reader.u32()?;
        reader.expect_exhausted()?;
        if recorded == self.shape.columns_per_axis() {
            return Ok(());
        }
        Err(Error::new(
            Domain::Save,
            "region-store",
            "this store was written with a different region extent",
        )
        .with_recovery(Recovery::Reject)
        .with_context("recorded", recorded.to_string())
        .with_context("opened_with", self.shape.columns_per_axis().to_string())
        .with_context("path", self.header_path().display().to_string()))
    }

    fn check_recorded_shape(&self) -> Result<()> {
        let path = self.header_path();
        if !path.exists() {
            return Ok(());
        }
        self.compare_shape(&SaveContainer::read(&path)?)
    }

    /// Group the resident columns by the region that stores them.
    fn group(&self, world: &World) -> Vec<(RegionCoord, Vec<ChunkCoord>)> {
        let mut columns = world.loaded_chunks();
        columns.sort_unstable_by_key(|coord| (self.shape.region_of(*coord), coord.x, coord.z));

        let mut grouped: Vec<(RegionCoord, Vec<ChunkCoord>)> = Vec::new();
        for coord in columns {
            let region = self.shape.region_of(coord);
            match grouped.last_mut() {
                Some((last, members)) if *last == region => members.push(coord),
                _ => grouped.push((region, vec![coord])),
            }
        }
        grouped
    }
}

/// Which regions a write is allowed to skip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// Write every resident region.
    Everything,
    /// Write the regions that changed, and any that has no file yet.
    Changed,
}

fn parse_region_stem(stem: &str, path: &Path) -> Result<RegionCoord> {
    let malformed = || {
        Error::new(
            Domain::Save,
            "region-store",
            "the region directory holds a save file whose name is not a region address",
        )
        .with_recovery(Recovery::Quarantine)
        .with_context("path", path.display().to_string())
        .with_context("expected", "r.<x>.<z>")
    };

    let rest = stem.strip_prefix("r.").ok_or_else(malformed)?;
    // Split from the right: the x coordinate may itself be negative, so the
    // separator between the two numbers is the last dot, not the first.
    let (x, z) = rest.rsplit_once('.').ok_or_else(malformed)?;
    Ok(RegionCoord::new(
        x.parse().map_err(|_| malformed())?,
        z.parse().map_err(|_| malformed())?,
    ))
}

fn io_error(action: &'static str, path: &Path, cause: &std::io::Error) -> Error {
    Error::new(
        Domain::Save,
        "region-store",
        "a region-store filesystem operation failed",
    )
    .with_recovery(Recovery::Retry)
    .with_context("action", action)
    .with_context("path", path.display().to_string())
    .with_context("cause", cause.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxel::AIR;
    use crate::world::WorldDescriptor;
    use nexora_foundation::ident::Identifier;
    use nexora_foundation::spatial::BlockPos;
    use nexora_foundation::time::{CalendarConfig, TimeScale};

    /// A scratch directory that cleans itself up.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let base = std::env::temp_dir().join(format!(
                "nexora-region-{label}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = fs::remove_dir_all(&base);
            fs::create_dir_all(&base).expect("create scratch directory");
            Self(base)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Two columns per region, so a 3x3 world spans four regions instead of
    /// one. At the engine default of 32 every world in this repo is a single
    /// region, and a test at that extent could not tell skipping from writing.
    fn small_regions() -> RegionShape {
        RegionShape::new(2).expect("two columns per region")
    }

    fn built_world(seed: u64) -> World {
        let mut world = World::create(
            WorldDescriptor::new("region-test", seed).expect("descriptor"),
            CalendarConfig::earthlike(),
        )
        .expect("world");
        world.bring_online().expect("online");
        for x in -1..=1 {
            for z in -1..=1 {
                world
                    .load_or_generate(ChunkCoord::new(x, z))
                    .expect("generate");
            }
        }
        world
    }

    fn block(world: &World, path: &str) -> BlockStateId {
        world
            .block_id(&Identifier::parse(path).expect("identifier"))
            .expect("registered")
    }

    /// Every resident column, as `(position, state)` over a coarse lattice, so
    /// two worlds can be compared without comparing their internals.
    fn probe(world: &World) -> Vec<(BlockPos, BlockStateId)> {
        let shape = world.descriptor().shape;
        let mut out = Vec::new();
        let mut coords = world.loaded_chunks();
        coords.sort_unstable_by_key(|c| (c.x, c.z));
        for coord in coords {
            for local_x in (0..shape.size_x() as i64).step_by(7) {
                for local_z in (0..shape.size_z() as i64).step_by(7) {
                    let x = coord.x * shape.size_x() as i64 + local_x;
                    let z = coord.z * shape.size_z() as i64 + local_z;
                    for y in (world.descriptor().bounds.min_y..=200).step_by(13) {
                        let position = BlockPos::new(x, y, z);
                        out.push((position, world.get_block(position).expect("in bounds")));
                    }
                }
            }
        }
        out
    }

    #[test]
    fn a_world_survives_a_write_and_read_cycle() {
        let dir = TempDir::new("roundtrip");
        let store = RegionStore::with_shape(&dir.0, small_regions());

        let mut original = built_world(0xC0FFEE);
        original
            .clock_mut()
            .advance_by(TimeScale::Hour, 7)
            .expect("advance");
        let stone = block(&original, "nexora:block/stone");
        for position in [
            BlockPos::new(5, 200, 5),
            BlockPos::new(-20, 64, -20),
            BlockPos::new(31, 100, 31),
        ] {
            original.set_block(position, stone).expect("edit");
        }
        original
            .set_block(BlockPos::new(0, 40, 0), AIR)
            .expect("edit");

        store.write_all(&mut original).expect("write");
        let restored = store.read().expect("read");

        assert_eq!(restored.descriptor(), original.descriptor());
        assert_eq!(restored.clock().now(), original.clock().now());
        assert_eq!(restored.chunk_count(), original.chunk_count());
        assert_eq!(probe(&restored), probe(&original));
    }

    #[test]
    fn columns_are_filed_under_the_region_that_stores_them() {
        let dir = TempDir::new("grouping");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(1);
        let report = store.write_all(&mut world).expect("write");

        // Columns -1..=1 on each axis, two per region: -1 lands in region -1,
        // and 0 and 1 share region 0.
        assert_eq!(
            report.written,
            vec![
                RegionCoord::new(-1, -1),
                RegionCoord::new(-1, 0),
                RegionCoord::new(0, -1),
                RegionCoord::new(0, 0),
            ]
        );
        assert!(report.skipped.is_empty());
        assert_eq!(store.regions_on_disk().expect("list"), report.written);
        assert_eq!(
            store
                .read_region(&world, RegionCoord::new(-1, -1))
                .expect("read")
                .len(),
            1,
            "the corner region holds one column"
        );
        assert_eq!(
            store
                .read_region(&world, RegionCoord::new(0, 0))
                .expect("read")
                .len(),
            4,
            "the origin region holds a two-by-two block of columns"
        );
    }

    #[test]
    fn an_unchanged_region_is_not_rewritten() {
        let dir = TempDir::new("skip");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(2);
        store.write_all(&mut world).expect("first write");

        let before: Vec<(RegionCoord, Vec<u8>)> = store
            .regions_on_disk()
            .expect("list")
            .into_iter()
            .map(|r| (r, fs::read(store.region_path(r)).expect("read file")))
            .collect();

        // One block, in the region that holds column (-1, -1).
        let stone = block(&world, "nexora:block/stone");
        world
            .set_block(BlockPos::new(-20, 90, -20), stone)
            .expect("edit");

        let report = store.write_dirty(&mut world).expect("second write");
        assert_eq!(report.written, vec![RegionCoord::new(-1, -1)]);
        assert_eq!(
            report.skipped,
            vec![
                RegionCoord::new(-1, 0),
                RegionCoord::new(0, -1),
                RegionCoord::new(0, 0),
            ]
        );

        for (region, bytes) in before {
            let now = fs::read(store.region_path(region)).expect("read file");
            if region == RegionCoord::new(-1, -1) {
                assert_ne!(now, bytes, "the edited region must have been rewritten");
            } else {
                assert_eq!(now, bytes, "an untouched region must be byte-identical");
            }
        }
    }

    #[test]
    fn writing_only_the_dirty_regions_reads_back_the_same_world() {
        let dir = TempDir::new("equivalent");
        let incremental = RegionStore::with_shape(dir.0.join("incremental"), small_regions());
        let full = RegionStore::with_shape(dir.0.join("full"), small_regions());

        let mut world = built_world(3);
        incremental.write_all(&mut world).expect("seed");

        let stone = block(&world, "nexora:block/stone");
        for position in [BlockPos::new(-20, 90, -20), BlockPos::new(5, 95, 5)] {
            world.set_block(position, stone).expect("edit");
        }

        let report = incremental.write_dirty(&mut world).expect("incremental");
        assert_eq!(report.written.len(), 2, "two regions held an edit");
        assert_eq!(report.skipped.len(), 2);

        full.write_all(&mut world).expect("full");

        let from_incremental = incremental.read().expect("read incremental");
        let from_full = full.read().expect("read full");
        assert_eq!(probe(&from_incremental), probe(&from_full));
        assert_eq!(probe(&from_incremental), probe(&world));
    }

    #[test]
    fn a_region_with_no_file_is_written_even_when_nothing_changed() {
        let dir = TempDir::new("newcomer");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(4);
        store.write_all(&mut world).expect("first write");

        // Read a column back from disk, which arrives clean, and put it in a
        // region that has no file. Nothing is dirty, and it must still land.
        let mut fresh = store.read().expect("read");
        let restored = store
            .read_region(&fresh, RegionCoord::new(0, 0))
            .expect("read region");
        assert!(restored.iter().all(|c| !c.is_dirty()));

        fresh
            .load_or_generate(ChunkCoord::new(9, 9))
            .expect("generate");
        fresh.mark_saved(ChunkCoord::new(9, 9));
        assert!(!fresh
            .chunk(ChunkCoord::new(9, 9))
            .expect("resident")
            .is_dirty());

        let report = store.write_dirty(&mut fresh).expect("second write");
        assert_eq!(report.written, vec![RegionCoord::new(4, 4)]);
        assert!(store.region_path(RegionCoord::new(4, 4)).exists());
    }

    #[test]
    fn a_region_that_left_the_resident_set_stays_on_disk() {
        let dir = TempDir::new("eviction");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(5);
        let stone = block(&world, "nexora:block/stone");
        world
            .set_block(BlockPos::new(-20, 90, -20), stone)
            .expect("edit");
        store.write_all(&mut world).expect("write");

        // The corner region's only column leaves the resident set.
        world
            .unload_chunk(ChunkCoord::new(-1, -1))
            .expect("resident");
        assert!(world.chunk(ChunkCoord::new(-1, -1)).is_none());

        let report = store.write_all(&mut world).expect("rewrite");
        assert!(!report.written.contains(&RegionCoord::new(-1, -1)));
        assert!(
            store.region_path(RegionCoord::new(-1, -1)).exists(),
            "an unloaded region is not a deleted region"
        );

        let restored = store.read().expect("read");
        assert_eq!(restored.chunk_count(), 9, "the unloaded column came back");
        assert_eq!(
            restored
                .get_block(BlockPos::new(-20, 90, -20))
                .expect("in bounds"),
            stone
        );
    }

    #[test]
    fn regions_ahead_of_the_header_load_with_the_regions_and_the_older_clock() {
        let dir = TempDir::new("crash");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(6);
        store.write_all(&mut world).expect("first write");
        let old_header = fs::read(store.header_path()).expect("header");

        world
            .clock_mut()
            .advance_by(TimeScale::Hour, 5)
            .expect("advance");
        let stone = block(&world, "nexora:block/stone");
        world
            .set_block(BlockPos::new(-20, 90, -20), stone)
            .expect("edit");
        store.write_dirty(&mut world).expect("second write");

        // What a crash between the regions and the header leaves behind.
        fs::write(store.header_path(), &old_header).expect("restore old header");

        let restored = store.read().expect("read");
        assert_eq!(
            restored
                .get_block(BlockPos::new(-20, 90, -20))
                .expect("in bounds"),
            stone,
            "the block that reached its region file is there"
        );
        assert!(
            restored.clock().now() < world.clock().now(),
            "and the clock is the older one, which replay can carry forward"
        );
    }

    #[test]
    fn a_region_file_reads_without_the_header() {
        let dir = TempDir::new("standalone");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(7);
        let stone = block(&world, "nexora:block/stone");
        world
            .set_block(BlockPos::new(5, 95, 5), stone)
            .expect("edit");
        store.write_all(&mut world).expect("write");

        fs::remove_file(store.header_path()).expect("remove header");
        let columns = store
            .read_region(&world, RegionCoord::new(0, 0))
            .expect("a region file stands on its own");
        assert_eq!(columns.len(), 4);
        assert!(store.read().is_err(), "but the world still needs a header");
    }

    #[test]
    fn an_absent_region_reads_as_no_columns() {
        let dir = TempDir::new("absent");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(8);
        store.write_all(&mut world).expect("write");
        assert!(store
            .read_region(&world, RegionCoord::new(50, 50))
            .expect("not an error")
            .is_empty());
    }

    #[test]
    fn a_stray_save_file_in_the_region_directory_is_reported() {
        let dir = TempDir::new("stray");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(9);
        store.write_all(&mut world).expect("write");

        let stray = dir.0.join(REGION_DIR).join("notes.nxsv");
        fs::write(&stray, b"not a region").expect("write stray");
        let error = store
            .regions_on_disk()
            .expect_err("a save file that is not a region is a corrupt store");
        assert!(error.to_string().contains("notes"), "got: {error}");
    }

    #[test]
    fn a_quarantined_region_is_not_mistaken_for_a_region() {
        let dir = TempDir::new("quarantine");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(10);
        store.write_all(&mut world).expect("write");

        let listed = store.regions_on_disk().expect("list");
        let quarantined = store
            .region_path(RegionCoord::new(0, 0))
            .with_extension("nxsv.quarantine.0");
        fs::write(&quarantined, b"damaged").expect("write quarantine");
        assert_eq!(
            store.regions_on_disk().expect("list"),
            listed,
            "a quarantined copy is evidence, not a region"
        );
    }

    #[test]
    fn a_region_address_survives_its_file_name_in_both_signs() {
        for region in [
            RegionCoord::new(0, 0),
            RegionCoord::new(-1, 2),
            RegionCoord::new(3, -4),
            RegionCoord::new(-5, -6),
            RegionCoord::new(i64::MIN, i64::MAX),
        ] {
            let store = RegionStore::new("root");
            let path = store.region_path(region);
            let stem = path.file_stem().and_then(|s| s.to_str()).expect("stem");
            assert_eq!(
                parse_region_stem(stem, &path).expect("parses"),
                region,
                "round trip through {stem}"
            );
        }
    }

    #[test]
    fn a_stored_column_merges_into_its_region_without_displacing_the_others() {
        let dir = TempDir::new("merge");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(12);
        store.write_all(&mut world).expect("write");

        // The column leaves the world entirely, the way an eviction takes it,
        // and is written back on its own.
        let evicted = ChunkCoord::new(0, 0);
        let mut carried = world.unload_chunk(evicted).expect("resident");
        let stone = block(&world, "nexora:block/stone");
        carried
            .set(BlockPos::new(3, 90, 3), stone, world.clock().now())
            .expect("edit the evicted column");

        let report = store
            .store_columns(&world, vec![carried])
            .expect("store the evicted column");
        assert_eq!(report.written, vec![RegionCoord::new(0, 0)]);

        // Region (0,0) held four columns; three are still in the world and one
        // is not. All four must still be in the file.
        let back = store
            .read_region(&world, RegionCoord::new(0, 0))
            .expect("read");
        assert_eq!(back.len(), 4, "merging must not drop the other columns");
        let restored = back
            .iter()
            .find(|chunk| chunk.coord() == evicted)
            .expect("the stored column is there");
        assert_eq!(
            restored.get(BlockPos::new(3, 90, 3)).expect("in range"),
            stone
        );

        // And the whole world still reads back, evicted column included.
        assert_eq!(store.read().expect("read").chunk_count(), 9);
    }

    #[test]
    fn storing_a_column_twice_keeps_the_second_write() {
        let dir = TempDir::new("overwrite");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(13);
        store.write_all(&mut world).expect("write");

        let coord = ChunkCoord::new(1, 1);
        let stone = block(&world, "nexora:block/stone");
        let grass = block(&world, "nexora:block/grass");
        let at = BlockPos::new(35, 95, 35);
        let now = world.clock().now();

        let mut first = world.unload_chunk(coord).expect("resident");
        first.set(at, stone, now).expect("edit");
        store
            .store_columns(&world, vec![first.clone()])
            .expect("first");

        let mut second = first;
        second.set(at, grass, now).expect("edit again");
        store.store_columns(&world, vec![second]).expect("second");

        let back = store
            .read_column(&world, coord)
            .expect("read")
            .expect("stored");
        assert_eq!(back.get(at).expect("in range"), grass);
    }

    #[test]
    fn a_column_the_store_has_never_seen_reads_as_nothing() {
        let dir = TempDir::new("unseen");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(14);
        store.write_all(&mut world).expect("write");

        // A region with a file, but not this column.
        assert!(store
            .read_column(&world, ChunkCoord::new(0, 0))
            .expect("read")
            .is_some());
        // A region with no file at all.
        assert!(store
            .read_column(&world, ChunkCoord::new(40, 40))
            .expect("read")
            .is_none());
    }

    #[test]
    fn a_column_stored_into_a_region_with_no_file_creates_one() {
        let dir = TempDir::new("firstcolumn");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        let mut world = built_world(15);
        world
            .load_or_generate(ChunkCoord::new(20, 20))
            .expect("generate");
        let carried = world
            .unload_chunk(ChunkCoord::new(20, 20))
            .expect("resident");

        store.store_columns(&world, vec![carried]).expect("store");
        assert!(store.region_path(RegionCoord::new(10, 10)).exists());
        assert!(store
            .read_column(&world, ChunkCoord::new(20, 20))
            .expect("read")
            .is_some());
    }

    #[test]
    fn reopening_a_store_with_a_different_region_extent_is_refused() {
        let dir = TempDir::new("extent");
        let mut world = built_world(11);
        RegionStore::with_shape(&dir.0, small_regions())
            .write_all(&mut world)
            .expect("write");

        let other = RegionStore::with_shape(&dir.0, RegionShape::new(4).expect("four"));
        let read_error = other.read().expect_err("the layout does not match");
        assert!(
            read_error.to_string().contains("extent"),
            "got: {read_error}"
        );
        let write_error = other
            .write_all(&mut world)
            .expect_err("and writing into it would file columns under the wrong address");
        assert!(
            write_error.to_string().contains("extent"),
            "got: {write_error}"
        );
    }

    #[test]
    fn content_blocks_read_back_from_regions_and_their_absence_is_named() {
        let basalt = Identifier::parse("nexora:block/stone/basalt").unwrap();
        let content = [(basalt.clone(), BlockDefinition { solid: true })];
        let mut world = World::create_with(
            WorldDescriptor::new("region-content", 3).unwrap(),
            CalendarConfig::earthlike(),
            &content,
        )
        .unwrap();
        world.bring_online().unwrap();
        world.load_or_generate(ChunkCoord::new(0, 0)).unwrap();
        let state = world.block_id(&basalt).unwrap();
        let at = BlockPos::new(3, 100, 3);
        world.set_block(at, state).unwrap();

        let dir = TempDir::new("content");
        let store = RegionStore::with_shape(&dir.0, small_regions());
        store.write_all(&mut world).unwrap();

        let reopened = store.read_with(&content).expect("the content is present");
        assert_eq!(
            reopened.get_block(at).unwrap(),
            reopened.block_id(&basalt).unwrap()
        );

        // A build without the content refuses by name, as `persist::load` does.
        let err = store.read().expect_err("basalt is not a built-in");
        assert!(
            err.to_string().contains("nexora:block/stone/basalt"),
            "{err}"
        );
    }
}
