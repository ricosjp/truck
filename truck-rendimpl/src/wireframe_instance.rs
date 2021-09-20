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

impl WireFrameInstance {
    /// Clone the instance as another drawn element.
    #[inline(always)]
    pub fn clone_instance(&self) -> Self {
        Self {
            vertices: Arc::clone(&self.vertices),
            strips: Arc::clone(&self.strips),
            state: self.state.clone(),
            shaders: self.shaders.clone(),
            id: RenderID::gen(),
        }
    }
    /// Returns the wireframe state
    #[inline(always)]
    pub fn instance_state(&self) -> &WireFrameState {
        &self.state
    }
    /// Returns the mutable reference to wireframe state
    #[inline(always)]
    pub fn instance_state_mut(&mut self) -> &mut WireFrameState {
        &mut self.state
    }
}

impl Instance for WireFrameInstance {
    type Shaders = WireShaders;
    fn standard_shaders(creator: &InstanceCreator) -> WireShaders {
        creator.wire_shaders.clone()
    }
}

impl Rendered for WireFrameInstance {
    impl_render_id!(id);
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        (self.vertices.clone(), Some(self.strips.clone()))
    }
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(bind_group_util::create_bind_group_layout(
            handler.device(),
            &[
                // matrix
                PreBindGroupLayoutEntry {
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // color
                PreBindGroupLayoutEntry {
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
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
        let matrix_buffer = BufferHandler::from_slice(&matrix_data, device, BufferUsages::UNIFORM);
        let color_data: [f32; 4] = self.state.color.cast::<f32>().unwrap().into();
        let color_buffer = BufferHandler::from_slice(&color_data, device, BufferUsages::UNIFORM);
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
        let (device, config) = (handler.device(), handler.config());
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(layout),
            vertex: VertexState {
                module: &self.shaders.vertex_module,
                entry_point: self.shaders.vertex_entry,
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(FragmentState {
                module: &self.shaders.fragment_module,
                entry_point: self.shaders.fragment_entry,
                targets: &[ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: None,
        });
        Arc::new(pipeline)
    }
}

impl IntoInstance<WireFrameInstance> for Vec<(Point3, Point3)> {
    type State = WireFrameState;
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        state: &WireFrameState,
    ) -> WireFrameInstance {
        let device = handler.device();
        let positions: Vec<[f32; 3]> = self
            .iter()
            .flat_map(|p| vec![p.0, p.1])
            .map(|p| p.cast().unwrap().into())
            .collect();
        let strips: Vec<u32> = (0..2 * self.len()).map(|i| i as u32).collect();
        let vb = BufferHandler::from_slice(&positions, device, BufferUsages::VERTEX);
        let ib = BufferHandler::from_slice(&strips, device, BufferUsages::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vb),
            strips: Arc::new(ib),
            state: state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}
