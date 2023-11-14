use std::default::Default;
use std::fs::File;
use std::io::Read;
use wgpu::{PowerPreference, RequestAdapterOptions, StoreOp};
use winit::window::Window;
use crate::app::{GameLogic, texture};
use crate::app::buffers::{Mesh, Vertex};
use crate::app::camera::Camera;
use crate::app::matrix::MatrixUniform;
use crate::app::texture::{DepthTexture, Texture};


pub struct DrawCall {
    pub mesh_id: usize,
    pub texture_id: usize,
    pub matrix: cgmath::Matrix4<f32>,
}

pub struct Renderer<'a> {
    context: &'a mut Context,
    draw_calls: Vec<DrawCall>,
}

impl<'a> Renderer<'a> {
    fn new<'b>(context: &'b mut Context) -> Renderer<'a> where 'b : 'a {
        Renderer {
            context,
            draw_calls: vec![],
        }
    }

    pub fn add_texture(&mut self, filepath: &str) -> Option<usize> {

        let mut f = File::open(filepath).ok()?;
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer).ok()?;

        let texture = Texture::new(
            buffer.as_slice(),
            "Texture",
            &self.context.device,
            &self.context.queue
        );


        self.context.textures.push(texture);
        Some(self.context.textures.len() - 1)
    }

    pub fn add_mesh(&mut self, vertices: &[Vertex], indices: &[u16]) -> usize {
        let mesh = Mesh::new(
            &self.context.device,
            bytemuck::cast_slice(vertices),
            bytemuck::cast_slice(indices)
        );

        self.context.meshes.push(mesh);
        self.context.meshes.len() - 1
    }

    pub fn draw(&mut self, draw_call: DrawCall) {
        self.draw_calls.push(draw_call);
    }
}

pub struct Context {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    pub camera: Camera,

    meshes: Vec<Mesh>,
    textures: Vec<Texture>,

    depth_texture: DepthTexture,
}

impl Context {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),

                ..Default::default()
            },
            None,
        )
            .await
            .expect("Failed to request device");

        let config = surface.get_default_config(&adapter, size.width, size.height).unwrap();

        surface.configure(&device, &config);

        let first_shader = device.create_shader_module(
            wgpu::include_wgsl!("shader.wgsl")
        );

        let matrix_uniform = MatrixUniform::new(&device);

        let depth_texture = DepthTexture::new(&device, &config, "depth texture");

        let pipeline = Context::create_render_pipeline(
            &device,
            &config,
            first_shader,
            &[&Texture::create_bind_group_layout(&device), &matrix_uniform.layout],
        );

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0,
        };

        Context {
            window,
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            camera,
            meshes: vec![],
            textures: vec![],
            depth_texture,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.surface.configure(&self.device, &self.config);
            self.depth_texture = DepthTexture::new(
                &self.device,
                &self.config,
                "depth texture"
            );
        }
    }

    pub fn init(&mut self, game_logic: &mut dyn GameLogic) {
        game_logic.init(&mut Renderer::new(self));
    }

    pub fn render(&mut self, game_logic: &mut dyn GameLogic) -> Result<(), wgpu::SurfaceError> {

        let draw_calls = {
            let mut renderer = Renderer::new(self);
            game_logic.render(&mut renderer);
            renderer.draw_calls
        };

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );



        self.camera.aspect = self.config.width as f32 / self.config.height as f32;


        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder")
            }
        );

        {
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: StoreOp::Store,
                },
            };

            let depth_stencil_attachment = wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(
                    wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    },
                ),
                stencil_ops: None,
            };

            let descriptor = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(color_attachment)
                ],
                depth_stencil_attachment: Some(depth_stencil_attachment),
                occlusion_query_set: None,
                timestamp_writes: None,
            };

            let mut uniforms = Vec::new();

            for draw_call in draw_calls.iter() {
                let total_matrix = self.camera.calculate_matrix() * draw_call.matrix;
                let mut uniform = MatrixUniform::new(&self.device);
                uniform.update(total_matrix, &self.queue);

                uniforms.push(
                    uniform
                );
            }

            let mut render_pass = encoder.begin_render_pass(&descriptor);
            render_pass.set_pipeline(&self.pipeline);


            for (draw_call, uniform) in std::iter::zip(draw_calls.iter(), uniforms.iter()) {

                let texture = &self.textures[draw_call.texture_id];
                let mesh = &self.meshes[draw_call.mesh_id];

                render_pass.set_bind_group(1, &uniform.bind_group, &[]);
                mesh.draw(texture, &mut render_pass);
            }
        }

        self.queue.submit(Some(encoder.finish()));

        output.present();

        Ok(())
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shader: wgpu::ShaderModule,
        bind_groups: &[&wgpu::BindGroupLayout],
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline"),
                bind_group_layouts: bind_groups,
                push_constant_ranges: &[],
            }
        );

        let target_state = wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let vertex_state = wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        };

        let fragment_state = wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(target_state)],
        };

        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        };

        let depth_stencil = wgpu::DepthStencilState {
            format: DepthTexture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: vertex_state,
                fragment: Some(fragment_state),
                primitive: primitive_state,
                depth_stencil: Some(depth_stencil),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }
        )
    }
}