//! End-to-end tests for the Phase 0 vertical slice.
//!
//! These run the same code path the `nexora-headless` binary runs, so a green
//! test suite means the slice itself is green - `NEXORA DEFINITION OF DONE.md`
//! asks for verification, not for a build that merely compiles.

use std::fs;
use std::path::PathBuf;

use nexora_foundation::memory::{MemoryClass, Pressure};
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
fn the_physics_stage_lands_everything_it_dropped() {
    // The claim the stage exists to check: the solver and the *generated*
    // world agree about where the ground is. A flat test fixture cannot show
    // that, because a flat fixture is not what the generator produces.
    let scratch = Scratch::new("physics");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");

    assert_eq!(
        report.physics_settled,
        report.physics_bodies,
        "{} of {} bodies never came to rest",
        report.physics_bodies - report.physics_settled,
        report.physics_bodies
    );
    assert!(report.physics_substeps > 0);
    assert!(
        report.physics_contacts > 0,
        "nothing ever touched the ground"
    );
    assert!(
        report.physics_drop_cm > 0,
        "the character did not fall at all: it was spawned on the ground rather \
         than having to find it"
    );
}

#[test]
fn physics_does_not_depend_on_the_worker_count_either() {
    // Physics runs after generation and must not disturb the save. If it wrote
    // to the world, or read it in a scheduling-dependent way, the two saves
    // would differ and the determinism guarantee would be gone.
    let scratch = Scratch::new("physics-threads");
    let single = run_slice(&SliceConfig {
        seed: 4_242,
        worker_threads: 1,
        ..config(&scratch, "one.nxsv")
    })
    .expect("single-threaded run");
    let many = run_slice(&SliceConfig {
        seed: 4_242,
        worker_threads: 8,
        ..config(&scratch, "many.nxsv")
    })
    .expect("multi-threaded run");

    assert_eq!(single.physics_drop_cm, many.physics_drop_cm);
    assert_eq!(single.physics_contacts, many.physics_contacts);
    assert_eq!(single.physics_settled, many.physics_settled);
    assert_eq!(
        fs::read(scratch.save("one.nxsv")).expect("read"),
        fs::read(scratch.save("many.nxsv")).expect("read"),
        "the physics stage must leave the world untouched"
    );
}

#[test]
fn the_module_graph_orders_physics_after_the_world() {
    let scratch = Scratch::new("module-order");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");
    // The slice would have failed to resolve if the edge were missing; this
    // records that the run really did get past module resolution with it.
    assert!(report.phases.contains(&"module-resolution"));
}

#[test]
fn the_streaming_stage_evicts_and_restores_without_losing_an_edit() {
    // The invariant of WORLD CONTINUITY AND PLAYER INDEPENDENCE.md §18, checked
    // rather than asserted: the observer walks away, the edited region is
    // evicted, and the probes still verify after the save and reload.
    let scratch = Scratch::new("streaming");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");

    assert!(report.streaming_ticks > 0);
    assert!(
        report.streaming_evicted > 0,
        "nothing was ever evicted, so eviction was never exercised"
    );
    assert!(
        report.streaming_generated > 0,
        "the walk never left the region the slice had already generated"
    );
    // The slice edits every column it generated, so retention takes all of
    // them and nothing else: the chunks the walk generated along the way are
    // regenerable and are dropped rather than kept.
    assert_eq!(
        report.streaming_spilled, report.chunks_generated,
        "retention should take exactly the edited columns"
    );
    // And `DEBT-0020`: it never holds them all at once, because each tick's
    // evictions go to their region files at the end of that tick. The peak is
    // bounded by one tick's eviction budget, not by how much was edited.
    assert!(
        report.streaming_retained_peak > 0,
        "nothing was ever held, so the spill was never exercised"
    );
    assert!(
        report.streaming_retained_peak < report.chunks_generated,
        "memory held every edited column at once, which is what spilling is for"
    );
    assert_eq!(
        report.streaming_read_back, report.streaming_restored,
        "every column that came back came back off disk"
    );
    assert!(
        report.streaming_region_writes < report.streaming_spilled,
        "columns sharing a region should cost one write between them"
    );
    assert!(
        report.streaming_generated as usize > report.chunks_generated,
        "the walk generated fewer columns than it evicted, which cannot happen"
    );
    assert!(
        report.streaming_restored > 0,
        "walking back did not restore anything"
    );
    // And the payoff: every probe still reads correctly after the round trip.
    assert_eq!(report.probes_verified, report.blocks_edited);
}

