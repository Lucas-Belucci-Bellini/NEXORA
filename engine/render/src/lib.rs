//! The renderer's first pass (RENDER-3, RENDER-8, RENDER-10): chunk meshes,
//! drawn through the camera, over the RHI contract.
//!
//! The renderer does not render the world; it renders what it is handed
//! (`RENDERER and GRAPHICS.md`: *"O mundo não renderiza diretamente"*). A
//! [`ChunkMesh`] is data (ADR-0012), a [`CameraState`] is matrices
//! (ADR-0029), and [`ChunkPass`] turns them into RHI commands. It names the
//! [`Rhi`] trait and nothing below it, so the same pass runs on the null
//! backend without a GPU and on `wgpu` on one.
//!
//! # What a frame is, and why it is one submission
//!
//! [`ChunkPass::record`] appends a whole frame to one [`CommandList`]: clear
//! the colour and depth targets, write the camera, and draw every chunk the
//! frustum keeps. The caller submits it once and waits on one fence. On the
//! operator's GPU a fence round trip costs more than a small draw (baseline
//! Appendix K, Finding 29), so a pass that waited per chunk would spend its
//! frame waiting.
//!
//! # Where geometry lives
//!
//! A chunk's vertices are **relative to its region's minimum corner**, as
//! small exact `f32` integers, and uploaded once. Where the chunk is relative
//! to the render origin is a 16-byte uniform written every frame. So when the
//! origin moves (ADR-0029), no vertex buffer is rebuilt.
//!
//! # Not built yet
//!
//! No textures: faces are coloured by the direction they face, which makes a
//! frame checkable texel by texel against a ray cast on the CPU. The
//! first-generation albedos, cutout and transparent layers, lighting and
//! sorting come after this pass exists. So do index buffers (ADR-0028: six
//! vertices a quad until a renderer measures that it matters).

pub mod reference;

use nexora_camera::{CameraState, EXACT_OFFSET};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{Axis, BlockPos};
use nexora_mesh::{ChunkMesh, Extent, Facing};
use nexora_rhi::{
    Binding, BindingKind, BufferDesc, BufferHandle, ClearValue, Command, CommandList, Compare,
    DepthState, PipelineDesc, PipelineHandle, Rhi, ShaderStage, TextureFormat, TextureHandle,
    Usage, VertexAttribute, VertexFormat,
};

/// The pass's shaders, in WGSL (ADR-0026).
///
/// Slot 0 is the camera's view-projection, slot 1 the chunk's offset from the
/// render origin. Positions arrive relative to the chunk, so the shader adds
/// the offset before projecting.
pub const CHUNK_WGSL: &str = r"
struct Camera {
    view_projection: mat4x4<f32>,
};

