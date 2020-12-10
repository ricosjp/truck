use crate::*;
use std::sync::{LockResult, MutexGuard};

lazy_static::lazy_static! {
    static ref MAXID: Mutex<usize> = Mutex::new(0);
}

impl Default for RenderID {
    #[inline(always)]
    fn default() -> Self {
        let mut id = MAXID.lock().unwrap();
        *id += 1;
        RenderID(*id - 1)
    }
}

impl DeviceHandler {
    #[inline(always)]
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        sc_desc: Arc<Mutex<SwapChainDescriptor>>,
    ) -> DeviceHandler {
        DeviceHandler {
            device,
            queue,
            sc_desc,
        }
    }
    #[inline(always)]
    pub fn device(&self) -> &Device { &self.device }
    #[inline(always)]
    pub fn queue(&self) -> &Queue { &self.queue }
    #[inline(always)]
    pub fn sc_desc(&self) -> SwapChainDescriptor { self.sc_desc.lock().unwrap().clone() }
    #[inline(always)]
    pub fn lock_sc_desc(&self) -> LockResult<MutexGuard<SwapChainDescriptor>> {
        self.sc_desc.lock()
    }
}

impl Default for SceneDescriptor {
    fn default() -> SceneDescriptor {
        SceneDescriptor {
            background: Color::BLACK,
            camera: Camera::default(),
            lights: vec![Light::default()],
        }
    }
}

