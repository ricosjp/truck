mod common;
use common::Plane;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use wgpu::*;

const PICTURE_SIZE: (u32, u32) = (256, 256);

fn save_buffer<P: AsRef<std::path::Path>>(path: P, vec: &Vec<u8>) {
    image::save_buffer(
        path,
        &vec,
        PICTURE_SIZE.0,
        PICTURE_SIZE.1,
        image::ColorType::Rgba8,
    )
    .unwrap();
}

#[test]
fn microfacet_module_test() {
    std::fs::create_dir_all("output").unwrap();
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor(PICTURE_SIZE)));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::nontex_answer_texture(&mut scene);
    let answer = common::read_texture(scene.device_handler(), &answer);
    save_buffer("output/nontex-answer-texture.png", &answer);
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);

    let mut fragment_shader = "#version 450\n\n".to_string();
    fragment_shader += include_str!("../src/shaders/microfacet-module.frag");
    fragment_shader += include_str!("shaders/check-mf-module.frag");
    let mut plane = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: &fragment_shader,
        id: RenderID::gen(),
    };
    common::render_one(&mut scene, &texture, &mut plane);
    let buffer0 = common::read_texture(scene.device_handler(), &texture);
    save_buffer("output/check-mf-module.png", &buffer0);
    assert!(common::same_buffer(&answer, &buffer0));

    let mut fragment_shader = "#version 450\n\n".to_string();
    fragment_shader += include_str!("../src/shaders/microfacet-module.frag");
    fragment_shader += include_str!("shaders/anti-check-mf-module.frag");
    let plane = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: &fragment_shader,
        id: RenderID::gen(),
    };
    common::render_one(&mut scene, &texture, &plane);
    let buffer1 = common::read_texture(scene.device_handler(), &texture);
    save_buffer("output/anti-check-mf-module.png", &buffer1);
    assert!(!common::same_buffer(&answer, &buffer1));
}
