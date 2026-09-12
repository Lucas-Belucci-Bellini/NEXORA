//! Batch manifests: many materials without repeating yourself.
//!
//! The brief §12 asks for `generate biome temperate_forest` producing a whole
//! set at once, and says not to make it rigid — to prepare for manifests
//! instead. This is that file.
//!
//! ```json
//! {
//!   "schema": 1,
//!   "name": "temperate forest",
//!   "namespace": "nexora",
//!   "prefix": "temperate_forest",
//!   "author": "operator",
//!   "defaults": { "resolution": { "width": 64, "height": 64 },
//!                 "maps": ["normal", "roughness", "height"] },
//!   "materials": [ { "name": "oak_bark", "category": "wood" },
//!                  { "name": "forest_soil", "category": "soil" } ]
//! }
//! ```
//!
//! # Grouping is namespacing, because it already was
//!
//! §12 wants terrain, environment and structure sets. Rather than invent a
//! `group` field that only this tool understands, a manifest's `prefix` becomes
//! a path segment: `nexora:material/temperate_forest/oak_bark`. Identifiers
//! already nest, `layout` already turns nesting into directories, and the
//! registry already sorts by identifier — so the grouping shows up everywhere
//! for free, and nothing had to learn a new concept.
//!
//! # Every field is inherited or explicit, and nothing is guessed
//!
//! An entry takes the manifest's defaults and overrides what it names. A field
//! nobody named falls back to the same value the single-material path uses, so
//! a manifest and a hand-written definition describe the same material when
//! they say the same things.

use nexora_asset::json::{self, Json};
use nexora_asset::material::{
    BlendMode, MaterialCategory, PbrParameters, PhysicalScale, SurfaceMaterial,
};
use nexora_asset::provenance::Provenance;
use nexora_asset::texture::{MapRole, Resolution};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::{Identifier, Namespace};

/// Manifest schema version this build reads.
pub const MANIFEST_SCHEMA: u32 = 1;

/// Most materials one manifest may declare.
///
/// A bound rather than a limit anybody will meet: the operator's target is ten
/// thousand materials across many manifests, and a single file naming a
/// million is a mistake rather than a plan.
pub const MAX_MATERIALS: usize = 4096;

/// Fields a manifest may carry.
const MANIFEST_FIELDS: [&str; 8] = [
    "schema",
    "name",
    "namespace",
    "prefix",
    "author",
    "seed",
    "defaults",
    "materials",
];

/// Fields a `defaults` block or a material entry may carry.
///
/// Deliberately the same list, minus `name` and `category`, which only an
/// entry has: a default that could not be overridden, or an override that
/// could not be defaulted, would be a difference to remember.
const SHARED_FIELDS: [&str; 6] = [
    "resolution",
    "physical_scale",
    "seamless",
    "blend",
    "pbr",
    "maps",
];

/// What a manifest says, before any of it has been realised.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    /// Human-readable name for the set.
    pub name: String,
    /// The seed every material in the set is generated from.
    pub seed: u64,
    /// Who authored the set.
    pub author: String,
    /// The materials it declares, in the order it declares them.
    pub materials: Vec<SurfaceMaterial>,
}

impl Manifest {
    /// How many materials the manifest declares.
    #[must_use]
    pub fn len(&self) -> usize {
        self.materials.len()
    }

    /// Whether it declares none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}

/// The values an entry inherits when it does not say otherwise.
#[derive(Debug, Clone, PartialEq)]
struct Defaults {
    resolution: Resolution,
    scale: PhysicalScale,
    seamless: bool,
    blend: BlendMode,
    pbr: PbrParameters,
    maps: Vec<MapRole>,
}

impl Defaults {
    /// The same values a hand-written definition falls back to.
    fn baseline() -> Result<Self> {
        Ok(Self {
            resolution: Resolution::square(64)?,
            scale: PhysicalScale::PER_BLOCK,
            seamless: true,
            blend: BlendMode::Opaque,
            pbr: PbrParameters::DEFAULT,
            maps: Vec::new(),
        })
    }

