//! Recipes as data, and the book that finds them.
//!
//! `recipe.rs` has always said its category defaults are *"starting points a
//! material document overrides"*. This module is the override. A recipe
//! document is a [`Recipe`] written down, and a material names one by
//! identifier:
//!
//! ```json
//! {
//!   "schema": 1,
//!   "id": "nexora:recipe/stone/basalt",
//!   "category": "stone",
//!   "ramp": ["#1c1c20", "#2b2b30", "#3b3a3f", "#4f4d52"],
//!   "levels": 4,
//!   "contrast": 2.2,
//!   "relief": 0.5,
//!   "mottle": { "cells_x": 4, "cells_y": 4, "octaves": 4, "strength": 1.0, "cracks": 0.1 },
//!   "strips": null,
//!   "courses": null,
//!   "speckle": { "cells": 16, "density": 0.12, "contrast": 0.2 }
//! }
//! ```
//!
//! That is what turns sixteen stones into sixteen stones rather than one stone
//! with sixteen seeds (`DEBT-0044`): a new family member is a new file of
//! numbers, not new code, which is the goal `NEXORA ART DIRECTION AND
//! PROCEDURAL VARIATION.md` sets.
//!
//! # Where a recipe lives
//!
//! Derived from its identifier and never accepted as a path, the rule
//! `layout.rs` applies to materials:
//! `nexora:recipe/stone/basalt` → `<root>/nexora/stone/basalt.json`.
//!
//! # Why a recipe carries a fingerprint
//!
//! A material records the recipe by *identifier*, so editing `basalt.json`
//! changes nothing the material's appearance hash can see. Without more, the
//! forge would call the old pixels `unchanged` forever. So the book hashes
//! the recipe's canonical text, the generator records that hash in the
//! material's trace, and the forge compares it before deciding nothing moved.
//!
//! # Strict, like every other content document
//!
//! Every field is required, unknown fields are refused, and every number is
//! range-checked at read time. A recipe is content, so it is also untrusted:
//! the bounds below are what keep a hostile one from asking the renderer for a
//! billion octaves.

use std::path::{Path, PathBuf};

use nexora_asset::json::{self, Json};
use nexora_asset::material::{MaterialCategory, SurfaceMaterial, RECIPE_PATH_PREFIX};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::hashing::Fnv1a64;
use nexora_foundation::ident::Identifier;

use crate::color::{Ramp, Rgba};
use crate::recipe::{Courses, Mottle, Recipe, Speckle, Strips};

/// The newest recipe schema this build reads.
pub const RECIPE_SCHEMA: u32 = 1;

/// The trace parameter a recipe's fingerprint is recorded under.
pub const FINGERPRINT_PARAMETER: &str = "recipe_fingerprint";

/// Most stops a ramp may have. A first-generation palette is four to eight.
pub const MAX_RAMP_STOPS: usize = 32;

/// Most lattice cells along one axis. Far past a 16-texel tile.
pub const MAX_CELLS: u32 = 1024;

/// Most octaves a field may sum.
pub const MAX_OCTAVES: u32 = 8;

const RECIPE_FIELDS: [&str; 11] = [
    "schema", "id", "category", "ramp", "levels", "contrast", "relief", "mottle", "strips",
    "courses", "speckle",
];

/// A recipe, resolved for one material.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    /// What to render.
    pub recipe: Recipe,
    /// What to record as the preset in the generation trace.
    pub preset: Identifier,
    /// The recipe document's fingerprint, when it came from one.
    ///
    /// `None` for a category default, whose numbers are code and therefore
    /// versioned by the generator version instead.
    pub fingerprint: Option<u64>,
}

/// A recipe document, read and checked.
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeDocument {
    /// Its identifier.
    pub id: Identifier,
    /// The family it draws.
    pub category: MaterialCategory,
    /// The numbers.
    pub recipe: Recipe,
}

impl RecipeDocument {
    /// A hash of the canonical form, stable across whitespace and key order.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = Fnv1a64::new();
        hasher.write_str(&to_json(self).to_compact());
        hasher.finish()
    }
}

/// Finds recipes under a root directory.
#[derive(Debug, Clone)]
pub struct RecipeBook {
    root: Option<PathBuf>,
}