struct Chunk {
    offset: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> chunk: Chunk;

struct Shaded {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_chunk(@location(0) position: vec3<f32>, @location(1) color: vec4<f32>) -> Shaded {
    var out: Shaded;
    out.position = camera.view_projection * vec4<f32>(position + chunk.offset.xyz, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_chunk(in: Shaded) -> @location(0) vec4<f32> {
    return in.color;
}
";

/// Bytes per vertex: a position of three `f32`, then a colour of four `u8`.
pub const VERTEX_STRIDE: u32 = 16;

/// Vertices per quad: two triangles, no index buffer (ADR-0028).
pub const VERTICES_PER_QUAD: u32 = 6;

/// The colour a frame starts with, where nothing is drawn.
pub const CLEAR_COLOR: [u8; 4] = [0, 0, 0, 255];

/// The colour of a face, by the direction it faces.
///
/// Six colours with components of only 0 and 255, so they survive any colour
/// target and any conversion exactly, and a texel read back names its face.
#[must_use]
pub const fn face_color(axis: Axis, facing: Facing) -> [u8; 4] {
    match (axis, facing) {
        (Axis::X, Facing::Positive) => [255, 0, 0, 255],
        (Axis::X, Facing::Negative) => [0, 255, 255, 255],
        (Axis::Y, Facing::Positive) => [0, 255, 0, 255],
        (Axis::Y, Facing::Negative) => [255, 0, 255, 255],
        (Axis::Z, Facing::Positive) => [0, 0, 255, 255],
        (Axis::Z, Facing::Negative) => [255, 255, 0, 255],
    }
}

/// A mesh's vertex bytes, relative to `region_min`.
///
/// A quad of cell `origin` facing `Positive` along an axis lies on the plane
/// `origin + 1`, one facing `Negative` on the plane `origin`, and spans
/// `width` blocks along the first of the axis's other two axes and `height`
/// along the second (`Axis::others`), as the mesher emits them.
///
/// # Errors
///
/// A quad is so far from `region_min` that its corners are not exact in
/// `f32`: the mesh does not belong to this region.
pub fn chunk_vertices(mesh: &ChunkMesh, region_min: BlockPos) -> Result<Vec<u8>> {
    let base = [region_min.x, region_min.y, region_min.z];
    let mut out = Vec::with_capacity(mesh.len() * (VERTICES_PER_QUAD * VERTEX_STRIDE) as usize);
    for quad in &mesh.quads {
        let along = quad.axis.index();
        let [first, second] = quad.axis.others().map(Axis::index);
        let mut corner = [0i64; 3];
        for axis in 0..3 {
            corner[axis] = quad.origin[axis] - base[axis];
        }
        if quad.facing == Facing::Positive {
            corner[along] += 1;
        }
        let far = corner[first].max(corner[second]).max(corner[along])
            + i64::from(quad.width.max(quad.height));
        let near = corner[first].min(corner[second]).min(corner[along]);
        if far >= EXACT_OFFSET || near <= -EXACT_OFFSET {
            return Err(wrong("a quad is too far from its region to place exactly")
                .with_context("origin", format!("{:?}", quad.origin))
                .with_context("region_min", format!("{region_min:?}")));
        }
        let color = face_color(quad.axis, quad.facing);
        let point = |u: u32, v: u32| {
            let mut p = corner;
            p[first] += i64::from(u);
            p[second] += i64::from(v);
            p.map(|value| value as f32)
        };
        let (w, h) = (quad.width, quad.height);
        for p in [
            point(0, 0),
            point(w, 0),
            point(w, h),
            point(0, 0),
            point(w, h),
            point(0, h),
        ] {
            for value in p {
                out.extend_from_slice(&value.to_le_bytes());
            }
            out.extend_from_slice(&color);
        }
    }
    Ok(out)
}

/// A chunk's geometry on the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuChunk {
    /// The meshed region: its minimum corner is where the vertices start.
    pub region: Extent,
    /// The vertex buffer, or `None` when the mesh was empty.
    pub vertices: Option<BufferHandle>,
    /// The chunk's offset from the render origin, written every frame.
    pub offset: BufferHandle,
    /// How many vertices the buffer holds.
    pub vertex_count: u32,
}

impl GpuChunk {
    /// Release the chunk's buffers. Deferred by the RHI until the last frame
    /// that drew it completes.
    ///
    /// # Errors
    ///
    /// A handle is stale, or the device is lost.
    pub fn destroy<R: Rhi + ?Sized>(self, rhi: &mut R) -> Result<()> {
        if let Some(vertices) = self.vertices {
            rhi.destroy_buffer(vertices)?;
        }
        rhi.destroy_buffer(self.offset)
    }
}

/// What one recorded frame holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameStats {
    /// Chunks drawn.
    pub drawn: u32,
    /// Chunks the frustum dropped.
    pub culled: u32,
    /// Chunks with no geometry.
    pub empty: u32,
    /// Vertices drawn.
    pub vertices: u64,
}

/// The pass: one pipeline, one camera uniform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPass {
    pipeline: PipelineHandle,
    camera: BufferHandle,
}

impl ChunkPass {
    /// Create the pipeline, drawing into `target` format with a reverse-Z
    /// depth test (ADR-0029), and the camera's uniform buffer.
    ///
    /// # Errors
    ///
    /// The backend refused the pipeline or the buffer.
    pub fn new<R: Rhi + ?Sized>(rhi: &mut R, target: TextureFormat) -> Result<Self> {
        let stage = |entry: &str| ShaderStage {
            entry: entry.into(),
            code: CHUNK_WGSL.as_bytes().to_vec(),
        };
        let pipeline = rhi.create_pipeline(&PipelineDesc {
            label: "chunk pass".into(),
            vertex: stage("vs_chunk"),
            fragment: stage("fs_chunk"),
            vertex_stride: VERTEX_STRIDE,
            attributes: vec![
                VertexAttribute {
                    location: 0,
                    format: VertexFormat::Float32x3,
                    offset: 0,
                },
                VertexAttribute {
                    location: 1,
                    format: VertexFormat::Unorm8x4,
                    offset: 12,
                },
            ],
            bindings: vec![BindingKind::Uniform, BindingKind::Uniform],
            depth: Some(DepthState {
                compare: Compare::Greater,
                write: true,
            }),
            targets: vec![target],
        })?;
        let camera = match rhi.create_buffer(&BufferDesc {
            label: "chunk pass camera".into(),
            size: 64,
            usage: Usage::UNIFORM | Usage::COPY_DST,
        }) {
            Ok(camera) => camera,
            Err(error) => {
                rhi.destroy_pipeline(pipeline)?;
                return Err(error);
            }
        };
        Ok(Self { pipeline, camera })
    }

