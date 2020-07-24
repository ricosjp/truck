extern crate bytemuck;
extern crate wgpu;
extern crate glsl_to_spirv;
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
use wgpu::*;
use bytemuck::*;
pub use geometry::{Vector2, Vector3, Matrix3, Matrix4, vector, matrix};

#[derive(Debug, Clone, Copy)]
pub struct WGPUVertex {
    position: [f32; 3],
    uv_coord: [f32; 2],
    normal: [f32; 3],
}
unsafe impl Zeroable for WGPUVertex {}
unsafe impl Pod for WGPUVertex {}

#[derive(Clone, Copy, Debug)]
struct CameraInfo {
    camera_matrix: [[f32; 4]; 4],
    camera_projection: [[f32; 4]; 4],
}
unsafe impl Zeroable for CameraInfo {}
unsafe impl Pod for CameraInfo {}

#[derive(Clone, Copy, Debug)]
struct LightInfo {
    light_position: [f32; 3],
    light_strength: f32,
    light_type: i32,
}
unsafe impl Zeroable for LightInfo {}
unsafe impl Pod for LightInfo {}

#[derive(Clone, Copy, Debug)]
struct MaterialInfo {
    material: [f32; 4],
    reflect_ratio: [f32; 3],
}
unsafe impl Zeroable for MaterialInfo {}
unsafe impl Pod for MaterialInfo {}

#[derive(Debug, Clone)]
pub struct WGPUPolygonMesh {
    vertices: Vec<WGPUVertex>,
    indices: Vec<u16>,
    pub matrix: Matrix4,
    pub color: [f32; 3],
    pub reflect_ratio: [f32; 3],
}

#[derive(Debug)]
pub struct RenderObject {
    vertex_buffer: Buffer,
    vertex_size: usize,
    index_buffer: Buffer,
    index_size: usize,
    matrix: [[f32; 4]; 4],
    color: [f32; 3],
    reflect_ratio: [f32; 3],
    bind_group: Option<BindGroup>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    Perspective,
    Parallel,
}

#[derive(Debug, Clone)]
pub struct Camera {
    matrix: Matrix4,
    screen_size: f64,
    front_clipping_plane: f64,
    back_clipping_plane: f64,
    projection_type: ProjectionType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightType {
    Point,
    Uniform,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    pub position: Vector3,
    pub strength: f64,
    pub light_type: LightType,
}

#[derive(Debug)]
pub struct Scene {
    objects: Vec<RenderObject>,
    vertex_shader: ShaderModule,
    fragment_shader: ShaderModule,
    geometry_shader: Option<ShaderModule>,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    pipeline: Option<RenderPipeline>,
    foward_depth: TextureView,
    clock: std::time::Instant,
    pub camera: Camera,
    pub light: Light,
}

pub mod camera;
pub mod light;
pub mod wgpupolymesh;
pub mod scene;