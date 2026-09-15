//! Material documents: the on-disk form of a [`SurfaceMaterial`].
//!
//! The brief §5 asks for a formal material definition *"in a format suited to
//! NEXORA's real architecture"*. `Block System.md` §51 already shows data-driven
//! content as JSON, so JSON it is, through [`crate::json`] — strict, ordered and
//! written by this project rather than pulled in.
//!
//! # What the format guarantees
//!
//! * **Round trip.** `write → read → write` is byte-identical. Object keys are
//!   sorted, floats always carry a decimal point, and the timestamp is written
//!   in a form that parses back exactly. Provenance that cannot survive being
//!   saved is provenance nobody can audit.
//! * **No silent defaults.** Every field the runtime depends on is required.
//!   A document missing `category` is an error, not a stone.
//! * **Unknown fields are refused.** A misspelt `rougness` that is ignored is a
//!   material that silently ships with the wrong surface.

use std::collections::BTreeMap;

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_foundation::version::{
    ContentGeneratorVersion, ContentPipelineVersion, MaterialSchemaVersion,
};

use crate::json::{self, Json};
use crate::material::{
    BlendMode, MaterialCategory, PbrParameters, PhysicalScale, Revision, SurfaceMaterial,
    MATERIAL_SCHEMA_VERSION,
};
use crate::provenance::{
    AssetStatus, Backend, GenerationTrace, License, Modification, Provenance, ProvenanceClass,
    ReleaseStatus, Timestamp,
};
use crate::texture::{MapRole, Resolution};

/// Fields a material document may carry at the top level.
const MATERIAL_FIELDS: [&str; 11] = [
    "schema",
    "id",
    "name",
    "category",
    "revision",
    "resolution",
    "physical_scale",
    "seamless",
    "blend",
    "pbr",
    "maps",
];

/// Render a material as a JSON document.
#[must_use]
pub fn to_json(material: &SurfaceMaterial) -> Json {
    let mut fields: BTreeMap<String, Json> = BTreeMap::new();
    fields.insert(
        "schema".to_owned(),
        Json::Integer(i64::from(material.schema().0)),
    );
    fields.insert("id".to_owned(), Json::text(material.id().to_string()));
    fields.insert("name".to_owned(), Json::text(material.name()));
    fields.insert(
        "category".to_owned(),
        Json::text(material.category().as_str()),
    );
    fields.insert(
        "revision".to_owned(),
        Json::Integer(i64::from(material.revision().0)),
    );
    fields.insert(
        "resolution".to_owned(),
        Json::object([
            (
                "width",
                Json::Integer(i64::from(material.resolution().width)),
            ),
            (
                "height",
                Json::Integer(i64::from(material.resolution().height)),
            ),
        ]),
    );
    fields.insert(
        "physical_scale".to_owned(),
        Json::object([(
            "metres_per_tile",
            Json::Float(material.physical_scale().metres_per_tile),
        )]),
    );
    fields.insert("seamless".to_owned(), Json::Bool(material.is_seamless()));
    fields.insert("blend".to_owned(), Json::text(material.blend().as_str()));
    fields.insert(
        "pbr".to_owned(),
        Json::object([
            ("metallic", Json::Float(material.pbr().metallic)),
            ("roughness", Json::Float(material.pbr().roughness)),
            (
                "normal_strength",
                Json::Float(material.pbr().normal_strength),
            ),
            (
                "height_strength",
                Json::Float(material.pbr().height_strength),
            ),
        ]),
    );
    fields.insert(
        "maps".to_owned(),
        Json::Array(
            material
                .wanted_maps()
                .iter()
                .map(|role| Json::text(role.as_str()))
                .collect(),
        ),
    );
    fields.insert(
        "provenance".to_owned(),
        provenance_to_json(material.provenance()),
    );
    Json::Object(fields)
}

/// Render a material as indented JSON text, with a trailing newline.
#[must_use]
pub fn to_text(material: &SurfaceMaterial) -> String {
    to_json(material).to_pretty()
}

/// Read a material from JSON text.
///
/// # Errors
///
/// Returns an error when the text is not valid JSON, the schema version is one
/// this build cannot read, a required field is absent, or any value is out of
/// range. Every failure names the field.
pub fn from_text(text: &str) -> Result<SurfaceMaterial> {
    from_json(&json::parse(text)?)
}

