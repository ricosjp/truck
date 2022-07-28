mod common;
use common::Plane;
use truck_platform::*;
use wgpu::*;

const PICTURE_WIDTH: u32 = 256;
const PICTURE_HEIGHT: u32 = 256;

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

fn exec_math_util_test(backend: Backends, out_dir: &str) {
    let out_dir = String::from(out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    let desc = SceneDescriptor {
        render_texture: RenderTextureConfig {
            canvas_size: (PICTURE_WIDTH, PICTURE_HEIGHT),
            format: TextureFormat::Rgba8Unorm,
        },
        ..Default::default()
    };
    let mut scene = Scene::new(common::init_device(backend), &desc);
    let plane = new_plane!("shaders/unicolor.wgsl", "vs_main", "fs_main");
    let buffer0 = common::render_one(&mut scene, &plane);
    let shader = include_str!("../wgsl-utils/math.wgsl").to_string()
        + include_str!("shaders/math-util.wgsl");
    let plane = Plane {
        shader: &shader,
        vs_entpt: "vs_main",
        fs_entpt: "fs_main",
        id: RenderID::gen(),
    };
    let buffer1 = common::render_one(&mut scene, &plane);
    let plane = Plane {
        shader: &shader,
        vs_entpt: "vs_main",
        fs_entpt: "fs_main_anti",
        id: RenderID::gen(),
    };
    let buffer2 = common::render_one(&mut scene, &plane);
    save_buffer(out_dir.clone() + "unicolor.png", &buffer0);
    save_buffer(out_dir.clone() + "math-util.png", &buffer1);
    save_buffer(out_dir + "anti-math-util.png", &buffer2);
    assert!(common::same_buffer(&buffer0, &buffer1));
    assert!(!common::same_buffer(&buffer0, &buffer2));
}

#[test]
fn math_util_test() { common::os_alt_exec_test(exec_math_util_test); }
