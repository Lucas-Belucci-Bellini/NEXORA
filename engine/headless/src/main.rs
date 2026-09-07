//! NEXORA headless runner.
//!
//! Runs the Phase 0 vertical slice and exits non-zero if anything about it
//! fails to verify, so it works as a smoke test in CI as well as by hand.

use std::path::PathBuf;
use std::process::ExitCode;

use nexora_headless::{format_report, run_slice, SliceConfig};

fn main() -> ExitCode {
    let config = match parse_args() {
        Ok(Some(config)) => config,
        Ok(None) => return ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            eprintln!();
            eprintln!("{}", usage());
            return ExitCode::FAILURE;
        }
    };

    match run_slice(&config) {
        Ok(report) => {
            print!("{}", format_report(&report));
            println!("\nresult             OK");
            ExitCode::SUCCESS
        }
        Err(cause) => {
            eprintln!("\nresult             FAILED");
            eprintln!("{cause}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    let default = SliceConfig::default();
    format!(
        "usage: nexora-headless [options]\n\
         \n\
         options:\n\
         \x20 --seed <u64>        world generation seed (default {})\n\
         \x20 --radius <i64>      chunk radius to generate (default {})\n\
         \x20 --save <path>       save file path (default {})\n\
         \x20 --threads <n>       worker threads (default {})\n\
         \x20 --quiet             suppress progress diagnostics\n\
         \x20 --help              show this message",
        default.seed,
        default.radius,
        default.save_path.display(),
        default.worker_threads
    )
}

fn parse_args() -> Result<Option<SliceConfig>, String> {
    let mut config = SliceConfig::default();
    let mut args = std::env::args().skip(1);

    while let Some(argument) = args.next() {
        let mut value = |name: &str| -> Result<String, String> {
            args.next().ok_or_else(|| format!("{name} needs a value"))
        };
        match argument.as_str() {
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(None);
            }
            "--seed" => {
                config.seed = value("--seed")?
                    .parse()
                    .map_err(|_| "--seed must be a u64".to_owned())?;
            }
            "--radius" => {
                let radius: i64 = value("--radius")?
                    .parse()
                    .map_err(|_| "--radius must be an integer".to_owned())?;
                if !(0..=16).contains(&radius) {
                    return Err("--radius must be between 0 and 16".to_owned());
                }
                config.radius = radius;
            }
            "--save" => config.save_path = PathBuf::from(value("--save")?),
            "--threads" => {
                let threads: usize = value("--threads")?
                    .parse()
                    .map_err(|_| "--threads must be an integer".to_owned())?;
                if threads == 0 {
                    return Err("--threads must be at least 1".to_owned());
                }
                config.worker_threads = threads;
            }
            "--quiet" => config.verbose = false,
            unknown => return Err(format!("unknown argument `{unknown}`")),
        }
    }

    Ok(Some(config))
}
