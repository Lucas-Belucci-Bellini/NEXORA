//! The camera (RENDER-5, RENDER-6, RENDER-7; ADR-0029).
//!
//! A camera turns an authoritative world position into what a shader reads: a
//! view, a projection and their product, all relative to a floating
//! [`RenderOrigin`], plus the [`Frustum`] a culler asks. It owns no gameplay
//! state (`CAMERA SYSTEM.md`): it is told where to be and where to look, and
//! it answers with a [`CameraState`].
//!
//! # Conventions, fixed here and in ADR-0029
//!
//! | space | convention |
//! | --- | --- |
//! | world | `f64` positions, `i64` blocks; `+Y` up (`nexora_foundation::spatial`) |
//! | render | world minus an integer [`RenderOrigin`], `f32`; same axes |
//! | view | right-handed: `+X` right, `+Y` up, the camera looks down `-Z` |
//! | clip / NDC | `x, y ∈ [-1, 1]`, `+Y` up; **depth `∈ [0, 1]`, reverse-Z**: near is `1`, far is `0` |
//! | matrices | column-major `f32`, built and multiplied in `f64`, rounded once |
//!
//! **Reverse-Z** because the RHI's one depth format is `Depth32Float`. A float
//! is densest near zero, and reverse-Z puts zero at the far plane, where a
//! perspective projection is otherwise starved of precision. The depth test
//! is then the RHI's `Compare::Greater` against a depth cleared to `0`.
//! (This crate does not depend on the RHI; the convention is the link.)
//!
//! # Not built here
//!
//! Target tracking, first- and third-person modes, interpolation between
//! simulation ticks, collision-aware placement and cinematic cameras
//! (`CAMERA SYSTEM.md`) need a player, a clock and a renderer that do not
//! exist yet. This crate is the part every one of them sits on.

pub mod frustum;
pub mod matrix;
pub mod origin;

use core::f64::consts::{FRAC_PI_2, PI};

use nexora_foundation::error::{Domain, Error, Recovery, Result};
use nexora_foundation::spatial::{BlockPos, WorldPosition};

pub use frustum::{Frustum, Plane};
pub use matrix::Mat4;
pub use origin::{RenderOrigin, EXACT_OFFSET, REBASE_DISTANCE};

use matrix::{multiply, Columns};

/// The steepest a camera may look up or down, in radians: 89.9°.
///
/// At exactly 90° the view direction is parallel to the world's up, and the
/// camera's right-hand direction is undefined. Clamped rather than refused:
/// a mouse moved too far is not an error.
pub const MAX_PITCH: f64 = FRAC_PI_2 - 0.1 * PI / 180.0;

/// How the camera maps view space to clip space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    /// Things farther away look smaller.
    Perspective {
        /// Vertical field of view, in radians, strictly between `0` and `π`.
        fov_y: f64,
        /// Distance to the near plane, in blocks; greater than zero.
        near: f64,
        /// Distance to the far plane, in blocks; greater than `near`.
        far: f64,
    },
    /// Parallel: size does not change with distance (maps, editors).
    Orthographic {
        /// Height of the view volume, in blocks.
        height: f64,
        /// Distance to the near plane; zero or more.
        near: f64,
        /// Distance to the far plane; greater than `near`.
        far: f64,
    },
}

impl Projection {
    /// A perspective projection.
    ///
    /// # Errors
    ///
    /// A parameter is out of range or not finite.
    pub fn perspective(fov_y: f64, near: f64, far: f64) -> Result<Self> {
        let projection = Self::Perspective { fov_y, near, far };
        projection.validate()?;
        Ok(projection)
    }

    /// An orthographic projection.
    ///
    /// # Errors
    ///
    /// A parameter is out of range or not finite.
    pub fn orthographic(height: f64, near: f64, far: f64) -> Result<Self> {
        let projection = Self::Orthographic { height, near, far };
        projection.validate()?;
        Ok(projection)
    }

