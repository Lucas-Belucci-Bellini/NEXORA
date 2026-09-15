//! Edits recorded between snapshots, and the world they rebuild.
//!
//! `NEXORA SAVE FORMAT AND COMPATIBILITY.md` defines recovery as
//! `Snapshot + Journal → Recovery`. [`nexora_persistence::journal`] frames and
//! checksums opaque records; this module decides what a record *means* for a
//! voxel world, which is the same split the crate already has for sections and
//! save containers.
//!
//! # An edit is recorded by identifier, not by runtime id
//!
//! A `BlockStateId` is assigned when a registry is built, and a journal outlives
//! the process that wrote it. Recording `17` and replaying it into a session
//! where `17` means something else would place the wrong block, silently — the
//! same trap `persist.rs` avoids by remapping through identifiers. So a record
//! carries `nexora:block/stone`, and replay resolves it against the registry it
//! is replaying into. A block whose identifier is no longer registered is
//! **reported**, never guessed.
//!
//! # This is not the chunk journal
//!
//! `Chunk` keeps a bounded in-memory ring of recent edits for diagnostics
//! (`DEBT-0004`). It is not durable, it drops its oldest entry under pressure,
//! and it is not what recovery reads. These are different mechanisms with
//! unfortunately similar names, and conflating them would mean trusting a lossy
//! debug aid to reconstruct a world.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{BlockPos, ChunkCoord, ChunkShape};
use nexora_persistence::codec::{Reader, Writer};
use nexora_persistence::journal::{Damage, Replay};

// Both appear in this module's public signatures, so a caller has to be able to
// name them. Re-exported rather than left for every caller to reach into
// `nexora-persistence` for a type it only sees through this API.
pub use nexora_persistence::journal::{Damage as JournalDamage, Replay as JournalReplay};

use crate::world::World;

/// One recorded change to the world.
///
/// `#[non_exhaustive]` because entity and metadata edits join it later; a new
/// variant must not break a caller that already matches on these.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditRecord {
    /// A block was written.
    SetBlock {
        /// Where.
        position: BlockPos,
        /// Which block, by registry identifier rather than runtime id.
        block: Identifier,
    },
}

/// Tag byte for [`EditRecord::SetBlock`].
const TAG_SET_BLOCK: u8 = 1;

impl EditRecord {
    /// The column this edit writes into.
    ///
    /// Every edit that names a position names exactly one column, and that is
    /// what makes deferring safe: two records at the same position are
    /// necessarily in the same column, so keeping each column's records in
    /// journal order keeps the final state right, whatever order the columns
    /// arrive in. See [`PendingEdits`].
    #[must_use]
    pub fn column(&self, shape: ChunkShape) -> ChunkCoord {
        match self {
            Self::SetBlock { position, .. } => shape.section_of(*position).column(),
        }
    }
}

impl EditRecord {
    /// Encode this edit into journal record bytes.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut writer = Writer::new();
        match self {
            Self::SetBlock { position, block } => {
                writer.u8(TAG_SET_BLOCK);
                writer.i64(position.x);
                writer.i64(position.y);
                writer.i64(position.z);
                writer.string(&block.to_string());
            }
        }
        writer.finish()
    }

    /// Decode one edit from journal record bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the record is truncated, carries an unknown tag,
    /// or holds an identifier that is not well formed.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        let tag = reader.u8()?;
        match tag {
            TAG_SET_BLOCK => {
                let x = reader.i64()?;
                let y = reader.i64()?;
                let z = reader.i64()?;
                let block = Identifier::parse(reader.string()?)?;
                reader.expect_exhausted()?;
                Ok(Self::SetBlock {
                    position: BlockPos::new(x, y, z),
                    block,
                })
            }
            unknown => Err(Error::new(
                Domain::Save,
                "world-recovery",
                "journal record carries an unknown edit tag",
            )
            .with_recovery(Recovery::Quarantine)
            .with_context("tag", unknown.to_string())),
        }
    }
}