impl RecipeBook {
    /// A book reading recipe documents under a directory.
    #[must_use]
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
        }
    }

    /// A book with no documents: only category defaults resolve.
    #[must_use]
    pub const fn empty() -> Self {
        Self { root: None }
    }

    /// The directory recipes are read from, when there is one.
    #[must_use]
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// The recipe a material is drawn with.
    ///
    /// # Errors
    ///
    /// Returns an error when the material names a recipe that is not in the
    /// book, does not parse, or draws a different category. The last one is
    /// refused rather than tolerated: a stone material pointed at a wood
    /// recipe is a typo far more often than it is a choice.
    pub fn resolve(&self, material: &SurfaceMaterial) -> Result<Resolved> {
        let Some(id) = material.recipe() else {
            return Ok(Resolved {
                recipe: Recipe::for_category(material.category())?,
                preset: Identifier::nexora(&format!("preset/{}", material.category().as_str()))?,
                fingerprint: None,
            });
        };
        let document = self.read(id)?;
        if document.category != material.category() {
            return Err(unusable("the recipe draws a different category")
                .with_context("material", material.id().to_string())
                .with_context("material_category", material.category().as_str())
                .with_context("recipe", id.to_string())
                .with_context("recipe_category", document.category.as_str()));
        }
        let fingerprint = document.fingerprint();
        Ok(Resolved {
            recipe: document.recipe,
            preset: document.id,
            fingerprint: Some(fingerprint),
        })
    }

    /// Read one recipe document by identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the book has no root, the file is absent or does
    /// not parse, or the identifier inside it is not the one asked for.
    pub fn read(&self, id: &Identifier) -> Result<RecipeDocument> {
        let Some(root) = &self.root else {
            return Err(
                unusable("a recipe was named and no recipe directory was given")
                    .with_context("recipe", id.to_string()),
            );
        };
        let path = recipe_file(root, id)?;
        let text = std::fs::read_to_string(&path).map_err(|cause| {
            unusable("the recipe could not be read")
                .with_context("recipe", id.to_string())
                .with_context("path", path.display().to_string())
                .with_context("cause", cause.to_string())
        })?;
        let document = from_text(&text)?;
        if &document.id != id {
            // The file name is a consequence of the identifier. A file whose
            // contents claim another is a copy someone forgot to edit.
            return Err(unusable("the recipe file declares a different identifier")
                .with_context("expected", id.to_string())
                .with_context("found", document.id.to_string())
                .with_context("path", path.display().to_string()));
        }
        Ok(document)
    }
}

/// Where a recipe's document lives, under a root.
///
/// # Errors
///
/// Returns an error when the identifier has no path beyond its prefix, or a
/// segment would mean something to the filesystem.
pub fn recipe_file(root: &Path, id: &Identifier) -> Result<PathBuf> {
    let stem = id
        .path()
        .strip_prefix(RECIPE_PATH_PREFIX)
        .unwrap_or_else(|| id.path());
    if stem.is_empty() {
        return Err(
            unusable("a recipe identifier needs a path beyond its prefix")
                .with_context("recipe", id.to_string()),
        );
    }
    let mut path = root.join(id.namespace().as_str());
    for segment in stem.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Err(
                unusable("a recipe path segment means something to the filesystem")
                    .with_context("recipe", id.to_string()),
            );
        }
        path.push(segment);
    }
    path.set_extension("json");
    Ok(path)
}

/// Read a recipe document from text.
///
/// # Errors
///
/// Returns an error naming the field when anything is missing, unknown,
/// malformed or out of range.
pub fn from_text(text: &str) -> Result<RecipeDocument> {
    from_json(&json::parse(text)?)
}

