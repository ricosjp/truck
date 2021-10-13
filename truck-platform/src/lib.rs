//! Graphic utility library based on wgpu.
//!
//! This crate is independent from other truck crates except `truck-base`.
//! It provides an API that allows users to handle drawing elements in a unified manner.
//! By implementing the [`Rendered`] trait, developers can define
//! their own rendering elements and have them rendered in [`Scene`]
//! in the same way as other rendering elements provided by truck.
//!
//! This documentation is intended to be read by two kinds of people: users and developers.
//! Users, those who just want to draw the shape of an existing mesh or boundary representation,
//! will only use:
//! - [`Scene`],
//! - [`SceneDescriptor`],
//! - [`DeviceHandler`],
//! - [`Camera`], and
//! - [`Light`].
//!
//! If you are a developer, who wants to try out new
//! visual representations, you can implement Rendered in your own structure and standardize it in
//! a form that can be used by users in [`Scene`].
//!
//! The sample code in this crate is for developers.
//! Users may wish to refer to the one in `truck-rendimpl`.
//!
//! [`Rendered`]: ./trait.Rendered.html
//! [`Scene`]: ./struct.Scene.html
//! [`DeviceHandler`]: ./struct.DeviceHandler.html
//! [`SceneDescriptor`]: ./struct.SceneDescriptor.html
//! [`Camera`]: ./struct.Camera.html
//! [`Light`]: ./struct.Light.html

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

extern crate bytemuck;
extern crate truck_base;
pub extern crate wgpu;
use bytemuck::{Pod, Zeroable};
use std::sync::{Arc, Mutex};
use truck_base::cgmath64::*;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

/// maximum number of light
pub const LIGHT_MAX: usize = 255;

