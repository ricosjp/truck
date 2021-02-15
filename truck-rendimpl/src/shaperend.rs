use crate::*;
use std::f64::consts::PI;
use truck_polymesh::prelude::*;

impl Default for ShapeInstanceDescriptor {
    #[inline(always)]
    fn default() -> Self {
        ShapeInstanceDescriptor {
            instance_state: Default::default(),
            mesh_precision: 0.005,
            normal_completion_tolerance: PI / 6.0,
        }
    }
}

impl FaceInstance {
    /// Clone the instance as another drawn element.
    #[inline(always)]
    fn clone_instance(&self) -> Self {
        FaceInstance {
            buffer: self.buffer.clone(),
            id: RenderID::gen(),
        }
    }
}

fn presearch(surface: &NURBSSurface, point: Point3) -> (f64, f64) {
    const N: usize = 50;
    let mut res = (0.0, 0.0);
    let mut min = std::f64::INFINITY;
    for i in 0..=N {
        for j in 0..=N {
            let p = i as f64 / N as f64;
            let q = j as f64 / N as f64;
            let u = surface.uknot_vec()[0] + p * surface.uknot_vec().range_length();
            let v = surface.vknot_vec()[0] + q * surface.vknot_vec().range_length();
            let dist = surface.subs(u, v).distance2(point);
            if dist < min {
                min = dist;
                res = (u, v);
            }
        }
    }
    res
}

fn face_buffer(
    device: &Device,
    face: &Face,
    mesh_precision: f64,
    normal_completion_tolerance: f64,
) -> Option<FaceBuffer> {
    let surface = face.oriented_surface();
    let mesh = StructuredMesh::from_surface(&surface, mesh_precision);
    let mut normals = mesh.normals().unwrap().iter().flatten();
    let reg_norm = normals.all(|normal| normal.magnitude2().near(&1.0));
    let mut boundary = Vec::new();
    for edge in face.boundary_iters().into_iter().flatten() {
        let curve = edge.oriented_curve();
        let division = curve.parameter_division(mesh_precision);
        let mut hint = presearch(&surface, curve.subs(division[0]));
        let mut this_boundary = Vec::new();
        for t in division {
            let pt = curve.subs(t);
            hint = match surface.search_parameter(pt, hint) {
                Some(got) => got,
                None => {
                    if surface.subs(hint.0, hint.1).near(&pt) {
                        hint
                    } else {
                        let hint0 = presearch(&surface, pt);
                        match surface.search_parameter(pt, hint0) {
                            Some(got) => got,
                            None => return None,
                        }
                    }
                },
            };
            this_boundary.push([hint.0 as f32, hint.1 as f32]);
        }
        for window in this_boundary.as_slice().windows(2) {
            boundary.push([window[0][0], window[0][1], window[1][0], window[1][1]]);
        }
    }
    let (vb, ib) = if reg_norm {
        ExpandedPolygon::from(&mesh).buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device)
    } else {
        let mut mesh = mesh.destruct();
        mesh.remove_degenerate_faces()
            .normalize_normals()
            .add_smooth_normals(normal_completion_tolerance, false);
        ExpandedPolygon::from(&mesh).buffers(BufferUsage::VERTEX, BufferUsage::INDEX, device)
    };
    Some(FaceBuffer {
        surface: (Arc::new(vb), Arc::new(ib)),
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

impl IntoInstance for Shell {
    type Instance = ShapeInstance;
    type Descriptor = ShapeInstanceDescriptor;
    #[inline(always)]
    fn into_instance(&self, device: &Device, desc: &Self::Descriptor) -> ShapeInstance {
        let faces = self
            .face_iter()
            .map(|face| FaceInstance {
                buffer: Arc::new(Mutex::new(
                    face_buffer(
                        device,
                        face,
                        desc.mesh_precision,
                        desc.normal_completion_tolerance,
                    )
                    .unwrap(),
                )),
                id: RenderID::gen(),
            })
            .collect();
        ShapeInstance {
            faces,
            state: desc.instance_state.clone(),
        }
    }
}

impl IntoInstance for Solid {
    type Instance = ShapeInstance;
    type Descriptor = ShapeInstanceDescriptor;
    #[inline(always)]
    fn into_instance(&self, device: &Device, desc: &Self::Descriptor) -> ShapeInstance {
        let faces = self
            .boundaries()
            .iter()
            .flat_map(Shell::face_iter)
            .map(|face| FaceInstance {
                buffer: Arc::new(Mutex::new(
                    face_buffer(
                        device,
                        face,
                        desc.mesh_precision,
                        desc.normal_completion_tolerance,
                    )
                    .unwrap(),
                )),
                id: RenderID::gen(),
            })
            .collect();
        ShapeInstance {
            faces,
            state: desc.instance_state.clone(),
        }
    }
}

mod ficonfig {
    use super::*;
    #[inline(always)]
    pub fn boundary_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::StorageBuffer {
                dynamic: false,
                min_binding_size: None,
                readonly: true,
            },
            count: None,
        }
    }
    #[inline(always)]
    pub fn boundary_length_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
    #[inline(always)]
    pub fn non_textured_bdl(device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                InstanceState::matrix_bgl_entry(),
                InstanceState::material_bgl_entry(),
                boundary_bgl_entry(),
                boundary_length_bgl_entry(),
            ],
        )
    }
    #[inline(always)]
    pub fn textured_bdl(device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                InstanceState::matrix_bgl_entry(),
                InstanceState::material_bgl_entry(),
                InstanceState::textureview_bgl_entry(),
                InstanceState::sampler_bgl_entry(),
                boundary_bgl_entry(),
                boundary_length_bgl_entry(),
            ],
        )
    }
    #[inline(always)]
    pub fn bind_group_layout(device: &Device, textured: bool) -> BindGroupLayout {
        match textured {
            true => textured_bdl(device),
            false => non_textured_bdl(device),
        }
    }
    #[inline(always)]
    pub(super) fn non_textured_bind_group(
        handler: &DeviceHandler,
        layout: &BindGroupLayout,
        face: &RenderFace,
    ) -> BindGroup {
        let (buffer, state) = (&face.instance.buffer.lock().unwrap(), &face.state);
        bind_group_util::create_bind_group(
            handler.device(),
            layout,
            vec![
                state.matrix_buffer(handler.device()).binding_resource(),
                state.material_buffer(handler.device()).binding_resource(),
                buffer.boundary.binding_resource(),
                buffer.boundary_length.binding_resource(),
            ],
        )
    }
    #[inline(always)]
    pub(super) fn textured_bind_group(
        handler: &DeviceHandler,
        layout: &BindGroupLayout,
        face: &RenderFace,
    ) -> BindGroup {
        let (buffer, state) = (&face.instance.buffer.lock().unwrap(), &face.state);
        let (view, sampler) = state.textureview_and_sampler(handler.device());
        bind_group_util::create_bind_group(
            handler.device(),
            layout,
            vec![
                state.matrix_buffer(handler.device()).binding_resource(),
                state.material_buffer(handler.device()).binding_resource(),
                BindingResource::TextureView(&view),
                BindingResource::Sampler(&sampler),
                buffer.boundary.binding_resource(),
                buffer.boundary_length.binding_resource(),
            ],
        )
    }
}

