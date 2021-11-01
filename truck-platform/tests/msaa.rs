mod common;
use common::Plane;
use truck_platform::*;
use wgpu::*;

pub const PICTURE_WIDTH: u32 = 512;
pub const PICTURE_HEIGHT: u32 = 512;

fn save_buffer<P: AsRef<std::path::Path>>(path: P, vec: &[u8]) {
    image::save_buffer(
        path,
        vec,
        PICTURE_WIDTH,
        PICTURE_HEIGHT,
        image::ColorType::Rgba8,
    )
    .unwrap();
}

fn exec_msaa_test(backend: Backends, out_dir: &str) {
    let out_dir = String::from(out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    let handler = common::init_device(backend);
    let mut scene = Scene::new(
        handler.clone(),
        &SceneDescriptor {
            backend_buffer: BackendBufferConfig {
                sample_count: 1,
                ..Default::default()
            },
            render_texture: RenderTextureConfig {
                canvas_size: (PICTURE_WIDTH, PICTURE_HEIGHT),
                format: TextureFormat::Rgba8UnormSrgb,
            },
            ..Default::default()
        },
    );
    let plane = new_plane!("shaders/trapezoid.wgsl", "vs_main", "fs_main");
    let buffer0 = common::render_one(&mut scene, &plane);
    save_buffer(out_dir.clone() + "sample_count_one.png", &buffer0);
    scene.descriptor_mut().backend_buffer.sample_count = 4;
    let buffer1 = common::render_one(&mut scene, &plane);
    save_buffer(out_dir + "sample_count_two.png", &buffer1);
    assert!(!common::same_buffer(&buffer0, &buffer1));
}

#[test]
fn msaa_test() { common::os_alt_exec_test(exec_msaa_test); }
