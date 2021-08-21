mod common;
use common::Plane;
use std::io::Write;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use wgpu::*;

pub const PICTURE_WIDTH: u32 = 512;
pub const PICTURE_HEIGHT: u32 = 512;

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

fn exec_msaa_test(backend: Backends, out_dir: &str) {
    let out_dir = String::from(out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: TextureFormat::Rgba8UnormSrgb,
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        present_mode: PresentMode::Mailbox,
    };
    let texture0 = device.create_texture(&common::texture_descriptor(&config));
    let texture1 = device.create_texture(&common::texture_descriptor(&config));
    let config = Arc::new(Mutex::new(config));
    let handler = DeviceHandler::new(device, queue, config);
    let mut scene = Scene::new(
        handler.clone(),
        &SceneDescriptor {
            sample_count: 1,
            ..Default::default()
        },
    );
    let plane = new_plane!("shaders/trapezoid.wgsl", "vs_main", "fs_main");
    common::render_one(&mut scene, &texture0, &plane);
    let buffer0 = common::read_texture(&handler, &texture0);
    save_buffer(out_dir.clone() + "sample_count_one.png", &buffer0);
    scene.descriptor_mut().sample_count = 2;
    common::render_one(&mut scene, &texture1, &plane);
    let buffer1 = common::read_texture(&handler, &texture1);
    save_buffer(out_dir.clone() + "sample_count_two.png", &buffer1);
    assert!(!common::same_buffer(&buffer0, &buffer1));
}

#[test]
fn msaa_test() {
    let _ = env_logger::try_init();
    if cfg!(target_os = "windows") {
        exec_msaa_test(Backends::VULKAN, "output/vulkan/");
        exec_msaa_test(Backends::DX12, "output/dx12/");
    } else if cfg!(target_os = "macos") {
        writeln!(
            &mut std::io::stderr(),
            "Metal is not compatible with wgpu MSAA."
        )
        .unwrap();
    } else {
        exec_msaa_test(Backends::VULKAN, "output/");
    }
}
