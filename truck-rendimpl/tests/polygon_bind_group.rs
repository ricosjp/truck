mod common;
use glsl_to_spirv::ShaderType;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

const PICTURE_SIZE: (u32, u32) = (256, 256);

struct BGCheckPolygonInstance<'a> {
    polygon: PolygonInstance,
    fragment_shader: &'a str,
}

impl<'a> Rendered for BGCheckPolygonInstance<'a> {
    derive_render_id!(polygon);
    derive_vertex_buffer!(polygon);
    derive_bind_group_layout!(polygon);
    derive_bind_group!(polygon);
    #[inline(always)]
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
        let vertex_shader = include_str!("shaders/mesh-bindgroup.vert");
        let vertex_spirv = common::compile_shader(vertex_shader, ShaderType::Vertex);
        let vertex_module = wgpu::util::make_spirv(&vertex_spirv);
        let fragment_spirv = common::compile_shader(self.fragment_shader, ShaderType::Fragment);
        let fragment_module = wgpu::util::make_spirv(&fragment_spirv);
        self.polygon.pipeline_with_shader(
            vertex_module,
            fragment_module,
            device_handler,
            layout,
            sample_count,
        )
    }
}

fn test_polygons() -> [PolygonMesh; 2] {
    let positions = vec![
        Point3::new(-1.0, 2.0, -1.0),
        Point3::new(1.0, 2.0, -1.0),
        Point3::new(-1.0, 2.0, 1.0),
        Point3::new(1.0, 2.0, 1.0),
    ];
    let uv_coords = vec![
        Vector2::new(-1.0, -1.0),
        Vector2::new(1.0, -1.0),
        Vector2::new(-1.0, 1.0),
        Vector2::new(1.0, 1.0),
    ];
    let normals = vec![
        Vector3::new(-1.0, 0.2, -1.0),
        Vector3::new(-1.0, 0.2, 1.0),
        Vector3::new(1.0, 0.2, -1.0),
        Vector3::new(1.0, 0.2, 1.0),
    ];
    let tri_faces = vec![
        [[0, 0, 0].into(), [1, 1, 1].into(), [2, 2, 2].into()],
        [[2, 2, 2].into(), [1, 1, 1].into(), [3, 3, 3].into()],
    ];
    let quad_faces = vec![[
        [0, 0, 0].into(),
        [1, 1, 1].into(),
        [3, 3, 3].into(),
        [2, 2, 2].into(),
    ]];
    [
        PolygonMesh::new(
            positions.clone(),
            uv_coords.clone(),
            normals.clone(),
            Faces::from_tri_and_quad_faces(tri_faces, Vec::new()),
        ),
        PolygonMesh::new(
            positions.clone(),
            uv_coords.clone(),
            normals.clone(),
            Faces::from_tri_and_quad_faces(Vec::new(), quad_faces),
        ),
    ]
}

fn nontex_inst_desc() -> InstanceDescriptor {
    InstanceDescriptor {
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
    }
}

fn exec_polygon_bgtest(
    scene: &mut Scene,
    instance: &PolygonInstance,
    shader: &str,
    answer: &Vec<u8>,
    id: usize,
) -> bool {
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    let mut bgc_instance = BGCheckPolygonInstance {
        polygon: instance.clone(),
        fragment_shader: shader,
    };
    common::render_one(scene, &texture, &mut bgc_instance);
    let buffer = common::read_texture(scene.device_handler(), &texture);
    let path = format!("output/polygon-bgtest-{}.png", id);
    save_buffer(path, &buffer);
    common::same_buffer(&answer, &buffer)
}

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
fn polymesh_nontex_bind_group_test() {
    std::fs::create_dir_all("output").unwrap();
    let instance = Instance::new(BackendBit::PRIMARY);
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
            let instance = scene.create_instance(polygon, &inst_desc);
            let shader = include_str!("shaders/mesh-nontex-bindgroup.frag");
            assert!(exec_polygon_bgtest(
                &mut scene, &instance, shader, &answer, i
            ));
            let shader = include_str!("shaders/anti-mesh-nontex-bindgroup.frag");
            assert!(!exec_polygon_bgtest(
                &mut scene, &instance, shader, &answer, i
            ));
        })
}

#[test]
fn polymesh_tex_bind_group_test() {
    std::fs::create_dir_all("output").unwrap();
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor(PICTURE_SIZE)));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::random_texture(&mut scene);
    let buffer = common::read_texture(scene.device_handler(), &answer);
    save_buffer("output/random-texture.png", &buffer);
    let mut inst_desc = nontex_inst_desc();
    let image_buffer =
        ImageBuffer::<Rgba<_>, _>::from_raw(PICTURE_SIZE.0, PICTURE_SIZE.1, buffer.clone())
            .unwrap();
    inst_desc.texture = Some(Arc::new(DynamicImage::ImageRgba8(image_buffer)));
    test_polygons()
        .iter()
        .enumerate()
        .for_each(move |(i, polygon)| {
            let instance = scene.create_instance(polygon, &inst_desc);
            let shader = include_str!("shaders/mesh-tex-bindgroup.frag");
            assert!(exec_polygon_bgtest(
                &mut scene,
                &instance,
                shader,
                &buffer,
                i + 3
            ));
            let shader = include_str!("shaders/anti-mesh-tex-bindgroup.frag");
            assert!(!exec_polygon_bgtest(
                &mut scene,
                &instance,
                shader,
                &buffer,
                i + 3
            ));
        })
}
