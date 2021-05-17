use crate::*;
use modeling::geometry::Surface;

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

fn presearch(surface: &Surface, pt: Point3) -> (f64, f64) {
    match surface {
        Surface::Plane(surface) => {
            let v = surface.get_parameter(pt);
            (v[0], v[1])
        }
        Surface::BSplineSurface(surface) => modeling::algo::surface::presearch(surface, pt, surface.parameter_range(), 50),
        Surface::NURBSSurface(surface) => modeling::algo::surface::presearch(surface, pt, surface.parameter_range(), 50),
        Surface::RevolutedCurve(surface) => modeling::algo::surface::presearch(surface, pt, surface.parameter_range(), 50),
    }
}

fn meshing_surface(surface: &Surface, precision: f64, boundary: &Vec<[f32; 4]>) -> StructuredMesh {
    match surface {
        Surface::BSplineSurface(surface) => StructuredMesh::from_surface(surface, precision),
        Surface::NURBSSurface(surface) => StructuredMesh::from_surface(surface, precision),
        Surface::RevolutedCurve(surface) => StructuredMesh::from_surface(surface, precision),
        Surface::Plane(plane) => {
            let bdd: BoundingBox<Vector2> = boundary
                .iter()
                .flat_map(|v| {
                    vec![
                        Vector2::new(v[0] as f64, v[1] as f64),
                        Vector2::new(v[2] as f64, v[3] as f64),
                    ]
                })
                .collect();
            let min = bdd.min();
            let max = bdd.max();
            let positions = vec![
                vec![plane.subs(min[0], min[1]), plane.subs(min[0], max[1])],
                vec![plane.subs(max[0], min[1]), plane.subs(max[0], max[1])],
            ];
            let uv_division = (vec![min[0], max[0]], vec![min[1], max[1]]);
            let normal = plane.normal();
            let normals = vec![vec![normal, normal], vec![normal, normal]];
            StructuredMesh::new(positions, uv_division, normals)
        }
    }
}

fn add_face(
    face: &modeling::Face,
    mesh_precision: f64,
    expolygon: &mut ExpandedPolygon<AttrVertex>,
    boundaries: &mut Vec<[f32; 4]>,
) -> Option<()> {
    let ExpandedPolygon {
        ref mut vertices,
        ref mut indices,
    } = expolygon;
    let inf = boundaries.len() as u32;
    let index_offset = vertices.len() as u32;
    let surface = face.oriented_surface();
    for edge in face.boundary_iters().into_iter().flatten() {
        let curve = edge.oriented_curve();
        let division = curve.parameter_division(mesh_precision);
        let mut hint = presearch(&surface, curve.subs(division[0]));
        let mut this_boundary = Vec::new();
        for t in division {
            let pt = curve.subs(t);
            hint = match surface.search_parameter(pt, hint, SURFACE_MESHING_TRIALS) {
                Some(got) => got,
                None => {
                    if surface.subs(hint.0, hint.1).near(&pt) {
                        hint
                    } else {
                        let hint0 = presearch(&surface, pt);
                        match surface.search_parameter(pt, hint0, SURFACE_MESHING_TRIALS) {
                            Some(got) => got,
                            None => return None,
                        }
                    }
                }
            };
            this_boundary.push([hint.0 as f32, hint.1 as f32]);
        }
        for window in this_boundary.as_slice().windows(2) {
            boundaries.push([window[0][0], window[0][1], window[1][0], window[1][1]]);
        }
    }
    let sup = boundaries.len() as u32;
    let mesh = &meshing_surface(&surface, mesh_precision, &boundaries);
    vertices.extend(
        (0..mesh.positions().len())
            .flat_map(move |i| (0..mesh.positions()[0].len()).map(move |j| (i, j)))
            .map(move |(i, j)| AttrVertex {
                position: mesh.positions()[i][j].cast().unwrap().into(),
                uv_coord: match mesh.uv_division() {
                    Some((u, v)) => [u[i] as f32, v[j] as f32],
                    None => [0.0, 0.0],
                },
                normal: match mesh.normals() {
                    Some(normals) => normals[i][j].cast().unwrap().into(),
                    None => [0.0, 0.0, 0.0],
                },
                boundary_range: [inf, sup],
            }),
    );
    let len = mesh.positions()[0].len() as u32;
    (1..mesh.positions().len() as u32)
        .flat_map(move |i| (1..len).map(move |j| (i, j)))
        .for_each(move |(i, j)| {
            indices.push(index_offset + (i - 1) * len + (j - 1));
            indices.push(index_offset + i * len + (j - 1));
            indices.push(index_offset + (i - 1) * len + j);
            indices.push(index_offset + (i - 1) * len + j);
            indices.push(index_offset + i * len + (j - 1));
            indices.push(index_offset + i * len + j);
        });
    Some(())
}

