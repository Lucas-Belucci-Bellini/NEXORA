//! NEXORA Texture Forge.
//!
//! `TOOLING ARCHITECTURE.md` shapes its commands as `nexora <noun> <verb>` and
//! names `nexora world inspect` and `nexora registry` among them, so the verbs
//! here read the same way. Argument parsing is by hand, like every other
//! binary in this workspace, because ADR-0002 forbids a dependency and a
//! handful of flags does not need one.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use nexora_asset::document;
use nexora_asset::generator::Backend;
use nexora_asset::texture::MapRole;
use nexora_asset::validation::Verdict;
use nexora_foundation::error::Result;
use nexora_foundation::ident::Identifier;
use nexora_resource::manifest::MANIFEST_FILE;
use nexora_texture_forge::forge::{Forge, Outcome, WriteStatus};
use nexora_texture_forge::layout;
use nexora_texture_forge::manifest;
use nexora_texture_forge::plan::Plan;
use nexora_texture_forge::recipe_book::RecipeBook;

/// Where materials are written when `--out` is not given.
const DEFAULT_ROOT: &str = "assets/materials";

/// Where named recipes are read from when `--recipes` is not given.
const DEFAULT_RECIPES: &str = "content/recipes";

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
     \x20 batch    <manifest.json>     realise every material a manifest declares\n\
     \x20 build    <plan.json>         realise the definitions a build plan lists,\n\
     \x20                              under its policy, then index\n\
     \x20 index                        write resources.json for the runtime\n\
     \x20 variant  <definition.json>    another material like one already written\n\
     \x20 repair   <material-id>        restore maps that are lost or corrupt\n\
     \x20 validate <material-id>       check what is on disk for a material\n\
     \x20 inspect  <material-id>       print a material's definition and origin\n\
     \x20 list                         every material written under the root\n\
     \n\
     options:\n\
     \x20 --out <dir>     where materials live (default: assets/materials)\n\
     \x20 --recipes <dir> where named recipes live (default: content/recipes)\n\
     \x20 --seed <value>  generation seed, decimal or 0x-prefixed (default: 0;\n\
     \x20                 `batch` takes the manifest's seed instead, and\n\
     \x20                 `build` refuses it: the plan owns its seeds)\n\
     \x20 --force         replace a material that is already written\n\
     \x20 --of <material-id>  the material a variant is varied from (required)\n\
     \x20 --index <n>     which variant, counted from one (default: 1)\n\
     \x20 --no-preview    skip the lit preview image\n\
     \x20 --backend <name>  procedural | ai | hybrid (default: procedural;\n\
     \x20                   the other two have no backend installed yet)\n\
     \x20 --help          show this message\n\
     \n\
     exit codes:\n\
     \x20 0  the command succeeded\n\
     \x20 1  the command ran and the answer was no\n\
     \x20 2  the command line was wrong"
        .to_owned()
}

