mod common;
use common::{PICTURE_HEIGHT, PICTURE_WIDTH};
use glsl_to_spirv::ShaderType;
use image::{ColorType, DynamicImage, ImageBuffer, Rgba};
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

struct BGCheckPolygonInstance<'a> {
    polygon: PolygonInstance,
    fragment_shader: &'a str,
}

impl<'a> Rendered for BGCheckPolygonInstance<'a> {
    derive_get_set_id!(polygon);
    derive_vertex_buffer!(polygon);
    derive_bind_group_layout!(polygon);
    derive_bind_group!(polygon);
    #[inline(always)]
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline> {
        let vertex_shader = include_str!("shaders/mesh-bindgroup.vert");
        let vertex_spirv = common::compile_shader(vertex_shader, ShaderType::Vertex);
        let vertex_module = wgpu::util::make_spirv(&vertex_spirv);
        let fragment_spirv = common::compile_shader(self.fragment_shader, ShaderType::Fragment);
        let fragment_module = wgpu::util::make_spirv(&fragment_spirv);
        self.polygon
            .pipeline_with_shader(vertex_module, fragment_module, device_handler, layout)
    }
}

fn test_polygons() -> [PolygonMesh; 3] {
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
        [[0, 0, 0], [1, 1, 1], [2, 2, 2]],
        [[2, 2, 2], [1, 1, 1], [3, 3, 3]],
    ];
    let quad_faces = vec![[[0, 0, 0], [1, 1, 1], [3, 3, 3], [2, 2, 2]]];
    let other_faces = vec![vec![[0, 0, 0], [1, 1, 1], [3, 3, 3], [2, 2, 2]]];

    [
        PolygonMesh {
            positions: positions.clone(),
            uv_coords: uv_coords.clone(),
            normals: normals.clone(),
            tri_faces,
            ..Default::default()
        },
        PolygonMesh {
            positions: positions.clone(),
            uv_coords: uv_coords.clone(),
            normals: normals.clone(),
            quad_faces,
            ..Default::default()
        },
        PolygonMesh {
            positions,
            uv_coords,
            normals,
            other_faces,
            ..Default::default()
        },
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
    answer: &Texture,
) -> bool {
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    let mut bgc_instance = BGCheckPolygonInstance {
        polygon: instance.clone(),
        fragment_shader: shader,
    };
    common::render_one(scene, &texture, &mut bgc_instance);
    common::same_texture(scene.device_handler(), &answer, &texture)
}

#[test]
fn polymesh_nontex_bind_group_test() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor()));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::nontex_answer_texture(&mut scene);
    let inst_desc = nontex_inst_desc();
    test_polygons().iter().for_each(move |polygon| {
        let instance = scene.create_instance(polygon, &inst_desc);
        let shader = include_str!("shaders/mesh-nontex-bindgroup.frag");
        assert!(exec_polygon_bgtest(&mut scene, &instance, shader, &answer));
        let shader = include_str!("shaders/anti-mesh-nontex-bindgroup.frag");
        assert!(!exec_polygon_bgtest(&mut scene, &instance, shader, &answer));
    })
}

#[test]
fn polymesh_tex_bind_group_test() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor()));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::random_texture(&mut scene);
    let buffer = common::read_texture(scene.device_handler(), &answer);
    image::save_buffer(
        "random-texture.png",
        &buffer,
        PICTURE_WIDTH,
        PICTURE_HEIGHT,
        ColorType::Rgba8,
    )
    .unwrap();
    let mut inst_desc = nontex_inst_desc();
    let image_buffer =
        ImageBuffer::<Rgba<_>, _>::from_raw(PICTURE_WIDTH, PICTURE_HEIGHT, buffer).unwrap();
    inst_desc.texture = Some(Arc::new(DynamicImage::ImageRgba8(image_buffer)));
    test_polygons().iter().for_each(move |polygon| {
        let instance = scene.create_instance(polygon, &inst_desc);
        let shader = include_str!("shaders/mesh-tex-bindgroup.frag");
        assert!(exec_polygon_bgtest(&mut scene, &instance, shader, &answer));
        let shader = include_str!("shaders/anti-mesh-tex-bindgroup.frag");
        assert!(!exec_polygon_bgtest(&mut scene, &instance, shader, &answer));
    })
}
