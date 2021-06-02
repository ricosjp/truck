use crate::*;
use truck_meshalgo::tessellation::*;
use truck_topology::*;

impl Default for ShapeInstanceDescriptor {
    #[inline(always)]
    fn default() -> Self {
        ShapeInstanceDescriptor {
            instance_state: Default::default(),
            mesh_precision: 0.005,
        }
    }
}

impl<Shape: MeshableShape> TryIntoInstance<PolygonInstance> for Shape {
    type Descriptor = ShapeInstanceDescriptor;
    /// Creates `PolygonInstance` from shapes.
    /// # Panics
    /// Panic occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    fn try_into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> Option<PolygonInstance> {
        let polygon = self.triangulation(desc.mesh_precision)?;
        Some(polygon.into_instance(
            handler,
            shaders,
            &PolygonInstanceDescriptor {
                instance_state: desc.instance_state.clone(),
            },
        ))
    }
}

impl<P, C, S> IntoInstance<PolygonInstance> for Shell<P, C, S>
where Shell<P, C, S>: MeshableShape
{
    type Descriptor = ShapeInstanceDescriptor;
    /// Creates `PolygonInstance` from `Shell`.
    /// # Panics
    /// Panic occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    #[inline(always)]
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> PolygonInstance {
        self.try_into_instance(handler, shaders, desc)
            .expect("failed to create instance")
    }
}

impl<P, C, S> IntoInstance<PolygonInstance> for Solid<P, C, S>
where Solid<P, C, S>: MeshableShape
{
    type Descriptor = ShapeInstanceDescriptor;
    /// Creates `PolygonInstance` from `Solid`.
    /// # Panics
    /// Panic occurs when the polylined boundary cannot be
    /// converted to the polyline in the surface parameter space.
    /// This may be due to the following reasons.
    /// - A boundary curve is not contained within the surface.
    /// - The surface is not injective, or is too complecated.
    /// - The surface is not regular: non-degenerate and differentiable.
    #[inline(always)]
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        desc: &ShapeInstanceDescriptor,
    ) -> PolygonInstance {
        self.try_into_instance(handler, shaders, desc)
            .expect("failed to create instance")
    }
}

impl<C, S> IntoInstance<WireFrameInstance> for Shell<Point3, C, S>
where C: PolylineableCurve
{
    type Descriptor = ShapeWireFrameDescriptor;
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        desc: &ShapeWireFrameDescriptor,
    ) -> WireFrameInstance {
        let mut lengths = Vec::new();
        let points: Vec<[f32; 3]> = self
            .face_iter()
            .flat_map(|face| face.boundary_iters())
            .flatten()
            .flat_map(|edge| {
                let curve = edge.oriented_curve();
                let division =
                    curve.parameter_division(curve.parameter_range(), desc.polyline_precision);
                lengths.push(division.len() as u32);
                division
                    .into_iter()
                    .map(move |t| curve.subs(t).cast().unwrap().into())
            })
            .collect();
        let mut strips = Vec::<u32>::new();
        let mut counter = 0_u32;
        for len in lengths {
            for i in 1..len {
                strips.push(counter + i - 1);
                strips.push(counter + i);
            }
            counter += len;
        }
        let vertices = BufferHandler::from_slice(&points, handler.device(), BufferUsage::VERTEX);
        let strips = BufferHandler::from_slice(&strips, handler.device(), BufferUsage::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vertices),
            strips: Arc::new(strips),
            state: desc.wireframe_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl<C, S> IntoInstance<WireFrameInstance> for Solid<Point3, C, S>
where C: PolylineableCurve
{
    type Descriptor = ShapeWireFrameDescriptor;
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        desc: &ShapeWireFrameDescriptor,
    ) -> WireFrameInstance {
        let mut lengths = Vec::new();
        let points: Vec<[f32; 3]> = self
            .boundaries()
            .iter()
            .flatten()
            .flat_map(|face| face.boundary_iters())
            .flatten()
            .flat_map(|edge| {
                let curve = edge.oriented_curve();
                let division =
                    curve.parameter_division(curve.parameter_range(), desc.polyline_precision);
                lengths.push(division.len() as u32);
                division
                    .into_iter()
                    .map(move |t| curve.subs(t).cast().unwrap().into())
            })
            .collect();
        let mut strips = Vec::<u32>::new();
        let mut counter = 0_u32;
        for len in lengths {
            for i in 1..len {
                strips.push(counter + i - 1);
                strips.push(counter + i);
            }
            counter += len;
        }
        let vertices = BufferHandler::from_slice(&points, handler.device(), BufferUsage::VERTEX);
        let strips = BufferHandler::from_slice(&strips, handler.device(), BufferUsage::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vertices),
            strips: Arc::new(strips),
            state: desc.wireframe_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}
