use super::vector::{Vec2, Vec3, Vec4};
use std::ops;

#[derive(thiserror::Error, Debug)]
pub enum MatrixError {
    #[error("Matrix has no inverse")]
    NonInvertable,
}

macro_rules! matrix {
    ($name:ident, $vec_type:ident { $($col:ident: $comp:ident),+ }) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        #[repr(C)]
        pub struct $name {
            $(pub $col: $vec_type),+
        }

        impl $name {
            pub const fn new($($comp: $vec_type),+) -> Self {
                Self { $($col: $comp),+ }
            }

            pub fn invert(&mut self) -> bool {
                let inverse = self.inv();

                if inverse.is_ok() {
                    *self = inverse.unwrap();
                    return true;
                }

                false
            }

            pub fn transpose(&mut self) {
                *self = self.T();
            }
        }

        // Matrix Ops
        matrix!(@mm_op $name, Add, Self, add, +, { |m|, $($col: m.$col),+ }); // Add
        matrix!(@mm_op $name, Sub, Self, sub, -, { |m|, $($col: m.$col),+ }); // Sub

        impl ops::Mul<$name> for $name {
            type Output = Self;

            fn mul(self, m: $name) -> $name {
                Self{ $($col: self * m.$col),+ }
            }
        }

        // Vector Ops
        impl ops::Mul<$vec_type> for $name {
            type Output = $vec_type;

            fn mul(self, v: $vec_type) -> $vec_type {
                $vec_type::ZERO $(+ v.$comp * self.$col)+
            }
        }

        // Scalar Ops
        matrix!(@ms_op $name, Add, f32, add, +, { |s|, $($col: s),+ });
        matrix!(@ms_op $name, Sub, f32, sub, -, { |s|, $($col: s),+ });
        matrix!(@ms_op $name, Mul, f32, mul, *, { |s|, $($col: s),+ });
        matrix!(@ms_op $name, Div, f32, div, /, { |s|, $($col: s),+ });

        // Allow for s*M and M*s
        impl ops::Mul<$name> for f32 {
            type Output = $name;

            fn mul(self, rhs: $name) -> $name {
                rhs * self
            }
        }

        // Allow for negation
        impl ops::Neg for $name {
            type Output = Self;

            fn neg(self) -> Self {
                Self {
                    $($col: -self.$col),+
                }
            }
        }

        /// `m[i]` returns **column** `i`, so `m[col][row]` (GLSL/WGSL order).
        /// Use `m[(row, col)]` for math-style row-then-column indexing.
        impl ops::Index<usize> for $name {
            type Output = $vec_type;
            fn index(&self, i: usize) -> &$vec_type {
                [$(&self.$col),+][i]
            }
        }

        impl ops::IndexMut<usize> for $name {
            fn index_mut(&mut self, i: usize) -> &mut $vec_type {
                [$(&mut self.$col),+]
                    .into_iter()
                    .nth(i)
                    .unwrap_or_else(|| panic!("column {i} out of range for {}", stringify!($name)))
            }
        }

        /// `m[(row, col)]`: math-style entry access.
        impl ops::Index<(usize, usize)> for $name {
            type Output = f32;
            fn index(&self, (row, col): (usize, usize)) -> &f32 {
                &self[col][row]
            }
        }

        impl ops::IndexMut<(usize, usize)> for $name {
            fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut f32 {
                &mut self[col][row]
            }
        }
    };

    (@mm_op $name:ident, $trait:ident, $rhs:ty, $method:ident,
        $op:tt, { |$r:ident|, $($field:ident: $other:expr),+ }) => {

        impl ops::$trait for $name {
            type Output = Self;

            fn $method(self, $r: $rhs) -> Self {
                Self { $($field: self.$field $op $other),+ }
            }
        }

    };

    (@ms_op $name:ident, $trait:ident, $rhs:ty, $method:ident,
        $op:tt, { |$r:ident|, $($field:ident: $other:expr),+ }) => {

        impl ops::$trait<$rhs> for $name {
            type Output = Self;

            fn $method(self, $r: $rhs) -> Self {
                Self { $($field: self.$field $op $other),+ }
            }
        }

    };

    (@assign_op $name:ident, $rhs:ident, $assign_trait:ident, $assign_method:ident, $output:ty,
        $op:tt, { |$r:ident|, $($field:ident: other:expr),+ }) => {

        impl ops::$assign_trait for $name {
            fn $assign_method(&mut self, $r: $rhs) {
                *self = *self $op $r
            }
        }
    };



}

matrix!(
    Mat2,
    Vec2 {
        x_axis: x,
        y_axis: y
    }
);
matrix!(
    Mat3,
    Vec3 {
        x_axis: x,
        y_axis: y,
        z_axis: z
    }
);
matrix!(
    Mat4,
    Vec4 {
        x_axis: x,
        y_axis: y,
        z_axis: z,
        w_axis: w
    }
);

// pub struct Mat2 {
//     pub x_axis: Vec2,
//     pub y_axis: Vec2,
// }

// impl ops::Mul<Mat2> for f32 {
//     type Output = Mat2;

