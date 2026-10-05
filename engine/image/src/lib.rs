//! # NEXORA image decoding
//!
//! The formats the runtime reads, and the resource loaders that read them.
//! Moved out of `tools/texture-forge` by ADR-0022 so that there is one
//! decoder: the forge writes PNGs, the runtime reads them, and both read with
//! this.
//!
//! ```text
//! resource manifest ─► verified bytes ─► png::decode ─► TextureMap
//!   (ADR-0021)          (size + hash)     (bounded inflate)
//! ```
//!
//! The inflater is `nexora_foundation::deflate`'s, the same one the save
//! codec and the encoder's own tests use: one implementation of RFC 1951 in
//! the workspace, reading all three block types.
pub mod loader;
pub mod png;

pub use loader::TextureLoader;

use nexora_asset::texture::{ChannelLayout, TextureMap};
use nexora_foundation::error::{Domain, Error, Recovery, Result};

/// A decoded 8-bit map as four channels a texel, in its own colour space
/// (`TextureMap::color_space`): what a GPU samples natively and what an
/// atlas tiles. No backend guarantees a three-channel format, so RGB, grey
/// and grey-alpha are widened here, once, rather than by each consumer.
///
/// # Errors
///
/// [`Recovery::Reject`] when the map is not 8 bits a channel: nothing in the
/// first generation is, and there is no path for it yet.
pub fn rgba8(map: &TextureMap) -> Result<Vec<u8>> {
    if map.format().bits_per_channel != 8 {
        return Err(Error::new(
            Domain::Content,
            "image",
            "only 8-bit maps widen to RGBA8; the first generation has no other",
        )
        .with_recovery(Recovery::Reject)
        .with_context("bits", map.format().bits_per_channel.to_string()));
    }
    let pixels = map.pixels();
    Ok(match map.format().channels {
        ChannelLayout::Rgba => pixels.to_vec(),
        ChannelLayout::Rgb => pixels
            .chunks_exact(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], u8::MAX])
            .collect(),
        ChannelLayout::GreyAlpha => pixels
            .chunks_exact(2)
            .flat_map(|ga| [ga[0], ga[0], ga[0], ga[1]])
            .collect(),
        ChannelLayout::Grey => pixels.iter().flat_map(|g| [*g, *g, *g, u8::MAX]).collect(),
    })
}
