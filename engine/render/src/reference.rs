//! What a frame must show, computed without the GPU.
//!
//! A ray per pixel, walked through the voxel grid cell by cell (Amanatides
//! and Woo) to the first face of the drawn region it meets. That face names
//! the colour the pixel must have. Like the frame, it sees only the region:
//! cells outside it are other chunks, which this frame does not draw. None of this shares code with the
//! path it checks: not the mesher, not the matrices, not the rasteriser, not
//! the depth test. That is what lets it catch a mistake in any of them.
//!
//! It is a reference, not a renderer: a few hundred nanoseconds a ray on a
//! CPU, used to check a frame before a benchmark times it and in tests.

use nexora_camera::{Camera, Projection};
use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{Axis, BlockPos};
use nexora_mesh::{Extent, Facing, SurfaceId, VoxelView};

use crate::atlas::{face_index, shade_texel, srgb_to_linear, texel_of, Atlas, FACE_SHADE};
use crate::{face_color, CLEAR_COLOR};

/// What a ray through a pixel reaches first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    /// Nothing within the far plane: the pixel keeps the clear colour.
    Clear,
    /// A face of a cell inside the drawn region.
    Face(Axis, Facing),
    /// The ray enters the region through a boundary face the mesher culled,
    /// because the neighbouring chunk occludes it (ADR-0012). What shows
    /// there depends on that chunk, which this frame does not draw, so the
    /// pixel is not judged.
    Outside,
}

impl Hit {
    /// The colour the pass draws for this hit, or `None` when it is not
    /// judged.
    #[must_use]
    pub const fn color(self) -> Option<[u8; 4]> {
        match self {
            Self::Clear => Some(CLEAR_COLOR),
            Self::Face(axis, facing) => Some(face_color(axis, facing)),
            Self::Outside => None,
        }
    }
}

/// A frame against the reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameCheck {
    /// Pixels in the frame.
    pub pixels: usize,
    /// Pixels judged: the ray through the centre and four rays around it
    /// agree, and none of them reaches another chunk.
    pub judged: usize,
    /// Judged pixels whose colour is the one the reference expects.
    pub matching: usize,
    /// Judged pixels that do not match and show a face turned away from the
    /// ray through them.
    ///
    /// No correct frame shows one: every face the mesher emits separates a
    /// solid cell from an empty one, so a face seen from behind is seen from
    /// inside a solid. Before the pass culled back faces and split its quads
    /// at every corner, this is what a ray showed when it slipped through a
    /// sub-pixel gap at a T-junction, or when a silhouette edge's tie went to
    /// the back face (DEBT-0047). Now the pass culls them, so any is an
    /// error; it is counted apart so that the error says what it is.
    pub backfacing: usize,
    /// Judged pixels that do not match, are not back faces, and show a
    /// colour the reference itself finds within [`SNAP`] of the pixel's
    /// centre: an edge that passes that close to the centre, and that the
    /// rasteriser placed on the other side of it.
    pub snapped: usize,
    /// Judged pixels of a textured frame that do not match and are explained
    /// only because the frame is textured: an edge between faces turned the
    /// same way, which a frame of face colours would not show; a quad read
    /// past its own edge, where the fraction wraps to the far side of its
    /// tile; or a texel boundary within [`TEXEL_SNAP`] of the centre, or
    /// within [`TEXEL_EPSILON`] of the exact point (ADR-0037, DEBT-0053).
    ///
    /// Counted apart from [`Self::snapped`], which keeps meaning what it
    /// means in a frame of face colours: neither rule has a counterpart
    /// there, a textured face has a texel boundary every sixteenth of a
    /// block, and how many land that close to a centre depends on how the
    /// device interpolates.
    pub texel_snapped: usize,
}

/// How close to a pixel's centre an edge must pass for a mismatch there to
/// count as [`FrameCheck::snapped`]: a sixty-fourth of a pixel. Vulkan
/// guarantees at least four bits of sub-pixel precision (a sixteenth), and
/// lavapipe uses eight.
pub const SNAP: f64 = 1.0 / 64.0;

