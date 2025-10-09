//! An example of using texture.

// The texture is referenced by:
// https://cc0textures.com/view?id=WoodFloor024

use std::f64::consts::PI;
use std::sync::Arc;
use truck_meshalgo::prelude::*;
use truck_modeling::*;
use truck_platform::*;
use truck_polymesh::algo::DefaultSplitParams;
use truck_rendimpl::*;
use winit::{dpi::*, event::*, keyboard::*};
mod app;
use app::*;

const TEXTURE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/texture/WoodFloor024_1K_Color.png",
));

struct MyApp {
    scene: WindowScene,
    rotate_flag: bool,
    prev_cursor: Option<Vector2>,
    light_changed: Option<std::time::Instant>,
    camera_changed: Option<std::time::Instant>,
}

impl MyApp {
    fn create_camera() -> Camera {
        let matrix = Matrix4::look_at_rh(
            Point3::new(1.5, 1.5, 1.5),
            Point3::origin(),
            Vector3::unit_y(),
        );
        Camera {
            matrix: matrix.invert().unwrap(),
            method: ProjectionMethod::perspective(Rad(PI / 4.0)),
            near_clip: 0.1,
            far_clip: 40.0,
        }
    }

    fn create_cube() -> Solid {
        let v = builder::vertex(Point3::origin());
        let edge = builder::tsweep(&v, Vector3::unit_x());
        let face = builder::tsweep(&edge, Vector3::unit_y());
        builder::tsweep(&face, Vector3::unit_z())
    }
}

#[async_trait(?Send)]
impl App for MyApp {
    async fn init(window: Arc<winit::window::Window>) -> MyApp {
        let sample_count = 4;
        let desc = WindowSceneDescriptor {
            studio: StudioConfig {
                camera: MyApp::create_camera(),
                lights: vec![Light {
                    position: Point3::new(1.0, 1.0, 1.0),
                    color: Vector3::new(1.0, 1.0, 1.0),
                    light_type: LightType::Point,
                }],
                ..Default::default()
            },
            backend_buffer: BackendBufferConfig {
                sample_count,
                ..Default::default()
            },
        };
        let mut scene = WindowScene::from_window(window, &desc).await;
        let texture = image::load_from_memory(TEXTURE_BYTES).unwrap();
        let texture = image2texture::image2texture(scene.device_handler(), &texture);
        let state = PolygonState {
            matrix: Matrix4::from_translation(Vector3::new(-0.5, -0.5, -0.5)),
            material: Material {
                albedo: Vector4::new(0.402, 0.262, 0.176, 1.0),
                roughness: 0.9,
                reflectance: 0.04,
                ambient_ratio: 0.05,
                background_ratio: 0.0,
                alpha_blend: false,
            },
            texture: Some(std::sync::Arc::new(texture)),
            backface_culling: true,
        };
        let mesh = Self::create_cube().triangulation(DefaultSplitParams::new(0.05)).to_polygon();
        let shape: PolygonInstance = scene.instance_creator().create_instance(&mesh, &state);
        scene.add_object(&shape);
        MyApp {
            scene,
            rotate_flag: false,
            prev_cursor: None,
            camera_changed: None,
            light_changed: None,
        }
    }

    fn app_title<'a>() -> Option<&'a str> { Some("textured cube") }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> ControlFlow {
        match button {
            MouseButton::Left => {
                self.rotate_flag = state == ElementState::Pressed;
                if !self.rotate_flag {
                    self.prev_cursor = None;
                }
            }
            MouseButton::Right => {
                let (light, camera) = {
                    let desc = self.scene.studio_config_mut();
                    (&mut desc.lights[0], &desc.camera)
                };
                match light.light_type {
                    LightType::Point => {
                        light.position = camera.position();
                    }
                    LightType::Uniform => {
                        light.position = Point3::from_vec(camera.position().to_vec().normalize());
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
                let camera = &mut self.scene.studio_config_mut().camera;
                match &mut camera.method {
                    ProjectionMethod::Parallel { screen_size } => {
                        *screen_size *= 0.9f64.powf(y as f64);
                    }
                    ProjectionMethod::Perspective { .. } => {
                        let trans_vec = camera.eye_direction() * y as f64 * 0.2;
                        camera.matrix = Matrix4::from_translation(trans_vec) * camera.matrix;
                    }
                }
            }
            MouseScrollDelta::PixelDelta(_) => {}
        };
        Self::default_control_flow()
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) -> ControlFlow {
        if self.rotate_flag {
            let position = Vector2::new(position.x, position.y);
            if let Some(ref prev_position) = self.prev_cursor {
                let matrix = &mut self.scene.studio_config_mut().camera.matrix;
                let dir2d = position - prev_position;
                if dir2d.so_small() {
                    return Self::default_control_flow();
                }
                let mut axis = dir2d[1] * matrix[0].truncate();
                axis += dir2d[0] * matrix[1].truncate();
                axis /= axis.magnitude();
                let angle = dir2d.magnitude() * 0.01;
                let mat = Matrix4::from_axis_angle(axis, Rad(angle));
                *matrix = mat.invert().unwrap() * *matrix;
            }
            self.prev_cursor = Some(position);
        }
        Self::default_control_flow()
    }
    fn keyboard_input(&mut self, input: KeyEvent, _: bool) -> ControlFlow {
        let keycode = match input.physical_key {
            PhysicalKey::Code(keycode) => keycode,
            _ => return Self::default_control_flow(),
        };
        match keycode {
            KeyCode::KeyP => {
                if let Some(ref instant) = self.camera_changed {
                    let time = instant.elapsed().as_secs_f64();
                    if time < 0.2 {
                        return Self::default_control_flow();
                    }
                }
                let camera = &mut self.scene.studio_config_mut().camera;
                self.camera_changed = Some(std::time::Instant::now());
                camera.method = match camera.method {
                    ProjectionMethod::Parallel { .. } => {
                        ProjectionMethod::perspective(Rad(PI / 4.0))
                    }
                    ProjectionMethod::Perspective { .. } => ProjectionMethod::parallel(1.0),
                };
            }
            KeyCode::KeyL => {
                if let Some(ref instant) = self.light_changed {
                    let time = instant.elapsed().as_secs_f64();
                    if time < 0.2 {
                        return Self::default_control_flow();
                    }
                }
                self.light_changed = Some(std::time::Instant::now());
                let (light, camera) = {
                    let desc = self.scene.studio_config_mut();
                    (&mut desc.lights[0], &desc.camera)
                };
                *light = match light.light_type {
                    LightType::Point => {
                        let position = Point3::from_vec(camera.position().to_vec().normalize());
                        Light {
                            position,
                            color: Vector3::new(1.0, 1.0, 1.0),
                            light_type: LightType::Uniform,
                        }
                    }
                    LightType::Uniform => {
                        let position = camera.position();
                        Light {
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

    fn render(&mut self) { self.scene.render_frame(); }
}

fn main() { MyApp::run(); }
