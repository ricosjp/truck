// The texture is referenced by:
// https://cc0textures.com/view?id=WoodFloor024

use truck_modeling::*;
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;
use winit::{dpi::*, event::*, event_loop::ControlFlow};
mod app;
use app::*;
use std::sync::Arc;

struct MyApp {
    scene: Scene,
    rotate_flag: bool,
    prev_cursor: Option<Vector2>,
    light_changed: Option<std::time::Instant>,
    camera_changed: Option<std::time::Instant>,
}

impl MyApp {
    fn create_camera() -> Camera {
        let matrix = Matrix4::look_at(
            Point3::new(1.0, 1.0, 1.0),
            Point3::origin(),
            Vector3::unit_y(),
        );
        Camera::perspective_camera(
            matrix.invert().unwrap(),
            std::f64::consts::PI / 4.0,
            0.1,
            40.0,
        )
    }

    fn create_cube() -> Shell {
        let v = builder::vertex(Point3::origin());
        let edge = builder::tsweep(&v, Vector3::unit_x());
        let face = builder::tsweep(&edge, Vector3::unit_y());
        let solid = builder::tsweep(&face, Vector3::unit_z());
        solid.into_boundaries()[0]
    }

    fn signup_cube(&mut self) {
        let cube = Self::create_cube();
        let texture = image::load_from_memory(include_bytes!("WoodFloor024_2K_Color.png")).unwrap();
        let desc = InstanceDescriptor {
            matrix: Matrix4::from_translation(Vector3::new(-0.5, -0.5, -0.5)),
            material: Material {
                albedo: Vector4::new(0.402, 0.262, 0.176, 1.0),
                roughness: 0.9,
                reflectance: 0.04,
            },
            texture: Some(Arc::new(texture)),
            backface_culling: true,
        };
        let mut mesh = self.scene.create_instance(&cube, &desc);
        self.scene.add_objects::<shaperend::FaceInstanceMut, &mut ShapeInstance>(&mut mesh);
    }
}

impl App for MyApp {
    fn init(handler: &WGPUHandler) -> MyApp {
        let (device, queue, sc_desc) = (&handler.device, &handler.queue, &handler.sc_desc);
        let desc = SceneDescriptor {
            camera: MyApp::create_camera(),
            lights: vec![Light {
                position: Point3::new(1.0, 1.0, 1.0),
                color: Vector3::new(1.0, 1.0, 1.0),
                light_type: LightType::Point,
            }],
            ..Default::default()
        };
        let mut render = MyApp {
            scene: Scene::new(device, queue, sc_desc, &desc),
            rotate_flag: false,
            prev_cursor: None,
            camera_changed: None,
            light_changed: None,
        };
        render.signup_cube();
        render
    }

    fn app_title<'a>() -> Option<&'a str> { Some("simple obj viewer") }

    fn depth_stencil_attachment_descriptor<'a>(
        &'a self,
    ) -> Option<RenderPassDepthStencilAttachmentDescriptor<'a>> {
        Some(self.scene.depth_stencil_attachment_descriptor())
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> ControlFlow {
        match button {
            MouseButton::Left => {
                self.rotate_flag = state == ElementState::Pressed;
                if !self.rotate_flag {
                    self.prev_cursor = None;
                }
            }
            MouseButton::Right => {
                let scene = &mut self.scene;
                match scene.lights[0].light_type {
                    LightType::Point => {
                        scene.lights[0].position = scene.camera.position();
                    }
                    LightType::Uniform => {
                        scene.lights[0].position = scene.camera.position();
                        let strength = scene.lights[0].position.to_vec().magnitude();
                        scene.lights[0].position /= strength;
                    }
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }
    fn mouse_wheel(&mut self, delta: MouseScrollDelta, _: TouchPhase) -> ControlFlow {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                let trans_vec = self.scene.camera.eye_direction() * 0.2 * y as f64;
                self.scene.camera.matrix =
                    Matrix4::from_translation(trans_vec) * self.scene.camera.matrix;
            }
            MouseScrollDelta::PixelDelta(_) => {}
        };
        Self::default_control_flow()
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) -> ControlFlow {
        if self.rotate_flag {
            let position = Vector2::new(position.x, position.y);
            if let Some(ref prev_position) = self.prev_cursor {
                let dir2d = &position - prev_position;
                let mut axis = dir2d[1] * &self.scene.camera.matrix[0].truncate();
                axis += dir2d[0] * &self.scene.camera.matrix[1].truncate();
                axis /= axis.magnitude();
                let angle = dir2d.magnitude() * 0.01;
                let mat = Matrix4::from_axis_angle(axis, cgmath::Rad(angle));
                self.scene.camera.matrix = mat.invert().unwrap() * self.scene.camera.matrix;
            }
            self.prev_cursor = Some(position);
        }
        Self::default_control_flow()
    }
    fn keyboard_input(&mut self, input: KeyboardInput, _: bool) -> ControlFlow {
        let keycode = match input.virtual_keycode {
            Some(keycode) => keycode,
            None => return Self::default_control_flow(),
        };
        match keycode {
            VirtualKeyCode::P => {
                if let Some(ref instant) = self.camera_changed {
                    let time = instant.elapsed().as_secs_f64();
                    if time < 0.2 {
                        return Self::default_control_flow();
                    }
                }
                self.camera_changed = Some(std::time::Instant::now());
                self.scene.camera = match self.scene.camera.projection_type() {
                    ProjectionType::Parallel => Camera::perspective_camera(
                        self.scene.camera.matrix,
                        std::f64::consts::PI / 4.0,
                        0.1,
                        40.0,
                    ),
                    ProjectionType::Perspective => {
                        Camera::parallel_camera(self.scene.camera.matrix, 1.0, 0.1, 100.0)
                    }
                }
            }
            VirtualKeyCode::L => {
                if let Some(ref instant) = self.light_changed {
                    let time = instant.elapsed().as_secs_f64();
                    if time < 0.2 {
                        return Self::default_control_flow();
                    }
                }
                self.light_changed = Some(std::time::Instant::now());
                match self.scene.lights[0].light_type {
                    LightType::Point => {
                        let mut vec = self.scene.camera.position();
                        vec /= vec.to_vec().magnitude();
                        self.scene.lights[0] = Light {
                            position: vec,
                            color: Vector3::new(1.0, 1.0, 1.0),
                            light_type: LightType::Uniform,
                        }
                    }
                    LightType::Uniform => {
                        let position = self.scene.camera.position();
                        self.scene.lights[0] = Light {
                            position,
                            color: Vector3::new(1.0, 1.0, 1.0),
                            light_type: LightType::Point,
                        }
                    }
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }

    fn update(&mut self, _: &WGPUHandler) { self.scene.prepare_render(); }

    fn render(&self, frame: &SwapChainFrame) { self.scene.render_scene(&frame.output.view); }
}

fn main() { MyApp::run(); }