/// How far from a pixel's centre a textured quad may have evaluated its
/// position, for a texel boundary passing that close to count as
/// [`FrameCheck::snapped`] (ADR-0037): a sixteenth of a pixel, the coarsest
/// sub-pixel grid Vulkan allows its vertices to be snapped to, and so the
/// furthest the position interpolated between them may move. Geometry edges
/// keep [`SNAP`]; only a texel boundary inside a quad's tile, or the edge of
/// that tile, gets this. The worst measured on lavapipe over twenty seeds was
/// 0.04 of a pixel.
pub const TEXEL_SNAP: f64 = 1.0 / 16.0;

/// The same allowance in the texture's own units, for faces close enough to
/// the eye that a texel covers tens of pixels: a texel boundary within a
/// 128th of a texel (a 2,048th of a block) of the exact point may fall
/// either way. Close up, the position a clipped, perspective-corrected
/// triangle interpolates is off by a share of the attribute, not of the
/// pixel. The worst measured on lavapipe: a 256th of a texel, on a face half
/// a block from the eye, where a texel spans forty pixels.
pub const TEXEL_EPSILON: f64 = 1.0 / 2048.0;

/// The direction of the ray through the point `(px, py)` of a `width` ×
/// `height` viewport, scaled so that its component along the camera's
/// forward axis is 1.
///
/// # Errors
///
/// The camera is not a perspective camera.
pub fn ray_direction(camera: &Camera, viewport: (u32, u32), px: f64, py: f64) -> Result<[f64; 3]> {
    let Projection::Perspective { fov_y, .. } = camera.projection() else {
        return Err(perspective_only());
    };
    let (width, height) = (f64::from(viewport.0), f64::from(viewport.1));
    let f = camera.forward();
    let length = f[0].hypot(f[2]);
    let r = [-f[2] / length, 0.0, f[0] / length];
    let u = [
        r[1] * f[2] - r[2] * f[1],
        r[2] * f[0] - r[0] * f[2],
        r[0] * f[1] - r[1] * f[0],
    ];
    let tan = (fov_y / 2.0).tan();
    let x = (2.0 * px / width - 1.0) * tan * (width / height);
    let y = (1.0 - 2.0 * py / height) * tan;
    Ok([0, 1, 2].map(|i| f[i] + x * r[i] + y * u[i]))
}

/// The face a colour of the pass names, if it names one.
#[must_use]
pub fn face_of(color: [u8; 4]) -> Option<(Axis, Facing)> {
    Axis::ALL.into_iter().find_map(|axis| {
        [Facing::Positive, Facing::Negative]
            .into_iter()
            .find(|facing| face_color(axis, *facing) == color)
            .map(|facing| (axis, facing))
    })
}

/// Whether a face points away from a ray travelling along `direction`.
#[must_use]
pub fn faces_away(axis: Axis, facing: Facing, direction: [f64; 3]) -> bool {
    let along = direction[axis.index()];
    match facing {
        Facing::Positive => along > 0.0,
        Facing::Negative => along < 0.0,
    }
}

fn perspective_only() -> Error {
    Error::new(
        Domain::Render,
        "render-reference",
        "the reference traces perspective cameras only",
    )
    .with_recovery(Recovery::Reject)
}

/// Trace the ray through the point `(px, py)` of a `width` × `height`
/// viewport, in pixels from the top-left corner.
///
/// # Errors
///
/// The camera is not a perspective camera.
pub fn trace<V: VoxelView + ?Sized>(
    view: &V,
    region: Extent,
    camera: &Camera,
    viewport: (u32, u32),
    px: f64,
    py: f64,
) -> Result<Hit> {
    walk(view, region, camera, viewport, px, py).map(|walked| walked.hit)
}

