use crate::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use winit::window::Window;

static MAXID: AtomicUsize = AtomicUsize::new(0);

impl RenderID {
    /// Generate the unique `RenderID`.
    #[inline(always)]
    pub fn gen() -> Self { RenderID(MAXID.fetch_add(1, Ordering::SeqCst)) }
}

async fn init_default_device(
    window: Option<Arc<Window>>,
) -> (DeviceHandler, Option<WindowHandler>) {
    #[cfg(not(feature = "webgl"))]
    let instance = Instance::new(Backends::PRIMARY);
    #[cfg(feature = "webgl")]
    let instance = Instance::new(Backends::all());

    // trust winit
    #[allow(unsafe_code)]
    let surface = unsafe {
        let window = window.as_ref();
        window.map(|window| instance.create_surface(window.as_ref()))
    };

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            #[cfg(not(feature = "webgl"))]
            power_preference: PowerPreference::HighPerformance,
            #[cfg(feature = "webgl")]
            power_preference: PowerPreference::LowPower,
            compatible_surface: surface.as_ref(),
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find an appropriate adapter");

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                features: Default::default(),
                #[cfg(not(feature = "webgl"))]
                limits: Limits::downlevel_defaults().using_resolution(adapter.limits()),
                #[cfg(feature = "webgl")]
                limits: Limits::downlevel_webgl2_defaults(),
                label: None,
            },
            None,
        )
        .await
        .expect("Failed to create device");
    let device_handler = DeviceHandler {
        adapter: Arc::new(adapter),
        device: Arc::new(device),
        queue: Arc::new(queue),
    };
    let window_handler = window.map(|window| WindowHandler {
        window,
        surface: Arc::new(surface.unwrap()),
    });
    (device_handler, window_handler)
}

impl DeviceHandler {
    /// constructor
    #[inline(always)]
    pub fn new(adapter: Arc<Adapter>, device: Arc<Device>, queue: Arc<Queue>) -> DeviceHandler {
        DeviceHandler {
            adapter,
            device,
            queue,
        }
    }
    /// Returns the reference of the adapter.
    #[inline(always)]
    pub fn adapter(&self) -> &Arc<Adapter> { &self.adapter }
    /// Returns the reference of the device.
    #[inline(always)]
    pub fn device(&self) -> &Arc<Device> { &self.device }
    /// Returns the reference of the queue.
    #[inline(always)]
    pub fn queue(&self) -> &Arc<Queue> { &self.queue }

    /// Creates default device handler.
    pub async fn default_device() -> Self { init_default_device(None).await.0 }
}

impl Default for StudioConfig {
    #[inline(always)]
    fn default() -> StudioConfig {
        StudioConfig {
            background: Color::BLACK,
            camera: Camera::default(),
            lights: vec![Light::default()],
        }
    }
}

impl Default for BackendBufferConfig {
    #[inline(always)]
    fn default() -> BackendBufferConfig {
        BackendBufferConfig {
            depth_test: true,
            sample_count: 1,
        }
    }
}

impl Default for RenderTextureConfig {
    #[inline(always)]
    fn default() -> RenderTextureConfig {
        RenderTextureConfig {
            canvas_size: (1024, 768),
            format: TextureFormat::Rgba8Unorm,
        }
    }
}

impl RenderTextureConfig {
    /// Returns compatible `SurfaceConfiguation`.
    #[inline(always)]
    pub fn compatible_surface_config(self) -> SurfaceConfiguration {
        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
            width: self.canvas_size.0,
            height: self.canvas_size.1,
            present_mode: PresentMode::Fifo,
        }
    }
}

impl SceneDescriptor {
    /// Creates a `UNIFORM` buffer of camera.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// layout(set = 0, binding = 0) uniform Camera {
    ///     mat4 camera_matrix;     // the camera matrix
    ///     mat4 camera_projection; // the projection into the normalized view volume
    /// };
    /// ```
    #[inline(always)]
    pub fn camera_buffer(&self, device: &Device) -> BufferHandler {
        let (width, height) = self.render_texture.canvas_size;
        let as_rat = width as f64 / height as f64;
        self.studio.camera.buffer(as_rat, device)
    }

