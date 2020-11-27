use crate::*;

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

fn face_instance(
    device: &Device,
    _: &Queue,
    face: &Face,
    desc: &Arc<Mutex<InstanceDescriptor>>,
) -> Option<FaceInstance>
{
    let surface = face.oriented_surface();
    let mesh = StructuredMesh::from_surface(&surface, 0.01);
    let (vb, ib) = ExpandedPolygon::from(&mesh).buffers(device);
    let mut boundary = Vec::new();
    for edge in face.boundary_iters().into_iter().flatten() {
        let curve = edge.oriented_curve();
        let division = curve.parameter_division(0.01);
        let mut hint = presearch(&surface, curve.subs(division[0]));
        let mut this_boundary = Vec::new();
        for t in division {
            let pt = curve.subs(t);
            hint = match surface.search_parameter(pt, hint) {
                Some(got) => got,
                None => return None,
            };
            this_boundary.push([hint.0 as f32, hint.1 as f32]);
        }
        for window in this_boundary.as_slice().windows(2) {
            boundary.push([window[0][0], window[0][1], window[1][0], window[1][1]]);
        }
    }
    Some(FaceInstance {
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
        desc: Arc::clone(desc),
        id: Default::default(),
    })
}

impl IntoInstance for Shell {
    type Instance = ShapeInstance;
    fn into_instance(
        &self,
        device: &Device,
        queue: &Queue,
        desc: InstanceDescriptor,
    ) -> ShapeInstance
    {
        let desc = Arc::new(Mutex::new(desc));
        let closure = |face| face_instance(device, queue, face, &desc).unwrap();
        let faces = self.face_iter().map(closure).collect();
        ShapeInstance { faces, desc }
    }
}

mod ficonfig {
    use crate::*;
    #[inline(always)]
    pub fn boundary_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
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
            visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
            ty: BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
    #[inline(always)]
    pub fn non_textured_bdl(device: &Device) -> BindGroupLayout {
        truck_platform::create_bind_group_layout(
            device,
            &[
                InstanceDescriptor::matrix_bgl_entry(),
                InstanceDescriptor::material_bgl_entry(),
                boundary_bgl_entry(),
                boundary_length_bgl_entry(),
            ],
        )
    }
    #[inline(always)]
    pub fn textured_bdl(device: &Device) -> BindGroupLayout {
        truck_platform::create_bind_group_layout(
            device,
            &[
                InstanceDescriptor::matrix_bgl_entry(),
                InstanceDescriptor::material_bgl_entry(),
                InstanceDescriptor::textureview_bgl_entry(),
                InstanceDescriptor::sampler_bgl_entry(),
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
        buffer: &FaceInstance,
    ) -> BindGroup
    {
        let desc = &*buffer.desc.lock().unwrap();
        crate::create_bind_group(
            handler.device(),
            layout,
            vec![
                desc.matrix_buffer(handler.device()).binding_resource(),
                desc.material.buffer(handler.device()).binding_resource(),
                buffer.boundary.binding_resource(),
                buffer.boundary_length.binding_resource(),
            ],
        )
    }
    #[inline(always)]
    pub(super) fn textured_bind_group(
        handler: &DeviceHandler,
        layout: &BindGroupLayout,
        buffer: &FaceInstance,
    ) -> BindGroup
    {
        let desc = &*buffer.desc.lock().unwrap();
        let (view, sampler) = desc.textureview_and_sampler(handler.device(), handler.queue());
        crate::create_bind_group(
            handler.device(),
            layout,
            vec![
                desc.matrix_buffer(handler.device()).binding_resource(),
                desc.material.buffer(handler.device()).binding_resource(),
                BindingResource::TextureView(&view),
                BindingResource::Sampler(&sampler),
                buffer.boundary.binding_resource(),
                buffer.boundary_length.binding_resource(),
            ],
        )
    }
}

impl<'a> Rendered for FaceInstance {
    impl_get_set_id!(id);
    #[inline(always)]
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let buffers = &self.surface;
        (Arc::clone(&buffers.0), Some(Arc::clone(&buffers.1)))
    }
    #[inline(always)]
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        let flag = self.desc.try_lock().unwrap().texture.is_some();
        Arc::new(ficonfig::bind_group_layout(handler.device(), flag))
    }
    #[inline(always)]
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let flag = self.desc.try_lock().unwrap().texture.is_some();
        let bind_group = match flag {
            true => ficonfig::textured_bind_group(handler, layout, self),
            false => ficonfig::non_textured_bind_group(handler, layout, self),
        };
        Arc::new(bind_group)
    }
    #[inline(always)]
    fn pipeline(&self, handler: &DeviceHandler, layout: &PipelineLayout) -> Arc<RenderPipeline> {
        let desc = &*self.desc.try_lock().unwrap();
        let vertex_shader = include_spirv!("shaders/polygon.vert.spv");
        let fragment_shader = match desc.texture.is_some() {
            true => include_spirv!("shaders/textured-face.frag.spv"),
            false => include_spirv!("shaders/face.frag.spv"),
        };
        desc.pipeline_with_shader(vertex_shader, fragment_shader, handler, layout)
    }
}

impl<'a> IntoIterator for &'a ShapeInstance {
    type Item = &'a FaceInstance;
    type IntoIter = std::slice::Iter<'a, FaceInstance>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.faces.iter() }
}

impl<'a> IntoIterator for &'a mut ShapeInstance {
    type Item = &'a mut FaceInstance;
    type IntoIter = std::slice::IterMut<'a, FaceInstance>;
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter { self.faces.iter_mut() }
}
