use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 4],
    color: [f32; 4],
}


pub const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0, 1.0], color: [1.0, 0.0, 0.0, 1.0] },
    Vertex { position: [-0.5, -0.5, 0.0, 1.0], color: [0.0, 1.0, 0.0, 1.0] },
    Vertex { position: [0.5, -0.5, 0.0, 1.0], color: [0.0, 0.0, 1.0, 1.0] },
];

pub fn create_vertex_buffer(device: &wgpu::Device, slice: &[u8]) -> wgpu::Buffer {
    let desc = wgpu::util::BufferInitDescriptor {
        label: Some("Main buffer"),
        usage: wgpu::BufferUsages::VERTEX,
        contents: slice,
    };

    device.create_buffer_init(&desc)
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        static ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
            wgpu::VertexAttribute {
                offset: 0,
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                format: wgpu::VertexFormat::Float32x4,
                shader_location: 1,
            },
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}