/// Read a material from a parsed document.
///
/// # Errors
///
/// See [`from_text`].
pub fn from_json(document: &Json) -> Result<SurfaceMaterial> {
    reject_unknown_fields(document, &MATERIAL_FIELDS, &["provenance"], "material")?;

    let schema = MaterialSchemaVersion(document.field("schema")?.as_u32()?);
    if schema.0 > MATERIAL_SCHEMA_VERSION.0 {
        return Err(unreadable("the document was written by a newer build")
            .with_context("schema", schema.to_string())
            .with_context("supported", MATERIAL_SCHEMA_VERSION.to_string()));
    }

    let id = Identifier::parse(document.field("id")?.as_text()?)?;
    let category = MaterialCategory::parse(document.field("category")?.as_text()?)?;
    let resolution = {
        let node = document.field("resolution")?;
        reject_unknown_fields(node, &["width", "height"], &[], "resolution")?;
        Resolution::new(
            node.field("width")?.as_u32()?,
            node.field("height")?.as_u32()?,
        )?
    };
    let provenance = provenance_from_json(document.field("provenance")?, schema)?;

    let mut builder = SurfaceMaterial::builder(id, category, resolution, provenance)
        .named(document.field("name")?.as_text()?)
        .revision(Revision(document.field("revision")?.as_u32()?))
        .seamless(document.field("seamless")?.as_bool()?)
        .blended(BlendMode::parse(document.field("blend")?.as_text()?)?);

    let scale = document.field("physical_scale")?;
    reject_unknown_fields(scale, &["metres_per_tile"], &[], "physical_scale")?;
    builder = builder.scaled(PhysicalScale::new(
        scale.field("metres_per_tile")?.as_f64()?,
    )?);

    let pbr = document.field("pbr")?;
    reject_unknown_fields(
        pbr,
        &[
            "metallic",
            "roughness",
            "normal_strength",
            "height_strength",
        ],
        &[],
        "pbr",
    )?;
    builder = builder.pbr(PbrParameters {
        metallic: pbr.field("metallic")?.as_f64()?,
        roughness: pbr.field("roughness")?.as_f64()?,
        normal_strength: pbr.field("normal_strength")?.as_f64()?,
        height_strength: pbr.field("height_strength")?.as_f64()?,
    });

    for entry in document.field("maps")?.as_array()? {
        builder = builder.wants(MapRole::parse(entry.as_text()?)?);
    }

    builder.build()
}

fn provenance_to_json(provenance: &Provenance) -> Json {
    let mut fields: BTreeMap<String, Json> = BTreeMap::new();
    fields.insert("class".to_owned(), Json::text(provenance.class.as_str()));
    fields.insert("status".to_owned(), Json::text(provenance.status.as_str()));
    fields.insert(
        "release".to_owned(),
        Json::text(provenance.release.as_str()),
    );
    fields.insert("author".to_owned(), Json::text(&provenance.author));
    fields.insert(
        "source_tool".to_owned(),
        Json::text(&provenance.source_tool),
    );
    fields.insert(
        "source_repository".to_owned(),
        optional_text(provenance.source_repository.as_deref()),
    );
    fields.insert(
        "modification".to_owned(),
        match &provenance.modification {
            Modification::Unmodified => Json::text("unmodified"),
            Modification::Modified => Json::text("modified"),
            Modification::DerivedFrom(source) => {
                Json::object([("derived_from", Json::text(source.to_string()))])
            }
        },
    );
    fields.insert(
        "license".to_owned(),
        provenance.license.as_ref().map_or(Json::Null, |license| {
            Json::object([
                ("name", Json::text(&license.name)),
                ("url", optional_text(license.url.as_deref())),
                (
                    "attribution_required",
                    Json::Bool(license.attribution_required),
                ),
                (
                    "redistribution_allowed",
                    Json::Bool(license.redistribution_allowed),
                ),
                (
                    "commercial_use_allowed",
                    Json::Bool(license.commercial_use_allowed),
                ),
            ])
        }),
    );
    fields.insert(
        "reviewer".to_owned(),
        optional_text(provenance.reviewer.as_deref()),
    );
    fields.insert(
        "reviewed_at".to_owned(),
        provenance
            .reviewed_at
            .map_or(Json::Null, |at| Json::text(at.to_rfc3339_utc())),
    );
    fields.insert(
        "notes".to_owned(),
        optional_text(provenance.notes.as_deref()),
    );
    fields.insert(
        "recorded_at".to_owned(),
        Json::text(provenance.recorded_at.to_rfc3339_utc()),
    );
    fields.insert(
        "generation".to_owned(),
        provenance
            .generation
            .as_ref()
            .map_or(Json::Null, generation_to_json),
    );
    // Written, never read back: a record whose hash was taken from the file it
    // is stored in could not detect the file being edited. Reading recomputes.
    fields.insert(
        "content_hash".to_owned(),
        Json::text(format!("{:#018x}", provenance.content_hash())),
    );
    Json::Object(fields)
}

