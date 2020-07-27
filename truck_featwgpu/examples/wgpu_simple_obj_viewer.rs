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

    fn obj_bounding(mesh: &PolygonMesh) -> (f64, Vector3) {
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
        (size, center)
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
        if self.scene.number_of_objects() != 0 {
            self.scene.remove_object(0);
        }
        let file = std::fs::File::open(path).unwrap();
        let mesh = truck_io::obj::read(file).unwrap();
        let mesh = MyApp::set_normals(mesh);
        let (size, center) = MyApp::obj_bounding(&mesh);
        let mut object = RenderObject::new(mesh, device);
        let diag = vector!(size, size, size);
        object.matrix = Matrix3::diagonal(&diag).affine(&center).inverse();
        object.color = vector![1.0, 1.0, 1.0, 1.0];
        object.reflect_ratio = [0.2, 0.6, 0.2];
        self.scene.add_object(object);
    }
}

impl App for MyApp {
    fn init(device: &Device, sc_desc: &SwapChainDescriptor) -> MyApp {
        let mut render = MyApp {
            scene: Scene::new(&device, &sc_desc),
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
            position: vector!(2, 2, 2),
            strength: 2.0,
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
                        scene.light.strength = 2.5;
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
                            strength: 0.25,
                            light_type: LightType::Uniform,
                        }
                    }
                    LightType::Uniform => {
                        let position = self.scene.camera.position();
                        self.scene.light = Light {
                            position,
                            strength: 2.0,
                            light_type: LightType::Point,
                        }
                    }
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }

    fn update(&mut self, device: &Device, sc_desc: &SwapChainDescriptor) {
        if let Some(path) = self.path.take() {
            self.load_obj(path, device);
        }
        self.width = sc_desc.width;
        self.height = sc_desc.height;
        self.scene.prepare_render(device, sc_desc);
    }

    fn render<'a>(&'a self, rpass: &mut RenderPass<'a>) { self.scene.render_scene(rpass); }
}

fn main() { futures::executor::block_on(app::run::<MyApp>()); }
