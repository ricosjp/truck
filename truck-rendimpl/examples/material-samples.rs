//! Material Samples
//!
//! - The more right the model, the higher the reflectance.
//! - The upper the model, the higher the roughness.
//!
//! The most right and lowest model is black because it does not diffuse light
//! and no roughness in microfacet.

mod app;
use app::*;
use std::f64::consts::PI;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

const N: usize = 5;
const BACKGROUND: [f64; 4] = [45.0 / 255.0, 36.0 / 255.0, 42.0 / 255.0, 1.0];
const BOXCOLOR: [f64; 4] = [208.0 / 255.0, 176.0 / 255.0, 107.0 / 255.0, 1.0];

struct MyApp {
    scene: Scene,
    instances: Vec<PolygonInstance>,
    matrices: Vec<Matrix4>,
}

impl App for MyApp {
    fn init(handler: &DeviceHandler, _info: AdapterInfo) -> MyApp {
        let side_length = (N + 1) as f64 * 1.5;
        let camera_dist = side_length / 2.0 / (PI / 8.0).tan();
        let a = side_length / 2.0;
        let b = camera_dist / 2.0;
        let sample_count = 4;
        let scene_desc = SceneDescriptor {
            camera: Camera::perspective_camera(
                Matrix4::from_translation(camera_dist * Vector3::unit_z()),
                Rad(PI / 4.0),
                0.1,
                100.0,
            ),
            lights: vec![
                Light {
                    position: Point3::new(-a, -a, b),
                    color: Vector3::new(0.5, 0.5, 0.5),
                    light_type: LightType::Point,
                },
                Light {
                    position: Point3::new(-a, a, b),
                    color: Vector3::new(0.5, 0.5, 0.5),
                    light_type: LightType::Point,
                },
                Light {
                    position: Point3::new(a, -a, b),
                    color: Vector3::new(0.5, 0.5, 0.5),
                    light_type: LightType::Point,
                },
                Light {
                    position: Point3::new(a, a, b),
                    color: Vector3::new(0.5, 0.5, 0.5),
                    light_type: LightType::Point,
                },
            ],
            background: Color {
                r: BACKGROUND[0],
                g: BACKGROUND[1],
                b: BACKGROUND[2],
                a: BACKGROUND[3],
            },
            sample_count,
            ..Default::default()
        };
        let mut scene = Scene::new(handler.clone(), &scene_desc);
        let v = builder::vertex(Point3::new(-0.5, -0.5, -0.5));
        let e = builder::tsweep(&v, Vector3::unit_x());
        let f = builder::tsweep(&e, Vector3::unit_y());
        let cube = builder::tsweep(&f, Vector3::unit_z());
        let mesh = cube.triangulation(0.01).unwrap().to_polygon();
        let instance: PolygonInstance = scene
            .instance_creator()
            .create_instance(&mesh, &Default::default());
        let mut matrices = Vec::new();
        let instances: Vec<_> = (0..N)
            .flat_map(move |i| (0..N).map(move |j| (i, j)))
            .map(|(i, j)| {
                let mut instance = instance.clone_instance();
                let (s, t) = (i as f64 / (N - 1) as f64, j as f64 / (N - 1) as f64);
                let matrix = Matrix4::from_translation(Vector3::new(
                    1.5 * (i + 1) as f64 - side_length / 2.0,
                    1.5 * (j + 1) as f64 - side_length / 2.0,
                    0.0,
                ));
                matrices.push(matrix);
                *instance.instance_state_mut() = PolygonState {
                    matrix,
                    material: Material {
                        albedo: Vector4::from(BOXCOLOR),
                        reflectance: s,
                        roughness: t,
                        ambient_ratio: 0.02,
                        background_ratio: 0.0,
                        alpha_blend: false,
                    },
                    ..Default::default()
                };
                instance
            })
            .collect();
        instances.iter().for_each(|shape| {
            scene.add_object(shape);
        });
        MyApp {
            scene,
            instances,
            matrices,
        }
    }
    fn update(&mut self, _: &DeviceHandler) {
        let time = self.scene.elapsed().as_secs_f64();
        for (i, shape) in self.instances.iter_mut().enumerate() {
            let axis = if i % 2 == 0 {
                (-1.0_f64).powi(i as i32 / 2) * Vector3::unit_y()
            } else {
                -(-1.0_f64).powi(i as i32 / 2) * Vector3::unit_x()
            };
            shape.instance_state_mut().matrix =
                self.matrices[i] * Matrix4::from_axis_angle(axis, Rad(time * PI / 2.0));
            self.scene.update_bind_group(shape);
        }
    }
    fn render(&mut self, view: &TextureView) { self.scene.render_scene(view); }
}

fn main() { MyApp::run() }
