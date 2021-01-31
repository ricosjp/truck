//! Module for visualizing models.

use std::sync::{Arc, Mutex};
use std::time::*;
use truck_platform::{wgpu::*, *};
use truck_rendimpl::*;
use winit::dpi::*;
use winit::event::*;
use winit::event_loop::ControlFlow;

/// Shape Viewer
pub struct ShapeViewer {
    scene: Scene,
    rotate_flag: bool,
    prev_cursor: Option<Vector2>,
}

impl ShapeViewer {
    /// Initializes the application
    fn init<T: IntoInstance<Instance = ShapeInstance>>(
        handler: &DeviceHandler,
        info: AdapterInfo,
        shape: T,
    ) -> Self {
        let sample_count = match info.backend {
            Backend::Vulkan => 2,
            Backend::Dx12 => 2,
            _ => 1,
        };
        let scene_desc = SceneDescriptor {
            background: Color::BLACK,
            camera: create_camera(),
            lights: vec![Light {
                position: Point3::new(1.5, 1.5, 1.5),
                color: Vector3::new(1.0, 1.0, 1.0),
                light_type: LightType::Point,
            }],
            sample_count,
        };
        let mut scene = Scene::new(handler.clone(), &scene_desc);
        let inst_desc = InstanceDescriptor {
            material: Material {
                albedo: Vector4::new(0.75, 0.75, 0.75, 1.0),
                reflectance: 0.04,
                roughness: 0.9,
                ..Default::default()
            },
            ..Default::default()
        };
        let instance = scene.create_instance(&shape, &inst_desc);
        scene.add_objects(&instance.render_faces());
        ShapeViewer {
            scene,
            rotate_flag: false,
            prev_cursor: None,
        }
    }

    /// The default control flow
    fn default_control_flow() -> ControlFlow {
        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        ControlFlow::WaitUntil(next_frame_time)
    }

    /// Render scene
    fn render(&mut self, frame: &SwapChainFrame) { self.scene.render_scene(&frame.output.view); }

    /// Processing when a mouse click occurs.
    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> ControlFlow {
        match button {
            MouseButton::Left => {
                self.rotate_flag = state == ElementState::Pressed;
                if !self.rotate_flag {
                    self.prev_cursor = None;
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }

    /// Processing when the mouse wheel is moved.
    fn mouse_wheel(&mut self, delta: MouseScrollDelta, _: TouchPhase) -> ControlFlow {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                let sc_desc = self.scene.descriptor_mut();
                let camera = &mut sc_desc.camera;
                let light_position = &mut sc_desc.lights[0].position;
                let trans_vec = camera.eye_direction() * 0.2 * y as f64;
                camera.matrix = Matrix4::from_translation(trans_vec) * camera.matrix;
                *light_position = camera.matrix[3].to_point();
            }
            MouseScrollDelta::PixelDelta(_) => {}
        };
        Self::default_control_flow()
    }

    /// Processing when the cursor moved.
    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) -> ControlFlow {
        if self.rotate_flag {
            let sc_desc = self.scene.descriptor_mut();
            let matrix = &mut sc_desc.camera.matrix;
            let light_position = &mut sc_desc.lights[0].position;
            let position = Vector2::new(position.x, position.y);
            if let Some(ref prev_position) = self.prev_cursor {
                let dir2d = &position - prev_position;
                if dir2d.so_small() {
                    return Self::default_control_flow();
                }
                let mut axis = dir2d[1] * matrix[0].truncate();
                axis += dir2d[0] * &matrix[1].truncate();
                axis /= axis.magnitude();
                let angle = dir2d.magnitude() * 0.01;
                let mat = Matrix4::from_axis_angle(axis, Rad(-angle));
                *matrix = mat * *matrix;
                *light_position = matrix[3].to_point();
            }
            self.prev_cursor = Some(position);
        }
        Self::default_control_flow()
    }

    /// Running the shape viewer viewing `shape`.
    pub fn run<I: IntoInstance<Instance = ShapeInstance>>(shape: I) {
        let event_loop = winit::event_loop::EventLoop::new();
        let mut wb = winit::window::WindowBuilder::new();
        wb = wb.with_title("Shape Viewer");
        let window = wb.build(&event_loop).unwrap();
        let size = window.inner_size();
        let instance = Instance::new(BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        let (device, queue, info) = futures::executor::block_on(init_device(&instance, &surface));

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

        let mut app = Self::init(&handler, info, shape);

        event_loop.run(move |ev, _, control_flow| {
            *control_flow = match ev {
                Event::MainEventsCleared => {
                    window.request_redraw();
                    Self::default_control_flow()
                }
                Event::RedrawRequested(_) => {
                    let frame = swap_chain
                        .get_current_frame()
                        .expect("Timeout when acquiring next swap chain texture");
                    app.render(&frame);
                    Self::default_control_flow()
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        let mut sc_desc = handler.lock_sc_desc().unwrap();
                        sc_desc.width = size.width;
                        sc_desc.height = size.height;
                        swap_chain = handler.device().create_swap_chain(&surface, &sc_desc);
                        Self::default_control_flow()
                    }
                    WindowEvent::CloseRequested => ControlFlow::Exit,
                    WindowEvent::MouseInput { state, button, .. } => app.mouse_input(state, button),
                    WindowEvent::MouseWheel { delta, phase, .. } => app.mouse_wheel(delta, phase),
                    WindowEvent::CursorMoved { position, .. } => app.cursor_moved(position),
                    _ => Self::default_control_flow(),
                },
                _ => Self::default_control_flow(),
            };
        })
    }
}

async fn init_device(instance: &Instance, surface: &Surface) -> (Device, Queue, AdapterInfo) {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::Default,
            compatible_surface: Some(surface),
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let tuple = adapter
        .request_device(
            &DeviceDescriptor {
                features: Default::default(),
                limits: Limits::default(),
                shader_validation: true,
            },
            None,
        )
        .await
        .expect("Failed to create device");
    (tuple.0, tuple.1, adapter.get_info())
}

fn create_camera() -> Camera {
    let matrix = Matrix4::look_at_rh(
        Point3::new(1.5, 1.5, 1.5),
        Point3::origin(),
        Vector3::unit_y(),
    );
    Camera::perspective_camera(
        matrix.invert().unwrap(),
        Rad(std::f64::consts::PI / 4.0),
        0.1,
        40.0,
    )
}

#[allow(dead_code)]
fn main() {}
