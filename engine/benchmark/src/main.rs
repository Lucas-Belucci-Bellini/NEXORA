//! NEXORA benchmark runner.
//!
//! Produces the Phase 0 reference numbers for the language selection gate in
//! `ENGINE ARCHITECTURE AND TECHNOLOGY DECISION.md` §17.

use std::path::PathBuf;
use std::process::ExitCode;

use nexora_benchmark::{
    conformance, format_markdown, format_text, suites, Budget, Environment, Measurement, Report,
};

fn main() -> ExitCode {
    let options = match parse_args() {
        Ok(Some(options)) => options,
        Ok(None) => return ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}\n\n{}", usage());
            return ExitCode::FAILURE;
        }
    };

    if options.conformance {
        return match conformance::digests() {
            Ok(digests) => {
                for digest in digests {
                    println!("{}\t{:#018x}", digest.name, digest.value);
                }
                ExitCode::SUCCESS
            }
            Err(cause) => {
                eprintln!("conformance failed: {cause}");
                ExitCode::FAILURE
            }
        };
    }

    match run(&options) {
        Ok(report) => {
            if options.markdown {
                print!("{}", markdown_document(&report));
            } else {
                print!("{}", format_text(&report));
            }
            ExitCode::SUCCESS
        }
        Err(cause) => {
            eprintln!("benchmark failed: {cause}");
            ExitCode::FAILURE
        }
    }
}

struct Options {
    markdown: bool,
    smoke: bool,
    conformance: bool,
    scratch: PathBuf,
}

fn usage() -> String {
    "usage: nexora-benchmark [options]\n\
     \n\
     options:\n\
     \x20 --markdown     emit a Markdown table instead of plain text\n\
     \x20 --smoke        run a minimal pass, for CI rot detection\n\
     \x20 --conformance  print kernel digests for cross-stack comparison\n\
     \x20 --scratch <p>  directory for temporary save files\n\
     \x20 --help         show this message"
        .to_owned()
}

fn parse_args() -> Result<Option<Options>, String> {
    let mut options = Options {
        markdown: false,
        smoke: false,
        conformance: false,
        scratch: std::env::temp_dir().join("nexora-benchmark"),
    };
    let mut args = std::env::args().skip(1);

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(None);
            }
            "--markdown" => options.markdown = true,
            "--conformance" => options.conformance = true,
            "--smoke" => options.smoke = true,
            "--scratch" => {
                options.scratch = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--scratch needs a path".to_owned())?,
                );
            }
            unknown => return Err(format!("unknown argument `{unknown}`")),
        }
    }
    Ok(Some(options))
}

fn run(options: &Options) -> nexora_foundation::error::Result<Report> {
    std::fs::create_dir_all(&options.scratch).map_err(|cause| {
        nexora_foundation::error::Error::new(
            nexora_foundation::error::Domain::Core,
            "benchmark",
            "could not create the scratch directory",
        )
        .with_context("path", options.scratch.display().to_string())
        .with_context("cause", cause.to_string())
    })?;

    let standard = if options.smoke {
        Budget::standard(1).smoke()
    } else {
        Budget::standard(1)
    };
    let coarse = if options.smoke {
        Budget::coarse(1).smoke()
    } else {
        Budget::coarse(1)
    };

    let mut measurements: Vec<Measurement> = Vec::new();
    measurements.extend(suites::startup(coarse)?);
    measurements.extend(suites::spatial(standard));
    measurements.extend(suites::voxel(standard)?);
    measurements.extend(suites::worldgen(coarse)?);
    measurements.extend(suites::entities(coarse)?);
    measurements.extend(suites::physics(coarse)?);
    measurements.extend(suites::streaming(coarse)?);
    measurements.extend(suites::jobs(coarse)?);
    measurements.extend(suites::persistence(coarse, &options.scratch)?);
    measurements.extend(suites::meshing(coarse)?);
    measurements.extend(suites::ffi(standard));

    Ok(Report {
        measurements,
        unmeasured: suites::unmeasured_stages(),
        // Captured last, so peak memory reflects the whole run.
        environment: Environment::capture(),
    })
}

fn markdown_document(report: &Report) -> String {
    let environment = &report.environment;
    let memory = environment.peak_resident_bytes.map_or_else(
        || "unavailable".to_owned(),
        |bytes| nexora_benchmark::format_bytes(bytes as f64),
    );
    let binary = environment.executable_bytes.map_or_else(
        || "unavailable".to_owned(),
        |bytes| nexora_benchmark::format_bytes(bytes as f64),
    );

    format!(
        "## Environment\n\n\
         | | |\n| --- | --- |\n\
         | logical CPUs | {cpus} |\n\
         | build profile | `{profile}` |\n\
         | architecture | `{target}` |\n\
         | peak resident memory | {memory} |\n\
         | benchmark binary size | {binary} |\n\
         \n## Measurements\n\n{table}",
        cpus = environment.cpus,
        profile = environment.profile,
        target = environment.target,
        table = format_markdown(report),
    )
}
