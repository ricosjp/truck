use truck_featwgpu::*;
use truck_polymesh::{MeshHandler, PolygonMesh};
use winit::{
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

struct MyRender {
    scene: Scene,
    rotate_flag: bool,
    prev_cursor: Option<Vector2>,
}

impl MyRender {
    fn create_camera() -> Camera {
        let mut vec0 = vector!(1.5, 0.0, -1.5, 0.0);
        vec0 /= vec0.norm();
        let mut vec1 = vector!(-0.5, 1, -0.5, 0.0);
        vec1 /= vec1.norm();
        let mut vec2 = vector!(1, 1, 1, 0);
        vec2 /= vec2.norm();
        let vec3 = vector!(2, 2, 2, 1);
        let matrix = matrix!(vec0, vec1, vec2, vec3);
        Camera::perspective_camera(matrix, std::f64::consts::PI / 4.0, 0.1, 40.0)
    }

    fn obj_normalize(mut mesh: PolygonMesh) -> PolygonMesh {
        let bdd_box = mesh.bounding_box();
        let center = vector![
            (bdd_box[0].0 + bdd_box[0].1) / 2.0,
            (bdd_box[1].0 + bdd_box[1].1) / 2.0,
            (bdd_box[2].0 + bdd_box[2].1) / 2.0,
        ];
        let size_vector = vector![
            bdd_box[0].1 - bdd_box[0].0,
            bdd_box[1].1 - bdd_box[1].0,
            bdd_box[2].1 - bdd_box[2].0,
        ];
        let pre_size = if size_vector[0] < size_vector[1] {
            size_vector[1]
        } else {
            size_vector[0]
        };
        let size = if pre_size < size_vector[2] {
            size_vector[2]
        } else {
            pre_size
        };
        for pos in &mut mesh.positions {
            *pos -= &center;
            *pos /= size;
        }
        mesh
    }
    fn set_normals(mesh: PolygonMesh) -> PolygonMesh {
        match mesh.normals.is_empty() {
            false => mesh,
            true => {
                let mut mesh_handler = MeshHandler::new(mesh);
                mesh_handler
                    .put_together_same_attrs()
                    .add_smooth_normal(0.5);
                mesh_handler.into()
            }
        }
    }

    fn load_obj<P: AsRef<std::path::Path>>(path: P) -> PolygonMesh {
        let file = std::fs::File::open(path).unwrap();
        let mesh = truck_io::obj::read(file).unwrap();
        let mesh = MyRender::set_normals(mesh);
        MyRender::obj_normalize(mesh)
    }

    async fn run() {
        let event_loop = EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let mut sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let mut render = MyRender {
            scene: Scene::new(&device, &sc_desc),
            rotate_flag: false,
            prev_cursor: None,
        };
        render.scene.camera = MyRender::create_camera();
        render.scene.light = Light {
            position: vector!(2, 2, 2),
            strength: 2.0,
            light_type: LightType::Point,
        };

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::MainEventsCleared => window.request_redraw(),
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    sc_desc.width = size.width;
                    sc_desc.height = size.height;
                    swap_chain = device.create_swap_chain(&surface, &sc_desc);
                }
                Event::RedrawRequested(_) => {
                    render.scene.prepare_render(&device, &sc_desc);
                    let frame = swap_chain
                        .get_next_texture()
                        .expect("Timeout when acquiring next swap chain texture");
                    let mut encoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                    {
                        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                                attachment: &frame.view,
                                resolve_target: None,
                                load_op: wgpu::LoadOp::Clear,
                                store_op: wgpu::StoreOp::Store,
                                clear_color: wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                },
                            }],
                            depth_stencil_attachment: Some(
                                render.scene.depth_stencil_attachment_descriptor(),
                            ),
                        });
                        render.scene.render_scene(&mut rpass);
                    }
                    queue.submit(&[encoder.finish()]);
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::DroppedFile(path) => {
                        if render.scene.number_of_objects() != 0 {
                            render.scene.remove_object(0);
                        }
                        let mesh = MyRender::load_obj(path);
                        let mut glmesh: WGPUPolygonMesh = mesh.into();
                        glmesh.color = [1.0, 1.0, 1.0];
                        glmesh.reflect_ratio = [0.2, 0.6, 0.2];
                        render.scene.add_glpolymesh(&glmesh, &device);
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left {
                            render.rotate_flag = state == ElementState::Pressed;
                            if !render.rotate_flag {
                                render.prev_cursor = None;
                            }
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if render.rotate_flag {
                            let position = vector!(position.x, position.y);
                            if let Some(ref prev_position) = render.prev_cursor {
                                let dir2d = &position - prev_position;
                                let mut axis = dir2d[1] * &render.scene.camera.matrix()[0];
                                axis += dir2d[0] * &render.scene.camera.matrix()[1];
                                axis /= axis.norm();
                                let angle = dir2d.norm() * 0.01;
                                let mat =
                                    Matrix3::rotation(&axis.into(), angle).affine(&Vector3::zero());
                                render.scene.camera *= mat.inverse();
                            }
                            render.prev_cursor = Some(position);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    }
}

fn main() { futures::executor::block_on(MyRender::run()); }
