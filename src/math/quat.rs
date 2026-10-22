//! Unit quaternions for 3D rotation.
//!
//! A rotation by angle `a` about unit axis `n` is the quaternion
//!     q = (n * sin(a/2), cos(a/2))       i.e. x, y, z = the axis part, w = the scalar part
//! Rotating a vector is `q * v * q⁻¹`, and composing rotations is quaternion multiplication:
//! `(q2 * q1)` applies `q1` first, then `q2`, the same order as matrices.
//!
//! Conventions match `transform.rs`: right-handed, roll phi (x), pitch theta (y), yaw psi (z),
//! applied x then y then z, so `Quat::from_euler(phi, theta, psi)` == `RotMat(phi, theta, psi)`.
use std::ops;

use crate::math::{Mat3, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Quat {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let n = axis / axis.norm();
        let (s, c) = (angle * 0.5).sin_cos();
        Self {
            x: n.x * s,
            y: n.y * s,
            z: n.z * s,
            w: c,
        }
    }

    /// Same rotation as `RotZ(psi) * RotY(theta) * RotX(phi)`.
    /// Equivalent to `qz(psi) * qy(theta) * qx(phi)` multiplied out by hand.
    pub fn from_euler(phi: f32, theta: f32, psi: f32) -> Self {
        let (sr, cr) = (phi * 0.5).sin_cos();
        let (sp, cp) = (theta * 0.5).sin_cos();
        let (sy, cy) = (psi * 0.5).sin_cos();

        Self {
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
            w: cr * cp * cy + sr * sp * sy,
        }
    }

    /// Returns `(phi, theta, psi)`. Pitch is limited to [-90°, 90°]; at exactly ±90°
    /// (gimbal lock) roll and yaw spin about the same axis, so only their sum/difference
    /// is meaningful and the split returned is one of infinitely many valid answers.
    pub fn to_euler(self) -> (f32, f32, f32) {
        let Self { x, y, z, w } = self;

        let phi = f32::atan2(2.0 * (w * x + y * z), 1.0 - 2.0 * (x * x + y * y));
        // Clamp: float error can push this just past ±1, which would make asin return NaN
        let theta = f32::asin((2.0 * (w * y - z * x)).clamp(-1.0, 1.0));
        let psi = f32::atan2(2.0 * (w * z + x * y), 1.0 - 2.0 * (y * y + z * z));

        (phi, theta, psi)
    }

    pub fn norm(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    /// Rotations must be unit length; renormalize occasionally after many multiplications
    /// so float error doesn't slowly turn the rotation into a rotation + scale.
    pub fn normalize(self) -> Self {
        let n = self.norm();
        Self {
            x: self.x / n,
            y: self.y / n,
            z: self.z / n,
            w: self.w / n,
        }
    }

    /// The inverse rotation, for unit quaternions (negate the axis, keep the angle).
    pub fn conjugate(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    /// `q * v * q⁻¹`, expanded so it only needs two cross products.
    pub fn rotate(self, v: Vec3) -> Vec3 {
        let u = Vec3::new(self.x, self.y, self.z);
        let t = 2.0 * u.cross(v);
        v + self.w * t + u.cross(t)
    }

    /// Column-major rotation matrix (columns = where x, y, z end up).
    pub fn to_mat3(self) -> Mat3 {
        let Self { x, y, z, w } = self;
        Mat3::new(
            Vec3::new(
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y + w * z),
                2.0 * (x * z - w * y),
            ),
            Vec3::new(
                2.0 * (x * y - w * z),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z + w * x),
            ),
            Vec3::new(
                2.0 * (x * z + w * y),
                2.0 * (y * z - w * x),
                1.0 - 2.0 * (x * x + y * y),
            ),
        )
    }

    pub fn dot(self, q: Self) -> f32 {
        self.x * q.x + self.y * q.y + self.z * q.z + self.w * q.w
    }

    /// Spherical interpolation: constant angular speed from `self` (t = 0) to `q` (t = 1)
    /// along the shortest arc. This is the main reason to use quaternions for animation.
    pub fn slerp(self, mut q: Self, t: f32) -> Self {
        // q and -q are the same rotation; flip to take the short way around
        let mut d = self.dot(q);
        if d < 0.0 {
            q = Self {
                x: -q.x,
                y: -q.y,
                z: -q.z,
                w: -q.w,
            };
            d = -d;
        }

        // Nearly identical: fall back to a normalized lerp to avoid dividing by ~0
        let (a, b) = if d > 0.9995 {
            (1.0 - t, t)
        } else {
            let angle = d.acos();
            let s = angle.sin();
            (((1.0 - t) * angle).sin() / s, (t * angle).sin() / s)
        };

        Self {
            x: a * self.x + b * q.x,
            y: a * self.y + b * q.y,
            z: a * self.z + b * q.z,
            w: a * self.w + b * q.w,
        }
        .normalize()
    }
}

