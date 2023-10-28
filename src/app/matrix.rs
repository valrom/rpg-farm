use wgpu::BufferAddress;
use wgpu::util::DeviceExt;

pub struct MatrixUniform {
    pub matrix: [[f32;4];4],
    pub layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl MatrixUniform {
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
            layout: bind_group_layout,
            buffer,
            bind_group
        }
    }

    pub fn update(&mut self, matrix: cgmath::Matrix4<f32>, queue: &wgpu::Queue) {
        let matrix: [[f32; 4]; 4] = matrix.into();

        queue.write_buffer(
            &self.buffer,
            BufferAddress::default(),
            bytemuck::cast_slice(&matrix)
        );

        queue.submit([]);
    }

    fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

    fn create_buffer(device: &wgpu::Device, matrix: &[[f32;4]; 4]) -> wgpu::Buffer {
        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(matrix),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        )
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer
    ) -> wgpu::BindGroup {

        let desc = wgpu::BindGroupDescriptor {
            label: Some("Matrix bind group"),
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