use glium::*;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use truck_geometry::*;
use truck_glrender::*;
use truck_polymesh::*;

struct BSpAnimation {
    scene: Scene,
    mesh: Arc<Mutex<Option<PolygonMesh>>>,
    closed: Arc<Mutex<bool>>,
    thread: Option<JoinHandle<()>>,
}

impl BSpAnimation {
    fn init_surface(degree: usize, division: usize) -> BSplineSurface<[f64; 4]> {
        let range = degree + division - 1;
        let knot_vec = KnotVec::uniform_knot(degree, division);
        let mut ctrl_pts = Vec::new();
        for i in 0..=range {
            let u = (i as f64) / (range as f64);
            let mut vec = Vec::new();
            for j in 0..=range {
                let v = (j as f64) / (range as f64);
                vec.push(vector!(v, 0, u, 1));
            }
            ctrl_pts.push(vec);
        }
        BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts)
    }

    fn init_scene(display: &Display) -> Scene {
        let mut scene = Scene::new(display);
        let mut vec0 = vector!(1.5, 0.0, -1.5, 0.0);
        vec0 /= vec0.norm();
        let mut vec1 = vector!(-0.5, 1, -0.5, 0.0);
        vec1 /= vec1.norm();
        let mut vec2 = vector!(1, 1, 1, 0);
        vec2 /= vec2.norm();
        let vec3 = vector!(1.5, 0.8, 1.5, 1);
        let matrix = matrix!(vec0, vec1, vec2, vec3);
        scene.camera = Camera::perspective_camera(matrix, std::f64::consts::PI / 2.0, 0.1, 40.0);
        scene.light = Light {
            position: vector!(0.5, 2.0, 0.5),
            strength: 1.0,
            light_type: LightType::Point,
        };
        scene
    }

    fn init_thread(
        arc_mesh: Arc<Mutex<Option<PolygonMesh>>>,
        closed: Arc<Mutex<bool>>,
    ) -> JoinHandle<()>
    {
        std::thread::spawn(move || {
            let mut bspsurface = BSpAnimation::init_surface(3, 4);
            let mut time: f64 = 0.0;
            let mut count = 0;
            let mut instant = std::time::Instant::now();
            loop {
                if *closed.lock().unwrap() {
                    break;
                }
                //let mut bspsurface0 = bspsurface.clone();
                //bspsurface0.optimize();
                count += 1;
                let mesh = StructuredMesh::from_surface(&bspsurface, 0.01).destruct();
                *arc_mesh.lock().unwrap() = Some(mesh);
                bspsurface.control_point_mut(3, 3)[1] = time.sin();
                time += 0.1;
                if count == 100 {
                    let fps_inv = instant.elapsed().as_secs_f64();
                    println!("{}", 100.0 / fps_inv);
                    instant = std::time::Instant::now();
                    count = 0;
                }
            }
        })
    }
}

impl Render for BSpAnimation {
    fn init(display: &Display) -> Self {
        let scene = BSpAnimation::init_scene(display);
        let mesh = Arc::new(Mutex::new(None));
        let closed = Arc::new(Mutex::new(false));
        let thread = Some(BSpAnimation::init_thread(
            Arc::clone(&mesh),
            Arc::clone(&closed),
        ));
        BSpAnimation {
            scene,
            mesh,
            closed,
            thread,
        }
    }

    fn update(&mut self, display: &Display) {
        match self.mesh.lock().unwrap().take() {
            Some(mesh) => {
                if self.scene.number_of_objects() > 0 {
                    self.scene.remove_object(0);
                }
                self.scene.add_glpolymesh(&mesh.into(), display);
            }
            None => return,
        }
    }

    fn draw(&mut self, target: &mut Frame) {
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        self.scene.render_scene(target);
    }

    fn closed_requested(&mut self) -> glium::glutin::event_loop::ControlFlow {
        *self.closed.lock().unwrap() = true;
        self.thread.take().unwrap().join().unwrap();
        glium::glutin::event_loop::ControlFlow::Exit
    }
}

fn main() { BSpAnimation::run() }