/// What a ray reached, and where: the hit, the surface of the cell it
/// entered and the point it entered it at, in blocks from the camera's own
/// block (whole blocks off the world's coordinates, which [`texel_of`] does
/// not see).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Walked {
    hit: Hit,
    surface: Option<SurfaceId>,
    point: [f64; 3],
    /// The cell entered, relative to the camera's own block.
    cell: [i64; 3],
}

/// The eye, in blocks from its own block: where every ray [`walk`]s from.
fn eye_in_home_block(camera: &Camera) -> [f64; 3] {
    let eye = camera.position();
    let home = eye.to_block_pos();
    [
        eye.x - home.x as f64,
        eye.y - home.y as f64,
        eye.z - home.z as f64,
    ]
}

/// Whether a pixel that shows the wrong colour is on an edge the rasteriser
/// may have put on the other side: a geometry edge within [`SNAP`] of its
/// centre, or — textured — a quad, the centre's or one across such an edge,
/// whose position interpolated within [`TEXEL_SNAP`] of the centre, on its
/// own plane, reads the texel shown.
#[allow(clippy::too_many_arguments)]
fn snapped_edge<V: VoxelView + ?Sized>(
    view: &V,
    region: Extent,
    camera: &Camera,
    viewport: (u32, u32),
    (cx, cy): (f64, f64),
    centre: Walked,
    shading: &Shading<'_>,
    shown: [u8; 4],
) -> Result<Snapped> {
    let explains = |walked: &Walked| {
        shading
            .expect(walked)
            .is_some_and(|colour| shading.same(shown, colour))
    };
    let around = |radius: f64| {
        (0..8).map(move |k| {
            let angle = f64::from(k) * std::f64::consts::FRAC_PI_4;
            (cx + radius * angle.cos(), cy + radius * angle.sin())
        })
    };
    // An edge a frame of face colours would also show is between faces
    // turned different ways; one between faces turned the same way shows
    // only because their textures differ, and is counted with the texels.
    let kind = |by: &Walked| {
        if by.hit == centre.hit {
            Snapped::Texel
        } else {
            Snapped::Edge
        }
    };
    let mut quads = vec![centre];
    for (x, y) in around(SNAP) {
        let near = walk(view, region, camera, viewport, x, y)?;
        if explains(&near) {
            return Ok(kind(&near));
        }
        quads.push(near);
    }
    // A block whose face the centre ray meets within SNAP of an edge may
    // lose the pixel to whatever is behind it — and when the ray grazes
    // corners lined up with the eye, what is behind is in a wedge thinner
    // than any ring of samples can find. So the ray is walked on past each
    // such block, a few deep.
    let mut skipped = Vec::new();
    let mut layer = centre;
    for _ in 0..GRAZED_DEPTH {
        let grazed = matches!(layer.hit, Hit::Face(..))
            && around(SNAP).any(|(x, y)| {
                on_plane_of(camera, viewport, x, y, layer)
                    .is_none_or(|on| outside_cell(&on, layer.cell))
            });
        if !grazed {
            break;
        }
        // The block's other faces meet this one at the grazed edge: one
        // turned towards the eye may take the pixel instead.
        for sibling in siblings(view, camera, viewport, (cx, cy), layer) {
            if explains(&sibling) {
                return Ok(kind(&sibling));
            }
            quads.push(sibling);
        }
        skipped.push(layer.cell);
        layer = walk_past(view, region, camera, viewport, cx, cy, &skipped)?;
        if explains(&layer) {
            return Ok(kind(&layer));
        }
        quads.push(layer);
    }
    if matches!(shading, Shading::Faces) {
        return Ok(Snapped::No);
    }
    // The quad that covers the pixel evaluates its position at the centre,
    // past its own edge when it is a neighbour's, and to within TEXEL_SNAP
    // of it: the two textured-only rules (DEBT-0053). Neither has a
    // counterpart in a frame of face colours, so neither is counted with its
    // edges.
    let reads = |x: f64, y: f64| {
        quads
            .iter()
            .any(|&quad| on_plane_of(camera, viewport, x, y, quad).is_some_and(|w| explains(&w)))
    };
    if std::iter::once((cx, cy))
        .chain(around(TEXEL_SNAP))
        .any(|(x, y)| reads(x, y))
    {
        return Ok(Snapped::Texel);
    }
    // And within TEXEL_EPSILON of the exact point, in the plane of the quad
    // the centre ray reached.
    if let Hit::Face(axis, _) = centre.hit {
        let i = axis.index();
        let (a, b) = ((i + 1) % 3, (i + 2) % 3);
        for (da, db) in [
            (1.0, 0.0),
            (-1.0, 0.0),
            (0.0, 1.0),
            (0.0, -1.0),
            (1.0, 1.0),
            (1.0, -1.0),
            (-1.0, 1.0),
            (-1.0, -1.0),
        ] {
            let mut point = centre.point;
            point[a] += da * TEXEL_EPSILON;
            point[b] += db * TEXEL_EPSILON;
            if explains(&Walked { point, ..centre }) {
                return Ok(Snapped::Texel);
            }
        }
    }
    Ok(Snapped::No)
}

