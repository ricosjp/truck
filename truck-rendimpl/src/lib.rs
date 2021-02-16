//! Visualization of shape and polygon mesh based on platform

#![warn(
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

extern crate truck_modeling;
extern crate truck_platform;
extern crate truck_polymesh;
use bytemuck::{Pod, Zeroable};
use image::DynamicImage;
use std::sync::{Arc, Mutex};
use truck_platform::{
    wgpu::*,
    *,
};

/// Re-exports `truck_modeling`.
pub mod modeling {
    pub use truck_modeling::*;
}
pub use modeling::*;

/// Re-exports `truck_polymesh`.
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
#[derive(Clone, Debug)]
pub struct InstanceState {
    /// instance matrix
    pub matrix: Matrix4,
    /// material of instance
    pub material: Material,
    /// texture of instance
    pub texture: Option<Arc<Texture>>,
    /// If this parameter is true, the backface culling will be activated.
    pub backface_culling: bool,
}

/// Configures of polygon instance
#[derive(Clone, Debug, Default)]
pub struct PolygonInstanceDescriptor {
    /// configure of instance
    pub instance_state: InstanceState,    
}

/// Configures of shape instance
#[derive(Clone, Debug)]
pub struct ShapeInstanceDescriptor {
    /// configure of instance
    pub instance_state: InstanceState,
    /// precision for meshing
    pub mesh_precision: f64,
}

/// Instance of polygon
///
/// One can duplicate polygons with different postures and materials
/// that have the same mesh data.
/// To save memory, mesh data on the GPU can be used again.
///
/// The duplicated polygon by `Clone::clone` has the same mesh data and descriptor
/// with original, however, its render id is different from the one of original.
#[derive(Debug)]
pub struct PolygonInstance {
    polygon: Arc<Mutex<(Arc<BufferHandler>, Arc<BufferHandler>)>>,
    state: InstanceState,
    id: RenderID,
}

/// Wire frame rendering
#[derive(Debug, Clone)]
pub struct WireFrameInstance {
    vertices: Arc<BufferHandler>,
    strips: Arc<BufferHandler>,
    id: RenderID,
    /// instance matrix
    pub matrix: Matrix4,
    /// color of line
    pub color: Vector4,
}

#[derive(Clone, Debug)]
struct FaceBuffer {
    surface: (Arc<BufferHandler>, Arc<BufferHandler>),
    boundary: Arc<BufferHandler>,
    boundary_length: Arc<BufferHandler>,
}

#[derive(Debug)]
struct FaceInstance {
    buffer: Arc<Mutex<FaceBuffer>>,
    id: RenderID,
}

/// Instance of shape: `Shell` and `Solid` with geometric data.
///
/// One can duplicate shapes with different postures and materials
/// that have the same mesh data.
/// To save memory, mesh data on the GPU can be used again.
///
/// The duplicated shape by `Clone::clone` has the same mesh data and descriptor
/// with original, however, its render id is different from the one of original.
#[derive(Debug)]
pub struct ShapeInstance {
    faces: Vec<FaceInstance>,
    state: InstanceState,
}

/// Iterated face for rendering `ShapeInstance`.
#[derive(Clone, Copy, Debug)]
pub struct RenderFace<'a> {
    instance: &'a FaceInstance,
    state: &'a InstanceState,
}

/// The trait for generate `PolygonInstance` from `PolygonMesh` and `StructuredMesh`, and
/// `ShapeInstance` from `Shell` and `Solid`.
pub trait IntoInstance {
    /// the type of instance
    type Instance;
    /// instance descriptor
    type Descriptor;
    #[doc(hidden)]
    fn into_instance(&self, device: &Device, desc: &Self::Descriptor) -> Self::Instance;
}

/// Extend trait for `Scene` to create instance.
pub trait CreateInstance {
    /// Creates `PolygonInstance` from `PolygonMesh` and `StructuredMesh`, and
    /// `ShapeInstance` from `Shell` and `Solid`.
    fn create_instance<T: IntoInstance>(
        &self,
        object: &T,
        desc: &T::Descriptor,
    ) -> T::Instance;
}

impl CreateInstance for Scene {
    #[inline(always)]
    fn create_instance<T: IntoInstance>(
        &self,
        object: &T,
        desc: &T::Descriptor,
    ) -> T::Instance {
        object.into_instance(self.device(), desc.clone())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}

#[derive(Debug, Clone)]
struct ExpandedPolygon {
    vertices: Vec<AttrVertex>,
    indices: Vec<u32>,
}

mod instdesc;
mod polyrend;
mod shaperend;
mod wireframe;
/// utility for creating `Texture`
pub mod image2texture;