    /// Read a partial override, keeping whatever it does not mention.
    ///
    /// `also` names the fields this node may carry beyond the inherited ones —
    /// an entry has a `name` and a `category`, a `defaults` block has neither.
    fn overridden(&self, node: &Json, also: &[&str], what: &'static str) -> Result<Self> {
        reject_unknown(node, &SHARED_FIELDS, also, what)?;
        let mut next = self.clone();

        if let Some(value) = node.optional_field("resolution")? {
            reject_unknown(value, &["width", "height"], &[], "resolution")?;
            next.resolution = Resolution::new(
                value.field("width")?.as_u32()?,
                value.field("height")?.as_u32()?,
            )?;
        }
        if let Some(value) = node.optional_field("physical_scale")? {
            reject_unknown(value, &["metres_per_tile"], &[], "physical_scale")?;
            next.scale = PhysicalScale::new(value.field("metres_per_tile")?.as_f64()?)?;
        }
        if let Some(value) = node.optional_field("seamless")? {
            next.seamless = value.as_bool()?;
        }
        if let Some(value) = node.optional_field("blend")? {
            next.blend = BlendMode::parse(value.as_text()?)?;
        }
        if let Some(value) = node.optional_field("pbr")? {
            reject_unknown(
                value,
                &[
                    "metallic",
                    "roughness",
                    "normal_strength",
                    "height_strength",
                ],
                &[],
                "pbr",
            )?;
            // Partial: a manifest that only cares about roughness says only
            // roughness, and the rest stays where it was.
            if let Some(number) = value.optional_field("metallic")? {
                next.pbr.metallic = number.as_f64()?;
            }
            if let Some(number) = value.optional_field("roughness")? {
                next.pbr.roughness = number.as_f64()?;
            }
            if let Some(number) = value.optional_field("normal_strength")? {
                next.pbr.normal_strength = number.as_f64()?;
            }
            if let Some(number) = value.optional_field("height_strength")? {
                next.pbr.height_strength = number.as_f64()?;
            }
            next.pbr = next.pbr.validate()?;
        }
        if let Some(value) = node.optional_field("maps")? {
            let mut roles = Vec::new();
            for entry in value.as_array()? {
                roles.push(MapRole::parse(entry.as_text()?)?);
            }
            next.maps = roles;
        }
        Ok(next)
    }
}

/// Read a manifest from JSON text.
///
/// # Errors
///
/// Returns an error when the text is not valid JSON, the schema is newer than
/// this build, a required field is absent, an unknown field is present, a
/// material's name does not form a valid identifier, two entries claim the
/// same identifier, or the manifest declares more than [`MAX_MATERIALS`].
pub fn from_text(text: &str) -> Result<Manifest> {
    from_json(&json::parse(text)?)
}

/// Read a manifest from a parsed document.
///
/// # Errors
///
/// See [`from_text`].
pub fn from_json(document: &Json) -> Result<Manifest> {
    reject_unknown(document, &MANIFEST_FIELDS, &[], "manifest")?;

    let schema = document.field("schema")?.as_u32()?;
    if schema > MANIFEST_SCHEMA {
        return Err(unreadable("the manifest was written by a newer build")
            .with_context("schema", schema.to_string())
            .with_context("supported", MANIFEST_SCHEMA.to_string()));
    }

    let name = document.field("name")?.as_text()?.to_owned();
    let namespace = Namespace::parse(
        document
            .optional_field("namespace")?
            .map_or(Ok("nexora"), Json::as_text)?,
    )?;
    let prefix = document
        .optional_field("prefix")?
        .map(Json::as_text)
        .transpose()?
        .unwrap_or("");
    let author = document
        .optional_field("author")?
        .map_or(Ok("NEXORA"), Json::as_text)?
        .to_owned();
    let seed = match document.optional_field("seed")? {
        Some(value) => parse_seed(value.as_text()?)?,
        None => 0,
    };

    let defaults = match document.optional_field("defaults")? {
        Some(node) => Defaults::baseline()?.overridden(node, &[], "defaults")?,
        None => Defaults::baseline()?,
    };

    let entries = document.field("materials")?.as_array()?;
    if entries.len() > MAX_MATERIALS {
        return Err(unreadable("the manifest declares too many materials")
            .with_context("declared", entries.len().to_string())
            .with_context("limit", MAX_MATERIALS.to_string()));
    }

    let mut materials: Vec<SurfaceMaterial> = Vec::with_capacity(entries.len());
    for entry in entries {
        let settings = defaults.overridden(entry, &["name", "category", "notes"], "material")?;
        let short = entry.field("name")?.as_text()?;
        let category = MaterialCategory::parse(entry.field("category")?.as_text()?)?;

        let path = if prefix.is_empty() {
            format!("material/{short}")
        } else {
            format!("material/{prefix}/{short}")
        };
        let id = Identifier::new(namespace.clone(), &path)?;
        if materials.iter().any(|existing| existing.id() == &id) {
            // Two entries under one id is two materials that cannot both
            // exist. Refusing here is refusing before anything is written.
            return Err(unreadable("two entries declare the same material")
                .with_context("material", id.to_string()));
        }

        let mut provenance = Provenance::authored(&author, format!("manifest:{name}"));
        if let Some(notes) = entry.optional_field("notes")? {
            provenance.notes = Some(notes.as_text()?.to_owned());
        }

        let mut builder = SurfaceMaterial::builder(id, category, settings.resolution, provenance)
            .scaled(settings.scale)
            .seamless(settings.seamless)
            .blended(settings.blend)
            .pbr(settings.pbr);
        for role in settings.maps {
            builder = builder.wants(role);
        }
        materials.push(builder.build()?);
    }

    Ok(Manifest {
        name,
        seed,
        author,
        materials,
    })
}

fn parse_seed(raw: &str) -> Result<u64> {
    let digits = raw.strip_prefix("0x").ok_or_else(|| {
        unreadable("a seed must be written as `0x`-prefixed hex")
            .with_context("value", raw.to_owned())
    })?;
    if digits.is_empty() || digits.len() > 16 {
        return Err(unreadable("a seed must be one to sixteen hex digits")
            .with_context("value", raw.to_owned()));
    }
    u64::from_str_radix(digits, 16)
        .map_err(|_| unreadable("a seed must be hexadecimal").with_context("value", raw.to_owned()))
}

fn reject_unknown(node: &Json, known: &[&str], also: &[&str], what: &'static str) -> Result<()> {
    let Json::Object(fields) = node else {
        return Err(unreadable("expected an object").with_context("section", what));
    };
    for key in fields.keys() {
        if !known.contains(&key.as_str()) && !also.contains(&key.as_str()) {
            return Err(
                unreadable("the manifest carries a field this build does not know")
                    .with_context("section", what)
                    .with_context("field", key.clone()),
            );
        }
    }
    Ok(())
}

fn unreadable(message: &'static str) -> Error {
    Error::new(Domain::Content, "manifest", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_asset::document;

    /// The smallest manifest that says anything.
    const MINIMAL: &str = r#"{
        "schema": 1,
        "name": "temperate forest",
        "materials": [ { "name": "oak_bark", "category": "wood" } ]
    }"#;

    fn read(text: &str) -> Manifest {
        from_text(text).expect("the manifest must read")
    }

    #[test]
    fn a_minimal_manifest_reads() {
        let manifest = read(MINIMAL);
        assert_eq!(manifest.name, "temperate forest");
        assert_eq!(manifest.len(), 1);
        assert!(!manifest.is_empty());
        assert_eq!(
            manifest.materials[0].id().to_string(),
            "nexora:material/oak_bark"
        );
        assert_eq!(manifest.materials[0].category(), MaterialCategory::Wood);
    }

    #[test]
    fn a_manifest_declaring_nothing_is_empty_rather_than_an_error() {
        // An empty set is a set. A manifest being written up, or one whose
        // entries were commented out, should read and produce nothing rather
        // than fail and say nothing about why.
        let manifest = read(r#"{ "schema": 1, "name": "empty", "materials": [] }"#);
        assert!(manifest.is_empty());
        assert_eq!(manifest.len(), 0);
    }

    #[test]
    fn the_prefix_becomes_a_path_segment() {
        let manifest = read(
            r#"{
                "schema": 1,
                "name": "temperate forest",
                "prefix": "temperate_forest",
                "materials": [ { "name": "oak_bark", "category": "wood" },
                               { "name": "forest_soil", "category": "soil" } ]
            }"#,
        );
        assert_eq!(
            manifest.materials[0].id().to_string(),
            "nexora:material/temperate_forest/oak_bark"
        );
        assert_eq!(
            manifest.materials[1].id().to_string(),
            "nexora:material/temperate_forest/forest_soil"
        );
    }

    #[test]
    fn the_namespace_defaults_to_nexora_and_can_be_said() {
        assert_eq!(
            read(MINIMAL).materials[0].id().namespace().as_str(),
            "nexora"
        );
        let manifest = read(
            r#"{
                "schema": 1,
                "name": "a mod's set",
                "namespace": "example_mod",
                "materials": [ { "name": "ore", "category": "mineral" } ]
            }"#,
        );
        assert_eq!(
            manifest.materials[0].id().to_string(),
            "example_mod:material/ore"
        );
    }

