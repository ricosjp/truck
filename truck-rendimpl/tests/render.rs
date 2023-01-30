mod common;
use common::Plane;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::sync::Arc;
use truck_meshalgo::prelude::obj;
use truck_modeling::*;
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

const CUBE_OBJ: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/obj/cube.obj",
));

const PICTURE_SIZE: (u32, u32) = (1024, 768);

fn test_scene(backend: Backends) -> Scene {
    let instance = wgpu::Instance::new(backend);
    let handler = common::init_device(&instance);
    Scene::new(
        handler,
        &SceneDescriptor {
            studio: StudioConfig {
                camera: Camera::perspective_camera(
                    Matrix4::look_at_rh(
                        Point3::new(-1.0, 2.5, 2.0),
                        Point3::new(0.25, 0.25, 0.25),
                        Vector3::unit_y(),
                    )
                    .invert()
                    .unwrap(),
                    Rad(std::f64::consts::PI / 4.0),
                    0.1,
                    100.0,
                ),
                lights: vec![Light {
                    position: Point3::new(-3.0, 4.0, -2.0),
                    color: Vector3::new(1.0, 1.0, 1.0),
                    light_type: LightType::Point,
                }],
                ..Default::default()
            },
            render_texture: RenderTextureConfig {
                canvas_size: PICTURE_SIZE,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

fn nontex_raytracing(scene: &mut Scene) -> Vec<u8> {
    let mut shader = include_str!("../src/shaders/microfacet-module.wgsl").to_string();
    shader += include_str!("shaders/raytraces.wgsl");
    let plane = Plane {
        shader: &shader,
        vs_entpt: "vs_main",
        fs_entpt: "nontex_raytracing",
        id: RenderID::gen(),
    };
    common::render_one(scene, &plane)
}

fn nontex_polygon(scene: &mut Scene, creator: &InstanceCreator) -> Vec<u8> {
    let cube: PolygonInstance = creator.create_instance(
        &obj::read(CUBE_OBJ).unwrap(),
        &PolygonState {
            material: Material {
                albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                roughness: 0.5,
                reflectance: 0.25,
                ambient_ratio: 0.02,
                background_ratio: 0.0,
                alpha_blend: false,
            },
            ..Default::default()
        },
    );
    common::render_one(scene, &cube)
}

fn exec_nontex_render_test(backend: Backends, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let mut scene = test_scene(backend);
    let creator = scene.instance_creator();
    let buffer0 = nontex_raytracing(&mut scene);
    let buffer1 = nontex_polygon(&mut scene, &creator);
    let filename = out_dir.clone() + "nontex-raytracing.png";
    common::save_buffer(filename, &buffer0, PICTURE_SIZE);
    let filename = out_dir + "nontex-polygon.png";
    common::save_buffer(filename, &buffer1, PICTURE_SIZE);
    let diff = common::count_difference(&buffer0, &buffer1);
    println!("{diff} pixel difference: ray-tracing and polymesh");
    assert!(diff < 10);
}

#[test]
fn nontex_render_test() { common::os_alt_exec_test(exec_nontex_render_test); }

fn generate_texture(scene: &mut Scene, out_dir: String) -> DynamicImage {
    let buffer = common::gradation_texture(scene);
    common::save_buffer(out_dir + "gradation-texture.png", &buffer, PICTURE_SIZE);
    let image_buffer =
        ImageBuffer::<Rgba<_>, _>::from_raw(PICTURE_SIZE.0, PICTURE_SIZE.1, buffer).unwrap();
    DynamicImage::ImageRgba8(image_buffer)
}

fn tex_raytracing(scene: &mut Scene) -> Vec<u8> {
    let shader = include_str!("../src/shaders/microfacet-module.wgsl").to_string()
        + include_str!("shaders/raytraces.wgsl");
    let plane = Plane {
        shader: &shader,
        vs_entpt: "vs_main",
        fs_entpt: "tex_raytracing",
        id: RenderID::gen(),
    };
    common::render_one(scene, &plane)
}

fn tex_polygon(
    scene: &mut Scene,
    creator: &InstanceCreator,
    gradtex: &Arc<DynamicImage>,
) -> Vec<u8> {
    let attach = creator.create_texture(gradtex);
    let cube: PolygonInstance = creator.create_instance(
        &obj::read(CUBE_OBJ).unwrap(),
        &PolygonState {
            material: Material {
                albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                roughness: 0.5,
                reflectance: 0.25,
                ambient_ratio: 0.02,
                background_ratio: 0.0,
                alpha_blend: false,
            },
            texture: Some(attach),
            ..Default::default()
        },
    );
    common::render_one(scene, &cube)
}

fn exec_tex_render_test(backend: Backends, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let mut scene = test_scene(backend);
    let creator = scene.instance_creator();
    let image = Arc::new(generate_texture(&mut scene, out_dir.clone()));
    let anti_buffer = nontex_raytracing(&mut scene);
    let buffer0 = tex_raytracing(&mut scene);
    let buffer1 = tex_polygon(&mut scene, &creator, &image);
    let filename = out_dir.clone() + "tex-raytracing.png";
    common::save_buffer(filename, &buffer0, PICTURE_SIZE);
    let filename = out_dir + "tex-polygon.png";
    common::save_buffer(filename, &buffer1, PICTURE_SIZE);
    let diff = common::count_difference(&buffer0, &buffer1);
    let anti_diff = common::count_difference(&anti_buffer, &buffer0);
    println!("{diff} pixel difference: ray-tracing and polymesh");
    assert!(diff < 10);
    assert!(anti_diff > 1000);
}

#[test]
fn tex_render_test() { common::os_alt_exec_test(exec_tex_render_test) }