impl Scene {
    #[inline(always)]
    fn camera_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
            ty: BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn lights_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
            ty: BindingType::StorageBuffer {
                dynamic: false,
                min_binding_size: None,
                readonly: true,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn scene_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
            ty: BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    #[inline(always)]
    fn init_scene_bind_group_layout(device: &Device) -> BindGroupLayout {
        crate::create_bind_group_layout(
            device,
            &[
                Self::camera_bgl_entry(),
                Self::lights_bgl_entry(),
                Self::scene_bgl_entry(),
            ],
        )
    }

    #[inline(always)]
    pub fn update_scene_bind_group(&mut self) {
        let DeviceHandler {
            ref device,
            ref sc_desc,
            ..
        } = &self.device_handler;
        let sc_desc = sc_desc.try_lock().unwrap();
        let as_rat = sc_desc.width as f64 / sc_desc.height as f64;
        drop(sc_desc);
        let bind_group = crate::create_bind_group(
            device,
            &self.bind_group_layout,
            vec![
                self.scene_desc
                    .camera
                    .buffer(as_rat, device)
                    .binding_resource(),
                self.lights_buffer().binding_resource(),
                self.scene_status_buffer().binding_resource(),
            ],
        );
        self.bind_group = Some(bind_group);
    }

    #[inline(always)]
    fn default_depth_texture(device: &Device, sc_desc: &SwapChainDescriptor) -> Texture {
        device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        })
    }

    #[inline(always)]
    pub fn update_depth_texture(&mut self) {
        let depth_texture = Self::default_depth_texture(&self.device(), &self.sc_desc());
        self.foward_depth = depth_texture.create_view(&Default::default());
    }

    pub fn prepare_render(&mut self) {
        self.update_depth_texture();
        self.update_scene_bind_group();
    }

    #[inline(always)]
    pub fn new(device_handler: DeviceHandler, scene_desc: &SceneDescriptor) -> Scene {
        let (device, sc_desc) = (device_handler.device(), device_handler.sc_desc());
        let depth_texture = Self::default_depth_texture(device, &sc_desc);
        Scene {
            objects: Default::default(),
            bind_group_layout: Self::init_scene_bind_group_layout(device),
            bind_group: None,
            foward_depth: depth_texture.create_view(&Default::default()),
            clock: std::time::Instant::now(),
            scene_desc: scene_desc.clone(),
            device_handler,
        }
    }

    #[inline(always)]
    pub fn device_handler(&self) -> &DeviceHandler { &self.device_handler }

    #[inline(always)]
    pub fn device(&self) -> &Device { &self.device_handler.device }

    #[inline(always)]
    pub fn queue(&self) -> &Queue { &self.device_handler.queue }

    #[inline(always)]
    pub fn sc_desc(&self) -> SwapChainDescriptor { self.device_handler.sc_desc() }
    #[inline(always)]
    pub fn lock_sc_desc(&self) -> LockResult<MutexGuard<SwapChainDescriptor>> {
        self.device_handler.lock_sc_desc()
    }

    #[inline(always)]
    pub fn add_object<R: Rendered>(&mut self, object: &R) -> bool {
        let render_object = object.render_object(self);
        self.objects.insert(object.render_id(), render_object).is_none()
    }
    #[inline(always)]
    pub fn add_objects<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        objects
            .into_iter()
            .fold(true, move |flag, object| flag && self.add_object(object))
    }
    #[inline(always)]
    pub fn remove_object<R: Rendered>(&mut self, object: &R) -> bool {
        self.objects.remove(&object.render_id()).is_some()
    }
    #[inline(always)]
    pub fn remove_objects<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a mut R>, {
        objects.into_iter().all(|object| self.remove_object(object))
    }

    #[inline(always)]
    pub fn clear_objects(&mut self) { self.objects.clear() }
    #[inline(always)]
    pub fn number_of_objects(&self) -> usize { self.objects.len() }

    #[inline(always)]
    pub fn update_vertex_buffer<R: Rendered>(&mut self, object: &R) -> bool {
        let (handler, objects) = (&self.device_handler, &mut self.objects);
        match objects.get_mut(&object.render_id()) {
            Some(render_object) => {
                let (vb, ib) = object.vertex_buffer(handler);
                render_object.vertex_buffer = vb;
                render_object.index_buffer = ib;
                true
            }
            _ => false,
        }
    }
    
    #[inline(always)]
    pub fn update_vertex_buffers<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        let closure = move |flag, object: &R| flag && self.update_vertex_buffer(object);
        objects.into_iter().fold(true, closure)
    }

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
    #[inline(always)]
    pub fn update_bind_groups<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        let closure = move |flag, object: &R| flag && self.update_bind_group(object);
        objects.into_iter().fold(true, closure)
    }
    #[inline(always)]
    pub fn update_pipeline<R: Rendered>(&mut self, object: &R) -> bool {
        let (handler, objects) = (&self.device_handler, &mut self.objects);
        match objects.get_mut(&object.render_id()) {
            Some(render_object) => {
                let pipeline_layout =
                    handler
                        .device()
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            bind_group_layouts: &[
                                &self.bind_group_layout,
                                &render_object.bind_group_layout,
                            ],
                            push_constant_ranges: &[],
                            label: None,
                        });
                render_object.pipeline = object.pipeline(handler, &pipeline_layout);
                true
            }
            _ => false,
        }
    }
    #[inline(always)]
    pub fn update_pipelines<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        let closure = move |flag, object: &R| flag && self.update_pipeline(object);
        objects.into_iter().fold(true, closure)
    }

    #[inline(always)]
    pub fn elapsed(&self) -> std::time::Duration { self.clock.elapsed() }

    #[inline(always)]
    pub fn descriptor(&self) -> &SceneDescriptor { &self.scene_desc }

    #[inline(always)]
    pub fn descriptor_mut(&mut self) -> &mut SceneDescriptor { &mut self.scene_desc }

    #[inline(always)]
    pub fn bind_group_layout(&self) -> &BindGroupLayout { &self.bind_group_layout }

    pub fn scene_status_buffer(&self) -> BufferHandler {
        let scene_info = SceneInfo {
            time: self.elapsed().as_secs_f32(),
            num_of_lights: self.scene_desc.lights.len() as u32,
        };
        BufferHandler::from_slice(&[scene_info], self.device(), BufferUsage::UNIFORM)
    }

    pub fn lights_buffer(&self) -> BufferHandler {
        let (desc, device) = (&self.scene_desc, self.device());
        let light_vec: Vec<_> = desc.lights.iter().map(Light::light_info).collect();
        BufferHandler::from_slice(&light_vec, device, BufferUsage::STORAGE)
    }

    pub fn render_scene(&self, view: &TextureView) {
        let mut encoder = self
            .device()
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.scene_desc.background),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(self.depth_stencil_attachment_descriptor()),
            });
            rpass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            for (_, object) in self.objects.iter() {
                rpass.set_pipeline(&object.pipeline);
                rpass.set_bind_group(1, &object.bind_group, &[]);
                rpass.set_vertex_buffer(0, object.vertex_buffer.buffer.slice(..));
                match object.index_buffer {
                    Some(ref index_buffer) => {
                        rpass.set_index_buffer(index_buffer.buffer.slice(..));
                        let index_size =
                            index_buffer.size as u32 / std::mem::size_of::<u32>() as u32;
                        rpass.draw_indexed(0..index_size, 0, 0..1);
                    }
                    None => rpass.draw(0..object.vertex_buffer.size as u32, 0..1),
                }
            }
        }
        self.queue().submit(vec![encoder.finish()]);
    }

    pub fn depth_stencil_attachment_descriptor(
        &self,
    ) -> RenderPassDepthStencilAttachmentDescriptor {
        RenderPassDepthStencilAttachmentDescriptor {
            attachment: &self.foward_depth,
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
}
