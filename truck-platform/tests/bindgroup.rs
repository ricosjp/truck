mod common;
use common::*;
use std::sync::{Arc, Mutex};
use truck_base::cgmath64::*;
use truck_platform::*;
use wgpu::*;

#[test]
fn bind_group_test() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = common::swap_chain_descriptor();
    let texture0 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let texture1 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let texture2 = device.create_texture(&common::texture_descriptor(&sc_desc));
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let camera = Camera::perspective_camera(
        Matrix4::from_cols(
            [1.0, 2.1, 3.2, 4.3].into(),
            [5.4, 6.5, 7.6, 8.7].into(),
            [9.8, 10.9, 11.0, 12.0].into(),
            [13.0, 14.0, 15.0, 16.23].into(),
        ),
        Rad(std::f64::consts::PI / 4.0),
        0.1,
        10.0,
    );
    println!("camera projection matrix:\n{:?}", camera.projection(1.0));
    let lights = vec![
        Light {
            position: Point3::new(0.1, 0.2, 0.3),
            color: Vector3::new(0.4, 0.5, 0.6),
            light_type: LightType::Point,
        },
        Light {
            position: Point3::new(1.1, 1.2, 1.3),
            color: Vector3::new(1.4, 1.5, 1.6),
            light_type: LightType::Uniform,
        },
    ];
    let desc = SceneDescriptor {
        camera,
        lights,
        ..Default::default()
    };
    let mut scene = Scene::new(&device, &queue, &sc_desc, &desc);
    let mut plane = new_plane!("shaders/plane.vert", "shaders/unicolor.frag");
    render_one(&mut scene, &texture0, &mut plane);
    let mut plane = new_plane!("shaders/bindgroup.vert", "shaders/bindgroup.frag");
    render_one(&mut scene, &texture1, &mut plane);
    let mut plane = new_plane!("shaders/bindgroup.vert", "shaders/anti-bindgroup.frag");
    render_one(&mut scene, &texture2, &mut plane);
    let handler = scene.device_handler();
    assert!(common::same_texture(handler, &texture0, &texture1));
    assert!(!common::same_texture(handler, &texture0, &texture2));
}
