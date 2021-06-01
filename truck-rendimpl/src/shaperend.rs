use crate::*;
use truck_meshalgo::tessellation::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
    pub boundary_range: [u32; 2],
}

impl Default for ShapeInstanceDescriptor {
    #[inline(always)]
    fn default() -> Self {
        ShapeInstanceDescriptor {
            instance_state: Default::default(),
            mesh_precision: 0.005,
        }
    }
}

impl<Shape: MeshableShape> TryIntoInstance<PolygonInstance> for Shape {
    type Descriptor = ShapeInstanceDescriptor;
    fn try_into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> Option<PolygonInstance> {
        let polygon = self.triangulation(desc.mesh_precision)?;
        Some(polygon.into_instance(
            handler,
            shaders,
            &PolygonInstanceDescriptor {
                instance_state: desc.instance_state.clone(),
            },
        ))
    }
}

impl IntoInstance<PolygonInstance> for Shell {
    type Descriptor = ShapeInstanceDescriptor;
    /// Creates `ShapeInstance` from `Shell`.
    /// # Panics
    /// Panic occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    #[inline(always)]
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> PolygonInstance {
        self.try_into_instance(handler, shaders, desc)
            .expect("failed to create instance")
    }
}

impl IntoInstance<PolygonInstance> for Solid {
    type Descriptor = ShapeInstanceDescriptor;
    /// Creates `ShapeInstance` from `Shell`.
    /// # Panics
    /// Panic occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    #[inline(always)]
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> PolygonInstance {
        self.try_into_instance(handler, shaders, desc)
            .expect("failed to create instance")
    }
}

impl IntoInstance<WireFrameInstance> for Shell {
    type Descriptor = ShapeWireFrameInstanceDescriptor;
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        desc: &ShapeWireFrameInstanceDescriptor,
    ) -> WireFrameInstance {
        let mut lengths = Vec::new();
        let points: Vec<[f32; 3]> = self
            .face_iter()
            .flat_map(|face| face.boundary_iters())
            .flatten()
            .flat_map(|edge| {
                let curve = edge.oriented_curve();
                let division =
                    curve.parameter_division(curve.parameter_range(), desc.polyline_precision);
                lengths.push(division.len() as u32);
                division
                    .into_iter()
                    .map(move |t| curve.subs(t).cast().unwrap().into())
            })
            .collect();
        let mut strips = Vec::<u32>::new();
        let mut counter = 0_u32;
        for len in lengths {
            for i in 1..len {
                strips.push(counter + i - 1);
                strips.push(counter + i);
            }
            counter += len;
        }
        let vertices = BufferHandler::from_slice(&points, handler.device(), BufferUsage::VERTEX);
        let strips = BufferHandler::from_slice(&strips, handler.device(), BufferUsage::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vertices),
            strips: Arc::new(strips),
            state: desc.wireframe_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl IntoInstance<WireFrameInstance> for Solid {
    type Descriptor = ShapeWireFrameInstanceDescriptor;
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        desc: &ShapeWireFrameInstanceDescriptor,
    ) -> WireFrameInstance {
        let mut lengths = Vec::new();
        let points: Vec<[f32; 3]> = self
            .boundaries()
            .iter()
            .flatten()
            .flat_map(|face| face.boundary_iters())
            .flatten()
            .flat_map(|edge| {
                let curve = edge.oriented_curve();
                let division =
                    curve.parameter_division(curve.parameter_range(), desc.polyline_precision);
                lengths.push(division.len() as u32);
                division
                    .into_iter()
                    .map(move |t| curve.subs(t).cast().unwrap().into())
            })
            .collect();
        let mut strips = Vec::<u32>::new();
        let mut counter = 0_u32;
        for len in lengths {
            for i in 1..len {
                strips.push(counter + i - 1);
                strips.push(counter + i);
            }
            counter += len;
        }
        let vertices = BufferHandler::from_slice(&points, handler.device(), BufferUsage::VERTEX);
        let strips = BufferHandler::from_slice(&strips, handler.device(), BufferUsage::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vertices),
            strips: Arc::new(strips),
            state: desc.wireframe_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl ShapeInstance {
    #[inline(always)]
    fn boundary_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn non_textured_bgl(device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                Self::boundary_bgl_entry(),
                InstanceState::matrix_bgl_entry(),
                InstanceState::material_bgl_entry(),
            ],
        )
    }

    #[inline(always)]
    fn textured_bgl(device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                Self::boundary_bgl_entry(),
                InstanceState::matrix_bgl_entry(),
                InstanceState::material_bgl_entry(),
                InstanceState::textureview_bgl_entry(),
                InstanceState::sampler_bgl_entry(),
            ],
        )
    }

    #[inline(always)]
    fn non_textured_bind_group(
        &self,
        handler: &DeviceHandler,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        bind_group_util::create_bind_group(
            handler.device(),
            layout,
            vec![
                self.boundary.binding_resource(),
                self.state
                    .matrix_buffer(handler.device())
                    .binding_resource(),
                self.state
                    .material_buffer(handler.device())
                    .binding_resource(),
            ],
        )
    }
    #[inline(always)]
    fn textured_bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> BindGroup {
        let (view, sampler) = self.state.textureview_and_sampler(handler.device());
        bind_group_util::create_bind_group(
            handler.device(),
            layout,
            vec![
                self.boundary.binding_resource(),
                self.state
                    .matrix_buffer(handler.device())
                    .binding_resource(),
                self.state
                    .material_buffer(handler.device())
                    .binding_resource(),
                BindingResource::TextureView(&view),
                BindingResource::Sampler(&sampler),
            ],
        )
    }
}

