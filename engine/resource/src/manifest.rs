//! The manifest: what exists, where, and what it must hash to.
//!
//! The **INDEX** stage of `NEXORA CONTENT PIPELINE SPECIFICATION.md`
//! (`… → PACKAGE → INDEX → RUNTIME RESOURCE`). A content tool writes it; the
//! runtime reads it and trusts nothing else about the files it names.
//!
//! ```json
//! {
//!   "schema": 1,
//!   "resources": [
//!     { "id": "nexora:texture/stone/basalt/albedo", "kind": "texture",
//!       "path": "nexora/stone/basalt/albedo.png", "size": 337,
//!       "hash": "0x303ca201317ae40b", "dependencies": [],
//!       "optional": false, "fallback": null }
//!   ]
//! }
//! ```
//!
//! # Checked whole, at read time
//!
//! A manifest that names a dependency or a fallback it does not contain, that
//! has a dependency cycle, a fallback of another kind, a fallback on a
//! required entry, or a path that leaves its root, is refused before any
//! resource is loaded from it. Finding those at load time would mean finding
//! them in front of a player.
//!
//! # Checked against the content, before loading
//!
//! A manifest can be internally sound and still not provide what the content
//! asks for: the forge indexes the map files it finds, so a material whose
//! albedo was never written is indexed without it, and the runtime finds out
//! at the first load. [`Manifest::gaps`] is the `CROSS-SYSTEM` level of
//! `NEXORA DATA VALIDATION AND INVARIANTS.md` for that boundary: every
//! material, every map it asks for, and the link between them — all of them
//! reported at once, not the first one a loader trips over.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use nexora_asset::json::{self, Json};
use nexora_asset::material::SurfaceMaterial;
use nexora_asset::texture::MapRole;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;

/// The file a manifest is written to, at the root of what it indexes.
pub const MANIFEST_FILE: &str = "resources.json";

/// The newest manifest schema this build reads.
pub const MANIFEST_SCHEMA: u32 = 1;

/// Most entries one manifest may hold.
pub const MAX_ENTRIES: usize = 1_048_576;

/// Largest single resource the manifest may declare, in bytes.
///
/// A size is read before a file is: a manifest cannot make the runtime
/// allocate more than this for one resource, whatever the file holds.
pub const MAX_RESOURCE_BYTES: u64 = 256 * 1024 * 1024;

const MANIFEST_FIELDS: [&str; 2] = ["schema", "resources"];
const ENTRY_FIELDS: [&str; 8] = [
    "id",
    "kind",
    "path",
    "size",
    "hash",
    "dependencies",
    "optional",
    "fallback",
];

/// What kind of thing a resource is.
///
/// The list `RESOURCE AND ASSET SYSTEM.md` names. A kind is checked when a
/// resource is resolved, so asking for a texture by the id of a sound is an
/// error rather than a sound decoded as pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ResourceKind {
    /// Image data a renderer samples.
    Texture,
    /// A surface material definition.
    Material,
    /// Geometry.
    Mesh,
    /// A shader program.
    Shader,
    /// Sound.
    Audio,
    /// A typeface.
    Font,
    /// Motion data.
    Animation,
    /// Interface layout or imagery.
    Ui,
    /// Data-driven content: block lists, recipes, tables.
    Data,
}

impl ResourceKind {
    /// Every kind, in declaration order.
    pub const ALL: [Self; 9] = [
        Self::Texture,
        Self::Material,
        Self::Mesh,
        Self::Shader,
        Self::Audio,
        Self::Font,
        Self::Animation,
        Self::Ui,
        Self::Data,
    ];