//     fn mul(self, rhs: Mat2) -> Mat2 {
//         Mat2 {
//             x_axis: self * rhs.x_axis,
//             y_axis: self * rhs.y_axis,
//         }
//     }
// }

// impl ops::Mul<Vec2> for Mat2 {
//     type Output = Vec2;

//     fn mul(self, rhs: Vec2) -> Vec2 {
//         rhs.x * self.x_axis + rhs.y * self.y_axis
//     }
// }

impl Mat2 {
    pub fn identity() -> Self {
        Self {
            x_axis: Vec2 { x: 1.0, y: 0.0 },
            y_axis: Vec2 { x: 0.0, y: 1.0 },
        }
    }

    pub fn determinant(&self) -> f32 {
        self.x_axis.x * self.y_axis.y - self.y_axis.x * self.x_axis.y
    }

    pub fn inv(&self) -> Result<Self, MatrixError> {
        let det = self.determinant();
        if det.abs() < f32::EPSILON {
            return Err(MatrixError::NonInvertable);
        }

        Ok(Self {
            x_axis: Vec2 {
                x: self.y_axis.y,
                y: -self.x_axis.y,
            } / det,
            y_axis: Vec2 {
                x: -self.y_axis.x,
                y: self.x_axis.x,
            } / det,
        })
    }

    #[allow(non_snake_case)]
    pub fn T(&self) -> Self {
        Self {
            x_axis: Vec2 {
                x: self.x_axis.x,
                y: self.y_axis.x,
            },
            y_axis: Vec2 {
                x: self.x_axis.y,
                y: self.y_axis.y,
            },
        }
    }
}

impl Mat3 {
    fn identity() -> Self {
        Self {
            x_axis: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            y_axis: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            z_axis: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        }
    }

    fn cofactor(&self) -> Self {
        Self {
            x_axis: self.y_axis.cross(self.z_axis),
            y_axis: self.z_axis.cross(self.x_axis),
            z_axis: self.x_axis.cross(self.y_axis),
        }
    }

    pub fn determinant(&self) -> f32 {
        let cof = self.cofactor();
        self.x_axis.dot(cof.x_axis)
    }

    pub fn inv(&self) -> Result<Self, MatrixError> {
        let cof = self.cofactor();
        let det = self.x_axis.dot(cof.x_axis);
        if det.abs() < f32::EPSILON {
            return Err(MatrixError::NonInvertable);
        }

        Ok(cof.T() / det)
    }

    #[allow(non_snake_case)]
    pub fn T(&self) -> Self {
        Self {
            x_axis: Vec3 {
                x: self.x_axis.x,
                y: self.y_axis.x,
                z: self.z_axis.x,
            },
            y_axis: Vec3 {
                x: self.x_axis.y,
                y: self.y_axis.y,
                z: self.z_axis.y,
            },
            z_axis: Vec3 {
                x: self.x_axis.z,
                y: self.y_axis.z,
                z: self.z_axis.z,
            },
        }
    }
}

