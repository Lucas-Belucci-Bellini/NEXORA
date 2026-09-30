//! The benchmark's window stage, in a real window (ADR-0027, DEBT-0008).
//!
//! The event loop must own the main thread, so this file is its own `main`
//! (`harness = false`), like `nexora-window`'s test. No display is a failure,
//! not a skip; a machine without one says so with `NEXORA_DISPLAY=none`, and
//! one without a GPU with `NEXORA_GPU=none`.

use std::process::ExitCode;

use nexora_benchmark::window::window;
use nexora_benchmark::Budget;

fn main() -> ExitCode {
    for (variable, what) in [("NEXORA_DISPLAY", "display"), ("NEXORA_GPU", "GPU")] {
        if std::env::var(variable).as_deref() == Ok("none") {
            println!("window stage: not run, {variable}=none (no {what})");
            return ExitCode::SUCCESS;
        }
    }
    let budget = Budget {
        warmup_iterations: 0,
        samples: 1,
        iterations_per_sample: 1,
    };
    let stage = match window(budget, None) {
        Ok(stage) => stage,
        Err(error) => {
            eprintln!("window stage: FAILED: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(gap) = stage.gap {
        eprintln!(
            "window stage: not measured: {gap}\n\
             run under a display (Linux CI: xvfb-run), or set NEXORA_DISPLAY=none"
        );
        return ExitCode::FAILURE;
    }
    let value = |name: &str| {
        stage
            .measurements
            .iter()
            .find(|m| m.name == name)
            .map(nexora_benchmark::Measurement::median)
    };
    let (Some(first), Some(interval), Some(wait), Some(judged)) = (
        value("window.first_frame"),
        value("window.present_chunk_16"),
        value("window.present_wait_chunk_16"),
        value("window.pixels_judged"),
    ) else {
        eprintln!("window stage: FAILED: a measurement is missing");
        return ExitCode::FAILURE;
    };
    // The frame read back from the surface matched the ray cast, or the stage
    // would have returned an error; at least half the pixels were judged.
    // Each wait lies inside its interval, so the median wait cannot pass the
    // median interval.
    if first <= 0.0 || interval <= 0.0 || wait > interval || judged * 2.0 < 256.0 * 256.0 {
        eprintln!(
            "window stage: FAILED: first {first} ns, interval {interval} ns, \
             wait {wait} ns, judged {judged}"
        );
        return ExitCode::FAILURE;
    }
    println!("window stage: OK, {judged} pixels judged, a frame every {interval} ns");
    ExitCode::SUCCESS
}