/// What replaying a journal into a world achieved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryReport {
    /// Edits applied to the world.
    pub applied: usize,
    /// Records that could not be applied, and why.
    ///
    /// Non-fatal but never silent: an edit into a chunk that is not resident is
    /// a legitimate outcome of a partial load, while an unregistered block is a
    /// content problem. Both are surfaced rather than counted as success.
    pub skipped: Vec<SkippedEdit>,
    /// What the journal read reported, if anything was wrong with the file.
    pub damage: Option<Damage>,
    /// The skipped edits that are only waiting for their column, indexed by it.
    ///
    /// These also appear in `skipped` — the report still says what did not make
    /// it into the world. What this adds is the ability to finish the job:
    /// hand it to the thing that brings columns in and each edit is applied the
    /// moment its column arrives. `DEBT-0024`.
    pub deferred: PendingEdits,
}

/// Edits a replay could not apply because their column was not resident,
/// indexed so they can be applied when it becomes so.
///
/// **Why this can be deferred at all.** `apply` insists on journal order,
/// because an earlier edit written over a later one produces a world that never
/// existed. Deferring looks like it breaks that, and does not: two records that
/// can overwrite each other are at the same position, and the same position is
/// in the same column. Order is preserved *within* each column, and between
/// columns there is nothing to preserve.
///
/// **What it does not do.** A column nobody ever brings in keeps its edits
/// here, unapplied. That is the honest outcome and it is countable —
/// [`PendingEdits::len`] — rather than a silence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PendingEdits {
    by_column: BTreeMap<ChunkCoord, Vec<EditRecord>>,
    applied: usize,
}

impl PendingEdits {
    /// How many edits are still waiting.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_column.values().map(Vec::len).sum()
    }

    /// Whether nothing is waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_column.is_empty()
    }

    /// How many deferred edits have been applied through this index so far.
    #[must_use]
    pub const fn applied(&self) -> usize {
        self.applied
    }

    /// The columns something is waiting on, in a stable order.
    pub fn columns(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.by_column.keys().copied()
    }

    /// Whether anything is waiting on one column.
    #[must_use]
    pub fn waiting_on(&self, column: ChunkCoord) -> usize {
        self.by_column.get(&column).map_or(0, Vec::len)
    }

    fn push(&mut self, column: ChunkCoord, record: EditRecord) {
        self.by_column.entry(column).or_default().push(record);
    }

    /// Apply everything waiting on one column, in the order it was journalled.
    ///
    /// The column is cleared whether or not every edit lands: an edit that
    /// still cannot be applied with the column resident is not waiting on
    /// residency, and keeping it here would mean retrying it forever. Those
    /// come back in the returned report instead.
    ///
    /// # Errors
    ///
    /// Returns an error only when the world itself cannot be queried.
    pub fn apply_column(
        &mut self,
        world: &mut World,
        column: ChunkCoord,
    ) -> Result<ColumnRecovery> {
        let Some(records) = self.by_column.remove(&column) else {
            return Ok(ColumnRecovery::default());
        };
        let mut recovery = ColumnRecovery::default();
        for record in records {
            match apply_one(world, &record) {
                Ok(()) => recovery.applied += 1,
                Err(reason) => recovery.refused.push(reason),
            }
        }
        self.applied += recovery.applied;
        Ok(recovery)
    }
}

/// What applying one column's deferred edits achieved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColumnRecovery {
    /// Edits that landed.
    pub applied: usize,
    /// Edits that still could not be applied, with the column resident.
    pub refused: Vec<SkipReason>,
}

impl RecoveryReport {
    /// Whether every record in the journal was applied.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.skipped.is_empty() && self.damage.is_none()
    }
}

/// One record that replay could not apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedEdit {
    /// Its position in the journal.
    pub index: usize,
    /// Why it was skipped.
    pub reason: SkipReason,
}

/// Why an edit could not be applied.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkipReason {
    /// The record did not decode.
    Undecodable,
    /// The block identifier is not registered in this session.
    ///
    /// `NEXORA SAVE FORMAT AND COMPATIBILITY.md` calls this Missing Content and
    /// requires a controlled state rather than silent corruption — a removed
    /// mod is the ordinary cause.
    UnknownBlock(Identifier),
    /// The chunk holding the position is not resident, or the write was refused.
    NotWritable(BlockPos),
}

