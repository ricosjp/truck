use crate::*;

impl PolygonShaders {
    #[inline(always)]
    fn new(device: &Device) -> Self {
        Self {
            vertex: device.create_shader_module(PolygonInstance::default_vertex_shader()),
            fragment: device.create_shader_module(PolygonInstance::default_fragment_shader()),
            tex_fragment: device
                .create_shader_module(PolygonInstance::default_textured_fragment_shader()),
        }
    }
}

impl ShapeShaders {
    #[inline(always)]
    fn new(device: &Device) -> Self {
        Self {
            vertex: device.create_shader_module(ShapeInstance::default_vertex_shader()),
            fragment: device.create_shader_module(ShapeInstance::default_fragment_shader()),
            tex_fragment: device
                .create_shader_module(ShapeInstance::default_textured_fragment_shader()),
        }
    }
}

impl WireShaders {
    #[inline(always)]
    fn new(device: &Device) -> Self {
        Self {
            vertex: device.create_shader_module(include_spirv!("shaders/line.vert.spv")),
            fragment: device.create_shader_module(include_spirv!("shaders/line.frag.spv")),
        }
    }
}

impl CreatorCreator for Scene {
    #[inline(always)]
    fn instance_creator(&self) -> InstanceCreator {
        let device = self.device();
        InstanceCreator {
            handler: self.device_handler().clone(),
            polygon_shaders: Arc::new(PolygonShaders::new(device)),
            shape_shaders: Arc::new(ShapeShaders::new(device)),
            wire_shaders: Arc::new(WireShaders::new(device)),
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
