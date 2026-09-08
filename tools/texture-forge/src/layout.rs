//! Where a material's files go on disk.
//!
//! `NEXORA CONTENT PIPELINE SPECIFICATION.md` is explicit that *"nome de
//! arquivo não deve ser o identificador lógico primário"* — the file name is a
//! consequence of the identifier, never the other way round. So this module
//! derives paths and never accepts them, and the derivation is the only one in
//! the project.
//!
//! ```text
//! <root>/<namespace>/<material path>/material.json
//!                                   /albedo.png
//!                                   /normal.png
//!                                   /…
//!                                   /preview.png
//! ```
//!
//! # Why the namespace is a directory
//!
//! A mod owns its namespace (`Registry System.md` §33). Giving each one its own
//! directory means a mod's assets can be added or removed as a unit, and two
//! mods that both call something `stone` cannot collide on the filesystem any
//! more than they can collide in the registry.

use std::path::{Path, PathBuf};

use nexora_asset::material::{SurfaceMaterial, MATERIAL_PATH_PREFIX};
use nexora_asset::texture::MapRole;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;

/// The file a material's definition is written to.
pub const DEFINITION_FILE: &str = "material.json";

/// The file a material's preview is written to.
pub const PREVIEW_FILE: &str = "preview.png";

/// Where one material's files live, under a root.
#[must_use]
pub fn material_dir(root: &Path, id: &Identifier) -> PathBuf {
    let stem = id
        .path()
        .strip_prefix(MATERIAL_PATH_PREFIX)
        .unwrap_or_else(|| id.path());
    let mut path = root.join(id.namespace().as_str());
    for segment in stem.split('/') {
        path.push(segment);
    }
    path
}

/// The file one of a material's maps is written to.
#[must_use]
pub fn map_file(root: &Path, id: &Identifier, role: MapRole) -> PathBuf {
    material_dir(root, id).join(format!("{}.png", role.as_str()))
}

/// The file a material's definition is written to.
#[must_use]
pub fn definition_file(root: &Path, id: &Identifier) -> PathBuf {
    material_dir(root, id).join(DEFINITION_FILE)
}

/// The file a material's preview is written to.
#[must_use]
pub fn preview_file(root: &Path, id: &Identifier) -> PathBuf {
    material_dir(root, id).join(PREVIEW_FILE)
}

/// Check that a material's identifier yields file names a filesystem accepts.
///
/// `Identifier` already restricts the path to `[a-z0-9_/.-]`, which rules out
/// separators, spaces and case collisions. What it does not rule out is a
/// segment that means something to a filesystem rather than to us.
///
/// # Errors
///
/// Returns an error when a path segment is `.` or `..`, or when the identifier
/// would produce an empty directory name.
pub fn check_naming(material: &SurfaceMaterial) -> Result<()> {
    let stem = material
        .id()
        .path()
        .strip_prefix(MATERIAL_PATH_PREFIX)
        .unwrap_or_else(|| material.id().path());
    if stem.is_empty() {
        return Err(
            unusable("a material identifier needs a path beyond its prefix")
                .with_context("material", material.id().to_string()),
        );
    }
    for segment in stem.split('/') {
        if segment == "." || segment == ".." {
            return Err(unusable("a path segment means something to the filesystem")
                .with_context("material", material.id().to_string())
                .with_context("segment", segment.to_owned()));
        }
    }
    Ok(())
}

fn unusable(message: &'static str) -> Error {
    Error::new(Domain::Content, "asset-layout", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_asset::material::MaterialCategory;
    use nexora_asset::provenance::Provenance;
    use nexora_asset::texture::Resolution;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).expect("test identifier must be valid")
    }

    fn material(path: &str) -> SurfaceMaterial {
        SurfaceMaterial::builder(
            id(path),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .build()
        .expect("valid material")
    }

    #[test]
    fn paths_are_derived_from_the_identifier_and_never_from_a_name() {
        let root = Path::new("/out");
        let stone = id("nexora:material/stone_rough");
        assert_eq!(
            material_dir(root, &stone),
            Path::new("/out/nexora/stone_rough")
        );
        assert_eq!(
            map_file(root, &stone, MapRole::Albedo),
            Path::new("/out/nexora/stone_rough/albedo.png")
        );
        assert_eq!(
            definition_file(root, &stone),
            Path::new("/out/nexora/stone_rough/material.json")
        );
        assert_eq!(
            preview_file(root, &stone),
            Path::new("/out/nexora/stone_rough/preview.png")
        );
    }

    #[test]
    fn a_nested_identifier_becomes_nested_directories() {
        let root = Path::new("/out");
        let nested = id("example:material/wood/oak/dark");
        assert_eq!(
            material_dir(root, &nested),
            Path::new("/out/example/wood/oak/dark")
        );
    }

    #[test]
    fn two_namespaces_cannot_collide_on_disk() {
        let root = Path::new("/out");
        assert_ne!(
            material_dir(root, &id("nexora:material/stone")),
            material_dir(root, &id("example:material/stone"))
        );
    }

    #[test]
    fn a_segment_the_filesystem_would_interpret_is_refused() {
        // `Identifier` permits `.` and `-`, so `..` is reachable and would
        // escape the output root.
        let escaping = material("nexora:material/../../etc");
        let err = check_naming(&escaping).expect_err("`..` must never become a directory");
        assert_eq!(err.recovery(), Recovery::Reject);
        assert!(err.to_string().contains(".."), "{err}");

        check_naming(&material("nexora:material/stone")).expect("an ordinary name is fine");
        check_naming(&material("nexora:material/wood/oak")).expect("nesting is fine");
        // A dot inside a segment is not a path segment of its own.
        check_naming(&material("nexora:material/v1.2/stone")).expect("a dotted segment is fine");
    }
}
