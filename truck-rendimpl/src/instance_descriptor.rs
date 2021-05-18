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
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
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
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
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
    pub fn textureview_and_sampler(&self, device: &Device) -> (TextureView, Sampler) {
        let texture = self.texture.as_ref().unwrap();
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        (view, sampler)
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn textureview_bgl_entry() -> PreBindGroupLayoutEntry {
        PreBindGroupLayoutEntry {
            visibility: ShaderStage::FRAGMENT,
            ty: BindingType::Texture {
                view_dimension: TextureViewDimension::D2,
                sample_type: TextureSampleType::Float { filterable: true },
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
            ty: BindingType::Sampler {
                filtering: true,
                comparison: false,
            },
            count: None,
        }
    }
}
