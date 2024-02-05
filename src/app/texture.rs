pub struct Texture {
    _texture: wgpu::Texture,
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn new(bytes: &[u8], name: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = Self::create_texture(bytes, name, device, queue);
        let layout = Self::create_bind_group_layout(device);
        let bind_group = Self::create_bind_group(device, &texture, &layout);

        Self {
            _texture: texture,
            layout,
            bind_group,
        }
    }

    fn create_texture(bytes: &[u8], name: &str, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let diffuse_bytes = bytes;
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
            label: Some(name),
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

    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

    fn create_bind_group(
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

#[allow(unused)]
pub struct DepthTexture {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl DepthTexture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT |
                wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        const CLAMP : wgpu::AddressMode = wgpu::AddressMode::ClampToEdge;

        let desc = wgpu::SamplerDescriptor {
            address_mode_u: CLAMP,
            address_mode_v: CLAMP,
            address_mode_w: CLAMP,

            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,

            mipmap_filter: wgpu::FilterMode::Nearest,

            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,

            ..Default::default()
        };

        let sampler = device.create_sampler(&desc);

        Self {
            texture,
            view,
            sampler,
        }
    }
}