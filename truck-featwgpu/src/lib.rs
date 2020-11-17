extern crate bytemuck;
extern crate cgmath;
extern crate futures;
extern crate image;
extern crate truck_modeling as modeling;
extern crate truck_polymesh as polymesh;
extern crate wgpu;
use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
pub use modeling::*;
pub use polymesh::PolygonMesh;
use std::sync::{Arc, Mutex};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}
unsafe impl Zeroable for AttrVertex {}
unsafe impl Pod for AttrVertex {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CameraInfo {
    camera_matrix: [[f32; 4]; 4],
    camera_projection: [[f32; 4]; 4],
}
unsafe impl Zeroable for CameraInfo {}
unsafe impl Pod for CameraInfo {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LightInfo {
    light_position: [f32; 4],
    light_color: [f32; 4],
    light_type: [u32; 4],
}
unsafe impl Zeroable for LightInfo {}
unsafe impl Pod for LightInfo {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct SceneInfo {
    time: f32,
    num_of_lights: u32,
}
unsafe impl Zeroable for SceneInfo {}
unsafe impl Pod for SceneInfo {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct SurfaceInfo {
    ctrl_row_size: u32,
    ctrl_column_size: u32,
    uknots_size: u32,
    vknots_size: u32,
    param_row_size: u32,
    param_column_size: u32,
    boundary_length: u32,
}
unsafe impl Zeroable for SurfaceInfo {}
unsafe impl Pod for SurfaceInfo {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ExpandedPolygon {
    vertices: Vec<AttrVertex>,
    indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub albedo: Vector4,
    pub roughness: f64,
    pub reflectance: f64,
}

#[derive(Clone)]
pub struct PolygonInstance {
    polygon: (Arc<BufferHandler>, Arc<BufferHandler>),
    pub matrix: Matrix4,
    pub material: Material,
    pub texture: Option<Arc<DynamicImage>>,
    pub bf_culling: bool,
}

#[derive(Clone)]
pub struct RenderFace {
    polygon: (Arc<BufferHandler>, Arc<BufferHandler>),
    boundary: Arc<BufferHandler>,
    boundary_length: Arc<BufferHandler>,
}

#[derive(Debug, Clone)]
pub struct RenderObject {
    vertex_buffer: Arc<BufferHandler>,
    index_buffer: Option<Arc<BufferHandler>>,
    pipeline: Arc<RenderPipeline>,
    bind_group_layout: Arc<BindGroupLayout>,
    bind_group: Arc<BindGroup>,
}

pub trait Rendered {
    fn vertex_buffer(&self, scene: &Scene) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>);
    fn bind_group_layout(&self, scene: &Scene) -> Arc<BindGroupLayout>;
    fn bind_group(&self, scene: &Scene, layout: &BindGroupLayout) -> Arc<BindGroup>;
    fn pipeline(&self, scene: &Scene, layout: &PipelineLayout) -> Arc<RenderPipeline>;
    fn render_object(&self, scene: &Scene) -> RenderObject {
        let (vertex_buffer, index_buffer) = self.vertex_buffer(scene);
        let bind_group_layout = self.bind_group_layout(scene);
        let bind_group = self.bind_group(scene, &bind_group_layout);
        let pipeline_layout = scene
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: &[&scene.bind_group_layout, &bind_group_layout],
                push_constant_ranges: &[],
                label: None,
            });
        let pipeline = self.pipeline(&scene, &pipeline_layout);
        RenderObject {
            vertex_buffer,
            index_buffer,
            bind_group_layout,
            bind_group,
            pipeline,
        }
    }
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
    pub color: Vector3,
    pub light_type: LightType,
}

#[derive(Debug)]
pub struct BufferHandler {
    pub buffer: Buffer,
    pub size: u64,
}

#[derive(Debug)]
pub struct Scene {
    device: Arc<Device>,
    queue: Arc<Queue>,
    sc_desc: Arc<Mutex<SwapChainDescriptor>>,
    objects: Vec<RenderObject>,
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    foward_depth: TextureView,
    clock: std::time::Instant,
    pub back_ground: Color,
    pub camera: Camera,
    pub lights: Vec<Light>,
}

mod buffer_handler;
pub mod camera;
pub mod light;
pub mod render_polygon;
pub mod render_shell;
pub mod scene;

fn create_bind_group<'a, T: IntoIterator<Item = BindingResource<'a>>>(
    device: &Device,
    layout: &BindGroupLayout,
    resources: T,
) -> BindGroup
{
    let entries: &Vec<BindGroupEntry> = &resources
        .into_iter()
        .enumerate()
        .map(|(i, resource)| BindGroupEntry {
            binding: i as u32,
            resource,
        })
        .collect();
    device.create_bind_group(&BindGroupDescriptor {
        layout,
        entries,
        label: None,
    })
}
