extern crate bytemuck;
extern crate cgmath;
extern crate futures;
extern crate image;
extern crate truck_geometry as geometry;
extern crate truck_polymesh as polymesh;
extern crate wgpu;
use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
pub use geometry::*;
pub use polymesh::PolygonMesh;
use std::sync::{Arc, Mutex};
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

#[derive(Clone)]
pub struct PolygonInstance {
    polygon: (Arc<BufferHandler>, Arc<BufferHandler>),
    pub matrix: Matrix4,
    pub color: ColorConfig,
    pub texture: Option<Arc<DynamicImage>>,
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
    fn pipeline(
        &self,
        device: &Device,
        sc_desc: &SwapChainDescriptor,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline>;
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
        let pipeline = self.pipeline(&scene.device, &scene.sc_desc.try_lock().unwrap(), &pipeline_layout);
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
    pub strength: f64,
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
    pub light: Light,
}

mod buffer_handler;
pub mod camera;
pub mod light;
//pub mod render_object;
pub mod scene;
//pub mod wgpumesher;
pub mod render_polygon;

fn create_bind_group<'a, T: IntoIterator<Item = BindingResource<'a>>> (
    device: &Device,
    layout: &BindGroupLayout,
    resources: T,
) -> BindGroup
{
    let entries: &Vec<BindGroupEntry> = &resources
        .into_iter()
        .enumerate()
        .map(|(i, resource)|
            BindGroupEntry {
                binding: i as u32,
                resource,
            }
        )
        .collect();
    device.create_bind_group(&BindGroupDescriptor {
        layout,
        entries,
        label: None,
    })
}
