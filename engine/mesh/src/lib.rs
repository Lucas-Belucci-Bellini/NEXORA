//! Voxel meshing: which surfaces exist, and how few rectangles describe them.
//!
//! Implements `RENDERER and GRAPHICS.md` **RENDER-9** — face culling and greedy
//! meshing — and the neighbour-awareness `CHUNK & VOXEL ENGINE.md` **CHUNK-27**
//! and **CHUNK-28** require at chunk borders.
//!
//! # There is no GPU here, and none is needed
//!
//! An earlier version of the benchmark listed mesh generation as unmeasurable
//! because there was *"no renderer to consume a mesh"*. That conflated two
//! different things. **Displaying** a mesh needs a renderer; **building** one
//! does not, and neither does checking that it is right:
//!
//! * a solid region must emit only its outer shell, never its interior;
//! * merging must not create or destroy surface area;
//! * a face on a chunk border must be culled by the neighbouring chunk;
//! * the same voxels must always produce the same geometry.
//!
//! Every one of those is a property of a data structure. RENDER-11's pipeline
//! is `block changed → chunk dirty → neighbour check → remesh → GPU update`,
//! and only the last arrow needs hardware.
//!
//! # What a mesher must not become
//!
//! `NEXORA DEPENDENCY MATRIX.md` puts Presentation above Simulation and forbids
//! it from mutating authoritative state. This crate reads voxels through
//! [`VoxelView`] and returns geometry; it depends on `nexora-foundation` alone,
//! so it cannot reach a world, a registry or a render backend even by accident.
//!
//! # Scope
//!
//! Face culling, greedy merging, and RENDER-10's split into render layers —
//! three of its four. `opaqueMesh`, `cutoutMesh` and `transparentMesh` follow
//! from the `BlendMode` a material declares; `waterMesh` does not, because
//! **nothing in the engine says a block is water**. Emitting it would mean
//! inventing the data that decides what goes in it. See [`RenderLayer`].
//!
//! RENDER-12's async meshing is not built either, and is recorded in the debt
//! register with a trigger.

pub mod greedy;
pub mod mesh;
pub mod view;

pub use greedy::mesh_region;
pub use mesh::{ChunkMesh, Facing, LayeredMesh, MeshState, Quad, RenderLayer, SurfaceId};
pub use view::{Extent, VoxelView};
