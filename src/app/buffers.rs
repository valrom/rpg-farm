use wgpu::util::DeviceExt;

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }
];

pub const INDICES: &[u16] = &[
    0, 1, 2,
    1, 2, 3
];


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    len: u32,
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u16]) -> Self {

        let vertex_buffer = Self::create_vertex_buffer(
            device,
            bytemuck::cast_slice(vertices)
        );

        let index_buffer = Self::create_index_buffer(
            device,
            bytemuck::cast_slice(indices)
        );

        Self {
            vertex_buffer,
            index_buffer,
            len: indices.len() as u32,
        }
    }

    fn create_vertex_buffer(device: &wgpu::Device, slice: &[u8]) -> wgpu::Buffer {
        let desc = wgpu::util::BufferInitDescriptor {
            label: Some("Main buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: slice,
        };

        device.create_buffer_init(&desc)
    }

    fn create_index_buffer(device: &wgpu::Device, slice: &[u8]) -> wgpu::Buffer {
        let desc = wgpu::util::BufferInitDescriptor {
            label: Some("Main index buffer"),
            usage: wgpu::BufferUsages::INDEX,
            contents: slice,
        };

        device.create_buffer_init(&desc)
    }
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        static ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
            wgpu::VertexAttribute {
                offset: 0,
                format: wgpu::VertexFormat::Float32x3,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                format: wgpu::VertexFormat::Float32x2,
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

pub trait DrawMesh<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh);
}

impl<'a, 'b> DrawMesh<'b> for wgpu::RenderPass<'a>
where 'b : 'a {
    fn draw_mesh(&mut self, mesh: &'b Mesh) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(
            mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16
        );
        self.draw_indexed(0..mesh.len, 0, 0..1);
    }
}