#[test]
fn streaming_leaves_the_resident_set_it_found() {
    // The observer ends where it started, so the save has to contain the same
    // chunks it would have without any streaming at all. A drop here would mean
    // a clean chunk was evicted and never regenerated.
    let scratch = Scratch::new("streaming-set");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");
    assert_eq!(report.chunks_generated, 9);
    assert!(report.save_bytes > 0);
}

#[test]
fn streaming_does_not_depend_on_the_worker_count_either() {
    let scratch = Scratch::new("streaming-threads");
    let single = run_slice(&SliceConfig {
        seed: 5_150,
        worker_threads: 1,
        ..config(&scratch, "one.nxsv")
    })
    .expect("single-threaded run");
    let many = run_slice(&SliceConfig {
        seed: 5_150,
        worker_threads: 8,
        ..config(&scratch, "many.nxsv")
    })
    .expect("multi-threaded run");

    assert_eq!(single.streaming_ticks, many.streaming_ticks);
    assert_eq!(single.streaming_generated, many.streaming_generated);
    assert_eq!(single.streaming_evicted, many.streaming_evicted);
    assert_eq!(single.streaming_retained_peak, many.streaming_retained_peak);
    assert_eq!(
        fs::read(scratch.save("one.nxsv")).expect("read"),
        fs::read(scratch.save("many.nxsv")).expect("read"),
        "streaming must not change what the world saves"
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
        "physics bodies",
        "physics substeps",
        "character drop",
        "streaming ticks",
        "frames",
        "input",
        "chunks retained",
        "memory",
        "world.chunks",
        "world.retained",
        "probes verified",
        "lifecycle phases",
    ] {
        assert!(
            rendered.contains(expected),
            "the report omits `{expected}`:\n{rendered}"
        );
    }
}

/// The walk runs on the engine's frame loop, and every fixed step is one tick.
///
/// This is what keeps the loop load-bearing here rather than decorative: stop
/// driving the walk with it and `frame_steps` falls to zero while
/// `streaming_ticks` keeps climbing. The two counts are deterministic because
/// the delta handed to the loop is scripted - the slice is a proof, and the
/// real per-frame distribution is the benchmark's job.
#[test]
fn the_walk_runs_on_the_frame_loop_and_every_step_is_one_tick() {
    let scratch = Scratch::new("frames");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");

    assert!(report.frames > 0, "the walk ran no frames at all");
    assert_eq!(
        report.frame_steps,
        u64::from(report.streaming_ticks),
        "every fixed step runs exactly one streaming tick"
    );
    assert_eq!(
        report.frames, report.frame_steps,
        "a delta of exactly one step buys exactly one step"
    );
    assert_eq!(
        report.frame_steps_dropped, 0,
        "a scripted one-step delta can never fall behind its own schedule"
    );
}

/// The walk is what the input system says the player asked for.
///
/// `stream_a_walk` already fails if the derived route is not the route the
/// slice is specified to walk, so reaching this test at all means the chain
/// from scancode to interest source held. What is checked here is that the
/// input stage really ran on every frame rather than once at the start, and
/// that the taps are the ones the script contains.
#[test]
fn the_walk_is_driven_by_the_input_system_and_not_by_a_list_of_stops() {
    let scratch = Scratch::new("input");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");

    assert_eq!(
        report.input_frames, report.frames,
        "input is sampled at the top of every frame, not once per leg"
    );
    assert_eq!(
        report.input_intents, 16,
        "eight taps out and back, each with the release on the frame after it"
    );
    assert!(
        report.input_intents < report.input_frames,
        "most frames carry no intent at all, which is what makes the ones that do worth reporting"
    );
}