    /// Stable lowercase name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Texture => "texture",
            Self::Material => "material",
            Self::Mesh => "mesh",
            Self::Shader => "shader",
            Self::Audio => "audio",
            Self::Font => "font",
            Self::Animation => "animation",
            Self::Ui => "ui",
            Self::Data => "data",
        }
    }

    /// Parse a name written by [`ResourceKind::as_str`].
    ///
    /// # Errors
    ///
    /// Returns an error for any other text.
    pub fn parse(raw: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == raw)
            .ok_or_else(|| invalid("not a resource kind").with_context("kind", raw.to_owned()))
    }
}

/// One resource the manifest declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    /// The identifier the runtime asks for.
    pub id: Identifier,
    /// What it is.
    pub kind: ResourceKind,
    /// Where it is, relative to the manifest's root. Never shown to callers.
    pub path: String,
    /// Exactly how many bytes the file holds.
    pub size: u64,
    /// FNV-1a 64 of those bytes.
    pub hash: u64,
    /// Resources this one needs loaded first.
    pub dependencies: Vec<Identifier>,
    /// Whether the runtime may continue without it.
    pub optional: bool,
    /// What stands in for it when it cannot be read. Only on optional entries.
    pub fallback: Option<Identifier>,
}

/// One thing the content asks for that a manifest does not provide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gap {
    /// No entry for the material.
    MissingMaterial(Identifier),
    /// No entry for one of its maps.
    MissingMap {
        /// The material asking.
        material: Identifier,
        /// Which map.
        role: MapRole,
        /// The identifier the map would have.
        texture: Identifier,
    },
    /// An entry under the right identifier, of the wrong kind.
    WrongKind {
        /// The identifier.
        id: Identifier,
        /// The kind the content needs.
        expected: ResourceKind,
        /// The kind the manifest declares.
        found: ResourceKind,
    },
    /// The material and the map are both there, but loading the material
    /// would not load the map.
    UnlinkedMap {
        /// The material.
        material: Identifier,
        /// Its map.
        texture: Identifier,
    },
}

impl std::fmt::Display for Gap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingMaterial(id) => write!(f, "{id}: no material entry"),
            Self::MissingMap {
                material,
                role,
                texture,
            } => write!(f, "{material}: no {} map ({texture})", role.as_str()),
            Self::WrongKind {
                id,
                expected,
                found,
            } => write!(
                f,
                "{id}: declared as {}, needed as {}",
                found.as_str(),
                expected.as_str()
            ),
            Self::UnlinkedMap { material, texture } => {
                write!(f, "{material}: does not depend on {texture}")
            }
        }
    }
}

/// Every resource under one root, checked.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Manifest {
    entries: BTreeMap<Identifier, ManifestEntry>,
}

impl Manifest {
    /// Build a manifest from entries, checking them as a whole.
    ///
    /// # Errors
    ///
    /// Returns an error for a duplicate identifier, an unsafe path, an
    /// oversized entry, a dangling dependency or fallback, a fallback on a
    /// required entry or of another kind, or a dependency cycle.
    pub fn new(entries: impl IntoIterator<Item = ManifestEntry>) -> Result<Self> {
        let mut map = BTreeMap::new();
        for entry in entries {
            check_path(&entry.path)
                .map_err(|error| error.with_context("resource", entry.id.to_string()))?;
            if entry.size > MAX_RESOURCE_BYTES {
                return Err(invalid("a resource is larger than the runtime accepts")
                    .with_context("resource", entry.id.to_string())
                    .with_context("size", entry.size.to_string()));
            }
            if entry.fallback.is_some() && !entry.optional {
                return Err(invalid("only an optional resource may declare a fallback")
                    .with_context("resource", entry.id.to_string()));
            }
            if map.contains_key(&entry.id) {
                return Err(invalid("a resource is declared twice")
                    .with_context("resource", entry.id.to_string()));
            }
            map.insert(entry.id.clone(), entry);
        }
        if map.len() > MAX_ENTRIES {
            return Err(invalid("the manifest holds more entries than one may")
                .with_context("entries", map.len().to_string()));
        }

        let manifest = Self { entries: map };
        for entry in manifest.entries.values() {
            for dependency in &entry.dependencies {
                if !manifest.entries.contains_key(dependency) {
                    return Err(
                        invalid("a resource depends on one the manifest does not declare")
                            .with_context("resource", entry.id.to_string())
                            .with_context("dependency", dependency.to_string()),
                    );
                }
            }
            if let Some(fallback) = &entry.fallback {
                let Some(target) = manifest.entries.get(fallback) else {
                    return Err(invalid(
                        "a fallback names a resource the manifest does not declare",
                    )
                    .with_context("resource", entry.id.to_string())
                    .with_context("fallback", fallback.to_string()));
                };
                if target.kind != entry.kind {
                    return Err(invalid("a fallback is a different kind of resource")
                        .with_context("resource", entry.id.to_string())
                        .with_context("fallback", fallback.to_string()));
                }
                if target.optional {
                    // A fallback that can itself be missing is a chain that
                    // can end nowhere; the last resort must be required.
                    return Err(invalid("a fallback must itself be required")
                        .with_context("resource", entry.id.to_string())
                        .with_context("fallback", fallback.to_string()));
                }
            }
        }
        manifest.check_acyclic()?;
        Ok(manifest)
    }

