use crate::model::*;
use ahash::AHashMap;
use glam::*;
use strum::IntoEnumIterator;

pub struct UnitTextureResource {
    texcoords: AHashMap<UnitKind, Aabb2>,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl UnitTextureResource {
    const SIZE: u32 = 1024;
    const UNIT_SIZE: u32 = 32;

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let grid = Self::SIZE / Self::UNIT_SIZE;
        let mip_level_count = Self::UNIT_SIZE.ilog2();

        let mut atlas = vec![];
        let mut texcoords = AHashMap::new();

        for mip_level in 0..mip_level_count {
            let size = Self::SIZE >> mip_level;
            atlas.push(image::DynamicImage::new_rgba8(size, size));
        }

        // TODO: improve packing algorithm
        // allocate texture to atlas by strip packing algolithm
        let (mut x, mut y, mut y_upper_bounds) = (0, 0, 0);
        for unit_kind in UnitKind::iter() {
            let texture = unit_kind.texture();
            let (size_x, size_y) = unit_kind.texture_size();

            if grid < x + size_x && grid < y + size_y {
                panic!("Atlas texture size is too small!");
            }

            for mip_level in 0..mip_level_count {
                let unit_size = Self::UNIT_SIZE >> mip_level;

                let texture = texture.resize_exact(
                    unit_size * size_x,
                    unit_size * size_y,
                    image::imageops::FilterType::Triangle,
                );

                image::imageops::replace(
                    &mut atlas[mip_level as usize],
                    &texture,
                    (unit_size * x) as i64,
                    (unit_size * y) as i64,
                );
            }

            texcoords.insert(
                unit_kind,
                Aabb2::from_element(
                    x as f32 / grid as f32,
                    y as f32 / grid as f32,
                    (x + size_x) as f32 / grid as f32,
                    (y + size_y) as f32 / grid as f32,
                ),
            );

            x += size_x;
            y_upper_bounds = y_upper_bounds.max(size_y);
            if grid <= x {
                (x, y, y_upper_bounds) = (0, y_upper_bounds, 0);
            }
        }

        let atlas_data = atlas
            .into_iter()
            .flat_map(|atlas| atlas.to_rgba8().to_vec())
            .collect::<Vec<_>>();

        use wgpu::util::DeviceExt;
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: Self::SIZE,
                    height: Self::SIZE,
                    depth_or_array_layers: 1,
                },
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &atlas_data,
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        Self {
            texcoords,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn get_texcoord(&self, unit_kind: &UnitKind) -> Option<Aabb2> {
        self.texcoords.get(unit_kind).cloned()
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}