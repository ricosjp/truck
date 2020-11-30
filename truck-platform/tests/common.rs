use glsl_to_spirv::ShaderType;
use std::convert::TryInto;
use std::io::Read;
use std::sync::Arc;
use truck_platform::*;
use wgpu::*;

pub const PICTURE_WIDTH: u32 = 512;
pub const PICTURE_HEIGHT: u32 = 512;

#[derive(Clone, Default, Debug)]
pub struct Plane {
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
    pub id: RenderID,
}

impl Rendered for Plane {
    impl_get_set_id!(id);
    fn vertex_buffer(
        &self,
        handler: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>)
    {
        let buffer = BufferHandler::from_slice(
            &[0 as u32, 1, 2, 2, 1, 3],
            handler.device(),
            BufferUsage::VERTEX,
        );
        (Arc::new(buffer), None)
    }
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(truck_platform::create_bind_group_layout(
            handler.device(),
            &[],
        ))
    }
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        Arc::new(handler.device().create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[],
        }))
    }
    fn pipeline(&self, handler: &DeviceHandler, layout: &PipelineLayout) -> Arc<RenderPipeline> {
        let (device, sc_desc) = (handler.device(), handler.sc_desc());
        let vertex_module = read_shader(device, self.vertex_shader, ShaderType::Vertex);
        let fragment_module = read_shader(device, self.fragment_shader, ShaderType::Fragment);
        Arc::new(
            handler
                .device()
                .create_render_pipeline(&RenderPipelineDescriptor {
                    layout: Some(layout),
                    vertex_stage: ProgrammableStageDescriptor {
                        module: &vertex_module,
                        entry_point: "main",
                    },
                    fragment_stage: Some(ProgrammableStageDescriptor {
                        module: &fragment_module,
                        entry_point: "main",
                    }),
                    rasterization_state: Some(RasterizationStateDescriptor {
                        front_face: FrontFace::Ccw,
                        cull_mode: CullMode::None,
                        depth_bias: 0,
                        depth_bias_slope_scale: 0.0,
                        depth_bias_clamp: 0.0,
                        clamp_depth: false,
                    }),
                    primitive_topology: PrimitiveTopology::TriangleList,
                    color_states: &[ColorStateDescriptor {
                        format: sc_desc.format,
                        color_blend: BlendDescriptor::REPLACE,
                        alpha_blend: BlendDescriptor::REPLACE,
                        write_mask: ColorWrite::ALL,
                    }],
                    depth_stencil_state: Some(DepthStencilStateDescriptor {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: StencilStateDescriptor {
                            front: StencilStateFaceDescriptor::IGNORE,
                            back: StencilStateFaceDescriptor::IGNORE,
                            read_mask: 0,
                            write_mask: 0,
                        },
                    }),
                    vertex_state: VertexStateDescriptor {
                        index_format: IndexFormat::Uint32,
                        vertex_buffers: &[VertexBufferDescriptor {
                            stride: std::mem::size_of::<u32>() as BufferAddress,
                            step_mode: InputStepMode::Vertex,
                            attributes: &[VertexAttributeDescriptor {
                                format: VertexFormat::Uint,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    sample_count: 1,
                    sample_mask: !0,
                    alpha_to_coverage_enabled: false,
                    label: None,
                }),
        )
    }
}

pub fn read_shader(device: &Device, code: &str, shadertype: ShaderType) -> ShaderModule {
    let mut spirv = glsl_to_spirv::compile(&code, shadertype).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    device.create_shader_module(wgpu::util::make_spirv(&compiled))
}

pub fn init_device(instance: &Instance) -> (Arc<Device>, Arc<Queue>) {
    futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: None,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();
        (Arc::new(device), Arc::new(queue))
    })
}

pub fn swap_chain_descriptor() -> SwapChainDescriptor {
    SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Bgra8UnormSrgb,
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        present_mode: PresentMode::Mailbox,
    }
}

pub fn extend3d() -> Extent3d {
    Extent3d {
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        depth: 1,
    }
}

pub fn texture_descriptor(sc_desc: &SwapChainDescriptor) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: extend3d(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: sc_desc.format,
        usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::COPY_SRC,
    }
}

pub fn texture_copy_view<'a>(texture: &'a Texture) -> TextureCopyView<'a> {
    TextureCopyView {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
    }
}

pub fn buffer_copy_view<'a>(buffer: &'a Buffer) -> BufferCopyView<'a> {
    BufferCopyView {
        buffer: &buffer,
        layout: TextureDataLayout {
            offset: 0,
            bytes_per_row: PICTURE_WIDTH * 4,
            rows_per_image: PICTURE_HEIGHT,
        },
    }
}

pub fn compare_texture(handler: &DeviceHandler, texture0: &Texture, texture1: &Texture) -> bool {
    let (device, queue) = (handler.device(), handler.queue());
    let buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        mapped_at_creation: false,
        usage: BufferUsage::STORAGE | BufferUsage::MAP_READ,
        size: (PICTURE_WIDTH * PICTURE_HEIGHT * 4) as u64,
    });
    let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::COMPUTE,
                ty: BindingType::StorageTexture {
                    dimension: TextureViewDimension::D2,
                    format: TextureFormat::Rgba8UnormSrgb,
                    readonly: true,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStage::COMPUTE,
                ty: BindingType::StorageTexture {
                    dimension: TextureViewDimension::D2,
                    format: TextureFormat::Rgba8UnormSrgb,
                    readonly: true,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStage::COMPUTE,
                ty: BindingType::StorageBuffer {
                    dynamic: false,
                    min_binding_size: None,
                    readonly: true,
                },
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&texture0.create_view(&Default::default())),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture1.create_view(&Default::default())),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::Buffer(buffer.slice(..)),
            },
        ],
    });
    let compute_shader = read_shader(
        device,
        include_str!("shaders/compare.comp"),
        ShaderType::Compute,
    );
    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: None,
        layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        })),
        compute_stage: ProgrammableStageDescriptor {
            module: &compute_shader,
            entry_point: "main",
        },
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    let mut cpass = encoder.begin_compute_pass();
    cpass.set_bind_group(0, &bind_group, &[]);
    cpass.set_pipeline(&pipeline);
    cpass.dispatch(PICTURE_WIDTH, PICTURE_HEIGHT, 1);
    drop(cpass);
    queue.submit(Some(encoder.finish()));
    let buffer_slice = buffer.slice(..);
    let buffer_future = buffer_slice.map_async(MapMode::Read);
    device.poll(Maintain::Wait);
    let vec: Vec<u32> = futures::executor::block_on(async {
        if let Ok(()) = buffer_future.await {
            let data = buffer_slice.get_mapped_range();
            data.chunks_exact(4)
                .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
                .collect()
        } else {
            panic!("failed to run compute on gpu!")
        }
    });
    vec.into_iter().all(|i| i == 0)
}
