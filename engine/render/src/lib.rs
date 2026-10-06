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
//! sorting come after this pass exists. So do index buffers: ADR-0028 left
//! them until a renderer measured that vertices matter, and ADR-0033 measured
//! splitting quads at every corner at +52% vertices, a frame still faster
//! on lavapipe because back faces are culled.

pub mod atlas;
pub mod reference;

use nexora_camera::{CameraState, EXACT_OFFSET};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{Axis, BlockPos};
use nexora_mesh::{ChunkMesh, Extent, Facing};
use nexora_rhi::{
    Binding, BindingKind, BufferDesc, BufferHandle, ClearValue, Command, CommandList, Compare,
    Cull, DepthState, PipelineDesc, PipelineHandle, Rhi, ShaderStage, TextureFormat, TextureHandle,
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

/// The textured pass's shaders, in WGSL (ADR-0037): the rule of
/// [`atlas::texel_of`], [`atlas::FACE_SHADE`] and [`atlas::shade_texel`],
/// transcribed.
///
/// Built from the constants it transcribes, so the factors and the tile
/// geometry cannot drift from the reference. Slots 0 and 1 are the camera and
/// the chunk, as in [`CHUNK_WGSL`]; slot 2 is the atlas, read by
/// `textureLoad` at integer coordinates, with no sampler.
#[must_use]
pub fn textured_wgsl() -> String {
    let shade = atlas::FACE_SHADE
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    let tile = atlas::TILE;
    let columns = atlas::ATLAS_COLUMNS;
    format!(
        r"
struct Camera {{
    view_projection: mat4x4<f32>,
}};

struct Chunk {{
    offset: vec4<f32>,
}};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<uniform> chunk: Chunk;
@group(0) @binding(2) var atlas: texture_2d<f32>;

struct Shaded {{
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec3<f32>,
    @location(1) @interpolate(flat) packed: u32,
}};

@vertex
fn vs_textured(@location(0) position: vec3<f32>, @location(1) packed: u32) -> Shaded {{
    var out: Shaded;
    out.position = camera.view_projection * vec4<f32>(position + chunk.offset.xyz, 1.0);
    out.local = position;
    out.packed = packed;
    return out;
}}

fn texel(value: f32) -> u32 {{
    return min(u32(floor(fract(value) * {tile}.0)), {last}u);
}}

@fragment
fn fs_textured(in: Shaded) -> @location(0) vec4<f32> {{
    let tile = in.packed & 0xffffu;
    let face = (in.packed >> 16u) & 0xffu;
    let axis = face / 2u;
    var s = in.local.x;
    var t = in.local.y;
    if (axis == 0u) {{
        s = in.local.z;
    }} else if (axis == 1u) {{
        t = in.local.z;
    }}
    let x = texel(s);
    var y = texel(t);
    if (axis != 1u) {{
        y = {last}u - y;
    }}
    let origin = vec2<u32>((tile % {columns}u) * {tile}u, (tile / {columns}u) * {tile}u);
    let albedo = textureLoad(atlas, vec2<i32>(origin + vec2<u32>(x, y)), 0);
    var shade = array<f32, 6>({shade});
    return vec4<f32>(albedo.rgb * shade[face], 1.0);
}}
",
        last = tile - 1,
    )
}

/// Bytes per vertex: a position of three `f32`, then a colour of four `u8`.
pub const VERTEX_STRIDE: u32 = 16;

/// Vertices of a quad no corner splits: two triangles, no index buffer
/// (ADR-0028). A split quad has more (ADR-0033).
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

/// Every quad corner of a set of meshes, indexed by the axis-aligned lines it
/// lies on: what [`chunk_vertices`] needs to leave no T-junction (DEBT-0047).
///
/// Greedy meshing (ADR-0012) merges faces into rectangles of different
/// sizes, so a corner of one rectangle can lie in the middle of a
/// neighbour's edge. Rasterisation is watertight only along edges whose
/// endpoints are the same vertices: at such a T-junction the neighbour's long
/// edge and the two short ones are rounded to the pixel grid separately, and
/// a gap a fraction of a pixel wide shows what is behind. Splitting every
/// edge at every corner that lies on it makes each shared edge the same
/// vertices on both sides. The corners of neighbouring regions' meshes must
/// be in the set too, or the seams between chunks keep their T-junctions.
#[derive(Debug, Clone, Default)]
pub struct Corners {
    /// `(axis, the two other coordinates)` → sorted positions along `axis`.
    lines: std::collections::HashMap<(usize, i64, i64), Vec<i64>>,
}

impl Corners {
    /// No corners: every quad stays two triangles.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// The corners of every quad of `meshes`, in world blocks.
    #[must_use]
    pub fn of(meshes: &[&ChunkMesh]) -> Self {
        let mut corners = Self::default();
        for quad in meshes.iter().flat_map(|mesh| &mesh.quads) {
            for point in quad_corners(quad) {
                for axis in 0..3 {
                    let key = line_key(axis, point);
                    corners.lines.entry(key).or_default().push(point[axis]);
                }
            }
        }
        for line in corners.lines.values_mut() {
            line.sort_unstable();
            line.dedup();
        }
        corners
    }

    /// The corners of the quads of `meshes` that touch `extent`'s closed
    /// box: every corner [`chunk_vertices`] can need for a mesh of `extent`.
    ///
    /// A quad's edges lie inside its own region's closed box, and a corner
    /// splits an edge only by lying on it; so a corner can split an edge of
    /// a region only if it lies in that region's closed box, and its quad
    /// then touches the box. For a mesh of `extent`, these corners give the
    /// same vertices as [`Corners::of`] over every mesh drawn with it, from
    /// the region's own quads and the few of its neighbours' that meet its
    /// boundary, rather than from every quad of every neighbour.
    #[must_use]
    pub fn touching(meshes: &[&ChunkMesh], extent: Extent) -> Self {
        let touching: Vec<nexora_mesh::Quad> = meshes
            .iter()
            .flat_map(|mesh| &mesh.quads)
            .filter(|quad| quad_touches(quad, extent))
            .copied()
            .collect();
        Self::of(&[&ChunkMesh { quads: touching }])
    }

    /// Corners strictly between `from` and `to`, which differ only along
    /// `axis`, in order from `from`.
    fn between(&self, axis: usize, from: [i64; 3], to: [i64; 3]) -> Vec<[i64; 3]> {
        let Some(line) = self.lines.get(&line_key(axis, from)) else {
            return Vec::new();
        };
        let (low, high) = (from[axis].min(to[axis]), from[axis].max(to[axis]));
        let start = line.partition_point(|value| *value <= low);
        let end = line.partition_point(|value| *value < high);
        let mut points: Vec<[i64; 3]> = line[start..end]
            .iter()
            .map(|value| {
                let mut point = from;
                point[axis] = *value;
                point
            })
            .collect();
        if from[axis] > to[axis] {
            points.reverse();
        }
        points
    }
}

/// The key of the line along `axis` through `point`.
fn line_key(axis: usize, point: [i64; 3]) -> (usize, i64, i64) {
    let [a, b] = Axis::ALL[axis].others().map(Axis::index);
    (axis, point[a], point[b])
}

/// Whether a quad's rectangle meets `extent`'s closed box: shares at least a
/// point with it, its boundary included.
#[must_use]
pub fn quad_touches(quad: &nexora_mesh::Quad, extent: Extent) -> bool {
    let corners = quad_corners(quad);
    let low = [extent.origin.x, extent.origin.y, extent.origin.z];
    (0..3).all(|axis| {
        let min = corners.iter().map(|c| c[axis]).min().unwrap_or(i64::MAX);
        let max = corners.iter().map(|c| c[axis]).max().unwrap_or(i64::MIN);
        min <= low[axis] + i64::from(extent.size[axis]) && max >= low[axis]
    })
}

/// The corners of a mesh's quads that lie in `extent`'s closed box, sorted
/// and without repeats: what of the mesh a neighbour across that box's
/// boundary can be split at.
#[must_use]
pub fn corners_in(mesh: &ChunkMesh, extent: Extent) -> Vec<[i64; 3]> {
    let low = [extent.origin.x, extent.origin.y, extent.origin.z];
    let inside = |point: &[i64; 3]| {
        (0..3).all(|axis| {
            point[axis] >= low[axis] && point[axis] <= low[axis] + i64::from(extent.size[axis])
        })
    };
    let mut points: Vec<[i64; 3]> = mesh
        .quads
        .iter()
        .flat_map(quad_corners)
        .filter(inside)
        .collect();
    points.sort_unstable();
    points.dedup();
    points
}

/// A quad's four corners in world blocks, in the order its triangles walk
/// them: `(0, 0)`, `(w, 0)`, `(w, h)`, `(0, h)`.
fn quad_corners(quad: &nexora_mesh::Quad) -> [[i64; 3]; 4] {
    let along = quad.axis.index();
    let [first, second] = quad.axis.others().map(Axis::index);
    let mut base = quad.origin;
    if quad.facing == Facing::Positive {
        base[along] += 1;
    }
    let at = |u: u32, v: u32| {
        let mut p = base;
        p[first] += i64::from(u);
        p[second] += i64::from(v);
        p
    };
    let (w, h) = (quad.width, quad.height);
    [at(0, 0), at(w, 0), at(w, h), at(0, h)]
}

/// Whether a quad's corners in the order `(0, 0)`, `(w, 0)`, `(w, h)`,
/// `(0, h)` run counter-clockwise seen from the side the face points to.
///
/// That order turns from the first of the axis's other two axes to the
/// second (`Axis::others`), which is counter-clockwise about their cross
/// product: `Y × Z = +X`, `X × Z = −Y`, `X × Y = +Z`.
const fn winds_outward(axis: Axis, facing: Facing) -> bool {
    let cross_is_positive = !matches!(axis, Axis::Y);
    matches!(facing, Facing::Positive) == cross_is_positive
}

/// A mesh's vertex bytes, relative to `region_min`.
///
/// A quad of cell `origin` facing `Positive` along an axis lies on the plane
/// `origin + 1`, one facing `Negative` on the plane `origin`, and spans
/// `width` blocks along the first of the axis's other two axes and `height`
/// along the second (`Axis::others`), as the mesher emits them.
///
/// A quad with no other corner on its edges is two triangles. One with
/// corners of `corners` on its edges is a fan through every one of them, so
/// that no edge it shares ends in the middle of another (see [`Corners`]):
/// from a corner whose two sides have none, in `n - 2` triangles for `n`
/// boundary vertices, or from the centre, in `n`. Every triangle runs
/// counter-clockwise seen from the side the face points to, the front the
/// pass keeps when it culls back faces.
///
/// # Errors
///
/// A quad is so far from `region_min` that its corners are not exact in
/// `f32`: the mesh does not belong to this region.
pub fn chunk_vertices(
    mesh: &ChunkMesh,
    region_min: BlockPos,
    corners: &Corners,
) -> Result<Vec<u8>> {
    chunk_vertices_with(mesh, region_min, corners, &|quad| {
        face_color(quad.axis, quad.facing)
    })
}

/// The word a textured vertex carries in place of a colour: the quad's tile
/// in `atlas` in the low 16 bits and its face index (`atlas::face_index`) in
/// the next 8, little-endian — what [`textured_wgsl`] reads as one `u32`.
#[must_use]
pub fn textured_word(quad: &nexora_mesh::Quad, atlas: &atlas::Atlas) -> [u8; 4] {
    let tile = atlas.tile_of(quad.surface).min(0xFFFF);
    let face = atlas::face_index(quad.axis, quad.facing);
    (tile | (face << 16)).to_le_bytes()
}

/// [`chunk_vertices`] for the textured pass: every vertex carries its quad's
/// [`textured_word`] instead of its face's colour. The geometry is the same
/// bytes, vertex for vertex.
///
/// # Errors
///
/// See [`chunk_vertices`].
pub fn textured_vertices(
    mesh: &ChunkMesh,
    region_min: BlockPos,
    corners: &Corners,
    atlas: &atlas::Atlas,
) -> Result<Vec<u8>> {
    chunk_vertices_with(mesh, region_min, corners, &|quad| {
        textured_word(quad, atlas)
    })
}

/// The vertices of a mesh, each carrying `word(quad)` after its position.
fn chunk_vertices_with(
    mesh: &ChunkMesh,
    region_min: BlockPos,
    corners: &Corners,
    word: &dyn Fn(&nexora_mesh::Quad) -> [u8; 4],
) -> Result<Vec<u8>> {
    let base = [region_min.x, region_min.y, region_min.z];
    let mut out = Vec::with_capacity(mesh.len() * (VERTICES_PER_QUAD * VERTEX_STRIDE) as usize);
    for quad in &mesh.quads {
        let world = quad_corners(quad);
        let local = world.map(|p| [0, 1, 2].map(|axis| p[axis] - base[axis]));
        let far = local.iter().flatten().copied().max().unwrap_or(0);
        let near = local.iter().flatten().copied().min().unwrap_or(0);
        if far >= EXACT_OFFSET || near <= -EXACT_OFFSET {
            return Err(wrong("a quad is too far from its region to place exactly")
                .with_context("origin", format!("{:?}", quad.origin))
                .with_context("region_min", format!("{region_min:?}")));
        }
        let color = word(quad);
        let flip = !winds_outward(quad.axis, quad.facing);
        let mut pushed = 0usize;
        let mut triangle = [[0.0f64; 3]; 3];
        let mut push = |p: [f64; 3]| {
            triangle[pushed % 3] = p;
            pushed += 1;
            if pushed % 3 != 0 {
                return;
            }
            // Counter-clockwise seen from outside the face: the front the
            // pass keeps when it culls (`Cull::Back`, DEBT-0047).
            let order = if flip { [0, 2, 1] } else { [0, 1, 2] };
            for index in order {
                for value in triangle[index] {
                    out.extend_from_slice(&(value as f32).to_le_bytes());
                }
                out.extend_from_slice(&color);
            }
        };
        let local_f = |p: [i64; 3]| [0, 1, 2].map(|axis| (p[axis] - base[axis]) as f64);

        // The boundary, corner to corner, with every corner on each edge,
        // and where each of the four corners sits in it.
        let mut boundary = Vec::with_capacity(4);
        let mut at_corner = [0usize; 4];
        for side in 0..4 {
            let (from, to) = (world[side], world[(side + 1) % 4]);
            let axis = (0..3).find(|axis| from[*axis] != to[*axis]).unwrap_or(0);
            at_corner[side] = boundary.len();
            boundary.push(from);
            boundary.extend(corners.between(axis, from, to));
        }
        let n = boundary.len();
        if n == 4 {
            for index in [0, 1, 2, 0, 2, 3] {
                push(local_f(world[index]));
            }
            continue;
        }
        // A fan from a corner whose two sides have no corner on them covers
        // the quad in n - 2 triangles, and keeps every edge's vertices.
        let plain_sides = |corner: usize| {
            let next = at_corner[(corner + 1) % 4];
            let own = at_corner[corner];
            let before = at_corner[(corner + 3) % 4];
            let side_after = (next + n - own) % n == 1;
            let side_before = (own + n - before) % n == 1;
            side_after && side_before
        };
        if let Some(corner) = (0..4).find(|corner| plain_sides(*corner)) {
            let apex = at_corner[corner];
            for step in 1..n - 1 {
                push(local_f(boundary[apex]));
                push(local_f(boundary[(apex + step) % n]));
                push(local_f(boundary[(apex + step + 1) % n]));
            }
            continue;
        }
        // Otherwise a fan from the centre: n triangles. Half-integer, so
        // exact in `f32` within `EXACT_OFFSET`.
        let centre =
            [0, 1, 2].map(|axis| (local_f(world[0])[axis] + local_f(world[2])[axis]) / 2.0);
        for index in 0..n {
            push(centre);
            push(local_f(boundary[index]));
            push(local_f(boundary[(index + 1) % n]));
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

/// The pass: one pipeline, one camera uniform, and the atlas when it draws
/// albedo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPass {
    pipeline: PipelineHandle,
    camera: BufferHandle,
    /// The albedo atlas the textured pass reads; `None` for the pass that
    /// colours faces by direction.
    atlas: Option<TextureHandle>,
}

impl ChunkPass {
    /// Create the pipeline, drawing into `target` format with a reverse-Z
    /// depth test (ADR-0029), and the camera's uniform buffer.
    ///
    /// # Errors
    ///
    /// The backend refused the pipeline or the buffer.
    pub fn new<R: Rhi + ?Sized>(rhi: &mut R, target: TextureFormat) -> Result<Self> {
        Self::build(rhi, target, None)
    }

    /// The textured pass (ADR-0037): the same geometry, drawing each face's
    /// albedo from `atlas` — an [`atlas::Atlas`] uploaded by
    /// [`atlas::Atlas::upload`] — lit by [`atlas::FACE_SHADE`]. Its chunks
    /// carry [`textured_vertices`], and `target` should be
    /// [`TextureFormat::Rgba8UnormSrgb`] for the colours the reference
    /// expects.
    ///
    /// # Errors
    ///
    /// The backend refused the pipeline or the buffer.
    pub fn textured<R: Rhi + ?Sized>(
        rhi: &mut R,
        target: TextureFormat,
        atlas: TextureHandle,
    ) -> Result<Self> {
        Self::build(rhi, target, Some(atlas))
    }

    /// Whether this pass draws albedo.
    #[must_use]
    pub const fn is_textured(&self) -> bool {
        self.atlas.is_some()
    }

    fn build<R: Rhi + ?Sized>(
        rhi: &mut R,
        target: TextureFormat,
        atlas: Option<TextureHandle>,
    ) -> Result<Self> {
        let (code, vertex, fragment, word, bindings) = match atlas {
            None => (
                CHUNK_WGSL.to_owned(),
                "vs_chunk",
                "fs_chunk",
                VertexFormat::Unorm8x4,
                vec![BindingKind::Uniform, BindingKind::Uniform],
            ),
            Some(_) => (
                textured_wgsl(),
                "vs_textured",
                "fs_textured",
                VertexFormat::Uint32,
                vec![
                    BindingKind::Uniform,
                    BindingKind::Uniform,
                    BindingKind::Texture,
                ],
            ),
        };
        let stage = |entry: &str| ShaderStage {
            entry: entry.into(),
            code: code.as_bytes().to_vec(),
        };
        let pipeline = rhi.create_pipeline(&PipelineDesc {
            label: if atlas.is_some() {
                "textured chunk pass".into()
            } else {
                "chunk pass".into()
            },
            vertex: stage(vertex),
            fragment: stage(fragment),
            vertex_stride: VERTEX_STRIDE,
            attributes: vec![
                VertexAttribute {
                    location: 0,
                    format: VertexFormat::Float32x3,
                    offset: 0,
                },
                VertexAttribute {
                    location: 1,
                    format: word,
                    offset: 12,
                },
            ],
            bindings,
            depth: Some(DepthState {
                compare: Compare::Greater,
                write: true,
            }),

            cull: Cull::Back,
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
        Ok(Self {
            pipeline,
            camera,
            atlas,
        })
    }

    /// Create a chunk's buffers, and append the write of its vertices to
    /// `list`: an upload rides in the same submission as the frame that
    /// first needs it. `corners` holds the corners of every mesh drawn with
    /// this one, its own included, so shared edges leave no gap
    /// ([`Corners`]).
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
        corners: &Corners,
    ) -> Result<GpuChunk> {
        let bytes = chunk_vertices(mesh, region.origin, corners)?;
        self.upload_vertices(rhi, list, bytes, region)
    }

    /// [`ChunkPass::upload`], for vertex bytes already made by
    /// [`chunk_vertices`] for `region`.
    ///
    /// # Errors
    ///
    /// The bytes are not whole vertices, there are more than one draw holds,
    /// or the backend refused a buffer.
    pub fn upload_vertices<R: Rhi + ?Sized>(
        &self,
        rhi: &mut R,
        list: &mut CommandList,
        bytes: Vec<u8>,
        region: Extent,
    ) -> Result<GpuChunk> {
        if bytes.len() % VERTEX_STRIDE as usize != 0 {
            return Err(wrong("vertex bytes are not a whole number of vertices"));
        }
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
                bindings: match self.atlas {
                    None => vec![
                        Binding::Uniform(self.camera),
                        Binding::Uniform(chunk.offset),
                    ],
                    Some(atlas) => vec![
                        Binding::Uniform(self.camera),
                        Binding::Uniform(chunk.offset),
                        Binding::Texture(atlas),
                    ],
                },
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
        let bytes = chunk_vertices(&mesh, BlockPos::new(8, 0, 16), &Corners::none()).unwrap();
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
        let vertices = floats(&chunk_vertices(&mesh, BlockPos::ORIGIN, &Corners::none()).unwrap());
        assert!(vertices.iter().all(|(p, _)| p[0] == 3.0));
        // X's other axes are Y then Z: width runs along Y, height along Z.
        assert!(vertices.iter().any(|(p, _)| *p == [3.0, 5.0, 6.0]));
    }

    fn quad(origin: [i64; 3], axis: Axis, facing: Facing, width: u32, height: u32) -> Quad {
        Quad {
            origin,
            axis,
            facing,
            width,
            height,
            surface: SurfaceId(0),
        }
    }

    fn triangles(bytes: &[u8]) -> Vec<[[f32; 3]; 3]> {
        floats(bytes)
            .chunks_exact(3)
            .map(|t| [t[0].0, t[1].0, t[2].0])
            .collect()
    }

    /// Twice the signed area of a triangle projected on the plane normal to
    /// `axis`, positive when counter-clockwise seen from `+axis`.
    fn signed_area(t: [[f32; 3]; 3], axis: Axis) -> f32 {
        let [a, b] = axis.others().map(Axis::index);
        let (p, q, r) = (t[0], t[1], t[2]);
        let cross = (q[a] - p[a]) * (r[b] - p[b]) - (q[b] - p[b]) * (r[a] - p[a]);
        // (a, b) is (Y, Z) for X, (X, Z) for Y and (X, Y) for Z: only Y's
        // pair turns the other way about its axis.
        if axis == Axis::Y {
            -cross
        } else {
            cross
        }
    }

    /// Every face, of every facing, is counter-clockwise seen from the side
    /// it points to: the front a pass that culls back faces keeps.
    #[test]
    fn every_triangle_is_wound_counter_clockwise_from_outside() {
        for axis in Axis::ALL {
            for facing in [Facing::Positive, Facing::Negative] {
                let mesh = ChunkMesh {
                    quads: vec![quad([0, 0, 0], axis, facing, 2, 3)],
                };
                let bytes = chunk_vertices(&mesh, BlockPos::ORIGIN, &Corners::none()).unwrap();
                for t in triangles(&bytes) {
                    let area = signed_area(t, axis);
                    let outward = if facing == Facing::Positive {
                        area
                    } else {
                        -area
                    };
                    assert!(outward > 0.0, "{axis:?} {facing:?}: {t:?}");
                }
            }
        }
    }

    /// A 1x1 face beside a 2x1 one on the same plane puts a corner in the
    /// middle of the long one's edge: a T-junction. With the corners given,
    /// the long face's triangles end at that corner, cover the same area,
    /// and keep their winding.
    #[test]
    fn a_corner_in_the_middle_of_an_edge_splits_it() {
        let long = quad([0, 5, 0], Axis::Y, Facing::Positive, 2, 1);
        let short = quad([1, 5, 1], Axis::Y, Facing::Positive, 1, 1);
        let mesh = ChunkMesh {
            quads: vec![long, short],
        };
        let corners = Corners::of(&[&mesh]);
        let only_long = ChunkMesh { quads: vec![long] };
        let split = triangles(&chunk_vertices(&only_long, BlockPos::ORIGIN, &corners).unwrap());
        let plain =
            triangles(&chunk_vertices(&only_long, BlockPos::ORIGIN, &Corners::none()).unwrap());
        assert_eq!(plain.len(), 2);
        assert_eq!(
            split.len(),
            3,
            "a fan from a plain corner: five vertices, three triangles"
        );
        assert!(
            split.iter().flatten().any(|p| *p == [1.0, 6.0, 1.0]),
            "the neighbour's corner is a vertex of the long face"
        );
        let area = |ts: &[[[f32; 3]; 3]]| ts.iter().map(|t| signed_area(*t, Axis::Y)).sum::<f32>();
        assert_eq!(area(&split), area(&plain));
        assert!(split.iter().all(|t| signed_area(*t, Axis::Y) > 0.0));
    }

    /// A corner of another region's mesh, on the seam, splits the edge too:
    /// the corners are world blocks, not region-relative ones.
    #[test]
    fn a_corner_across_a_region_seam_splits_the_edge() {
        let here = ChunkMesh {
            quads: vec![quad([0, 0, 0], Axis::Z, Facing::Positive, 4, 1)],
        };
        let there = ChunkMesh {
            quads: vec![quad([2, 1, 0], Axis::Z, Facing::Positive, 1, 1)],
        };
        let corners = Corners::of(&[&here, &there]);
        let region_min = BlockPos::new(-16, 0, -16);
        let split = triangles(&chunk_vertices(&here, region_min, &corners).unwrap());
        // (2, 1, 1) in the world is (18, 1, 17) relative to the region.
        assert!(
            split.iter().flatten().any(|p| *p == [18.0, 1.0, 17.0]),
            "{split:?}"
        );
    }

    /// Corners on both sides and at every corner of a face leave no corner
    /// to fan from: the fan starts at the centre.
    #[test]
    fn a_face_split_on_every_side_fans_from_its_centre() {
        let big = quad([0, 0, 0], Axis::X, Facing::Positive, 2, 2);
        let mut quads = vec![big];
        // Neighbours whose corners land mid-edge on all four sides.
        for (y, z) in [(1, -1), (2, 1), (1, 2), (-1, 1)] {
            quads.push(quad([0, y, z], Axis::X, Facing::Positive, 1, 1));
        }
        let mesh = ChunkMesh { quads };
        let corners = Corners::of(&[&mesh]);
        let only = ChunkMesh { quads: vec![big] };
        let split = triangles(&chunk_vertices(&only, BlockPos::ORIGIN, &corners).unwrap());
        assert_eq!(split.len(), 8, "eight boundary vertices, eight triangles");
        assert!(
            split.iter().all(|t| t[0] == [1.0, 1.0, 1.0]),
            "all from the centre"
        );
        assert!(split.iter().all(|t| signed_area(*t, Axis::X) > 0.0));
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
        assert!(chunk_vertices(&mesh, BlockPos::ORIGIN, &Corners::none()).is_err());
    }
}
