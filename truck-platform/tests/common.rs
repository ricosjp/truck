#![allow(dead_code)]

use glsl_to_spirv::ShaderType;
use rayon::prelude::*;
use std::io::{Read, Write};
use std::sync::Arc;
use std::convert::TryInto;
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

pub fn init_device(instance: &Instance) -> (Arc<Device>, Arc<Queue>) {
    futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .await
            .unwrap();
        writeln!(&mut std::io::stderr(), "{:?}", adapter.get_info()).unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Default::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        (Arc::new(device), Arc::new(queue))
    })
}

impl<'a> Rendered for Plane<'a> {
    impl_render_id!(id);
    fn vertex_buffer(
        &self,
        handler: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        writeln!(&mut std::io::stderr(), "create vertex buffer").unwrap();
        let vertex_buffer = BufferHandler::from_slice(
            &[0 as u32, 1, 2, 2, 1, 3],
            handler.device(),
            BufferUsage::VERTEX,
        );
        (Arc::new(vertex_buffer), None)
    }
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        writeln!(&mut std::io::stderr(), "create bind group layout").unwrap();
        Arc::new(bind_group_util::create_bind_group_layout(
            handler.device(),
            &[],
        ))
    }
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        writeln!(&mut std::io::stderr(), "create bind group").unwrap();
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
        writeln!(&mut std::io::stderr(), "create pipeline").unwrap();
        let (device, sc_desc) = (handler.device(), handler.sc_desc());
        let vertex_module = read_shader(device, self.vertex_shader, ShaderType::Vertex);
        let fragment_module = read_shader(device, self.fragment_shader, ShaderType::Fragment);
        Arc::new(
            handler
                .device()
                .create_render_pipeline(&RenderPipelineDescriptor {
                    layout: Some(layout),
                    vertex: VertexState {
                        module: &vertex_module,
                        entry_point: "main",
                        buffers: &[VertexBufferLayout {
                            array_stride: std::mem::size_of::<u32>() as BufferAddress,
                            step_mode: InputStepMode::Vertex,
                            attributes: &[VertexAttribute {
                                format: VertexFormat::Uint32,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        front_face: FrontFace::Ccw,
                        cull_mode: Some(Face::Back),
                        polygon_mode: PolygonMode::Fill,
                        clamp_depth: false,
                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: MultisampleState {
                        count: sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(FragmentState {
                        module: &fragment_module,
                        entry_point: "main",
                        targets: &[ColorTargetState {
                            format: sc_desc.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrite::ALL,
                        }],
                    }),
                    label: None,
                }),
        )
    }
}

#[derive(Clone, Debug)]
pub struct WGSLPlane<'a> {
    pub vertex_shader: &'a str,
    pub fragment_shader: &'a str,
    pub id: RenderID,
}

#[macro_export]
macro_rules! new_wgsl_plane {
    ($vertex_shader: expr, $fragment_shader: expr) => {
        WGSLPlane {
            vertex_shader: include_str!($vertex_shader),
            fragment_shader: include_str!($fragment_shader),
            id: RenderID::gen(),
        }
    };
}

impl<'a> Rendered for WGSLPlane<'a> {
    impl_render_id!(id);
    fn vertex_buffer(
        &self,
        handler: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        writeln!(&mut std::io::stderr(), "create vertex buffer").unwrap();
        let vertex_buffer = BufferHandler::from_slice(
            &[0 as u32, 1, 2, 2, 1, 3],
            handler.device(),
            BufferUsage::VERTEX,
        );
        (Arc::new(vertex_buffer), None)
    }
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        writeln!(&mut std::io::stderr(), "create bind group layout").unwrap();
        Arc::new(bind_group_util::create_bind_group_layout(
            handler.device(),
            &[],
        ))
    }
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        writeln!(&mut std::io::stderr(), "create bind group").unwrap();
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
        writeln!(&mut std::io::stderr(), "create pipeline").unwrap();
        let (device, sc_desc) = (handler.device(), handler.sc_desc());
        let shader = String::from(self.vertex_shader) + self.fragment_shader;
        println!("{}", shader);
        let source = ShaderSource::Wgsl(shader.into());
        let module = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source,
            flags: ShaderFlags::VALIDATION,
        });
        println!("compile done!");
        Arc::new(
            handler
                .device()
                .create_render_pipeline(&RenderPipelineDescriptor {
                    layout: Some(layout),
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs_main",
                        buffers: &[VertexBufferLayout {
                            array_stride: std::mem::size_of::<u32>() as BufferAddress,
                            step_mode: InputStepMode::Vertex,
                            attributes: &[VertexAttribute {
                                format: VertexFormat::Uint32,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        front_face: FrontFace::Ccw,
                        cull_mode: Some(Face::Back),
                        polygon_mode: PolygonMode::Fill,
                        clamp_depth: false,
                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: MultisampleState {
                        count: sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: "fs_main",
                        targets: &[ColorTargetState {
                            format: sc_desc.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrite::ALL,
                        }],
                    }),
                    label: None,
                }),
        )
    }
}


pub fn render_one<R: Rendered>(scene: &mut Scene, texture: &Texture, object: &R) {
    println!("add plane");
    scene.add_object(object);
    println!("render plane");
    scene.render_scene(&texture.create_view(&Default::default()));
    println!("remove plane");
    scene.remove_object(object);
}

pub fn read_shader(device: &Device, code: &str, shadertype: ShaderType) -> ShaderModule {
    let mut spirv = glsl_to_spirv::compile(&code, shadertype).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    device.create_shader_module(&ShaderModuleDescriptor {
        source: wgpu::util::make_spirv(&compiled),
        flags: ShaderFlags::VALIDATION,
        label: None,
    })
}

pub fn texture_descriptor(sc_desc: &SwapChainDescriptor) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: sc_desc.format,
        usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    }
}

pub fn texture_copy_view<'a>(texture: &'a Texture) -> ImageCopyTexture<'a> {
    ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
    }
}

pub fn buffer_copy_view<'a>(buffer: &'a Buffer, size: (u32, u32)) -> ImageCopyBuffer<'a> {
    ImageCopyBuffer {
        buffer: &buffer,
        layout: ImageDataLayout {
            offset: 0,
            bytes_per_row: (size.0 * 4).try_into().ok(),
            rows_per_image: size.1.try_into().ok(),
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
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
    read_buffer(device, &buffer)
}

pub fn same_buffer(vec0: &Vec<u8>, vec1: &Vec<u8>) -> bool {
    vec0.par_iter()
        .zip(vec1)
        .all(move |(i, j)| std::cmp::max(i, j) - std::cmp::min(i, j) < 3)
}
