//! # NEXORA Asset
//!
//! What the runtime must know about an asset, and nothing about how one is
//! made. Implements the parts of `RESOURCE AND ASSET SYSTEM.md`,
//! `MATERIAL AND SHADER SYSTEM.md` and
//! `NEXORA ASSET PROVENANCE AND LICENSE REGISTRY.md` that a running engine
//! needs, and none of the parts a content tool needs.
//!
//! The split is deliberate. `NEXORA DEPENDENCY MATRIX.md` places tools at the
//! end of the dependency direction, depending on public contracts; an image
//! encoder inside `engine/` would invert that. So this crate holds the
//! definitions, the registry and the validation verdicts, and
//! `tools/texture-forge` holds the generators, the pipelines and the encoders
//! that produce them.
//!
//! # Two things named "material"
//!
//! `nexora_physics::material::MaterialId` already exists and means friction,
//! restitution, density and drag — a surface *in contact*.
//! `NEXORA NAMING AND TERMINOLOGY.md` forbids one name covering two concepts,
//! so the visual one is [`SurfaceMaterial`] throughout, and its session handle
//! is `nexora_mesh::mesh::SurfaceId`, which was already defined as "what a face
//! shows" with the caller deciding what the number means.
//!
//! A block may carry both. They never meet.

pub mod document;
pub mod generator;
pub mod json;
pub mod material;
pub mod provenance;
pub mod registry;
pub mod texture;
pub mod validation;

pub use generator::{GeneratedMaterial, GenerationMode, GenerationRequest, TextureGenerator};
pub use json::Json;
pub use material::{MaterialCategory, PbrParameters, PhysicalScale, SurfaceMaterial};
pub use provenance::{AssetStatus, Provenance, ProvenanceClass, ReleaseStatus};
pub use registry::MaterialRegistry;
pub use texture::{MapRole, MapStatus, Resolution, TextureFormat, TextureMap};
pub use validation::{Finding, Severity, TextureValidationResult, Verdict};