    /// The entry for an identifier.
    #[must_use]
    pub fn get(&self, id: &Identifier) -> Option<&ManifestEntry> {
        self.entries.get(id)
    }

    /// Every entry, in identifier order.
    pub fn entries(&self) -> impl Iterator<Item = &ManifestEntry> {
        self.entries.values()
    }

    /// How many resources are declared.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether nothing is declared.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Everything this manifest fails to provide for `materials`.
    ///
    /// For each material: an entry of kind `material` under its identifier;
    /// for each map it asks for (the required ones and the ones it wants), an
    /// entry of kind `texture` under the map's derived identifier; and the
    /// material entry depending on that texture, so loading the material loads
    /// its maps. Empty means the manifest provides all of it.
    ///
    /// # Errors
    ///
    /// Returns an error only when a map identifier cannot be derived — the
    /// material itself is malformed, which is a different failure from a gap.
    pub fn gaps(&self, materials: &[SurfaceMaterial]) -> Result<Vec<Gap>> {
        let mut gaps = Vec::new();
        for material in materials {
            let entry = self.entries.get(material.id());
            match entry {
                None => gaps.push(Gap::MissingMaterial(material.id().clone())),
                Some(entry) if entry.kind != ResourceKind::Material => gaps.push(Gap::WrongKind {
                    id: entry.id.clone(),
                    expected: ResourceKind::Material,
                    found: entry.kind,
                }),
                Some(_) => {}
            }
            for role in MapRole::ALL {
                if !(role.is_required() || material.wanted_maps().contains(&role)) {
                    continue;
                }
                let texture = material.map_asset_id(role)?;
                match self.entries.get(&texture) {
                    None => gaps.push(Gap::MissingMap {
                        material: material.id().clone(),
                        role,
                        texture,
                    }),
                    Some(found) if found.kind != ResourceKind::Texture => {
                        gaps.push(Gap::WrongKind {
                            id: texture,
                            expected: ResourceKind::Texture,
                            found: found.kind,
                        });
                    }
                    Some(_) => {
                        if entry.is_some_and(|entry| {
                            entry.kind == ResourceKind::Material
                                && !entry.dependencies.contains(&texture)
                        }) {
                            gaps.push(Gap::UnlinkedMap {
                                material: material.id().clone(),
                                texture,
                            });
                        }
                    }
                }
            }
        }
        Ok(gaps)
    }

