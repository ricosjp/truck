mod common;
use common::Plane;
use truck_platform::*;
use wgpu::*;

const PICTURE_SIZE: (u32, u32) = (256, 256);

fn exec_microfacet_module_test(backend: Backends, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = Instance::new(InstanceDescriptor {
        backends: backend,
        ..Default::default()
    });
    let handler = common::init_device(&instance);
    let mut scene = Scene::new(
        handler,
        &SceneDescriptor {
            render_texture: RenderTextureConfig {
                canvas_size: PICTURE_SIZE,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let answer = common::nontex_answer_texture(&mut scene);
    common::save_buffer(
        out_dir.clone() + "nontex-answer-texture.png",
        &answer,
        PICTURE_SIZE,
    );

    let mut shader = include_str!("../src/shaders/microfacet-module.wgsl").to_string();
    shader += include_str!("shaders/microfacet-module-test.wgsl");
    let mut plane = Plane {
        shader: &shader,
        vs_entpt: "vs_main",
        fs_entpt: "fs_main",
        id: RenderID::gen(),
    };
    let buffer0 = common::render_one(&mut scene, &plane);
    common::save_buffer(
        out_dir.clone() + "check-mf-module.png",
        &buffer0,
        PICTURE_SIZE,
    );
    assert!(common::same_buffer(&answer, &buffer0));

    plane.fs_entpt = "fs_main_anti";
    let buffer1 = common::render_one(&mut scene, &plane);
    common::save_buffer(out_dir + "anti-check-mf-module.png", &buffer1, PICTURE_SIZE);
    assert!(!common::same_buffer(&answer, &buffer1));
}

#[test]
fn microfacet_module_test() { common::os_alt_exec_test(exec_microfacet_module_test) }