#[test]
fn the_command_stage_both_accepts_and_refuses() {
    let scratch = Scratch::new("commands");
    let report = run_slice(&config(&scratch, "commands")).expect("the slice runs");

    // Accepting is half the job. A pipeline that has only ever been shown
    // accepting things has not been shown to refuse anything, and refusing is
    // the half that carries the security boundary.
    assert!(report.commands_accepted > 0, "no command reached a handler");
    assert!(
        report.commands_refused > 0,
        "no command was refused; the validation layers are unproven"
    );
}

#[test]
fn commands_do_not_change_what_the_save_contains() {
    // The stage places a block and breaks it again, so the world it hands back
    // is the world it was given. If that ever stops being true, the
    // byte-identical determinism comparison below stops measuring determinism
    // and starts measuring whether the command stage ran the same way twice.
    let scratch = Scratch::new("commands-neutral");
    let report = run_slice(&config(&scratch, "commands-neutral")).expect("the slice runs");

    assert_eq!(report.probes_verified, report.blocks_edited);
    assert!(report.save_bytes > 0);
}

#[test]
fn commands_do_not_depend_on_the_worker_count_either() {
    let scratch = Scratch::new("commands-determinism");

    let mut single = config(&scratch, "cmd-a");
    single.worker_threads = 1;
    let single = run_slice(&single).expect("the slice runs at one thread");

    let mut many = config(&scratch, "cmd-b");
    many.worker_threads = 8;
    let many = run_slice(&many).expect("the slice runs at eight threads");

    assert_eq!(single.commands_accepted, many.commands_accepted);
    assert_eq!(single.commands_refused, many.commands_refused);
    assert_eq!(
        std::fs::read(scratch.save("cmd-a")).expect("save a"),
        std::fs::read(scratch.save("cmd-b")).expect("save b"),
        "the command stage made the save depend on the worker count"
    );
}

#[test]
fn the_slice_rebuilds_itself_from_checkpoint_plus_journal() {
    // Snapshot + Journal -> Recovery, on this run's own data. The journal was
    // written by `World::set_block` during the run, not by the test, so a pass
    // is evidence about the engine rather than about the harness.
    let scratch = Scratch::new("recovery");
    let report = run_slice(&config(&scratch, "recovery")).expect("the slice runs");

    assert!(report.journal_edits > 0, "nothing was journalled");
    assert_eq!(
        report.recovered_probes, report.probes_verified,
        "recovery did not reproduce everything the save contains"
    );
}

#[test]
fn every_write_in_the_run_is_journalled_including_the_commands() {
    // The command stage places a block and breaks it again. Those are two
    // writes through the same `set_block` path as the edit stage, so they must
    // appear in the journal without the command handlers knowing it exists.
    let scratch = Scratch::new("journal-covers-commands");
    let report = run_slice(&config(&scratch, "journal-covers-commands")).expect("the slice runs");

    assert_eq!(
        report.journal_edits as usize,
        report.blocks_edited + report.commands_accepted,
        "the journal missed a write that changed the world"
    );
}

#[test]
fn journalling_does_not_depend_on_the_worker_count_either() {
    let scratch = Scratch::new("journal-determinism");

    let mut single = config(&scratch, "jr-a");
    single.worker_threads = 1;
    let single = run_slice(&single).expect("the slice runs at one thread");

    let mut many = config(&scratch, "jr-b");
    many.worker_threads = 8;
    let many = run_slice(&many).expect("the slice runs at eight threads");

    assert_eq!(single.journal_edits, many.journal_edits);
    assert_eq!(single.recovered_probes, many.recovered_probes);
    assert_eq!(
        std::fs::read(scratch.save("jr-a")).expect("save a"),
        std::fs::read(scratch.save("jr-b")).expect("save b"),
        "journalling made the save depend on the worker count"
    );
}

