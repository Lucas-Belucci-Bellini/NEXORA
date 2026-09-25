//! Build plans: the definitions a set is made of, generated under a policy.
//!
//! Texture Forge FASE 7: *"batch por manifesto — manifesto inválido falha
//! alto"*. A build plan is the checked-in list of definition files a run
//! generates, each with its seed, so the thing that makes a run reproducible
//! lives in the repository rather than in someone's shell history:
//!
//! ```json
//! {
//!   "schema": 1,
//!   "name": "NEXORA authored materials",
//!   "seed": "0x0000000000c0ffee",
//!   "policy": { "resolution": { "width": 16, "height": 16 }, "maps": [] },
//!   "materials": [
//!     { "definition": "stone_rough.json", "seed": null },
//!     { "definition": "dark_oak_plank.json", "seed": "0x2a" }
//!   ]
//! }
//! ```
//!
//! # Not a batch manifest
//!
//! [`crate::manifest`] *declares* materials inline, from defaults, and is how a
//! new set is sketched. A build plan *points at* definitions that already
//! exist as files -- with their recipes, provenance notes and review history
//! -- and is how a set that has been authored is rebuilt, exactly. The first
//! generation is the second kind: sixteen stones, each one written and
//! reviewed by hand, and a policy they are all held to.
//!
//! # Failing loudly means failing before anything is written
//!
//! A plan is read in two passes. The first reads every definition it
//! names and checks all of them — paths, parsing, duplicate identifiers, the
//! naming rules, the policy — and **collects every problem** rather than
//! stopping at the first, for the reason [`crate::forge::Listing`] does: one
//! bad entry should not hide the forty beside it. Only a plan with no problems
//! at all may run. A build that generated the first half of a broken plan
//! and then stopped would leave a tree that matches neither the old plan
//! nor the new one, which is the state nobody can reason about.
//!
//! Once it runs, a material the forge refuses or fails on is reported and the
//! run continues: those are outcomes of generation, not defects in the list,
//! and the report says which ones happened.
//!
//! # The policy is the first generation's rule, made checkable
//!
//! The operator's rule for 2026 is *"PRIMEIRA GERAÇÃO = 16×16"*, with no
//! normal, roughness or height maps. A rule a person has to remember is a rule
//! that breaks on the three-thousandth asset, so a plan can carry it:
//! `"resolution"` pins every definition's size, and `"maps"` lists the only
//! optional maps a definition may want (albedo is always required, so `[]`
//! means albedo alone). `null` in either field means that dimension is not
//! constrained — an explicit statement, not a default.
//!
//! # Paths are content, and content is untrusted
//!
//! `NEXORA SECURITY THREAT MODEL.md` treats content as a hostile source. A
//! plan names files relative to its own directory; an absolute path or a
//! `..` segment is refused, so a plan can never make the forge read
//! outside the tree it was checked in with.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use nexora_asset::document;
use nexora_asset::json::{self, Json};
use nexora_asset::material::SurfaceMaterial;
use nexora_asset::texture::{MapRole, Resolution};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;

use crate::forge::{Forge, Outcome, WriteStatus};
use crate::layout;

/// The newest plan schema this build reads.
pub const PLAN_SCHEMA: u32 = 1;

/// Most entries a plan may list.
///
/// The whole first-generation backlog is a few thousand assets. Sixty-five
/// thousand is far past anything legitimate, and a bound is what keeps a
/// hostile plan from being a way to exhaust memory.
pub const MAX_ENTRIES: usize = 65_536;

const PLAN_FIELDS: [&str; 5] = ["schema", "name", "seed", "policy", "materials"];
const POLICY_FIELDS: [&str; 2] = ["resolution", "maps"];
const ENTRY_FIELDS: [&str; 2] = ["definition", "seed"];

/// What every definition in a plan must satisfy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Policy {
    /// The exact size every definition must declare, when pinned.
    pub resolution: Option<Resolution>,
    /// The only optional maps a definition may want, when restricted.
    ///
    /// Albedo is required of every material and is never listed here.
    pub maps: Option<Vec<MapRole>>,
}

impl Policy {
    /// Whether a lit preview may be written beside the maps.
    ///
    /// Only when the policy leaves the maps open. A policy that lists what
    /// may be written lists maps, and a preview is not one; at twice the
    /// edge it would also be the one file breaking a pinned resolution.
    #[must_use]
    pub const fn allows_previews(&self) -> bool {
        self.maps.is_none()
    }