impl Instance for ShapeInstance {
    type Shaders = ShapeShaders;
    fn standard_shaders(creator: &InstanceCreator) -> ShapeShaders { creator.shape_shaders.clone() }
}

impl Rendered for ShapeInstance {
    impl_render_id!(id);

    #[inline(always)]
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let (vb, ib) = self.polygon.clone();
        (vb, Some(ib))
    }
    #[inline(always)]
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(match self.state.texture.is_some() {
            true => Self::textured_bgl(handler.device()),
            false => Self::non_textured_bgl(handler.device()),
        })
    }
    #[inline(always)]
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let bind_group = match self.state.texture.is_some() {
            true => self.textured_bind_group(handler, layout),
            false => self.non_textured_bind_group(handler, layout),
        };
        Arc::new(bind_group)
    }
    #[inline(always)]
    fn pipeline(
        &self,
        handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
        let (fragment_shader, fragment_entry) = match self.state.texture.is_some() {
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
        Arc::new(
            handler
                .device()
                .create_render_pipeline(&RenderPipelineDescriptor {
                    layout: Some(layout),
                    vertex: VertexState {
                        module: &self.shaders.vertex_module,
                        entry_point: self.shaders.vertex_entry,
                        buffers: &[VertexBufferLayout {
                            array_stride: std::mem::size_of::<AttrVertex>() as BufferAddress,
                            step_mode: InputStepMode::Vertex,
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
                                VertexAttribute {
                                    format: VertexFormat::Uint32x2,
                                    offset: 3 * 4 + 2 * 4 + 3 * 4,
                                    shader_location: 3,
                                },
                            ],
                        }],
                    },
                    fragment: Some(FragmentState {
                        module: fragment_shader,
                        entry_point: fragment_entry,
                        targets: &[ColorTargetState {
                            format: handler.sc_desc().format,
                            blend: Some(BlendState::REPLACE),
                            write_mask: ColorWrite::ALL,
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
                        alpha_to_coverage_enabled: false,
                    },
                    label: None,
                }),
        )
    }
}

impl ShapeInstance {
    /// Clone the instance as another drawn element.
    #[inline(always)]
    pub fn clone_instance(&self) -> Self {
        ShapeInstance {
            polygon: self.polygon.clone(),
            boundary: self.boundary.clone(),
            state: self.state.clone(),
            shaders: self.shaders.clone(),
            id: RenderID::gen(),
        }
    }
    /// Returns a reference to the instance descriptor.
    #[inline(always)]
    pub fn instance_state(&self) -> &InstanceState { &self.state }
    /// Returns the mutable reference to the instance descriptor.
    #[inline(always)]
    pub fn instance_state_mut(&mut self) -> &mut InstanceState { &mut self.state }
    /// swap render faces
    #[inline(always)]
    pub fn swap_faces(&mut self, other: &mut Self) {
        let tmp = self.polygon.clone();
        self.polygon = other.polygon.clone();
        other.polygon = tmp;
        let tmp = self.boundary.clone();
        self.boundary = other.boundary.clone();
        other.boundary = tmp;
    }
}