enum Command {
    Variant {
        definition: PathBuf,
        of: Identifier,
        index: u32,
        setup: Setup,
        seed: u64,
        force: bool,
    },
    Repair {
        id: Identifier,
        setup: Setup,
    },
    Batch {
        manifest: PathBuf,
        setup: Setup,
        force: bool,
    },
    Build {
        plan: PathBuf,
        setup: Setup,
        force: bool,
    },
    Index {
        root: PathBuf,
    },
    Generate {
        definition: PathBuf,
        setup: Setup,
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
    let mut seed_given = false;
    let mut recipes = PathBuf::from(DEFAULT_RECIPES);
    let mut force = false;
    let mut of: Option<String> = None;
    let mut index = 1u32;
    let mut preview = true;
    let mut backend = Backend::Procedural;

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
                seed_given = true;
            }
            "--recipes" => {
                recipes = PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--recipes needs a path".to_owned())?,
                );
            }
            "--force" => force = true,
            "--no-preview" => preview = false,
            "--backend" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "--backend needs a name".to_owned())?;
                backend = Backend::parse(&raw).map_err(|_| {
                    format!(
                        "`{raw}` is not a backend; use {}",
                        Backend::ALL
                            .iter()
                            .map(|each| each.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;
            }
            "--of" => {
                of = Some(
                    args.next()
                        .ok_or_else(|| "--of needs a material id".to_owned())?,
                );
            }
            "--index" => {
                let raw = args
                    .next()
                    .ok_or_else(|| "--index needs a number".to_owned())?;
                index = raw
                    .parse()
                    .map_err(|_| format!("`{raw}` is not a variant index"))?;
            }
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

    let setup = Setup {
        root: root.clone(),
        recipes,
        preview,
        backend,
    };

    let needed = |what: &str| -> std::result::Result<String, String> {
        positional
            .clone()
            .ok_or_else(|| format!("`{verb}` needs {what}"))
    };

    match verb.as_str() {
        "variant" => Ok(Some(Command::Variant {
            definition: PathBuf::from(needed("a definition file")?),
            of: identifier(&of.ok_or_else(|| "`variant` needs --of <material-id>".to_owned())?)?,
            index,
            setup: setup.clone(),
            seed,
            force,
        })),
        "repair" => Ok(Some(Command::Repair {
            id: identifier(&needed("a material id")?)?,
            setup,
        })),
        "batch" => Ok(Some(Command::Batch {
            manifest: PathBuf::from(needed("a manifest file")?),
            setup,
            force,
        })),
        "build" => {
            // A seed on the command line would make the run depend on
            // something that is not in the repository, which is the one thing
            // a build plan exists to prevent.
            if seed_given {
                return Err("`build` takes its seeds from the plan, not --seed".to_owned());
            }
            Ok(Some(Command::Build {
                plan: PathBuf::from(needed("a build plan")?),
                setup,
                force,
            }))
        }
        "index" => {
            if positional.is_some() {
                return Err("`index` takes no argument".to_owned());
            }
            Ok(Some(Command::Index { root }))
        }
        "generate" => Ok(Some(Command::Generate {
            definition: PathBuf::from(needed("a definition file")?),
            setup,
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
        Command::Variant {
            definition,
            of,
            index,
            setup,
            seed,
            force,
        } => variant(&definition, &of, index, setup, seed, force),
        Command::Repair { id, setup } => repair(&id, setup),
        Command::Batch {
            manifest,
            setup,
            force,
        } => batch(&manifest, setup, force),
        Command::Build { plan, setup, force } => build(&plan, setup, force),
        Command::Index { root } => index(root),
        Command::Generate {
            definition,
            setup,
            seed,
            force,
        } => generate(&definition, setup, seed, force),
        Command::Validate { id, root } => validate(&id, root),
        Command::Inspect { id, root } => inspect(&id, root),
        Command::List { root } => list(root),
    }
}

/// How to build the forge, gathered because these always travel together:
/// where materials live, where named recipes live, whether to render
/// previews, and which backend generates.
#[derive(Debug, Clone)]
struct Setup {
    root: PathBuf,
    recipes: PathBuf,
    preview: bool,
    backend: Backend,
}

/// Build a forge with the backend that was asked for.
///
/// The one place in the tool that decides which generator runs, and the one
/// place a new one gets installed. An uninstalled backend is refused by name —
/// never quietly served by the procedural one, because a caller who asked for
/// a model and silently got noise would ship the noise.
fn forge(setup: Setup) -> Result<Forge> {
    let built = Forge::new(setup.root)?.with_recipes(RecipeBook::at(setup.recipes));
    let built = match setup.backend {
        Backend::Procedural => built,
        other => {
            return Err(nexora_foundation::error::Error::new(
                nexora_foundation::error::Domain::Content,
                "texture-forge",
                "no generator is installed for that backend",
            )
            .with_context("backend", other.as_str())
            .with_context("installed", Backend::Procedural.as_str())
            .with_recovery(nexora_foundation::error::Recovery::Reject))
        }
    };
    Ok(if setup.preview {
        built
    } else {
        built.without_previews()
    })
}

fn variant(
    path: &PathBuf,
    of: &Identifier,
    index: u32,
    setup: Setup,
    seed: u64,
    force: bool,
) -> Result<ExitCode> {
    let definition = document::from_text(&read_input(path, "the definition could not be read")?)?;
    let forge = forge(setup)?;
    let outcome = forge.variant(&definition, of, index, seed, force)?;
    report(&outcome)
}

fn repair(id: &Identifier, setup: Setup) -> Result<ExitCode> {
    let forge = forge(setup)?;
    let definition = forge.read_definition(id)?;

    // What is wrong is printed before anything is done about it: a repair that
    // only says "replaced" leaves nobody any wiser about what was lost.
    let survey = forge.survey(&definition);
    for (role, health) in &survey {
        if !health.is_intact() {
            println!("  {:<18} {}", role.as_str(), health.as_str());
        }
    }

    let outcome = forge.repair(id)?;
    if outcome.status == WriteStatus::Unchanged {
        println!("intact {id}");
        return Ok(ExitCode::SUCCESS);
    }
    report(&outcome)
}

/// Print what one run did, the same way for every verb that does one.
fn report(outcome: &Outcome) -> Result<ExitCode> {
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

fn batch(path: &PathBuf, setup: Setup, force: bool) -> Result<ExitCode> {
    let manifest = manifest::from_text(&read_input(path, "the manifest could not be read")?)?;

    let forge = forge(setup)?;
    let report = forge.batch(&manifest, force);

    for outcome in &report.outcomes {
        println!("{:<10} {}", outcome.status.as_str(), outcome.material.id());
        if let Some(refusal) = &outcome.refusal {
            eprintln!("texture-forge: {refusal}");
        }
        if outcome.validation.verdict() != Verdict::Pass {
            println!("{}", outcome.validation);
        }
    }
    for (id, cause) in &report.failures {
        eprintln!("texture-forge: {id} could not be produced -- {cause}");
    }

    println!(
        "\n{} in `{}`: {} written, {} replaced, {} unchanged, {} refused, {} failed ({})",
        manifest.len(),
        manifest.name,
        report.counted(WriteStatus::Written),
        report.counted(WriteStatus::Replaced),
        report.counted(WriteStatus::Unchanged),
        report.counted(WriteStatus::Refused),
        report.failures.len(),
        human(report.byte_len()),
    );
    Ok(if report.is_clean() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// Realise a build plan: every definition it lists, with the seed it gives,
/// under its policy -- then write the index, so the runtime is never handed
/// an index of a half-generated tree.
fn build(path: &Path, setup: Setup, force: bool) -> Result<ExitCode> {
    let plan = Plan::load(path)?;
    if !plan.is_valid() {
        for problem in &plan.problems {
            eprintln!(
                "invalid  entry {} ({}): {}",
                problem.index, problem.definition, problem.error
            );
        }
        eprintln!(
            "texture-forge: {} has {} problem(s); nothing was generated",
            path.display(),
            plan.problems.len()
        );
        return Ok(ExitCode::FAILURE);
    }

    // A policy that lists what may be written lists maps; a preview is not
    // one of them, and at twice the edge it would be the only file breaking
    // the plan's resolution.
    let setup = Setup {
        preview: setup.preview && plan.policy.allows_previews(),
        ..setup
    };
    let forge = forge(setup)?;
    let unresolved = plan.unresolved(&forge);
    if !unresolved.is_empty() {
        for problem in &unresolved {
            eprintln!(
                "invalid  entry {} ({}): {}",
                problem.index, problem.definition, problem.error
            );
        }
        eprintln!(
            "texture-forge: {} names {} recipe(s) that cannot be resolved; nothing was generated",
            path.display(),
            unresolved.len()
        );
        return Ok(ExitCode::FAILURE);
    }
    let report = plan.run(&forge, force)?;
    for entry in &report.entries {
        println!("{:<9} {}", entry.status(), entry.id);
        match &entry.result {
            Ok(outcome) => {
                if let Some(refusal) = &outcome.refusal {
                    eprintln!("  {refusal}");
                }
            }
            Err(cause) => eprintln!("  {} -- {cause}", entry.source.display()),
        }
    }
    println!("build {}: {}", plan.name, report.tally());
    if !report.succeeded() {
        return Ok(ExitCode::FAILURE);
    }
    let index = forge.write_index()?;
    println!(
        "index {} ({} resources)",
        forge.root().join(MANIFEST_FILE).display(),
        index.len()
    );
    Ok(ExitCode::SUCCESS)
}

/// The pipeline's INDEX stage on its own.
fn index(root: PathBuf) -> Result<ExitCode> {
    let forge = Forge::new(root)?;
    let index = forge.write_index()?;
    println!(
        "index {} ({} resources)",
        forge.root().join(MANIFEST_FILE).display(),
        index.len()
    );
    Ok(ExitCode::SUCCESS)
}

fn generate(path: &PathBuf, setup: Setup, seed: u64, force: bool) -> Result<ExitCode> {
    let text = read_input(path, "the definition could not be read")?;
    let definition = document::from_text(&text)?;
    report(&forge(setup)?.generate(&definition, seed, force)?)
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
            "{:<52} {:<10} {:<4} {}x{}  {} maps",
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

/// Read a file the command line named, saying which one when it cannot.
fn read_input(path: &PathBuf, message: &'static str) -> Result<String> {
    std::fs::read_to_string(path).map_err(|cause| {
        nexora_foundation::error::Error::new(
            nexora_foundation::error::Domain::Content,
            "texture-forge",
            message,
        )
        .with_context("path", path.display().to_string())
        .with_context("cause", cause.to_string())
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
