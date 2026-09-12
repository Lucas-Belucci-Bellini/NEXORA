//! # NEXORA Texture Forge
//!
//! Generates, transforms, validates, organises, versions and records surface
//! materials. A content tool, not an engine crate: it depends on
//! `nexora-asset`'s public contracts and on `nexora-foundation`, and on nothing
//! that needs a running game.
//!
//! ```text
//! definition ──► generator ──► pipeline ──► validator ──► registry
//!                    │             │            │
//!                 recipe        PBR maps     PASS/WARN/FAIL
//! ```
//!
//! # Originality
//!
//! Everything this crate emits is computed from numbers. No external image is
//! decoded, embedded, sampled or shipped, so the output is
//! `NEXORA_ORIGINAL` / `PROCEDURAL_DERIVATIVE` by construction rather than by
//! assertion — and `nexora_asset::GeneratedMaterial` refuses to exist without
//! the record that says so.
//!
//! A reference image may be *studied* to choose the numbers, which is what
//! `NEXORA ORIGINAL CONTENT AND ASSET POLICY.md` permits: *"architectural
//! inspiration may be studied; implementation, assets and distinctive protected
//! expression must be independently created"*.

pub mod color;
pub mod deflate;
pub mod forge;
pub mod layout;
pub mod manifest;
pub mod noise;
pub mod pbr;
pub mod png;
pub mod procedural;
pub mod raster;
pub mod recipe;
pub mod validator;

pub use color::{Ramp, Rgba};
pub use forge::{Forge, Listing, Outcome, WriteStatus};
pub use manifest::Manifest;
pub use noise::Noise;
pub use pbr::{PbrPipeline, PBR_PIPELINE, PBR_VERSION};
pub use png::{adler32, decode as decode_png, encode as encode_png, Decoded};
pub use procedural::{ProceduralGenerator, AUTHOR, PROCEDURAL_GENERATOR, PROCEDURAL_VERSION, TOOL};
pub use raster::Canvas;
pub use recipe::Recipe;
pub use validator::{seam_ratio, Validator};
