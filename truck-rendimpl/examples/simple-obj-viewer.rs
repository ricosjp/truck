//! Simple OBJ viewer
//!
//! - Drag the mouse to rotate the model.
//! - Drag and drop obj files into the window to switch models.
//! - Right-click to move the light to the camera's position.
//! - Enter "P" on the keyboard to switch between parallel projection and perspective projection of the camera.
//! - Enter "L" on the keyboard to switch the point light source/uniform light source of the light.
//! - Enter "Space" on the keyboard to switch the rendering mode for the wireframe and surface.

use std::io::Read;
use std::sync::Arc;
use truck_meshalgo::prelude::*;
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;
use winit::{dpi::*, event::*, keyboard::*};
mod app;
use app::*;

const TEAPOT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/obj/teapot.obj",
));

enum RenderMode {
    NaiveSurface,
    NaiveWireFrame,
    HiddenLineEliminate,
    SurfaceAndWireFrame,
}

struct MyApp {
    scene: WindowScene,
    creator: InstanceCreator,
    rotate_flag: bool,
    prev_cursor: Vector2,
    instance: PolygonInstance,
    wireframe: WireFrameInstance,
    render_mode: RenderMode,
}

impl MyApp {
    fn create_camera() -> Camera {
        let matrix = Matrix4::look_at_rh(
            Point3::new(1.0, 1.0, 1.0),
            Point3::origin(),
            Vector3::unit_y(),
        );
        Camera::perspective_camera(
            matrix.invert().unwrap(),
            Rad(std::f64::consts::PI / 4.0),
            0.1,
            40.0,
        )
    }

    fn update_render_mode(&mut self) {
        match self.render_mode {
            RenderMode::NaiveSurface => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    reflectance: 0.5,
                    roughness: 0.1,
                    ambient_ratio: 0.02,
                    background_ratio: 0.0,
                    alpha_blend: false,
                };
                self.scene.update_bind_group(&self.instance);
                self.scene.set_visibility(&self.instance, true);
                self.scene.set_visibility(&self.wireframe, false);
            }
            RenderMode::NaiveWireFrame => {
                self.wireframe.instance_state_mut().color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                self.scene.update_bind_group(&self.wireframe);
                self.scene.set_visibility(&self.instance, false);
                self.scene.set_visibility(&self.wireframe, true);
            }
            RenderMode::HiddenLineEliminate => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(0.0, 0.0, 0.0, 1.0),
                    reflectance: 0.0,
                    roughness: 0.0,
                    ambient_ratio: 1.0,
                    background_ratio: 0.0,
                    alpha_blend: false,
                };
                self.wireframe.instance_state_mut().color = Vector4::new(1.0, 1.0, 1.0, 1.0);
                self.scene.update_bind_group(&self.instance);
                self.scene.update_bind_group(&self.wireframe);
                self.scene.set_visibility(&self.instance, true);
                self.scene.set_visibility(&self.wireframe, true);
            }
            RenderMode::SurfaceAndWireFrame => {
                self.instance.instance_state_mut().material = Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    reflectance: 0.5,
                    roughness: 0.1,
                    ambient_ratio: 0.02,
                    background_ratio: 0.0,
                    alpha_blend: false,
                };
                self.wireframe.instance_state_mut().color = Vector4::new(0.0, 0.0, 0.0, 1.0);
                self.scene.update_bind_group(&self.instance);
                self.scene.update_bind_group(&self.wireframe);
                self.scene.set_visibility(&self.instance, true);
                self.scene.set_visibility(&self.wireframe, true);
            }
        }
    }

    fn load_obj<R: Read>(
        creator: &InstanceCreator,
        reader: R,
    ) -> (PolygonInstance, WireFrameInstance) {
        let mut mesh = obj::read(reader).unwrap();
        mesh.put_together_same_attrs(TOLERANCE * 2.0)
            .add_smooth_normals(0.5, false);
        let bdd_box = mesh.bounding_box();
        let (size, center) = (bdd_box.size(), bdd_box.center());
        let mat = Matrix4::from_translation(center.to_vec()) * Matrix4::from_scale(size);
        let polygon_state = PolygonState {
            matrix: mat.invert().unwrap(),
            ..Default::default()
        };
        let wire_state = WireFrameState {
            matrix: mat.invert().unwrap(),
            ..Default::default()
        };
        (
            creator.create_instance(&mesh, &polygon_state),
            creator.create_instance(&mesh, &wire_state),
        )
    }
}