/// What explains a pixel that shows the wrong colour, if anything does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Snapped {
    /// Nothing: the pixel is wrong.
    No,
    /// An edge between faces turned different ways, within [`SNAP`] of the
    /// centre ([`FrameCheck::snapped`]).
    Edge,
    /// A textured-only rule ([`FrameCheck::texel_snapped`]).
    Texel,
}

/// How many grazed blocks [`snapped_edge`] looks past along one ray.
const GRAZED_DEPTH: usize = 4;

/// The faces of the block `layer` hit, other than the one it hit, that
/// exist (nothing opaque against them) and face the eye, each where the ray
/// through `(px, py)` meets its plane.
fn siblings<V: VoxelView + ?Sized>(
    view: &V,
    camera: &Camera,
    viewport: (u32, u32),
    (px, py): (f64, f64),
    layer: Walked,
) -> Vec<Walked> {
    let Hit::Face(hit_axis, _) = layer.hit else {
        return Vec::new();
    };
    let Ok(direction) = ray_direction(camera, viewport, px, py) else {
        return Vec::new();
    };
    let home = camera.position().to_block_pos();
    let mut faces = Vec::new();
    for axis in Axis::ALL.into_iter().filter(|&axis| axis != hit_axis) {
        for facing in [Facing::Positive, Facing::Negative] {
            if faces_away(axis, facing, direction) {
                continue;
            }
            let j = axis.index();
            let (out, side) = match facing {
                Facing::Positive => (1, 1),
                Facing::Negative => (-1, 0),
            };
            let mut beyond = layer.cell;
            beyond[j] += out;
            let at = BlockPos::new(home.x + beyond[0], home.y + beyond[1], home.z + beyond[2]);
            if view.occludes(at) {
                continue;
            }
            let mut point = layer.point;
            point[j] = (layer.cell[j] + side) as f64;
            let face = Walked {
                hit: Hit::Face(axis, facing),
                point,
                ..layer
            };
            faces.extend(on_plane_of(camera, viewport, px, py, face));
        }
    }
    faces
}

/// Whether a point on a face of `cell` lies off that cell's face.
fn outside_cell(on: &Walked, cell: [i64; 3]) -> bool {
    let Hit::Face(axis, _) = on.hit else {
        return true;
    };
    (0..3).filter(|&k| k != axis.index()).any(|k| {
        let low = cell[k] as f64;
        on.point[k] < low || on.point[k] > low + 1.0
    })
}

