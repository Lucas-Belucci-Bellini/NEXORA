//! The runtime in client mode, in a real window: a desktop, or Xvfb in CI
//! (ADR-0032).
//!
//! The event loop must own the main thread, so this file is its own `main`
//! (`harness = false`). No display is a failure, not a skip; a machine without
//! one says so with `NEXORA_DISPLAY=none`.

use std::process::ExitCode;
use std::time::Duration;

use nexora_client::{frame_holds, run_client, ClientConfig, Ending};

fn main() -> ExitCode {
    if std::env::var("NEXORA_DISPLAY").as_deref() == Ok("none") {
        println!("client: not run, NEXORA_DISPLAY=none");
        return ExitCode::SUCCESS;
    }
    if std::env::var("NEXORA_GPU").as_deref() == Ok("none") {
        println!("client: not run, NEXORA_GPU=none");
        return ExitCode::SUCCESS;
    }
    let config = ClientConfig {
        frames: Some(12),
        timeout: Some(Duration::from_secs(120)),
        ..ClientConfig::default()
    };
    let report = match run_client(&config) {
        Ok(report) => report,
        Err(error) => {
            eprintln!(
                "client: FAILED: {error}\n\
                 run under a display (Linux CI: xvfb-run), or set NEXORA_DISPLAY=none \
                 to declare that this machine has none"
            );
            return ExitCode::FAILURE;
        }
    };
    let mut failures = Vec::new();
    let phases = &report.phases;
    if !phases.contains(&"presentation-running") || phases.last() != Some(&"process-exit") {
        failures.push(format!("lifecycle: {}", phases.join(" -> ")));
    }
    // The renderer, and physics: the player runs inside the frame.
    for wanted in ["nexora:module/renderer", "nexora:module/physics"] {
        if !report.modules.iter().any(|module| module == wanted) {
            failures.push(format!("modules: {}", report.modules.join(", ")));
        }
    }
    if report.frames != 12 || report.ending != Ending::FrameLimit {
        failures.push(format!(
            "{} frames, ended by {}",
            report.frames,
            report.ending.as_str()
        ));
    }
    if report.columns != (9, 25) || report.drawn.drawn == 0 {
        failures.push(format!(
            "columns {:?}, drawn {:?}",
            report.columns, report.drawn
        ));
    }
    // No key was pressed: the camera must not have moved. It is the
    // player's eye now, under gravity, so this is also the claim that a
    // resting player is a fixed point of the solver, to the bit.
    if report.moved != [0.0; 3] || report.active_frames != 0 {
        failures.push(format!("moved {:?} with no key", report.moved));
    }
    if report.player.walked != [0.0; 3] || !report.player.grounded {
        failures.push(format!(
            "the player walked {:?} with no key, grounded {}",
            report.player.walked, report.player.grounded
        ));
    }
    // Work is each frame's wall time less its wait on presentation, so no
    // order statistic of the work can exceed the wall's, and no wait can
    // outlast the longest frame.
    if (0..3).any(|k| report.work[k] > report.wall[k])
        || report.presentation_wait[2] > report.wall[2]
    {
        failures.push(format!(
            "work {:?} against wall {:?}, presentation wait {:?}",
            report.work, report.wall, report.presentation_wait
        ));
    }
    if let Some(check) = report.first_frame {
        if !frame_holds(&check) {
            failures.push(format!("first frame {check:?}"));
        }
    }
    if failures.is_empty() {
        println!(
            "client: ok ({} frames on {}, first frame {})",
            report.frames,
            report.adapter,
            report
                .first_frame
                .map_or("not readable".to_owned(), |check| format!(
                    "{} of {} judged, {} matching",
                    check.judged, check.pixels, check.matching
                ))
        );
        ExitCode::SUCCESS
    } else {
        eprintln!("client: FAILED: {}", failures.join("; "));
        ExitCode::FAILURE
    }
}
