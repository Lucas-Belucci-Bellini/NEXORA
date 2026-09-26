//! A real window, on whatever display this machine has: a desktop, or Xvfb in
//! CI (ADR-0027).
//!
//! The event loop must own the main thread, so this file is its own `main`
//! (`harness = false`) and runs one probe. No display is a failure, not a
//! skip; a machine without one says so with `NEXORA_DISPLAY=none`.

use std::process::ExitCode;

use nexora_rhi::conformance::CASES;
use nexora_window::probe::run_probe;

fn main() -> ExitCode {
    if std::env::var("NEXORA_DISPLAY").as_deref() == Ok("none") {
        println!("window: not run, NEXORA_DISPLAY=none");
        return ExitCode::SUCCESS;
    }
    if std::env::var("NEXORA_GPU").as_deref() == Ok("none") {
        println!("window: not run, NEXORA_GPU=none");
        return ExitCode::SUCCESS;
    }
    let report = match run_probe(96, 64, 3) {
        Ok(report) => report,
        Err(error) => {
            eprintln!(
                "window: FAILED: {error}\n\
                 run under a display (Linux CI: xvfb-run), or set NEXORA_DISPLAY=none \
                 to declare that this machine has none"
            );
            return ExitCode::FAILURE;
        }
    };
    let mut failures = Vec::new();
    if report.conformance != CASES.len() {
        failures.push(format!(
            "conformance with presentation: {}/{}",
            report.conformance,
            CASES.len()
        ));
    }
    // The conformance suite presents too, so the surface has shown at least
    // the probe's frames.
    if report.frames != 3 || report.surface.presented < 3 {
        failures.push(format!(
            "3 frames wanted, {} completed and {} presented",
            report.frames, report.surface.presented
        ));
    }
    if let Some(captured) = report.captured {
        if captured.checked == 0 || captured.matching != captured.checked {
            failures.push(format!(
                "{} of {} window texels showed the target",
                captured.matching, captured.checked
            ));
        }
    } else if report.surface.readable {
        failures.push("a readable surface was not read".into());
    }
    if failures.is_empty() {
        println!(
            "window: ok, {} frames on {} ({}), {}",
            report.frames,
            report.window.platform,
            report.adapter.backend,
            if report.captured.is_some() {
                "first frame read back from the surface"
            } else {
                "surface not readable here"
            }
        );
        ExitCode::SUCCESS
    } else {
        for failure in &failures {
            eprintln!("window: FAILED: {failure}");
        }
        ExitCode::FAILURE
    }
}
