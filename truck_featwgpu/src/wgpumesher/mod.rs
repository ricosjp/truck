use crate::*;

const F64_SIZE: usize = std::mem::size_of::<f64>();
const F32_SIZE: usize = std::mem::size_of::<f32>();
const U32_SIZE: usize = std::mem::size_of::<u32>();

#[derive(Debug)]
struct ComputeHandler {
    bind_group_layout: BindGroupLayout,
    pipeline: ComputePipeline,
}

#[derive(Debug)]
pub(super) struct FarChecker(ComputeHandler);

#[derive(Debug)]
pub(super) struct VertexCreator(ComputeHandler);

#[derive(Debug)]
pub(super) struct IndexCreator(ComputeHandler);

impl ComputeHandler {
    fn new(
        device: &Device,
        bind_group_layout: BindGroupLayout,
        shader_source: &str,
    ) -> ComputeHandler
    {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        use glsl_to_spirv::ShaderType;
        let spirv = glsl_to_spirv::compile(shader_source, ShaderType::Compute).unwrap();
        let compute_shader = device.create_shader_module(&wgpu::read_spirv(spirv).unwrap());
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            layout: &pipeline_layout,
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &compute_shader,
                entry_point: "main",
            },
        });
        ComputeHandler {
            bind_group_layout,
            pipeline,
        }
    }
}

impl WGPUMesher {
    pub fn new() -> WGPUMesher {
        let (device, queue) = futures::executor::block_on(init_device());
        WGPUMesher::with_device(&Arc::new(device), &Arc::new(queue))
    }

    #[inline(always)]
    pub fn with_device(device: &Arc<Device>, queue: &Arc<Queue>) -> WGPUMesher {
        WGPUMesher {
            device: Arc::clone(device),
            queue: Arc::clone(queue),
            vertex_creator: VertexCreator::new(device),
            index_creator: IndexCreator::new(device),
        }
    }

    pub fn meshing(&self, surface: &BSplineSurface, tol: f64) -> RenderObject {
        let (device, queue) = (&self.device, &self.queue);
        let (vertex_buffer, div_lens) = self.vertex_creator.vertex_buffer(device, queue, surface, tol);
        let index_buffer = self.index_creator.index_buffer(device, queue, &div_lens);
        RenderObject {
            vertex_buffer: Arc::new(vertex_buffer),
            vertex_size: div_lens[0] * div_lens[1],
            index_buffer: Arc::new(index_buffer),
            index_size: (div_lens[0] - 1) * (div_lens[1] - 1) * 6,
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
    let pt00 = bspsurface(u0, v0);
    let pt01 = bspsurface(u0, v1);
    let pt10 = bspsurface(u1, v0);
    let pt11 = bspsurface(u1, v1);
    for i in 0..=degree0 {
        for j in 0..=degree1 {
            let p = (i as f64) / (degree0 as f64);
            let q = (j as f64) / (degree1 as f64);
            let u = u0 * p + u1 * (1.0 - p);
            let v = v0 * q + v1 * (1.0 - q);
            let val_mid = bspsurface(u, v);
            let par_mid = &pt00 * p * q
                + &pt01 * p * (1.0 - q)
                + &pt10 * (1.0 - p) * q
                + &pt11 * (1.0 - p) * (1.0 - q);
            let res = val_mid.rational_projection() - par_mid.rational_projection();
            if res.norm2() > tol * tol {
                return true;
            }
        }
    }
    false
}

mod create_vertex;
mod create_index;
mod is_far;

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

async fn init_device() -> (Device, Queue) {
    let adapter = Adapter::request(
        &RequestAdapterOptions {
            power_preference: PowerPreference::Default,
            compatible_surface: None,
        },
        BackendBit::PRIMARY,
    )
    .await
    .unwrap();

    adapter
        .request_device(&DeviceDescriptor {
            extensions: Extensions {
                anisotropic_filtering: false,
            },
            limits: Limits::default(),
        })
        .await
}
