mod common;
use common::*;
use std::sync::{Arc, Mutex};
use wgpu::*;
use truck_platform::*;

#[test]
fn compare_pictures() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = common::swap_chain_descriptor();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture00 = device.create_texture(&tex_desc);
    let texture01 = device.create_texture(&tex_desc);
    let texture1 = device.create_texture(&tex_desc);
    let view00 = texture00.create_view(&Default::default());
    let view01 = texture01.create_view(&Default::default());
    let view1 = texture1.create_view(&Default::default());
    let mut plane00 = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: include_str!("shaders/compare0.frag"),
        id: Default::default(),
    };
    let mut plane01 = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: include_str!("shaders/compare0.frag"),
        id: Default::default(),
    };
    let mut plane1 = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: include_str!("shaders/compare1.frag"),
        id: Default::default(),
    };
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let mut scene = Scene::new(&device, &queue, &sc_desc, &Default::default());
    scene.add_object(&mut plane00);
    scene.prepare_render();
    scene.render_scene(&view00);
    scene.remove_object(&plane00);
    scene.add_object(&mut plane01);
    scene.prepare_render();
    scene.render_scene(&view01);
    scene.remove_object(&plane01);
    scene.add_object(&mut plane1);
    scene.prepare_render();
    scene.render_scene(&view1);
    scene.remove_object(&plane1);
    assert!(common::same_texture(scene.device_handler(), &texture00, &texture01));
    assert!(!common::same_texture(scene.device_handler(), &texture00, &texture1));
}
