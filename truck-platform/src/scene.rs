use crate::*;

impl DeviceHandler {
    #[inline(always)]
    pub fn device(&self) -> &Device { &self.device }
    #[inline(always)]
    pub fn queue(&self) -> &Queue { &self.queue }
    #[inline(always)]
    pub fn sc_desc(&self) -> SwapChainDescriptor { self.sc_desc.lock().unwrap().clone() }
}

impl Default for SceneDescriptor {
    fn default() -> SceneDescriptor {
        SceneDescriptor {
            back_ground: Color::BLACK,
            camera: Camera::default(),
            lights: vec![Light::default()],
        }
    }
}

impl ObjectsHandler {
    #[inline(always)]
    fn is_include<R: Rendered>(&self, object: &R) -> bool {
        match object.get_id().map(|idx| self.objects.get(&idx)) {
            Some(Some(_)) => true,
            _ => false,
        }
    }
    #[inline(always)]
    fn add_object<R: Rendered>(&mut self, object: &mut R, robject: RenderObject) {
        let idx = self.objects_number;
        self.objects.insert(idx, robject);
        object.set_id(idx);
        self.objects_number += 1;
    }
    #[inline(always)]
    fn remove_object<R: Rendered>(&mut self, object: &R) -> bool {
        match object.get_id().map(|idx| self.objects.remove(&idx)) {
            Some(None) => true,
            _ => false,
        }
    }
    #[inline(always)]
    fn update_vertex_buffer<R: Rendered>(&mut self, object: &R, handler: &DeviceHandler) -> bool {
        match object.get_id().map(|idx| self.objects.get_mut(&idx)) {
            Some(Some(render_object)) => {
                let (vb, ib) = object.vertex_buffer(handler);
                render_object.vertex_buffer = vb;
                render_object.index_buffer = ib;
                true
            }
            _ => false,
        }
    }
    #[inline(always)]
    fn update_bind_group<R: Rendered>(&mut self, object: &R, handler: &DeviceHandler) -> bool {
        match object.get_id().map(|idx| self.objects.get_mut(&idx)) {
            Some(Some(render_object)) => {
                let bind_group = object.bind_group(handler, &render_object.bind_group_layout);
                render_object.bind_group = bind_group;
                true
            }
            _ => false,
        }
    }
}

impl Scene {
    #[inline(always)]
    fn init_scene_bind_group_layout(device: &Device) -> BindGroupLayout {
        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // camera
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // light
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                // timer
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        };
        device.create_bind_group_layout(&descriptor)
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
    pub fn new(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        sc_desc: &Arc<Mutex<SwapChainDescriptor>>,
        scene_desc: SceneDescriptor,
    ) -> Scene
    {
        let device_handler = DeviceHandler {
            device: Arc::clone(device),
            queue: Arc::clone(queue),
            sc_desc: Arc::clone(sc_desc),
        };
        let objects_handler = ObjectsHandler {
            objects: Default::default(),
            objects_number: 0,
        };
        let depth_texture = Self::default_depth_texture(&device, &sc_desc.try_lock().unwrap());
        Scene {
            device_handler,
            objects_handler,
            bind_group_layout: Self::init_scene_bind_group_layout(device),
            bind_group: None,
            foward_depth: depth_texture.create_view(&Default::default()),
            clock: std::time::Instant::now(),
            scene_desc,
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
    pub fn add_object<R: Rendered>(&mut self, object: &mut R) -> bool {
        if self.objects_handler.is_include(object) {
            return false;
        }
        let render_object = object.render_object(self);
        self.objects_handler.add_object(object, render_object);
        true
    }
    #[inline(always)]
    pub fn add_objects<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a mut R>, {
        objects
            .into_iter()
            .fold(true, move |flag, object| flag && self.add_object(object))
    }
    #[inline(always)]
    pub fn remove_object<R: Rendered>(&mut self, object: &R) -> bool {
        self.objects_handler.remove_object(object)
    }
    #[inline(always)]
    pub fn remove_objects<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a mut R>, {
        objects.into_iter().all(|object| self.remove_object(object))
    }

    #[inline(always)]
    pub fn clear_objects(&mut self) { self.objects_handler.objects.clear() }
    #[inline(always)]
    pub fn number_of_objects(&self) -> usize { self.objects_handler.objects.len() }

    #[inline(always)]
    pub fn update_vertex_buffer<R: Rendered>(&mut self, object: &R) -> bool {
        let device_handler = &self.device_handler;
        self.objects_handler
            .update_vertex_buffer(object, &device_handler)
    }
    #[inline(always)]
    pub fn update_vertex_buffers<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        objects
            .into_iter()
            .all(move|object| self.update_vertex_buffer(object))
    }

    #[inline(always)]
    pub fn update_bind_group<R: Rendered>(&mut self, object: &R) -> bool {
        let device_handler = &self.device_handler;
        self.objects_handler
            .update_bind_group(object, &device_handler)
    }
    #[inline(always)]
    pub fn update_bind_groups<'a, R, I>(&mut self, objects: I) -> bool
    where
        R: 'a + Rendered,
        I: IntoIterator<Item = &'a R>, {
        objects
            .into_iter()
            .all(|object| self.update_bind_group(object))
    }

    #[inline(always)]
    pub fn elapsed(&self) -> f64 { self.clock.elapsed().as_secs_f64() }
    #[inline(always)]
    pub fn bind_group_layout(&self) -> &BindGroupLayout { &self.bind_group_layout }

    pub fn scene_status_buffer(&self) -> BufferHandler {
        let scene_info = SceneInfo {
            time: self.elapsed() as f32,
            num_of_lights: self.scene_desc.lights.len() as u32,
        };
        let buffer = self.device().create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[scene_info]),
            usage: BufferUsage::UNIFORM,
            label: None,
        });
        BufferHandler::new(buffer, std::mem::size_of::<f32>() as u64)
    }

    pub fn lights_buffer(&self) -> BufferHandler {
        let mut light_vec: Vec<LightInfo> = Vec::new();
        for light in &self.scene_desc.lights {
            light_vec.push(light.light_info());
        }
        let buffer = self.device().create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&light_vec),
            usage: BufferUsage::STORAGE,
            label: None,
        });
        BufferHandler::new(
            buffer,
            (light_vec.len() * std::mem::size_of::<LightInfo>()) as u64,
        )
    }

    pub fn render_scene(&self, view: &TextureView) {
        let mut encoder = self
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.scene_desc.back_ground),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(self.depth_stencil_attachment_descriptor()),
            });
            rpass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            for (_, object) in self.objects_handler.objects.iter() {
                rpass.set_pipeline(&object.pipeline);
                rpass.set_bind_group(1, &object.bind_group, &[]);
                rpass.set_vertex_buffer(0, object.vertex_buffer.buffer.slice(..));
                match object.index_buffer {
                    Some(ref index_buffer) => {
                        rpass.set_index_buffer(index_buffer.buffer.slice(..));
                        let index_size = index_buffer.size as u32;
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
