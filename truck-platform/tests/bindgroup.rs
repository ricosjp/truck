mod common;
use common::*;
use std::sync::{Arc, Mutex};
use truck_base::cgmath64::*;
use truck_platform::*;
use wgpu::*;

const PICTURE_WIDTH: u32 = 256;
const PICTURE_HEIGHT: u32 = 256;
const PICTURE_ASP: f64 = PICTURE_WIDTH as f64 / PICTURE_HEIGHT as f64;

const CAMERA_MATRIX: Matrix4 = Matrix4::from_cols(
    Vector4::new(1.0, 2.1, 3.2, 4.3),
    Vector4::new(5.4, 6.5, 7.6, 8.7),
    Vector4::new(9.8, 10.9, 11.0, 12.0),
    Vector4::new(13.0, 14.0, 15.0, 16.23),
);
const CAMERA_FOV: Rad<f64> = Rad(std::f64::consts::PI / 4.0);
const CAMERA_NEARCLIP: f64 = 0.1;
const CAMERA_FARCLIP: f64 = 10.0;

const POINT_LIGHT: Light = Light {
    position: Point3::new(0.1, 0.2, 0.3),
    color: Vector3::new(0.4, 0.5, 0.6),
    light_type: LightType::Point,
};
const UNIFORM_LIGHT: Light = Light {
    position: Point3::new(1.1, 1.2, 1.3),
    color: Vector3::new(1.4, 1.5, 1.6),
    light_type: LightType::Uniform,
};

fn init_device(instance: &Instance) -> (Arc<Device>, Arc<Queue>) {
    futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: None,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Default::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();
        (Arc::new(device), Arc::new(queue))
    })
}

fn save_buffer<P: AsRef<std::path::Path>>(path: P, vec: &Vec<u8>) {
    image::save_buffer(
        path,
        &vec,
        PICTURE_WIDTH,
        PICTURE_HEIGHT,
        image::ColorType::Rgba8,
    )
    .unwrap();
}

#[test]
fn bind_group_test() {
    std::fs::create_dir_all("output").unwrap();
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = init_device(&instance);
    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Rgba8UnormSrgb,
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        present_mode: PresentMode::Mailbox,
    };
    let texture0 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let texture1 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let texture2 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let camera = Camera::perspective_camera(
        CAMERA_MATRIX.into(),
        CAMERA_FOV,
        CAMERA_NEARCLIP,
        CAMERA_FARCLIP,
    );
    println!("camera projection:\n{:?}", camera.projection(PICTURE_ASP));
    let lights = vec![POINT_LIGHT, UNIFORM_LIGHT];
    let desc = SceneDescriptor {
        camera,
        lights,
        ..Default::default()
    };
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler.clone(), &desc);
    let plane = new_plane!("shaders/plane.vert", "shaders/unicolor.frag");
    render_one(&mut scene, &texture0, &plane);
    let plane = new_plane!("shaders/bindgroup.vert", "shaders/bindgroup.frag");
    render_one(&mut scene, &texture1, &plane);
    let plane = new_plane!("shaders/bindgroup.vert", "shaders/anti-bindgroup.frag");
    render_one(&mut scene, &texture2, &plane);
    let buffer0 = read_texture(&handler, &texture0);
    let buffer1 = read_texture(&handler, &texture1);
    let buffer2 = read_texture(&handler, &texture2);
    save_buffer("output/unicolor.png", &buffer0);
    save_buffer("output/bindgroup.png", &buffer1);
    save_buffer("output/anti-bindgroup.png", &buffer2);
    assert!(common::same_buffer(&buffer0, &buffer1));
    assert!(!common::same_buffer(&buffer0, &buffer2));
}
