use crate::*;

impl PolygonShaders {
    /// Constructor
    /// # Parameters
    /// - `vertex_module`: vertex shader module
    /// - `vertex_entry`: entry point of vertex shader module
    /// - `fragment_module`: fragment shader module without texture
    /// - `fragment_entry`: entry point of fragment shader module without texture
    /// - `tex_fragment_module`: fragment shader module with texture
    /// - `tex_fragment_entry`: entry point of fragment shader module with texture
    #[inline(always)]
    pub fn new(
        vertex_module: Arc<ShaderModule>,
        vertex_entry: &'static str,
        fragment_module: Arc<ShaderModule>,
        fragment_entry: &'static str,
        tex_fragment_module: Arc<ShaderModule>,
        tex_fragment_entry: &'static str,
    ) -> Self {
        Self {
            vertex_module,
            vertex_entry,
            fragment_module,
            fragment_entry,
            tex_fragment_module,
            tex_fragment_entry,
        }
    }

    /// Creates default polygon shaders.
    #[inline(always)]
    pub fn default(device: &Device) -> Self {
        let source = include_str!("shaders/microfacet-module.wgsl").to_string() + include_str!("shaders/polygon.wgsl");
        let shader_module = Arc::new(device.create_shader_module(&ShaderModuleDescriptor {
            source: ShaderSource::Wgsl(source.into()),
            flags: ShaderFlags::VALIDATION,
            label: None,
        }));
        Self::new(
            Arc::clone(&shader_module),
            "vs_main",
            Arc::clone(&shader_module),
            "nontex_main",
            Arc::clone(&shader_module),
            "tex_main",
        )
    }
}

impl WireShaders {
    /// Constructor
    /// # Parameters
    /// - `vertex_module`: vertex shader module
    /// - `vertex_entry`: entry point of vertex shader module
    /// - `fragment_module`: fragment shader module without texture
    /// - `fragment_entry`: entry point of fragment shader module without texture
    #[inline(always)]
    pub fn new(
        vertex_module: Arc<ShaderModule>,
        vertex_entry: &'static str,
        fragment_module: Arc<ShaderModule>,
        fragment_entry: &'static str,
    ) -> Self {
        Self {
            vertex_module,
            vertex_entry,
            fragment_module,
            fragment_entry,
        }
    }

    /// Creates default wireframe shaders
    #[inline(always)]
    fn default(device: &Device) -> Self {
        let shader_module = Arc::new(device.create_shader_module(&ShaderModuleDescriptor {
            source: ShaderSource::Wgsl(include_str!("shaders/line.wgsl").into()),
            flags: ShaderFlags::VALIDATION,
            label: None,
        }));
        Self::new(
            Arc::clone(&shader_module),
            "vs_main",
            shader_module,
            "fs_main",
        )
    }
}

impl CreatorCreator for Scene {
    #[inline(always)]
    fn instance_creator(&self) -> InstanceCreator {
        let device = self.device();
        InstanceCreator {
            handler: self.device_handler().clone(),
            polygon_shaders: PolygonShaders::default(device),
            wire_shaders: WireShaders::default(device),
        }
    }
}

impl InstanceCreator {
    /// Creates Instance from object.
    #[inline(always)]
    pub fn try_create_instance<I, T>(&self, object: &T, desc: &T::Descriptor) -> Option<I>
    where
        T: TryIntoInstance<I>,
        I: Instance, {
        object.try_into_instance(&self.handler, &I::standard_shaders(self), desc)
    }
    /// Creates Instance from object.
    #[inline(always)]
    pub fn create_instance<I, T>(&self, object: &T, desc: &T::Descriptor) -> I
    where
        T: IntoInstance<I>,
        I: Instance, {
        object.into_instance(&self.handler, &I::standard_shaders(self), desc)
    }
    /// Creates `Texture` for attaching faces.
    #[inline(always)]
    pub fn create_texture(&self, image: &DynamicImage) -> Arc<Texture> {
        Arc::new(image2texture::image2texture(&self.handler, image))
    }
}
