use crate::*;

impl PolygonShaders {
    #[inline(always)]
    pub fn new(
        vertex_module: ShaderModule,
        vertex_entry: &'static str,
        fragment_module: ShaderModule,
        fragment_entry: &'static str,
        tex_fragment_module: ShaderModule,
        tex_fragment_entry: &'static str,
    ) -> Self {
        Self {
            vertex_module: Arc::new(vertex_module),
            fragment_module: Arc::new(fragment_module),
            tex_fragment_module: Arc::new(tex_fragment_module),
            vertex_entry,
            fragment_entry,
            tex_fragment_entry,
        }
    }

    #[inline(always)]
    pub fn default(device: &Device) -> Self {
        Self::new(
            device.create_shader_module(&PolygonInstance::default_vertex_shader()),
            "main",
            device.create_shader_module(&PolygonInstance::default_fragment_shader()),
            "main",
            device.create_shader_module(&PolygonInstance::default_textured_fragment_shader()),
            "main",
        )
    }
}

impl ShapeShaders {
    #[inline(always)]
    pub fn new(
        vertex_module: ShaderModule,
        vertex_entry: &'static str,
        fragment_module: ShaderModule,
        fragment_entry: &'static str,
        tex_fragment_module: ShaderModule,
        tex_fragment_entry: &'static str,
    ) -> Self {
        Self {
            vertex_module: Arc::new(vertex_module),
            fragment_module: Arc::new(fragment_module),
            tex_fragment_module: Arc::new(tex_fragment_module),
            vertex_entry,
            fragment_entry,
            tex_fragment_entry,
        }
    }

    #[inline(always)]
    fn default(device: &Device) -> Self {
        Self::new(
            device.create_shader_module(&ShapeInstance::default_vertex_shader()),
            "main",
            device.create_shader_module(&ShapeInstance::default_fragment_shader()),
            "main",
            device.create_shader_module(&ShapeInstance::default_textured_fragment_shader()),
            "main",
        ) 
    }
}

impl WireShaders {
    #[inline(always)]
    pub fn new(
        vertex_module: ShaderModule,
        vertex_entry: &'static str,
        fragment_module: ShaderModule,
        fragment_entry: &'static str,
    ) -> Self {
        Self {
            vertex_module: Arc::new(vertex_module),
            fragment_module: Arc::new(fragment_module),
            vertex_entry,
            fragment_entry,
        }
    }

    #[inline(always)]
    fn default(device: &Device) -> Self {
        Self::new(
            device.create_shader_module(&include_spirv!("shaders/line.vert.spv")),
            "main",
            device.create_shader_module(&include_spirv!("shaders/line.frag.spv")),
            "main",
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
            shape_shaders: ShapeShaders::default(device),
            wire_shaders: WireShaders::default(device),
        }
    }
}

impl InstanceCreator {
    /// Creates Instance from object.
    #[inline(always)]
    pub fn try_create_instance<Instance, T: TryIntoInstance<Instance>>(
        &self,
        object: &T,
        desc: &T::Descriptor,
    ) -> Option<Instance> {
        object.try_into_instance(self, desc)
    }
    /// Creates Instance from object.
    #[inline(always)]
    pub fn create_instance<Instance, T: IntoInstance<Instance>>(
        &self,
        object: &T,
        desc: &T::Descriptor,
    ) -> Instance {
        object.into_instance(self, desc)
    }
    /// Creates `Texture` for attaching faces.
    #[inline(always)]
    pub fn create_texture(&self, image: &DynamicImage) -> Arc<Texture> {
        Arc::new(image2texture::image2texture(&self.handler, image))
    }
}