    fn validate(&self) -> Result<()> {
        let (ok, what) = match *self {
            Self::Perspective { fov_y, near, far } => (
                fov_y.is_finite()
                    && fov_y > 0.0
                    && fov_y < PI
                    && near.is_finite()
                    && near > 0.0
                    && far.is_finite()
                    && far > near,
                format!("perspective fov_y {fov_y}, near {near}, far {far}"),
            ),
            Self::Orthographic { height, near, far } => (
                height.is_finite()
                    && height > 0.0
                    && near.is_finite()
                    && near >= 0.0
                    && far.is_finite()
                    && far > near,
                format!("orthographic height {height}, near {near}, far {far}"),
            ),
        };
        if ok {
            Ok(())
        } else {
            Err(wrong("a projection parameter is out of range").with_context("projection", what))
        }
    }

    /// The matrix, reverse-Z, for a viewport of this width over height.
    fn matrix(&self, aspect: f64) -> Columns {
        match *self {
            Self::Perspective { fov_y, near, far } => {
                let focal = 1.0 / (fov_y / 2.0).tan();
                // depth = (a·z + b) / -z, with z = -near → 1 and z = -far → 0.
                let a = near / (far - near);
                let b = near * far / (far - near);
                [
                    [focal / aspect, 0.0, 0.0, 0.0],
                    [0.0, focal, 0.0, 0.0],
                    [0.0, 0.0, a, -1.0],
                    [0.0, 0.0, b, 0.0],
                ]
            }
            Self::Orthographic { height, near, far } => {
                let half_height = height / 2.0;
                let half_width = half_height * aspect;
                // depth = (z + far) / (far - near), again near → 1, far → 0.
                [
                    [1.0 / half_width, 0.0, 0.0, 0.0],
                    [0.0, 1.0 / half_height, 0.0, 0.0],
                    [0.0, 0.0, 1.0 / (far - near), 0.0],
                    [0.0, 0.0, far / (far - near), 1.0],
                ]
            }
        }
    }
}

/// Where the camera is, where it looks, and how it projects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    position: WorldPosition,
    yaw: f64,
    pitch: f64,
    projection: Projection,
}

impl Camera {
    /// A camera at `position`, looking down `-Z` (yaw and pitch zero).
    ///
    /// # Errors
    ///
    /// The position is not finite, or the projection is invalid.
    pub fn new(position: WorldPosition, projection: Projection) -> Result<Self> {
        projection.validate()?;
        Ok(Self {
            position: position.require_finite()?,
            yaw: 0.0,
            pitch: 0.0,
            projection,
        })
    }

    /// Where the camera is.
    #[must_use]
    pub const fn position(&self) -> WorldPosition {
        self.position
    }

    /// Rotation about the world's `+Y`, in radians, in `(-π, π]`. Zero looks
    /// down `-Z`; a quarter turn (counter-clockwise seen from above) looks
    /// down `-X`.
    #[must_use]
    pub const fn yaw(&self) -> f64 {
        self.yaw
    }

    /// Elevation, in radians, within `±MAX_PITCH`; positive looks up.
    #[must_use]
    pub const fn pitch(&self) -> f64 {
        self.pitch
    }

    /// The projection.
    #[must_use]
    pub const fn projection(&self) -> Projection {
        self.projection
    }

    /// Move the camera.
    ///
    /// # Errors
    ///
    /// The position is not finite; the camera does not move.
    pub fn set_position(&mut self, position: WorldPosition) -> Result<()> {
        self.position = position.require_finite()?;
        Ok(())
    }

    /// Turn the camera. Yaw wraps into `(-π, π]`; pitch is clamped to
    /// `±MAX_PITCH`.
    ///
    /// # Errors
    ///
    /// An angle is not finite; the camera does not turn.
    pub fn set_orientation(&mut self, yaw: f64, pitch: f64) -> Result<()> {
        if !yaw.is_finite() || !pitch.is_finite() {
            return Err(wrong("an orientation angle is not finite")
                .with_context("yaw", yaw.to_string())
                .with_context("pitch", pitch.to_string()));
        }
        let mut wrapped = yaw.rem_euclid(2.0 * PI);
        if wrapped > PI {
            wrapped -= 2.0 * PI;
        }
        self.yaw = wrapped;
        self.pitch = pitch.clamp(-MAX_PITCH, MAX_PITCH);
        Ok(())
    }

