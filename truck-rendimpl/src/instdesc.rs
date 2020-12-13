use crate::*;

impl Default for Material {
    #[inline(always)]
    fn default() -> Material {
        Material {
            albedo: Vector4::new(1.0, 1.0, 1.0, 1.0),
            roughness: 0.5,
            reflectance: 0.25,
            ambient_ratio: 0.02,
        }
    }
}

impl Material {
    #[inline(always)]
    pub fn buffer(&self, device: &Device) -> BufferHandler {
        let material_data: [f32; 7] = [
            self.albedo[0] as f32,
            self.albedo[1] as f32,
            self.albedo[2] as f32,
            self.albedo[3] as f32,
            self.roughness as f32,
            self.reflectance as f32,
            self.ambient_ratio as f32,
        ];
        BufferHandler::from_slice(&material_data, device, BufferUsage::UNIFORM)
    }

    #[inline(always)]
    pub fn bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}

impl Default for InstanceDescriptor {
    #[inline(always)]
    fn default() -> InstanceDescriptor {
        InstanceDescriptor {
            matrix: Matrix4::identity(),
            material: Default::default(),
            texture: None,
            backface_culling: true,
        }
    }
}

impl InstanceDescriptor {
    #[inline(always)]
    pub fn matrix_buffer(&self, device: &Device) -> BufferHandler {
        let matrix_data: [[f32; 4]; 4] = self.matrix.cast::<f32>().unwrap().into();
        BufferHandler::from_slice(&matrix_data, device, BufferUsage::UNIFORM)
    }

    #[inline(always)]
    pub fn matrix_bgl_entry() -> PreBindGroupLayoutEntry {
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
    pub fn material_buffer(&self, device: &Device) -> BufferHandler { self.material.buffer(device) }

    #[inline(always)]
    pub fn material_bgl_entry() -> PreBindGroupLayoutEntry { Material::bgl_entry() }

    pub fn textureview_and_sampler(
        &self,
        device: &Device,
        queue: &Queue,
    ) -> (TextureView, Sampler)
    {
        let texture_image = self.texture.as_ref().unwrap();
        let rgba = texture_image.to_rgba8();
        let dim = texture_image.dimensions();
        let size = Extent3d {
            width: dim.0,
            height: dim.1,
            depth: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            label: None,
        });
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: &rgba,
            usage: BufferUsage::COPY_SRC,
            label: None,
        });
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_texture(
            BufferCopyView {
                buffer: &buffer,
                layout: TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 4 * dim.0,
                    rows_per_image: dim.1,
                },
            },
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            size,
        );
        queue.submit(vec![encoder.finish()]);

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            label: None,
        });
        (view, sampler)
    }

    #[inline(always)]
    pub fn textureview_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::SampledTexture {
                dimension: TextureViewDimension::D2,
                component_type: TextureComponentType::Uint,
                multisampled: false,
            },
            count: None,
        }
    }

    #[inline(always)]
    pub fn sampler_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::Sampler { comparison: false },
            count: None,
        }
    }
    #[inline(always)]
    pub(super) fn pipeline_with_shader(
        &self,
        vertex_shader: ShaderModuleSource,
        fragment_shader: ShaderModuleSource,
        device_handler: &DeviceHandler,
        layout: &PipelineLayout,
    ) -> Arc<RenderPipeline>
    {
        let device = device_handler.device();
        let sc_desc = device_handler.sc_desc();
        let cull_mode = match self.backface_culling {
            true => CullMode::Back,
            false => CullMode::None,
        };
        let vertex_module = device.create_shader_module(vertex_shader);
        let fragment_module = device.create_shader_module(fragment_shader);
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: Some(layout),
            vertex_stage: ProgrammableStageDescriptor {
                module: &vertex_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fragment_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: StencilStateDescriptor {
                    front: StencilStateFaceDescriptor::IGNORE,
                    back: StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint32,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: std::mem::size_of::<AttrVertex>() as BufferAddress,
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
            sample_count: 2,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            label: None,
        });
        Arc::new(pipeline)
    }
}