    /// Creates a `STORAGE` buffer of all lights.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// struct Light {
    ///     vec4 position;      // the position of light, position.w == 1.0
    ///     vec4 color;         // the color of light, color.w == 1.0
    ///     uvec4 light_type;   // Point => uvec4(0, 0, 0, 0), Uniform => uvec4(1, 0, 0, 0)
    /// };
    ///
    /// layout(set = 0, binding = 1) buffer Lights {
    ///     Light lights[];
    /// };
    /// ```
    #[inline(always)]
    pub fn lights_buffer(&self, device: &Device) -> BufferHandler {
        let mut light_vec: Vec<_> = self.studio.lights.iter().map(Light::light_info).collect();
        light_vec.resize(LIGHT_MAX, LightInfo::zeroed());
        BufferHandler::from_slice(&light_vec, device, BufferUsages::UNIFORM)
    }

    #[inline(always)]
    fn sampling_buffer(
        device: &Device,
        render_texture: RenderTextureConfig,
        sample_count: u32,
    ) -> Texture {
        device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: render_texture.canvas_size.0,
                height: render_texture.canvas_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format: render_texture.format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: None,
        })
    }

    #[inline(always)]
    fn depth_texture(device: &Device, size: (u32, u32), sample_count: u32) -> Texture {
        device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: None,
        })
    }

    fn backend_buffers(&self, device: &Device) -> (Option<Texture>, Option<Texture>) {
        let foward_depth = if self.backend_buffer.depth_test {
            Some(Self::depth_texture(
                device,
                self.render_texture.canvas_size,
                self.backend_buffer.sample_count,
            ))
        } else {
            None
        };
        let sampling_buffer = if self.backend_buffer.sample_count > 1 {
            Some(Self::sampling_buffer(
                device,
                self.render_texture,
                self.backend_buffer.sample_count,
            ))
        } else {
            None
        };
        (foward_depth, sampling_buffer)
    }
}

/// Mutable reference of `SceneDescriptor` in `Scene`.
///
/// When this struct is dropped, the backend buffers of scene will be updated.
#[derive(Debug)]
pub struct SceneDescriptorMut<'a>(&'a mut Scene);

impl<'a> std::ops::Deref for SceneDescriptorMut<'a> {
    type Target = SceneDescriptor;
    #[inline(always)]
    fn deref(&self) -> &SceneDescriptor { &self.0.scene_desc }
}

impl<'a> std::ops::DerefMut for SceneDescriptorMut<'a> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut SceneDescriptor { &mut self.0.scene_desc }
}

impl<'a> Drop for SceneDescriptorMut<'a> {
    fn drop(&mut self) {
        let (forward_depth, sampling_buffer) = self.backend_buffers(self.0.device());
        self.0.foward_depth = forward_depth;
        self.0.sampling_buffer = sampling_buffer;
    }
}

impl Scene {
    #[inline(always)]
    fn camera_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn lights_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn scene_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn init_scene_bind_group_layout(device: &Device) -> BindGroupLayout {
        bind_group_util::create_bind_group_layout(
            device,
            &[
                Self::camera_bgl_entry(),
                Self::lights_bgl_entry(),
                Self::scene_bgl_entry(),
            ],
        )
    }

    /// constructor
    // About `scene_desc`, entity is better than reference for the performance.
    // This is referece because only for as wgpu is.
    #[inline(always)]
    pub fn new(device_handler: DeviceHandler, scene_desc: &SceneDescriptor) -> Scene {
        let device = device_handler.device();
        let (foward_depth, sampling_buffer) = scene_desc.backend_buffers(device);
        let bind_group_layout = Self::init_scene_bind_group_layout(device);
        Scene {
            objects: Default::default(),
            bind_group_layout,
            foward_depth,
            sampling_buffer,
            clock: instant::Instant::now(),
            scene_desc: scene_desc.clone(),
            device_handler,
        }
    }

