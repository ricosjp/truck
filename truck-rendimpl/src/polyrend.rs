use crate::*;
use polymesh::Vertex;
use std::collections::HashMap;

impl IntoInstance for PolygonMesh {
    type Instance = PolygonInstance;
    #[inline(always)]
    fn into_instance(&self, device: &Device, desc: InstanceDescriptor) -> Self::Instance {
        let (vb, ib) =
            ExpandedPolygon::from(self).buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device);
        PolygonInstance {
            polygon: Arc::new(Mutex::new((Arc::new(vb), Arc::new(ib)))),
            desc,
            id: RenderID::gen(),
        }
    }
    #[inline(always)]
    fn update_instance(&self, device: &Device, instance: &mut Self::Instance) {
        let (vb, ib) =
            ExpandedPolygon::from(self).buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device);
        *instance.polygon.lock().unwrap() = (Arc::new(vb), Arc::new(ib));
    }
}

impl IntoInstance for StructuredMesh {
    type Instance = PolygonInstance;
    fn into_instance(&self, device: &Device, desc: InstanceDescriptor) -> Self::Instance {
        let (vb, ib) =
            ExpandedPolygon::from(self).buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device);
        PolygonInstance {
            polygon: Arc::new(Mutex::new((Arc::new(vb), Arc::new(ib)))),
            desc,
            id: RenderID::gen(),
        }
    }
    #[inline(always)]
    fn update_instance(&self, device: &Device, instance: &mut Self::Instance) {
        let (vb, ib) =
            ExpandedPolygon::from(self).buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device);
        *instance.polygon.lock().unwrap() = (Arc::new(vb), Arc::new(ib));
    }
}

impl PolygonInstance {
    /// Clone the instance as another drawn element.
    #[inline(always)]
    pub fn clone_instance(&self) -> PolygonInstance {
        PolygonInstance {
            polygon: self.polygon.clone(),
            desc: self.desc.clone(),
            id: RenderID::gen(),
        }
    }
    /// Returns a reference to the instance descriptor.
    #[inline(always)]
    pub fn descriptor(&self) -> &InstanceDescriptor { &self.desc }
    /// Returns the mutable reference to instance descriptor.
    #[inline(always)]
    pub fn descriptor_mut(&mut self) -> &mut InstanceDescriptor { &mut self.desc }

    #[inline(always)]
    fn non_textured_bdl(&self, device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(device, {
            &[
                InstanceDescriptor::matrix_bgl_entry(),
                InstanceDescriptor::material_bgl_entry(),
            ]
        })
    }

    #[inline(always)]
    fn textured_bdl(&self, device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                InstanceDescriptor::matrix_bgl_entry(),
                InstanceDescriptor::material_bgl_entry(),
                InstanceDescriptor::textureview_bgl_entry(),
                InstanceDescriptor::sampler_bgl_entry(),
            ],
        )
    }

    #[inline(always)]
    fn non_textured_bg(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        bind_group_util::create_bind_group(
            device,
            layout,
            vec![
                self.desc.matrix_buffer(device).binding_resource(),
                self.desc.material.buffer(device).binding_resource(),
            ],
        )
    }
    #[inline(always)]
    fn textured_bg(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        let (view, sampler) = self.desc.textureview_and_sampler(device);
        bind_group_util::create_bind_group(
            device,
            layout,
            vec![
                self.desc.matrix_buffer(device).binding_resource(),
                self.desc.material.buffer(device).binding_resource(),
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
        self.desc.pipeline_with_shader(
            vertex_shader,
            fragment_shader,
            device_handler,
            layout,
            sample_count,
        )
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
        Arc::new(match self.desc.texture.is_some() {
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
        Arc::new(match self.desc.texture.is_some() {
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
        let fragment_shader = match self.desc.texture.is_some() {
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

impl ExpandedPolygon {
    pub fn buffers(
        &self,
        vertex_usage: BufferUsage,
        index_usage: BufferUsage,
        device: &Device,
    ) -> (BufferHandler, BufferHandler) {
        let vertex_buffer = BufferHandler::from_slice(&self.vertices, device, vertex_usage);
        let index_buffer = BufferHandler::from_slice(&self.indices, device, index_usage);
        (vertex_buffer, index_buffer)
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: Vertex,
    glpolymesh: &mut ExpandedPolygon,
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

impl Default for ExpandedPolygon {
    fn default() -> ExpandedPolygon {
        ExpandedPolygon {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl From<&PolygonMesh> for ExpandedPolygon {
    fn from(polymesh: &PolygonMesh) -> ExpandedPolygon {
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

impl From<&StructuredMesh> for ExpandedPolygon {
    fn from(mesh: &StructuredMesh) -> ExpandedPolygon {
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
