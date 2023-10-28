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
    pub fn build_final_matrix(&self) -> cgmath::Matrix4<f32> {
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


pub struct CameraUniform {
    pub matrix: [[f32;4];4],
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl CameraUniform {
    pub fn new(device: &wgpu::Device) -> Self {
        use cgmath::SquareMatrix;

        let matrix = cgmath::Matrix4::identity().into();

        let bind_group_layout = Self::bind_group_layout(device);
        let buffer = Self::create_buffer(device, &matrix);
        let bind_group = Self::create_bind_group(
            device,
            &bind_group_layout,
            &buffer
        );

        Self {
            matrix,
            bind_group_layout,
            buffer,
            bind_group
        }
    }

    pub fn update_matrix(&mut self, camera: &Camera, queue: &wgpu::Queue) {
        self.matrix = camera.build_final_matrix().into();

        queue.write_buffer(
            &self.buffer,
            BufferAddress::default(),
            bytemuck::cast_slice(&self.matrix)
        );

        queue.submit([]);
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        static DESC : wgpu::BindGroupLayoutDescriptor = wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Camera Bind Group Layout"),
        };

        device.create_bind_group_layout(&DESC)
    }

    pub fn create_buffer(device: &wgpu::Device, matrix: &[[f32;4]; 4]) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(matrix),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    pub fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer
    ) -> wgpu::BindGroup {

        let desc = wgpu::BindGroupDescriptor {
            label: Some("Camera bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ]
        };

        device.create_bind_group(&desc)
    }
}