    /// The first visual generation: 16×16, albedo alone.
    ///
    /// # Panics
    ///
    /// Never: sixteen is a valid resolution.
    #[must_use]
    pub fn first_generation() -> Self {
        Self {
            resolution: Some(Resolution::square(16).expect("16 is a valid resolution")),
            maps: Some(Vec::new()),
        }
    }

    /// Every way a definition breaks this policy, as sentences.
    #[must_use]
    pub fn violations(&self, material: &SurfaceMaterial) -> Vec<String> {
        let mut found = Vec::new();
        if let Some(pinned) = self.resolution {
            let actual = material.resolution();
            if actual != pinned {
                found.push(format!(
                    "{} is {}x{}; the plan pins every material to {}x{}",
                    material.id(),
                    actual.width,
                    actual.height,
                    pinned.width,
                    pinned.height
                ));
            }
        }
        if let Some(allowed) = &self.maps {
            for role in material.wanted_maps() {
                if !role.is_required() && !allowed.contains(role) {
                    found.push(format!(
                        "{} wants the {} map, which the plan does not allow",
                        material.id(),
                        role.as_str()
                    ));
                }
            }
        }
        found
    }
}

/// One definition a plan names, read and ready to generate.
#[derive(Debug, Clone)]
pub struct Entry {
    /// Its position in the plan's list, from zero.
    pub index: usize,
    /// The file it was read from.
    pub source: PathBuf,
    /// The definition.
    pub definition: SurfaceMaterial,
    /// The seed it is generated with: its own, or the plan's.
    pub seed: u64,
}

/// Something wrong with one entry of a plan.
#[derive(Debug)]
pub struct Problem {
    /// Which entry, by its position in the list (from zero).
    pub index: usize,
    /// The path the entry names, as written.
    pub definition: String,
    /// What is wrong.
    pub error: Error,
}

/// A plan, read and checked, and not yet run.
#[derive(Debug)]
pub struct Plan {
    /// The plan's name, for reports.
    pub name: String,
    /// The seed entries without their own use.
    pub seed: u64,
    /// What every definition had to satisfy.
    pub policy: Policy,
    /// Every entry that read back and passed, in plan order.
    pub entries: Vec<Entry>,
    /// Every entry that did not. A plan with any of these does not run.
    pub problems: Vec<Problem>,
}

