use crate::*;
use glsl_to_spirv::ShaderType;

impl Scene {
    fn default_bind_group_layout(device: &Device) -> BindGroupLayout {
        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                // Camera
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                // Light
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                // Model Status
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
                // Scene Status
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::VERTEX | ShaderStage::FRAGMENT,
                    ty: BindingType::UniformBuffer { dynamic: false },
                },
            ],
        };
        device.create_bind_group_layout(&descriptor)
    }
    fn default_depth_texture(device: &Device, sc_desc: &SwapChainDescriptor) -> Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        })
    }
    pub fn update_depth_texture(&mut self, sc_desc: &SwapChainDescriptor) {
        let depth_texture = Self::default_depth_texture(&self.device, &sc_desc);
        self.foward_depth = depth_texture.create_default_view();
    }

    pub fn new(device: &Arc<Device>, queue: &Arc<Queue>, sc_desc: &SwapChainDescriptor) -> Scene {
        let vertex_shader = read_spirv(include_str!("vshader.vert"), ShaderType::Vertex, device);
        let fragment_shader =
            read_spirv(include_str!("fshader.frag"), ShaderType::Fragment, device);
        let bind_group_layout = Self::default_bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        let depth_texture = Self::default_depth_texture(&device, &sc_desc);
        Scene {
            device: Arc::clone(device),
            queue: Arc::clone(queue),
            objects: Default::default(),
            bind_group_layout,
            pipeline: Self::init_pipeline(
                &vertex_shader,
                &fragment_shader,
                &pipeline_layout,
                device,
                sc_desc,
            ),
            foward_depth: depth_texture.create_default_view(),
            clock: std::time::Instant::now(),
            camera: Default::default(),
            light: Default::default(),
        }
    }
    pub fn with_glsl_shader(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        sc_desc: &SwapChainDescriptor,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Scene
    {
        let vertex_shader = read_spirv(vertex_shader, ShaderType::Vertex, device);
        let fragment_shader = read_spirv(fragment_shader, ShaderType::Fragment, device);
        let bind_group_layout = Scene::default_bind_group_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });
        let depth_texture = Self::default_depth_texture(device, sc_desc);
        Scene {
            device: Arc::clone(device),
            queue: Arc::clone(queue),
            objects: Default::default(),
            bind_group_layout,
            pipeline: Self::init_pipeline(
                &vertex_shader,
                &fragment_shader,
                &pipeline_layout,
                device,
                sc_desc,
            ),
            foward_depth: depth_texture.create_default_view(),
            clock: std::time::Instant::now(),
            camera: Default::default(),
            light: Default::default(),
        }
    }

    #[inline(always)]
    pub fn add_polymesh<T: Into<WGPUPolygonMesh>>(&mut self, polymesh: T) -> usize {
        self.add_object(RenderObject::new(polymesh, &self.device))
    }

    #[inline(always)]
    pub fn add_object(&mut self, object: RenderObject) -> usize {
        self.objects.push(object);
        self.objects.len() - 1
    }

    #[inline(always)]
    pub fn get_object(&mut self, idx: usize) -> &RenderObject { &self.objects[idx] }
    #[inline(always)]
    pub fn get_object_mut(&mut self, idx: usize) -> &mut RenderObject { &mut self.objects[idx] }

    #[inline(always)]
    pub fn remove_object(&mut self, idx: usize) -> RenderObject { self.objects.remove(idx) }

    #[inline(always)]
    pub fn clear_objects(&mut self) { self.objects.clear() }

    #[inline(always)]
    pub fn objects(&self) -> &Vec<RenderObject> { &self.objects }

    #[inline(always)]
    pub fn objects_mut(&mut self) -> &mut Vec<RenderObject> { &mut self.objects }

    #[inline(always)]
    pub fn number_of_objects(&self) -> usize { self.objects.len() }

    #[inline(always)]
    pub fn elapsed(&self) -> f64 { self.clock.elapsed().as_secs_f64() }

    pub fn camera_projection(&self, as_rat: f64) -> [[f32; 4]; 4] {
        let mat = self.camera.projection() * Matrix4::diagonal(&vector!(as_rat, 1, 1, 1));
        mat.into()
    }

    pub fn init_pipeline(
        vertex_shader: &ShaderModule,
        fragment_shader: &ShaderModule,
        pipeline_layout: &PipelineLayout,
        device: &Device,
        sc_desc: &SwapChainDescriptor,
    ) -> RenderPipeline
    {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: vertex_shader,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: fragment_shader,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                depth_bias: 1,
                depth_bias_slope_scale: 1.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint32,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: std::mem::size_of::<WGPUVertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float2,
                            offset: 3 * 4,
                            shader_location: 1,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float3,
                            offset: 2 * 4 + 3 * 4,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        })
    }

    fn camera_buffer(&self, device: &Device, sc_desc: &SwapChainDescriptor) -> Buffer {
        let as_rat = sc_desc.height as f64 / sc_desc.width as f64;
        let camera_info = CameraInfo {
            camera_matrix: (&self.camera.matrix).into(),
            camera_projection: self.camera_projection(as_rat).into(),
        };
        device.create_buffer_with_data(
            bytemuck::cast_slice(&[camera_info]),
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        )
    }

    fn light_buffer(&self) -> Buffer {
        let light_info = LightInfo {
            light_position: (&self.light.position).into(),
            light_strength: self.light.strength as f32,
            light_type: self.light.light_type.type_id(),
        };
        self.device.create_buffer_with_data(
            bytemuck::cast_slice(&[light_info]),
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        )
    }

    fn scene_status_buffer(&self) -> Buffer {
        self.device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.elapsed() as f32]),
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        )
    }

    pub fn create_bind_group(&mut self, sc_desc: &SwapChainDescriptor) {
        let camera_buffer = self.camera_buffer(&self.device, sc_desc);
        let light_buffer = self.light_buffer();
        let scene_status_buffer = self.scene_status_buffer();
        for object in &mut self.objects {
            object.create_bind_group(
                &camera_buffer,
                &light_buffer,
                &scene_status_buffer,
                &self.bind_group_layout,
                &self.device,
            );
        }
    }

    pub fn prepare_render(&mut self, sc_desc: &SwapChainDescriptor) {
        self.update_depth_texture(sc_desc);
        self.create_bind_group(sc_desc);
    }

    pub fn render_scene<'b>(&'b self, rpass: &mut RenderPass<'b>) {
        rpass.set_pipeline(&self.pipeline);
        for object in &self.objects {
            if object.bind_group.is_none() {
                continue;
            }
            rpass.set_bind_group(0, object.bind_group.as_ref().unwrap(), &[]);
            rpass.set_index_buffer(&object.index_buffer, 0, 0);
            rpass.set_vertex_buffer(0, &object.vertex_buffer, 0, 0);
            rpass.draw_indexed(0..object.index_size as u32, 0, 0..1);
        }
    }

    pub fn depth_stencil_attachment_descriptor(
        &self,
    ) -> RenderPassDepthStencilAttachmentDescriptor {
        RenderPassDepthStencilAttachmentDescriptor {
            attachment: &self.foward_depth,
            depth_load_op: wgpu::LoadOp::Clear,
            depth_store_op: wgpu::StoreOp::Store,
            stencil_load_op: wgpu::LoadOp::Clear,
            stencil_store_op: wgpu::StoreOp::Store,
            clear_depth: 1.0,
            clear_stencil: 0,
        }
    }
}

fn read_spirv(code: &str, shadertype: ShaderType, device: &Device) -> ShaderModule {
    let spirv = glsl_to_spirv::compile(code, shadertype).unwrap();
    device.create_shader_module(&wgpu::read_spirv(spirv).unwrap())
}
