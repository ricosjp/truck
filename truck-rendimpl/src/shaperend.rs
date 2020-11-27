use crate::*;

impl Clone for FaceInstance {
    #[inline(always)]
    fn clone(&self) -> FaceInstance {
        FaceInstance {
            buffer: self.buffer.clone(),
            id: Default::default(),
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

fn face_buffer(device: &Device, face: &Face) -> Option<FaceBuffer> {
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
    #[inline(always)]
    fn into_instance(&self, device: &Device, desc: InstanceDescriptor) -> ShapeInstance {
        let faces = self
            .face_iter()
            .map(|face| FaceInstance {
                buffer: Arc::new(Mutex::new(face_buffer(device, face).unwrap())),
                id: Default::default(),
            })
            .collect();
        ShapeInstance { faces, desc }
    }
    #[inline(always)]
    fn update_instance(&self, device: &Device, instance: &mut ShapeInstance) {
        self.face_iter()
            .zip(&mut instance.faces)
            .for_each(|(face, instance)| {
                *instance.buffer.lock().unwrap() = face_buffer(device, face).unwrap()
            })
    }
}

impl IntoInstance for Solid {
    type Instance = ShapeInstance;
    #[inline(always)]
    fn into_instance(&self, device: &Device, desc: InstanceDescriptor) -> ShapeInstance {
        let faces = self
            .boundaries()
            .iter()
            .flat_map(Shell::face_iter)
            .map(|face| FaceInstance {
                buffer: Arc::new(Mutex::new(face_buffer(device, face).unwrap())),
                id: Default::default(),
            })
            .collect();
        ShapeInstance { faces, desc }
    }
    #[inline(always)]
    fn update_instance(&self, device: &Device, instance: &mut ShapeInstance) {
        self.boundaries()
            .iter()
            .flat_map(Shell::face_iter)
            .zip(&mut instance.faces)
            .for_each(|(face, instance)| {
                *instance.buffer.lock().unwrap() = face_buffer(device, face).unwrap()
            })
    }
}

pub struct RenderFace<'a> {
    instance: &'a mut FaceInstance,
    desc: &'a InstanceDescriptor,
}

mod ficonfig {
    use super::*;
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
        face: &RenderFace,
    ) -> BindGroup
    {
        let (buffer, desc) = (&face.instance.buffer.lock().unwrap(), &face.desc);
        crate::create_bind_group(
            handler.device(),
            layout,
            vec![
                desc.matrix_buffer(handler.device()).binding_resource(),
                desc.material_buffer(handler.device()).binding_resource(),
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
    ) -> BindGroup
    {
        let (buffer, desc) = (&face.instance.buffer.lock().unwrap(), &face.desc);
        let (view, sampler) = desc.textureview_and_sampler(handler.device(), handler.queue());
        crate::create_bind_group(
            handler.device(),
            layout,
            vec![
                desc.matrix_buffer(handler.device()).binding_resource(),
                desc.material_buffer(handler.device()).binding_resource(),
                BindingResource::TextureView(&view),
                BindingResource::Sampler(&sampler),
                buffer.boundary.binding_resource(),
                buffer.boundary_length.binding_resource(),
            ],
        )
    }
}

impl<'a> Rendered for RenderFace<'a> {
    #[inline(always)]
    fn get_id(&self) -> RenderID { self.instance.id }
    #[inline(always)]
    fn set_id(&mut self, handler: &mut ObjectsHandler) { handler.set_id(&mut self.instance.id) }

    #[inline(always)]
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let buffers = &self.instance.buffer.lock().unwrap().surface;
        (Arc::clone(&buffers.0), Some(Arc::clone(&buffers.1)))
    }
    #[inline(always)]
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(ficonfig::bind_group_layout(
            handler.device(),
            self.desc.texture.is_some(),
        ))
    }
    #[inline(always)]
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let bind_group = match self.desc.texture.is_some() {
            true => ficonfig::textured_bind_group(handler, layout, self),
            false => ficonfig::non_textured_bind_group(handler, layout, self),
        };
        Arc::new(bind_group)
    }
    #[inline(always)]
    fn pipeline(&self, handler: &DeviceHandler, layout: &PipelineLayout) -> Arc<RenderPipeline> {
        let vertex_shader = include_spirv!("shaders/polygon.vert.spv");
        let fragment_shader = match self.desc.texture.is_some() {
            true => include_spirv!("shaders/textured-face.frag.spv"),
            false => include_spirv!("shaders/face.frag.spv"),
        };
        self.desc
            .pipeline_with_shader(vertex_shader, fragment_shader, handler, layout)
    }
}

impl ShapeInstance {
    #[inline(always)]
    pub fn render_faces(&mut self) -> Vec<RenderFace> {
        let desc = &self.desc;
        self.faces
            .iter_mut()
            .map(move |instance| RenderFace { instance, desc })
            .collect()
    }

    #[inline(always)]
    pub fn descriptor(&self) -> &InstanceDescriptor { &self.desc }
    #[inline(always)]
    pub fn descriptor_mut(&mut self) -> &mut InstanceDescriptor { &mut self.desc }
}