impl Plan {
    /// Read a plan file and every definition it names.
    ///
    /// # Errors
    ///
    /// Returns an error when the plan itself cannot be read or is not a
    /// plan: bad JSON, an unknown or missing field, a newer schema, no
    /// entries, too many. A problem with one *entry* is not an error here; it
    /// is collected in [`Plan::problems`], so every one of them is reported.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|cause| {
            invalid("the plan could not be read")
                .with_context("path", path.display().to_string())
                .with_context("cause", cause.to_string())
        })?;
        let base = path.parent().unwrap_or_else(|| Path::new(""));
        Self::from_text(&text, base)
    }

    /// Read a plan from text, resolving entries against a directory.
    ///
    /// # Errors
    ///
    /// See [`Plan::load`].
    pub fn from_text(text: &str, base: &Path) -> Result<Self> {
        let document = json::parse(text)?;
        reject_unknown(&document, &PLAN_FIELDS, "build plan")?;

        let schema = document.field("schema")?.as_u32()?;
        if schema == 0 || schema > PLAN_SCHEMA {
            return Err(invalid("the plan schema is not one this build reads")
                .with_context("schema", schema.to_string())
                .with_context("supported", PLAN_SCHEMA.to_string()));
        }
        let name = document.field("name")?.as_text()?.to_owned();
        let seed = parse_seed(document.field("seed")?)?;
        let policy = parse_policy(document.field("policy")?)?;

        let listed = document.field("materials")?.as_array()?;
        if listed.is_empty() {
            return Err(invalid(
                "the plan lists no materials; an empty build is a mistake",
            ));
        }
        if listed.len() > MAX_ENTRIES {
            return Err(invalid("the plan lists more entries than a build accepts")
                .with_context("entries", listed.len().to_string())
                .with_context("limit", MAX_ENTRIES.to_string()));
        }

        let mut plan = Self {
            name,
            seed,
            policy,
            entries: Vec::with_capacity(listed.len()),
            problems: Vec::new(),
        };
        // Which entry first claimed each identifier, so a duplicate can name
        // the entry it collides with rather than only itself.
        let mut claimed: BTreeMap<Identifier, (usize, String)> = BTreeMap::new();

        for (index, node) in listed.iter().enumerate() {
            let written = node
                .optional_field("definition")
                .ok()
                .flatten()
                .and_then(|value| value.as_text().ok())
                .unwrap_or("")
                .to_owned();
            match plan.read_entry(index, node, base) {
                Ok(entry) => {
                    let id = entry.definition.id().clone();
                    if let Some((first, first_path)) = claimed.get(&id) {
                        plan.problems.push(Problem {
                            index,
                            definition: written,
                            error: invalid("two entries define the same material")
                                .with_context("material", id.to_string())
                                .with_context("first", format!("entry {first} ({first_path})")),
                        });
                        continue;
                    }
                    claimed.insert(id, (index, written));
                    plan.entries.push(entry);
                }
                Err(error) => plan.problems.push(Problem {
                    index,
                    definition: written,
                    error,
                }),
            }
        }
        Ok(plan)
    }

    /// Whether every entry read back and passed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.problems.is_empty()
    }

    /// Every entry whose recipe the forge cannot resolve.
    ///
    /// Separate from [`Plan::problems`] because it depends on the forge's
    /// recipe book, which the plan does not choose. Checked by
    /// [`Plan::run`] before anything is written, for the same reason the
    /// plan is: a build that stops at entry forty over a missing recipe
    /// has already written thirty-nine.
    #[must_use]
    pub fn unresolved(&self, forge: &Forge) -> Vec<Problem> {
        self.entries
            .iter()
            .filter_map(|entry| {
                forge
                    .recipes()
                    .resolve(&entry.definition)
                    .err()
                    .map(|error| Problem {
                        index: entry.index,
                        definition: entry.source.display().to_string(),
                        error,
                    })
            })
            .collect()
    }

    /// Generate every entry, in plan order.
    ///
    /// # Errors
    ///
    /// Returns an error, and writes nothing, when the plan has problems or
    /// names a recipe the forge cannot resolve. A material that fails or is
    /// refused once the run has started is not an error: it is recorded in
    /// the report and the run continues.
    pub fn run(&self, forge: &Forge, force: bool) -> Result<BuildReport> {
        if !self.is_valid() {
            return Err(invalid("the plan has problems, so nothing was generated")
                .with_context("plan", self.name.clone())
                .with_context("problems", self.problems.len().to_string()));
        }
        let unresolved = self.unresolved(forge);
        if !unresolved.is_empty() {
            return Err(invalid(
                "the plan names recipes that cannot be resolved, so nothing was generated",
            )
            .with_context("plan", self.name.clone())
            .with_context("problems", unresolved.len().to_string()));
        }
        let mut report = BuildReport::default();
        for entry in &self.entries {
            let result = forge.generate(&entry.definition, entry.seed, force);
            report.entries.push(EntryReport {
                id: entry.definition.id().clone(),
                source: entry.source.clone(),
                result,
            });
        }
        Ok(report)
    }

    fn read_entry(&self, index: usize, node: &Json, base: &Path) -> Result<Entry> {
        reject_unknown(node, &ENTRY_FIELDS, "materials[]")?;
        let written = node.field("definition")?.as_text()?;
        let source = base.join(contained(written)?);
        let seed = match node.field("seed")? {
            Json::Null => self.seed,
            value => parse_seed(value)?,
        };

        let text = std::fs::read_to_string(&source).map_err(|cause| {
            invalid("the definition could not be read")
                .with_context("path", source.display().to_string())
                .with_context("cause", cause.to_string())
        })?;
        let definition = document::from_text(&text)?;
        layout::check_naming(&definition)?;

        let violations = self.policy.violations(&definition);
        if let Some(first) = violations.first() {
            let mut error = invalid("the definition breaks the plan's policy")
                .with_context("violation", first.clone());
            for more in &violations[1..] {
                error = error.with_context("violation", more.clone());
            }
            return Err(error);
        }
        Ok(Entry {
            index,
            source,
            definition,
            seed,
        })
    }
}

