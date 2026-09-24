//! Block content, described by data.
//!
//! `Block System.md` §51: *"idealmente um bloco pode ser descrito por dados"*,
//! and §50: official content uses exactly the API a mod does. So the blocks the
//! first visual generation adds are not written into `World::create`; they are
//! a document, read here and handed to [`World::create_with`] like any other
//! content would be:
//!
//! ```json
//! {
//!   "schema": 1,
//!   "materials": ["stone/basalt.json"],
//!   "blocks": [
//!     { "id": "nexora:block/stone/basalt", "solid": true,
//!       "surface": "nexora:material/stone/basalt" }
//!   ]
//! }
//! ```
//!
//! `materials` are the definition files to register, relative to the document.
//! Each block names the material it is drawn with by **identifier** — the same
//! reference a block would use for any asset (`Block System.md` §21) — and the
//! loader refuses a block whose surface is not among them, so a typo is an
//! error at load rather than a block drawn with [`UNMAPPED_SURFACE`].
//!
//! # Why `surface` and not `material`
//!
//! `NEXORA NAMING AND TERMINOLOGY.md` forbids one name for two concepts, and
//! "material" already means the physics material in `nexora-physics`. The field
//! names the thing the mesher shows: a surface.
//!
//! [`UNMAPPED_SURFACE`]: crate::surfaces::UNMAPPED_SURFACE

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use nexora_asset::document;
use nexora_asset::json::{self, Json};
use nexora_asset::material::SurfaceMaterial;
use nexora_asset::registry::MaterialRegistry;
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_world::world::{BlockDefinition, World};

use crate::surfaces::SurfaceTable;

/// The newest block content schema this build reads.
pub const BLOCK_CONTENT_SCHEMA: u32 = 1;

/// Most blocks one document may add.
///
/// Registry runtime ids are `u32`, and a hostile document should not be a way
/// to exhaust memory before that limit is reached.
pub const MAX_CONTENT_BLOCKS: usize = 16_384;

const DOCUMENT_FIELDS: [&str; 3] = ["schema", "materials", "blocks"];
const BLOCK_FIELDS: [&str; 3] = ["id", "solid", "surface"];

/// One block a content document adds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentBlock {
    /// Its identifier.
    pub id: Identifier,
    /// What the world needs to know about it.
    pub definition: BlockDefinition,
    /// The surface material it is drawn with.
    pub surface: Identifier,
}

/// Blocks and the surface materials they are drawn with, read and checked.
#[derive(Debug, Clone)]
pub struct BlockContent {
    materials: Vec<SurfaceMaterial>,
    blocks: Vec<ContentBlock>,
}

