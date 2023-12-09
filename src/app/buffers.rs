use std::mem;
use std::ops::Range;
use wgpu::util::DeviceExt;
use crate::app::texture::Texture;

pub const VERTICES: &[Vertex] = &[
    Vertex { position: Position([-0.5, 0.5, 0.0]), tex_coords: UV([0.0, 0.0]) },
    Vertex { position: Position([0.5, 0.5, 0.0]), tex_coords: UV([1.0, 0.0]) },
    Vertex { position: Position([-0.5, -0.5, 0.0]), tex_coords: UV([0.0, 1.0]) },
    Vertex { position: Position([0.5, -0.5, 0.0]), tex_coords: UV([1.0, 1.0]) }
];

pub const INDICES: &[u16] = &[
    0, 1, 2,
    1, 2, 3
];


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: Position,
    tex_coords: UV,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Position([f32;3]);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UV([f32;2]);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32;4];4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
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
            bytemuck::cast_slice(vertices),
        );

        let index_buffer = Self::create_index_buffer(
            device,
            bytemuck::cast_slice(indices),
        );

        Self {
            vertex_buffer,
            index_buffer,
            len: indices.len() as u32,
        }
    }

    pub fn draw<'a, 'b>(&'a self, texture: &'a Texture, render_pass: &'b mut wgpu::RenderPass<'a>, range: Range<u32>) where 'a : 'b {

        let vertex_slice = self.vertex_buffer.slice(..);
        let index_slice = self.index_buffer.slice(..);

        render_pass.set_bind_group(0, &texture.bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_slice);
        render_pass.set_index_buffer(index_slice, wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.len, 0, range);
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
                offset: std::mem::size_of::<Position>() as wgpu::BufferAddress,
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