mod common;
use common::Plane;
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

fn exec_bind_group_test(backend: BackendBit, out_dir: &str) {
    let out_dir = String::from(out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
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
    println!("first plane");
    let plane = new_plane!("shaders/unicolor.wgsl", "vs_main", "fs_main");
    common::render_one(&mut scene, &texture0, &plane);
    println!("second plane");
    let plane = new_plane!("shaders/bindgroup.wgsl", "vs_main", "fs_main");
    common::render_one(&mut scene, &texture1, &plane);
    println!("third plane");
    let plane = new_plane!("shaders/bindgroup.wgsl", "vs_main", "fs_main_anti");
    common::render_one(&mut scene, &texture2, &plane);
    let buffer0 = common::read_texture(&handler, &texture0);
    let buffer1 = common::read_texture(&handler, &texture1);
    let buffer2 = common::read_texture(&handler, &texture2);
    save_buffer(out_dir.clone() + "unicolor.png", &buffer0);
    save_buffer(out_dir.clone() + "bindgroup.png", &buffer1);
    save_buffer(out_dir.clone() + "anti-bindgroup.png", &buffer2);
    assert!(common::same_buffer(&buffer0, &buffer1));
    assert!(!common::same_buffer(&buffer0, &buffer2));
}

#[test]
fn bind_group_test() {
    let _ = env_logger::try_init();
    if cfg!(target_os = "windows") {
        exec_bind_group_test(BackendBit::VULKAN, "output/vulkan/");
        exec_bind_group_test(BackendBit::DX12, "output/dx12/");
    } else if cfg!(target_os = "macos") {
        exec_bind_group_test(BackendBit::METAL, "output/");
    } else {
        exec_bind_group_test(BackendBit::VULKAN, "output/");
    }
}
