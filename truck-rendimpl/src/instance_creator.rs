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
    /// Creates `PolygonInstance` from `PolygonMesh` and `StructuredMesh`.
    #[inline(always)]
    pub fn create_polygon_instance<P: Polygon>(
        &self,
        object: &P,
        desc: &PolygonInstanceDescriptor,
    ) -> PolygonInstance {
        object.into_instance(self, desc)
    }
    /// Tries to create `ShapeInstance` from `Shell` and `Solid`.
    /// # Failure
    /// Failure occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    #[inline(always)]
    pub fn try_create_shape_instance<S: Shape>(
        &self,
        object: &S,
        desc: &ShapeInstanceDescriptor,
    ) -> Option<ShapeInstance> {
        object.try_into_instance(self, desc)
    }
    /// Creates `ShapeInstance` from `Shell` and `Solid`.
    /// # Panics
    /// Panic occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    #[inline(always)]
    pub fn create_shape_instance<S: Shape>(
        &self,
        object: &S,
        desc: &ShapeInstanceDescriptor,
    ) -> ShapeInstance {
        object.into_instance(self, desc)
    }
    /// Creates `WireFrameInstance` from `Shell` and `Solid`.
    #[inline(always)]
    pub fn create_wire_frame_instance<W: IntoWireFrame>(
        &self,
        object: &W,
        desc: &WireFrameInstanceDescriptor,
    ) -> WireFrameInstance {
        object.into_wire_frame(self, desc)
    }
    /// Creates `Texture` for attaching faces.
    #[inline(always)]
    pub fn create_texture(&self, image: &DynamicImage) -> Arc<Texture> {
        Arc::new(image2texture::image2texture(&self.handler, image))
    }
}