/// Hamilton product: `a * b` applies `b` first, then `a`.
impl ops::Mul for Quat {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        let a = self;
        Self {
            x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
            y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
            z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
            w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
        }
    }
}

impl ops::Mul<Vec3> for Quat {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        self.rotate(v)
    }
}

#[cfg(test)]
mod tests {
    use super::Quat;
    use crate::math::transform::{RotMat, RotX, RotY, RotZ};
    use crate::math::{Mat3, Vec3};
    use std::f32::consts::FRAC_PI_2;

    const TOL: f32 = 1e-5;

    fn mat_approx(a: Mat3, b: Mat3) -> bool {
        (0..3).all(|r| (0..3).all(|c| (a[(r, c)] - b[(r, c)]).abs() < TOL))
    }

    fn vec_approx(a: Vec3, b: Vec3) -> bool {
        (a - b).norm() < TOL
    }

    #[test]
    fn single_axis_matches_rotation_matrices() {
        let a = 0.7;
        let x = Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), a);
        let y = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), a);
        let z = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), a);
        assert!(mat_approx(x.to_mat3(), RotX(a)));
        assert!(mat_approx(y.to_mat3(), RotY(a)));
        assert!(mat_approx(z.to_mat3(), RotZ(a)));
    }

    #[test]
    fn from_euler_matches_rot_mat_and_composition() {
        let (phi, theta, psi) = (0.3, -0.8, 2.1);
        let q = Quat::from_euler(phi, theta, psi);
        assert!(mat_approx(q.to_mat3(), RotMat(phi, theta, psi)));

        // Same thing built by composing single-axis quaternions (x first, so rightmost)
        let composed = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), psi)
            * Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), theta)
            * Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), phi);
        assert!(mat_approx(composed.to_mat3(), q.to_mat3()));
    }

    #[test]
    fn rotate_matches_matrix() {
        let q = Quat::from_euler(0.3, -0.8, 2.1);
        let v = Vec3::new(1.0, -2.0, 0.5);
        assert!(vec_approx(q * v, q.to_mat3() * v));
        assert!(vec_approx(q.conjugate() * (q * v), v));
    }

    #[test]
    fn euler_round_trip() {
        let (phi, theta, psi) = (0.3, -0.8, 2.1);
        let (p, t, s) = Quat::from_euler(phi, theta, psi).to_euler();
        assert!((p - phi).abs() < TOL && (t - theta).abs() < TOL && (s - psi).abs() < TOL);
    }

    #[test]
    fn slerp_endpoints_and_midpoint() {
        let a = Quat::IDENTITY;
        let b = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), FRAC_PI_2);
        let mid = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), FRAC_PI_2 / 2.0);
        assert!(mat_approx(a.slerp(b, 0.0).to_mat3(), a.to_mat3()));
        assert!(mat_approx(a.slerp(b, 1.0).to_mat3(), b.to_mat3()));
        assert!(mat_approx(a.slerp(b, 0.5).to_mat3(), mid.to_mat3()));
    }
}