/// Read a recipe document from a parsed value.
///
/// # Errors
///
/// See [`from_text`].
pub fn from_json(document: &Json) -> Result<RecipeDocument> {
    reject_unknown(document, &RECIPE_FIELDS, "recipe")?;
    let schema = document.field("schema")?.as_u32()?;
    if schema == 0 || schema > RECIPE_SCHEMA {
        return Err(unusable("the recipe schema is not one this build reads")
            .with_context("schema", schema.to_string())
            .with_context("supported", RECIPE_SCHEMA.to_string()));
    }
    let id = Identifier::parse(document.field("id")?.as_text()?)?;
    let category = MaterialCategory::parse(document.field("category")?.as_text()?)?;

    let listed = document.field("ramp")?.as_array()?;
    if listed.len() > MAX_RAMP_STOPS {
        return Err(unusable("the ramp has more stops than a recipe accepts")
            .with_context("stops", listed.len().to_string()));
    }
    let mut stops = Vec::with_capacity(listed.len());
    for stop in listed {
        stops.push(parse_colour(stop.as_text()?)?);
    }
    let ramp = Ramp::new(stops)?;

    let levels = document.field("levels")?.as_u32()?;
    if levels as usize > MAX_RAMP_STOPS {
        return Err(unusable("levels is out of range").with_context("levels", levels.to_string()));
    }

    let mottle = {
        let node = document.field("mottle")?;
        reject_unknown(
            node,
            &["cells_x", "cells_y", "octaves", "strength", "cracks"],
            "mottle",
        )?;
        Mottle {
            cells_x: cells(node, "cells_x")?,
            cells_y: cells(node, "cells_y")?,
            octaves: octaves(node, "octaves")?,
            strength: unit(node, "strength")?,
            cracks: unit(node, "cracks")?,
        }
    };

    let strips = match document.field("strips")? {
        Json::Null => None,
        node => {
            reject_unknown(
                node,
                &[
                    "count",
                    "gap",
                    "jitter",
                    "grain_across",
                    "grain_along",
                    "grain_strength",
                ],
                "strips",
            )?;
            Some(Strips {
                count: cells(node, "count")?,
                gap: bounded(node, "gap", 0.0, 0.45)?,
                jitter: unit(node, "jitter")?,
                grain_across: cells(node, "grain_across")?,
                grain_along: cells(node, "grain_along")?,
                grain_strength: unit(node, "grain_strength")?,
            })
        }
    };

    let courses = match document.field("courses")? {
        Json::Null => None,
        node => {
            reject_unknown(node, &["rows", "columns", "mortar", "jitter"], "courses")?;
            let rows = cells(node, "rows")?;
            if rows % 2 != 0 {
                // Every other course is offset by half a unit; an odd count
                // leaves the offset mid-way at the seam, and the bond breaks.
                return Err(unusable("courses need an even number of rows to tile")
                    .with_context("rows", rows.to_string()));
            }
            Some(Courses {
                rows,
                columns: cells(node, "columns")?,
                mortar: bounded(node, "mortar", 0.0, 0.45)?,
                jitter: unit(node, "jitter")?,
            })
        }
    };

    let speckle = match document.field("speckle")? {
        Json::Null => None,
        node => {
            reject_unknown(node, &["cells", "density", "contrast"], "speckle")?;
            Some(Speckle {
                cells: cells(node, "cells")?,
                density: unit(node, "density")?,
                contrast: unit(node, "contrast")?,
            })
        }
    };

    Ok(RecipeDocument {
        id,
        category,
        recipe: Recipe {
            ramp,
            mottle,
            strips,
            courses,
            speckle,
            contrast: bounded(document, "contrast", 0.0, 16.0)?,
            levels,
            relief: unit(document, "relief")?,
        },
    })
}

