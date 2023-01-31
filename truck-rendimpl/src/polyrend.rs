use crate::*;

impl CreateBuffers for PolygonMesh {
    #[inline(always)]
    fn buffers(
        &self,
        vertex_usage: BufferUsages,
        index_usage: BufferUsages,
        device: &Device,
    ) -> (BufferHandler, BufferHandler) {
        let expanded = self.expands(|attr| AttrVertex {
            position: attr.position.cast().unwrap().into(),
            uv_coord: attr
                .uv_coord
                .and_then(|v| Some(v.cast()?.into()))
                .unwrap_or([0.0, 0.0]),
            normal: attr
                .normal
                .and_then(|v| Some(v.cast()?.into()))
                .unwrap_or([0.0, 0.0, 0.0]),
        });
        let indices = expanded
            .faces()
            .triangle_iter()
            .flatten()
            .map(|x| x as u32)
            .collect::<Vec<_>>();
        (
            BufferHandler::from_slice(expanded.attributes(), device, vertex_usage),
            BufferHandler::from_slice(&indices, device, index_usage),
        )
    }
}

impl Instance for PolygonInstance {
    type Shaders = PolygonShaders;
    fn standard_shaders(creator: &InstanceCreator) -> PolygonShaders {
        creator.polygon_shaders.clone()
    }
}

impl ToInstance<PolygonInstance> for PolygonMesh {
    type State = PolygonState;
    #[inline(always)]
    fn to_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        state: &PolygonState,
    ) -> PolygonInstance {
        let (vb, ib) = self.buffers(BufferUsages::VERTEX, BufferUsages::INDEX, handler.device());
        PolygonInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            state: state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl ToInstance<WireFrameInstance> for PolygonMesh {
    type State = WireFrameState;
    fn to_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        state: &WireFrameState,
    ) -> WireFrameInstance {
        let device = handler.device();
        let positions: Vec<[f32; 3]> = self
            .positions()
            .iter()
            .map(|p| p.cast().unwrap().into())
            .collect();
        let mut strips = Vec::<u32>::new();
        self.faces().face_iter().for_each(|face| {
            for i in 0..face.len() {
                strips.push(face[i].pos as u32);
                strips.push(face[(i + 1) % face.len()].pos as u32);
            }
        });
        let vb = BufferHandler::from_slice(&positions, device, BufferUsages::VERTEX);
        let ib = BufferHandler::from_slice(&strips, device, BufferUsages::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vb),
            strips: Arc::new(ib),
            state: state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl CreateBuffers for StructuredMesh {
    #[inline(always)]
    fn buffers(
        &self,
        vertex_usage: BufferUsages,
        index_usage: BufferUsages,
        device: &Device,
    ) -> (BufferHandler, BufferHandler) {
        let mut vertices = Vec::new();
        let (m, n) = (self.positions().len(), self.positions()[0].len());
        for i in 0..m {
            for j in 0..n {
                vertices.push(AttrVertex {
                    position: self.positions()[i][j].cast().unwrap().into(),
                    uv_coord: match self.uv_division() {
                        Some(uv_division) => [uv_division.0[i] as f32, uv_division.1[j] as f32],
                        None => [0.0, 0.0],
                    },
                    normal: match self.normals() {
                        Some(normals) => normals[i][j].cast().unwrap().into(),
                        None => [0.0, 0.0, 0.0],
                    },
                });
            }
        }
        let mut indices = Vec::<u32>::new();
        for i in 1..m {
            for j in 1..n {
                indices.extend([
                    ((i - 1) * n + j - 1) as u32,
                    (i * n + j - 1) as u32,
                    ((i - 1) * n + j) as u32,
                    ((i - 1) * n + j) as u32,
                    (i * n + j - 1) as u32,
                    (i * n + j) as u32,
                ]);
            }
        }
        (
            BufferHandler::from_slice(&vertices, device, vertex_usage),
            BufferHandler::from_slice(&indices, device, index_usage),
        )
    }
}

impl ToInstance<PolygonInstance> for StructuredMesh {
    type State = PolygonState;
    #[inline(always)]
    fn to_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        state: &PolygonState,
    ) -> PolygonInstance {
        let (vb, ib) = self.buffers(BufferUsages::VERTEX, BufferUsages::INDEX, handler.device());
        PolygonInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            state: state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl ToInstance<WireFrameInstance> for StructuredMesh {
    type State = WireFrameState;
    fn to_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        state: &WireFrameState,
    ) -> WireFrameInstance {
        let device = handler.device();
        let positions: Vec<[f32; 3]> = self
            .positions()
            .iter()
            .flatten()
            .map(|p| p.cast().unwrap().into())
            .collect();
        let mut strips = Vec::<u32>::new();
        let len = self.positions()[0].len() as u32;
        for i in 1..positions.len() as u32 {
            strips.push((i - 1) * len);
            strips.push(i * len);
        }
        for j in 1..len {
            strips.push(j - 1);
            strips.push(j);
        }
        for i in 1..self.positions().len() as u32 {
            for j in 1..len {
                strips.push((i - 1) * len + j);
                strips.push(i * len + j);
                strips.push(i * len + (j - 1));
                strips.push(i * len + j);
            }
        }
        let vb = BufferHandler::from_slice(&positions, device, BufferUsages::VERTEX);
        let ib = BufferHandler::from_slice(&strips, device, BufferUsages::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vb),
            strips: Arc::new(ib),
            state: state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl ToInstance<WireFrameInstance> for PolylineCurve<Point3> {
    type State = WireFrameState;
    fn to_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        state: &WireFrameState,
    ) -> WireFrameInstance {
        let device = handler.device();
        let positions: Vec<[f32; 3]> = self.iter().map(|p| p.cast().unwrap().into()).collect();
        let strips: Vec<u32> = (1..positions.len())
            .flat_map(|i| vec![i as u32 - 1, i as u32])
            .collect();
        let vb = BufferHandler::from_slice(&positions, device, BufferUsages::VERTEX);
        let ib = BufferHandler::from_slice(&strips, device, BufferUsages::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vb),
            strips: Arc::new(ib),
            state: state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl ToInstance<WireFrameInstance> for Vec<PolylineCurve<Point3>> {
    type State = WireFrameState;
    fn to_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        state: &WireFrameState,
    ) -> WireFrameInstance {
        let device = handler.device();
        let positions: Vec<[f32; 3]> = self
            .iter()
            .flat_map(|poly| poly.iter())
            .map(|p| p.cast().unwrap().into())
            .collect();
        let mut counter = 0;
        let strips: Vec<u32> = self
            .iter()
            .flat_map(|poly| {
                let len = counter as u32;
                counter += poly.len();
                (1..poly.len()).flat_map(move |i| vec![len + i as u32 - 1, len + i as u32])
            })
            .collect();
        let vb = BufferHandler::from_slice(&positions, device, BufferUsages::VERTEX);
        let ib = BufferHandler::from_slice(&strips, device, BufferUsages::INDEX);
        WireFrameInstance {
            vertices: Arc::new(vb),
            strips: Arc::new(ib),
            state: state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}