    #[test]
    fn entries_inherit_the_defaults() {
        let manifest = read(
            r#"{
                "schema": 1,
                "name": "set",
                "defaults": {
                    "resolution": { "width": 128, "height": 128 },
                    "blend": "cutout",
                    "seamless": false,
                    "physical_scale": { "metres_per_tile": 2.0 },
                    "maps": ["normal", "height"]
                },
                "materials": [ { "name": "leaves", "category": "vegetation" },
                               { "name": "bark", "category": "wood" } ]
            }"#,
        );
        for material in &manifest.materials {
            assert_eq!(material.resolution().width, 128);
            assert_eq!(material.blend(), BlendMode::Cutout);
            assert!(!material.is_seamless());
            assert!((material.physical_scale().metres_per_tile - 2.0).abs() < f64::EPSILON);
            assert_eq!(
                material.wanted_maps(),
                &[MapRole::Normal, MapRole::Height][..]
            );
        }
    }

    #[test]
    fn an_entry_overrides_only_what_it_names() {
        let manifest = read(
            r#"{
                "schema": 1,
                "name": "set",
                "defaults": { "resolution": { "width": 128, "height": 128 },
                              "blend": "cutout" },
                "materials": [
                    { "name": "leaves", "category": "vegetation" },
                    { "name": "pane", "category": "glass",
                      "resolution": { "width": 32, "height": 32 } }
                ]
            }"#,
        );
        // The one that said nothing keeps both defaults.
        assert_eq!(manifest.materials[0].resolution().width, 128);
        assert_eq!(manifest.materials[0].blend(), BlendMode::Cutout);
        // The one that named a resolution keeps the blend it did not name.
        assert_eq!(manifest.materials[1].resolution().width, 32);
        assert_eq!(manifest.materials[1].blend(), BlendMode::Cutout);
    }

    #[test]
    fn a_partial_pbr_override_keeps_the_rest() {
        // The interesting half of inheritance: `pbr` is a block, and naming
        // one number inside it must not silently reset the other three.
        let manifest = read(
            r#"{
                "schema": 1,
                "name": "set",
                "defaults": { "pbr": { "metallic": 1.0, "roughness": 0.2,
                                       "normal_strength": 0.5, "height_strength": 0.5 } },
                "materials": [ { "name": "brushed", "category": "metal",
                                 "pbr": { "roughness": 0.8 } } ]
            }"#,
        );
        let pbr = manifest.materials[0].pbr();
        assert!(
            (pbr.roughness - 0.8).abs() < f64::EPSILON,
            "the named one moved"
        );
        assert!(
            (pbr.metallic - 1.0).abs() < f64::EPSILON,
            "the others stayed"
        );
        assert!((pbr.normal_strength - 0.5).abs() < f64::EPSILON);
        assert!((pbr.height_strength - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn an_out_of_range_pbr_value_is_refused_where_it_is_written() {
        // Not at generation time, three files later.
        let error = from_text(
            r#"{
                "schema": 1,
                "name": "set",
                "materials": [ { "name": "x", "category": "metal",
                                 "pbr": { "roughness": 4.0 } } ]
            }"#,
        )
        .expect_err("a roughness of 4 is not a roughness");
        assert_eq!(error.domain(), Domain::Content);
    }

    #[test]
    fn a_manifest_entry_and_a_hand_written_definition_describe_the_same_material() {
        // The module's promise: say the same things, get the same material.
        // `appearance_hash` is exactly the set of fields generation reads, so
        // equal hashes mean the two paths produce identical textures.
        let manifest = read(
            r#"{
                "schema": 1,
                "name": "one",
                "materials": [ { "name": "stone_rough", "category": "stone",
                                 "resolution": { "width": 32, "height": 32 },
                                 "maps": ["normal", "roughness"] } ]
            }"#,
        );
        let by_hand = document::from_text(
            r#"{
                "schema": 1,
                "id": "nexora:material/stone_rough",
                "name": "stone rough",
                "category": "stone",
                "revision": 1,
                "resolution": { "width": 32, "height": 32 },
                "physical_scale": { "metres_per_tile": 1.0 },
                "seamless": true,
                "blend": "opaque",
                "pbr": { "metallic": 0.0, "roughness": 1.0,
                         "normal_strength": 1.0, "height_strength": 1.0 },
                "maps": ["normal", "roughness"],
                "provenance": { "class": "original", "status": "nexora_original",
                                "release": "draft", "author": "operator",
                                "source_tool": "test", "source_repository": null,
                                "license": null, "modification": "unmodified",
                                "reviewer": null, "reviewed_at": null, "notes": null,
                                "generation": null,
                                "recorded_at": "2026-01-01T00:00:00Z" }
            }"#,
        )
        .expect("the hand-written definition must read");

        assert_eq!(
            manifest.materials[0].appearance_hash(),
            by_hand.appearance_hash(),
            "a manifest that says what a definition says must mean it"
        );
    }

    #[test]
    fn two_entries_claiming_one_id_are_refused_before_anything_is_written() {
        let error = from_text(
            r#"{
                "schema": 1,
                "name": "set",
                "materials": [ { "name": "oak", "category": "wood" },
                               { "name": "oak", "category": "stone" } ]
            }"#,
        )
        .expect_err("one id cannot name two materials");
        assert!(
            error.to_string().contains("same material"),
            "the message must say what collided: {error}"
        );
    }

    #[test]
    fn the_same_name_under_two_prefixes_is_two_materials() {
        // The other half of the rule above: grouping is what makes `soil`
        // usable twice, and it must actually work.
        let forest = read(
            r#"{ "schema": 1, "name": "f", "prefix": "forest",
                 "materials": [ { "name": "soil", "category": "soil" } ] }"#,
        );
        let desert = read(
            r#"{ "schema": 1, "name": "d", "prefix": "desert",
                 "materials": [ { "name": "soil", "category": "soil" } ] }"#,
        );
        assert_ne!(forest.materials[0].id(), desert.materials[0].id());
    }

    #[test]
    fn a_newer_schema_is_refused_rather_than_guessed_at() {
        let error = from_text(r#"{ "schema": 99, "name": "x", "materials": [] }"#)
            .expect_err("a build cannot read a format it does not know");
        assert!(error.to_string().contains("newer build"), "{error}");
    }

    #[test]
    fn an_unknown_field_is_refused_at_every_level() {
        for text in [
            r#"{ "schema": 1, "name": "x", "materials": [], "colour": "red" }"#,
            r#"{ "schema": 1, "name": "x", "materials": [], "defaults": { "colour": "red" } }"#,
            r#"{ "schema": 1, "name": "x",
                 "materials": [ { "name": "a", "category": "wood", "colour": "red" } ] }"#,
        ] {
            let error = from_text(text).expect_err("a typo must not be ignored");
            assert!(
                error.to_string().contains("does not know"),
                "the message must name the problem: {error}"
            );
        }
    }

    #[test]
    fn a_name_that_is_not_an_identifier_is_refused() {
        let error = from_text(
            r#"{ "schema": 1, "name": "x",
                 "materials": [ { "name": "Oak Bark!", "category": "wood" } ] }"#,
        )
        .expect_err("an identifier has rules");
        assert_eq!(error.domain(), Domain::Content);
    }

    #[test]
    fn more_materials_than_the_bound_are_refused() {
        let entries: Vec<String> = (0..=MAX_MATERIALS)
            .map(|index| format!(r#"{{ "name": "m{index}", "category": "stone" }}"#))
            .collect();
        let error = from_text(&format!(
            r#"{{ "schema": 1, "name": "too many", "materials": [{}] }}"#,
            entries.join(",")
        ))
        .expect_err("the bound must hold");
        assert!(error.to_string().contains("too many materials"), "{error}");
    }

    #[test]
    fn the_bound_itself_is_allowed() {
        // A limit that rejects the number it names is off by one.
        let entries: Vec<String> = (0..MAX_MATERIALS)
            .map(|index| format!(r#"{{ "name": "m{index}", "category": "stone" }}"#))
            .collect();
        let manifest = read(&format!(
            r#"{{ "schema": 1, "name": "exactly", "materials": [{}] }}"#,
            entries.join(",")
        ));
        assert_eq!(manifest.len(), MAX_MATERIALS);
    }

    #[test]
    fn the_seed_is_read_as_hex_and_defaults_to_zero() {
        assert_eq!(read(MINIMAL).seed, 0);
        let manifest = read(
            r#"{ "schema": 1, "name": "x", "seed": "0xdeadbeef",
                 "materials": [] }"#,
        );
        assert_eq!(manifest.seed, 0xdead_beef);
    }

    #[test]
    fn a_seed_that_is_not_hex_is_refused() {
        for raw in ["1234", "0x", "0xzz", "0x00000000000000000"] {
            let error = from_text(&format!(
                r#"{{ "schema": 1, "name": "x", "seed": "{raw}", "materials": [] }}"#
            ))
            .expect_err("a decimal seed here would read as a different number");
            assert_eq!(error.domain(), Domain::Content, "for `{raw}`");
        }
    }

    #[test]
    fn the_author_reaches_every_material_and_defaults_to_nexora() {
        assert_eq!(read(MINIMAL).materials[0].provenance().author, "NEXORA");
        let manifest = read(
            r#"{ "schema": 1, "name": "set", "author": "operator",
                 "materials": [ { "name": "a", "category": "wood" },
                                { "name": "b", "category": "stone" } ] }"#,
        );
        assert_eq!(manifest.author, "operator");
        for material in &manifest.materials {
            assert_eq!(material.provenance().author, "operator");
            assert_eq!(material.provenance().source_tool, "manifest:set");
        }
    }

    #[test]
    fn a_note_on_an_entry_lands_on_its_record() {
        let manifest = read(
            r#"{ "schema": 1, "name": "set",
                 "materials": [ { "name": "a", "category": "wood",
                                  "notes": "studied a photograph of oak bark" } ] }"#,
        );
        assert_eq!(
            manifest.materials[0].provenance().notes.as_deref(),
            Some("studied a photograph of oak bark")
        );
    }

    #[test]
    fn a_manifest_that_is_not_an_object_says_so() {
        let error = from_text("[1, 2, 3]").expect_err("a list is not a manifest");
        assert!(error.to_string().contains("expected an object"), "{error}");
    }
}
