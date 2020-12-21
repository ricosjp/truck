mod common;
use common::Plane;
use image::{ColorType, DynamicImage, ImageBuffer, Rgba};
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

const PICTURE_SIZE: (u32, u32) = (1024, 768);

fn test_scene() -> Scene {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = common::swap_chain_descriptor(PICTURE_SIZE);
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    Scene::new(
        handler,
        &SceneDescriptor {
            camera: Camera::perspective_camera(
                Matrix4::look_at(
                    Point3::new(-2.0, 5.0, 4.0),
                    Point3::origin(),
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

fn polygon_cube() -> PolygonMesh {
    let positions = vec![
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 1.0, 1.0),
        Point3::new(1.0, 0.0, 1.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 1.0),
    ];
    let uv_coords = vec![
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 1.0),
    ];
    let normals = vec![
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(-1.0, 0.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, 0.0, -1.0),
    ];
    let quad_faces = vec![
        [[0, 0, 4], [1, 1, 4], [5, 2, 4], [3, 3, 4]],
        [[0, 0, 5], [2, 1, 5], [6, 2, 5], [1, 3, 5]],
        [[0, 0, 3], [3, 1, 3], [4, 2, 3], [2, 3, 3]],
        [[7, 0, 2], [4, 1, 2], [3, 2, 2], [5, 3, 2]],
        [[7, 0, 1], [6, 1, 1], [2, 2, 1], [4, 3, 1]],
        [[7, 0, 0], [5, 1, 0], [1, 2, 0], [6, 3, 0]],
    ];
    PolygonMesh {
        positions,
        uv_coords,
        normals,
        quad_faces,
        ..Default::default()
    }
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
        id: Default::default(),
    };
    common::render_one(scene, &texture, &plane);    
    common::read_texture(scene.device_handler(), &texture)
}

fn nontex_polygon(scene: &mut Scene) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let cube = scene.create_instance(
        &polygon_cube(),
        &InstanceDescriptor {
            material: Material {
                albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                roughness: 0.5,
                reflectance: 0.25,
                ambient_ratio: 0.02,
            },
            ..Default::default()
        },
    );
    common::render_one(scene, &texture, &cube);
    common::read_texture(scene.device_handler(), &texture)
}

fn nontex_shape(scene: &mut Scene) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let cube = scene.create_instance(
        &shape_cube(),
        &InstanceDescriptor {
            material: Material {
                albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                roughness: 0.5,
                reflectance: 0.25,
                ambient_ratio: 0.02,
            },
            ..Default::default()
        },
    );
    common::render_ones(scene, &texture, &cube.render_faces());
    common::read_texture(scene.device_handler(), &texture)
}

#[test]
fn nontex_render_test() {
    let mut scene = test_scene();
    let buffer0 = nontex_raymarching(&mut scene);
    let buffer1 = nontex_polygon(&mut scene);
    let buffer2 = nontex_shape(&mut scene);
    image::save_buffer(
        "nontex-raymarching.png",
        &buffer0,
        PICTURE_SIZE.0,
        PICTURE_SIZE.1,
        ColorType::Rgba8,
    ).unwrap();
    image::save_buffer(
        "nontex-polygon.png",
        &buffer1,
        PICTURE_SIZE.0,
        PICTURE_SIZE.1,
        ColorType::Rgba8,
    ).unwrap();
    image::save_buffer(
        "nontex-shape.png",
        &buffer2,
        PICTURE_SIZE.0,
        PICTURE_SIZE.1,
        ColorType::Rgba8,
    ).unwrap();
    let whole_rgb = (PICTURE_SIZE.0 * PICTURE_SIZE.1 * 3) as f64;
    let diff0 = common::buffer_difference(&buffer0, &buffer1) / whole_rgb;
    let diff1 = common::buffer_difference(&buffer1, &buffer2) / whole_rgb;
    let diff2 = common::buffer_difference(&buffer2, &buffer0) / whole_rgb;
    println!("{}% difference: ray-marching and polymesh", diff0 * 100.0);
    println!("{}% difference: polymesh and shape", diff1 * 100.0);
    println!("{}% difference: ray-marching and shape", diff2 * 100.0);
    assert!(diff0 < 1.0e-3);
    assert!(diff1 < 1.0e-3);
    assert!(diff2 < 1.0e-3);
}

