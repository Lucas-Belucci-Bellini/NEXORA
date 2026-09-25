//! The resource loader that turns a verified PNG into a [`TextureMap`].
//!
//! `nexora_resource` hands a loader bytes whose size and hash already match
//! the manifest (ADR-0021); this is the loader that knows the format. It is
//! where DEBT-0045 closes: the runtime can now go from
//! `nexora:texture/stone/basalt/albedo` to pixels without the tool that wrote
//! them.

use nexora_asset::texture::{MapRole, TextureFormat, TextureMap};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::ident::Identifier;
use nexora_resource::manager::Loader;
use nexora_resource::manifest::ResourceKind;

use crate::png;

/// Loads texture resources as one role of map.
///
/// The role is the caller's: a resource manifest knows a texture is a
/// texture, and the material that asked for it knows it wanted the albedo.
#[derive(Debug, Clone, Copy)]
pub struct TextureLoader {
    role: MapRole,
    max_edge: u32,
}

impl TextureLoader {
    /// A loader for one role, accepting any edge [`nexora_asset`] allows.
    #[must_use]
    pub const fn new(role: MapRole) -> Self {
        Self {
            role,
            max_edge: nexora_asset::texture::MAX_EDGE,
        }
    }

    /// The same loader, refusing any texture with an edge above `max_edge`.
    ///
    /// The first visual generation is 16x16; a runtime that only ever loads
    /// that generation can say so, and a larger file is then a content error
    /// found at load rather than a surprise in video memory.
    #[must_use]
    pub const fn at_most(mut self, max_edge: u32) -> Self {
        self.max_edge = max_edge;
        self
    }
}

impl Loader for TextureLoader {
    type Output = TextureMap;

    fn kind(&self) -> ResourceKind {
        ResourceKind::Texture
    }

    fn load(&self, id: &Identifier, bytes: &[u8]) -> Result<TextureMap> {
        let decoded =
            png::decode(bytes).map_err(|error| error.with_context("resource", id.to_string()))?;
        let resolution = decoded.resolution;
        if resolution.width > self.max_edge || resolution.height > self.max_edge {
            return Err(Error::new(
                Domain::Content,
                "texture-loader",
                "the texture is larger than this loader accepts",
            )
            .with_recovery(Recovery::Reject)
            .with_context("resource", id.to_string())
            .with_context(
                "size",
                format!("{}x{}", resolution.width, resolution.height),
            )
            .with_context("max_edge", self.max_edge.to_string()));
        }
        TextureMap::new(
            self.role,
            TextureFormat::eight_bit(decoded.layout),
            resolution,
            decoded.pixels,
        )
    }

    fn cost(&self, value: &TextureMap, _file_size: u64) -> u64 {
        // What it holds in memory, not what it took on disk: a 337-byte PNG
        // is a kilobyte of pixels, and the budget is a memory budget.
        value.pixels().len() as u64
    }
}