#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
struct CameraInfo {
    camera_matrix: [[f32; 4]; 4],
    camera_projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
struct LightInfo {
    light_position: [f32; 4],
    light_color: [f32; 4],
    light_type: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
struct SceneInfo {
    background_color: [f32; 4],
    time: f32,
    num_of_lights: u32,
}

/// safe handler of GPU buffer
/// [`Buffer`](https://docs.rs/wgpu/0.10.1/wgpu/struct.Buffer.html)
#[derive(Debug)]
pub struct BufferHandler {
    buffer: Buffer,
    size: u64,
    stride: u64,
}

/// Utility for [`BindGroupLayoutEntry`]
///
/// The member variables of this struct are the ones of [`BindGroupLayoutEntry`]
/// with only `binding` removed. We can create `BindGroupLayout` by
/// giving its iterator to the function truck_platform::[`create_bind_group_layout`].
/// # Examples
/// ```
/// use std::sync::{Arc, Mutex};
/// use truck_platform::*;
/// use wgpu::*;
/// // let device: Device = ...
/// # let instance = Instance::new(Backends::PRIMARY);
/// # let (device, queue) = futures::executor::block_on(async {
/// #    let adapter = instance
/// #        .request_adapter(&RequestAdapterOptions {
/// #            power_preference: PowerPreference::HighPerformance,
/// #            compatible_surface: None,
/// #            force_fallback_adapter: false,            
/// #        })
/// #        .await
/// #        .unwrap();
/// #    adapter
/// #        .request_device(
/// #            &DeviceDescriptor {
/// #                features: Default::default(),
/// #                limits: Limits::default(),
/// #                label: None,
/// #            },
/// #            None,
/// #        )
/// #        .await
/// #        .unwrap()
/// # });
/// let entries = [
///     PreBindGroupLayoutEntry {
///         visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
///         ty: BindingType::Buffer {
///             ty: BufferBindingType::Uniform,
///             has_dynamic_offset: false,
///             min_binding_size: None,
///         },
///         count: None,
///     },
///     PreBindGroupLayoutEntry {
///         visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
///         ty: BindingType::Buffer {
///             ty: BufferBindingType::Uniform,
///             has_dynamic_offset: false,
///             min_binding_size: None,
///         },
///         count: None,
///     },
/// ];
/// let layout: BindGroupLayout = bind_group_util::create_bind_group_layout(&device, &entries);
/// ```
///
/// [`BindGroupLayoutEntry`]: https://docs.rs/wgpu/0.10.1/wgpu/struct.BindGroupLayoutEntry.html
#[doc(hidden)]
#[derive(Debug)]
pub struct PreBindGroupLayoutEntry {
    pub visibility: ShaderStages,
    pub ty: BindingType,
    pub count: Option<core::num::NonZeroU32>,
}

/// A collection of GPU buffers used by `wgpu` for rendering
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct RenderObject {
    vertex_buffer: Arc<BufferHandler>,
    index_buffer: Option<Arc<BufferHandler>>,
    pipeline: Arc<RenderPipeline>,
    bind_group_layout: Arc<BindGroupLayout>,
    bind_group: Arc<BindGroup>,
    visible: bool,
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
///
/// A [`Scene`](./struct.Scene.html) holds only one `Camera`.
#[derive(Debug, Clone)]
pub struct Camera {
    /// camera matrix
    ///
    /// This matrix must be in the Euclidean momentum group, the semi-direct product of O(3) and R^3.
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
///
/// There is no limit to the number of lights that can be added to a [`Scene`](./struct.Scene.html).
/// The information about the lights is sent to the shader as a storage buffer
/// (cf: [`Scene::lights_buffer()`](./struct.Scene.html#method.lights_buffer)).
#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    /// position of light
    pub position: Point3,
    /// [0, 1] range RGB color of light
    pub color: Vector3,
    /// type of light source: point or uniform
    pub light_type: LightType,
}

/// Chain that holds [`Device`], [`Queue`] and [`SurfaceConfiguration`].
///
/// This struct is used for creating [`Scene`].
/// [`Device`] and [`Queue`] must be wrapped `Arc`,
/// and [`SurfaceConfiguration`] `Arc<Mutex>`.
/// # Examples
/// ```
/// use std::sync::{Arc, Mutex};
/// use truck_platform::*;
/// use wgpu::*;
/// let instance = Instance::new(Backends::PRIMARY);
/// let (device, queue) = futures::executor::block_on(async {
///     let adapter = instance
///         .request_adapter(&RequestAdapterOptions {
///             power_preference: PowerPreference::HighPerformance,
///             compatible_surface: None,
///             force_fallback_adapter: false,
///         })
///         .await
///         .unwrap();
///     adapter
///         .request_device(
///             &DeviceDescriptor {
///                 features: Default::default(),
///                 limits: Limits::default(),
///                 label: None,
///             },
///             None,
///         )
///         .await
///         .unwrap()
/// });
/// let config = SurfaceConfiguration {
///     usage: TextureUsages::RENDER_ATTACHMENT,
///     format: TextureFormat::Bgra8UnormSrgb,
///     width: 512,
///     height: 512,
///     present_mode: PresentMode::Mailbox,
/// };
/// // creates SwapChain or Texture to draw by Scene.
/// let device_handler = DeviceHandler::new(
///     Arc::new(device),
///     Arc::new(queue),
///     Arc::new(Mutex::new(config)),
/// );
/// ```
///
/// [`Device`]: https://docs.rs/wgpu/0.10.1/wgpu/struct.Device.html
/// [`Queue`]: https://docs.rs/wgpu/0.10.1/wgpu/struct.Queue.html
/// [`SurfaceConfiguration`]: https://docs.rs/wgpu/0.10.1/wgpu/struct.SurfaceConfiguration.html
/// [`Scene`]: ./struct.Scene.html
#[derive(Debug, Clone)]
pub struct DeviceHandler {
    device: Arc<Device>,
    queue: Arc<Queue>,
    config: Arc<Mutex<SurfaceConfiguration>>,
}

/// The unique ID for `Rendered` struct.
///
/// This structure is not used explicitly by users for modeling by `truck-modeling` and `truck-rendimpl`.
/// If you want to define a new drawing element in which "Rendered" will be implemented, you need to add
/// this structure to the member variables of that drawing element.
///
/// This structure is assigned a unique value each time it is generated by `RenderID::gen()`.
/// This property allows us to map a `Rendred` entity to data in GPU memory held by `Scene`.
/// ```
/// use truck_platform::RenderID;
/// assert_ne!(RenderID::gen(), RenderID::gen());
/// ```
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct RenderID(usize);

/// Configures of [`Scene`](./struct.Scene.html).
#[derive(Debug, Clone)]
pub struct SceneDescriptor {
    /// background color. Default is `Color::BLACK`.
    pub background: Color,
    /// camera of the scene. Default is `Camera::default()`.
    pub camera: Camera,
    /// All lights in the scene. Default is `vec![Light::default()]`.
    pub lights: Vec<Light>,
    /// sample count for anti-aliasing by MSAA. 1, 2, 4, 8, or 16.
    pub sample_count: u32,
}

/// Wraps `wgpu` and provides an intuitive graphics API.
///
/// `Scene` is the most important in `truck-platform`.
/// This structure holds information about rendering and
/// serves as a bridge to the actual rendering of `Rendered` objects.
#[derive(Debug)]
pub struct Scene {
    device_handler: DeviceHandler,
    objects: SliceHashMap<RenderID, RenderObject>,
    bind_group_layout: BindGroupLayout,
    foward_depth: Texture,
    depth_texture_size: (u32, u32), // (width, height)
    sampling_buffer: Texture,
    previous_sample_count: u32,
    clock: instant::Instant,
    scene_desc: SceneDescriptor,
}

/// Rendered objects in the scene.
pub trait Rendered {
    /// Returns the render id.
    ///
    /// [`RenderID`](./struct.RenderID.html) is a key that maps `self` to a drawing element.
    /// Each object must have a RenderID to ensure that there are no duplicates.
    fn render_id(&self) -> RenderID;
    /// Creates the pair (vertex buffer, index buffer).
    fn vertex_buffer(
        &self,
        device_handler: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>);
    /// Creates the bind group layout.
    fn bind_group_layout(&self, device_handler: &DeviceHandler) -> Arc<BindGroupLayout>;
    /// Creates the bind group in `set = 1`.
    fn bind_group(
        &self,
        device_handler: &DeviceHandler,
        layout: &BindGroupLayout,
    ) -> Arc<BindGroup>;
    /// Creates the render pipeline.
    fn pipeline(
        &self,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline>;
    #[doc(hidden)]
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
        let pipeline = self.pipeline(
            scene.device_handler(),
            &pipeline_layout,
            scene.scene_desc.sample_count,
        );
        RenderObject {
            vertex_buffer,
            index_buffer,
            bind_group_layout,
            bind_group,
            pipeline,
            visible: true,
        }
    }
}

mod buffer_handler;
mod camera;
mod light;
#[doc(hidden)]
pub mod rendered_macros;
mod scene;
mod slice_hashmap;
use slice_hashmap::SliceHashMap;

#[doc(hidden)]
pub mod bind_group_util {
    use crate::*;
    #[doc(hidden)]
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

    #[doc(hidden)]
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
                ty: e.ty,
                count: e.count,
            })
            .collect();
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &vec,
        })
    }
}
