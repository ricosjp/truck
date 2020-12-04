mod common;
use truck_platform::*;
use truck_rendimpl::*;
use glsl_to_spirv::ShaderType;
use std::sync::Arc;
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
        let mut fragment_shader = String::new();
        fragment_shader += include_str!("../src/shaders/microfacet_module.frag");
        fragment_shader += include_str!("shaders/check-mf-module.frag");
        let fragment_spirv = common::compile_shader(&fragment_shader, ShaderType::Fragment);
        let fragment_module = wgpu::util::make_spirv(&fragment_spirv);
        self.polygon
            .pipeline_with_shader(vertex_module, fragment_module, device_handler, layout)
    }
}

