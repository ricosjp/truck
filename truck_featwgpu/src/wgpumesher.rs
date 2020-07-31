use crate::*;

const F64_SIZE: usize = std::mem::size_of::<f64>();
const F32_SIZE: usize = std::mem::size_of::<f32>();
const U32_SIZE: usize = std::mem::size_of::<u32>();

impl WGPUMesher {
    #[inline(always)]
    pub fn new(device: &Arc<Device>, queue: &Arc<Queue>) -> WGPUMesher {
        let bind_group_layout = Self::init_bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        WGPUMesher {
            device: Arc::clone(device),
            queue: Arc::clone(queue),
            bind_group_layout,
            pipeline: Self::init_pipeline(device, &pipeline_layout),
        }
    }

    #[inline(always)]
    fn init_pipeline(device: &Device, pipeline_layout: &PipelineLayout) -> ComputePipeline {
        use glsl_to_spirv::ShaderType;
        let spirv =
            glsl_to_spirv::compile(include_str!("surface_meshing.comp"), ShaderType::Compute)
                .unwrap();
        let compute_shader = device.create_shader_module(&wgpu::read_spirv(spirv).unwrap());
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: pipeline_layout,
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &compute_shader,
                entry_point: "main",
            },
        })
    }

    #[inline(always)]
    fn init_bind_group_layout(device: &Device) -> BindGroupLayout {
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

    fn create_vec_buffer<T>(&self, knot_vec: &T) -> (Buffer, u32)
    where T: Clone + Into<Vec<f64>> {
        let knot_vec: Vec<f64> = knot_vec.clone().into();
        let knot_buffer = self
            .device
            .create_buffer_with_data(bytemuck::cast_slice(&knot_vec), BufferUsage::STORAGE_READ);
        (knot_buffer, (knot_vec.len() * F64_SIZE) as u32)
    }

    fn create_control_points_buffer(&self, control_points: &Vec<Vec<Vector4>>) -> (Buffer, u32) {
        let control_points: Vec<[f64; 4]> = control_points
            .iter()
            .flat_map(|vec| vec)
            .map(|pt| pt.clone().into())
            .collect();
        let ctrlpts_buffer = self.device.create_buffer_with_data(
            bytemuck::cast_slice(&control_points),
            BufferUsage::STORAGE_READ,
        );
        (ctrlpts_buffer, (control_points.len() * F64_SIZE * 4) as u32)
    }

    fn create_vertex_buffer(&self, udivision: &Vec<f64>, vdivision: &Vec<f64>) -> (Buffer, u32) {
        let mesh_size = udivision.len() * vdivision.len() * F32_SIZE * 8;
        let vertex_storage = self.device.create_buffer(&BufferDescriptor {
            size: mesh_size as u64,
            usage: BufferUsage::all(),
            label: None,
        });
        (vertex_storage, mesh_size as u32)
    }

    fn create_bind_group(&self, surface: &BSplineSurface) -> (BindGroup, Buffer, [usize; 2]) {
        let device = &self.device;
        let uder = surface.uderivation();
        let vder = surface.vderivation();
        let (udivision, vdivision) = create_space_division(surface, 0.01);
        let (uknot_buffer, uknot_byte) = self.create_vec_buffer(surface.uknot_vec());
        let (vknot_buffer, vknot_byte) = self.create_vec_buffer(surface.vknot_vec());
        let (udiv_buffer, udiv_byte) = self.create_vec_buffer(&udivision);
        let (vdiv_buffer, vdiv_byte) = self.create_vec_buffer(&vdivision);
        let (ctrlpts_buffer, ctrlpts_byte) =
            self.create_control_points_buffer(surface.control_points());
        let (uder_ctrlpts_buffer, uder_ctrlpts_byte) =
            self.create_control_points_buffer(uder.control_points());
        let (vder_ctrlpts_buffer, vder_ctrlpts_byte) =
            self.create_control_points_buffer(vder.control_points());
        let surface_info = [
            surface.uknot_vec().len() as u32,
            surface.vknot_vec().len() as u32,
            udivision.len() as u32,
            vdivision.len() as u32,
            surface.control_points().len() as u32,
            surface.control_points()[0].len() as u32,
        ];
        let surface_info_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(&surface_info), BufferUsage::UNIFORM);
        let surface_info_length = U32_SIZE * surface_info.len();
        let (vertex_storage, vertex_size) = self.create_vertex_buffer(&udivision, &vdivision);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            bindings: &[
                // uknot
                Binding {
                    binding: 0,
                    resource: BindingResource::Buffer {
                        buffer: &uknot_buffer,
                        range: 0..uknot_byte as u64,
                    },
                },
                // vknot
                Binding {
                    binding: 1,
                    resource: BindingResource::Buffer {
                        buffer: &vknot_buffer,
                        range: 0..vknot_byte as u64,
                    },
                },
                // udiv buffer
                Binding {
                    binding: 2,
                    resource: BindingResource::Buffer {
                        buffer: &udiv_buffer,
                        range: 0..udiv_byte as u64,
                    },
                },
                // vdiv buffer
                Binding {
                    binding: 3,
                    resource: BindingResource::Buffer {
                        buffer: &vdiv_buffer,
                        range: 0..vdiv_byte as u64,
                    },
                },
                // control points
                Binding {
                    binding: 4,
                    resource: BindingResource::Buffer {
                        buffer: &ctrlpts_buffer,
                        range: 0..ctrlpts_byte as u64,
                    },
                },
                // uderived control points
                Binding {
                    binding: 5,
                    resource: BindingResource::Buffer {
                        buffer: &uder_ctrlpts_buffer,
                        range: 0..uder_ctrlpts_byte as u64,
                    },
                },
                // vderived control points
                Binding {
                    binding: 6,
                    resource: BindingResource::Buffer {
                        buffer: &vder_ctrlpts_buffer,
                        range: 0..vder_ctrlpts_byte as u64,
                    },
                },
                // surface info
                Binding {
                    binding: 7,
                    resource: BindingResource::Buffer {
                        buffer: &surface_info_buffer,
                        range: 0..surface_info_length as u64,
                    },
                },
                // vertex storage
                Binding {
                    binding: 8,
                    resource: BindingResource::Buffer {
                        buffer: &vertex_storage,
                        range: 0..vertex_size as u64,
                    },
                },
            ],
            label: None,
        });
        (
            bind_group,
            vertex_storage,
            [udivision.len(), vdivision.len()],
        )
    }

    pub fn meshing(&self, surface: &BSplineSurface) -> RenderObject {
        let device = &self.device;
        let (bind_group, vertex_storage, div_lens) = self.create_bind_group(surface);
        let [udiv_length, vdiv_length] = [div_lens[0] as u32, div_lens[1] as u32];
        let mut indices = Vec::new();
        for i in 0..(udiv_length - 1) {
            for j in 0..(vdiv_length - 1) {
                indices.push(i * vdiv_length + j);
                indices.push((i + 1) * vdiv_length + j);
                indices.push(i * vdiv_length + j + 1);
                indices.push(i * vdiv_length + j + 1);
                indices.push((i + 1) * vdiv_length + j);
                indices.push((i + 1) * vdiv_length + j + 1);
            }
        }
        let index_buffer =
            device.create_buffer_with_data(bytemuck::cast_slice(&indices), BufferUsage::INDEX);
        let vertex_buffer_size = udiv_length as u64 * vdiv_length as u64 * F32_SIZE as u64 * 8;
        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            size: vertex_buffer_size,
            usage: BufferUsage::VERTEX | BufferUsage::COPY_DST | BufferUsage::MAP_READ,
            label: None,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass();
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch(udiv_length as u32, vdiv_length as u32, 1);
        }
        encoder.copy_buffer_to_buffer(
            &vertex_storage,
            0,
            &vertex_buffer,
            0,
            vertex_buffer_size as u64,
        );
        self.queue.submit(&[encoder.finish()]);
        RenderObject {
            vertex_buffer: Arc::new(vertex_buffer),
            vertex_size: (udiv_length * vdiv_length) as usize,
            index_buffer: Arc::new(index_buffer),
            index_size: ((udiv_length - 1) * (vdiv_length - 1) * 6) as usize,
            matrix: Matrix4::identity(),
            color: Vector4::from([1.0; 4]),
            reflect_ratio: [0.2, 0.6, 0.2],
            bind_group: None,
        }
    }
}