impl TryIntoInstance<ShapeInstance> for Shell {
    type Descriptor = ShapeInstanceDescriptor;
    /// Tries to create `ShapeInstance` from `Shell`.
    /// # Failures
    /// Failure occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    fn try_into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &ShapeShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> Option<ShapeInstance> {
        let device = handler.device();
        let mut expolygon = ExpandedPolygon::default();
        let mut boundaries = Vec::new();
        self.face_iter().try_for_each(|face| {
            add_face(face, desc.mesh_precision, &mut expolygon, &mut boundaries)
        })?;
        let (vb, ib) = expolygon.buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device);
        Some(ShapeInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            boundary: Arc::new(BufferHandler::from_slice(
                &boundaries,
                device,
                BufferUsage::STORAGE,
            )),
            state: desc.instance_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        })
    }
}

impl IntoInstance<ShapeInstance> for Shell {
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
        shaders: &ShapeShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> ShapeInstance {
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
                let division = curve.parameter_division(desc.polyline_precision);
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

impl TryIntoInstance<ShapeInstance> for Solid {
    type Descriptor = ShapeInstanceDescriptor;
    /// Tries to create `ShapeInstance` from `Solid`.
    /// # Failures
    /// Failure occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    fn try_into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &ShapeShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> Option<ShapeInstance> {
        let device = handler.device();
        let mut expolygon = ExpandedPolygon::default();
        let mut boundaries = Vec::new();
        self.boundaries()
            .iter()
            .flat_map(Shell::face_iter)
            .try_for_each(|face| {
                add_face(face, desc.mesh_precision, &mut expolygon, &mut boundaries)
            })?;
        let (vb, ib) = expolygon.buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device);
        Some(ShapeInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            boundary: Arc::new(BufferHandler::from_slice(
                &boundaries,
                device,
                BufferUsage::STORAGE,
            )),
            state: desc.instance_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        })
    }
}

impl IntoInstance<ShapeInstance> for Solid {
    type Descriptor = ShapeInstanceDescriptor;
    /// Tries to create `ShapeInstance` from `Solid`.
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
        shaders: &ShapeShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> ShapeInstance {
        self.try_into_instance(handler, shaders, desc)
            .expect("failed to create instance")
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
                let division = curve.parameter_division(desc.polyline_precision);
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
                ty: BufferBindingType::Storage {
                    read_only: true,
                },
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
                InstanceState::matrix_bgl_entry(),
                InstanceState::material_bgl_entry(),
                Self::boundary_bgl_entry(),
            ],
        )
    }

    #[inline(always)]
    fn textured_bgl(device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                InstanceState::matrix_bgl_entry(),
                InstanceState::material_bgl_entry(),
                InstanceState::textureview_bgl_entry(),
                InstanceState::sampler_bgl_entry(),
                Self::boundary_bgl_entry(),
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
                self.state
                    .matrix_buffer(handler.device())
                    .binding_resource(),
                self.state
                    .material_buffer(handler.device())
                    .binding_resource(),
                self.boundary.binding_resource(),
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
                self.state
                    .matrix_buffer(handler.device())
                    .binding_resource(),
                self.state
                    .material_buffer(handler.device())
                    .binding_resource(),
                BindingResource::TextureView(&view),
                BindingResource::Sampler(&sampler),
                self.boundary.binding_resource(),
            ],
        )
    }
}

impl ShapeInstance {
    /// Returns the default vertex shader module source.
    ///
    /// The GLSL original code is `src/shaders/polygon.vert`.
    #[inline(always)]
    pub fn default_vertex_shader() -> ShaderModuleDescriptor<'static> {
        include_spirv!("shaders/face.vert.spv")
    }

    /// Returns the default fragment shader module source for non-textured polygons.
    ///
    /// The GLSL original code is `src/shaders/face.frag`.
    #[inline(always)]
    pub fn default_fragment_shader() -> ShaderModuleDescriptor<'static> {
        //include_spirv!("shaders/face.frag.spv")
        ShaderModuleDescriptor {
            label: Some("shaders/face.frag.spv"),
            source: wgpu::util::make_spirv(include_bytes!("shaders/face.frag.spv")),
            flags: ShaderFlags::empty(),
        }
    }

    /// Returns the default fragment shader module source for textured polygons.
    ///
    /// The GLSL original code is `src/shaders/textured-face.frag`.
    #[inline(always)]
    pub fn default_textured_fragment_shader() -> ShaderModuleDescriptor<'static> {
        //include_spirv!("shaders/textured-face.frag.spv")
        ShaderModuleDescriptor {
            label: Some("shaders/textured-face.frag.spv"),
            source: wgpu::util::make_spirv(include_bytes!("shaders/textured-face.frag.spv")),
            flags: ShaderFlags::empty(),
        }
    }
}

impl Instance for ShapeInstance {
    type Shaders = ShapeShaders;
    fn standard_shaders(creator: &InstanceCreator) -> ShapeShaders {
        creator.shape_shaders.clone()
    }
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
            true => (&self.shaders.tex_fragment_module, self.shaders.tex_fragment_entry),
            false => (&self.shaders.fragment_module, self.shaders.fragment_entry),
        };
        let cull_mode = match self.state.backface_culling {
            true => Some(wgpu::Face::Back),
            false => None,
        };
        Arc::new(handler.device().create_render_pipeline(&RenderPipelineDescriptor {
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
        }))
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