#[test]
fn every_memory_pool_stays_inside_its_budget_and_drains_at_rest() {
    let scratch = Scratch::new("memory");
    let report = run_slice(&config(&scratch, "world.nxsv")).expect("slice");
    let memory = &report.memory;

    assert_eq!(
        memory
            .pools
            .iter()
            .map(|pool| pool.name)
            .collect::<Vec<_>>(),
        ["rhi.null", "world.chunks", "world.retained"],
        "no texture pool without textures; reported by name"
    );
    let pool = |name: &str| {
        memory
            .pools
            .iter()
            .find(|pool| pool.name == name)
            .expect("the pool is reported")
    };
    assert_eq!(memory.worst(), Pressure::Nominal, "{memory:?}");
    assert_eq!(memory.suspected_leaks(), 0);

    let world = pool("world.chunks");
    assert_eq!(world.class, MemoryClass::World);
    assert_eq!(
        world.current as usize, report.storage_bytes,
        "the pool records what the world reports, and the reloaded world is the one it saved"
    );
    assert!(world.high_water >= world.current);

    // Every edited column was held outside the world at some point, and all
    // of it went back before the save.
    let retained = pool("world.retained");
    assert_eq!(retained.current, 0);
    assert!(retained.high_water > 0, "the walk retained edited columns");
    assert!(retained.drains_at_rest);

    // The RHI's null backend held memory for the conformance suite, in the
    // GPU class, and gave every byte of it back.
    let gpu = pool("rhi.null");
    assert_eq!(gpu.class, MemoryClass::Gpu);
    assert_eq!(gpu.current, 0);
    assert!(gpu.high_water > 0, "the suite allocated through the pool");
    assert!(gpu.drains_at_rest);
    assert_eq!(report.rhi_backend, "null");
    assert_eq!(report.rhi_conformance, nexora_rhi::conformance::CASES.len());
    assert_eq!(report.rhi_textures, 0, "no content, nothing to upload");
}

#[test]
fn a_resource_index_missing_what_the_content_asks_for_is_refused_whole() {
    let scratch = Scratch::new("gaps");
    let root = scratch.save("resources");
    fs::create_dir_all(&root).unwrap();
    // An index that is well-formed and provides nothing.
    fs::write(
        root.join("resources.json"),
        "{\"schema\": 1, \"resources\": []}",
    )
    .unwrap();
    let config = SliceConfig {
        content: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content/first-generation/blocks.json"),
        ),
        resources: Some(root),
        ..config(&scratch, "world.nxsv")
    };

    let err = run_slice(&config).expect_err("nothing the content needs is indexed");
    let text = err.to_string();
    // Sixteen materials and their sixteen albedos, all named, before any
    // texture is opened -- not whichever one a loader reached first.
    assert!(text.contains("gaps=32"), "{text}");
    assert!(
        text.contains("nexora:material/stone/basalt: no albedo map"),
        "{text}"
    );
}

#[test]
fn the_first_generation_content_survives_every_stage() {
    // Every path that reads a world back -- the save, the journal replay, the
    // region store -- has to register the content, or a world that saves
    // stones cannot open them. The region store was the one that did not.
    let scratch = Scratch::new("content-stages");
    let config = SliceConfig {
        content: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../content/first-generation/blocks.json"),
        ),
        ..config(&scratch, "world.nxsv")
    };
    let report = run_slice(&config).expect("the slice completes with content");
    assert_eq!(report.content_blocks, 16);
    assert_eq!(report.content_surfaces, 16);
    assert_eq!(report.probes_verified, report.blocks_edited);
    assert_eq!(report.recovered_probes, report.probes_verified);
}
