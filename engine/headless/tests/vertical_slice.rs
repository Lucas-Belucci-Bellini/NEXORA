//! End-to-end tests for the Phase 0 vertical slice.
//!
//! These run the same code path the `nexora-headless` binary runs, so a green
//! test suite means the slice itself is green - `NEXORA DEFINITION OF DONE.md`
//! asks for verification, not for a build that merely compiles.

use std::fs;
use std::path::PathBuf;

use nexora_headless::{format_report, run_slice, SliceConfig};

/// A scratch directory that removes itself.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nexora-slice-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create scratch directory");
        Self(path)
    }

    fn save(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn config(scratch: &Scratch, name: &str) -> SliceConfig {
    SliceConfig {
        radius: 1,
        save_path: scratch.save(name),
        verbose: false,
        ..SliceConfig::default()
    }
}

#[test]
fn the_slice_completes_and_verifies_itself() {
    let scratch = Scratch::new("complete");
    let config = config(&scratch, "world.nxsv");

    let report = run_slice(&config).expect("the vertical slice must complete");

    assert_eq!(
        report.chunks_generated, 9,
        "a radius of 1 covers a 3x3 area"
    );
    assert_eq!(report.probes_verified, report.blocks_edited);
    assert!(report.blocks_edited > 0);
    assert!(report.non_air_blocks > 0);
    assert!(report.save_bytes > 0);
    assert!(
        config.save_path.exists(),
        "the save must be on disk afterwards"
    );
}

#[test]
fn a_headless_run_never_enters_presentation() {
    let scratch = Scratch::new("phases");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");

    assert!(
        !report.phases.contains(&"presentation-running"),
        "a headless runtime has no presentation layer to start"
    );
    assert_eq!(report.phases.first(), Some(&"process-start"));
    assert_eq!(report.phases.last(), Some(&"process-exit"));
}

#[test]
fn the_same_seed_produces_the_same_world_twice() {
    // The reproducibility contract from
    // NEXORA WORLD GENERATION SEED AND REPRODUCIBILITY.md, checked end to end
    // rather than only at the generator.
    let scratch = Scratch::new("determinism");

    let first = run_slice(&SliceConfig {
        seed: 4_242,
        ..config(&scratch, "a.nxsv")
    })
    .expect("first");
    let second = run_slice(&SliceConfig {
        seed: 4_242,
        ..config(&scratch, "b.nxsv")
    })
    .expect("second");

    assert_eq!(first.world_id, second.world_id);
    assert_eq!(first.non_air_blocks, second.non_air_blocks);
    assert_eq!(first.storage_bytes, second.storage_bytes);
    assert_eq!(first.save_bytes, second.save_bytes);

    // The byte-for-byte comparison is the strong form: identical inputs must
    // produce an identical save, not merely an equivalent one.
    let a = fs::read(scratch.save("a.nxsv")).expect("read first save");
    let b = fs::read(scratch.save("b.nxsv")).expect("read second save");
    assert_eq!(a, b, "the same seed must produce a byte-identical save");
}

#[test]
fn a_different_seed_produces_a_different_world() {
    let scratch = Scratch::new("divergence");

    let first = run_slice(&SliceConfig {
        seed: 1,
        ..config(&scratch, "a.nxsv")
    })
    .expect("first");
    let second = run_slice(&SliceConfig {
        seed: 2,
        ..config(&scratch, "b.nxsv")
    })
    .expect("second");

    assert_ne!(first.world_id, second.world_id);
    let a = fs::read(scratch.save("a.nxsv")).expect("read first save");
    let b = fs::read(scratch.save("b.nxsv")).expect("read second save");
    assert_ne!(a, b);
}

#[test]
fn generation_does_not_depend_on_the_worker_count() {
    // Chunk generation runs across a worker pool. If any of it depended on
    // scheduling order, the resulting world would change with the thread count.
    let scratch = Scratch::new("threads");

    let single = run_slice(&SliceConfig {
        seed: 77,
        worker_threads: 1,
        ..config(&scratch, "one.nxsv")
    })
    .expect("single-threaded run");
    let many = run_slice(&SliceConfig {
        seed: 77,
        worker_threads: 8,
        ..config(&scratch, "many.nxsv")
    })
    .expect("multi-threaded run");

    assert_eq!(single.non_air_blocks, many.non_air_blocks);
    let a = fs::read(scratch.save("one.nxsv")).expect("read");
    let b = fs::read(scratch.save("many.nxsv")).expect("read");
    assert_eq!(a, b, "worker count must not change the generated world");
}

#[test]
fn rerunning_over_an_existing_save_replaces_it_safely() {
    let scratch = Scratch::new("rerun");
    let path = scratch.save("world.nxsv");

    run_slice(&SliceConfig {
        seed: 10,
        ..config(&scratch, "world.nxsv")
    })
    .expect("first run");
    let first = fs::read(&path).expect("read");

    run_slice(&SliceConfig {
        seed: 20,
        ..config(&scratch, "world.nxsv")
    })
    .expect("second run");
    let second = fs::read(&path).expect("read");

    assert_ne!(first, second, "the second run must have replaced the save");
    assert!(
        !path.with_file_name("world.nxsv.tmp").exists(),
        "no temporary file may be left behind"
    );
}

#[test]
fn a_zero_radius_world_still_completes() {
    let scratch = Scratch::new("minimal");
    let report = run_slice(&SliceConfig {
        radius: 0,
        ..config(&scratch, "world.nxsv")
    })
    .expect("slice");
    assert_eq!(report.chunks_generated, 1);
}

#[test]
fn the_report_renders_every_field() {
    let scratch = Scratch::new("report");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");
    let rendered = format_report(&report);

    for expected in [
        "world id",
        "seed",
        "chunks generated",
        "non-air blocks",
        "voxel storage",
        "blocks edited",
        "ticks advanced",
        "save size",
        "probes verified",
        "lifecycle phases",
    ] {
        assert!(
            rendered.contains(expected),
            "the report omits `{expected}`:\n{rendered}"
        );
    }
}