/// Render a recipe document as JSON.
#[must_use]
pub fn to_json(document: &RecipeDocument) -> Json {
    let recipe = &document.recipe;
    let float = Json::Float;
    let count = |value: u32| Json::Integer(i64::from(value));
    Json::object([
        ("schema", count(RECIPE_SCHEMA)),
        ("id", Json::text(document.id.to_string())),
        ("category", Json::text(document.category.as_str())),
        (
            "ramp",
            Json::Array(
                recipe
                    .ramp
                    .stops()
                    .iter()
                    .map(|stop| Json::text(format_colour(*stop)))
                    .collect(),
            ),
        ),
        ("levels", count(recipe.levels)),
        ("contrast", float(recipe.contrast)),
        ("relief", float(recipe.relief)),
        (
            "mottle",
            Json::object([
                ("cells_x", count(recipe.mottle.cells_x)),
                ("cells_y", count(recipe.mottle.cells_y)),
                ("octaves", count(recipe.mottle.octaves)),
                ("strength", float(recipe.mottle.strength)),
                ("cracks", float(recipe.mottle.cracks)),
            ]),
        ),
        (
            "strips",
            recipe.strips.map_or(Json::Null, |strips| {
                Json::object([
                    ("count", count(strips.count)),
                    ("gap", float(strips.gap)),
                    ("jitter", float(strips.jitter)),
                    ("grain_across", count(strips.grain_across)),
                    ("grain_along", count(strips.grain_along)),
                    ("grain_strength", float(strips.grain_strength)),
                ])
            }),
        ),
        (
            "courses",
            recipe.courses.map_or(Json::Null, |courses| {
                Json::object([
                    ("rows", count(courses.rows)),
                    ("columns", count(courses.columns)),
                    ("mortar", float(courses.mortar)),
                    ("jitter", float(courses.jitter)),
                ])
            }),
        ),
        (
            "speckle",
            recipe.speckle.map_or(Json::Null, |speckle| {
                Json::object([
                    ("cells", count(speckle.cells)),
                    ("density", float(speckle.density)),
                    ("contrast", float(speckle.contrast)),
                ])
            }),
        ),
    ])
}

/// Render a recipe document as text, in the form `from_text` reads.
#[must_use]
pub fn to_text(document: &RecipeDocument) -> String {
    to_json(document).to_pretty()
}

/// `#rrggbb`, opaque. Alpha is not a recipe's business: coverage comes from
/// the blend mode, and a translucent stop in a stone ramp is a mistake.
fn parse_colour(raw: &str) -> Result<Rgba> {
    let digits = raw
        .strip_prefix('#')
        .filter(|digits| digits.len() == 6 && digits.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| {
            unusable("a ramp stop is written #rrggbb").with_context("stop", raw.to_owned())
        })?;
    let channel = |at: usize| u8::from_str_radix(&digits[at..at + 2], 16).unwrap_or(0);
    Ok(Rgba::opaque(channel(0), channel(2), channel(4)))
}

fn format_colour(colour: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b)
}

fn cells(node: &Json, name: &'static str) -> Result<u32> {
    let value = node.field(name)?.as_u32()?;
    if value == 0 || value > MAX_CELLS {
        return Err(unusable("a lattice count is out of range")
            .with_context("field", name)
            .with_context("value", value.to_string())
            .with_context("range", format!("1..={MAX_CELLS}")));
    }
    Ok(value)
}

fn octaves(node: &Json, name: &'static str) -> Result<u32> {
    let value = node.field(name)?.as_u32()?;
    if value == 0 || value > MAX_OCTAVES {
        return Err(unusable("an octave count is out of range")
            .with_context("field", name)
            .with_context("value", value.to_string())
            .with_context("range", format!("1..={MAX_OCTAVES}")));
    }
    Ok(value)
}

fn unit(node: &Json, name: &'static str) -> Result<f64> {
    bounded(node, name, 0.0, 1.0)
}

fn bounded(node: &Json, name: &'static str, low: f64, high: f64) -> Result<f64> {
    let value = node.field(name)?.as_f64()?;
    if !(low..=high).contains(&value) {
        return Err(unusable("a recipe value is out of range")
            .with_context("field", name)
            .with_context("value", value.to_string())
            .with_context("range", format!("{low}..={high}")));
    }
    Ok(value)
}

fn reject_unknown(node: &Json, known: &[&str], section: &'static str) -> Result<()> {
    let Json::Object(fields) = node else {
        return Err(unusable("expected an object").with_context("section", section));
    };
    for key in fields.keys() {
        if !known.contains(&key.as_str()) {
            return Err(
                unusable("the recipe carries a field this build does not know")
                    .with_context("section", section)
                    .with_context("field", key.clone()),
            );
        }
    }
    Ok(())
}

