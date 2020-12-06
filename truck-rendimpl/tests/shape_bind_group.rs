mod common;
use common::{PICTURE_HEIGHT, PICTURE_WIDTH};
use glsl_to_spirv::ShaderType;
use image::{ColorType, DynamicImage, ImageBuffer, Rgba};
use shaperend::RenderFace;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

struct BGCheckShapeInstance<'a> {
    shape: ShapeInstance,
    fragment_shader: &'a str,
}

struct BGCheckRenderFace<'a, 'b> {
    face: RenderFace<'a>,
    fragment_shader: &'b str,
}

impl<'a> BGCheckShapeInstance<'a> {
    #[inline(always)]
    pub fn render_faces(&mut self) -> Vec<BGCheckRenderFace> {
        let fragment_shader = self.fragment_shader;
        self.shape
            .render_faces()
            .into_iter()
            .map(|face| BGCheckRenderFace {
                face,
                fragment_shader,
            })
            .collect()
    }
}

impl<'a, 'b> Rendered for BGCheckRenderFace<'a, 'b> {
    derive_get_set_id!(face);
    derive_vertex_buffer!(face);
    derive_bind_group_layout!(face);
    derive_bind_group!(face);
    #[inline(always)]
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline>
    {
        let vertex_shader = include_str!("shaders/shape-bindgroup.vert");
        let vertex_spirv = common::compile_shader(vertex_shader, ShaderType::Vertex);
        let vertex_module = wgpu::util::make_spirv(&vertex_spirv);
        let fragment_spirv = common::compile_shader(self.fragment_shader, ShaderType::Fragment);
        let fragment_module = wgpu::util::make_spirv(&fragment_spirv);
        self.face
            .pipeline_with_shader(vertex_module, fragment_module, device_handler, layout)
    }
}

fn test_shape() -> Shell {
    let v = builder::vertex(Point3::new(-1.0, -1.0, 0.0));
    let e = builder::tsweep(&v, Vector3::unit_x());
    let face0 = builder::tsweep(&e, Vector3::unit_y() * 2.0);
    let edge = face0.boundaries()[0][1].clone();
    let face1 = builder::tsweep(&edge, Vector3::new(1.0, 0.0, 1.0)).inverse();
    vec![face0, face1].into()
}

fn exec_shape_bgtest(
    scene: &mut Scene,
    instance: &ShapeInstance,
    shader: &str,
    answer: &Texture,
) -> bool
{
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    let mut bgc_instance = BGCheckShapeInstance {
        shape: instance.clone(),
        fragment_shader: shader,
    };
    common::render_ones(scene, &texture, &mut bgc_instance.render_faces());
    common::same_texture(scene.device_handler(), &answer, &texture)
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

#[test]
fn shape_nontex_bind_group_test() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor()));
    let handler = DeviceHandler::new(device, queue, sc_desc);
    let mut scene = Scene::new(handler, &Default::default());
    let answer = common::nontex_answer_texture(&mut scene);
    let inst_desc = nontex_inst_desc();
    let shell = test_shape();
    let instance = scene.create_instance(&shell, &inst_desc);
    let shader = include_str!("shaders/shape-nontex-bindgroup.frag");
    assert!(exec_shape_bgtest(&mut scene, &instance, shader, &answer));
    let shader = include_str!("shaders/anti-shape-nontex-bindgroup.frag");
    assert!(!exec_shape_bgtest(&mut scene, &instance, shader, &answer));
}

#[test]
fn shape_tex_bind_group_test() {
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
    let shell = test_shape();
    let instance = scene.create_instance(&shell, &inst_desc);
    let shader = include_str!("shaders/shape-tex-bindgroup.frag");
    assert!(exec_shape_bgtest(&mut scene, &instance, shader, &answer));
    let shader = include_str!("shaders/anti-shape-tex-bindgroup.frag");
    assert!(!exec_shape_bgtest(&mut scene, &instance, shader, &answer));
}
