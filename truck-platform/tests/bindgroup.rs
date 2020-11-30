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
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let camera = Camera::perspective_camera(
        Matrix4::new(
            1.0, 2.1, 3.2, 4.3, 5.4, 6.5, 7.6, 8.7, 9.8, 10.9, 11.0, 12.0, 13.0, 14.0, 15.0, 16.23,
        ),
        Rad(std::f64::consts::PI / 4.0),
        0.1,
        10.0,
    );
    println!("{:?}", camera.projection(1.0));
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
    let mut plane = Plane {
        vertex_shader: include_str!("shaders/bindgroup.vert"),
        fragment_shader: include_str!("shaders/bindgroup.frag"),
        id: Default::default(),
    };
    scene.add_object(&mut plane);
    scene.prepare_render();
    scene.render_scene(&texture0.create_view(&Default::default()));
    scene.remove_object(&plane);
    let mut plane = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: include_str!("shaders/unicolor.frag"),
        id: Default::default(),
    };
    scene.add_object(&mut plane);
    scene.prepare_render();
    scene.render_scene(&texture1.create_view(&Default::default()));
    scene.remove_object(&plane);
    assert!(common::same_texture(
        scene.device_handler(),
        &texture0,
        &texture1
    ));
}