    /// Create a chunk's buffers, and append the write of its vertices to
    /// `list`: an upload rides in the same submission as the frame that
    /// first needs it.
    ///
    /// # Errors
    ///
    /// The mesh does not belong to `region`, it is too large for one draw, or
    /// the backend refused a buffer.
    pub fn upload<R: Rhi + ?Sized>(
        &self,
        rhi: &mut R,
        list: &mut CommandList,
        mesh: &ChunkMesh,
        region: Extent,
    ) -> Result<GpuChunk> {
        let bytes = chunk_vertices(mesh, region.origin)?;
        let vertex_count = u32::try_from(bytes.len() / VERTEX_STRIDE as usize)
            .map_err(|_| wrong("a chunk has more vertices than one draw holds"))?;
        let offset = rhi.create_buffer(&BufferDesc {
            label: "chunk offset".into(),
            size: 16,
            usage: Usage::UNIFORM | Usage::COPY_DST,
        })?;
        let vertices = if bytes.is_empty() {
            None
        } else {
            let buffer = match rhi.create_buffer(&BufferDesc {
                label: "chunk vertices".into(),
                size: bytes.len() as u64,
                usage: Usage::VERTEX | Usage::COPY_DST,
            }) {
                Ok(buffer) => buffer,
                Err(error) => {
                    rhi.destroy_buffer(offset)?;
                    return Err(error);
                }
            };
            list.push(Command::WriteBuffer {
                buffer,
                offset: 0,
                data: bytes,
            });
            Some(buffer)
        };
        Ok(GpuChunk {
            region,
            vertices,
            offset,
            vertex_count,
        })
    }

    /// Append one frame to `list`: clear `target` and `depth`, write the
    /// camera, and draw every chunk the frustum keeps.
    ///
    /// # Errors
    ///
    /// A chunk is too far from the camera's render origin to place exactly.
    pub fn record(
        &self,
        list: &mut CommandList,
        camera: &CameraState,
        target: TextureHandle,
        depth: TextureHandle,
        chunks: &[GpuChunk],
    ) -> Result<FrameStats> {
        let mut stats = FrameStats::default();
        list.push(Command::Clear {
            texture: target,
            value: ClearValue::Color(CLEAR_COLOR.map(|c| f32::from(c) / 255.0)),
        })
        .push(Command::Clear {
            texture: depth,
            // Reverse-Z: the far plane is 0 (ADR-0029).
            value: ClearValue::Depth(0.0),
        })
        .push(Command::WriteBuffer {
            buffer: self.camera,
            offset: 0,
            data: camera.view_projection.to_bytes().to_vec(),
        });
        for chunk in chunks {
            let Some(vertices) = chunk.vertices else {
                stats.empty += 1;
                continue;
            };
            let min = chunk.region.origin;
            let [sx, sy, sz] = chunk.region.size.map(i64::from);
            let max = BlockPos::new(min.x + sx, min.y + sy, min.z + sz);
            if !camera.sees_blocks(min, max) {
                stats.culled += 1;
                continue;
            }
            let offset = camera.origin.offset_of(min)?;
            let mut data = Vec::with_capacity(16);
            for value in [offset[0], offset[1], offset[2], 0.0] {
                data.extend_from_slice(&value.to_le_bytes());
            }
            list.push(Command::WriteBuffer {
                buffer: chunk.offset,
                offset: 0,
                data,
            })
            .push(Command::Draw {
                pipeline: self.pipeline,
                buffer: vertices,
                target,
                depth: Some(depth),
                bindings: vec![
                    Binding::Uniform(self.camera),
                    Binding::Uniform(chunk.offset),
                ],
                vertices: chunk.vertex_count,
            });
            stats.drawn += 1;
            stats.vertices += u64::from(chunk.vertex_count);
        }
        Ok(stats)
    }