/// What the quad `near` hit shows at the pixel `(px, py)` when the rasteriser
/// gives it that pixel: its position is interpolated at the pixel's centre,
/// which is where the centre ray crosses the quad's *plane* — past the quad's
/// edge, so a textured quad reads its tile from the far side. Two quads in
/// one plane meet the centre ray at the same point; at a depth edge the near
/// quad's plane is not where the centre ray ends.
fn on_plane_of(
    camera: &Camera,
    viewport: (u32, u32),
    px: f64,
    py: f64,
    near: Walked,
) -> Option<Walked> {
    let Hit::Face(axis, _) = near.hit else {
        return None;
    };
    let i = axis.index();
    let d = ray_direction(camera, viewport, px, py).ok()?;
    if d[i] == 0.0 {
        return None;
    }
    let start = eye_in_home_block(camera);
    let t = (near.point[i] - start[i]) / d[i];
    let mut point = [0, 1, 2].map(|k| start[k] + d[k] * t);
    point[i] = near.point[i];
    Some(Walked { point, ..near })
}

fn walk<V: VoxelView + ?Sized>(
    view: &V,
    region: Extent,
    camera: &Camera,
    viewport: (u32, u32),
    px: f64,
    py: f64,
) -> Result<Walked> {
    walk_past(view, region, camera, viewport, px, py, &[])
}

/// [`walk`], as if the cells in `skipped` (relative to the camera's own
/// block) were air.
fn walk_past<V: VoxelView + ?Sized>(
    view: &V,
    region: Extent,
    camera: &Camera,
    viewport: (u32, u32),
    px: f64,
    py: f64,
    skipped: &[[i64; 3]],
) -> Result<Walked> {
    let missed = |hit| Walked {
        hit,
        surface: None,
        point: [0.0; 3],
        cell: [0; 3],
    };
    let Projection::Perspective { far, .. } = camera.projection() else {
        return Err(perspective_only());
    };
    let d = ray_direction(camera, viewport, px, py)?;
    // Along d, the far plane is `far` away along f; d·f = 1.
    let reach = far;

    // Walk relative to the camera's own block: exact at any distance from
    // the world's origin.
    let home = camera.position().to_block_pos();
    let start = eye_in_home_block(camera);
    let mut cell = [0i64; 3];
    let step = d.map(|v| if v > 0.0 { 1i64 } else { -1 });
    let mut t_max = [0, 1, 2].map(|i| {
        if d[i] == 0.0 {
            f64::INFINITY
        } else {
            let boundary = if step[i] > 0 { 1.0 } else { 0.0 };
            (boundary - start[i]) / d[i]
        }
    });
    let t_delta = d.map(|v| {
        if v == 0.0 {
            f64::INFINITY
        } else {
            1.0 / v.abs()
        }
    });
    let inside = |p: BlockPos| {
        let o = region.origin;
        let s = region.size.map(i64::from);
        (o.x..o.x + s[0]).contains(&p.x)
            && (o.y..o.y + s[1]).contains(&p.y)
            && (o.z..o.z + s[2]).contains(&p.z)
    };
    loop {
        let i = if t_max[0] <= t_max[1] && t_max[0] <= t_max[2] {
            0
        } else if t_max[1] <= t_max[2] {
            1
        } else {
            2
        };
        if t_max[i] > reach {
            return Ok(missed(Hit::Clear));
        }
        let entered = t_max[i];
        t_max[i] += t_delta[i];
        let from = BlockPos::new(home.x + cell[0], home.y + cell[1], home.z + cell[2]);
        let from_skipped = skipped.contains(&cell);
        cell[i] += step[i];
        let to = BlockPos::new(home.x + cell[0], home.y + cell[1], home.z + cell[2]);
        if !inside(to) || view.surface_at(to).is_none() || skipped.contains(&cell) {
            // Air, or another chunk: nothing this frame draws.
            continue;
        }
        // The face between `from` and `to` exists when `from` does not
        // occlude it, whether `from` is in the region or a neighbour's cell.
        if !from_skipped && view.occludes(from) {
            return Ok(missed(Hit::Outside));
        }
        // Moving up an axis, a ray enters a cell through its negative face.
        let facing = if step[i] > 0 {
            Facing::Negative
        } else {
            Facing::Positive
        };
        let mut point = [0, 1, 2].map(|axis| start[axis] + d[axis] * entered);
        // On the face's own plane exactly, whatever the rounding said.
        point[i] = (cell[i] + if step[i] > 0 { 0 } else { 1 }) as f64;
        return Ok(Walked {
            hit: Hit::Face(Axis::ALL[i], facing),
            surface: view.surface_at(to),
            point,
            cell,
        });
    }
}

