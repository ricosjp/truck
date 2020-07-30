use truck_featwgpu::*;
use truck_geometry::KnotVec;
use wgpu::*;
mod app;
use app::*;
const VERTEX_SIZE: u64 = 10000 * 8 * std::mem::size_of::<f32>() as u64;

struct MyApp {
    scene: Scene,
    mesher: WGPUMesher,
    surface: BSplineSurface,
    time: usize,
    since: std::time::Instant,
}

impl MyApp {
    fn init_surface(degree: usize, division: usize) -> BSplineSurface {
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
    fn init_camera() -> Camera {
        let mut vec0 = vector!(1.5, 0.0, -1.5, 0.0);
        vec0 /= vec0.norm();
        let mut vec1 = vector!(-0.5, 1, -0.5, 0.0);
        vec1 /= vec1.norm();
        let mut vec2 = vector!(1, 1, 1, 0);
        vec2 /= vec2.norm();
        let vec3 = vector!(1.5, 0.8, 1.5, 1);
        let matrix = matrix!(vec0, vec1, vec2, vec3);
        Camera::perspective_camera(matrix, std::f64::consts::PI / 2.0, 0.1, 40.0)
    }
    fn update_surface(&mut self) {
        self.surface.control_point_mut(3, 3)[1] = (self.time as f64 * 0.1).sin();
        self.time += 1;
    }
}

impl App for MyApp {
    fn init(handler: &WGPUHandler) -> MyApp {
        let (device, sc_desc) = (&handler.device, &handler.sc_desc);
        let mut render = MyApp {
            scene: Scene::new(device, sc_desc),
            mesher: WGPUMesher::new(device),
            surface: Self::init_surface(3, 4),
            time: 0,
            since: std::time::Instant::now(),
        };
        render.scene.camera = MyApp::init_camera();
        render.scene.light = Light {
            position: vector!(0.5, 2.0, 0.5),
            strength: 1.0,
            light_type: LightType::Point,
        };
        render
    }

    fn app_title<'a>() -> Option<&'a str> { Some("BSpline Benchmark Animation") }

    fn depth_stencil_attachment_descriptor<'a>(
        &'a self,
    ) -> Option<RenderPassDepthStencilAttachmentDescriptor<'a>> {
        Some(self.scene.depth_stencil_attachment_descriptor())
    }

    fn update(&mut self, handler: &WGPUHandler) {
        self.update_surface();
        if self.time % 100 == 0 {
            let time = self.since.elapsed().as_secs_f64();
            println!("FPS: {}", 100.0 / time);
            self.since = std::time::Instant::now();
        }
        let (device, queue, sc_desc) = (&handler.device, &handler.queue, &handler.sc_desc);
        let (render_object, command_buffer) = self.mesher.meshing(&self.surface, device);
        queue.submit(&[command_buffer]);
        if self.scene.number_of_objects() > 0 {
            self.scene.remove_object(0);
        }
        self.scene.add_object(render_object);
        self.scene.prepare_render(device, sc_desc);
    }

    fn render<'a>(&'a self, rpass: &mut RenderPass<'a>) { self.scene.render_scene(rpass); }
}

#[allow(dead_code)]
async fn get_vertex(buffer: &Buffer, device: &Device) -> Vec<[[f32; 4]; 3]> {
    let buffer_future = buffer.map_read(0, VERTEX_SIZE);
    device.poll(wgpu::Maintain::Wait);

    if let Ok(mapping) = buffer_future.await {
        mapping
            .as_slice()
            .chunks_exact(48)
            .map(|b| {
                let vec = bytemuck::cast_slice::<u8, f32>(b);
                [
                    [vec[0], vec[1], vec[2], vec[3]],
                    [vec[4], vec[5], vec[6], vec[7]],
                    [vec[8], vec[9], vec[10], vec[11]],
                ]
            })
            .collect()
    } else {
        panic!("failed to run compute on gpu!")
    }
}

fn main() { MyApp::run(); }
