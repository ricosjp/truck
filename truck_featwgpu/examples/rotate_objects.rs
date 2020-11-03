use std::f64::consts::PI;
use std::path::PathBuf;
use truck_featwgpu::*;
use truck_polymesh::{MeshHandler, PolygonMesh};
use wgpu::*;
use winit::{dpi::*, event::*, event_loop::ControlFlow};
mod app;
use app::*;

const NUM_OF_OBJECTS: usize = 8;

struct MyRender {
    scene: Scene,
    instances: Vec<PolygonInstance>,
    rotate_flag: bool,
    prev_cursor: Option<Vector2>,
    prev_time: f64,
    path: Option<PathBuf>,
    light_changed: Option<std::time::Instant>,
    camera_changed: Option<std::time::Instant>,
}

impl MyRender {
    fn create_camera() -> Camera {
        let mat = Matrix4::look_at(
            Point3::new(15.0, 15.0, 15.0),
            Point3::origin(),
            Vector3::unit_y(),
        );
        Camera::perspective_camera(
            mat.invert().unwrap(),
            std::f64::consts::PI / 8.0,
            0.1,
            200.0,
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

    fn load_obj<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    )
    {
        let scene = &mut self.scene;
        scene.clear_objects();
        self.instances.clear();
        let file = std::fs::File::open(path).unwrap();
        let mesh = truck_io::obj::read(file).unwrap();
        let mesh = MyRender::set_normals(mesh);
        let bdd_box = mesh.bounding_box();
        let (size, center) = (bdd_box.size(), bdd_box.center());
        let original_mesh = PolygonInstance::new(mesh, scene.device());
        let rad = cgmath::Rad(2.0 * PI / NUM_OF_OBJECTS as f64);
        let mut mat = Matrix4::from_scale(size / 2.0);
        mat = Matrix4::from_translation(center.to_vec()) * mat;
        mat = mat.invert().unwrap();
        mat = Matrix4::from_translation(Vector3::new(0.0, 0.5, 5.0)) * mat;
        for _ in 0..NUM_OF_OBJECTS {
            let mut instance = original_mesh.clone();
            instance.matrix = mat;
            scene.add_object(&instance);
            self.instances.push(instance);
            mat = Matrix4::from_axis_angle(Vector3::unit_y(), rad) * mat;
        }
    }

    fn update_objects(&mut self) {
        let time = self.scene.elapsed();
        let delta_time = time - self.prev_time;
        self.prev_time = time;
        let mat0 = Matrix4::from_axis_angle(Vector3::unit_y(), cgmath::Rad(delta_time));
        for (idx, instance) in self.instances.iter_mut().enumerate() {
            let k = (-1_f64).powi(idx as i32) * 5.0;
            let mat1 = Matrix4::from_axis_angle(Vector3::unit_y(), cgmath::Rad(k * delta_time));
            let x = instance.matrix[3][2];
            let mat = mat0 * instance.matrix * mat1 * instance.matrix.invert().unwrap();
            instance.matrix = mat * instance.matrix;
            let obj_pos = instance.matrix[3].truncate();
            let new_length = 5.0 + time.sin() + (time * 3.0).sin() / 3.0;
            let obj_dir = &obj_pos / obj_pos.magnitude();
            let move_vec = obj_dir * (new_length - obj_pos.magnitude());
            let mat = Matrix4::from_translation(move_vec);
            instance.matrix = mat * instance.matrix;
            let color = Self::calculate_color(x / 14.0 + 0.5);
            instance.material = Material {
                albedo: color,
                roughness: (0.5 + (time / 5.0).sin() / 2.0),
                reflectance: 0.04 + 0.96 * (0.5 + (time / 2.0).sin() / 2.0),
            };
            self.scene.update_bind_group(&*instance, idx);
        }
    }

    fn calculate_color(x: f64) -> Vector4 {
        Vector4::new(
            (-25.0 * (x - 0.2) * (x - 0.2)).exp() + (-25.0 * (x - 1.3) * (x - 1.3)).exp(),
            (-25.0 * (x - 0.5) * (x - 0.5)).exp(),
            (-25.0 * (x - 0.8) * (x - 0.8)).exp(),
            1.0,
        )
    }
}

impl App for MyRender {
    fn init(handler: &WGPUHandler) -> MyRender {
        let (device, queue, sc_desc) = (&handler.device, &handler.queue, &handler.sc_desc);
        let mut render = MyRender {
            scene: Scene::new(device, queue, sc_desc),
            instances: Vec::new(),
            rotate_flag: false,
            prev_cursor: None,
            prev_time: 0.0,
            path: None,
            camera_changed: None,
            light_changed: None,
        };
        render.scene.camera = MyRender::create_camera();
        render.scene.lights.push(Light {
            position: Point3::new(0.0, 20.0, 0.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            light_type: LightType::Point,
        });
        render
    }

    fn app_title<'a>() -> Option<&'a str> { Some("rotation object") }

    fn depth_stencil_attachment_descriptor<'a>(
        &'a self,
    ) -> Option<RenderPassDepthStencilAttachmentDescriptor<'a>> {
        Some(self.scene.depth_stencil_attachment_descriptor())
    }

    fn dropped_file(&mut self, path: std::path::PathBuf) -> ControlFlow {
        self.path = Some(path);
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
                match scene.lights[0].light_type {
                    LightType::Point => {
                        scene.lights[0].position = scene.camera.position();
                    }
                    LightType::Uniform => {
                        scene.lights[0].position = scene.camera.position();
                        let tmp = scene.lights[0].position.to_vec().magnitude();
                        scene.lights[0].position /= tmp
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
                let mut axis = dir2d[1] * &self.scene.camera.matrix[0];
                axis += dir2d[0] * &self.scene.camera.matrix[1];
                axis /= axis.magnitude();
                let angle = dir2d.magnitude() * 0.01;
                let mat = Matrix4::from_axis_angle(axis.truncate(), cgmath::Rad(angle));
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
                    ProjectionType::Parallel => {
                        let mut camera = Camera::default();
                        camera.matrix = self.scene.camera.matrix;
                        camera
                    }
                    ProjectionType::Perspective => {
                        let matrix = self.scene.camera.matrix;
                        Camera::parallel_camera(matrix, 2.0, 0.1, 40.0)
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

    fn update(&mut self, _: &WGPUHandler) {
        if let Some(path) = self.path.take() {
            self.load_obj(path);
        }
        if self.scene.number_of_objects() != 0 {
            self.update_objects();
        }
        self.scene.prepare_render();
    }

    fn render(&self, frame: &SwapChainFrame) { self.scene.render_scene(&frame.output.view); }
}

fn main() { MyRender::run() }
