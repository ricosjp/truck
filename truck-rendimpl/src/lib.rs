extern crate truck_modeling;
extern crate truck_platform;
extern crate truck_polymesh;
use image::{DynamicImage, GenericImageView};
use std::sync::{Arc, Mutex};
pub use truck_modeling::*;
use bytemuck::{Pod, Zeroable};
use truck_platform::{
    wgpu::util::{BufferInitDescriptor, DeviceExt},
    wgpu::*,
    *,
};

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub albedo: Vector4,
    pub roughness: f64,
    pub reflectance: f64,
    pub ambient_ratio: f64,
}

#[derive(Clone)]
pub struct InstanceDescriptor {
    pub matrix: Matrix4,
    pub material: Material,
    pub texture: Option<Arc<DynamicImage>>,
    pub backface_culling: bool,
}

pub struct PolygonInstance {
    polygon: Arc<Mutex<(Arc<BufferHandler>, Arc<BufferHandler>)>>,
    desc: InstanceDescriptor,
    id: RenderID,
}

#[derive(Clone)]
struct FaceBuffer {
    surface: (Arc<BufferHandler>, Arc<BufferHandler>),
    boundary: Arc<BufferHandler>,
    boundary_length: Arc<BufferHandler>,
}

pub struct FaceInstance {
    buffer: Arc<Mutex<FaceBuffer>>,
    id: RenderID,
}

#[derive(Clone)]
pub struct ShapeInstance {
    faces: Vec<FaceInstance>,
    desc: InstanceDescriptor,
}

pub trait IntoInstance {
    type Instance;
    #[doc(hidden)]
    fn into_instance(&self, device: &Device, desc: InstanceDescriptor) -> Self::Instance;
    #[doc(hidden)]
    fn update_instance(&self, device: &Device, instance: &mut Self::Instance);
}

pub trait CreateInstance {
    fn create_instance<T: IntoInstance>(
        &self,
        object: &T,
        desc: &InstanceDescriptor,
    ) -> T::Instance;
    fn update_instance<T: IntoInstance>(&self, instance: &mut T::Instance, object: &T);
}

impl CreateInstance for Scene {
    #[inline(always)]
    fn create_instance<T: IntoInstance>(
        &self,
        object: &T,
        desc: &InstanceDescriptor,
    ) -> T::Instance {
        object.into_instance(self.device(), desc.clone())
    }
    #[inline(always)]
    fn update_instance<T: IntoInstance>(&self, instance: &mut T::Instance, object: &T) {
        object.update_instance(self.device(), instance)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}

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
) -> BindGroup {
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
