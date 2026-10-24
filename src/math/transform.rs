use crate::math::{Mat3, Quat, Vec3};

#[derive(Debug)]
pub struct Pose {
    pub pos: Vec3,
    pub dir: Quat,
}

impl Default for Pose {
    fn default() -> Pose {
        Pose {
            pos: Vec3::ZERO,
            dir: Quat::IDENTITY,
        }
    }
}

impl Pose {
    pub fn with_position(p: Vec3) -> Self {
        Self {
            pos: p,
            dir: Quat::default(),
        }
    }
}

pub fn newell_normal(points: &Vec<&Vec3>) -> Vec3 {
    let mut n = Vec3::ZERO;

    for (i, p) in points.iter().enumerate() {
        let j = (i + 1) % points.len();
        let p_next = points[j];

        n.x += (p.y - p_next.y) * (p.z + p_next.z);
        n.y += (p.z - p_next.z) * (p.x + p_next.x);
        n.z += (p.x - p_next.x) * (p.y + p_next.y);
    }

    n /= n.norm();

    n
}

#[allow(non_snake_case)]
pub fn RotX(phi: f32) -> Mat3 {
    let c = f32::cos(phi);
    let s = f32::sin(phi);

    Mat3 {
        x_axis: Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        y_axis: Vec3 { x: 0.0, y: c, z: s },
        z_axis: Vec3 {
            x: 0.0,
            y: -s,
            z: c,
        },
    }
}

pub fn rotate_x(v: Vec3, phi: f32) -> Vec3 {
    RotX(phi) * v
}

#[allow(non_snake_case)]
pub fn RotY(theta: f32) -> Mat3 {
    let c = f32::cos(theta);
    let s = f32::sin(theta);

    Mat3 {
        x_axis: Vec3 {
            x: c,
            y: 0.0,
            z: -s,
        },
        y_axis: Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        z_axis: Vec3 { x: s, y: 0.0, z: c },
    }
}

pub fn rotate_y(v: Vec3, theta: f32) -> Vec3 {
    RotY(theta) * v
}

#[allow(non_snake_case)]
pub fn RotZ(psi: f32) -> Mat3 {
    let c = f32::cos(psi);
    let s = f32::sin(psi);

    Mat3 {
        x_axis: Vec3 { x: c, y: s, z: 0.0 },
        y_axis: Vec3 {
            x: -s,
            y: c,
            z: 0.0,
        },
        z_axis: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    }
}

pub fn rotate_z(v: Vec3, psi: f32) -> Vec3 {
    RotZ(psi) * v
}

// Applied in x, y, z order about the fixed world axes (rightmost matrix acts first)
#[allow(non_snake_case)]
pub fn RotMat(phi: f32, theta: f32, psi: f32) -> Mat3 {
    RotZ(psi) * RotY(theta) * RotX(phi)
}

pub fn rotate(v: Vec3, phi: f32, theta: f32, psi: f32) -> Vec3 {
    RotMat(phi, theta, psi) * v
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const X: Vec3 = Vec3::new(1.0, 0.0, 0.0);
    const Y: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    const Z: Vec3 = Vec3::new(0.0, 0.0, 1.0);

    fn approx_eq_vec(a: Vec3, b: Vec3) -> bool {
        (a - b).norm() < 1e-6
    }

    // A positive quarter turn about each axis maps the next two axes cyclically:
    // x: y -> z, y: z -> x, z: x -> y
    #[test]
    fn quarter_turns_follow_right_hand_rule() {
        assert!(approx_eq_vec(rotate_x(Y, FRAC_PI_2), Z));
        assert!(approx_eq_vec(rotate_x(Z, FRAC_PI_2), -Y));
        assert!(approx_eq_vec(rotate_y(Z, FRAC_PI_2), X));
        assert!(approx_eq_vec(rotate_y(X, FRAC_PI_2), -Z));
        assert!(approx_eq_vec(rotate_z(X, FRAC_PI_2), Y));
        assert!(approx_eq_vec(rotate_z(Y, FRAC_PI_2), -X));
    }

    #[test]
    fn combined_rotation_applies_x_then_y_then_z() {
        // x first leaves X alone, y sends X to -Z, z leaves -Z alone
        assert!(approx_eq_vec(rotate(X, FRAC_PI_2, FRAC_PI_2, 0.0), -Z));
        // y first sends X to -Z, z leaves it alone
        assert!(approx_eq_vec(rotate(X, 0.0, FRAC_PI_2, FRAC_PI_2), -Z));
        // x sends Y to Z, y sends Z to X, z sends X to Y
        assert!(approx_eq_vec(rotate(Y, FRAC_PI_2, FRAC_PI_2, FRAC_PI_2), Y));
    }
}
