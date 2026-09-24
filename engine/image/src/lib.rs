//! # NEXORA image decoding
//!
//! The formats the runtime reads, and the resource loaders that read them.
//! Moved out of `tools/texture-forge` by ADR-0016 so that there is one
//! decoder: the forge writes PNGs, the runtime reads them, and both read with
//! this.
//!
//! ```text
//! resource manifest ─► verified bytes ─► png::decode ─► TextureMap
//!   (ADR-0015)          (size + hash)     (bounded inflate)
//! ```

pub mod inflate;
pub mod loader;
pub mod png;

pub use loader::TextureLoader;
