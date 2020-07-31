extern crate bytemuck;
extern crate wgpu;
extern crate glsl_to_spirv;
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
use std::sync::Arc;
use wgpu::*;
use bytemuck::*;
pub use geometry::{Vector2, Vector3, Vector4, Matrix3, Matrix4, vector, matrix};
pub use polymesh::PolygonMesh;
pub type BSplineSurface = geometry::BSplineSurface<[f64; 4]>;

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
struct ObjectInfo {
    matrix: [[f32; 4]; 4],
    material: [f32; 4],
    reflect_ratio: [f32; 3],
}
unsafe impl Zeroable for ObjectInfo {}
unsafe impl Pod for ObjectInfo {}

#[derive(Debug, Clone)]
pub struct WGPUPolygonMesh {
    vertices: Vec<WGPUVertex>,
    indices: Vec<u32>,
}

#[derive(Debug)]
pub struct WGPUMesher {
    pub device: Arc<Device>,
    queue: Arc<Queue>,
    bind_group_layout: BindGroupLayout,
    pipeline: ComputePipeline,
}

#[derive(Debug)]
pub struct RenderObject {
    pub vertex_buffer: Arc<Buffer>,
    pub vertex_size: usize,
    index_buffer: Arc<Buffer>,
    index_size: usize,
    bind_group: Option<BindGroup>,
    pub matrix: Matrix4,
    pub color: Vector4,
    pub reflect_ratio: [f32; 3],
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
    near_clip: f64,
    far_clip: f64,
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
    device: Arc<Device>,
    queue: Arc<Queue>,
    objects: Vec<RenderObject>,
    bind_group_layout: BindGroupLayout,
    pipeline: RenderPipeline,
    foward_depth: TextureView,
    clock: std::time::Instant,
    pub camera: Camera,
    pub light: Light,
}

pub mod camera;
pub mod light;
pub mod scene;
pub mod render_object;
pub mod wgpupolymesh;
pub mod wgpumesher;