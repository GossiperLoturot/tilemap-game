use super::{texture::UnitTextureResource, CameraResource, DepthResource};
use crate::service::Service;
use glam::*;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    texcoord: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: &[wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 3],
    shape: [f32; 4],
    texcoord: [f32; 4],
}

impl Instance {
    const ATTRIBUTES: &[wgpu::VertexAttribute] =
        &wgpu::vertex_attr_array![2 => Float32x3, 3 => Float32x4, 4 => Float32x4];

    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES,
        }
    }
}

pub struct UnitPipeline {
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
    index_buffer: wgpu::Buffer,
    texture_resource: UnitTextureResource,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl UnitPipeline {
    #[rustfmt::skip]
    const VERTICES: &[Vertex] = &[
        Vertex { position: [0.0, 0.0], texcoord: [0.0, 1.0] },
        Vertex { position: [1.0, 0.0], texcoord: [1.0, 1.0] },
        Vertex { position: [1.0, 1.0], texcoord: [1.0, 0.0] },
        Vertex { position: [0.0, 1.0], texcoord: [0.0, 0.0] },
    ];

    #[rustfmt::skip]
    const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        camera_resource: &CameraResource,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(Self::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: device.limits().max_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(Self::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture_resource = UnitTextureResource::new(device, queue);

        let shape_matrix = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 0.0),
        );

        let shape_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[shape_matrix]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: shape_matrix_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                camera_resource.bind_group_layout(),
                texture_resource.bind_group_layout(),
                &bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../../assets/shaders/unit.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::layout(), Instance::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthResource::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            vertex_buffer,
            instance_buffer,
            instance_count: 0,
            index_buffer,
            bind_group,
            texture_resource,
            pipeline,
        }
    }

    pub fn pre_draw(&mut self, queue: &wgpu::Queue, service: &Service) {
        if let Some(camera) = service.camera.get_camera() {
            let view_aabb = camera.view_aabb();

            let instances = service
                .unit
                .get_units(view_aabb)
                .into_iter()
                .map(|unit| {
                    let texcoord = self
                        .texture_resource
                        .get_texcoord(&unit.kind)
                        .unwrap_or_else(|| panic!("not registered unit kind {:?}", &unit.kind));

                    let position = unit.position.into();
                    let shape = unit.kind.shape().into();
                    let texcoord = texcoord.as_vec4().into();

                    Instance {
                        position,
                        shape,
                        texcoord,
                    }
                })
                .collect::<Vec<_>>();

            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
            self.instance_count = instances.len() as u32;
        }
    }

    pub fn draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_resouce: &'a CameraResource,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, camera_resouce.bind_group(), &[]);
        render_pass.set_bind_group(1, self.texture_resource.bind_group(), &[]);
        render_pass.set_bind_group(2, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..Self::INDICES.len() as u32, 0, 0..self.instance_count);
    }
}
