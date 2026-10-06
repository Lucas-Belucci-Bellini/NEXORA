//! `nexora-client`: the runtime in client mode (ADR-0032).
//!
//! Opens a window on this machine and shows the generated world through the
//! first render pass, from the eye of a player standing in it (ADR-0035).
//! W, A, S, D walk; Space jumps; the arrow keys turn and look; the primary
//! mouse button mines the block at the centre of the view and the secondary
//! one builds against it (ADR-0036); Escape exits, as does closing the
//! window. `--world PATH` goes on from the world and the player saved there,
//! and saves them back at the end. `--content` registers a block content
//! document, and `--resources` with it draws the world's albedo from the
//! texture forge's output (ADR-0037). Prints what the run did, and exits
//! non-zero at the first failure.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use nexora_client::{format_report, run_client, ClientConfig};

const USAGE: &str = "usage: nexora-client [--frames N] [--timeout S] [--radius R] [--seed N] \
[--size WxH] [--save PATH | --world PATH] [--content PATH [--resources DIR]] [--capture PATH] [--verbose]";

fn main() -> ExitCode {
    let config = match parse(std::env::args().skip(1)) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run_client(&config) {
        Ok(report) => {
            print!("{}", format_report(&report));
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("result             FAILED");
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<ClientConfig, String> {
    let mut config = ClientConfig::default();
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| args.next().ok_or_else(|| format!("{flag} needs a value"));
        match arg.as_str() {
            "--frames" => config.frames = Some(positive(&value("--frames")?, "--frames")?),
            "--timeout" => {
                config.timeout = Some(Duration::from_secs(positive(
                    &value("--timeout")?,
                    "--timeout",
                )?));
            }
            "--radius" => {
                config.radius = value("--radius")?
                    .parse()
                    .map_err(|_| "--radius needs a number".to_owned())?;
            }
            "--seed" => {
                config.seed = value("--seed")?
                    .parse()
                    .map_err(|_| "--seed needs a number".to_owned())?;
            }
            "--size" => {
                let size = value("--size")?;
                let (w, h) = size
                    .split_once('x')
                    .ok_or_else(|| "--size needs WxH".to_owned())?;
                config.width = positive(w, "--size")? as u32;
                config.height = positive(h, "--size")? as u32;
            }
            "--save" => config.save = Some(PathBuf::from(value("--save")?)),
            "--world" => config.world = Some(PathBuf::from(value("--world")?)),
            "--content" => config.content = Some(PathBuf::from(value("--content")?)),
            "--resources" => config.resources = Some(PathBuf::from(value("--resources")?)),
            "--capture" => config.capture = Some(PathBuf::from(value("--capture")?)),
            "--verbose" => config.verbose = true,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(config)
}

fn positive(text: &str, flag: &str) -> Result<u64, String> {
    text.parse()
        .ok()
        .filter(|value| *value > 0 && *value <= u64::from(u32::MAX))
        .ok_or_else(|| format!("{flag} needs a positive number"))
}
