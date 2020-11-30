use glsl_to_spirv::ShaderType;
use std::io::Read;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use wgpu::*;

const PICTURE_WIDTH: u32 = 512;
const PICTURE_HEIGHT: u32 = 512;

#[derive(Clone, Default, Debug)]
struct Plane {
    id: RenderID,
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
        let vertex_module = read_shader(device, include_str!("picture.vert"), ShaderType::Vertex);
        let fragment_module =
            read_shader(device, include_str!("picture.frag"), ShaderType::Fragment);
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

fn read_shader(device: &Device, code: &str, shadertype: ShaderType) -> ShaderModule {
    let mut spirv = glsl_to_spirv::compile(&code, shadertype).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    device.create_shader_module(wgpu::util::make_spirv(&compiled))
}

async fn init_device(instance: &Instance) -> (Arc<Device>, Arc<Queue>) {
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
}

fn swap_chain_descriptor() -> SwapChainDescriptor {
    SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Bgra8UnormSrgb,
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        present_mode: PresentMode::Mailbox,
    }
}

fn extend3d() -> Extent3d {
    Extent3d {
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        depth: 1,
    }
}

fn texture_descriptor(sc_desc: &SwapChainDescriptor) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: extend3d(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: sc_desc.format,
        usage: sc_desc.usage | TextureUsage::COPY_SRC,
    }
}

fn output_buffer() -> BufferDescriptor<'static> {
    BufferDescriptor {
        label: None,
        usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
        mapped_at_creation: false,
        size: (PICTURE_WIDTH * PICTURE_HEIGHT * 4) as u64,
    }
}

fn texture_copy_view<'a>(texture: &'a Texture) -> TextureCopyView<'a> {
    TextureCopyView {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
    }
}

fn buffer_copy_view<'a>(buffer: &'a Buffer) -> BufferCopyView<'a> {
    BufferCopyView {
        buffer: &buffer,
        layout: TextureDataLayout {
            offset: 0,
            bytes_per_row: PICTURE_WIDTH * 4,
            rows_per_image: PICTURE_HEIGHT,
        },
    }
}

fn main() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = futures::executor::block_on(init_device(&instance));
    let sc_desc = swap_chain_descriptor();
    let texture = device.create_texture(&texture_descriptor(&sc_desc));
    let buffer = device.create_buffer(&output_buffer());
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let view = texture.create_view(&Default::default());
    let desc = SceneDescriptor {
        background: Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
        ..Default::default()
    };
    let mut scene = Scene::new(&device, &queue, &sc_desc, &desc);
    scene.add_object(&mut Plane::default());
    scene.prepare_render();
    scene.render_scene(&view);
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    let tc_view = texture_copy_view(&texture);
    let bc_view = buffer_copy_view(&buffer);
    encoder.copy_texture_to_buffer(tc_view, bc_view, extend3d());
    queue.submit(Some(encoder.finish()));
    let buffer_slice = buffer.slice(..);
    let buffer_future = buffer_slice.map_async(MapMode::Read);
    device.poll(Maintain::Wait);
    let vec: Vec<u8> = futures::executor::block_on(async {
        if let Ok(()) = buffer_future.await {
            buffer_slice.get_mapped_range().iter().map(|b| *b).collect()
        } else {
            panic!("failed to run compute on gpu!")
        }
    });
    image::save_buffer(
        "result.png",
        &vec,
        PICTURE_WIDTH,
        PICTURE_HEIGHT,
        image::ColorType::Rgba8,
    )
    .unwrap();
}
