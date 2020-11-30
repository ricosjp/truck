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
    let tex_desc = TextureDescriptor {
        label: None,
        size: common::extend3d(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: sc_desc.format,
        usage: sc_desc.usage | TextureUsage::COPY_SRC,
    };
    let texture00 = device.create_texture(&tex_desc);
    let texture01 = device.create_texture(&tex_desc);
    let texture1 = device.create_texture(&tex_desc);
    let view00 = texture00.create_view(&Default::default());
    let view01 = texture01.create_view(&Default::default());
    let view1 = texture1.create_view(&Default::default());
    let mut plane00 = Plane {
        vertex_shader: include_str!("shaders/compare.vert"),
        fragment_shader: include_str!("shaders/compare0.frag"),
        id: Default::default(),
    };
    let mut plane01 = Plane {
        vertex_shader: include_str!("shaders/compare.vert"),
        fragment_shader: include_str!("shaders/compare0.frag"),
        id: Default::default(),
    };
    let mut plane1 = Plane {
        vertex_shader: include_str!("shaders/compare.vert"),
        fragment_shader: include_str!("shaders/compare1.frag"),
        id: Default::default(),
    };
    let sc_desc = Arc::new(Mutex::new(sc_desc));
    let mut scene = Scene::new(&device, &queue, &sc_desc, &Default::default());
    scene.add_object(&mut plane00);
    scene.render_scene(&view00);
    scene.remove_object(&plane00);
    scene.add_object(&mut plane01);
    scene.render_scene(&view01);
    scene.remove_object(&plane01);
    scene.add_object(&mut plane1);
    scene.render_scene(&view1);
    scene.remove_object(&plane1);
}