impl<'a> RenderFace<'a> {
    /// Returns the default vertex shader module source.
    ///
    /// The GLSL original code is `src/shaders/polygon.vert`.
    #[inline(always)]
    pub fn default_vertex_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/polygon.vert.spv")
    }

    /// Returns the default fragment shader module source for non-textured polygons.
    ///
    /// The GLSL original code is `src/shaders/face.frag`.
    #[inline(always)]
    pub fn default_fragment_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/face.frag.spv")
    }

    /// Returns the default fragment shader module source for textured polygons.
    ///
    /// The GLSL original code is `src/shaders/textured-face.frag`.
    #[inline(always)]
    pub fn default_textured_fragment_shader() -> ShaderModuleSource<'static> {
        include_spirv!("shaders/textured-face.frag.spv")
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
        self.state.pipeline_with_shader(
            vertex_shader,
            fragment_shader,
            device_handler,
            layout,
            sample_count,
        )
    }
}

impl<'a> Rendered for RenderFace<'a> {
    impl_render_id!(instance.id);

    #[inline(always)]
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let buffers = &self.instance.buffer.lock().unwrap().surface;
        (Arc::clone(&buffers.0), Some(Arc::clone(&buffers.1)))
    }
    #[inline(always)]
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(ficonfig::bind_group_layout(
            handler.device(),
            self.state.texture.is_some(),
        ))
    }
    #[inline(always)]
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let bind_group = match self.state.texture.is_some() {
            true => ficonfig::textured_bind_group(handler, layout, self),
            false => ficonfig::non_textured_bind_group(handler, layout, self),
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
        let fragment_shader = match self.state.texture.is_some() {
            true => Self::default_textured_fragment_shader(),
            false => Self::default_fragment_shader(),
        };
        self.pipeline_with_shader(
            Self::default_vertex_shader(),
            fragment_shader,
            handler,
            layout,
            sample_count,
        )
    }
}

impl ShapeInstance {
    /// Clone the instance as another drawn element.
    #[inline(always)]
    pub fn clone_instance(&self) -> Self {
        ShapeInstance {
            faces: self
                .faces
                .iter()
                .map(|face| face.clone_instance())
                .collect(),
            state: self.state.clone(),
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
    pub fn swap_vertex(&mut self, other: &mut Self) {
        let faces: Vec<_> = self.faces.iter().map(|face| face.buffer.clone()).collect();
        self.faces
            .iter_mut()
            .zip(&other.faces)
            .for_each(|(face0, face1)| {
                face0.buffer = face1.buffer.clone();
            });
        other
            .faces
            .iter_mut()
            .zip(faces)
            .for_each(|(face0, buffer)| {
                face0.buffer = buffer;
            });
    }
    /// Creates the vector of `RenderFace` for rendering the shape.
    #[inline(always)]
    pub fn render_faces(&self) -> Vec<RenderFace> {
        let state = &self.state;
        self.faces
            .iter()
            .map(move |instance| RenderFace { instance, state })
            .collect()
    }
}
