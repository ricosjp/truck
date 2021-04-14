mod common;
use common::Plane;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

const PICTURE_SIZE: (u32, u32) = (1024, 768);

fn test_scene(backend: BackendBit) -> Scene {
    let instance = Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = common::swap_chain_descriptor(PICTURE_SIZE);
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    Scene::new(
        handler,
        &SceneDescriptor {
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
    )
}

fn shape_cube() -> Solid {
    let s = builder::vertex(Point3::new(0.0, 0.0, 0.0));
    let s = builder::tsweep(&s, Vector3::unit_x());
    let s = builder::tsweep(&s, Vector3::unit_y());
    builder::tsweep(&s, Vector3::unit_z())
}

fn nontex_raymarching(scene: &mut Scene) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let mut fragment_shader = "#version 450\n\n".to_string();
    fragment_shader += include_str!("../src/shaders/microfacet-module.frag");
    fragment_shader += include_str!("shaders/nontex-ray-marching.frag");
    let plane = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: &fragment_shader,
        id: RenderID::gen(),
    };
    common::render_one(scene, &texture, &plane);
    common::read_texture(scene.device_handler(), &texture)
}

fn nontex_polygon(scene: &mut Scene, creator: &InstanceCreator) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let cube: PolygonInstance = creator.create_instance(
        &obj::read(include_bytes!("cube.obj").as_ref()).unwrap(),
        &PolygonInstanceDescriptor {
            instance_state: InstanceState {
                material: Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    roughness: 0.5,
                    reflectance: 0.25,
                    ambient_ratio: 0.02,
                },
                ..Default::default()
            },
        },
    );
    common::render_one(scene, &texture, &cube);
    common::read_texture(scene.device_handler(), &texture)
}

fn nontex_shape(scene: &mut Scene, creator: &InstanceCreator) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let cube: ShapeInstance = creator.create_instance(
        &shape_cube(),
        &ShapeInstanceDescriptor {
            instance_state: InstanceState {
                material: Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    roughness: 0.5,
                    reflectance: 0.25,
                    ambient_ratio: 0.02,
                },
                ..Default::default()
            },
            ..Default::default()
        },
    );
    common::render_one(scene, &texture, &cube);
    common::read_texture(scene.device_handler(), &texture)
}

fn exec_nontex_render_test(backend: BackendBit, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let mut scene = test_scene(backend);
    let creator = scene.instance_creator();
    let buffer0 = nontex_raymarching(&mut scene);
    let buffer1 = nontex_polygon(&mut scene, &creator);
    let buffer2 = nontex_shape(&mut scene, &creator);
    let filename = out_dir.clone() + "nontex-raymarching.png";
    common::save_buffer(filename, &buffer0, PICTURE_SIZE);
    let filename = out_dir.clone() + "nontex-polygon.png";
    common::save_buffer(filename, &buffer1, PICTURE_SIZE);
    common::save_buffer(out_dir.clone() + "nontex-shape.png", &buffer2, PICTURE_SIZE);
    let diff0 = common::count_difference(&buffer0, &buffer1);
    let diff1 = common::count_difference(&buffer1, &buffer2);
    let diff2 = common::count_difference(&buffer2, &buffer0);
    println!("{} pixel difference: ray-marching and polymesh", diff0);
    println!("{} pixel difference: polymesh and shape", diff1);
    println!("{} pixel difference: ray-marching and shape", diff2);
    assert!(diff0 < 10);
    assert!(diff1 == 0);
    assert!(diff2 < 10);
}

#[test]
fn nontex_render_test() { common::os_alt_exec_test(exec_nontex_render_test); }

fn generate_texture(scene: &mut Scene, out_dir: String) -> DynamicImage {
    let texture = common::gradation_texture(scene);
    let buffer = common::read_texture(scene.device_handler(), &texture);
    common::save_buffer(out_dir + "gradation-texture.png", &buffer, PICTURE_SIZE);
    let image_buffer =
        ImageBuffer::<Rgba<_>, _>::from_raw(PICTURE_SIZE.0, PICTURE_SIZE.1, buffer).unwrap();
    DynamicImage::ImageRgba8(image_buffer)
}

fn tex_raymarching(scene: &mut Scene) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let mut fragment_shader = "#version 450\n\n".to_string();
    fragment_shader += include_str!("../src/shaders/microfacet-module.frag");
    fragment_shader += include_str!("shaders/tex-ray-marching.frag");
    let plane = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: &fragment_shader,
        id: RenderID::gen(),
    };
    common::render_one(scene, &texture, &plane);
    common::read_texture(scene.device_handler(), &texture)
}

fn tex_polygon(
    scene: &mut Scene,
    creator: &InstanceCreator,
    gradtex: &Arc<DynamicImage>,
) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let attach = creator.create_texture(gradtex);
    let cube: PolygonInstance = creator.create_instance(
        &obj::read(include_bytes!("cube.obj").as_ref()).unwrap(),
        &PolygonInstanceDescriptor {
            instance_state: InstanceState {
                material: Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    roughness: 0.5,
                    reflectance: 0.25,
                    ambient_ratio: 0.02,
                },
                texture: Some(attach),
                ..Default::default()
            },
        },
    );
    common::render_one(scene, &texture, &cube);
    common::read_texture(scene.device_handler(), &texture)
}

fn tex_shape(scene: &mut Scene, creator: &InstanceCreator, gradtex: &Arc<DynamicImage>) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let attach = creator.create_texture(gradtex);
    let cube: ShapeInstance = creator.create_instance(
        &shape_cube(),
        &ShapeInstanceDescriptor {
            instance_state: InstanceState {
                material: Material {
                    albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    roughness: 0.5,
                    reflectance: 0.25,
                    ambient_ratio: 0.02,
                },
                texture: Some(attach),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    common::render_one(scene, &texture, &cube);
    common::read_texture(scene.device_handler(), &texture)
}

fn exec_tex_render_test(backend: BackendBit, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let mut scene = test_scene(backend);
    let creator = scene.instance_creator();
    let image = Arc::new(generate_texture(&mut scene, out_dir.clone()));
    let anti_buffer = nontex_raymarching(&mut scene);
    let buffer0 = tex_raymarching(&mut scene);
    let buffer1 = tex_polygon(&mut scene, &creator, &image);
    let buffer2 = tex_shape(&mut scene, &creator, &image);
    let filename = out_dir.clone() + "tex-raymarching.png";
    common::save_buffer(filename, &buffer0, PICTURE_SIZE);
    let filename = out_dir.clone() + "tex-polygon.png";
    common::save_buffer(filename, &buffer1, PICTURE_SIZE);
    common::save_buffer(out_dir.clone() + "tex-shape.png", &buffer2, PICTURE_SIZE);
    let diff0 = common::count_difference(&buffer0, &buffer1);
    let diff1 = common::count_difference(&buffer1, &buffer2);
    let diff2 = common::count_difference(&buffer2, &buffer0);
    let anti_diff = common::count_difference(&anti_buffer, &buffer0);
    println!("{} pixel difference: ray-marching and polymesh", diff0);
    println!("{} pixel difference: polymesh and shape", diff1);
    println!("{} pixel difference: ray-marching and shape", diff2);
    assert!(diff0 < 10);
    assert!(diff1 == 0);
    assert!(diff2 < 10);
    assert!(anti_diff > 1000);
}

#[test]
fn tex_render_test() { common::os_alt_exec_test(exec_tex_render_test) }
