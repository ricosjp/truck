use super::*;

impl VertexCreator {
    #[inline(always)]
    fn bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                // uknot
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // vknot
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // uknot division
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // vknot division
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // control points
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // uderived control points
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // vderived control points
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: true,
                    },
                },
                // Surface info
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                // created vertex buffer
                BindGroupLayoutEntry {
                    binding: 8,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        readonly: false,
                    },
                },
            ],
            label: None,
        })
    }

    #[inline(always)]
    pub(super) fn new(device: &Device) -> Self {
        let bind_group_layout = Self::bind_group_layout(device);
        let compute_handler =
            ComputeHandler::new(device, bind_group_layout, include_str!("create_vertex.comp"));
        Self(compute_handler)
    }

    #[inline(always)]
    fn create_vec_buffer<T>(knot_vec: &T, device: &Device) -> BufferHandler
    where T: Clone + AsRef<[f64]> {
        let knot_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(knot_vec.as_ref()),
            BufferUsage::STORAGE_READ,
        );
        BufferHandler::new(knot_buffer, (knot_vec.as_ref().len() * F64_SIZE) as u64)
    }

    #[inline(always)]
    fn create_control_points_buffer(
        control_points: &Vec<Vec<Vector4>>,
        device: &Device,
    ) -> BufferHandler
    {
        let control_points: Vec<[f64; 4]> = control_points
            .iter()
            .flat_map(|vec| vec)
            .map(|pt| pt.clone().into())
            .collect();
        let ctrlpts_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&control_points),
            BufferUsage::STORAGE_READ,
        );
        BufferHandler::new(ctrlpts_buffer, (control_points.len() * F64_SIZE * 4) as u64)
    }

    fn create_vertex_buffer(
        udivision: &Vec<f64>,
        vdivision: &Vec<f64>,
        device: &Device,
    ) -> BufferHandler
    {
        let mesh_size = udivision.len() * vdivision.len() * F32_SIZE * 8;
        let vertex_storage = device.create_buffer(&BufferDescriptor {
            size: mesh_size as u64,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
            label: None,
        });
        BufferHandler::new(vertex_storage, mesh_size as u64)
    }

    fn create_bind_group(
        &self,
        surface: &BSplineSurface,
        tol: f64,
        device: &Device,
    ) -> (BindGroup, Buffer, [usize; 2])
    {
        let uder = surface.uderivation();
        let vder = surface.vderivation();
        let (udivision, vdivision) = create_space_division(surface, tol);
        let surface_info = [
            surface.uknot_vec().len() as u32,
            surface.vknot_vec().len() as u32,
            udivision.len() as u32,
            vdivision.len() as u32,
            surface.control_points().len() as u32,
            surface.control_points()[0].len() as u32,
        ];
        let surface_info_buffer = BufferHandler::new(
            device
                .create_buffer_with_data(bytemuck::cast_slice(&surface_info), BufferUsage::UNIFORM),
            (U32_SIZE * surface_info.len()) as u64,
        );
        let buffers = vec![
            Self::create_vec_buffer(surface.uknot_vec(), device),
            Self::create_vec_buffer(surface.vknot_vec(), device),
            Self::create_vec_buffer(&udivision, device),
            Self::create_vec_buffer(&vdivision, device),
            Self::create_control_points_buffer(surface.control_points(), device),
            Self::create_control_points_buffer(uder.control_points(), device),
            Self::create_control_points_buffer(vder.control_points(), device),
            surface_info_buffer,
            Self::create_vertex_buffer(&udivision, &vdivision, device),
        ];
        let bind_group =
            buffer_handler::create_bind_group(device, &self.0.bind_group_layout, &buffers);
        (
            bind_group,
            buffers.into_iter().skip(8).next().unwrap().buffer,
            [udivision.len(), vdivision.len()],
        )
    }

    pub(super) fn vertex_buffer(
        &self,
        device: &Device,
        queue: &Queue,
        surface: &BSplineSurface,
        tol: f64,
    ) -> (Buffer, [usize; 2])
    {
        let (bind_group, vertex_storage, div_lens) = self.create_bind_group(surface, tol, device);
        let vertex_buffer_size = div_lens[0] as u64 * div_lens[1] as u64 * F32_SIZE as u64 * 8;
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            size: vertex_buffer_size,
            usage: BufferUsage::VERTEX | BufferUsage::COPY_DST,
            label: None,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.0.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch(div_lens[0] as u32, div_lens[1] as u32, 1);
        }
        encoder.copy_buffer_to_buffer(
            &vertex_storage,
            0,
            &vertex_buffer,
            0,
            vertex_buffer_size as u64,
        );
        queue.submit(&[encoder.finish()]);
        (vertex_buffer, div_lens)
    }
}
