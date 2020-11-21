extern crate truck_modeling;
extern crate truck_polymesh;
use image::{DynamicImage, GenericImageView};
use std::sync::Arc;
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
    id: Option<usize>,
}

#[derive(Clone)]
pub struct ShapeInstance {
    shape: Arc<Vec<(BufferHandler, BufferHandler)>>,
    desc: InstanceDescriptor,
}

pub trait IntoInstance {
    type Instance: Rendered;
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

pub use truck_polymesh::*;
pub mod polyrend;

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
