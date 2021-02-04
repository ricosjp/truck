#![allow(dead_code)]

use glsl_to_spirv::ShaderType;
use std::io::Read;
use std::sync::Arc;
use truck_platform::*;
use wgpu::*;

#[derive(Clone, Debug)]
pub struct Plane<'a> {
    pub vertex_shader: &'a str,
    pub fragment_shader: &'a str,
    pub id: RenderID,
}

#[macro_export]
macro_rules! new_plane {
    ($vertex_shader: expr, $fragment_shader: expr) => {
        Plane {
            vertex_shader: include_str!($vertex_shader),
            fragment_shader: include_str!($fragment_shader),
            id: RenderID::gen(),
        }
    };
}

impl<'a> Rendered for Plane<'a> {
    impl_render_id!(id);
    fn vertex_buffer(
        &self,
        handler: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let buffer = BufferHandler::from_slice(
            &[0 as u32, 1, 2, 2, 1, 3],
            handler.device(),
            BufferUsage::VERTEX,
        );
        (Arc::new(buffer), None)
    }
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(bind_group_util::create_bind_group_layout(
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
    fn pipeline(
        &self,
        handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
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
                    sample_count,
                    sample_mask: !0,
                    alpha_to_coverage_enabled: false,
                    label: None,
                }),
        )
    }
}

pub fn render_one<R: Rendered>(scene: &mut Scene, texture: &Texture, object: &R) {
    scene.add_object(object);
    scene.render_scene(&texture.create_view(&Default::default()));
    scene.remove_object(object);
}

pub fn read_shader(device: &Device, code: &str, shadertype: ShaderType) -> ShaderModule {
    let mut spirv = glsl_to_spirv::compile(&code, shadertype).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    device.create_shader_module(wgpu::util::make_spirv(&compiled))
}

pub fn texture_descriptor(sc_desc: &SwapChainDescriptor) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        },
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

pub fn buffer_copy_view<'a>(buffer: &'a Buffer, size: (u32, u32)) -> BufferCopyView<'a> {
    BufferCopyView {
        buffer: &buffer,
        layout: TextureDataLayout {
            offset: 0,
            bytes_per_row: size.0 * 4,
            rows_per_image: size.1,
        },
    }
}

pub fn read_buffer(device: &Device, buffer: &Buffer) -> Vec<u8> {
    let buffer_slice = buffer.slice(..);
    let buffer_future = buffer_slice.map_async(MapMode::Read);
    device.poll(Maintain::Wait);
    futures::executor::block_on(async {
        match buffer_future.await {
            Ok(_) => buffer_slice.get_mapped_range().iter().map(|b| *b).collect(),
            Err(_) => panic!("failed to run compute on gpu!"),
        }
    })
}

pub fn read_texture(handler: &DeviceHandler, texture: &Texture) -> Vec<u8> {
    let (device, queue, sc_desc) = (handler.device(), handler.queue(), handler.sc_desc());
    let size = (sc_desc.width * sc_desc.height * 4) as u64;
    let buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        mapped_at_creation: false,
        usage: BufferUsage::COPY_DST | BufferUsage::MAP_READ,
        size,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        texture_copy_view(&texture),
        buffer_copy_view(&buffer, (sc_desc.width, sc_desc.height)),
        Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
    read_buffer(device, &buffer)
}

pub fn same_buffer(vec0: &Vec<u8>, vec1: &Vec<u8>) -> bool {
    vec0.into_iter()
        .zip(vec1)
        .all(move |(i, j)| std::cmp::max(i, j) - std::cmp::min(i, j) < 3)
}
