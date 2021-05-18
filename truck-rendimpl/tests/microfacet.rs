mod common;
use common::Plane;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use wgpu::*;

const PICTURE_SIZE: (u32, u32) = (256, 256);

fn exec_microfacet_module_test(backend: BackendBit, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor(PICTURE_SIZE)));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::nontex_answer_texture(&mut scene);
    let answer = common::read_texture(scene.device_handler(), &answer);
    common::save_buffer(
        out_dir.clone() + "nontex-answer-texture.png",
        &answer,
        PICTURE_SIZE,
    );
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);

    let mut shader = include_str!("../src/shaders/microfacet-module.wgsl").to_string();
    shader += include_str!("shaders/microfacet-module-test.wgsl");
    let mut plane = Plane {
        shader: &shader,
        vs_entpt: "vs_main",
        fs_entpt: "fs_main",
        id: RenderID::gen(),
    };
    common::render_one(&mut scene, &texture, &plane);
    let buffer0 = common::read_texture(scene.device_handler(), &texture);
    common::save_buffer(
        out_dir.clone() + "check-mf-module.png",
        &buffer0,
        PICTURE_SIZE,
    );
    assert!(common::same_buffer(&answer, &buffer0));

    plane.fs_entpt = "fs_main_anti";
    common::render_one(&mut scene, &texture, &plane);
    let buffer1 = common::read_texture(scene.device_handler(), &texture);
    common::save_buffer(
        out_dir.clone() + "anti-check-mf-module.png",
        &buffer1,
        PICTURE_SIZE,
    );
    assert!(!common::same_buffer(&answer, &buffer1));
}

#[test]
fn microfacet_module_test() { common::os_alt_exec_test(exec_microfacet_module_test) }
