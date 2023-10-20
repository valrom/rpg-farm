use std::default::Default;
use wgpu::{PowerPreference, RequestAdapterOptions};
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::app::buffers;
use crate::app::camera::{Camera, CameraUniform};

pub struct Context {
    window: Window,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    first_pipeline: wgpu::RenderPipeline,
    second_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,

    pub is_render_first: bool,
    pub camera: Camera,
    camera_uniform: CameraUniform,
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

        let second_shader = device.create_shader_module(
            wgpu::include_wgsl!("color_shader.wgsl")
        );

        let vertex_buffer = buffers::create_vertex_buffer(
            &device,
            bytemuck::cast_slice(buffers::VERTICES),
        );

        let index_buffer = buffers::create_index_buffer(
            &device,
            bytemuck::cast_slice(buffers::INDICES),
        );

        let texture = Self::create_texture(&device, &queue);
        let layout = Self::create_texture_bind_group_layout(&device);
        let camera_layout = CameraUniform::bind_group_layout(&device);

        let bind_group = Self::create_diffuse_bind_group(
            &device,
            &texture,
            &layout,
        );

        let first_pipeline = Context::create_render_pipeline(
            &device,
            &config,
            first_shader,
            &[&layout, &camera_layout]
        );
        let second_pipeline = Context::create_render_pipeline(
            &device,
            &config,
            second_shader,
            &[&layout, &camera_layout]
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

        let camera_uniform = CameraUniform::new(&device);


        Context {
            window,
            surface,
            device,
            queue,
            config,
            size,
            first_pipeline,
            second_pipeline,
            vertex_buffer,
            index_buffer,
            is_render_first: false,
            texture,
            bind_group,
            camera,
            camera_uniform
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
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        self.camera.aspect = self.config.width as f32 / self.config.height as f32;

        self.camera_uniform.update_matrix(&self.camera, &self.queue);

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
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: true,
                },
            };

            let descriptor = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(color_attachment)
                ],
                depth_stencil_attachment: None,
            };

            let mut render_pass = encoder.begin_render_pass(&descriptor);

            if self.is_render_first {
                render_pass.set_pipeline(&self.first_pipeline);
            } else {
                render_pass.set_pipeline(&self.second_pipeline);
            }

            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_uniform.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..buffers::INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
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

    fn create_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let diffuse_bytes = include_bytes!("stone.jpeg");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();

        let dimensions = diffuse_rgba.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };


        let desc = wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,

            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Stone texture"),
            view_formats: &[],
        };

        let texture = device.create_texture(&desc);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        texture
    }

    fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let fragment = wgpu::ShaderStages::FRAGMENT;

        let first_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: fragment,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        };

        let second_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: fragment,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };

        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[first_entry, second_entry],
            }
        )
    }

    fn create_diffuse_bind_group(
        device: &wgpu::Device,
        texture: &wgpu::Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let texture_view = texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let edge = wgpu::AddressMode::ClampToEdge;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: edge,
            address_mode_v: edge,
            address_mode_w: edge,

            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,

            mipmap_filter: wgpu::FilterMode::Nearest,

            ..Default::default()
        });

        let desc = wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }
            ],
            label: Some("Bind Group"),
        };

        device.create_bind_group(&desc)
    }
}

