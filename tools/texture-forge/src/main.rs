//! NEXORA Texture Forge.
//!
//! `TOOLING ARCHITECTURE.md` shapes its commands as `nexora <noun> <verb>` and
//! names `nexora world inspect` and `nexora registry` among them, so the verbs
//! here read the same way. Argument parsing is by hand, like every other
//! binary in this workspace, because ADR-0002 forbids a dependency and a
//! handful of flags does not need one.

use std::path::PathBuf;
use std::process::ExitCode;

use nexora_asset::document;
use nexora_asset::texture::MapRole;
use nexora_asset::validation::Verdict;
use nexora_foundation::error::Result;
use nexora_foundation::ident::Identifier;
use nexora_texture_forge::forge::Forge;
use nexora_texture_forge::layout;

/// Where materials are written when `--out` is not given.
const DEFAULT_ROOT: &str = "assets/materials";

fn main() -> ExitCode {
    let command = match parse_args() {
        Ok(Some(command)) => command,
        Ok(None) => return ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}\n\n{}", usage());
            // Two, not one: a usage mistake is not the same as a material that
            // failed to validate, and a script should be able to tell them
            // apart without reading the message.
            return ExitCode::from(2);
        }
    };

    match run(command) {
        Ok(code) => code,
        Err(cause) => {
            eprintln!("texture-forge: {cause}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: nexora-texture-forge <command> [options]\n\
     \n\
     commands:\n\
     \x20 generate <definition.json>   realise a material and write it\n\
     \x20 validate <material-id>       check what is on disk for a material\n\
     \x20 inspect  <material-id>       print a material's definition and origin\n\
     \x20 list                         every material written under the root\n\
     \n\
     options:\n\
     \x20 --out <dir>     where materials live (default: assets/materials)\n\
     \x20 --seed <value>  generation seed, decimal or 0x-prefixed (default: 0)\n\
     \x20 --force         replace a material that is already written\n\
     \x20 --help          show this message\n\
     \n\
     exit codes:\n\
     \x20 0  the command succeeded\n\
     \x20 1  the command ran and the answer was no\n\
     \x20 2  the command line was wrong"
        .to_owned()
}

enum Command {
    Generate {
        definition: PathBuf,
        root: PathBuf,
        seed: u64,
        force: bool,
    },
    Validate {
        id: Identifier,
        root: PathBuf,
    },
    Inspect {
        id: Identifier,
        root: PathBuf,
    },
    List {
        root: PathBuf,
    },
}

fn parse_args() -> std::result::Result<Option<Command>, String> {
    let mut args = std::env::args().skip(1);
    let Some(verb) = args.next() else {
        println!("{}", usage());
        return Ok(None);
    };
    if verb == "--help" || verb == "-h" {
        println!("{}", usage());
        return Ok(None);
    }

    let mut positional: Option<String> = None;
    let mut root = PathBuf::from(DEFAULT_ROOT);
    let mut seed = 0u64;
    let mut force = false;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(None);
            }
            "--out" => {
                root = PathBuf::from(args.next().ok_or_else(|| "--out needs a path".to_owned())?);
            }
            "--seed" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "--seed needs a value".to_owned())?;
                seed = parse_seed(&raw)?;
            }
            "--force" => force = true,
            other if other.starts_with('-') => {
                return Err(format!("unknown option `{other}`"));
            }
            other => {
                if positional.replace(other.to_owned()).is_some() {
                    return Err(format!("`{verb}` takes one argument"));
                }
            }
        }
    }

    let needed = |what: &str| -> std::result::Result<String, String> {
        positional
            .clone()
            .ok_or_else(|| format!("`{verb}` needs {what}"))
    };

    match verb.as_str() {
        "generate" => Ok(Some(Command::Generate {
            definition: PathBuf::from(needed("a definition file")?),
            root,
            seed,
            force,
        })),
        "validate" => Ok(Some(Command::Validate {
            id: identifier(&needed("a material id")?)?,
            root,
        })),
        "inspect" => Ok(Some(Command::Inspect {
            id: identifier(&needed("a material id")?)?,
            root,
        })),
        "list" => {
            if positional.is_some() {
                return Err("`list` takes no argument".to_owned());
            }
            Ok(Some(Command::List { root }))
        }
        unknown => Err(format!("unknown command `{unknown}`")),
    }
}

fn parse_seed(raw: &str) -> std::result::Result<u64, String> {
    let parsed = raw.strip_prefix("0x").map_or_else(
        || raw.parse::<u64>().ok(),
        |hex| u64::from_str_radix(hex, 16).ok(),
    );
    parsed.ok_or_else(|| format!("`{raw}` is not a seed; use a decimal or 0x-prefixed number"))
}

fn identifier(raw: &str) -> std::result::Result<Identifier, String> {
    Identifier::parse(raw).map_err(|cause| format!("`{raw}` is not a material id: {cause}"))
}

fn run(command: Command) -> Result<ExitCode> {
    match command {
        Command::Generate {
            definition,
            root,
            seed,
            force,
        } => generate(&definition, root, seed, force),
        Command::Validate { id, root } => validate(&id, root),
        Command::Inspect { id, root } => inspect(&id, root),
        Command::List { root } => list(root),
    }
}