/// Apply a replayed journal to a world.
///
/// Records are applied **in order**: a journal replayed out of order would
/// apply an earlier edit over a later one and produce a world that never
/// existed.
///
/// # Errors
///
/// Returns an error only when the world itself cannot be queried. A record that
/// cannot be applied is reported in [`RecoveryReport::skipped`], because losing
/// one edit must not cost the rest of the journal.
pub fn apply(world: &mut World, replay: &Replay) -> Result<RecoveryReport> {
    let shape = world.descriptor().shape;
    let mut applied = 0usize;
    let mut skipped = Vec::new();
    let mut deferred = PendingEdits::default();

    for (index, bytes) in replay.records.iter().enumerate() {
        let Ok(record) = EditRecord::decode(bytes) else {
            skipped.push(SkippedEdit {
                index,
                reason: SkipReason::Undecodable,
            });
            continue;
        };

        match apply_one(world, &record) {
            Ok(()) => applied += 1,
            Err(reason) => {
                // A refusal that residency can undo is filed against the column
                // it is waiting for, as well as reported. Anything else — a
                // block this session does not know — will not improve by
                // waiting, so it is only reported. `DEBT-0024`.
                if matches!(reason, SkipReason::NotWritable(_)) {
                    deferred.push(record.column(shape), record);
                }
                skipped.push(SkippedEdit { index, reason });
            }
        }
    }

    Ok(RecoveryReport {
        applied,
        skipped,
        damage: replay.damage,
        deferred,
    })
}

