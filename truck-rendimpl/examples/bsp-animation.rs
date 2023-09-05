//! Benchmark Animation
//!
//! In each frame, the NURBS surface is divided into mesh.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::*;
use truck_modeling::*;
use truck_platform::*;
use truck_rendimpl::*;
use winit::window::Window;
mod app;
use app::*;

struct MyApp {
    scene: WindowScene,
    creator: InstanceCreator,
    object: Arc<Mutex<StructuredMesh>>,
    instance: PolygonInstance,
    closed: Arc<AtomicBool>,
    updated: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl MyApp {
    fn init_surface(degree: usize, division: usize) -> BSplineSurface<Point3> {
        let range = degree + division - 1;
        let knot_vec = KnotVec::uniform_knot(degree, division);
        let mut ctrl_pts = Vec::new();
        for i in 0..=range {
            let u = (i as f64) / (range as f64);
            let mut vec = Vec::new();
            for j in 0..=range {
                let v = (j as f64) / (range as f64);
                vec.push(Point3::new(v, 0.0, u));
            }
            ctrl_pts.push(vec);
        }
        BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts)
    }
    fn init_camera() -> Camera {
        let mut vec0 = Vector4::new(1.5, 0.0, -1.5, 0.0);
        vec0 /= vec0.magnitude();
        let mut vec1 = Vector4::new(-0.5, 1.0, -0.5, 0.0);
        vec1 /= vec1.magnitude();
        let mut vec2 = Vector4::new(1.0, 1.0, 1.0, 0.0);
        vec2 /= vec2.magnitude();
        let vec3 = Vector4::new(1.5, 0.8, 1.5, 1.0);
        let matrix = Matrix4::from_cols(vec0, vec1, vec2, vec3);
        let mut camera = Camera::default();
        camera.matrix = matrix;
        camera
    }
    fn init_thread(
        object: Arc<Mutex<StructuredMesh>>,
        closed: Arc<AtomicBool>,
        updated: Arc<AtomicBool>,
        surface: Arc<Mutex<BSplineSurface<Point3>>>,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let mut time: f64 = 0.0;
            let mut count = 0;
            let mut instant = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1));
                if closed.load(Ordering::SeqCst) {
                    break;
                }
                if updated.load(Ordering::SeqCst) {
                    continue;
                }
                if let Ok(mut surface) = surface.lock() {
                    surface.control_point_mut(3, 3)[1] = time.sin();
                    let surface0 = surface.clone();
                    drop(surface);
                    let mesh =
                        StructuredMesh::from_surface(&surface0, surface0.range_tuple(), 0.01);
                    *object.lock().unwrap() = mesh;
                }
                updated.store(true, Ordering::SeqCst);
                count += 1;
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

#[async_trait(?Send)]
impl App for MyApp {
    async fn init(window: Arc<Window>) -> Self {
        let desc = WindowSceneDescriptor {
            studio: StudioConfig {
                camera: MyApp::init_camera(),
                lights: vec![Light {
                    position: Point3::new(0.5, 2.0, 0.5),
                    color: Vector3::new(1.0, 1.0, 1.0),
                    light_type: LightType::Point,
                }],
                ..Default::default()
            },
            backend_buffer: BackendBufferConfig {
                sample_count: 4,
                ..Default::default()
            },
        };
        let mut scene = WindowScene::from_window(window, &desc).await;
        let creator = scene.instance_creator();
        let surface = Self::init_surface(3, 4);
        let object = StructuredMesh::from_surface(&surface, surface.range_tuple(), 0.01);
        let instance = creator.create_instance(&object, &Default::default());
        scene.add_object(&instance);
        let object = Arc::new(Mutex::new(object));
        let closed = Arc::new(AtomicBool::new(false));
        let updated = Arc::new(AtomicBool::new(false));
        let thread = Some(MyApp::init_thread(
            Arc::clone(&object),
            Arc::clone(&closed),
            Arc::clone(&updated),
            Arc::new(Mutex::new(surface)),
        ));
        MyApp {
            scene,
            creator,
            instance,
            object,
            closed,
            updated,
            thread,
        }
    }
    fn app_title<'a>() -> Option<&'a str> { Some("BSpline Benchmark Animation") }

    fn render(&mut self) {
        if self.updated.load(Ordering::SeqCst) {
            let object = self.object.lock().unwrap();
            let mut object: PolygonInstance =
                self.creator.create_instance(&*object, &Default::default());
            self.instance.swap_vertex(&mut object);
            self.scene.update_vertex_buffer(&self.instance);
            self.updated.store(false, Ordering::SeqCst);
        }
        self.scene.render_frame();
    }

    fn closed_requested(&mut self) -> winit::event_loop::ControlFlow {
        self.closed.store(true, Ordering::SeqCst);
        self.thread.take().unwrap().join().unwrap();
        winit::event_loop::ControlFlow::Exit
    }
}

fn main() { MyApp::run(); }
