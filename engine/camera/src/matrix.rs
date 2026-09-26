//! The one matrix type a shader reads.
//!
//! Every matrix is **built and multiplied in `f64`**, and rounded to `f32`
//! once, at the end. Composing in `f32` would round after every product, and a
//! view-projection would then differ from the product of its own view and
//! projection by more than the last bit. Rounding once keeps the three matrices
//! a camera publishes consistent with each other.
//!
//! The layout is **column-major**, the layout of WGSL's `mat4x4<f32>` (and of
//! GLSL, HLSL with `column_major`, and MSL): [`Mat4::to_bytes`] is exactly what
//! a uniform buffer holds. Vectors are columns, and a point transforms as
//! `M · (x, y, z, 1)`.

/// Four columns of four `f64`: the working precision.
pub(crate) type Columns = [[f64; 4]; 4];

/// The identity, in working precision.
#[cfg(test)]
pub(crate) const IDENTITY: Columns = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// `a · b`, in working precision.
pub(crate) fn multiply(a: &Columns, b: &Columns) -> Columns {
    let mut out = [[0.0; 4]; 4];
    for (column, b_column) in out.iter_mut().zip(b) {
        for (row, value) in column.iter_mut().enumerate() {
            *value = (0..4).map(|k| a[k][row] * b_column[k]).sum();
        }
    }
    out
}

/// Row `index` of a column-major matrix.
pub(crate) fn row(m: &Columns, index: usize) -> [f64; 4] {
    [m[0][index], m[1][index], m[2][index], m[3][index]]
}

/// A 4×4 `f32` matrix, column-major, as a shader receives it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    /// The four columns.
    pub cols: [[f32; 4]; 4],
}

impl Mat4 {
    /// The identity.
    pub const IDENTITY: Self = Self {
        cols: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// Bytes a uniform buffer holds: sixteen little-endian `f32`, column by
    /// column. Sixty-four bytes, a multiple of the RHI's uniform alignment.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        for (chunk, value) in out.chunks_exact_mut(4).zip(self.cols.iter().flatten()) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        out
    }

    /// `M · v`, in `f32`, as a shader computes it.
    #[must_use]
    pub fn transform(&self, v: [f32; 4]) -> [f32; 4] {
        let mut out = [0.0f32; 4];
        for (row, value) in out.iter_mut().enumerate() {
            *value = (0..4).map(|k| self.cols[k][row] * v[k]).sum();
        }
        out
    }

    /// Round a working-precision matrix, once.
    pub(crate) fn rounded(m: &Columns) -> Self {
        Self {
            cols: m.map(|column| column.map(|value| value as f32)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplying_by_the_identity_changes_nothing() {
        let m: Columns = [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ];
        assert_eq!(multiply(&m, &IDENTITY), m);
        assert_eq!(multiply(&IDENTITY, &m), m);
    }

    #[test]
    fn products_compose_left_to_right_as_column_vectors_do() {
        // A translation by +x after a scale by 2: T · S applied to (1,0,0,1)
        // is (2+5, 0, 0, 1), not (1+5)*2.
        let scale: Columns = [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mut translate = IDENTITY;
        translate[3] = [5.0, 0.0, 0.0, 1.0];
        let m = Mat4::rounded(&multiply(&translate, &scale));
        assert_eq!(m.transform([1.0, 0.0, 0.0, 1.0]), [7.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn bytes_are_column_major_little_endian() {
        let mut m = Mat4::IDENTITY;
        m.cols[3][0] = 5.0;
        let bytes = m.to_bytes();
        assert_eq!(&bytes[0..4], &1.0f32.to_le_bytes());
        // Column 3, row 0 is the twelfth float.
        assert_eq!(&bytes[48..52], &5.0f32.to_le_bytes());
        assert_eq!(&bytes[60..64], &1.0f32.to_le_bytes());
    }

    #[test]
    fn rows_read_across_columns() {
        let mut m = IDENTITY;
        m[3][1] = 9.0;
        assert_eq!(row(&m, 1), [0.0, 1.0, 0.0, 9.0]);
    }
}
