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
