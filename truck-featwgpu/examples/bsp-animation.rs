use std::sync::{Arc, Mutex};
use std::thread::*;
use truck_featwgpu::*;
use wgpu::*;
mod app;
use app::*;

struct MyApp {
    scene: Scene,
    object: Arc<Mutex<Option<PolygonInstance>>>,
    closed: Arc<Mutex<bool>>,
    thread: Option<JoinHandle<()>>,
}

impl MyApp {
    fn init_surface(degree: usize, division: usize) -> NURBSSurface {
        let range = degree + division - 1;
        let knot_vec = KnotVec::uniform_knot(degree, division);
        let mut ctrl_pts = Vec::new();
        for i in 0..=range {
            let u = (i as f64) / (range as f64);
            let mut vec = Vec::new();
            for j in 0..=range {
                let v = (j as f64) / (range as f64);
                vec.push(Vector4::new(v, 0.0, u, 1.0));
            }
            ctrl_pts.push(vec);
        }
        NURBSSurface::new(BSplineSurface::new((knot_vec.clone(), knot_vec), ctrl_pts))
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
        device: &Arc<Device>,
        object: &Arc<Mutex<Option<PolygonInstance>>>,
        closed: &Arc<Mutex<bool>>,
    ) -> JoinHandle<()>
    {
        let device = Arc::clone(&device);
        let arc_object = Arc::clone(object);
        let closed = Arc::clone(closed);
        std::thread::spawn(move || {
            let mut bspsurface = Self::init_surface(3, 4);
            let mut time: f64 = 0.0;
            let mut count = 0;
            let mut instant = std::time::Instant::now();
            loop {
                if *closed.lock().unwrap() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
                let mut bspsurface0 = bspsurface.clone();
                bspsurface0.non_rationalized_mut().optimize();
                let mesh = truck_polymesh::StructuredMesh::from_surface(&bspsurface0, 0.01);
                let object = PolygonInstance::new(mesh.destruct(), &device);
                count += 1;
                bspsurface.control_point_mut(3, 3)[1] = time.sin();
                time += 0.1;
                if count == 100 {
                    let fps_inv = instant.elapsed().as_secs_f64();
                    println!("{}", 100.0 / fps_inv);
                    instant = std::time::Instant::now();
                    count = 0;
                }
                let mut object_mut = arc_object.lock().unwrap();
                if object_mut.is_none() {
                    *object_mut = Some(object);
                }
            }
        })
    }
}

impl App for MyApp {
    fn init(handler: &WGPUHandler) -> MyApp {
        let (device, queue, sc_desc) = (&handler.device, &handler.queue, &handler.sc_desc);
        let object = Arc::new(Mutex::new(None));
        let closed = Arc::new(Mutex::new(false));
        let thread = Some(MyApp::init_thread(&handler.device, &object, &closed));
        let mut render = MyApp {
            scene: Scene::new(device, queue, sc_desc),
            object,
            closed,
            thread,
        };
        render.scene.camera = MyApp::init_camera();
        render.scene.lights.push(Light {
            position: Point3::new(0.5, 2.0, 0.5),
            color: Vector3::new(1.0, 1.0, 1.0),
            light_type: LightType::Point,
        });
        render
    }

    fn app_title<'a>() -> Option<&'a str> { Some("BSpline Benchmark Animation") }

    fn depth_stencil_attachment_descriptor<'a>(
        &'a self,
    ) -> Option<RenderPassDepthStencilAttachmentDescriptor<'a>> {
        Some(self.scene.depth_stencil_attachment_descriptor())
    }

    fn update(&mut self, _: &WGPUHandler) {
        match self.object.lock().unwrap().take() {
            Some(object) => {
                if self.scene.number_of_objects() == 0 {
                    self.scene.add_object(&object);
                } else {
                    self.scene.update_vertex_buffer(&object, 0);
                };
            }
            None => {}
        }
        self.scene.prepare_render();
    }

    fn render(&self, frame: &SwapChainFrame) {
        self.scene.render_scene(&frame.output.view);
    }
    fn closed_requested(&mut self) -> winit::event_loop::ControlFlow {
        *self.closed.lock().unwrap() = true;
        self.thread.take().unwrap().join().unwrap();
        winit::event_loop::ControlFlow::Exit
    }
}

fn main() { MyApp::run(); }