/// How a frame is coloured, so the reference knows what to expect of it.
#[derive(Debug, Clone, Copy)]
pub enum Shading<'a> {
    /// Each face in its direction's colour ([`face_color`]): the first pass,
    /// whose colours name their face.
    Faces,
    /// Each face's albedo from the atlas, lit by [`FACE_SHADE`] (ADR-0037).
    Albedo {
        /// The atlas the pass reads.
        atlas: &'a Atlas,
        /// Whether the texels judged are the sRGB target itself (or a
        /// surface that keeps its encoding), rather than a linear surface it
        /// was presented to, which holds the decoded values.
        encoded: bool,
    },
}

/// How far a channel of a textured frame may be from the reference: the GPU
/// and this module decode and encode sRGB with different arithmetic, and a
/// result exactly between two values may round either way.
pub const ALBEDO_TOLERANCE: u8 = 2;

impl Shading<'_> {
    /// The colour a pixel whose ray did this must show, or `None` when it is
    /// not judged.
    fn expect(&self, walked: &Walked) -> Option<[u8; 4]> {
        match (self, walked.hit) {
            (_, Hit::Outside) => None,
            (Self::Faces, hit) => hit.color(),
            (Self::Albedo { .. }, Hit::Clear) => Some(CLEAR_COLOR),
            (Self::Albedo { atlas, encoded }, Hit::Face(axis, facing)) => {
                let surface = walked.surface?;
                let (x, y) = texel_of(axis, walked.point);
                let face = face_index(axis, facing) as usize;
                let lit = shade_texel(atlas.texel(atlas.tile_of(surface), x, y), FACE_SHADE[face]);
                Some(if *encoded {
                    lit
                } else {
                    let linear = |c: u8| (srgb_to_linear(c) * 255.0).round() as u8;
                    [linear(lit[0]), linear(lit[1]), linear(lit[2]), 255]
                })
            }
        }
    }

    /// Whether a shown colour is the expected one.
    fn same(&self, shown: [u8; 4], want: [u8; 4]) -> bool {
        match self {
            Self::Faces => shown == want,
            Self::Albedo { .. } => shown
                .iter()
                .zip(&want)
                .all(|(a, b)| a.abs_diff(*b) <= ALBEDO_TOLERANCE),
        }
    }
}

/// Check a frame of RGBA8 texels, row by row from the top, against the
/// reference, for the pass that colours faces by direction.
///
/// A pixel is judged when the ray through its centre and four rays 0.35 of
/// a pixel around it agree: near an edge, rasterisation and a grid walk may
/// round to different faces, and neither is wrong.
///
/// # Errors
///
/// The camera is not a perspective camera, or `texels` is not
/// `width × height × 4` bytes.
pub fn check_frame<V: VoxelView + ?Sized>(
    view: &V,
    region: Extent,
    camera: &Camera,
    viewport: (u32, u32),
    texels: &[u8],
) -> Result<FrameCheck> {
    check_frame_shaded(view, region, camera, viewport, texels, Shading::Faces)
}