    /// Turn the camera toward `target`.
    ///
    /// # Errors
    ///
    /// The target is not finite, or is where the camera is.
    pub fn look_at(&mut self, target: WorldPosition) -> Result<()> {
        let target = target.require_finite()?;
        let d = [
            target.x - self.position.x,
            target.y - self.position.y,
            target.z - self.position.z,
        ];
        let horizontal = d[0].hypot(d[2]);
        if horizontal == 0.0 && d[1] == 0.0 {
            return Err(wrong("a camera cannot look at the point it is at"));
        }
        self.set_orientation((-d[0]).atan2(-d[2]), d[1].atan2(horizontal))
    }

    /// The unit direction the camera looks in, in world (and render) axes.
    #[must_use]
    pub fn forward(&self) -> [f64; 3] {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        [-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch]
    }

    /// Everything a renderer needs this frame, for a `width` × `height`
    /// viewport, relative to `origin`.
    ///
    /// # Errors
    ///
    /// The viewport is empty, or the camera has drifted past
    /// [`REBASE_DISTANCE`] from `origin`: [`RenderOrigin::follow`] it first.
    pub fn sample(&self, origin: RenderOrigin, width: u32, height: u32) -> Result<CameraState> {
        if width == 0 || height == 0 {
            return Err(wrong("a camera needs a viewport with an area")
                .with_context("width", width.to_string())
                .with_context("height", height.to_string()));
        }
        if origin.needs_rebase(self.position) {
            return Err(
                wrong("the camera is too far from the render origin for f32 precision")
                    .with_context("origin", format!("{:?}", origin.block()))
                    .with_context("rebase_distance", REBASE_DISTANCE.to_string()),
            );
        }
        let eye = origin.relative(self.position);
        let f = self.forward();
        // right = forward × up, with up = +Y; never zero, since |pitch| < 90°.
        let right_length = f[0].hypot(f[2]);
        let r = [-f[2] / right_length, 0.0, f[0] / right_length];
        let u = [
            r[1] * f[2] - r[2] * f[1],
            r[2] * f[0] - r[0] * f[2],
            r[0] * f[1] - r[1] * f[0],
        ];
        let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let view: Columns = [
            [r[0], u[0], -f[0], 0.0],
            [r[1], u[1], -f[1], 0.0],
            [r[2], u[2], -f[2], 0.0],
            [-dot(r, eye), -dot(u, eye), dot(f, eye), 1.0],
        ];
        let projection = self.projection.matrix(f64::from(width) / f64::from(height));
        let view_projection = multiply(&projection, &view);
        Ok(CameraState {
            origin,
            eye: eye.map(|value| value as f32),
            forward: f.map(|value| value as f32),
            view: Mat4::rounded(&view),
            projection: Mat4::rounded(&projection),
            view_projection: Mat4::rounded(&view_projection),
            frustum: Frustum::from_view_projection(&view_projection),
        })
    }
}

/// A camera, resolved for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraState {
    /// The render origin everything here is relative to.
    pub origin: RenderOrigin,
    /// The camera's position in render space.
    pub eye: [f32; 3],
    /// The unit view direction.
    pub forward: [f32; 3],
    /// Render space to view space.
    pub view: Mat4,
    /// View space to clip space, reverse-Z.
    pub projection: Mat4,
    /// Render space to clip space: what a vertex shader multiplies by.
    pub view_projection: Mat4,
    /// The six planes, in render space.
    pub frustum: Frustum,
}

impl CameraState {
    /// Whether any of the blocks from `min` up to, not including, `max` may
    /// be visible. Conservative, as [`Frustum::intersects_box`] is.
    #[must_use]
    pub fn sees_blocks(&self, min: BlockPos, max: BlockPos) -> bool {
        let base = self.origin.block();
        // i64 differences, exact in f64 for anything within the world.
        let corner = |block: BlockPos| {
            [
                (block.x - base.x) as f64,
                (block.y - base.y) as f64,
                (block.z - base.z) as f64,
            ]
        };
        self.frustum.intersects_box(corner(min), corner(max))
    }
}

fn wrong(message: &'static str) -> Error {
    Error::new(Domain::Render, "camera", message).with_recovery(Recovery::Reject)
}
