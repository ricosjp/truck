#![allow(dead_code)]

use rayon::prelude::*;
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
        let vertex_buffer =
            BufferHandler::from_slice(&[0, 1, 2, 2, 1, 3], handler.device(), BufferUsages::VERTEX);
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
        scene_desc: &SceneDescriptor,
    ) -> Arc<RenderPipeline> {
        writeln!(&mut std::io::stderr(), "create pipeline").unwrap();
        let device = handler.device();
        let source = ShaderSource::Wgsl(self.shader.into());
        let module = device.create_shader_module(ShaderModuleDescriptor {
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
                        compilation_options: Default::default(),
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: self.fs_entpt,
                        targets: &[Some(ColorTargetState {
                            format: scene_desc.render_texture.format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        front_face: FrontFace::Ccw,
                        cull_mode: Some(Face::Back),
                        polygon_mode: PolygonMode::Fill,
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
                        count: scene_desc.backend_buffer.sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    label: None,
                    multiview: None,
                    cache: None,
                }),
        )
    }
}

pub fn init_device(backends: Backends) -> DeviceHandler {
    pollster::block_on(async {
        let instance = Instance::new(InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        writeln!(&mut std::io::stderr(), "{:?}", adapter.get_info()).unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Default::default(),
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        DeviceHandler::new(Arc::new(adapter), Arc::new(device), Arc::new(queue))
    })
}

pub fn render_one<R: Rendered>(scene: &mut Scene, object: &R) -> Vec<u8> {
    scene.add_object(object);
    let res = pollster::block_on(scene.render_to_buffer());
    scene.remove_object(object);
    res
}

pub fn same_buffer(vec0: &[u8], vec1: &[u8]) -> bool {
    vec0.par_iter()
        .zip(vec1)
        .all(move |(i, j)| std::cmp::max(i, j) - std::cmp::min(i, j) < 3)
}

pub fn os_alt_exec_test<F: Fn(Backends, &str)>(test: F) {
    let _ = env_logger::try_init();
    if cfg!(target_os = "windows") {
        test(Backends::VULKAN, "output/vulkan/");
        test(Backends::DX12, "output/dx12/");
    } else if cfg!(target_os = "macos") {
        test(Backends::METAL, "output/");
    } else {
        test(Backends::VULKAN, "output/");
    }
}