fn is_far(bspsurface: &BSplineSurface, u0: f64, u1: f64, v0: f64, v1: f64, tol: f64) -> bool {
    let (mut degree0, mut degree1) = bspsurface.degrees();
    let bspsurface = bspsurface.get_closure();
    degree0 *= 2;
    degree1 *= 2;
    for i in 0..=degree0 {
        for j in 0..=degree1 {
            let p = (i as f64) / (degree0 as f64);
            let q = (j as f64) / (degree1 as f64);
            let u = u0 * p + u1 * (1.0 - p);
            let v = v0 * q + v1 * (1.0 - q);
            let val_mid = bspsurface(u, v);
            let par_mid = bspsurface(u0, v0) * p * q
                + bspsurface(u0, v1) * p * (1.0 - q)
                + bspsurface(u1, v0) * (1.0 - p) * q
                + bspsurface(u1, v1) * (1.0 - p) * (1.0 - q);
            let res = val_mid.rational_projection() - par_mid.rational_projection();
            if res.norm2() > tol * tol {
                return true;
            }
        }
    }
    false
}

fn sub_create_space_division(
    bspsurface: &BSplineSurface,
    tol: f64,
    mut div0: &mut Vec<f64>,
    mut div1: &mut Vec<f64>,
)
{
    let (mut degree0, mut degree1) = bspsurface.degrees();
    degree0 *= 2;
    degree1 *= 2;

    let mut divide_flag0 = vec![false; div0.len() - 1];
    let mut divide_flag1 = vec![false; div1.len() - 1];

    for i in 1..div0.len() {
        for j in 1..div1.len() {
            let far = is_far(bspsurface, div0[i - 1], div0[i], div1[j - 1], div1[j], tol);
            divide_flag0[i - 1] = divide_flag0[i - 1] || far;
            divide_flag1[j - 1] = divide_flag1[j - 1] || far;
        }
    }

    let mut new_div0 = vec![div0[0]];
    for i in 1..div0.len() {
        if divide_flag0[i - 1] {
            for j in 1..=degree0 {
                let p = (j as f64) / (degree0 as f64);
                new_div0.push(div0[i - 1] * (1.0 - p) + div0[i] * p);
            }
        } else {
            new_div0.push(div0[i]);
        }
    }

    let mut new_div1 = vec![div1[0]];
    for i in 1..div1.len() {
        if divide_flag1[i - 1] {
            for j in 1..=degree1 {
                let p = (j as f64) / (degree1 as f64);
                new_div1.push(div1[i - 1] * (1.0 - p) + div1[i] * p);
            }
        } else {
            new_div1.push(div1[i]);
        }
    }

    let divided0 = div0.len() != new_div0.len();
    let divided1 = div1.len() != new_div1.len();
    if divided0 {
        *div0 = new_div0;
    }
    if divided1 {
        *div1 = new_div1;
    }
    if divided0 || divided1 {
        sub_create_space_division(bspsurface, tol, &mut div0, &mut div1);
    }
}

fn create_space_division(bspsurface: &BSplineSurface, tol: f64) -> (Vec<f64>, Vec<f64>) {
    let (knot_vec0, knot_vec1) = bspsurface.knot_vecs();
    let u0 = knot_vec0[0];
    let u1 = knot_vec0[knot_vec0.len() - 1];
    let mut div0 = vec![u0, u1];
    let v0 = knot_vec1[0];
    let v1 = knot_vec1[knot_vec1.len() - 1];
    let mut div1 = vec![v0, v1];
    sub_create_space_division(bspsurface, tol, &mut div0, &mut div1);
    (div0, div1)
}