fn generation_to_json(trace: &GenerationTrace) -> Json {
    let mut fields: BTreeMap<String, Json> = BTreeMap::new();
    fields.insert(
        "generator".to_owned(),
        Json::text(trace.generator.to_string()),
    );
    fields.insert(
        "generator_version".to_owned(),
        Json::Integer(i64::from(trace.generator_version.0)),
    );
    fields.insert("backend".to_owned(), Json::text(trace.backend.as_str()));
    fields.insert(
        "pipeline".to_owned(),
        trace
            .pipeline
            .as_ref()
            .map_or(Json::Null, |id| Json::text(id.to_string())),
    );
    fields.insert(
        "pipeline_version".to_owned(),
        trace
            .pipeline_version
            .map_or(Json::Null, |version| Json::Integer(i64::from(version.0))),
    );
    fields.insert(
        "preset".to_owned(),
        trace
            .preset
            .as_ref()
            .map_or(Json::Null, |id| Json::text(id.to_string())),
    );
    // A u64 seed does not fit a JSON integer that anything else can read
    // safely, so it goes out as hex text. Explicit beats nearly-right.
    fields.insert(
        "seed".to_owned(),
        Json::text(format!("{:#018x}", trace.seed)),
    );
    fields.insert("prompt".to_owned(), optional_text(trace.prompt.as_deref()));
    fields.insert(
        "parameters".to_owned(),
        Json::Object(
            trace
                .parameters
                .iter()
                .map(|(key, value)| (key.clone(), Json::text(value)))
                .collect(),
        ),
    );
    fields.insert(
        "inputs".to_owned(),
        Json::Array(
            trace
                .inputs
                .iter()
                .map(|id| Json::text(id.to_string()))
                .collect(),
        ),
    );
    Json::Object(fields)
}

fn provenance_from_json(node: &Json, schema: MaterialSchemaVersion) -> Result<Provenance> {
    reject_unknown_fields(
        node,
        &[
            "class",
            "status",
            "release",
            "author",
            "source_tool",
            "source_repository",
            "modification",
            "license",
            "reviewer",
            "reviewed_at",
            "notes",
            "recorded_at",
            "generation",
            "content_hash",
        ],
        &[],
        "provenance",
    )?;

    let provenance = Provenance {
        class: ProvenanceClass::parse(node.field("class")?.as_text()?)?,
        status: AssetStatus::parse(node.field("status")?.as_text()?)?,
        release: ReleaseStatus::parse(node.field("release")?.as_text()?)?,
        author: node.field("author")?.as_text()?.to_owned(),
        source_tool: node.field("source_tool")?.as_text()?.to_owned(),
        source_repository: text_or_null(node.field("source_repository")?)?,
        license: license_from_json(node.field("license")?)?,
        modification: modification_from_json(node.field("modification")?)?,
        reviewer: text_or_null(node.field("reviewer")?)?,
        reviewed_at: timestamp_or_null(node.field("reviewed_at")?)?,
        notes: text_or_null(node.field("notes")?)?,
        generation: generation_from_json(node.field("generation")?, schema)?,
        recorded_at: Timestamp::parse_rfc3339_utc(node.field("recorded_at")?.as_text()?)?,
    };
    provenance.validate()?;
    Ok(provenance)
}