/// [`check_frame`], for a frame coloured by `shading`.
///
/// With [`Shading::Albedo`] a pixel is judged by the same rule — the five
/// rays agree on the face — and matches when every channel is within
/// [`ALBEDO_TOLERANCE`] of the texel its centre ray reaches, lit. Where a
/// texel's edge, like a face's, passes within [`SNAP`] of the centre, a
/// colour the reference finds there counts as [`FrameCheck::snapped`] — and
/// so does the colour a neighbouring quad shows when the rasteriser gives it
/// the pixel: that quad's tile, read at the pixel's centre, which lies a
/// hair past the quad's edge and so in its tile's opposite column or row. A back
/// face cannot be told by its colour here, so none is counted apart: one on
/// screen is a pixel that does not match.
///
/// # Errors
///
/// See [`check_frame`].
pub fn check_frame_shaded<V: VoxelView + ?Sized>(
    view: &V,
    region: Extent,
    camera: &Camera,
    viewport: (u32, u32),
    texels: &[u8],
    shading: Shading<'_>,
) -> Result<FrameCheck> {
    let (width, height) = viewport;
    let pixels = width as usize * height as usize;
    if texels.len() != pixels * 4 {
        return Err(Error::new(
            Domain::Render,
            "render-reference",
            "a frame's texels do not match its size",
        )
        .with_recovery(Recovery::Reject)
        .with_context("bytes", texels.len().to_string())
        .with_context("pixels", pixels.to_string()));
    }
    let mut check = FrameCheck {
        pixels,
        judged: 0,
        matching: 0,
        backfacing: 0,
        snapped: 0,
        texel_snapped: 0,
    };
    for py in 0..height {
        for px in 0..width {
            let (cx, cy) = (f64::from(px) + 0.5, f64::from(py) + 0.5);
            let centre = walk(view, region, camera, viewport, cx, cy)?;
            let Some(want) = shading.expect(&centre) else {
                continue;
            };
            let mut agrees = true;
            for (dx, dy) in [(-0.35, -0.35), (0.35, -0.35), (-0.35, 0.35), (0.35, 0.35)] {
                if trace(view, region, camera, viewport, cx + dx, cy + dy)? != centre.hit {
                    agrees = false;
                    break;
                }
            }
            if !agrees {
                continue;
            }
            check.judged += 1;
            let index = (py as usize * width as usize + px as usize) * 4;
            let shown: [u8; 4] = [0, 1, 2, 3].map(|i| texels[index + i]);
            if shading.same(shown, want) {
                check.matching += 1;
            } else if matches!(shading, Shading::Faces)
                && face_of(shown).is_some_and(|(axis, facing)| {
                    ray_direction(camera, viewport, cx, cy)
                        .is_ok_and(|direction| faces_away(axis, facing, direction))
                })
            {
                check.backfacing += 1;
            } else {
                match snapped_edge(
                    view,
                    region,
                    camera,
                    viewport,
                    (cx, cy),
                    centre,
                    &shading,
                    shown,
                )? {
                    Snapped::Edge => check.snapped += 1,
                    Snapped::Texel => check.texel_snapped += 1,
                    Snapped::No => {}
                }
            }
        }
    }
    Ok(check)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexora_foundation::spatial::WorldPosition;

    #[test]
    fn every_face_colour_names_its_face_and_the_clear_colour_none() {
        for axis in Axis::ALL {
            for facing in [Facing::Positive, Facing::Negative] {
                assert_eq!(face_of(face_color(axis, facing)), Some((axis, facing)));
            }
        }
        assert_eq!(face_of(CLEAR_COLOR), None);
    }

    /// A face is seen from behind when the ray travels the way it points.
    #[test]
    fn a_face_turned_away_from_the_ray_is_a_back_face() {
        let along_x = [1.0, 0.0, 0.0];
        assert!(faces_away(Axis::X, Facing::Positive, along_x));
        assert!(!faces_away(Axis::X, Facing::Negative, along_x));
        assert!(
            !faces_away(Axis::Y, Facing::Positive, along_x),
            "edge-on is not behind"
        );
    }

    /// The ray through the centre of the viewport is the camera's forward
    /// axis.
    #[test]
    fn the_centre_ray_is_forward() {
        let mut camera = Camera::new(
            WorldPosition::new(0.5, 0.5, 0.5),
            Projection::perspective(1.0, 0.1, 100.0).unwrap(),
        )
        .unwrap();
        camera.set_orientation(0.7, -0.3).unwrap();
        let d = ray_direction(&camera, (200, 100), 100.0, 50.0).unwrap();
        let f = camera.forward();
        for i in 0..3 {
            assert!((d[i] - f[i]).abs() < 1e-12, "{d:?} {f:?}");
        }
    }
}
