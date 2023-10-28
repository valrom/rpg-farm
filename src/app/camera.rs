use wgpu::BufferAddress;
use wgpu::util::DeviceExt;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0
);

impl Camera {
    pub fn calculate_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let proj = cgmath::perspective(
            cgmath::Deg(self.fov_y),
                self.aspect,
                self.z_near,
                self.z_far
        );

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn set_angles(&mut self, coords: cgmath::Point2<f32>) {
        self.eye = cgmath::Point3::new(
            5.0 * coords.x.cos(),
            0.0,
            5.0 * coords.x.sin(),
        );
    }
}