    /// Release the pipeline and the camera buffer.
    ///
    /// # Errors
    ///
    /// A handle is stale, or the device is lost.
    pub fn destroy<R: Rhi + ?Sized>(self, rhi: &mut R) -> Result<()> {
        rhi.destroy_buffer(self.camera)?;
        rhi.destroy_pipeline(self.pipeline)
    }
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Render, "chunk-pass", message).with_recovery(Recovery::Reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_mesh::{Quad, SurfaceId};

    fn floats(bytes: &[u8]) -> Vec<([f32; 3], [u8; 4])> {
        bytes
            .chunks_exact(VERTEX_STRIDE as usize)
            .map(|v| {
                let f = |i: usize| f32::from_le_bytes([v[i], v[i + 1], v[i + 2], v[i + 3]]);
                ([f(0), f(4), f(8)], [v[12], v[13], v[14], v[15]])
            })
            .collect()
    }

    #[test]
    fn a_quad_becomes_two_triangles_on_its_plane_relative_to_the_region() {
        // The top of cell (10, 5, 20), two blocks along X and three along Z,
        // in a region starting at (8, 0, 16).
        let mesh = ChunkMesh {
            quads: vec![Quad {
                origin: [10, 5, 20],
                axis: Axis::Y,
                facing: Facing::Positive,
                width: 2,
                height: 3,
                surface: SurfaceId(1),
            }],
        };
        let bytes = chunk_vertices(&mesh, BlockPos::new(8, 0, 16)).unwrap();
        let vertices = floats(&bytes);
        assert_eq!(vertices.len(), VERTICES_PER_QUAD as usize);
        for (p, color) in &vertices {
            assert_eq!(p[1], 6.0, "a positive face lies on origin + 1");
            assert!((2.0..=4.0).contains(&p[0]) && (4.0..=7.0).contains(&p[2]));
            assert_eq!(*color, face_color(Axis::Y, Facing::Positive));
        }
        // The two triangles cover the rectangle: all four corners appear.
        for corner in [
            [2.0, 6.0, 4.0],
            [4.0, 6.0, 4.0],
            [4.0, 6.0, 7.0],
            [2.0, 6.0, 7.0],
        ] {
            assert!(vertices.iter().any(|(p, _)| *p == corner), "{corner:?}");
        }
    }

    #[test]
    fn a_negative_face_lies_on_its_cell_s_own_plane() {
        let mesh = ChunkMesh {
            quads: vec![Quad {
                origin: [3, 4, 5],
                axis: Axis::X,
                facing: Facing::Negative,
                width: 1,
                height: 1,
                surface: SurfaceId(0),
            }],
        };
        let vertices = floats(&chunk_vertices(&mesh, BlockPos::ORIGIN).unwrap());
        assert!(vertices.iter().all(|(p, _)| p[0] == 3.0));
        // X's other axes are Y then Z: width runs along Y, height along Z.
        assert!(vertices.iter().any(|(p, _)| *p == [3.0, 5.0, 6.0]));
    }

    #[test]
    fn every_facing_has_its_own_colour() {
        let mut seen = Vec::new();
        for axis in Axis::ALL {
            for facing in [Facing::Positive, Facing::Negative] {
                let color = face_color(axis, facing);
                assert!(!seen.contains(&color));
                assert_ne!(color, CLEAR_COLOR);
                seen.push(color);
            }
        }
    }

    #[test]
    fn a_mesh_from_another_region_is_refused() {
        let mesh = ChunkMesh {
            quads: vec![Quad {
                origin: [EXACT_OFFSET, 0, 0],
                axis: Axis::Z,
                facing: Facing::Positive,
                width: 1,
                height: 1,
                surface: SurfaceId(0),
            }],
        };
        assert!(chunk_vertices(&mesh, BlockPos::ORIGIN).is_err());
    }
}
