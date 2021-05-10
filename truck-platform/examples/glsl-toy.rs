//! A sample of creating a render object by implementing "Rendered" in a new structure.
//!
//! One can use xyr fragment shader in the following way:
//! - Enter the shader path as an argument when executing the program.
//! - Drag and drop the shader into the window.
//!
//! The shader syntax follows that of shadertoy. One can use `iResolution`, `iTime` and `iMouse`.
//! Since this is a simple sample, not supports `iChannel`s, i.e. buffering textures, sounds, and so on.
//! The default shader sample is "newton-cuberoot.frag" in the same directory.

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

    /// Canvas to draw by fragment shader.
    pub struct Plane {
        vertex_module: ShaderModule,
        fragment_module: ShaderModule,
        pub mouse: [f32; 4],
        id: RenderID,
    }

    /// GLSL vertex shader of Plane
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

    /// prefix of GLSL fragment shader
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
        layout(set = 1, binding = 1) uniform Mouse {
            vec4 iMouse;
        };
    ";

    /// suffix of GLSL fragment shader
    const FRAGMENT_SHADER_SUFFIX: &str = "
        void main() {
            vec2 fragCoord = vec2(gl_FragCoord.x, iResolution.y - gl_FragCoord.y);
            mainImage(color, fragCoord);
        }    
    ";

    /// Reads the GLSL fragment shader with `void mainImage(out vec4 fragColor, in vec2 fragCoord)`.
    fn read_shader(
        device: &Device,
        code: &str,
        shadertype: ShaderType,
    ) -> Result<ShaderModule, String> {
        let mut spirv =
            glsl_to_spirv::compile(&code, shadertype).map_err(|error| format!("{:?}", error))?;
        let mut compiled = Vec::new();
        spirv
            .read_to_end(&mut compiled)
            .map_err(|error| format!("{:?}", error))?;
        Ok(device.create_shader_module(&ShaderModuleDescriptor{
            source: wgpu::util::make_spirv(&compiled),
            flags: ShaderFlags::VALIDATION,
            label: None,
        }))
    }

    // Implementation of Rendered for Plane.
    impl Rendered for Plane {
        // `Rendered::render_id()` can be implemented by macro.
        impl_render_id!(id);

        // Vertices: [0, 1, 2, 2, 1, 3] as [u32; 6].
        // There is not the index buffer.
        fn vertex_buffer(
            &self,
            handler: &DeviceHandler,
        ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
            let vertex_buffer = BufferHandler::from_slice(
                &[0 as u32, 1, 2, 2, 1, 3],
                handler.device(),
                BufferUsage::VERTEX,
            );
            (Arc::new(vertex_buffer), None)
        }

        // bind group is only one uniform buffer: resolution
        fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
            Arc::new(
                handler
                    .device()
                    .create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: None,
                        entries: &[
                            BindGroupLayoutEntry {
                                binding: 0,
                                visibility: ShaderStage::FRAGMENT,
                                ty: BindingType::Buffer {
                                    ty: BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            BindGroupLayoutEntry {
                                binding: 1,
                                visibility: ShaderStage::FRAGMENT,
                                ty: BindingType::Buffer {
                                    ty: BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                    }),
            )
        }
        // bind group is only one uniform buffer: resolution
        fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
            let sc_desc = handler.sc_desc();
            let resolution = [sc_desc.width as f32, sc_desc.height as f32, 1.0];
            Arc::new(bind_group_util::create_bind_group(
                handler.device(),
                layout,
                vec![
                    BufferHandler::from_slice(&resolution, handler.device(), BufferUsage::UNIFORM)
                        .binding_resource(),
                    BufferHandler::from_slice(&self.mouse, handler.device(), BufferUsage::UNIFORM)
                        .binding_resource(),
                ],
            ))
        }

        // Describe pipeline
        fn pipeline(
            &self,
            handler: &DeviceHandler,
            layout: &PipelineLayout,
            sample_count: u32,
        ) -> Arc<RenderPipeline> {
            let sc_desc = handler.sc_desc();
            Arc::new(
                handler
                    .device()
                    .create_render_pipeline(&RenderPipelineDescriptor {
                    layout: Some(layout),
                    vertex: VertexState {
                        module: &self.vertex_module,
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
                        module: &self.fragment_module,
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

    impl Plane {
        /// constructor
        /// # Arguments
        /// - device: Device, provided by wgpu.
        /// - shader: the inputed fragment shader
        pub fn new(device: &Device, shader: String) -> Plane {
            let vertex_module = read_shader(device, VERTEX_SHADER, ShaderType::Vertex).unwrap();
            let fragment_shader =
                FRAGMENT_SHADER_PREFIX.to_string() + &shader + FRAGMENT_SHADER_SUFFIX;
            let fragment_module =
                read_shader(device, &fragment_shader, ShaderType::Fragment).unwrap();
            Plane {
                vertex_module,
                fragment_module,
                mouse: [0.0; 4],
                id: RenderID::gen(),
            }
        }

        /// Change the shader of fragment.
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
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap()
    });

    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
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
    // Adds a plane to the scene!
    scene.add_object(&mut plane);

    let mut dragging = false;
    let mut clicked = false;
    let mut cursor = [0.0; 2];
    event_loop.run(move |ev, _, control_flow| {
        *control_flow = match ev {
            Event::MainEventsCleared => {
                window.request_redraw();
                ControlFlow::Poll
            }
            Event::RedrawRequested(_) => {
                scene.update_bind_group(&plane);
                let frame = swap_chain
                    .get_current_frame()
                    .expect("Timeout when acquiring next swap chain texture");
                scene.render_scene(&frame.output.view);
                if clicked {
                    plane.mouse[3] = -plane.mouse[3];
                    clicked = false;
                }
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
                WindowEvent::MouseInput { state, .. } => {
                    dragging = state == ElementState::Pressed;
                    clicked = dragging;
                    if dragging {
                        plane.mouse[0] = cursor[0];
                        plane.mouse[1] = cursor[1];
                        plane.mouse[2] = cursor[0];
                        plane.mouse[3] = cursor[1];
                    } else {
                        plane.mouse[2] = -plane.mouse[2];
                    }
                    ControlFlow::Poll
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let height = scene.sc_desc().height as f32;
                    cursor = [position.x as f32, height - position.y as f32];
                    if dragging {
                        plane.mouse[0] = cursor[0];
                        plane.mouse[1] = cursor[1];
                    }
                    ControlFlow::Poll
                }
                _ => ControlFlow::Poll,
            },
            _ => ControlFlow::Poll,
        };
    })
}
