mod common;
use common::Plane;
use glsl_to_spirv::ShaderType;
use std::sync::{Arc, Mutex};
use truck_platform::*;
use truck_rendimpl::*;
use wgpu::*;

struct MFCheckPolygonInstance {
    polygon: PolygonInstance,
}

impl Rendered for MFCheckPolygonInstance {
    derive_get_set_id!(polygon);
    derive_vertex_buffer!(polygon);
    derive_bind_group_layout!(polygon);
    derive_bind_group!(polygon);
    #[inline(always)]
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline>
    {
        let vertex_shader = include_str!("shaders/mesh-bindgroup.vert");
        let vertex_spirv = common::compile_shader(vertex_shader, ShaderType::Vertex);
        let vertex_module = wgpu::util::make_spirv(&vertex_spirv);
        let mut fragment_shader = "#version 450\n\n".to_string();
        fragment_shader += include_str!("../src/shaders/microfacet_module.frag");
        fragment_shader += include_str!("shaders/check-mf-module.frag");
        let fragment_spirv = common::compile_shader(&fragment_shader, ShaderType::Fragment);
        let fragment_module = wgpu::util::make_spirv(&fragment_spirv);
        self.polygon
            .pipeline_with_shader(vertex_module, fragment_module, device_handler, layout)
    }
}

#[test]
fn microfacet_module_test() {
    let instance = Instance::new(BackendBit::PRIMARY);
    let (device, queue) = common::init_device(&instance);
    let sc_desc = Arc::new(Mutex::new(common::swap_chain_descriptor()));
    let mut scene = Scene::new(&device, &queue, &sc_desc, &Default::default());
    let answer = common::nontex_answer_texture(&mut scene);
    let sc_desc = scene.sc_desc();
    let tex_desc = common::texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    
    let mut fragment_shader = "#version 450\n\n".to_string();
    fragment_shader += include_str!("../src/shaders/microfacet_module.frag");
    fragment_shader += include_str!("shaders/check-mf-module.frag");
    let mut plane = Plane {
        vertex_shader: include_str!("shaders/plane.vert"),
        fragment_shader: &fragment_shader,
        id: Default::default(),
    };
    common::render_one(&mut scene, &texture, &mut plane);
    assert!(common::same_texture(scene.device_handler(), &answer, &texture));
}