#[async_trait(?Send)]
impl App for MyApp {
    async fn init(window: Arc<winit::window::Window>) -> MyApp {
        let sample_count = 4;
        let scene_desc = WindowSceneDescriptor {
            studio: StudioConfig {
                background: Color::BLACK,
                camera: MyApp::create_camera(),
                lights: vec![Light {
                    position: Point3::new(1.0, 1.0, 1.0),
                    color: Vector3::new(1.0, 1.0, 1.0),
                    light_type: LightType::Point,
                }],
            },
            backend_buffer: BackendBufferConfig {
                sample_count,
                ..Default::default()
            },
        };
        let mut scene = WindowScene::from_window(window, &scene_desc).await;
        let creator = scene.instance_creator();
        let (instance, wireframe) = MyApp::load_obj(&creator, TEAPOT_BYTES);
        scene.add_object(&instance);
        scene.add_object(&wireframe);
        let mut app = MyApp {
            scene,
            creator,
            rotate_flag: false,
            prev_cursor: Vector2::zero(),
            instance,
            wireframe,
            render_mode: RenderMode::NaiveSurface,
        };
        app.update_render_mode();
        app
    }

    fn app_title<'a>() -> Option<&'a str> {
        Some("simple obj viewer")
    }

    fn dropped_file(&mut self, path: std::path::PathBuf) -> ControlFlow {
        let file = std::fs::File::open(path).unwrap();
        self.scene.clear_objects();
        let (instance, wireframe) = MyApp::load_obj(&self.creator, file);
        self.scene.add_object(&instance);
        self.scene.add_object(&wireframe);
        self.instance = instance;
        self.wireframe = wireframe;
        self.update_render_mode();
        Self::default_control_flow()
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> ControlFlow {
        match button {
            MouseButton::Left => {
                self.rotate_flag = state == ElementState::Pressed;
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
                        light.position = camera.position();
                        let strength = light.position.to_vec().magnitude();
                        light.position /= strength;
                    }
                }
            }
            _ => {}
        }
        Self::default_control_flow()
    }
    fn mouse_wheel(&mut self, delta: MouseScrollDelta, _: TouchPhase) -> ControlFlow {
        match delta {
            MouseScrollDelta::LineDelta(_, d) => {
                let camera = &mut self.scene.studio_config_mut().camera;
                let d = d as f64;
                let factor = 0.2;
                match camera.projection_type() {
                    ProjectionType::Perspective => {
                        let t = camera.eye_direction() * factor * d;
                        camera.matrix.w += t.extend(0.)
                    }
                    ProjectionType::Parallel => {
                        let mut scale = 1.0;
                        if d > 0. {
                            scale += factor
                        } else {
                            scale -= factor
                        }
                        camera.matrix.w[3] *= scale;
                    }
                }
            }
            MouseScrollDelta::PixelDelta(_) => {}
        };
        Self::default_control_flow()
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) -> ControlFlow {
        let position = Vector2::new(position.x, position.y);
        if self.rotate_flag {
            let matrix = &mut self.scene.studio_config_mut().camera.matrix;
            let position = Vector2::new(position.x, position.y);
            let dir2d = position - self.prev_cursor;
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
        self.prev_cursor = position;
        Self::default_control_flow()
    }
    fn keyboard_input(&mut self, input: KeyEvent, _: bool) -> ControlFlow {
        if input.state == ElementState::Released {
            return Self::default_control_flow();
        }
        let keycode = match input.physical_key {
            PhysicalKey::Code(keycode) => keycode,
            _ => return Self::default_control_flow(),
        };
        match keycode {
            KeyCode::KeyP => {
                let camera = &mut self.scene.studio_config_mut().camera;
                *camera = match camera.projection_type() {
                    ProjectionType::Parallel => Camera::perspective_camera(
                        camera.matrix,
                        Rad(std::f64::consts::PI / 4.0),
                        0.1,
                        40.0,
                    ),
                    ProjectionType::Perspective => {
                        Camera::parallel_camera(camera.matrix, 1.0, 0.1, 40.0)
                    }
                };
            }
            KeyCode::KeyL => {
                let (light, camera) = {
                    let desc = self.scene.studio_config_mut();
                    (&mut desc.lights[0], &desc.camera)
                };
                *light = match light.light_type {
                    LightType::Point => {
                        let mut vec = camera.position();
                        vec /= vec.to_vec().magnitude();
                        Light {
                            position: vec,
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
                };
            }
            KeyCode::Space => {
                self.render_mode = match self.render_mode {
                    RenderMode::NaiveSurface => RenderMode::SurfaceAndWireFrame,
                    RenderMode::SurfaceAndWireFrame => RenderMode::NaiveWireFrame,
                    RenderMode::NaiveWireFrame => RenderMode::HiddenLineEliminate,
                    RenderMode::HiddenLineEliminate => RenderMode::NaiveSurface,
                };
                self.update_render_mode();
            }
            _ => {}
        }
        Self::default_control_flow()
    }

    fn render(&mut self) {
        self.scene.render_frame();
    }
}

fn main() {
    MyApp::run();
}
