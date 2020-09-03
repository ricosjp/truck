use std::path::PathBuf;
use truck_featwgpu::*;
use truck_polymesh::{MeshHandler, PolygonMesh};
use wgpu::*;
use winit::{dpi::*, event::*, event_loop::ControlFlow};
mod app;
use app::*;

struct MyApp {
    scene: Scene,
    rotate_flag: bool,
    prev_cursor: Option<Vector2>,
    path: Option<PathBuf>,
    width: u32,
    height: u32,
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

    fn load_obj<P: AsRef<std::path::Path>>(&mut self, path: P, sc_desc: &SwapChainDescriptor) {
        if self.scene.number_of_objects() != 0 {
            self.scene.remove_object(0);
        }
        let file = std::fs::File::open(path).unwrap();
        let mesh = truck_io::obj::read(file).unwrap();
        let mesh = MyApp::set_normals(mesh);
        let bdd_box = mesh.bounding_box();
        let (size, center) = (bdd_box.size(), bdd_box.center());
        let mut mesh = PolygonInstance::new(mesh);
        let mat = Matrix4::from_translation(center.to_vec()) * Matrix4::from_scale(size);
        mesh.matrix = mat.invert().unwrap();
        mesh.color.ambient = Vector4::new(0.7, 0.7, 0.7, 1.0);
        mesh.color.diffuse = Vector4::new(0.7, 0.7, 0.7, 1.0);
        mesh.color.specular = Vector4::new(1.0, 1.0, 1.0, 1.0);
        mesh.color.reflect_ratio = Vector3::new(0.2, 0.6, 0.2);
        let object = mesh.render_object(&self.scene, sc_desc, None);
        self.scene.add_object(object);
    }
}

impl App for MyApp {
    fn init(handler: &WGPUHandler) -> MyApp {
        let (device, queue, sc_desc) = (&handler.device, &handler.queue, &handler.sc_desc);
        let mut render = MyApp {
            scene: Scene::new(device, queue, sc_desc),
            rotate_flag: false,
            prev_cursor: None,
            path: None,
            width: sc_desc.width,
            height: sc_desc.height,
            camera_changed: None,
            light_changed: None,
        };
        render.scene.camera = MyApp::create_camera();
        render.scene.light = Light {
            position: Point3::new(1.0, 1.0, 1.0),
            strength: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),
            light_type: LightType::Point,
        };
        render
    }

    fn app_title<'a>() -> Option<&'a str> { Some("simple obj viewer") }

    fn depth_stencil_attachment_descriptor<'a>(
        &'a self,
    ) -> Option<RenderPassDepthStencilAttachmentDescriptor<'a>> {
        Some(self.scene.depth_stencil_attachment_descriptor())
    }

    fn dropped_file(&mut self, path: std::path::PathBuf) -> ControlFlow {
        self.path = Some(path);
        Self::default_control_flow()
    }

    fn resized(&mut self, size: PhysicalSize<u32>) -> ControlFlow {
        self.width = size.width;
        self.height = size.height;
        Self::default_control_flow()
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
                match scene.light.light_type {
                    LightType::Point => {
                        scene.light.position = scene.camera.position();
                        scene.light.strength = 1.0;
                    }
                    LightType::Uniform => {
                        scene.light.position = scene.camera.position();
                        scene.light.strength = 0.5;
                        scene.light.position /= scene.light.position.to_vec().magnitude();
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
                match self.scene.light.light_type {
                    LightType::Point => {
                        let mut vec = self.scene.camera.position();
                        vec /= vec.to_vec().magnitude();
                        self.scene.light = Light {
                            position: vec,
                            strength: 0.5,
                            color: Vector3::new(1.0, 1.0, 1.0),
                            light_type: LightType::Uniform,
                        }
                    }
                    LightType::Uniform => {
                        let position = self.scene.camera.position();
                        self.scene.light = Light {
                            position,
                            strength: 1.0,
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

    fn update(&mut self, handler: &WGPUHandler) {
        let sc_desc = &handler.sc_desc;
        if let Some(path) = self.path.take() {
            self.load_obj(path, sc_desc);
        }
        self.width = sc_desc.width;
        self.height = sc_desc.height;
        self.scene.prepare_render(sc_desc);
    }

    fn render(&self, frame: &SwapChainFrame) { self.scene.render_scene(&frame.output); }
}

fn main() { MyApp::run(); }
