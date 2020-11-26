use crate::*;

fn bdle(idx: u32, readonly: bool) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding: idx,
        visibility: ShaderStage::COMPUTE,
        ty: BindingType::StorageBuffer {
            dynamic: false,
            min_binding_size: None,
            readonly,
        },
        count: None,
    }
}

fn nurbs_bglayout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            bdle(0, true), // control points
            bdle(1, true), // uknot_vec
            bdle(2, true), // vknot vec
            bdle(3, true), // udivision
            bdle(4, true), // vdivision
            BindGroupLayoutEntry {
                // nurbs information
                binding: 5,
                visibility: ShaderStage::COMPUTE,
                ty: BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            },
            bdle(6, false), // vertex buffer
            bdle(7, false), // index buffer
        ],
    })
}

fn control_points_buffer(device: &Device, surface: &NURBSSurface) -> BufferHandler {
    let vec: Vec<[f32; 4]> = surface
        .control_points()
        .iter()
        .flatten()
        .map(|pt| pt.cast().unwrap().into())
        .collect();
    BufferHandler::from_slice(&vec, device, BufferUsage::STORAGE)
}

fn floatvector_buffer(device: &Device, slice: &[f64]) -> BufferHandler {
    let vec: Vec<f32> = slice.iter().map(|t| *t as f32).collect();
    BufferHandler::from_slice(&vec, device, BufferUsage::STORAGE)
}

fn nurbs_information_buffer(
    device: &Device,
    surface: &NURBSSurface,
    param_division: &(Vec<f64>, Vec<f64>),
) -> BufferHandler
{
    let vec = vec![
        surface.control_points().len() as u32,
        surface.control_points()[0].len() as u32,
        surface.uknot_vec().len() as u32,
        surface.vknot_vec().len() as u32,
        param_division.0.len() as u32,
        param_division.1.len() as u32,
    ];
    BufferHandler::from_slice(&vec, device, BufferUsage::UNIFORM)
}

fn nurbs_bg(
    device: &Device,
    surface: &NURBSSurface,
    division: &(Vec<f64>, Vec<f64>),
    vertex_buffer: &BufferHandler,
    index_buffer: &BufferHandler,
) -> BindGroup
{
    truck_platform::create_bind_group(
        device,
        &nurbs_bglayout(device),
        vec![
            control_points_buffer(device, surface).binding_resource(),
            floatvector_buffer(device, surface.uknot_vec()).binding_resource(),
            floatvector_buffer(device, surface.vknot_vec()).binding_resource(),
            floatvector_buffer(device, &division.0).binding_resource(),
            floatvector_buffer(device, &division.1).binding_resource(),
            nurbs_information_buffer(device, surface, &division).binding_resource(),
            vertex_buffer.binding_resource(),
            index_buffer.binding_resource(),
        ],
    )
}

fn nurbs_pipeline(device: &Device, layout: &BindGroupLayout) -> ComputePipeline {
    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        bind_group_layouts: &[layout],
        push_constant_ranges: &[],
        label: None,
    });
    let compute_module = device.create_shader_module(include_spirv!("shaders/face.comp.spv"));
    device.create_compute_pipeline(&ComputePipelineDescriptor {
        layout: Some(&layout),
        compute_stage: ProgrammableStageDescriptor {
            module: &compute_module,
            entry_point: "main",
        },
        label: None,
    })
}

fn nurbs_buffer(
    device: &Device,
    queue: &Queue,
    surface: &NURBSSurface,
    tol: f64,
) -> (BufferHandler, BufferHandler)
{
    let division = surface.parameter_division(tol);
    let vb_size = (division.0.len() * division.1.len() * 32) as u64;
    let vertex_buffer0 = device.create_buffer(&BufferDescriptor {
        label: None,
        size: vb_size,
        usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
        mapped_at_creation: true,
    });
    let vertex_buffer0 = BufferHandler::new(vertex_buffer0, vb_size);
    let vertex_buffer1 = device.create_buffer(&BufferDescriptor {
        label: None,
        size: vb_size,
        usage: BufferUsage::VERTEX | BufferUsage::COPY_DST,
        mapped_at_creation: true,
    });
    let vertex_buffer1 = BufferHandler::new(vertex_buffer1, vb_size);
    let ib_size = ((division.0.len() - 1) * (division.1.len() - 1) * 4) as u64;
    let index_buffer0 = device.create_buffer(&BufferDescriptor {
        label: None,
        size: ib_size,
        usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
        mapped_at_creation: true,
    });
    let index_buffer0 = BufferHandler::new(index_buffer0, ib_size);
    let index_buffer1 = device.create_buffer(&BufferDescriptor {
        label: None,
        size: ib_size,
        usage: BufferUsage::INDEX | BufferUsage::COPY_DST,
        mapped_at_creation: true,
    });
    let index_buffer1 = BufferHandler::new(index_buffer1, ib_size);
    let bind_group_layout = nurbs_bglayout(device);
    let bind_group = nurbs_bg(device, surface, &division, &vertex_buffer0, &index_buffer0);
    let pipeline = nurbs_pipeline(device, &bind_group_layout);
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    let mut cpass = encoder.begin_compute_pass();
    cpass.set_bind_group(0, &bind_group, &[]);
    cpass.set_pipeline(&pipeline);
    cpass.dispatch(division.0.len() as u32, division.1.len() as u32, 1);
    drop(cpass);
    vertex_buffer0.copy_buffer(&mut encoder, &vertex_buffer1);
    index_buffer0.copy_buffer(&mut encoder, &index_buffer1);
    queue.submit(vec![encoder.finish()]);
    (vertex_buffer1, index_buffer1)
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

fn face_instance(
    device: &Device,
    queue: &Queue,
    face: &Face,
    desc: &Arc<Mutex<InstanceDescriptor>>,
) -> Option<FaceInstance>
{
    let surface = face.oriented_surface();
    let (vb, ib) = nurbs_buffer(device, queue, &surface, 0.01);
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
        id: None,
        desc: Arc::clone(desc),
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
                buffer.boundary.binding_resource(),
                buffer.boundary_length.binding_resource(),
                BindingResource::TextureView(&view),
                BindingResource::Sampler(&sampler),
            ],
        )
    }
}

impl<'a> Rendered for FaceInstance {
    #[inline(always)]
    fn get_id(&self) -> Option<usize> { self.id }
    #[inline(always)]
    fn set_id(&mut self, id: usize) { self.id = Some(id) }
    #[inline(always)]
    fn vertex_buffer(&self, _: &DeviceHandler) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let buffers = &self.surface;
        (Arc::clone(&buffers.0), Some(Arc::clone(&buffers.1)))
    }
    #[inline(always)]
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        let desc = &*self.desc.lock().unwrap();
        Arc::new(ficonfig::bind_group_layout(
            handler.device(),
            desc.texture.is_some(),
        ))
    }
    #[inline(always)]
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        let desc = &*self.desc.lock().unwrap();
        let bind_group = match desc.texture.is_some() {
            true => ficonfig::textured_bind_group(handler, layout, self),
            false => ficonfig::non_textured_bind_group(handler, layout, self),
        };
        Arc::new(bind_group)
    }
    #[inline(always)]
    fn pipeline(&self, handler: &DeviceHandler, layout: &PipelineLayout) -> Arc<RenderPipeline> {
        let desc = &*self.desc.lock().unwrap();
        let vertex_shader = include_spirv!("shaders/polygon.vert.spv");
        let fragment_shader = match desc.texture.is_some() {
            true => include_spirv!("shaders/textured-polygon.frag.spv"),
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
