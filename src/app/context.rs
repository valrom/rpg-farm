use std::default::Default;
use wgpu::{PowerPreference, RequestAdapterOptions, StoreOp};
use winit::window::Window;
use crate::app::{buffers, GameLogic};
use crate::app::buffers::{INDICES, Mesh, VERTICES};
use crate::app::camera::Camera;
use crate::app::matrix::MatrixUniform;
use crate::app::texture::Texture;

pub struct Context<'a> {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub pipeline: wgpu::RenderPipeline,
    pub mesh: Mesh,
    pub first_texture: Texture,
    pub second_texture: Texture,
    pub is_render_first: bool,
    pub camera: Camera,
    pub matrix_uniform: MatrixUniform,

    game_logic: &'a dyn GameLogic,
}

impl<'a> Context<'a> {
    pub async fn new(window: Window, game_logic: &'a dyn GameLogic) -> Context<'a> {
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

        let mesh = Mesh::new(&device, VERTICES, INDICES);
        let first_texture = Texture::new(
            include_bytes!("../../resources/stone.jpeg"),
            "stone_texture",
            &device,
            &queue
        );
        let second_texture = Texture::new(
            include_bytes!("../../resources/grass.jpeg"),
            "grass_texture",
            &device,
            &queue
        );

        let matrix_uniform = MatrixUniform::new(&device);

        let pipeline = Context::create_render_pipeline(
            &device,
            &config,
            first_shader,
            &[&first_texture.layout, &matrix_uniform.layout],
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
            mesh,
            is_render_first: false,
            first_texture,
            second_texture,
            camera,
            matrix_uniform,
            game_logic,
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
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

        println!("Render");

        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder")
            }
        );

        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        self.matrix_uniform.update(self.camera.calculate_matrix(), &self.queue);

        {
            let color_attachment = wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: StoreOp::Discard,
                },
            };

            let descriptor = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(color_attachment)
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            };

            let mut render_pass = encoder.begin_render_pass(&descriptor);
            self.game_logic.render(&mut render_pass, &self);

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
            buffers: &[buffers::Vertex::desc()],
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

        device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: vertex_state,
                fragment: Some(fragment_state),
                primitive: primitive_state,
                depth_stencil: None,
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