impl BlockContent {
    /// Read a block content document and every material file it lists.
    ///
    /// # Errors
    ///
    /// Returns an error when the document or a material file cannot be read,
    /// does not parse, lists a path outside its own directory, repeats a block
    /// or a material, or names a surface it does not list.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|cause| {
            invalid("the block content document could not be read")
                .with_context("path", path.display().to_string())
                .with_context("cause", cause.to_string())
        })?;
        Self::from_text(&text, path.parent().unwrap_or_else(|| Path::new("")))
    }

    /// Read a block content document from text, resolving files against a
    /// directory.
    ///
    /// # Errors
    ///
    /// See [`BlockContent::load`].
    pub fn from_text(text: &str, base: &Path) -> Result<Self> {
        let document = json::parse(text)?;
        reject_unknown(&document, &DOCUMENT_FIELDS, "block content")?;
        let schema = document.field("schema")?.as_u32()?;
        if schema == 0 || schema > BLOCK_CONTENT_SCHEMA {
            return Err(
                invalid("the block content schema is not one this build reads")
                    .with_context("schema", schema.to_string())
                    .with_context("supported", BLOCK_CONTENT_SCHEMA.to_string()),
            );
        }

        let mut materials = Vec::new();
        let mut material_ids = BTreeSet::new();
        for entry in document.field("materials")?.as_array()? {
            let written = entry.as_text()?;
            let file = base.join(contained(written)?);
            let text = std::fs::read_to_string(&file).map_err(|cause| {
                invalid("a material definition could not be read")
                    .with_context("path", file.display().to_string())
                    .with_context("cause", cause.to_string())
            })?;
            let material = document::from_text(&text)?;
            if !material_ids.insert(material.id().clone()) {
                return Err(invalid("a material is listed twice")
                    .with_context("material", material.id().to_string()));
            }
            materials.push(material);
        }

        let listed = document.field("blocks")?.as_array()?;
        if listed.len() > MAX_CONTENT_BLOCKS {
            return Err(
                invalid("the document adds more blocks than one document may")
                    .with_context("blocks", listed.len().to_string())
                    .with_context("limit", MAX_CONTENT_BLOCKS.to_string()),
            );
        }
        let mut blocks = Vec::with_capacity(listed.len());
        let mut block_ids = BTreeSet::new();
        for node in listed {
            reject_unknown(node, &BLOCK_FIELDS, "blocks[]")?;
            let id = Identifier::parse(node.field("id")?.as_text()?)?;
            let surface = Identifier::parse(node.field("surface")?.as_text()?)?;
            if !material_ids.contains(&surface) {
                return Err(
                    invalid("a block names a surface the document does not list")
                        .with_context("block", id.to_string())
                        .with_context("surface", surface.to_string()),
                );
            }
            if !block_ids.insert(id.clone()) {
                return Err(
                    invalid("a block is listed twice").with_context("block", id.to_string())
                );
            }
            blocks.push(ContentBlock {
                id,
                definition: BlockDefinition {
                    solid: node.field("solid")?.as_bool()?,
                },
                surface,
            });
        }
        Ok(Self { materials, blocks })
    }

    /// The blocks, in document order.
    #[must_use]
    pub fn blocks(&self) -> &[ContentBlock] {
        &self.blocks
    }

    /// The material definitions, in document order.
    #[must_use]
    pub fn materials(&self) -> &[SurfaceMaterial] {
        &self.materials
    }

    /// The blocks in the form [`World::create_with`] and
    /// `nexora_world::persist::load_with` take.
    #[must_use]
    pub fn block_definitions(&self) -> Vec<(Identifier, BlockDefinition)> {
        self.blocks
            .iter()
            .map(|block| (block.id.clone(), block.definition.clone()))
            .collect()
    }

    /// A frozen registry holding every listed material.
    ///
    /// # Errors
    ///
    /// Returns an error when a material breaks a registry rule.
    pub fn material_registry(&self) -> Result<MaterialRegistry> {
        let mut registry = MaterialRegistry::new()?;
        for material in &self.materials {
            registry.register(material.clone())?;
        }
        registry.freeze();
        Ok(registry)
    }

    /// The surface table for a world holding this content.
    ///
    /// # Errors
    ///
    /// Returns an error when a block is not registered in the world, which
    /// means the world was created without this content.
    pub fn surface_table(
        &self,
        world: &World,
        materials: &MaterialRegistry,
    ) -> Result<SurfaceTable> {
        let mut builder = SurfaceTable::resolved(world, materials);
        for block in &self.blocks {
            builder.assign(&block.id, &block.surface)?;
        }
        Ok(builder.build())
    }
}

/// A relative path that stays inside the directory it is resolved against.
fn contained(written: &str) -> Result<PathBuf> {
    let path = Path::new(written);
    let escapes = written.is_empty()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        });
    if escapes {
        return Err(invalid("a path leaves the content document's directory")
            .with_context("path", written.to_owned()));
    }
    Ok(path.to_path_buf())
}

fn reject_unknown(node: &Json, known: &[&str], section: &'static str) -> Result<()> {
    let Json::Object(fields) = node else {
        return Err(invalid("expected an object").with_context("section", section));
    };
    for key in fields.keys() {
        if !known.contains(&key.as_str()) {
            return Err(
                invalid("the block content carries a field this build does not know")
                    .with_context("section", section)
                    .with_context("field", key.clone()),
            );
        }
    }
    Ok(())
}

