mod common;
use common::Plane;
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

fn exec_bind_group_test(backend: Backends, out_dir: &str) {
    let out_dir = String::from(out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    let camera =
        Camera::perspective_camera(CAMERA_MATRIX, CAMERA_FOV, CAMERA_NEARCLIP, CAMERA_FARCLIP);
    println!("camera projection:\n{:?}", camera.projection(PICTURE_ASP));
    let lights = vec![POINT_LIGHT, UNIFORM_LIGHT];
    let desc = SceneDescriptor {
        studio: StudioConfig {
            camera,
            lights,
            background: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 0.4,
            },
        },
        render_texture: RenderTextureConfig {
            canvas_size: (PICTURE_WIDTH, PICTURE_HEIGHT),
            format: TextureFormat::Rgba8Unorm,
        },
        ..Default::default()
    };
    let handler = common::init_device(backend);
    let mut scene = Scene::new(handler, &desc);
    let plane = new_plane!("shaders/unicolor.wgsl", "vs_main", "fs_main");
    let buffer0 = common::render_one(&mut scene, &plane);
    let plane = new_plane!("shaders/bindgroup.wgsl", "vs_main", "fs_main");
    let buffer1 = common::render_one(&mut scene, &plane);
    let plane = new_plane!("shaders/bindgroup.wgsl", "vs_main", "fs_main_anti");
    let buffer2 = common::render_one(&mut scene, &plane);
    save_buffer(out_dir.clone() + "unicolor.png", &buffer0);
    save_buffer(out_dir.clone() + "bindgroup.png", &buffer1);
    save_buffer(out_dir + "anti-bindgroup.png", &buffer2);
    assert!(common::same_buffer(&buffer0, &buffer1));
    assert!(!common::same_buffer(&buffer0, &buffer2));
}

#[test]
fn bind_group_test() { common::os_alt_exec_test(exec_bind_group_test); }
