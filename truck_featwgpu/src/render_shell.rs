use super::*;
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
        let surface = face.oriented_surface();
        let control_points: Vec<[f32; 4]> = surface
            .control_points()
            .iter()
            .flatten()
            .map(|v| v.cast().unwrap().into())
            .collect();
        let knot_vecs: (Vec<f32>, Vec<f32>) = (
            surface.uknot_vec().iter().map(move |k| *k as f32).collect(),
            surface.vknot_vec().iter().map(move |k| *k as f32).collect(),
        );
        let surface_division = surface.rational_parameter_division(tol);
        let mut boundary = Vec::<f32>::new();
        for edge in face.boundary_iters().into_iter().flatten() {
            let curve = edge.oriented_curve();
            let division = curve.rational_parameter_division(tol);
            let mut hint = presearch(&surface, curve.subs(division[0]).rational_projection());
            let mut this_boundary = Vec::<(f32, f32)>::new();
            for t in division {
                let pt = curve.subs(t).rational_projection();
                hint = match surface.search_rational_parameter(pt, hint) {
                    Some(got) => got,
                    None => return None,
                };
                this_boundary.push((hint.0 as f32, hint.1 as f32));
            }
            let mut iter = this_boundary.into_iter().peekable();
            while let Some(tuple) = iter.next() {
                boundary.push(tuple.0);
                boundary.push(tuple.1);
                if let Some(next_tuple) = iter.peek() {
                    boundary.push(next_tuple.0);
                    boundary.push(next_tuple.1);
                }
            }
        }
        let surface_info = SurfaceInfo {
            ctrl_row_size: surface.control_points().len() as u32,
            ctrl_column_size: surface.control_points()[0].len() as u32,
            uknots_size: surface.uknot_vec().len() as u32,
            vknots_size: surface.vknot_vec().len() as u32,
            param_row_size: surface_division.0.len() as u32,
            param_column_size: surface_division.1.len() as u32,
            boundary_length: boundary.len() as u32,
        };
        Some(RenderFace {
            control_points: Arc::new(BufferHandler::from_slice(
                &control_points,
                device,
                BufferUsage::STORAGE,
            )),
            uknot_vec: Arc::new(BufferHandler::from_slice(
                &knot_vecs.0,
                device,
                BufferUsage::STORAGE,
            )),
            vknot_vec: Arc::new(BufferHandler::from_slice(
                &knot_vecs.1,
                device,
                BufferUsage::STORAGE,
            )),
            udivision: Arc::new(BufferHandler::from_slice(
                &surface_division.0,
                device,
                BufferUsage::STORAGE,
            )),
            vdivision: Arc::new(BufferHandler::from_slice(
                &surface_division.1,
                device,
                BufferUsage::STORAGE,
            )),
            boundary: Arc::new(BufferHandler::from_slice(
                &boundary,
                device,
                BufferUsage::STORAGE,
            )),
            surface_info: Arc::new(BufferHandler::from_slice(
                &[surface_info],
                device,
                BufferUsage::UNIFORM,
            )),
        })
    }
}

impl Rendered for RenderFace {
    fn vertex_buffer(&self, scene: &Scene) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let udiv_size = self.udivision.size / 4;
        let vdiv_size = self.vdivision.size / 4;
        let slice: Vec<u32> = (1..udiv_size)
            .zip(1..vdiv_size)
            .flat_map(|(i, j)|
                vec![
                    (vdiv_size * (i - 1) + j - 1) as u32,
                    (vdiv_size * i + j - 1) as u32,
                    (vdiv_size * (i - 1) + j) as u32,
                    (vdiv_size * (i - 1) + j) as u32,
                    (vdiv_size * i + j - 1) as u32,
                    (vdiv_size * i + j) as u32,
                ]
            ).collect();
        let buffer = BufferHandler::from_slice(&slice, scene.device(), BufferUsage::VERTEX);
        (Arc::new(buffer), None)
    }
    fn bind_group_layout(&self, scene: &Scene) -> Arc<BindGroupLayout> {
        let layout = scene.device().create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                // control_points
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                // uknot_vec
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                // vknot_vec
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                // udivision
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                // vdivision
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                // boundary
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                // surface info
                BindGroupLayoutEntry {
                    binding: 6,
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
                self.control_points.binding_resource(),
                self.uknot_vec.binding_resource(),
                self.vknot_vec.binding_resource(),
                self.udivision.binding_resource(),
                self.vdivision.binding_resource(),
                self.boundary.binding_resource(),
                self.surface_info.binding_resource(),
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
                            format: VertexFormat::Uint,
                            offset: 0,
                            shader_location: 0,
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