fn invalid(message: &'static str) -> Error {
    Error::new(Domain::Content, "block-content", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surfaces::{WorldSurfaces, UNMAPPED_SURFACE};
    use nexora_asset::material::MaterialCategory;
    use nexora_asset::provenance::Provenance;
    use nexora_asset::texture::Resolution;
    use nexora_foundation::spatial::{BlockPos, ChunkCoord};
    use nexora_foundation::time::CalendarConfig;
    use nexora_mesh::view::VoxelView;
    use nexora_world::world::WorldDescriptor;

    fn id(raw: &str) -> Identifier {
        Identifier::parse(raw).unwrap()
    }

    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "nexora-block-content-{}-{name}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn author(dir: &Path, file: &str, material: &str) {
        let definition = SurfaceMaterial::builder(
            id(material),
            MaterialCategory::Stone,
            Resolution::square(16).unwrap(),
            Provenance::authored("operator", "test"),
        )
        .build()
        .unwrap();
        std::fs::write(dir.join(file), document::to_text(&definition)).unwrap();
    }

    const TWO_STONES: &str = r#"{
        "schema": 1,
        "materials": ["basalt.json", "slate.json"],
        "blocks": [
            { "id": "nexora:block/stone/basalt", "solid": true, "surface": "nexora:material/stone/basalt" },
            { "id": "nexora:block/stone/slate", "solid": true, "surface": "nexora:material/stone/slate" }
        ]
    }"#;

    fn two_stones(dir: &Path) -> BlockContent {
        author(dir, "basalt.json", "nexora:material/stone/basalt");
        author(dir, "slate.json", "nexora:material/stone/slate");
        BlockContent::from_text(TWO_STONES, dir).expect("valid content")
    }

    #[test]
    fn content_blocks_reach_the_mesher_through_their_own_surfaces() {
        let dir = scratch("reach");
        let content = two_stones(&dir);
        let materials = content.material_registry().unwrap();

        let mut world = World::create_with(
            WorldDescriptor::new("content", 9).unwrap(),
            CalendarConfig::earthlike(),
            &content.block_definitions(),
        )
        .unwrap();
        world.bring_online().unwrap();
        world.load_or_generate(ChunkCoord::new(0, 0)).unwrap();

        let table = content.surface_table(&world, &materials).unwrap();
        let basalt = world.block_id(&id("nexora:block/stone/basalt")).unwrap();
        let slate = world.block_id(&id("nexora:block/stone/slate")).unwrap();
        let basalt_surface = table.surface_of(basalt.0).unwrap();
        let slate_surface = table.surface_of(slate.0).unwrap();
        assert_ne!(basalt_surface, UNMAPPED_SURFACE);
        assert_ne!(slate_surface, UNMAPPED_SURFACE);
        assert_ne!(basalt_surface, slate_surface, "two stones, two surfaces");

        // Only the built-ins lack a surface: nothing the content added.
        let unmapped: Vec<String> = table.unmapped().iter().map(ToString::to_string).collect();
        assert!(
            unmapped.iter().all(|block| !block.contains("/stone/")),
            "{unmapped:?}"
        );

        // And the view the mesher reads answers with them.
        let at = BlockPos::new(1, 150, 1);
        world.set_block(at, basalt).unwrap();
        let view = WorldSurfaces::new(&world, table);
        assert_eq!(view.surface_at(at), Some(basalt_surface));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_broken_document_is_refused_with_its_reason() {
        let dir = scratch("broken");
        author(&dir, "basalt.json", "nexora:material/stone/basalt");
        author(&dir, "slate.json", "nexora:material/stone/slate");
        let cases = [
            (
                TWO_STONES.replace("material/stone/slate\" }", "material/stone/granite\" }"),
                "does not list",
            ),
            (
                TWO_STONES.replace("block/stone/slate", "block/stone/basalt"),
                "listed twice",
            ),
            (
                TWO_STONES.replace("\"slate.json\"", "\"basalt.json\""),
                "listed twice",
            ),
            (TWO_STONES.replace("\"slate.json\"", "\"../slate.json\""), "leaves"),
            (TWO_STONES.replace("\"slate.json\"", "\"absent.json\""), "could not be read"),
            (TWO_STONES.replace("\"schema\": 1", "\"schema\": 2"), "schema"),
            (
                TWO_STONES.replace("\"solid\": true, \"surface\": \"nexora:material/stone/basalt\"",
                    "\"solid\": true, \"hardness\": 3, \"surface\": \"nexora:material/stone/basalt\""),
                "hardness",
            ),
        ];
        for (text, needle) in cases {
            let err = BlockContent::from_text(&text, &dir).expect_err(needle);
            assert!(err.to_string().contains(needle), "`{needle}`: {err}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_world_created_without_the_content_cannot_take_its_surfaces() {
        let dir = scratch("without");
        let content = two_stones(&dir);
        let materials = content.material_registry().unwrap();
        let world = World::create(
            WorldDescriptor::new("bare", 9).unwrap(),
            CalendarConfig::earthlike(),
        )
        .unwrap();
        let err = content
            .surface_table(&world, &materials)
            .expect_err("the blocks are not in this world");
        assert!(
            err.to_string().contains("nexora:block/stone/basalt"),
            "{err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
