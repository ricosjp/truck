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
    /// Creates a `UNIFORM` buffer of material.
    ///
    /// The bind group provided by the instances holds this uniform buffer.
    /// # Shader Examples
    /// ```glsl
    /// layout(set = 1, binding = 1) uniform Material {
    ///     vec4 albedo;
    ///     float roughness;
    ///     float reflectance;
    ///     float ambient_ratio;
    /// };
    /// ```
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

    #[doc(hidden)]
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

impl Default for InstanceState {
    #[inline(always)]
    fn default() -> InstanceState {
        InstanceState {
            matrix: Matrix4::identity(),
            material: Default::default(),
            texture: None,
            backface_culling: true,
        }
    }
}

impl InstanceState {
    /// Creates a `UNIFORM` buffer of instance matrix.
    ///
    /// The bind group provided by the instances holds this uniform buffer.
    /// # Shader Examples
    /// ```glsl
    /// layout(set = 1, binding = 0) uniform ModelMatrix {
    ///     mat4 uniform_matrix;
    /// };
    /// ```
    #[inline(always)]
    pub fn matrix_buffer(&self, device: &Device) -> BufferHandler {
        let matrix_data: [[f32; 4]; 4] = self.matrix.cast::<f32>().unwrap().into();
        BufferHandler::from_slice(&matrix_data, device, BufferUsage::UNIFORM)
    }

    #[doc(hidden)]
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

    /// Creates a `UNIFORM` buffer of material.
    ///
    /// The bind group provided by the instances holds this uniform buffer.
    /// # Shader Examples
    /// ```glsl
    /// layout(set = 1, binding = 1) uniform Material {
    ///     vec4 albedo;
    ///     float roughness;
    ///     float reflectance;
    ///     float ambient_ratio;
    /// };
    /// ```
    #[inline(always)]
    pub fn material_buffer(&self, device: &Device) -> BufferHandler { self.material.buffer(device) }

    #[doc(hidden)]
    #[inline(always)]
    pub fn material_bgl_entry() -> PreBindGroupLayoutEntry { Material::bgl_entry() }

    /// Creates texture view and sampler of the instance's texture image.
    ///
    /// The bind group provided by the instances holds this uniform buffer.
    /// # Shader Examples
    /// ```glsl
    /// layout(set = 1, binding = 2) uniform texture2D texture_view;
    /// layout(set = 1, binding = 3) uniform sampler texture_sampler;
    /// ```
    pub fn textureview_and_sampler(
        &self,
        device: &Device,
    ) -> (TextureView, Sampler) {
        let texture = self.texture.as_ref().unwrap();
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

    #[doc(hidden)]
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

    #[doc(hidden)]
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
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
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
                    stride: (3 + 2 + 3) * 4,
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
            sample_count,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
            label: None,
        });
        Arc::new(pipeline)
    }
}
