use crate::*;
use polymesh::Vertex;
use std::collections::HashMap;

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub(super) struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}

impl IntoInstance for PolygonMesh {
    type Instance = PolygonInstance;
    type Descriptor = PolygonInstanceDescriptor;
    #[inline(always)]
    fn into_instance(&self, device: &Device, desc: &Self::Descriptor) -> Self::Instance {
        let (vb, ib) =
            ExpandedPolygon::from(self).buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device);
        PolygonInstance {
            polygon: Arc::new(Mutex::new((Arc::new(vb), Arc::new(ib)))),
            state: desc.instance_state.clone(),
            id: RenderID::gen(),
        }
    }
}

impl IntoInstance for StructuredMesh {
    type Instance = PolygonInstance;
    type Descriptor = PolygonInstanceDescriptor;
    fn into_instance(&self, device: &Device, desc: &Self::Descriptor) -> Self::Instance {
        let (vb, ib) =
            ExpandedPolygon::from(self).buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device);
        PolygonInstance {
            polygon: Arc::new(Mutex::new((Arc::new(vb), Arc::new(ib)))),
            state: desc.instance_state.clone(),
            id: RenderID::gen(),
        }
    }
}

impl PolygonInstance {
    /// Clone the instance as another drawn element.
    #[inline(always)]
    pub fn clone_instance(&self) -> PolygonInstance {
        PolygonInstance {
            polygon: self.polygon.clone(),
            state: self.state.clone(),
            id: RenderID::gen(),
        }
    }
    /// Returns a reference to the instance descriptor.
    #[inline(always)]
    pub fn instance_state(&self) -> &InstanceState { &self.state }
    /// Returns the mutable reference to instance descriptor.
    #[inline(always)]
    pub fn instance_state_mut(&mut self) -> &mut InstanceState { &mut self.state }

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
                InstanceState::matrix_bgl_entry(),
                InstanceState::material_bgl_entry(),
            ]
        })
    }

    #[inline(always)]
    fn textured_bdl(&self, device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                InstanceState::matrix_bgl_entry(),
                InstanceState::material_bgl_entry(),
                InstanceState::textureview_bgl_entry(),
                InstanceState::sampler_bgl_entry(),
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

    /// Returns the default vertex shader module source.
    ///
    /// The GLSL original code is `src/shaders/polygon.vert`.
    #[inline(always)]
    pub fn default_vertex_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/polygon.vert.spv")
    }

    /// Returns the default fragment shader module source for non-textured polygons.
    ///
    /// The GLSL original code is `src/shaders/polygon.frag`.
    #[inline(always)]
    pub fn default_fragment_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/polygon.frag.spv")
    }

    /// Returns the default fragment shader module source for textured polygons.
    ///
    /// The GLSL original code is `src/shaders/textured-polygon.frag`.
    #[inline(always)]
    pub fn default_textured_fragment_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/textured-polygon.frag.spv")
    }

    /// Returns the pipeline with developer's custom shader.
    #[inline(always)]
    pub fn pipeline_with_shader(
        &self,
        vertex_shader: ShaderModuleSource,
        fragment_shader: ShaderModuleSource,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
        let device = device_handler.device();
        let sc_desc = device_handler.sc_desc();
        let cull_mode = match self.state.backface_culling {
            true => CullMode::Back,
            false => CullMode::None,
        };
        let vertex_module = device.create_shader_module(vertex_shader);
        let fragment_module = device.create_shader_module(fragment_shader);
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
                cull_mode,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
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
                stencil: StencilStateDescriptor::default(),
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
            sample_count,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            label: None,
        });
        Arc::new(pipeline)
 
    }
}

impl Rendered for PolygonInstance {
    impl_render_id!(id);

    #[inline(always)]
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let polygon = self.polygon.lock().unwrap().clone();
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
        let fragment_shader = match self.state.texture.is_some() {
            true => Self::default_textured_fragment_shader(),
            false => Self::default_fragment_shader(),
        };
        self.pipeline_with_shader(
            Self::default_vertex_shader(),
            fragment_shader,
            device_handler,
            layout,
            sample_count,
        )
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: Vertex,
    glpolymesh: &mut ExpandedPolygon<AttrVertex>,
    vertex_map: &mut HashMap<Vertex, u32>,
) {
    let idx = match vertex_map.get(&vertex) {
        Some(idx) => *idx,
        None => {
            let idx = glpolymesh.vertices.len() as u32;
            let position = polymesh.positions()[vertex.pos].cast().unwrap().into();
            let uv_coord = match vertex.uv {
                Some(uv) => polymesh.uv_coords()[uv].cast().unwrap().into(),
                None => [0.0, 0.0],
            };
            let normal = match vertex.nor {
                Some(nor) => polymesh.normals()[nor].cast().unwrap().into(),
                None => [0.0, 0.0, 0.0],
            };
            let wgpuvertex = AttrVertex {
                position,
                uv_coord,
                normal,
            };
            vertex_map.insert(vertex, idx);
            glpolymesh.vertices.push(wgpuvertex);
            idx
        }
    };
    glpolymesh.indices.push(idx);
}

impl From<&PolygonMesh> for ExpandedPolygon<AttrVertex> {
    fn from(polymesh: &PolygonMesh) -> ExpandedPolygon<AttrVertex> {
        let mut glpolymesh = ExpandedPolygon::default();
        let mut vertex_map = HashMap::<Vertex, u32>::new();
        for tri in polymesh.faces().tri_faces() {
            signup_vertex(polymesh, tri[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, tri[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, tri[2], &mut glpolymesh, &mut vertex_map);
        }
        for quad in polymesh.faces().quad_faces() {
            signup_vertex(polymesh, quad[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[3], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[2], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[3], &mut glpolymesh, &mut vertex_map);
        }
        for face in polymesh.faces().other_faces() {
            for i in 2..face.len() {
                signup_vertex(polymesh, face[0], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, face[i - 1], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, face[i], &mut glpolymesh, &mut vertex_map);
            }
        }
        glpolymesh
    }
}

impl From<&StructuredMesh> for ExpandedPolygon<AttrVertex> {
    fn from(mesh: &StructuredMesh) -> ExpandedPolygon<AttrVertex> {
        let mut glpolymesh = ExpandedPolygon::default();
        let (m, n) = (mesh.positions().len(), mesh.positions()[0].len());
        for i in 0..m {
            for j in 0..n {
                glpolymesh.vertices.push(AttrVertex {
                    position: mesh.positions()[i][j].cast().unwrap().into(),
                    uv_coord: match mesh.uv_division() {
                        Some(uv_division) => [uv_division.0[i] as f32, uv_division.1[j] as f32],
                        None => [0.0, 0.0],
                    },
                    normal: match mesh.normals() {
                        Some(normals) => normals[i][j].cast().unwrap().into(),
                        None => [0.0, 0.0, 0.0],
                    },
                });
            }
        }
        for i in 1..m {
            for j in 1..n {
                glpolymesh.indices.push(((i - 1) * n + j - 1) as u32);
                glpolymesh.indices.push((i * n + j - 1) as u32);
                glpolymesh.indices.push(((i - 1) * n + j) as u32);
                glpolymesh.indices.push(((i - 1) * n + j) as u32);
                glpolymesh.indices.push((i * n + j - 1) as u32);
                glpolymesh.indices.push((i * n + j) as u32);
            }
        }
        glpolymesh
    }
}
