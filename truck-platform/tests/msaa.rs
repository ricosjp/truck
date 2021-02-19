mod common;
use common::Plane;
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

fn exec_msaa_test(backend: BackendBit, out_dir: &str) {
    let out_dir = String::from(out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Rgba8UnormSrgb,
        width: PICTURE_WIDTH,
        height: PICTURE_HEIGHT,
        present_mode: PresentMode::Mailbox,
    };
    let texture0 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let texture1 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(
        handler.clone(),
        &SceneDescriptor {
            sample_count: 1,
            ..Default::default()
        },
    );
    let plane = new_plane!("shaders/trapezoid.vert", "shaders/trapezoid.frag");
    common::render_one(&mut scene, &texture0, &plane);
    let buffer0 = common::read_texture(&handler, &texture0);
    save_buffer(out_dir.clone() + "sample_count_one.png", &buffer0);
    scene.descriptor_mut().sample_count = 2;
    let plane = new_plane!("shaders/trapezoid.vert", "shaders/trapezoid.frag");
    common::render_one(&mut scene, &texture1, &plane);
    let buffer1 = common::read_texture(&handler, &texture1);
    save_buffer(out_dir.clone() + "sample_count_two.png", &buffer1);
    assert!(!common::same_buffer(&buffer0, &buffer1));
}

#[test]
fn msaa_test() {
    if cfg!(windows) {
        exec_msaa_test(BackendBit::VULKAN, "output/vulkan/");
        exec_msaa_test(BackendBit::DX12, "output/dx12/");
    } else if cfg!(macos) {
        eprintln!("Metal is not compatible with wgpu MSAA.");
    } else {
        exec_msaa_test(BackendBit::VULKAN, "output/");
    }
}
