use super::*;
use polymesh::StructuredMesh;
use integral::{EdgeEx, FaceEx};

fn presearch(surface: &BSplineSurface, point: Vector3) -> (f64, f64) {
    const N: usize = 50;
    let mut res = (0.0, 0.0);
    let mut min = std::f64::INFINITY;
    for i in 0..=N {
        for j in 0..=N {
            let p = i as f64 / N as f64;
            let q = j as f64 / N as f64;
            let u = surface.uknot_vec()[0] + p * surface.uknot_vec().range_length();
            let v = surface.vknot_vec()[0] + q * surface.vknot_vec().range_length();
            let dist = surface.subs(u, v).rational_projection().distance2(point);
            if dist < min {
                min = dist;
                res = (u, v);
            }
        }
    }
    res
}

impl RenderFace {
    pub fn new(face: &Face, tol: f64, device: &Device) -> Option<RenderFace> {
        let surface = NURBSSurface::new(face.oriented_surface());
        let polymesh: ExpandedPolygon = StructuredMesh::from_surface(&surface, tol).into();
        let buffers = polymesh.buffers(device);
        let mut boundary = Vec::<[f32; 4]>::new();
        for edge in face.boundary_iters().into_iter().flatten() {
            let curve = edge.oriented_curve();
            let division = curve.parameter_division(tol * 0.1);
            let mut hint = presearch(&surface, curve.subs(division[0]).rational_projection());
            let mut this_boundary = Vec::new();
            for t in division {
                let pt = curve.subs(t).rational_projection();
                hint = match surface.search_parameter(pt, hint) {
                    Some(got) => got,
                    None => return None,
                };
                this_boundary.push([hint.0 as f32, hint.1 as f32]);
            }
            for window in this_boundary.as_slice().windows(2) {
                boundary.push([window[0][0], window[0][1], window[1][0], window[1][1]]);
            }
        }
        Some(RenderFace {
            polygon: (Arc::new(buffers.0), Arc::new(buffers.1)),
            boundary: Arc::new(BufferHandler::from_slice(
                &boundary,
                device,
                BufferUsage::STORAGE,
            )),
            boundary_length: Arc::new(BufferHandler::from_slice(
                &[boundary.len() as u32],
                device,
                BufferUsage::UNIFORM,
            )),
        })
    }
    
    pub fn from_shell(shell: &Shell, tol: f64, device: &Device) -> Vec<Option<RenderFace>> {
        shell.face_iter().map(|face| Self::new(face, tol, device)).collect()
    }
}

impl Rendered for RenderFace {
    fn vertex_buffer(&self, _: &Scene) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let (a, b) = self.polygon.clone();
        (a, Some(b))
    }
    fn bind_group_layout(&self, scene: &Scene) -> Arc<BindGroupLayout> {
        let layout = scene.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // boundary
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                // boundary length
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        });
        Arc::new(layout)
    }
    fn bind_group(&self, scene: &Scene, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let bind_group = crate::create_bind_group(
            scene.device(),
            layout,
            vec![
                self.boundary.binding_resource(),
                self.boundary_length.binding_resource(),
            ]);
        Arc::new(bind_group)
    }
    fn pipeline(&self, scene: &Scene, layout: &PipelineLayout) -> Arc<RenderPipeline> {
        let vertex_module = scene.device().create_shader_module(include_spirv!("shaders/face.vert.spv"));
        let fragment_module = scene.device().create_shader_module(include_spirv!("shaders/face.frag.spv"));
        let pipeline = scene.device().create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(layout),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: scene.sc_desc().format,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilStateDescriptor {
                    front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint32,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: std::mem::size_of::<AttrVertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float2,
                            offset: 3 * 4,
                            shader_location: 1,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 2 * 4 + 3 * 4,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            label: None,
        });
        Arc::new(pipeline)
    }
}
