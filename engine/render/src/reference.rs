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
use nexora_mesh::{Extent, Facing, VoxelView};

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
    let Projection::Perspective { fov_y, far, .. } = camera.projection() else {
        return Err(Error::new(
            Domain::Render,
            "render-reference",
            "the reference traces perspective cameras only",
        )
        .with_recovery(Recovery::Reject));
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
    let d = [0, 1, 2].map(|i| f[i] + x * r[i] + y * u[i]);
    // Along d, the far plane is `far` away along f; d·f = 1.
    let reach = far;

    // Walk relative to the camera's own block: exact at any distance from
    // the world's origin.
    let eye = camera.position();
    let home = eye.to_block_pos();
    let start = [
        eye.x - home.x as f64,
        eye.y - home.y as f64,
        eye.z - home.z as f64,
    ];
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
            return Ok(Hit::Clear);
        }
        t_max[i] += t_delta[i];
        let from = BlockPos::new(home.x + cell[0], home.y + cell[1], home.z + cell[2]);
        cell[i] += step[i];
        let to = BlockPos::new(home.x + cell[0], home.y + cell[1], home.z + cell[2]);
        if !inside(to) || view.surface_at(to).is_none() {
            // Air, or another chunk: nothing this frame draws.
            continue;
        }
        // The face between `from` and `to` exists when `from` does not
        // occlude it, whether `from` is in the region or a neighbour's cell.
        if view.occludes(from) {
            return Ok(Hit::Outside);
        }
        // Moving up an axis, a ray enters a cell through its negative face.
        let facing = if step[i] > 0 {
            Facing::Negative
        } else {
            Facing::Positive
        };
        return Ok(Hit::Face(Axis::ALL[i], facing));
    }
}

/// Check a frame of RGBA8 texels, row by row from the top, against the
/// reference.
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
    };
    for py in 0..height {
        for px in 0..width {
            let (cx, cy) = (f64::from(px) + 0.5, f64::from(py) + 0.5);
            let centre = trace(view, region, camera, viewport, cx, cy)?;
            let Some(want) = centre.color() else { continue };
            let mut agrees = true;
            for (dx, dy) in [(-0.35, -0.35), (0.35, -0.35), (-0.35, 0.35), (0.35, 0.35)] {
                if trace(view, region, camera, viewport, cx + dx, cy + dy)? != centre {
                    agrees = false;
                    break;
                }
            }
            if !agrees {
                continue;
            }
            check.judged += 1;
            let index = (py as usize * width as usize + px as usize) * 4;
            if texels[index..index + 4] == want {
                check.matching += 1;
            }
        }
    }
    Ok(check)
}
