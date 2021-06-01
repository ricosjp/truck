mod common;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_meshalgo::prelude::obj;
use truck_rendimpl::*;
use wgpu::*;

const PICTURE_SIZE: (u32, u32) = (256, 256);

fn bgcheck_shaders(handler: &DeviceHandler) -> PolygonShaders {
    let source = include_str!("shaders/mesh-bindgroup.wgsl");
    let module = Arc::new(
        handler
            .device()
            .create_shader_module(&ShaderModuleDescriptor {
                source: ShaderSource::Wgsl(source.into()),
                flags: ShaderFlags::VALIDATION,
                label: None,
            }),
    );
    PolygonShaders::new(
        Arc::clone(&module),
        "vs_main",
        Arc::clone(&module),
        "nontex_main",
        Arc::clone(&module),
        "tex_main",
    )
}

fn bgcheck_anti_shaders(handler: &DeviceHandler) -> PolygonShaders {
    let source = include_str!("shaders/mesh-bindgroup.wgsl");
    let module = Arc::new(
        handler
            .device()
            .create_shader_module(&ShaderModuleDescriptor {
                source: ShaderSource::Wgsl(source.into()),
                flags: ShaderFlags::VALIDATION,
                label: None,
            }),
    );
    PolygonShaders::new(
        Arc::clone(&module),
        "vs_main",
        Arc::clone(&module),
        "nontex_main_anti",
        Arc::clone(&module),
        "tex_main_anti",
    )
}

const ATTRS_OBJ: &str = "
v -1.0 2.0 -1.0\nv 1.0 2.0 -1.0\nv -1.0 2.0 1.0\nv 1.0 2.0 1.0
vt -1.0 -1.0\nvt 1.0 -1.0\nvt 1.0 1.0\nvt -1.0 1.0
vn -1.0 0.2 -1.0\nvn 1.0 0.2 -1.0\nvn -1.0 0.2 1.0\nvn 1.0 0.2 1.0
";
const TRIS_OBJ: &str = "f 1/1/1 2/2/3 3/4/2\nf 3/4/2 2/2/3 4/3/4\n";
const QUADS_OBJ: &str = "f 1/1/1 2/2/3 4/3/4 3/4/2\n";

fn test_polygons() -> [PolygonMesh; 2] {
    [
        obj::read((ATTRS_OBJ.to_string() + TRIS_OBJ).as_bytes()).unwrap(),
        obj::read((ATTRS_OBJ.to_string() + QUADS_OBJ).as_bytes()).unwrap(),
    ]
}

fn nontex_inst_desc() -> PolygonInstanceDescriptor {
    PolygonInstanceDescriptor {
        instance_state: InstanceState {
            matrix: Matrix4::from_cols(
                [1.0, 2.0, 3.0, 4.0].into(),
                [5.0, 6.0, 7.0, 8.0].into(),
                [9.0, 10.0, 11.0, 12.0].into(),
                [13.0, 14.0, 15.0, 16.0].into(),
            ),
            material: Material {
                albedo: Vector4::new(0.2, 0.4, 0.6, 1.0),
                roughness: 0.31415,
                reflectance: 0.29613,
                ambient_ratio: 0.92,
            },
            texture: None,
            backface_culling: true,
        },
    }
}

fn exec_polygon_bgtest(
    scene: &mut Scene,
    instance: &PolygonInstance,
    answer: &Vec<u8>,
    id: usize,
    out_dir: String,
) -> bool {
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    common::render_one(scene, &texture, instance);
    let buffer = common::read_texture(scene.device_handler(), &texture);
    let path = format!("{}polygon-bgtest-{}.png", out_dir, id);
    common::save_buffer(path, &buffer, PICTURE_SIZE);
    common::same_buffer(&answer, &buffer)
}

fn exec_polymesh_nontex_bind_group_test(backend: BackendBit, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = wgpu::Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor(PICTURE_SIZE)));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::nontex_answer_texture(&mut scene);
    let answer = common::read_texture(scene.device_handler(), &answer);
    let inst_desc = nontex_inst_desc();
    test_polygons()
        .iter()
        .enumerate()
        .for_each(move |(i, polygon)| {
            let instance: PolygonInstance = polygon.into_instance(
                scene.device_handler(),
                &bgcheck_shaders(scene.device_handler()),
                &inst_desc,
            );
            assert!(exec_polygon_bgtest(
                &mut scene,
                &instance,
                &answer,
                i,
                out_dir.clone()
            ));
            let instance: PolygonInstance = polygon.into_instance(
                scene.device_handler(),
                &bgcheck_anti_shaders(scene.device_handler()),
                &inst_desc,
            );
            assert!(!exec_polygon_bgtest(
                &mut scene,
                &instance,
                &answer,
                i,
                out_dir.clone()
            ));
        })
}

#[test]
fn polymesh_nontex_bind_group_test() {
    common::os_alt_exec_test(exec_polymesh_nontex_bind_group_test)
}

fn exec_polymesh_tex_bind_group_test(backend: BackendBit, out_dir: &str) {
    let out_dir = out_dir.to_string();
    std::fs::create_dir_all(&out_dir).unwrap();
    let instance = wgpu::Instance::new(backend);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor(PICTURE_SIZE)));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::random_texture(&mut scene);
    let buffer = common::read_texture(scene.device_handler(), &answer);
    let pngpath = out_dir.clone() + "random-texture.png";
    common::save_buffer(pngpath, &buffer, PICTURE_SIZE);
    let mut desc = nontex_inst_desc();
    let image_buffer =
        ImageBuffer::<Rgba<_>, _>::from_raw(PICTURE_SIZE.0, PICTURE_SIZE.1, buffer.clone())
            .unwrap();
    let attach = image2texture::image2texture(
        scene.device_handler(),
        &DynamicImage::ImageRgba8(image_buffer),
    );
    desc.instance_state.texture = Some(Arc::new(attach));
    test_polygons()
        .iter()
        .enumerate()
        .for_each(move |(i, polygon)| {
            let instance: PolygonInstance = polygon.into_instance(
                scene.device_handler(),
                &bgcheck_shaders(scene.device_handler()),
                &desc,
            );
            assert!(exec_polygon_bgtest(
                &mut scene,
                &instance,
                &buffer,
                i + 3,
                out_dir.clone(),
            ));
            let instance: PolygonInstance = polygon.into_instance(
                scene.device_handler(),
                &bgcheck_anti_shaders(scene.device_handler()),
                &desc,
            );
            assert!(!exec_polygon_bgtest(
                &mut scene,
                &instance,
                &buffer,
                i + 3,
                out_dir.clone(),
            ));
        })
}

#[test]
fn polymesh_tex_bind_group_test() { common::os_alt_exec_test(exec_polymesh_tex_bind_group_test) }