impl Mat4 {
    fn identity() -> Self {
        Self {
            x_axis: Vec4 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            },
            y_axis: Vec4 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
                w: 0.0,
            },
            z_axis: Vec4 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
                w: 0.0,
            },
            w_axis: Vec4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 1.0,
            },
        }
    }

    /// Every 3x3 minor of a 4x4 is built from 2x2 determinants of either the top two rows
    /// (x, y) or the bottom two rows (z, w), so those 12 are computed once and reused.
    /// Top-row cofactors expand their minor along the other top row using the bottom 2x2s;
    /// bottom-row cofactors expand along the other bottom row using the top 2x2s.
    fn cofactor(&self) -> Self {
        let (a, b, c, d) = (self.x_axis, self.y_axis, self.z_axis, self.w_axis);

        // 2x2 determinants of the top two rows, for column pairs (0=a, 1=b, 2=c, 3=d)
        let s01 = a.x * b.y - b.x * a.y;
        let s02 = a.x * c.y - c.x * a.y;
        let s03 = a.x * d.y - d.x * a.y;
        let s12 = b.x * c.y - c.x * b.y;
        let s13 = b.x * d.y - d.x * b.y;
        let s23 = c.x * d.y - d.x * c.y;

        // 2x2 determinants of the bottom two rows
        let t01 = a.z * b.w - b.z * a.w;
        let t02 = a.z * c.w - c.z * a.w;
        let t03 = a.z * d.w - d.z * a.w;
        let t12 = b.z * c.w - c.z * b.w;
        let t13 = b.z * d.w - d.z * b.w;
        let t23 = c.z * d.w - d.z * c.w;

        Self {
            x_axis: Vec4 {
                x: b.y * t23 - c.y * t13 + d.y * t12,
                y: -(b.x * t23 - c.x * t13 + d.x * t12),
                z: b.w * s23 - c.w * s13 + d.w * s12,
                w: -(b.z * s23 - c.z * s13 + d.z * s12),
            },
            y_axis: Vec4 {
                x: -(a.y * t23 - c.y * t03 + d.y * t02),
                y: a.x * t23 - c.x * t03 + d.x * t02,
                z: -(a.w * s23 - c.w * s03 + d.w * s02),
                w: a.z * s23 - c.z * s03 + d.z * s02,
            },
            z_axis: Vec4 {
                x: a.y * t13 - b.y * t03 + d.y * t01,
                y: -(a.x * t13 - b.x * t03 + d.x * t01),
                z: a.w * s13 - b.w * s03 + d.w * s01,
                w: -(a.z * s13 - b.z * s03 + d.z * s01),
            },
            w_axis: Vec4 {
                x: -(a.y * t12 - b.y * t02 + c.y * t01),
                y: a.x * t12 - b.x * t02 + c.x * t01,
                z: -(a.w * s12 - b.w * s02 + c.w * s01),
                w: a.z * s12 - b.z * s02 + c.z * s01,
            },
        }
    }

    pub fn determinant(&self) -> f32 {
        let cof = self.cofactor();
        self.x_axis.dot(cof.x_axis)
    }

    pub fn inv(&self) -> Result<Self, MatrixError> {
        let cof = self.cofactor();
        let det = self.x_axis.dot(cof.x_axis);
        if det.abs() < f32::EPSILON {
            return Err(MatrixError::NonInvertable);
        }

        Ok(cof.T() / det)
    }

    #[allow(non_snake_case)]
    pub fn T(&self) -> Self {
        Self {
            x_axis: Vec4 {
                x: self.x_axis.x,
                y: self.y_axis.x,
                z: self.z_axis.x,
                w: self.w_axis.x,
            },
            y_axis: Vec4 {
                x: self.x_axis.y,
                y: self.y_axis.y,
                z: self.z_axis.y,
                w: self.w_axis.y,
            },
            z_axis: Vec4 {
                x: self.x_axis.z,
                y: self.y_axis.z,
                z: self.z_axis.z,
                w: self.w_axis.z,
            },
            w_axis: Vec4 {
                x: self.x_axis.w,
                y: self.y_axis.w,
                z: self.z_axis.w,
                w: self.w_axis.w,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Mat3, Mat4};
    use crate::math::{Vec3, Vec4};

    const TOL: f32 = 1e-5;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < TOL
    }

    #[test]
    fn mat3_inverse_gives_identity() {
        let m = Mat3::new(
            Vec3::new(2.0, 1.0, 0.0),
            Vec3::new(-1.0, 3.0, 2.0),
            Vec3::new(4.0, 0.5, 1.0),
        );
        let inv = m.inv().unwrap();
        let ident = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        for (col, expected) in [inv.x_axis, inv.y_axis, inv.z_axis].into_iter().zip(ident) {
            let got = m * col;
            assert!(
                approx(got.x, expected.x) && approx(got.y, expected.y) && approx(got.z, expected.z)
            );
        }
    }

    #[test]
    fn mat4_determinant_of_triangular_is_diagonal_product() {
        let m = Mat4::new(
            Vec4::new(2.0, 0.0, 0.0, 0.0),
            Vec4::new(5.0, 3.0, 0.0, 0.0),
            Vec4::new(-1.0, 7.0, 4.0, 0.0),
            Vec4::new(6.0, 2.0, 9.0, 0.5),
        );
        assert!(approx(m.determinant(), 2.0 * 3.0 * 4.0 * 0.5));
    }

    #[test]
    fn mat4_inverse_gives_identity() {
        let m = Mat4::new(
            Vec4::new(2.0, 1.0, 0.0, 3.0),
            Vec4::new(-1.0, 3.0, 2.0, 0.0),
            Vec4::new(4.0, 0.5, 1.0, -2.0),
            Vec4::new(1.0, 1.0, -3.0, 2.0),
        );
        let inv = m.inv().unwrap();
        let ident = [
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        ];
        let cols = [inv.x_axis, inv.y_axis, inv.z_axis, inv.w_axis];
        for (col, expected) in cols.into_iter().zip(ident) {
            let got = m * col;
            assert!(
                approx(got.x, expected.x)
                    && approx(got.y, expected.y)
                    && approx(got.z, expected.z)
                    && approx(got.w, expected.w),
                "{got:?} != {expected:?}"
            );
        }
    }

    #[test]
    fn singular_matrix_has_no_inverse() {
        let m = Mat4::new(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(2.0, 4.0, 6.0, 8.0),
            Vec4::new(0.0, 1.0, 0.0, 1.0),
            Vec4::new(1.0, 0.0, 1.0, 0.0),
        );
        assert!(m.inv().is_err());
    }

    #[test]
    fn index_row_col_and_column() {
        let mut m = Mat3::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        assert_eq!(m[(0, 1)], 4.0); // row 0, column 1 = y_axis.x
        assert_eq!(m[(1, 0)], 2.0); // row 1, column 0 = x_axis.y
        assert_eq!(m[2], Vec3::new(7.0, 8.0, 9.0));
        assert_eq!(m[2][1], m[(1, 2)]);

        m[(2, 2)] = 42.0;
        m[0][0] = -1.0;
        assert_eq!(m.z_axis.z, 42.0);
        assert_eq!(m.x_axis.x, -1.0);
    }
}
