#![allow(dead_code)]

use rayon::prelude::*;
use std::convert::TryInto;
use std::io::Write;
use std::sync::Arc;
use truck_platform::*;
use wgpu::*;

#[derive(Clone, Debug)]
pub struct Plane<'a> {
    pub shader: &'a str,
    pub vs_entpt: &'a str,
    pub fs_entpt: &'a str,
    pub id: RenderID,
}

#[macro_export]
macro_rules! new_plane {
    ($shader: expr, $vs_endpt: expr, $fs_endpt: expr) => {
        Plane {
            shader: include_str!($shader),
            vs_entpt: $vs_endpt,
            fs_entpt: $fs_endpt,
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
        writeln!(&mut std::io::stderr(), "create vertex buffer").unwrap();
        let vertex_buffer = BufferHandler::from_slice(
            &[0 as u32, 1, 2, 2, 1, 3],
            handler.device(),
            BufferUsages::VERTEX,
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
        let (device, config) = (handler.device(), handler.config());
        let source = ShaderSource::Wgsl(self.shader.into());
        let module = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source,
        });
        Arc::new(
            handler
                .device()
                .create_render_pipeline(&RenderPipelineDescriptor {
                    layout: Some(layout),
                    vertex: VertexState {
                        module: &module,
                        entry_point: self.vs_entpt,
                        buffers: &[VertexBufferLayout {
                            array_stride: std::mem::size_of::<u32>() as BufferAddress,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &[VertexAttribute {
                                format: VertexFormat::Uint32,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: self.fs_entpt,
                        targets: &[ColorTargetState {
                            format: config.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        }],
                    }),
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
                    label: None,
                }),
        )
    }
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

pub fn render_one<R: Rendered>(scene: &mut Scene, texture: &Texture, object: &R) {
    scene.add_object(object);
    scene.render_scene(&texture.create_view(&Default::default()));
    scene.remove_object(object);
}

pub fn texture_descriptor(config: &SurfaceConfiguration) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: config.format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
    }
}

pub fn texture_copy_view<'a>(texture: &'a Texture) -> ImageCopyTexture<'a> {
    ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
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
    let (device, queue, config) = (handler.device(), handler.queue(), handler.config());
    let size = (config.width * config.height * 4) as u64;
    let buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        mapped_at_creation: false,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        size,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        texture_copy_view(&texture),
        buffer_copy_view(&buffer, (config.width, config.height)),
        Extent3d {
            width: config.width,
            height: config.height,
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