    /// Fail unless this manifest provides everything `materials` asks for.
    ///
    /// # Errors
    ///
    /// Returns an error naming how many gaps there are and every one of them,
    /// or the error from [`Manifest::gaps`].
    pub fn provides(&self, materials: &[SurfaceMaterial]) -> Result<()> {
        let gaps = self.gaps(materials)?;
        if gaps.is_empty() {
            return Ok(());
        }
        let mut error = invalid("the resource manifest does not provide what the content asks for")
            .with_context("gaps", gaps.len().to_string());
        for gap in &gaps {
            error = error.with_context("gap", gap.to_string());
        }
        Err(error)
    }

    /// A resource and everything it depends on, dependencies first.
    ///
    /// # Errors
    ///
    /// Returns an error when the identifier is not declared.
    pub fn load_order(&self, id: &Identifier) -> Result<Vec<Identifier>> {
        if !self.entries.contains_key(id) {
            return Err(missing(id));
        }
        let mut order = Vec::new();
        let mut seen = BTreeSet::new();
        self.visit(id, &mut seen, &mut order);
        Ok(order)
    }

    fn visit(&self, id: &Identifier, seen: &mut BTreeSet<Identifier>, order: &mut Vec<Identifier>) {
        if !seen.insert(id.clone()) {
            return;
        }
        if let Some(entry) = self.entries.get(id) {
            for dependency in &entry.dependencies {
                self.visit(dependency, seen, order);
            }
        }
        order.push(id.clone());
    }

    fn check_acyclic(&self) -> Result<()> {
        // Iterative three-colour walk: a recursive one is a stack a hostile
        // manifest could exhaust with a long enough chain.
        let mut state: BTreeMap<&Identifier, u8> = BTreeMap::new();
        for root in self.entries.keys() {
            if state.get(root).copied() == Some(2) {
                continue;
            }
            let mut stack: Vec<(&Identifier, usize)> = vec![(root, 0)];
            state.insert(root, 1);
            while let Some((node, next)) = stack.pop() {
                let dependencies = &self.entries[node].dependencies;
                if next < dependencies.len() {
                    stack.push((node, next + 1));
                    let child = &dependencies[next];
                    match state.get(child).copied() {
                        Some(1) => {
                            return Err(invalid("the manifest's dependencies form a cycle")
                                .with_context("resource", child.to_string()));
                        }
                        Some(2) => {}
                        _ => {
                            state.insert(child, 1);
                            stack.push((child, 0));
                        }
                    }
                } else {
                    state.insert(node, 2);
                }
            }
        }
        Ok(())
    }

    /// Read a manifest from text.
    ///
    /// # Errors
    ///
    /// Returns an error when the text is not a manifest, or the entries fail
    /// [`Manifest::new`]'s checks. Unknown fields are refused.
    pub fn from_text(text: &str) -> Result<Self> {
        let document = json::parse(text)?;
        reject_unknown(&document, &MANIFEST_FIELDS, "manifest")?;
        let schema = document.field("schema")?.as_u32()?;
        if schema == 0 || schema > MANIFEST_SCHEMA {
            return Err(
                invalid("the resource manifest schema is not one this build reads")
                    .with_context("schema", schema.to_string())
                    .with_context("supported", MANIFEST_SCHEMA.to_string()),
            );
        }
        let mut entries = Vec::new();
        for node in document.field("resources")?.as_array()? {
            reject_unknown(node, &ENTRY_FIELDS, "resources[]")?;
            let hash_text = node.field("hash")?.as_text()?;
            let hash = hash_text
                .strip_prefix("0x")
                .and_then(|digits| u64::from_str_radix(digits, 16).ok())
                .ok_or_else(|| {
                    invalid("a hash is 0x-prefixed hexadecimal text")
                        .with_context("hash", hash_text.to_owned())
                })?;
            let size = u64::try_from(node.field("size")?.as_integer()?)
                .map_err(|_| invalid("a size cannot be negative"))?;
            let mut dependencies = Vec::new();
            for dependency in node.field("dependencies")?.as_array()? {
                dependencies.push(Identifier::parse(dependency.as_text()?)?);
            }
            entries.push(ManifestEntry {
                id: Identifier::parse(node.field("id")?.as_text()?)?,
                kind: ResourceKind::parse(node.field("kind")?.as_text()?)?,
                path: node.field("path")?.as_text()?.to_owned(),
                size,
                hash,
                dependencies,
                optional: node.field("optional")?.as_bool()?,
                fallback: match node.field("fallback")? {
                    Json::Null => None,
                    value => Some(Identifier::parse(value.as_text()?)?),
                },
            });
        }
        Self::new(entries)
    }

