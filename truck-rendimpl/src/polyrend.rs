use crate::*;
use std::collections::HashMap;

impl IntoInstance for PolygonMesh {
    type Instance = PolygonInstance;
    #[inline(always)]
    fn into_instance(&self, device: &Device, desc: InstanceDescriptor) -> Self::Instance {
        let (vb, ib) = ExpandedPolygon::from(self).buffers(device);
        PolygonInstance {
            polygon: Arc::new(Mutex::new((Arc::new(vb), Arc::new(ib)))),
            desc,
            id: Default::default(),
        }
    }
    #[inline(always)]
    fn update_instance(&self, device: &Device, instance: &mut Self::Instance) {
        let (vb, ib) = ExpandedPolygon::from(self).buffers(device);
        *instance.polygon.lock().unwrap() = (Arc::new(vb), Arc::new(ib));
    }
}

impl IntoInstance for StructuredMesh {
    type Instance = PolygonInstance;
    fn into_instance(&self, device: &Device, desc: InstanceDescriptor) -> Self::Instance {
        let (vb, ib) = ExpandedPolygon::from(self).buffers(device);
        PolygonInstance {
            polygon: Arc::new(Mutex::new((Arc::new(vb), Arc::new(ib)))),
            desc,
            id: Default::default(),
        }
    }
    #[inline(always)]
    fn update_instance(&self, device: &Device, instance: &mut Self::Instance) {
        let (vb, ib) = ExpandedPolygon::from(self).buffers(device);
        *instance.polygon.lock().unwrap() = (Arc::new(vb), Arc::new(ib));
    }
}

impl Clone for PolygonInstance {
    #[inline(always)]
    fn clone(&self) -> PolygonInstance {
        PolygonInstance {
            polygon: self.polygon.clone(),
            desc: self.desc.clone(),
            id: Default::default(),
        }
    }
}

impl PolygonInstance {
    #[inline(always)]
    pub fn descriptor(&self) -> &InstanceDescriptor { &self.desc }
    #[inline(always)]
    pub fn descriptor_mut(&mut self) -> &mut InstanceDescriptor { &mut self.desc }

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
        crate::create_bind_group(
            device,
            layout,
            vec![
                self.desc.matrix_buffer(device).binding_resource(),
                self.desc.material.buffer(device).binding_resource(),
            ],
        )
    }
    #[inline(always)]
    fn textured_bg(&self, device: &Device, queue: &Queue, layout: &BindGroupLayout) -> BindGroup {
        let (view, sampler) = self.desc.textureview_and_sampler(device, queue);
        crate::create_bind_group(
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
    #[inline(always)]
    pub fn default_vertex_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/polygon.vert.spv")
    }

    #[inline(always)]
    pub fn default_fragment_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/polygon.frag.spv")
    }

    #[inline(always)]
    pub fn default_textured_fragment_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/textured-polygon.frag.spv")
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
            true => self.textured_bg(&device_handler.device(), &device_handler.queue(), layout),
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
        let vertex_shader = Self::default_vertex_shader();
        let fragment_shader = match self.desc.texture.is_some() {
            true => include_spirv!("shaders/textured-polygon.frag.spv"),
            false => include_spirv!("shaders/polygon.frag.spv"),
        };
        self.pipeline_with_shader(
            vertex_shader,
            fragment_shader,
            device_handler,
            layout,
            sample_count,
        )
    }
}

impl ExpandedPolygon {
    pub fn buffers(&self, device: &Device) -> (BufferHandler, BufferHandler) {
        let vertex_buffer = BufferHandler::from_slice(&self.vertices, device, BufferUsage::VERTEX);
        let index_buffer = BufferHandler::from_slice(&self.indices, device, BufferUsage::INDEX);
        (vertex_buffer, index_buffer)
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: &[usize; 3],
    glpolymesh: &mut ExpandedPolygon,
    vertex_map: &mut HashMap<[usize; 3], u32>,
) {
    let key = [vertex[0], vertex[1], vertex[2]];
    let idx = match vertex_map.get(&key) {
        Some(idx) => *idx,
        None => {
            let idx = glpolymesh.vertices.len() as u32;
            let position = (&polymesh.positions[key[0]]).cast().unwrap().into();
            let uv_coord = match polymesh.uv_coords.is_empty() {
                true => [0.0, 0.0],
                false => (&polymesh.uv_coords[key[1]]).cast().unwrap().into(),
            };
            let normal = match polymesh.normals.is_empty() {
                true => [0.0, 0.0, 0.0],
                false => (&polymesh.normals[key[2]]).cast().unwrap().into(),
            };
            let wgpuvertex = AttrVertex {
                position,
                uv_coord,
                normal,
            };
            vertex_map.insert(key, idx);
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
        let mut vertex_map = HashMap::<[usize; 3], u32>::new();
        for tri in &polymesh.tri_faces {
            signup_vertex(polymesh, &tri[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[2], &mut glpolymesh, &mut vertex_map);
        }
        for quad in &polymesh.quad_faces {
            signup_vertex(polymesh, &quad[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[2], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
        }
        for face in &polymesh.other_faces {
            for i in 2..face.len() {
                signup_vertex(polymesh, &face[0], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i - 1], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i], &mut glpolymesh, &mut vertex_map);
            }
        }
        glpolymesh
    }
}

impl From<&StructuredMesh> for ExpandedPolygon {
    fn from(mesh: &StructuredMesh) -> ExpandedPolygon {
        let mut glpolymesh = ExpandedPolygon::default();
        let (m, n) = (mesh.uv_division.0.len(), mesh.uv_division.1.len());
        for i in 0..m {
            for j in 0..n {
                glpolymesh.vertices.push(AttrVertex {
                    position: mesh.positions[i][j].cast().unwrap().into(),
                    uv_coord: [mesh.uv_division.0[i] as f32, mesh.uv_division.1[j] as f32],
                    normal: mesh.normals[i][j].cast().unwrap().into(),
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

impl std::fmt::Debug for PolygonInstance {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        f.pad("PolygonInstance {\n")?;
        f.write_fmt(format_args!("  polygon: {:?}\n", self.polygon))?;
        f.write_fmt(format_args!("  matrix: {:?}\n", self.desc.matrix))?;
        f.write_fmt(format_args!("  material: {:?}\n", self.desc.material))?;
        match self.desc.texture {
            Some(_) => f.write_fmt(format_args!("Some(<omitted>)\n}}")),
            None => f.write_fmt(format_args!("None\n}}")),
        }
    }
}
