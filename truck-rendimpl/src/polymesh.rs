use crate::*;
use std::collections::HashMap;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct AttrVertex {
    pub position: [f32; 3],
    pub uv_coord: [f32; 2],
    pub normal: [f32; 3],
}
unsafe impl Zeroable for AttrVertex {}
unsafe impl Pod for AttrVertex {}

#[repr(C)]
#[derive(Debug, Clone)]
struct ExpandedPolygon {
    vertices: Vec<AttrVertex>,
    indices: Vec<u32>,
}

impl ExpandedPolygon {
    fn buffers(&self, device: &Device) -> (BufferHandler, BufferHandler) {
        let vertex_buffer = BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&self.vertices),
                usage: BufferUsage::VERTEX,
                label: None,
            }),
            (self.vertices.len() * std::mem::size_of::<AttrVertex>()) as u64,
        );
        let index_buffer = BufferHandler::new(
            device.create_buffer_init(&BufferInitDescriptor {
                contents: bytemuck::cast_slice(&self.indices),
                usage: BufferUsage::INDEX,
                label: None,
            }),
            self.indices.len() as u64,
        );
        (vertex_buffer, index_buffer)
    }
}

impl Default for ExpandedPolygon {
    #[inline(always)]
    fn default() -> ExpandedPolygon {
        ExpandedPolygon {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

fn signup_vertex(
    polymesh: &PolygonMesh,
    vertex: &[usize; 3],
    glpolymesh: &mut ExpandedPolygon,
    vertex_map: &mut HashMap<[usize; 3], u32>,
)
{
    let key = [vertex[0], vertex[1], vertex[2]];
    let idx = match vertex_map.get(&key) {
        Some(idx) => *idx,
        None => {
            let idx = glpolymesh.vertices.len() as u32;
            let position = (&polymesh.positions[key[0]]).cast().unwrap().into();
            let uv_coord = match polymesh.uv_coords.is_empty() {
                true => [0.0, 0.0],
                false => (&polymesh.uv_coords[key[1]]).cast().unwrap().into(),
            };
            let normal = match polymesh.normals.is_empty() {
                true => [0.0, 0.0, 0.0],
                false => (&polymesh.normals[key[2]]).cast().unwrap().into(),
            };
            let wgpuvertex = AttrVertex {
                position,
                uv_coord,
                normal,
            };
            vertex_map.insert(key, idx);
            glpolymesh.vertices.push(wgpuvertex);
            idx
        }
    };
    glpolymesh.indices.push(idx);
}

impl From<&PolygonMesh> for ExpandedPolygon {
    fn from(polymesh: &PolygonMesh) -> ExpandedPolygon {
        let mut glpolymesh = ExpandedPolygon::default();
        let mut vertex_map = HashMap::<[usize; 3], u32>::new();
        for tri in &polymesh.tri_faces {
            signup_vertex(polymesh, &tri[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &tri[2], &mut glpolymesh, &mut vertex_map);
        }
        for quad in &polymesh.quad_faces {
            signup_vertex(polymesh, &quad[0], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[1], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[2], &mut glpolymesh, &mut vertex_map);
            signup_vertex(polymesh, &quad[3], &mut glpolymesh, &mut vertex_map);
        }
        for face in &polymesh.other_faces {
            for i in 2..face.len() {
                signup_vertex(polymesh, &face[0], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i - 1], &mut glpolymesh, &mut vertex_map);
                signup_vertex(polymesh, &face[i], &mut glpolymesh, &mut vertex_map);
            }
        }
        glpolymesh
    }
}

impl From<&StructuredMesh> for ExpandedPolygon {
    fn from(mesh: &StructuredMesh) -> ExpandedPolygon {
        let mut glpolymesh = ExpandedPolygon::default();
        let (m, n) = (mesh.uv_division.0.len(), mesh.uv_division.1.len());
        for i in 0..m {
            for j in 0..n {
                glpolymesh.vertices.push(AttrVertex {
                    position: mesh.positions[i][j].cast().unwrap().into(),
                    uv_coord: [mesh.uv_division.0[i] as f32, mesh.uv_division.1[j] as f32],
                    normal: mesh.normals[i][j].cast().unwrap().into(),
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

