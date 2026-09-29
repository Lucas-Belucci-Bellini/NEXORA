//! What the camera can see: six planes in render space.
//!
//! Extracted from the view-projection (Gribb and Hartmann), so the frustum and
//! the projection cannot disagree: a point is inside exactly when the matrix
//! maps it into the clip volume. With the engine's reverse-Z depth that volume
//! is `-w ≤ x ≤ w`, `-w ≤ y ≤ w` and `0 ≤ z ≤ w`, where `z = w` is the near
//! plane and `z = 0` the far one.
//!
//! Kept in `f64`, in render space. Culling is a CPU question and has no reason
//! to inherit the GPU's precision.

use crate::matrix::{row, Columns};

/// A plane `n · p + d = 0`, with `n` pointing inside.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    /// The inward normal, unit length.
    pub normal: [f64; 3],
    /// The offset.
    pub d: f64,
}

impl Plane {
    fn from_coefficients(c: [f64; 4]) -> Self {
        let length = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
        Self {
            normal: [c[0] / length, c[1] / length, c[2] / length],
            d: c[3] / length,
        }
    }

    /// Signed distance from `p`: positive inside.
    #[must_use]
    pub fn distance(&self, p: [f64; 3]) -> f64 {
        self.normal[0] * p[0] + self.normal[1] * p[1] + self.normal[2] * p[2] + self.d
    }
}

/// The six planes, in the order left, right, bottom, top, near, far.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frustum {
    /// The planes.
    pub planes: [Plane; 6],
}

impl Frustum {
    /// The frustum of a view-projection whose clip volume has `0 ≤ z ≤ w`.
    pub(crate) fn from_view_projection(m: &Columns) -> Self {
        let r = [row(m, 0), row(m, 1), row(m, 2), row(m, 3)];
        let add = |a: [f64; 4], b: [f64; 4]| [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]];
        let sub = |a: [f64; 4], b: [f64; 4]| [a[0] - b[0], a[1] - b[1], a[2] - b[2], a[3] - b[3]];
        Self {
            planes: [
                Plane::from_coefficients(add(r[3], r[0])),
                Plane::from_coefficients(sub(r[3], r[0])),
                Plane::from_coefficients(add(r[3], r[1])),
                Plane::from_coefficients(sub(r[3], r[1])),
                // Reverse-Z: the near plane is z = w, the far plane z = 0.
                Plane::from_coefficients(sub(r[3], r[2])),
                Plane::from_coefficients(r[2]),
            ],
        }
    }

    /// Whether `p` is inside or on the boundary.
    #[must_use]
    pub fn contains(&self, p: [f64; 3]) -> bool {
        self.planes.iter().all(|plane| plane.distance(p) >= 0.0)
    }

    /// Whether any of the box from `min` to `max` may be visible.
    ///
    /// Conservative: `false` only when the box is entirely outside one plane.
    /// A box near a corner of the frustum, outside it but not wholly behind any
    /// single plane, answers `true`. A culler may draw too much, never too
    /// little.
    #[must_use]
    pub fn intersects_box(&self, min: [f64; 3], max: [f64; 3]) -> bool {
        self.planes.iter().all(|plane| {
            // The corner farthest along the normal: if even it is outside,
            // the whole box is.
            let corner = [0, 1, 2].map(|axis| {
                if plane.normal[axis] >= 0.0 {
                    max[axis]
                } else {
                    min[axis]
                }
            });
            plane.distance(corner) >= 0.0
        })
    }
}
