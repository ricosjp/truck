mod common;
use common::Plane;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use wgpu::*;

#[test]
fn microfacet_module_test() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor()));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::nontex_answer_texture(&mut scene);
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);

    let mut fragment_shader = "#version 450\n\n".to_string();
    fragment_shader += include_str!("../src/shaders/microfacet-module.frag");
    fragment_shader += include_str!("shaders/check-mf-module.frag");
    let mut plane = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: &fragment_shader,
        id: Default::default(),
    };
    common::render_one(&mut scene, &texture, &mut plane);
    assert!(common::same_texture(
        scene.device_handler(),
        &answer,
        &texture
    ));

    let mut fragment_shader = "#version 450\n\n".to_string();
    fragment_shader += include_str!("../src/shaders/microfacet-module.frag");
    fragment_shader += include_str!("shaders/anti-check-mf-module.frag");
    let mut plane = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: &fragment_shader,
        id: Default::default(),
    };
    common::render_one(&mut scene, &texture, &mut plane);
    assert!(!common::same_texture(
        scene.device_handler(),
        &answer,
        &texture
    ));
}