fn generation_from_json(
    node: &Json,
    schema: MaterialSchemaVersion,
) -> Result<Option<GenerationTrace>> {
    if matches!(node, Json::Null) {
        return Ok(None);
    }
    reject_unknown_fields(
        node,
        &[
            "generator",
            "generator_version",
            "backend",
            "pipeline",
            "pipeline_version",
            "preset",
            "seed",
            "prompt",
            "parameters",
            "inputs",
        ],
        &[],
        "generation",
    )?;

    let mut parameters = BTreeMap::new();
    if let Json::Object(entries) = node.field("parameters")? {
        for (key, value) in entries {
            parameters.insert(key.clone(), value.as_text()?.to_owned());
        }
    } else {
        return Err(unreadable("generation parameters must be an object"));
    }

    let mut inputs = Vec::new();
    for entry in node.field("inputs")?.as_array()? {
        inputs.push(Identifier::parse(entry.as_text()?)?);
    }

    Ok(Some(GenerationTrace {
        generator: Identifier::parse(node.field("generator")?.as_text()?)?,
        generator_version: ContentGeneratorVersion(node.field("generator_version")?.as_u32()?),
        pipeline: identifier_or_null(node.field("pipeline")?)?,
        pipeline_version: match node.field("pipeline_version")? {
            Json::Null => None,
            other => Some(ContentPipelineVersion(other.as_u32()?)),
        },
        backend: match node.optional_field("backend")? {
            Some(value) => Backend::parse(value.as_text()?)?,
            // Absent only in a schema-1 document, and a schema-1 document was
            // necessarily written by the procedural generator, because it was
            // the only one that existed. Not a guess — the one answer the
            // format's own history allows.
            None if schema.0 < 2 => Backend::Procedural,
            None => {
                return Err(unreadable(
                    "a generation record must say which backend produced it",
                ))
            }
        },
        preset: identifier_or_null(node.field("preset")?)?,
        seed: parse_hex_u64(node.field("seed")?.as_text()?)?,
        prompt: text_or_null(node.field("prompt")?)?,
        parameters,
        inputs,
    }))
}

fn license_from_json(node: &Json) -> Result<Option<License>> {
    if matches!(node, Json::Null) {
        return Ok(None);
    }
    reject_unknown_fields(
        node,
        &[
            "name",
            "url",
            "attribution_required",
            "redistribution_allowed",
            "commercial_use_allowed",
        ],
        &[],
        "license",
    )?;
    Ok(Some(License {
        name: node.field("name")?.as_text()?.to_owned(),
        url: text_or_null(node.field("url")?)?,
        attribution_required: node.field("attribution_required")?.as_bool()?,
        redistribution_allowed: node.field("redistribution_allowed")?.as_bool()?,
        commercial_use_allowed: node.field("commercial_use_allowed")?.as_bool()?,
    }))
}

fn modification_from_json(node: &Json) -> Result<Modification> {
    match node {
        Json::Text(value) if value == "unmodified" => Ok(Modification::Unmodified),
        Json::Text(value) if value == "modified" => Ok(Modification::Modified),
        Json::Object(_) => {
            reject_unknown_fields(node, &["derived_from"], &[], "modification")?;
            Ok(Modification::DerivedFrom(Identifier::parse(
                node.field("derived_from")?.as_text()?,
            )?))
        }
        other => {
            Err(unreadable("modification status is not recognised")
                .with_context("found", other.kind()))
        }
    }
}

fn optional_text(value: Option<&str>) -> Json {
    value.map_or(Json::Null, Json::text)
}

fn text_or_null(node: &Json) -> Result<Option<String>> {
    match node {
        Json::Null => Ok(None),
        other => Ok(Some(other.as_text()?.to_owned())),
    }
}

fn identifier_or_null(node: &Json) -> Result<Option<Identifier>> {
    match node {
        Json::Null => Ok(None),
        other => Ok(Some(Identifier::parse(other.as_text()?)?)),
    }
}

fn timestamp_or_null(node: &Json) -> Result<Option<Timestamp>> {
    match node {
        Json::Null => Ok(None),
        other => Ok(Some(Timestamp::parse_rfc3339_utc(other.as_text()?)?)),
    }
}

