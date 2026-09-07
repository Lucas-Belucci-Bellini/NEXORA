//! `Snapshot + Journal → Recovery`, end to end, against a real world.
//!
//! `NEXORA SAVE FORMAT AND COMPATIBILITY.md` lists these among its **testes
//! obrigatórios**: *"crash durante save"*, *"corrupção parcial"* and
//! *"recuperação de journal"*. They are integration tests rather than unit
//! tests because the property under test spans the container, the journal and
//! the world — each of which is already correct on its own, and none of which
//! proves that a crashed process comes back with its edits.
//!
//! A crash is simulated by **truncating the journal mid-record**, which is what
//! a process death during an append actually leaves behind.

use std::fs;
use std::path::PathBuf;

use nexora_foundation::ident::Identifier;
use nexora_foundation::spatial::{BlockPos, ChunkCoord};
use nexora_foundation::time::CalendarConfig;
use nexora_persistence::journal::{self, Damage, Journal, SnapshotId};
use nexora_persistence::SaveContainer;
use nexora_world::persist;
use nexora_world::recovery::{self, EditRecord, SkipReason};
use nexora_world::voxel::AIR;
use nexora_world::world::{World, WorldDescriptor};

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("nexora-recovery-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("scratch");
        Self(path)
    }

    fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn world() -> World {
    let descriptor = WorldDescriptor::new("crash-recovery", 0x5EED_5EED).expect("valid");
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

/// Save a world and return the snapshot bytes plus the id a journal binds to.
fn snapshot(world: &World) -> (Vec<u8>, SnapshotId) {
    let container = persist::save(world).expect("saved");
    let bytes = container.encode();
    let id = SnapshotId::of(world.descriptor().id.0, &bytes);
    (bytes, id)
}

fn reload(bytes: &[u8]) -> World {
    let container = SaveContainer::decode(bytes).expect("decodes");
    persist::load(&container).expect("loads")
}

#[test]
fn edits_made_after_a_snapshot_survive_a_crash_mid_journal() {
    // The whole point of DEBT-0001: without a journal these edits are simply
    // gone, because the snapshot predates them.
    let scratch = Scratch::new("survives");
    let path = scratch.file("world.nxjr");

    let mut live = world();
    let (bytes, id) = snapshot(&live);

    // Three edits after the checkpoint, journalled as they happen.
    let positions = [
        BlockPos::new(1, 200, 1),
        BlockPos::new(2, 200, 2),
        BlockPos::new(3, 200, 3),
    ];
    let state = live.block_id(&stone()).expect("registered");

    let mut log = Journal::create(&path, id).expect("created");
    for position in positions {
        live.set_block(position, state).expect("written");
        log.append_durable(
            &EditRecord::SetBlock {
                position,
                block: stone(),
            }
            .encode(),
        )
        .expect("journalled");
    }
    // A fourth edit is journalled but the process dies part-way through the
    // append: write the record, then cut the file short.
    let doomed = BlockPos::new(4, 200, 4);
    log.append(
        &EditRecord::SetBlock {
            position: doomed,
            block: stone(),
        }
        .encode(),
    )
    .expect("appended");
    log.sync().expect("flushed");
    drop(log);

    let full = fs::read(&path).expect("read");
    fs::write(&path, &full[..full.len() - 4]).expect("simulate the crash");

    // --- restart -----------------------------------------------------------
    let mut recovered = reload(&bytes);
    for position in positions {
        assert_eq!(
            recovered.get_block(position).expect("resident"),
            AIR,
            "the snapshot predates the edits, as expected"
        );
    }

    let replay = journal::replay(&path, id).expect("a torn tail is recoverable");
    let report = recovery::apply(&mut recovered, &replay).expect("applied");

    assert_eq!(report.applied, 3, "every durable edit came back");
    assert!(
        report
            .damage
            .expect("the tail was torn")
            .is_expected_after_a_crash(),
        "a truncated journal is a crash, not corruption"
    );
    for position in positions {
        assert_ne!(
            recovered.get_block(position).expect("resident"),
            AIR,
            "edit at {position:?} did not survive"
        );
    }
    assert_eq!(
        recovered.get_block(doomed).expect("resident"),
        AIR,
        "the record lost to the crash must not appear"
    );
}

#[test]
fn a_journal_belonging_to_a_different_snapshot_is_refused() {
    // The dangerous case. These records would apply cleanly and produce a world
    // that never existed; losing the journal is recoverable, that is not.
    let scratch = Scratch::new("wrong-snapshot");
    let path = scratch.file("world.nxjr");

    let mut live = world();
    let (_, before) = snapshot(&live);

    let mut log = Journal::create(&path, before).expect("created");
    log.append_durable(
        &EditRecord::SetBlock {
            position: BlockPos::new(1, 200, 1),
            block: stone(),
        }
        .encode(),
    )
    .expect("journalled");
    drop(log);

    // The world moves on and is checkpointed again. The old journal now
    // describes changes relative to a snapshot that is no longer current.
    let state = live.block_id(&stone()).expect("registered");
    live.set_block(BlockPos::new(9, 200, 9), state)
        .expect("written");
    let (_, after) = snapshot(&live);
    assert_ne!(before, after);

    let error = journal::replay(&path, after).expect_err("must refuse");
    assert!(error.to_string().contains("different snapshot"), "{error}");
}

#[test]
fn partial_corruption_is_reported_and_never_applied_as_an_edit() {
    // "corrupção parcial", from the mandatory list. A flipped bit inside a
    // complete record is storage lying -- replay stops, and the world is left
    // holding only what was verifiably written.
    let scratch = Scratch::new("corruption");
    let path = scratch.file("world.nxjr");

    let live = world();
    let (bytes, id) = snapshot(&live);

    let good = BlockPos::new(1, 200, 1);
    let after_the_damage = BlockPos::new(2, 200, 2);

    let mut log = Journal::create(&path, id).expect("created");
    for position in [good, after_the_damage] {
        log.append_durable(
            &EditRecord::SetBlock {
                position,
                block: stone(),
            }
            .encode(),
        )
        .expect("journalled");
    }
    drop(log);

    // Flip a bit inside the last record's payload.
    let mut raw = fs::read(&path).expect("read");
    let last = raw.len() - 1;
    raw[last] ^= 0b0000_0001;
    fs::write(&path, &raw).expect("corrupt");

    let replay = journal::replay(&path, id).expect("damage is reported, not thrown");
    assert!(matches!(replay.damage, Some(Damage::Corruption { .. })));
    assert!(
        !replay.damage.expect("damaged").is_expected_after_a_crash(),
        "corruption must not be filed as a routine crash"
    );

    let mut recovered = reload(&bytes);
    let report = recovery::apply(&mut recovered, &replay).expect("applied");
    assert_eq!(report.applied, 1);
    assert!(!report.is_complete());
    assert_ne!(
        recovered.get_block(good).expect("resident"),
        AIR,
        "the verified record still applies"
    );
    assert_eq!(
        recovered.get_block(after_the_damage).expect("resident"),
        AIR,
        "nothing past the damage may be applied"
    );
}

#[test]
fn a_corrupt_journal_can_be_quarantined_and_the_snapshot_still_loads() {
    // detect -> quarantine -> recover latest valid snapshot -> report.
    // Losing the journal costs the edits since the checkpoint; it must never
    // cost the world.
    let scratch = Scratch::new("quarantine");
    let path = scratch.file("world.nxjr");

    let live = world();
    let (bytes, id) = snapshot(&live);

    let mut log = Journal::create(&path, id).expect("created");
    log.append_durable(b"not a valid edit record at all")
        .expect("journalled");
    drop(log);

    // Wreck the header so the file is not a journal any more.
    let mut raw = fs::read(&path).expect("read");
    raw[0] = b'X';
    fs::write(&path, &raw).expect("wreck");

    assert!(journal::replay(&path, id).is_err());
    let moved = nexora_persistence::quarantine(&path).expect("quarantined");
    assert!(moved.exists(), "the evidence is kept");
    assert!(!path.exists());

    // The world still comes back, at the checkpoint.
    let recovered = reload(&bytes);
    assert_eq!(
        recovered.descriptor().id,
        live.descriptor().id,
        "the snapshot is untouched by the journal's failure"
    );
}

#[test]
fn replaying_the_same_journal_twice_lands_on_the_same_world() {
    // Recovery has to be deterministic -- NEXORA FAILURE AND RECOVERY
    // ARCHITECTURE.md requires it where the subsystem does, and the world does.
    let scratch = Scratch::new("deterministic");
    let path = scratch.file("world.nxjr");

    let live = world();
    let (bytes, id) = snapshot(&live);

    let mut log = Journal::create(&path, id).expect("created");
    for step in 0..8i64 {
        log.append_durable(
            &EditRecord::SetBlock {
                position: BlockPos::new(step, 200, step),
                block: stone(),
            }
            .encode(),
        )
        .expect("journalled");
    }
    drop(log);

    let replay = journal::replay(&path, id).expect("read");

    let mut first = reload(&bytes);
    recovery::apply(&mut first, &replay).expect("applied");

    let mut second = reload(&bytes);
    recovery::apply(&mut second, &replay).expect("applied");

    assert_eq!(
        persist::save(&first).expect("saved").encode(),
        persist::save(&second).expect("saved").encode(),
        "two recoveries from one journal produced different worlds"
    );
}

#[test]
fn a_recovered_world_can_be_checkpointed_and_its_journal_started_again() {
    // The loop has to close: recover, snapshot, journal onward. Otherwise
    // recovery is a one-shot trick rather than part of the save cycle.
    let scratch = Scratch::new("cycle");

    let live = world();
    let (bytes, first_id) = snapshot(&live);

    let first_path = scratch.file("first.nxjr");
    let mut log = Journal::create(&first_path, first_id).expect("created");
    log.append_durable(
        &EditRecord::SetBlock {
            position: BlockPos::new(1, 200, 1),
            block: stone(),
        }
        .encode(),
    )
    .expect("journalled");
    drop(log);

    let mut recovered = reload(&bytes);
    let replay = journal::replay(&first_path, first_id).expect("read");
    assert_eq!(
        recovery::apply(&mut recovered, &replay)
            .expect("applied")
            .applied,
        1
    );

    // New checkpoint, new journal, bound to the new snapshot.
    let (next_bytes, next_id) = snapshot(&recovered);
    assert_ne!(first_id, next_id, "the checkpoint moved");

    let next_path = scratch.file("second.nxjr");
    let mut log = Journal::create(&next_path, next_id).expect("created");
    log.append_durable(
        &EditRecord::SetBlock {
            position: BlockPos::new(2, 200, 2),
            block: stone(),
        }
        .encode(),
    )
    .expect("journalled");
    drop(log);

    let mut again = reload(&next_bytes);
    let replay = journal::replay(&next_path, next_id).expect("read");
    assert_eq!(
        recovery::apply(&mut again, &replay)
            .expect("applied")
            .applied,
        1
    );
    // Both edits are present: the first through the snapshot, the second
    // through the new journal.
    for position in [BlockPos::new(1, 200, 1), BlockPos::new(2, 200, 2)] {
        assert_ne!(again.get_block(position).expect("resident"), AIR);
    }

    // And the stale journal is refused against the new checkpoint.
    assert!(journal::replay(&first_path, next_id).is_err());
}

#[test]
fn an_edit_naming_content_this_build_does_not_have_is_reported_not_guessed() {
    // Missing Content, from the save format document: a removed mod must
    // produce a controlled state, never silent corruption.
    let scratch = Scratch::new("missing-content");
    let path = scratch.file("world.nxjr");

    let live = world();
    let (bytes, id) = snapshot(&live);

    let mut log = Journal::create(&path, id).expect("created");
    log.append_durable(
        &EditRecord::SetBlock {
            position: BlockPos::new(1, 200, 1),
            block: Identifier::parse("removedmod:block/reactor").expect("valid"),
        }
        .encode(),
    )
    .expect("journalled");
    drop(log);

    let mut recovered = reload(&bytes);
    let replay = journal::replay(&path, id).expect("read");
    let report = recovery::apply(&mut recovered, &replay).expect("reported");

    assert_eq!(report.applied, 0);
    assert!(matches!(
        report.skipped.first().map(|s| &s.reason),
        Some(SkipReason::UnknownBlock(_))
    ));
}
