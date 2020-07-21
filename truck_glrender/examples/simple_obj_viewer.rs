use glium::glutin::event::*;
use glium::glutin::event_loop::ControlFlow;
use glium::*;
use truck_geometry::*;
use truck_glrender::*;
use truck_polymesh::*;

struct MyRenderer {
    scene: Scene,
    prev_cursor: Option<Vector2>,
    rotate_flag: bool,
    new_object: Option<std::path::PathBuf>,
    light_changed: Option<std::time::Instant>,
    camera_changed: Option<std::time::Instant>,
}

impl MyRenderer {
    fn fit_scene(&mut self) {
        let scene = &mut self.scene;
        scene.fit_camera();
        match scene.light.light_type {
            LightType::Point => {
                scene.light.position = scene.camera.position();
                let bdd_box = scene.objects_bounding_box();
                scene.light.strength = bdd_box.1[2] * bdd_box.1[2] * 0.25;
            }
            LightType::Uniform => {
                scene.light.position = scene.camera.position();
                scene.light.position /= scene.light.position.norm();
            }
        }
    }

    fn create_camera() -> Camera {
        let mut vec0 = vector!(1.5, 0.0, -1.5, 0.0);
        vec0 /= vec0.norm();
        let mut vec1 = vector!(-0.5, 1, -0.5, 0.0);
        vec1 /= vec1.norm();
        let mut vec2 = vector!(1, 1, 1, 0);
        vec2 /= vec2.norm();
        let vec3 = vector!(10, 12, 10, 1);
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

    fn load_obj(path: std::path::PathBuf) -> PolygonMesh {
        let file = std::fs::File::open(path).unwrap();
        let mesh = truck_io::obj::read(file).unwrap();
        let mesh = MyRenderer::set_normals(mesh);
        MyRenderer::obj_normalize(mesh)
    }
}

impl Render for MyRenderer {
    fn create_display(event_loop: &glium::glutin::event_loop::EventLoop<()>) -> Display {
        let cb = glium::glutin::ContextBuilder::new();
        let wb = glium::glutin::window::WindowBuilder::new().with_title("simple OBJ viewer");
        Display::new(wb, cb, event_loop).unwrap()
    }

    fn init(display: &Display) -> MyRenderer {
        let mut scene = Scene::new(display);
        scene.camera = MyRenderer::create_camera();
        MyRenderer {
            scene,
            prev_cursor: None,
            rotate_flag: false,
            new_object: None,
            light_changed: None,
            camera_changed: None,
        }
    }

    fn draw(&mut self, target: &mut glium::Frame) {
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        self.scene.render_scene(target);
    }

    fn closed_requested(&mut self) -> ControlFlow { glium::glutin::event_loop::ControlFlow::Exit }

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
                    ProjectionType::Perspective => Camera::parallel_camera(
                        self.scene.camera.matrix().clone(),
                        1.0,
                        0.1,
                        40.0,
                    ),
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
                        let bdd_box = self.scene.objects_bounding_box();
                        let strength = bdd_box.1[2] * bdd_box.1[2] * 0.25;
                        self.scene.light = Light {
                            position,
                            strength,
                            light_type: LightType::Point,
                        }
                    }
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> ControlFlow {
        if state == ElementState::Pressed && button == MouseButton::Right {
            self.fit_scene();
        }
        if button == MouseButton::Left {
            self.rotate_flag = state == ElementState::Pressed;
            if !self.rotate_flag {
                self.prev_cursor = None;
            }
        }
        Self::default_control_flow()
    }
    fn dropped_file(&mut self, path: std::path::PathBuf) -> ControlFlow {
        self.new_object = Some(path);
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

    fn cursor_moved(&mut self, position: glium::glutin::dpi::PhysicalPosition<f64>) -> ControlFlow {
        if !self.rotate_flag {
            return Self::default_control_flow();
        }
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
        Self::default_control_flow()
    }

    fn update(&mut self, display: &Display) {
        let path = match self.new_object.take() {
            Some(got) => got,
            None => return,
        };
        let mesh = MyRenderer::load_obj(path);
        if self.scene.number_of_objects() != 0 {
            self.scene.remove_object(0);
        }
        let mut glmesh: GLPolygonMesh = mesh.into();
        glmesh.color = [1.0, 1.0, 1.0];
        glmesh.reflect_ratio = [0.2, 0.6, 0.2];
        self.scene.add_glpolymesh(&glmesh, display);
        self.fit_scene();
    }
}

fn main() { MyRenderer::run() }
