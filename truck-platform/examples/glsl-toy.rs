use std::io::Read;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use wgpu::*;
use winit::event::*;
use winit::event_loop::ControlFlow;

/// minimum example for implementing `Rendered`.
mod plane {
    use super::*;
    use glsl_to_spirv::ShaderType;
    pub struct Plane {
        vertex_module: ShaderModule,
        fragment_module: ShaderModule,
        id: RenderID,
    }

    const VERTEX_SHADER: &str = "
        #version 450
        layout(location = 0) in uint idx;
        const vec2 VERTICES[4] = vec2[](
            vec2(-1.0, -1.0),
            vec2(1.0, -1.0),
            vec2(-1.0, 1.0),
            vec2(1.0, 1.0)
        );

        void main() {
            gl_Position = vec4(VERTICES[idx], 0.0, 1.0);
        }
    ";

    const FRAGMENT_SHADER_PREFIX: &str = "
        #version 450
        layout(location = 0) out vec4 color;
        layout(set = 0, binding = 2) uniform SceneStatus {
            float iTime;
            uint _nlights;
        };
        layout(set = 1, binding = 0) uniform Resolution {
            vec3 iResolution;
        };
    ";

    const FRAGMENT_SHADER_SUFFIX: &str = "
        void main() {
            vec2 fragCoord = vec2(gl_FragCoord.x, iResolution.y - gl_FragCoord.y);
            mainImage(color, fragCoord);
        }    
    ";

    fn read_shader(
        device: &Device,
        code: &str,
        shadertype: ShaderType,
    ) -> Result<ShaderModule, String> {
        let mut spirv = glsl_to_spirv::compile(&code, shadertype).map_err(|error| format!("{:?}", error))?;
        let mut compiled = Vec::new();
        spirv.read_to_end(&mut compiled).map_err(|error| format!("{:?}", error))?;
        Ok(device.create_shader_module(wgpu::util::make_spirv(&compiled)))
    }

    impl Rendered for Plane {
        impl_get_set_id!(id);
        fn vertex_buffer(
            &self,
            handler: &DeviceHandler,
        ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
            let buffer = BufferHandler::from_slice(
                &[0, 1, 2, 2, 1, 3],
                handler.device(),
                BufferUsage::VERTEX,
            );
            (Arc::new(buffer), None)
        }
        fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
            Arc::new(
                handler
                    .device()
                    .create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStage::FRAGMENT,
                            ty: BindingType::UniformBuffer {
                                dynamic: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    }),
            )
        }
        fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
            let sc_desc = handler.sc_desc();
            let resolution = [sc_desc.width as f32, sc_desc.height as f32, 1.0];
            Arc::new(truck_platform::create_bind_group(
                handler.device(),
                layout,
                Some(
                    BufferHandler::from_slice(&resolution, handler.device(), BufferUsage::UNIFORM)
                        .binding_resource(),
                ),
            ))
        }
        fn pipeline(
            &self,
            handler: &DeviceHandler,
            layout: &PipelineLayout,
        ) -> Arc<RenderPipeline> {
            let sc_desc = handler.sc_desc();
            Arc::new(
                handler
                    .device()
                    .create_render_pipeline(&RenderPipelineDescriptor {
                        layout: Some(layout),
                        vertex_stage: ProgrammableStageDescriptor {
                            module: &self.vertex_module,
                            entry_point: "main",
                        },
                        fragment_stage: Some(ProgrammableStageDescriptor {
                            module: &self.fragment_module,
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

    impl Plane {
        pub fn new(device: &Device, shader: String) -> Plane {
            let vertex_module = read_shader(device, VERTEX_SHADER, ShaderType::Vertex).unwrap();
            let fragment_shader =
                FRAGMENT_SHADER_PREFIX.to_string() + &shader + FRAGMENT_SHADER_SUFFIX;
            let fragment_module =
                read_shader(device, &fragment_shader, ShaderType::Fragment).unwrap();
            Plane {
                vertex_module,
                fragment_module,
                id: Default::default(),
            }
        }

        pub fn set_shader(&mut self, device: &Device, shader: String) {
            let fragment_shader =
                FRAGMENT_SHADER_PREFIX.to_string() + &shader + FRAGMENT_SHADER_SUFFIX;
            let fragment_module = match read_shader(device, &fragment_shader, ShaderType::Fragment)
            {
                Ok(got) => got,
                Err(error) => {
                    println!("Failed to compile:\n{:?}", error);
                    return;
                }
            };
            self.fragment_module = fragment_module;
        }
    }
}
use plane::Plane;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut wb = winit::window::WindowBuilder::new();
    wb = wb.with_title("GLSL Toy");
    let window = wb.build(&event_loop).unwrap();
    let size = window.inner_size();
    let instance = Instance::new(BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let (device, queue) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap()
    });

    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Bgra8Unorm,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Mailbox,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
    let handler = DeviceHandler::new(
        Arc::new(device),
        Arc::new(queue),
        Arc::new(Mutex::new(sc_desc)),
    );
    let mut scene = Scene::new(handler.clone(), &Default::default());
    let mut plane = Plane::new(
        handler.device(),
        include_str!("newton-cuberoot.frag").to_string(),
    );
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        match std::fs::read_to_string(&args[1]) {
            Ok(code) => {
                plane.set_shader(handler.device(), code);
                scene.update_pipeline(&plane);
            }
            Err(error) => println!("{:?}", error),
        }
    }
    scene.add_object(&mut plane);

    event_loop.run(move |ev, _, control_flow| {
        *control_flow = match ev {
            Event::MainEventsCleared => {
                window.request_redraw();
                ControlFlow::Poll
            }
            Event::RedrawRequested(_) => {
                scene.update_bind_group(&plane);
                scene.prepare_render();
                let frame = swap_chain
                    .get_current_frame()
                    .expect("Timeout when acquiring next swap chain texture");
                scene.render_scene(&frame.output.view);
                ControlFlow::Poll
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    let mut sc_desc = handler.lock_sc_desc().unwrap();
                    sc_desc.width = size.width;
                    sc_desc.height = size.height;
                    swap_chain = handler.device().create_swap_chain(&surface, &sc_desc);
                    ControlFlow::Poll
                }
                WindowEvent::CloseRequested => ControlFlow::Exit,
                WindowEvent::DroppedFile(path) => {
                    match std::fs::read_to_string(path) {
                        Ok(code) => {
                            plane.set_shader(handler.device(), code);
                            scene.update_pipeline(&plane);
                        }
                        Err(error) => println!("{:?}", error),
                    }
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            },
            _ => ControlFlow::Poll,
        };
    })
}