    /// Render the manifest as text `from_text` reads, byte-identical on a
    /// round trip.
    #[must_use]
    pub fn to_text(&self) -> String {
        let resources = self
            .entries
            .values()
            .map(|entry| {
                Json::object([
                    ("id", Json::text(entry.id.to_string())),
                    ("kind", Json::text(entry.kind.as_str())),
                    ("path", Json::text(&entry.path)),
                    // Bounded by MAX_RESOURCE_BYTES, which fits an i64.
                    ("size", Json::Integer(entry.size as i64)),
                    ("hash", Json::text(format!("{:#018x}", entry.hash))),
                    (
                        "dependencies",
                        Json::Array(
                            entry
                                .dependencies
                                .iter()
                                .map(|dependency| Json::text(dependency.to_string()))
                                .collect(),
                        ),
                    ),
                    ("optional", Json::Bool(entry.optional)),
                    (
                        "fallback",
                        entry
                            .fallback
                            .as_ref()
                            .map_or(Json::Null, |fallback| Json::text(fallback.to_string())),
                    ),
                ])
            })
            .collect();
        Json::object([
            ("schema", Json::Integer(i64::from(MANIFEST_SCHEMA))),
            ("resources", Json::Array(resources)),
        ])
        .to_pretty()
    }
}

/// A manifest path must stay inside its root.
fn check_path(path: &str) -> Result<()> {
    let escapes = path.is_empty()
        || path.contains('\\')
        || Path::new(path)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)));
    if escapes {
        return Err(
            invalid("a resource path must be relative, and stay inside its root")
                .with_context("path", path.to_owned()),
        );
    }
    Ok(())
}

fn reject_unknown(node: &Json, known: &[&str], section: &'static str) -> Result<()> {
    let Json::Object(fields) = node else {
        return Err(invalid("expected an object").with_context("section", section));
    };
    for key in fields.keys() {
        if !known.contains(&key.as_str()) {
            return Err(
                invalid("the manifest carries a field this build does not know")
                    .with_context("section", section)
                    .with_context("field", key.clone()),
            );
        }
    }
    Ok(())
}

