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

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::BlockPos;
use nexora_persistence::codec::{Reader, Writer};
use nexora_persistence::journal::{Damage, Replay};

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
    let mut applied = 0usize;
    let mut skipped = Vec::new();

    for (index, bytes) in replay.records.iter().enumerate() {
        let Ok(record) = EditRecord::decode(bytes) else {
            skipped.push(SkippedEdit {
                index,
                reason: SkipReason::Undecodable,
            });
            continue;
        };

        match record {
            EditRecord::SetBlock { position, block } => {
                let Ok(state) = world.block_id(&block) else {
                    skipped.push(SkippedEdit {
                        index,
                        reason: SkipReason::UnknownBlock(block),
                    });
                    continue;
                };
                if world.set_block(position, state).is_err() {
                    skipped.push(SkippedEdit {
                        index,
                        reason: SkipReason::NotWritable(position),
                    });
                    continue;
                }
                applied += 1;
            }
        }
    }

    Ok(RecoveryReport {
        applied,
        skipped,
        damage: replay.damage,
    })
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

    fn replay_of(records: Vec<Vec<u8>>, damage: Option<Damage>) -> Replay {
        Replay { records, damage }
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
