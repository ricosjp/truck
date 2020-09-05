use crate::*;

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
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
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
        let sc_desc = self.sc_desc.try_lock().unwrap();
        let as_rat = sc_desc.width as f64 / sc_desc.height as f64;
        drop(sc_desc);
        let bind_group = buffer_handler::create_bind_group(
            &self.device,
            &self.bind_group_layout,
            &[
                self.camera.buffer(as_rat, &self.device),
                self.light.buffer(&self.device),
                self.timer_buffer(),
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
        let depth_texture = Self::default_depth_texture(&self.device, &self.sc_desc.try_lock().unwrap());
        self.foward_depth = depth_texture.create_view(&Default::default());
    }

    pub fn prepare_render(&mut self) {
        self.update_depth_texture();
        self.update_scene_bind_group();
    }

    #[inline(always)]
    pub fn new(device: &Arc<Device>, queue: &Arc<Queue>, sc_desc: &Arc<Mutex<SwapChainDescriptor>>) -> Scene {
        let depth_texture = Self::default_depth_texture(&device, &sc_desc.try_lock().unwrap());
        Scene {
            device: Arc::clone(device),
            queue: Arc::clone(queue),
            sc_desc: Arc::clone(sc_desc),
            objects: Default::default(),
            bind_group_layout: Self::init_scene_bind_group_layout(device),
            bind_group: None,
            foward_depth: depth_texture.create_view(&Default::default()),
            clock: std::time::Instant::now(),
            back_ground: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            camera: Default::default(),
            light: Default::default(),
        }
    }

    #[inline(always)]
    pub fn device(&self) -> &Device { &self.device }

    #[inline(always)]
    pub fn queue(&self) -> &Queue { &self.queue }

    #[inline(always)]
    pub fn add_object<R: Rendered>(&mut self, object: &R) -> usize {
        let object = object.render_object(&*self);
        self.objects.push(object);
        self.objects.len() - 1
    }
    #[inline(always)]
    pub fn remove_object(&mut self, idx: usize) { self.objects.remove(idx); }

    #[inline(always)]
    pub fn clear_objects(&mut self) { self.objects.clear() }
    #[inline(always)]
    pub fn number_of_objects(&self) -> usize { self.objects.len() }

    #[inline(always)]
    pub fn update_vertex_buffer<R: Rendered>(&mut self, rendered: &R, idx: usize) {
        let (vb, ib) = rendered.vertex_buffer(&self);
        self.objects[idx].vertex_buffer = vb;
        self.objects[idx].index_buffer = ib;
    }

    #[inline(always)]
    pub fn update_bind_group<R: Rendered>(&mut self, rendered: &R, idx: usize) {
        let bind_group = rendered.bind_group(&self, &self.objects[idx].bind_group_layout);
        self.objects[idx].bind_group = bind_group;
    }

    #[inline(always)]
    pub fn elapsed(&self) -> f64 { self.clock.elapsed().as_secs_f64() }
    #[inline(always)]
    pub fn bind_group_layout(&self) -> &BindGroupLayout { &self.bind_group_layout }

    pub fn timer_buffer(&self) -> BufferHandler {
        let buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[self.elapsed() as f32]),
            usage: BufferUsage::UNIFORM,
            label: None,
        });
        BufferHandler::new(buffer, std::mem::size_of::<f32>() as u64)
    }

    pub fn render_scene(&self, sc_texture: &SwapChainTexture) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &sc_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.back_ground),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(self.depth_stencil_attachment_descriptor()),
            });
            rpass.set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);
            for object in &self.objects {
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
        self.queue.submit(vec![encoder.finish()]);
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