pub(crate) fn missing(id: &Identifier) -> Error {
    Error::new(
        Domain::Content,
        "resource",
        "no resource is declared under this id",
    )
    .with_recovery(Recovery::Reject)
    .with_context("resource", id.to_string())
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Content, "resource-manifest", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).unwrap()
    }

    pub(crate) fn entry(raw: &str, kind: ResourceKind) -> ManifestEntry {
        ManifestEntry {
            id: id(raw),
            kind,
            path: format!("{}.bin", raw.replace([':', '/'], "_")),
            size: 4,
            hash: 0,
            dependencies: Vec::new(),
            optional: false,
            fallback: None,
        }
    }

    #[test]
    fn a_manifest_round_trips_byte_for_byte() {
        let mut material = entry("nexora:material/stone", ResourceKind::Material);
        material.dependencies = vec![id("nexora:texture/stone/albedo")];
        let mut optional = entry("nexora:texture/stone/detail", ResourceKind::Texture);
        optional.optional = true;
        optional.fallback = Some(id("nexora:texture/stone/albedo"));
        let manifest = Manifest::new([
            entry("nexora:texture/stone/albedo", ResourceKind::Texture),
            material,
            optional,
        ])
        .unwrap();

        let text = manifest.to_text();
        let again = Manifest::from_text(&text).unwrap();
        assert_eq!(again, manifest);
        assert_eq!(again.to_text(), text);
    }

    #[test]
    fn dependencies_load_first_and_each_only_once() {
        let mut a = entry("nexora:data/a", ResourceKind::Data);
        a.dependencies = vec![id("nexora:data/b"), id("nexora:data/c")];
        let mut b = entry("nexora:data/b", ResourceKind::Data);
        b.dependencies = vec![id("nexora:data/c")];
        let c = entry("nexora:data/c", ResourceKind::Data);
        let manifest = Manifest::new([a, b, c]).unwrap();
        let order: Vec<String> = manifest
            .load_order(&id("nexora:data/a"))
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(order, ["nexora:data/c", "nexora:data/b", "nexora:data/a"]);
    }

    #[test]
    fn a_manifest_that_cannot_be_trusted_is_refused_whole() {
        let t = |raw| entry(raw, ResourceKind::Texture);

        let mut escape = t("nexora:texture/x");
        escape.path = "../x.png".to_owned();
        let mut rooted = t("nexora:texture/x");
        rooted.path = "/etc/x".to_owned();
        let mut dangling = t("nexora:texture/x");
        dangling.dependencies = vec![id("nexora:texture/absent")];
        let mut required_with_fallback = t("nexora:texture/x");
        required_with_fallback.fallback = Some(id("nexora:texture/y"));
        let mut wrong_kind = t("nexora:texture/x");
        wrong_kind.optional = true;
        wrong_kind.fallback = Some(id("nexora:data/y"));
        let mut huge = t("nexora:texture/x");
        huge.size = MAX_RESOURCE_BYTES + 1;
        let mut loop_a = t("nexora:texture/a");
        loop_a.dependencies = vec![id("nexora:texture/b")];
        let mut loop_b = t("nexora:texture/b");
        loop_b.dependencies = vec![id("nexora:texture/a")];

        let cases: Vec<(Vec<ManifestEntry>, &str)> = vec![
            (vec![escape], "inside its root"),
            (vec![rooted], "inside its root"),
            (vec![dangling], "does not declare"),
            (
                vec![required_with_fallback, t("nexora:texture/y")],
                "optional",
            ),
            (
                vec![wrong_kind, entry("nexora:data/y", ResourceKind::Data)],
                "different kind",
            ),
            (vec![huge], "larger"),
            (vec![loop_a, loop_b], "cycle"),
            (vec![t("nexora:texture/x"), t("nexora:texture/x")], "twice"),
        ];
        for (entries, needle) in cases {
            let err = Manifest::new(entries).expect_err(needle);
            assert!(err.to_string().contains(needle), "`{needle}`: {err}");
        }
    }

    #[test]
    fn malformed_text_names_what_is_wrong() {
        let good = Manifest::new([entry("nexora:texture/x", ResourceKind::Texture)])
            .unwrap()
            .to_text();
        for (text, needle) in [
            (good.replace("\"schema\": 1", "\"schema\": 9"), "schema"),
            (good.replace("\"texture\"", "\"picture\""), "kind"),
            (
                good.replace("\"optional\": false", "\"optional\": false, \"extra\": 1"),
                "extra",
            ),
            (good.replace("0x0000000000000000", "12"), "hash"),
            (good.replace("\"size\": 4", "\"size\": -4"), "negative"),
        ] {
            let err = Manifest::from_text(&text).expect_err(needle);
            assert!(err.to_string().contains(needle), "`{needle}`: {err}");
        }
    }

    fn material(raw: &str, wants: &[MapRole]) -> SurfaceMaterial {
        use nexora_asset::material::MaterialCategory;
        use nexora_asset::provenance::{GenerationTrace, Provenance};
        use nexora_asset::texture::Resolution;
        use nexora_foundation::version::ContentGeneratorVersion;
        let trace = GenerationTrace::new(
            id("nexora:generator/procedural"),
            ContentGeneratorVersion(1),
            1,
        );
        let mut builder = SurfaceMaterial::builder(
            id(raw),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            Provenance::generated("NEXORA", "test", trace),
        );
        for role in wants {
            builder = builder.wants(*role);
        }
        builder.build().unwrap()
    }

    fn material_entry(raw: &str, maps: &[&str]) -> ManifestEntry {
        ManifestEntry {
            dependencies: maps.iter().map(|map| id(map)).collect(),
            ..entry(raw, ResourceKind::Material)
        }
    }

    #[test]
    fn a_manifest_that_provides_every_map_has_no_gaps() {
        let manifest = Manifest::new([
            entry("nexora:texture/basalt/albedo", ResourceKind::Texture),
            entry("nexora:texture/basalt/normal", ResourceKind::Texture),
            material_entry(
                "nexora:material/basalt",
                &[
                    "nexora:texture/basalt/albedo",
                    "nexora:texture/basalt/normal",
                ],
            ),
        ])
        .unwrap();
        let basalt = material("nexora:material/basalt", &[MapRole::Normal]);
        let materials = [basalt];
        assert_eq!(manifest.gaps(&materials).unwrap(), []);
        assert!(manifest.provides(&materials).is_ok());
    }

    #[test]
    fn every_gap_is_reported_not_just_the_first() {
        let manifest = Manifest::new([
            // basalt: the material is there, its normal map is not, and its
            // albedo is there but the material does not depend on it.
            entry("nexora:texture/basalt/albedo", ResourceKind::Texture),
            material_entry("nexora:material/basalt", &[]),
            // granite: its albedo is declared as data.
            entry("nexora:texture/granite/albedo", ResourceKind::Data),
            material_entry("nexora:material/granite", &[]),
        ])
        .unwrap();
        let materials = [
            material("nexora:material/basalt", &[MapRole::Normal]),
            material("nexora:material/granite", &[]),
            // marble: nothing at all.
            material("nexora:material/marble", &[]),
        ];
        let gaps = manifest.gaps(&materials).unwrap();
        assert_eq!(
            gaps,
            [
                Gap::UnlinkedMap {
                    material: id("nexora:material/basalt"),
                    texture: id("nexora:texture/basalt/albedo"),
                },
                Gap::MissingMap {
                    material: id("nexora:material/basalt"),
                    role: MapRole::Normal,
                    texture: id("nexora:texture/basalt/normal"),
                },
                Gap::WrongKind {
                    id: id("nexora:texture/granite/albedo"),
                    expected: ResourceKind::Texture,
                    found: ResourceKind::Data,
                },
                Gap::MissingMaterial(id("nexora:material/marble")),
                Gap::MissingMap {
                    material: id("nexora:material/marble"),
                    role: MapRole::Albedo,
                    texture: id("nexora:texture/marble/albedo"),
                },
            ]
        );
        let err = manifest.provides(&materials).unwrap_err();
        let text = err.to_string();
        assert!(text.contains("gaps=5"), "{text}");
        assert!(
            text.contains("nexora:material/marble: no material entry"),
            "{text}"
        );
        assert_eq!(err.recovery(), Recovery::Reject);
    }

    #[test]
    fn a_material_entry_of_the_wrong_kind_is_a_gap() {
        let manifest = Manifest::new([
            entry("nexora:texture/basalt/albedo", ResourceKind::Texture),
            entry("nexora:material/basalt", ResourceKind::Texture),
        ])
        .unwrap();
        let gaps = manifest
            .gaps(&[material("nexora:material/basalt", &[])])
            .unwrap();
        assert_eq!(
            gaps,
            [Gap::WrongKind {
                id: id("nexora:material/basalt"),
                expected: ResourceKind::Material,
                found: ResourceKind::Texture,
            }],
            "no unlinked-map report on top: a texture has no dependencies to check"
        );
    }
}