    /// Construct scene from default GPU device.
    /// # Arguments
    /// - `size`: (width, height)
    pub async fn from_default_device(scene_desc: &SceneDescriptor) -> Scene {
        Scene::new(DeviceHandler::default_device().await, scene_desc)
    }

    /// Creates compatible texture for render attachment.
    ///
    /// # Remarks
    /// The usage of texture is `TextureUsages::RENDER_ATTACHMENT | TetureUsages::COPY_SRC`.
    #[inline(always)]
    pub fn compatible_texture(&self) -> Texture {
        let config = self.scene_desc.render_texture;
        self.device().create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: config.canvas_size.0,
                height: config.canvas_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: config.format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        })
    }

    /// Returns the reference of its own `DeviceHandler`.
    #[inline(always)]
    pub fn device_handler(&self) -> &DeviceHandler { &self.device_handler }

    /// Returns the reference of the device.
    #[inline(always)]
    pub fn device(&self) -> &Arc<Device> { &self.device_handler.device }

    /// Returns the reference of the queue.
    #[inline(always)]
    pub fn queue(&self) -> &Arc<Queue> { &self.device_handler.queue }

    /// Returns the elapsed time since the scene was created.
    #[inline(always)]
    pub fn elapsed(&self) -> std::time::Duration { self.clock.elapsed() }

    /// Returns the reference of the descriptor.
    #[inline(always)]
    pub fn descriptor(&self) -> &SceneDescriptor { &self.scene_desc }

    /// Returns the mutable reference of the descriptor.
    ///
    /// # Remarks
    ///
    /// When the return value is dropped, the depth buffer and sampling buffer are automatically updated.
    /// Use `studio_config_mut` if you only want to update the colors of the camera, lights, and background.
    #[inline(always)]
    pub fn descriptor_mut(&mut self) -> SceneDescriptorMut<'_> { SceneDescriptorMut(self) }

    /// Returns the reference of the studio configuation.
    #[inline(always)]
    pub fn studio_config(&self) -> &StudioConfig { &self.scene_desc.studio }

    /// Returns the mutable reference of the studio configuation.
    #[inline(always)]
    pub fn studio_config_mut(&mut self) -> &mut StudioConfig { &mut self.scene_desc.studio }

    /// Returns the bind group layout in the scene.
    #[inline(always)]
    pub fn bind_group_layout(&self) -> &BindGroupLayout { &self.bind_group_layout }

    /// Creates a `UNIFORM` buffer of the camera.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// layout(set = 0, binding = 0) uniform Camera {
    ///     mat4 camera_matrix;     // the camera matrix
    ///     mat4 camera_projection; // the projection into the normalized view volume
    /// };
    /// ```
    #[inline(always)]
    pub fn camera_buffer(&self) -> BufferHandler { self.scene_desc.camera_buffer(self.device()) }

    /// Creates a `STORAGE` buffer of all lights.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// struct Light {
    ///     vec4 position;      // the position of light, position.w == 1.0
    ///     vec4 color;         // the color of light, color.w == 1.0
    ///     uvec4 light_type;   // Point => uvec4(0, 0, 0, 0), Uniform => uvec4(1, 0, 0, 0)
    /// };
    ///
    /// layout(set = 0, binding = 1) buffer Lights {
    ///     Light lights[]; // the number of lights must be gotten from another place
    /// };
    /// ```
    #[inline(always)]
    pub fn lights_buffer(&self) -> BufferHandler { self.scene_desc.lights_buffer(self.device()) }

    /// Creates a `UNIFORM` buffer of the scene status.
    ///
    /// The bind group provides [`Scene`] holds this uniform buffer.
    ///
    /// # Shader Example
    /// ```glsl
    /// layout(set = 0, binding = 2) uniform Scene {
    ///     vec4 bk_color;  // color of back ground
    ///     float time;     // elapsed time since the scene was created.
    ///     uint nlights;   // the number of lights
    /// };
    /// ```
    #[inline(always)]
    pub fn scene_status_buffer(&self) -> BufferHandler {
        let bk = self.scene_desc.studio.background;
        let size = self.scene_desc.render_texture.canvas_size;
        let scene_info = SceneInfo {
            background_color: [bk.r as f32, bk.g as f32, bk.b as f32, bk.a as f32],
            resolution: [size.0, size.1],
            time: self.elapsed().as_secs_f32(),
            num_of_lights: self.scene_desc.studio.lights.len() as u32,
        };
        BufferHandler::from_slice(&[scene_info], self.device(), BufferUsages::UNIFORM)
    }

    /// Creates bind group.
    /// # Shader Examples
    /// Suppose binded as `set = 0`.
    /// ```glsl
    /// layout(set = 0, binding = 0) uniform Camera {
    ///     mat4 camera_matrix;     // the camera matrix
    ///     mat4 camera_projection; // the projection into the normalized view volume
    /// };
    ///
    /// struct Light {
    ///     vec4 position;      // the position of light, position.w == 1.0
    ///     vec4 color;         // the color of light, color.w == 1.0
    ///     uvec4 light_type;   // Point => uvec4(0, 0, 0, 0), Uniform => uvec4(1, 0, 0, 0)
    /// };
    ///
    /// layout(set = 0, binding = 1) buffer Lights {
    ///     Light lights[];
    /// };
    ///
    /// layout(set = 0, binding = 2) uniform Scene {
    ///     float time;     // elapsed time since the scene was created.
    ///     uint nlights;   // the number of lights
    /// };
    /// ```
    #[inline(always)]
    pub fn scene_bind_group(&self) -> BindGroup {
        bind_group_util::create_bind_group(
            self.device(),
            &self.bind_group_layout,
            vec![
                self.camera_buffer().binding_resource(),
                self.lights_buffer().binding_resource(),
                self.scene_status_buffer().binding_resource(),
            ],
        )
    }

    /// Adds a render object to the scene.
    ///
    /// If there already exists a render object with the same ID,
    /// replaces the render object and returns false.
    #[inline(always)]
    pub fn add_object<R: Rendered>(&mut self, object: &R) -> bool {
        let render_object = object.render_object(self);
        self.objects
            .insert(object.render_id(), render_object)
            .is_none()
    }
    /// Sets the visibility of a render object.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns `false`.
    #[inline(always)]
    pub fn set_visibility<R: Rendered>(&mut self, object: &R, visible: bool) -> bool {
        self.objects
            .get_mut(&object.render_id())
            .map(|obj| obj.visible = visible)
            .is_some()
    }
    /// Adds render objects to the scene.
    ///
    /// If there already exists a render object with the same ID,
    /// replaces the render object and returns false.
    #[inline(always)]
    pub fn add_objects<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        let closure = move |flag, object| flag && self.add_object(object);
        objects.into_iter().fold(true, closure)
    }
    /// Removes a render object from the scene.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns `false`.
    #[inline(always)]
    pub fn remove_object<R: Rendered>(&mut self, object: &R) -> bool {
        self.objects.remove(&object.render_id()).is_some()
    }
    /// Removes render objects from the scene.
    ///
    /// If there exists a render object which does not exist in the scene, returns `false`.
    #[inline(always)]
    pub fn remove_objects<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        let closure = move |flag, object| flag && self.remove_object(object);
        objects.into_iter().fold(true, closure)
    }

    /// Removes all render objects from the scene.
    #[inline(always)]
    pub fn clear_objects(&mut self) { self.objects.clear() }

    /// Returns the number of the render objects in the scene.
    #[inline(always)]
    pub fn number_of_objects(&self) -> usize { self.objects.len() }

    /// Syncronizes the information of vertices of `object` in the CPU memory
    /// and that in the GPU memory.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns false.
    #[inline(always)]
    pub fn update_vertex_buffer<R: Rendered>(&mut self, object: &R) -> bool {
        let (handler, objects) = (&self.device_handler, &mut self.objects);
        match objects.get_mut(&object.render_id()) {
            None => false,
            Some(render_object) => {
                let (vb, ib) = object.vertex_buffer(handler);
                render_object.vertex_buffer = vb;
                render_object.index_buffer = ib;
                true
            }
        }
    }

    /// Syncronizes the information of vertices of `objects` in the CPU memory
    /// and that in the GPU memory.
    ///
    /// If there exists a render object which does not exist in the scene, returns false.
    #[inline(always)]
    pub fn update_vertex_buffers<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        let closure = move |flag, object: &R| flag && self.update_vertex_buffer(object);
        objects.into_iter().fold(true, closure)
    }

    /// Syncronizes the information of bind group of `object` in the CPU memory
    /// and that in the GPU memory.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns false.
    #[inline(always)]
    pub fn update_bind_group<R: Rendered>(&mut self, object: &R) -> bool {
        let (handler, objects) = (&self.device_handler, &mut self.objects);
        match objects.get_mut(&object.render_id()) {
            Some(render_object) => {
                let bind_group = object.bind_group(handler, &render_object.bind_group_layout);
                render_object.bind_group = bind_group;
                true
            }
            _ => false,
        }
    }
    /// Syncronizes the information of bind group of `object` in the CPU memory
    /// and that in the GPU memory.
    ///
    /// If there exists a render object which does not exist in the scene, returns false.
    #[inline(always)]
    pub fn update_bind_groups<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        let closure = move |flag, object: &R| flag && self.update_bind_group(object);
        objects.into_iter().fold(true, closure)
    }
    /// Syncronizes the information of pipeline of `object` in the CPU memory
    /// and that in the GPU memory.
    ///
    /// If there does not exist the render object in the scene, does nothing and returns false.
    #[inline(always)]
    pub fn update_pipeline<R: Rendered>(&mut self, object: &R) -> bool {
        let (handler, objects) = (&self.device_handler, &mut self.objects);
        match objects.get_mut(&object.render_id()) {
            Some(render_object) => {
                let device = handler.device();
                let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    bind_group_layouts: &[
                        &self.bind_group_layout,
                        &render_object.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                    label: None,
                });
                render_object.pipeline =
                    object.pipeline(handler, &pipeline_layout, &self.scene_desc);
                true
            }
            _ => false,
        }
    }
    /// Syncronizes the information of pipeline of `object` in the CPU memory
    /// and that in the GPU memory.
    ///
    /// If there exists a render object which does not exist in the scene, returns false.
    #[inline(always)]
    pub fn update_pipelines<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        let closure = move |flag, object: &R| flag && self.update_pipeline(object);
        objects.into_iter().fold(true, closure)
    }
    #[inline(always)]
    fn depth_stencil_attachment_descriptor(
        depth_view: &TextureView,
    ) -> RenderPassDepthStencilAttachment<'_> {
        RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }
    }

    /// Renders the scene to `view`.
    pub fn render(&self, view: &TextureView) {
        let bind_group = self.scene_bind_group();
        let depth_view = self
            .foward_depth
            .as_ref()
            .map(|tex| tex.create_view(&Default::default()));
        let sampled_view = self
            .sampling_buffer
            .as_ref()
            .map(|tex| tex.create_view(&Default::default()));
        let mut encoder = self
            .device()
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let (attachment, resolve_target) = match sampled_view.as_ref() {
                Some(sampled_view) => (sampled_view, Some(view)),
                None => (view, None),
            };
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: attachment,
                    resolve_target,
                    ops: Operations {
                        load: LoadOp::Clear(self.scene_desc.studio.background),
                        store: true,
                    },
                })],
                depth_stencil_attachment: depth_view
                    .as_ref()
                    .map(Self::depth_stencil_attachment_descriptor),
                ..Default::default()
            });
            rpass.set_bind_group(0, &bind_group, &[]);
            for (_, object) in &self.objects {
                if !object.visible {
                    continue;
                }
                rpass.set_pipeline(&object.pipeline);
                rpass.set_bind_group(1, &object.bind_group, &[]);
                rpass.set_vertex_buffer(0, object.vertex_buffer.buffer.slice(..));
                match object.index_buffer {
                    Some(ref index_buffer) => {
                        rpass.set_index_buffer(index_buffer.buffer.slice(..), IndexFormat::Uint32);
                        let index_size =
                            index_buffer.size as u32 / std::mem::size_of::<u32>() as u32;
                        rpass.draw_indexed(0..index_size, 0, 0..1);
                    }
                    None => rpass.draw(
                        0..(object.vertex_buffer.size / object.vertex_buffer.stride) as u32,
                        0..1,
                    ),
                }
            }
        }
        self.queue().submit(vec![encoder.finish()]);
    }

    /// Render image to buffer.
    pub async fn render_to_buffer(&self) -> Vec<u8> {
        let texture = self.compatible_texture();
        let view = texture.create_view(&Default::default());
        self.render(&view);
        let (device, queue) = (self.device(), self.queue());

        let (width, height) = self.scene_desc.render_texture.canvas_size;
        let size = (width * height * 4) as u64;
        let buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            size,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            ImageCopyBuffer {
                buffer: &buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: (width * 4).try_into().ok(),
                    rows_per_image: height.try_into().ok(),
                },
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(Some(encoder.finish()));
        let buffer_slice = buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(MapMode::Read, move |v| sender.send(v).unwrap());
        device.poll(Maintain::Wait);
        match receiver.receive().await {
            Some(Ok(_)) => buffer_slice.get_mapped_range().iter().copied().collect(),
            Some(Err(e)) => panic!("{}", e),
            None => panic!("Asynchronous processing fails"),
        }
    }
}

