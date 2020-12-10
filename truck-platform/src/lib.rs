pub extern crate bytemuck;
extern crate truck_base;
pub extern crate wgpu;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use truck_base::cgmath64::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

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

/// safe handler of GPU buffer
/// [`Buffer`](../wgpu/struct.Buffer.html)
#[derive(Debug)]
pub struct BufferHandler {
    buffer: Buffer,
    size: u64,
}

/// Utility for [`BindGroupLayoutEntry`](../wgpu/struct.BindGroupLayoutEntry.html)
#[derive(Debug)]
pub struct PreBindGroupLayoutEntry {
    pub visibility: ShaderStage,
    pub ty: BindingType,
    pub count: Option<core::num::NonZeroU32>,
}

/// A collection of GPU buffers used by [`wgpu`](../wgpu/index.html) for rendering
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct RenderObject {
    vertex_buffer: Arc<BufferHandler>,
    index_buffer: Option<Arc<BufferHandler>>,
    pipeline: Arc<RenderPipeline>,
    bind_group_layout: Arc<BindGroupLayout>,
    bind_group: Arc<BindGroup>,
}

/// the projection type of camera
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectionType {
    /// perspective camera
    Perspective,
    /// parallel camera
    Parallel,
}

/// Camera
#[derive(Debug, Clone)]
pub struct Camera {
    /// camera matrix
    pub matrix: Matrix4,
    projection: Matrix4,
    projection_type: ProjectionType,
}

/// the kinds of light sources: point or uniform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightType {
    /// point light source
    Point,
    /// uniform light source
    Uniform,
}

/// Light
#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    /// position of light
    pub position: Point3,
    /// color of light
    pub color: Vector3,
    /// type of light source: point or uniform
    pub light_type: LightType,
}

/// Chain that holds [`Device`], [`Queue`] and [`SwapChainDescriptor`].
/// 
/// [`Device`]: ../wgpu/struct.Device.html
/// [`Queue`]: ../wgpu/struct.Queue.html
/// [`SwapChainDescriptor`]: ../wgpu/struct.SwapChainDescriptor.html
#[derive(Debug, Clone)]
pub struct DeviceHandler {
    device: Arc<Device>,
    queue: Arc<Queue>,
    sc_desc: Arc<Mutex<SwapChainDescriptor>>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Hash, Debug)]
pub struct RenderID(Option<usize>);

#[derive(Debug)]
pub struct ObjectsHandler {
    objects: HashMap<usize, RenderObject>,
    objects_number: usize,
}

#[derive(Debug, Clone)]
pub struct SceneDescriptor {
    pub background: Color,
    pub camera: Camera,
    pub lights: Vec<Light>,
}

#[derive(Debug)]
pub struct Scene {
    device_handler: DeviceHandler,
    objects_handler: ObjectsHandler,
    bind_group_layout: BindGroupLayout,
    bind_group: Option<BindGroup>,
    foward_depth: TextureView,
    clock: std::time::Instant,
    scene_desc: SceneDescriptor,
}

pub trait Rendered {
    fn get_id(&self) -> RenderID;
    fn set_id(&mut self, objects_handler: &mut ObjectsHandler);
    fn vertex_buffer(
        &self,
        device_handler: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>);
    fn bind_group_layout(&self, device_handler: &DeviceHandler) -> Arc<BindGroupLayout>;
    fn bind_group(
        &self,
        device_handler: &DeviceHandler,
        layout: &BindGroupLayout,
    ) -> Arc<BindGroup>;
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline>;
    fn render_object(&self, scene: &Scene) -> RenderObject {
        let (vertex_buffer, index_buffer) = self.vertex_buffer(scene.device_handler());
        let bind_group_layout = self.bind_group_layout(scene.device_handler());
        let bind_group = self.bind_group(scene.device_handler(), &bind_group_layout);
        let pipeline_layout = scene
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                bind_group_layouts: &[&scene.bind_group_layout, &bind_group_layout],
                push_constant_ranges: &[],
                label: None,
            });
        let pipeline = self.pipeline(&scene.device_handler(), &pipeline_layout);
        RenderObject {
            vertex_buffer,
            index_buffer,
            bind_group_layout,
            bind_group,
            pipeline,
        }
    }
}

pub mod buffer_handler;
pub mod camera;
pub mod light;
pub mod rendered_macros;
pub mod scene;

pub fn create_bind_group<'a, T: IntoIterator<Item = BindingResource<'a>>>(
    device: &Device,
    layout: &BindGroupLayout,
    resources: T,
) -> BindGroup {
    let entries: &Vec<BindGroupEntry> = &resources
        .into_iter()
        .enumerate()
        .map(move |(i, resource)| BindGroupEntry {
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

pub fn create_bind_group_layout<'a, T: IntoIterator<Item = &'a PreBindGroupLayoutEntry>>(
    device: &Device,
    entries: T,
) -> BindGroupLayout {
    let vec: Vec<_> = entries
        .into_iter()
        .enumerate()
        .map(|(i, e)| BindGroupLayoutEntry {
            binding: i as u32,
            visibility: e.visibility,
            ty: e.ty.clone(),
            count: e.count,
        })
        .collect();
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &vec,
    })
}