fn parse_hex_u64(raw: &str) -> Result<u64> {
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

/// Refuse a document carrying a field this build does not know.
///
/// A misspelt field that is ignored is the failure mode this whole format is
/// trying to avoid: the material loads, looks nearly right, and the value the
/// author wrote is nowhere.
fn reject_unknown_fields(
    node: &Json,
    known: &[&str],
    also: &[&str],
    what: &'static str,
) -> Result<()> {
    let Json::Object(fields) = node else {
        return Err(unreadable("expected an object").with_context("section", what));
    };
    for key in fields.keys() {
        if !known.contains(&key.as_str()) && !also.contains(&key.as_str()) {
            return Err(
                unreadable("the document carries a field this build does not know")
                    .with_context("section", what)
                    .with_context("field", key.clone()),
            );
        }
    }
    Ok(())
}

fn unreadable(message: &'static str) -> Error {
    Error::new(Domain::Content, "material-document", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::ProvenanceClass;
    use crate::texture::MapRole;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn rich_material() -> SurfaceMaterial {
        let trace = GenerationTrace::new(
            id("nexora:generator/procedural"),
            ContentGeneratorVersion(3),
            0xDEAD_BEEF_CAFE_F00D,
        )
        .with_parameter("style", "plank")
        .with_parameter("wear", "0.35")
        .from_preset(id("nexora:preset/wood"))
        .through_pipeline(id("nexora:pipeline/pbr"), ContentPipelineVersion(2));

        let mut provenance = Provenance::generated("NEXORA", "nexora-texture-forge@0.0.1", trace);
        provenance.notes = Some("reference style: vertical planks".to_owned());
        provenance.reviewer = Some("operator".to_owned());
        provenance.reviewed_at = Some(Timestamp(1_757_000_000));
        provenance.recorded_at = Timestamp(1_757_000_123);

        SurfaceMaterial::builder(
            id("nexora:material/dark_oak_plank"),
            MaterialCategory::Wood,
            Resolution::square(64).unwrap(),
            provenance,
        )
        .named("Dark Oak Plank")
        .scaled(PhysicalScale::new(0.5).unwrap())
        .blended(BlendMode::Opaque)
        .pbr(PbrParameters {
            metallic: 0.0,
            roughness: 0.85,
            normal_strength: 1.25,
            height_strength: 0.4,
        })
        .wants(MapRole::Normal)
        .wants(MapRole::Roughness)
        .wants(MapRole::Height)
        .build()
        .expect("valid material")
    }

    #[test]
    fn a_material_survives_being_written_and_read_back() {
        let original = rich_material();
        let text = to_text(&original);
        let restored = from_text(&text).expect("the document must read back");

        assert_eq!(restored.id(), original.id());
        assert_eq!(restored.name(), original.name());
        assert_eq!(restored.category(), original.category());
        assert_eq!(restored.revision(), original.revision());
        assert_eq!(restored.resolution(), original.resolution());
        assert_eq!(restored.blend(), original.blend());
        assert_eq!(restored.wanted_maps(), original.wanted_maps());
        assert_eq!(restored.appearance_hash(), original.appearance_hash());
        assert_eq!(restored.provenance(), original.provenance());
        assert_eq!(restored, original);
    }

    #[test]
    fn writing_what_was_read_produces_identical_bytes() {
        let text = to_text(&rich_material());
        let again = to_text(&from_text(&text).unwrap());
        assert_eq!(text, again);

        // And a third pass, because a format that is stable once may not be.
        assert_eq!(again, to_text(&from_text(&again).unwrap()));
    }

    #[test]
    fn the_document_is_readable_by_a_person() {
        let text = to_text(&rich_material());
        assert!(
            text.contains("\"id\": \"nexora:material/dark_oak_plank\""),
            "{text}"
        );
        assert!(text.contains("\"category\": \"wood\""), "{text}");
        assert!(text.contains("\"roughness\": 0.85"), "{text}");
        // Whole numbers keep their decimal point so they read back as floats.
        assert!(text.contains("\"metallic\": 0.0"), "{text}");
        // The seed is hex text, not a number nothing can hold.
        assert!(text.contains("\"seed\": \"0xdeadbeefcafef00d\""), "{text}");
        // A date a reviewer can read, not epoch seconds.
        assert!(text.contains("\"recorded_at\": \"2025-09-04T"), "{text}");
    }

    #[test]
    fn a_misspelt_field_is_refused_rather_than_ignored() {
        let text = to_text(&rich_material()).replace("\"roughness\"", "\"rougness\"");
        let err = from_text(&text).expect_err("a typo must not be silently dropped");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("rougness"), "{err}");
        assert!(err.to_string().contains("pbr"), "{err}");
    }

    #[test]
    fn a_missing_required_field_names_itself() {
        let mut document = to_json(&rich_material());
        if let Json::Object(fields) = &mut document {
            fields.remove("category");
        }
        let err = from_json(&document).expect_err("category is not optional");
        assert!(err.to_string().contains("category"), "{err}");
    }

    #[test]
    fn a_newer_schema_is_refused_rather_than_read_optimistically() {
        let mut document = to_json(&rich_material());
        if let Json::Object(fields) = &mut document {
            fields.insert(
                "schema".to_owned(),
                Json::Integer(i64::from(MATERIAL_SCHEMA_VERSION.0) + 1),
            );
        }
        let err = from_json(&document).expect_err("a newer document must be refused");
        assert!(err.to_string().contains("newer build"), "{err}");
    }

    #[test]
    fn out_of_range_values_are_caught_at_read_time_not_at_use_time() {
        let text = to_text(&rich_material()).replace("\"roughness\": 0.85", "\"roughness\": 4.0");
        let err = from_text(&text).expect_err("roughness above one must be refused");
        assert!(err.to_string().contains("range"), "{err}");

        let text = to_text(&rich_material()).replace("\"width\": 64", "\"width\": 63");
        let err = from_text(&text).expect_err("a non-power-of-two must be refused");
        assert!(err.to_string().contains("power of two"), "{err}");
    }

    #[test]
    fn a_document_whose_provenance_breaks_a_rule_does_not_load() {
        // Procedural class with no trace: the exact contradiction the registry
        // rules exist to catch, and it must be caught on load rather than at
        // the point the material is used.
        let mut document = to_json(&rich_material());
        if let Json::Object(fields) = &mut document {
            let Some(Json::Object(provenance)) = fields.get_mut("provenance") else {
                panic!("the document must carry a provenance object");
            };
            provenance.insert("generation".to_owned(), Json::Null);
        }
        let err = from_json(&document).expect_err("provenance rules apply on load");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains("reproduces"), "{err}");
    }

    #[test]
    fn an_unknown_provenance_class_blocks_rather_than_defaulting() {
        // "unknown origin → BLOCK" from the content policy, at the one place a
        // document can introduce an origin nobody has heard of.
        let text =
            to_text(&rich_material()).replace("\"procedural_derivative\"", "\"ai_generated\"");
        let err = from_text(&text).expect_err("an unknown class must not load");
        assert!(err.to_string().contains("provenance class"), "{err}");
        assert!(err.to_string().contains("ai_generated"), "{err}");
    }

    #[test]
    fn a_license_round_trips_with_every_permission() {
        let mut material = rich_material();
        let provenance = material.provenance_mut();
        provenance.class = ProvenanceClass::LicensedThirdParty;
        provenance.license = Some(License {
            name: "CC-BY-4.0".to_owned(),
            url: Some("https://example.invalid/by".to_owned()),
            attribution_required: true,
            redistribution_allowed: true,
            commercial_use_allowed: false,
        });

        let restored = from_text(&to_text(&material)).expect("reads back");
        let license = restored.provenance().license.as_ref().unwrap();
        assert_eq!(license.name, "CC-BY-4.0");
        assert!(license.attribution_required);
        assert!(!license.commercial_use_allowed);
        assert_eq!(
            restored.provenance().class,
            ProvenanceClass::LicensedThirdParty
        );
    }

    #[test]
    fn a_derived_modification_names_its_source() {
        let mut material = rich_material();
        material.provenance_mut().modification =
            Modification::DerivedFrom(id("nexora:material/oak_plank"));

        let text = to_text(&material);
        assert!(text.contains("\"derived_from\""), "{text}");
        let restored = from_text(&text).expect("reads back");
        assert_eq!(
            restored.provenance().modification,
            Modification::DerivedFrom(id("nexora:material/oak_plank"))
        );
    }

    #[test]
    fn a_malformed_seed_is_refused() {
        // `0x1` is a legitimate hand-written seed and must be accepted; the
        // writer always emits sixteen digits, the reader does not demand them.
        let text = to_text(&rich_material()).replace("\"0xdeadbeefcafef00d\"", "\"0x1\"");
        assert_eq!(
            from_text(&text)
                .expect("a short seed reads back")
                .provenance()
                .generation
                .as_ref()
                .unwrap()
                .seed,
            1
        );

        for bad in [
            "\"12345\"",
            "\"0xzz\"",
            "\"0x\"",
            "\"0x00000000000000001\"",
            "42",
        ] {
            let text = to_text(&rich_material()).replace("\"0xdeadbeefcafef00d\"", bad);
            assert!(from_text(&text).is_err(), "seed {bad} must be refused");
        }
    }

    #[test]
    fn an_authored_definition_with_no_generation_still_round_trips() {
        let material = SurfaceMaterial::builder(
            id("nexora:material/plain"),
            MaterialCategory::Custom,
            Resolution::square(16).unwrap(),
            Provenance::authored("operator", "hand-written"),
        )
        .build()
        .unwrap();

        let text = to_text(&material);
        assert!(text.contains("\"generation\": null"), "{text}");
        let restored = from_text(&text).expect("reads back");
        assert!(restored.provenance().generation.is_none());
        assert_eq!(restored, material);
    }
}
