extern crate truck_modeling;
extern crate truck_platform;
extern crate truck_polymesh;
use bytemuck::{Pod, Zeroable};
use image::{DynamicImage, GenericImageView};
use std::sync::{Arc, Mutex};
use truck_platform::{
    wgpu::util::{BufferInitDescriptor, DeviceExt},
    wgpu::*,
    *,
};

pub mod modeling {
    pub use truck_modeling::*;
}
pub use modeling::*;

pub mod polymesh {
    pub use truck_polymesh::*;
}
pub use polymesh::*;

/// Material information.
/// 
/// Each instance is rendered based on the microfacet theory.
#[derive(Debug, Clone, Copy)]
pub struct Material {
    /// albedo, base color, [0, 1]-normalized rgba. Default is `Vector4::new(1.0, 1.0, 1.0, 1.0)`.  
    /// Transparent by alpha is not yet supported in the current standard shader.
    pub albedo: Vector4,
    /// roughness of the surface: [0, 1]. Default is 0.5.
    pub roughness: f64,
    /// ratio of specular: [0, 1]. Default is 0.25.
    pub reflectance: f64,
    /// ratio of ambient: [0, 1]. Default is 0.02.
    pub ambient_ratio: f64,
}

/// Configures of instances.
#[derive(Clone)]
pub struct InstanceDescriptor {
    /// instance matrix
    pub matrix: Matrix4,
    /// material of instance
    pub material: Material,
    /// texture of instance
    pub texture: Option<Arc<DynamicImage>>,
    /// If this parameter is true, the backface culling will be activated.
    pub backface_culling: bool,
}

/// Instance of polygon
/// 
/// One can duplicate polygons with different postures and materials
/// that have the same mesh data.
/// To save memory, mesh data on the GPU can be used again.
/// 
/// The duplicated polygon by `Clone::clone` has the same mesh data and descriptor
/// with original, however, its render id is different from the one of original.
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
