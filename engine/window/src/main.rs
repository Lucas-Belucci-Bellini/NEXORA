//! `nexora-window-probe`: open a window on this machine and prove a frame
//! reaches it.
//!
//! Run by `scripts/local-validation.py` as the `window` check. Prints the
//! adapter as a device class, the window and its surface, runs the conformance
//! suite with presentation on, and shows a 16x16 target for `--frames` frames
//! (default 60), reading the first one back from the surface where the
//! platform allows. Exits non-zero at the first failure, with the reason.

use std::process::ExitCode;

use nexora_rhi::conformance::CASES;
use nexora_window::probe::{run_probe, EDGE};

fn main() -> ExitCode {
    let frames = match frames_argument() {
        Ok(frames) => frames,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("usage: nexora-window-probe [--frames N]");
            return ExitCode::from(2);
        }
    };
    match run_probe(256, 256, frames) {
        Ok(report) => {
            let adapter = &report.adapter;
            println!(
                "adapter            {} ({}, {})",
                adapter.name, adapter.backend, adapter.kind
            );
            println!(
                "window             {}x{} on {}",
                report.window.width, report.window.height, report.window.platform
            );
            println!(
                "surface            {}x{} {}, {}",
                report.surface.width,
                report.surface.height,
                report.surface.format,
                report.surface.present_mode
            );
            println!(
                "conformance        {}/{} cases on wgpu, presenting",
                report.conformance,
                CASES.len()
            );
            match report.captured {
                Some(captured) => println!(
                    "frame              {} of {} window texels show the {EDGE}x{EDGE} target, read back from the surface",
                    captured.matching, captured.checked
                ),
                None => println!("frame              not readable on this surface"),
            }
            println!(
                "presented          {} frames, {} of them by the probe",
                report.surface.presented, report.frames
            );
            println!("result             OK");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("result             FAILED");
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn frames_argument() -> Result<u64, String> {
    let mut args = std::env::args().skip(1);
    let mut frames = 60;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => {
                frames = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .filter(|value| *value > 0)
                    .ok_or("--frames needs a positive number")?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(frames)
}