/// Apply one decoded record, or say why it could not be.
fn apply_one(world: &mut World, record: &EditRecord) -> core::result::Result<(), SkipReason> {
    match record {
        EditRecord::SetBlock { position, block } => {
            let Ok(state) = world.block_id(block) else {
                return Err(SkipReason::UnknownBlock(block.clone()));
            };
            world
                .set_block(*position, state)
                .map(|_| ())
                .map_err(|_| SkipReason::NotWritable(*position))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::WorldDescriptor;
    use nexora_foundation::spatial::ChunkCoord;
    use nexora_foundation::time::CalendarConfig;
    use nexora_persistence::journal::SnapshotId;

    fn world() -> World {
        let descriptor = WorldDescriptor::new("recovery-test", 0xC0FFEE).expect("valid");
        let mut world = World::create(descriptor, CalendarConfig::earthlike()).expect("created");
        world.bring_online().expect("online");
        world
            .load_or_generate(ChunkCoord::new(0, 0))
            .expect("generated");
        world
    }

    fn stone() -> Identifier {
        Identifier::parse("nexora:block/stone").expect("valid")
    }

    fn air() -> Identifier {
        Identifier::parse("nexora:block/air").expect("valid")
    }

    fn replay_of(records: Vec<Vec<u8>>, damage: Option<Damage>) -> Replay {
        Replay { records, damage }
    }

    /// `DEBT-0024`. An edit whose column is not resident is not lost and is not
    /// silently dropped: it is filed against the column it needs, and applied
    /// the moment that column arrives.
    #[test]
    fn an_edit_waiting_on_a_column_is_applied_when_the_column_arrives() {
        let mut world = world();
        let away = ChunkCoord::new(9, 9);
        let position = world
            .descriptor()
            .shape
            .section_origin(nexora_foundation::spatial::SectionCoord::new(9, 2, 9))
            .expect("in range");
        assert!(
            world.chunk(away).is_none(),
            "the fixture must start without the column this edit needs"
        );

        let record = EditRecord::SetBlock {
            position,
            block: stone(),
        };
        let report = apply(&mut world, &replay_of(vec![record.encode()], None)).expect("replayed");

        // Reported as before: nothing about the honesty of the report changed.
        assert_eq!(report.applied, 0);
        assert_eq!(report.skipped.len(), 1);
        assert!(matches!(
            report.skipped[0].reason,
            SkipReason::NotWritable(_)
        ));
        // And now also recoverable.
        let mut deferred = report.deferred;
        assert_eq!(deferred.len(), 1);
        assert_eq!(deferred.waiting_on(away), 1);
        assert_eq!(deferred.columns().collect::<Vec<_>>(), vec![away]);

        world.load_or_generate(away).expect("column arrives");
        let recovered = deferred.apply_column(&mut world, away).expect("applied");
        assert_eq!(recovered.applied, 1);
        assert!(recovered.refused.is_empty());
        assert!(deferred.is_empty());
        assert_eq!(deferred.applied(), 1);

        let found = world.get_block(position).expect("resident now");
        assert_eq!(
            world.block_identifier(found),
            Some(stone()),
            "the deferred edit did not land"
        );
    }

    /// The order rule `apply` states has to survive being deferred. Two writes
    /// to one position can only be in one column, so keeping each column in
    /// journal order is enough — and this is the case that proves it: the last
    /// write must win, not the first.
    #[test]
    fn a_deferred_column_keeps_its_records_in_journal_order() {
        let mut world = world();
        let away = ChunkCoord::new(-3, 5);
        let position = world
            .descriptor()
            .shape
            .section_origin(nexora_foundation::spatial::SectionCoord::new(-3, 2, 5))
            .expect("in range");

        let first = EditRecord::SetBlock {
            position,
            block: stone(),
        };
        let last = EditRecord::SetBlock {
            position,
            block: air(),
        };
        let report = apply(
            &mut world,
            &replay_of(vec![first.encode(), last.encode()], None),
        )
        .expect("replayed");
        let mut deferred = report.deferred;
        assert_eq!(deferred.waiting_on(away), 2);

        world.load_or_generate(away).expect("column arrives");
        assert_eq!(
            deferred
                .apply_column(&mut world, away)
                .expect("applied")
                .applied,
            2
        );

        let found = world.get_block(position).expect("resident now");
        assert_eq!(
            world.block_identifier(found),
            Some(air()),
            "the second write must have landed over the first"
        );
    }

    /// A block this session does not know will not become known by waiting, so
    /// it is reported and not filed. Filing it would mean retrying it against
    /// every column that ever loads.
    #[test]
    fn an_unknown_block_is_not_deferred_because_waiting_cannot_help() {
        let mut world = world();
        let record = EditRecord::SetBlock {
            position: BlockPos::new(1, 2, 3),
            block: Identifier::parse("nexora:block/not-a-real-block").expect("valid"),
        };
        let report = apply(&mut world, &replay_of(vec![record.encode()], None)).expect("replayed");

        assert_eq!(report.skipped.len(), 1);
        assert!(matches!(
            report.skipped[0].reason,
            SkipReason::UnknownBlock(_)
        ));
        assert!(
            report.deferred.is_empty(),
            "an unknown block is not waiting on residency"
        );
    }

    /// Asking for a column nothing is waiting on is not an error, because the
    /// caller is residency and it does not know which columns are interesting.
    #[test]
    fn a_column_with_nothing_waiting_recovers_nothing_and_says_so() {
        let mut world = world();
        let mut deferred = PendingEdits::default();
        let recovered = deferred
            .apply_column(&mut world, ChunkCoord::new(0, 0))
            .expect("no error");
        assert_eq!(recovered, ColumnRecovery::default());
        assert_eq!(deferred.applied(), 0);
    }

    #[test]
    fn an_edit_record_round_trips() {
        let record = EditRecord::SetBlock {
            position: BlockPos::new(-4, 70, 9),
            block: stone(),
        };
        let decoded = EditRecord::decode(&record.encode()).expect("decodes");
        assert_eq!(decoded, record);
    }

    #[test]
    fn a_truncated_record_is_refused_rather_than_partially_read() {
        let bytes = EditRecord::SetBlock {
            position: BlockPos::new(1, 2, 3),
            block: stone(),
        }
        .encode();
        assert!(EditRecord::decode(&bytes[..bytes.len() - 2]).is_err());
    }

    #[test]
    fn an_unknown_tag_is_refused() {
        assert!(EditRecord::decode(&[0xFE]).is_err());
    }

    #[test]
    fn replaying_a_journal_reapplies_the_edits_it_recorded() {
        let mut world = world();
        let position = BlockPos::new(3, 200, 3);
        assert_eq!(
            world.get_block(position).expect("resident"),
            crate::voxel::AIR
        );

        let records = vec![EditRecord::SetBlock {
            position,
            block: stone(),
        }
        .encode()];

        let report = apply(&mut world, &replay_of(records, None)).expect("applied");
        assert_eq!(report.applied, 1);
        assert!(report.is_complete());
        assert_ne!(
            world.get_block(position).expect("resident"),
            crate::voxel::AIR
        );
    }

    #[test]
    fn edits_are_applied_in_order_so_the_last_write_wins() {
        // Out of order, the earlier edit would overwrite the later one and the
        // recovered world would never have existed.
        let mut world = world();
        let position = BlockPos::new(5, 200, 5);
        let dirt = Identifier::parse("nexora:block/dirt").expect("valid");

        let records = vec![
            EditRecord::SetBlock {
                position,
                block: stone(),
            }
            .encode(),
            EditRecord::SetBlock {
                position,
                block: dirt.clone(),
            }
            .encode(),
        ];

        let report = apply(&mut world, &replay_of(records, None)).expect("applied");
        assert_eq!(report.applied, 2);
        let final_state = world.get_block(position).expect("resident");
        assert_eq!(world.block_identifier(final_state), Some(dirt));
    }

    #[test]
    fn an_unregistered_block_is_reported_as_missing_content_not_guessed() {
        let mut world = world();
        let records = vec![EditRecord::SetBlock {
            position: BlockPos::new(1, 200, 1),
            block: Identifier::parse("removedmod:block/reactor").expect("valid"),
        }
        .encode()];

        let report = apply(&mut world, &replay_of(records, None)).expect("reported");
        assert_eq!(report.applied, 0);
        assert!(!report.is_complete());
        assert!(matches!(
            report.skipped.first().map(|s| &s.reason),
            Some(SkipReason::UnknownBlock(_))
        ));
    }

    #[test]
    fn one_bad_record_does_not_cost_the_rest_of_the_journal() {
        let mut world = world();
        let good = BlockPos::new(7, 200, 7);
        let records = vec![
            vec![0xFE],
            EditRecord::SetBlock {
                position: good,
                block: stone(),
            }
            .encode(),
        ];

        let report = apply(&mut world, &replay_of(records, None)).expect("applied");
        assert_eq!(report.applied, 1, "the good record still landed");
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(report.skipped[0].index, 0);
    }

    #[test]
    fn an_edit_into_a_chunk_that_is_not_resident_is_reported() {
        let mut world = world();
        let far = BlockPos::new(500_000, 70, 500_000);
        let records = vec![EditRecord::SetBlock {
            position: far,
            block: stone(),
        }
        .encode()];

        let report = apply(&mut world, &replay_of(records, None)).expect("reported");
        assert_eq!(report.applied, 0);
        assert!(matches!(
            report.skipped.first().map(|s| &s.reason),
            Some(SkipReason::NotWritable(_))
        ));
    }

    #[test]
    fn journal_damage_is_carried_into_the_recovery_report() {
        // The world's report has to say the journal was short, or a caller
        // cannot tell "recovered everything" from "recovered what survived".
        let mut world = world();
        let damage = Some(Damage::TornTail { trailing_bytes: 5 });
        let report = apply(&mut world, &replay_of(Vec::new(), damage)).expect("applied");
        assert_eq!(report.applied, 0);
        assert!(!report.is_complete());
        assert_eq!(report.damage, damage);
    }

    #[test]
    fn a_snapshot_id_is_derived_from_the_bytes_that_were_saved() {
        let one = SnapshotId::of(1, b"snapshot bytes");
        let same = SnapshotId::of(1, b"snapshot bytes");
        let different = SnapshotId::of(1, b"other bytes");
        assert_eq!(one, same);
        assert_ne!(one, different);
    }
}