fn unusable(message: &'static str) -> Error {
    Error::new(Domain::Content, "recipe", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_asset::provenance::Provenance;
    use nexora_asset::texture::Resolution;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).unwrap()
    }

    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "nexora-recipes-{}-{name}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    const BASALT: &str = r##"{
        "schema": 1,
        "id": "nexora:recipe/stone/basalt",
        "category": "stone",
        "ramp": ["#1c1c20", "#2b2b30", "#3b3a3f", "#4f4d52"],
        "levels": 4,
        "contrast": 2.2,
        "relief": 0.5,
        "mottle": { "cells_x": 4, "cells_y": 4, "octaves": 4, "strength": 1.0, "cracks": 0.1 },
        "strips": null,
        "courses": null,
        "speckle": { "cells": 16, "density": 0.12, "contrast": 0.2 }
    }"##;

    fn material(recipe: Option<&str>, category: MaterialCategory) -> SurfaceMaterial {
        let builder = SurfaceMaterial::builder(
            id("nexora:material/stone/basalt"),
            category,
            Resolution::square(16).unwrap(),
            Provenance::authored("operator", "test"),
        );
        match recipe {
            Some(raw) => builder.recipe(id(raw)),
            None => builder,
        }
        .build()
        .unwrap()
    }

    fn book_with(name: &str, file: &str, text: &str) -> (PathBuf, RecipeBook) {
        let root = scratch(name);
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        let book = RecipeBook::at(&root);
        (root, book)
    }

    #[test]
    fn a_recipe_document_round_trips_and_its_fingerprint_ignores_layout() {
        let document = from_text(BASALT).expect("a valid recipe");
        assert_eq!(document.id, id("nexora:recipe/stone/basalt"));
        assert_eq!(document.recipe.ramp.len(), 4);
        assert_eq!(
            document.recipe.ramp.stops()[0],
            Rgba::opaque(0x1c, 0x1c, 0x20)
        );

        let written = to_text(&document);
        let again = from_text(&written).unwrap();
        assert_eq!(again, document);
        assert_eq!(
            to_text(&again),
            written,
            "write, read, write is byte-identical"
        );

        // Whitespace and key order are not the recipe.
        let squashed: String = BASALT.split_whitespace().collect::<Vec<_>>().join(" ");
        assert_eq!(
            from_text(&squashed).unwrap().fingerprint(),
            document.fingerprint()
        );

        // A number is.
        let edited = from_text(&BASALT.replace("\"cracks\": 0.1", "\"cracks\": 0.2")).unwrap();
        assert_ne!(edited.fingerprint(), document.fingerprint());
    }

    #[test]
    fn a_malformed_recipe_names_what_is_wrong() {
        let cases = [
            (BASALT.replace("\"#1c1c20\"", "\"1c1c20\""), "#rrggbb"),
            (BASALT.replace("\"#1c1c20\"", "\"#1c1c2g\""), "#rrggbb"),
            (BASALT.replace("\"octaves\": 4", "\"octaves\": 99"), "octave"),
            (BASALT.replace("\"cells\": 16", "\"cells\": 0"), "lattice"),
            (BASALT.replace("\"relief\": 0.5", "\"relief\": 1.5"), "relief"),
            (BASALT.replace("\"levels\": 4", "\"levels\": 400"), "levels"),
            (BASALT.replace("\"schema\": 1", "\"schema\": 2"), "schema"),
            (BASALT.replace("\"strips\": null,", ""), "strips"),
            (
                BASALT.replace("\"strips\": null", "\"strips\": null, \"shine\": 1.0"),
                "shine",
            ),
            (
                BASALT.replace(
                    "\"courses\": null",
                    "\"courses\": { \"rows\": 3, \"columns\": 2, \"mortar\": 0.1, \"jitter\": 0.1 }",
                ),
                "even",
            ),
            (
                BASALT.replace(
                    "[\"#1c1c20\", \"#2b2b30\", \"#3b3a3f\", \"#4f4d52\"]",
                    "[\"#1c1c20\"]",
                ),
                "two stops",
            ),
        ];
        for (text, needle) in cases {
            let err = from_text(&text).expect_err(needle);
            assert!(err.to_string().contains(needle), "`{needle}`: {err}");
        }
    }

    #[test]
    fn a_material_without_a_recipe_gets_its_category_default() {
        let resolved = RecipeBook::empty()
            .resolve(&material(None, MaterialCategory::Stone))
            .expect("no book is needed for a default");
        assert_eq!(
            resolved.recipe,
            Recipe::for_category(MaterialCategory::Stone).unwrap()
        );
        assert_eq!(resolved.preset, id("nexora:preset/stone"));
        assert_eq!(resolved.fingerprint, None);
    }

    #[test]
    fn a_named_recipe_is_found_by_identifier_and_fingerprinted() {
        let (root, book) = book_with("found", "nexora/stone/basalt.json", BASALT);
        let resolved = book
            .resolve(&material(
                Some("nexora:recipe/stone/basalt"),
                MaterialCategory::Stone,
            ))
            .expect("resolves");
        assert_eq!(resolved.preset, id("nexora:recipe/stone/basalt"));
        assert_eq!(resolved.recipe.levels, 4);
        assert_eq!(
            resolved.fingerprint,
            Some(from_text(BASALT).unwrap().fingerprint())
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_recipe_that_cannot_be_used_is_refused_with_a_reason() {
        let (root, book) = book_with("refused", "nexora/stone/basalt.json", BASALT);

        let err = book
            .resolve(&material(
                Some("nexora:recipe/stone/absent"),
                MaterialCategory::Stone,
            ))
            .expect_err("absent");
        assert!(err.to_string().contains("could not be read"), "{err}");

        let err = book
            .resolve(&material(
                Some("nexora:recipe/stone/basalt"),
                MaterialCategory::Wood,
            ))
            .expect_err("wrong category");
        assert!(err.to_string().contains("different category"), "{err}");

        let err = RecipeBook::empty()
            .resolve(&material(
                Some("nexora:recipe/stone/basalt"),
                MaterialCategory::Stone,
            ))
            .expect_err("no directory");
        assert!(err.to_string().contains("no recipe directory"), "{err}");

        // A copy someone forgot to rename.
        std::fs::write(root.join("nexora/stone/copy.json"), BASALT).unwrap();
        let err = book
            .resolve(&material(
                Some("nexora:recipe/stone/copy"),
                MaterialCategory::Stone,
            ))
            .expect_err("identifier mismatch");
        assert!(err.to_string().contains("different identifier"), "{err}");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn recipe_paths_come_from_identifiers_and_cannot_escape() {
        let root = Path::new("/r");
        assert_eq!(
            recipe_file(root, &id("nexora:recipe/stone/basalt")).unwrap(),
            Path::new("/r/nexora/stone/basalt.json")
        );
        assert_eq!(
            recipe_file(root, &id("example:recipe/moss")).unwrap(),
            Path::new("/r/example/moss.json")
        );
        assert!(recipe_file(root, &id("nexora:recipe/../../etc")).is_err());
        if let Ok(bare) = Identifier::parse("nexora:recipe/") {
            assert!(
                recipe_file(root, &bare).is_err(),
                "no path beyond the prefix"
            );
        }
    }

    #[test]
    fn the_rendered_recipe_is_the_one_the_document_describes() {
        // Two stones that differ only in their recipe must not come out as the
        // same stone -- the whole point of the module.
        let basalt = from_text(BASALT).unwrap().recipe;
        let pale = from_text(
            &BASALT
                .replace("#1c1c20", "#b8b2a6")
                .replace("#2b2b30", "#c9c3b8")
                .replace("#3b3a3f", "#d8d3c9")
                .replace("#4f4d52", "#e6e2da"),
        )
        .unwrap()
        .recipe;
        let resolution = Resolution::square(16).unwrap();
        let dark = basalt.render(resolution, 7).albedo_map().unwrap();
        let light = pale.render(resolution, 7).albedo_map().unwrap();
        let mean = |map: &nexora_asset::texture::TextureMap| {
            let bytes = map.pixels();
            bytes.iter().map(|b| f64::from(*b)).sum::<f64>() / bytes.len() as f64
        };
        assert!(
            mean(&light) > mean(&dark) + 100.0,
            "a pale ramp renders pale: {} vs {}",
            mean(&light),
            mean(&dark)
        );
    }
}
