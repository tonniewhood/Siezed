use std::ops;

/// Defines an `f32` vector struct with the given fields, plus the methods and operator
/// overloads shared by every vector size. Size-specific methods (e.g. `cross`) live in
/// their own `impl` blocks below.
macro_rules! vector {
    ($name:ident { $($field:ident),+ }) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        pub struct $name {
            $(pub $field: f32),+
        }

        impl $name {
            pub const ZERO: Self = Self { $($field: 0.0),+ };

            pub const fn new($($field: f32),+) -> Self {
                Self { $($field),+ }
            }

            pub fn dot(self, v: Self) -> f32 {
                0.0 $(+ self.$field * v.$field)+
            }

            pub fn norm(self) -> f32 {
                self.dot(self).sqrt()
            }

            pub fn any(self, f: impl Fn(f32) -> bool) -> bool {
                false $(|| f(self.$field))+
            }

            pub fn all(self, f: impl Fn(f32) -> bool) -> bool {
                true $(&& f(self.$field))+
            }

            pub fn foreach(&self, f: impl Fn(usize, &f32)) {
                for (i, c) in [$(&self.$field),+].into_iter().enumerate() {
                    f(i, c);
                }
            }

            pub fn foreach_mut(&mut self, mut f: impl FnMut(usize, &mut f32)) {
                for (i, c) in [$(&mut self.$field),+].into_iter().enumerate() {
                    f(i, c);
                }
            }
        }

        // Vector (op) vector, component-wise
        vector!(@op $name, Self, Add, add, AddAssign, add_assign, +, |v| { $($field: v.$field),+ });
        vector!(@op $name, Self, Sub, sub, SubAssign, sub_assign, -, |v| { $($field: v.$field),+ });

        // Vector (op) scalar, applied to every component
        vector!(@op $name, f32, Add, add, AddAssign, add_assign, +, |s| { $($field: s),+ });
        vector!(@op $name, f32, Sub, sub, SubAssign, sub_assign, -, |s| { $($field: s),+ });
        vector!(@op $name, f32, Mul, mul, MulAssign, mul_assign, *, |s| { $($field: s),+ });
        vector!(@op $name, f32, Div, div, DivAssign, div_assign, /, |s| { $($field: s),+ });

        // Scalar * vector, so both `v * 2.0` and `2.0 * v` work
        impl ops::Mul<$name> for f32 {
            type Output = $name;
            fn mul(self, v: $name) -> $name {
                v * self
            }
        }

        impl ops::Neg for $name {
            type Output = Self;
            fn neg(self) -> Self {
                Self { $($field: -self.$field),+ }
            }
        }

        // `v[0]` -> x, `v[1]` -> y, ... Indexes a temporary array of field references so the
        // macro doesn't need to count fields; the array is optimized away.
        impl ops::Index<usize> for $name {
            type Output = f32;
            fn index(&self, i: usize) -> &f32 {
                [$(&self.$field),+][i]
            }
        }

        impl ops::IndexMut<usize> for $name {
            fn index_mut(&mut self, i: usize) -> &mut f32 {
                [$(&mut self.$field),+]
                    .into_iter()
                    .nth(i)
                    .unwrap_or_else(|| panic!("index {i} out of range for {}", stringify!($name)))
            }
        }
    };

    // Implements `$name (op) $rhs` and `$name (op)= $rhs`. `|$r| { field: expr, .. }` says
    // what to combine each of self's fields with, so the same rule covers vector and scalar rhs.
    (@op $name:ident, $rhs:ty, $trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident,
     $op:tt, |$r:ident| { $($field:ident: $other:expr),+ }) => {
        impl ops::$trait<$rhs> for $name {
            type Output = Self;
            fn $method(self, $r: $rhs) -> Self {
                Self { $($field: self.$field $op $other),+ }
            }
        }

        impl ops::$assign_trait<$rhs> for $name {
            fn $assign_method(&mut self, $r: $rhs) {
                *self = *self $op $r;
            }
        }
    };
}

vector!(Vec2 { x, y });
vector!(Vec3 { x, y, z });
vector!(Vec4 { x, y, z, w });

impl Vec2 {
    /// 2D cross product (a scalar): the signed area of the parallelogram spanned by `self`
    /// and `v`. Positive when `v` is counter-clockwise from `self`.
    pub fn cross(self, v: Self) -> f32 {
        self.x * v.y - self.y * v.x
    }
}

impl Vec3 {
    pub fn cross(self, v: Self) -> Self {
        Self {
            x: self.y * v.z - self.z * v.y,
            y: self.z * v.x - self.x * v.z,
            z: self.x * v.y - self.y * v.x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Vec2, Vec3, Vec4};

    #[test]
    fn dot_and_norm() {
        assert_eq!(Vec2::new(1.0, 2.0).dot(Vec2::new(3.0, 4.0)), 11.0);
        assert_eq!(Vec3::new(3.0, 4.0, 0.0).norm(), 5.0);
        assert_eq!(Vec4::new(1.0, 1.0, 1.0, 1.0).norm(), 2.0);
    }

    #[test]
    fn vector_and_scalar_ops() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
        assert_eq!(a * 2.0, 2.0 * a);
        assert_eq!(b / 2.0, Vec3::new(2.0, 2.5, 3.0));
        assert_eq!(a + 1.0, Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));

        let mut c = a;
        c += b;
        c *= 2.0;
        assert_eq!(c, Vec3::new(10.0, 14.0, 18.0));
    }

    #[test]
    fn cross_products() {
        assert_eq!(Vec2::new(1.0, 0.0).cross(Vec2::new(0.0, 1.0)), 1.0);
        assert_eq!(
            Vec3::new(1.0, 0.0, 0.0).cross(Vec3::new(0.0, 1.0, 0.0)),
            Vec3::new(0.0, 0.0, 1.0)
        );
    }

    #[test]
    fn index_components() {
        let mut v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!([v[0], v[1], v[2], v[3]], [1.0, 2.0, 3.0, 4.0]);
        v[2] = 9.0;
        assert_eq!(v.z, 9.0);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn index_out_of_range_panics() {
        let mut v = Vec2::new(1.0, 2.0);
        v[2] = 0.0;
    }
}
