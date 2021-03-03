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
use truck_platform::{wgpu::*, *};

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

/// Configures of `WireFrameInstance`.
#[derive(Clone, Debug)]
pub struct WireFrameState {
    /// instance matrix
    pub matrix: Matrix4,
    /// color of instance
    pub color: Vector4,
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

/// Configures of wire frame instance
#[derive(Clone, Debug)]
pub struct WireFrameInstanceDescriptor {
    /// configure of wire frame
    pub wireframe_state: WireFrameState,
    /// precision for polyline
    pub polyline_precision: f64,
}

#[derive(Debug)]
struct PolygonShaders {
    vertex: ShaderModule,
    fragment: ShaderModule,
    tex_fragment: ShaderModule,
}

#[derive(Debug)]
struct ShapeShaders {
    vertex: ShaderModule,
    fragment: ShaderModule,
    tex_fragment: ShaderModule,
}

#[derive(Debug)]
struct WireShaders {
    vertex: ShaderModule,
    fragment: ShaderModule,
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
    shaders: Arc<PolygonShaders>,
    id: RenderID,
}

/// Wire frame rendering
#[derive(Debug)]
pub struct WireFrameInstance {
    vertices: Arc<BufferHandler>,
    strips: Arc<BufferHandler>,
    state: WireFrameState,
    shaders: Arc<WireShaders>,
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
    polygon: (Arc<BufferHandler>, Arc<BufferHandler>),
    boundary: Arc<BufferHandler>,
    state: InstanceState,
    shaders: Arc<ShapeShaders>,
    id: RenderID,
}

/// Constroctor for instances
#[derive(Debug, Clone)]
pub struct InstanceCreator {
    handler: DeviceHandler,
    polygon_shaders: Arc<PolygonShaders>,
    shape_shaders: Arc<ShapeShaders>,
    wire_shaders: Arc<WireShaders>,
}

/// for creating `InstanceCreator`
pub trait CreatorCreator {
    /// create `InstanceCreator`
    fn instance_creator(&self) -> InstanceCreator;
}

/// The trait for generating `PolygonInstance` from `PolygonMesh` and `StructuredMesh`.
pub trait Polygon {
    /// Creates buffer handlers of attributes and indices.
    fn buffers(
        &self,
        vertex_usage: BufferUsage,
        index_usage: BufferUsage,
        device: &Device,
    ) -> (BufferHandler, BufferHandler);
    #[doc(hidden)]
    fn into_instance(
        &self,
        creator: &InstanceCreator,
        desc: &PolygonInstanceDescriptor,
    ) -> PolygonInstance;
    #[doc(hidden)]
    fn into_wire_frame(
        &self,
        creator: &InstanceCreator,
        state: &WireFrameInstanceDescriptor,
    ) -> WireFrameInstance;
}

/// The trait for generating `ShapeInstance` from `Shell` and `Solid`.
pub trait Shape {
    #[doc(hidden)]
    fn try_into_instance(
        &self,
        creator: &InstanceCreator,
        desc: &ShapeInstanceDescriptor,
    ) -> Option<ShapeInstance>;
    #[doc(hidden)]
    fn into_instance(
        &self,
        creator: &InstanceCreator,
        desc: &ShapeInstanceDescriptor,
    ) -> ShapeInstance;
    #[doc(hidden)]
    fn into_wire_frame(
        &self,
        creator: &InstanceCreator,
        state: &WireFrameInstanceDescriptor,
    ) -> WireFrameInstance;
}

#[derive(Debug, Clone)]
struct ExpandedPolygon<V> {
    vertices: Vec<V>,
    indices: Vec<u32>,
}

mod expanded;
/// utility for creating `Texture`
pub mod image2texture;
mod instance_creator;
mod instance_descriptor;
mod polyrend;
mod shaperend;
mod wireframe;
