extern crate truck_modeling;
extern crate truck_polymesh;
extern crate truck_platform;
use image::{DynamicImage, GenericImageView};
use std::sync::{Arc, Mutex};
pub use truck_modeling::*;
use truck_platform::{
    bytemuck::*,
    wgpu::util::{BufferInitDescriptor, DeviceExt},
    wgpu::*,
    *,
};

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub albedo: Vector4,
    pub roughness: f64,
    pub reflectance: f64,
}

#[derive(Clone)]
pub struct InstanceDescriptor {
    pub matrix: Matrix4,
    pub material: Material,
    pub texture: Option<Arc<DynamicImage>>,
    pub backface_culling: bool,
}

#[derive(Clone)]
pub struct PolygonInstance {
    polygon: (Arc<BufferHandler>, Arc<BufferHandler>),
    desc: InstanceDescriptor,
    id: RenderID,
}

#[derive(Clone)]
pub struct FaceInstance {
    surface: (Arc<BufferHandler>, Arc<BufferHandler>),
    boundary: Arc<BufferHandler>,
    boundary_length: Arc<BufferHandler>,
    desc: Arc<Mutex<InstanceDescriptor>>,
    id: RenderID,
}

pub struct ShapeInstance {
    faces: Vec<FaceInstance>,
    desc: Arc<Mutex<InstanceDescriptor>>,
}

pub trait IntoInstance {
    type Instance;
    #[doc(hidden)]
    fn into_instance(
        &self,
        device: &Device,
        queue: &Queue,
        desc: InstanceDescriptor,
    ) -> Self::Instance;
}

pub trait CreateInstance {
    fn create_instance<T: IntoInstance>(
        &self,
        object: &T,
        desc: &InstanceDescriptor,
    ) -> T::Instance;
}

impl CreateInstance for Scene {
    fn create_instance<T: IntoInstance>(
        &self,
        object: &T,
        desc: &InstanceDescriptor,
    ) -> T::Instance
    {
        object.into_instance(self.device(), self.queue(), desc.clone())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}
unsafe impl Zeroable for AttrVertex {}
unsafe impl Pod for AttrVertex {}

#[repr(C)]
#[derive(Debug, Clone)]
struct ExpandedPolygon {
    vertices: Vec<AttrVertex>,
    indices: Vec<u32>,
}

pub use truck_polymesh::*;
pub mod instdesc;
pub mod polyrend;
pub mod shaperend;

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
