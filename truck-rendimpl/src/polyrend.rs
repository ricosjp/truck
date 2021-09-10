use crate::*;
use polymesh::Vertex;
use std::collections::HashMap;
use truck_base::maputil::GetOrInsert;

impl<V: Sized + Zeroable + Pod> ExpandedPolygon<V> {
    pub fn buffers(
        &self,
        vertex_usage: BufferUsages,
        index_usage: BufferUsages,
        device: &Device,
    ) -> (BufferHandler, BufferHandler) {
        let vertex_buffer = BufferHandler::from_slice(&self.vertices, device, vertex_usage);
        let index_buffer = BufferHandler::from_slice(&self.indices, device, index_usage);
        (vertex_buffer, index_buffer)
    }
}

impl<V> Default for ExpandedPolygon<V> {
    fn default() -> ExpandedPolygon<V> {
        ExpandedPolygon {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl CreateBuffers for PolygonMesh {
    #[inline(always)]
    fn buffers(
        &self,
        vertex_usage: BufferUsages,
        index_usage: BufferUsages,
        device: &Device,
    ) -> (BufferHandler, BufferHandler) {
        ExpandedPolygon::from(self).buffers(vertex_usage, index_usage, device)
    }
}

impl Instance for PolygonInstance {
    type Shaders = PolygonShaders;
    fn standard_shaders(creator: &InstanceCreator) -> PolygonShaders {
        creator.polygon_shaders.clone()
    }
}

impl IntoInstance<PolygonInstance> for PolygonMesh {
    type Descriptor = PolygonInstanceDescriptor;
    #[inline(always)]
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        desc: &PolygonInstanceDescriptor,
    ) -> PolygonInstance {
        let (vb, ib) = self.buffers(BufferUsages::VERTEX, BufferUsages::INDEX, handler.device());
        PolygonInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            state: desc.instance_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl IntoInstance<WireFrameInstance> for PolygonMesh {
    type Descriptor = PolygonWireFrameDescriptor;
    #[doc(hidden)]
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        desc: &PolygonWireFrameDescriptor,
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
            state: desc.wireframe_state.clone(),
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
        ExpandedPolygon::from(self).buffers(vertex_usage, index_usage, device)
    }
}

impl IntoInstance<PolygonInstance> for StructuredMesh {
    type Descriptor = PolygonInstanceDescriptor;
    #[inline(always)]
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &PolygonShaders,
        desc: &PolygonInstanceDescriptor,
    ) -> PolygonInstance {
        let (vb, ib) = self.buffers(BufferUsages::VERTEX, BufferUsages::INDEX, handler.device());
        PolygonInstance {
            polygon: (Arc::new(vb), Arc::new(ib)),
            state: desc.instance_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

impl IntoInstance<WireFrameInstance> for StructuredMesh {
    type Descriptor = PolygonWireFrameDescriptor;
    #[doc(hidden)]
    fn into_instance(
        &self,
        handler: &DeviceHandler,
        shaders: &WireShaders,
        desc: &PolygonWireFrameDescriptor,
    ) -> WireFrameInstance {
        let device = handler.device();
        let positions: Vec<[f32; 3]> = self
            .positions()
            .iter()
            .flat_map(|vec| vec)
            .map(|p| p.cast().unwrap().into())
            .collect();
        let mut strips = Vec::<u32>::new();
        let len = positions[0].len() as u32;
        for i in 1..positions.len() as u32 {
            strips.push((i - 1) * len);
            strips.push(i * len);
        }
        for j in 1..len {
            strips.push(j - 1);
            strips.push(j);
        }
        for i in 1..positions.len() as u32 {
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
            state: desc.wireframe_state.clone(),
            shaders: shaders.clone(),
            id: RenderID::gen(),
        }
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: Vertex,
    glpolymesh: &mut ExpandedPolygon<AttrVertex>,
    vertex_map: &mut HashMap<Vertex, u32>,
) {
    let idx = *vertex_map.get_or_insert(vertex, || {
        let idx = glpolymesh.vertices.len() as u32;
        let position = polymesh.positions()[vertex.pos].cast().unwrap().into();
        let uv_coord = match vertex.uv {
            Some(uv) => polymesh.uv_coords()[uv].cast().unwrap().into(),
            None => [0.0, 0.0],
        };
        let normal = match vertex.nor {
            Some(nor) => polymesh.normals()[nor].cast().unwrap().into(),
            None => [0.0, 0.0, 0.0],
        };
        let wgpuvertex = AttrVertex {
            position,
            uv_coord,
            normal,
        };
        glpolymesh.vertices.push(wgpuvertex);
        idx
    });
    glpolymesh.indices.push(idx);
}

impl From<&PolygonMesh> for ExpandedPolygon<AttrVertex> {
    fn from(polymesh: &PolygonMesh) -> ExpandedPolygon<AttrVertex> {
        let mut glpolymesh = ExpandedPolygon::default();
        let mut vertex_map = HashMap::<Vertex, u32>::new();
        for tri in polymesh.faces().tri_faces() {
            signup_vertex(polymesh, tri[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, tri[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, tri[2], &mut glpolymesh, &mut vertex_map);
        }
        for quad in polymesh.faces().quad_faces() {
            signup_vertex(polymesh, quad[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[3], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[2], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, quad[3], &mut glpolymesh, &mut vertex_map);
        }
        for face in polymesh.faces().other_faces() {
            for i in 2..face.len() {
                signup_vertex(polymesh, face[0], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, face[i - 1], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, face[i], &mut glpolymesh, &mut vertex_map);
            }
        }
        glpolymesh
    }
}

impl From<&StructuredMesh> for ExpandedPolygon<AttrVertex> {
    fn from(mesh: &StructuredMesh) -> ExpandedPolygon<AttrVertex> {
        let mut glpolymesh = ExpandedPolygon::default();
        let (m, n) = (mesh.positions().len(), mesh.positions()[0].len());
        for i in 0..m {
            for j in 0..n {
                glpolymesh.vertices.push(AttrVertex {
                    position: mesh.positions()[i][j].cast().unwrap().into(),
                    uv_coord: match mesh.uv_division() {
                        Some(uv_division) => [uv_division.0[i] as f32, uv_division.1[j] as f32],
                        None => [0.0, 0.0],
                    },
                    normal: match mesh.normals() {
                        Some(normals) => normals[i][j].cast().unwrap().into(),
                        None => [0.0, 0.0, 0.0],
                    },
                });
            }
        }
        for i in 1..m {
            for j in 1..n {
                glpolymesh.indices.push(((i - 1) * n + j - 1) as u32);
                glpolymesh.indices.push((i * n + j - 1) as u32);
                glpolymesh.indices.push(((i - 1) * n + j) as u32);
                glpolymesh.indices.push(((i - 1) * n + j) as u32);
                glpolymesh.indices.push((i * n + j - 1) as u32);
                glpolymesh.indices.push((i * n + j) as u32);
            }
        }
        glpolymesh
    }
}