/// What happened to one entry of a build.
#[derive(Debug)]
pub struct EntryReport {
    /// The material.
    pub id: Identifier,
    /// The definition file it came from.
    pub source: PathBuf,
    /// What the forge did, or why it could not.
    pub result: Result<Outcome>,
}

impl EntryReport {
    /// A stable one-word description: a [`WriteStatus`] name, or `failed`.
    #[must_use]
    pub fn status(&self) -> &'static str {
        self.result
            .as_ref()
            .map_or("failed", |outcome| outcome.status.as_str())
    }
}

/// What a build did, entry by entry.
#[derive(Debug, Default)]
pub struct BuildReport {
    /// Every entry, in plan order.
    pub entries: Vec<EntryReport>,
}

/// How many entries of a build ended each way.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    /// Not there before, written now.
    pub written: usize,
    /// There and identical; nothing rewritten.
    pub unchanged: usize,
    /// There and different; replaced at a new revision.
    pub replaced: usize,
    /// Lost or damaged files given back, same revision, same bytes.
    pub restored: usize,
    /// There and different, and not forced.
    pub refused: usize,
    /// Generation, validation or the filesystem failed.
    pub failed: usize,
}

impl Tally {
    /// Every entry counted.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.written + self.unchanged + self.replaced + self.restored + self.refused + self.failed
    }
}

impl std::fmt::Display for Tally {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} materials: {} written, {} unchanged, {} replaced, {} restored, {} refused, {} failed",
            self.total(),
            self.written,
            self.unchanged,
            self.replaced,
            self.restored,
            self.refused,
            self.failed
        )
    }
}

impl BuildReport {
    /// How many entries ended each way.
    #[must_use]
    pub fn tally(&self) -> Tally {
        let mut tally = Tally::default();
        for entry in &self.entries {
            match &entry.result {
                Ok(outcome) => match outcome.status {
                    WriteStatus::Written => tally.written += 1,
                    WriteStatus::Unchanged => tally.unchanged += 1,
                    WriteStatus::Replaced => tally.replaced += 1,
                    WriteStatus::Restored => tally.restored += 1,
                    WriteStatus::Refused => tally.refused += 1,
                },
                Err(_) => tally.failed += 1,
            }
        }
        tally
    }

    /// Whether every entry now stands as its definition describes.
    ///
    /// A refusal counts against it: the material on disk is not the one the
    /// plan describes, which is exactly what a caller asking "did the
    /// build succeed" needs to hear.
    #[must_use]
    pub fn succeeded(&self) -> bool {
        let tally = self.tally();
        tally.refused == 0 && tally.failed == 0
    }
}

/// A relative path that stays inside the directory it is resolved against.
fn contained(written: &str) -> Result<PathBuf> {
    let path = Path::new(written);
    if written.is_empty() {
        return Err(invalid("an entry names no definition file"));
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(invalid("an entry path leaves the plan's directory")
                    .with_context("path", written.to_owned()));
            }
        }
    }
    Ok(path.to_path_buf())
}

fn parse_seed(value: &Json) -> Result<u64> {
    // Hex text, as the generation trace writes it: a u64 does not fit a JSON
    // integer that anything else can read safely.
    let raw = value.as_text()?;
    raw.strip_prefix("0x")
        .filter(|digits| !digits.is_empty() && digits.len() <= 16)
        .and_then(|digits| u64::from_str_radix(digits, 16).ok())
        .ok_or_else(|| {
            invalid("a seed is written as 0x-prefixed hexadecimal text")
                .with_context("seed", raw.to_owned())
        })
}

fn parse_policy(node: &Json) -> Result<Policy> {
    if matches!(node, Json::Null) {
        return Ok(Policy::default());
    }
    reject_unknown(node, &POLICY_FIELDS, "policy")?;
    let resolution = match node.field("resolution")? {
        Json::Null => None,
        pinned => {
            reject_unknown(pinned, &["width", "height"], "policy.resolution")?;
            Some(Resolution::new(
                pinned.field("width")?.as_u32()?,
                pinned.field("height")?.as_u32()?,
            )?)
        }
    };
    let maps = match node.field("maps")? {
        Json::Null => None,
        list => {
            let mut roles = Vec::new();
            for entry in list.as_array()? {
                let role = MapRole::parse(entry.as_text()?)?;
                if role.is_required() {
                    return Err(invalid(
                        "albedo is required of every material and is not listed in a policy",
                    ));
                }
                roles.push(role);
            }
            Some(roles)
        }
    };
    Ok(Policy { resolution, maps })
}

