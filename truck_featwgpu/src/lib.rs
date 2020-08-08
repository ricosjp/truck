extern crate bytemuck;
extern crate futures;
extern crate glsl_to_spirv;
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
extern crate wgpu;
use bytemuck::{Pod, Zeroable};
pub use geometry::*;
pub use polymesh::PolygonMesh;
use std::sync::Arc;
use wgpu::*;
pub type BSplineSurface = geometry::BSplineSurface<Vector4>;

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
    light_color: [f32; 3],
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
pub struct RenderObject {
    pub vertex_buffer: Arc<Buffer>,
    pub vertex_size: usize,
    pub index_buffer: Arc<Buffer>,
    pub index_size: usize,
    bind_group: Option<BindGroup>,
    pub matrix: Matrix4,
    pub color: Vector4,
    pub reflect_ratio: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    Perspective(cgmath::PerspectiveFov<f64>),
    Parallel(cgmath::Ortho<f64>),
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub matrix: Matrix4,
    pub projection: Projection,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LightType {
    Point,
    Uniform,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    pub position: Point3,
    pub strength: f64,
    pub color: Vector3,
    pub light_type: LightType,
}

pub struct BufferHandler {
    pub buffer: Buffer,
    pub size: u64,
}

//#[derive(Debug)]
//pub struct WGPUMesher {
//    pub device: Arc<Device>,
//    queue: Arc<Queue>,
//    vertex_creator: wgpumesher::MeshCreator,
//}

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

mod buffer_handler;
pub mod camera;
pub mod light;
pub mod render_object;
pub mod scene;
//pub mod wgpumesher;
pub mod wgpupolymesh;