fn generate_texture(scene: &mut Scene) -> DynamicImage {
    let texture = common::gradation_texture(scene);
    let buffer = common::read_texture(scene.device_handler(), &texture);
    image::save_buffer(
        "gradation-texture.png",
        &buffer,
        PICTURE_SIZE.0,
        PICTURE_SIZE.1,
        ColorType::Rgba8,
    )
    .unwrap();
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
        id: Default::default(),
    };
    common::render_one(scene, &texture, &plane);    
    common::read_texture(scene.device_handler(), &texture)
}

fn tex_polygon(scene: &mut Scene, gradtex: &Arc<DynamicImage>) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let cube = scene.create_instance(
        &polygon_cube(),
        &InstanceDescriptor {
            material: Material {
                albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                roughness: 0.5,
                reflectance: 0.25,
                ambient_ratio: 0.02,
            },
            texture: Some(Arc::clone(gradtex)),
            ..Default::default()
        },
    );
    common::render_one(scene, &texture, &cube);
    common::read_texture(scene.device_handler(), &texture)
}

fn tex_shape(scene: &mut Scene, gradtex: &Arc<DynamicImage>) -> Vec<u8> {
    let (device, sc_desc) = (scene.device(), scene.sc_desc());
    let texture = device.create_texture(&common::texture_descriptor(&sc_desc));
    let cube = scene.create_instance(
        &shape_cube(),
        &InstanceDescriptor {
            material: Material {
                albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
                roughness: 0.5,
                reflectance: 0.25,
                ambient_ratio: 0.02,
            },
            texture: Some(Arc::clone(gradtex)),
            ..Default::default()
        },
    );
    common::render_ones(scene, &texture, &cube.render_faces());
    common::read_texture(scene.device_handler(), &texture)
}

#[test]
fn tex_render_test() {
    let mut scene = test_scene();
    let image = Arc::new(generate_texture(&mut scene));
    let anti_buffer = nontex_raymarching(&mut scene);
    let buffer0 = tex_raymarching(&mut scene);
    let buffer1 = tex_polygon(&mut scene, &image);
    let buffer2 = tex_shape(&mut scene, &image);
    image::save_buffer(
        "tex-raymarching.png",
        &buffer0,
        PICTURE_SIZE.0,
        PICTURE_SIZE.1,
        ColorType::Rgba8,
    ).unwrap();
    image::save_buffer(
        "tex-polygon.png",
        &buffer1,
        PICTURE_SIZE.0,
        PICTURE_SIZE.1,
        ColorType::Rgba8,
    ).unwrap();
    image::save_buffer(
        "tex-shape.png",
        &buffer2,
        PICTURE_SIZE.0,
        PICTURE_SIZE.1,
        ColorType::Rgba8,
    ).unwrap();
    let whole_rgb = (PICTURE_SIZE.0 * PICTURE_SIZE.1 * 3) as f64;
    let diff0 = common::buffer_difference(&buffer0, &buffer1) / whole_rgb;
    let diff1 = common::buffer_difference(&buffer1, &buffer2) / whole_rgb;
    let diff2 = common::buffer_difference(&buffer2, &buffer0) / whole_rgb;
    let anti_diff = common::buffer_difference(&anti_buffer, &buffer0) / whole_rgb;
    println!("{}% difference: ray-marching and polymesh", diff0 * 100.0);
    println!("{}% difference: polymesh and shape", diff1 * 100.0);
    println!("{}% difference: ray-marching and shape", diff2 * 100.0);
    assert!(diff0 < 1.0e-3);
    assert!(diff1 < 1.0e-3);
    assert!(diff2 < 1.0e-3);
    assert!(anti_diff > 1.0e-3);
}
