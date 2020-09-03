extern crate bytemuck;
extern crate cgmath;
extern crate futures;
extern crate glsl_to_spirv;
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
extern crate wgpu;
use bytemuck::{Pod, Zeroable};
pub use geometry::*;
use glsl_to_spirv::ShaderType;
pub use polymesh::PolygonMesh;
use std::io::Read;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

pub type BSplineSurface = geometry::BSplineSurface<Vector4>;

#[derive(Debug, Clone, Copy)]
pub struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}
unsafe impl Zeroable for AttrVertex {}
unsafe impl Pod for AttrVertex {}

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

#[derive(Debug, Clone)]
pub struct ExpandedPolygon {
    vertices: Vec<AttrVertex>,
    indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct ColorConfig {
    pub ambient: Vector4,
    pub diffuse: Vector4,
    pub specular: Vector4,
    pub reflect_ratio: Vector3,
}

#[derive(Debug, Clone)]
pub struct PolygonInstance {
    pub polygon: Arc<ExpandedPolygon>,
    pub matrix: Matrix4,
    pub color: ColorConfig,
    pub texture: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct RenderObject {
    pub vertex_buffer: Arc<BufferHandler>,
    pub index_buffer: Option<Arc<BufferHandler>>,
    pub bind_group_layout: Arc<BindGroupLayout>,
    pub bind_group: Arc<BindGroup>,
    pub pipeline: Arc<RenderPipeline>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    Perspective,
    Parallel,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub matrix: Matrix4,
    projection: Matrix4,
    projection_type: ProjectionType,
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

#[derive(Debug)]
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
    bind_group: Option<BindGroup>,
    foward_depth: TextureView,
    clock: std::time::Instant,
    pub back_ground: Color,
    pub camera: Camera,
    pub light: Light,
}

mod buffer_handler;
pub mod camera;
pub mod light;
//pub mod render_object;
pub mod scene;
//pub mod wgpumesher;
pub mod render_polygon;

fn read_spirv(code: &str, shadertype: ShaderType, device: &Device) -> ShaderModule {
    let mut spirv = glsl_to_spirv::compile(code, shadertype).unwrap();
    let mut code = Vec::new();
    spirv.read_to_end(&mut code).unwrap();
    device.create_shader_module(wgpu::util::make_spirv(&code))
}
