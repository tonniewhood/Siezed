use crate::math::{Quat, Vec3, transform::Pose};

#[derive(Debug, Default)]
pub struct Camera {
    pose: Pose,
    /// Full vertical field of view, in degrees. Valid range is (0, 180).
    fov: f32,
}

impl Camera {
    pub fn new(start_pose: Pose, fov: f32) -> Self {
        Self {
            pose: start_pose,
            fov: fov.clamp(5.0, 175.0),
        }
    }

    pub fn transform(&mut self, translation: Vec3, rotaion: Quat) {
        self.pose.pos += translation;
        self.pose.dir = rotaion.conjugate();
    }

    pub fn pose(&self) -> &Pose {
        &self.pose
    }

    pub fn pos(&self) -> &Vec3 {
        &self.pose.pos
    }

    pub fn dir(&self) -> &Quat {
        &self.pose.dir
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.pose.pos = pos;
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov.clamp(5.0, 175.0);
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }
}