fn reject_unknown(node: &Json, known: &[&str], section: &'static str) -> Result<()> {
    let Json::Object(fields) = node else {
        return Err(invalid("expected an object").with_context("section", section));
    };
    for key in fields.keys() {
        if !known.contains(&key.as_str()) {
            return Err(invalid("the plan carries a field this build does not know")
                .with_context("section", section)
                .with_context("field", key.clone()));
        }
    }
    Ok(())
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Content, "build-plan", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_asset::material::{MaterialCategory, PbrParameters};
    use nexora_asset::provenance::Provenance;

    /// A directory nothing else in the suite writes to.
    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "nexora-build-plan-{}-{name}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn definition(path: &str, size: u32, maps: &[MapRole], roughness: f64) -> SurfaceMaterial {
        let mut builder = SurfaceMaterial::builder(
            Identifier::parse(path).unwrap(),
            MaterialCategory::Stone,
            Resolution::square(size).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .pbr(PbrParameters {
            roughness,
            ..PbrParameters::DEFAULT
        });
        for role in maps {
            builder = builder.wants(*role);
        }
        builder.build().unwrap()
    }

    /// Write a definition into a content directory, as a person would.
    fn author(dir: &Path, file: &str, material: &SurfaceMaterial) {
        std::fs::write(dir.join(file), document::to_text(material)).unwrap();
    }

    fn plan_text(policy: &str, entries: &[&str]) -> String {
        let listed: Vec<String> = entries
            .iter()
            .map(|file| format!(r#"{{ "definition": "{file}", "seed": null }}"#))
            .collect();
        format!(
            r#"{{ "schema": 1, "name": "test", "seed": "0x2a", "policy": {policy}, "materials": [{}] }}"#,
            listed.join(", ")
        )
    }

    const FIRST_GENERATION: &str = r#"{ "resolution": { "width": 16, "height": 16 }, "maps": [] }"#;

    #[test]
    fn a_build_generates_every_entry_and_a_second_pass_changes_nothing() {
        let content = scratch("run-content");
        let out = scratch("run-out");
        author(
            &content,
            "a.json",
            &definition("nexora:material/a", 16, &[], 0.9),
        );
        author(
            &content,
            "b.json",
            &definition("nexora:material/b", 16, &[], 0.5),
        );
        let text = plan_text("null", &["a.json", "b.json"]);

        let plan = Plan::from_text(&text, &content).expect("a valid plan");
        assert!(plan.is_valid(), "{:?}", plan.problems);
        assert_eq!(plan.entries.len(), 2);

        let forge = Forge::new(&out).unwrap();
        let first = plan.run(&forge, false).expect("runs");
        assert!(first.succeeded());
        assert_eq!(
            first.tally(),
            Tally {
                written: 2,
                ..Tally::default()
            }
        );
        // Plan order, not directory order or identifier order.
        let order: Vec<String> = first.entries.iter().map(|e| e.id.to_string()).collect();
        assert_eq!(order, ["nexora:material/a", "nexora:material/b"]);

        let second = plan.run(&forge, false).expect("runs again");
        assert_eq!(
            second.tally(),
            Tally {
                unchanged: 2,
                ..Tally::default()
            },
            "a deterministic build rewrites nothing"
        );
        assert_eq!(forge.list().unwrap().len(), 2);

        let _ = std::fs::remove_dir_all(&content);
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn the_plan_seed_is_used_unless_an_entry_names_its_own() {
        let content = scratch("seed");
        author(
            &content,
            "a.json",
            &definition("nexora:material/a", 16, &[], 0.9),
        );
        author(
            &content,
            "b.json",
            &definition("nexora:material/b", 16, &[], 0.9),
        );
        let text = r#"{ "schema": 1, "name": "seeds", "seed": "0x2a", "policy": null,
            "materials": [ { "definition": "a.json", "seed": null },
                           { "definition": "b.json", "seed": "0xC0FFEE" } ] }"#;
        let plan = Plan::from_text(text, &content).unwrap();
        assert_eq!(plan.entries[0].seed, 0x2a);
        assert_eq!(plan.entries[1].seed, 0x00C0_FFEE);

        for bad in ["42", "0x", "0xZZ", "0x11111111111111111"] {
            let text = format!(
                r#"{{ "schema": 1, "name": "x", "seed": "{bad}", "policy": null,
                    "materials": [ {{ "definition": "a.json", "seed": null }} ] }}"#
            );
            let err = Plan::from_text(&text, &content).expect_err(bad);
            assert!(err.to_string().contains("seed"), "{bad}: {err}");
        }
        let _ = std::fs::remove_dir_all(&content);
    }

    #[test]
    fn every_problem_is_reported_and_nothing_is_generated() {
        let content = scratch("problems");
        let out = scratch("problems-out");
        author(
            &content,
            "good.json",
            &definition("nexora:material/good", 16, &[], 0.9),
        );
        author(
            &content,
            "twin.json",
            &definition("nexora:material/good", 16, &[], 0.4),
        );
        author(
            &content,
            "big.json",
            &definition("nexora:material/big", 64, &[], 0.9),
        );
        author(
            &content,
            "pbr.json",
            &definition(
                "nexora:material/pbr",
                16,
                &[MapRole::Normal, MapRole::Height],
                0.9,
            ),
        );
        std::fs::write(content.join("broken.json"), "{ not json").unwrap();

        let text = plan_text(
            FIRST_GENERATION,
            &[
                "good.json",
                "twin.json",
                "big.json",
                "pbr.json",
                "broken.json",
                "absent.json",
                "../escape.json",
            ],
        );
        let plan = Plan::from_text(&text, &content).expect("the plan itself parses");
        assert!(!plan.is_valid());
        assert_eq!(plan.entries.len(), 1, "only good.json passes");

        let indices: Vec<usize> = plan.problems.iter().map(|p| p.index).collect();
        assert_eq!(indices, [1, 2, 3, 4, 5, 6], "every problem, not the first");
        let said = |index: usize| plan.problems[index].error.to_string();
        assert!(said(0).contains("same material"), "{}", said(0));
        assert!(
            said(0).contains("good.json"),
            "names the entry it collides with"
        );
        assert!(said(1).contains("64x64"), "{}", said(1));
        assert!(
            said(2).contains("normal") && said(2).contains("height"),
            "{}",
            said(2)
        );
        assert!(said(5).contains("leaves the plan"), "{}", said(5));
        assert_eq!(plan.problems[5].definition, "../escape.json");

        let forge = Forge::new(&out).unwrap();
        let err = plan
            .run(&forge, false)
            .expect_err("a broken plan does not run");
        assert!(err.to_string().contains("nothing was generated"), "{err}");
        assert!(
            forge.list().unwrap().is_empty(),
            "not even the one good entry is written"
        );

        let _ = std::fs::remove_dir_all(&content);
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn the_first_generation_policy_is_sixteen_square_and_albedo_alone() {
        let policy = Policy::first_generation();
        assert!(policy
            .violations(&definition("nexora:material/ok", 16, &[], 0.9))
            .is_empty());
        assert_eq!(
            policy
                .violations(&definition(
                    "nexora:material/big",
                    32,
                    &[MapRole::Roughness],
                    0.9
                ))
                .len(),
            2,
            "size and map are separate violations"
        );

        let parsed = parse_policy(&json::parse(FIRST_GENERATION).unwrap()).unwrap();
        assert_eq!(parsed, policy, "the written form and the constructor agree");
        assert_eq!(parse_policy(&Json::Null).unwrap(), Policy::default());

        let err =
            parse_policy(&json::parse(r#"{ "resolution": null, "maps": ["albedo"] }"#).unwrap())
                .expect_err("albedo is not optional");
        assert!(err.to_string().contains("albedo"), "{err}");
    }

    #[test]
    fn a_malformed_plan_is_an_error_not_a_plan() {
        let base = Path::new(".");
        let cases = [
            ("{ not json", "json"),
            (
                r#"{ "schema": 2, "name": "x", "seed": "0x1", "policy": null, "materials": [] }"#,
                "schema",
            ),
            (
                r#"{ "schema": 1, "name": "x", "seed": "0x1", "policy": null, "materials": [] }"#,
                "no materials",
            ),
            (
                r#"{ "schema": 1, "name": "x", "seed": "0x1", "policy": null, "materials": [], "extra": 1 }"#,
                "extra",
            ),
            (
                r#"{ "schema": 1, "name": "x", "policy": null, "materials": [] }"#,
                "seed",
            ),
            (
                r#"{ "schema": 1, "name": "x", "seed": "0x1", "policy": { "resolution": null }, "materials": [] }"#,
                "maps",
            ),
        ];
        for (text, needle) in cases {
            let err = Plan::from_text(text, base).expect_err(text);
            assert!(
                err.to_string().to_lowercase().contains(needle),
                "`{text}` should mention `{needle}`: {err}"
            );
        }
    }

    #[test]
    fn an_entry_with_an_unknown_field_is_a_problem_not_a_silent_skip() {
        let content = scratch("entry-field");
        author(
            &content,
            "a.json",
            &definition("nexora:material/a", 16, &[], 0.9),
        );
        let text = r#"{ "schema": 1, "name": "x", "seed": "0x1", "policy": null,
            "materials": [ { "definition": "a.json", "seed": null, "sede": "0x2" } ] }"#;
        let plan = Plan::from_text(text, &content).unwrap();
        assert_eq!(plan.problems.len(), 1);
        assert!(plan.problems[0].error.to_string().contains("sede"));
        let _ = std::fs::remove_dir_all(&content);
    }

    #[test]
    fn a_refusal_during_the_run_is_reported_and_the_run_continues() {
        let content = scratch("refusal");
        let out = scratch("refusal-out");
        author(
            &content,
            "a.json",
            &definition("nexora:material/a", 16, &[], 0.9),
        );
        author(
            &content,
            "b.json",
            &definition("nexora:material/b", 16, &[], 0.9),
        );
        let forge = Forge::new(&out).unwrap();
        Plan::from_text(&plan_text("null", &["a.json", "b.json"]), &content)
            .unwrap()
            .run(&forge, false)
            .unwrap();

        // Someone edits one definition without forcing.
        author(
            &content,
            "a.json",
            &definition("nexora:material/a", 16, &[], 0.1),
        );
        let plan = Plan::from_text(&plan_text("null", &["a.json", "b.json"]), &content).unwrap();
        let report = plan.run(&forge, false).unwrap();
        assert!(!report.succeeded(), "a refusal is not success");
        assert_eq!(report.entries[0].status(), "refused");
        assert_eq!(report.entries[1].status(), "unchanged", "the run went on");

        let forced = plan.run(&forge, true).unwrap();
        assert!(forced.succeeded());
        assert_eq!(forced.tally().replaced, 1);
        assert_eq!(forced.tally().unchanged, 1);

        let _ = std::fs::remove_dir_all(&content);
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn a_missing_recipe_stops_the_build_before_anything_is_written() {
        let content = scratch("recipe-content");
        let out = scratch("recipe-out");
        author(
            &content,
            "a.json",
            &definition("nexora:material/a", 16, &[], 0.9),
        );
        let named = SurfaceMaterial::builder(
            Identifier::parse("nexora:material/b").unwrap(),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .recipe(Identifier::parse("nexora:recipe/stone/absent").unwrap())
        .build()
        .unwrap();
        author(&content, "b.json", &named);

        let plan = Plan::from_text(&plan_text("null", &["a.json", "b.json"]), &content).unwrap();
        assert!(plan.is_valid(), "the plan itself is fine");

        let forge = Forge::new(&out).unwrap();
        let unresolved = plan.unresolved(&forge);
        assert_eq!(unresolved.len(), 1);
        assert_eq!(unresolved[0].index, 1);

        let err = plan.run(&forge, false).expect_err("stops");
        assert!(err.to_string().contains("recipes"), "{err}");
        assert!(
            forge.list().unwrap().is_empty(),
            "not even entry 0 was written"
        );

        let _ = std::fs::remove_dir_all(&content);
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn a_path_is_contained_or_it_is_refused() {
        assert!(contained("a.json").is_ok());
        assert!(contained("stone/granite.json").is_ok());
        assert!(contained("./a.json").is_ok());
        for escaping in ["", "../a.json", "stone/../../a.json", "/etc/passwd"] {
            assert!(contained(escaping).is_err(), "`{escaping}` must be refused");
        }
    }
}