fn generate(path: &PathBuf, root: PathBuf, seed: u64, force: bool) -> Result<ExitCode> {
    let text = std::fs::read_to_string(path).map_err(|cause| {
        nexora_foundation::error::Error::new(
            nexora_foundation::error::Domain::Content,
            "texture-forge",
            "the definition could not be read",
        )
        .with_context("path", path.display().to_string())
        .with_context("cause", cause.to_string())
    })?;
    let definition = document::from_text(&text)?;

    let forge = Forge::new(root)?;
    let outcome = forge.generate(&definition, seed, force)?;

    println!("{} {}", outcome.status.as_str(), outcome.material.id());
    if let Some(refusal) = &outcome.refusal {
        eprintln!("texture-forge: {refusal}");
        return Ok(ExitCode::FAILURE);
    }
    if outcome.status.wrote_files() {
        for (file, size) in &outcome.files {
            println!("  {:>9}  {}", human(*size), file.display());
        }
        println!("  {:>9}  total", human(outcome.byte_len()));
    }
    if outcome.validation.verdict() != Verdict::Pass {
        println!("{}", outcome.validation);
    }
    Ok(ExitCode::SUCCESS)
}

fn validate(id: &Identifier, root: PathBuf) -> Result<ExitCode> {
    let forge = Forge::new(root)?;
    let (definition, result) = forge.validate(id)?;
    println!(
        "{} {} {}",
        result.verdict(),
        definition.id(),
        definition.revision()
    );
    for finding in result.findings() {
        println!("  {finding}");
    }
    Ok(if result.is_usable() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn inspect(id: &Identifier, root: PathBuf) -> Result<ExitCode> {
    let forge = Forge::new(root)?;
    let definition = forge.read_definition(id)?;
    let maps = forge.read_maps(&definition)?;

    println!("{}", definition.id());
    println!("  name         {}", definition.name());
    println!("  category     {}", definition.category().as_str());
    println!("  revision     {}", definition.revision());
    println!(
        "  resolution   {}x{}",
        definition.resolution().width,
        definition.resolution().height
    );
    println!(
        "  scale        {} m per tile",
        definition.physical_scale().metres_per_tile
    );
    println!("  blend        {}", definition.blend().as_str());
    println!("  seamless     {}", definition.is_seamless());
    println!(
        "  pbr          metallic {:.2}  roughness {:.2}  normal {:.2}  height {:.2}",
        definition.pbr().metallic,
        definition.pbr().roughness,
        definition.pbr().normal_strength,
        definition.pbr().height_strength
    );

    println!("  maps");
    for (role, status) in maps.status(definition.wanted_maps()) {
        let path = layout::map_file(forge.root(), id, role);
        let size = std::fs::metadata(&path)
            .map(|meta| meta.len() as usize)
            .ok();
        println!(
            "    {:<18} {:<10} {}",
            role.as_str(),
            status.as_str(),
            size.map_or_else(|| "-".to_owned(), human)
        );
    }

    let provenance = definition.provenance();
    println!("  origin");
    println!("    class      {}", provenance.class.as_str());
    println!("    status     {}", provenance.status.as_str());
    println!("    release    {}", provenance.release.as_str());
    println!("    author     {}", provenance.author);
    println!("    tool       {}", provenance.source_tool);
    println!("    recorded   {}", provenance.recorded_at);
    println!("    hash       {:#018x}", provenance.content_hash());
    println!("    may ship   {}", provenance.may_ship());
    if let Some(trace) = &provenance.generation {
        println!(
            "    generator  {} {}",
            trace.generator, trace.generator_version
        );
        if let (Some(pipeline), Some(version)) = (&trace.pipeline, trace.pipeline_version) {
            println!("    pipeline   {pipeline} {version}");
        }
        if let Some(preset) = &trace.preset {
            println!("    preset     {preset}");
        }
        println!("    seed       {:#018x}", trace.seed);
        if let Some(prompt) = &trace.prompt {
            println!("    prompt     {prompt}");
        }
        for (key, value) in &trace.parameters {
            println!("    {key:<10} {value}");
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn list(root: PathBuf) -> Result<ExitCode> {
    let forge = Forge::new(root)?;
    let listing = forge.list()?;

    if listing.is_empty() {
        println!("no materials under {}", forge.root().display());
        return Ok(ExitCode::SUCCESS);
    }
    for material in &listing.materials {
        let maps = MapRole::ALL
            .into_iter()
            .filter(|role| layout::map_file(forge.root(), material.id(), *role).is_file())
            .count();
        println!(
            "{:<40} {:<10} {:<4} {}x{}  {} maps",
            material.id().to_string(),
            material.category().as_str(),
            material.revision().to_string(),
            material.resolution().width,
            material.resolution().height,
            maps
        );
    }
    for (path, error) in &listing.unreadable {
        eprintln!("unreadable: {} -- {error}", path.display());
    }
    Ok(if listing.unreadable.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// Bytes, in units a person reads without counting digits.
fn human(bytes: usize) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let kib = bytes as f64 / 1024.0;
    if kib < 1024.0 {
        return format!("{kib:.1} KiB");
    }
    format!("{:.1} MiB", kib / 1024.0)
}
