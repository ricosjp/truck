use crate::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LockResult, MutexGuard};

static MAXID: AtomicUsize = AtomicUsize::new(0);

impl RenderID {
    /// Generate the unique `RenderID`.
    #[inline(always)]
    pub fn gen() -> Self { RenderID(MAXID.fetch_add(1, Ordering::SeqCst)) }
}

impl DeviceHandler {
    /// constructor
    #[inline(always)]
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        config: Arc<Mutex<SurfaceConfiguration>>,
    ) -> DeviceHandler {
        DeviceHandler {
            device,
            queue,
            config,
        }
    }
    /// Returns the reference of the device.
    #[inline(always)]
    pub fn device(&self) -> &Arc<Device> { &self.device }
    /// Returns the reference of the queue.
    #[inline(always)]
    pub fn queue(&self) -> &Arc<Queue> { &self.queue }
    /// Returns the copy of surface configuration.
    #[inline(always)]
    pub fn config(&self) -> SurfaceConfiguration { self.config.lock().unwrap().clone() }
    /// Locks the surface configuration.    
    #[inline(always)]
    pub fn lock_config(&self) -> LockResult<MutexGuard<SurfaceConfiguration>> { self.config.lock() }
}

impl Default for SceneDescriptor {
    #[inline(always)]
    fn default() -> SceneDescriptor {
        SceneDescriptor {
            background: Color::BLACK,
            camera: Camera::default(),
            lights: vec![Light::default()],
            sample_count: 1,
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
    pub fn camera_buffer(&self, handler: &DeviceHandler) -> BufferHandler {
        let config = handler.config();
        let as_rat = config.width as f64 / config.height as f64;
        self.camera.buffer(as_rat, handler.device())
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
        let light_vec: Vec<_> = self.lights.iter().map(Light::light_info).collect();
        BufferHandler::from_slice(&light_vec, device, BufferUsages::STORAGE)
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
                ty: BufferBindingType::Storage { read_only: true },
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

    #[inline(always)]
    fn sampling_buffer(
        device: &Device,
        config: &SurfaceConfiguration,
        sample_count: u32,
    ) -> Texture {
        device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format: config.format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: None,
        })
    }

    #[inline(always)]
    fn depth_texture(device: &Device, config: &SurfaceConfiguration, sample_count: u32) -> Texture {
        device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: config.width,
                height: config.height,
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

    #[inline(always)]
    fn update_textures(&mut self) {
        let config = self.config();
        let sample_count = self.scene_desc.sample_count;
        if self.depth_texture_size != (config.width, config.height)
            || sample_count != self.previous_sample_count
        {
            self.depth_texture_size = (config.width, config.height);
            self.previous_sample_count = sample_count;
            self.foward_depth = Self::depth_texture(self.device(), &config, sample_count);
            self.sampling_buffer = Self::sampling_buffer(self.device(), &config, sample_count);
        }
    }

    /// constructor
    // About `scene_desc`, entity is better than reference for the performance.
    // This is referece because only for as wgpu is.
    #[inline(always)]
    pub fn new(device_handler: DeviceHandler, scene_desc: &SceneDescriptor) -> Scene {
        let (device, config) = (device_handler.device(), device_handler.config());
        let bind_group_layout = Self::init_scene_bind_group_layout(device);
        Scene {
            objects: Default::default(),
            bind_group_layout,
            foward_depth: Self::depth_texture(device, &config, scene_desc.sample_count),
            depth_texture_size: (config.width, config.height),
            sampling_buffer: Self::sampling_buffer(device, &config, scene_desc.sample_count),
            previous_sample_count: scene_desc.sample_count,
            clock: std::time::Instant::now(),
            scene_desc: scene_desc.clone(),
            device_handler,
        }
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

    /// Returns the copy of swap chain descriptor.
    #[inline(always)]
    pub fn config(&self) -> SurfaceConfiguration { self.device_handler.config() }
    /// Locks the swap chain descriptor.
    #[inline(always)]
    pub fn lock_sc_desc(&self) -> LockResult<MutexGuard<SurfaceConfiguration>> {
        self.device_handler.lock_config()
    }
    /// Returns the elapsed time since the scene was created.
    #[inline(always)]
    pub fn elapsed(&self) -> std::time::Duration { self.clock.elapsed() }

    /// Returns the reference of the descriptor.
    #[inline(always)]
    pub fn descriptor(&self) -> &SceneDescriptor { &self.scene_desc }

    /// Returns the mutable reference of the descriptor.
    #[inline(always)]
    pub fn descriptor_mut(&mut self) -> &mut SceneDescriptor { &mut self.scene_desc }
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
    pub fn camera_buffer(&self) -> BufferHandler {
        self.scene_desc.camera_buffer(self.device_handler())
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
    ///     float time;     // elapsed time since the scene was created.
    ///     uint nlights;   // the number of lights
    /// };
    /// ```
    #[inline(always)]
    pub fn scene_status_buffer(&self) -> BufferHandler {
        let scene_info = SceneInfo {
            time: self.elapsed().as_secs_f32(),
            num_of_lights: self.scene_desc.lights.len() as u32,
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
                    object.pipeline(handler, &pipeline_layout, self.scene_desc.sample_count);
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
    ) -> RenderPassDepthStencilAttachment {
        RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: Some(Operations {
                load: LoadOp::Clear(0),
                store: true,
            }),
        }
    }

    /// Renders the scene to `view`.
    pub fn render_scene(&mut self, view: &TextureView) {
        self.update_textures();
        let bind_group = self.scene_bind_group();
        let depth_view = self.foward_depth.create_view(&Default::default());
        let sampled_view = self.sampling_buffer.create_view(&Default::default());
        let mut encoder = self
            .device()
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let (attachment, resolve_target) = match self.scene_desc.sample_count != 1 {
                true => (&sampled_view, Some(view)),
                false => (view, None),
            };
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachment {
                    view: attachment,
                    resolve_target,
                    ops: Operations {
                        load: LoadOp::Clear(self.scene_desc.background),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(Self::depth_stencil_attachment_descriptor(
                    &depth_view,
                )),
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
}
