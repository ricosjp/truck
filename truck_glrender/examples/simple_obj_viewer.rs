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
    center: Vector3,
    new_object: Option<std::path::PathBuf>,
}

impl MyRenderer {
    fn fit_scene(&mut self) {
        let scene = &mut self.scene;
        scene.fit_camera();
        let light_position = [
            scene.camera.matrix()[3][0],
            scene.camera.matrix()[3][1],
            scene.camera.matrix()[3][2],
        ];
        let bdd_box = scene.objects_bounding_box();
        let light_strength = bdd_box[2].1 * bdd_box[2].0 * 0.25;
        scene.light = Light::Point {
            position: light_position,
            strength: light_strength,
        }
    }
}

impl Renderer for MyRenderer {
    fn create_display(event_loop: &glium::glutin::event_loop::EventLoop<()>) -> Display {
        let cb = glium::glutin::ContextBuilder::new();
        let wb = glium::glutin::window::WindowBuilder::new()
            .with_title("simple OBJ viewer");
        Display::new(wb, cb, event_loop).unwrap()
    }

    fn init(display: &Display) -> MyRenderer {
        let mut scene = Scene::new(display);
        scene.camera = create_camera();
        MyRenderer {
            scene,
            prev_cursor: None,
            rotate_flag: false,
            center: Default::default(),
            new_object: None,
        }
    }

    fn draw(&mut self, target: &mut glium::Frame) {
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        self.scene.render_scene(target);
    }

    fn closed_requested(&mut self) -> ControlFlow { glium::glutin::event_loop::ControlFlow::Exit }

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
        let y = match delta {
            MouseScrollDelta::LineDelta(_, y) => y as f64,
            MouseScrollDelta::PixelDelta(_) => return Self::default_control_flow(),
        };
        let trans_x = -y * self.scene.camera.matrix()[2][0];
        let trans_y = -y * self.scene.camera.matrix()[2][1];
        let trans_z = -y * self.scene.camera.matrix()[2][2];
        self.scene.camera *= matrix!(
            (1, 0, 0, 0),
            (0, 1, 0, 0),
            (0, 0, 1, 0),
            (trans_x, trans_y, trans_z, 1.0),
        );
        Self::default_control_flow()
    }

    fn cursor_moved(&mut self, position: glium::glutin::dpi::PhysicalPosition<f64>) -> ControlFlow {
        let position = vector!(position.x, position.y);
        let center_move = matrix!(
            (1, 0, 0, 0),
            (0, 1, 0, 0),
            (0, 0, 1, 0),
            (self.center[0], self.center[1], self.center[2], 1),
        );
        if self.rotate_flag {
            if self.prev_cursor.is_none() {
                self.prev_cursor = Some(position);
                return Self::default_control_flow();
            }
            let prev_position = self.prev_cursor.as_ref().unwrap();
            let dir2d = &position - prev_position;
            let mut axis = dir2d[1] * &self.scene.camera.matrix()[0];
            axis += dir2d[0] * &self.scene.camera.matrix()[1];
            axis /= axis.norm();
            let angle = dir2d.norm() * 0.01;
            let cos = angle.cos();
            let sin = angle.sin();
            let arr0 = [
                cos + axis[0] * axis[0] * (1.0 - cos),
                axis[0] * axis[1] * (1.0 - cos) + axis[2] * sin,
                axis[2] * axis[0] * (1.0 - cos) - axis[1] * sin,
                0.0,
            ];
            let arr1 = [
                axis[0] * axis[1] * (1.0 - cos) - axis[2] * sin,
                cos + axis[1] * axis[1] * (1.0 - cos),
                axis[1] * axis[2] * (1.0 - cos) + axis[0] * sin,
                0.0,
            ];
            let arr2 = [
                axis[2] * axis[0] * (1.0 - cos) + axis[1] * sin,
                axis[1] * axis[2] * (1.0 - cos) - axis[0] * sin,
                cos + axis[2] * axis[2] * (1.0 - cos),
                0.0,
            ];
            let arr3 = [0.0, 0.0, 0.0, 1.0];
            self.scene.camera *=
                center_move.inverse() * matrix!(arr0, arr1, arr2, arr3).inverse() * center_move;
            self.prev_cursor = Some(position);
        }
        Self::default_control_flow()
    }

    fn update(&mut self, display: &Display) {
        let path = match self.new_object.take() {
            Some(got) => got,
            None => return,
        };
        let file = std::fs::File::open(path).unwrap();
        let mut mesh = truck_io::obj::read(file).unwrap();
        mesh = if mesh.normals.is_empty() {
            let mut mesh_handler = MeshHandler::new(mesh);
            mesh_handler.put_together_same_attrs();
            mesh_handler.add_smooth_normal(0.5);
            mesh_handler.into()
        } else {
            mesh
        };
        let bdd_box = mesh.bounding_box();
        self.center = vector!(
            (bdd_box[0].0 + bdd_box[0].1) / 2.0,
            (bdd_box[1].0 + bdd_box[1].1) / 2.0,
            (bdd_box[2].0 + bdd_box[2].1) / 2.0,
        );
        if self.scene.number_of_objects() != 0 {
            self.scene.remove_object(0);
        }
        let mut glmesh: GLPolygonMesh = mesh.into();
        glmesh.color = [0.0, 1.0, 1.0];
        glmesh.reflect_ratio = [0.2, 0.5, 0.3];
        self.scene.add_glpolymesh(&glmesh, display);
        self.fit_scene();
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
    Camera::perspective_camera(matrix, std::f64::consts::PI / 2.0, 0.1, 40.0)
}

fn main() { MyRenderer::run(); }