impl WindowScene {
    /// Initialize scene compatible with `window`.
    pub async fn from_window(window: Arc<Window>, scene_desc: &WindowSceneDescriptor) -> Self {
        let size = window.inner_size();
        let got = init_default_device(Some(window)).await;
        let (device_handler, window_handler) = (got.0, got.1.unwrap());
        let (device, surface) = (&device_handler.device, &window_handler.surface);
        let render_texture = RenderTextureConfig {
            canvas_size: size.into(),
            format: TextureFormat::Bgra8Unorm,
        };
        let config = render_texture.compatible_surface_config();
        surface.configure(device, &config);

        Self {
            scene: Scene::new(
                device_handler,
                &SceneDescriptor {
                    studio: scene_desc.studio.clone(),
                    backend_buffer: scene_desc.backend_buffer,
                    render_texture,
                },
            ),
            window_handler,
        }
    }
    /// Get the reference of initializing window.
    #[inline(always)]
    pub fn window(&self) -> &Arc<Window> { &self.window_handler.window }
    /// Get the reference of surface.
    #[inline(always)]
    pub fn surface(&self) -> &Arc<Surface> { &self.window_handler.surface }
    /// Adjusts the size of the backend buffers (depth or sampling buffer) to the size of the window.
    pub fn size_alignment(&mut self) {
        let size = self.window().inner_size();
        let canvas_size = self.scene.scene_desc.render_texture.canvas_size;
        if canvas_size != (size.width, size.height) {
            let mut desc = self.scene.descriptor_mut();
            desc.render_texture.canvas_size = size.into();
            let config = desc.render_texture.compatible_surface_config();
            drop(desc);
            self.surface().configure(self.device(), &config);
        }
    }
    /// Render scene to initializing window.
    pub fn render_frame(&mut self) {
        self.size_alignment();
        let surface = self.surface();
        let surface_texture = match surface.get_current_texture() {
            Ok(got) => got,
            Err(_) => {
                let config = self
                    .scene
                    .scene_desc
                    .render_texture
                    .compatible_surface_config();
                surface.configure(self.device(), &config);
                surface
                    .get_current_texture()
                    .expect("Failed to acquire next surface texture!")
            }
        };
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.render(&view);
        surface_texture.present();
    }
}
