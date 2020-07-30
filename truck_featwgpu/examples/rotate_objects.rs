use std::f64::consts::PI;
use std::path::PathBuf;
use truck_featwgpu::*;
use truck_polymesh::{MeshHandler, PolygonMesh};
use wgpu::*;
use winit::{dpi::*, event::*, event_loop::ControlFlow};
mod app;
use app::*;

struct MyRender {
    scene: Scene,
    rotate_flag: bool,
    prev_cursor: Option<Vector2>,
    prev_time: f64,
    path: Option<PathBuf>,
    width: u32,
    height: u32,
    light_changed: Option<std::time::Instant>,
    camera_changed: Option<std::time::Instant>,
}

impl MyRender {
    fn create_camera() -> Camera {
        let mut vec0 = vector!(1.5, 0.0, -1.5, 0.0);
        vec0 /= vec0.norm();
        let mut vec1 = vector!(-0.5, 1, -0.5, 0.0);
        vec1 /= vec1.norm();
        let mut vec2 = vector!(1, 1, 1, 0);
        vec2 /= vec2.norm();
        let vec3 = vector!(15, 15, 15, 1);
        let matrix = matrix!(vec0, vec1, vec2, vec3);
        Camera::perspective_camera(matrix, std::f64::consts::PI / 4.0, 0.1, 200.0)
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

    fn load_obj<P: AsRef<std::path::Path>>(&mut self, path: P, device: &Device) {
        self.scene.clear_objects();
        let file = std::fs::File::open(path).unwrap();
        let mesh = truck_io::obj::read(file).unwrap();
        let mesh = MyRender::set_normals(mesh);
        let bdd_box = mesh.bounding_box();
        let (size, center) = (bdd_box.size(), bdd_box.center());
        let diag = vector!(size / 2.0, size / 2.0, size / 2.0);
        let mut object = RenderObject::new(mesh, device);
        let mut mat = Matrix3::diagonal(&diag).affine(&center).inverse();
        mat *= Matrix3::identity().affine(&vector!(0.0, 0.5, 0.0));
        object.matrix = mat;
        object.color = vector![1.0, 1.0, 1.0, 1.0];
        object.reflect_ratio = [0.2, 0.6, 0.2];
        self.scene.add_object(object);
    }

    fn arrange_objects(&mut self) {
        let scene = &mut self.scene;
        let object = scene.get_object_mut(0);
        object.matrix *= Matrix3::identity().affine(&vector!(0, 0, 5));
        let mat = Matrix3::rotation(&vector!(0, 1, 0), PI / 4.0).affine(&Vector3::zero());
        for i in 1..8 {
            let mut new_object = scene.get_object(i - 1).clone();
            new_object *= &mat;
            scene.add_object(new_object);
        }
    }

    fn update_objects(&mut self) {
        let scene = &mut self.scene;
        let time = scene.elapsed();
        let delta_time = time - self.prev_time;
        self.prev_time = time;
        let mat0 = Matrix3::rotation(&vector!(0, 1, 0), delta_time).affine(&Vector3::zero());
        for (i, object) in scene.objects_mut().iter_mut().enumerate() {
            let k = (-1_f64).powi(i as i32) * 5.0;
            let mat1 = Matrix3::rotation(&vector!(0, 1, 0), k * delta_time).affine(&Vector3::zero());
            let obj_mat = &object.matrix;
            let x = obj_mat[3][2];
            let mat = obj_mat.inverse() * &mat1 * obj_mat * &mat0;
            *object *= mat;
            let obj_pos: Vector3 = object.matrix[3].clone().into();
            let new_length = 5.0 + time.sin() + (time * 3.0).sin() / 3.0;
            let obj_dir = &obj_pos / obj_pos.norm();
            let move_vec = obj_dir * (new_length - obj_pos.norm());
            let mat = Matrix3::identity().affine(&move_vec);
            *object *= mat;
            object.color = Self::calculate_color(x / 14.0 + 0.5);
        }
    }

    fn calculate_color(x: f64) -> Vector4 {
        vector!(
            (-25.0 * (x - 0.2) * (x - 0.2)).exp() + (-25.0 * (x - 1.3) * (x - 1.3)).exp(),
            (-25.0 * (x - 0.5) * (x - 0.5)).exp(),
            (-25.0 * (x - 0.8) * (x - 0.8)).exp(),
            1.0,
        )
    }
}

impl App for MyRender {
    fn init(handler: &WGPUHandler) -> MyRender {
        let device = &handler.device;
        let sc_desc = &handler.sc_desc;
        let mut render = MyRender {
            scene: Scene::new(device, sc_desc),
            rotate_flag: false,
            prev_cursor: None,
            prev_time: 0.0,
            path: None,
            width: sc_desc.width,
            height: sc_desc.height,
            camera_changed: None,
            light_changed: None,
        };
        render.scene.camera = MyRender::create_camera();
        render.scene.light = Light {
            position: vector!(0, 20, 0),
            strength: 100.0,
            light_type: LightType::Point,
        };
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
                        scene.light.strength = 200.0;
                    }
                    LightType::Uniform => {
                        scene.light.position = scene.camera.position();
                        scene.light.position /= scene.light.position.norm();
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
                self.scene.camera *= Matrix3::identity().affine(&trans_vec);
            }
            MouseScrollDelta::PixelDelta(_) => {}
        };
        Self::default_control_flow()
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) -> ControlFlow {
        if self.rotate_flag {
            let position = vector!(position.x, position.y);
            if let Some(ref prev_position) = self.prev_cursor {
                let dir2d = &position - prev_position;
                let mut axis = dir2d[1] * &self.scene.camera.matrix()[0];
                axis += dir2d[0] * &self.scene.camera.matrix()[1];
                axis /= axis.norm();
                let angle = dir2d.norm() * 0.01;
                let mat = Matrix3::rotation(&axis.into(), angle).affine(&Vector3::zero());
                self.scene.camera *= mat.inverse();
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
                        self.scene.camera.matrix().clone(),
                        std::f64::consts::PI / 4.0,
                        0.1,
                        40.0,
                    ),
                    ProjectionType::Perspective => {
                        Camera::parallel_camera(self.scene.camera.matrix().clone(), 1.0, 0.1, 40.0)
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
                        vec /= vec.norm();
                        self.scene.light = Light {
                            position: vec,
                            strength: 0.5,
                            light_type: LightType::Uniform,
                        }
                    }
                    LightType::Uniform => {
                        let position = self.scene.camera.position();
                        self.scene.light = Light {
                            position,
                            strength: 125.0,
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
        let device = &handler.device;
        let sc_desc = &handler.sc_desc;
        if let Some(path) = self.path.take() {
            self.load_obj(path, device);
            self.arrange_objects();
        }
        self.width = sc_desc.width;
        self.height = sc_desc.height;
        if self.scene.number_of_objects() != 0 {
            self.update_objects();
        }
        self.scene.prepare_render(device, sc_desc);
    }

    fn render<'a>(&'a self, rpass: &mut RenderPass<'a>) { self.scene.render_scene(rpass); }
}

fn main() { MyRender::run() }
