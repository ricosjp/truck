use crate::*;

impl PolygonInstance {
    /// Clone the instance as another drawn element.
    #[inline(always)]
    pub fn clone_instance(&self) -> PolygonInstance {
        PolygonInstance {
            polygon: self.polygon.clone(),
            state: self.state.clone(),
            shaders: self.shaders.clone(),
            id: RenderID::gen(),
        }
    }
    /// Returns a reference to the instance descriptor.
    #[inline(always)]
    pub fn instance_state(&self) -> &PolygonState {
        &self.state
    }
    /// Returns the mutable reference to instance descriptor.
    #[inline(always)]
    pub fn instance_state_mut(&mut self) -> &mut PolygonState {
        &mut self.state
    }

    /// swap vertex buffers
    #[inline(always)]
    pub fn swap_vertex(&mut self, other: &mut PolygonInstance) {
        let polygon = self.polygon.clone();
        self.polygon = other.polygon.clone();
        other.polygon = polygon;
    }

    #[inline(always)]
    fn non_textured_bdl(&self, device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(device, {
            &[
                PolygonState::matrix_bgl_entry(),
                PolygonState::material_bgl_entry(),
            ]
        })
    }

    #[inline(always)]
    fn textured_bdl(&self, device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                PolygonState::matrix_bgl_entry(),
                PolygonState::material_bgl_entry(),
                PolygonState::textureview_bgl_entry(),
                PolygonState::sampler_bgl_entry(),
            ],
        )
    }

    #[inline(always)]
    fn non_textured_bg(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        bind_group_util::create_bind_group(
            device,
            layout,
            vec![
                self.state.matrix_buffer(device).binding_resource(),
                self.state.material.buffer(device).binding_resource(),
            ],
        )
    }
    #[inline(always)]
    fn textured_bg(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        let (view, sampler) = self.state.textureview_and_sampler(device);
        bind_group_util::create_bind_group(
            device,
            layout,
            vec![
                self.state.matrix_buffer(device).binding_resource(),
                self.state.material.buffer(device).binding_resource(),
                BindingResource::TextureView(&view),
                BindingResource::Sampler(&sampler),
            ],
        )
    }
}

impl Rendered for PolygonInstance {
    impl_render_id!(id);

    #[inline(always)]
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let polygon = self.polygon.clone();
        (polygon.0, Some(polygon.1))
    }
    #[inline(always)]
    fn bind_group_layout(&self, device_handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(match self.state.texture.is_some() {
            true => self.textured_bdl(device_handler.device()),
            false => self.non_textured_bdl(device_handler.device()),
        })
    }
    #[inline(always)]
    fn bind_group(
        &self,
        device_handler: &DeviceHandler,
        layout: &BindGroupLayout,
    ) -> Arc<BindGroup> {
        Arc::new(match self.state.texture.is_some() {
            true => self.textured_bg(device_handler.device(), layout),
            false => self.non_textured_bg(&device_handler.device(), layout),
        })
    }
    #[inline(always)]
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
        let device = device_handler.device();
        let config = device_handler.config();
        let (fragment_module, fragment_entry) = match self.state.texture.is_some() {
            true => (
                &self.shaders.tex_fragment_module,
                self.shaders.tex_fragment_entry,
            ),
            false => (&self.shaders.fragment_module, self.shaders.fragment_entry),
        };
        let cull_mode = match self.state.backface_culling {
            true => Some(wgpu::Face::Back),
            false => None,
        };
        let blend = match self.state.material.alpha_blend {
            true => Some(BlendState::ALPHA_BLENDING),
            false => Some(BlendState::REPLACE),
        };
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(layout),
            vertex: VertexState {
                module: &self.shaders.vertex_module,
                entry_point: self.shaders.vertex_entry,
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<AttrVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 3 * 4,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 2 * 4 + 3 * 4,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: fragment_module,
                entry_point: fragment_entry,
                targets: &[ColorTargetState {
                    format: config.format,
                    blend,
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                cull_mode,
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
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
                alpha_to_coverage_enabled: sample_count > 1,
            },
            label: None,
        });
        Arc::new(pipeline)
    }
}
