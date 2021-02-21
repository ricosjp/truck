use crate::*;

impl Default for WireFrameState {
    #[inline(always)]
    fn default() -> WireFrameState {
        WireFrameState {
            matrix: Matrix4::identity(),
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Default for WireFrameInstanceDescriptor {
    #[inline(always)]
    fn default() -> WireFrameInstanceDescriptor {
        WireFrameInstanceDescriptor {
            wireframe_state: WireFrameState::default(),
            polyline_precision: 0.005,
        }
    }
}

impl Rendered for WireFrameInstance {
    impl_render_id!(id);
    fn vertex_buffer(
        &self,
        _: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        (self.vertices.clone(), Some(self.strips.clone()))
    }
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(bind_group_util::create_bind_group_layout(
            handler.device(),
            &[
                // matrix
                PreBindGroupLayoutEntry {
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // color
                PreBindGroupLayoutEntry {
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        ))
    }
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let device = handler.device();
        let matrix_data: [[f32; 4]; 4] = self.state.matrix.cast::<f32>().unwrap().into();
        let matrix_buffer = BufferHandler::from_slice(&matrix_data, device, BufferUsage::UNIFORM);
        let color_data: [f32; 4] = self.state.color.cast::<f32>().unwrap().into();
        let color_buffer = BufferHandler::from_slice(&color_data, device, BufferUsage::UNIFORM);
        Arc::new(bind_group_util::create_bind_group(
            device,
            layout,
            vec![
                matrix_buffer.binding_resource(),
                color_buffer.binding_resource(),
            ],
        ))
    }
    fn pipeline(
        &self,
        handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
        let (device, sc_desc) = (handler.device(), handler.sc_desc());
        let vertex_module = device.create_shader_module(include_spirv!("shaders/line.vert.spv"));
        let fragment_module = device.create_shader_module(include_spirv!("shaders/line.frag.spv"));
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
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
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: PrimitiveTopology::LineList,
            color_states: &[ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilStateDescriptor {
                    front: StencilStateFaceDescriptor::IGNORE,
                    back: StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint32,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 0,
                            shader_location: 0,
                        },
                    ],
                }],
            },
            sample_count,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            label: None,
        });
        Arc::new(pipeline)
    }
}
